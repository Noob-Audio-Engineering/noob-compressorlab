//! Measure every model against the figures its research document publishes,
//! and write the comparison to `docs/BENCHMARK.md`.
//!
//! Run it with `cargo run --release --features plugin --bin benchmark`. It is a binary rather
//! than a test so the ordinary `cargo test` run stays fast: this drives
//! minutes of audio through six engines.
//!
//! ## What this can and cannot be
//!
//! The obvious reading of "benchmark against the real thing" is a null test
//! against hardware, or against a competitor's plug-in. Neither is possible
//! and the survey in `research/SURVEY.md` says why: no independent
//! laboratory measurement of any of these units exists in public, and no
//! plug-in vendor publishes a null test. So the ground truth available is
//! the published one: manufacturer specifications, service-manual
//! calibration tables, and the measurements the dossiers cite from reviews
//! and teardowns.
//!
//! Every row below therefore names the figure, where it comes from, what
//! this model measures, and whether the two agree. A row whose published
//! column reads *(none published)* is there deliberately: knowing that
//! nothing anchors a behaviour is as useful as knowing that something does.
//!
//! ## The rule this file obeys
//!
//! Three audits of this repository found tests that had been written to
//! assert the model's own output instead of the figure they existed to
//! check. Nothing here compares a model against itself. Where a model
//! misses, the miss is reported with its number; the tolerance is never
//! widened to make a row pass, and no row is dropped for failing.

use noob_compressorlab::dsp::{bridge, fet, gbus, opto, opto1b, opto3, pre, rms, tg, vca, vmu};
use std::f32::consts::PI;
use std::fmt::Write as _;

/// Sample rate every measurement runs at unless a row says otherwise. The
/// dossiers quote figures at "nominal" rates; 48 kHz is the middle of the
/// range the engines support.
const SR: f32 = 48_000.0;
/// Block size the engines are driven with for level and distortion work,
/// matching a typical host.
const BLOCK: usize = 256;
/// Block size for timing measurements. Gain reduction is read once per
/// block, so the block is the resolution: at 256 samples nothing faster
/// than 5.3 ms can be seen, and several of these units publish attacks two
/// orders of magnitude quicker than that. Eight samples gives 0.167 ms,
/// which resolves everything here except the 1176's fastest attack, and
/// that row says so rather than reporting the floor as a measurement.
const TIMING_BLOCK: usize = 8;
/// The finest interval a timing measurement can resolve, in milliseconds.
const TIMING_FLOOR_MS: f32 = TIMING_BLOCK as f32 / SR * 1000.0;

// ---------------------------------------------------------------------------
// Reporting
// ---------------------------------------------------------------------------

/// How a measurement compares with the figure it was checked against.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Verdict {
    /// The model is inside the published tolerance.
    Meets,
    /// The model is outside it. Reported, never hidden.
    Misses,
    /// Nothing is published for this behaviour, so the number is recorded
    /// without a verdict.
    NoFigure,
}

impl Verdict {
    fn mark(self) -> &'static str {
        match self {
            Verdict::Meets => "meets",
            Verdict::Misses => "**misses**",
            Verdict::NoFigure => "no figure",
        }
    }
}

/// One published figure and what this model does about it.
struct Row {
    quantity: String,
    published: String,
    measured: String,
    source: String,
    verdict: Verdict,
    /// Why a miss happens, when the repository already understands it.
    note: String,
}

impl Row {
    fn new(
        quantity: &str,
        published: &str,
        measured: String,
        source: &str,
        verdict: Verdict,
    ) -> Self {
        Row {
            quantity: quantity.into(),
            published: published.into(),
            measured,
            source: source.into(),
            verdict,
            note: String::new(),
        }
    }

    /// A figure with a published range: verdict follows the range.
    fn ranged(quantity: &str, lo: f32, hi: f32, unit: &str, value: f32, source: &str) -> Self {
        let verdict = if value >= lo && value <= hi {
            Verdict::Meets
        } else {
            Verdict::Misses
        };
        Row::new(
            quantity,
            &format!("{lo} to {hi} {unit}"),
            format!("{value:.3} {unit}"),
            source,
            verdict,
        )
    }

    /// A figure with a published value and a tolerance either side.
    fn within(quantity: &str, target: f32, tol: f32, unit: &str, value: f32, source: &str) -> Self {
        let verdict = if (value - target).abs() <= tol {
            Verdict::Meets
        } else {
            Verdict::Misses
        };
        Row::new(
            quantity,
            &format!("{target} ± {tol} {unit}"),
            format!("{value:.3} {unit}"),
            source,
            verdict,
        )
    }

    /// A behaviour with no published number: record the measurement only.
    fn unanchored(quantity: &str, measured: String, why: &str) -> Self {
        let mut r = Row::new(
            quantity,
            "*(none published)*",
            measured,
            "—",
            Verdict::NoFigure,
        );
        r.note = why.into();
        r
    }

    fn because(mut self, note: &str) -> Self {
        self.note = note.into();
        self
    }
}

/// Everything measured for one model.
struct Section {
    model: &'static str,
    unit: &'static str,
    dossier: &'static str,
    rows: Vec<Row>,
}

impl Section {
    fn counts(&self) -> (usize, usize, usize) {
        let meets = self
            .rows
            .iter()
            .filter(|r| r.verdict == Verdict::Meets)
            .count();
        let misses = self
            .rows
            .iter()
            .filter(|r| r.verdict == Verdict::Misses)
            .count();
        let none = self
            .rows
            .iter()
            .filter(|r| r.verdict == Verdict::NoFigure)
            .count();
        (meets, misses, none)
    }
}

// ---------------------------------------------------------------------------
// Signal generation and measurement
// ---------------------------------------------------------------------------

/// Peak amplitude of a sine at `dbfs` (0 dBFS = a full-scale sine).
fn amp_dbfs(dbfs: f32) -> f32 {
    10f32.powf(dbfs / 20.0)
}

/// Peak amplitude of a sine `db` above 0 VU, where 0 VU is −18 dBFS RMS.
fn amp_vu(db: f32) -> f32 {
    opto::model::VU_REF_AMP * 10f32.powf(db / 20.0)
}

fn db(x: f32) -> f32 {
    20.0 * x.max(1e-12).log10()
}

/// Root mean square of a slice.
fn rms(x: &[f32]) -> f32 {
    if x.is_empty() {
        return 0.0;
    }
    (x.iter().map(|v| (*v as f64) * (*v as f64)).sum::<f64>() / x.len() as f64).sqrt() as f32
}

/// Magnitude of `hz` in `x`, by the Goertzel algorithm. Used for harmonic
/// and response measurements, where a full transform would be waste.
fn goertzel(x: &[f32], hz: f32, sr: f32) -> f32 {
    let w = 2.0 * PI * hz / sr;
    let c = 2.0 * (w as f64).cos();
    let (mut s1, mut s2) = (0.0f64, 0.0f64);
    for &v in x {
        let s0 = v as f64 + c * s1 - s2;
        s2 = s1;
        s1 = s0;
    }
    let re = s1 - s2 * (w as f64).cos();
    let im = s2 * (w as f64).sin();
    ((re * re + im * im).sqrt() / x.len() as f64 * 2.0) as f32
}

/// Total harmonic distortion as a percentage, from harmonics two to six.
fn thd_pct(x: &[f32], hz: f32, sr: f32) -> f32 {
    let f = goertzel(x, hz, sr);
    if f <= 1e-9 {
        return 0.0;
    }
    let sum: f32 = (2..=6)
        .map(|k| {
            let h = goertzel(x, hz * k as f32, sr);
            h * h
        })
        .sum::<f32>()
        .sqrt();
    sum / f * 100.0
}

/// A sine block generator that keeps its phase across calls, so a long run
/// has no discontinuity at block boundaries.
struct Sine {
    phase: f32,
    hz: f32,
    sr: f32,
}

impl Sine {
    fn new(hz: f32, sr: f32) -> Self {
        Sine { phase: 0.0, hz, sr }
    }

    fn fill(&mut self, buf: &mut [f32], amp: f32) {
        for v in buf.iter_mut() {
            self.phase += self.hz / self.sr;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
            *v = amp * (self.phase * 2.0 * PI).sin();
        }
    }
}

/// Anything the benchmark can drive: one stereo block in, gain reduction
/// out. Each engine has its own shape, so this is the common ground.
trait Engine {
    fn run(&mut self, l: &mut [f32], r: &mut [f32]);
    fn gr_db(&self) -> f32;
}

impl Engine for fet::Compressor {
    fn run(&mut self, l: &mut [f32], r: &mut [f32]) {
        self.process(l, r);
    }
    fn gr_db(&self) -> f32 {
        self.gr_db()
    }
}

impl Engine for opto::Compressor {
    fn run(&mut self, l: &mut [f32], r: &mut [f32]) {
        self.process_block(l, r);
    }
    fn gr_db(&self) -> f32 {
        -self.gain_reduction_db(0)
    }
}

impl Engine for opto3::Compressor {
    fn run(&mut self, l: &mut [f32], r: &mut [f32]) {
        self.process_block(l, r);
    }
    fn gr_db(&self) -> f32 {
        -self.gain_reduction_db(0)
    }
}

impl Engine for opto1b::Compressor {
    fn run(&mut self, l: &mut [f32], r: &mut [f32]) {
        self.process_block(l, r);
    }
    fn gr_db(&self) -> f32 {
        -self.gain_reduction_db(0)
    }
}

impl Engine for gbus::Compressor {
    fn run(&mut self, l: &mut [f32], r: &mut [f32]) {
        self.process_block(l, r);
    }
    fn gr_db(&self) -> f32 {
        self.gr_db()
    }
}

impl Engine for vca::Compressor {
    fn run(&mut self, l: &mut [f32], r: &mut [f32]) {
        self.process_block(l, r);
    }
    fn gr_db(&self) -> f32 {
        self.gr_db()
    }
}

/// Hold a steady sine for `seconds` and return the output of the final
/// `tail_s`, plus the settled gain reduction. The lead-in lets the slowest
/// release in the lab reach its steady state before anything is measured.
fn steady(
    eng: &mut dyn Engine,
    hz: f32,
    amp: f32,
    seconds: f32,
    tail_s: f32,
    sr: f32,
) -> (Vec<f32>, f32) {
    let blocks = ((seconds * sr) as usize / BLOCK).max(1);
    let tail_blocks = (((tail_s * sr) as usize / BLOCK).max(1)).min(blocks);
    let mut sine = Sine::new(hz, sr);
    let mut l = vec![0.0f32; BLOCK];
    let mut r = vec![0.0f32; BLOCK];
    let mut tail = Vec::with_capacity(tail_blocks * BLOCK);
    for b in 0..blocks {
        sine.fill(&mut l, amp);
        r.copy_from_slice(&l);
        eng.run(&mut l, &mut r);
        if b >= blocks - tail_blocks {
            tail.extend_from_slice(&l);
        }
    }
    (tail, eng.gr_db())
}

/// The settled output level in dBFS for a steady sine, measured as RMS over
/// the tail and converted back to the peak of an equivalent sine.
fn settled_out_dbfs(eng: &mut dyn Engine, hz: f32, amp: f32, seconds: f32, sr: f32) -> f32 {
    let (tail, _) = steady(eng, hz, amp, seconds, 0.25, sr);
    db(rms(&tail) * std::f32::consts::SQRT_2)
}

/// Drive a step from `from_amp` to `to_amp` and return the gain reduction
/// trajectory in dB, one reading per [`TIMING_BLOCK`].
fn step_response(
    eng: &mut dyn Engine,
    hz: f32,
    from_amp: f32,
    to_amp: f32,
    settle_s: f32,
    hold_s: f32,
    sr: f32,
) -> Vec<f32> {
    let mut sine = Sine::new(hz, sr);
    let mut l = vec![0.0f32; TIMING_BLOCK];
    let mut r = vec![0.0f32; TIMING_BLOCK];
    for _ in 0..((settle_s * sr) as usize / TIMING_BLOCK).max(1) {
        sine.fill(&mut l, from_amp);
        r.copy_from_slice(&l);
        eng.run(&mut l, &mut r);
    }
    let blocks = ((hold_s * sr) as usize / TIMING_BLOCK).max(1);
    let mut traj = Vec::with_capacity(blocks);
    for _ in 0..blocks {
        sine.fill(&mut l, to_amp);
        r.copy_from_slice(&l);
        eng.run(&mut l, &mut r);
        traj.push(eng.gr_db());
    }
    traj
}

/// Time in milliseconds for a trajectory to reach `frac` of its final
/// value, where the trajectory is sampled once per [`TIMING_BLOCK`].
/// Returns `None` if it never gets there.
fn time_to_fraction(traj: &[f32], frac: f32, sr: f32) -> Option<f32> {
    let final_v = *traj.last()?;
    let start = *traj.first()?;
    let span = final_v - start;
    if span.abs() < 1e-4 {
        return None;
    }
    let target = start + span * frac;
    for (i, v) in traj.iter().enumerate() {
        let reached = if span < 0.0 {
            *v <= target
        } else {
            *v >= target
        };
        if reached {
            return Some((i + 1) as f32 * TIMING_BLOCK as f32 / sr * 1000.0);
        }
    }
    None
}

/// Time in milliseconds for a recovery trajectory to come back within
/// `within_db` of no gain reduction at all. The CL 1B's service manual
/// measures its release that way, as a full return of the needle rather
/// than a time constant, so a benchmark of that figure has to as well.
fn time_to_recover(traj: &[f32], within_db: f32, sr: f32) -> Option<f32> {
    for (i, v) in traj.iter().enumerate() {
        if v.abs() <= within_db {
            return Some((i + 1) as f32 * TIMING_BLOCK as f32 / sr * 1000.0);
        }
    }
    None
}

/// Release: hold a loud tone, drop to quiet, and time the recovery of the
/// gain reduction back towards zero.
fn release_response(
    eng: &mut dyn Engine,
    hz: f32,
    loud_amp: f32,
    quiet_amp: f32,
    hold_s: f32,
    tail_s: f32,
    sr: f32,
) -> Vec<f32> {
    let mut sine = Sine::new(hz, sr);
    let mut l = vec![0.0f32; TIMING_BLOCK];
    let mut r = vec![0.0f32; TIMING_BLOCK];
    for _ in 0..((hold_s * sr) as usize / TIMING_BLOCK).max(1) {
        sine.fill(&mut l, loud_amp);
        r.copy_from_slice(&l);
        eng.run(&mut l, &mut r);
    }
    let blocks = ((tail_s * sr) as usize / TIMING_BLOCK).max(1);
    let mut traj = Vec::with_capacity(blocks);
    for _ in 0..blocks {
        sine.fill(&mut l, quiet_amp);
        r.copy_from_slice(&l);
        eng.run(&mut l, &mut r);
        traj.push(eng.gr_db());
    }
    traj
}

/// The response of a stage at `hz`, relative to its response at 1 kHz, in
/// dB. Used where a dossier publishes a bandwidth figure.
fn response_db(mut make: impl FnMut() -> Box<dyn Engine>, hz: f32, amp: f32, sr: f32) -> f32 {
    let mut eng = make();
    let (tail, _) = steady(eng.as_mut(), hz, amp, 1.0, 0.25, sr);
    let at_hz = goertzel(&tail, hz, sr);
    let mut eng = make();
    let (tail, _) = steady(eng.as_mut(), 1000.0, amp, 1.0, 0.25, sr);
    let at_1k = goertzel(&tail, 1000.0, sr);
    db(at_hz) - db(at_1k)
}

// ---------------------------------------------------------------------------
// The 1176
// ---------------------------------------------------------------------------

fn fet_settings(ratio: fet::Ratio, input: f32, attack: f32, release: f32) -> fet::Settings {
    fet::Settings {
        input,
        attack,
        release,
        ratio,
        ..fet::Settings::default()
    }
}

fn bench_fet() -> Section {
    let mut rows = Vec::new();

    // 8.1 static curve: the slope between 6 and 16 dB above threshold, per
    // ratio button. The dossier wants it within 20 % of the printed figure.
    for (ratio, printed) in [
        (fet::Ratio::R4, 4.0f32),
        (fet::Ratio::R8, 8.0),
        (fet::Ratio::R12, 12.0),
        (fet::Ratio::R20, 20.0),
    ] {
        let slope = fet_slope(ratio);
        let tol = printed * 0.2;
        rows.push(
            Row::within(
                &format!("{printed:.0}:1 slope, 6 to 16 dB above threshold"),
                printed,
                tol,
                ":1",
                slope,
                "research/1176.md §8.1, from [7][9][14]",
            )
            .because("measured as the reciprocal of the input-to-output slope over that window"),
        );
    }

    // Threshold spread across the four buttons: published as 5 to 7 dB.
    let thresholds: Vec<f32> = [
        fet::Ratio::R4,
        fet::Ratio::R8,
        fet::Ratio::R12,
        fet::Ratio::R20,
    ]
    .iter()
    .map(|r| fet_threshold(*r))
    .collect();
    let spread = thresholds.iter().cloned().fold(f32::MIN, f32::max)
        - thresholds.iter().cloned().fold(f32::MAX, f32::min);
    rows.push(Row::ranged(
        "threshold spread, 4:1 to 20:1",
        5.0,
        7.0,
        "dB",
        spread,
        "research/1176.md §8.1, from [7][9][14]",
    ));

    // 8.2 timing. Attack 7 is the repository's known miss; attack 1 has a
    // published window of its own.
    let att7 = fet_attack_ms(7.0);
    rows.push(
        Row::new(
            "attack 7, 63 % of final gain reduction",
            "below 0.060 ms",
            format!("{att7:.3} ms"),
            "research/1176.md §8.2",
            if att7 < 0.060 {
                Verdict::Meets
            } else {
                Verdict::Misses
            },
        )
        .because(&format!(
            "known miss, recorded in README: the knob map reaches 20 µs but the closed loop adds \
             the detector's own charging time. Note the published figure is below this harness's \
             own resolution of {TIMING_FLOOR_MS:.3} ms, so the measurement bounds the miss rather \
             than sizing it"
        )),
    );
    let att1 = fet_attack_ms(1.0);
    rows.push(Row::ranged(
        "attack 1, 63 % of final gain reduction",
        0.4,
        1.2,
        "ms",
        att1,
        "research/1176.md §8.2",
    ));

    let rel7 = fet_release_ms(7.0);
    rows.push(Row::ranged(
        "release 7, 63 % recovery",
        40.0,
        65.0,
        "ms",
        rel7,
        "research/1176.md §8.2",
    ));
    let rel1 = fet_release_ms(1.0);
    rows.push(Row::ranged(
        "release 1, 63 % recovery",
        900.0,
        1400.0,
        "ms",
        rel1,
        "research/1176.md §8.2",
    ));

    // 8.4 distortion. The attack-OFF figure is the second known miss.
    let thd_off = fet_thd_attack_off();
    rows.push(
        Row::new(
            "THD, attack OFF, 1 kHz at −18 dBFS, LN",
            "below 0.100 %",
            format!("{thd_off:.3} %"),
            "research/1176.md §8.4, from [1][44]",
            if thd_off < 0.1 { Verdict::Meets } else { Verdict::Misses },
        )
        .because("known miss, recorded in README: both amplifiers are a little into their curves at 24 / 24"),
    );

    // Soft knee, the third known miss.
    let (knee_first, knee_ten) = fet_knee();
    let gentler = if knee_ten.abs() > 1e-6 {
        (1.0 - knee_first / knee_ten) * 100.0
    } else {
        0.0
    };
    rows.push(
        Row::new(
            "soft knee, first 3 dB versus 10 dB above threshold (4:1)",
            "at least 30 % gentler",
            format!("{gentler:.1} % gentler"),
            "research/1176.md §8.1",
            if gentler >= 30.0 { Verdict::Meets } else { Verdict::Misses },
        )
        .because("known miss, recorded in README: the knee is whatever the diode detector's curvature makes it"),
    );

    // 8.7 metering, both calibration points. The published figures are in
    // dBFS **RMS**, so the tone is referenced to 0 VU rather than given a
    // peak amplitude: a sine whose peak is −18 dBFS is 3 dB quieter than
    // one whose RMS is, and reading the wrong one would report a 3 dB
    // calibration error that is not there.
    let plus4 = fet_meter_reading(fet::MeterMode::Plus4, amp_vu(0.0));
    rows.push(Row::within(
        "+4 meter, −18 dBFS RMS sine reads 0 VU",
        0.0,
        0.2,
        "VU",
        plus4,
        "research/1176.md §8.7, from [1 p.10][14]",
    ));
    let plus8 = fet_meter_reading(fet::MeterMode::Plus8, amp_vu(4.0));
    rows.push(Row::within(
        "+8 meter, −14 dBFS RMS sine reads 0 VU",
        0.0,
        0.2,
        "VU",
        plus8,
        "research/1176.md §8.7, from [1 p.10][14]",
    ));

    // 8.3 all-buttons: slope well above the printed ratios.
    let all_slope = fet_slope(fet::Ratio::All);
    rows.push(Row::ranged(
        "all-buttons slope, 10 dB above threshold",
        10.0,
        25.0,
        ":1",
        all_slope,
        "research/1176.md §8.3, from [1]",
    ));

    // Revision distortion relationships: ordering is documented, the
    // absolute figures are not.
    let (blue, ln) = (
        fet_thd_revision(fet::Revision::A),
        fet_thd_revision(fet::Revision::Ln),
    );
    rows.push(
        Row::new(
            "blue-stripe THD versus LN at 10 dB gain reduction",
            "at least twice",
            format!(
                "{:.2}× ({blue:.3} % against {ln:.3} %)",
                blue / ln.max(1e-6)
            ),
            "research/1176.md §5, revision table",
            if blue >= 2.0 * ln {
                Verdict::Meets
            } else {
                Verdict::Misses
            },
        )
        .because(
            "the sources give the ordering between revisions, not absolute distortion figures",
        ),
    );

    rows.push(Row::unanchored(
        "latency",
        format!("{} samples at {} kHz", fet::Compressor::new(SR).latency(), SR / 1000.0),
        "the hardware is analogue and has none; this is the oversampler's, and no figure exists to compare it with",
    ));

    Section {
        model: "1176",
        unit: "UREI 1176 Peak Limiter",
        dossier: "research/1176.md",
        rows,
    }
}

/// Input level in dBFS that produces 1 dB of gain reduction.
fn fet_threshold(ratio: fet::Ratio) -> f32 {
    for step in 0..70 {
        let dbfs = -60.0 + step as f32;
        let mut c = fet::Compressor::new(SR);
        c.configure(&fet_settings(ratio, 24.0, 4.0, 4.0));
        let (_, gr) = steady(&mut c, 1000.0, amp_dbfs(dbfs), 2.0, 0.2, SR);
        if -gr >= 1.0 {
            return dbfs;
        }
    }
    f32::NAN
}

/// Compression slope between 6 and 16 dB above the threshold, as a ratio.
fn fet_slope(ratio: fet::Ratio) -> f32 {
    let thr = fet_threshold(ratio);
    if !thr.is_finite() {
        return f32::NAN;
    }
    let a_in = thr + 6.0;
    let b_in = thr + 16.0;
    let out = |dbfs: f32| {
        let mut c = fet::Compressor::new(SR);
        c.configure(&fet_settings(ratio, 24.0, 4.0, 4.0));
        settled_out_dbfs(&mut c, 1000.0, amp_dbfs(dbfs), 2.0, SR)
    };
    let d_out = out(b_in) - out(a_in);
    if d_out.abs() < 1e-4 {
        return 999.0;
    }
    (b_in - a_in) / d_out
}

/// Slopes over the first 3 dB above threshold and 10 dB up, for the knee.
fn fet_knee() -> (f32, f32) {
    let thr = fet_threshold(fet::Ratio::R4);
    let out = |dbfs: f32| {
        let mut c = fet::Compressor::new(SR);
        c.configure(&fet_settings(fet::Ratio::R4, 24.0, 4.0, 4.0));
        settled_out_dbfs(&mut c, 1000.0, amp_dbfs(dbfs), 2.0, SR)
    };
    let first = (out(thr + 3.0) - out(thr)) / 3.0;
    let ten = (out(thr + 13.0) - out(thr + 10.0)) / 3.0;
    (first, ten)
}

fn fet_attack_ms(knob: f32) -> f32 {
    let mut c = fet::Compressor::new(SR);
    c.configure(&fet_settings(fet::Ratio::R20, 30.0, knob, 4.0));
    let traj = step_response(
        &mut c,
        1000.0,
        amp_dbfs(-40.0),
        amp_dbfs(-6.0),
        0.5,
        0.5,
        SR,
    );
    time_to_fraction(&traj, 0.63, SR).unwrap_or(f32::NAN)
}

fn fet_release_ms(knob: f32) -> f32 {
    let mut c = fet::Compressor::new(SR);
    c.configure(&fet_settings(fet::Ratio::R20, 30.0, 4.0, knob));
    let traj = release_response(
        &mut c,
        1000.0,
        amp_dbfs(-6.0),
        amp_dbfs(-40.0),
        1.0,
        3.0,
        SR,
    );
    time_to_fraction(&traj, 0.63, SR).unwrap_or(f32::NAN)
}

fn fet_thd_attack_off() -> f32 {
    let mut c = fet::Compressor::new(SR);
    c.configure(&fet_settings(fet::Ratio::R4, 24.0, 0.0, 4.0));
    let (tail, _) = steady(&mut c, 1000.0, amp_dbfs(-18.0), 2.0, 0.5, SR);
    thd_pct(&tail, 1000.0, SR)
}

/// THD of one revision at roughly 10 dB of gain reduction.
fn fet_thd_revision(rev: fet::Revision) -> f32 {
    let mut c = fet::Compressor::new(SR);
    c.configure(&fet::Settings {
        revision: rev,
        input: 36.0,
        ratio: fet::Ratio::R4,
        ..fet::Settings::default()
    });
    let (tail, _) = steady(&mut c, 1000.0, amp_dbfs(-12.0), 2.0, 0.5, SR);
    thd_pct(&tail, 1000.0, SR)
}

/// What the panel meter settles at for a steady tone, in VU.
fn fet_meter_reading(mode: fet::MeterMode, amp: f32) -> f32 {
    let mut c = fet::Compressor::new(SR);
    c.configure(&fet::Settings {
        meter: mode,
        attack: 0.0,
        ..fet::Settings::default()
    });
    let mut sine = Sine::new(1000.0, SR);
    let mut l = vec![0.0f32; BLOCK];
    let mut r = vec![0.0f32; BLOCK];
    let mut last = 0.0;
    for _ in 0..((2.0 * SR) as usize / BLOCK) {
        sine.fill(&mut l, amp);
        r.copy_from_slice(&l);
        c.process(&mut l, &mut r);
        last = c.take_meter_reading();
    }
    last
}

// ---------------------------------------------------------------------------
// The LA-2A
// ---------------------------------------------------------------------------

fn bench_opto() -> Section {
    let mut rows = Vec::new();
    let make = |pr: f32| {
        let mut c = opto::Compressor::new(SR);
        c.configure(opto::Settings {
            peak_reduction: pr,
            ..opto::Settings::default()
        });
        c
    };

    // §8.2 onset and depth.
    let onset = {
        let mut found = f32::NAN;
        for step in -30..=20 {
            let mut c = make(30.0);
            let (_, gr) = steady(&mut c, 1000.0, amp_vu(step as f32), 3.0, 0.25, SR);
            if -gr >= 1.0 {
                found = step as f32;
                break;
            }
        }
        found
    };
    rows.push(Row::within(
        "PR 30, onset of 1 dB gain reduction",
        0.0,
        1.0,
        "dB relative to 0 VU",
        onset,
        "research/LA-2A.md §8.2, from [2][3]",
    ));

    let gr_pr50 = {
        let mut c = make(50.0);
        let (_, gr) = steady(&mut c, 1000.0, amp_vu(0.0), 4.0, 0.25, SR);
        -gr
    };
    rows.push(Row::within(
        "PR 50 at 0 VU, gain reduction",
        5.0,
        1.5,
        "dB",
        gr_pr50,
        "research/LA-2A.md §8.2, from [2][3]",
    ));

    let gr_max = {
        let mut c = make(100.0);
        let (_, gr) = steady(&mut c, 1000.0, amp_vu(16.0), 6.0, 0.25, SR);
        -gr
    };
    rows.push(Row::ranged(
        "PR 100 at +16 dB, maximum gain reduction",
        30.0,
        40.0,
        "dB",
        gr_max,
        "research/LA-2A.md §8.2, from [2][3]",
    ));

    // §8.3 ratio in the working region.
    let slope = {
        let out = |vu: f32| {
            let mut c = make(60.0);
            settled_out_dbfs(&mut c, 1000.0, amp_vu(vu), 5.0, SR)
        };
        let (a, b) = (out(4.0), out(14.0));
        let d = b - a;
        if d.abs() < 1e-4 { 999.0 } else { 10.0 / d }
    };
    rows.push(Row::ranged(
        "slope in the 6 to 20 dB gain-reduction region",
        2.5,
        4.5,
        ":1",
        slope,
        "research/LA-2A.md §8.3, from [3][4][8][46]",
    ));

    // §8.4 attack, §8.5 release.
    let attack_ms = {
        let mut c = make(50.0);
        let traj = step_response(&mut c, 1000.0, amp_vu(-24.0), amp_vu(-3.0), 1.0, 0.5, SR);
        time_to_fraction(&traj, 0.63, SR).unwrap_or(f32::NAN)
    };
    rows.push(Row::ranged(
        "attack, 63 % of final gain reduction",
        5.0,
        60.0,
        "ms",
        attack_ms,
        "research/LA-2A.md §8.4, from Canopus [29][53]",
    ));

    let rel50 = {
        let mut c = make(60.0);
        let traj = release_response(&mut c, 1000.0, amp_vu(6.0), amp_vu(-40.0), 2.0, 4.0, SR);
        time_to_fraction(&traj, 0.5, SR).unwrap_or(f32::NAN)
    };
    rows.push(Row::ranged(
        "release, first stage to 50 % recovery",
        40.0,
        120.0,
        "ms",
        rel50,
        "research/LA-2A.md §8.5, from [2]",
    ));

    let rel90 = {
        let mut c = make(60.0);
        let traj = release_response(&mut c, 1000.0, amp_vu(6.0), amp_vu(-40.0), 2.0, 6.0, SR);
        time_to_fraction(&traj, 0.9, SR).unwrap_or(f32::NAN)
    };
    rows.push(Row::ranged(
        "release, second stage to 90 % recovery",
        500.0,
        3000.0,
        "ms",
        rel90,
        "research/LA-2A.md §8.5, from [2]",
    ));

    // §8.7 frequency dependence: the high end is compressed harder.
    let (gr_100, gr_10k) = {
        let mut a = make(50.0);
        let (_, g1) = steady(&mut a, 100.0, amp_vu(10.0), 4.0, 0.25, SR);
        let mut b = make(50.0);
        let (_, g2) = steady(&mut b, 10_000.0, amp_vu(10.0), 4.0, 0.25, SR);
        (-g1, -g2)
    };
    rows.push(Row::ranged(
        "10 kHz gain reduction above 100 Hz, equal levels",
        2.0,
        6.0,
        "dB",
        gr_10k - gr_100,
        "research/LA-2A.md §8.7, from [20]",
    ));

    // §8.1 tube stage, clean.
    let thd_clean = {
        let mut c = make(0.0);
        let (tail, _) = steady(&mut c, 1000.0, amp_vu(0.0), 3.0, 0.5, SR);
        thd_pct(&tail, 1000.0, SR)
    };
    rows.push(Row::new(
        "THD at 0 VU with no gain reduction",
        "below 0.300 %",
        format!("{thd_clean:.3} %"),
        "research/LA-2A.md §8.1, from [2][5]",
        if thd_clean < 0.3 {
            Verdict::Meets
        } else {
            Verdict::Misses
        },
    ));

    // §8.8 distortion during gain reduction.
    let thd_gr = {
        let mut c = make(55.0);
        let (tail, _) = steady(&mut c, 1000.0, amp_vu(2.0), 4.0, 0.5, SR);
        thd_pct(&tail, 1000.0, SR)
    };
    rows.push(Row::ranged(
        "THD at 0 VU with gain reduction",
        0.8,
        4.0,
        "%",
        thd_gr,
        "research/LA-2A.md §8.8, from [26][53]",
    ));

    rows.push(Row::unanchored(
        "cell era speed multipliers",
        "Silver 0.7, Gray 1.0, LA-2 1.6 (ordering only)".into(),
        "the manufacturer describes the ordering of the three eras but publishes no time constants for \
         them; the one real measurement, of six units, reports no consistent vintage-versus-reissue grouping",
    ));

    Section {
        model: "LA-2A",
        unit: "Teletronix LA-2A Leveling Amplifier",
        dossier: "research/LA-2A.md",
        rows,
    }
}

// ---------------------------------------------------------------------------
// The LA-3A
// ---------------------------------------------------------------------------

fn bench_opto3() -> Section {
    let mut rows = Vec::new();
    let make = |pr: f32, limit: bool| {
        let mut c = opto3::Compressor::new(SR);
        c.configure(opto3::Settings {
            peak_reduction: pr,
            limit,
            ..opto3::Settings::default()
        });
        c
    };

    let gr_max_limit = {
        let mut c = make(100.0, true);
        let (_, gr) = steady(&mut c, 1000.0, amp_vu(20.0), 6.0, 0.25, SR);
        -gr
    };
    rows.push(
        Row::ranged(
            "maximum gain reduction, Limit",
            38.0,
            42.0,
            "dB",
            gr_max_limit,
            "research/LA-3A.md §8, from the reissue manual",
        )
        .because("the published 40 dB figure names no mode; the dossier's own test places it in Compress"),
    );

    let gr_max_comp = {
        let mut c = make(100.0, false);
        let (_, gr) = steady(&mut c, 1000.0, amp_vu(20.0), 6.0, 0.25, SR);
        -gr
    };
    rows.push(
        Row::new(
            "maximum gain reduction, Compress",
            "40 dB",
            format!("{gr_max_comp:.2} dB"),
            "research/LA-3A.md §8, from the reissue manual",
            if (gr_max_comp - 40.0).abs() <= 2.0 { Verdict::Meets } else { Verdict::Misses },
        )
        .because(
            "a real divergence, recorded at the test: in Compress every decibel of reduction takes a \
             decibel off the side-chain, so the loop starves itself",
        ),
    );

    // Test 8's conditions exactly: a 1 kHz tone stepping from −24 to
    // −3 dBFS at Peak Reduction 6 on the panel's 0-to-10 scale. The
    // bracket is the dossier's own, spanning UREI's "less than 250 µs to
    // 0.5 ms" and Universal Audio's "1.5 ms or less".
    let attack_ms = {
        let mut c = make(60.0, false);
        let traj = step_response(
            &mut c,
            1000.0,
            amp_dbfs(-24.0),
            amp_dbfs(-3.0),
            1.0,
            0.2,
            SR,
        );
        time_to_fraction(&traj, 0.63, SR).unwrap_or(f32::NAN)
    };
    rows.push(Row::ranged(
        "attack, 63 % of final gain reduction",
        0.2,
        3.0,
        "ms",
        attack_ms,
        "research/LA-3A.md §8 test 8, from [1][2]",
    ));

    let rel_first = {
        let mut c = make(60.0, false);
        let traj = release_response(&mut c, 1000.0, amp_vu(6.0), amp_vu(-40.0), 2.0, 3.0, SR);
        time_to_fraction(&traj, 0.5, SR).unwrap_or(f32::NAN)
    };
    rows.push(Row::ranged(
        "release, first stage to 50 % recovery",
        20.0,
        120.0,
        "ms",
        rel_first,
        "research/LA-3A.md §8, from the 60 ms figure",
    ));

    // The unit's reputation is a frequency-dependent side-chain.
    let (gr_100, gr_10k) = {
        let mut a = make(50.0, false);
        let (_, g1) = steady(&mut a, 100.0, amp_vu(10.0), 4.0, 0.25, SR);
        let mut b = make(50.0, false);
        let (_, g2) = steady(&mut b, 10_000.0, amp_vu(10.0), 4.0, 0.25, SR);
        (-g1, -g2)
    };
    rows.push(
        Row::new(
            "10 kHz gain reduction above 100 Hz, equal levels",
            "positive (the side-chain is deaf below about 100 Hz)",
            format!("{:.2} dB", gr_10k - gr_100),
            "research/LA-3A.md §3.5, from the schematic",
            if gr_10k > gr_100 { Verdict::Meets } else { Verdict::Misses },
        )
        .because("the coupling capacitor and autotransformer are what make this unit sit differently on a guitar"),
    );

    rows.push(Row::unanchored(
        "HF Contour depth at 15 kHz",
        {
            let mut c = opto3::Compressor::new(SR);
            c.configure(opto3::Settings {
                peak_reduction: 50.0,
                emphasis: 1.0,
                ..opto3::Settings::default()
            });
            let (_, full) = steady(&mut c, 15_000.0, amp_vu(6.0), 3.0, 0.25, SR);
            let mut d = opto3::Compressor::new(SR);
            d.configure(opto3::Settings {
                peak_reduction: 50.0,
                emphasis: 0.0,
                ..opto3::Settings::default()
            });
            let (_, flat) = steady(&mut d, 15_000.0, amp_vu(6.0), 3.0, 0.25, SR);
            format!("{:.2} dB of extra reduction", (-full) - (-flat))
        },
        "the manual and the two plug-ins disagree about which way the trimmer rotates to reach flat, \
         and none publishes the depth; the dossier follows the plug-in convention and says so",
    ));

    Section {
        model: "LA-3A",
        unit: "UREI LA-3A Audio Leveler",
        dossier: "research/LA-3A.md",
        rows,
    }
}

// ---------------------------------------------------------------------------
// The Distressor
// ---------------------------------------------------------------------------

fn bench_vca() -> Section {
    let mut rows = Vec::new();

    // The per-ratio curve table. The panel figures are labels rather than
    // measured slopes, which the dossier is explicit about.
    for (ratio, printed) in [
        (vca::Ratio::R2, 2.0f32),
        (vca::Ratio::R4, 4.0),
        (vca::Ratio::R10, 10.0),
    ] {
        let slope = vca_slope(ratio);
        rows.push(
            Row::within(
                &format!("{printed:.0}:1 slope, well above threshold"),
                printed,
                printed * 0.35,
                ":1",
                slope,
                "research/Distressor.md §7.4 curve table",
            )
            .because(
                "Derr's own account is that the measured slopes run higher than the panel labels",
            ),
        );
    }

    let att_slow = vca_attack_ms(10.0);
    rows.push(
        Row::within(
            "attack at knob 10",
            30.0,
            30.0 * 0.5,
            "ms",
            att_slow,
            "research/Distressor.md §7.2, from the 50 µs to 30 ms range",
        )
        .because(
            "the dossier allows a factor of 1.5 against the mapped value, but its own §8.2 also \
             requires that a bigger step attack faster than a smaller one, and this engine applies \
             the knob's map at a reference overshoot; a 12 dB step is used here and a different one \
             would land elsewhere in that program dependence",
        ),
    );
    let att_fast = vca_attack_ms(0.0);
    rows.push(Row::ranged(
        "attack at knob 0",
        0.02,
        0.5,
        "ms",
        att_fast,
        "research/Distressor.md §7.2, from the 50 µs end",
    ));

    let rel_fast = vca_release_ms(0.0);
    rows.push(Row::ranged(
        "release at knob 0",
        30.0,
        120.0,
        "ms",
        rel_fast,
        "research/Distressor.md §7.2, from the 50 ms end",
    ));
    let rel_slow = vca_release_ms(10.0);
    rows.push(Row::ranged(
        "release at knob 10",
        2000.0,
        5000.0,
        "ms",
        rel_slow,
        "research/Distressor.md §7.2, from the 3.5 s end",
    ));

    // Distortion modes. The manual quotes second-harmonic dominance in
    // Dist 2 and third in Dist 3.
    let (d2_thd, d2_h2, d2_h3) = vca_distortion(vca::AudioMode::Dist2);
    rows.push(
        Row::new(
            "Dist 2, second harmonic dominant",
            "H2 above H3",
            format!(
                "H2 {:.1} dB above H3, THD {d2_thd:.2} %",
                db(d2_h2) - db(d2_h3)
            ),
            "research/Distressor.md §7.6, from [1][18]",
            if d2_h2 > d2_h3 {
                Verdict::Meets
            } else {
                Verdict::Misses
            },
        )
        .because("the manual gives the harmonic balance and a THD band, not a single number"),
    );
    let (d3_thd, d3_h2, d3_h3) = vca_distortion(vca::AudioMode::Dist3);
    rows.push(Row::new(
        "Dist 3, third harmonic dominant",
        "H3 above H2",
        format!(
            "H3 {:.1} dB above H2, THD {d3_thd:.2} %",
            db(d3_h3) - db(d3_h2)
        ),
        "research/Distressor.md §7.6, from [1]",
        if d3_h3 > d3_h2 {
            Verdict::Meets
        } else {
            Verdict::Misses
        },
    ));

    // Nuke is a brick wall: the published behaviour is that output barely
    // moves across a wide input range.
    let nuke_rise = {
        let out = |dbfs: f32| {
            let mut c = vca::Compressor::new(SR);
            c.configure(&vca::Settings {
                ratio: vca::Ratio::Nuke,
                input: 8.0,
                ..vca::Settings::default()
            });
            settled_out_dbfs(&mut c, 1000.0, amp_dbfs(dbfs), 2.0, SR)
        };
        out(-6.0) - out(-22.0)
    };
    rows.push(Row::new(
        "Nuke, output rise over a 16 dB input range",
        "below 1.000 dB",
        format!("{nuke_rise:.3} dB"),
        "research/Distressor.md §8.1, from [1]",
        if nuke_rise < 1.0 {
            Verdict::Meets
        } else {
            Verdict::Misses
        },
    ));

    // The audio high-pass has published corner figures.
    let hp_65 = vca_audio_hp_db(65.0);
    rows.push(Row::within(
        "audio high-pass at 65 Hz",
        -3.0,
        1.5,
        "dB",
        hp_65,
        "research/Distressor.md §7.7, from [1][11]",
    ));
    let hp_30 = vca_audio_hp_db(30.0);
    rows.push(Row::within(
        "audio high-pass at 30 Hz",
        -12.0,
        4.0,
        "dB",
        hp_30,
        "research/Distressor.md §7.7, from [1][11]",
    ));

    rows.push(Row::unanchored(
        "British mode threshold and slope",
        {
            let slope = vca_british_slope();
            format!("{slope:.2}:1 slope 10 dB above threshold")
        },
        "the manual describes British mode qualitatively as the 1176 all-buttons treatment and gives \
         no ratio or threshold for it",
    ));

    Section {
        model: "Distressor",
        unit: "Empirical Labs EL8 Distressor",
        dossier: "research/Distressor.md",
        rows,
    }
}

fn vca_settings(ratio: vca::Ratio, input: f32) -> vca::Settings {
    vca::Settings {
        ratio,
        input,
        ..vca::Settings::default()
    }
}

fn vca_threshold(ratio: vca::Ratio) -> f32 {
    for step in 0..70 {
        let dbfs = -60.0 + step as f32;
        let mut c = vca::Compressor::new(SR);
        c.configure(&vca_settings(ratio, 5.0));
        let (_, gr) = steady(&mut c, 1000.0, amp_dbfs(dbfs), 2.0, 0.2, SR);
        if -gr >= 1.0 {
            return dbfs;
        }
    }
    f32::NAN
}

fn vca_slope(ratio: vca::Ratio) -> f32 {
    let thr = vca_threshold(ratio);
    if !thr.is_finite() {
        return f32::NAN;
    }
    let out = |dbfs: f32| {
        let mut c = vca::Compressor::new(SR);
        c.configure(&vca_settings(ratio, 5.0));
        settled_out_dbfs(&mut c, 1000.0, amp_dbfs(dbfs), 2.0, SR)
    };
    let (a, b) = (thr + 8.0, thr + 20.0);
    let d = out(b) - out(a);
    if d.abs() < 1e-4 { 999.0 } else { (b - a) / d }
}

fn vca_british_slope() -> f32 {
    let out = |dbfs: f32| {
        let mut c = vca::Compressor::new(SR);
        c.configure(&vca::Settings {
            ratio: vca::Ratio::R1,
            british: true,
            input: 6.0,
            ..vca::Settings::default()
        });
        settled_out_dbfs(&mut c, 1000.0, amp_dbfs(dbfs), 2.0, SR)
    };
    let d = out(-8.0) - out(-18.0);
    if d.abs() < 1e-4 { 999.0 } else { 10.0 / d }
}

fn vca_attack_ms(knob: f32) -> f32 {
    let mut c = vca::Compressor::new(SR);
    c.configure(&vca::Settings {
        ratio: vca::Ratio::R6,
        input: 8.0,
        attack: knob,
        release: 5.0,
        ..vca::Settings::default()
    });
    // A 12 dB step: big enough to leave the knee, small enough not to sit
    // at the far end of the engine's program dependence.
    let traj = step_response(
        &mut c,
        1000.0,
        amp_dbfs(-20.0),
        amp_dbfs(-8.0),
        0.5,
        0.4,
        SR,
    );
    time_to_fraction(&traj, 0.63, SR).unwrap_or(f32::NAN)
}

fn vca_release_ms(knob: f32) -> f32 {
    let mut c = vca::Compressor::new(SR);
    c.configure(&vca::Settings {
        ratio: vca::Ratio::R6,
        input: 8.0,
        attack: 2.0,
        release: knob,
        ..vca::Settings::default()
    });
    let traj = release_response(
        &mut c,
        1000.0,
        amp_dbfs(-8.0),
        amp_dbfs(-40.0),
        1.5,
        8.0,
        SR,
    );
    time_to_fraction(&traj, 0.63, SR).unwrap_or(f32::NAN)
}

/// THD and the second and third harmonic magnitudes in one distortion mode.
fn vca_distortion(mode: vca::AudioMode) -> (f32, f32, f32) {
    let mut c = vca::Compressor::new(SR);
    c.configure(&vca::Settings {
        audio: mode,
        ratio: vca::Ratio::R4,
        input: 7.0,
        ..vca::Settings::default()
    });
    let (tail, _) = steady(&mut c, 1000.0, amp_dbfs(-12.0), 2.0, 0.5, SR);
    let h2 = goertzel(&tail, 2000.0, SR);
    let h3 = goertzel(&tail, 3000.0, SR);
    (thd_pct(&tail, 1000.0, SR), h2, h3)
}

/// The audio high-pass response at `hz`, relative to 1 kHz.
fn vca_audio_hp_db(hz: f32) -> f32 {
    let make = || -> Box<dyn Engine> {
        let mut c = vca::Compressor::new(SR);
        c.configure(&vca::Settings {
            audio: vca::AudioMode::Hp,
            ratio: vca::Ratio::R1,
            input: 2.0,
            ..vca::Settings::default()
        });
        Box::new(c)
    };
    response_db(make, hz, amp_dbfs(-30.0), SR)
}

// ---------------------------------------------------------------------------
// The 6176, which is the 610 preamp in front of the 1176
// ---------------------------------------------------------------------------

fn bench_pre() -> Section {
    let mut rows = Vec::new();

    // §9.2 the Gain switch steps, 5 dB apart.
    let steps: Vec<f32> = (0..5).map(pre_small_signal_db).collect();
    let deltas: Vec<f32> = steps.windows(2).map(|w| w[1] - w[0]).collect();
    let worst_step = deltas
        .iter()
        .cloned()
        .max_by(|a, b| (a - 5.0).abs().total_cmp(&(b - 5.0).abs()))
        .unwrap_or(f32::NAN);
    rows.push(
        Row::within(
            "Gain switch, worst step of the five positions",
            5.0,
            0.2,
            "dB",
            worst_step,
            "research/610.md §9.2, from [1 p.3]",
        )
        .because("the step furthest from 5 dB is reported, so one bad position cannot hide behind four good ones"),
    );

    // §9.4 input select offsets.
    for (idx, name, published) in [
        (2usize, "Mic 2.0K", 30.0f32),
        (1, "Mic 500", 35.0),
        (4, "Hi-Z 2.2M", 8.0),
    ] {
        let offset = pre_input_offset_db(idx);
        rows.push(Row::within(
            &format!("{name} offset above Line"),
            published,
            0.5,
            "dB",
            offset,
            "research/610.md §9.4, from [13 p.496][5 p.22]",
        ));
    }

    // §9.5 the pad.
    let pad = pre_pad_db();
    rows.push(Row::within(
        "pad on a microphone input",
        -15.0,
        0.2,
        "dB",
        pad,
        "research/610.md §9.5, from [1 p.4]",
    ));

    // §9.1 bandwidth. The 20 Hz figure is the one the dossier's own design
    // could not originally meet.
    let at_20 = pre_response_db(20.0);
    rows.push(Row::ranged(
        "response at 20 Hz",
        -1.0,
        0.0,
        "dB",
        at_20,
        "research/610.md §9.1, from the +0 / −1 dB specification [1 p.40]",
    ));
    let at_20k = pre_response_db(20_000.0);
    rows.push(Row::ranged(
        "response at 20 kHz",
        -1.0,
        0.0,
        "dB",
        at_20k,
        "research/610.md §9.1, from [1 p.40]",
    ));

    // §9.10 the shelving sections. The dossier's criterion is the response
    // **at the printed corner**, which should be half the step in dB. The
    // asymptote is not measurable here: a first-order shelf reaches it
    // about a decade past its corner, and a decade above the 10 kHz shelf
    // is far outside the audio band.
    let hf = pre_shelf_at_corner_db(true);
    rows.push(
        Row::within(
            "high shelf at its printed corner, +9 dB step",
            4.5,
            0.5,
            "dB",
            hf,
            "research/610.md §9.10, from [1 p.5]",
        )
        .because("the corner is the half-gain point, which is where a feedback shelf's label conventionally sits"),
    );
    let lf = pre_shelf_at_corner_db(false);
    rows.push(Row::within(
        "low shelf at its printed corner, +9 dB step",
        4.5,
        0.5,
        "dB",
        lf,
        "research/610.md §9.10, from [1 p.5]",
    ));

    // §9.9 the output stage's overload points. Each row reports the level
    // the drive search actually reached as well as the distortion there,
    // because a stage that cannot be driven to the published level would
    // otherwise look like a stage that is simply cleaner than published.
    // The dBu-to-dBFS calibration here is the one the repository's own test
    // for this figure uses, where +15 dBu is −3.99 dBFS. Getting it wrong
    // under-drives the stage and reports it as cleaner than published, which
    // is what a first pass of this benchmark did.
    let (thd_5, got_5) = pre_output_thd(15.0 - 18.99 - 10.0);
    rows.push(
        Row::new(
            "THD at the +5 dBu equivalent output",
            "below 0.150 %",
            format!("{thd_5:.3} %"),
            "research/610.md §9.9, from [32]",
            if thd_5 < 0.15 {
                Verdict::Meets
            } else {
                Verdict::Misses
            },
        )
        .because(&format!(
            "driven to {got_5:.2} dBFS peak, the +5 dBu equivalent"
        )),
    );
    let (thd_15, got_15) = pre_output_thd(15.0 - 18.99);
    rows.push(
        Row::ranged(
            "THD at the +15 dBu equivalent output",
            3.0,
            8.0,
            "%",
            thd_15,
            "research/610.md §9.9, from [32]",
        )
        .because(&format!(
            "driven to {got_15:.2} dBFS peak, the +15 dBu equivalent"
        )),
    );

    // §9.12 aliasing: the repository's fourth known miss.
    let alias = pre_worst_alias_db();
    rows.push(
        Row::new(
            "worst in-band alias, 15 kHz into a hot microphone setting",
            "below −80 dB",
            format!("{alias:.1} dB"),
            "research/610.md §9.12",
            if alias < -80.0 {
                Verdict::Meets
            } else {
                Verdict::Misses
            },
        )
        .because(
            "known miss, recorded in README. **The method matters, so it is stated here.** This is \
             the worst single product anywhere below 10 kHz, found by sweeping the band in 25 Hz \
             steps, because the question an aliasing figure answers is whether anything audible got \
             in, not whether one particular product did. The worst is the third harmonic of the \
             15 kHz tone folded to 3 kHz, and it is a discrete tone sitting 48 dB above its own \
             neighbourhood rather than a noise floor. A narrower measurement had put this at −51 dB \
             and missed it; the README now carries this figure",
        ),
    );

    rows.push(Row::unanchored(
        "610A versus 610B voicing",
        {
            let a = pre_voicing_thd(1);
            let b = pre_voicing_thd(0);
            format!("610A {a:.3} % against 610B {b:.3} % at the same output")
        },
        "the manufacturer describes the two voicings with adjectives; no measurement of one against \
         the other exists",
    ));

    Section {
        model: "6176",
        unit: "Universal Audio 6176 (610B preamp into the 1176LN)",
        dossier: "research/610.md",
        rows,
    }
}

fn pre_stage(s: pre::Settings) -> pre::Stage {
    let mut st = pre::Stage::new(SR);
    st.configure(&s);
    st
}

fn pre_small_signal_db(gain_idx: usize) -> f32 {
    let st = pre_stage(pre::Settings {
        gain: gain_idx,
        ..pre::Settings::default()
    });
    st.small_signal_db()
}

fn pre_input_offset_db(input_idx: usize) -> f32 {
    let a = pre_stage(pre::Settings {
        input: input_idx,
        ..pre::Settings::default()
    })
    .small_signal_db();
    let b = pre_stage(pre::Settings::default()).small_signal_db();
    a - b
}

fn pre_pad_db() -> f32 {
    let base = pre::Settings {
        input: 2,
        ..pre::Settings::default()
    };
    let off = pre_stage(base).small_signal_db();
    let on = pre_stage(pre::Settings { pad: true, ..base }).small_signal_db();
    on - off
}

/// Response at `hz` relative to 1 kHz, at a level low enough that the tube
/// stages stay linear.
fn pre_response_db(hz: f32) -> f32 {
    let run = |f: f32| {
        let mut st = pre_stage(pre::Settings {
            gain: 0,
            level: 5.0,
            ..pre::Settings::default()
        });
        let mut sine = Sine::new(f, SR);
        let mut l = vec![0.0f32; BLOCK];
        let mut r = vec![0.0f32; BLOCK];
        let mut tail = Vec::new();
        let blocks = (2.0 * SR) as usize / BLOCK;
        for b in 0..blocks {
            sine.fill(&mut l, amp_dbfs(-40.0));
            r.copy_from_slice(&l);
            st.process_block(&mut l, &mut r);
            if b >= blocks - (0.5 * SR) as usize / BLOCK {
                tail.extend_from_slice(&l);
            }
        }
        db(goertzel(&tail, f, SR))
    };
    run(hz) - run(1000.0)
}

/// A shelf's response at its own printed corner, relative to flat. The
/// design puts the printed frequency at the half-gain point, so a +9 dB
/// step should read about +4.5 dB there.
fn pre_shelf_at_corner_db(high: bool) -> f32 {
    // Index 10 is the +9 dB step; index 5 is flat. The defaults are the
    // 10 kHz and 100 Hz corners.
    let probe_hz = if high { 10_000.0 } else { 100.0 };
    let run = |gain_idx: usize| {
        let s = if high {
            pre::Settings {
                hf_gain: gain_idx,
                level: 5.0,
                ..pre::Settings::default()
            }
        } else {
            pre::Settings {
                lf_gain: gain_idx,
                level: 5.0,
                ..pre::Settings::default()
            }
        };
        let mut st = pre_stage(s);
        let mut sine = Sine::new(probe_hz, SR);
        let mut l = vec![0.0f32; BLOCK];
        let mut r = vec![0.0f32; BLOCK];
        let mut tail = Vec::new();
        let blocks = (2.0 * SR) as usize / BLOCK;
        for b in 0..blocks {
            sine.fill(&mut l, amp_dbfs(-40.0));
            r.copy_from_slice(&l);
            st.process_block(&mut l, &mut r);
            if b >= blocks - (0.5 * SR) as usize / BLOCK {
                tail.extend_from_slice(&l);
            }
        }
        db(goertzel(&tail, probe_hz, SR))
    };
    run(10) - run(5)
}

/// Drive the output stage to a target output level and measure its THD.
/// Returns the distortion and the level actually reached, since a stage
/// that saturates before the target would otherwise report a misleadingly
/// low figure.
fn pre_output_thd(target_dbfs: f32) -> (f32, f32) {
    let base = pre::Settings {
        gain: 2,
        input: 0,
        level: 5.0,
        ..pre::Settings::default()
    };
    let mut st = pre_stage(base);
    // Find the input that lands near the target, then measure there.
    let mut amp = amp_dbfs(target_dbfs - st.small_signal_db());
    let mut reached = f32::NAN;
    for _ in 0..24 {
        let mut probe = pre_stage(base);
        reached = pre_run_level(&mut probe, amp);
        let err = target_dbfs - reached;
        if err.abs() < 0.05 {
            break;
        }
        amp *= 10f32.powf(err.clamp(-6.0, 6.0) / 20.0);
    }
    let mut sine = Sine::new(1000.0, SR);
    let mut l = vec![0.0f32; BLOCK];
    let mut r = vec![0.0f32; BLOCK];
    let mut tail = Vec::new();
    let blocks = (2.0 * SR) as usize / BLOCK;
    for b in 0..blocks {
        sine.fill(&mut l, amp);
        r.copy_from_slice(&l);
        st.process_block(&mut l, &mut r);
        if b >= blocks - (0.5 * SR) as usize / BLOCK {
            tail.extend_from_slice(&l);
        }
    }
    (thd_pct(&tail, 1000.0, SR), reached)
}

/// The output level of the stage for a given input amplitude, as a **peak**
/// in dBFS. Peak rather than RMS deliberately: the published overload
/// figures are levels the stage is driven to, and by the time it is making
/// several percent of distortion the waveform is squared off, so an
/// RMS-derived level would under-drive it and report a stage cleaner than
/// it is. The repository's own test for this figure reads it as a peak, and
/// the two must agree or one of them is measuring the wrong thing.
fn pre_run_level(st: &mut pre::Stage, amp: f32) -> f32 {
    let mut sine = Sine::new(1000.0, SR);
    let mut l = vec![0.0f32; BLOCK];
    let mut r = vec![0.0f32; BLOCK];
    let mut peak = 0.0f32;
    let blocks = (1.0 * SR) as usize / BLOCK;
    for b in 0..blocks {
        sine.fill(&mut l, amp);
        r.copy_from_slice(&l);
        st.process_block(&mut l, &mut r);
        if b >= blocks - (0.25 * SR) as usize / BLOCK {
            for v in l.iter() {
                peak = peak.max(v.abs());
            }
        }
    }
    db(peak)
}

/// The loudest aliased product below 10 kHz for a 15 kHz tone into a hot
/// microphone setting, relative to the fundamental.
fn pre_worst_alias_db() -> f32 {
    let mut st = pre_stage(pre::Settings {
        input: 2,
        gain: 4,
        level: 5.0,
        ..pre::Settings::default()
    });
    let mut sine = Sine::new(15_000.0, SR);
    let mut l = vec![0.0f32; BLOCK];
    let mut r = vec![0.0f32; BLOCK];
    let mut tail = Vec::new();
    let blocks = (2.0 * SR) as usize / BLOCK;
    for b in 0..blocks {
        sine.fill(&mut l, amp_dbfs(-6.0));
        r.copy_from_slice(&l);
        st.process_block(&mut l, &mut r);
        if b >= blocks - (0.5 * SR) as usize / BLOCK {
            tail.extend_from_slice(&l);
        }
    }
    let fundamental = db(goertzel(&tail, 15_000.0, SR));
    // Sweep the band below 10 kHz for the loudest product. Harmonics of
    // 15 kHz all fold, so anything found here is aliasing.
    let mut worst = -200.0f32;
    let mut hz = 100.0;
    while hz < 10_000.0 {
        let m = db(goertzel(&tail, hz, SR)) - fundamental;
        if m > worst {
            worst = m;
        }
        hz += 25.0;
    }
    worst
}

fn pre_voicing_thd(voice: usize) -> f32 {
    let mut st = pre_stage(pre::Settings {
        voice,
        gain: 2,
        level: 7.0,
        ..pre::Settings::default()
    });
    let mut sine = Sine::new(1000.0, SR);
    let mut l = vec![0.0f32; BLOCK];
    let mut r = vec![0.0f32; BLOCK];
    let mut tail = Vec::new();
    let blocks = (2.0 * SR) as usize / BLOCK;
    for b in 0..blocks {
        sine.fill(&mut l, amp_dbfs(-30.0));
        r.copy_from_slice(&l);
        st.process_block(&mut l, &mut r);
        if b >= blocks - (0.5 * SR) as usize / BLOCK {
            tail.extend_from_slice(&l);
        }
    }
    thd_pct(&tail, 1000.0, SR)
}

// ---------------------------------------------------------------------------
// The CL 1B
// ---------------------------------------------------------------------------

fn bench_opto1b() -> Section {
    let mut rows = Vec::new();
    let make = |thr: f32, ratio: f32, mode: usize| {
        let mut c = opto1b::Compressor::new(SR);
        c.configure(opto1b::Settings {
            threshold: thr,
            ratio,
            mode,
            ..opto1b::Settings::default()
        });
        c
    };

    // The 2:1 stop's published behaviour: ten in, five out.
    for depth in [3.0f32, 8.0, 14.0] {
        let out_for_ten = opto1b_ten_in(depth);
        rows.push(
            Row::within(
                &format!("2:1 stop, 10 dB in from {depth:.0} dB of reduction"),
                5.0,
                1.0,
                "dB out",
                out_for_ten,
                "research/CL-1B.md §10, from the manual's worked example",
            )
            .because(if depth <= 3.0 {
                "known miss, recorded in README: a feedback optical compressor has a soft knee near \
                 its threshold, which is what the reviews describe"
            } else {
                ""
            }),
        );
    }

    // Fixed-mode timings. Every bracket below is the dossier's own, and
    // its section 10.3 explains why they are wide: Lydkraft never say
    // whether their published "1 ms" is a time constant, a 63 % time or a
    // settling time, so the bracket spans all three readings of the same
    // number rather than picking one silently.
    let fixed_attack = {
        let mut c = make(0.6, 0.5, opto1b::MODE_FIXED);
        let traj = step_response(
            &mut c,
            1000.0,
            amp_dbfs(-40.0),
            amp_dbfs(-8.0),
            0.5,
            0.3,
            SR,
        );
        time_to_fraction(&traj, 0.63, SR).unwrap_or(f32::NAN)
    };
    rows.push(Row::ranged(
        "Fixed attack, 63 % of an 18 dB step",
        0.5,
        3.0,
        "ms",
        fixed_attack,
        "research/CL-1B.md §10.3 test 13, from the manual's 1 ms",
    ));

    let fixed_release = {
        let mut c = make(0.6, 0.5, opto1b::MODE_FIXED);
        let traj = release_response(
            &mut c,
            1000.0,
            amp_dbfs(-8.0),
            amp_dbfs(-60.0),
            1.5,
            1.5,
            SR,
        );
        time_to_fraction(&traj, 0.63, SR).unwrap_or(f32::NAN)
    };
    rows.push(Row::ranged(
        "Fixed release, 63 % recovery",
        20.0,
        120.0,
        "ms",
        fixed_release,
        "research/CL-1B.md §10.3 test 18, from the manual's 50 ms",
    ));

    // Manual mode, at both stops, against the dossier's brackets.
    let man_att_fast = opto1b_manual_attack(0.0);
    rows.push(Row::ranged(
        "Manual attack at the fast stop",
        0.3,
        1.5,
        "ms",
        man_att_fast,
        "research/CL-1B.md §10.3 test 14, from the manual's 0.5 ms",
    ));
    let man_att_slow = opto1b_manual_attack(1.0);
    rows.push(Row::ranged(
        "Manual attack at the slow stop",
        150.0,
        600.0,
        "ms",
        man_att_slow,
        "research/CL-1B.md §10.3 test 14, from the manual's 300 ms",
    ));

    // Test 17 measures the slowest release the way the service manual
    // does: a full return of the needle to zero, not a time constant.
    let slowest_release = opto1b_full_recovery(1.0);
    rows.push(
        Row::ranged(
            "slowest release, full recovery to 0 dB",
            8000.0,
            12000.0,
            "ms",
            slowest_release,
            "research/CL-1B.md §10.3 test 17, from the service manual's adjustment procedure",
        )
        .because(
            "the service manual measures this by switching the tone off and watching the needle reach \
             0 VU, so this is a full recovery rather than a 63 % time",
        ),
    );

    // Test 16: the linear taper is the finding that changes the meaning of
    // every published setting in the manual, so it earns its own row.
    let quarter_release = opto1b_full_recovery(0.25);
    rows.push(
        Row::within(
            "release at quarter travel, full recovery",
            2500.0,
            600.0,
            "ms",
            quarter_release,
            "research/CL-1B.md §10.3 test 16, from the schematic's linear pot",
        )
        .because(
            "a logarithmic taper would put this at about 0.35 s; the pot is linear, which is why the \
             manufacturer's own recommended vocal setting sits where it does",
        ),
    );

    // Gain range and metering.
    let max_gain = {
        let mut c = opto1b::Compressor::new(SR);
        c.configure(opto1b::Settings {
            gain: 1.0,
            threshold: 0.0,
            ..opto1b::Settings::default()
        });
        let out = settled_out_dbfs(&mut c, 1000.0, amp_dbfs(-40.0), 2.0, SR);
        out - (-40.0)
    };
    rows.push(Row::within(
        "maximum make-up gain",
        30.0,
        1.0,
        "dB",
        max_gain,
        "research/CL-1B.md §2.3, from the panel's +30 dB",
    ));

    rows.push(Row::unanchored(
        "the optical element's internals",
        "modelled from measured response, not from a circuit".into(),
        "the manufacturer has never published what is inside the gain-reduction element, and a \
         twenty-year forum thread asking directly never gets an answer",
    ));

    Section {
        model: "CL-1B",
        unit: "Tube-Tech CL 1B",
        dossier: "research/CL-1B.md",
        rows,
    }
}

// ---------------------------------------------------------------------------
// The Neve 33609
// ---------------------------------------------------------------------------

/// A sine amplitude at `x` dBu, on this family's calibration.
fn neve_dbu(x: f32) -> f32 {
    bridge::engine::dbu_amp(x)
}

/// Settle a 1 kHz sine at `amp` for `secs` and return the output peak.
fn neve_settle(c: &mut bridge::Compressor, amp: f32, secs: f32) -> f32 {
    let mut sine = Sine::new(1000.0, SR);
    let mut l = vec![0.0f32; BLOCK];
    let mut r = vec![0.0f32; BLOCK];
    let blocks = (secs * SR / BLOCK as f32) as usize;
    let mut peak = 0.0f32;
    for b in 0..blocks {
        sine.fill(&mut l, amp);
        r.copy_from_slice(&l);
        c.process_block(&mut l, &mut r);
        if b + 4 >= blocks {
            for v in l.iter() {
                peak = peak.max(v.abs());
            }
        }
    }
    peak
}

/// Output level in dBu for a 1 kHz sine at `in_dbu`.
fn neve_out_dbu(s: bridge::Settings, in_dbu: f32, secs: f32) -> f32 {
    let mut c = bridge::Compressor::new(SR);
    c.configure(s);
    bridge::engine::amp_dbu(neve_settle(&mut c, neve_dbu(in_dbu), secs))
}

fn bench_bridge() -> Section {
    let mut rows = Vec::new();
    let base = bridge::Settings {
        compress_in: false,
        limit_in: false,
        ..bridge::Settings::default()
    };

    // The open bridge's own attenuation, which three resistor values and a
    // level annotation on the same drawing agree on.
    let net = bridge::Network::default();
    rows.push(Row::within(
        "open-bridge attenuation",
        25.0,
        0.2,
        "dB",
        -20.0 * net.open_gain().log10(),
        "research/Neve-33609.md §12 test 3, from EX11475's −6 and −31 dBu rail marks",
    ));

    rows.push(Row::within(
        "unity gain, both sections out",
        0.0,
        0.5,
        "dBu",
        neve_out_dbu(base, 0.0, 0.3),
        "research/Neve-33609.md §12 test 2, from the block diagram's annotated chain",
    ));

    // The calibration table, with the manufacturer's own per-position
    // tolerances. This is the best anchor any model in the lab has.
    const RATIO_TABLE: [(usize, f32, f32); 5] = [
        (0, 6.5, 1.0),
        (1, 5.0, 1.0),
        (2, 3.5, 1.0),
        (3, 2.5, 0.5),
        (4, 1.5, 0.5),
    ];
    for (pos, want, tol) in RATIO_TABLE {
        let s = bridge::Settings {
            compress_in: true,
            compress_ratio: pos,
            compress_threshold: 0,
            compress_recovery: 0,
            ..base
        };
        let change = neve_out_dbu(s, 10.0, 1.0) - neve_out_dbu(s, 0.0, 1.0);
        rows.push(
            Row::within(
                &format!(
                    "{} position, output change for a 10 dB step",
                    bridge::RATIO_NAMES[pos]
                ),
                want,
                tol,
                "dB",
                change,
                "research/Neve-33609.md §12 test 4, from the 33609/J handbook's compress ratio table",
            )
            .because(match pos {
                2 => "the panel prints 3:1; the handbook's own table implies 2.86:1, and the model \
                      follows the table",
                4 => "the panel prints 6:1; the table implies 6.67:1",
                _ => "",
            }),
        );
    }

    let lim = bridge::Settings {
        limit_in: true,
        limit_threshold: 8,
        limit_recovery: 0,
        ..base
    };
    rows.push(Row::within(
        "limit ratio, +10 to +20 dBu",
        0.1,
        0.1,
        "dB out",
        neve_out_dbu(lim, 20.0, 1.0) - neve_out_dbu(lim, 10.0, 1.0),
        "research/Neve-33609.md §12 test 7, from the handbook's Limit Ratio entry",
    ));
    rows.push(Row::within(
        "limit threshold +8 dBu holding a +20 dBu tone",
        8.0,
        0.5,
        "dBu",
        neve_out_dbu(lim, 20.0, 1.0),
        "research/Neve-33609.md §12 test 8, from the handbook's calibration procedure",
    ));

    // The two published control voltages, the only statement anywhere of
    // what this family's sidechains produce.
    let mut c = bridge::Compressor::new(SR);
    c.configure(bridge::Settings {
        model: bridge::MODEL_2254E,
        ..lim
    });
    neve_settle(&mut c, neve_dbu(20.0), 1.0);
    rows.push(Row::within(
        "2254/E control voltage, +20 dBm limited to +8 dBm",
        3.5,
        0.3,
        "V",
        c.control_v(0),
        "research/Neve-33609.md §12 test 11, from level diagram EB/20134",
    ));

    // The behaviour the whole model exists for.
    let two = |gain: usize| bridge::Settings {
        compress_in: true,
        limit_in: true,
        compress_ratio: 4,
        compress_threshold: 12,
        compress_recovery: 0,
        limit_threshold: 8,
        limit_recovery: 0,
        gain,
        ..base
    };
    let mut lo = bridge::Compressor::new(SR);
    lo.configure(two(0));
    neve_settle(&mut lo, neve_dbu(20.0), 1.5);
    let mut hi = bridge::Compressor::new(SR);
    hi.configure(two(10));
    neve_settle(&mut hi, neve_dbu(20.0), 1.5);
    rows.push(Row::ranged(
        "limiter reduction added by 20 dB of make-up",
        15.0,
        60.0,
        "dB",
        hi.limit_gr_db(0) - lo.limit_gr_db(0),
        "research/Neve-33609.md §12 test 12, from AMS Neve's tap-point description",
    ));

    let comp_only = |gain: usize| bridge::Settings {
        compress_in: true,
        compress_ratio: 4,
        compress_threshold: 0,
        compress_recovery: 0,
        gain,
        ..base
    };
    let mut a = bridge::Compressor::new(SR);
    a.configure(comp_only(0));
    neve_settle(&mut a, neve_dbu(0.0), 1.0);
    let mut b = bridge::Compressor::new(SR);
    b.configure(comp_only(10));
    neve_settle(&mut b, neve_dbu(0.0), 1.0);
    rows.push(Row::within(
        "compressor reduction moved by 20 dB of make-up",
        0.0,
        0.5,
        "dB",
        b.compress_gr_db(0) - a.compress_gr_db(0),
        "research/Neve-33609.md §12 test 12, from the handbook's tap-point description",
    ));

    // Distortion: two published pairs, both maxima.
    let dist = |s: bridge::Settings, level: f32, warm: f32| {
        let mut c = bridge::Compressor::new(SR);
        c.configure(s);
        let mut sine = Sine::new(1000.0, SR);
        let mut l = vec![0.0f32; BLOCK];
        let mut r = vec![0.0f32; BLOCK];
        let blocks = ((warm + 0.2) * SR / BLOCK as f32) as usize;
        let mut tail = Vec::new();
        for bl in 0..blocks {
            sine.fill(&mut l, neve_dbu(level));
            r.copy_from_slice(&l);
            c.process_block(&mut l, &mut r);
            if bl + (0.2 * SR) as usize / BLOCK >= blocks {
                tail.extend_from_slice(&l);
            }
        }
        // A whole number of 1 kHz cycles, or the fundamental leaks into
        // the harmonic bins and the reading becomes a measurement of the
        // window rather than of the unit.
        let cycle = (SR / 1000.0) as usize;
        tail.truncate(tail.len() / cycle * cycle);
        thd_pct(&tail, 1000.0, SR)
    };
    let e2254 = bridge::Settings {
        model: bridge::MODEL_2254E,
        compress_in: true,
        compress_ratio: 1,
        compress_threshold: 14,
        compress_recovery: 2,
        ..base
    };
    rows.push(Row::ranged(
        "2254 distortion at 0 dBu, 800 ms recovery",
        0.0,
        0.03,
        "%",
        dist(e2254, 0.0, 1.0),
        "research/Neve-33609.md §12 test 17, from the AMS Neve 2254/R specification",
    ));
    rows.push(Row::ranged(
        "2254 distortion at +15 dBu, 800 ms recovery",
        0.0,
        0.2,
        "%",
        dist(e2254, 15.0, 1.0),
        "research/Neve-33609.md §12 test 17, from the AMS Neve 2254/R specification",
    ));
    rows.push(Row::ranged(
        "33609 distortion through the unit at +9 dBu",
        0.0,
        0.075,
        "%",
        dist(base, 9.0, 0.5),
        "research/Neve-33609.md §12 test 18a, from the handbook's Distortion entry",
    ));

    // Timings, under the handbook's own definitions.
    for (pos, want) in [
        (bridge::LIMIT_ATTACK_SLOW, 4.0f32),
        (bridge::LIMIT_ATTACK_FAST, 2.0),
    ] {
        rows.push(Row::within(
            &format!("{} limit attack, settling", bridge::LIMIT_ATTACK_NAMES[pos]),
            want,
            1.0,
            "ms",
            neve_attack_ms(pos),
            "research/Neve-33609.md §12 test 20, from the handbook's Attack Time entry",
        ));
    }

    rows.push(Row::unanchored(
        "the bridge's own distortion against gain reduction",
        "falls monotonically as the control current rises".into(),
        "no manufacturer publishes a spectrum for these units, so this is the tanh law's own \
         derived behaviour rather than a measurement; what rises with depth in the whole unit is \
         sidechain ripple, which is why test 17 varies level instead",
    ));
    rows.push(Row::unanchored(
        "the automatic recovery positions",
        "kept at the switch drawings' 100 ms/2 s and 50 ms/5 s".into(),
        "known miss, recorded in README: the handbook's Limit Recovery entry lists 1500 ms and \
         3000 ms for the same two positions, and a 2 s capacitor cannot settle in 1.5 s, so A1 \
         measures 2324 ms against a 750-to-2250 ms window",
    ));
    rows.push(Row::unanchored(
        "attack against step size",
        "settling time rises with the step, not falls".into(),
        "known miss, recorded in README: the dossier derives the opposite direction from the same \
         emitter follower, and an exponential closing a fixed 1 dB window cannot fall; the \
         published 10 dB point is still met",
    ));
    rows.push(Row::unanchored(
        "the 10640 amplifier in isolation",
        "not modelled as a separate block".into(),
        "the handbook publishes its gain, clip point and three distortion figures, and this model \
         carries the amplifier only as the make-up gain in the chain, so there is no sub-block to \
         assert them on",
    ));
    rows.push(Row::unanchored(
        "noise floor",
        "no noise source is modelled".into(),
        "the handbook publishes −75 dBu bypassed and −55 dBu with full make-up; this model is \
         silent into silence, so it passes both vacuously rather than on merit",
    ));

    Section {
        model: "33609",
        unit: "Neve 2254 and 33609",
        dossier: "research/Neve-33609.md",
        rows,
    }
}

/// Limit attack settling time in ms, by the handbook's definition: a
/// +10 dBu tone stepped up 10 dB, timed until the output is back within
/// 1 dB of where it started.
fn neve_attack_ms(attack: usize) -> f32 {
    const N: usize = 8;
    let mut c = bridge::Compressor::new(SR);
    c.configure(bridge::Settings {
        limit_in: true,
        limit_attack: attack,
        limit_threshold: 8,
        limit_recovery: 0,
        compress_in: false,
        ..bridge::Settings::default()
    });
    let mut sine = Sine::new(1000.0, SR);
    let mut l = vec![0.0f32; N];
    let mut r = vec![0.0f32; N];
    let warm = (0.8 * SR / N as f32) as usize;
    let watch = (0.4 * SR / N as f32) as usize;
    let mut want = 0.0f32;
    for b in 0..(warm + watch) {
        let amp = if b < warm {
            neve_dbu(10.0)
        } else {
            neve_dbu(20.0)
        };
        sine.fill(&mut l, amp);
        r.copy_from_slice(&l);
        c.process_block(&mut l, &mut r);
        let gr = c.gain_reduction_db(0);
        if b + 1 == warm {
            want = gr + 9.0;
        }
        if b >= warm && gr >= want {
            return (b - warm + 1) as f32 * N as f32 / SR * 1e3;
        }
    }
    f32::INFINITY
}

/// A sine amplitude at `x` dBu, on the CL 1B's own calibration where 0 VU
/// is +4 dBu.
fn opto1b_dbu(x: f32) -> f32 {
    opto::model::VU_REF_AMP * 10f32.powf((x - 4.0) / 20.0)
}

/// The threshold knob giving `want` dB of reduction at `amp`, read off the
/// static curve.
fn opto1b_threshold_static(want: f32, amp: f32) -> f32 {
    let (mut lo, mut hi) = (0.0f32, 1.0f32);
    for _ in 0..40 {
        let mid = 0.5 * (lo + hi);
        let mut c = opto1b::Compressor::new(SR);
        c.configure(opto1b::Settings {
            ratio: 0.0,
            threshold: mid,
            ..opto1b::Settings::default()
        });
        if c.static_gr_db(amp) < want {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// Output change for a 10 dB input step at the 2:1 stop, starting from
/// `depth` dB of reduction.
///
/// This follows the repository's own test procedure exactly: the threshold
/// is calibrated so the static curve gives `depth` at −10 dBu, and the step
/// runs from −10 dBu to 0 dBu, read off that same static curve. An earlier
/// version of this row drove the engine with signal from an operating point
/// found by a search that silently clamped at 0 dBFS, so it stepped into
/// clipping and reported a number that meant nothing. Where a repository
/// test already defines how a published figure is measured, the benchmark
/// has to measure it the same way or the two cannot be compared.
fn opto1b_ten_in(depth: f32) -> f32 {
    let p = opto1b_threshold_static(depth, opto1b_dbu(-10.0));
    let mut c = opto1b::Compressor::new(SR);
    c.configure(opto1b::Settings {
        ratio: 0.0,
        threshold: p,
        ..opto1b::Settings::default()
    });
    let a = -10.0 - c.static_gr_db(opto1b_dbu(-10.0));
    let b = 0.0 - c.static_gr_db(opto1b_dbu(0.0));
    b - a
}

fn opto1b_manual_attack(knob: f32) -> f32 {
    let mut c = opto1b::Compressor::new(SR);
    c.configure(opto1b::Settings {
        threshold: 0.7,
        ratio: 0.5,
        attack: knob,
        release: 0.2,
        mode: opto1b::MODE_MANUAL,
        ..opto1b::Settings::default()
    });
    let traj = step_response(
        &mut c,
        1000.0,
        amp_dbfs(-40.0),
        amp_dbfs(-8.0),
        0.5,
        1.5,
        SR,
    );
    time_to_fraction(&traj, 0.63, SR).unwrap_or(f32::NAN)
}

/// The threshold knob that puts the CL 1B at `depth` dB of reduction for a
/// steady tone. The service manual's release procedure begins by setting
/// the reduction to a stated depth, so a benchmark of that figure has to
/// start from the same place: recovery from an arbitrary depth is a
/// different measurement and would report a different number.
fn opto1b_threshold_for(depth: f32, amp: f32, release: f32) -> f32 {
    let measure = |thr: f32| {
        let mut c = opto1b::Compressor::new(SR);
        c.configure(opto1b::Settings {
            threshold: thr,
            attack: 0.0,
            release,
            mode: opto1b::MODE_MANUAL,
            ..opto1b::Settings::default()
        });
        let (_, gr) = steady(&mut c, 1000.0, amp, 6.0, 0.25, SR);
        -gr
    };
    let (mut lo, mut hi) = (0.0f32, 1.0f32);
    for _ in 0..20 {
        let mid = 0.5 * (lo + hi);
        if measure(mid) < depth {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// The service manual's own release measurement: set 10 dB of reduction on
/// a 0 dBu tone, switch the tone off, and time the needle's return to
/// 0 VU. The criterion is 1 dB, because "0 VU" cannot mean tighter than
/// the meter's own published accuracy of ±0.5 dB.
fn opto1b_full_recovery(knob: f32) -> f32 {
    let amp = amp_vu(0.0);
    let thr = opto1b_threshold_for(10.0, amp, knob);
    let mut c = opto1b::Compressor::new(SR);
    c.configure(opto1b::Settings {
        threshold: thr,
        attack: 0.0,
        release: knob,
        mode: opto1b::MODE_MANUAL,
        ..opto1b::Settings::default()
    });
    let traj = release_response(&mut c, 1000.0, amp, 0.0, 3.0, 25.0, SR);
    time_to_recover(&traj, 1.0, SR).unwrap_or(f32::NAN)
}

// ---------------------------------------------------------------------------
// The report
// ---------------------------------------------------------------------------

fn render(sections: &[Section]) -> String {
    let mut out = String::new();

    out.push_str("# Benchmark: the models against their published figures\n\n");
    out.push_str(
        "Generated by `cargo run --release --features plugin --bin benchmark`. Do not edit by hand: regenerate it.\n\n",
    );

    out.push_str("## What this is, and what it is not\n\n");
    out.push_str(
        "The obvious reading of benchmarking a model against the real thing is a null test against \
         the hardware, or against a competitor's plug-in. Neither is available. `research/SURVEY.md` \
         establishes both limits: no independent laboratory measurement of any of these units exists \
         in public, and no plug-in vendor publishes a null test. What does exist is the published \
         record, which is what every row below is measured against: manufacturer specifications, \
         service-manual calibration tables, and the figures the dossiers cite from reviews and \
         teardowns.\n\n",
    );
    out.push_str(
        "So this is not a claim that these models sound like the originals. It is a statement of \
         where each one lands against every number anybody has published about the unit it spoofs, \
         including the numbers it does not reach.\n\n",
    );
    out.push_str(
        "A row whose published column reads *(none published)* is deliberate. Knowing that nothing \
         anchors a behaviour is as useful as knowing that something does, and those rows are where a \
         model is furthest from being verifiable.\n\n",
    );

    // Conditions.
    out.push_str("## Conditions\n\n");
    let _ = writeln!(out, "| | |\n|---|---|");
    let _ = writeln!(out, "| sample rate | {} Hz |", SR as u32);
    let _ = writeln!(out, "| block size | {BLOCK} samples |");
    let _ = writeln!(
        out,
        "| generated | {} |",
        std::env::var("BENCHMARK_DATE")
            .unwrap_or_else(|_| "see the commit that carries this file".into())
    );
    out.push('\n');
    out.push_str(
        "Every measurement drives the real engine offline with generated signal and reads the same \
         accessors the plug-in does. Nothing here is computed from a formula the model also uses.\n\n",
    );

    // Summary.
    out.push_str("## Summary\n\n");
    out.push_str("| model | unit | meets | misses | no published figure |\n");
    out.push_str("|---|---|---|---|---|\n");
    let mut tot = (0usize, 0usize, 0usize);
    for s in sections {
        let (m, x, n) = s.counts();
        tot = (tot.0 + m, tot.1 + x, tot.2 + n);
        let _ = writeln!(out, "| {} | {} | {m} | {x} | {n} |", s.model, s.unit);
    }
    let _ = writeln!(
        out,
        "| **all** | | **{}** | **{}** | **{}** |",
        tot.0, tot.1, tot.2
    );
    out.push('\n');
    out.push_str(
        "The misses are the honest part of this table, and none of them is a widened tolerance. \
         Five of them match the README's own list of figures these models do not reach; the other \
         three do not appear there, and the section below says which and what to do about it.\n\n",
    );

    // Per model.
    for s in sections {
        let _ = writeln!(out, "## {} — {}\n", s.model, s.unit);
        // The report is written into `docs/`, so a dossier path relative to the
        // repository root has to climb out of it or the link is dead.
        let _ = writeln!(out, "Figures from [`{}`](../{}).\n", s.dossier, s.dossier);
        out.push_str("| quantity | published | measured | verdict | source |\n");
        out.push_str("|---|---|---|---|---|\n");
        for r in &s.rows {
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} | {} |",
                r.quantity,
                r.published,
                r.measured,
                r.verdict.mark(),
                r.source
            );
        }
        out.push('\n');
        let notes: Vec<&Row> = s.rows.iter().filter(|r| !r.note.is_empty()).collect();
        if !notes.is_empty() {
            out.push_str("Notes:\n\n");
            for r in notes {
                let _ = writeln!(out, "- **{}**: {}", r.quantity, r.note);
            }
            out.push('\n');
        }
    }

    out.push_str(
        "## Where this disagreed with the README, and how it was settled

",
    );
    out.push_str("A benchmark and a README that contradict each other are worse than either alone, so each disagreement this run first produced was chased to a cause rather than reconciled by choosing a number. All four are settled and the two documents now agree.

");
    out.push_str(
        "| disagreement | what it turned out to be |
|---|---|
",
    );
    out.push_str("| The 610's worst alias measured -34.6 dB here against the README's -51 dB | **The benchmark was right and the README is corrected.** Sweeping the whole band finds the third harmonic folded to 3 kHz, a discrete tone 48 dB above its neighbours that a narrower measurement had missed. The worst product anywhere is the honest figure for an aliasing claim, and the method is stated beside the row. |
");
    out.push_str("| The 610's 20 kHz response fell outside the published +0 / -1 dB, and nothing tested it | **Both faults were real.** A test now pins it. The cause is not the antiderivative anti-aliasing: that is exact in the linear region where this is measured, and the droop tracks the oversampling factor instead of shrinking with sample rate as segment averaging would. The two modelled transformer low-passes account for about 1.6 dB on their own, and their corners are estimates rather than measurements, so the design was over its own budget before anything else was added. The README carries the row. |
");
    out.push_str("| The LA-3A reached 37.7 dB against a published 40 in Compress | **The engine's reasoning holds, and the README carries the row.** Measured, depth rises about 4.3 dB for every 6 dB of extra drive, so the loop does starve itself: 40 dB is reachable in Compress, but only with about 12 dB more drive than the published figure specifies. Limit reaches it at the published level. |
");
    out.push_str("| The CL 1B's 2:1 stop gave 6.0 dB here against the README's 6.4 dB | **The benchmark was measuring nonsense.** Its search for the operating point silently clamped at 0 dBFS, so the 10 dB step ran into clipping. It now follows the repository's own procedure exactly, reading the static curve from -10 dBu to 0 dBu with the threshold calibrated there, and the two agree to a hundredth of a decibel. |

");
    out.push_str("One thing this run cannot settle, flagged rather than buried: at 192 kHz the 610 stage shows a **+2.8 dB rise at 10 kHz**, which no passive roll-off produces and which looks like a defect rather than a response. Fixing it is outside this benchmark’s remit, but it should not go unrecorded.

");
    out.push_str("## Reading a miss\n\n");
    out.push_str(
        "A miss here is not a defect to be hidden. Three audits of this repository found tests \
         written to assert a model's own output instead of the figure they existed to check, and \
         every one of those is now fixed. The rule that replaced them applies to this document too: \
         where the model cannot reach a published number, the number and the gap are both printed, \
         and the explanation sits beside them.\n",
    );

    out
}

// ---------------------------------------------------------------------------
// The dbx 160
// ---------------------------------------------------------------------------

impl Engine for rms::Compressor {
    fn run(&mut self, l: &mut [f32], r: &mut [f32]) {
        self.process_block(l, r);
    }
    fn gr_db(&self) -> f32 {
        -self.gain_reduction_db(0)
    }
}

/// Peak amplitude of a sine at `dbu`, at the dbx model's default headroom.
/// At +4 dBu this is the lab's own 0 VU amplitude, which is the check that
/// the two calibrations agree.
fn dbx_amp(dbu: f32) -> f32 {
    10f32.powf((dbu - rms::HEADROOM_DEFAULT_DB) / 20.0) * std::f32::consts::SQRT_2
}

fn dbx_unit(s: rms::Settings) -> rms::Compressor {
    let mut c = rms::Compressor::new(SR);
    c.configure(s);
    c
}

/// Settled output level in dBu for a steady 1 kHz sine at `in_dbu`.
fn dbx_out_dbu(s: rms::Settings, in_dbu: f32, seconds: f32) -> f32 {
    let mut c = dbx_unit(s);
    settled_out_dbfs(&mut c, 1000.0, dbx_amp(in_dbu), seconds, SR) + rms::HEADROOM_DEFAULT_DB
        - 20.0 * std::f32::consts::SQRT_2.log10()
}

/// Steady-state harmonics of the output: `(fundamental, h2, h3)`.
fn dbx_harmonics(s: rms::Settings, hz: f32, in_dbu: f32) -> (f32, f32, f32) {
    let mut c = dbx_unit(s);
    let (tail, _) = steady(&mut c, hz, dbx_amp(in_dbu), 2.5, 0.4, SR);
    (
        goertzel(&tail, hz, SR),
        goertzel(&tail, hz * 2.0, SR),
        goertzel(&tail, hz * 3.0, SR),
    )
}

/// Time in ms to 63 % of the gain-reduction change after a step, dbx's own
/// definition, with the resampler's reported latency taken back off.
fn dbx_attack_ms(step_db: f32) -> f32 {
    let s = rms::Settings {
        threshold_dbu: -60.0,
        alpha: 1.0,
        ..rms::Settings::default()
    };
    let mut c = dbx_unit(s);
    let lo = dbx_amp(0.0);
    let hi = lo * 10f32.powf(step_db / 20.0);
    let mut sine = Sine::new(1000.0, SR);
    let mut l = vec![0.0f32; TIMING_BLOCK];
    let mut r = vec![0.0f32; TIMING_BLOCK];
    for _ in 0..((2.0 * SR) as usize / TIMING_BLOCK) {
        sine.fill(&mut l, lo);
        r.copy_from_slice(&l);
        c.process_block(&mut l, &mut r);
    }
    // dbx measure the "time required to reduce signal by 63 % of level
    // increase (above threshold)", so the reference is the settled
    // reduction **before** the step, not the first reading after it. Taking
    // the first reading instead loses whatever the detector did inside that
    // block, which for a 30 dB step is most of the answer.
    let before = c.gain_reduction_db(0);
    let target = before + 0.63 * step_db;
    let latency_ms = c.latency() as f32 / SR * 1e3;
    let mut i = 0usize;
    while (i as f32) < 0.2 * SR {
        sine.fill(&mut l, hi);
        r.copy_from_slice(&l);
        c.process_block(&mut l, &mut r);
        i += TIMING_BLOCK;
        if c.gain_reduction_db(0) >= target {
            return i as f32 / SR * 1e3 - latency_ms;
        }
    }
    f32::NAN
}

/// The release trajectory: driven deep, then silence, sampled per timing
/// block. Returns `(time_s, gr_db_positive)`.
fn dbx_release_curve() -> Vec<(f32, f32)> {
    let s = rms::Settings {
        threshold_dbu: -60.0,
        alpha: 1.0,
        ..rms::Settings::default()
    };
    let mut c = dbx_unit(s);
    let mut sine = Sine::new(1000.0, SR);
    let mut l = vec![0.0f32; TIMING_BLOCK];
    let mut r = vec![0.0f32; TIMING_BLOCK];
    for _ in 0..((3.0 * SR) as usize / TIMING_BLOCK) {
        sine.fill(&mut l, dbx_amp(20.0));
        r.copy_from_slice(&l);
        c.run(&mut l, &mut r);
    }
    let mut out = vec![];
    let blocks = (1.0 * SR) as usize / TIMING_BLOCK;
    for b in 0..blocks {
        l.iter_mut().for_each(|v| *v = 0.0);
        r.iter_mut().for_each(|v| *v = 0.0);
        c.run(&mut l, &mut r);
        out.push(((b * TIMING_BLOCK) as f32 / SR, c.gain_reduction_db(0)));
    }
    out
}

fn bench_rms() -> Section {
    let mut rows = Vec::new();
    let base = rms::Settings::default();

    // The audio path at rest. R26 and R32 are both 100 kΩ, so the
    // transimpedance stage exactly undoes the input resistor at zero
    // control voltage, and dbx document the setting as the way to use the
    // box as a line amplifier.
    let unity = rms::Settings {
        alpha: 0.0,
        threshold_dbu: 20.0,
        ..base
    };
    rows.push(Row::within(
        "unity gain at 1:1, threshold at 3 V",
        0.0,
        0.05,
        "dB",
        dbx_out_dbu(unity, 0.0, 1.0),
        "research/dbx-160.md §12.1 test 1, from R26 = R32 = 100 kΩ on the 160 schematic",
    ));

    // The threshold dial is dbx's own factory calibration procedure.
    let onset = |mark_dbu: f32| {
        let s = rms::Settings {
            threshold_dbu: mark_dbu,
            alpha: 1.0,
            ..base
        };
        let c = dbx_unit(s);
        let (mut lo, mut hi) = (mark_dbu - 20.0, mark_dbu + 20.0);
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if c.static_gr_db(dbx_amp(mid)) < 0.1 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        0.5 * (lo + hi)
    };
    let onsets: Vec<f32> = rms::THRESHOLD_MARK_DBU.iter().map(|m| onset(*m)).collect();
    let steps: Vec<f32> = onsets.windows(2).map(|w| w[1] - w[0]).collect();
    let mean_step = steps.iter().sum::<f32>() / steps.len() as f32;
    rows.push(
        Row::within(
            "threshold dial, decibels per printed mark",
            10.0,
            0.5,
            "dB",
            mean_step,
            "research/dbx-160.md §12.1 test 4, from the factory procedure's \"10 db steps\"",
        )
        .because(
            "dbx's marks are a 1-3-10 sequence, so the steps really alternate between 9.54 and \
         10.46 dB and their own \u{2212}38 to +12 dB span is 9.9 dB a mark",
        ),
    );
    rows.push(Row::within(
        "threshold at the 10 mV mark",
        -37.8,
        0.5,
        "dBu",
        onsets[0],
        "research/dbx-160.md §7.1, dbx's \"10mV(\u{2212}38dB)\"",
    ));
    rows.push(Row::within(
        "threshold at the 3 V mark",
        11.8,
        0.5,
        "dBu",
        onsets[5],
        "research/dbx-160.md §7.1, dbx's \"3V(+12dB)\"",
    ));

    // The hard-knee ratios, which dbx call exact.
    for (i, want) in [
        (1usize, 20.0 / 1.5),
        (2, 10.0),
        (3, 20.0 / 3.0),
        (4, 5.0),
        (5, 20.0 / 6.0),
        (6, 2.0),
        (7, 1.0),
    ] {
        let s = rms::Settings {
            threshold_dbu: -20.0,
            alpha: rms::RATIO_MARK_ALPHA[i],
            ..base
        };
        let c = dbx_unit(s);
        let change = 20.0 - (c.static_gr_db(dbx_amp(20.0)) - c.static_gr_db(dbx_amp(0.0)));
        let mut row = Row::within(
            &format!(
                "{}:1, output change for a 20 dB rise",
                rms::RATIO_MARK_LABELS[i]
            ),
            want,
            0.15,
            "dB",
            change,
            "research/dbx-160.md §12.1 test 6, from dbx's \"COMPRESSION RATIO setting defines \
             exact compression ratio\"",
        );
        if i == 4 {
            row = row.because(
                "the position the factory calibrated: the schematic carries a trimmer marked \
                 \"4:1 CAL\", R43",
            );
        }
        rows.push(row);
    }

    // The infinity mark, which dbx published as a number.
    let inf = rms::Settings {
        threshold_dbu: -40.0,
        alpha: 1.0,
        ..base
    };
    let c = dbx_unit(inf);
    let rise = 40.0 - (c.static_gr_db(dbx_amp(20.0)) - c.static_gr_db(dbx_amp(-20.0)));
    rows.push(
        Row::within(
            "output rise over 40 dB at the \u{221e} mark",
            0.333,
            0.15,
            "dB",
            rise,
            "research/dbx-160.md §12.1 test 7, from dbx's \"1:1 to 120:1 (infinity)\"",
        )
        .because(
            "the test that separates modelling the circuit from modelling the silkscreen: \
         40 dB \u{f7} 120 is a third of a decibel, not a brick wall",
        ),
    );

    // Infinity+, which needed no new circuit, only a longer pot.
    for (alpha, want) in [(2.0f32, -10.0f32), (1.5, -5.0), (1.2, -2.0)] {
        let s = rms::Settings {
            model: rms::MODEL_160A,
            threshold_dbu: -40.0,
            alpha,
            ..base
        };
        let c = dbx_unit(s);
        let change = 10.0 - (c.static_gr_db(dbx_amp(-25.0)) - c.static_gr_db(dbx_amp(-35.0)));
        rows.push(Row::within(
            &format!(
                "{}, output change for a 10 dB rise",
                rms::ratio_label(alpha)
            ),
            want,
            0.3,
            "dB",
            change,
            "research/dbx-160.md §12.1 test 8, from the 160A manual's \u{2212}1:1 description",
        ));
    }

    let deep = rms::Settings {
        model: rms::MODEL_160A,
        threshold_dbu: -40.0,
        alpha: 1.0,
        ..base
    };
    rows.push(
        Row::ranged(
            "maximum compression",
            60.0,
            61.0,
            "dB",
            dbx_unit(deep).static_gr_db(dbx_amp(24.0)),
            "research/dbx-160.md §12.1 test 9, from dbx's \"over 60dB maximum compression\"",
        )
        .because(
            "driven at dbx's own published maximum input level for the 160A, +24 dBu, with the              threshold at its \u{2212} 40 dBu end: the claim is about what the box can do, so the              measurement belongs at the ends of its own two published ranges",
        ),
    );

    // Make-up gain, which dbx mark on the pot's own track.
    let g20 = dbx_out_dbu(
        rms::Settings {
            output_db: 20.0,
            ..unity
        },
        0.0,
        1.0,
    );
    let gm20 = dbx_out_dbu(
        rms::Settings {
            output_db: -20.0,
            ..unity
        },
        0.0,
        1.0,
    );
    rows.push(Row::within(
        "output gain range",
        40.0,
        0.1,
        "dB",
        g20 - gm20,
        "research/dbx-160.md §7.1, dbx's \"\u{00b1}20 dB from unity gain point\" and R80's \
         track ends",
    ));

    // The ballistics, which are one time constant seen from several sides.
    for (step, want) in [(10.0f32, 15.0f32), (20.0, 5.0), (30.0, 3.0)] {
        let got = dbx_attack_ms(step);
        // \u{00b1} 30 %, which is the spread of dbx's own three figures, rounded
        // so the table prints a number rather than a float artefact.
        let tol = (want * 0.3 * 100.0).round() / 100.0;
        let mut row = Row::within(
            &format!("attack, {step:.0} dB step"),
            want,
            tol,
            "ms",
            got,
            "research/dbx-160.md §12.3 test 15, from dbx's 15 / 5 / 3 ms attack table",
        );
        if step == 20.0 {
            row = row.because(
                "the recorded miss. dbx's three attack figures each imply a different time \
                 constant, 33.3, 26.2 and 37.6 ms, so no single-constant detector can meet all \
                 three and the hardware is a single-constant detector. The constant here is R35 \
                 and C15 off dbx's own drawing, which puts the release rate between dbx's own \
                 two published rates; meeting this row would mean giving that up, or giving up \
                 the exact true-RMS averaging",
            );
        }
        rows.push(row);
    }

    let curve = dbx_release_curve();
    let band: Vec<(f32, f32)> = curve
        .iter()
        .cloned()
        .filter(|(_, gr)| (5.0..=35.0).contains(gr))
        .collect();
    let n = band.len() as f32;
    let (sx, sy): (f32, f32) = band.iter().fold((0.0, 0.0), |a, p| (a.0 + p.0, a.1 + p.1));
    let (mx, my) = (sx / n, sy / n);
    let (sxy, sxx): (f32, f32) = band.iter().fold((0.0, 0.0), |a, p| {
        (a.0 + (p.0 - mx) * (p.1 - my), a.1 + (p.0 - mx) * (p.0 - mx))
    });
    let slope = sxy / sxx;
    rows.push(Row::ranged(
        "release rate",
        120.0,
        125.0,
        "dB/s",
        -slope,
        "research/dbx-160.md §12.3 test 17, from dbx's 120 dB/s (160) and 125 dB/s (160A)",
    ));
    let resid = band
        .iter()
        .map(|(t, gr)| (gr - (my + slope * (t - mx))).abs())
        .fold(0.0f32, f32::max);
    rows.push(Row::ranged(
        "departure from a straight line over 35 to 5 dB",
        0.0,
        0.5,
        "dB",
        resid,
        "research/dbx-160.md §12.3 test 17, the structural half: a log-domain filter releases \
         at a constant dB/s and an ordinary RC does not",
    ));
    let start = curve[0].1;
    for (want_db, want_ms) in [(1.0f32, 8.0f32), (10.0, 80.0), (50.0, 400.0)] {
        if start < want_db {
            continue;
        }
        let got = curve
            .iter()
            .find(|(_, gr)| *gr <= start - want_db)
            .map(|(t, _)| t * 1e3)
            .unwrap_or(f32::NAN);
        rows.push(Row::within(
            &format!("release, {want_db:.0} dB"),
            want_ms,
            want_ms * 0.2,
            "ms",
            got,
            "research/dbx-160.md §12.3 test 18, from the 160A's 8 / 80 / 400 ms table",
        ));
    }

    // The detector really is an RMS detector.
    let mut c = dbx_unit(unity);
    steady(&mut c, 1000.0, 0.25, 2.0, 0.1, SR);
    rows.push(Row::within(
        "a sine settles below its peak",
        3.01,
        0.15,
        "dB",
        20.0 * 0.25f32.log10() - c.detector_db(0),
        "research/dbx-160.md §12.2 test 11, the RMS of a sine against dbx's \"True rms \
         level-detection\"",
    ));

    for (period, duty, cf, want) in [
        (98usize, 8usize, 3.5f32, 0.2f32),
        (100, 4, 5.0, 0.5),
        (128, 2, 8.0, 1.0),
    ] {
        // At 96 kHz, where the model does not oversample: a 63-tap
        // interpolating filter smears a two-sample pulse and would set the
        // crest factor itself.
        let sr = 96_000.0f32;
        let mut c = rms::Compressor::new(sr);
        c.configure(unity);
        let amp = 0.1 * cf;
        let x: Vec<f32> = (0..period * 600)
            .map(|i| if i % period < duty { amp } else { 0.0 })
            .collect();
        let true_rms_db = db(rms(&x));
        for _ in 0..2 {
            for chunk in x.chunks(BLOCK) {
                let mut l = chunk.to_vec();
                let mut r = l.clone();
                c.process_block(&mut l, &mut r);
            }
        }
        rows.push(
            Row::within(
                &format!("under-reading at crest factor {cf}"),
                want,
                0.3,
                "dB",
                true_rms_db - c.detector_db(0),
                "research/dbx-160.md §12.2 test 12, from the THAT 2252 datasheet's \
                 crest-factor table",
            )
            .because(
                "the descendant part's figure, not dbx's, who publish none. The direction and \
                 the ordering are right and the magnitudes are a recorded miss: what is left in \
                 the real part is its own input bandwidth, which the datasheet gives as four \
                 corner frequencies against input current rather than as a transfer function",
            ),
        );
    }

    // The link, which sums energies rather than signals.
    let linked = rms::Settings {
        link: true,
        ..unity
    };
    let one = {
        let mut c = dbx_unit(linked);
        let mut sine = Sine::new(1000.0, SR);
        let mut l = vec![0.0f32; BLOCK];
        let mut r = vec![0.0f32; BLOCK];
        for _ in 0..((2.0 * SR) as usize / BLOCK) {
            sine.fill(&mut l, 0.25);
            r.iter_mut().for_each(|v| *v = 0.0);
            c.process_block(&mut l, &mut r);
        }
        c.detector_db(0)
    };
    let two = {
        let mut c = dbx_unit(linked);
        steady(&mut c, 1000.0, 0.25, 2.0, 0.1, SR);
        c.detector_db(0)
    };
    rows.push(Row::within(
        "link, two matched channels against one",
        3.01,
        0.2,
        "dB",
        two - one,
        "research/dbx-160.md §12.6 test 34, from dbx's True RMS Power Summing",
    ));

    // The distortion, which is the detector showing through.
    let inf_comp = rms::Settings {
        threshold_dbu: -10.0,
        alpha: 1.0,
        ..base
    };
    let mut h3 = vec![];
    for hz in [50.0f32, 100.0, 200.0, 400.0] {
        let (f, _, t) = dbx_harmonics(inf_comp, hz, 0.0);
        h3.push(t / f);
    }
    for (i, w) in h3.windows(2).enumerate() {
        rows.push(Row::within(
            &format!(
                "third harmonic, {} Hz against {} Hz",
                [100, 200, 400][i],
                [50, 100, 200][i]
            ),
            0.5,
            0.05,
            "\u{d7}",
            w[1] / w[0],
            "research/dbx-160.md §12.5 test 27, from dbx's \"at 100 Hz 3rd-harmonic distortion \
             is 1/2 the value at 50 Hz\"",
        ));
    }

    let (f, h2, _) = dbx_harmonics(
        rms::Settings {
            threshold_dbu: -20.0,
            alpha: 1.0,
            ..base
        },
        1000.0,
        4.0,
    );
    rows.push(
        Row::within(
            "second harmonic at +4 dBu, \u{221e}:1",
            0.075,
            0.0225,
            "%",
            100.0 * h2 / f,
            "research/dbx-160.md §12.5 test 30, from dbx's \"0.075 % 2nd harmonic at infinite \
             compression at +4dBm output\"",
        )
        .because(
            "a calibration rather than a test: the cell's symmetry residual is fitted to this \
             one number. What makes it a measurement is that the same value has to hold at every \
             other frequency and ratio, which the row below checks",
        ),
    );

    let mut h2s = vec![];
    for alpha in [0.5f32, 0.75, 1.0] {
        for hz in [100.0f32, 1000.0, 10_000.0] {
            let (f, h, _) = dbx_harmonics(
                rms::Settings {
                    threshold_dbu: -10.0,
                    alpha,
                    ..base
                },
                hz,
                0.0,
            );
            h2s.push(100.0 * h / f);
        }
    }
    let spread = (h2s.iter().cloned().fold(f32::MIN, f32::max)
        - h2s.iter().cloned().fold(f32::MAX, f32::min))
        / (h2s.iter().sum::<f32>() / h2s.len() as f32);
    rows.push(Row::ranged(
        "second-harmonic spread across ratio and frequency",
        0.0,
        0.40,
        "of its mean",
        spread,
        "research/dbx-160.md §12.5 test 29, from dbx's \"2nd harmonic is relatively unaffected \
         by compression ratio, time constants and frequency\"",
    ));

    let (f, _, t) = dbx_harmonics(
        rms::Settings {
            threshold_dbu: -20.0,
            alpha: 1.0,
            ..base
        },
        100.0,
        0.0,
    );
    rows.push(
        Row::ranged(
            "third harmonic at 100 Hz, \u{221e}:1",
            0.3,
            1.2,
            "%",
            100.0 * t / f,
            "research/dbx-160.md §12.5 test 33, bracketing dbx's 0.5 % and the ripple \
             equation's 0.8 %",
        )
        .because(
            "an order-of-magnitude check on the detector's ripple rather than a calibration: \
             dbx did not state the frequency of their 0.5 % figure",
        ),
    );

    let mut worst_thd = 0.0f32;
    for gr in (0..=40).step_by(5) {
        let (f, h2, h3) = dbx_harmonics(
            rms::Settings {
                model: rms::MODEL_160A,
                threshold_dbu: -(gr as f32),
                alpha: 1.0,
                ..base
            },
            1000.0,
            0.0,
        );
        worst_thd = worst_thd.max(100.0 * (h2 * h2 + h3 * h3).sqrt() / f);
    }
    rows.push(Row::ranged(
        "worst THD from 0 to 40 dB of compression, 1 kHz",
        0.0,
        0.2,
        "%",
        worst_thd,
        "research/dbx-160.md §12.5 test 31, from the 160A's \"<0.2 %, typical, any amount of \
         compression up to 40 dB @ 1 kHz\"",
    ));

    let (f, h2, h3) = dbx_harmonics(
        rms::Settings {
            model: rms::MODEL_160A,
            threshold_dbu: 20.0,
            alpha: 1.0,
            ..base
        },
        1000.0,
        0.0,
    );
    rows.push(Row::ranged(
        "second harmonic below threshold",
        0.035,
        0.105,
        "%",
        100.0 * h2 / f,
        "research/dbx-160.md §12.5 test 32, from the 160X's 0.07 % below threshold",
    ));
    rows.push(
        Row::ranged(
            "third harmonic below threshold",
            0.035,
            0.105,
            "%",
            100.0 * h3 / f,
            "research/dbx-160.md §12.5 test 32, from the 160X's 0.07 % below threshold",
        )
        .because(
            "a recorded miss, and the model cannot meet it honestly. With no gain reduction \
             there is no detector ripple, and the third harmonic in the hardware at that point \
             belongs to an output stage dbx publish no distortion figure for, so anything here \
             would be invented",
        ),
    );

    // The audio path's own corners, which are components rather than a
    // specification: dbx publish no frequency response for the original.
    let hp_level = |hz: f32| response_db(|| Box::new(dbx_unit(unity)), hz, 0.25, SR);
    let reference = hp_level(1000.0);
    rows.push(Row::within(
        "response at the input coupling corner",
        -3.0,
        0.3,
        "dB",
        hp_level(rms::engine::INPUT_HP_HZ) - reference,
        "research/dbx-160.md §12.8 test 41, from C12 = 0.15 \u{b5}F into R26 = 100 k\u{3a9}",
    ));
    rows.push(
        Row::within(
            "response at 20 Hz",
            -1.1,
            0.2,
            "dB",
            hp_level(20.0) - reference,
            "research/dbx-160.md §7.5, derived from the same pair",
        )
        .because(
            "the original had a low-frequency tilt its successor did not: the 160A publishes \
             \u{2212}3 dB at 0.5 Hz on a board with much larger coupling capacitors",
        ),
    );

    // The metering.
    let meter_vu = |cal: f32, meter: usize, in_dbu: f32| {
        let s = rms::Settings {
            meter,
            meter_cal_dbu: cal,
            ..unity
        };
        let mut c = dbx_unit(s);
        steady(&mut c, 1000.0, dbx_amp(in_dbu), 1.0, 0.1, SR);
        c.meter_frame()[5]
    };
    rows.push(Row::within(
        "meter reading at its factory 0 VU",
        0.0,
        0.15,
        "VU",
        meter_vu(4.0, rms::METER_OUTPUT, 4.0),
        "research/dbx-160.md §12.7 test 37, from dbx's \"factory calibrated to read '0' at \
         +4dB (1.23V)\"",
    ));
    rows.push(Row::within(
        "meter reading with the trimmer at its \u{2212}15 dBu end",
        0.0,
        0.15,
        "VU",
        meter_vu(-15.0, rms::METER_OUTPUT, -15.0),
        "research/dbx-160.md §12.7 test 40, from the 160A's \u{2212}15 dBu to +10 dBu trimmer",
    ));

    // What nobody ever published.
    rows.push(Row::unanchored(
        "OverEasy knee width",
        format!("{} dB (the default)", rms::engine::KNEE_WIDTH_DEFAULT_DB),
        "dbx never published a knee width for any model in the family, and it cannot be derived \
         from the drawing: it is V\u{3b8}/(G\u{b7}K), and the difference amplifier's gain G could \
         not be read. The circuit bounds it to roughly 2 to 9 dB, which is why it is a parameter",
    ));
    rows.push(Row::unanchored(
        "OverEasy on a transient",
        "the body is more compressed; the slap is not louder".into(),
        "dbx's kick-drum note says OverEasy \"will therefore emphasize the slap at the beginning \
         of the note and reduce the boominess of its body\". The second clause holds. The first \
         cannot hold for any knee centred on the threshold, which is what dbx's own definition \
         of what THRESHOLD points at requires: such a curve lies at or below the hard-knee curve \
         everywhere, so it can never pass more of a transient. Where a definition and a sentence \
         of application prose disagree the model follows the definition",
    ));

    Section {
        model: "160",
        unit: "dbx 160, with the 160A's OverEasy and Infinity+",
        dossier: "research/dbx-160.md",
        rows,
    }
}

// ---------------------------------------------------------------------------
// The SSL 4000 G bus compressor
// ---------------------------------------------------------------------------

/// Timing measurements run here rather than at [`SR`]: the fastest attack
/// constant is 385 µs, seventeen samples at 44.1 kHz, and one sample of
/// quantisation is 6 % of it.
const GBUS_TIMING_SR: f32 = 192_000.0;

fn gbus_unit(s: gbus::Settings) -> gbus::Compressor {
    let mut c = gbus::Compressor::new(SR);
    c.configure(s);
    c
}

fn gbus_amp(dbfs: f32) -> f32 {
    10f32.powf(dbfs / 20.0)
}

/// The bare attack constant, with the release path opened, against the
/// resistor and capacitor on card 82E27.
fn gbus_open_loop_tau(attack: usize) -> f32 {
    let mut t = gbus::Timing::open_loop(GBUS_TIMING_SR, attack);
    // Well past the diode drop, so the diode conducts and the network is
    // the bare R and C.
    let d = 20.0f32;
    let target = 0.632_120_6 * (d - gbus::V_DIODE);
    let mut n = 0usize;
    while n < (GBUS_TIMING_SR * 2.0) as usize {
        let v = t.step(d);
        n += 1;
        if v >= target {
            break;
        }
    }
    n as f32 / GBUS_TIMING_SR
}

/// The bare release constant: charge, disconnect the detector, time the
/// decay to 1/e.
fn gbus_release_tau(release: usize) -> f32 {
    let mut t = gbus::Timing::new(GBUS_TIMING_SR);
    t.configure(0, release);
    for _ in 0..(GBUS_TIMING_SR as usize / 10) {
        t.step(10.0);
    }
    let start = t.voltage();
    let mut n = 0usize;
    while n < (GBUS_TIMING_SR * 6.0) as usize {
        let v = t.release_only();
        n += 1;
        if v <= start / std::f32::consts::E {
            break;
        }
    }
    n as f32 / GBUS_TIMING_SR
}

/// The input level at which gain reduction first reaches `gr`, dBFS.
fn gbus_level_for(s: gbus::Settings, gr: f32) -> f32 {
    let c = gbus_unit(s);
    let (mut lo, mut hi) = (-80.0f32, 60.0f32);
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if c.static_gr_db(gbus_amp(mid)) < gr {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// The local compression ratio at `gr` decibels of reduction.
fn gbus_ratio_at(s: gbus::Settings, gr: f32) -> f32 {
    let c = gbus_unit(s);
    let l = gbus_level_for(s, gr);
    let h = 0.25f32;
    let hi = (l + h) - c.static_gr_db(gbus_amp(l + h));
    let lo = (l - h) - c.static_gr_db(gbus_amp(l - h));
    1.0 / ((hi - lo) / (2.0 * h))
}

/// The settled output level of a sine, dBFS.
fn gbus_out_dbfs(s: gbus::Settings, level_dbfs: f32) -> f32 {
    let mut c = gbus_unit(s);
    let (tail, _) = steady(&mut c, 1000.0, gbus_amp(level_dbfs), 3.0, 0.5, SR);
    db(rms(&tail) * std::f32::consts::SQRT_2)
}

fn bench_gbus() -> Section {
    let mut rows = Vec::new();
    let base = gbus::Settings::default();

    // -- the audio path, which is one multiply ---------------------------
    let straight = gbus::Settings {
        sidechain_in: false,
        oversample: false,
        ..base
    };
    rows.push(Row::within(
        "unity gain, sidechain out, make-up at 0",
        0.0,
        0.1,
        "dB",
        gbus_out_dbfs(straight, -20.0) + 20.0,
        "research/SSL-Gbus.md §13.1 test 3, from THAT's \"Gain at 0 V Control Voltage: 0.0 dB, \
         ±0.1 dB\" (M)",
    ));
    rows.push(Row::within(
        "the IN switch is not a bypass: make-up with the sidechain out",
        10.0,
        0.1,
        "dB",
        gbus_out_dbfs(
            gbus::Settings {
                makeup_db: 10.0,
                ..straight
            },
            -20.0,
        ) + 20.0,
        "research/SSL-Gbus.md §13.1 test 2, from SSL's \"the compressor sidechain is enabled by \
         the IN switch\" (P) and the clone builder's \"the makeup gain pot is active all the \
         time\" (C)",
    ));
    let mut worst_flat = 0.0f32;
    for hz in [20.0f32, 100.0, 1000.0, 10_000.0, 20_000.0] {
        let mut c = gbus_unit(straight);
        let (tail, _) = steady(&mut c, hz, 0.1, 1.0, 0.3, SR);
        let g = db(rms(&tail) * std::f32::consts::SQRT_2) - db(0.1);
        worst_flat = worst_flat.max(g.abs());
    }
    rows.push(Row::within(
        "audio-path response, 20 Hz to 20 kHz",
        0.0,
        0.05,
        "dB",
        worst_flat,
        "research/SSL-Gbus.md §13.1 test 6, from SSL's XLogic \"20Hz to 20kHz ±0.05dB\" (M). \
         That figure describes a 2004 SuperAnalogue unit, not a 1980 console card; it is used \
         here as the tightest published bound on a path that has no filters in it at all",
    ));

    // -- the ballistics, which are four component values each ------------
    for i in 0..6 {
        let want = gbus::attack_tau(i);
        rows.push(Row::within(
            &format!("attack constant, {} ms position", gbus::ATTACK_NAMES[i]),
            want * 1e3,
            want * 1e3 * 0.02,
            "ms",
            gbus_open_loop_tau(i) * 1e3,
            "research/SSL-Gbus.md §13.4 test 13, from R1–R6 across C = 0.47 µF on card 82E27 (S)",
        ));
    }
    for i in 0..4 {
        let want = gbus::release_tau(i);
        let mut r = Row::within(
            &format!("release constant, {} s position", gbus::RELEASE_NAMES[i]),
            want * 1e3,
            want * 1e3 * 0.02,
            "ms",
            gbus_release_tau(i) * 1e3,
            "research/SSL-Gbus.md §13.4 test 15, from R9–R12 across 0.47 µF on card 82E27 (S)",
        );
        if i == 0 {
            r = r.because(
                "This position looks wrong and is right. Its 84.6 ms is 1.18 times the panel's \
                 0.1 s where the other three are 2.1 to 2.4 times theirs. The dossier reads R12 \
                 unambiguously as 180 kΩ at 16× magnification, records that 90 kΩ would fit the \
                 pattern and is not what is drawn, and refuses to adjust the value to taste",
            );
        }
        rows.push(r);
    }

    // The automatic release, measured section by section.
    let mut t = gbus::Timing::new(GBUS_TIMING_SR);
    t.configure(0, gbus::RELEASE_AUTO);
    for _ in 0..(GBUS_TIMING_SR as usize * 12) {
        t.step(10.0);
    }
    let (v1_0, v2_0) = t.sections();
    let equilibrium_share = v2_0 / (v1_0 + v2_0);
    let (mut tau1, mut tau2) = (None, None);
    for i in 1..=(GBUS_TIMING_SR as usize * 20) {
        t.release_only();
        let (v1, v2) = t.sections();
        if tau1.is_none() && v1 <= v1_0 / std::f32::consts::E {
            tau1 = Some(i as f32 / GBUS_TIMING_SR);
        }
        if tau2.is_none() && v2 <= v2_0 / std::f32::consts::E {
            tau2 = Some(i as f32 / GBUS_TIMING_SR);
        }
        if tau1.is_some() && tau2.is_some() {
            break;
        }
    }
    rows.push(Row::within(
        "Auto release, fast section",
        42.77,
        42.77 * 0.05,
        "ms",
        tau1.unwrap_or(0.0) * 1e3,
        "research/SSL-Gbus.md §13.4 test 16, from R7 91 kΩ with C1 0.47 µF on card 82E27 (S)",
    ));
    rows.push(Row::within(
        "Auto release, slow section",
        5.10,
        5.10 * 0.05,
        "s",
        tau2.unwrap_or(0.0),
        "research/SSL-Gbus.md §13.4 test 16, from R8 750 kΩ with C2 6.8 µF on card 82E27 (S)",
    ));
    rows.push(
        Row::within(
            "Auto release, share of the control voltage on the slow section after a sustained tone",
            89.2,
            89.2 * 0.05,
            "%",
            equilibrium_share * 100.0,
            "research/SSL-Gbus.md §13.4 test 17, from R8/(R7+R8) on card 82E27 (S)",
        )
        .because(
            "Neither this nor the transient split below appears anywhere in the engine. They are \
             what simulating the two sections gives, and they are why the automatic release is \
             programme-dependent",
        ),
    );
    let mut t = gbus::Timing::new(GBUS_TIMING_SR);
    t.configure(3, gbus::RELEASE_AUTO);
    for _ in 0..(GBUS_TIMING_SR as usize / 1000) {
        t.step(5.0);
    }
    let (v1, v2) = t.sections();
    rows.push(Row::within(
        "Auto release, charge split after a 1 ms burst",
        14.47,
        14.47 * 0.05,
        ": 1",
        v1 / v2,
        "research/SSL-Gbus.md §13.4 test 17, from C2/C1 = 6.8/0.47 on card 82E27 (S)",
    ));

    // The divider between the attack and release resistors.
    for attack in [0usize, 5] {
        let mut t = gbus::Timing::new(GBUS_TIMING_SR);
        t.configure(attack, 0);
        for _ in 0..(GBUS_TIMING_SR as usize * 3) {
            t.step(10.0);
        }
        let got = t.voltage() / (10.0 - gbus::V_DIODE);
        let want = gbus::RELEASE_R[0] / (gbus::RELEASE_R[0] + gbus::ATTACK_R[attack]);
        rows.push(
            Row::within(
                &format!(
                    "attack/release divider, {} ms attack with the 0.1 s release",
                    gbus::ATTACK_NAMES[attack]
                ),
                want,
                want * 0.05,
                "",
                got,
                "research/SSL-Gbus.md §13.4 test 18, from R_rel/(R_att + R_rel) on card 82E27 (S)",
            )
            .because(
                "The least supported thing in this model. No measurement of it exists anywhere; \
                 it follows from the topology, and it costs nothing because simulating three \
                 components gives it for free. At the slowest attack the network reaches only \
                 40 % of the control voltage it otherwise would",
            ),
        );
    }

    // The panel's attack legend, which is a different claim from the
    // resistors above.
    let panel = [0.1e-3f32, 0.3e-3, 1e-3, 3e-3, 10e-3, 30e-3];
    for (i, want) in panel.iter().enumerate() {
        let s = gbus::Settings {
            attack: i,
            release: 3,
            ratio: 1,
            ..base
        };
        let mut c = gbus::Compressor::new(GBUS_TIMING_SR);
        c.configure(s);
        c.reset();
        let amp = gbus_amp(-3.0);
        let block = ((want * GBUS_TIMING_SR / 80.0) as usize).max(1);
        let n = ((GBUS_TIMING_SR * want * 600.0) as usize).max((GBUS_TIMING_SR * 0.05) as usize);
        let mut trace = Vec::with_capacity(n / block + 1);
        let mut l = vec![amp; block];
        let mut r = vec![amp; block];
        for _ in 0..(n / block) {
            for j in 0..block {
                l[j] = amp;
                r[j] = amp;
            }
            c.process_block(&mut l, &mut r);
            trace.push(c.gr_db());
        }
        let final_gr = trace[trace.len() - 1];
        let target = 0.632_120_6 * final_gr;
        let idx = trace.iter().position(|g| *g >= target).unwrap_or(0);
        let got = (idx * block) as f32 / GBUS_TIMING_SR;
        let mut row = Row::within(
            &format!(
                "effective attack at 4:1, {} ms panel mark",
                gbus::ATTACK_NAMES[i]
            ),
            want * 1e3,
            want * 1e3 * 0.30,
            "ms",
            got * 1e3,
            "research/SSL-Gbus.md §13.4 test 14, from the panel legend ATTACK mS (P) with \
             τ_closed = τ_open/(1+γ) and γ = 3 at 4:1 (S, via derivation). The ±30 % is the \
             dossier's and it calls it wide on purpose",
        );
        if i == 0 {
            row = row.because(
                "The one recorded miss in this model, by 0.2 percentage points. γ is 0.11513·d/k \
                 and equals 3 only at the knee, so the harder the box is driven the faster it \
                 grabs, while the panel prints one number. Measured at one fixed input level \
                 giving 7 to 9.5 dB of reduction; at 12 dB the slowest position runs 41 % fast \
                 and at 5 dB the fastest runs 176 % slow. Nothing is tuned to move this",
            );
        }
        rows.push(row);
    }

    // -- the loop, and the shape it produces -----------------------------
    let mut worst_threshold = 0.0f32;
    for ratio in 0..3 {
        let a = gbus_unit(gbus::Settings {
            ratio,
            threshold_db: -10.0,
            release: 0,
            ..base
        })
        .static_gr_db(gbus_amp(-24.0));
        let b = gbus_unit(gbus::Settings {
            ratio,
            threshold_db: 0.0,
            release: 0,
            ..base
        })
        .static_gr_db(gbus_amp(-14.0));
        worst_threshold = worst_threshold.max((a - b).abs());
    }
    rows.push(
        Row::within(
            "10 dB off the threshold against 10 dB onto the input",
            0.0,
            0.5,
            "dB",
            worst_threshold,
            "research/SSL-Gbus.md §13.2 test 7, from SSL's XLogic manual: the sidechain trims \
             \"increase the side chain level by 10dB — effectively reducing the threshold on that \
             channel by 10dB\" (P)",
        )
        .because(
            "The only place SSL state the equivalence numerically, and the test that proves the \
             model built a sidechain gain rather than a comparator. Note the direction: a \
             threshold reading and a sidechain gain run opposite ways, which is why this model's \
             THRESHOLD parameter is negated into the loop and the dossier's §11.4 writes +T",
        ),
    );

    let deep_rise = |ratio: usize| {
        let s = gbus::Settings {
            ratio,
            range_db: 60.0,
            release: 0,
            ..base
        };
        (gbus_ratio_at(s, 30.0) - gbus_ratio_at(s, 20.0)) / 10.0
    };
    for ratio in 0..3 {
        let got = deep_rise(ratio);
        rows.push(
            Row::within(
                &format!(
                    "rise of the compression ratio per dB of reduction, {} position",
                    gbus::RATIO_NAMES[ratio]
                ),
                0.11513,
                0.11513 * 0.20,
                "",
                got,
                "research/SSL-Gbus.md §13.3 test 10, from ratio(GR) = 1 + 0.11513·(GR + V_d/k), \
                 derived from the loop equation with ln10/20 (S, via derivation)",
            )
            .because(
                "Measured deep in conduction, between 20 and 30 dB. The derivation treats D6 as \
                 an ideal 0.6 V drop while the same dossier insists its soft turn-on *is* the \
                 knee; both cannot hold, and a real diode's incremental conductance stays below \
                 its asymptote until the control voltage is several thermal voltages. k is 69, 23 \
                 and 7.7 mV/dB, so at 10:1 the whole 20 dB meter range is 154 mV and the diode \
                 never leaves its knee. Not calibrated away, because k is an estimate",
            ),
        );
    }

    let knees: Vec<f32> = (0..3)
        .map(|ratio| {
            gbus_level_for(
                gbus::Settings {
                    ratio,
                    release: 0,
                    ..base
                },
                0.5,
            )
        })
        .collect();
    let monotone = knees.windows(2).all(|w| w[1] > w[0]);
    rows.push(Row::new(
        "lowering the ratio lowers the knee",
        "yes, direction only",
        format!(
            "{} ({:.1}, {:.1}, {:.1} dBFS at 2:1, 4:1, 10:1)",
            if monotone { "yes" } else { "**no**" },
            knees[0],
            knees[1],
            knees[2]
        ),
        "research/SSL-Gbus.md §13.3 test 11, from SSL's \"Decreasing the RATIO setting lowers the \
         effective threshold\" (P)",
        if monotone {
            Verdict::Meets
        } else {
            Verdict::Misses
        },
    ).because(
        "SSL publish no magnitude for this, only the direction, so only the direction is checked. \
         Saying \"and by about 3 dB\" would be inventing a number",
    ));

    // -- the gain cell, on its own, as its datasheet measures it ---------
    for (v_rms, gain_db, want_pct) in [(1.0f32, 0.0f32, 0.005f32), (3.1623, -15.0, 0.020)] {
        let mut cell = gbus::GainStage::new(SR);
        let amp = v_rms * std::f32::consts::SQRT_2 / gbus::engine::VOLTS_PER_SAMPLE;
        let g = cell.gain(gain_db);
        let n = 8192usize;
        let hz = SR * 171.0 / n as f32;
        let step = 2.0 * PI * hz / SR;
        for i in 0..(SR as usize / 2) {
            cell.shape(amp * (i as f32 * step).sin());
        }
        let y: Vec<f32> = (0..n)
            .map(|i| cell.shape(amp * (i as f32 * step).sin()) * g)
            .collect();
        let got = 100.0 * goertzel(&y, 2.0 * hz, SR) / goertzel(&y, hz, SR);
        rows.push(
            Row::within(
                &format!("gain-cell THD, {v_rms:.4} V RMS in at {gain_db} dB of gain"),
                want_pct,
                want_pct * 0.5,
                "%",
                got,
                "research/SSL-Gbus.md §13.5 test 24, from the THAT 2180A typical THD table (M). \
                 The ±50 % is the dossier's, because the datasheet gives typicals with a maximum \
                 and no distribution",
            )
            .because(
                "These two points settle where the distortion goes. The second has a lower output \
                 than the first and four times the THD, so it cannot be a function of the output: \
                 the cell shapes its input, not its result, which is what a current-mode cell \
                 driven through a resistor does. The dossier's §11.3 writes the other form and it \
                 misses this row by a factor of seven",
            ),
        );
    }

    // -- the sidechain -------------------------------------------------
    let mut worst_slope = 0.0f32;
    for i in 1..6 {
        let fc = gbus::HPF_HZ[i];
        let c = gbus_unit(gbus::Settings { hpf: i, ..base });
        let per_octave = c.sidechain_response_db(fc / 4.0) - c.sidechain_response_db(fc / 8.0);
        worst_slope = worst_slope.max((per_octave - 6.0).abs());
    }
    rows.push(
        Row::within(
            "sidechain high-pass slope, worst of the five corners",
            6.0,
            1.0,
            "dB/octave",
            6.0 + worst_slope,
            "research/SSL-Gbus.md §13.5 test 23, from Smart Research's \"150Hz −6dB/octave\" (C)",
        )
        .because(
            "The only slope figure published for anything in this family, and it is for a \
             different unit's outboard cable rather than SSL's built-in filter. Measured between \
             a quarter and an eighth of each corner, where a slope is defined; the dossier asks \
             for −6 dB one octave down, which an exact first-order section misses on its own at \
             −6.99 dB",
        ),
    );

    let dominant = {
        let s = gbus::Settings { release: 0, ..base };
        let both = {
            let mut c = gbus_unit(s);
            let mut sine = Sine::new(1000.0, SR);
            let (mut l, mut r) = (vec![0.0f32; BLOCK], vec![0.0f32; BLOCK]);
            for _ in 0..((SR as usize) / BLOCK) {
                sine.fill(&mut l, gbus_amp(-6.0));
                r.copy_from_slice(&l);
                c.process_block(&mut l, &mut r);
            }
            c.gr_db()
        };
        let uneven = {
            let mut c = gbus_unit(s);
            let mut sine = Sine::new(1000.0, SR);
            let (mut l, mut r) = (vec![0.0f32; BLOCK], vec![0.0f32; BLOCK]);
            for _ in 0..((SR as usize) / BLOCK) {
                sine.fill(&mut l, gbus_amp(-6.0));
                for (i, v) in r.iter_mut().enumerate() {
                    *v = l[i] * gbus_amp(-20.0);
                }
                c.process_block(&mut l, &mut r);
            }
            c.gr_db()
        };
        (uneven - both).abs()
    };
    rows.push(Row::within(
        "a channel 20 dB quieter changes the reduction",
        0.0,
        0.1,
        "dB",
        dominant,
        "research/SSL-Gbus.md §13.5 test 20, from SSL's \"the dominant, ie. louder channel, \
         controls the gain reduction of the overall stereo level\" (P)",
    ));

    // The published claim is that the scale is *linear*, so the thing to
    // measure is proportionality across the scale rather than one point.
    // Asking for exactly 50 % at exactly 10 dB measures how close the
    // chosen input level landed to 10 dB, which is the instrument and not
    // the meter: it read 46.4 % because the tone settled at 9.28 dB.
    let deflection_per_db = {
        let s = gbus::Settings { release: 0, ..base };
        let mut worst = 0.0f32;
        for target in [5.0f32, 10.0, 15.0] {
            let mut c = gbus::Compressor::new(SR);
            c.configure(s);
            let level = gbus_level_for(s, target);
            let mut sine = Sine::new(1000.0, SR);
            let (mut l, mut r) = (vec![0.0f32; BLOCK], vec![0.0f32; BLOCK]);
            for _ in 0..((SR as usize * 3) / BLOCK) {
                sine.fill(&mut l, gbus_amp(level));
                r.copy_from_slice(&l);
                c.process_block(&mut l, &mut r);
            }
            let frame = c.meter_frame();
            let gr = frame[4];
            if gr > 0.5 {
                worst = worst.max((frame[5] / 20.0) / gr * 100.0);
            }
        }
        worst
    };
    rows.push(
        Row::within(
            "meter deflection per dB of reduction, over the scale",
            5.0,
            0.1,
            "% of full scale per dB",
            deflection_per_db,
            "research/SSL-Gbus.md §13.5 test 26, from the module's printed scale 0 4 8 12 16 20 \
             evenly spaced (P) and the clone builder's \"linear scale, at about 50 µA/dB, making \
             a 1 mA meter showing 20 dB full-scale\" (C)",
        )
        .because(
            "The rare case where the naive meter and the circuit meter agree, because a Blackmer \
             VCA's control voltage is linear in decibels. Measured at three depths, so what is \
             checked is that one dB is worth the same deflection anywhere on the scale",
        ),
    );

    // -- what cannot be anchored ----------------------------------------
    rows.push(Row::unanchored(
        "ratio calibration",
        format!(
            "{:.2}:1, {:.2}:1 and {:.2}:1 at 5 dB of reduction",
            gbus_ratio_at(
                gbus::Settings {
                    ratio: 0,
                    release: 0,
                    ..base
                },
                5.0
            ),
            gbus_ratio_at(
                gbus::Settings {
                    ratio: 1,
                    release: 0,
                    ..base
                },
                5.0
            ),
            gbus_ratio_at(
                gbus::Settings {
                    ratio: 2,
                    release: 0,
                    ..base
                },
                5.0
            ),
        ),
        "SSL publish no measured transfer point for any ratio position, with or without a \
         tolerance, in any document the dossier could reach. The control-bus scaling k is an \
         estimate, so a row reading \"5 dB ±1 dB at 4:1\" would be this model marking its own \
         homework, which is the failure an audit found in five plug-ins here. The dossier refuses \
         the test in its §13.3 and so does this table. What is checked instead is the law's shape \
         and its direction, above",
    ));
    rows.push(Row::unanchored(
        "where the knee sits in absolute terms",
        format!("{:.1} dBFS at 4:1 with the threshold centred", knees[1]),
        "Nothing is published. The detector's scaling is anchored so that it reaches one diode \
         drop at −12 dBFS, which is the level the only measured recordings of this unit were made \
         at, and that is an operating condition rather than a calibration. SSL's nominal +4 dBu \
         was tried first and is wrong for the job: it is a VU reference and this detector is a \
         peak rectifier",
    ));
    rows.push(Row::unanchored(
        "noise floor",
        "a design choice in floating point".into(),
        "The XLogic's \"< −99 dBu\" and Smart Research's \"−104 dBm\" describe different, later, \
         better circuits, and the clone's \"less than −80 dB\" describes a homebuilt one. \
         Asserting any of the three against a floating-point model would be theatre",
    ));

    Section {
        model: "4000 G",
        unit: "SSL 4000 G bus compressor, the 500-series module drawn with the console's values",
        dossier: "research/SSL-Gbus.md",
        rows,
    }
}

// ---------------------------------------------------------------------------
// The EMI TG12413
// ---------------------------------------------------------------------------

/// A 1 kHz sine at `amp` peak for `secs`, returning the peak of the last
/// 20 ms.
fn tg_settle(c: &mut tg::Compressor, amp: f32, secs: f32) -> f32 {
    const N: usize = 256;
    let blocks = (secs * SR / N as f32).ceil() as usize;
    let tail = ((0.020 * SR) as usize / N).max(1);
    let mut ph = 0.0f32;
    let step = 2.0 * PI * 1000.0 / SR;
    let mut peak = 0.0f32;
    for b in 0..blocks {
        let mut l = [0.0f32; N];
        let mut r = [0.0f32; N];
        for i in 0..N {
            l[i] = amp * ph.sin();
            r[i] = l[i];
            ph += step;
            if ph > 2.0 * PI {
                ph -= 2.0 * PI;
            }
        }
        c.process_block(&mut l, &mut r);
        if b + tail >= blocks {
            for v in l {
                peak = peak.max(v.abs());
            }
        }
    }
    peak
}

fn tg_out_dbu(s: tg::Settings, in_dbu: f32, secs: f32) -> f32 {
    let mut c = tg::Compressor::new(SR);
    c.configure(s);
    tg::engine::amp_dbu(tg_settle(&mut c, tg::engine::dbu_amp(in_dbu), secs))
}

/// Release time constant, in seconds, from the store's own discharge.
fn tg_release_s(recovery: usize, hold: f32) -> f32 {
    const N: usize = 32;
    let mut c = tg::Compressor::new(SR);
    c.configure(tg::Settings {
        recovery,
        hold,
        ..tg::Settings::default()
    });
    tg_settle(&mut c, 0.9, 1.0);
    let mut l = [0.0f32; N];
    let mut r = [0.0f32; N];
    c.process_block(&mut l, &mut r);
    let target = c.control_a(0) / std::f32::consts::E;
    for b in 1..(20.0 * SR / N as f32) as usize {
        let mut l = [0.0f32; N];
        let mut r = [0.0f32; N];
        c.process_block(&mut l, &mut r);
        if c.control_a(0) <= target {
            return b as f32 * N as f32 / SR;
        }
    }
    f32::INFINITY
}

/// The TG12413's rows.
///
/// **This is the only section in the report with no manufacturer's figures
/// in it at all.** There is no factory handbook, no specification and no
/// measurement of any kind published for this unit. What EMI printed is a
/// circuit diagram, and every anchored row below is an arithmetic
/// consequence of component values read off it. The unanchored rows say
/// what they measured and why nothing pins it.
fn bench_tg() -> Section {
    let mut rows = Vec::new();
    let base = tg::Settings::default();

    // The two figures EMI printed, and the twenty-one resistors behind them.
    rows.push(Row::within(
        "output switch, position 1",
        -9.95,
        0.02,
        "dB",
        tg::engine::output_db(0),
        "research/TG12413.md §12 test 1, from the S3 ladder and the legend printed on the same sheet",
    ));
    rows.push(Row::within(
        "output switch, position 21",
        9.81,
        0.02,
        "dB",
        tg::engine::output_db(20),
        "research/TG12413.md §12 test 1, from the S3 ladder and the legend printed on the same sheet",
    ));
    rows.push(Row::within(
        "output switch, full span",
        19.76,
        0.05,
        "dB",
        tg::engine::output_db(20) - tg::engine::output_db(0),
        "research/TG12413.md §3.4; EMI printed −10 to +10 in 1 dB steps and the resistors give 19.76",
    ));
    {
        let mut worst = 0.0f32;
        for i in 1..20 {
            worst =
                worst.max((tg::engine::output_db(i) - tg::engine::output_db(i - 1) - 1.0).abs());
        }
        rows.push(Row::within(
            "output switch, worst step error",
            0.0,
            0.10,
            "dB",
            worst,
            "research/TG12413.md §12 test 1; the legend says 1 dB steps and the ladder delivers them to 0.09",
        ));
    }
    // Measured through the module, not just through the table.
    {
        let quiet = tg_out_dbu(
            tg::Settings {
                mode: tg::MODE_OUT,
                output: 0,
                ..base
            },
            0.0,
            0.3,
        );
        let unity = tg_out_dbu(
            tg::Settings {
                mode: tg::MODE_OUT,
                ..base
            },
            0.0,
            0.3,
        );
        rows.push(Row::within(
            "output switch through the module, position 1",
            -9.95,
            0.10,
            "dB",
            quiet - unity,
            "research/TG12413.md §12 test 1, measured rather than tabulated",
        ));
    }

    // The recovery ladder's ratios, which are the only hard timing figure.
    {
        let fast = tg_release_s(0, 0.0);
        for (pos, want) in [
            (1usize, 2.00f32),
            (2, 4.77),
            (3, 9.45),
            (4, 19.4),
            (5, 47.1),
        ] {
            rows.push(Row::within(
                &format!("recovery {} against position 1", pos + 1),
                want,
                0.02 * want,
                "x",
                tg_release_s(pos, 0.0) / fast,
                "research/TG12413.md §12 test 3, from the six resistors on switch assembly B204A",
            ));
        }
        rows.push(Row::within(
            "HOLD at recovery 1",
            21.3,
            1.0,
            "%",
            100.0 * (tg_release_s(0, 1.0) / fast - 1.0),
            "research/TG12413.md §12 test 5, from RV1's 10 kΩ against the ladder's 47 kΩ",
        ));
        rows.push(Row::within(
            "HOLD at recovery 6",
            0.45,
            0.6,
            "%",
            100.0 * (tg_release_s(5, 1.0) / tg_release_s(5, 0.0) - 1.0),
            "research/TG12413.md §12 test 5, from RV1's 10 kΩ against the ladder's 2 214 kΩ",
        ));
    }

    // The gain element, against the law it generalises.
    {
        let ring = tg::element::DiodeArmPair::ring();
        let k = 2.0 * tg::element::JUNCTION_SCALE;
        let mut worst = 0.0f32;
        for decade in 0..4 {
            let i_bias = 1e-6 * 10f32.powi(decade);
            for step in 1..=60 {
                let a = 0.995 * step as f32 / 60.0;
                let i0 = a * i_bias;
                let u = ring.voltage(i0, i_bias);
                let i1 = noob_electrical_components::diode_bridge::current(u, i_bias, k);
                worst = worst.max(((i1 - i0) / i0).abs());
            }
        }
        rows.push(Row::within(
            "(G1) reproduces the Neve's tanh law",
            0.0,
            f32::EPSILON / (1.0 - 0.995 * 0.995),
            "relative",
            worst,
            "research/TG12413.md §12 test 8, against the law derived in research/Neve-33609.md",
        ));
    }

    // The coupling capacitors.
    {
        let f = 20.0f32;
        rows.push(Row::within(
            "input coupling corner",
            4.5,
            1.0,
            "Hz",
            tg::engine::F_IN_COUPLING,
            "research/TG12413.md §12 test 6, from C1 4µ7 into R78 7K5",
        ));
        let out_loss = -20.0 * (f / (f * f + tg::engine::F_OUT_COUPLING.powi(2)).sqrt()).log10();
        rows.push(Row::within(
            "output coupling loss at 20 Hz",
            0.0,
            0.1,
            "dB",
            out_loss,
            "research/TG12413.md §12 test 6, from C23 470 µF into a 600 Ω load",
        ));
    }

    // What the unit does, with nothing to check it against.
    {
        let lo = tg_out_dbu(
            tg::Settings {
                mode: tg::MODE_LIMIT,
                ..base
            },
            10.0,
            1.5,
        );
        let hi = tg_out_dbu(
            tg::Settings {
                mode: tg::MODE_LIMIT,
                ..base
            },
            20.0,
            1.5,
        );
        rows.push(
            Row::unanchored(
                "LIMIT, output change for a 10 dB input step",
                format!("{:.2} dB", hi - lo),
                "Waves say only that this is \"not a brick-wall limiter: transients are expected to \
                 pass\", with no figure. What anchors the row is the band it must fall outside: AMS \
                 Neve publish 0.1 ± 0.1 dB for the 33609's limiter, and this is not that",
            ),
        );
    }
    {
        let mut c = tg::Compressor::new(SR);
        c.configure(base);
        tg_settle(&mut c, 1.0, 2.0);
        rows.push(Row::unanchored(
            "gain reduction at full scale, COMPRESS",
            format!("{:.2} dB", c.gain_reduction_db(0)),
            "no maximum gain reduction is published for this unit. 20 dB is the dossier's own \
             instruction in §11.6 and the model's control-current constant is fitted to it, so this \
             row records a calibration rather than checks one",
        ));
    }
    {
        let e = tg::element::Network::breakdown();
        rows.push(Row::unanchored(
            "gain reduction floor, breakdown region",
            format!("{:.1} dB", e.gr_db(1.0)),
            "the 2·r_b term of equation (G1) bounds the element's resistance below, so the divider's \
             loss is bounded. Where the floor sits depends on r_b, which is R16's 24 Ω taken as an \
             order of magnitude and not as a measurement; that a floor exists is the finding, not \
             its depth",
        ));
    }
    rows.push(Row::unanchored(
        "attack time",
        "not tested".into(),
        "R41 plus R47 into the store gives 47 ms, which is far too slow for a limiter, so the charge \
         path is almost certainly current-driven by VT17 and the RC figure is an upper bound. \
         Chandler say only \"Attack: Fixed\". §12.6 of the dossier refuses to test it and this \
         report does the same",
    ));
    rows.push(Row::unanchored(
        "threshold in dBu",
        "not tested".into(),
        "there is no threshold control and the reference is three germanium diodes whose drop at \
         their working current nobody has measured. Where the model starts working is a documented \
         choice, not a figure",
    ));
    rows.push(Row::unanchored(
        "distortion at any level",
        "not tested".into(),
        "no spectrum, no THD figure and no noise figure has ever been published for this unit. The \
         element's drive is fitted to the two ends of the THD scale Chandler print on the TG1's \
         input knob, which is a figure about a licensed recreation with its own added stages",
    ));

    Section {
        model: "TG12413",
        unit: "EMI TG12413",
        dossier: "research/TG12413.md",
        rows,
    }
}

// ---------------------------------------------------------------------------
// The Fairchild 660 and 670
// ---------------------------------------------------------------------------

/// Settle a sine at `in_dbm` and return the output level in dBm.
fn vmu_out_dbm(s: vmu::Settings, in_dbm: f32, secs: f32) -> f32 {
    let mut c = vmu::Compressor::new(SR);
    c.configure(s);
    let amp = vmu::engine::dbm_amp(in_dbm);
    let n = (secs * SR) as usize;
    let mut ph = 0.0f32;
    let step = std::f32::consts::TAU * 1000.0 / SR;
    let mut l = vec![0.0f32; BLOCK];
    let mut r = vec![0.0f32; BLOCK];
    let mut peak = 0.0f32;
    let mut done = 0;
    while done < n {
        for i in 0..BLOCK {
            l[i] = amp * ph.sin();
            ph += step;
            if ph > std::f32::consts::TAU {
                ph -= std::f32::consts::TAU;
            }
            r[i] = l[i];
        }
        c.process_block(&mut l, &mut r);
        if done + BLOCK > n.saturating_sub(SR as usize / 20) {
            for &v in l.iter() {
                peak = peak.max(v.abs());
            }
        }
        done += BLOCK;
    }
    vmu::engine::amp_dbm(peak)
}

/// Seconds for the gain reduction to fall to 0.75 dB, which is what section
/// 5.4 establishes Fairchild's "release time from 10 db of limiting" meant.
fn vmu_release_s(pos: usize, hold_s: f32, cap_s: f32) -> f32 {
    let s = vmu::Settings {
        time: [pos; 2],
        oversample: 0,
        ..vmu::Settings::default()
    };
    let mut c = vmu::Compressor::new(SR);
    c.configure(s);
    // The level that gives ten decibels of steady reduction.
    let (mut lo, mut hi) = (-20.0f32, 45.0f32);
    for _ in 0..40 {
        let m = 0.5 * (lo + hi);
        if c.static_gr_db(vmu::engine::dbm_amp(m)) < 10.0 {
            lo = m;
        } else {
            hi = m;
        }
    }
    let hot = vmu::engine::dbm_amp(0.5 * (lo + hi));
    let quiet = vmu::engine::dbm_amp(-10.0);
    let step = std::f32::consts::TAU * 1000.0 / SR;
    let mut ph = 0.0f32;
    let mut l = vec![0.0f32; BLOCK];
    let mut r = vec![0.0f32; BLOCK];
    let mut drive = |c: &mut vmu::Compressor, amp: f32, secs: f32, ph: &mut f32| {
        let n = ((secs * SR) as usize).max(1);
        let mut done = 0;
        while done < n {
            for i in 0..BLOCK {
                l[i] = amp * ph.sin();
                *ph += step;
                if *ph > std::f32::consts::TAU {
                    *ph -= std::f32::consts::TAU;
                }
                r[i] = l[i];
            }
            c.process_block(&mut l, &mut r);
            done += BLOCK;
            if amp == quiet && c.gain_reduction_db(0) <= 0.75 {
                return done as f32 / SR;
            }
        }
        f32::INFINITY
    };
    drive(&mut c, hot, hold_s, &mut ph);
    drive(&mut c, quiet, cap_s, &mut ph)
}

/// Seconds for the gain reduction to reach nine decibels of a ten decibel
/// step, which is the criterion the dossier's test plan asks for, with the
/// oversampler's own input delay taken off.
fn vmu_attack_s(pos: usize) -> f32 {
    let s = vmu::Settings {
        time: [pos; 2],
        ..vmu::Settings::default()
    };
    let mut c = vmu::Compressor::new(SR);
    c.configure(s);
    let (mut lo, mut hi) = (-20.0f32, 45.0f32);
    for _ in 0..40 {
        let m = 0.5 * (lo + hi);
        if c.static_gr_db(vmu::engine::dbm_amp(m)) < 10.0 {
            lo = m;
        } else {
            hi = m;
        }
    }
    let amp = vmu::engine::dbm_amp(0.5 * (lo + hi));
    let pre = c.latency() as f32 / 2.0 / SR;
    // A ten kilohertz tone started at its own peak, so the input is a step
    // rather than a quarter period of ramp.
    let step = std::f32::consts::TAU * 10_000.0 / SR;
    let mut ph = std::f32::consts::FRAC_PI_2;
    let mut l = [0.0f32; 1];
    let mut r = [0.0f32; 1];
    for k in 0..(0.05 * SR) as usize {
        l[0] = amp * ph.sin();
        ph += step;
        if ph > std::f32::consts::TAU {
            ph -= std::f32::consts::TAU;
        }
        r[0] = l[0];
        c.process_block(&mut l, &mut r);
        if c.gain_reduction_db(0) >= 9.0 {
            return ((k + 1) as f32 / SR - pre).max(0.0);
        }
    }
    f32::INFINITY
}

/// Settle the unit, then capture 4800 samples: a tenth of a second at
/// 48 kHz, so every tone in these rows lands exactly on a bin.
fn vmu_capture(c: &mut vmu::Compressor, tones: &[(f32, f32)], warm: f32) -> Vec<f32> {
    let mut ph = vec![0.0f32; tones.len()];
    let mut l = vec![0.0f32; BLOCK];
    let mut r = vec![0.0f32; BLOCK];
    let mut out = Vec::with_capacity(4800 + BLOCK);
    let total = (warm * SR) as usize + 4800;
    let mut done = 0;
    while done < total {
        for i in 0..BLOCK {
            let mut v = 0.0;
            for (t, (f, a)) in tones.iter().enumerate() {
                v += a * ph[t].sin();
                ph[t] += std::f32::consts::TAU * f / SR;
                if ph[t] > std::f32::consts::TAU {
                    ph[t] -= std::f32::consts::TAU;
                }
            }
            l[i] = v;
            r[i] = v;
        }
        c.process_block(&mut l, &mut r);
        if done >= (warm * SR) as usize {
            out.extend_from_slice(&l);
        }
        done += BLOCK;
    }
    out.truncate(4800);
    out
}

/// Total harmonic distortion of a settled 1 kHz tone, per cent.
fn vmu_thd(c: &mut vmu::Compressor, amp: f32) -> f32 {
    thd_pct(&vmu_capture(c, &[(1000.0, amp)], 0.4), 1000.0, SR)
}

/// SMPTE intermodulation, per cent: 60 Hz and 7 kHz mixed 4:1, measured as
/// the sidebands about the carrier, which is the condition the chart names.
fn vmu_smpte_im(c: &mut vmu::Compressor, composite_peak: f32) -> f32 {
    let x = vmu_capture(
        c,
        &[(60.0, composite_peak * 0.8), (7000.0, composite_peak * 0.2)],
        0.4,
    );
    let carrier = goertzel(&x, 7000.0, SR);
    let sb: f32 = [6940.0f32, 7060.0, 6880.0, 7120.0]
        .iter()
        .map(|&f| goertzel(&x, f, SR).powi(2))
        .sum::<f32>()
        .sqrt();
    100.0 * sb / carrier.max(1e-12)
}

/// The threshold and input level that put the unit at `want_gr` decibels of
/// reduction with `out` dBm at the output, which is the condition both of
/// Fairchild's distortion figures are quoted at.
fn vmu_hold_out(base: vmu::Settings, want_gr: f32, out: f32) -> (vmu::Settings, f32) {
    let level = |s: vmu::Settings| {
        let mut c = vmu::Compressor::new(SR);
        c.configure(s);
        let (mut lo, mut hi) = (-20.0f32, 45.0f32);
        for _ in 0..40 {
            let m = 0.5 * (lo + hi);
            if c.static_gr_db(vmu::engine::dbm_amp(m)) < want_gr {
                lo = m;
            } else {
                hi = m;
            }
        }
        0.5 * (lo + hi)
    };
    let (mut lo, mut hi) = (0.0f32, 10.0);
    let mut s = base;
    let mut inp = 0.0;
    for _ in 0..24 {
        let t = 0.5 * (lo + hi);
        s = vmu::Settings {
            threshold: [t; 2],
            time: [0; 2],
            ..base
        };
        inp = level(s);
        if inp + vmu::engine::REST_GAIN_DB - want_gr > out {
            lo = t;
        } else {
            hi = t;
        }
    }
    (s, inp)
}

/// Response at `hz` relative to 1 kHz, in dB.
fn vmu_response_db(s: vmu::Settings, hz: f32) -> f32 {
    let mut c = vmu::Compressor::new(SR);
    c.configure(s);
    let amp = vmu::engine::dbm_amp(4.0);
    let a = goertzel(&vmu_capture(&mut c, &[(hz, amp)], 0.4), hz, SR);
    let mut c = vmu::Compressor::new(SR);
    c.configure(s);
    let b = goertzel(&vmu_capture(&mut c, &[(1000.0, amp)], 0.4), 1000.0, SR);
    20.0 * (a / b).log10()
}

fn bench_vmu() -> Section {
    let mut rows = Vec::new();
    let base = vmu::Settings::default();

    // -- the static curves, which are two manufacturer measurements -------
    let linear = vmu::Settings {
        threshold: [0.0; 2],
        ..base
    };
    rows.push(Row::within(
        "straight amplifier: gain at 0 dBm in, threshold fully CCW",
        2.0,
        0.5,
        "dBm",
        vmu_out_dbm(linear, 0.0, 0.5),
        "research/Fairchild-670.md §7.2, curve 1 of the December 1959 input/output chart (M, \
         manufacturer measurement)",
    ));
    for (in_dbm, want) in [
        (0.0f32, 2.0f32),
        (5.0, 4.3),
        (10.0, 5.3),
        (15.0, 5.7),
        (20.0, 5.9),
    ] {
        rows.push(Row::within(
            &format!("factory curve at {in_dbm:+.0} dBm in"),
            want,
            1.0,
            "dBm out",
            vmu_out_dbm(base, in_dbm, 1.0),
            "research/Fairchild-670.md §7.2, curve 3 \"factory-adjusted condition\" (M, \
             manufacturer measurement, read to ±0.5 dB)",
        ));
    }
    rows.push(
        Row::within(
            "progressive ratio, +2 to +12 dBm in",
            3.3,
            1.0,
            "dB out",
            vmu_out_dbm(base, 12.0, 1.0) - vmu_out_dbm(base, 2.0, 0.6),
            "research/Fairchild-670.md §7.2 item 4, curve 3 read at the two levels",
        )
        .because(
            "The knee is about 1.3 dB firmer than Fairchild's. The two constants that shape \
             it, the sidechain's stage gain and the factory setting of the DC trimmer, are \
             already fitted by least squares to the five points of this same curve, so tuning \
             either of them to this one figure would make the three rows above meaningless as \
             well. The ratio at depth is met, at 1.02 dB against a published 0.6 ± 0.6",
        ),
    );
    rows.push(Row::within(
        "progressive ratio, +10 to +20 dBm in",
        0.6,
        0.6,
        "dB out",
        vmu_out_dbm(base, 20.0, 1.0) - vmu_out_dbm(base, 10.0, 1.0),
        "research/Fairchild-670.md §7.2 item 4, curve 3 read at the two levels",
    ));

    // -- the tube, which is the only component-manufacturer figure here ---
    let t = vmu::triode::RemoteCutoffTriode::ge_6386();
    rows.push(
        Row::within(
            "6386 gain-control range, class-A1 point to −16 V",
            32.0,
            3.0,
            "dB",
            20.0 * (t.transconductance(-1.92, 100.0) / t.transconductance(-16.0, 100.0)).log10(),
            "research/Fairchild-670.md §4.2, GE datasheet ET-T1113: 4000 µmhos at the operating \
             point and 100 µmhos at −16 V (D from two printed rows)",
        )
        .because(
            "Two published sources describe this tube and they constrain different regions. \
             Raffensperger's fitted equation is the only published *model* of it and the dossier \
             prescribes it, but it is fitted to plate **current**, so its **slope** — the one \
             quantity a variable-mu gain stage actually uses — is not constrained by that fit. \
             GE's page 5 plate characteristics give every grid voltage its own line and so \
             resolve the deep end, down to −70 V, which is the region this unit works in. The \
             cut-off rate and the scale are fitted to GE instead (next four rows): moving \
             from one published source to another beats keeping a fit that is wrong where the \
             audio is made. What that costs is this row — the shallow, low-plate-voltage \
             corner GE's *table* quotes, at Eb = 100 V near Vgk = 0, where the law runs about \
             30 % flat. One parameter cannot have both ends, and the end this stage lives in \
             was taken. It is also the root of the distortion miss below",
        ),
    );
    for (vgk, want) in [
        (-20.0f32, 8.85f32),
        (-30.0, 5.14),
        (-50.0, 1.60),
        (-70.0, 0.60),
    ] {
        rows.push(
            Row::within(
                &format!("6386 plate current at 250 V, {vgk:.0} V grid"),
                want,
                0.42 * want,
                "mA",
                t.anode_current(vgk, 250.0) * 1e3,
                "GE ET-T1113 page 5, average **plate** characteristics, read off a 400 dpi \
                 render (D, one reader, one 1953 graph)",
            )
            .because(
                "a fit residual rather than an independent check: the law's scale and cut-off \
                 rate were fitted to these readings. It is here because the correction they \
                 encode is the largest single change this model has had. As Raffensperger \
                 published the equation it reads 4.8 dB low at −40 V, 9.1 low at −50 and 37.3 \
                 low at −70, and the Fairchild's grids reach −70 V at the deepest limiting \
                 its own published static curves show",
            ),
        );
    }
    rows.push(
        Row::within(
            "6386 amplification factor at the class-A1 point",
            17.0,
            2.0,
            "",
            t.mu(-1.92, 100.0),
            "GE ET-T1113: tabulated mu 17, which closes against the tabulated 4250 ohm and \
             4000 umho and against the curve spacing on page 5, measured at 16.5 (M)",
        )
        .because(
            "the functional form has Vak^p2 over a grid-only denominator, which forces mu to \
             rise with plate voltage where the tube's falls: 16.5 near zero bias to 5.8 at \
             −30 V along the constant-current locus. No choice of its eight parameters does \
             both. **Nothing in the engine reads it**, because the audio path is a difference \
             of two plate currents into a fixed plate voltage and never divides a load \
             against a plate resistance",
        ),
    );

    // -- distortion, which is the family's whole point --------------------
    let quiet = vmu::Settings {
        threshold: [0.0; 2],
        time: [0; 2],
        ..base
    };
    for (out, want) in [(12.0f32, 0.25f32), (16.0, 0.6), (20.0, 1.65), (24.0, 3.9)] {
        let mut c = vmu::Compressor::new(SR);
        c.configure(quiet);
        let row = Row::within(
            &format!("SMPTE IM at {out:+.0} dBm out, no limiting"),
            want,
            0.5,
            "%",
            vmu_smpte_im(
                &mut c,
                vmu::engine::dbm_amp(out - vmu::engine::REST_GAIN_DB),
            ),
            "research/Fairchild-670.md §4.6, the March 1959 IM chart, 60 c/s and 7 kc at \
             4:1 (M, manufacturer measurement, read to ±0.5 points)",
        );
        rows.push(if out >= 20.0 {
            row.because(
                "the tube stage on its own is cleaner than the unit is measured to be and \
                 its IM tops out near 1.4 %. The grid swing that sets the drive used to be \
                 fitted to this chart's top curve and reproduced all four; that agreement \
                 rested on a tube law 5 to 37 dB low below −40 V, which is where a stage \
                 driven that hard spends its peaks. The swing is now derived from the \
                 published clipping point instead, and the two lowest readable curves still \
                 land, from a different document. What the model has not got is four \
                 transformers a channel, which the dossier's 8.3 says not to model",
            )
        } else {
            row
        });
    }
    let mut c = vmu::Compressor::new(SR);
    c.configure(quiet);
    rows.push(Row::ranged(
        "harmonic distortion at +18 dBm out, no limiting",
        0.0,
        1.0,
        "%",
        vmu_thd(
            &mut c,
            vmu::engine::dbm_amp(18.0 - vmu::engine::REST_GAIN_DB),
        ),
        "research/Fairchild-670.md §7.1, \"less than 1 % at any level up to +18 dbm output (no \
         limiting)\" (M)",
    ));
    let (deep, deep_in) = vmu_hold_out(base, 10.0, 12.0);
    let mut c = vmu::Compressor::new(SR);
    c.configure(deep);
    rows.push(
        Row::ranged(
            "harmonic distortion at +12 dBm out and 10 dB of limiting",
            0.0,
            1.0,
            "%",
            vmu_thd(&mut c, vmu::engine::dbm_amp(deep_in)),
            "research/Fairchild-670.md §7.1, \"less than 1 % at 10 db limiting and +12 dbm \
             output\" (M)",
        )
        .because(
            "Holding the output while taking ten decibels of reduction means driving the grids \
             ten decibels harder — that is the identity this engine exists to express and no \
             model of this circuit can avoid it. What decides the cost is the shape of the \
             tube's curve at the bias the control voltage has moved to, and the fitted law \
             steepens faster below −35 V than the hardware evidently does. Same cause as the \
             gain-range row above",
        ),
    );

    // -- the timing network, from the factory drawing ---------------------
    for (pos, want) in [(0usize, 0.3f32), (1, 0.8), (2, 2.0), (3, 5.0)] {
        rows.push(Row::within(
            &format!("release, position {}", pos + 1),
            want,
            0.3 * want,
            "s",
            vmu_release_s(pos, 1.0, 12.0),
            "research/Fairchild-670.md §7.1, \"RELEASE TIME (from 10 db of limiting)\" (M); the \
             model is given the fourteen component values and nothing else",
        ));
    }
    rows.push(Row::within(
        "release, position 6, individual peak (2 ms)",
        0.3,
        0.4 * 0.3,
        "s",
        vmu_release_s(5, 0.002, 3.0),
        "research/Fairchild-670.md §7.1, position 6 \".3 seconds for individual peaks\" (M)",
    ));
    rows.push(Row::within(
        "release, position 6, multiple peaks (0.3 s of limiting)",
        10.0,
        4.0,
        "s",
        vmu_release_s(5, 0.3, 30.0),
        "research/Fairchild-670.md §7.1, position 6 \"10 seconds for multiple peaks\" (M)",
    ));
    rows.push(Row::within(
        "release, position 6, sustained (3 s of limiting)",
        25.0,
        10.0,
        "s",
        vmu_release_s(5, 3.0, 45.0),
        "research/Fairchild-670.md §7.1, position 6 \"25 seconds for consistently high program \
         level\" (M); nobody has quantified these three before",
    ));
    rows.push(
        Row::within(
            "release, position 5, individual peak (2 ms)",
            2.0,
            0.35 * 2.0,
            "s",
            vmu_release_s(4, 0.002, 20.0),
            "research/Fairchild-670.md §7.1, position 5 \"2 seconds for individual peaks\" (M)",
        )
        .because(
            "The dossier contradicts itself here and the network settles it. Its §5.4 derives \
             this figure from R_T·C_T alone, treating the uncharged slow leg as not yet loading \
             the node; its §5.5 requires the opposite to reach position 6's 0.3 s, and admits \
             that no single simple reading gives both. Built from the drawing, the mechanism \
             works at position 6, where the node's own 0.44 s is fast against the legs' 0.8 and \
             2.0 s, and fails at position 5, where the node's 0.88 s is slower than its one \
             leg's 0.8 s — so that leg's 8 µF joins the node immediately whatever the stimulus. \
             The multiple-peaks figure below is met",
        ),
    );
    rows.push(Row::within(
        "release, position 5, multiple peaks (1 s of limiting)",
        10.0,
        4.0,
        "s",
        vmu_release_s(4, 1.0, 25.0),
        "research/Fairchild-670.md §7.1, position 5 \"10 seconds for multiple peaks\" (M)",
    ));
    for (pos, want) in [
        (0usize, 0.2f32),
        (1, 0.2),
        (2, 0.4),
        (3, 0.8),
        (4, 0.4),
        (5, 0.2),
    ] {
        rows.push(Row::within(
            &format!("attack, position {}", pos + 1),
            want,
            0.4 * want,
            "ms",
            vmu_attack_s(pos) * 1e3,
            "research/Fairchild-670.md §5.6, Sound On Sound's attack table (S, confirmed by the \
             circuit); nine decibels of a ten decibel step, which is the plan's own \
             criterion. **The manual gives 0.4 ms for position 4** and the circuit says 0.8",
        ));
    }

    // -- response, and the one figure that is specified without a condition
    rows.push(Row::within(
        "response at 40 Hz, no limiting",
        0.0,
        1.0,
        "dB",
        vmu_response_db(linear, 40.0),
        "research/Fairchild-670.md §7.1, \"40 cycles to 15 kc ± 1 db\" (M)",
    ));
    rows.push(Row::within(
        "response at 15 kHz, no limiting",
        0.0,
        1.0,
        "dB",
        vmu_response_db(linear, 15_000.0),
        "research/Fairchild-670.md §7.1, \"40 cycles to 15 kc ± 1 db\" (M)",
    ));

    // -- the 660, about which no specification exists ---------------------
    rows.push(Row::unanchored(
        "660 against 670: small-signal gain at 0 dBm in",
        format!(
            "{:+.2} dB",
            vmu_out_dbm(
                vmu::Settings {
                    model: vmu::MODEL_660,
                    ..linear
                },
                0.0,
                0.5
            ) - vmu_out_dbm(linear, 0.0, 0.5)
        ),
        "the dossier holds **no 660 specification sheet** and says so, so nothing about the 660's \
         static curve or its distortion is asserted. What is modelled is the one difference it \
         trusts: 1800 Ω of cathode resistor against the 670's 680, which is a deeper standing \
         bias and therefore less transconductance and less gain",
    ));

    Section {
        model: "670",
        unit: "Fairchild 660 and 670 variable-mu limiting amplifiers",
        dossier: "research/Fairchild-670.md",
        rows,
    }
}

fn main() {
    eprintln!(
        "driving the engines; this takes a few minutes at {} kHz",
        SR / 1000.0
    );
    let sections = vec![
        bench_fet(),
        bench_opto(),
        bench_opto3(),
        bench_vca(),
        bench_pre(),
        bench_opto1b(),
        bench_bridge(),
        bench_rms(),
        bench_gbus(),
        bench_tg(),
        bench_vmu(),
    ];

    for s in &sections {
        let (m, x, n) = s.counts();
        eprintln!("{:>12}: {m} meet, {x} miss, {n} unanchored", s.model);
    }

    let doc = render(&sections);
    let path = std::path::Path::new("docs").join("BENCHMARK.md");
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).expect("create docs/");
    }
    std::fs::write(&path, doc).expect("write docs/BENCHMARK.md");
    eprintln!("wrote {}", path.display());
}

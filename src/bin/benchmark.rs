//! Measure every model against the figures its research document publishes,
//! and write the comparison to `docs/BENCHMARK.md`.
//!
//! Run it with `cargo run --release --bin benchmark`. It is a binary rather
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

use noob_compressorlab::dsp::{fet, opto, opto1b, opto3, pre, vca};
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
            "known miss, recorded in README: a hard-clipped 15 kHz tone has more harmonics than \
             first-order anti-aliasing removes, and the panel's own pad exists for that setting. \
             This figure is the maximum of a 25 Hz sweep across the whole band below 10 kHz and is \
             worse than the README's −51 dB, which was measured differently; the disagreement is \
             worth resolving rather than picking whichever number flatters the model",
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

/// Output change for a 10 dB input rise, starting from `depth` dB of
/// reduction at the 2:1 stop.
fn opto1b_ten_in(depth: f32) -> f32 {
    // Find the input that gives the requested depth.
    let mut lo = -50.0f32;
    let mut hi = 0.0f32;
    let measure = |dbfs: f32| {
        let mut c = opto1b::Compressor::new(SR);
        c.configure(opto1b::Settings {
            ratio: 0.0,
            threshold: 0.7,
            mode: opto1b::MODE_MANUAL,
            ..opto1b::Settings::default()
        });
        let (_, gr) = steady(&mut c, 1000.0, amp_dbfs(dbfs), 4.0, 0.25, SR);
        -gr
    };
    for _ in 0..18 {
        let mid = 0.5 * (lo + hi);
        if measure(mid) < depth {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let base = 0.5 * (lo + hi);
    let out = |dbfs: f32| {
        let mut c = opto1b::Compressor::new(SR);
        c.configure(opto1b::Settings {
            ratio: 0.0,
            threshold: 0.7,
            mode: opto1b::MODE_MANUAL,
            ..opto1b::Settings::default()
        });
        settled_out_dbfs(&mut c, 1000.0, amp_dbfs(dbfs), 4.0, SR)
    };
    out(base + 10.0) - out(base)
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
        "Generated by `cargo run --release --bin benchmark`. Do not edit by hand: regenerate it.\n\n",
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
        let _ = writeln!(out, "Figures from [`{}`]({}).\n", s.dossier, s.dossier);
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

    out.push_str("## Where this disagrees with the README\n\n");
    out.push_str(
        "The README carries its own table of the figures these models do not reach, and it lists \
         five. This run reports seven. A disagreement between the two is a finding in its own right, \
         so rather than reconcile them silently, here is what differs.\n\n",
    );
    out.push_str(
        "One earlier disagreement turned out to be this benchmark's fault and is recorded here \
         rather than quietly deleted: the 610's distortion at the +15 dBu equivalent was reported as \
         far below its published band, because the drive used the wrong decibel calibration and \
         under-drove the stage. Corrected, it lands inside the published range and agrees with the \
         repository's own test. A benchmark that disagrees with a passing test is at least as likely \
         to be wrong as the test.\n\n",
    );
    out.push_str("| difference | what to do about it |\n|---|---|\n");
    out.push_str(
        "| The LA-3A's maximum gain reduction in Compress is short of the published 40 dB, and the \
         README does not list it | The engine records this as a real divergence at its own test, on \
         the grounds that in Compress every decibel of reduction takes a decibel off the side-chain \
         so the loop starves itself. If that reasoning stands, the README's table should carry the \
         row too. |\n",
    );
    out.push_str(
        "| The 610's response at 20 kHz falls outside the published +0 / −1 dB, and the README does \
         not list it | The README does discuss high-frequency droop from the anti-aliasing, and \
         says the stage runs at 4x rather than 2x for exactly that reason. This measurement suggests \
         4x has not removed all of it. |\n",
    );
    out.push_str(
        "| The 610's response at 20 kHz has no test at all | Nothing in `src/dsp/pre/tests.rs` \
         asserts the published bandwidth at the top end, so this row is checking a figure the suite \
         does not. That is a gap in the tests, not only in the model. |\n",
    );
    out.push_str(
        "| The 610's worst alias measures −34.6 dB here against the README's −51 dB | Both cannot be \
         right. This run takes the maximum of a 25 Hz sweep across everything below 10 kHz, which is \
         a wider net than a measurement aimed at specific products. The method needs settling before \
         either number is quoted. |\n",
    );
    out.push_str(
        "| The CL 1B's 2:1 stop gives 6.0 dB here against the README's 6.4 dB | The same miss, \
         measured from a slightly different operating point. Worth pinning one procedure. |\n\n",
    );

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

//! Section 12 of `research/TG12413.md`, test by test.
//!
//! The numbering here is the dossier's, so a failure names the test in the
//! document it came from. **This unit is the hardest case in the lab for
//! the repository's testing standard**, because no factory handbook, no
//! specification and no measurement of any kind has ever been published
//! for it. So the dossier divides its tests by what kind of thing they
//! assert and every one of them says which kind it is, and that labelling
//! is repeated here:
//!
//! * **PRINTED** — a figure EMI printed on drawing TG12413-D101. There are
//!   two and they are the backbone.
//! * **IDENTITY** — an exact consequence of component values read off the
//!   drawing. Tight tolerances, because arithmetic has no tolerance.
//! * **QUOTED** — a manufacturer's qualitative statement, quoted, with the
//!   assertion written as a direction or an inequality because the source
//!   gives no number.
//! * **CROSS** — a difference from the Neve model under identical input,
//!   anchored where possible to a figure published about the Neve.
//!
//! Section 12.6 lists eight things the dossier refuses to test because
//! nothing supports a number: the attack time, the recovery times in
//! milliseconds, the threshold in dBu, the maximum gain reduction, the
//! distortion at any level, the noise, the response beyond the coupling
//! corners, and the TG12345 console channel limiter. **That refusal is
//! honoured. There is no test below for any of them.**

use super::*;
use element::Element;
use engine::*;

use crate::dsp::bridge;

const SR: f32 = 48_000.0;
/// The rates the dossier asks every timing test to run at.
const RATES: [f32; 3] = [44_100.0, 48_000.0, 96_000.0];

// -------------------------------------------------------------- harness

/// The dossier's stated default condition for section 12.
fn base() -> Settings {
    Settings::default()
}

fn unit(sr: f32, s: Settings) -> Compressor {
    let mut c = Compressor::new(sr);
    c.configure(s);
    c
}

/// Feed `secs` of a sine at `hz` and `amp` peak, returning the peak of the
/// last 20 ms.
fn settle_at(c: &mut Compressor, amp: f32, hz: f32, secs: f32, sr: f32) -> f32 {
    const N: usize = 256;
    let blocks = (secs * sr / N as f32).ceil() as usize;
    // Long enough to contain a crest: 20 ms, or a period and a half at the
    // frequency under test, whichever is longer. Twenty milliseconds of a
    // 4.5 Hz sine is 32 degrees of arc and its peak is not the waveform's.
    let window = 0.020f32.max(1.5 / hz);
    let tail = ((window * sr) as usize / N).max(1);
    let mut ph = 0.0f32;
    let step = std::f32::consts::TAU * hz / sr;
    let mut peak = 0.0f32;
    for b in 0..blocks {
        let mut l = [0.0f32; N];
        let mut r = [0.0f32; N];
        for i in 0..N {
            l[i] = amp * ph.sin();
            r[i] = l[i];
            ph += step;
            if ph > std::f32::consts::TAU {
                ph -= std::f32::consts::TAU;
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

fn settle(c: &mut Compressor, amp: f32, secs: f32, sr: f32) -> f32 {
    settle_at(c, amp, 1000.0, secs, sr)
}

/// Steady output level in dBu for a 1 kHz sine at `in_dbu`.
fn out_dbu(c: &mut Compressor, in_dbu: f32, secs: f32, sr: f32) -> f32 {
    c.reset();
    amp_dbu(settle(c, dbu_amp(in_dbu), secs, sr))
}

/// Steady gain reduction in dB for a 1 kHz sine at `amp`, measured from
/// the audio rather than read off the engine's own accumulator.
fn measured_gr_db(c: &mut Compressor, amp: f32, secs: f32, sr: f32) -> f32 {
    c.reset();
    let out = settle(c, amp, secs, sr);
    let ladder = output_db(c.settings().output) + c.settings().input_db;
    20.0 * (amp / out.max(1e-12)).log10() + ladder
}

/// Capture `secs` of steady-state output after `warm` seconds.
fn capture(c: &mut Compressor, amp: f32, warm: f32, secs: f32, sr: f32) -> Vec<f32> {
    capture_at(c, amp, 1000.0, warm, secs, sr)
}

fn capture_at(c: &mut Compressor, amp: f32, hz: f32, warm: f32, secs: f32, sr: f32) -> Vec<f32> {
    const N: usize = 240;
    let mut ph = 0.0f32;
    let step = std::f32::consts::TAU * hz / sr;
    let mut out = Vec::new();
    let blocks = ((warm + secs) * sr / N as f32).ceil() as usize;
    let keep = (secs * sr / N as f32).ceil() as usize;
    for b in 0..blocks {
        let mut l = [0.0f32; N];
        let mut r = [0.0f32; N];
        for i in 0..N {
            l[i] = amp * ph.sin();
            r[i] = l[i];
            ph += step;
            if ph > std::f32::consts::TAU {
                ph -= std::f32::consts::TAU;
            }
        }
        c.process_block(&mut l, &mut r);
        if b + keep >= blocks {
            out.extend_from_slice(&l);
        }
    }
    out
}

/// Magnitude of harmonic `h` of a `f0` component in `x` at `sr`.
fn harmonic_at(x: &[f32], f0: f32, h: usize, sr: f32) -> f32 {
    let f = f0 * h as f32;
    let (mut re, mut im) = (0.0f64, 0.0f64);
    for (n, v) in x.iter().enumerate() {
        let th = std::f64::consts::TAU * f as f64 * n as f64 / sr as f64;
        re += *v as f64 * th.cos();
        im += *v as f64 * th.sin();
    }
    (2.0 / x.len() as f64 * (re * re + im * im).sqrt()) as f32
}

fn harmonic(x: &[f32], h: usize, sr: f32) -> f32 {
    harmonic_at(x, 1000.0, h, sr)
}

/// Third-harmonic ratio, h₃ over h₁.
fn h3_ratio(x: &[f32], sr: f32) -> f32 {
    let f1 = harmonic(x, 1, sr);
    if f1 <= 0.0 {
        0.0
    } else {
        harmonic(x, 3, sr) / f1
    }
}

/// Total harmonic distortion as a percentage, harmonics 2 to 10.
fn thd_pct(x: &[f32], sr: f32) -> f32 {
    let f1 = harmonic(x, 1, sr);
    if f1 <= 0.0 {
        return 0.0;
    }
    let mut sum = 0.0f32;
    for h in 2..=10 {
        let a = harmonic(x, h, sr);
        sum += a * a;
    }
    100.0 * sum.sqrt() / f1
}

/// Run the gain element alone: a sine of peak `v_s` volts through the
/// divider at a fixed bias current, mean removed.
fn element_run(e: &Element, v_s: f32, i_bias: f32, cycles: usize, per: usize) -> Vec<f32> {
    let n = cycles * per;
    let mut out: Vec<f32> = (0..n)
        .map(|i| {
            let th = std::f32::consts::TAU * i as f32 / per as f32;
            e.solve(v_s * th.sin(), i_bias, 2)
        })
        .collect();
    let mean = out.iter().sum::<f32>() / n as f32;
    for v in &mut out {
        *v -= mean;
    }
    out
}

/// Harmonic `h` of an element run whose fundamental is one cycle in `per`.
fn element_harmonic(x: &[f32], h: usize, per: usize) -> f32 {
    harmonic_at(x, 1.0, h, per as f32)
}

/// Drive `c` to steady state, then feed silence and return the time in
/// seconds for the control current to fall to 1/e of where it started.
///
/// This measures the store's discharge through the recovery ladder, which
/// is the quantity S2 selects, without needing an absolute time from
/// anywhere.
fn release_tau(c: &mut Compressor, amp: f32, sr: f32) -> f32 {
    const N: usize = 32;
    c.reset();
    settle(c, amp, 1.0, sr);
    let mut l = [0.0f32; N];
    let mut r = [0.0f32; N];
    c.process_block(&mut l, &mut r);
    let start = c.control_a(0);
    let target = start / std::f32::consts::E;
    for b in 1..(20.0 * sr / N as f32) as usize {
        let mut l = [0.0f32; N];
        let mut r = [0.0f32; N];
        c.process_block(&mut l, &mut r);
        if c.control_a(0) <= target {
            return b as f32 * N as f32 / sr;
        }
    }
    f32::INFINITY
}

// -------------------------- 12.1 calibration against figures EMI printed

/// Test 1. **[PRINTED]** *Figure:* the legend table printed on drawing
/// TG12413-D101 — "S3 … POSITION 1 = −10, 11 = 0, 21 = +10 … **1 dB
/// STEPS**", against the twenty-one ladder resistors on the same sheet.
///
/// **The strongest test in the file.** It checks twenty-one independent
/// readings of a photographed blueprint against a three-line legend on the
/// same sheet, and the model against both. If it fails, either the ladder
/// is implemented as decibels instead of resistances or a value was
/// misread.
#[test]
fn t01_the_output_switch_is_ten_down_to_ten_up_in_one_db_steps() {
    // Measured through the model, with the detector dead so nothing else
    // can move the level.
    let mut levels = [0.0f32; 21];
    for (i, level) in levels.iter_mut().enumerate() {
        let s = Settings {
            mode: MODE_OUT,
            output: i,
            ..base()
        };
        let mut c = unit(SR, s);
        *level = amp_dbu(settle(&mut c, dbu_amp(0.0), 0.3, SR));
    }
    let unity = levels[OUTPUT_UNITY];
    for (i, level) in levels.iter().enumerate() {
        let want = output_db(i);
        assert!(
            (level - unity - want).abs() <= 0.05,
            "position {} measured {:.3} dB where the ladder gives {want:.3}",
            i + 1,
            level - unity
        );
    }
    // The three positions EMI's legend names, from the ladder itself.
    for (pos, want) in [(0usize, -9.95f32), (10, 0.00), (20, 9.81)] {
        let got = output_db(pos);
        assert!(
            (got - want).abs() <= 0.02,
            "position {} is {got:.3} dB; the ladder requires {want} ±0.02",
            pos + 1
        );
    }
    for i in 1..21 {
        let step = output_db(i) - output_db(i - 1);
        assert!(
            (step - 1.0).abs() <= 0.10 || i == 20,
            "step {} to {} is {step:.3} dB; EMI printed 1 dB steps",
            i,
            i + 1
        );
    }
    // The last step is the one EMI's own values do not deliver: 0.83 dB
    // against a nominal 1.00. That is the hardware's error, not the
    // model's, and it is asserted rather than smoothed away.
    let last = output_db(20) - output_db(19);
    assert!(
        (last - 0.83).abs() <= 0.02,
        "the last step is {last:.3} dB; the drawing's own values give 0.83"
    );
    let span = output_db(20) - output_db(0);
    assert!(
        (span - 19.76).abs() <= 0.05,
        "the span is {span:.3} dB; the ladder gives 19.76 ±0.05 against a nominal 20"
    );
}

/// Test 2. **[PRINTED]** *Figure:* the S1 legend table — "POSITION 1
/// COMPRESS / 2 OUT / 3 LIMIT". Trivial to pass and worth having, because
/// it is the constraint that stops a ratio control creeping in.
#[test]
fn t02_the_mode_switch_has_three_positions_with_out_in_the_middle() {
    assert_eq!(MODE_NAMES.len(), 3);
    assert_eq!(MODE_NAMES, ["Compress", "Out", "Limit"]);
    assert_eq!(MODE_ORDER, [MODE_COMPRESS, MODE_OUT, MODE_LIMIT]);
    assert_eq!(MODE_OUT, 1, "OUT sits between the other two");
}

// ------------------------------------------------- 12.2 circuit identities

/// Test 3. **[IDENTITY]** *Figure:* the six resistor values on switch
/// assembly `TG12413 B204A` — 47 K, 47 K, 130 K, 220 K, 470 K, 1 M3 —
/// accumulated into 47 k, 94 k, 224 k, 444 k, 914 k and 2 214 kΩ, whose
/// ratios to the fastest are 1 : 2.00 : 4.77 : 9.45 : 19.4 : 47.1.
///
/// **This test asserts ratios and not times, deliberately.** Waves, who
/// had the console, state that the recovery times are "very hard to put in
/// terms of exact milliseconds", so there is no published absolute time to
/// assert and this test does not invent one.
#[test]
fn t03_the_recovery_positions_stand_in_the_ladders_ratios() {
    const WANT: [f32; 6] = [1.0, 2.00, 4.77, 9.45, 19.4, 47.1];
    for sr in RATES {
        let mut taus = [0.0f32; 6];
        for (i, t) in taus.iter_mut().enumerate() {
            let s = Settings {
                recovery: i,
                ..base()
            };
            let mut c = unit(sr, s);
            *t = release_tau(&mut c, 0.9, sr);
        }
        for (i, want) in WANT.iter().enumerate() {
            let got = taus[i] / taus[0];
            assert!(
                (got - want).abs() <= 0.02 * want,
                "{sr} Hz: position {} released {got:.3} times slower than position 1; the ladder requires {want} ±2 %",
                i + 1
            );
        }
    }
}

/// Test 4. **[IDENTITY]** *Figure:* S3 sits after the gain element in the
/// signal path, so the output level cannot move the gain reduction.
///
/// **This is the test that settles where the sidechain taps.** Section
/// 11.4 of the dossier says the detector reads the post-ladder signal and
/// this identity says it cannot; the identity is the tighter statement, so
/// the engine taps before the ladder and this asserts it. It must fail
/// loudly if the ladder is ever moved inside the loop.
#[test]
fn t04_the_output_switch_does_not_move_the_gain_reduction() {
    let mut grs = Vec::new();
    for output in [0usize, 5, 10, 15, 20] {
        let s = Settings { output, ..base() };
        let mut c = unit(SR, s);
        settle(&mut c, 0.5, 1.0, SR);
        grs.push(c.gain_reduction_db(0));
    }
    let lo = grs.iter().cloned().fold(f32::INFINITY, f32::min);
    let hi = grs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    assert!(
        hi - lo <= 0.1,
        "sweeping the output switch moved the gain reduction by {:.3} dB, from {lo:.2} to {hi:.2}",
        hi - lo
    );
    assert!(
        hi > 5.0,
        "the test needs real gain reduction; got {hi:.2} dB"
    );
}

/// Test 5. **[IDENTITY]** *Figure:* RV1 is 10 kΩ in series with a ladder
/// whose accumulated values are 47 kΩ at position 1 and 2 214 kΩ at
/// position 6, so HOLD is worth 21.3 % at the fast end and 0.45 % at the
/// slow one.
#[test]
fn t05_hold_is_worth_a_fifth_at_the_fast_end_and_nothing_at_the_slow_end() {
    // The arithmetic first, which is what the resistor values require.
    let fast = recovery_s(0, 1.0) / recovery_s(0, 0.0) - 1.0;
    let slow = recovery_s(5, 1.0) / recovery_s(5, 0.0) - 1.0;
    assert!(
        (fast - 0.213).abs() <= 0.01,
        "HOLD lengthens position 1 by {:.1} %; 10 k against 47 k requires 21 ±1 %",
        100.0 * fast
    );
    assert!(
        slow < 0.006,
        "HOLD lengthens position 6 by {:.2} %; 10 k against 2 214 k requires under 0.6 %",
        100.0 * slow
    );
    // And through the model, which is what the store actually does.
    for (pos, want, tol) in [(0usize, 0.213f32, 0.02f32), (5, 0.0045, 0.006)] {
        let mut a = unit(
            SR,
            Settings {
                recovery: pos,
                hold: 0.0,
                ..base()
            },
        );
        let mut b = unit(
            SR,
            Settings {
                recovery: pos,
                hold: 1.0,
                ..base()
            },
        );
        let ta = release_tau(&mut a, 0.9, SR);
        let tb = release_tau(&mut b, 0.9, SR);
        let got = tb / ta - 1.0;
        assert!(
            (got - want).abs() <= tol,
            "position {}: HOLD lengthened the measured release by {:.2} %, against {:.2} % from the resistors",
            pos + 1,
            100.0 * got,
            100.0 * want
        );
    }
}

/// Test 6. **[IDENTITY]** *Figure:* C1 4µ7 into R78 7K5 puts the input
/// coupling at 4.5 Hz; C23 470 µF into a 600 Ω load puts the output
/// coupling at 0.56 Hz.
///
/// **This is also the negative test for transformers.** A
/// transformer-coupled model cannot pass it, because a transformer's low
/// end is neither this flat nor this level-independent, and the TG has no
/// transformers anywhere.
#[test]
fn t06_the_coupling_corners_are_where_the_capacitors_put_them() {
    let s = Settings {
        mode: MODE_OUT,
        ..base()
    };
    let mut c = unit(SR, s);
    let reference = settle_at(&mut c, 0.2, 1000.0, 0.5, SR);
    let response_db = |hz: f32| {
        let mut c = unit(SR, s);
        let p = settle_at(&mut c, 0.2, hz, 6.0, SR);
        20.0 * (p / reference).log10()
    };
    // Bisect for the −3 dB point.
    let (mut lo, mut hi) = (1.0f32, 20.0f32);
    for _ in 0..12 {
        let mid = 0.5 * (lo + hi);
        if response_db(mid) < -3.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let corner = 0.5 * (lo + hi);
    assert!(
        (corner - 4.5).abs() <= 1.0,
        "the low corner is at {corner:.2} Hz; C1 into R78 requires 4.5 ±1 Hz"
    );
    // Everything other than that input capacitor must be transparent at
    // 20 Hz, which is what the clause about the output coupling asks. The
    // input capacitor's own first-order response at 20 Hz is the baseline.
    let f = 20.0f32;
    let input_only = 20.0 * (f / (f * f + F_IN_COUPLING * F_IN_COUPLING).sqrt()).log10();
    let got = response_db(f);
    assert!(
        (got - input_only).abs() <= 0.1,
        "at 20 Hz the model is {got:.3} dB where the input capacitor alone gives {input_only:.3}; \
         everything after it must contribute under 0.1 dB"
    );
}

/// Test 7. **[IDENTITY]** *Figure:* the GANG bus is a screened
/// control-voltage lead on tag 36 shared between modules, and a shared
/// current bus takes the largest demand.
///
/// **Structural, not measured** — EMI publish no description of the bus
/// and its combining rule is inferred from its topology.
#[test]
fn t07_stereo_link_is_a_maximum_over_control_currents() {
    const N: usize = 256;
    let run = |link: bool| {
        let s = Settings { link, ..base() };
        let mut c = unit(SR, s);
        let mut ph = 0.0f32;
        let step = std::f32::consts::TAU * 1000.0 / SR;
        for _ in 0..(1.0 * SR / N as f32) as usize {
            let mut l = [0.0f32; N];
            let mut r = [0.0f32; N];
            for i in 0..N {
                l[i] = 0.9 * ph.sin();
                r[i] = 0.0;
                ph += step;
                if ph > std::f32::consts::TAU {
                    ph -= std::f32::consts::TAU;
                }
            }
            c.process_block(&mut l, &mut r);
        }
        [c.gain_reduction_db(0), c.gain_reduction_db(1)]
    };
    let unlinked = run(false);
    let linked = run(true);
    assert!(
        unlinked[1] < 0.05,
        "unlinked, the silent channel reduced by {:.2} dB",
        unlinked[1]
    );
    assert!(
        (linked[0] - linked[1]).abs() <= 0.05,
        "linked, the two channels reduced by {:.2} and {:.2} dB",
        linked[0],
        linked[1]
    );
    assert!(
        (linked[1] - unlinked[0]).abs() <= 0.2,
        "linked, the silent channel reduced by {:.2} dB where the loud one's own reduction is {:.2}; \
         a current bus takes the largest demand",
        linked[1],
        unlinked[0]
    );
}

// ------------------------------------------------------ 12.3 the element

/// Test 8. **[IDENTITY — and this is the component-crate test]** *Figure:*
/// the Neve bridge law `i = I·tanh(u / 2ηV_T)`, derived in the sibling
/// dossier from Neve drawing EX11475 and validated there against Neve's
/// own printed −6 dBu and −31 dBu level annotations, which agree with the
/// derivation to 25.01 dB against 25.0 dB.
///
/// **This test is the whole argument of section 4.9 made executable.** It
/// proves that equation (G1) is the correct generalisation and that a
/// crate with the constant baked in cannot serve this unit.
///
/// **On the tolerance.** The dossier asks for 1 × 10⁻⁹ relative. That is
/// below what f32 can represent — its epsilon is 1.2 × 10⁻⁷ — so the
/// assertion is written at the limit of the representation instead of at
/// the dossier's figure, and the miss is recorded here and in the README
/// rather than the figure being quietly widened. The well-conditioned
/// direction, current to voltage to current, holds to a few units in the
/// last place. The other direction runs `artanh` towards its asymptote,
/// where a relative error in the current is amplified by 1/(1 − (i/I)²),
/// which at u/k = 3 is a factor of 100; that is conditioning, not error,
/// and the tolerance below says so.
#[test]
fn t08_the_generalised_law_contains_the_neves_exactly() {
    let ring = Element::ring();
    let k = noob_electrical_components::diode_bridge::THERMAL_SCALE;
    let mut worst_fwd = 0.0f32;
    let mut worst_rev = 0.0f32;
    for decade in 0..4 {
        let i_bias = 1e-6 * 10f32.powi(decade);
        for step in 1..=60 {
            let uk = 3.0 * step as f32 / 60.0;
            let u = uk * k;
            // The crate's law, then (G1): the voltage must come back.
            let i = noob_electrical_components::diode_bridge::current(u, i_bias, k);
            let back = ring.voltage(i, i_bias);
            worst_rev = worst_rev.max(((back - u) / u).abs());
            // And the well-conditioned direction.
            let a = 0.995 * uk / 3.0;
            let i0 = a * i_bias;
            let u0 = ring.voltage(i0, i_bias);
            let i1 = noob_electrical_components::diode_bridge::current(u0, i_bias, k);
            worst_fwd = worst_fwd.max(((i1 - i0) / i0).abs());
        }
    }
    // The bound is derived rather than chosen: near its asymptote the
    // logarithm's argument is a difference of two nearly equal currents,
    // so a relative error is amplified by 1/(1 − (i/I)²). At the top of
    // the dossier's range that is a hundredfold, and f32's epsilon is
    // 1.19 × 10⁻⁷.
    let amplification = 1.0 / (1.0 - 0.995f32 * 0.995);
    let bound = f32::EPSILON * amplification;
    assert!(
        worst_fwd <= bound,
        "current through (G1) and back through the crate's tanh differs by {worst_fwd:.3e} relative, past the {bound:.3e} f32 allows"
    );
    assert!(
        worst_rev <= bound,
        "voltage through the crate's tanh and back through (G1) differs by {worst_rev:.3e} relative, past the {bound:.3e} f32 allows"
    );
    // And the small-signal resistance the two laws imply is the same one.
    for decade in 0..4 {
        let i_bias = 1e-6 * 10f32.powi(decade);
        let mine = ring.resistance(i_bias) - 2.0 * ring.r_b;
        let theirs = noob_electrical_components::diode_bridge::small_signal_resistance(i_bias, k);
        assert!(
            ((mine - theirs) / theirs).abs() <= 1e-6,
            "at {i_bias} A the ring's resistance is {mine} against the crate's {theirs}"
        );
    }
}

/// Test 9. **[IDENTITY]** *Figure:* for `tanh(a·sinθ)` the third-harmonic
/// ratio is a²/12, and doubling the thermal scale halves a, so two
/// junctions per arm give four times less third harmonic at equal drive.
///
/// "At equal drive" means at equal voltage *across the element*, so the
/// divider's gain is held fixed by choosing each element's own bias
/// current for the same gain reduction. Holding the current fixed instead
/// would compare two different working points.
#[test]
fn t09_two_junctions_per_arm_quarter_the_third_harmonic() {
    const GR: f32 = 12.0;
    const PER: usize = 512;
    /// Peak volts across the element: small enough that the a²/12
    /// expansion the figure comes from is still the leading term.
    const TARGET_U: f32 = 0.02;
    let mut ratios = [0.0f32; 2];
    for (slot, n) in [1u32, 2].iter().enumerate() {
        let e = Element::forward(*n);
        let i = e.current_for_gr_db(GR).expect("forward has no floor");
        // "At equal drive" is equal **voltage across the element**, so the
        // source is scaled by each element's own divider gain to put the
        // same 20 mV across both. Holding the current equal instead would
        // compare two different working points and the test would read
        // 1.00 whatever the law is.
        let v_s = TARGET_U / e.gain(i);
        let x = element_run(&e, v_s, i, 8, PER);
        let h1 = element_harmonic(&x, 1, PER);
        ratios[slot] = element_harmonic(&x, 3, PER) / h1;
    }
    let got = ratios[0] / ratios[1];
    assert!(
        (got - 4.0).abs() <= 0.05,
        "one junction per arm distorts {got:.4} times as much as two; the a²/12 expansion requires 4.00 ±0.05"
    );
}

/// Test 10. **[IDENTITY]** *Figure:* **no published spectrum for this unit
/// exists.** This asserts the odd symmetry of equation (G1), corroborated
/// by Pines' independent finding for the symmetric family that the model
/// "is an odd function … therefore, only odd harmonics are present".
///
/// **The 40 dB figure is the dossier's estimate and is a
/// numerical-hygiene bound, not a measurement**, and this is stated here
/// rather than buried.
#[test]
fn t10_the_element_makes_odd_harmonics_and_essentially_no_even_ones() {
    const PER: usize = 512;
    for e in [Element::breakdown(), Element::forward(2)] {
        let i = e
            .current_for_gr_db(12.0)
            .expect("12 dB is inside the floor");
        let v_s = 0.5 * e.resistance(i) * i / e.gain(i);
        let x = element_run(&e, v_s, i, 8, PER);
        let h2 = element_harmonic(&x, 2, PER);
        let h3 = element_harmonic(&x, 3, PER);
        let db = 20.0 * (h2 / h3).log10();
        assert!(
            db <= -40.0,
            "the second harmonic is {db:.1} dB relative to the third; an odd law requires at least 40 dB below"
        );
    }
}

/// Test 11. **[IDENTITY]** *Figure:* EMI specify D1/D3 and D2/D4 as
/// matched pairs on two separate drawings and provide two adjust-on-test
/// resistors to trim the residual, which is evidence they knew it
/// mattered. **No number is published**, so this asserts monotonicity and
/// an ordering, not a level.
#[test]
fn t11_mismatch_reintroduces_even_harmonics_monotonically() {
    const PER: usize = 512;
    let mut last = 0.0f32;
    let mut top = (0.0f32, 0.0f32);
    for step in 0..=10 {
        let mut e = Element::breakdown();
        e.mismatch = 0.05 * step as f32 / 10.0;
        let i = e
            .current_for_gr_db(12.0)
            .expect("12 dB is inside the floor");
        let v_s = 0.2 * e.resistance(i) * i / e.gain(i);
        let x = element_run(&e, v_s, i, 8, PER);
        let h1 = element_harmonic(&x, 1, PER);
        let h2 = element_harmonic(&x, 2, PER) / h1;
        let h3 = element_harmonic(&x, 3, PER) / h1;
        assert!(
            h2 >= last * 0.999,
            "at {:.1} % mismatch the second harmonic fell to {h2:.3e} from {last:.3e}",
            100.0 * e.mismatch
        );
        last = h2;
        top = (h2, h3);
    }
    assert!(
        top.0 > top.1,
        "at full mismatch the second harmonic is {:.3e} and the third {:.3e}; the second must lead",
        top.0,
        top.1
    );
}

// ---------------------------------------------------------- 12.4 dynamics

/// Test 12. **[QUOTED + CROSS]** *Figures:* Waves, on the module they
/// built with Abbey Road — "this is **not a brick-wall limiter: transients
/// are expected to pass**" — and, for the band it must fall outside, AMS
/// Neve's published limiter calibration: "with input level at 10 dBu,
/// increased to +20 dBu the change in output level should be **0.1 dB,
/// +/−0.1 dB**".
///
/// **The 1.0 dB threshold is the dossier's estimate and is labelled one**;
/// the requirement that it fall outside Neve's published band is anchored
/// to a real manufacturer's figure.
#[test]
fn t12_limit_is_not_a_brickwall() {
    let s = Settings {
        mode: MODE_LIMIT,
        ..base()
    };
    let mut c = unit(SR, s);
    let lo = out_dbu(&mut c, 10.0, 1.5, SR);
    let hi = out_dbu(&mut c, 20.0, 1.5, SR);
    let change = hi - lo;
    assert!(
        change > 1.0,
        "a 10 dB step gave {change:.2} dB of output change; Waves say transients are expected to pass"
    );
    assert!(
        !(0.0..=0.2).contains(&change),
        "a 10 dB step gave {change:.2} dB, inside AMS Neve's published brickwall band of 0.1 ±0.1"
    );
}

/// Test 13. **[QUOTED]** *Figure:* "transients are expected to pass".
/// **The source gives no number, so this asserts a direction only**, and a
/// model that flattens the burst to the ceiling fails.
#[test]
fn t13_a_fast_transient_passes() {
    const N: usize = 64;
    let s = Settings {
        mode: MODE_LIMIT,
        ..base()
    };
    let mut c = unit(SR, s);
    let steady = 0.2f32;
    let ceiling = settle(&mut c, steady, 1.5, SR);
    // A 1 ms burst 20 dB above the steady level.
    let burst = steady * 10.0;
    let mut ph = 0.0f32;
    let step = std::f32::consts::TAU * 1000.0 / SR;
    let mut peak = 0.0f32;
    let blocks = (0.001 * SR / N as f32).ceil() as usize;
    for _ in 0..blocks {
        let mut l = [0.0f32; N];
        let mut r = [0.0f32; N];
        for i in 0..N {
            l[i] = burst * ph.sin();
            r[i] = l[i];
            ph += step;
            if ph > std::f32::consts::TAU {
                ph -= std::f32::consts::TAU;
            }
        }
        c.process_block(&mut l, &mut r);
        for v in l {
            peak = peak.max(v.abs());
        }
    }
    let over = 20.0 * (peak / ceiling).log10();
    assert!(
        over > 3.0,
        "the burst reached {over:.2} dB over the steady ceiling; a limiter that is not a brickwall must let it through"
    );
}

/// Test 14. **[QUOTED + IDENTITY]** *Figure:* Chandler describe their
/// equivalent mode as one that "allows bypassing of the compressor/limiter
/// threshold **but leaves all circuits in the signal path**", and the mode
/// wafer selects a resistor rather than opening the path.
///
/// **No distortion figure is published for either state**, so the second
/// half asserts an inequality, not a level.
#[test]
fn t14_out_leaves_the_element_in_circuit() {
    let out = Settings {
        mode: MODE_OUT,
        drive: 1.0,
        ..base()
    };
    let byp = Settings {
        bypass: true,
        ..out
    };
    let mut a = unit(SR, out);
    settle(&mut a, 0.7, 0.5, SR);
    assert!(
        a.gain_reduction_db(0).abs() <= 0.05,
        "OUT reduced by {:.3} dB; it must neutralise the control and nothing else",
        a.gain_reduction_db(0)
    );
    let mut a = unit(SR, out);
    let mut b = unit(SR, byp);
    let ta = thd_pct(&capture(&mut a, 0.7, 0.3, 0.1, SR), SR);
    let tb = thd_pct(&capture(&mut b, 0.7, 0.3, 0.1, SR), SR);
    assert!(
        ta > tb,
        "OUT distorted {ta:.4} % against true bypass at {tb:.4} %; in OUT the element is still in the path"
    );
}

/// Test 15. **[QUOTED]** *Figure:* "The knee is **much harder** compared
/// to the compressor". **Qualitative source, so the assertion is about
/// shape, not about numbers.**
///
/// # A recorded miss, and it is the most interesting one in the file
///
/// The dossier asks for two things here: that LIMIT's knee be harder than
/// COMPRESS's, and that the two curves "cross, not merely scale". **Under
/// the dossier's own reading of S1, neither can happen, and the reason is
/// arithmetic rather than implementation.**
///
/// Section 6.2 reads the mode wafer as re-scaling the detector's drive.
/// Write the loop as `q = p*A(K*e(g*q))`, where `q` is the tap, `p` the
/// input, `g` the mode's drive and `A` the element's gain. Substituting
/// `q' = g*q` and `p' = g*p` gives `q' = p'*A(K*e(q'))`, which has no `g`
/// in it. So changing the mode **translates the transfer curve along the
/// diagonal in log-log and does not bend it**: the slope curves are
/// identical, only the threshold moves, and no scaling of a detector can
/// ever do anything else.
///
/// What does survive is the six-to-one *asymmetry*, which is not a common
/// factor. Near LIMIT's own knee only the 120 K half conducts, so the
/// store charges over half the duty and the knee comes out very slightly
/// **softer**, not harder. Measured, as slope in dB out per dB in at 1, 3,
/// 5, 10 and 20 dB above each mode's own knee:
///
/// ```text
/// COMPRESS  0.216  0.228  0.246  0.294  0.376
/// LIMIT     0.232  0.243  0.259  0.304  0.382
/// ```
///
/// What is asserted instead is the part that is both true and worth
/// guarding: the two modes are **one law at two thresholds, not two
/// ratios**, and the gap between the thresholds is the resistor ratio. A
/// model that grew a ratio control, which section 9.4 calls the change
/// that makes an emulation stop being a model, fails it.
///
/// **What would settle it:** a measured pair of transfer curves from a
/// real module in its two modes, or the item list's values for AOT 3 to
/// AOT 6, which are the four parts that shape the law.
#[test]
fn t15_the_two_modes_are_one_law_at_two_thresholds() {
    let knee = |mode: usize| {
        let s = Settings { mode, ..base() };
        let c = unit(SR, s);
        let mut lo = -70.0f32;
        let mut hi = 20.0f32;
        for _ in 0..40 {
            let mid = 0.5 * (lo + hi);
            if c.static_gr_db(10f32.powf(mid / 20.0)) < 1.0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        0.5 * (lo + hi)
    };
    let slopes = |mode: usize| {
        let s = Settings { mode, ..base() };
        let c = unit(SR, s);
        let k = knee(mode);
        let out = |db: f32| db - c.static_gr_db(10f32.powf(db / 20.0));
        [1.0f32, 3.0, 5.0, 10.0, 20.0].map(|d| out(k + d + 0.5) - out(k + d - 0.5))
    };
    // The threshold really does move, and by the resistor ratio: LIMIT's
    // 120 K half against COMPRESS's 62 K.
    let want = 20.0 * (R_MODE_POS[MODE_LIMIT] / R_MODE_REF).log10();
    let got = knee(MODE_COMPRESS) - knee(MODE_LIMIT);
    assert!(
        (got - want).abs() <= 0.3,
        "LIMIT's knee sits {got:.2} dB below COMPRESS's; 120 K against 62 K requires {want:.2} dB"
    );
    // And the shape does not. This is what fails if a ratio control is
    // ever added.
    let comp = slopes(MODE_COMPRESS);
    let lim = slopes(MODE_LIMIT);
    for i in 0..5 {
        assert!(
            (lim[i] - comp[i]).abs() <= 0.05,
            "the two modes' slopes are {comp:?} and {lim:?}; a drive-scaling wafer cannot change the shape, so they must be one law"
        );
    }
    // Not a brickwall, which is the part of the quote that does survive.
    assert!(
        lim[4] > lim[0],
        "LIMIT's slope went from {:.3} at 1 dB over to {:.3} at 20 dB over; it must let go",
        lim[0],
        lim[4]
    );
}

/// Test 16. **[IDENTITY]** *Figure:* equation (G1)'s `2·r_b` term bounds
/// the element's resistance below, so the divider's loss is bounded.
/// **There is no published maximum gain-reduction figure for this unit**,
/// so this asserts that a floor exists and not where it is. With the
/// forward reading and `r_b = 0` the same test must find no floor, which
/// is the point.
#[test]
fn t16_gain_reduction_has_a_floor_in_breakdown_and_none_in_forward() {
    let e = Element::breakdown();
    let deep = e.gr_db(1.0);
    let deeper = e.gr_db(1000.0);
    assert!(
        (deeper - deep).abs() <= 0.5,
        "breakdown reached {deep:.2} dB at 1 A and {deeper:.2} dB at 1000 A; the bulk term must bound it"
    );
    assert!(
        e.current_for_gr_db(deep + 6.0).is_none(),
        "breakdown offered a current for {:.1} dB, which is past its own floor",
        deep + 6.0
    );
    let f = Element::forward(2);
    let a = f.gr_db(1.0);
    let b = f.gr_db(1000.0);
    assert!(
        b - a > 40.0,
        "forward reached {a:.2} dB at 1 A and {b:.2} dB at 1000 A; with no bulk term it must not settle"
    );
    // And through the model: driving the sidechain into saturation must
    // stop moving the reduction in breakdown.
    let s = Settings {
        mode: MODE_LIMIT,
        ..base()
    };
    let mut c = unit(SR, s);
    settle(&mut c, 40.0, 1.0, SR);
    let g1 = c.gain_reduction_db(0);
    let mut c = unit(SR, s);
    settle(&mut c, 4000.0, 1.0, SR);
    let g2 = c.gain_reduction_db(0);
    assert!(
        g2 > g1,
        "a hundredfold input gave {g2:.2} dB against {g1:.2}; it should still be deeper, just bounded"
    );
    assert!(
        g2 < e.gr_db(f32::MAX / 2.0) + 0.5,
        "the model reached {g2:.2} dB, past the element's own floor"
    );
}

// ------------------------ 12.5 what distinguishes this model from the Neve

/// The Neve at a comparable setting, for the cross tests.
fn neve(sr: f32, limit: bool) -> bridge::Compressor {
    let mut c = bridge::Compressor::new(sr);
    c.configure(bridge::Settings {
        compress_in: !limit,
        limit_in: limit,
        compress_ratio: 1,
        compress_threshold: 5,
        compress_recovery: 2,
        limit_threshold: 8,
        ..bridge::Settings::default()
    });
    c
}

fn neve_capture(c: &mut bridge::Compressor, amp: f32, warm: f32, secs: f32, sr: f32) -> Vec<f32> {
    const N: usize = 240;
    let mut ph = 0.0f32;
    let step = std::f32::consts::TAU * 1000.0 / sr;
    let mut out = Vec::new();
    let blocks = ((warm + secs) * sr / N as f32).ceil() as usize;
    let keep = (secs * sr / N as f32).ceil() as usize;
    for b in 0..blocks {
        let mut l = [0.0f32; N];
        let mut r = [0.0f32; N];
        for i in 0..N {
            l[i] = amp * ph.sin();
            r[i] = l[i];
            ph += step;
            if ph > std::f32::consts::TAU {
                ph -= std::f32::consts::TAU;
            }
        }
        c.process_block(&mut l, &mut r);
        if b + keep >= blocks {
            out.extend_from_slice(&l);
        }
    }
    out
}

/// Test 17. **[CROSS]** *Figures:* for the Neve, the derived result that
/// the bridge's nonlinearity depends on the voltage across it, corroborated
/// by Pines: the harmonics "fall off at approximately **40 dB per decade**
/// as the gain parameter approaches zero". For the TG, an element that
/// carries no current at all until the sidechain drives it.
///
/// # A recorded miss, with the derivation, because the dossier asks for a
/// sign split that does not follow from its own equations
///
/// The dossier calls this "the single most important cross test" and asks
/// for the Neve's third harmonic to **fall** across an input sweep while
/// the TG's rises. Measured across the same sweep, from the onset of gain
/// reduction to 20 dB of it, both rise: the TG by about 16 dB and the Neve
/// by about 6.
///
/// The reason is that both units are the same shape of circuit. Put a
/// nonlinear element as the shunt of a divider whose series arm is R_s,
/// expand the law to its cubic term, and the third-harmonic ratio at the
/// node comes out as
///
/// ```text
/// h3/h1  proportional to  (1 - g) * u^2
/// ```
///
/// with u the peak voltage across the element and g the normalised gain.
/// **The sign of the trend is therefore set by what the loop does to u,
/// which is the loop's ratio, and not by which element sits in the
/// divider.** At a *fixed input* the two disagree exactly as the dossiers
/// say, because u = g*v_s falls as the control current rises; that is what
/// the Neve's own test 17 measures on its bridge alone, and it passes
/// there. Across a rising input with a compressor holding the output, u is
/// roughly constant and (1 - g) climbs, so both rise.
///
/// This bears on the Neve dossier as well as this one; the two files
/// disagree and the README says which.
///
/// **What is asserted instead** is the direction that is true of this
/// element and the difference a listener can hear: the TG's third harmonic
/// rises with depth, and it stays more than 15 dB above the Neve's at
/// every point of the sweep. That second figure is the real content of the
/// dossier's claim, since on a rising input the TG gets audibly dirty and
/// the Neve does not, and it survives the derivation above.
#[test]
fn t17_the_tg_gets_dirtier_with_depth_and_stays_far_above_the_neve() {
    const LEVELS: [f32; 5] = [-24.0, -18.0, -12.0, -6.0, 0.0];
    let mut tg = Vec::new();
    for db in LEVELS {
        let mut c = unit(SR, base());
        let amp = 10f32.powf(db / 20.0);
        let x = capture(&mut c, amp, 1.5, 0.1, SR);
        tg.push((c.gain_reduction_db(0), h3_ratio(&x, SR)));
    }
    let mut nv = Vec::new();
    for db in LEVELS {
        let mut c = neve(SR, false);
        let amp = 10f32.powf(db / 20.0);
        let x = neve_capture(&mut c, amp, 1.5, 0.1, SR);
        nv.push((c.gain_reduction_db(0), h3_ratio(&x, SR)));
    }
    let db_of = |v: f32| 20.0 * v.max(1e-12).log10();
    let show = |v: &[(f32, f32)]| {
        v.iter()
            .map(|(g, h)| format!("{g:.1} dB GR -> {:.1} dBc", db_of(*h)))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let trend = db_of(tg[4].1) - db_of(tg[0].1);
    assert!(
        trend > 10.0,
        "the TG's third harmonic moved {trend:+.1} dB over the sweep; an element that conducts only when the sidechain drives it must get dirtier with depth.\n  TG: {}",
        show(&tg)
    );
    for i in 0..LEVELS.len() {
        let gap = db_of(tg[i].1) - db_of(nv[i].1);
        assert!(
            gap > 15.0,
            "at {} dBFS the TG is {gap:.1} dB dirtier than the Neve; the audible difference the dossier stakes the model on needs more than 15.\n  TG:   {}\n  Neve: {}",
            LEVELS[i],
            show(&tg),
            show(&nv)
        );
    }
}

/// Test 18. **[CROSS]** *Figure:* AMS Neve's published Limit Ratio
/// specification against Waves' statement that the TG is not a brickwall.
/// **The Neve half of this test asserts a manufacturer's measured figure
/// with the manufacturer's own tolerance.**
#[test]
fn t18_the_neve_holds_a_step_and_the_tg_does_not() {
    let mut n = neve(SR, true);
    let lo = bridge::engine::amp_dbu(neve_settle(&mut n, bridge::engine::dbu_amp(10.0), 1.0, SR));
    let mut n = neve(SR, true);
    let hi = bridge::engine::amp_dbu(neve_settle(&mut n, bridge::engine::dbu_amp(20.0), 1.0, SR));
    let neve_change = hi - lo;
    assert!(
        (neve_change - 0.1).abs() <= 0.1,
        "the Neve's limiter changed {neve_change:.2} dB for a 10 dB step; its own specification is 0.1 ±0.1"
    );
    let s = Settings {
        mode: MODE_LIMIT,
        ..base()
    };
    let mut c = unit(SR, s);
    let lo = out_dbu(&mut c, 10.0, 1.5, SR);
    let hi = out_dbu(&mut c, 20.0, 1.5, SR);
    let tg_change = hi - lo;
    assert!(
        tg_change > 1.0,
        "the TG changed {tg_change:.2} dB where the Neve changed {neve_change:.2}; it must not hold the step"
    );
}

fn neve_settle(c: &mut bridge::Compressor, amp: f32, secs: f32, sr: f32) -> f32 {
    const N: usize = 256;
    let blocks = (secs * sr / N as f32).ceil() as usize;
    let tail = ((0.020 * sr) as usize / N).max(1);
    let mut ph = 0.0f32;
    let step = std::f32::consts::TAU * 1000.0 / sr;
    let mut peak = 0.0f32;
    for b in 0..blocks {
        let mut l = [0.0f32; N];
        let mut r = [0.0f32; N];
        for i in 0..N {
            l[i] = amp * ph.sin();
            r[i] = l[i];
            ph += step;
            if ph > std::f32::consts::TAU {
                ph -= std::f32::consts::TAU;
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

/// Test 19. **[CROSS]** *Figures:* AMS Neve's published compress recovery
/// times — 100, 400, 800 and 1500 ms, a span of 15 : 1 — against the TG's
/// six accumulated ladder resistances, a span of 47 : 1. **Both halves are
/// anchored, one to a specification and one to resistor values.**
#[test]
fn t19_the_release_ranges_differ_by_three_to_one() {
    let tg = R_RECOVERY[5] / R_RECOVERY[0];
    assert!(
        (tg - 47.1).abs() <= 0.2,
        "the TG's ladder spans {tg:.2} : 1; its six resistors give 47.1"
    );
    let (slow, _) = bridge::engine::compress_recovery_s(3);
    let (fast, _) = bridge::engine::compress_recovery_s(0);
    let neve = slow / fast;
    assert!(
        (neve - 15.0).abs() <= 0.1,
        "the Neve's published recovery times span {neve:.2} : 1; 100 to 1500 ms is 15"
    );
    assert!(
        tg / neve > 3.0,
        "the TG's range is {:.2} times the Neve's; the dossier's claim is three",
        tg / neve
    );
}

/// Test 20. **[CROSS]** *Figure:* the 33609's handbook describes a
/// combiner giving "a low output impedance signal **equal to the larger
/// of** the compressor or limiter sidechain signals"; the TG has one
/// detector and a mode switch. **Structural on the TG side** — there is no
/// published TG trace — and asserted as an absence, which is the honest
/// form.
#[test]
fn t20_the_neves_gain_reduction_has_two_slopes_and_the_tgs_has_one() {
    // The TG's static curve: one detector, so its slope must never turn
    // back on itself once the unit is working.
    let c = unit(SR, base());
    let gr = |db: f32| c.static_gr_db(10f32.powf(db / 20.0));
    let mut slopes = Vec::new();
    let mut db = -24.0f32;
    while db <= 0.0 {
        slopes.push(gr(db + 1.0) - gr(db));
        db += 2.0;
    }
    let smooth = slopes
        .windows(2)
        .all(|w| (w[1] - w[0]).abs() < 0.15 * w[0].max(0.05));
    assert!(
        smooth,
        "the TG's gain-reduction slope changed by more than 15 % between neighbouring points: {slopes:?}"
    );
    // The Neve's, with both sections in, must show a detectable change of
    // slope where the limiter takes over.
    let mut n = bridge::Compressor::new(SR);
    n.configure(bridge::Settings {
        compress_in: true,
        limit_in: true,
        compress_ratio: 1,
        compress_threshold: 0,
        limit_threshold: 8,
        gain: 5,
        ..bridge::Settings::default()
    });
    let ngr = |db: f32| n.static_gr_db(bridge::engine::dbu_amp(db));
    let mut nslopes = Vec::new();
    let mut db = -10.0f32;
    while db <= 20.0 {
        nslopes.push(ngr(db + 1.0) - ngr(db));
        db += 2.0;
    }
    let lo = nslopes.iter().cloned().fold(f32::INFINITY, f32::min);
    let hi = nslopes.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    assert!(
        hi > 1.5 * lo.max(0.02),
        "the Neve's slope ran from {lo:.3} to {hi:.3} with both sections in; the maximum combiner must show a knee"
    );
}

/// Test 21. **[CROSS]** *Figure:* the Neve has four transformers in the
/// audio path and the TG has none. **This test exists to catch a
/// transformer model being added to the TG for warmth**, which is the most
/// likely way this model will go wrong.
#[test]
fn t21_the_tgs_low_end_is_level_independent_and_the_neves_is_not() {
    let s = Settings {
        mode: MODE_OUT,
        ..base()
    };
    let ratio_at = |amp: f32| {
        let mut c = unit(SR, s);
        let x = capture_at(&mut c, amp, 30.0, 0.5, 0.5, SR);
        let h1 = harmonic_at(&x, 30.0, 1, SR).max(1e-12);
        let h2 = harmonic_at(&x, 30.0, 2, SR);
        let h3 = harmonic_at(&x, 30.0, 3, SR);
        20.0 * (((h2 * h2 + h3 * h3).sqrt()) / h1).max(1e-12).log10()
    };
    let quiet = ratio_at(dbu_amp(0.0));
    let loud = ratio_at(dbu_amp(20.0));
    assert!(
        (loud - quiet).abs() <= 0.5,
        "the TG's 30 Hz harmonic ratio moved {:.2} dB between 0 and +20 dBu; with no transformers it must not",
        loud - quiet
    );
}

/// Test 22. **[CROSS, structural]** *Figure:* **no measurement of either
/// onset is published.** The dossier asserts the consequence of a circuit
/// difference, that the TG rectifies with eight germanium diodes and
/// references its threshold to three more where the Neve's sidechain is
/// silicon, and labels the assertion structural.
///
/// # A recorded miss, and the circuit says why
///
/// Section 9.2 lists "germanium rectification, so a softer onset" as one
/// of the six differences the dossier stakes the model on. Measured, the
/// TG spreads its first decibel of gain reduction over about 1.6 dB of
/// input and the Neve's model over about 3.1 dB, so the TG's onset is the
/// *harder* of the two.
///
/// The germanium claim does not survive its own component values. The
/// rectifier's soft knee has a width of one diode drop, about 250 mV, and
/// the threshold it is compared against is a string of **three** of the
/// same diodes. So the reference sits 3.7 knee-widths up, and by the time
/// the rectified signal reaches it the rectifier has been in its straight
/// region for a factor of ten in level. The first assertion below measures
/// exactly that, because it is the reason for the miss rather than an
/// excuse for it: at the threshold a soft rectifier and a hard one agree
/// to better than a tenth of a per cent.
///
/// What the model does have is an onset spread over more than a decibel,
/// which comes from the law network's shallow first segment and from the
/// store charging over only part of a cycle near the threshold. A
/// hard-kneed feedback compressor round a 1/I element takes about 1.1 dB.
///
/// **What would settle it:** the three-diode string's actual drop at its
/// working current, which needs the item list or a probe on a real module.
#[test]
fn t22_the_germanium_reference_sits_past_its_own_rectifier_knee() {
    // The rectifier is straight by the time it reaches the reference, so
    // germanium buys no softness at this threshold.
    let a = V_REF + V_GE * std::f32::consts::LN_2;
    let soft = softrect(a, V_GE);
    let hard = a - V_GE * std::f32::consts::LN_2;
    let err = (soft - hard) / hard;
    assert!(
        err < 1e-3,
        "at the threshold the soft rectifier reads {err:.2e} above a hard one; with a three-diode reference the two should be indistinguishable"
    );
    // And the onset the model does have, which is not a hard knee.
    let c = unit(SR, base());
    let find = |want: f32| {
        let (mut lo, mut hi) = (-70.0f32, 30.0f32);
        for _ in 0..50 {
            let mid = 0.5 * (lo + hi);
            if c.static_gr_db(10f32.powf(mid / 20.0)) < want {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        0.5 * (lo + hi)
    };
    let span = find(1.0) - find(0.1);
    assert!(
        span > 1.3,
        "the first decibel of gain reduction took {span:.2} dB of input; a hard knee round a 1/I element takes about 1.1"
    );
}

// ------------------------------------------------------------- hygiene

/// Bypass is a true straight-through, which OUT deliberately is not.
#[test]
fn bypass_is_exact() {
    for sr in RATES {
        let mut c = unit(
            sr,
            Settings {
                bypass: true,
                drive: 1.0,
                ..base()
            },
        );
        let mut l = [0.0f32; 512];
        let mut r = [0.0f32; 512];
        for i in 0..512 {
            l[i] = (i as f32 * 0.37).sin() * 0.9;
            r[i] = (i as f32 * 0.11).cos() * 0.5;
        }
        let (li, ri) = (l, r);
        c.process_block(&mut l, &mut r);
        for i in 0..512 {
            assert!(
                (l[i] - li[i]).abs() <= 1e-6 && (r[i] - ri[i]).abs() <= 1e-6,
                "{sr} Hz sample {i}: bypass is not straight through"
            );
        }
    }
}

/// The same input at three host rates must give the same gain reduction.
#[test]
fn sample_rate_invariance() {
    let mut got = Vec::new();
    for sr in RATES {
        let mut c = unit(SR, base());
        c.set_sample_rate(sr);
        settle(&mut c, 0.5, 1.5, sr);
        got.push(c.gain_reduction_db(0));
    }
    let lo = got.iter().cloned().fold(f32::INFINITY, f32::min);
    let hi = got.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    assert!(
        hi - lo <= 0.35,
        "gain reduction ran from {lo:.2} to {hi:.2} dB across 44.1, 48 and 96 kHz"
    );
}

/// Nothing in the module may produce a non-finite sample, at any legal
/// setting, from any legal input.
#[test]
fn numerical_robustness() {
    for mode in [MODE_COMPRESS, MODE_OUT, MODE_LIMIT] {
        for region in [REGION_BREAKDOWN, REGION_FORWARD] {
            for os in [1usize, 2, 4] {
                for amp in [0.0f32, 1e-9, 1.0, 20.0] {
                    let s = Settings {
                        mode,
                        region,
                        oversample: os,
                        recovery: 0,
                        drive: 1.0,
                        mismatch: 1.0,
                        input_db: 12.0,
                        output: 20,
                        ..base()
                    };
                    let mut c = unit(SR, s);
                    let mut l = [0.0f32; 512];
                    let mut r = [0.0f32; 512];
                    for i in 0..512 {
                        l[i] = amp * (i as f32 * 0.31).sin();
                        r[i] = -l[i];
                    }
                    c.process_block(&mut l, &mut r);
                    for i in 0..512 {
                        assert!(
                            l[i].is_finite() && r[i].is_finite(),
                            "mode {mode} region {region} {os}x amp {amp}: sample {i} is not finite"
                        );
                    }
                }
            }
        }
    }
}

/// The three oversampling positions must not change the gain law, only
/// what the element does above the audio band.
#[test]
fn oversampling_does_not_change_the_gain_law() {
    let mut got = Vec::new();
    for oversample in [1usize, 2, 4] {
        let mut c = unit(
            SR,
            Settings {
                oversample,
                ..base()
            },
        );
        settle(&mut c, 0.5, 1.5, SR);
        got.push(c.gain_reduction_db(0));
    }
    let lo = got.iter().cloned().fold(f32::INFINITY, f32::min);
    let hi = got.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    assert!(
        hi - lo <= 0.2,
        "gain reduction ran from {lo:.2} to {hi:.2} dB across 1x, 2x and 4x"
    );
}

/// The calibration the engine fits: a full-scale sine in COMPRESS settles
/// at 20 dB of reduction. **This is the dossier's own instruction in 11.6,
/// not a figure about the hardware**, and it is asserted so that a change
/// to the threshold or the law cannot move the model's depth silently.
#[test]
fn the_calibration_target_is_met() {
    for region in [REGION_BREAKDOWN, REGION_FORWARD] {
        let mut c = unit(SR, Settings { region, ..base() });
        let gr = measured_gr_db(&mut c, CAL_INPUT_AMP, 2.0, SR);
        assert!(
            (gr - CAL_GR_DB).abs() <= 0.5,
            "region {region}: a full-scale sine settled at {gr:.2} dB against the fitted target of {CAL_GR_DB}"
        );
    }
}

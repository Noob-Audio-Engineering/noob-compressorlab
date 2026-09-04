//! The dbx 160's test plan, `research/dbx-160.md` section 12.
//!
//! **Every test that exists to check a real figure asserts that figure and
//! names its source.** Where dbx published nothing, the test says so and
//! asserts a direction, an ordering or a circuit identity instead of an
//! invented bound. Two of the tests here assert figures from the
//! descendant part's datasheet rather than from dbx, and both say so at
//! the assertion.

use super::engine::*;
use super::*;

const SR: f32 = 48_000.0;

fn unit(sr: f32, f: impl FnOnce(&mut Settings)) -> Compressor {
    let mut c = Compressor::new(sr);
    let mut s = Settings::default();
    f(&mut s);
    c.configure(s);
    c
}

/// A sine of peak `amp` at `hz`, `n` samples.
fn sine(hz: f32, amp: f32, n: usize, sr: f32) -> Vec<f32> {
    (0..n)
        .map(|i| amp * (std::f32::consts::TAU * hz * i as f32 / sr).sin())
        .collect()
}

/// Run a mono signal through and return the output.
fn run(c: &mut Compressor, x: &[f32]) -> Vec<f32> {
    let mut l = x.to_vec();
    let mut r = x.to_vec();
    c.process_block(&mut l, &mut r);
    l
}

/// Settle the unit on a steady sine and return the mean gain reduction of
/// the last block.
fn settled_gr(c: &mut Compressor, hz: f32, amp: f32, sr: f32) -> f32 {
    let n = (sr * 0.05) as usize;
    let mut phase = 0.0f32;
    let mut gr = 0.0;
    for _ in 0..40 {
        let mut l: Vec<f32> = (0..n)
            .map(|i| amp * (std::f32::consts::TAU * hz * (phase + i as f32) / sr).sin())
            .collect();
        phase += n as f32;
        let mut r = l.clone();
        c.process_block(&mut l, &mut r);
        gr = c.gain_reduction_db(0);
    }
    gr
}

/// Peak amplitude of a sine at `dbu`, at the default headroom.
fn dbu_peak(dbu: f32) -> f32 {
    10f32.powf((dbu - HEADROOM_DEFAULT_DB) / 20.0) * std::f32::consts::SQRT_2
}

/// RMS level in dBFS of a block.
fn rms_dbfs(x: &[f32]) -> f32 {
    let p: f64 = x.iter().map(|v| (*v as f64) * (*v as f64)).sum::<f64>() / x.len() as f64;
    10.0 * p.max(1e-30).log10() as f32
}

/// Amplitude of the `k`-th harmonic of `hz` in `x`, by a Goertzel-style
/// direct correlation (the block is an exact number of cycles).
fn harmonic(x: &[f32], hz: f32, k: u32, sr: f32) -> f32 {
    let w = std::f32::consts::TAU * hz * k as f32 / sr;
    let (mut re, mut im) = (0.0f64, 0.0f64);
    for (i, v) in x.iter().enumerate() {
        let t = w * i as f32;
        re += *v as f64 * t.cos() as f64;
        im += *v as f64 * t.sin() as f64;
    }
    let n = x.len() as f64;
    (2.0 / n * (re * re + im * im).sqrt()) as f32
}

/// Settle on a sine, then measure the harmonics of a whole number of
/// cycles of the steady output. Returns `(fundamental, h2, h3)`.
fn harmonics(c: &mut Compressor, hz: f32, amp: f32, sr: f32) -> (f32, f32, f32) {
    // Settle for two seconds, which is fifty release-rate time constants.
    let block = (sr * 0.05) as usize;
    let mut phase = 0.0f32;
    for _ in 0..40 {
        let mut l: Vec<f32> = (0..block)
            .map(|i| amp * (std::f32::consts::TAU * hz * (phase + i as f32) / sr).sin())
            .collect();
        phase += block as f32;
        let mut r = l.clone();
        c.process_block(&mut l, &mut r);
    }
    // An exact number of cycles, so the correlation needs no window.
    let cycles = 40.0f32.max((hz * 0.25).round());
    let n = (cycles * sr / hz).round() as usize;
    let mut l: Vec<f32> = (0..n)
        .map(|i| amp * (std::f32::consts::TAU * hz * (phase + i as f32) / sr).sin())
        .collect();
    let mut r = l.clone();
    c.process_block(&mut l, &mut r);
    (
        harmonic(&l, hz, 1, sr),
        harmonic(&l, hz, 2, sr),
        harmonic(&l, hz, 3, sr),
    )
}

// ================================================= 12.1 static behaviour

/// 1. Unity gain with no compression.
///
/// Figure: a circuit identity. R26 and R32 are **both 100 kΩ**, so the
/// transimpedance stage exactly undoes the input resistor at zero control
/// voltage (the 160 schematic). dbx document the setting too: "To use
/// either model as a line amplifier, adjust COMPRESSION RATIO to its
/// maximum counterclockwise position ('1:1'), THRESHOLD to its maximum
/// clockwise position ('3V')".
#[test]
fn t01_unity_at_1_to_1() {
    let mut c = unit(SR, |s| {
        s.alpha = 0.0;
        s.threshold_dbu = 20.0;
    });
    let x = sine(1000.0, 0.25, 4096, SR);
    // Prime the filters, then measure.
    run(&mut c, &x);
    let y = run(&mut c, &x);
    // Above the 10.6 Hz corner, so the only departure is that corner's own
    // phase shift and the cell's residual, both far below a hundredth of a
    // decibel at 1 kHz.
    let err = rms_dbfs(&y) - rms_dbfs(&x);
    assert!(err.abs() < 0.01, "unity gain off by {err} dB");
}

/// 2. Output gain is exact over its whole range.
///
/// Figure: "Output Level Adjust (Continuous) — ±20 dB from unity gain
/// point", and the schematic marks R80's track ends "−20db" and "+20db".
/// The ±0.05 dB is mine; dbx publish no tolerance and a DC potentiometer
/// across a trimmed rail deserves a tight one.
#[test]
fn t02_output_gain_is_exact() {
    let x = sine(1000.0, 0.25, 4096, SR);
    let base = {
        let mut c = unit(SR, |s| {
            s.alpha = 0.0;
            s.threshold_dbu = 20.0;
        });
        run(&mut c, &x);
        rms_dbfs(&run(&mut c, &x))
    };
    for step in -20..=20 {
        let g = step as f32;
        let mut c = unit(SR, |s| {
            s.alpha = 0.0;
            s.threshold_dbu = 20.0;
            s.output_db = g;
        });
        run(&mut c, &x);
        let got = rms_dbfs(&run(&mut c, &x)) - base;
        assert!((got - g).abs() < 0.05, "output gain {g} dB gave {got} dB");
    }
}

/// 3. Output gain does not move the threshold.
///
/// Figure: "The OUTPUT GAIN control **does not interact with the threshold
/// of compression**" (the 160A manual, and the same sentence in the 160X
/// manual). A structural test: it fails immediately if anyone wires the
/// make-up before the detector tap.
#[test]
fn t03_output_gain_does_not_move_the_threshold() {
    let mut grs = vec![];
    for g in [-20.0f32, -10.0, 0.0, 10.0, 20.0] {
        let mut c = unit(SR, |s| {
            s.threshold_dbu = -10.0;
            s.alpha = 0.75;
            s.output_db = g;
        });
        grs.push(settled_gr(&mut c, 1000.0, dbu_peak(30.0), SR));
    }
    let spread =
        grs.iter().cloned().fold(f32::MIN, f32::max) - grs.iter().cloned().fold(f32::MAX, f32::min);
    assert!(
        spread < 0.05,
        "gain reduction moved by {spread} dB: {grs:?}"
    );
}

/// 4. The threshold dial is logarithmic at 10 dB per mark, and its ends
///    are dbx's own.
///
/// Figure: "Continuously variable from **10mV(−38dB) to 3V(+12dB)**", and
/// the factory procedure "Step the oscillator up and down in **10 db
/// steps** verifying that the threshold level matches the input signal at
/// successive calibration marks on the threshold dial". This is the
/// primary calibration of the model, because it is dbx's own.
#[test]
fn t04_threshold_marks_are_ten_decibels_apart() {
    let mut onsets = vec![];
    for mark in THRESHOLD_MARK_DBU {
        let c = unit(SR, |s| {
            s.threshold_dbu = mark;
            s.alpha = 1.0;
        });
        // Where gain reduction first reaches 0.1 dB, found by bisection on
        // the static curve.
        let (mut lo, mut hi) = (mark - 20.0, mark + 20.0);
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if c.static_gr_db(dbu_peak(mid)) < 0.1 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        onsets.push(0.5 * (lo + hi));
        // Silence the unused-mut warning about `c`.
        let _ = c.settings();
    }
    for w in onsets.windows(2) {
        let step = w[1] - w[0];
        assert!(
            (step - 10.0).abs() < 0.5,
            "successive marks are {step} dB apart, not 10"
        );
    }
    // The endpoints, against dbx's printed −38 dB and +12 dB. The onset is
    // 0.1 dB of reduction rather than exactly zero, so it sits a hair
    // above the mark.
    assert!(
        (onsets[0] - (-37.78)).abs() < 0.5,
        "the 10 mV mark is at {} dBu, not −38",
        onsets[0]
    );
    assert!(
        (onsets[5] - 11.76).abs() < 0.5,
        "the 3 V mark is at {} dBu, not +12",
        onsets[5]
    );
}

/// 5. The threshold indicators, including the both-dim case.
///
/// Figure: "A steady-state, sine-wave tone **exactly at the threshold
/// voltage** causes both L.E.D.'s to remain **dimly illuminated**", and
/// the factory procedure calibrates the threshold by turning the control
/// "until both led's are off".
#[test]
fn t05_threshold_lamps() {
    let at = {
        let mut c = unit(SR, |s| s.threshold_dbu = 0.0);
        settled_gr(&mut c, 1000.0, dbu_peak(0.0), SR);
        c.threshold_lamps()
    };
    assert!(
        (at.0 - at.1).abs() < 0.1 && at.0 > 0.3 && at.1 > 0.3,
        "at threshold the lamps read {at:?}, which is not both dim and equal"
    );
    let below = {
        let mut c = unit(SR, |s| s.threshold_dbu = 0.0);
        settled_gr(&mut c, 1000.0, dbu_peak(-3.0), SR);
        c.threshold_lamps()
    };
    assert!(below.0 > 0.99 && below.1 < 0.01, "below reads {below:?}");
    let above = {
        let mut c = unit(SR, |s| s.threshold_dbu = 0.0);
        settled_gr(&mut c, 1000.0, dbu_peak(3.0), SR);
        c.threshold_lamps()
    };
    assert!(above.0 < 0.01 && above.1 > 0.99, "above reads {above:?}");
}

/// 6. The hard-knee ratio is exact at every printed mark.
///
/// Figure: "Hard-knee: **COMPRESSION RATIO setting defines exact
/// compression ratio**" (the 1981 brochure's table and the 160X manual's
/// section 1.4). dbx publish no tolerance; the ±0.15 dB over the 20 dB
/// span is mine. The 4:1 row is weighted hardest because the schematic
/// carries a trimmer labelled "4:1 CAL", R43, so 4:1 is the point the
/// factory actually calibrated.
#[test]
fn t06_hard_knee_ratios_are_exact() {
    // The eight marks past 1:1, with the output change dbx's law requires
    // for a 20 dB input rise: 20/R.
    let cases: [(usize, f32); 7] = [
        (1, 20.0 / 1.5),
        (2, 10.0),
        (3, 20.0 / 3.0),
        (4, 5.0),
        (5, 20.0 / 6.0),
        (6, 2.0),
        (7, 1.0),
    ];
    for (i, want) in cases {
        let alpha = RATIO_MARK_ALPHA[i];
        let c = unit(SR, |s| {
            s.threshold_dbu = -20.0;
            s.alpha = alpha;
        });
        let a = c.static_gr_db(dbu_peak(0.0));
        let b = c.static_gr_db(dbu_peak(20.0));
        // Output change = input change − extra reduction.
        let got = 20.0 - (b - a);
        assert!(
            (got - want).abs() < 0.15,
            "{} gave {got} dB out for 20 dB in, not {want}",
            RATIO_MARK_LABELS[i]
        );
    }
}

/// 7. The infinity mark is 120:1, not infinity.
///
/// Figure: "Continuously variable from **1:1 to 120:1 (infinity)**",
/// restated in the same manual's introduction as "infinite compression
/// (**approximately 120:1**)". 40 dB ÷ 120 = 0.333 dB. The tolerance is
/// asymmetric on purpose: a model producing *less* than 0.33 dB has built
/// a brick wall dbx did not, which is the failure this test exists to
/// catch, and it is the single test that separates modelling the circuit
/// from modelling the silkscreen.
#[test]
fn t07_infinity_is_one_hundred_and_twenty_to_one() {
    let c = unit(SR, |s| {
        s.threshold_dbu = -40.0;
        s.alpha = 1.0;
    });
    let a = c.static_gr_db(dbu_peak(-20.0));
    let b = c.static_gr_db(dbu_peak(20.0));
    let rise = 40.0 - (b - a);
    assert!(
        (0.0..=0.48).contains(&rise) && rise >= 0.33 - 0.33,
        "40 dB in lifted the output by {rise} dB"
    );
    assert!(
        rise >= 0.30,
        "40 dB in lifted the output by only {rise} dB: dbx publish 120:1, not a brick wall"
    );
    assert!(
        rise <= 0.48,
        "40 dB in lifted the output by {rise} dB, over 0.33 + 0.15"
    );
}

/// 8. Negative ratios invert.
///
/// Figure: "At a setting of −1:1, the above threshold input signal must
/// increase by 1dB in level to **decrease** the signal at the output of
/// the 160A by 1dB" (the 160A manual). The −2:1 and −5:1 rows follow from
/// α = 1 − 1/R.
#[test]
fn t08_infinity_plus_inverts() {
    for (alpha, want_fall) in [(2.0f32, 10.0f32), (1.5, 5.0), (1.2, 2.0)] {
        let c = unit(SR, |s| {
            s.model = MODEL_160A;
            s.threshold_dbu = -40.0;
            s.alpha = alpha;
        });
        // Well above threshold but shallow enough that the 60 dB ceiling
        // is not what is being measured: at α = 2 an excess of 15 dB is
        // already 30 dB of reduction.
        let a = c.static_gr_db(dbu_peak(-35.0));
        let b = c.static_gr_db(dbu_peak(-25.0));
        // Output change for a 10 dB input rise; negative means it fell.
        let change = 10.0 - (b - a);
        assert!(
            (change + want_fall).abs() < 0.3,
            "α = {alpha} gave {change} dB out for 10 dB in, not −{want_fall}"
        );
    }
}

/// 9. Maximum compression reaches 60 dB.
///
/// Figure: "**over 60dB maximum compression**" (the 160A specification,
/// and ">60 dB" on the 160X).
#[test]
fn t09_sixty_decibels_of_compression() {
    let c = unit(SR, |s| {
        s.model = MODEL_160A;
        s.threshold_dbu = -40.0;
        s.alpha = 1.0;
    });
    // At dbx's own published maximum input for the 160A, +24 dBu, with the
    // threshold at its published lowest: the claim is about what the box
    // can do, so the measurement belongs at the ends of both ranges. At
    // 60 dB of excess the residual slope of the ∞ mark leaves 59.5, which
    // is the circuit being right rather than the model falling short.
    let gr = c.static_gr_db(dbu_peak(24.0));
    assert!(gr >= 60.0, "deepest reduction was {gr} dB, not 60");
}

/// 10. Infinity+ and OverEasy are unreachable on the original.
///
/// Figure: the 160's published ratio range stops at "120:1 (infinity)",
/// and OverEasy is "the classic 'Hard Knee' curve popularized by the
/// **original dbx 160, 161 and 162**", i.e. the original has the hard knee
/// and not the switch. The composite this model is (research 2.5) is the
/// easiest thing in the file to get wrong, which is why this test exists.
#[test]
fn t10_the_original_has_neither() {
    let mut c = Compressor::new(SR);
    c.configure(Settings {
        model: MODEL_160,
        alpha: 2.0,
        knee: KNEE_OVEREASY,
        threshold_dbu: 19.0,
        ..Settings::default()
    });
    let s = *c.settings();
    assert_eq!(s.alpha, 1.0, "the original reached α = {}", s.alpha);
    assert_eq!(s.knee, KNEE_HARD, "the original took the OverEasy switch");
    assert!(
        s.threshold_dbu <= THRESHOLD_160_MAX_DBU + 1e-4,
        "the original's threshold reached {} dBu, past its 3 V mark",
        s.threshold_dbu
    );
}

// ============================================ 12.2 the detector is RMS

/// 11. A sine settles 3.01 dB below its peak.
///
/// Figure: a mathematical identity — the RMS of a sine is 1/√2 of its
/// peak — against dbx's "**True rms level-detection**". The cheapest
/// possible test that nobody has quietly substituted a peak detector.
///
/// The model lands on 3.01, because [`D_DB`] is `10/ln 10` exactly rather
/// than the quotient of two datasheet figures that carry different
/// ideality assumptions. That quotient, 4.246, would leave a sine 2.98 dB
/// below its peak; the argument for the exact value is at [`D_DB`], and
/// the component that owns it asserts the same identity directly on the
/// detector, without a compressor around it.
#[test]
fn t11_a_sine_settles_three_decibels_below_its_peak() {
    let mut c = unit(SR, |s| {
        s.alpha = 0.0;
        s.threshold_dbu = 20.0;
    });
    let amp = 0.25f32;
    settled_gr(&mut c, 1000.0, amp, SR);
    let got = c.detector_db(0);
    let want = 20.0 * amp.log10() - 3.01;
    assert!(
        (got - want).abs() < 0.15,
        "the detector settled at {got} dBFS, not {want}"
    );
}

/// 12. Crest-factor under-reading, at the three published points.
///
/// Figure: the crest-factor table "1 ms pulse repetition rate; 0.2 dB
/// error — 3.5; 0.5 dB error — 5; 1.0 dB error — 8". **The source is the
/// THAT 2252 datasheet, the descendant part, not dbx**, who publish no
/// crest-factor figure at all; the substitution is argued in research 5.1
/// and named here.
#[test]
fn t12_crest_factor_under_reading() {
    // Run at 96 kHz, where the model does not oversample, because a
    // 63-tap interpolating filter smears a one-sample pulse and would set
    // the crest factor itself rather than measuring it.
    let sr = 96_000.0f32;
    // A rectangular train of `duty` samples in `period`: the crest factor
    // is exactly √(period/duty), so both are chosen to make each of the
    // datasheet's three factors exact rather than rounded. The repetition
    // rates that come out are 1.02, 1.04 and 1.33 ms against the
    // datasheet's 1 ms; nothing here is within a factor of thirty of the
    // detector's own time constant, so that spread does not matter.
    let cases: [(usize, usize, f32, f32); 3] =
        [(98, 8, 3.5, 0.2), (100, 4, 5.0, 0.5), (128, 2, 8.0, 1.0)];
    let mut errors = vec![];
    for (period, duty, cf, _) in cases {
        assert!(
            ((period as f32 / duty as f32).sqrt() - cf).abs() < 1e-4,
            "the {cf} case is not that crest factor"
        );
        let rms = 0.1f32;
        let amp = rms * cf;
        let x: Vec<f32> = (0..period * 600)
            .map(|i| if i % period < duty { amp } else { 0.0 })
            .collect();
        let true_rms_db = rms_dbfs(&x);
        let mut c = unit(sr, |s| {
            s.alpha = 0.0;
            s.threshold_dbu = 20.0;
        });
        for _ in 0..2 {
            for chunk in x.chunks(2048) {
                run(&mut c, chunk);
            }
        }
        errors.push(true_rms_db - c.detector_db(0));
    }
    // **The direction and the ordering, which is the research's own stated
    // fallback, and the magnitudes are a recorded miss.** The detector
    // under-reads high-crest material and under-reads it more as the crest
    // factor rises, which is the property the box's sound rests on. It
    // under-reads by 0.06 to 0.08 dB where the datasheet's part reads low
    // by 0.2 to 1.0, and the gap is not in the averaging: with the decibel
    // unit at 10/ln 10 the log-domain filter's steady-state reading of a
    // pulse train is the true mean square by construction, and what is
    // left in the real part is its own input bandwidth, which the
    // datasheet publishes as four corner frequencies against input current
    // and not as a transfer function. The research declined to model that
    // and so does this. `README.md` carries the row.
    for w in errors.windows(2) {
        assert!(
            w[1] > w[0],
            "the under-reading did not grow with crest factor: {errors:?}"
        );
    }
    for got in &errors {
        assert!(
            *got > 0.0,
            "the detector read high on peaky material: {errors:?}"
        );
    }
    for ((.., want), got) in cases.iter().zip(errors.iter()) {
        assert!(
            got < want,
            "the model under-reads by {got} dB where the datasheet's part reads low by {want};              this is the recorded miss and it is in the wrong direction"
        );
    }
}

/// 13. A peak detector would read very differently.
///
/// **No published number exists.** This asserts a direction and a
/// magnitude that follows from the definitions, and is a sanity check on
/// the ghost trace rather than evidence about the hardware.
#[test]
fn t13_the_ghost_trace_reads_higher() {
    let period = (SR * 1e-3) as usize;
    let duty = (period as f32 / 64.0).round() as usize;
    let x: Vec<f32> = (0..period * 400)
        .map(|i| if i % period < duty { 0.8 } else { 0.0 })
        .collect();
    let mut c = unit(SR, |s| {
        s.threshold_dbu = -60.0;
        s.alpha = 1.0;
    });
    for chunk in x.chunks(2048) {
        run(&mut c, chunk);
    }
    let rms_gr = c.gain_reduction_db(0);
    let ghost = c.ghost_gr_db();
    assert!(
        ghost - rms_gr >= 6.0,
        "the peak ghost asked for {ghost} dB against the RMS detector's {rms_gr}"
    );
}

/// 14. The detector responds to power, not to sum.
///
/// Figure: a circuit identity from "**True RMS Power Summing**" and from
/// the squaring action of the detector's two diode junctions. Doubling the
/// power is 3.01 dB by arithmetic.
#[test]
fn t14_the_detector_adds_power() {
    let mut one = unit(SR, |s| {
        s.alpha = 0.0;
        s.threshold_dbu = 20.0;
        s.link = true;
    });
    let x = sine(1000.0, 0.25, 4096, SR);
    // Left driven, right silent.
    for _ in 0..30 {
        let mut l = x.clone();
        let mut r = vec![0.0; x.len()];
        one.process_block(&mut l, &mut r);
    }
    let a = one.detector_db(0);
    // Both driven.
    let mut two = unit(SR, |s| {
        s.alpha = 0.0;
        s.threshold_dbu = 20.0;
        s.link = true;
    });
    for _ in 0..30 {
        let mut l = x.clone();
        let mut r = x.clone();
        two.process_block(&mut l, &mut r);
    }
    let b = two.detector_db(0);
    assert!(
        (b - a - 3.01).abs() < 0.2,
        "two channels read {} dB above one, not 3.01",
        b - a
    );
}

/// 35. Antiphase channels do not cancel.
///
/// Figure: dbx sum the RMS energies "to prevent phase cancellation of the
/// two signals from causing unmusical compressor action". A one-line test
/// that catches the wrong implementation instantly.
#[test]
fn t35_antiphase_channels_do_not_cancel() {
    let x = sine(1000.0, 0.25, 4096, SR);
    let read = |flip: bool| {
        let mut c = unit(SR, |s| {
            s.alpha = 0.0;
            s.threshold_dbu = 20.0;
            s.link = true;
        });
        for _ in 0..30 {
            let mut l = x.clone();
            let mut r: Vec<f32> = x.iter().map(|v| if flip { -*v } else { *v }).collect();
            c.process_block(&mut l, &mut r);
        }
        c.detector_db(0)
    };
    let d = read(true) - read(false);
    assert!(d.abs() < 0.2, "antiphase read {d} dB away from in phase");
}

/// 36. Unlinked channels are independent.
///
/// Figure: dbx's own warning that a strapped pair does *not* operate
/// independently because "the 'ring' of the strapping cable still sums the
/// audio", and that "the strapping cable must be removed for proper
/// single-channel operation". The model reproduces that by having no cable
/// at all.
#[test]
fn t36_unlinked_channels_are_independent() {
    let x = sine(1000.0, 0.5, 4096, SR);
    let mut c = unit(SR, |s| {
        s.threshold_dbu = -30.0;
        s.alpha = 1.0;
        s.link = false;
    });
    for _ in 0..30 {
        let mut l = x.clone();
        let mut r = vec![0.0; x.len()];
        c.process_block(&mut l, &mut r);
    }
    assert!(
        c.gain_reduction_db(0) > 10.0,
        "the driven channel did nothing"
    );
    assert!(
        c.gain_reduction_db(1) < 0.001,
        "the silent channel reduced by {} dB",
        c.gain_reduction_db(1)
    );
}

// ================================================== 12.3 the ballistics

/// Time in milliseconds for the gain reduction to reach 63 % of its final
/// value after a step of `step_db`, measured as dbx define it: "time
/// required to reduce signal by 63 % of level increase (above threshold)",
/// "measured in the infinite compression region of the threshold curve".
fn attack_ms(step_db: f32, tau_s: f32, sr: f32) -> f32 {
    let mut c = unit(sr, |s| {
        s.threshold_dbu = -60.0;
        s.alpha = 1.0;
        s.tau_s = tau_s;
    });
    let hz = 1000.0;
    let lo = dbu_peak(0.0);
    let hi = lo * 10f32.powf(step_db / 20.0);
    // Settle on the quiet level.
    for _ in 0..40 {
        let x = sine(hz, lo, (sr * 0.05) as usize, sr);
        run(&mut c, &x);
    }
    let before = c.gain_reduction_db(0);
    let after = before + step_db;
    let target = before + 0.63 * step_db;
    // Step, one sample at a time so the block size cannot set the answer.
    let n = (sr * 0.2) as usize;
    let mut hit = None;
    for i in 0..n {
        let mut l = [hi * (std::f32::consts::TAU * hz * i as f32 / sr).sin()];
        let mut r = l;
        c.process_block(&mut l, &mut r);
        if hit.is_none() && c.gain_reduction_db(0) >= target {
            hit = Some(i as f32 / sr * 1e3);
        }
    }
    let _ = after;
    // dbx measured an analogue box with nothing in front of it. The model
    // reports its resampler's group delay as latency and a host puts it
    // back, so it is subtracted here too rather than being counted as part
    // of the detector's response.
    let latency_ms = c.latency() as f32 / sr * 1e3;
    hit.map(|t| t - latency_ms).unwrap_or(f32::INFINITY)
}

/// The attack time this detector's own components imply for a step, from
/// the closed-form solution of the log-domain filter.
///
/// `t/τ = ln[(1 − e^(−u)) / (1 − e^(−0.37u))]` with `u = Δ/D`, which is
/// dbx's own definition of attack — the time to 63 % of the level increase
/// — solved rather than measured. Nothing about the model enters it: it is
/// [`D_DB`], which is `10/ln 10`, and [`TAU_DEFAULT_S`], which is R35 and
/// C15 off the drawing.
fn attack_from_components_ms(step_db: f32) -> f32 {
    let u = step_db / D_DB;
    let r = (1.0 - (-u).exp()) / (1.0 - (-0.37 * u).exp());
    r.ln() * TAU_DEFAULT_S * 1e3
}

/// 15. The three published attack times.
///
/// Figure: "Program-Dependent; Typically **15 ms for 10 dB, 5 ms for
/// 20 dB, 3 ms for 30 dB**", identical in the 160, 160X, 160XT and 160A
/// specifications, with the definition "time required to reduce signal by
/// **63 % of level increase** (above threshold)" measured "in the infinite
/// compression region of the threshold curve".
///
/// **The tolerance and why it is what it is.** dbx's own three figures are
/// mutually inconsistent: each implies a different detector time constant,
/// spanning 27 to 40 ms, so no single-constant model — and the hardware is
/// one — can hit all three. ±30 % is the spread of dbx's own data. The
/// 20 dB point is the binding one.
#[test]
fn t15_the_published_attack_times() {
    for (step, want) in [(10.0f32, 15.0f32), (30.0, 3.0)] {
        let got = attack_ms(step, TAU_DEFAULT_S, SR);
        let err = (got - want).abs() / want;
        assert!(
            err <= 0.30,
            "a {step} dB step attacked in {got} ms against dbx's {want} ms ({:.0} % out)",
            err * 100.0
        );
    }
}

/// 15b. The 20 dB attack point, which is a **recorded miss**.
///
/// dbx publish **5 ms** for a 20 dB step. The model takes about 6.7 ms,
/// which is 35 % slow, and **the gap cannot be closed**. Every quantity in
/// the detector is fixed by something published: the decibel unit is
/// `10/ln 10` exactly, because any other value stops the averaging being
/// an average of the square and the box's whole claim is true RMS; and the
/// time constant is R35 and C15 off dbx's own drawing, which puts the
/// release rate between dbx's own two published rates. Between them they
/// leave no free parameter.
///
/// The reason is in dbx's own numbers rather than in the model. Their three
/// attack figures each imply a different time constant — 33.3 ms for the
/// 10 dB step, 26.2 ms for the 20 dB step and 37.6 ms for the 30 dB step —
/// so no single-constant detector can satisfy all three, and the hardware
/// is a single-constant detector. Meeting the 20 dB figure would mean
/// giving up either the release rate or the true-RMS property.
///
/// So this asserts the figure that **is** derivable from published
/// quantities — the closed-form attack time of a filter built from R35, C15
/// and `10/ln 10` — and states the miss against dbx's 5 ms rather than
/// widening a band until it swallows it. `README.md` carries the row.
#[test]
fn t15b_the_twenty_decibel_attack_is_a_recorded_miss() {
    let got = attack_ms(20.0, TAU_DEFAULT_S, SR);
    let from_components = attack_from_components_ms(20.0);
    assert!(
        (from_components - 6.74).abs() < 0.1,
        "R35, C15 and 10/ln10 imply {from_components} ms, not 6.74"
    );
    assert!(
        (got - from_components).abs() / from_components <= 0.05,
        "the model attacked in {got} ms where its own components imply {from_components}"
    );
    // And the miss itself, stated so it cannot drift unnoticed.
    let miss = (got - 5.0) / 5.0;
    assert!(
        (0.25..=0.45).contains(&miss),
        "the model is {:.0} % away from dbx's published 5 ms; the recorded miss is 35 %",
        miss * 100.0
    );
}

/// 16. Attack really does get faster with bigger steps.
///
/// Figure: dbx's 15 / 5 / 3 ms is a 5:1 spread. Asserting the ordering and
/// a conservative bound tests the programme dependence itself,
/// independently of the absolute values, and fails loudly if anyone
/// substitutes a fixed-time-constant attack.
#[test]
fn t16_attack_is_programme_dependent() {
    let a = attack_ms(10.0, TAU_DEFAULT_S, SR);
    let b = attack_ms(20.0, TAU_DEFAULT_S, SR);
    let c = attack_ms(30.0, TAU_DEFAULT_S, SR);
    assert!(
        a > b && b > c,
        "attack times {a}, {b}, {c} are not decreasing"
    );
    assert!(a / c >= 4.0, "the 30 dB step is only {}× faster", a / c);
}

/// Release: drive to deep reduction, remove the signal, and sample the
/// gain reduction against time.
fn release_curve(sr: f32, tau_s: f32, alpha: f32) -> Vec<(f32, f32)> {
    let mut c = unit(sr, |s| {
        s.threshold_dbu = -60.0;
        s.alpha = alpha;
        s.tau_s = tau_s;
    });
    for _ in 0..60 {
        let x = sine(1000.0, dbu_peak(20.0), (sr * 0.05) as usize, sr);
        run(&mut c, &x);
    }
    let mut out = vec![];
    let n = (sr * 1.0) as usize;
    for i in 0..n {
        let mut l = [0.0f32];
        let mut r = [0.0f32];
        c.process_block(&mut l, &mut r);
        out.push((i as f32 / sr, c.gain_reduction_db(0)));
    }
    out
}

/// 17. The release is a straight line in decibels.
///
/// Figures: "**RELEASE RATE — 120dB/second**" (the 160) and "**125dB/Sec
/// rate**" (the 160A). The straight-line assertion is the important half,
/// because it is the structural claim: a log-domain filter releases at a
/// constant dB/s and an ordinary RC does not. A model with an exponential
/// release passes the slope test near one point and fails the linearity
/// test everywhere.
#[test]
fn t17_the_release_is_a_straight_line() {
    let curve = release_curve(SR, TAU_DEFAULT_S, 1.0);
    let band: Vec<(f32, f32)> = curve
        .iter()
        .cloned()
        .filter(|(_, gr)| (5.0..=35.0).contains(gr))
        .collect();
    assert!(band.len() > 100, "only {} points in the band", band.len());
    // Least squares.
    let n = band.len() as f32;
    let (sx, sy): (f32, f32) = band.iter().fold((0.0, 0.0), |a, p| (a.0 + p.0, a.1 + p.1));
    let (mx, my) = (sx / n, sy / n);
    let (sxy, sxx): (f32, f32) = band.iter().fold((0.0, 0.0), |a, p| {
        (a.0 + (p.0 - mx) * (p.1 - my), a.1 + (p.0 - mx) * (p.0 - mx))
    });
    let slope = sxy / sxx;
    let rate = -slope;
    let resid = band
        .iter()
        .map(|(t, gr)| (gr - (my + slope * (t - mx))).abs())
        .fold(0.0f32, f32::max);
    assert!(
        resid < 0.5,
        "the release departs from a straight line by {resid} dB"
    );
    assert!(
        (120.0..=125.0).contains(&rate),
        "the release rate is {rate} dB/s, outside dbx's 120 to 125"
    );
}

/// 18. The three published release times are one rate.
///
/// Figure: "Program Dependent; Typically **8ms for 1dB, 80ms for 10dB,
/// 400ms for 50dB**" (the 160A). These are 1, 10 and 50 divided by
/// 125 dB/s to the digit, so this is a consistency check on test 17 — and
/// if it passes while 17's linearity assertion fails, the implementation
/// has hard-coded three numbers instead of one rate.
#[test]
fn t18_the_three_published_release_times() {
    let curve = release_curve(SR, TAU_DEFAULT_S, 1.0);
    let start = curve[0].1;
    let time_to_recover = |db: f32| -> f32 {
        curve
            .iter()
            .find(|(_, gr)| *gr <= start - db)
            .map(|(t, _)| t * 1e3)
            .unwrap_or(f32::INFINITY)
    };
    for (db, want) in [(1.0f32, 8.0f32), (10.0, 80.0), (50.0, 400.0)] {
        if start < db {
            continue;
        }
        let got = time_to_recover(db);
        let err = (got - want).abs() / want;
        assert!(
            err <= 0.20,
            "recovering {db} dB took {got} ms against dbx's {want} ms"
        );
    }
}

/// 19. Attack and release are locked.
///
/// Figure: "How fast the 2252 acquires a signal (the 'attack'), and how
/// fast it returns to rest following a signal (the 'release'), are
/// **locked in relationship to each other** ... **Separate attack and
/// release adjustments are not possible** within the constraint of rms
/// response" (the THAT 2252 datasheet, the descendant part), and dbx's own
/// "no manual attack/release adjustments are required". This is the test
/// that proves the model has one time constant rather than two dressed up
/// as one.
#[test]
fn t19_attack_and_release_are_locked() {
    let a1 = attack_ms(10.0, TAU_DEFAULT_S, SR);
    let a2 = attack_ms(10.0, TAU_DEFAULT_S * 2.0, SR);
    assert!(
        ((a2 / a1) - 2.0).abs() < 0.1,
        "doubling τ multiplied the attack by {}",
        a2 / a1
    );
    let rate = |tau: f32| {
        let c = release_curve(SR, tau, 1.0);
        let a = c.iter().find(|(_, gr)| *gr <= 35.0).unwrap();
        let b = c.iter().find(|(_, gr)| *gr <= 10.0).unwrap();
        (a.1 - b.1) / (b.0 - a.0)
    };
    let r1 = rate(TAU_DEFAULT_S);
    let r2 = rate(TAU_DEFAULT_S * 2.0);
    assert!(
        ((r1 / r2) - 2.0).abs() < 0.1,
        "doubling τ divided the release rate by {}",
        r1 / r2
    );
    // The product of the attack time and the release rate is the invariant.
    let p1 = a1 * r1;
    let p2 = a2 * r2;
    assert!(
        ((p1 / p2) - 1.0).abs() < 0.05,
        "the attack × rate product moved from {p1} to {p2}"
    );
}

/// 20. Release does not depend on the ratio.
///
/// **No published number.** A circuit identity: the ratio coefficient α
/// scales the detector's output *after* the filter, so it cannot change
/// the filter's rate. Recorded as a structural test.
#[test]
fn t20_release_does_not_depend_on_the_ratio() {
    let rate_at = |alpha: f32| {
        let c = release_curve(SR, TAU_DEFAULT_S, alpha);
        let top = c[0].1;
        let a = c.iter().find(|(_, gr)| *gr <= top * 0.8).unwrap();
        let b = c.iter().find(|(_, gr)| *gr <= top * 0.2).unwrap();
        (a.1 - b.1) / (b.0 - a.0) / alpha
    };
    let r = [rate_at(0.5), rate_at(0.75), rate_at(1.0)];
    let spread = (r.iter().cloned().fold(f32::MIN, f32::max)
        - r.iter().cloned().fold(f32::MAX, f32::min))
        / r[2];
    assert!(
        spread < 0.05,
        "the detector's rate moved by {spread} across ratios: {r:?}"
    );
}

// ======================================================== 12.4 the knee

/// Local slope of the static curve at an input level, dB out per dB in.
fn slope_at(c: &Compressor, dbu: f32) -> f32 {
    let d = 0.05;
    let a = -c.static_gr_db(dbu_peak(dbu - d));
    let b = -c.static_gr_db(dbu_peak(dbu + d));
    1.0 + (b - a) / (2.0 * d)
}

/// 21. The hard knee is genuinely hard.
///
/// Figure: "a **mathematically precise** 'hard' threshold — at any
/// compression ratio selected", and "In Hard Knee mode, the threshold of
/// compression is defined as that point above which the output level **no
/// longer changes on a 1:1 basis** with changes in the input level". The
/// tight tolerance is justified by the mechanism: the diode sits inside a
/// feedback loop, so the corner is sharpened by the amplifier's open-loop
/// gain.
#[test]
fn t21_the_hard_knee_is_hard() {
    let c = unit(SR, |s| {
        s.threshold_dbu = 0.0;
        s.alpha = 0.75;
        s.knee = KNEE_HARD;
    });
    let below = slope_at(&c, -0.5);
    let above = slope_at(&c, 0.5);
    assert!(
        (below - 1.0).abs() < 0.002,
        "below threshold the slope is {below}"
    );
    assert!(
        (above - 0.25).abs() < 0.002,
        "above threshold the slope is {above}"
    );
}

/// 22. OverEasy has a continuously rising slope.
///
/// Figure: dbx's OverEasy patent, US 4,182,993, states the design
/// objective as "an input-output characteristic curve which is a function
/// having a **continuous slope or first derivative** ... and a
/// **gradually increasing compression ratio** between these two portions
/// of the curve".
#[test]
fn t22_overeasy_has_a_continuously_rising_slope() {
    let c = unit(SR, |s| {
        s.model = MODEL_160A;
        s.threshold_dbu = 0.0;
        s.alpha = 0.75;
        s.knee = KNEE_OVEREASY;
        s.knee_width_db = KNEE_WIDTH_DEFAULT_DB;
    });
    let pts: Vec<f32> = (0..20)
        .map(|i| slope_at(&c, -20.0 + 40.0 * i as f32 / 19.0))
        .collect();
    for w in pts.windows(2) {
        assert!(
            w[1] <= w[0] + 1e-4,
            "the slope rose from {} to {} instead of falling",
            w[0],
            w[1]
        );
        // Continuous: no step bigger than the sampling of the sweep can
        // account for. The whole slope change is α over the knee, and
        // twenty samples across forty decibels cannot show more than a
        // fraction of it per step.
        assert!(
            (w[0] - w[1]) < 0.25,
            "the slope jumped from {} to {}",
            w[0],
            w[1]
        );
    }
    // The two asymptotes. **The research asks for 2 % at ±20 dB and its own
    // recommended 6 dB knee width makes that unreachable**: the logistic is
    // still 3.4 % of the way in at 20 dB below threshold, so the two
    // numbers come from different sections of the same document and
    // contradict each other. The patent's claim is about the limits, not
    // about where they are reached, so 2 % is kept and measured at ±30 dB,
    // where a 6 dB knee has resolved to 0.5 %.
    assert!(
        (slope_at(&c, -30.0) - 1.0).abs() < 0.02,
        "30 dB below threshold the slope is {}",
        slope_at(&c, -30.0)
    );
    assert!(
        (slope_at(&c, 30.0) - 0.25).abs() < 0.02,
        "30 dB above threshold the slope is {}",
        slope_at(&c, 30.0)
    );
}

/// 23. The ratio control is a maximum in OverEasy and an exact value in
///     hard knee.
///
/// Figure: "OverEasy: ... **COMPRESSION RATIO control determines maximum
/// compression ratio**" against "Hard-knee: COMPRESSION RATIO setting
/// defines **exact** compression ratio".
#[test]
fn t23_overeasy_ratio_is_a_maximum() {
    let c = unit(SR, |s| {
        s.model = MODEL_160A;
        s.threshold_dbu = 0.0;
        s.alpha = 0.75;
        s.knee = KNEE_OVEREASY;
    });
    for i in 0..80 {
        let dbu = -30.0 + 60.0 * i as f32 / 79.0;
        let sl = slope_at(&c, dbu);
        assert!(sl >= 0.25 - 1e-3, "the slope reached {sl} at {dbu} dBu");
        if dbu < 10.0 {
            assert!(
                sl > 0.25,
                "the slope had already reached its limit at {dbu} dBu"
            );
        }
    }
}

/// 24. The threshold sits in the middle of the knee.
///
/// Figure: "the THRESHOLD setting corresponds to a point on the
/// input/output transfer curve **midway between the onset of processing
/// and that point at which the transfer curve corresponds to the setting
/// of the RATIO control**".
///
/// **This asserts dbx's sentence rather than the research's arithmetic for
/// it.** dbx say the threshold is a point midway between two *places on
/// the transfer curve*, which is a statement about the input axis: it sits
/// halfway between where processing begins and where the ratio is
/// attained. The research turns that into a band of 40 to 60 % of a *gain
/// reduction*, and its own figure for the value at threshold — α·w·ln 2 —
/// gives 23.5 % of that reference, so the two halves of its own section
/// disagree. The reading below is exact, needs no invented band, and is
/// what the words say.
#[test]
fn t24_the_threshold_is_the_middle_of_the_knee() {
    for alpha in [0.5f32, 0.75, 1.0] {
        for width in [3.0f32, 6.0, 9.0] {
            let c = unit(SR, |s| {
                s.model = MODEL_160A;
                s.threshold_dbu = 0.0;
                s.alpha = alpha;
                s.knee = KNEE_OVEREASY;
                s.knee_width_db = width;
            });
            let a = effective_alpha(alpha);
            let mut onset = f32::NAN;
            let mut attained = f32::NAN;
            for i in 0..2000 {
                let dbu = -50.0 + i as f32 * 0.05;
                let done = (1.0 - slope_at(&c, dbu)) / a;
                if onset.is_nan() && done >= 0.05 {
                    onset = dbu;
                }
                if done >= 0.95 {
                    attained = dbu;
                    break;
                }
            }
            let middle = 0.5 * (onset + attained);
            assert!(
                middle.abs() < 0.2,
                "at α = {alpha}, w = {width} the knee runs {onset} to {attained} dBu, \
                 whose middle is {middle} and not the 0 dBu threshold"
            );
        }
    }
}

/// 25. OverEasy emphasises the slap and reduces the body.
///
/// Figure: "In **OverEasy mode, the 160A takes slightly longer to react
/// than in Hard Knee mode**, and will therefore **emphasize the slap at the
/// beginning of the note** and reduce the boominess of its body" (the 160A
/// manual's kick-drum application note). The only published sentence
/// anywhere predicting an audible consequence of the knee switch, so the
/// test asserts its two observable clauses as directions.
///
/// **Run at dbx's own settings for that sentence**, because they are part
/// of the claim: the same note lists "Kick drum — 6:1, threshold for 15 dB
/// GR, OverEasy", and a kick starts from near silence between hits. Both
/// halves matter. At 6:1 the body sits well above threshold where the two
/// curves have converged, and from silence the slow RMS detector has barely
/// moved during the first few milliseconds, so both modes pass the slap
/// almost untouched and the difference is all in the body. Starting instead
/// from a steady tone a few decibels below threshold — inside the OverEasy
/// knee, where it is already a decibel down — reverses the result, and
/// would be testing dbx's sentence at settings dbx did not give for it.
///
/// **One clause is not asserted, and is a recorded tension.** Read as a
/// time to 63 % of the final gain reduction, "takes slightly longer to
/// react" does not hold for this knee and cannot hold for any knee centred
/// on the threshold: the softplus reaches any given depth of reduction at a
/// *lower* detector level than the hard knee does, so it gets there sooner.
/// Centring the knee on the threshold is not a choice here — it is dbx's
/// own definition of what the THRESHOLD control points at (test 24) — so
/// where a definition and a sentence of application prose disagree the
/// model follows the definition. `README.md` records it.
#[test]
fn t25_overeasy_emphasises_the_slap() {
    // dbx's own kick-drum recipe: 6:1, and the threshold set for 15 dB of
    // gain reduction on the body.
    let alpha = RATIO_MARK_ALPHA[5];
    let body_dbu = 0.0f32;
    let threshold_dbu = body_dbu - 15.0 / alpha;
    let measure = |knee: usize| {
        let mut c = unit(SR, |s| {
            s.model = MODEL_160A;
            s.threshold_dbu = threshold_dbu;
            s.alpha = alpha;
            s.knee = knee;
        });
        // Silence between hits, long enough for the detector to fall away.
        for _ in 0..40 {
            run(&mut c, &vec![0.0f32; (SR * 0.05) as usize]);
        }
        // The slap: the first three milliseconds of the hit.
        let hit = dbu_peak(body_dbu);
        let n = (SR * 0.003) as usize;
        let slap = rms_dbfs(&run(&mut c, &sine(1000.0, hit, n, SR)));
        // The body: where the note settles.
        for _ in 0..40 {
            run(&mut c, &sine(1000.0, hit, (SR * 0.05) as usize, SR));
        }
        let body = rms_dbfs(&run(&mut c, &sine(1000.0, hit, (SR * 0.05) as usize, SR)));
        (slap - body, c.gain_reduction_db(0))
    };
    let (hard_slap, hard_gr) = measure(KNEE_HARD);
    let (easy_slap, easy_gr) = measure(KNEE_OVEREASY);
    // The clause that holds: the body is more reduced.
    assert!(
        easy_gr > hard_gr,
        "OverEasy held the body at {easy_gr} dB of reduction against hard knee's {hard_gr}"
    );
    // The clause that does not, measured and left visible rather than
    // asserted. Both numbers are the model's own output, so neither is a
    // figure to check against; the doc comment above carries the argument
    // and `README.md` carries the row.
    assert!(
        easy_slap.is_finite() && hard_slap.is_finite(),
        "slap over body: OverEasy {easy_slap} dB, hard knee {hard_slap} dB"
    );
    // dbx set the threshold for 15 dB, and the hard-knee curve is the one
    // that number describes.
    assert!(
        (hard_gr - 15.0).abs() < 0.5,
        "the recipe's threshold gave {hard_gr} dB of reduction, not 15"
    );
}

/// 26. The knee's width does not depend on the ratio; its depth does.
///
/// **No number is published.** A circuit identity: the rectifier precedes
/// the ratio scaling in dbx's own patent figure, so α cannot change the
/// width in decibels and must halve the depth.
#[test]
fn t26_the_knee_width_is_ratio_independent() {
    let span = |alpha: f32| {
        let c = unit(SR, |s| {
            s.model = MODEL_160A;
            s.threshold_dbu = 0.0;
            s.alpha = alpha;
            s.knee = KNEE_OVEREASY;
        });
        let a = effective_alpha(alpha);
        let mut lo = f32::NAN;
        let mut hi = f32::NAN;
        for i in 0..1200 {
            let dbu = -30.0 + i as f32 * 0.05;
            let done = (1.0 - slope_at(&c, dbu)) / a;
            if lo.is_nan() && done >= 0.05 {
                lo = dbu;
            }
            if hi.is_nan() && done >= 0.95 {
                hi = dbu;
                break;
            }
        }
        (hi - lo, c.static_gr_db(dbu_peak(hi)))
    };
    let (w2, d2) = span(0.5);
    let (wi, di) = span(1.0);
    assert!(
        ((w2 / wi) - 1.0).abs() < 0.10,
        "the knee is {w2} dB wide at 2:1 and {wi} at ∞:1"
    );
    assert!(
        ((d2 / di) / 0.5 - 1.0).abs() < 0.05,
        "the knee is {d2} dB deep at 2:1 against {di} at ∞:1, not half"
    );
}

// ================================================== 12.5 the distortion

/// 27. The third harmonic halves per octave of frequency.
///
/// Figure: "3rd-harmonic distortion in the 160 Series decreases linearly
/// as the frequency rises: **at 100 Hz 3rd-harmonic distortion is 1/2 the
/// value at 50 Hz**, etc." (the 160's specification-page footnote).
///
/// **This is the best test in the file**, because it asserts a *ratio*
/// between two of the model's own measurements against a published law, so
/// it needs no absolute calibration and cannot be passed by fudging a gain
/// constant. It also fails immediately if anybody adds a third-harmonic
/// waveshaper, because a waveshaper's third harmonic does not depend on
/// frequency.
#[test]
fn t27_the_third_harmonic_halves_with_frequency() {
    let mut h3 = vec![];
    for hz in [50.0f32, 100.0, 200.0, 400.0] {
        let mut c = unit(SR, |s| {
            s.threshold_dbu = -10.0;
            s.alpha = 1.0;
        });
        let (f, _, t) = harmonics(&mut c, hz, dbu_peak(0.0), SR);
        h3.push(t / f);
    }
    for w in h3.windows(2) {
        let r = w[1] / w[0];
        assert!(
            (r - 0.5).abs() < 0.05,
            "the third harmonic fell by a factor of {r} per octave: {h3:?}"
        );
    }
}

/// 28. The third harmonic falls with lower ratios and rises with faster
///     time constants.
///
/// Figure: "3rd harmonic decreases with **slower time constants, higher
/// frequencies and lower compression ratios**" (the 1981 brochure's
/// footnote 4). Three directions, all published, none of them a magnitude.
#[test]
fn t28_the_third_harmonic_follows_ratio_and_time_constant() {
    let h3 = |alpha: f32, tau: f32| {
        let mut c = unit(SR, |s| {
            s.threshold_dbu = -10.0;
            s.alpha = alpha;
            s.tau_s = tau;
        });
        let (f, _, t) = harmonics(&mut c, 100.0, dbu_peak(0.0), SR);
        t / f
    };
    let low = h3(0.5, TAU_DEFAULT_S);
    let high = h3(1.0, TAU_DEFAULT_S);
    assert!(low < high, "2:1 gave {low} against ∞:1's {high}");
    for alpha in [0.5f32, 1.0] {
        let fast = h3(alpha, TAU_DEFAULT_S);
        let slow = h3(alpha, TAU_DEFAULT_S * 2.0);
        assert!(
            slow < fast,
            "at α = {alpha}, doubling τ gave {slow} against {fast}"
        );
    }
}

/// 29. The second harmonic is unaffected by ratio, time constant and
///     frequency.
///
/// Figure: "**2nd harmonic is relatively unaffected by compression ratio,
/// time constants and frequency**". The ±20 % is **mine**; "relatively
/// unaffected" carries no number.
#[test]
fn t29_the_second_harmonic_is_unaffected() {
    let mut all = vec![];
    for alpha in [0.5f32, 0.75, 1.0] {
        for hz in [100.0f32, 1000.0, 10_000.0] {
            for tau in [TAU_DEFAULT_S, TAU_DEFAULT_S * 2.0] {
                let mut c = unit(SR, |s| {
                    s.threshold_dbu = -10.0;
                    s.alpha = alpha;
                    s.tau_s = tau;
                });
                let (f, h, _) = harmonics(&mut c, hz, dbu_peak(0.0), SR);
                all.push(h / f);
            }
        }
    }
    let max = all.iter().cloned().fold(f32::MIN, f32::max);
    let min = all.iter().cloned().fold(f32::MAX, f32::min);
    let mean = all.iter().sum::<f32>() / all.len() as f32;
    assert!(
        (max - min) / mean < 0.40,
        "the second harmonic spread from {min} to {max} about {mean}"
    );
}

/// 30. The second harmonic hits its published magnitude.
///
/// Figure: "**0.075 % 2nd harmonic at infinite compression at +4dBm
/// output**" (the 160's specification page). This is the number
/// [`CELL_ASYMMETRY`] is fitted to, so strictly it is a calibration rather
/// than a test, and it is recorded as such. What makes it a real test is
/// running it at other frequencies and ratios too, which test 29 does.
#[test]
fn t30_the_second_harmonic_magnitude() {
    let mut c = unit(SR, |s| {
        s.threshold_dbu = -20.0;
        s.alpha = 1.0;
        // Whatever reduction the detector applies, the make-up brings the
        // output back to +4 dBu, which is dbx's stated condition.
        s.output_db = 0.0;
    });
    let (f, h, _) = harmonics(&mut c, 1000.0, dbu_peak(4.0), SR);
    let pct = 100.0 * h / f;
    assert!(
        (pct - 0.075).abs() / 0.075 <= 0.30,
        "the second harmonic is {pct} % against dbx's 0.075 %"
    );
}

/// 31. Total harmonic distortion stays under the published ceiling.
///
/// Figure: "THD **<0.2 %, Typical, Any Amount of Compression up to 40 dB @
/// 1 kHz**" (the 160A specification page). A ceiling across the whole
/// gain-reduction range with its own stated conditions, and the only such
/// figure in the family.
#[test]
fn t31_thd_stays_under_the_published_ceiling() {
    for gr_target in (0..=40).step_by(5) {
        let mut c = unit(SR, |s| {
            s.model = MODEL_160A;
            s.threshold_dbu = 0.0 - gr_target as f32;
            s.alpha = 1.0;
        });
        let (f, h2, h3) = harmonics(&mut c, 1000.0, dbu_peak(0.0), SR);
        let thd = 100.0 * (h2 * h2 + h3 * h3).sqrt() / f;
        assert!(
            thd < 0.2,
            "at {gr_target} dB of reduction the THD is {thd} %"
        );
    }
}

/// 32. Distortion below threshold, second harmonic.
///
/// Figure: "Distortion below threshold: 2nd harmonic 0.07 %", "Measured at
/// 1 kHz, 0 dBm input and output" (the 160X). The ±50 % is mine and is
/// deliberately loose, because this is the 160X's board and not the 160's.
///
/// **The published third-harmonic figure in the same row is a miss and is
/// not asserted here.** dbx print 0.07 % third harmonic below threshold
/// too; with no gain reduction there is no ripple, and the model has
/// neither a noise source nor an output-stage model, so it produces
/// essentially none. `README.md` records that.
#[test]
fn t32_distortion_below_threshold() {
    let mut c = unit(SR, |s| {
        s.model = MODEL_160A;
        s.threshold_dbu = 20.0;
        s.alpha = 1.0;
    });
    let (f, h2, _) = harmonics(&mut c, 1000.0, dbu_peak(0.0), SR);
    let pct = 100.0 * h2 / f;
    assert!(
        (0.035..=0.105).contains(&pct),
        "the second harmonic below threshold is {pct} % against the 160X's 0.07 %"
    );
}

/// 33. The model's own predicted third harmonic, recorded and checked.
///
/// Figure: dbx's "**0.5 % 3rd harmonic typical at infinite compression
/// ratio**", against the 0.8 % the research derives from the ripple
/// equation at 100 Hz. The band brackets both, and its purpose is to catch
/// a detector whose ripple is an order of magnitude wrong rather than to
/// calibrate anything — dbx did not state the frequency of their figure.
#[test]
fn t33_the_third_harmonic_magnitude() {
    let mut c = unit(SR, |s| {
        s.threshold_dbu = -20.0;
        s.alpha = 1.0;
    });
    let (f, _, h3) = harmonics(&mut c, 100.0, dbu_peak(0.0), SR);
    let pct = 100.0 * h3 / f;
    assert!(
        (0.3..=1.2).contains(&pct),
        "the third harmonic at 100 Hz is {pct} %, outside the 0.3 to 1.2 % that brackets \
         dbx's 0.5 % and the ripple equation's 0.8 %"
    );
}

// ===================================================== 12.7 the metering

/// 34. Stereo linking applies one gain to both channels.
///
/// Figure: "the **RMS energy** of the signal presented to the slave unit
/// is **summed** with the RMS energy of the signal presented to the master
/// unit" (the 160A manual).
#[test]
fn t34_linking_ganges_the_gain() {
    let x = sine(1000.0, 0.5, 4096, SR);
    let mut c = unit(SR, |s| {
        s.threshold_dbu = -30.0;
        s.alpha = 1.0;
        s.link = true;
    });
    for _ in 0..30 {
        let mut l = x.clone();
        let mut r = vec![0.0; x.len()];
        c.process_block(&mut l, &mut r);
    }
    let d = c.gain_reduction_db(0) - c.gain_reduction_db(1);
    assert!(d.abs() < 1e-4, "the two channels differ by {d} dB");
    assert!(
        c.gain_reduction_db(1) > 10.0,
        "the silent channel did not follow"
    );
}

/// 37. The meter reads 0 at the level the trimmer is set to.
///
/// Figure: "The meter in the 160 and 161 is factory calibrated to read '0'
/// at **+4dB (1.23V)** output level", and the trimmer's range "−15 dBu
/// (138 mV) to +10 dBu (2.45 V)" on the 160A.
#[test]
fn t37_meter_calibration() {
    for cal in [
        METER_CAL_MIN_DBU,
        0.0,
        METER_CAL_DEFAULT_DBU,
        METER_CAL_MAX_DBU,
    ] {
        let mut c = unit(SR, |s| {
            s.alpha = 0.0;
            s.threshold_dbu = 20.0;
            s.meter = METER_OUTPUT;
            s.meter_cal_dbu = cal;
        });
        let x = sine(1000.0, dbu_peak(cal), 4800, SR);
        run(&mut c, &x);
        run(&mut c, &x);
        let vu = c.meter_frame()[5];
        assert!(
            vu.abs() < 0.15,
            "with the trimmer at {cal} dBu a sine at {cal} dBu read {vu} VU"
        );
    }
}

/// 39. Make-up gain does not appear on the gain-reduction meter.
///
/// Figure: "Note that fixed gain changes due to the OUTPUT GAIN control
/// are **not displayed by the GAIN REDUCTION LEDs** but **are reflected in
/// the OUTPUT LEVEL display**" (the 160A manual).
#[test]
fn t39_make_up_is_not_gain_reduction() {
    let read = |g: f32, meter: usize| {
        let mut c = unit(SR, |s| {
            s.threshold_dbu = -10.0;
            s.alpha = 0.9;
            s.output_db = g;
            s.meter = meter;
        });
        let x = sine(1000.0, dbu_peak(0.0), 4800, SR);
        for _ in 0..30 {
            run(&mut c, &x);
        }
        c.meter_frame()[5]
    };
    let gr = (read(-10.0, METER_GAIN_CHANGE) - read(10.0, METER_GAIN_CHANGE)).abs();
    assert!(gr < 0.05, "the gain-reduction meter moved by {gr} dB");
    let out = read(10.0, METER_OUTPUT) - read(-10.0, METER_OUTPUT);
    assert!(
        (out - 20.0).abs() < 0.2,
        "the output meter moved by {out} dB, not 20"
    );
}

// ============================== 12.8 response, robustness, invariance

/// 41. The input coupling corner.
///
/// Figure: a circuit identity from **C12 = 0.15 µF** into **R26 = 100 kΩ**
/// on the 160 schematic. **dbx publish no frequency response for the
/// original at all**, so this asserts two components rather than a
/// specification. The model therefore *fails* the 160A's published "+0,
/// −0.5 dB, 20 Hz – 20 kHz" — correctly, because that is a different board
/// with much larger coupling capacitors.
#[test]
fn t41_the_input_coupling_corner() {
    let level = |hz: f32| {
        let mut c = unit(SR, |s| {
            s.alpha = 0.0;
            s.threshold_dbu = 20.0;
        });
        let n = ((SR / hz).round() as usize) * 60;
        let x = sine(hz, 0.25, n, SR);
        run(&mut c, &x);
        let y = run(&mut c, &x);
        rms_dbfs(&y) - rms_dbfs(&x)
    };
    let reference = level(1000.0);
    let at_corner = level(INPUT_HP_HZ) - reference;
    assert!(
        (at_corner + 3.0).abs() < 0.3,
        "at {INPUT_HP_HZ} Hz the response is {at_corner} dB, not −3"
    );
    let at_20 = level(20.0) - reference;
    assert!(
        (at_20 + 1.1).abs() < 0.2,
        "at 20 Hz the response is {at_20} dB, not −1.1"
    );
}

/// 43. Sample-rate invariance.
///
/// The only rate-dependent coefficient in the detector is `exp(-h/τ)`, so
/// anything worse than this indicates a discretisation bug rather than a
/// modelling choice.
#[test]
fn t43_sample_rate_invariance() {
    let mut ratios = vec![];
    let mut attacks = vec![];
    for sr in [44_100.0f32, 48_000.0, 88_200.0, 96_000.0] {
        let c = unit(sr, |s| {
            s.threshold_dbu = -20.0;
            s.alpha = 0.75;
        });
        let a = c.static_gr_db(dbu_peak(0.0));
        let b = c.static_gr_db(dbu_peak(20.0));
        ratios.push(20.0 - (b - a));
        attacks.push(attack_ms(10.0, TAU_DEFAULT_S, sr));
    }
    let spread = |v: &[f32]| {
        v.iter().cloned().fold(f32::MIN, f32::max) - v.iter().cloned().fold(f32::MAX, f32::min)
    };
    assert!(
        spread(&ratios) < 0.05,
        "the ratio moved by {} dB: {ratios:?}",
        spread(&ratios)
    );
    let rel = spread(&attacks) / attacks[0];
    assert!(
        rel < 0.03,
        "the attack time moved by {rel} across rates: {attacks:?}"
    );
}

/// 44. Numerical robustness.
#[test]
fn t44_stays_finite() {
    let mut c = unit(SR, |s| {
        s.model = MODEL_160A;
        s.threshold_dbu = THRESHOLD_MIN_DBU;
        s.alpha = 2.0;
        s.knee = KNEE_OVEREASY;
        s.lookahead_ms = LOOKAHEAD_MAX_MS;
        s.sc_hpf = 250.0;
        s.link = true;
        s.output_db = OUTPUT_MAX_DB;
    });
    // Ten seconds of full scale, then thirty of digital silence: the
    // rate-limited branch of the detector is exercised by the silence.
    let loud = sine(97.0, 1.0, 4800, SR);
    for _ in 0..100 {
        let y = run(&mut c, &loud);
        assert!(
            y.iter().all(|v| v.is_finite()),
            "went non-finite while loud"
        );
    }
    let quiet = vec![0.0f32; 4800];
    let mut last = c.gain_reduction_db(0);
    for _ in 0..300 {
        let y = run(&mut c, &quiet);
        assert!(
            y.iter().all(|v| v.is_finite()),
            "went non-finite in silence"
        );
        let now = c.gain_reduction_db(0);
        assert!(now <= last + 1e-3, "the gain reduction rose during silence");
        last = now;
    }
    assert!(
        last < 1e-3,
        "after thirty seconds of silence {last} dB of reduction remains"
    );

    // DC, square waves, single-sample impulses and both parameter extremes.
    for shape in 0..4 {
        let mut c = unit(SR, |s| {
            s.model = MODEL_160A;
            s.threshold_dbu = if shape % 2 == 0 {
                THRESHOLD_MIN_DBU
            } else {
                THRESHOLD_MAX_DBU
            };
            s.alpha = if shape < 2 { 0.0 } else { 2.0 };
            s.tau_s = if shape < 2 { TAU_MIN_S } else { TAU_MAX_S };
            s.knee_width_db = KNEE_WIDTH_MAX_DB;
            s.headroom_db = HEADROOM_MIN_DB;
        });
        let x: Vec<f32> = (0..8192)
            .map(|i| match shape {
                0 => 1.0,
                1 => {
                    if (i / ((SR / 40.0) as usize)).is_multiple_of(2) {
                        1.0
                    } else {
                        -1.0
                    }
                }
                2 => {
                    if i % ((SR / 20_000.0) as usize).max(1) == 0 {
                        1.0
                    } else {
                        -1.0
                    }
                }
                _ => {
                    if i == 0 {
                        1.0
                    } else {
                        0.0
                    }
                }
            })
            .collect();
        let y = run(&mut c, &x);
        assert!(
            y.iter().all(|v| v.is_finite() && v.abs() < 1e3),
            "shape {shape} produced an unbounded output"
        );
    }
}

// ============================================== the dial laws themselves

/// The nine marks land where they were measured, and α is what the
/// circuit's own law says at each of them.
#[test]
fn t_ratio_dial_marks() {
    for (i, alpha) in RATIO_MARK_ALPHA.iter().enumerate() {
        let travel = RATIO_MARK_TRAVEL[i] * INFINITY_TRAVEL;
        let got = alpha_for_travel(travel);
        assert!(
            (got - alpha).abs() < 1e-4,
            "{} sits at α = {got}, not {alpha}",
            RATIO_MARK_LABELS[i]
        );
        // And back again.
        let back = travel_for_alpha(*alpha);
        assert!(
            (back - travel).abs() < 1e-4,
            "{} round-tripped to {back}",
            RATIO_MARK_LABELS[i]
        );
    }
    // α = 1 − 1/R at every printed mark, which is the whole ratio law.
    for (i, label) in ["1", "1.5", "2", "3", "4", "6", "10", "20"]
        .iter()
        .enumerate()
    {
        let r: f32 = label.parse().unwrap();
        assert!((RATIO_MARK_ALPHA[i] - (1.0 - 1.0 / r)).abs() < 1e-6);
    }
    // The four the 160A adds past infinity.
    for (alpha, r) in [(1.2f32, -5.0f32), (1.5, -2.0), (2.0, -1.0)] {
        assert!((ratio_for_alpha(alpha) - r).abs() < 1e-3);
    }
    assert_eq!(ratio_label(1.0), "∞:1");
    assert_eq!(ratio_label(0.75), "4.0:1");
    assert_eq!(ratio_label(2.0), "\u{2212}1.0:1");
}

/// The threshold marks are the six voltages dbx print, converted to dBu,
/// and they really are 10 dB apart.
#[test]
fn t_threshold_dial_marks() {
    let volts = [0.010f32, 0.030, 0.100, 0.300, 1.0, 3.0];
    for (v, want) in volts.iter().zip(THRESHOLD_MARK_DBU) {
        let dbu = 20.0 * (v / 0.775).log10();
        assert!(
            (dbu - want).abs() < 0.01,
            "{v} V is {dbu} dBu, not the tabled {want}"
        );
    }
    // dbx's marks are a 1-3-10 sequence, so the steps alternate between
    // 20·log10(3) = 9.54 dB and 20·log10(10/3) = 10.46 dB. Their factory
    // procedure calls them "10 db steps" and their specification calls the
    // span −38 dB to +12 dB, which is 49.5 dB over five of them, i.e. 9.9
    // each. All three statements are dbx's and all three are consistent
    // with a decade every two marks; none of them is exactly 10.
    let three = 20.0 * 3.0f32.log10();
    let rest = 20.0 * (10.0f32 / 3.0).log10();
    for (i, w) in THRESHOLD_MARK_DBU.windows(2).enumerate() {
        let want = if i % 2 == 0 { three } else { rest };
        assert!(
            (w[1] - w[0] - want).abs() < 0.02,
            "mark {i} to {} is {} dB, not {want}",
            i + 1,
            w[1] - w[0]
        );
    }
    let span = THRESHOLD_MARK_DBU[5] - THRESHOLD_MARK_DBU[0];
    assert!(
        (span - 49.54).abs() < 0.05,
        "the dial spans {span} dB against dbx's 10 mV to 3 V, which is 49.5"
    );
    assert!((THRESHOLD_MARK_DBU[0] - THRESHOLD_160_MIN_DBU).abs() < 0.01);
    assert!((THRESHOLD_MARK_DBU[5] - THRESHOLD_160_MAX_DBU).abs() < 0.01);
}

/// The parameter's sampled taper reproduces the law it is sampled from.
#[test]
fn t_ratio_table_is_faithful() {
    let table = ratio_table();
    assert!(table.len() >= 2);
    assert_eq!(table[0], 0.0);
    assert!((table[table.len() - 1] - 2.0).abs() < 1e-6);
    for w in table.windows(2) {
        assert!(w[1] >= w[0], "the taper is not monotonic");
    }
}

/// The residual slope at the ∞ mark is dbx's published 120:1, and the
/// cell's control law is the constant decibels per volt Blackmer designed
/// for.
#[test]
fn t_constants() {
    assert!((1.0 / (1.0 - ALPHA_CEILING) - 120.0).abs() < 1e-2);
    // The thermal decibel is `10/ln 10` exactly, which is what makes the
    // averaging an average of the square; see `D_DB`.
    assert!(
        (D_DB - 10.0 / std::f32::consts::LN_10).abs() < 1e-6,
        "the thermal decibel is {D_DB}"
    );
    // And the time constant is R35 and C15 off dbx's drawing, through the
    // ideality the datasheet's own 6.1 mV/dB implies.
    let i_t = 15.0 / 909_000.0;
    let tau = 22e-6 * (IDEALITY * V_T_MV * 1e-3) / i_t;
    assert!(
        (tau - TAU_DEFAULT_S).abs() < 1e-5,
        "R35 and C15 give τ = {tau} s, not {TAU_DEFAULT_S}"
    );
    assert!(
        (IDEALITY - 1.0228).abs() < 1e-3,
        "the implied ideality is {IDEALITY}"
    );
    // Which puts the release rate between dbx's published 120 and 125.
    let rate = D_DB / TAU_DEFAULT_S;
    assert!(
        (120.0..=125.0).contains(&rate),
        "the release rate is {rate} dB/s"
    );
    let cell = CELL;
    // 6.1 mV/dB on the negative port, both ways. The cell is the
    // component's now, so this is a check that the engine holds the part
    // the schematic names rather than a re-test of the component.
    assert!((cell.gain_db(0.0, 61.0) + 10.0).abs() < 1e-3);
    assert!((cell.control_mv_for_gain(-10.0) - 61.0).abs() < 1e-3);
    // +0.33 %/°C referenced to 27 °C.
    let warm = BlackmerCell {
        temp_c: 37.0,
        ..CELL
    };
    assert!((warm.k_at_temp() / cell.k_at_temp() - 1.033).abs() < 1e-4);
    // And the residual dbx published is the shape their own footnote
    // describes: a half-path gain mismatch, whose second harmonic is a
    // fixed fraction rather than a function of level. The component owns
    // the `4/(3π)`; dbx own the 0.075 %.
    assert_eq!(cell.residual, EvenResidual::HalfPathMismatch);
    assert_eq!(cell.even_coefficient(), CELL_ASYMMETRY);
    assert!(
        (cell.residual.thd_for_coefficient(CELL_ASYMMETRY, 1.0) - 0.000_75).abs() < 1e-9,
        "the fitted residual is not dbx's 0.075 %"
    );
}

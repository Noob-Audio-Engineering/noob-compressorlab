//! The test plan of `research/1176.md` section 8, run offline against the engine.
//!
//! Levels are dBFS of a sine's peak unless stated. `steady_gr` drives the
//! compressor with a sine long enough for the detector to settle and reads
//! the gain reduction; `envelope` records the gain reduction sample by
//! sample for the timing tests; `thd` measures harmonics with a DFT at the
//! harmonic bins.

use super::compressor::TAP_THRESHOLD_DB;
use super::*;
use std::f32::consts::PI;

fn sine(level_dbfs: f32, hz: f32, sr: f32, n: usize) -> Vec<f32> {
    let a = 10f32.powf(level_dbfs / 20.0);
    (0..n)
        .map(|i| a * (2.0 * PI * hz * i as f32 / sr).sin())
        .collect()
}

fn run(c: &mut Compressor, input: &[f32]) -> Vec<f32> {
    let mut l = input.to_vec();
    let mut r = input.to_vec();
    for (bl, br) in l.chunks_mut(256).zip(r.chunks_mut(256)) {
        c.process(bl, br);
    }
    l
}

/// Steady-state gain reduction (dB, ≤ 0) for a 1 kHz sine at `level` dBFS.
fn steady_gr(s: &Settings, level: f32, sr: f32) -> f32 {
    let mut c = Compressor::new(sr);
    c.configure(s);
    let x = sine(level, 1000.0, sr, (sr * 1.5) as usize);
    run(&mut c, &x);
    c.gr_db()
}

/// Output level in dBFS (peak of the last 0.2 s) for a 1 kHz sine at `level`.
fn steady_out(s: &Settings, level: f32, sr: f32) -> f32 {
    let mut c = Compressor::new(sr);
    c.configure(s);
    let x = sine(level, 1000.0, sr, (sr * 1.5) as usize);
    let y = run(&mut c, &x);
    let tail = &y[y.len() - (sr * 0.2) as usize..];
    20.0 * tail
        .iter()
        .fold(0.0f32, |m, v| m.max(v.abs()))
        .max(1e-9)
        .log10()
}

/// Input level at which the gain reduction first reaches `gr` dB.
fn level_for_gr(s: &Settings, gr: f32, sr: f32) -> f32 {
    let mut lo = -70.0f32;
    let mut hi = 0.0f32;
    for _ in 0..12 {
        let mid = 0.5 * (lo + hi);
        if -steady_gr(s, mid, sr) >= gr {
            hi = mid
        } else {
            lo = mid
        }
    }
    hi
}

/// Slope of the static curve (input dB per output dB) between `a` and `b`
/// dB above the 1 dB-GR onset.
fn slope_above_onset(s: &Settings, a: f32, b: f32, sr: f32) -> f32 {
    let onset = level_for_gr(s, 1.0, sr);
    let o1 = steady_out(s, onset + a, sr);
    let o2 = steady_out(s, onset + b, sr);
    (b - a) / (o2 - o1).max(1e-3)
}

fn settings(ratio: Ratio) -> Settings {
    Settings {
        ratio,
        ..Settings::default()
    }
}

#[test]
fn ratios_hold_within_20_percent_above_onset() {
    let sr = 48_000.0;
    for (i, (ratio, nominal)) in [
        (Ratio::R4, 4.0),
        (Ratio::R8, 8.0),
        (Ratio::R12, 12.0),
        (Ratio::R20, 20.0),
    ]
    .into_iter()
    .enumerate()
    {
        let s = settings(ratio);
        let slope = slope_above_onset(&s, 6.0, 16.0, sr);
        assert!(
            (slope - nominal).abs() <= 0.2 * nominal,
            "{}: slope {slope:.2} not within 20 % of {nominal}",
            RATIO_NAMES[i]
        );
        // thresholds rise with the ratio and sit near the design values
        let onset = level_for_gr(&s, 1.0, sr);
        assert!(
            (onset - TAP_THRESHOLD_DB[i]).abs() < 3.0,
            "{}: onset {onset:.1} dBFS",
            RATIO_NAMES[i]
        );
    }
    assert!(
        level_for_gr(&settings(Ratio::R20), 1.0, sr) > level_for_gr(&settings(Ratio::R4), 1.0, sr)
    );
}

#[test]
fn twenty_to_one_is_nearly_flat() {
    let s = settings(Ratio::R20);
    let sr = 48_000.0;
    let onset = level_for_gr(&s, 1.0, sr);
    let rise = steady_out(&s, onset + 20.0, sr) - steady_out(&s, onset, sr);
    assert!(
        rise < 2.5,
        "20 dB more input raised the output by {rise:.2} dB"
    );
}

#[test]
fn input_knob_drives_compression_and_output_is_unity_at_24_24() {
    let sr = 48_000.0;
    let quiet = steady_out(&Settings::default(), -40.0, sr);
    assert!(
        (quiet + 40.0).abs() < 0.5,
        "unity below threshold: {quiet:.2} dBFS for −40"
    );
    let s30 = Settings {
        input: 30.0,
        ..Settings::default()
    };
    assert!(steady_gr(&s30, -24.0, sr) < steady_gr(&Settings::default(), -24.0, sr) - 1.0);
}

/// Gain reduction sample by sample for a level step of a 5 kHz sine (its
/// peaks come every 100 µs, faster than the slowest attack, so the timing
/// reflects the RC and not the waveform): 0.2 s at `from`, `hold_s` at `to`,
/// then 2.5 s back at `from`. Returns the trace and the index of the step.
fn envelope(s: &Settings, from: f32, to: f32, sr: f32, hold_s: f32) -> (Vec<f32>, usize) {
    envelope_at(s, from, to, sr, hold_s, 5000.0)
}

fn envelope_at(
    s: &Settings,
    from: f32,
    to: f32,
    sr: f32,
    hold_s: f32,
    hz: f32,
) -> (Vec<f32>, usize) {
    let mut c = Compressor::new(sr);
    c.configure(s);
    let pre = (sr * 0.2) as usize;
    let on = (sr * hold_s) as usize;
    let mut x = sine(from, hz, sr, pre);
    x.extend(sine(to, hz, sr, on));
    x.extend(sine(from, hz, sr, (sr * 2.5) as usize));
    let mut env = Vec::with_capacity(x.len());
    for &v in &x {
        let mut l = [v];
        let mut r = [v];
        c.process(&mut l, &mut r);
        env.push(-c.gr_db());
    }
    (env, pre)
}

/// Seconds from `start` until the trace first reaches `target`.
fn time_to(env: &[f32], start: usize, target: f32, sr: f32) -> f32 {
    let mut i = start;
    while i < env.len() && env[i] < target {
        i += 1;
    }
    (i - start) as f32 / sr
}

/// Seconds from `start` until the trace first falls below `target`.
fn time_below(env: &[f32], start: usize, target: f32, sr: f32) -> f32 {
    let mut i = start;
    while i < env.len() && env[i] > target {
        i += 1;
    }
    (i - start) as f32 / sr
}

#[test]
fn attack_and_release_follow_the_knobs() {
    let sr = 48_000.0;
    let hold = 1.0;
    let at = |attack: f32, release: f32| Settings {
        attack,
        release,
        ..settings(Ratio::R4)
    };
    let (e1, on) = envelope(&at(1.0, 1.0), -40.0, -14.0, sr, hold);
    let (e4, _) = envelope(&at(4.0, 4.0), -40.0, -14.0, sr, hold);
    let (e7, _) = envelope(&at(7.0, 7.0), -40.0, -14.0, sr, hold);
    let settle = on + (sr * 0.9) as usize;
    let fin = [e1[settle], e4[settle], e7[settle]];
    assert!(fin.iter().all(|f| *f > 5.0), "step produced {fin:?} dB GR");
    // attack: time to 90 % of the final reduction shortens with the knob
    let t = [
        time_to(&e1, on, 0.9 * fin[0], sr),
        time_to(&e4, on, 0.9 * fin[1], sr),
        time_to(&e7, on, 0.9 * fin[2], sr),
    ];
    assert!(t[2] < 0.001, "attack 7 took {:.5} s", t[2]);
    assert!(t[0] < 0.02, "attack 1 took {:.5} s", t[0]);
    assert!(
        t[0] > t[1] && t[1] > t[2],
        "attack times not ordered: {t:?}"
    );
    assert!(
        t[0] > 3.0 * t[2],
        "attack 1 ({:.5} s) should be much slower than 7 ({:.5} s)",
        t[0],
        t[2]
    );
    // release: time for the reduction to fall to 37 % once the step ends
    let off = on + (sr * hold) as usize;
    let r = [
        time_below(&e1, off, 0.37 * fin[0], sr),
        time_below(&e4, off, 0.37 * fin[1], sr),
        time_below(&e7, off, 0.37 * fin[2], sr),
    ];
    assert!(r[2] > 0.02 && r[2] < 0.15, "release 7 {:.3} s", r[2]);
    assert!(r[0] > 0.5 && r[0] < 2.5, "release 1 {:.3} s", r[0]);
    assert!(
        r[0] > r[1] && r[1] > r[2],
        "release times not ordered: {r:?}"
    );
}

#[test]
fn all_buttons_raises_threshold_lags_and_distorts_more() {
    let sr = 48_000.0;
    let all = settings(Ratio::All);
    let r20 = settings(Ratio::R20);
    let onset_all = level_for_gr(&all, 1.0, sr);
    let onset_4 = level_for_gr(&settings(Ratio::R4), 1.0, sr);
    assert!(
        onset_all > onset_4 + 3.0,
        "all-buttons onset {onset_all:.1} vs 4:1 {onset_4:.1}"
    );
    let slope = slope_above_onset(&all, 6.0, 16.0, sr);
    assert!(
        (10.0..=24.0).contains(&slope),
        "all-buttons slope {slope:.1}"
    );
    // attack lag: the same moderate step reaches half of its final reduction
    // later than at 20:1, because the capacitor first charges through the bias offset
    let (ea, on) = envelope(&all, -40.0, onset_all + 8.0, sr, 1.0);
    let (e20, _) = envelope(&r20, -40.0, onset_all + 8.0, sr, 1.0);
    let settle = on + (sr * 0.9) as usize;
    let ta = time_to(&ea, on, 0.5 * ea[settle], sr);
    let t20 = time_to(&e20, on, 0.5 * e20[settle], sr);
    assert!(
        ta > t20,
        "all-buttons attack {ta:.5} s should lag 20:1 {t20:.5} s"
    );
    // more distortion at matched gain reduction
    let la = level_for_gr(&all, 10.0, sr);
    let l20 = level_for_gr(&r20, 10.0, sr);
    assert!(thd(&all, la, sr).0 > thd(&r20, l20, sr).0);
}

/// (THD ratio, second harmonic ratio, third harmonic ratio) of the output
/// for a 1 kHz sine at `level`, over the last second.
fn thd(s: &Settings, level: f32, sr: f32) -> (f32, f32, f32) {
    let mut c = Compressor::new(sr);
    c.configure(s);
    let x = sine(level, 1000.0, sr, (sr * 3.0) as usize);
    let y = run(&mut c, &x);
    let tail = &y[y.len() - sr as usize..];
    let bin = |h: f32| {
        let (mut re, mut im) = (0.0f64, 0.0f64);
        for (i, v) in tail.iter().enumerate() {
            let ph = 2.0 * std::f64::consts::PI * (1000.0 * h) as f64 * i as f64 / sr as f64;
            re += *v as f64 * ph.cos();
            im += *v as f64 * ph.sin();
        }
        ((re * re + im * im).sqrt() / tail.len() as f64) as f32
    };
    let f1 = bin(1.0);
    let hs: Vec<f32> = (2..=8).map(|h| bin(h as f32)).collect();
    let thd = (hs.iter().map(|h| h * h).sum::<f32>()).sqrt() / f1;
    (thd, hs[0] / f1, hs[1] / f1)
}

#[test]
fn ln_is_clean_and_bluestripe_adds_second_harmonic() {
    let sr = 48_000.0;
    let ln = settings(Ratio::R4);
    let blue = Settings {
        revision: Revision::A,
        ..ln
    };
    let (t_ln, _, _) = thd(&ln, level_for_gr(&ln, 10.0, sr), sr);
    let (t_blue, h2, h3) = thd(&blue, level_for_gr(&blue, 10.0, sr), sr);
    assert!(t_ln < 0.005, "LN THD {:.3} %", t_ln * 100.0);
    assert!(
        t_blue > t_ln * 2.0 && t_blue > 0.005 && t_blue < 0.06,
        "Rev A THD {:.3} % (LN {:.3} %)",
        t_blue * 100.0,
        t_ln * 100.0
    );
    assert!(
        h2 > h3,
        "blue stripe second harmonic {h2:.4} should dominate the third {h3:.4}"
    );
}

/// Every revision runs finite and bounded when overdriven, and the ones the
/// sources call noisier and more coloured (A, B) measure more THD than the
/// LN family (C to H, the reissue); F, the push-pull output stage, measures
/// no more than D.
#[test]
fn every_revision_is_bounded_and_the_blue_stripes_distort_more() {
    let sr = 48_000.0;
    let base = settings(Ratio::R4);
    let level = level_for_gr(&base, 10.0, sr);
    let mut t = [0.0f32; 9];
    for rev in Revision::ALL {
        let hot = Settings {
            revision: rev,
            input: 44.0,
            ..base
        };
        let mut c = Compressor::new(sr);
        c.configure(&hot);
        let x = sine(0.0, 80.0, sr, (sr * 0.5) as usize);
        let y = run(&mut c, &x);
        assert!(
            y.iter().all(|v| v.is_finite()),
            "{rev:?} produced NaN / inf"
        );
        let peak = y.iter().fold(0.0f32, |m, v| m.max(v.abs()));
        assert!(peak < 3.0, "{rev:?} peak {peak} at 20 dB over full scale");
        let s = Settings {
            revision: rev,
            ..base
        };
        t[rev.index()] = thd(&s, level, sr).0;
    }
    let ln_family = [
        Revision::C,
        Revision::D,
        Revision::E,
        Revision::F,
        Revision::G,
        Revision::H,
        Revision::Ln,
    ];
    let ln_max = ln_family
        .iter()
        .map(|r| t[r.index()])
        .fold(0.0f32, f32::max);
    let pct: Vec<String> = Revision::ALL
        .iter()
        .map(|r| format!("{}: {:.3} %", r.label(), t[r.index()] * 100.0))
        .collect();
    println!("THD by revision at 10 dB of gain reduction: {pct:?}");
    assert!(ln_max < 0.005, "LN family THD too high: {pct:?}");
    assert!(
        t[Revision::A.index()] > 2.0 * ln_max && t[Revision::B.index()] > 2.0 * ln_max,
        "blue stripes should distort at least twice as much: {pct:?}"
    );
    assert!(
        t[Revision::A.index()] > t[Revision::B.index()],
        "Rev A should be the most coloured: {pct:?}"
    );
    assert!(
        t[Revision::F.index()] <= t[Revision::D.index()],
        "Rev F (push-pull) should not exceed Rev D: {pct:?}"
    );
    assert_eq!(
        t[Revision::C.index()],
        t[Revision::D.index()],
        "C and D share a circuit"
    );
    assert_eq!(
        t[Revision::G.index()],
        t[Revision::H.index()],
        "G and H share a circuit"
    );
}

#[test]
fn bypass_is_transparent_and_mix_blends() {
    let sr = 48_000.0;
    let mut c = Compressor::new(sr);
    c.configure(&Settings {
        bypass: true,
        input: 40.0,
        ..Settings::default()
    });
    let x = sine(-6.0, 440.0, sr, 4096);
    let y = run(&mut c, &x);
    let lat = c.latency();
    for i in 200..4096 {
        assert!((y[i] - x[i - lat]).abs() < 1e-6, "bypass differs at {i}");
    }
    let dry = steady_out(
        &Settings {
            mix: 0.0,
            input: 40.0,
            ..Settings::default()
        },
        -12.0,
        sr,
    );
    assert!((dry + 12.0).abs() < 0.2, "mix 0 gives {dry:.2}");
}

#[test]
fn numerically_robust() {
    let sr = 48_000.0;
    let mut c = Compressor::new(sr);
    c.configure(&Settings {
        input: 48.0,
        ..Settings::default()
    });
    let mut x = sine(0.0, 60.0, sr, (sr * 1.0) as usize);
    x.iter_mut().for_each(|v| *v *= 10.0);
    x.extend(std::iter::repeat_n(0.0, (sr * 5.0) as usize));
    let y = run(&mut c, &x);
    assert!(y.iter().all(|v| v.is_finite()));
    assert_eq!(
        c.gr_db(),
        0.0,
        "gain reduction did not return to zero: {}",
        c.gr_db()
    );
    // Only the modelled noise floor remains (−78 dBFS for the reissue);
    // the filters and the detector have flushed to zero.
    let tail = &y[y.len() - sr as usize / 2..];
    let rms = (tail.iter().map(|v| v * v).sum::<f32>() / tail.len() as f32).sqrt();
    assert!(
        rms < 10f32.powf(-70.0 / 20.0),
        "not at the noise floor after silence: {} dBFS",
        20.0 * rms.log10()
    );
    assert!(rms > 10f32.powf(-90.0 / 20.0), "the noise floor is missing");
}

#[test]
fn sample_rate_invariant() {
    let s = Settings {
        input: 30.0,
        ..settings(Ratio::R8)
    };
    let g44 = steady_gr(&s, -12.0, 44_100.0);
    let g96 = steady_gr(&s, -12.0, 96_000.0);
    assert!(
        (g44 - g96).abs() < 0.5,
        "44.1 kHz {g44:.2} vs 96 kHz {g96:.2} dB"
    );
    assert!(g44 < -6.0);
}

#[test]
fn stereo_link_shares_one_detector() {
    let sr = 48_000.0;
    let n = (sr * 2.0) as usize;
    let loud = sine(-6.0, 1000.0, sr, n);
    let quiet = sine(-40.0, 1000.0, sr, n);
    let out_r = |link: bool| {
        let mut c = Compressor::new(sr);
        c.configure(&Settings {
            link,
            ..settings(Ratio::R8)
        });
        let mut l = loud.clone();
        let mut r = quiet.clone();
        for (bl, br) in l.chunks_mut(256).zip(r.chunks_mut(256)) {
            c.process(bl, br);
        }
        let tail = &r[n - (sr * 0.2) as usize..];
        20.0 * tail.iter().fold(0.0f32, |m, v| m.max(v.abs())).log10()
    };
    let linked = out_r(true);
    let unlinked = out_r(false);
    assert!(
        linked < unlinked - 5.0,
        "linked right channel {linked:.1} dB, unlinked {unlinked:.1} dB"
    );
}

#[test]
fn meter_reads_gr_and_vu() {
    let sr = 48_000.0;
    let mut c = Compressor::new(sr);
    // attack OFF: no compression, so the +4 meter sees the signal at unity
    c.configure(&Settings {
        meter: MeterMode::Plus4,
        attack: 0.0,
        ..Settings::default()
    });
    // a −18 dBFS RMS sine (−15 dBFS peak) reads 0 VU in +4 mode
    let x = sine(-18.0 + 3.0103, 1000.0, sr, (sr * 0.5) as usize);
    run(&mut c, &x);
    let vu = c.take_meter_reading();
    assert!(vu.abs() < 0.5, "+4 mode reads {vu:.2} VU");
    c.configure(&Settings {
        meter: MeterMode::Plus8,
        attack: 0.0,
        ..Settings::default()
    });
    run(&mut c, &x);
    let vu8 = c.take_meter_reading();
    assert!(
        (vu8 + 4.0).abs() < 0.5,
        "+8 mode reads {vu8:.2} VU for a +4 dBu signal"
    );
    c.configure(&Settings {
        meter: MeterMode::Gr,
        ..Settings::default()
    });
    let x = sine(-6.0, 1000.0, sr, (sr * 1.0) as usize);
    run(&mut c, &x);
    assert!((c.take_meter_reading() - c.gr_db()).abs() < 1e-3);
}

#[test]
fn transfer_curve_matches_the_engine_within_a_couple_of_db() {
    let sr = 48_000.0;
    let s = settings(Ratio::R8);
    let mut c = Compressor::new(sr);
    c.configure(&s);
    let mut curve = [0.0f32; TRANSFER_POINTS];
    c.transfer(&mut curve);
    for level in [-50.0f32, -30.0, -12.0] {
        let i = ((level + 60.0) / 60.0 * (TRANSFER_POINTS - 1) as f32).round() as usize;
        let measured = steady_out(&s, level, sr);
        assert!(
            (curve[i] - measured).abs() < 2.5,
            "at {level} dBFS curve {:.1} vs engine {measured:.1}",
            curve[i]
        );
    }
}

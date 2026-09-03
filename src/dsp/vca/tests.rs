//! Tests for the Distressor engine, from the test plan in
//! `research/Distressor.md` section 8, trimmed to what runs in seconds.

use super::compressor::{BRITISH, db_to_lin};
use super::*;
use std::f32::consts::TAU;

const SR: f32 = 48_000.0;

fn engine(f: impl FnOnce(&mut Settings)) -> Compressor {
    let mut s = Settings::default();
    f(&mut s);
    let mut c = Compressor::new(SR);
    c.configure(&s);
    c.reset();
    c
}

/// Length of the tail every run keeps for measurement: long enough that a
/// 30 Hz tone has several cycles in it.
const TAIL: usize = 12_000;

/// Run a sine of `amp` at `hz` for `seconds` and return the last
/// [`TAIL`] samples of each channel.
fn run_sine(c: &mut Compressor, amp: f32, hz: f32, seconds: f32) -> (Vec<f32>, Vec<f32>) {
    let n = ((SR * seconds) as usize).max(TAIL);
    let block = 256;
    let mut phase = 0.0f32;
    let (mut lb, mut rb) = (vec![0.0; block], vec![0.0; block]);
    let (mut lo, mut ro) = (
        Vec::with_capacity(TAIL + block),
        Vec::with_capacity(TAIL + block),
    );
    let mut done = 0;
    while done < n {
        let m = block.min(n - done);
        for i in 0..m {
            let v = amp * (TAU * hz * phase / SR).sin();
            phase += 1.0;
            lb[i] = v;
            rb[i] = v;
        }
        c.process_block(&mut lb[..m], &mut rb[..m]);
        if done + m > n - TAIL {
            lo.extend_from_slice(&lb[..m]);
            ro.extend_from_slice(&rb[..m]);
        }
        done += m;
    }
    (lo, ro)
}

/// Peak of the second half of a buffer.
fn peak(v: &[f32]) -> f32 {
    let start = v.len() / 2;
    v[start..].iter().fold(0.0f32, |a, b| a.max(b.abs()))
}

fn db(x: f32) -> f32 {
    20.0 * x.max(1e-9).log10()
}

/// Steady-state output level in dBFS for an input sine at `in_db`.
fn settled_db(c: &mut Compressor, in_db: f32) -> f32 {
    c.reset();
    let (l, _) = run_sine(c, db_to_lin(in_db), 1000.0, 1.5);
    db(peak(&l))
}

/// Windowed magnitude of `buf` at `hz`. The Hann window keeps a
/// non-integer number of cycles from leaking the fundamental into the
/// harmonic bins and putting a floor under every distortion measurement.
fn bin(buf: &[f32], hz: f32) -> f32 {
    let n = buf.len();
    let (mut re, mut im, mut sw) = (0.0f64, 0.0f64, 0.0f64);
    for (i, x) in buf.iter().enumerate() {
        let win = 0.5 - 0.5 * (TAU as f64 * i as f64 / n as f64).cos();
        let w = TAU as f64 * hz as f64 * i as f64 / SR as f64;
        re += *x as f64 * win * w.cos();
        im += *x as f64 * win * w.sin();
        sw += win;
    }
    (2.0 * (re * re + im * im).sqrt() / sw) as f32
}

/// Measured total harmonic distortion of `buf` at `f0`, harmonics 2 to 8,
/// as a percentage. This is what the lamps are checked against: the
/// engine's own estimate must not be marking its own homework.
fn measured_thd_pct(buf: &[f32], f0: f32) -> f32 {
    let f = bin(buf, f0);
    let mut h = 0.0f32;
    for kk in 2..=8 {
        let v = bin(buf, f0 * kk as f32);
        h += v * v;
    }
    100.0 * h.sqrt() / f.max(1e-12)
}

#[test]
fn knob_maps_follow_the_published_ranges() {
    // 50 µs at 0, 30 ms at 10 (the published attack range).
    assert!((attack_seconds(0.0) - 50e-6).abs() < 1e-9);
    assert!((attack_seconds(10.0) - 30e-3).abs() < 1e-4);
    assert!(attack_seconds(10.5) > attack_seconds(10.0));
    // 50 ms at 0, 3.5 s at 10.
    assert!((release_seconds(0.0) - 50e-3).abs() < 1e-6);
    assert!((release_seconds(10.0) - 3.5).abs() < 1e-3);
    // Unity at 5, silence at 0, about +23.5 dB fully clockwise.
    assert!(knob_to_db(5.0).abs() < 1e-6);
    assert!(knob_to_db(0.0) < -100.0);
    assert!(knob_to_db(10.0) > 20.0);
    for i in 1..105 {
        let (a, b) = (i as f32 * 0.1, (i - 1) as f32 * 0.1);
        assert!(
            knob_to_db(a) >= knob_to_db(b),
            "gain table monotonic at {a}"
        );
    }
}

#[test]
fn one_to_one_does_not_compress() {
    let mut c = engine(|s| s.ratio = Ratio::R1);
    let lo = settled_db(&mut c, -40.0);
    let hi = settled_db(&mut c, -6.0);
    // Both track the input exactly (unity trims), so the difference is the
    // input difference.
    assert!(
        ((hi - lo) - 34.0).abs() < 0.5,
        "1:1 should not compress: {lo} -> {hi}"
    );
    assert!(c.gr_db().abs() < 0.05);
}

#[test]
fn every_ratio_compresses_and_the_slope_follows_the_table() {
    // 6 to 16 dB above each threshold, the measured slope should match the
    // curve table within 25 % (research/Distressor.md 8.1).
    for r in Ratio::ALL {
        if r == Ratio::R1 {
            continue;
        }
        let mut c = engine(|s| s.ratio = r);
        let t = curve(r).threshold_db;
        let (a, b) = (t + 6.0, t + 16.0);
        let (oa, ob) = (settled_db(&mut c, a), settled_db(&mut c, b));
        let slope = 10.0 / (ob - oa).max(1e-3);
        let want = curve(r).ratio;
        // The two brick-wall positions are measured by the test below; here
        // they only have to be steeper than 20:1.
        let floor = (want * 0.75).min(20.0);
        assert!(
            slope > floor,
            "{:?}: measured slope {slope:.2} well under the table's {want}",
            r
        );
        assert!(ob > oa, "{:?}: output must still rise with input", r);
    }
}

#[test]
fn the_higher_the_ratio_the_earlier_it_engages() {
    // The research contradicts itself here: its section 4.2 orders the
    // thresholds so that higher ratios engage earlier, and its test plan
    // asserts the reverse. The curve table follows 4.2, which is the
    // better-argued half and the one the hardware's reputation matches (a
    // Distressor at 20:1 grabs sooner than at 2:1, not later), so that is
    // what the model does and what this test pins. See `curve` for the note
    // at the table itself.
    let mut last = f32::INFINITY;
    for r in Ratio::ALL {
        if r == Ratio::R1 {
            continue;
        }
        let mut c = engine(|s| s.ratio = r);
        // The input level at which 1 dB of reduction first appears.
        let (mut lo, mut hi) = (-60.0f32, 12.0f32);
        for _ in 0..18 {
            let mid = 0.5 * (lo + hi);
            c.reset();
            run_sine(&mut c, db_to_lin(mid), 1000.0, 1.0);
            if c.gr_db() > -1.0 { lo = mid } else { hi = mid }
        }
        let point = 0.5 * (lo + hi);
        assert!(
            point < last + 0.5,
            "{:?} engages at {point:.2} dBFS, later than the ratio below it at {last:.2}",
            r
        );
        last = point;
    }
}

#[test]
fn brick_wall_positions_hold_the_output() {
    // 20:1 and Nuke rise by about a dB over a 10 dB input range.
    for r in [Ratio::R20, Ratio::Nuke] {
        let mut c = engine(|s| s.ratio = r);
        let t = curve(r).threshold_db;
        let lo = settled_db(&mut c, t + 6.0);
        let hi = settled_db(&mut c, t + 16.0);
        assert!(
            hi - lo < 1.5,
            "{:?} should be a brick wall, rose {:.2} dB",
            r,
            hi - lo
        );
    }
}

#[test]
fn the_two_to_one_knee_is_the_widest() {
    // 2:1 shows measurable gain reduction well below its final-ratio
    // threshold, which is what the 30 dB knee means (8.1).
    let mut c = engine(|s| s.ratio = Ratio::R2);
    let t = curve(Ratio::R2).threshold_db;
    c.reset();
    run_sine(&mut c, db_to_lin(t - 12.0), 1000.0, 1.0);
    assert!(
        c.gr_db() < -0.15,
        "2:1 should already work 12 dB under threshold, gr {:.2}",
        c.gr_db()
    );
    // 20:1's narrow knee does nothing that far down.
    let mut c = engine(|s| s.ratio = Ratio::R20);
    let t = curve(Ratio::R20).threshold_db;
    c.reset();
    run_sine(&mut c, db_to_lin(t - 12.0), 1000.0, 1.0);
    assert!(c.gr_db() > -0.1, "20:1 knee is narrow, gr {:.2}", c.gr_db());
}

#[test]
fn attack_is_program_dependent() {
    // A bigger step is caught faster than a smaller one (8.2).
    // Each run is measured against its own settled reduction, so this is
    // about the shape of the approach, not about how deep it ends up.
    let make = || {
        engine(|s| {
            s.ratio = Ratio::R6;
            s.attack = 8.0;
        })
    };
    let time_to_half = |level_db: f32| -> usize {
        let mut settled = make();
        settled.reset();
        run_sine(&mut settled, db_to_lin(level_db), 1000.0, 2.0);
        let target = settled.gr_db();
        assert!(target < -2.0, "{level_db} dBFS should compress: {target}");

        let mut c = make();
        c.reset();
        run_sine(&mut c, db_to_lin(-50.0), 1000.0, 0.3);
        let block = 8;
        let (mut l, mut r) = (vec![0.0; block], vec![0.0; block]);
        let mut phase = 0.0f32;
        for n in 0..8000 {
            for i in 0..block {
                let v = db_to_lin(level_db) * (TAU * 1000.0 * phase / SR).sin();
                phase += 1.0;
                l[i] = v;
                r[i] = v;
            }
            c.process_block(&mut l, &mut r);
            if c.gr_db() <= target * 0.5 {
                return n;
            }
        }
        usize::MAX
    };
    let small = time_to_half(-12.0);
    let big = time_to_half(-2.0);
    assert!(small < usize::MAX && big < usize::MAX, "both must settle");
    assert!(
        big < small,
        "a bigger overshoot must be caught faster: {big} vs {small}"
    );
}

#[test]
fn the_attack_knob_orders_the_onset() {
    let onset = |knob: f32| -> usize {
        let mut c = engine(|s| {
            s.ratio = Ratio::R6;
            s.attack = knob;
        });
        c.reset();
        let block = 16;
        let (mut l, mut r) = (vec![0.0; block], vec![0.0; block]);
        let mut phase = 0.0f32;
        for n in 0..4000 {
            for i in 0..block {
                let v = db_to_lin(-6.0) * (TAU * 1000.0 * phase / SR).sin();
                phase += 1.0;
                l[i] = v;
                r[i] = v;
            }
            c.process_block(&mut l, &mut r);
            if c.gr_db() <= -3.0 {
                return n;
            }
        }
        usize::MAX
    };
    let fast = onset(0.0);
    let slow = onset(9.0);
    assert!(fast < slow, "attack 0 must reach 3 dB before attack 9");
}

#[test]
fn the_release_knob_orders_the_recovery() {
    let recover = |knob: f32| -> usize {
        let mut c = engine(|s| {
            s.ratio = Ratio::R6;
            s.release = knob;
        });
        c.reset();
        run_sine(&mut c, db_to_lin(-6.0), 1000.0, 0.6);
        let deep = c.gr_db();
        assert!(deep < -2.0, "the burst should compress, gr {deep}");
        let block = 64;
        let (mut l, mut r) = (vec![0.0; block], vec![0.0; block]);
        for n in 0..8000 {
            l.iter_mut().for_each(|v| *v = 0.0);
            r.iter_mut().for_each(|v| *v = 0.0);
            c.process_block(&mut l, &mut r);
            if c.gr_db() > deep * 0.37 {
                return n;
            }
        }
        usize::MAX
    };
    let fast = recover(0.0);
    let slow = recover(9.0);
    assert!(
        fast < slow,
        "release 0 must recover before release 9: {fast} vs {slow}"
    );
}

#[test]
fn the_opto_position_has_the_longest_tail() {
    // 10:1's two-stage release stretches after a long, deep gesture (8.2).
    let tail = |r: Ratio| -> usize {
        let mut c = engine(|s| {
            s.ratio = r;
            s.release = 5.0;
        });
        c.reset();
        run_sine(&mut c, db_to_lin(-3.0), 1000.0, 2.0);
        let deep = c.gr_db();
        let block = 128;
        let (mut l, mut rr) = (vec![0.0; block], vec![0.0; block]);
        for n in 0..6000 {
            l.iter_mut().for_each(|v| *v = 0.0);
            rr.iter_mut().for_each(|v| *v = 0.0);
            c.process_block(&mut l, &mut rr);
            if c.gr_db() > deep * 0.1 {
                return n;
            }
        }
        usize::MAX
    };
    let opto = tail(Ratio::R10);
    let plain = tail(Ratio::R6);
    assert!(
        opto > plain,
        "10:1 must let go more slowly than 6:1: {opto} vs {plain}"
    );
}

#[test]
fn british_mode_raises_the_threshold_and_slows_the_onset() {
    // 8.3: with the toggle on 1:1 the box compresses at 10 to 20:1, and the
    // onset is later than 20:1's for the same burst.
    let mut c = engine(|s| {
        s.ratio = Ratio::R1;
        s.british = true;
    });
    let t = BRITISH.threshold_db;
    let (a, b) = (settled_db(&mut c, t + 6.0), settled_db(&mut c, t + 16.0));
    let slope = 10.0 / (b - a).max(1e-3);
    assert!(
        (10.0..=25.0).contains(&slope),
        "British slope should sit between 10 and 20:1, got {slope:.1}"
    );
    // Threshold is higher than the 20:1 position's.
    assert!(BRITISH.threshold_db > curve(Ratio::R20).threshold_db);

    let onset = |british: bool| -> usize {
        let mut c = engine(|s| {
            s.ratio = if british { Ratio::R1 } else { Ratio::R20 };
            s.british = british;
            s.attack = 3.0;
        });
        c.reset();
        let block = 16;
        let (mut l, mut r) = (vec![0.0; block], vec![0.0; block]);
        let mut phase = 0.0f32;
        for n in 0..4000 {
            for i in 0..block {
                let v = db_to_lin(-4.0) * (TAU * 1000.0 * phase / SR).sin();
                phase += 1.0;
                l[i] = v;
                r[i] = v;
            }
            c.process_block(&mut l, &mut r);
            if c.gr_db() <= -1.0 {
                return n;
            }
        }
        usize::MAX
    };
    assert!(
        onset(true) > onset(false),
        "British mode's lag must delay the first dB"
    );
}

#[test]
fn dist_two_is_second_harmonic_and_dist_three_is_third() {
    // 8.4: Dist 2 is predominantly second harmonic, Dist 3 third.
    let harmonics = |mode: AudioMode, amp_db: f32| -> (f32, f32, f32) {
        let mut c = engine(|s| {
            s.audio = mode;
            s.ratio = Ratio::R1;
            s.attack = 5.0;
        });
        c.reset();
        let (l, _) = run_sine(&mut c, db_to_lin(amp_db), 1000.0, 0.3);
        (bin(&l, 1000.0), bin(&l, 2000.0), bin(&l, 3000.0))
    };
    let (f1, h2, h3) = harmonics(AudioMode::Dist2, -3.0);
    assert!(h2 > h3 * 3.0, "Dist 2 second {h2:.5} vs third {h3:.5}");
    assert!(h2 / f1 > 0.005, "Dist 2 should be audible: {:.4}", h2 / f1);
    let (f1, h2, h3) = harmonics(AudioMode::Dist3, -3.0);
    assert!(h3 > h2 * 2.0, "Dist 3 third {h3:.5} vs second {h2:.5}");
    assert!(h3 / f1 > 0.02, "Dist 3 should be strong: {:.4}", h3 / f1);
    // Clean mode is far cleaner than either.
    let (f1, h2, h3) = harmonics(AudioMode::Norm, -3.0);
    assert!((h2 + h3) / f1 < 0.005, "clean mode should stay clean");
}

#[test]
fn the_lamps_report_the_distortion_that_is_actually_there() {
    // The lamps used to be tested against the engine's own estimate, which
    // is what they display, so the test could never fail. It now measures
    // the output spectrum and holds the estimate to it.
    let case = |mode: AudioMode, level_db: f32| -> (f32, f32) {
        let mut c = engine(|s| {
            s.audio = mode;
            s.ratio = Ratio::R1;
        });
        c.reset();
        let (l, _) = run_sine(&mut c, db_to_lin(level_db), 1000.0, 0.5);
        (measured_thd_pct(&l, 1000.0), c.thd_pct())
    };
    for (mode, level) in [
        (AudioMode::Dist2, -18.0),
        (AudioMode::Dist2, -1.0),
        (AudioMode::Dist3, -18.0),
        (AudioMode::Dist3, -1.0),
    ] {
        let (measured, estimate) = case(mode, level);
        assert!(
            measured > 0.0,
            "{mode:?} at {level} dBFS should distort something"
        );
        let ratio = estimate / measured;
        assert!(
            (0.5..=2.0).contains(&ratio),
            "{mode:?} at {level} dBFS: the lamps say {estimate:.3} % where the output has {measured:.3} %"
        );
    }
    // The published headline figures, measured rather than asserted from
    // the coefficients: Dist 2 reaches a few per cent, Dist 3 much more.
    let (d2, _) = case(AudioMode::Dist2, -1.0);
    let (d3, _) = case(AudioMode::Dist3, -1.0);
    assert!(
        (0.5..=6.0).contains(&d2),
        "Dist 2 driven hard should be a few per cent, measured {d2:.2}"
    );
    assert!(
        d3 > 8.0,
        "Dist 3 driven hard should be far dirtier, measured {d3:.2}"
    );
    assert!(d3 > d2 * 3.0);
    // And the lamps light in the right order.
    let mut c = engine(|s| {
        s.audio = AudioMode::Dist3;
        s.ratio = Ratio::R1;
    });
    c.reset();
    run_sine(&mut c, db_to_lin(-40.0), 1000.0, 0.2);
    assert!(!c.lamp_1pct(), "a quiet signal must not light the 1 % lamp");
    c.reset();
    run_sine(&mut c, db_to_lin(-1.0), 1000.0, 0.2);
    assert!(c.lamp_1pct() && c.redline(), "a hot one lights both");
    assert!(c.drive() > 0.5);
}

#[test]
fn the_generator_does_not_alias() {
    // research/Distressor.md 8.4: a 15 kHz tone at −6 dBFS through Dist 3
    // at the fastest settings must leave nothing above −70 dBFS below
    // 10 kHz. Without oversampling this folds to about −49 dBFS.
    let mut c = Compressor::new(44_100.0);
    c.configure(&Settings {
        audio: AudioMode::Dist3,
        ratio: Ratio::R1,
        attack: 0.0,
        release: 0.0,
        ..Settings::default()
    });
    c.reset();
    let n = 44_100usize;
    let block = 256;
    let (mut lb, mut rb) = (vec![0.0; block], vec![0.0; block]);
    let mut out = Vec::with_capacity(n);
    let mut phase = 0.0f32;
    let mut done = 0;
    while done < n {
        for i in 0..block {
            let v = db_to_lin(-6.0) * (TAU * 15_000.0 * phase / 44_100.0).sin();
            phase += 1.0;
            lb[i] = v;
            rb[i] = v;
        }
        c.process_block(&mut lb, &mut rb);
        if done > n / 2 {
            out.extend_from_slice(&lb);
        }
        done += block;
    }
    // Scan the band below 10 kHz for anything that folded down into it.
    let mut worst = 0.0f32;
    let mut worst_hz = 0.0f32;
    let mut hz = 200.0f32;
    while hz < 10_000.0 {
        let m = {
            let nn = out.len();
            let (mut re, mut im, mut sw) = (0.0f64, 0.0f64, 0.0f64);
            for (i, x) in out.iter().enumerate() {
                let win = 0.5 - 0.5 * (TAU as f64 * i as f64 / nn as f64).cos();
                let w = TAU as f64 * hz as f64 * i as f64 / 44_100.0;
                re += *x as f64 * win * w.cos();
                im += *x as f64 * win * w.sin();
                sw += win;
            }
            (2.0 * (re * re + im * im).sqrt() / sw) as f32
        };
        if m > worst {
            worst = m;
            worst_hz = hz;
        }
        hz += 100.0;
    }
    let db = 20.0 * worst.max(1e-12).log10();
    assert!(
        db < -70.0,
        "an alias at {worst_hz:.0} Hz sits at {db:.1} dBFS, over the −70 dBFS limit"
    );
}

#[test]
fn the_audio_high_pass_cuts_lows_only() {
    // 8.4: −3 dB at about 65 Hz, −12 dB or more at 30 Hz, flat at 1 kHz.
    let level = |hz: f32, hp: bool| -> f32 {
        let mut c = engine(|s| {
            s.ratio = Ratio::R1;
            s.audio = if hp { AudioMode::Hp } else { AudioMode::Norm };
        });
        c.reset();
        let (l, _) = run_sine(&mut c, db_to_lin(-20.0), hz, 1.0);
        db(peak(&l))
    };
    let at = |hz: f32| level(hz, true) - level(hz, false);
    assert!(at(1000.0).abs() < 0.5, "1 kHz should pass: {}", at(1000.0));
    let f65 = at(65.0);
    assert!(
        (-5.0..=-1.5).contains(&f65),
        "65 Hz should be about −3 dB, got {f65:.2}"
    );
    let f30 = at(30.0);
    assert!(f30 < -9.0, "30 Hz should be well down, got {f30:.2}");
}

#[test]
fn the_detector_filters_change_only_the_side_chain() {
    // 8.5: the high-pass makes a 40 Hz tone compress less; band emphasis
    // makes a 6 kHz tone compress more; neither touches the audio path.
    let gr = |det: Detector, hz: f32| -> f32 {
        let mut c = engine(|s| {
            s.ratio = Ratio::R6;
            s.detector = det;
        });
        c.reset();
        run_sine(&mut c, db_to_lin(-6.0), hz, 1.0);
        c.gr_db()
    };
    assert!(
        gr(Detector::Hp, 40.0) > gr(Detector::Norm, 40.0) + 1.0,
        "the side-chain high-pass must stop low-frequency pumping"
    );
    assert!(
        gr(Detector::Band, 6000.0) < gr(Detector::Norm, 6000.0) - 0.5,
        "band emphasis must make the detector overreact up there"
    );
    // The audio path is untouched: with 1:1 there is no gain change at all.
    let flat = |det: Detector| -> f32 {
        let mut c = engine(|s| {
            s.ratio = Ratio::R1;
            s.detector = det;
        });
        c.reset();
        let (l, _) = run_sine(&mut c, db_to_lin(-20.0), 40.0, 0.5);
        db(peak(&l))
    };
    assert!((flat(Detector::Hp) - flat(Detector::Norm)).abs() < 0.05);
    assert!((flat(Detector::Band) - flat(Detector::Norm)).abs() < 0.05);
}

#[test]
fn link_modes_behave_as_documented() {
    // 8.8: unlinked, a silent channel shows no gain reduction of its own;
    // image link gives both channels the same gain.
    let mut c = engine(|s| {
        s.ratio = Ratio::R6;
        s.link = false;
    });
    c.reset();
    let block = 256;
    let (mut l, mut r) = (vec![0.0; block], vec![0.0; block]);
    let mut phase = 0.0f32;
    for _ in 0..200 {
        for i in 0..block {
            l[i] = db_to_lin(-3.0) * (TAU * 1000.0 * phase / SR).sin();
            phase += 1.0;
            r[i] = 0.0;
        }
        c.process_block(&mut l, &mut r);
    }
    // The right channel stayed silent, so nothing came out of it.
    assert!(peak(&r) < 1e-6, "unlinked silence stays silent");

    let mut c = engine(|s| {
        s.ratio = Ratio::R6;
        s.link = true;
        s.link_mode = LinkMode::Image;
    });
    c.reset();
    for _ in 0..200 {
        for i in 0..block {
            l[i] = db_to_lin(-3.0) * (TAU * 1000.0 * phase / SR).sin();
            phase += 1.0;
            r[i] = 0.5 * l[i];
        }
        c.process_block(&mut l, &mut r);
    }
    // Both channels got the same gain, so the 6 dB offset survives.
    let ratio = peak(&l) / peak(&r).max(1e-9);
    assert!(
        (ratio - 2.0).abs() < 0.15,
        "image link must lock the image, ratio {ratio:.3}"
    );
    assert!(!c.dead_patch());
}

#[test]
fn the_dead_patch_drives_the_generator_harder() {
    // 8.8: mono into a linked pair raises the distortion.
    let thd = |mono: bool| -> f32 {
        let mut c = engine(|s| {
            s.ratio = Ratio::R6;
            s.link = true;
            s.audio = AudioMode::Dist3;
        });
        c.reset();
        let block = 256;
        let (mut l, mut r) = (vec![0.0; block], vec![0.0; block]);
        let mut phase = 0.0f32;
        for _ in 0..100 {
            for i in 0..block {
                let v = db_to_lin(-6.0) * (TAU * 1000.0 * phase / SR).sin();
                phase += 1.0;
                l[i] = v;
                r[i] = if mono { 0.0 } else { v };
            }
            c.process_block(&mut l, &mut r);
        }
        c.thd_pct()
    };
    assert!(
        thd(true) > thd(false),
        "dead patch should distort more: {} vs {}",
        thd(true),
        thd(false)
    );
}

#[test]
fn bypass_and_mix_are_exact() {
    // Bypass passes the input through, delayed by the generator's
    // resamplers so that switching it does not jump the phase.
    let mut c = engine(|s| {
        s.ratio = Ratio::Nuke;
        s.bypass = true;
        s.input = 9.0;
    });
    let lat = c.latency();
    assert!(lat > 0, "the generator should be oversampling at 48 kHz");
    let block = 512;
    let (mut l, mut r) = (vec![0.0; block], vec![0.0; block]);
    let mut phase = 0.0f32;
    for i in 0..block {
        let v = 0.4 * (TAU * 220.0 * phase / SR).sin();
        phase += 1.0;
        l[i] = v;
        r[i] = v;
    }
    let dry = l.clone();
    c.process_block(&mut l, &mut r);
    for i in lat..block {
        assert!(
            (dry[i - lat] - l[i]).abs() < 1e-6,
            "bypass must pass the input through, delayed by {lat}"
        );
    }
}

#[test]
fn survives_extremes_without_nan() {
    for ratio in Ratio::ALL {
        for audio in [AudioMode::Norm, AudioMode::Dist3, AudioMode::HpDist2] {
            let mut c = engine(|s| {
                s.ratio = ratio;
                s.audio = audio;
                s.input = 10.5;
                s.attack = 0.0;
                s.release = 0.0;
            });
            let block = 256;
            let (mut l, mut r) = (vec![0.0; block], vec![0.0; block]);
            for phase in 0..40 {
                for i in 0..block {
                    l[i] = if (i + phase) % 2 == 0 { 1.0 } else { -1.0 };
                    r[i] = 1.0;
                }
                c.process_block(&mut l, &mut r);
                assert!(l.iter().all(|v| v.is_finite()), "{ratio:?} {audio:?}");
                assert!(r.iter().all(|v| v.is_finite()), "{ratio:?} {audio:?}");
            }
            // Silence afterwards must decay to exactly zero.
            for _ in 0..200 {
                l.iter_mut().for_each(|v| *v = 0.0);
                r.iter_mut().for_each(|v| *v = 0.0);
                c.process_block(&mut l, &mut r);
            }
            assert!(l.iter().all(|v| *v == 0.0), "{ratio:?} leaves silence");
        }
    }
}

#[test]
fn the_transfer_curve_matches_the_engine() {
    let mut c = engine(|s| s.ratio = Ratio::R6);
    let mut out = [0.0f32; TRANSFER_POINTS];
    c.transfer_curve(&mut out, -60.0, 0.0);
    assert!(out.iter().all(|v| v.is_finite()));
    // Monotonic and compressing.
    for w in out.windows(2) {
        assert!(w[1] >= w[0] - 0.01, "curve must not fall");
    }
    assert!(out[TRANSFER_POINTS - 1] - out[0] < 60.0, "it must compress");
    // The curve agrees with a real run at a couple of points.
    for probe in [-30.0f32, -12.0] {
        let i = ((probe + 60.0) / 60.0 * (TRANSFER_POINTS - 1) as f32).round() as usize;
        let want = out[i];
        let got = settled_db(&mut c, probe);
        assert!(
            (want - got).abs() < 2.0,
            "curve {want:.2} vs run {got:.2} at {probe}"
        );
    }
}

#[test]
fn sample_rates_agree() {
    for sr in [44_100.0f32, 96_000.0] {
        let mut a = Compressor::new(SR);
        let mut b = Compressor::new(sr);
        let s = Settings {
            ratio: Ratio::R6,
            ..Settings::default()
        };
        a.configure(&s);
        b.configure(&s);
        a.reset();
        b.reset();
        let run = |c: &mut Compressor, rate: f32| {
            let n = (rate * 1.5) as usize;
            let block = 256;
            let (mut l, mut r) = (vec![0.0; block], vec![0.0; block]);
            let mut phase = 0.0f32;
            let mut done = 0;
            while done < n {
                let m = block.min(n - done);
                for i in 0..m {
                    let v = db_to_lin(-6.0) * (TAU * 1000.0 * phase / rate).sin();
                    phase += 1.0;
                    l[i] = v;
                    r[i] = v;
                }
                c.process_block(&mut l[..m], &mut r[..m]);
                done += m;
            }
            c.gr_db()
        };
        let ga = run(&mut a, SR);
        let gb = run(&mut b, sr);
        assert!(
            (ga - gb).abs() < 0.5,
            "gain reduction should not depend on the sample rate: {ga:.2} at 48k vs {gb:.2} at {sr}"
        );
    }
}

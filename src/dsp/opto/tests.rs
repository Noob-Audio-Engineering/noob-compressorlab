//! The research test plan (`research/LA-2A.md` section 8) as unit tests. Every
//! test drives the model offline with sines and bursts and checks the
//! published behaviour with the research tolerances; the constants in
//! `model.rs` were tuned until these pass.

use super::model::*;
use std::f32::consts::{PI, SQRT_2};

const SR: f32 = 48_000.0;
const BLOCK: usize = 256;

/// Peak amplitude of a sine `db` dB above 0 VU (−18 dBFS RMS).
fn amp_vu(db: f32) -> f32 {
    VU_REF_AMP * 10f32.powf(db / 20.0)
}

fn settings(pr: f32) -> Settings {
    Settings {
        peak_reduction: pr,
        ..Settings::default()
    }
}

/// Run a sine of `amp` and `hz` for `seconds` through `comp` (in place on
/// fresh buffers), returning one gain-reduction reading per block and the
/// full output.
fn run_sine(
    comp: &mut Compressor,
    amp: f32,
    hz: f32,
    seconds: f32,
    sr: f32,
) -> (Vec<f32>, Vec<f32>) {
    let n = (seconds * sr) as usize / BLOCK * BLOCK;
    let mut out = Vec::with_capacity(n);
    let mut gr = Vec::with_capacity(n / BLOCK);
    let mut l = vec![0.0f32; BLOCK];
    let mut r = vec![0.0f32; BLOCK];
    let mut phase = 0.0f32;
    for _ in 0..n / BLOCK {
        for i in 0..BLOCK {
            phase += hz / sr;
            if phase >= 1.0 {
                phase -= 1.0;
            }
            l[i] = amp * (phase * 2.0 * PI).sin();
            r[i] = l[i];
        }
        comp.process_block(&mut l, &mut r);
        gr.push(comp.gain_reduction_db(0));
        out.extend_from_slice(&l);
    }
    (gr, out)
}

/// Goertzel magnitude of `hz` in `x` at `sr`.
fn goertzel(x: &[f32], hz: f32, sr: f32) -> f32 {
    let w = 2.0 * PI * hz / sr;
    let c = 2.0 * w.cos();
    let (mut s1, mut s2) = (0.0f64, 0.0f64);
    for &v in x {
        let s0 = v as f64 + c as f64 * s1 - s2;
        s2 = s1;
        s1 = s0;
    }
    let re = s1 - s2 * w.cos() as f64;
    let im = s2 * w.sin() as f64;
    (re * re + im * im).sqrt() as f32 / x.len() as f32 * 2.0
}

/// THD (fraction) and the second and third harmonic levels relative to the
/// fundamental, over the last second of `out`.
fn thd(out: &[f32], hz: f32, sr: f32) -> (f32, f32, f32) {
    let tail = &out[out.len() - sr as usize..];
    let f = goertzel(tail, hz, sr);
    let h: Vec<f32> = (2..=6).map(|k| goertzel(tail, hz * k as f32, sr)).collect();
    let sum: f32 = h.iter().map(|v| v * v).sum::<f32>().sqrt();
    (sum / f, h[0] / f, h[1] / f)
}

/// Steady-state gain reduction for `amp` at `pr`, measured by processing.
fn measured_gr(pr: f32, amp: f32, limit: bool) -> f32 {
    let mut c = Compressor::new(SR);
    c.configure(Settings {
        limit,
        ..settings(pr)
    });
    let (gr, _) = run_sine(&mut c, amp, 1000.0, 4.0, SR);
    gr[gr.len() - 20..].iter().sum::<f32>() / 20.0
}

#[test]
fn bypass_is_transparent_and_the_tube_stage_is_clean() {
    let mut c = Compressor::new(SR);
    c.configure(Settings {
        bypass: true,
        ..settings(60.0)
    });
    let mut l: Vec<f32> = (0..2048).map(|i| ((i as f32) * 0.1).sin() * 0.5).collect();
    let mut r = l.clone();
    let orig = l.clone();
    c.process_block(&mut l, &mut r);
    for (a, b) in l.iter().zip(&orig) {
        assert!((a - b).abs() < 1e-6, "bypass changed the signal");
    }
    // PR 0, Gain at unity: no reduction and THD below 0.3 % at 0 VU.
    let mut c = Compressor::new(SR);
    c.configure(settings(0.0));
    let (gr, out) = run_sine(&mut c, amp_vu(0.0), 1000.0, 2.0, SR);
    assert!(
        gr.last().unwrap() < &0.1,
        "PR 0 must not compress: {}",
        gr.last().unwrap()
    );
    let (t, _, _) = thd(&out, 1000.0, SR);
    assert!(t < 0.003, "THD at 0 VU with PR 0 = {:.4}", t);
    // Flat within +0 / −1 dB from 30 Hz to 15 kHz.
    for hz in [30.0, 100.0, 1000.0, 10_000.0, 15_000.0] {
        let mut c = Compressor::new(SR);
        c.configure(settings(0.0));
        let (_, out) = run_sine(&mut c, 0.05, hz, 1.5, SR);
        let g = goertzel(&out[out.len() - 24_000..], hz, SR) / 0.05;
        let db = 20.0 * g.log10();
        assert!(
            (-1.05..=0.3).contains(&db),
            "response at {hz} Hz = {db:.2} dB"
        );
    }
}

#[test]
fn the_tube_stage_hits_its_two_published_distortion_points() {
    // research/LA-2A.md 7.2 sets the tube constant from two figures: about
    // 0.75 % at the +16 dBu equivalent and about 0.3 % at +10 dBu. The
    // constant's own comment claimed the first; nothing measured either.
    let at = |dbu: f32| -> f32 {
        let mut c = Compressor::new(SR);
        c.configure(settings(0.0));
        let (_, out) = run_sine(&mut c, amp_vu(dbu - 4.0), 1000.0, 2.0, SR);
        thd(&out, 1000.0, SR).0 * 100.0
    };
    let hot = at(16.0);
    let nominal = at(10.0);
    assert!(
        (0.4..=1.2).contains(&hot),
        "+16 dBu is published at about 0.75 %, measured {hot:.3} %"
    );
    assert!(
        (0.15..=0.5).contains(&nominal),
        "+10 dBu is published at about 0.3 %, measured {nominal:.3} %"
    );
    assert!(hot > nominal, "and it must rise with level");
}

#[test]
fn steady_state_reduction_follows_peak_reduction_and_level() {
    let c = {
        let mut c = Compressor::new(SR);
        c.configure(settings(30.0));
        c
    };
    // PR 30: 1 dB of reduction within ±1 dB of 0 VU.
    let (mut lo, mut hi) = (-20.0f32, 20.0f32);
    for _ in 0..40 {
        let mid = 0.5 * (lo + hi);
        if c.static_gr_db(amp_vu(mid)) < 1.0 {
            lo = mid
        } else {
            hi = mid
        }
    }
    let onset_db = 0.5 * (lo + hi);
    assert!(
        onset_db.abs() <= 1.0,
        "PR 30 onset at {onset_db:.2} dB re 0 VU"
    );
    // The processed model agrees with the static solver.
    let m = measured_gr(30.0, amp_vu(0.0), false);
    assert!(
        (m - c.static_gr_db(amp_vu(0.0))).abs() < 0.4,
        "measured {m:.2} vs static {:.2}",
        c.static_gr_db(amp_vu(0.0))
    );
    // PR 50: about 5 dB at 0 VU.
    let g50 = measured_gr(50.0, amp_vu(0.0), false);
    assert!((3.5..=6.5).contains(&g50), "PR 50 at 0 VU = {g50:.2} dB");
    // PR 0 never compresses up to +16; PR 100 at +16 gives 30 to 40 dB.
    assert!(measured_gr(0.0, amp_vu(16.0), false) < 0.5);
    let g100 = measured_gr(100.0, amp_vu(16.0), false);
    assert!(
        (30.0..=40.0).contains(&g100),
        "PR 100 at +16 = {g100:.2} dB"
    );
    // Monotonic in both variables.
    let mut prev = -1.0;
    for pr in [0.0, 20.0, 30.0, 40.0, 50.0, 60.0, 80.0, 100.0] {
        let mut cc = Compressor::new(SR);
        cc.configure(settings(pr));
        let g = cc.static_gr_db(amp_vu(0.0));
        assert!(g >= prev - 1e-3, "GR not monotonic in PR at {pr}");
        prev = g;
        let mut plev = -1.0;
        for db in (-40..=16).step_by(4) {
            let g = cc.static_gr_db(amp_vu(db as f32));
            assert!(
                g >= plev - 1e-3,
                "GR not monotonic in level at PR {pr}, {db} dB"
            );
            plev = g;
        }
    }
}

/// Local slope of the input/output curve (dB out per dB in) around `db_in`.
fn ratio_at(c: &Compressor, db_in: f32) -> f32 {
    let g0 = c.static_gr_db(amp_vu(db_in - 1.0));
    let g1 = c.static_gr_db(amp_vu(db_in + 1.0));
    2.0 / (2.0 - (g1 - g0))
}

#[test]
fn ratio_and_knee_compress_versus_limit() {
    let mut c = Compressor::new(SR);
    c.configure(settings(50.0));
    // Find the levels giving 6 and 20 dB of reduction in Compress.
    let level_for = |c: &Compressor, target: f32| {
        let (mut lo, mut hi) = (-30.0f32, 60.0f32);
        for _ in 0..50 {
            let mid = 0.5 * (lo + hi);
            if c.static_gr_db(amp_vu(mid)) < target {
                lo = mid
            } else {
                hi = mid
            }
        }
        0.5 * (lo + hi)
    };
    let l6 = level_for(&c, 6.0);
    let l20 = level_for(&c, 20.0);
    for db in [l6, 0.5 * (l6 + l20), l20] {
        let r = ratio_at(&c, db);
        assert!(
            (2.5..=4.5).contains(&r),
            "Compress ratio at {db:.1} dB = {r:.2}:1"
        );
    }
    // Soft knee: the ratio grows smoothly, no corner (no jump over 1:1 between neighbours).
    let mut last = ratio_at(&c, -30.0);
    let mut db = -29.0;
    while db < l20 {
        let r = ratio_at(&c, db);
        assert!(
            r - last < 1.0,
            "knee has a corner at {db:.1} dB ({last:.2} -> {r:.2})"
        );
        last = r;
        db += 1.0;
    }
    // Limit versus Compress.
    let mut lim = Compressor::new(SR);
    lim.configure(Settings {
        limit: true,
        ..settings(50.0)
    });
    let l3 = level_for(&c, 3.0);
    let d3 = (lim.static_gr_db(amp_vu(l3)) - 3.0).abs();
    assert!(
        d3 < 0.3,
        "Limit differs from Compress by {d3:.2} dB at 3 dB GR"
    );
    let extra = lim.static_gr_db(amp_vu(l20)) - 20.0;
    assert!(
        extra >= 4.0,
        "Limit gives only {extra:.2} dB more than Compress at 20 dB"
    );
    // ... and the processed model agrees in Limit too.
    let m = measured_gr(50.0, amp_vu(l20), true);
    assert!(
        (m - lim.static_gr_db(amp_vu(l20))).abs() < 0.6,
        "Limit measured {m:.2} vs static {:.2}",
        lim.static_gr_db(amp_vu(l20))
    );
}

/// Time (seconds) until the block GR first reaches `frac` of `final_gr`
/// after `start_block`.
fn time_to(gr: &[f32], start_block: usize, from: f32, to: f32, frac: f32, sr: f32) -> f32 {
    let target = from + (to - from) * frac;
    for (i, g) in gr.iter().enumerate().skip(start_block) {
        let hit = if to > from {
            *g >= target
        } else {
            *g <= target
        };
        if hit {
            return (i - start_block) as f32 * BLOCK as f32 / sr;
        }
    }
    f32::INFINITY
}

/// A tone at `low_db` for `pre` seconds, then at `high_db` for `on`
/// seconds, then back at `low_db` for `post` seconds; returns the block GR
/// trace and the block index where the step up and the step down happen.
fn burst(
    c: &mut Compressor,
    low_db: f32,
    high_db: f32,
    pre: f32,
    on: f32,
    post: f32,
    sr: f32,
) -> (Vec<f32>, usize, usize) {
    let mut gr = Vec::new();
    let mut l = vec![0.0f32; BLOCK];
    let mut r = vec![0.0f32; BLOCK];
    let mut phase = 0.0f32;
    let mut seg = |c: &mut Compressor, db: f32, secs: f32, gr: &mut Vec<f32>| {
        let amp = if db.is_finite() { amp_vu(db) } else { 0.0 };
        for _ in 0..((secs * sr) as usize / BLOCK) {
            for i in 0..BLOCK {
                phase += 1000.0 / sr;
                if phase >= 1.0 {
                    phase -= 1.0;
                }
                l[i] = amp * (phase * 2.0 * PI).sin();
                r[i] = l[i];
            }
            c.process_block(&mut l, &mut r);
            gr.push(c.gain_reduction_db(0));
        }
    };
    seg(c, low_db, pre, &mut gr);
    let up = gr.len();
    seg(c, high_db, on, &mut gr);
    let down = gr.len();
    seg(c, low_db, post, &mut gr);
    (gr, up, down)
}

#[test]
fn attack_is_about_ten_milliseconds_and_level_dependent() {
    // −24 dB tone stepping to −3 dB at PR 50 (the Canopus test).
    let mut c = Compressor::new(SR);
    c.configure(settings(50.0));
    let (gr, up, down) = burst(&mut c, -24.0, -3.0, 1.0, 1.5, 0.1, SR);
    let g0 = gr[up - 1];
    let g1 = gr[down - 1];
    assert!(g1 - g0 > 2.0, "the step must compress ({g0:.2} -> {g1:.2})");
    let t63 = time_to(&gr, up, g0, g1, 0.63, SR);
    assert!(
        (0.005..=0.060).contains(&t63),
        "63 % of the reduction in {:.1} ms",
        t63 * 1000.0
    );
    // Stabilisation: the slope over 10 ms drops to 1 % of the initial slope
    // (the gain reduction keeps creeping slowly afterwards as the traps
    // fill, which the original does too).
    let two = (0.010 * SR / BLOCK as f32).round() as usize;
    let slope0 = (gr[up + two] - gr[up]).abs().max(1e-6);
    let mut settled = f32::INFINITY;
    for i in up..gr.len() - two {
        if (gr[i + two] - gr[i]).abs() < 0.01 * slope0 {
            settled = (i - up) as f32 * BLOCK as f32 / SR;
            break;
        }
    }
    assert!(
        (0.015..=0.100).contains(&settled),
        "stabilised in {:.1} ms",
        settled * 1000.0
    );
    // A 6 dB step attacks slower than an 18 dB step (in time to 63 %).
    let mut c6 = Compressor::new(SR);
    c6.configure(settings(50.0));
    let (gr6, up6, down6) = burst(&mut c6, -6.0, 0.0, 1.0, 1.0, 0.1, SR);
    let t6 = time_to(&gr6, up6, gr6[up6 - 1], gr6[down6 - 1], 0.63, SR);
    let mut c18 = Compressor::new(SR);
    c18.configure(settings(50.0));
    let (gr18, up18, down18) = burst(&mut c18, -6.0, 12.0, 1.0, 1.0, 0.1, SR);
    let t18 = time_to(&gr18, up18, gr18[up18 - 1], gr18[down18 - 1], 0.63, SR);
    assert!(
        t18 < t6,
        "18 dB step ({:.1} ms) should attack faster than a 6 dB step ({:.1} ms)",
        t18 * 1000.0,
        t6 * 1000.0
    );
}

/// Peak Reduction that gives about `target` dB of reduction at `db` re 0 VU.
fn pr_for(target: f32, db: f32) -> f32 {
    let (mut lo, mut hi) = (0.0f32, 100.0f32);
    for _ in 0..40 {
        let mid = 0.5 * (lo + hi);
        let mut c = Compressor::new(SR);
        c.configure(settings(mid));
        if c.static_gr_db(amp_vu(db)) < target {
            lo = mid
        } else {
            hi = mid
        }
    }
    0.5 * (lo + hi)
}

#[test]
fn release_has_two_stages() {
    let pr = pr_for(10.0, 0.0);
    let mut c = Compressor::new(SR);
    c.configure(settings(pr));
    let (gr, _up, down) = burst(&mut c, f32::NEG_INFINITY, 0.0, 0.2, 2.0, 4.0, SR);
    let g = gr[down - 1];
    assert!((8.0..=12.0).contains(&g), "burst reduction {g:.2} dB");
    let t50 = time_to(&gr, down, g, 0.0, 0.5, SR);
    assert!(
        (0.040..=0.120).contains(&t50),
        "half the reduction gone in {:.0} ms",
        t50 * 1000.0
    );
    let t90 = time_to(&gr, down, g, 0.0, 0.9, SR);
    assert!((0.5..=3.0).contains(&t90), "90 % recovered in {:.2} s", t90);
}

#[test]
fn the_cell_remembers_long_hard_compression() {
    let pr = pr_for(20.0, 0.0);
    let mut short = Compressor::new(SR);
    short.configure(settings(pr));
    let (gs, _, ds) = burst(&mut short, f32::NEG_INFINITY, 0.0, 0.2, 0.1, 12.0, SR);
    let mut long = Compressor::new(SR);
    long.configure(settings(pr));
    let (gl, _, dl) = burst(&mut long, f32::NEG_INFINITY, 0.0, 0.2, 20.0, 30.0, SR);
    let t90_short = time_to(&gs, ds, gs[ds - 1], 0.0, 0.9, SR);
    let t90_long = time_to(&gl, dl, gl[dl - 1], 0.0, 0.9, SR);
    assert!(
        t90_long >= 2.0 * t90_short,
        "memory: 90 % recovery {:.2} s after 20 s vs {:.2} s after 100 ms",
        t90_long,
        t90_short
    );
    let t99_long = time_to(&gl, dl, gl[dl - 1], 0.0, 0.99, SR);
    assert!(
        t99_long >= 5.0,
        "long-burst tail to 99 % is only {:.2} s",
        t99_long
    );
    assert!(t99_long.is_finite(), "the cell never recovered");
}

#[test]
fn highs_get_more_reduction_and_r37_shapes_the_lows() {
    let pr = 50.0;
    let mut onset = Compressor::new(SR);
    onset.configure(settings(pr));
    // 18 dB above the 1 kHz onset.
    let (mut lo, mut hi) = (-30.0f32, 40.0f32);
    for _ in 0..40 {
        let mid = 0.5 * (lo + hi);
        if onset.static_gr_db(amp_vu(mid)) < 1.0 {
            lo = mid
        } else {
            hi = mid
        }
    }
    let level = 0.5 * (lo + hi) + 18.0;
    let gr_at = |hz: f32, emphasis: f32| {
        let mut c = Compressor::new(SR);
        c.configure(Settings {
            emphasis,
            ..settings(pr)
        });
        let (gr, _) = run_sine(&mut c, amp_vu(level), hz, 4.0, SR);
        gr[gr.len() - 20..].iter().sum::<f32>() / 20.0
    };
    let g100 = gr_at(100.0, 1.0);
    let g10k = gr_at(10_000.0, 1.0);
    assert!(
        (2.0..=6.0).contains(&(g10k - g100)),
        "10 kHz gets {:.2} dB more reduction than 100 Hz",
        g10k - g100
    );
    let g100e = gr_at(100.0, 0.0);
    let g10ke = gr_at(10_000.0, 0.0);
    // The research asks for 6 to 12 dB here, not merely "some": R37 wound
    // fully off is what stops the bottom end from driving the cell.
    assert!(
        (6.0..=12.0).contains(&(g100 - g100e)),
        "R37 at 0 should take 6 to 12 dB off the 100 Hz reduction, it took {:.2}",
        g100 - g100e
    );
    assert!(
        (g10k - g10ke).abs() < 1.0,
        "R37 changed 10 kHz by {:.2} dB",
        g10k - g10ke
    );
}

#[test]
fn distortion_under_reduction_is_odd_and_modest() {
    let pr = pr_for(6.0, 0.0);
    let mut c = Compressor::new(SR);
    c.configure(settings(pr));
    let (gr, out) = run_sine(&mut c, amp_vu(0.0), 1000.0, 3.0, SR);
    assert!((5.0..=7.0).contains(gr.last().unwrap()));
    let (t, h2, h3) = thd(&out, 1000.0, SR);
    assert!(
        (0.008..=0.04).contains(&t),
        "THD with 6 dB GR = {:.3} %",
        t * 100.0
    );
    assert!(
        h3 > h2,
        "third harmonic ({h3:.4}) should dominate the second ({h2:.4})"
    );
}

#[test]
fn stereo_link_shares_one_cell() {
    let mut c = Compressor::new(SR);
    c.configure(Settings {
        link: true,
        ..settings(60.0)
    });
    let (_, _) = run_sine(&mut c, amp_vu(0.0), 1000.0, 2.0, SR);
    assert!((c.gain_reduction_db(0) - c.gain_reduction_db(1)).abs() < 1e-4);
    // Hard-panned burst: linked reduction sits between the two unlinked ones.
    let hard = |link: bool| {
        let mut c = Compressor::new(SR);
        c.configure(Settings {
            link,
            ..settings(60.0)
        });
        let mut l = vec![0.0f32; BLOCK];
        let mut r = vec![0.0f32; BLOCK];
        let mut phase = 0.0f32;
        for _ in 0..(2.0 * SR) as usize / BLOCK {
            for i in 0..BLOCK {
                phase += 1000.0 / SR;
                if phase >= 1.0 {
                    phase -= 1.0;
                }
                l[i] = amp_vu(6.0) * (phase * 2.0 * PI).sin();
                r[i] = 0.0;
            }
            c.process_block(&mut l, &mut r);
        }
        (c.gain_reduction_db(0), c.gain_reduction_db(1))
    };
    let (ul, ur) = hard(false);
    let (ll, _) = hard(true);
    assert!(
        ul > ur + 3.0,
        "unlinked: the loud channel must be reduced more ({ul:.2} vs {ur:.2})"
    );
    assert!(
        ll < ul && ll > ur,
        "linked {ll:.2} should lie between {ur:.2} and {ul:.2}"
    );
}

#[test]
fn numerical_hygiene() {
    let mut c = Compressor::new(SR);
    c.configure(settings(100.0));
    // Heavy reduction, then long silence: everything decays to exactly zero.
    let _ = run_sine(&mut c, amp_vu(16.0), 1000.0, 3.0, SR);
    let mut l = vec![0.0f32; BLOCK];
    let mut r = vec![0.0f32; BLOCK];
    for _ in 0..(60.0 * SR) as usize / BLOCK {
        c.process_block(&mut l, &mut r);
    }
    let state = c.cell_state();
    for v in state {
        assert!(v == 0.0 || v.is_normal(), "denormal state {v:e}");
    }
    assert!(c.gain_reduction_db(0) < 1e-3);
    // Hostile inputs stay finite.
    for amp in [10.0f32, -10.0, 0.0] {
        let mut c = Compressor::new(SR);
        c.configure(settings(80.0));
        let mut l = vec![amp; BLOCK];
        let mut r = vec![amp; BLOCK];
        for _ in 0..200 {
            c.process_block(&mut l, &mut r);
            l.fill(amp);
            r.fill(amp);
        }
        assert!(l.iter().all(|v| v.is_finite()) && c.gain_reduction_db(0).is_finite());
    }
}

#[test]
fn behaviour_is_sample_rate_independent() {
    let mut ref48 = Compressor::new(48_000.0);
    ref48.configure(settings(55.0));
    let (g48, _) = run_sine(&mut ref48, amp_vu(3.0), 1000.0, 3.0, 48_000.0);
    for sr in [44_100.0, 96_000.0] {
        let mut c = Compressor::new(sr);
        c.configure(settings(55.0));
        let (g, _) = run_sine(&mut c, amp_vu(3.0), 1000.0, 3.0, sr);
        let a = g48[g48.len() - 10..].iter().sum::<f32>() / 10.0;
        let b = g[g.len() - 10..].iter().sum::<f32>() / 10.0;
        assert!(
            (a - b).abs() < 0.2,
            "steady GR at {sr} Hz = {b:.2} vs {a:.2} at 48 kHz"
        );
        // Attack timing agrees too.
        let mut ca = Compressor::new(sr);
        ca.configure(settings(50.0));
        let (gr, up, down) = burst(&mut ca, -24.0, -3.0, 1.0, 1.0, 0.1, sr);
        let t = time_to(&gr, up, gr[up - 1], gr[down - 1], 0.63, sr);
        let (gr0, up0, down0) = burst(
            &mut {
                let mut c = Compressor::new(48_000.0);
                c.configure(settings(50.0));
                c
            },
            -24.0,
            -3.0,
            1.0,
            1.0,
            0.1,
            48_000.0,
        );
        let t0 = time_to(&gr0, up0, gr0[up0 - 1], gr0[down0 - 1], 0.63, 48_000.0);
        assert!(
            (t - t0).abs() < 0.008,
            "attack {:.1} ms at {sr} vs {:.1} ms at 48 kHz",
            t * 1000.0,
            t0 * 1000.0
        );
    }
}

#[test]
fn transfer_curve_is_monotonic_and_matches_the_solver() {
    let mut c = Compressor::new(SR);
    c.configure(settings(60.0));
    let mut curve = vec![0.0f32; 64];
    c.transfer_curve(&mut curve, -60.0, 0.0);
    for w in curve.windows(2) {
        assert!(w[1] >= w[0] - 1e-3, "transfer curve not monotonic");
    }
    // The curve at −18 dBFS equals the static solver with make-up applied.
    let i = 44; // −18 dBFS with 64 points over −60..0 is index 44.8; check both neighbours
    let db_in = -60.0 + 60.0 * i as f32 / 63.0;
    let expect = db_in - c.static_gr_db(10f32.powf(db_in / 20.0) * SQRT_2) + makeup_db(0.32);
    assert!(
        (curve[i] - expect).abs() < 0.05,
        "curve {} vs {}",
        curve[i],
        expect
    );
}

#[test]
fn make_up_is_unity_at_thirty_two_and_forty_db_at_full() {
    assert!(makeup_db(0.32).abs() < 0.05);
    assert!((makeup_db(1.0) - 40.0).abs() < 1e-3);
    let mut c = Compressor::new(SR);
    c.configure(Settings {
        gain: 100.0,
        ..settings(0.0)
    });
    let (_, out) = run_sine(&mut c, 0.001, 1000.0, 1.5, SR);
    let g = goertzel(&out[out.len() - 24_000..], 1000.0, SR) / 0.001;
    assert!(
        (20.0 * g.log10() - 40.0).abs() < 0.5,
        "+40 dB make-up measured {:.2} dB",
        20.0 * g.log10()
    );
}

#[test]
fn the_meter_reads_the_reduction_and_the_output() {
    let mut c = Compressor::new(SR);
    c.configure(settings(60.0));
    let _ = run_sine(&mut c, amp_vu(0.0), 1000.0, 2.0, SR);
    let m = c.meter_frame();
    assert!(
        (m[4] - c.gain_reduction_db(0)).abs() < 0.5,
        "gr_db {} vs {}",
        m[4],
        c.gain_reduction_db(0)
    );
    assert!((m[5] + m[4]).abs() < 1e-3, "GR mode shows −gr");
    // Output modes: a 0 VU output sine reads 0 VU on +4 and −6 on +10.
    let mut c = Compressor::new(SR);
    c.configure(Settings {
        meter: METER_OUT4,
        ..settings(0.0)
    });
    let _ = run_sine(&mut c, amp_vu(0.0), 1000.0, 2.0, SR);
    let m = c.meter_frame();
    assert!(m[5].abs() < 0.5, "+4 reading {}", m[5]);
    let mut c = Compressor::new(SR);
    c.configure(Settings {
        meter: METER_OUT10,
        ..settings(0.0)
    });
    let _ = run_sine(&mut c, amp_vu(0.0), 1000.0, 2.0, SR);
    let m = c.meter_frame();
    assert!((m[5] + 6.0).abs() < 0.5, "+10 reading {}", m[5]);
}

/// Diagnostic: print the static curves and the loop's operating points
/// (`cargo test print_curves -- --ignored --nocapture`).
#[test]
#[ignore]
fn print_curves() {
    for pr in [30.0, 50.0, 100.0] {
        let mut c = Compressor::new(SR);
        c.configure(settings(pr));
        println!("PR {pr}:");
        for db in (-40..=16).step_by(4) {
            let g = c.static_gr_db(amp_vu(db as f32));
            println!(
                "  in {db:>4} dB re 0 VU -> GR {g:6.2} dB  ratio {:5.2}",
                ratio_at(&c, db as f32)
            );
        }
    }
    let mut c = Compressor::new(SR);
    c.configure(Settings {
        gain: 100.0,
        ..settings(0.0)
    });
    let (_, out) = run_sine(&mut c, 0.001, 1000.0, 1.5, SR);
    let g = goertzel(&out[out.len() - 24_000..], 1000.0, SR) / 0.001;
    let tail = &out[out.len() - 24_000..];
    let mean = tail.iter().sum::<f32>() / tail.len() as f32;
    let rms = (tail.iter().map(|v| v * v).sum::<f32>() / tail.len() as f32).sqrt();
    println!(
        "make-up 100: {:.2} dB, peak out {:.4}, mean {:.5}, rms*sqrt2 {:.4}, g(1k) {:.5}, g(2k) {:.5}, g(50) {:.5}",
        20.0 * g.log10(),
        out.iter().fold(0.0f32, |m, v| m.max(v.abs())),
        mean,
        rms * SQRT_2,
        goertzel(tail, 1000.0, SR),
        goertzel(tail, 2000.0, SR),
        goertzel(tail, 50.0, SR)
    );
}

/// Recovery fractions for one cell variant: how much of the settled
/// reduction has come back at each of the given times after the tone
/// stops.
fn recovery_curve(cell: usize, at_s: &[f32]) -> Vec<f32> {
    let mut c = Compressor::new(SR);
    c.configure(Settings {
        peak_reduction: 60.0,
        cell,
        ..Settings::default()
    });
    c.reset();
    let (gr, _) = run_sine(&mut c, amp_vu(4.0), 1000.0, 4.0, SR);
    let settled = gr[gr.len() - 1];
    assert!(settled > 3.0, "cell {cell} only reached {settled:.2} dB");
    let block = 64;
    let (mut l, mut r) = (vec![0.0; block], vec![0.0; block]);
    let mut out = Vec::with_capacity(at_s.len());
    let mut done = 0usize;
    for t in at_s {
        let want = (SR * t) as usize;
        while done < want {
            l.iter_mut().for_each(|v| *v = 0.0);
            r.iter_mut().for_each(|v| *v = 0.0);
            c.process_block(&mut l, &mut r);
            done += block;
        }
        out.push(1.0 - c.meter_frame()[4] / settled);
    }
    out
}

/// The LA-2 position has a **dual time constant**, which is the one
/// difference between the three cells with a physical basis.
///
/// *Figure asserted:* the **shape**. The T4A in the LA-2 and early
/// LA-2A, and very early T4Bs, carried three photocells: the main
/// Clairex CL-505L pair plus a fast CL-705 in parallel with the audio
/// cell, "giving a dual time constant that broadcast engineers liked".
/// Later T4Bs, which is the Silver position and every reissue, dropped
/// it. *Source:* `research/LA-2A.md` section 3.
///
/// So this asserts that the LA-2 recovers a **larger** share early and a
/// **smaller** share later than the reference Gray cell. That pair cannot
/// both be true of any single speed multiplier, which is the point: a
/// scalar can be faster or slower, not both, and a test that only
/// measured total speed would pass with one and miss the whole feature.
///
/// **No magnitude is asserted against a published figure**, because the
/// sources give the cell's existence, direction and topology but not how
/// much conductance it carries; see [`CellParams::fast_share`]. Kantor,
/// who examined the modules, says the slower photocell dominates, so the
/// late-recovery half of this test is also what keeps the fast cell
/// secondary.
#[test]
fn the_la2_cell_has_a_dual_time_constant() {
    // 20 ms is inside the fast photocell's reach; 100 ms is after it has
    // done its work and the slow one has taken over; 3 s is the tail.
    let at = [0.02f32, 0.1, 3.0];
    let silver = recovery_curve(0, &at);
    let gray = recovery_curve(1, &at);
    let la2 = recovery_curve(2, &at);

    // The signature: quicker at first, then slower. No single speed
    // multiplier can be both, which is why this is the test.
    assert!(
        la2[0] > gray[0],
        "at 20 ms the LA-2 cell had recovered {:.3} against Gray's {:.3};          its third photocell should make the first part of the recovery quicker",
        la2[0],
        gray[0]
    );
    assert!(
        la2[1] < gray[1],
        "at 100 ms the LA-2 cell had recovered {:.3} against Gray's {:.3};          once the fast photocell has done its work the slower one must take over",
        la2[1],
        gray[1]
    );
    assert!(
        la2[2] < gray[2],
        "at 3 s the LA-2 cell had recovered {:.3} against Gray's {:.3};          the slower photocell dominates the response, so the tail stays behind",
        la2[2],
        gray[2]
    );

    // Silver and Gray have only the main pair, so between those two a
    // scalar is the whole story and the ordering is simply speed.
    assert!(
        silver[0] > gray[0] && silver[2] > gray[2],
        "Silver should lead Gray at both times: {silver:?} against {gray:?}"
    );

    // And the fast cell stays secondary, as the one source that examined
    // the modules says it is: it cannot carry the early recovery past the
    // fastest cell, which has no third photocell at all.
    assert!(
        la2[0] < silver[0],
        "the LA-2's early recovery ({:.3}) overtook Silver's ({:.3});          the third photocell is meant to be a secondary contribution",
        la2[0],
        silver[0]
    );
}

/// The third photocell touches the LA-2 position and nothing else.
///
/// Silver and Gray are late-1960s T4Bs and reissues, which have only the
/// main pair, so their conductance must still be exactly the free-carrier
/// figure. This is a bit-for-bit guard: adding a parallel population to a
/// shared cell is the kind of change that moves a default sound by
/// accident, and the LA-3A draws on the same cell.
#[test]
fn only_the_la2_cell_gained_the_third_photocell() {
    for cell in 0..3 {
        let p = cell_params_for(cell);
        if cell == 2 {
            assert!(p.fast_share > 0.0, "the LA-2 cell lost its third photocell");
            assert!(p.fast_speed > 1.0, "the third photocell is not faster");
        } else {
            assert_eq!(
                p.fast_share, 0.0,
                "cell {cell} gained a third photocell it never had"
            );
        }
    }
    // Silver and Gray report their free carriers untouched.
    for cell in [0usize, 1] {
        let mut c = Cell::new(cell_params_for(cell), SR);
        for _ in 0..2000 {
            c.step(0.4);
        }
        assert_eq!(
            c.conductance(),
            c.n_f,
            "cell {cell}'s conductance is no longer exactly its free carriers"
        );
        for _ in 0..2000 {
            c.step(0.0);
        }
        assert_eq!(c.conductance(), c.n_f, "cell {cell} drifted on release");
    }
}

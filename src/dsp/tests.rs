//! Tests of the lab itself: the parameter contract, the switch between the
//! models and the unified telemetry. Each model's own behaviour is tested
//! in its module (`fet::tests`, `opto::tests`).

use super::*;
use std::f32::consts::PI;

const SR: f32 = 48_000.0;
const BLOCK: usize = 256;

fn sine(amp: f32, hz: f32, n: usize) -> Vec<f32> {
    (0..n)
        .map(|i| amp * (2.0 * PI * hz * i as f32 / SR).sin())
        .collect()
}

/// Run `blocks` blocks of a sine through `p`; returns the output.
fn run(p: &mut Processor, amp: f32, blocks: usize) -> Vec<f32> {
    let x = sine(amp, 1000.0, blocks * BLOCK);
    let mut l = x.clone();
    let mut r = x;
    for b in 0..blocks {
        let s = b * BLOCK;
        p.process(&mut l[s..s + BLOCK], &mut r[s..s + BLOCK]);
    }
    l
}

fn settings(model: Model) -> Settings {
    Settings {
        model,
        ..Settings::default()
    }
}

#[test]
fn the_parameter_contract_holds() {
    let specs = param_specs(true);
    let ids: Vec<&str> = specs.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "model",
            "fet_input",
            "fet_output",
            "fet_attack",
            "fet_release",
            "fet_ratio",
            "fet_meter",
            "fet_revision",
            "opto_gain",
            "opto_peak_reduction",
            "opto_mode",
            "opto_meter",
            "opto_emphasis",
            "opto_cell",
            "link",
            "mix",
            "sc_hpf",
            "bypass",
            "src_kind",
            "src_level",
            "src_freq",
        ]
    );
    let by_id = |id: &str| specs.iter().find(|s| s.id == id).unwrap();
    assert_eq!(
        by_id("model").labels,
        vec!["1176".to_string(), "LA-2A".to_string()]
    );
    assert_eq!(by_id("fet_revision").default, 8.0);
    assert_eq!(by_id("opto_cell").default, 1.0);
    assert_eq!(by_id("src_kind").labels.len(), 7);
    assert_eq!(by_id("src_level").default, 0.4);
    assert_eq!(param_specs(false).len(), 18);

    let (bridge, ix) = build_bridge("test", SR);
    assert_eq!(ix.model, 0);
    assert_eq!(ix.src_freq, Some(20));
    let streams = streams(SR);
    assert_eq!(streams[STREAM_IX.meter].id, "meter");
    assert_eq!(streams[STREAM_IX.cell].id, "cell");
    assert_eq!(streams[STREAM_IX.transfer].id, "transfer");
    assert_eq!(streams[STREAM_IX.transfer].capacity, TRANSFER_POINTS);
    drop(bridge);
}

#[test]
fn shared_values_reach_both_engines() {
    let s = Settings::default().with_shared(Shared {
        link: false,
        mix: 0.25,
        sc_hpf_hz: 120.0,
        bypass: true,
    });
    assert!(!s.fet.link && !s.opto.link);
    assert_eq!(s.fet.mix, 0.25);
    assert_eq!(s.opto.mix, 0.25);
    assert_eq!(s.fet.sc_hpf_hz, 120.0);
    assert_eq!(s.opto.sc_hpf, 120.0);
    assert!(s.fet.bypass && s.opto.bypass);
    assert_eq!(s.shared().mix, 0.25);
}

#[test]
fn each_model_compresses_and_reports_reduction_below_zero() {
    for model in [Model::Fet, Model::Opto] {
        let mut p = Processor::new(SR);
        assert!(p.configure(&settings(model)));
        let out = run(&mut p, 0.5, 120);
        let tail = &out[out.len() - 8 * BLOCK..];
        let peak = tail.iter().fold(0.0f32, |m, v| m.max(v.abs()));
        assert!(peak.is_finite() && peak > 0.01, "{model:?}: peak {peak}");
        let f = p.meter_frame();
        assert!(f[0] > 0.49 && f[0] < 0.51, "{model:?}: in peak {}", f[0]);
        assert!(
            f[4] < -0.5,
            "{model:?}: gr_db {} should be well below 0",
            f[4]
        );
        assert!(f[5] <= 0.0, "{model:?}: GR-mode meter reads {}", f[5]);
        assert!(
            (f[5] - f[4]).abs() < 1e-3,
            "{model:?}: GR meter equals gr_db"
        );
        assert_eq!(p.latency() > 0, model == Model::Fet);
    }
}

#[test]
fn output_meter_modes_read_the_output_against_the_vu_reference() {
    // −18 dBFS RMS sine = 0 VU on both meters.
    let amp = 10f32.powf(VU_REF_DBFS / 20.0) * std::f32::consts::SQRT_2;
    let mut fet = settings(Model::Fet);
    fet.fet.meter = fet::MeterMode::Plus4;
    fet = fet.with_shared(Shared {
        bypass: true,
        ..Shared::default()
    });
    let mut p = Processor::new(SR);
    p.configure(&fet);
    run(&mut p, amp, 60);
    assert!(p.meter_vu().abs() < 1.0, "1176 +4: {}", p.meter_vu());

    let mut opto = settings(Model::Opto);
    opto.opto.meter = opto::METER_OUT4;
    opto = opto.with_shared(Shared {
        bypass: true,
        ..Shared::default()
    });
    let mut p = Processor::new(SR);
    p.configure(&opto);
    run(&mut p, amp, 60);
    assert!(p.meter_vu().abs() < 1.0, "LA-2A +4: {}", p.meter_vu());
}

#[test]
fn switching_models_crossfades_without_a_click() {
    let mut p = Processor::new(SR);
    p.configure(&settings(Model::Fet));
    run(&mut p, 0.3, 100);
    // Switch to the LA-2A and record the next blocks.
    assert!(p.configure(&settings(Model::Opto)));
    let x = sine(0.3, 1000.0, 20 * BLOCK);
    let mut l = x.clone();
    let mut r = x;
    for b in 0..20 {
        let s = b * BLOCK;
        p.process(&mut l[s..s + BLOCK], &mut r[s..s + BLOCK]);
    }
    // No sample-to-sample jump larger than the sine's own slope allows
    // (0.3 peak at 1 kHz moves at most ~0.04 per sample; leave headroom
    // for the two engines' different gains during the fade).
    let max_step = l
        .windows(2)
        .map(|w| (w[1] - w[0]).abs())
        .fold(0.0f32, f32::max);
    assert!(max_step < 0.12, "largest step {max_step}");
    assert_eq!(p.model(), Model::Opto);
    assert_eq!(p.latency(), 0);
    // Steady state afterwards is the LA-2A alone.
    let mut alone = Processor::new(SR);
    alone.configure(&settings(Model::Opto));
    let a = run(&mut alone, 0.3, 120);
    let b = run(&mut p, 0.3, 100);
    let pa = a[a.len() - BLOCK..]
        .iter()
        .fold(0.0f32, |m, v| m.max(v.abs()));
    let pb = b[b.len() - BLOCK..]
        .iter()
        .fold(0.0f32, |m, v| m.max(v.abs()));
    assert!((pa - pb).abs() < 0.02, "steady peaks {pa} vs {pb}");
}

#[test]
fn switching_back_and_forth_stays_finite() {
    let mut p = Processor::new(SR);
    let mut model = Model::Fet;
    for _ in 0..40 {
        model = if model == Model::Fet {
            Model::Opto
        } else {
            Model::Fet
        };
        p.configure(&settings(model));
        let out = run(&mut p, 0.8, 2);
        assert!(out.iter().all(|v| v.is_finite() && v.abs() < 4.0));
    }
}

#[test]
fn transfer_curve_follows_the_active_model() {
    let mut p = Processor::new(SR);
    p.configure(&settings(Model::Fet));
    let mut fet_curve = [0.0f32; TRANSFER_POINTS];
    p.transfer(&mut fet_curve);
    p.configure(&settings(Model::Opto));
    let mut opto_curve = [0.0f32; TRANSFER_POINTS];
    p.transfer(&mut opto_curve);
    // Both are monotonic and finite, and they differ (different make-up
    // and thresholds).
    for c in [&fet_curve, &opto_curve] {
        assert!(c.iter().all(|v| v.is_finite()));
        assert!(c.windows(2).all(|w| w[1] >= w[0] - 0.05), "monotonic");
    }
    let diff: f32 = fet_curve
        .iter()
        .zip(opto_curve.iter())
        .map(|(a, b)| (a - b).abs())
        .sum::<f32>()
        / TRANSFER_POINTS as f32;
    assert!(diff > 0.5, "curves differ by {diff} dB on average");
    assert_eq!(p.cell_state().len(), 3);
    p.configure(&settings(Model::Fet));
    assert_eq!(p.cell_state(), [0.0; 3]);
}

#[test]
fn the_source_plays_every_kind() {
    let mut src = Source::new(1);
    for kind in 0..SOURCE_NAMES.len() {
        let mut peak = 0.0f32;
        for _ in 0..(SR as usize) {
            let v = src.next(kind, 110.0, SR);
            assert!(
                v.is_finite() && v.abs() <= 2.5,
                "{}: {v}",
                SOURCE_NAMES[kind]
            );
            peak = peak.max(v.abs());
        }
        assert!(peak > 0.05, "{} is silent", SOURCE_NAMES[kind]);
    }
}

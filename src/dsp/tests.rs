//! Tests of the lab itself: the parameter contract, the switch between the
//! models and the unified telemetry. Each model's own behaviour is tested
//! in its module (`fet::tests`, `opto::tests`, `opto3::tests`, `vca::tests`,
//! `pre::tests`).

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
            "opto_meter_zero",
            "la3a_gain",
            "la3a_peak_reduction",
            "la3a_mode",
            "la3a_meter",
            "la3a_emphasis",
            "la3a_cell",
            "dist_input",
            "dist_output",
            "dist_attack",
            "dist_release",
            "dist_ratio",
            "dist_detector",
            "dist_audio",
            "dist_british",
            "dist_link_mode",
            "dist_headroom",
            "pre_join",
            "pre_gain",
            "pre_input",
            "pre_pad",
            "pre_polarity",
            "pre_level",
            "pre_lf_freq",
            "pre_lf_gain",
            "pre_hf_freq",
            "pre_hf_gain",
            "pre_hpf",
            "pre_voice",
            "pre_load",
            "pre_meter",
            "pre_phantom",
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
        MODEL_NAMES
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
    );
    assert_eq!(by_id("fet_revision").default, 8.0);
    assert_eq!(by_id("opto_cell").default, 1.0);
    assert_eq!(by_id("src_kind").labels.len(), 7);
    assert_eq!(by_id("src_level").default, 0.4);
    assert_eq!(param_specs(false).len(), 50);

    let (bridge, ix) = build_bridge("test", SR);
    assert_eq!(ix.model, 0);
    assert_eq!(ix.src_freq, Some(52));
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
        // `meter_vu` is now where the needle *is*, not what it is chasing,
        // so after a settled run it should have arrived at `gr_db`.
        assert!(
            (f[5] - f[4]).abs() < 0.3,
            "{model:?}: the needle should have reached gr_db, {} against {}",
            f[5],
            f[4]
        );
        assert_eq!(
            p.latency() > 0,
            matches!(model, Model::Fet | Model::Vca),
            "{model:?}: only the oversampling models report latency"
        );
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

#[test]
fn every_model_has_a_label_and_survives_a_block() {
    assert_eq!(MODEL_NAMES.len(), Model::ALL.len());
    for (i, m) in Model::ALL.iter().enumerate() {
        assert_eq!(Model::from_index(i), *m);
        assert_eq!(m.index(), i);
        assert!(!m.label().is_empty());
        let mut p = Processor::new(SR);
        p.configure(&settings(*m));
        let out = run(&mut p, 0.3, 20);
        assert!(
            out.iter().all(|v| v.is_finite()),
            "{} produced something that is not a number",
            m.label()
        );
    }
}

#[test]
fn every_model_compresses_and_reports_reduction_below_zero() {
    for m in Model::ALL {
        let mut p = Processor::new(SR);
        let mut s = settings(m);
        // Drive each model well past its threshold.
        s.opto.peak_reduction = 70.0;
        s.opto3.peak_reduction = 70.0;
        s.vca.input = 8.0;
        s.fet.input = 40.0;
        p.configure(&s);
        run(&mut p, 0.5, 200);
        assert!(
            p.gr_db() <= 0.0,
            "{}: the lab publishes gain change, never gain: {:.2}",
            m.label(),
            p.gr_db()
        );
        if m != Model::Pre6176 {
            assert!(
                p.gr_db() < -0.5,
                "{} should compress a hot signal, got {:.2}",
                m.label(),
                p.gr_db()
            );
        }
    }
}

#[test]
fn the_lamps_stream_only_speaks_for_the_models_that_have_lamps() {
    for m in Model::ALL {
        let mut p = Processor::new(SR);
        let mut s = settings(m);
        s.vca.audio = vca::AudioMode::Dist3;
        s.vca.input = 9.0;
        p.configure(&s);
        run(&mut p, 0.5, 40);
        let f = p.lamps_frame();
        assert_eq!(f.len(), LAMPS_LEN);
        assert!(f.iter().all(|v| v.is_finite()));
        match m {
            Model::Vca => assert!(f[0] > 0.0, "the Distressor should report distortion"),
            Model::Pre6176 => assert!(f[2] > -80.0, "the 6176 should report its PRE meter"),
            _ => assert_eq!(f, [0.0; LAMPS_LEN], "{} has no lamps", m.label()),
        }
    }
}

#[test]
fn the_cell_stream_speaks_for_both_optical_models() {
    for m in [Model::Opto, Model::Opto3] {
        let mut p = Processor::new(SR);
        let mut s = settings(m);
        s.opto.peak_reduction = 80.0;
        s.opto3.peak_reduction = 80.0;
        p.configure(&s);
        run(&mut p, 0.5, 200);
        let c = p.cell_state();
        assert!(c.iter().all(|v| v.is_finite()));
        assert!(c[1] > 0.0, "{}: the cell should be conducting", m.label());
    }
    let mut p = Processor::new(SR);
    p.configure(&settings(Model::Vca));
    run(&mut p, 0.5, 10);
    assert_eq!(p.cell_state(), [0.0; 3], "the Distressor has no cell");
}

#[test]
fn the_six_one_seven_six_routing_switch_does_what_it_says() {
    let mut base = settings(Model::Pre6176);
    base.pre.level = 5.0;
    base.fet.input = 44.0;
    let gr_for = |routing: pre::Routing| -> f32 {
        let mut p = Processor::new(SR);
        let mut s = base;
        s.pre.routing = routing;
        p.configure(&s);
        run(&mut p, 0.4, 200);
        p.gr_db()
    };
    assert!(
        gr_for(pre::Routing::Join) < -0.5,
        "Join must let the compressor work"
    );
    assert!(
        gr_for(pre::Routing::Unity).abs() < 0.05,
        "1:1 keeps the colour but takes the gain reduction away"
    );
    assert!(
        gr_for(pre::Routing::Bypass).abs() < 0.05,
        "BP takes the compressor out"
    );
    // The latency does not change with the switch, so a host never has to
    // re-report it.
    let latency = |routing: pre::Routing| -> usize {
        let mut p = Processor::new(SR);
        let mut s = base;
        s.pre.routing = routing;
        p.configure(&s);
        p.latency()
    };
    assert_eq!(latency(pre::Routing::Join), latency(pre::Routing::Bypass));
    assert_eq!(latency(pre::Routing::Join), latency(pre::Routing::Unity));
}

#[test]
fn the_six_one_seven_six_meter_switch_picks_its_source() {
    let read = |meter: usize| -> f32 {
        let mut p = Processor::new(SR);
        let mut s = settings(Model::Pre6176);
        s.pre.meter = meter;
        s.fet.input = 44.0;
        p.configure(&s);
        run(&mut p, 0.4, 200);
        p.meter_vu()
    };
    let gr = read(pre::METER_GR);
    assert!(gr <= 0.0, "the GR position shows the gain change: {gr:.2}");
    assert!(read(pre::METER_PRE).is_finite());
    assert!(read(pre::METER_COMP).is_finite());
}

#[test]
fn switching_between_all_five_stays_finite_and_quiet() {
    let mut p = Processor::new(SR);
    let mut last = 0.0f32;
    for round in 0..3 {
        for m in Model::ALL {
            p.configure(&settings(m));
            let out = run(&mut p, 0.3, 6);
            assert!(
                out.iter().all(|v| v.is_finite()),
                "round {round}, {}",
                m.label()
            );
            let jump = out
                .windows(2)
                .map(|w| (w[1] - w[0]).abs())
                .fold(0.0f32, f32::max);
            assert!(jump < 1.0, "a switch to {} clicked: {jump:.3}", m.label());
            last = out[out.len() - 1];
        }
    }
    assert!(last.is_finite());
}

#[test]
fn the_transfer_curve_follows_every_model() {
    for m in Model::ALL {
        let mut p = Processor::new(SR);
        p.configure(&settings(m));
        let mut out = [0.0f32; TRANSFER_POINTS];
        p.transfer(&mut out);
        assert!(
            out.iter().all(|v| v.is_finite()),
            "{} produced a curve with holes in it",
            m.label()
        );
        for w in out.windows(2) {
            assert!(w[1] >= w[0] - 0.5, "{}: the curve must not fall", m.label());
        }
    }
}

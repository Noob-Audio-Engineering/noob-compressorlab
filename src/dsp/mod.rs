//! The DSP of Noob CompressorLab, and the bridge description shared by the
//! plug-in and the standalone.
//!
//! One instance is one compressor at a time: the `model` parameter picks
//! the FET model ([`fet`], the 1176) or the optical model ([`opto`], the
//! LA-2A). Both engines live in the [`Processor`]; only the active one runs,
//! the other is silent until it is switched in, at which point it starts
//! from rest and takes over through a short crossfade so the switch does not
//! click. Every knob of both models is a parameter of the instance, so a
//! project saves the whole lab, not just the model in use.
//!
//! ## Layout
//!
//! | module | contents |
//! |---|---|
//! | [`fet`] | the 1176: oversampled feedback FET compressor, its revisions, knobs and tests |
//! | [`opto`] | the LA-2A: the T4 cell model, sidechain and tube stage, its knobs and tests |
//! | [`source`] | the standalone's demo signals |
//! | this file | [`Model`], [`Settings`], parameter ids and specs, streams, the bridge builder, the [`Processor`] |
//!
//! ## Parameters
//!
//! [`param_specs`] describes every parameter once; the standalone builds
//! its bridge from it directly and the plug-in's nih-plug parameters use
//! the same ids, so the same page drives both. Ids are stable API.
//!
//! | id | range / labels | default | group |
//! |---|---|---|---|
//! | `model` | 1176, LA-2A | 1176 | lab |
//! | `fet_input`, `fet_output` | 0..48 mark (= −48..0 dB) | 24 | 1176 |
//! | `fet_attack` | 0 (OFF)..7 | 4 | 1176 |
//! | `fet_release` | 1..7 | 4 | 1176 |
//! | `fet_ratio` | 4, 8, 12, 20, All | 4 | 1176 |
//! | `fet_meter` | GR, +4, +8, Off | GR | 1176 |
//! | `fet_revision` | A, B, C, D, E, F, G, H, LN | LN | 1176 |
//! | `opto_gain` | 0..100 | 32 | LA-2A |
//! | `opto_peak_reduction` | 0..100 | 40 | LA-2A |
//! | `opto_mode` | Compress, Limit | Compress | LA-2A |
//! | `opto_meter` | Gain Reduction, Output +10, Output +4 | Gain Reduction | LA-2A |
//! | `opto_emphasis` | 0..1 | 1 | LA-2A |
//! | `opto_cell` | Silver, Gray, LA-2 | Gray | LA-2A |
//! | `link` | toggle | on | extras |
//! | `mix` | 0..100 % | 100 | extras |
//! | `sc_hpf` | 0 (off)..300 Hz | 0 | extras |
//! | `bypass` | toggle | off | extras |
//! | `src_kind`, `src_level`, `src_freq` | standalone only | | source |
//!
//! ## Streams
//!
//! | id | kind | values | rate | contents |
//! |---|---|---|---|---|
//! | `meter` | meter | 6 | every block | `[in_l, in_r, out_l, out_r, gr_db, meter_vu]`: linear peaks (1.0 = 0 dBFS), the gain change in dB (≤ 0 for both models) and what the active model's panel meter reads in dB (its GR modes: `gr_db`; its output modes: the block's VU reading against −18 dBFS = 0 VU, the 1176's +8 mode 4 dB lower; the 1176's Off: −60) |
//! | `cell` | raw | 3 | every block while the LA-2A is active | `[light, free_carriers, trapped_carriers]`, 0..1; zeros once when the 1176 takes over |
//! | `transfer` | curve, sticky | 128 | on change | the active model's static output level in dBFS for a sine at −60..0 dBFS input |
//!
//! ## Real-time rules
//!
//! Everything reachable from [`Processor::process`] runs without
//! allocation, locks or I/O. Parameters are read from atomics into a
//! [`Settings`] snapshot once per block; the engines smooth the continuous
//! ones themselves.

pub mod fet;
pub mod opto;
pub mod source;

use noob_vst_webgui_framework::{
    AudioHandle, NoobVstWebguiFramework, ParamSpec, StreamKind, StreamSpec,
};
use serde_json::json;

pub use source::{SOURCE_NAMES, Source};

/// Labels of `model`, in parameter order.
pub const MODEL_NAMES: [&str; 2] = ["1176", "LA-2A"];
/// Points in the `transfer` stream (both engines draw the same grid).
pub const TRANSFER_POINTS: usize = fet::TRANSFER_POINTS;
/// Input range of the transfer curve, dBFS.
pub const TRANSFER_MIN_DB: f32 = -60.0;
pub const TRANSFER_MAX_DB: f32 = 0.0;
/// 0 VU of both panel meters, in dBFS.
pub const VU_REF_DBFS: f32 = opto::VU_REF_DBFS;
/// Upper end of the shared side-chain high-pass; 0 turns it off.
pub const SC_HPF_MAX_HZ: f32 = fet::SC_HPF_MAX_HZ;
/// Layout of one `meter` frame.
pub const METER_LEN: usize = 6;
/// Length of the crossfade when the model changes.
pub const XFADE_SECONDS: f32 = 0.02;
/// Longest block the crossfade scratch buffers cover; longer blocks fade
/// their first `MAX_BLOCK` samples and pass the rest from the new engine.
pub const MAX_BLOCK: usize = 8192;

/// Which compressor the instance is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Model {
    /// The 1176 ([`fet`]).
    #[default]
    Fet,
    /// The LA-2A ([`opto`]).
    Opto,
}

impl Model {
    /// From the parameter value / label index (clamped).
    pub fn from_index(i: usize) -> Self {
        if i == 0 { Model::Fet } else { Model::Opto }
    }

    /// The parameter value.
    pub fn index(self) -> usize {
        self as usize
    }

    /// The printed label ([`MODEL_NAMES`]).
    pub fn label(self) -> &'static str {
        MODEL_NAMES[self.index()]
    }
}

/// The values both models share: applied to whichever engine is active.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shared {
    /// One detector / cell for both channels.
    pub link: bool,
    /// Wet share, 0..1.
    pub mix: f32,
    /// Side-chain high-pass corner in Hz, 0 = off.
    pub sc_hpf_hz: f32,
    pub bypass: bool,
}

impl Default for Shared {
    fn default() -> Self {
        Shared {
            link: true,
            mix: 1.0,
            sc_hpf_hz: 0.0,
            bypass: false,
        }
    }
}

/// Everything the processor needs from the parameters, read once per
/// block. The shared values ([`Shared`]) are stamped into both engine
/// settings by [`Settings::with_shared`], so `fet.link == opto.link` and so
/// on by construction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub model: Model,
    pub fet: fet::Settings,
    pub opto: opto::Settings,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            model: Model::Fet,
            fet: fet::Settings::default(),
            opto: opto::Settings::default(),
        }
        .with_shared(Shared::default())
    }
}

impl Settings {
    /// Copy the shared values into both engine settings.
    pub fn with_shared(mut self, s: Shared) -> Self {
        self.fet.link = s.link;
        self.fet.mix = s.mix;
        self.fet.sc_hpf_hz = s.sc_hpf_hz;
        self.fet.bypass = s.bypass;
        self.opto.link = s.link;
        self.opto.mix = s.mix;
        self.opto.sc_hpf = s.sc_hpf_hz;
        self.opto.bypass = s.bypass;
        self
    }

    /// The shared values (as stored in the FET settings).
    pub fn shared(&self) -> Shared {
        Shared {
            link: self.fet.link,
            mix: self.fet.mix,
            sc_hpf_hz: self.fet.sc_hpf_hz,
            bypass: self.fet.bypass,
        }
    }
}

/// Parameter indices resolved once by id, so the audio thread never looks
/// anything up by string.
#[derive(Clone, Debug)]
pub struct ParamIx {
    pub model: usize,
    pub fet_input: usize,
    pub fet_output: usize,
    pub fet_attack: usize,
    pub fet_release: usize,
    pub fet_ratio: usize,
    pub fet_meter: usize,
    pub fet_revision: usize,
    pub opto_gain: usize,
    pub opto_peak_reduction: usize,
    pub opto_mode: usize,
    pub opto_meter: usize,
    pub opto_emphasis: usize,
    pub opto_cell: usize,
    pub link: usize,
    pub mix: usize,
    pub sc_hpf: usize,
    pub bypass: usize,
    /// Standalone only (`None` in the plug-in).
    pub src_kind: Option<usize>,
    pub src_freq: Option<usize>,
    pub src_level: Option<usize>,
}

/// Stream indices, in the order [`streams`] declares them.
#[derive(Clone, Copy, Debug)]
pub struct StreamIx {
    pub meter: usize,
    pub cell: usize,
    pub transfer: usize,
}

/// The fixed stream layout.
pub const STREAM_IX: StreamIx = StreamIx {
    meter: 0,
    cell: 1,
    transfer: 2,
};

/// The streams (see the module docs for the layouts).
pub fn streams(sr: f32) -> Vec<StreamSpec> {
    vec![
        StreamSpec::new("meter", METER_LEN)
            .name("Meter")
            .kind(StreamKind::Meter)
            .channels(2)
            .meta(json!({ "layout": "in_l,in_r,out_l,out_r,gr_db,meter_vu", "vu_ref_dbfs": VU_REF_DBFS, "sample_rate": sr })),
        StreamSpec::new("cell", 3)
            .name("T4 cell")
            .kind(StreamKind::Raw)
            .meta(json!({ "layout": "light,free_carriers,trapped_carriers" })),
        StreamSpec::new("transfer", TRANSFER_POINTS)
            .name("Transfer curve")
            .kind(StreamKind::Curve)
            .sticky()
            .meta(json!({ "in_db": [TRANSFER_MIN_DB, TRANSFER_MAX_DB], "unit": "dBFS" })),
    ]
}

/// Every parameter (see the module docs). `with_source` adds the
/// standalone's demo-source parameters (not automatable).
pub fn param_specs(with_source: bool) -> Vec<ParamSpec> {
    let mut v = vec![
        ParamSpec::new("model", "Model")
            .labels(MODEL_NAMES)
            .default(0.0)
            .not_automatable()
            .group("lab"),
        ParamSpec::new("fet_input", "Input")
            .range(0.0, fet::MARK_MAX)
            .default(24.0)
            .group("1176"),
        ParamSpec::new("fet_output", "Output")
            .range(0.0, fet::MARK_MAX)
            .default(24.0)
            .group("1176"),
        ParamSpec::new("fet_attack", "Attack")
            .range(0.0, fet::ATTACK_MAX)
            .default(4.0)
            .group("1176"),
        ParamSpec::new("fet_release", "Release")
            .range(1.0, fet::RELEASE_MAX)
            .default(4.0)
            .group("1176"),
        ParamSpec::new("fet_ratio", "Ratio")
            .labels(fet::RATIO_NAMES)
            .default(0.0)
            .group("1176"),
        ParamSpec::new("fet_meter", "Meter")
            .labels(fet::METER_NAMES)
            .default(0.0)
            .not_automatable()
            .group("1176"),
        ParamSpec::new("fet_revision", "Revision")
            .labels(fet::REVISION_NAMES)
            .default(8.0)
            .not_automatable()
            .group("1176"),
        ParamSpec::new("opto_gain", "Gain")
            .range(0.0, 100.0)
            .default(32.0)
            .group("LA-2A"),
        ParamSpec::new("opto_peak_reduction", "Peak Reduction")
            .range(0.0, 100.0)
            .default(40.0)
            .group("LA-2A"),
        ParamSpec::new("opto_mode", "Mode")
            .labels(opto::MODE_NAMES)
            .default(0.0)
            .group("LA-2A"),
        ParamSpec::new("opto_meter", "Meter")
            .labels(opto::METER_NAMES)
            .default(0.0)
            .not_automatable()
            .group("LA-2A"),
        ParamSpec::new("opto_emphasis", "Emphasis (R37)")
            .range(0.0, 1.0)
            .default(1.0)
            .group("LA-2A"),
        ParamSpec::new("opto_cell", "Cell")
            .labels(opto::CELL_NAMES)
            .default(1.0)
            .not_automatable()
            .group("LA-2A"),
        ParamSpec::new("link", "Stereo Link")
            .toggle()
            .default(1.0)
            .group("extras"),
        ParamSpec::new("mix", "Mix")
            .range(0.0, 100.0)
            .default(100.0)
            .unit("%")
            .group("extras"),
        ParamSpec::new("sc_hpf", "Side-chain HPF")
            .range(0.0, SC_HPF_MAX_HZ)
            .default(0.0)
            .unit("Hz")
            .group("extras"),
        ParamSpec::new("bypass", "Bypass").toggle().group("extras"),
    ];
    if with_source {
        v.push(
            ParamSpec::new("src_kind", "Source")
                .labels(SOURCE_NAMES)
                .default(0.0)
                .not_automatable()
                .group("source"),
        );
        v.push(
            ParamSpec::new("src_level", "Source Level")
                .range(0.0, 1.0)
                .default(0.4)
                .not_automatable()
                .group("source"),
        );
        v.push(
            ParamSpec::new("src_freq", "Source Frequency")
                .range(20.0, 20_000.0)
                .log()
                .default(110.0)
                .unit("Hz")
                .not_automatable()
                .group("source"),
        );
    }
    v
}

/// The standalone's bridge: the parameters (with the demo sources), the
/// streams and the manifest metadata.
pub fn build_bridge(name: &str, sr: f32) -> (NoobVstWebguiFramework, ParamIx) {
    let mut b = NoobVstWebguiFramework::builder(name)
        .meta(json!({
            "vendor": "Ely Erin Fox",
            "version": env!("CARGO_PKG_VERSION"),
            "sample_rate": sr,
            "vu_ref_dbfs": VU_REF_DBFS,
            "standalone": true,
            "transfer_points": TRANSFER_POINTS,
        }))
        .params(param_specs(true));
    for s in streams(sr) {
        b = b.stream(s);
    }
    let s = b.build();
    let ix = param_index(&s);
    (s, ix)
}

/// Resolve the parameter indices by id (works for the plug-in's mirror,
/// which has no source parameters, as well as the standalone).
pub fn param_index(s: &NoobVstWebguiFramework) -> ParamIx {
    let ix = |id: &str| s.index_of(id).expect(id);
    ParamIx {
        model: ix("model"),
        fet_input: ix("fet_input"),
        fet_output: ix("fet_output"),
        fet_attack: ix("fet_attack"),
        fet_release: ix("fet_release"),
        fet_ratio: ix("fet_ratio"),
        fet_meter: ix("fet_meter"),
        fet_revision: ix("fet_revision"),
        opto_gain: ix("opto_gain"),
        opto_peak_reduction: ix("opto_peak_reduction"),
        opto_mode: ix("opto_mode"),
        opto_meter: ix("opto_meter"),
        opto_emphasis: ix("opto_emphasis"),
        opto_cell: ix("opto_cell"),
        link: ix("link"),
        mix: ix("mix"),
        sc_hpf: ix("sc_hpf"),
        bypass: ix("bypass"),
        src_kind: s.index_of("src_kind"),
        src_freq: s.index_of("src_freq"),
        src_level: s.index_of("src_level"),
    }
}

/// Read the settings from the bridge on the audio thread (atomic loads).
#[inline]
pub fn read_settings(audio: &AudioHandle, ix: &ParamIx) -> Settings {
    let shared = Shared {
        link: audio.param(ix.link) >= 0.5,
        mix: (audio.param(ix.mix) / 100.0).clamp(0.0, 1.0),
        sc_hpf_hz: audio.param(ix.sc_hpf),
        bypass: audio.param(ix.bypass) >= 0.5,
    };
    Settings {
        model: Model::from_index(audio.param(ix.model).round() as usize),
        fet: fet::Settings {
            input: audio.param(ix.fet_input),
            output: audio.param(ix.fet_output),
            attack: audio.param(ix.fet_attack),
            release: audio.param(ix.fet_release),
            ratio: fet::Ratio::from_index(audio.param(ix.fet_ratio).round() as usize),
            meter: fet::MeterMode::from_index(audio.param(ix.fet_meter).round() as usize),
            revision: fet::Revision::from_index(audio.param(ix.fet_revision).round() as usize),
            ..fet::Settings::default()
        },
        opto: opto::Settings {
            gain: audio.param(ix.opto_gain),
            peak_reduction: audio.param(ix.opto_peak_reduction),
            limit: audio.param(ix.opto_mode) >= 0.5,
            meter: audio.param(ix.opto_meter).round().clamp(0.0, 2.0) as usize,
            emphasis: audio.param(ix.opto_emphasis),
            cell: audio.param(ix.opto_cell).round().clamp(0.0, 2.0) as usize,
            ..opto::Settings::default()
        },
    }
    .with_shared(shared)
}

/// Both engines and the switch between them, plus the block-rate
/// telemetry. The plug-in and the standalone drive it the same way:
/// [`configure`](Self::configure) with a fresh [`Settings`] snapshot,
/// [`process`](Self::process) the block, [`publish`](Self::publish) the
/// streams.
pub struct Processor {
    settings: Settings,
    first: bool,
    sr: f32,
    fet: fet::Compressor,
    opto: opto::Compressor,
    /// The engine fading out (only meaningful while `xfade > 0`).
    outgoing: Model,
    /// Samples of crossfade left.
    xfade: usize,
    xfade_len: usize,
    scratch_l: Vec<f32>,
    scratch_r: Vec<f32>,
    in_peak: [f32; 2],
    out_peak: [f32; 2],
    gr_db: f32,
    meter_vu: f32,
    transfer: [f32; TRANSFER_POINTS],
    curve_due: bool,
    cell_zeroed: bool,
    blocks: u64,
}

impl Processor {
    pub fn new(sr: f32) -> Self {
        Processor {
            settings: Settings::default(),
            first: true,
            sr,
            fet: fet::Compressor::new(sr),
            opto: opto::Compressor::new(sr),
            outgoing: Model::Fet,
            xfade: 0,
            xfade_len: (XFADE_SECONDS * sr).round() as usize,
            scratch_l: vec![0.0; MAX_BLOCK],
            scratch_r: vec![0.0; MAX_BLOCK],
            in_peak: [0.0; 2],
            out_peak: [0.0; 2],
            gr_db: 0.0,
            meter_vu: 0.0,
            transfer: [0.0; TRANSFER_POINTS],
            curve_due: true,
            cell_zeroed: false,
            blocks: 0,
        }
    }

    /// Retune both engines to `sr` and start from rest.
    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sr = sr;
        self.xfade_len = (XFADE_SECONDS * sr).round() as usize;
        self.fet.set_sample_rate(sr);
        self.opto.set_sample_rate(sr);
        self.first = true;
        self.reset();
    }

    /// Clear every state (both engines, the crossfade, the meters).
    pub fn reset(&mut self) {
        self.fet.reset();
        self.opto.reset();
        self.xfade = 0;
        self.in_peak = [0.0; 2];
        self.out_peak = [0.0; 2];
        self.gr_db = 0.0;
        self.meter_vu = 0.0;
        self.curve_due = true;
    }

    /// The active model.
    pub fn model(&self) -> Model {
        self.settings.model
    }

    /// The settings in force.
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Latency of the active model in samples (the 1176's oversampler; the
    /// LA-2A has none).
    pub fn latency(&self) -> usize {
        match self.settings.model {
            Model::Fet => self.fet.latency(),
            Model::Opto => 0,
        }
    }

    /// Apply a settings snapshot. Returns `true` when anything changed (the
    /// transfer curve is then republished). A model change resets the engine
    /// that becomes active and starts the crossfade from the outgoing one.
    pub fn configure(&mut self, s: &Settings) -> bool {
        let mut changed = self.first;
        if s.model != self.settings.model {
            if !self.first {
                self.outgoing = self.settings.model;
                self.xfade = self.xfade_len;
            }
            match s.model {
                Model::Fet => self.fet.reset(),
                Model::Opto => self.opto.reset(),
            }
            changed = true;
        }
        changed |= self.fet.configure(&s.fet);
        changed |= self.opto.configure(s.opto);
        self.settings = *s;
        self.first = false;
        if changed {
            self.curve_due = true;
        }
        changed
    }

    #[inline]
    fn run(
        model: Model,
        fet: &mut fet::Compressor,
        opto: &mut opto::Compressor,
        l: &mut [f32],
        r: &mut [f32],
    ) {
        match model {
            Model::Fet => fet.process(l, r),
            Model::Opto => opto.process_block(l, r),
        }
    }

    /// Process one stereo block in place through the active model (and the
    /// outgoing one while a crossfade is running), then refresh the meter
    /// values. Real-time safe.
    pub fn process(&mut self, l: &mut [f32], r: &mut [f32]) {
        let n = l.len().min(r.len());
        let (l, r) = (&mut l[..n], &mut r[..n]);
        let mut pin = [0.0f32; 2];
        for i in 0..n {
            pin[0] = pin[0].max(l[i].abs());
            pin[1] = pin[1].max(r[i].abs());
        }
        self.in_peak = pin;

        let model = self.settings.model;
        if self.xfade > 0 && self.outgoing != model {
            let m = n.min(MAX_BLOCK);
            self.scratch_l[..m].copy_from_slice(&l[..m]);
            self.scratch_r[..m].copy_from_slice(&r[..m]);
            Self::run(
                self.outgoing,
                &mut self.fet,
                &mut self.opto,
                &mut self.scratch_l[..m],
                &mut self.scratch_r[..m],
            );
            Self::run(model, &mut self.fet, &mut self.opto, l, r);
            let total = self.xfade_len.max(1) as f32;
            for i in 0..m {
                let remaining = self.xfade.saturating_sub(i) as f32;
                let w_new = 1.0 - remaining / total;
                let w_old = 1.0 - w_new;
                l[i] = l[i] * w_new + self.scratch_l[i] * w_old;
                r[i] = r[i] * w_new + self.scratch_r[i] * w_old;
            }
            self.xfade = self.xfade.saturating_sub(m);
        } else {
            self.xfade = 0;
            Self::run(model, &mut self.fet, &mut self.opto, l, r);
        }

        let mut pout = [0.0f32; 2];
        for i in 0..n {
            pout[0] = pout[0].max(l[i].abs());
            pout[1] = pout[1].max(r[i].abs());
        }
        self.out_peak = pout;
        match model {
            Model::Fet => {
                self.gr_db = self.fet.gr_db();
                self.meter_vu = self.fet.take_meter_reading();
            }
            Model::Opto => {
                let f = self.opto.meter_frame();
                self.gr_db = -f[4];
                self.meter_vu = f[5];
            }
        }
    }

    /// The gain change of the active model in dB (≤ 0) for the last block.
    pub fn gr_db(&self) -> f32 {
        self.gr_db
    }

    /// What the active model's panel meter reads, in dB, for the last block
    /// (see the `meter` stream in the module docs).
    pub fn meter_vu(&self) -> f32 {
        self.meter_vu
    }

    /// `[in_l, in_r, out_l, out_r, gr_db, meter_vu]` for the last block.
    pub fn meter_frame(&self) -> [f32; METER_LEN] {
        [
            self.in_peak[0],
            self.in_peak[1],
            self.out_peak[0],
            self.out_peak[1],
            self.gr_db,
            self.meter_vu,
        ]
    }

    /// The T4 cell state (`[light, free_carriers, trapped_carriers]`); zeros
    /// while the 1176 is active.
    pub fn cell_state(&self) -> [f32; 3] {
        match self.settings.model {
            Model::Fet => [0.0; 3],
            Model::Opto => self.opto.cell_state(),
        }
    }

    /// Fill `out` with the active model's static transfer curve: output
    /// level in dBFS for a sine at [`TRANSFER_MIN_DB`]..[`TRANSFER_MAX_DB`].
    pub fn transfer(&self, out: &mut [f32; TRANSFER_POINTS]) {
        match self.settings.model {
            Model::Fet => self.fet.transfer(out),
            Model::Opto => self
                .opto
                .transfer_curve(out, TRANSFER_MIN_DB, TRANSFER_MAX_DB),
        }
    }

    /// Publish the streams after [`process`](Self::process): the meter every
    /// block, the cell while the LA-2A is active (zeros once after a switch
    /// to the 1176), the transfer curve when it is due (on the fourth block
    /// after a change, so a knob sweep does not flood the stream). Real-time
    /// safe.
    pub fn publish(&mut self, audio: &mut AudioHandle) {
        audio.publish_slice(STREAM_IX.meter, &self.meter_frame());
        match self.settings.model {
            Model::Opto => {
                audio.publish_slice(STREAM_IX.cell, &self.opto.cell_state());
                self.cell_zeroed = false;
            }
            Model::Fet => {
                if !self.cell_zeroed {
                    audio.publish_slice(STREAM_IX.cell, &[0.0; 3]);
                    self.cell_zeroed = true;
                }
            }
        }
        self.blocks += 1;
        if self.curve_due && self.blocks.is_multiple_of(4) {
            match self.settings.model {
                Model::Fet => self.fet.transfer(&mut self.transfer),
                Model::Opto => {
                    self.opto
                        .transfer_curve(&mut self.transfer, TRANSFER_MIN_DB, TRANSFER_MAX_DB)
                }
            }
            audio.publish_slice(STREAM_IX.transfer, &self.transfer);
            self.curve_due = false;
        }
    }
}

#[cfg(test)]
mod tests;

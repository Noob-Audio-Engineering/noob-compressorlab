//! The DSP of Noob CompressorLab, and the bridge description shared by the
//! plug-in and the standalone.
//!
//! One instance is one compressor at a time, and the `model` parameter says
//! which: the 1176 ([`fet`]), the LA-2A ([`opto`]), the LA-3A ([`opto3`]),
//! the Distressor ([`vca`]) or the 6176, which is the 610 preamp ([`pre`])
//! in front of the 1176 engine. Every engine lives in the [`Processor`];
//! only the active one runs, the others are silent until switched in, at
//! which point the new one starts from rest and takes over through a short
//! crossfade so the switch does not click. Every knob of every model is a
//! parameter of the instance, so a project saves the whole lab, not just
//! the model in use.
//!
//! ## Layout
//!
//! | module | contents |
//! |---|---|
//! | [`fet`] | the 1176: oversampled feedback FET compressor, its revisions, knobs and tests |
//! | [`opto`] | the LA-2A: the T4 cell model, sidechain and tube stage, its knobs and tests |
//! | [`opto3`] | the LA-3A: the same T4 cell driven hard by a transistor sidechain, with a class-AB amplifier |
//! | [`vca`] | the Distressor: a feedback VCA compressor in the dB domain, its eight curves, its distortion modes and British mode |
//! | [`pre`] | the 610 tube preamp, which with the 1176 engine behind it makes the 6176 channel |
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
//! | `opto_meter_zero` | −2..2 dB (the panel trim; moves the needle only) | 0 | LA-2A |
//! | `la3a_gain`, `la3a_peak_reduction` | 0..100 | 32, 40 | LA-3A |
//! | `la3a_mode` | Compress, Limit | Compress | LA-3A |
//! | `la3a_meter` | Gain Reduction, Output, Off | Gain Reduction | LA-3A |
//! | `la3a_emphasis` | 0 (flat, as the trimmer ships) .. 1 | 0 | LA-3A |
//! | `la3a_cell` | Fresh, Used, Tired | Fresh | LA-3A |
//! | `dist_input`, `dist_output`, `dist_attack`, `dist_release` | 0..10.5 | 5 | Distressor |
//! | `dist_ratio` | 1:1, 2:1, 3:1, 4:1, 6:1, 10:1, 20:1, Nuke | 6:1 | Distressor |
//! | `dist_detector` | Norm, HP, Band, HP+Band | Norm | Distressor |
//! | `dist_audio` | Norm, HP, Dist 2, Dist 3, HP+Dist 2, HP+Dist 3 | Norm | Distressor |
//! | `dist_british` | toggle | off | Distressor |
//! | `dist_link_mode` | Phase, Image, Both | Phase | Distressor |
//! | `dist_headroom` | 4..28 dB | 16 | Distressor |
//! | `pre_join` | Join, BP, 1:1 | Join | 6176 |
//! | `pre_gain` | −10, −5, 0, +5, +10 | 0 | 6176 |
//! | `pre_input` | Line, Mic 500, Mic 2.0K, Hi-Z 47K, Hi-Z 2.2M | Line | 6176 |
//! | `pre_pad`, `pre_polarity`, `pre_hpf` | toggles | off | 6176 |
//! | `pre_level` | 0..10 | 7 | 6176 |
//! | `pre_lf_freq`, `pre_hf_freq` | 70/100/200 Hz, 4.5k/7k/10k | 100, 10k | 6176 |
//! | `pre_lf_gain`, `pre_hf_gain` | −9..+9 dB in eleven steps | 0 | 6176 |
//! | `pre_voice` | 610B, 610A | 610B | 6176 |
//! | `pre_load` | 15k, 600 | 15k | 6176 |
//! | `pre_meter` | PRE, GR, COMP | GR | 6176 |
//! | `pre_phantom` | toggle (+48 V; panel state, nothing audible) | off | 6176 |
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
//! | `meter` | meter | 6 | every block | `[in_l, in_r, out_l, out_r, gr_db, meter_vu]`: linear peaks (1.0 = 0 dBFS), the gain change in dB (≤ 0 for every model) and what the active model's panel meter reads in dB (the GR positions: `gr_db`; the output positions: the block's VU reading against −18 dBFS = 0 VU, the 1176's +8 mode 4 dB lower; the 1176's Off and the LA-3A's Off: −60; the 6176's PRE position: the preamp's own meter) |
//! | `cell` | raw | 3 | every block while the LA-2A or the LA-3A is active | `[light, free_carriers, trapped_carriers]`, 0..1; zeros once when a model without a cell takes over |
//! | `transfer` | curve, sticky | 128 | on change | the active model's static output level in dBFS for a sine at −60..0 dBFS input |
//! | `lamps` | raw | 4 | every block while the Distressor or the 6176 is active | `[thd_pct, redline, pre_vu_db, drive]`: the Distressor's estimated generator distortion and its two lamps, and the 610 section's meter reading and input-stage drive; zeros once when another model takes over |
//!
//! ## Real-time rules
//!
//! Everything reachable from [`Processor::process`] runs without
//! allocation, locks or I/O. Parameters are read from atomics into a
//! [`Settings`] snapshot once per block; the engines smooth the continuous
//! ones themselves.

pub mod fet;
pub mod opto;
pub mod opto3;
pub mod pre;
pub mod source;
pub mod vca;
pub mod vu;

use noob_vst_webgui_framework::{
    AudioHandle, NoobVstWebguiFramework, ParamSpec, StreamKind, StreamSpec,
};
use serde_json::json;

pub use source::{SOURCE_NAMES, Source};

/// Labels of `model`, in parameter order. The first two keep their places
/// so that a project saved before the lab grew still loads.
pub const MODEL_NAMES: [&str; 5] = ["1176", "LA-2A", "LA-3A", "Distressor", "6176"];
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
    /// The LA-3A ([`opto3`]).
    Opto3,
    /// The Distressor ([`vca`]).
    Vca,
    /// The 6176: the 610 preamp ([`pre`]) in front of the 1176.
    Pre6176,
}

impl Model {
    /// Every model in parameter order.
    pub const ALL: [Model; 5] = [
        Model::Fet,
        Model::Opto,
        Model::Opto3,
        Model::Vca,
        Model::Pre6176,
    ];

    /// From the parameter value / label index (clamped).
    pub fn from_index(i: usize) -> Self {
        Model::ALL.get(i).copied().unwrap_or(Model::Fet)
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
    pub opto3: opto3::Settings,
    pub vca: vca::Settings,
    /// The 610 section, which only the 6176 model uses.
    pub pre: pre::Settings,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            model: Model::Fet,
            fet: fet::Settings::default(),
            opto: opto::Settings::default(),
            opto3: opto3::Settings::default(),
            vca: vca::Settings::default(),
            pre: pre::Settings::default(),
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
        self.opto3.link = s.link;
        self.opto3.mix = s.mix;
        self.opto3.sc_hpf = s.sc_hpf_hz;
        self.opto3.bypass = s.bypass;
        self.vca.link = s.link;
        self.vca.mix = s.mix;
        self.vca.sc_hpf_hz = s.sc_hpf_hz;
        self.vca.bypass = s.bypass;
        // The 6176's preamp is in the path too, so a bypassed instance has
        // to take the tube stage and both transformers out with everything
        // else; without this a bypassed 6176 still passed a saturating
        // stage worth about +9 dB at the default Level.
        self.pre.bypass = s.bypass;
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
    pub opto_meter_zero: usize,
    pub la3a_gain: usize,
    pub la3a_peak_reduction: usize,
    pub la3a_mode: usize,
    pub la3a_meter: usize,
    pub la3a_emphasis: usize,
    pub la3a_cell: usize,
    pub dist_input: usize,
    pub dist_output: usize,
    pub dist_attack: usize,
    pub dist_release: usize,
    pub dist_ratio: usize,
    pub dist_detector: usize,
    pub dist_audio: usize,
    pub dist_british: usize,
    pub dist_link_mode: usize,
    pub dist_headroom: usize,
    pub pre_join: usize,
    pub pre_gain: usize,
    pub pre_input: usize,
    pub pre_pad: usize,
    pub pre_polarity: usize,
    pub pre_level: usize,
    pub pre_lf_freq: usize,
    pub pre_lf_gain: usize,
    pub pre_hf_freq: usize,
    pub pre_hf_gain: usize,
    pub pre_hpf: usize,
    pub pre_voice: usize,
    pub pre_load: usize,
    pub pre_meter: usize,
    pub pre_phantom: usize,
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
    pub lamps: usize,
}

/// The fixed stream layout.
pub const STREAM_IX: StreamIx = StreamIx {
    meter: 0,
    cell: 1,
    transfer: 2,
    lamps: 3,
};

/// Values in one `lamps` frame.
pub const LAMPS_LEN: usize = 4;

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
        StreamSpec::new("lamps", LAMPS_LEN)
            .name("Lamps")
            .kind(StreamKind::Raw)
            .meta(json!({ "layout": "thd_pct,redline,pre_vu_db,drive" })),
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
        ParamSpec::new("opto_meter_zero", "Meter Zero")
            .range(-2.0, 2.0)
            .default(0.0)
            .unit("dB")
            .not_automatable()
            .group("LA-2A"),
        ParamSpec::new("la3a_gain", "Gain")
            .range(0.0, 100.0)
            .default(32.0)
            .group("LA-3A"),
        ParamSpec::new("la3a_peak_reduction", "Peak Reduction")
            .range(0.0, 100.0)
            .default(40.0)
            .group("LA-3A"),
        ParamSpec::new("la3a_mode", "Mode")
            .labels(opto3::MODE_NAMES)
            .default(0.0)
            .group("LA-3A"),
        ParamSpec::new("la3a_meter", "Meter")
            .labels(opto3::METER_NAMES)
            .default(0.0)
            .not_automatable()
            .group("LA-3A"),
        ParamSpec::new("la3a_emphasis", "HF Contour")
            .range(0.0, 1.0)
            .default(0.0)
            .group("LA-3A"),
        ParamSpec::new("la3a_cell", "Cell")
            .labels(opto3::CELL_NAMES)
            .default(0.0)
            .not_automatable()
            .group("LA-3A"),
        ParamSpec::new("dist_input", "Input")
            .range(0.0, vca::KNOB_MAX)
            .default(5.0)
            .group("Distressor"),
        ParamSpec::new("dist_output", "Output")
            .range(0.0, vca::KNOB_MAX)
            .default(5.0)
            .group("Distressor"),
        ParamSpec::new("dist_attack", "Attack")
            .range(0.0, vca::KNOB_MAX)
            .default(5.0)
            .group("Distressor"),
        ParamSpec::new("dist_release", "Release")
            .range(0.0, vca::KNOB_MAX)
            .default(5.0)
            .group("Distressor"),
        ParamSpec::new("dist_ratio", "Ratio")
            .labels(vca::RATIO_NAMES)
            .default(4.0)
            .group("Distressor"),
        ParamSpec::new("dist_detector", "Detector")
            .labels(vca::DETECTOR_NAMES)
            .default(0.0)
            .group("Distressor"),
        ParamSpec::new("dist_audio", "Audio")
            .labels(vca::AUDIO_NAMES)
            .default(0.0)
            .group("Distressor"),
        ParamSpec::new("dist_british", "British Mode")
            .toggle()
            .default(0.0)
            .group("Distressor"),
        ParamSpec::new("dist_link_mode", "Link Mode")
            .labels(vca::LINK_MODE_NAMES)
            .default(0.0)
            .not_automatable()
            .group("Distressor"),
        ParamSpec::new("dist_headroom", "Headroom")
            .range(vca::HEADROOM_MIN_DB, vca::HEADROOM_MAX_DB)
            .default(vca::HEADROOM_DEFAULT_DB)
            .unit("dB")
            .not_automatable()
            .group("Distressor"),
        ParamSpec::new("pre_join", "Routing")
            .labels(pre::ROUTING_NAMES)
            .default(0.0)
            .group("6176"),
        ParamSpec::new("pre_gain", "Gain")
            .labels(pre::GAIN_NAMES)
            .default(2.0)
            .group("6176"),
        ParamSpec::new("pre_input", "Input")
            .labels(pre::INPUT_NAMES)
            .default(0.0)
            .not_automatable()
            .group("6176"),
        ParamSpec::new("pre_pad", "Pad").toggle().group("6176"),
        ParamSpec::new("pre_polarity", "Polarity")
            .toggle()
            .group("6176"),
        ParamSpec::new("pre_level", "Level")
            .range(0.0, pre::LEVEL_MAX)
            .default(7.0)
            .group("6176"),
        ParamSpec::new("pre_lf_freq", "Low Freq")
            .labels(pre::LF_FREQ_NAMES)
            .default(1.0)
            .group("6176"),
        ParamSpec::new("pre_lf_gain", "Low")
            .labels(pre::SHELF_GAIN_NAMES)
            .default(5.0)
            .group("6176"),
        ParamSpec::new("pre_hf_freq", "High Freq")
            .labels(pre::HF_FREQ_NAMES)
            .default(2.0)
            .group("6176"),
        ParamSpec::new("pre_hf_gain", "High")
            .labels(pre::SHELF_GAIN_NAMES)
            .default(5.0)
            .group("6176"),
        ParamSpec::new("pre_hpf", "Low Cut").toggle().group("6176"),
        ParamSpec::new("pre_voice", "Voicing")
            .labels(pre::VOICE_NAMES)
            .default(0.0)
            .not_automatable()
            .group("6176"),
        ParamSpec::new("pre_load", "Output Load")
            .labels(pre::LOAD_NAMES)
            .default(0.0)
            .not_automatable()
            .group("6176"),
        ParamSpec::new("pre_meter", "Meter")
            .labels(pre::METER_NAMES)
            .default(1.0)
            .not_automatable()
            .group("6176"),
        ParamSpec::new("pre_phantom", "+48 V")
            .toggle()
            .not_automatable()
            .group("6176"),
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
            "vendor": "Noob Audio Engineering",
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
        opto_meter_zero: ix("opto_meter_zero"),
        la3a_gain: ix("la3a_gain"),
        la3a_peak_reduction: ix("la3a_peak_reduction"),
        la3a_mode: ix("la3a_mode"),
        la3a_meter: ix("la3a_meter"),
        la3a_emphasis: ix("la3a_emphasis"),
        la3a_cell: ix("la3a_cell"),
        dist_input: ix("dist_input"),
        dist_output: ix("dist_output"),
        dist_attack: ix("dist_attack"),
        dist_release: ix("dist_release"),
        dist_ratio: ix("dist_ratio"),
        dist_detector: ix("dist_detector"),
        dist_audio: ix("dist_audio"),
        dist_british: ix("dist_british"),
        dist_link_mode: ix("dist_link_mode"),
        dist_headroom: ix("dist_headroom"),
        pre_join: ix("pre_join"),
        pre_gain: ix("pre_gain"),
        pre_input: ix("pre_input"),
        pre_pad: ix("pre_pad"),
        pre_polarity: ix("pre_polarity"),
        pre_level: ix("pre_level"),
        pre_lf_freq: ix("pre_lf_freq"),
        pre_lf_gain: ix("pre_lf_gain"),
        pre_hf_freq: ix("pre_hf_freq"),
        pre_hf_gain: ix("pre_hf_gain"),
        pre_hpf: ix("pre_hpf"),
        pre_voice: ix("pre_voice"),
        pre_load: ix("pre_load"),
        pre_meter: ix("pre_meter"),
        pre_phantom: ix("pre_phantom"),
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
            meter_zero: audio.param(ix.opto_meter_zero),
            ..opto::Settings::default()
        },
        opto3: opto3::Settings {
            gain: audio.param(ix.la3a_gain),
            peak_reduction: audio.param(ix.la3a_peak_reduction),
            limit: audio.param(ix.la3a_mode) >= 0.5,
            meter: audio.param(ix.la3a_meter).round().clamp(0.0, 2.0) as usize,
            emphasis: audio.param(ix.la3a_emphasis),
            cell: audio.param(ix.la3a_cell).round().clamp(0.0, 2.0) as usize,
            ..opto3::Settings::default()
        },
        vca: vca::Settings {
            input: audio.param(ix.dist_input),
            output: audio.param(ix.dist_output),
            attack: audio.param(ix.dist_attack),
            release: audio.param(ix.dist_release),
            ratio: vca::Ratio::from_index(audio.param(ix.dist_ratio).round() as usize),
            detector: vca::Detector::from_index(audio.param(ix.dist_detector).round() as usize),
            audio: vca::AudioMode::from_index(audio.param(ix.dist_audio).round() as usize),
            british: audio.param(ix.dist_british) >= 0.5,
            link_mode: vca::LinkMode::from_index(audio.param(ix.dist_link_mode).round() as usize),
            headroom_db: audio.param(ix.dist_headroom),
            ..vca::Settings::default()
        },
        pre: pre::Settings {
            routing: pre::Routing::from_index(audio.param(ix.pre_join).round() as usize),
            gain: audio.param(ix.pre_gain).round().clamp(0.0, 4.0) as usize,
            input: audio.param(ix.pre_input).round().clamp(0.0, 4.0) as usize,
            pad: audio.param(ix.pre_pad) >= 0.5,
            polarity: audio.param(ix.pre_polarity) >= 0.5,
            level: audio.param(ix.pre_level),
            lf_freq: audio.param(ix.pre_lf_freq).round().clamp(0.0, 2.0) as usize,
            lf_gain: audio.param(ix.pre_lf_gain).round().clamp(0.0, 10.0) as usize,
            hf_freq: audio.param(ix.pre_hf_freq).round().clamp(0.0, 2.0) as usize,
            hf_gain: audio.param(ix.pre_hf_gain).round().clamp(0.0, 10.0) as usize,
            hpf: audio.param(ix.pre_hpf) >= 0.5,
            voice: audio.param(ix.pre_voice).round().clamp(0.0, 1.0) as usize,
            load: audio.param(ix.pre_load).round().clamp(0.0, 1.0) as usize,
            meter: audio.param(ix.pre_meter).round().clamp(0.0, 2.0) as usize,
            phantom: audio.param(ix.pre_phantom) >= 0.5,
            // The shared bypass is stamped in by `with_shared`.
            bypass: false,
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
    opto3: opto3::Compressor,
    vca: vca::Compressor,
    pre: pre::Stage,
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
    /// The panel meter's movement, run on the audio thread so the needle
    /// does not depend on how often the page repaints (see [`vu`]).
    vu: vu::Vu,
    transfer: [f32; TRANSFER_POINTS],
    curve_due: bool,
    cell_zeroed: bool,
    lamps_zeroed: bool,
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
            opto3: opto3::Compressor::new(sr),
            vca: vca::Compressor::new(sr),
            pre: pre::Stage::new(sr),
            outgoing: Model::Fet,
            xfade: 0,
            xfade_len: (XFADE_SECONDS * sr).round() as usize,
            scratch_l: vec![0.0; MAX_BLOCK],
            scratch_r: vec![0.0; MAX_BLOCK],
            in_peak: [0.0; 2],
            out_peak: [0.0; 2],
            gr_db: 0.0,
            meter_vu: vu::REST_DB,
            vu: vu::Vu::new(sr),
            transfer: [0.0; TRANSFER_POINTS],
            curve_due: true,
            cell_zeroed: false,
            lamps_zeroed: false,
            blocks: 0,
        }
    }

    /// Retune both engines to `sr` and start from rest.
    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sr = sr;
        self.xfade_len = (XFADE_SECONDS * sr).round() as usize;
        self.vu.set_sample_rate(sr);
        self.fet.set_sample_rate(sr);
        self.opto.set_sample_rate(sr);
        self.opto3.set_sample_rate(sr);
        self.vca.set_sample_rate(sr);
        self.pre.set_sample_rate(sr);
        self.first = true;
        self.reset();
    }

    /// Clear every state (both engines, the crossfade, the meters).
    pub fn reset(&mut self) {
        self.fet.reset();
        self.opto.reset();
        self.opto3.reset();
        self.vca.reset();
        self.pre.reset();
        self.xfade = 0;
        self.in_peak = [0.0; 2];
        self.out_peak = [0.0; 2];
        self.gr_db = 0.0;
        self.vu.reset();
        self.meter_vu = self.vu.value();
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
            // The 6176 always runs the 1176 engine, even in the BP and 1:1
            // positions (bypassed there, which is delay-matched), so its
            // latency does not change with the routing switch.
            Model::Fet => self.fet.latency(),
            Model::Pre6176 => self.pre.latency() + self.fet.latency(),
            Model::Vca => self.vca.latency(),
            Model::Opto3 => self.opto3.latency(),
            Model::Opto => 0,
        }
    }

    /// The 1176 settings the 6176 runs with: the routing switch turns the
    /// compressor into a delay-matched pass-through (BP) or takes the gain
    /// reduction away but keeps the amplifiers (1:1), and the 6176's meter
    /// switch drives the 1176's.
    fn fet_settings(s: &Settings) -> fet::Settings {
        let mut f = s.fet;
        if s.model == Model::Pre6176 {
            match s.pre.routing {
                pre::Routing::Join => {}
                pre::Routing::Bypass => f.bypass = true,
                pre::Routing::Unity => f.attack = 0.0,
            }
            f.meter = match s.pre.meter {
                pre::METER_GR => fet::MeterMode::Gr,
                pre::METER_COMP => fet::MeterMode::Plus4,
                _ => fet::MeterMode::Off,
            };
        }
        f
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
                Model::Opto3 => self.opto3.reset(),
                Model::Vca => self.vca.reset(),
                Model::Pre6176 => {
                    self.pre.reset();
                    self.fet.reset();
                }
            }
            changed = true;
        }
        changed |= self.fet.configure(&Self::fet_settings(s));
        changed |= self.opto.configure(s.opto);
        changed |= self.opto3.configure(s.opto3);
        changed |= self.vca.configure(&s.vca);
        changed |= self.pre.configure(&s.pre);
        self.settings = *s;
        self.first = false;
        if changed {
            self.curve_due = true;
        }
        changed
    }

    #[inline]
    fn run(&mut self, model: Model, l: &mut [f32], r: &mut [f32]) {
        match model {
            Model::Fet => self.fet.process(l, r),
            Model::Opto => self.opto.process_block(l, r),
            Model::Opto3 => self.opto3.process_block(l, r),
            Model::Vca => self.vca.process_block(l, r),
            Model::Pre6176 => {
                // The preamp always runs; the routing switch is already in
                // the 1176's settings (see `fet_settings`).
                self.pre.process_block(l, r);
                self.fet.process(l, r);
            }
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
            let mut scratch_l = std::mem::take(&mut self.scratch_l);
            let mut scratch_r = std::mem::take(&mut self.scratch_r);
            self.run(self.outgoing, &mut scratch_l[..m], &mut scratch_r[..m]);
            self.scratch_l = scratch_l;
            self.scratch_r = scratch_r;
            self.run(model, l, r);
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
            self.run(model, l, r);
        }

        let mut pout = [0.0f32; 2];
        for i in 0..n {
            pout[0] = pout[0].max(l[i].abs());
            pout[1] = pout[1].max(r[i].abs());
        }
        self.out_peak = pout;
        // What the needle is chasing this block; the movement below turns
        // it into where the needle actually is.
        let target = match model {
            Model::Fet => {
                self.gr_db = self.fet.gr_db();
                self.fet.take_meter_reading()
            }
            Model::Opto => {
                let f = self.opto.meter_frame();
                self.gr_db = -f[4];
                f[5]
            }
            Model::Opto3 => {
                let f = self.opto3.meter_frame();
                self.gr_db = -f[4];
                f[5]
            }
            Model::Vca => {
                self.gr_db = self.vca.gr_db();
                self.gr_db
            }
            Model::Pre6176 => {
                // In the BP position the compressor is out of the path, so
                // its detector's reading is not the instance's.
                self.gr_db = if self.settings.pre.routing == pre::Routing::Bypass {
                    0.0
                } else {
                    self.fet.gr_db()
                };
                let comp = self.fet.take_meter_reading();
                match self.settings.pre.meter {
                    pre::METER_PRE => self.pre.pre_vu_db(),
                    pre::METER_GR => self.gr_db,
                    _ => comp,
                }
            }
        };
        self.vu.advance(target, n);
        self.meter_vu = self.vu.value();
    }

    /// The gain change of the active model in dB (≤ 0) for the last block.
    pub fn gr_db(&self) -> f32 {
        self.gr_db
    }

    /// Where the active model's needle **is**, in dB, after the last block:
    /// the meter's own target put through the standard VU movement (see
    /// [`vu`]). A page draws this as it arrives; smoothing it again would
    /// double the ballistics.
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
            Model::Opto => self.opto.cell_state(),
            Model::Opto3 => self.opto3.cell_state(),
            _ => [0.0; 3],
        }
    }

    /// `[thd_pct, redline, pre_vu_db, drive]` for the last block: the
    /// Distressor's distortion estimate and its two lamps, and the 610
    /// section's meter and input-stage drive. Zeros for the models that
    /// have neither.
    pub fn lamps_frame(&self) -> [f32; LAMPS_LEN] {
        match self.settings.model {
            Model::Vca => [
                self.vca.thd_pct(),
                if self.vca.redline() { 1.0 } else { 0.0 },
                0.0,
                self.vca.drive(),
            ],
            Model::Pre6176 => [0.0, 0.0, self.pre.pre_vu_db(), self.pre.drive()],
            _ => [0.0; LAMPS_LEN],
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
            Model::Opto3 => self
                .opto3
                .transfer_curve(out, TRANSFER_MIN_DB, TRANSFER_MAX_DB),
            Model::Vca => self
                .vca
                .transfer_curve(out, TRANSFER_MIN_DB, TRANSFER_MAX_DB),
            Model::Pre6176 => self.transfer_6176(out),
        }
    }

    /// The 6176's curve: the preamp's own static curve, then the 1176's
    /// read at the level the preamp hands it. The 1176 engine only fills a
    /// fixed grid, so its curve is sampled by interpolation.
    fn transfer_6176(&self, out: &mut [f32; TRANSFER_POINTS]) {
        let mut fet_curve = [0.0f32; TRANSFER_POINTS];
        self.fet.transfer(&mut fet_curve);
        let span = TRANSFER_MAX_DB - TRANSFER_MIN_DB;
        let last = (TRANSFER_POINTS - 1) as f32;
        for (i, o) in out.iter_mut().enumerate() {
            let x = TRANSFER_MIN_DB + span * i as f32 / last;
            let pre_out = self.pre.static_out_db(x);
            if self.settings.pre.routing == pre::Routing::Bypass {
                *o = pre_out;
                continue;
            }
            // Where that level sits on the 1176's grid.
            let t = ((pre_out - TRANSFER_MIN_DB) / span * last).clamp(0.0, last);
            let j = t.floor() as usize;
            let f = t - j as f32;
            let a = fet_curve[j];
            let b = fet_curve[(j + 1).min(TRANSFER_POINTS - 1)];
            *o = a + (b - a) * f;
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
            Model::Opto | Model::Opto3 => {
                audio.publish_slice(STREAM_IX.cell, &self.cell_state());
                self.cell_zeroed = false;
            }
            _ => {
                if !self.cell_zeroed {
                    audio.publish_slice(STREAM_IX.cell, &[0.0; 3]);
                    self.cell_zeroed = true;
                }
            }
        }
        match self.settings.model {
            Model::Vca | Model::Pre6176 => {
                audio.publish_slice(STREAM_IX.lamps, &self.lamps_frame());
                self.lamps_zeroed = false;
            }
            _ => {
                if !self.lamps_zeroed {
                    audio.publish_slice(STREAM_IX.lamps, &[0.0; LAMPS_LEN]);
                    self.lamps_zeroed = true;
                }
            }
        }
        self.blocks += 1;
        if self.curve_due && self.blocks.is_multiple_of(4) {
            let mut curve = self.transfer;
            self.transfer(&mut curve);
            self.transfer = curve;
            audio.publish_slice(STREAM_IX.transfer, &self.transfer);
            self.curve_due = false;
        }
    }
}

#[cfg(test)]
mod tests;

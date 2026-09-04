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
//! | [`rms`] | the dbx 160: a Blackmer cell fed forward from a true-RMS log-domain detector |
//! | [`vmu`] | the Fairchild 660 and 670: a push-pull pair of remote-cutoff triodes that are the amplifier and the gain element at once, with the six-position timing network and the lateral-vertical matrix |
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
//! | `dbx_model` | 160, 160A | 160 | dbx 160 |
//! | `dbx_threshold` | −40..+20 dBu | 0 | dbx 160 |
//! | `dbx_ratio` | α = 1 − 1/R, 0..2 along the dial's own taper | 0.75 (4:1) | dbx 160 |
//! | `dbx_output` | −20..+20 dB | 0 | dbx 160 |
//! | `dbx_knee` | Hard, OverEasy (160A only) | Hard | dbx 160 |
//! | `dbx_meter` | Input, Output, Gain Change | Gain Change | dbx 160 |
//! | `dbx_meter_cal` | −15..+10 dBu (the rear-panel trimmer) | +4 | dbx 160 |
//! | `dbx_knee_width` | 0..12 dB (ours; dbx published none) | 6 | dbx 160 |
//! | `dbx_tau` | 20..60 ms (ours; the one number the box is made of) | 35.32 | dbx 160 |
//! | `dbx_lookahead` | 0..10 ms (ours; dbx documented the trick in 1995) | 0 | dbx 160 |
//! | `dbx_headroom` | 4..28 dB | 22 | dbx 160 |
//! | `fc_model` | 660, 670 | 670 | 670 |
//! | `fc_input_gain_l`, `fc_input_gain_r` | 0..20 dB of attenuation, 21 detents | 10 | 670 |
//! | `fc_threshold_l`, `fc_threshold_r` | 0..10, the panel's own scale and **not decibels** | 10 | 670 |
//! | `fc_time_l`, `fc_time_r` | positions 1..6 | 3 | 670 |
//! | `fc_dc_threshold_l`, `fc_dc_threshold_r` | 0..1, the trimmer inside the chassis | 0.20 | 670 |
//! | `fc_zero_l`, `fc_zero_r` | −12..−3 V of standing grid bias | −7.2 | 670 |
//! | `fc_balance_l`, `fc_balance_r` | −1..1 | 0 | 670 |
//! | `fc_meter_l`, `fc_meter_r` | Bal Push / Zero / Bal Pull | Zero | 670 |
//! | `fc_agc` | Left/Right, Lat/Vert | Left/Right | 670 |
//! | `fc_tube` | GE 6386, JJ 6386 LGP (ours) | GE | 670 |
//! | `fc_oversample` | 4x, 8x, 16x (ours) | 8x | 670 |
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
//! | `lamps` | raw | 4 | every block while the Distressor, the 6176 or the dbx 160 is active | `[thd_pct, redline, pre_vu_db, drive]`: the Distressor's estimated generator distortion and its two lamps, and the 610 section's meter reading and input-stage drive. The dbx 160 fills the same four slots with `[below, above, ghost_gr_db, overeasy]`: its two threshold indicators, what a peak detector would have asked for, and its OverEasy indicator. Zeros once when a model with neither takes over |
//!
//! ## Real-time rules
//!
//! Everything reachable from [`Processor::process`] runs without
//! allocation, locks or I/O. Parameters are read from atomics into a
//! [`Settings`] snapshot once per block; the engines smooth the continuous
//! ones themselves.

pub mod bridge;
pub mod fet;
pub mod gbus;
pub mod opto;
pub mod opto1b;
pub mod opto3;
pub mod pre;
pub mod rms;
pub mod source;
pub mod tg;
pub mod vca;
pub mod vmu;
pub mod vu;

/// Flush a state to zero once it has decayed far enough that arithmetic on
/// it is a cost rather than a signal.
///
/// Every filter and follower here decays exponentially, so without this a
/// state creeps into the subnormal range and sits there, where arithmetic
/// is slow on some hardware. It is not theoretical: an envelope follower
/// in the equaliser next door parked on a subnormal permanently after
/// eleven seconds of silence.
///
/// **One threshold, and it is the smaller one.** This used to exist in
/// four copies across the engines with two different cutoffs, 1e-9 in
/// three of them and 1e-12 in the optical one, so a decaying tail parked
/// at different points depending on which engine held it. They now agree
/// on 1e-12, and the choice is not a count of which was more common.
///
/// The guard exists solely to keep a state out of the subnormal range,
/// which for `f32` begins near 1e-38. 1e-12 clears that by twenty-six
/// orders of magnitude, so it prevents the stall completely; a larger
/// threshold buys nothing against stalls and can only zero more state that
/// was genuinely non-zero. For a guard, the conservative direction is
/// therefore the smaller number.
///
/// That matters most where it was already smallest. The optical engines'
/// cell carries trapped-carrier state that decays over seconds and is the
/// mechanism behind the programme dependence an LA-2A is known for, so a
/// guard that clamps it early is exactly the wrong place to be generous.
/// None of the three engines that used 1e-9 gave a reason for it.
#[inline]
pub fn flush(x: f32) -> f32 {
    if x.abs() < 1e-12 { 0.0 } else { x }
}

use noob_vst_webgui_framework::{
    AudioHandle, NoobVstWebguiFramework, ParamSpec, StreamKind, StreamSpec,
};
use serde_json::json;

pub use source::{SOURCE_NAMES, Source};

/// Labels of `model`, in parameter order. The first two keep their places
/// so that a project saved before the lab grew still loads.
pub const MODEL_NAMES: [&str; 11] = [
    "1176",
    "LA-2A",
    "LA-3A",
    "Distressor",
    "6176",
    "CL-1B",
    "33609",
    "160",
    "TG12413",
    "4000 G",
    "670",
];
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
    /// The CL 1B ([`opto1b`]).
    Opto1b,
    /// The Neve 33609 ([`bridge`]).
    Bridge,
    /// The dbx 160 ([`rms`]).
    Rms,
    /// The EMI TG12413 ([`tg`]).
    Tg,
    /// The SSL 4000 G bus compressor ([`gbus`]).
    Gbus,
    /// The Fairchild 660 and 670 ([`vmu`]).
    Vmu,
}

impl Model {
    /// Every model in parameter order.
    pub const ALL: [Model; 11] = [
        Model::Fet,
        Model::Opto,
        Model::Opto3,
        Model::Vca,
        Model::Pre6176,
        Model::Opto1b,
        Model::Bridge,
        Model::Rms,
        Model::Tg,
        Model::Gbus,
        Model::Vmu,
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
    pub opto1b: opto1b::Settings,
    pub bridge: bridge::Settings,
    pub tg: tg::Settings,
    pub rms: rms::Settings,
    pub gbus: gbus::Settings,
    pub vmu: vmu::Settings,
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
            opto1b: opto1b::Settings::default(),
            bridge: bridge::Settings::default(),
            tg: tg::Settings::default(),
            rms: rms::Settings::default(),
            gbus: gbus::Settings::default(),
            vmu: vmu::Settings::default(),
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
        self.opto1b.link = s.link;
        self.opto1b.mix = s.mix;
        self.opto1b.sc_hpf = s.sc_hpf_hz;
        self.opto1b.bypass = s.bypass;
        self.bridge.link = s.link;
        self.bridge.mix = s.mix;
        self.bridge.sc_hpf = s.sc_hpf_hz;
        self.bridge.bypass = s.bypass;
        // The TG has no link switch and no bypass of its own: the GANG bus
        // is a wire rather than a control, and OUT is deliberately not a
        // bypass, so both come from the lab's shared strip and the panel
        // says which of its controls are ours.
        self.tg.link = s.link;
        self.tg.mix = s.mix;
        self.tg.sc_hpf = s.sc_hpf_hz;
        self.tg.bypass = s.bypass;
        // The dbx's SLAVE button and its BYPASS relay are the
        // shared link and bypass: the hardware has both and there is
        // no sense in a second copy of either.
        self.rms.link = s.link;
        self.rms.mix = s.mix;
        self.rms.sc_hpf = s.sc_hpf_hz;
        self.rms.bypass = s.bypass;
        // The SSL's own IN switch is **not** the shared bypass and keeps
        // its own parameter: it removes the sidechain and leaves the VCA
        // and the make-up gain in circuit, which is the one thing about
        // this box a plug-in author would guess wrong. Its four-way link
        // mode is its own too, since the shared link is a toggle; the
        // toggle still wins when it is off, and forces two detectors.
        self.gbus.link = s.link;
        self.gbus.mix = s.mix;
        self.gbus.sc_hpf = s.sc_hpf_hz;
        self.gbus.bypass = s.bypass;
        self.vmu.link = s.link;
        self.vmu.mix = s.mix;
        self.vmu.sc_hpf = s.sc_hpf_hz;
        self.vmu.bypass = s.bypass;
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
    pub cl1b_gain: usize,
    pub cl1b_ratio: usize,
    pub cl1b_threshold: usize,
    pub cl1b_attack: usize,
    pub cl1b_release: usize,
    pub cl1b_mode: usize,
    pub cl1b_meter: usize,
    pub cl1b_bus: usize,
    pub cl1b_power: usize,
    pub neve_model: usize,
    pub neve_limit_in: usize,
    pub neve_limit_threshold: usize,
    pub neve_limit_attack: usize,
    pub neve_limit_recovery: usize,
    pub neve_compress_in: usize,
    pub neve_compress_threshold: usize,
    pub neve_compress_ratio: usize,
    pub neve_compress_attack: usize,
    pub neve_compress_recovery: usize,
    pub neve_gain: usize,
    pub neve_meter_select: usize,
    pub neve_drive: usize,
    pub neve_power: usize,
    pub dbx_model: usize,
    pub dbx_threshold: usize,
    pub dbx_ratio: usize,
    pub dbx_output: usize,
    pub dbx_knee: usize,
    pub dbx_meter: usize,
    pub dbx_meter_cal: usize,
    pub dbx_knee_width: usize,
    pub dbx_tau: usize,
    pub dbx_lookahead: usize,
    pub dbx_headroom: usize,
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
    pub tg_mode: usize,
    pub tg_recovery: usize,
    pub tg_output: usize,
    pub tg_hold: usize,
    pub tg_region: usize,
    pub tg_mismatch: usize,
    pub tg_input: usize,
    pub tg_drive: usize,
    pub tg_oversample: usize,
    pub ssl_in: usize,
    pub ssl_threshold: usize,
    pub ssl_makeup: usize,
    pub ssl_attack: usize,
    pub ssl_release: usize,
    pub ssl_ratio: usize,
    pub ssl_hpf: usize,
    pub ssl_link: usize,
    pub ssl_drive: usize,
    pub ssl_range: usize,
    pub ssl_oversample: usize,
    pub fc_model: usize,
    pub fc_input_gain_l: usize,
    pub fc_input_gain_r: usize,
    pub fc_threshold_l: usize,
    pub fc_threshold_r: usize,
    pub fc_time_l: usize,
    pub fc_time_r: usize,
    pub fc_dc_threshold_l: usize,
    pub fc_dc_threshold_r: usize,
    pub fc_zero_l: usize,
    pub fc_zero_r: usize,
    pub fc_balance_l: usize,
    pub fc_balance_r: usize,
    pub fc_meter_l: usize,
    pub fc_meter_r: usize,
    pub fc_agc: usize,
    pub fc_tube: usize,
    pub fc_oversample: usize,
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
            .meta(json!({
                "layout": "thd_pct,redline,pre_vu_db,drive",
                "layout_160": "below,above,ghost_gr_db,overeasy"
            })),
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
        // The CL 1B's continuous knobs publish their **real** units with a
        // table sampled from the engine's own pot laws, so the page reads
        // decibels and seconds from the manifest instead of a 0-to-1
        // travel. The law therefore exists once, here, and nothing
        // reimplements it on the other side of the wire; two copies of one
        // law is the drift an audit of this repository punished across
        // five plug-ins.
        //
        // The normalised value stays linear in pot travel, which is what a
        // knob turns by and what the panel's measured scale dots are
        // fractions of, so `read_settings` takes the normalised value.
        ParamSpec::new("cl1b_gain", "Gain")
            .range(opto1b::engine::gain_db(0.0), opto1b::engine::gain_db(1.0))
            .with_table(opto1b::gain_table())
            .default(opto1b::engine::gain_db(0.265))
            .unit("dB")
            .group("CL-1B"),
        // Ratio publishes **travel between its two printed stops**, not a
        // ratio. The panel prints 2:1 and 10:1 with nothing between, and
        // the research is explicit that the printed ratio "is a label, not
        // a slope" and that the real behaviour is a ratio that rises with
        // depth, so a 2-to-10 plain value would be a number the machine
        // does not have. A percentage of travel is what the control
        // actually is, and it gives a host's parameter list and automation
        // lane something meaningful instead of a bare fraction; the two
        // stops are named in the value string the plug-in hands the host.
        ParamSpec::new("cl1b_ratio", "Ratio")
            .range(0.0, 100.0)
            .default(37.5)
            .unit("%")
            .group("CL-1B"),
        ParamSpec::new("cl1b_threshold", "Threshold")
            .range(
                opto1b::engine::threshold_dbu(1.0),
                opto1b::engine::threshold_dbu(0.0),
            )
            .with_table(opto1b::threshold_table())
            .default(opto1b::engine::threshold_dbu(0.5))
            .unit("dBu")
            .group("CL-1B"),
        ParamSpec::new("cl1b_attack", "Attack")
            .range(
                opto1b::engine::attack_s(0.0) * 1e3,
                opto1b::engine::attack_s(1.0) * 1e3,
            )
            .with_table(opto1b::attack_table())
            .default(opto1b::engine::attack_s(0.75) * 1e3)
            .unit("ms")
            .group("CL-1B"),
        ParamSpec::new("cl1b_release", "Release")
            .range(
                opto1b::engine::release_s(0.0),
                opto1b::engine::release_s(1.0),
            )
            .with_table(opto1b::release_table())
            .default(opto1b::engine::release_s(0.25))
            .unit("s")
            .group("CL-1B"),
        ParamSpec::new("cl1b_mode", "Attack/Release Select")
            .labels(opto1b::MODE_NAMES)
            .default(opto1b::MODE_MANUAL as f32)
            .group("CL-1B"),
        ParamSpec::new("cl1b_meter", "Meter")
            .labels(opto1b::METER_NAMES)
            .default(opto1b::METER_COMP as f32)
            .not_automatable()
            .group("CL-1B"),
        ParamSpec::new("cl1b_bus", "Sidechain Bus")
            .labels(opto1b::BUS_NAMES)
            .default(0.0)
            .group("CL-1B"),
        ParamSpec::new("cl1b_power", "Power")
            .toggle()
            .default(1.0)
            .not_automatable()
            .group("CL-1B"),
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
        // The 33609's controls are **all switches**, and AMS Neve make a
        // feature of that: two units set to the same positions match. So
        // every one of these is stepped, and the three with real units
        // publish decibels straight from the engine's own law rather than
        // a 0-to-1 travel. Those three laws are exactly linear, so a range
        // and a step count reproduce the switch without the sampled table
        // the CL-1B's tapered pots need.
        ParamSpec::new("neve_model", "Unit")
            .labels(bridge::MODEL_NAMES)
            .default(bridge::MODEL_33609J as f32)
            .not_automatable()
            .group("33609"),
        ParamSpec::new("neve_limit_in", "Limit In")
            .toggle()
            .default(0.0)
            .group("33609"),
        ParamSpec::new("neve_limit_threshold", "Limit Threshold")
            .range(
                bridge::engine::limit_threshold_dbu(0),
                bridge::engine::limit_threshold_dbu(22),
            )
            .steps(23)
            .default(bridge::engine::limit_threshold_dbu(8))
            .unit("dBu")
            .group("33609"),
        ParamSpec::new("neve_limit_attack", "Limit Attack")
            .labels(bridge::LIMIT_ATTACK_NAMES)
            .default(0.0)
            .group("33609"),
        ParamSpec::new("neve_limit_recovery", "Limit Recovery")
            .labels(bridge::LIMIT_RECOVERY_NAMES)
            .default(1.0)
            .group("33609"),
        ParamSpec::new("neve_compress_in", "Compress In")
            .toggle()
            .default(1.0)
            .group("33609"),
        ParamSpec::new("neve_compress_threshold", "Compress Threshold")
            .range(
                bridge::engine::compress_threshold_dbu(0),
                bridge::engine::compress_threshold_dbu(15),
            )
            .steps(16)
            .default(bridge::engine::compress_threshold_dbu(5))
            .unit("dBu")
            .group("33609"),
        // The printed labels, two of which the manufacturer's own
        // calibration table contradicts. The engine works in
        // `bridge::RATIO_TRUE` and the panel prints these, which is what
        // the hardware does.
        ParamSpec::new("neve_compress_ratio", "Ratio")
            .labels(bridge::RATIO_NAMES)
            .default(1.0)
            .group("33609"),
        // The /N's addition. The /J and the 2254 have a fixed compressor
        // attack and no control for it, so the engine ignores this unless
        // the unit is the /N.
        ParamSpec::new("neve_compress_attack", "Compress Attack")
            .labels(bridge::COMPRESS_ATTACK_NAMES)
            .default(0.0)
            .group("33609"),
        ParamSpec::new("neve_compress_recovery", "Compress Recovery")
            .labels(bridge::COMPRESS_RECOVERY_NAMES)
            .default(1.0)
            .group("33609"),
        ParamSpec::new("neve_gain", "Make-up Gain")
            .range(bridge::engine::gain_db(0), bridge::engine::gain_db(10))
            .steps(11)
            .default(bridge::engine::gain_db(0))
            .unit("dB")
            .group("33609"),
        // The 2254/E's meter switch. The 33609 has two meters and no room
        // for it, so the engine reads it only for the 2254.
        ParamSpec::new("neve_meter_select", "Meter")
            .labels(bridge::METER_NAMES)
            .default(1.0)
            .not_automatable()
            .group("33609"),
        // Not on any of the three. The bridge's distortion is set by the
        // voltage across it, and the hardware never lets the user raise
        // that, so this is the one way to hear the gain element itself.
        ParamSpec::new("neve_drive", "Drive")
            .range(0.0, 100.0)
            .default(0.0)
            .unit("%")
            .group("33609"),
        ParamSpec::new("neve_power", "Power")
            .toggle()
            .default(1.0)
            .not_automatable()
            .group("33609"),
        // The dbx's two continuous dials publish what the panel is marked
        // in, and the ratio publishes the coefficient the circuit actually
        // sets rather than a ratio, because the ratio runs through infinity
        // and comes back negative and no numeric range can say that. The
        // sampled taper is the original's own dial, measured off dbx's
        // drawing, so the nine printed marks land where they were measured.
        //
        // Both dials carry the **union** of the two units' ranges, and each
        // faceplate maps its own pot's rotation onto the part its hardware
        // has; `rms::Settings::clamped` gives the original its own limits
        // back so a preset written on the 160A face cannot smuggle them in.
        ParamSpec::new("dbx_model", "Unit")
            .labels(rms::MODEL_NAMES)
            .default(rms::MODEL_160 as f32)
            .not_automatable()
            .group("dbx 160"),
        ParamSpec::new("dbx_threshold", "Threshold")
            .range(rms::THRESHOLD_MIN_DBU, rms::THRESHOLD_MAX_DBU)
            .default(0.0)
            .unit("dBu")
            .group("dbx 160"),
        ParamSpec::new("dbx_ratio", "Compression")
            .with_table(rms::ratio_table())
            .default(0.75)
            .group("dbx 160"),
        ParamSpec::new("dbx_output", "Output Gain")
            .range(rms::OUTPUT_MIN_DB, rms::OUTPUT_MAX_DB)
            .default(0.0)
            .unit("dB")
            .group("dbx 160"),
        ParamSpec::new("dbx_knee", "Threshold Characteristic")
            .labels(rms::KNEE_NAMES)
            .default(rms::KNEE_HARD as f32)
            .group("dbx 160"),
        ParamSpec::new("dbx_meter", "Meter")
            .labels(rms::METER_NAMES)
            .default(rms::METER_GAIN_CHANGE as f32)
            .not_automatable()
            .group("dbx 160"),
        ParamSpec::new("dbx_meter_cal", "Meter Calibration")
            .range(rms::METER_CAL_MIN_DBU, rms::METER_CAL_MAX_DBU)
            .default(rms::METER_CAL_DEFAULT_DBU)
            .unit("dBu")
            .not_automatable()
            .group("dbx 160"),
        // The two dbx never gave anyone. The knee width because they never
        // published one and it cannot be read off the drawing; the time
        // constant because dbx's whole point is that you cannot adjust it,
        // and dragging it to hear the release rate change with the attack
        // is the clearest demonstration there is that they are one control.
        ParamSpec::new("dbx_knee_width", "OverEasy Width")
            .range(0.0, rms::engine::KNEE_WIDTH_MAX_DB)
            .default(rms::engine::KNEE_WIDTH_DEFAULT_DB)
            .unit("dB")
            .group("dbx 160"),
        ParamSpec::new("dbx_tau", "Detector Time Constant")
            .range(rms::engine::TAU_MIN_S * 1e3, rms::engine::TAU_MAX_S * 1e3)
            .default(rms::engine::TAU_DEFAULT_S * 1e3)
            .unit("ms")
            .group("dbx 160"),
        ParamSpec::new("dbx_lookahead", "Look-ahead")
            .range(0.0, rms::engine::LOOKAHEAD_MAX_MS)
            .default(0.0)
            .unit("ms")
            .group("dbx 160"),
        ParamSpec::new("dbx_headroom", "Headroom")
            .range(rms::HEADROOM_MIN_DB, rms::HEADROOM_MAX_DB)
            .default(rms::HEADROOM_DEFAULT_DB)
            .unit("dB")
            .not_automatable()
            .group("dbx 160"),
        // The TG12413 is three switches and one internal preset, and
        // nothing on it is continuous, so every control the hardware has
        // is stepped here too. `tg_output` is the one with real units: it
        // publishes the decibels EMI silkscreened, while the engine works
        // from the twenty-one resistor values behind them, which is what
        // the hardware does.
        ParamSpec::new("tg_mode", "Mode")
            .labels(tg::MODE_NAMES)
            .default(tg::MODE_COMPRESS as f32)
            .group("TG12413"),
        ParamSpec::new("tg_recovery", "Recovery")
            .labels(tg::RECOVERY_NAMES)
            .default(2.0)
            .group("TG12413"),
        ParamSpec::new("tg_output", "Output Level")
            .range(-10.0, 10.0)
            .steps(21)
            .default(0.0)
            .unit("dB")
            .group("TG12413"),
        // RV1 is a screwdriver preset on EMI's module, not a panel knob.
        ParamSpec::new("tg_hold", "Hold")
            .range(0.0, 100.0)
            .default(0.0)
            .unit("%")
            .group("TG12413"),
        // Not a control at all. The drawing is ambiguous about which side
        // of the diodes' characteristic the element works on, so the model
        // carries the choice where the page can say which one is on.
        ParamSpec::new("tg_region", "Region")
            .labels(tg::REGION_NAMES)
            .default(tg::REGION_BREAKDOWN as f32)
            .not_automatable()
            .group("TG12413"),
        ParamSpec::new("tg_mismatch", "Arm Mismatch")
            .range(0.0, 100.0)
            .default(0.0)
            .unit("%")
            .group("TG12413"),
        // Not on the hardware. Chandler's recreation adds a continuous
        // input, and since this module has no threshold control, driving
        // it is how you choose where it works.
        ParamSpec::new("tg_input", "Input")
            .range(-12.0, 12.0)
            .default(0.0)
            .unit("dB")
            .group("TG12413"),
        ParamSpec::new("tg_drive", "Drive")
            .range(0.0, 100.0)
            .default(0.0)
            .unit("%")
            .group("TG12413"),
        ParamSpec::new("tg_oversample", "Oversampling")
            .labels(tg::OVERSAMPLE_NAMES)
            .default(1.0)
            .not_automatable()
            .group("TG12413"),
        // The SSL's two continuous controls are 50 kOhm and 25 kOhm linear
        // pots, so they are plain ranges with no taper table; everything
        // else on the panel is a rotary switch with detents and is stepped.
        //
        // **The panel is the 500-series module and the values are the
        // console's**, which is what the dossier's section 2.5 instructs:
        // SSL publish a high-resolution render and a dimensioned recall
        // sheet of the module and nothing legible of the console, while
        // card 82E27 gives the console's component values and nothing
        // gives the module's. Drawing a panel nobody can see, or inventing
        // resistors for the module's ladder, are both worse. The plug-in's
        // own help text says so.
        ParamSpec::new("ssl_in", "Compressor In")
            .toggle()
            .default(1.0)
            .group("4000 G"),
        // Marked as the panel marks it, so more negative compresses more.
        // The engine negates it, because the sidechain gain and a
        // threshold reading run in opposite directions and SSL say so.
        ParamSpec::new("ssl_threshold", "Threshold")
            .range(-20.0, 20.0)
            .default(0.0)
            .unit("dB")
            .group("4000 G"),
        ParamSpec::new("ssl_makeup", "Make-up")
            .range(-5.0, 15.0)
            .default(0.0)
            .unit("dB")
            .group("4000 G"),
        ParamSpec::new("ssl_attack", "Attack")
            .labels(gbus::ATTACK_NAMES)
            .default(2.0)
            .group("4000 G"),
        ParamSpec::new("ssl_release", "Release")
            .labels(gbus::RELEASE_NAMES)
            .default(gbus::RELEASE_AUTO as f32)
            .group("4000 G"),
        ParamSpec::new("ssl_ratio", "Ratio")
            .labels(gbus::RATIO_NAMES)
            .default(1.0)
            .group("4000 G"),
        ParamSpec::new("ssl_hpf", "Sidechain HPF")
            .labels(gbus::HPF_NAMES)
            .default(0.0)
            .group("4000 G"),
        // Ours past the first position. The hardware has one behaviour and
        // it is `Dominant`; the other three are after the modes SSL put on
        // THE BUS+ and the extras strip says they are ours.
        ParamSpec::new("ssl_link", "Detector Link")
            .labels(gbus::LINK_NAMES)
            .default(0.0)
            .group("4000 G"),
        ParamSpec::new("ssl_drive", "Drive")
            .range(0.0, 100.0)
            .default(0.0)
            .unit("%")
            .group("4000 G"),
        ParamSpec::new("ssl_range", "Range")
            .range(0.0, 20.0)
            .default(20.0)
            .unit("dB")
            .group("4000 G"),
        ParamSpec::new("ssl_oversample", "Oversampling")
            .labels(gbus::OVERSAMPLE_NAMES)
            .default(1.0)
            .not_automatable()
            .group("4000 G"),
        // The Fairchild. Every control the hardware has twice is here
        // twice, because the 670 is two complete limiters and its two
        // channels are meant to be set differently: that is the whole point
        // of the lateral-and-vertical mode. The two screwdriver adjustments
        // on the front panel are live, and so is the DC threshold that
        // lives inside the chassis, because it is the ratio and knee
        // control (research/Fairchild-670.md 5.2) and hiding it would be
        // hiding the most interesting thing on the unit.
        ParamSpec::new("fc_model", "Unit")
            .labels(vmu::UNIT_NAMES)
            .default(vmu::MODEL_670 as f32)
            .not_automatable()
            .group("670"),
        // The step attenuator: 21 detents, 1 dB apart, printed as
        // attenuation from fully clockwise. The default is the manual's own
        // unity-gain setting, "approx 10 db attenuation".
        ParamSpec::new("fc_input_gain_l", "Left Input Gain")
            .range(0.0, vmu::INPUT_GAIN_MAX_DB)
            .steps(21)
            .default(10.0)
            .unit("dB")
            .group("670"),
        // **Panel units, not decibels.** The ring is printed 0 to 10 and
        // the pot is linear with a 24 kΩ resistor on its centre tap, so its
        // law has a kink; what it sets jointly with the DC threshold is a
        // curve rather than a point. The law lives in the engine
        // (`vmu::engine::ac_threshold_law`) and the panel prints the metal's
        // numbers.
        ParamSpec::new("fc_threshold_l", "Left Threshold")
            .range(0.0, vmu::THRESHOLD_MAX)
            .default(vmu::THRESHOLD_MAX)
            .group("670"),
        ParamSpec::new("fc_time_l", "Left Time Constant")
            .labels(vmu::TIME_NAMES)
            .default(2.0)
            .group("670"),
        // R117, the trimmer inside the chassis, as travel from fully
        // anticlockwise. The default is the factory-adjusted condition,
        // fitted to the published input/output curve 3.
        ParamSpec::new("fc_dc_threshold_l", "Left DC Threshold")
            .range(0.0, 1.0)
            .default(0.20)
            .group("670"),
        // R142, the front-panel screwdriver marked ZERO. It is a bias trim
        // wearing a meter-calibration label: moving it moves the operating
        // point of all eight 6386 sections, so it changes the standing
        // gain, the available gain reduction and the standing distortion
        // together, and it moves the needle.
        ParamSpec::new("fc_zero_l", "Left Zero")
            .range(vmu::ZERO_MIN_V, vmu::ZERO_MAX_V)
            .default(vmu::engine::V_BIAS_NOMINAL)
            .unit("V")
            .group("670"),
        ParamSpec::new("fc_balance_l", "Left Balance")
            .range(-1.0, 1.0)
            .default(0.0)
            .group("670"),
        ParamSpec::new("fc_meter_l", "Left Metering")
            .labels(vmu::METER_NAMES)
            .default(vmu::METER_ZERO as f32)
            .not_automatable()
            .group("670"),
        // The step attenuator: 21 detents, 1 dB apart, printed as
        // attenuation from fully clockwise. The default is the manual's own
        // unity-gain setting, "approx 10 db attenuation".
        ParamSpec::new("fc_input_gain_r", "Right Input Gain")
            .range(0.0, vmu::INPUT_GAIN_MAX_DB)
            .steps(21)
            .default(10.0)
            .unit("dB")
            .group("670"),
        // **Panel units, not decibels.** The ring is printed 0 to 10 and
        // the pot is linear with a 24 kΩ resistor on its centre tap, so its
        // law has a kink; what it sets jointly with the DC threshold is a
        // curve rather than a point. The law lives in the engine
        // (`vmu::engine::ac_threshold_law`) and the panel prints the metal's
        // numbers.
        ParamSpec::new("fc_threshold_r", "Right Threshold")
            .range(0.0, vmu::THRESHOLD_MAX)
            .default(vmu::THRESHOLD_MAX)
            .group("670"),
        ParamSpec::new("fc_time_r", "Right Time Constant")
            .labels(vmu::TIME_NAMES)
            .default(2.0)
            .group("670"),
        // R117, the trimmer inside the chassis, as travel from fully
        // anticlockwise. The default is the factory-adjusted condition,
        // fitted to the published input/output curve 3.
        ParamSpec::new("fc_dc_threshold_r", "Right DC Threshold")
            .range(0.0, 1.0)
            .default(0.20)
            .group("670"),
        // R142, the front-panel screwdriver marked ZERO. It is a bias trim
        // wearing a meter-calibration label: moving it moves the operating
        // point of all eight 6386 sections, so it changes the standing
        // gain, the available gain reduction and the standing distortion
        // together, and it moves the needle.
        ParamSpec::new("fc_zero_r", "Right Zero")
            .range(vmu::ZERO_MIN_V, vmu::ZERO_MAX_V)
            .default(vmu::engine::V_BIAS_NOMINAL)
            .unit("V")
            .group("670"),
        ParamSpec::new("fc_balance_r", "Right Balance")
            .range(-1.0, 1.0)
            .default(0.0)
            .group("670"),
        ParamSpec::new("fc_meter_r", "Right Metering")
            .labels(vmu::METER_NAMES)
            .default(vmu::METER_ZERO as f32)
            .not_automatable()
            .group("670"),
        // S301: ten wafers that put a sum-and-difference matrix in front of
        // both channels and another behind them. Not a stereo link — the
        // two limiters stay entirely independent and are now working on mid
        // and side.
        ParamSpec::new("fc_agc", "AGC Mode")
            .labels(vmu::AGC_NAMES)
            .default(vmu::AGC_LEFT_RIGHT as f32)
            .group("670"),
        // Ours. GE publish 4000 µmhos for the 6386 and JJ 3000 for their
        // modern replacement at the same operating point, which is a real
        // published difference of 2.5 dB rather than a flavour.
        ParamSpec::new("fc_tube", "Tube")
            .labels(vmu::TUBE_NAMES)
            .default(vmu::TUBE_GE_6386 as f32)
            .not_automatable()
            .group("670"),
        ParamSpec::new("fc_oversample", "Oversampling")
            .labels(vmu::OVERSAMPLE_NAMES)
            .default(1.0)
            .not_automatable()
            .group("670"),
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
        cl1b_gain: ix("cl1b_gain"),
        cl1b_ratio: ix("cl1b_ratio"),
        cl1b_threshold: ix("cl1b_threshold"),
        cl1b_attack: ix("cl1b_attack"),
        cl1b_release: ix("cl1b_release"),
        cl1b_mode: ix("cl1b_mode"),
        cl1b_meter: ix("cl1b_meter"),
        cl1b_bus: ix("cl1b_bus"),
        cl1b_power: ix("cl1b_power"),
        neve_model: ix("neve_model"),
        neve_limit_in: ix("neve_limit_in"),
        neve_limit_threshold: ix("neve_limit_threshold"),
        neve_limit_attack: ix("neve_limit_attack"),
        neve_limit_recovery: ix("neve_limit_recovery"),
        neve_compress_in: ix("neve_compress_in"),
        neve_compress_threshold: ix("neve_compress_threshold"),
        neve_compress_ratio: ix("neve_compress_ratio"),
        neve_compress_attack: ix("neve_compress_attack"),
        neve_compress_recovery: ix("neve_compress_recovery"),
        neve_gain: ix("neve_gain"),
        neve_meter_select: ix("neve_meter_select"),
        neve_drive: ix("neve_drive"),
        neve_power: ix("neve_power"),
        dbx_model: ix("dbx_model"),
        dbx_threshold: ix("dbx_threshold"),
        dbx_ratio: ix("dbx_ratio"),
        dbx_output: ix("dbx_output"),
        dbx_knee: ix("dbx_knee"),
        dbx_meter: ix("dbx_meter"),
        dbx_meter_cal: ix("dbx_meter_cal"),
        dbx_knee_width: ix("dbx_knee_width"),
        dbx_tau: ix("dbx_tau"),
        dbx_lookahead: ix("dbx_lookahead"),
        dbx_headroom: ix("dbx_headroom"),
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
        tg_mode: ix("tg_mode"),
        tg_recovery: ix("tg_recovery"),
        tg_output: ix("tg_output"),
        tg_hold: ix("tg_hold"),
        tg_region: ix("tg_region"),
        tg_mismatch: ix("tg_mismatch"),
        tg_input: ix("tg_input"),
        tg_drive: ix("tg_drive"),
        tg_oversample: ix("tg_oversample"),
        ssl_in: ix("ssl_in"),
        ssl_threshold: ix("ssl_threshold"),
        ssl_makeup: ix("ssl_makeup"),
        ssl_attack: ix("ssl_attack"),
        ssl_release: ix("ssl_release"),
        ssl_ratio: ix("ssl_ratio"),
        ssl_hpf: ix("ssl_hpf"),
        ssl_link: ix("ssl_link"),
        ssl_drive: ix("ssl_drive"),
        ssl_range: ix("ssl_range"),
        ssl_oversample: ix("ssl_oversample"),
        fc_model: ix("fc_model"),
        fc_input_gain_l: ix("fc_input_gain_l"),
        fc_input_gain_r: ix("fc_input_gain_r"),
        fc_threshold_l: ix("fc_threshold_l"),
        fc_threshold_r: ix("fc_threshold_r"),
        fc_time_l: ix("fc_time_l"),
        fc_time_r: ix("fc_time_r"),
        fc_dc_threshold_l: ix("fc_dc_threshold_l"),
        fc_dc_threshold_r: ix("fc_dc_threshold_r"),
        fc_zero_l: ix("fc_zero_l"),
        fc_zero_r: ix("fc_zero_r"),
        fc_balance_l: ix("fc_balance_l"),
        fc_balance_r: ix("fc_balance_r"),
        fc_meter_l: ix("fc_meter_l"),
        fc_meter_r: ix("fc_meter_r"),
        fc_agc: ix("fc_agc"),
        fc_tube: ix("fc_tube"),
        fc_oversample: ix("fc_oversample"),
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
        opto1b: opto1b::Settings {
            // The normalised value is the pot's travel, which is what the
            // engine's laws take; the plain value is the decibels or
            // seconds the panel is marked in.
            gain: audio.param_norm(ix.cl1b_gain),
            ratio: audio.param_norm(ix.cl1b_ratio),
            threshold: audio.param_norm(ix.cl1b_threshold),
            attack: audio.param_norm(ix.cl1b_attack),
            release: audio.param_norm(ix.cl1b_release),
            mode: audio.param(ix.cl1b_mode).round().clamp(0.0, 2.0) as usize,
            meter: audio.param(ix.cl1b_meter).round().clamp(0.0, 2.0) as usize,
            bus: audio.param(ix.cl1b_bus).round().clamp(0.0, 2.0) as usize,
            power: audio.param(ix.cl1b_power) >= 0.5,
            ..opto1b::Settings::default()
        },
        // Every switch here is read as an **index**, from the normalised
        // value rather than the plain one. The three with real units are
        // stepped linearly, so `norm * (steps - 1)` is the switch position
        // exactly, and the engine's own laws turn it back into decibels.
        // Going the other way, through the plain value, would mean the
        // decibel law existed twice.
        bridge: bridge::Settings {
            model: audio.param(ix.neve_model).round().clamp(0.0, 2.0) as usize,
            limit_in: audio.param(ix.neve_limit_in) >= 0.5,
            limit_threshold: (audio.param_norm(ix.neve_limit_threshold) * 22.0).round() as usize,
            limit_attack: audio.param(ix.neve_limit_attack).round().clamp(0.0, 1.0) as usize,
            limit_recovery: audio.param(ix.neve_limit_recovery).round().clamp(0.0, 5.0) as usize,
            compress_in: audio.param(ix.neve_compress_in) >= 0.5,
            compress_threshold: (audio.param_norm(ix.neve_compress_threshold) * 15.0).round()
                as usize,
            compress_ratio: audio.param(ix.neve_compress_ratio).round().clamp(0.0, 4.0) as usize,
            compress_attack: audio.param(ix.neve_compress_attack).round().clamp(0.0, 1.0) as usize,
            compress_recovery: audio
                .param(ix.neve_compress_recovery)
                .round()
                .clamp(0.0, 5.0) as usize,
            gain: (audio.param_norm(ix.neve_gain) * 10.0).round() as usize,
            meter_select: audio.param(ix.neve_meter_select).round().clamp(0.0, 2.0) as usize,
            drive: (audio.param(ix.neve_drive) / 100.0).clamp(0.0, 1.0),
            power: audio.param(ix.neve_power) >= 0.5,
            ..bridge::Settings::default()
        },
        rms: rms::Settings {
            model: audio.param(ix.dbx_model).round().clamp(0.0, 1.0) as usize,
            threshold_dbu: audio.param(ix.dbx_threshold),
            // The plain value **is** the coefficient: the taper lives in
            // the parameter, so the law exists once and the page reads α
            // from the manifest instead of reimplementing the dial.
            alpha: audio.param(ix.dbx_ratio),
            output_db: audio.param(ix.dbx_output),
            knee: audio.param(ix.dbx_knee).round().clamp(0.0, 1.0) as usize,
            knee_width_db: audio.param(ix.dbx_knee_width),
            tau_s: audio.param(ix.dbx_tau) * 1e-3,
            meter: audio.param(ix.dbx_meter).round().clamp(0.0, 2.0) as usize,
            meter_cal_dbu: audio.param(ix.dbx_meter_cal),
            lookahead_ms: audio.param(ix.dbx_lookahead),
            headroom_db: audio.param(ix.dbx_headroom),
            ..rms::Settings::default()
        },
        // Every control is a switch, so each is read as an index from the
        // normalised value. `tg_output` steps linearly over its twenty-one
        // positions, so `norm * 20` is the switch position exactly and the
        // engine turns it back into a resistance.
        tg: tg::Settings {
            mode: audio.param(ix.tg_mode).round().clamp(0.0, 2.0) as usize,
            recovery: audio.param(ix.tg_recovery).round().clamp(0.0, 5.0) as usize,
            output: (audio.param_norm(ix.tg_output) * 20.0).round() as usize,
            hold: (audio.param(ix.tg_hold) / 100.0).clamp(0.0, 1.0),
            region: audio.param(ix.tg_region).round().clamp(0.0, 1.0) as usize,
            mismatch: (audio.param(ix.tg_mismatch) / 100.0).clamp(0.0, 1.0),
            input_db: audio.param(ix.tg_input),
            drive: (audio.param(ix.tg_drive) / 100.0).clamp(0.0, 1.0),
            oversample: tg::oversample_factor(
                audio.param(ix.tg_oversample).round().clamp(0.0, 2.0) as usize,
            ),
            ..tg::Settings::default()
        },
        // Every switch here is read as an index. The two pots are linear,
        // so their plain value is the decibels the panel is marked in and
        // the engine takes it directly.
        gbus: gbus::Settings {
            sidechain_in: audio.param(ix.ssl_in) >= 0.5,
            threshold_db: audio.param(ix.ssl_threshold),
            makeup_db: audio.param(ix.ssl_makeup),
            attack: audio.param(ix.ssl_attack).round().clamp(0.0, 5.0) as usize,
            release: audio.param(ix.ssl_release).round().clamp(0.0, 4.0) as usize,
            ratio: audio.param(ix.ssl_ratio).round().clamp(0.0, 2.0) as usize,
            hpf: audio.param(ix.ssl_hpf).round().clamp(0.0, 5.0) as usize,
            link_mode: gbus::engine::Link::from_index(
                audio.param(ix.ssl_link).round().clamp(0.0, 3.0) as usize,
            ),
            drive: (audio.param(ix.ssl_drive) / 100.0).clamp(0.0, 1.0),
            range_db: audio.param(ix.ssl_range),
            oversample: audio.param(ix.ssl_oversample) >= 0.5,
            ..gbus::Settings::default()
        },
        // The Fairchild reads two of everything, because it is two
        // complete limiters. The three switches come from the normalised
        // value, which is the detent index; the rest are plain, because
        // their panel numbers are what they are.
        vmu: vmu::Settings {
            model: audio.param(ix.fc_model).round().clamp(0.0, 1.0) as usize,
            input_gain: [
                audio.param(ix.fc_input_gain_l),
                audio.param(ix.fc_input_gain_r),
            ],
            threshold: [
                audio.param(ix.fc_threshold_l),
                audio.param(ix.fc_threshold_r),
            ],
            time: [
                audio.param(ix.fc_time_l).round().clamp(0.0, 5.0) as usize,
                audio.param(ix.fc_time_r).round().clamp(0.0, 5.0) as usize,
            ],
            dc_threshold: [
                audio.param(ix.fc_dc_threshold_l),
                audio.param(ix.fc_dc_threshold_r),
            ],
            zero: [audio.param(ix.fc_zero_l), audio.param(ix.fc_zero_r)],
            balance: [audio.param(ix.fc_balance_l), audio.param(ix.fc_balance_r)],
            meter: [
                audio.param(ix.fc_meter_l).round().clamp(0.0, 2.0) as usize,
                audio.param(ix.fc_meter_r).round().clamp(0.0, 2.0) as usize,
            ],
            agc: audio.param(ix.fc_agc).round().clamp(0.0, 1.0) as usize,
            tube: audio.param(ix.fc_tube).round().clamp(0.0, 1.0) as usize,
            oversample: audio.param(ix.fc_oversample).round().clamp(0.0, 2.0) as usize,
            ..vmu::Settings::default()
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
    opto1b: opto1b::Compressor,
    bridge: bridge::Compressor,
    tg: tg::Compressor,
    gbus: gbus::Compressor,
    rms: rms::Compressor,
    vmu: vmu::Compressor,
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
            opto1b: opto1b::Compressor::new(sr),
            bridge: bridge::Compressor::new(sr),
            tg: tg::Compressor::new(sr),
            gbus: gbus::Compressor::new(sr),
            rms: rms::Compressor::new(sr),
            vmu: vmu::Compressor::new(sr),
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
        self.opto1b.set_sample_rate(sr);
        self.bridge.set_sample_rate(sr);
        self.tg.set_sample_rate(sr);
        self.gbus.set_sample_rate(sr);
        self.rms.set_sample_rate(sr);
        self.vmu.set_sample_rate(sr);
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
        self.opto1b.reset();
        self.bridge.reset();
        self.rms.reset();
        self.tg.reset();
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
            Model::Opto1b => self.opto1b.latency(),
            Model::Bridge => self.bridge.latency(),
            Model::Tg => self.tg.latency(),
            // Zero at 1x: the audio path never touches the detector, so
            // only the resampler's round trip can cost anything.
            Model::Gbus => self.gbus.latency(),
            Model::Rms => self.rms.latency(),
            Model::Vmu => self.vmu.latency(),
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
                Model::Opto1b => self.opto1b.reset(),
                Model::Bridge => self.bridge.reset(),
                Model::Tg => self.tg.reset(),
                Model::Gbus => self.gbus.reset(),
                Model::Rms => self.rms.reset(),
                Model::Vmu => self.vmu.reset(),
            }
            changed = true;
        }
        changed |= self.fet.configure(&Self::fet_settings(s));
        changed |= self.opto.configure(s.opto);
        changed |= self.opto3.configure(s.opto3);
        changed |= self.vca.configure(&s.vca);
        changed |= self.pre.configure(&s.pre);
        changed |= self.opto1b.configure(s.opto1b);
        changed |= self.bridge.configure(s.bridge);
        changed |= self.tg.configure(s.tg);
        changed |= self.rms.configure(s.rms);
        changed |= self.vmu.configure(s.vmu);
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
            Model::Opto1b => self.opto1b.process_block(l, r),
            Model::Bridge => self.bridge.process_block(l, r),
            Model::Tg => self.tg.process_block(l, r),
            Model::Gbus => self.gbus.process_block(l, r),
            Model::Rms => self.rms.process_block(l, r),
            Model::Vmu => self.vmu.process_block(l, r),
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
            Model::Opto1b => {
                let f = self.opto1b.meter_frame();
                self.gr_db = -f[4];
                f[5]
            }
            Model::Bridge => {
                let f = self.bridge.meter_frame();
                self.gr_db = -f[4];
                f[5]
            }
            Model::Tg => {
                let f = self.tg.meter_frame();
                self.gr_db = -f[4];
                f[5]
            }
            Model::Rms => {
                let f = self.rms.meter_frame();
                self.gr_db = -f[4];
                f[5]
            }
            // The meter is a plate-current bridge across the output stage,
            // not a gain-reduction movement: what the needle shows is how
            // far the current has moved from its standing value, which is
            // why the ZERO screw moves it too.
            Model::Vmu => {
                let f = self.vmu.meter_frame();
                self.gr_db = -f[4];
                f[5]
            }
            Model::Gbus => {
                let f = self.gbus.meter_frame();
                self.gr_db = -f[4];
                // The needle's target, on a linear 0 to 20 dB scale. The
                // movement itself runs in `vu` just below, so the page
                // must not smooth it again.
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
            // Not the T4's three states: this machine has none. It
            // publishes the control voltage its electronics hold, the
            // resistance the element presents, and the drive fraction.
            Model::Opto1b => self.opto1b.cell_state(),
            // Nor has this one. It publishes the control current in
            // microamps, the resistance the element presents and the
            // drive fraction, which are the three quantities the whole
            // circuit is actually about.
            Model::Tg => self.tg.cell_state(),
            // Not a photocell: the three timing capacitors of the
            // Fairchild's network, which positions 5 and 6 are
            // incomprehensible without.
            Model::Vmu => self.vmu.cell_state(),
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
            // The same four slots, saying something else: the dbx's
            // two threshold indicators, the peak-detector ghost the
            // page draws behind its gain reduction, and its OverEasy
            // indicator. The `cell` stream already carries two
            // meanings for the same reason.
            Model::Rms => self.rms.lamps_frame(),
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
            Model::Opto1b => self
                .opto1b
                .transfer_curve(out, TRANSFER_MIN_DB, TRANSFER_MAX_DB),
            Model::Bridge => self
                .bridge
                .transfer_curve(out, TRANSFER_MIN_DB, TRANSFER_MAX_DB),
            Model::Tg => self
                .tg
                .transfer_curve(out, TRANSFER_MIN_DB, TRANSFER_MAX_DB),
            Model::Rms => self
                .rms
                .transfer_curve(out, TRANSFER_MIN_DB, TRANSFER_MAX_DB),
            Model::Gbus => self
                .gbus
                .transfer_curve(out, TRANSFER_MIN_DB, TRANSFER_MAX_DB),
            Model::Vmu => self
                .vmu
                .transfer_curve(out, TRANSFER_MIN_DB, TRANSFER_MAX_DB),
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
            Model::Opto | Model::Opto3 | Model::Opto1b | Model::Tg | Model::Vmu => {
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
            Model::Vca | Model::Pre6176 | Model::Rms => {
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

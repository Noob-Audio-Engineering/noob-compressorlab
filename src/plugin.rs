//! The nih-plug plug-in: VST3 + CLAP, stereo in / stereo out. Its editor is
//! the OS web view showing the Vue SPA from `web/dist`, embedded in the
//! binary.
//!
//! How the pieces connect:
//!
//! * The parameters are nih-plug parameters with the same ids as the
//!   standalone's specs (`dsp::param_specs`), mirrored into the bridge by
//!   [`NoobVstWebguiFrameworkEditor::with_builder`], so the same page drives both.
//!   The `model` parameter is one of them, so the model an instance is set
//!   to is saved with the project.
//! * `process` reads a [`Settings`] snapshot from the nih-plug values,
//!   configures the [`Processor`], runs the block, and publishes the
//!   streams through the audio handle.
//! * The active model's latency (the 1176's oversampler) is reported to the
//!   host and updated when the model changes.
//! * The page's UI store (presets, window size) is persisted with the
//!   plug-in state by [`NoobCompressorLabParams::ui_store`], a `StoreSlot`.

use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::sync::Arc;

use include_dir::{Dir, include_dir};
use nih_plug::prelude::*;
use noob_vst_webgui_framework::{Assets, AudioHandle, NoobVstWebguiFramework};
use noob_vst_webgui_framework_nih::{EditorConfig, NoobVstWebguiFrameworkEditor, StoreSlot};

use crate::dsp::{self, Model, Processor, Settings, Shared, fet, opto, opto1b, opto3, pre, vca};

static UI: Dir = include_dir!("$CARGO_MANIFEST_DIR/web/dist");

fn ui_lookup(path: &str) -> Option<&'static [u8]> {
    UI.get_file(path).map(|f| f.contents())
}

/// Which compressor the instance is.
#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ModelParam {
    #[name = "1176"]
    Fet,
    #[name = "LA-2A"]
    Opto,
    #[name = "LA-3A"]
    Opto3,
    #[name = "Distressor"]
    Vca,
    #[name = "6176"]
    Pre6176,
    #[name = "CL-1B"]
    Opto1b,
}

/// The 1176's ratio buttons, as the host sees them.
#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum RatioParam {
    #[name = "4"]
    R4,
    #[name = "8"]
    R8,
    #[name = "12"]
    R12,
    #[name = "20"]
    R20,
    #[name = "All"]
    All,
}

/// The 1176's meter switch.
#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum FetMeterParam {
    #[name = "GR"]
    Gr,
    #[name = "+4"]
    Plus4,
    #[name = "+8"]
    Plus8,
    #[name = "Off"]
    Off,
}

/// The 1176's circuit revision (see [`fet::Revision`]); the index matches
/// the page's labels.
#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum RevisionParam {
    #[name = "A"]
    A,
    #[name = "B"]
    B,
    #[name = "C"]
    C,
    #[name = "D"]
    D,
    #[name = "E"]
    E,
    #[name = "F"]
    F,
    #[name = "G"]
    G,
    #[name = "H"]
    H,
    #[name = "LN"]
    Ln,
}

/// The LA-2A's Limit / Compress switch.
#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ModeParam {
    Compress,
    Limit,
}

/// What the LA-2A's panel meter shows.
#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum OptoMeterParam {
    #[name = "Gain Reduction"]
    GainReduction,
    #[name = "Output +10"]
    Output10,
    #[name = "Output +4"]
    Output4,
}

/// The LA-2A's photocell speed variant.
#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum CellParam {
    Silver,
    Gray,
    #[name = "LA-2"]
    La2,
}

/// What the LA-3A's panel meter shows. The hardware toggle has two
/// positions; `Off` is the plug-in's own, as on Universal Audio's.
#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum La3aMeterParam {
    #[name = "Gain Reduction"]
    GainReduction,
    Output,
    Off,
}

/// The CL 1B's attack/release select, in the panel's own left-to-right
/// order.
#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cl1bModeParam {
    Fixed,
    #[name = "Fix/Man"]
    FixMan,
    Manual,
}

/// What the CL 1B's meter shows. Three positions on the hardware, and no
/// Off among them.
#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cl1bMeterParam {
    Input,
    Compression,
    Output,
}

/// The CL 1B's side-chain bus select. On the hardware it chooses which of
/// two busses the unit joins; here it picks the stereo link group.
#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cl1bBusParam {
    Off,
    #[name = "1"]
    One,
    #[name = "2"]
    Two,
}

/// The Distressor's ratio switch.
#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum DistRatioParam {
    #[name = "1:1"]
    R1,
    #[name = "2:1"]
    R2,
    #[name = "3:1"]
    R3,
    #[name = "4:1"]
    R4,
    #[name = "6:1"]
    R6,
    #[name = "10:1"]
    R10,
    #[name = "20:1"]
    R20,
    Nuke,
}

/// The Distressor's Detector switch.
#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum DistDetectorParam {
    Norm,
    #[name = "HP"]
    Hp,
    Band,
    #[name = "HP+Band"]
    HpBand,
}

/// The Distressor's Audio switch.
#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum DistAudioParam {
    Norm,
    #[name = "HP"]
    Hp,
    #[name = "Dist 2"]
    Dist2,
    #[name = "Dist 3"]
    Dist3,
    #[name = "HP+Dist 2"]
    HpDist2,
    #[name = "HP+Dist 3"]
    HpDist3,
}

/// How the Distressor ties the two channels together.
#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum DistLinkParam {
    Phase,
    Image,
    Both,
}

/// The 6176's Ratio switch positions that are not ratios.
#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum PreRoutingParam {
    Join,
    #[name = "BP"]
    Bypass,
    #[name = "1:1"]
    Unity,
}

/// The 610's stepped Gain switch.
#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum PreGainParam {
    #[name = "-10"]
    Minus10,
    #[name = "-5"]
    Minus5,
    #[name = "0"]
    Zero,
    #[name = "+5"]
    Plus5,
    #[name = "+10"]
    Plus10,
}

/// The 610's input select.
#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum PreInputParam {
    Line,
    #[name = "Mic 500"]
    Mic500,
    #[name = "Mic 2.0K"]
    Mic2k,
    #[name = "Hi-Z 47K"]
    HiZ47k,
    #[name = "Hi-Z 2.2M"]
    HiZ22m,
}

/// The 610's low shelf corner.
#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum PreLfFreqParam {
    #[name = "70"]
    F70,
    #[name = "100"]
    F100,
    #[name = "200"]
    F200,
}

/// The 610's high shelf corner.
#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum PreHfFreqParam {
    #[name = "4.5k"]
    F4k5,
    #[name = "7k"]
    F7k,
    #[name = "10k"]
    F10k,
}

/// Either shelf's eleven-position gain switch.
#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum PreShelfGainParam {
    #[name = "-9"]
    M9,
    #[name = "-6"]
    M6,
    #[name = "-4.5"]
    M45,
    #[name = "-3"]
    M3,
    #[name = "-1.5"]
    M15,
    #[name = "0"]
    Zero,
    #[name = "+1.5"]
    P15,
    #[name = "+3"]
    P3,
    #[name = "+4.5"]
    P45,
    #[name = "+6"]
    P6,
    #[name = "+9"]
    P9,
}

/// Which 610 the preamp is.
#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum PreVoiceParam {
    #[name = "610B"]
    B,
    #[name = "610A"]
    A,
}

/// The 1176 section's rear input loading.
#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum PreLoadParam {
    #[name = "15k"]
    K15,
    #[name = "600"]
    R600,
}

/// What the 6176's meter switch shows.
#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum PreMeterParam {
    #[name = "PRE"]
    Pre,
    #[name = "GR"]
    Gr,
    #[name = "COMP"]
    Comp,
}

/// Every host parameter. Ids in `param_map` match the standalone and the
/// page.
pub struct NoobCompressorLabParams {
    pub model: EnumParam<ModelParam>,
    /// 1176 input mark, 0..48 (mark − 48 dB).
    pub fet_input: FloatParam,
    /// 1176 output mark, 0..48.
    pub fet_output: FloatParam,
    /// 1176 attack knob, 0 (OFF) or 1..7.
    pub fet_attack: FloatParam,
    /// 1176 release knob, 1..7.
    pub fet_release: FloatParam,
    pub fet_ratio: EnumParam<RatioParam>,
    pub fet_meter: EnumParam<FetMeterParam>,
    pub fet_revision: EnumParam<RevisionParam>,
    /// LA-2A make-up gain, 0..100 (unity at 32).
    pub opto_gain: FloatParam,
    /// LA-2A sidechain drive, 0..100.
    pub opto_peak_reduction: FloatParam,
    pub opto_mode: EnumParam<ModeParam>,
    pub opto_meter: EnumParam<OptoMeterParam>,
    /// LA-2A R37, 0 (10 dB less low-frequency sensitivity) .. 1 (flat).
    pub opto_emphasis: FloatParam,
    pub opto_cell: EnumParam<CellParam>,
    /// LA-2A meter-zero trim, ±2 dB. Moves the needle, not the audio.
    pub opto_meter_zero: FloatParam,
    /// LA-3A make-up gain, 0..100.
    pub la3a_gain: FloatParam,
    /// LA-3A sidechain drive, 0..100.
    pub la3a_peak_reduction: FloatParam,
    pub la3a_mode: EnumParam<ModeParam>,
    pub la3a_meter: EnumParam<La3aMeterParam>,
    /// LA-3A HF Contour, 0 (flat, as the trimmer ships) .. 1.
    pub la3a_emphasis: FloatParam,
    /// LA-3A cell age: a depleted T4 compresses far less.
    pub la3a_cell: EnumParam<CellParam>,
    /// The CL 1B's five knobs. All 0..1, because the panel prints scales
    /// rather than numbers and the taper is the pot's own: Gain and
    /// Threshold share one log law, Ratio and Release are linear, Attack
    /// is logarithmic.
    pub cl1b_gain: FloatParam,
    pub cl1b_ratio: FloatParam,
    pub cl1b_threshold: FloatParam,
    pub cl1b_attack: FloatParam,
    pub cl1b_release: FloatParam,
    pub cl1b_mode: EnumParam<Cl1bModeParam>,
    pub cl1b_meter: EnumParam<Cl1bMeterParam>,
    pub cl1b_bus: EnumParam<Cl1bBusParam>,
    /// The panel's OFF/ON mains knob. Parks the machine rather than
    /// silencing it, as the 1176's METER OFF does in this same plug-in.
    pub cl1b_power: BoolParam,
    /// Distressor knobs, 0..10.5.
    pub dist_input: FloatParam,
    pub dist_output: FloatParam,
    pub dist_attack: FloatParam,
    pub dist_release: FloatParam,
    pub dist_ratio: EnumParam<DistRatioParam>,
    pub dist_detector: EnumParam<DistDetectorParam>,
    pub dist_audio: EnumParam<DistAudioParam>,
    pub dist_british: BoolParam,
    pub dist_link_mode: EnumParam<DistLinkParam>,
    /// Distressor internal reference level, dB.
    pub dist_headroom: FloatParam,
    pub pre_join: EnumParam<PreRoutingParam>,
    pub pre_gain: EnumParam<PreGainParam>,
    pub pre_input: EnumParam<PreInputParam>,
    pub pre_pad: BoolParam,
    pub pre_polarity: BoolParam,
    /// 610 Level knob, 0..10.
    pub pre_level: FloatParam,
    pub pre_lf_freq: EnumParam<PreLfFreqParam>,
    pub pre_lf_gain: EnumParam<PreShelfGainParam>,
    pub pre_hf_freq: EnumParam<PreHfFreqParam>,
    pub pre_hf_gain: EnumParam<PreShelfGainParam>,
    pub pre_hpf: BoolParam,
    pub pre_voice: EnumParam<PreVoiceParam>,
    pub pre_load: EnumParam<PreLoadParam>,
    pub pre_meter: EnumParam<PreMeterParam>,
    /// +48 V. Panel state only; see `pre::Settings::phantom`.
    pub pre_phantom: BoolParam,
    pub link: BoolParam,
    /// Wet share, %.
    pub mix: FloatParam,
    /// Side-chain high-pass corner, Hz (0 = off).
    pub sc_hpf: FloatParam,
    pub bypass: BoolParam,
    /// The page's presets and window size; not parameters, but saved with
    /// the state.
    pub ui_store: StoreSlot,
}

impl Default for NoobCompressorLabParams {
    fn default() -> Self {
        let mark = |name: &str| {
            FloatParam::new(
                name,
                24.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: fet::MARK_MAX,
                },
            )
            .with_step_size(0.1)
        };
        let knob = |name: &str| {
            FloatParam::new(
                name,
                5.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: vca::KNOB_MAX,
                },
            )
            .with_step_size(0.1)
        };
        // The CL 1B's knobs are 0..1: its panel prints scales, not
        // numbers, and each pot has its own taper.
        let unit_knob = |name: &str, default: f32| {
            FloatParam::new(name, default, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_step_size(0.001)
        };
        let percent = |name: &str, default: f32| {
            FloatParam::new(
                name,
                default,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
            .with_step_size(0.1)
        };
        NoobCompressorLabParams {
            model: EnumParam::new("Model", ModelParam::Fet).non_automatable(),
            fet_input: mark("Input"),
            fet_output: mark("Output"),
            fet_attack: FloatParam::new(
                "Attack",
                4.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: fet::ATTACK_MAX,
                },
            )
            .with_step_size(0.1),
            fet_release: FloatParam::new(
                "Release",
                4.0,
                FloatRange::Linear {
                    min: 1.0,
                    max: fet::RELEASE_MAX,
                },
            )
            .with_step_size(0.1),
            fet_ratio: EnumParam::new("Ratio", RatioParam::R4),
            fet_meter: EnumParam::new("Meter", FetMeterParam::Gr).non_automatable(),
            fet_revision: EnumParam::new("Revision", RevisionParam::Ln).non_automatable(),
            opto_gain: percent("Gain", 32.0),
            opto_peak_reduction: percent("Peak Reduction", 40.0),
            opto_mode: EnumParam::new("Mode", ModeParam::Compress),
            opto_meter: EnumParam::new("Meter", OptoMeterParam::GainReduction).non_automatable(),
            opto_emphasis: FloatParam::new(
                "Emphasis (R37)",
                1.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_step_size(0.01),
            opto_cell: EnumParam::new("Cell", CellParam::Gray).non_automatable(),
            opto_meter_zero: FloatParam::new(
                "Meter Zero",
                0.0,
                FloatRange::Linear {
                    min: -2.0,
                    max: 2.0,
                },
            )
            .with_unit(" dB")
            .with_step_size(0.05)
            .non_automatable(),
            la3a_gain: percent("Gain", 32.0),
            la3a_peak_reduction: percent("Peak Reduction", 40.0),
            la3a_mode: EnumParam::new("Mode", ModeParam::Compress),
            la3a_meter: EnumParam::new("Meter", La3aMeterParam::GainReduction).non_automatable(),
            la3a_emphasis: FloatParam::new(
                "HF Contour",
                0.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_step_size(0.01),
            la3a_cell: EnumParam::new("Cell", CellParam::Silver).non_automatable(),
            // The host sees the panel's own units too, formatted by the
            // same engine functions the manifest's tables are sampled
            // from, so there is still exactly one copy of each pot law.
            cl1b_gain: unit_knob("Gain", 0.265).with_value_to_string(Arc::new(|p| {
                let db = opto1b::engine::gain_db(p);
                if db <= -60.0 {
                    "off".to_string()
                } else {
                    format!("{db:.1} dB")
                }
            })),
            cl1b_ratio: unit_knob("Ratio", 0.375),
            cl1b_threshold: unit_knob("Threshold", 0.5).with_value_to_string(Arc::new(|p| {
                format!("{:.1} dBu", opto1b::engine::threshold_dbu(p))
            })),
            cl1b_attack: unit_knob("Attack", 0.75).with_value_to_string(Arc::new(|p| {
                format!("{:.2} ms", opto1b::engine::attack_s(p) * 1e3)
            })),
            cl1b_release: unit_knob("Release", 0.25).with_value_to_string(Arc::new(|p| {
                format!("{:.2} s", opto1b::engine::release_s(p))
            })),
            cl1b_mode: EnumParam::new("Attack/Release Select", Cl1bModeParam::Manual),
            cl1b_meter: EnumParam::new("Meter", Cl1bMeterParam::Compression).non_automatable(),
            cl1b_bus: EnumParam::new("Sidechain Bus", Cl1bBusParam::Off),
            cl1b_power: BoolParam::new("Power", true).non_automatable(),
            dist_input: knob("Input"),
            dist_output: knob("Output"),
            dist_attack: knob("Attack"),
            dist_release: knob("Release"),
            dist_ratio: EnumParam::new("Ratio", DistRatioParam::R6),
            dist_detector: EnumParam::new("Detector", DistDetectorParam::Norm),
            dist_audio: EnumParam::new("Audio", DistAudioParam::Norm),
            dist_british: BoolParam::new("British Mode", false),
            dist_link_mode: EnumParam::new("Link Mode", DistLinkParam::Phase).non_automatable(),
            dist_headroom: FloatParam::new(
                "Headroom",
                vca::HEADROOM_DEFAULT_DB,
                FloatRange::Linear {
                    min: vca::HEADROOM_MIN_DB,
                    max: vca::HEADROOM_MAX_DB,
                },
            )
            .with_unit(" dB")
            .with_step_size(0.5)
            .non_automatable(),
            pre_join: EnumParam::new("Routing", PreRoutingParam::Join),
            pre_gain: EnumParam::new("Gain", PreGainParam::Zero),
            pre_input: EnumParam::new("Input", PreInputParam::Line).non_automatable(),
            pre_pad: BoolParam::new("Pad", false),
            pre_polarity: BoolParam::new("Polarity", false),
            pre_level: FloatParam::new(
                "Level",
                7.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: pre::LEVEL_MAX,
                },
            )
            .with_step_size(0.1),
            pre_lf_freq: EnumParam::new("Low Freq", PreLfFreqParam::F100),
            pre_lf_gain: EnumParam::new("Low", PreShelfGainParam::Zero),
            pre_hf_freq: EnumParam::new("High Freq", PreHfFreqParam::F10k),
            pre_hf_gain: EnumParam::new("High", PreShelfGainParam::Zero),
            pre_hpf: BoolParam::new("Low Cut", false),
            pre_voice: EnumParam::new("Voicing", PreVoiceParam::B).non_automatable(),
            pre_load: EnumParam::new("Output Load", PreLoadParam::K15).non_automatable(),
            pre_meter: EnumParam::new("Meter", PreMeterParam::Gr).non_automatable(),
            pre_phantom: BoolParam::new("+48 V", false),
            link: BoolParam::new("Stereo Link", true),
            mix: percent("Mix", 100.0).with_unit(" %").with_step_size(1.0),
            sc_hpf: FloatParam::new(
                "Side-chain HPF",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: dsp::SC_HPF_MAX_HZ,
                },
            )
            .with_unit(" Hz")
            .with_step_size(1.0),
            bypass: BoolParam::new("Bypass", false)
                .with_value_to_string(formatters::v2s_bool_bypass()),
            ui_store: StoreSlot::new(),
        }
    }
}

// SAFETY: every pointer comes from a field of `self`, which nih-plug keeps
// alive in an `Arc` for the plug-in's whole life. Written by hand so the
// ids match the standalone and the page.
unsafe impl Params for NoobCompressorLabParams {
    fn param_map(&self) -> Vec<(String, ParamPtr, String)> {
        let g = |s: &str| s.to_string();
        vec![
            (g("model"), self.model.as_ptr(), g("lab")),
            (g("fet_input"), self.fet_input.as_ptr(), g("1176")),
            (g("fet_output"), self.fet_output.as_ptr(), g("1176")),
            (g("fet_attack"), self.fet_attack.as_ptr(), g("1176")),
            (g("fet_release"), self.fet_release.as_ptr(), g("1176")),
            (g("fet_ratio"), self.fet_ratio.as_ptr(), g("1176")),
            (g("fet_meter"), self.fet_meter.as_ptr(), g("1176")),
            (g("fet_revision"), self.fet_revision.as_ptr(), g("1176")),
            (g("opto_gain"), self.opto_gain.as_ptr(), g("LA-2A")),
            (
                g("opto_peak_reduction"),
                self.opto_peak_reduction.as_ptr(),
                g("LA-2A"),
            ),
            (g("opto_mode"), self.opto_mode.as_ptr(), g("LA-2A")),
            (g("opto_meter"), self.opto_meter.as_ptr(), g("LA-2A")),
            (g("opto_emphasis"), self.opto_emphasis.as_ptr(), g("LA-2A")),
            (g("opto_cell"), self.opto_cell.as_ptr(), g("LA-2A")),
            (
                g("opto_meter_zero"),
                self.opto_meter_zero.as_ptr(),
                g("LA-2A"),
            ),
            (g("la3a_gain"), self.la3a_gain.as_ptr(), g("LA-3A")),
            (
                g("la3a_peak_reduction"),
                self.la3a_peak_reduction.as_ptr(),
                g("LA-3A"),
            ),
            (g("la3a_mode"), self.la3a_mode.as_ptr(), g("LA-3A")),
            (g("la3a_meter"), self.la3a_meter.as_ptr(), g("LA-3A")),
            (g("la3a_emphasis"), self.la3a_emphasis.as_ptr(), g("LA-3A")),
            (g("la3a_cell"), self.la3a_cell.as_ptr(), g("LA-3A")),
            (g("dist_input"), self.dist_input.as_ptr(), g("Distressor")),
            (g("dist_output"), self.dist_output.as_ptr(), g("Distressor")),
            (g("dist_attack"), self.dist_attack.as_ptr(), g("Distressor")),
            (
                g("dist_release"),
                self.dist_release.as_ptr(),
                g("Distressor"),
            ),
            (g("dist_ratio"), self.dist_ratio.as_ptr(), g("Distressor")),
            (
                g("dist_detector"),
                self.dist_detector.as_ptr(),
                g("Distressor"),
            ),
            (g("dist_audio"), self.dist_audio.as_ptr(), g("Distressor")),
            (
                g("dist_british"),
                self.dist_british.as_ptr(),
                g("Distressor"),
            ),
            (
                g("dist_link_mode"),
                self.dist_link_mode.as_ptr(),
                g("Distressor"),
            ),
            (
                g("dist_headroom"),
                self.dist_headroom.as_ptr(),
                g("Distressor"),
            ),
            (g("pre_join"), self.pre_join.as_ptr(), g("6176")),
            (g("pre_gain"), self.pre_gain.as_ptr(), g("6176")),
            (g("pre_input"), self.pre_input.as_ptr(), g("6176")),
            (g("pre_pad"), self.pre_pad.as_ptr(), g("6176")),
            (g("pre_polarity"), self.pre_polarity.as_ptr(), g("6176")),
            (g("pre_level"), self.pre_level.as_ptr(), g("6176")),
            (g("pre_lf_freq"), self.pre_lf_freq.as_ptr(), g("6176")),
            (g("pre_lf_gain"), self.pre_lf_gain.as_ptr(), g("6176")),
            (g("pre_hf_freq"), self.pre_hf_freq.as_ptr(), g("6176")),
            (g("pre_hf_gain"), self.pre_hf_gain.as_ptr(), g("6176")),
            (g("pre_hpf"), self.pre_hpf.as_ptr(), g("6176")),
            (g("pre_voice"), self.pre_voice.as_ptr(), g("6176")),
            (g("pre_load"), self.pre_load.as_ptr(), g("6176")),
            (g("pre_meter"), self.pre_meter.as_ptr(), g("6176")),
            (g("pre_phantom"), self.pre_phantom.as_ptr(), g("6176")),
            (g("cl1b_gain"), self.cl1b_gain.as_ptr(), g("CL-1B")),
            (g("cl1b_ratio"), self.cl1b_ratio.as_ptr(), g("CL-1B")),
            (
                g("cl1b_threshold"),
                self.cl1b_threshold.as_ptr(),
                g("CL-1B"),
            ),
            (g("cl1b_attack"), self.cl1b_attack.as_ptr(), g("CL-1B")),
            (g("cl1b_release"), self.cl1b_release.as_ptr(), g("CL-1B")),
            (g("cl1b_mode"), self.cl1b_mode.as_ptr(), g("CL-1B")),
            (g("cl1b_meter"), self.cl1b_meter.as_ptr(), g("CL-1B")),
            (g("cl1b_bus"), self.cl1b_bus.as_ptr(), g("CL-1B")),
            (g("cl1b_power"), self.cl1b_power.as_ptr(), g("CL-1B")),
            (g("link"), self.link.as_ptr(), g("extras")),
            (g("mix"), self.mix.as_ptr(), g("extras")),
            (g("sc_hpf"), self.sc_hpf.as_ptr(), g("extras")),
            (g("bypass"), self.bypass.as_ptr(), g("extras")),
        ]
    }

    fn serialize_fields(&self) -> BTreeMap<String, String> {
        let mut m = BTreeMap::new();
        self.ui_store.serialize_into(&mut m);
        m
    }

    fn deserialize_fields(&self, serialized: &BTreeMap<String, String>) {
        self.ui_store.deserialize_from(serialized);
    }
}

impl NoobCompressorLabParams {
    /// The processor settings for the current values.
    fn settings(&self) -> Settings {
        Settings {
            model: Model::from_index(self.model.value() as usize),
            fet: fet::Settings {
                input: self.fet_input.value(),
                output: self.fet_output.value(),
                attack: self.fet_attack.value(),
                release: self.fet_release.value(),
                ratio: fet::Ratio::from_index(self.fet_ratio.value() as usize),
                meter: fet::MeterMode::from_index(self.fet_meter.value() as usize),
                revision: fet::Revision::from_index(self.fet_revision.value() as usize),
                ..fet::Settings::default()
            },
            opto: opto::Settings {
                gain: self.opto_gain.value(),
                peak_reduction: self.opto_peak_reduction.value(),
                limit: self.opto_mode.value() == ModeParam::Limit,
                meter: self.opto_meter.value() as usize,
                emphasis: self.opto_emphasis.value(),
                cell: self.opto_cell.value() as usize,
                meter_zero: self.opto_meter_zero.value(),
                ..opto::Settings::default()
            },
            opto3: opto3::Settings {
                gain: self.la3a_gain.value(),
                peak_reduction: self.la3a_peak_reduction.value(),
                limit: self.la3a_mode.value() == ModeParam::Limit,
                meter: self.la3a_meter.value() as usize,
                emphasis: self.la3a_emphasis.value(),
                cell: self.la3a_cell.value() as usize,
                ..opto3::Settings::default()
            },
            opto1b: opto1b::Settings {
                gain: self.cl1b_gain.value(),
                ratio: self.cl1b_ratio.value(),
                threshold: self.cl1b_threshold.value(),
                attack: self.cl1b_attack.value(),
                release: self.cl1b_release.value(),
                mode: self.cl1b_mode.value() as usize,
                meter: self.cl1b_meter.value() as usize,
                bus: self.cl1b_bus.value() as usize,
                power: self.cl1b_power.value(),
                ..opto1b::Settings::default()
            },
            vca: vca::Settings {
                input: self.dist_input.value(),
                output: self.dist_output.value(),
                attack: self.dist_attack.value(),
                release: self.dist_release.value(),
                ratio: vca::Ratio::from_index(self.dist_ratio.value() as usize),
                detector: vca::Detector::from_index(self.dist_detector.value() as usize),
                audio: vca::AudioMode::from_index(self.dist_audio.value() as usize),
                british: self.dist_british.value(),
                link_mode: vca::LinkMode::from_index(self.dist_link_mode.value() as usize),
                headroom_db: self.dist_headroom.value(),
                ..vca::Settings::default()
            },
            pre: pre::Settings {
                routing: pre::Routing::from_index(self.pre_join.value() as usize),
                gain: self.pre_gain.value() as usize,
                input: self.pre_input.value() as usize,
                pad: self.pre_pad.value(),
                polarity: self.pre_polarity.value(),
                level: self.pre_level.value(),
                lf_freq: self.pre_lf_freq.value() as usize,
                lf_gain: self.pre_lf_gain.value() as usize,
                hf_freq: self.pre_hf_freq.value() as usize,
                hf_gain: self.pre_hf_gain.value() as usize,
                hpf: self.pre_hpf.value(),
                voice: self.pre_voice.value() as usize,
                load: self.pre_load.value() as usize,
                meter: self.pre_meter.value() as usize,
                phantom: self.pre_phantom.value(),
                bypass: false,
            },
        }
        .with_shared(Shared {
            link: self.link.value(),
            mix: self.mix.value() / 100.0,
            sc_hpf_hz: self.sc_hpf.value(),
            bypass: self.bypass.value(),
        })
    }
}

/// The plug-in.
pub struct NoobCompressorLab {
    params: Arc<NoobCompressorLabParams>,
    editor: Arc<NoobVstWebguiFrameworkEditor>,
    bridge: NoobVstWebguiFramework,
    audio: Option<AudioHandle>,
    processor: Processor,
    last_latency: usize,
}

impl Default for NoobCompressorLab {
    fn default() -> Self {
        let params = Arc::new(NoobCompressorLabParams::default());
        let (editor, bridge) = NoobVstWebguiFrameworkEditor::with_builder(
            "noob-compressorlab",
            params.as_ref(),
            dsp::streams(48_000.0),
            EditorConfig::new(1100, 620)
                .size_limits((900, 520), (7680, 4320))
                .assets(Assets::Lookup(ui_lookup)),
            |b| {
                b.meta(serde_json::json!({
                    "vendor": "Noob Audio Engineering",
                    "version": env!("CARGO_PKG_VERSION"),
                    "sample_rate": 48_000.0,
                    "vu_ref_dbfs": dsp::VU_REF_DBFS,
                    "standalone": false,
                    "transfer_points": dsp::TRANSFER_POINTS,
                }))
            },
        );
        let audio = bridge.take_audio();
        params.ui_store.attach(&bridge);
        NoobCompressorLab {
            params,
            editor,
            bridge,
            audio,
            processor: Processor::new(48_000.0),
            last_latency: usize::MAX,
        }
    }
}

impl Plugin for NoobCompressorLab {
    const NAME: &'static str = "Noob CompressorLab";
    const VENDOR: &'static str = "Noob Audio Engineering";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    const SAMPLE_ACCURATE_AUTOMATION: bool = false;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        Some(Box::new(self.editor.handle()))
    }

    fn initialize(
        &mut self,
        _layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        context: &mut impl InitContext<Self>,
    ) -> bool {
        self.processor.set_sample_rate(buffer_config.sample_rate);
        self.processor.configure(&self.params.settings());
        self.last_latency = self.processor.latency();
        context.set_latency_samples(self.last_latency as u32);
        self.bridge.send_json(
            "sample_rate",
            serde_json::json!({ "sample_rate": buffer_config.sample_rate }),
        );
        true
    }

    fn reset(&mut self) {
        self.processor.reset();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        self.processor.configure(&self.params.settings());
        let latency = self.processor.latency();
        if latency != self.last_latency {
            self.last_latency = latency;
            context.set_latency_samples(latency as u32);
        }
        let channels = buffer.channels();
        let slices = buffer.as_slice();
        if channels >= 2 {
            let (a, b) = slices.split_at_mut(1);
            self.processor.process(&mut *a[0], &mut *b[0]);
        } else if channels == 1 {
            // Mono: process the one channel against a copy of itself.
            let l = &mut *slices[0];
            let mut r = [0.0f32; 4096];
            let n = l.len().min(r.len());
            r[..n].copy_from_slice(&l[..n]);
            self.processor.process(&mut l[..n], &mut r[..n]);
        }
        if let Some(audio) = self.audio.as_mut() {
            self.processor.publish(audio);
        }
        ProcessStatus::Normal
    }
}

impl Vst3Plugin for NoobCompressorLab {
    const VST3_CLASS_ID: [u8; 16] = *b"NoobCompLabVst3W";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

impl ClapPlugin for NoobCompressorLab {
    const CLAP_ID: &'static str = "io.github.noob-audio-engineering.noob-compressorlab";
    const CLAP_DESCRIPTION: Option<&'static str> = Some(
        "1176-style FET and LA-2A-style optical compressors in one, with a web-view editor over noob-vst-webgui-framework",
    );
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Compressor,
        ClapFeature::Stereo,
    ];
}

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

use crate::dsp::{self, Model, Processor, Settings, Shared, fet, opto};

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
                ..opto::Settings::default()
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

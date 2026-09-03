//! Noob CompressorLab: two classic compressors in one plug-in, built on
//! noob-vst-webgui-framework as an example of a product-sized plug-in with a
//! browser-rendered front panel. Each instance picks its model, the 1176
//! (a feedback FET compressor) or the LA-2A (an optical leveling
//! amplifier), and the page draws the matching faceplate. Both are
//! humorous, affectionate spoofs of hardware I admire, not replacements.
//!
//! | layer | path | role |
//! |---|---|---|
//! | DSP | [`dsp`] | both engines, the model switch, the parameter and stream layout |
//! | plug-in | `plugin` (feature `plugin`) | nih-plug VST3 / CLAP effect whose editor is the OS web view |
//! | standalone | `src/bin/standalone.rs` | a dev server with a fake audio thread and demo sources |
//! | page | `web/` | the Vue + Tailwind front panels |
//!
//! `research/1176.md` and `research/LA-2A.md` document how the originals
//! work and how they are simulated; `README.md` documents this plug-in.
//!
//! Where the framework ends: everything here is specific to these
//! compressors. The bridge, server, parameter mirroring, host adapter,
//! browser client, gestures, needle ballistics and charts come from
//! noob-vst-webgui-framework.

// DSP loops index several buffers by the same sample index; iterator chains
// would hide the arithmetic the comments describe.
#![allow(clippy::needless_range_loop)]

pub mod dsp;

#[cfg(feature = "plugin")]
pub mod plugin;

// The VST3 and CLAP entry points. nih-plug generates the C ABI exports from
// the `Plugin` / `Vst3Plugin` / `ClapPlugin` impls in `plugin.rs`.
#[cfg(feature = "plugin")]
nih_plug::nih_export_vst3!(plugin::NoobCompressorLab);
#[cfg(feature = "plugin")]
nih_plug::nih_export_clap!(plugin::NoobCompressorLab);

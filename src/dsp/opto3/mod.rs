//! The solid-state optical model of the lab: the LA-3A. The same T4B cell
//! as the LA-2A ([`crate::dsp::opto`]), driven much harder through a
//! transistor sidechain, with a transistor amplifier around it instead of
//! tubes.
//!
//! `research/LA-3A.md` sections 1 and 2 are what this is built from. That
//! document's own summary of the machine is one sentence: **the cell
//! stayed, everything around it got faster, louder, wider and smaller**.
//! So the engine reuses [`crate::dsp::opto::Cell`] unchanged and rebuilds
//! everything around it:
//!
//! | element | LA-2A | LA-3A here |
//! |---|---|---|
//! | gain element | T4B cell | the same [`crate::dsp::opto::Cell`], with one time constant moved |
//! | detector | feedback | feedback |
//! | sidechain drive | modest, from a tube stage | hot, from a transistor stage into a step-up autotransformer: this is what turns a 10 ms attack into about 1.5 ms |
//! | sidechain shaping | R37, flat by default | HF Contour, flat as shipped, up to 10 dB more reduction at 15 kHz when wound up, over a fixed low-frequency roll-off |
//! | amplifier | 12AX7A and a 12BH7A follower | a class-AB transistor stage: cleaner, more symmetric, and it leaves with odd harmonics rather than even |
//! | bandwidth | 30 Hz to 15 kHz | 20 Hz to 20 kHz |
//! | meter switch | GR, Output +10, Output +4 | GR and Output only on the hardware; `Off` is the plug-in's addition |
//!
//! Two notes on scope. The rear 50/30 dB input pad is not a parameter: the
//! model is fixed at the 50 dB position, which costs nothing, because
//! UREI's two published thresholds differ by exactly that pad, so pinning
//! one pins both. And the dossier's cell-wear switch is left out too; the
//! model is a fresh cell, because real LA-3As null against each other.

pub mod engine;

pub use engine::{Compressor, Settings, VU_REFERENCE_DBFS};

/// Labels of `la3a_mode`.
pub const MODE_NAMES: [&str; 2] = ["Compress", "Limit"];
/// Labels of `la3a_meter`. The hardware toggle has two positions, GR and
/// OUTPUT (where 0 VU is +4 dBm); `Off` is the plug-in's own, as on UA's.
pub const METER_NAMES: [&str; 3] = ["Gain Reduction", "Output", "Off"];

/// Labels of `la3a_cell`.
pub const CELL_NAMES: [&str; 3] = ["Fresh", "Used", "Tired"];

/// Meter positions.
pub const METER_GR: usize = 0;
pub const METER_OUT: usize = 1;
pub const METER_OFF: usize = 2;

#[cfg(test)]
mod tests;

//! The tube preamp section of the lab: the 610, which with the 1176 engine
//! behind it makes the 6176 channel. This module owns everything the stage
//! needs from the parameters ([`Settings`]), its switch labels and its
//! knob maps; the parameter ids, streams and the processor that hosts it
//! live one level up in [`crate::dsp`].
//!
//! The 610 is a microphone preamp with a two-band shelving equaliser, not a
//! compressor, so it earns its place in a compressor lab the way Putnam
//! gave it one: bolted to an 1176. `research/610.md` section 8 is the
//! design. In short, per channel:
//!
//! ```text
//! in ─► input select + pad ─► input transformer ─► input tube stage (Gain)
//!     ─► Level ─► LOW / HIGH shelves ─► output tube stage ─► output transformer
//!     ─► polarity ─► optional low cut ─► out (and the PRE meter tap)
//! ```
//!
//! The Gain switch is the interesting one: it trades attenuation for
//! negative feedback, so turning it up does not only make the stage louder,
//! it makes it dirtier. That is the whole reason the two knobs exist.
//!
//! | module | contents |
//! |---|---|
//! | [`stage`] | the engine: the two tube stages, the shelves, the transformers, the meter |
//! | [`adaa`] | antiderivative anti-aliasing for the two tube stages |
//! | [`filters`] | first-order shelves and the small sections the stage is built from |
//! | this file | switch labels, knob maps, [`Routing`], [`Voicing`], [`Settings`] |

pub mod adaa;
pub mod filters;
pub mod stage;

pub use stage::{Stage, Voicing, voicing};

/// Labels of `pre_join`.
pub const ROUTING_NAMES: [&str; 3] = ["Join", "BP", "1:1"];
/// Labels of `pre_gain`: the five-position stepped Gain switch.
pub const GAIN_NAMES: [&str; 5] = ["-10", "-5", "0", "+5", "+10"];
/// Labels of `pre_input`.
pub const INPUT_NAMES: [&str; 5] = ["Line", "Mic 500", "Mic 2.0K", "Hi-Z 47K", "Hi-Z 2.2M"];
/// Labels of `pre_lf_freq`, Hz.
pub const LF_FREQ_NAMES: [&str; 3] = ["70", "100", "200"];
/// Labels of `pre_hf_freq`.
pub const HF_FREQ_NAMES: [&str; 3] = ["4.5k", "7k", "10k"];
/// Labels of both shelf gain switches: eleven steps of 1.5 dB, with a 3 dB
/// jump at each end (`research/610.md` 2.1).
pub const SHELF_GAIN_NAMES: [&str; 11] = [
    "-9", "-6", "-4.5", "-3", "-1.5", "0", "+1.5", "+3", "+4.5", "+6", "+9",
];
/// Labels of `pre_voice`.
pub const VOICE_NAMES: [&str; 2] = ["610B", "610A"];
/// Labels of `pre_load`: the 1176 section's rear input-loading button.
pub const LOAD_NAMES: [&str; 2] = ["15k", "600"];
/// Labels of `pre_meter`.
pub const METER_NAMES: [&str; 3] = ["PRE", "GR", "COMP"];

/// Panel range of the Level knob.
pub const LEVEL_MAX: f32 = 10.0;

/// Gain switch positions in dB.
pub const GAIN_STEPS_DB: [f32; 5] = [-10.0, -5.0, 0.0, 5.0, 10.0];
/// The 610-A's three positions (LO, OFF, HI), mapped onto the same five
/// switch positions so the parameter does not change with the voicing.
pub const GAIN_STEPS_A_DB: [f32; 5] = [-8.0, -8.0, 0.0, 8.0, 8.0];
/// Shelf gain steps in dB.
pub const SHELF_GAIN_DB: [f32; 11] = [-9.0, -6.0, -4.5, -3.0, -1.5, 0.0, 1.5, 3.0, 4.5, 6.0, 9.0];
/// Low shelf corners in Hz.
pub const LF_FREQ_HZ: [f32; 3] = [70.0, 100.0, 200.0];
/// High shelf corners in Hz.
pub const HF_FREQ_HZ: [f32; 3] = [4500.0, 7000.0, 10_000.0];

/// Input-select gain offsets against Line, dB (`research/610.md` 8.4;
/// **derived** from the published maximum gains of the family).
pub const INPUT_OFFSET_DB: [f32; 5] = [0.0, 35.0, 30.0, 5.0, 8.0];

/// Level knob → gain in dB, an audio taper with unity at 5 and about
/// +20 dB fully clockwise (`research/610.md` 8.4; **estimate**, checked
/// against the family's published maximum gains).
pub const LEVEL_TABLE_DB: [f32; 11] = [
    -120.0, -36.0, -24.0, -15.0, -7.0, 0.0, 5.0, 9.0, 13.0, 16.5, 20.0,
];

/// Interpolate [`LEVEL_TABLE_DB`] at knob position `k`.
#[inline]
pub fn level_to_db(k: f32) -> f32 {
    let k = k.clamp(0.0, LEVEL_MAX);
    if k < 0.05 {
        return -120.0;
    }
    let i = (k.floor() as usize).min(LEVEL_TABLE_DB.len() - 2);
    let f = k - i as f32;
    LEVEL_TABLE_DB[i] + (LEVEL_TABLE_DB[i + 1] - LEVEL_TABLE_DB[i]) * f
}

/// What the 6176's Ratio switch does with its two extra positions
/// (`research/610.md` 2.2): the preamp always runs, and this says what
/// happens to the compressor behind it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Routing {
    /// The preamp feeds the compressor.
    #[default]
    Join,
    /// "BP": the preamp output goes straight to the output, removing the
    /// compressor's tone entirely.
    Bypass,
    /// "1": the compressor runs with no gain reduction, but its amplifiers
    /// still colour the signal.
    Unity,
}

impl Routing {
    pub fn from_index(i: usize) -> Self {
        match i {
            0 => Routing::Join,
            1 => Routing::Bypass,
            _ => Routing::Unity,
        }
    }
}

/// Which meter the 6176's switch selects.
pub const METER_PRE: usize = 0;
pub const METER_GR: usize = 1;
pub const METER_COMP: usize = 2;

/// Everything the stage needs from the parameters, read once per block.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub routing: Routing,
    /// Index into [`GAIN_STEPS_DB`].
    pub gain: usize,
    /// Index into [`INPUT_OFFSET_DB`].
    pub input: usize,
    /// −15 dB on the microphone inputs only (−20 dB on the A voicing).
    pub pad: bool,
    pub polarity: bool,
    /// Level knob, 0..[`LEVEL_MAX`].
    pub level: f32,
    /// Index into [`LF_FREQ_HZ`] and [`SHELF_GAIN_DB`].
    pub lf_freq: usize,
    pub lf_gain: usize,
    /// Index into [`HF_FREQ_HZ`] and [`SHELF_GAIN_DB`].
    pub hf_freq: usize,
    pub hf_gain: usize,
    /// The low cut of the LA-6176 and SOLO/610.
    pub hpf: bool,
    /// 0 = 610B (the 6176's own), 1 = 610A (the 1958 module).
    pub voice: usize,
    /// The 1176 section's rear input loading: 0 = 15 kΩ, 1 = 600 Ω.
    pub load: usize,
    /// [`METER_PRE`], [`METER_GR`] or [`METER_COMP`].
    pub meter: usize,
    /// Hard bypass, shared with the rest of the lab: a bypassed instance
    /// must take the preamp out too, not just the compressor behind it.
    pub bypass: bool,
    /// +48 V on the microphone input. The switch is real and the panel has
    /// it, so the plug-in saves its state, but it cannot be audible here:
    /// phantom power feeds a microphone, and the model starts at the
    /// preamp's input. Nothing downstream reads this.
    pub phantom: bool,
}

impl Default for Settings {
    /// The manual's starting point: Line in, Gain 0, Level 7, EQ flat.
    fn default() -> Self {
        Settings {
            routing: Routing::Join,
            gain: 2,
            input: 0,
            pad: false,
            polarity: false,
            level: 7.0,
            lf_freq: 1,
            lf_gain: 5,
            hf_freq: 2,
            hf_gain: 5,
            hpf: false,
            voice: 0,
            load: 0,
            meter: METER_GR,
            bypass: false,
            phantom: false,
        }
    }
}

#[cfg(test)]
mod tests;

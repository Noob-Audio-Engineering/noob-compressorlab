//! The FET model of the lab: the 1176. This module owns everything the
//! 1176 engine needs from the parameters ([`Settings`]), its knob-to-time
//! maps and its circuit revisions; the parameter ids, streams and the
//! processor that hosts it live one level up in [`crate::dsp`].
//!
//! The model follows `research/1176.md` section 7: a voltage-domain
//! **feedback** compressor whose sidechain is fed from the preamp output, a
//! single-capacitor diode detector whose diode bias *is* the threshold, a
//! FET control law with a linear-then-saturating dB-per-volt curve, the FET
//! divider with a signal-dependent (second- and third-order) resistance,
//! preamp and line-amp soft saturation, an output-transformer high-pass, the
//! "all buttons in" operating point, stereo linking, and two modern
//! additions (mix, side-chain high-pass). See [`compressor`] for the
//! equations and the constants that were tuned against the tests.
//!
//! | module | contents |
//! |---|---|
//! | [`compressor`] | the engine: oversampling, stages, detector, meters, static transfer curve |
//! | [`filters`] | RBJ biquad and one-pole helpers |
//! | [`oversample`] | 2x half-band up- and down-sampler |
//! | this file | knob maps, [`Ratio`], [`MeterMode`], [`Revision`], [`Settings`] |

pub mod compressor;
pub mod filters;
pub mod oversample;

pub use compressor::{Circuit, Compressor, TRANSFER_POINTS, circuit};

/// in" operating point, not a fifth ratio.
pub const RATIO_NAMES: [&str; 5] = ["4", "8", "12", "20", "All"];
/// Meter switch labels.
pub const METER_NAMES: [&str; 4] = ["GR", "+4", "+8", "Off"];
/// Revision labels ([`Revision`]), in parameter order: the two blue-stripe
/// revisions, the black-face lineage C to G, the silver-face H and the
/// reissue.
pub const REVISION_NAMES: [&str; 9] = ["A", "B", "C", "D", "E", "F", "G", "H", "LN"];

/// Panel mark range of the Input and Output knobs. The printed marks are
/// attenuation in dB from the fully clockwise position: mark `m` is
/// `m − 48` dB, so 48 is 0 dB, 24 is −24 dB and 0 is −48 dB. The dial
/// *rotation* is not linear in dB; the page draws the marks with the taper
/// table from `research/1176.md` 7.2 and the DSP works in dB.
pub const MARK_MAX: f32 = 48.0;
/// Attack knob: 0 is the OFF detent, 1..7 the printed marks (800 µs at 1
/// down to 20 µs at 7, geometric).
pub const ATTACK_MAX: f32 = 7.0;
/// Release knob marks 1..7 (1100 ms at 1 down to 50 ms at 7, geometric).
pub const RELEASE_MAX: f32 = 7.0;
/// Upper end of the side-chain high-pass; 0 turns it off.
pub const SC_HPF_MAX_HZ: f32 = 300.0;

/// Panel mark → gain in dB (see [`MARK_MAX`]).
#[inline]
pub fn mark_to_db(mark: f32) -> f32 {
    mark.clamp(0.0, MARK_MAX) - MARK_MAX
}

/// Attack knob position (1..7) → time constant in seconds (geometric map,
/// `research/1176.md` 7.2): 800, 434, 236, 128, 69, 37, 20 µs at the marks.
/// Positions below 1 are the OFF detent and return `None`.
#[inline]
pub fn attack_seconds(knob: f32) -> Option<f32> {
    if knob < 0.5 {
        return None;
    }
    let p = knob.clamp(1.0, ATTACK_MAX);
    Some(800e-6 * (20.0f32 / 800.0).powf((p - 1.0) / 6.0))
}

/// Release knob position (1..7) → time constant in seconds (geometric map,
/// `research/1176.md` 7.2): 1100, 657, 393, 235, 140, 84, 50 ms at the marks.
#[inline]
pub fn release_seconds(knob: f32) -> f32 {
    let p = knob.clamp(1.0, RELEASE_MAX);
    1.1 * (50.0f32 / 1100.0).powf((p - 1.0) / 6.0)
}

/// Ratio button.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Ratio {
    #[default]
    R4,
    R8,
    R12,
    R20,
    /// All buttons in.
    All,
}

impl Ratio {
    /// From the parameter value / label index (clamped).
    pub fn from_index(i: usize) -> Self {
        match i {
            0 => Ratio::R4,
            1 => Ratio::R8,
            2 => Ratio::R12,
            3 => Ratio::R20,
            _ => Ratio::All,
        }
    }
}

/// Meter switch position.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MeterMode {
    /// Gain reduction: 0 at rest, swings left.
    #[default]
    Gr,
    /// Output level, 0 VU = +4 dBu (−18 dBFS RMS in this calibration).
    Plus4,
    /// Output level, 0 VU = +8 dBu (−14 dBFS RMS).
    Plus8,
    /// Meter off (the needle rests).
    Off,
}

impl MeterMode {
    pub fn from_index(i: usize) -> Self {
        match i {
            0 => MeterMode::Gr,
            1 => MeterMode::Plus4,
            2 => MeterMode::Plus8,
            _ => MeterMode::Off,
        }
    }
}

/// Circuit revision (research/1176.md §1.2). Each entry selects a
/// [`Circuit`] and, in the UI, a faceplate look:
/// A and B are the silver "Bluestripe", C to G the black face, H the late
/// silver face, LN the black reissue. Revisions that share a circuit share
/// constants (C = D = E, G = H); see the README's revision table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Revision {
    /// June 1967, 25 built: FET preamp, no low-noise circuit. Noisiest, most second harmonic.
    A,
    /// Late 1967 to 1970: bipolar preamp, still no LN circuit.
    B,
    /// 1970: the LN circuit as a potted module, black face.
    C,
    /// To 1973: the LN circuit on the main board; the reference black face.
    D,
    /// 1973: D with a switchable mains transformer, otherwise identical.
    E,
    /// 1973 on: push-pull class-AB output stage, new output transformer. Lowest THD.
    F,
    /// Electronically balanced input replaces the input transformer.
    G,
    /// Silver face, cosmetic only: the G circuit.
    H,
    /// The reissue: based on C / D / E with a modern noise floor.
    #[default]
    Ln,
}

impl Revision {
    /// Every revision in parameter order.
    pub const ALL: [Revision; 9] = [
        Revision::A,
        Revision::B,
        Revision::C,
        Revision::D,
        Revision::E,
        Revision::F,
        Revision::G,
        Revision::H,
        Revision::Ln,
    ];

    /// From the parameter value / label index (clamped to LN).
    pub fn from_index(i: usize) -> Self {
        Revision::ALL.get(i).copied().unwrap_or(Revision::Ln)
    }

    /// The parameter value.
    pub fn index(self) -> usize {
        self as usize
    }

    /// The printed label ([`REVISION_NAMES`]).
    pub fn label(self) -> &'static str {
        REVISION_NAMES[self.index()]
    }
}

/// Everything the engine needs from the parameters, read once per block.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// Input mark, 0..48 (see [`MARK_MAX`]).
    pub input: f32,
    /// Output mark, 0..48.
    pub output: f32,
    /// Attack knob, 0 (OFF) or 1..7.
    pub attack: f32,
    /// Release knob, 1..7.
    pub release: f32,
    pub ratio: Ratio,
    pub meter: MeterMode,
    /// Circuit revision: the character of the FET, amplifiers, transformers
    /// and noise floor (and the faceplate look in the UI).
    pub revision: Revision,
    /// Share one detector between the channels.
    pub link: bool,
    /// Wet share, 0..1 (modern addition).
    pub mix: f32,
    /// Side-chain high-pass corner in Hz, 0 = off (modern addition).
    pub sc_hpf_hz: f32,
    pub bypass: bool,
}

impl Default for Settings {
    /// The manufacturer's starting point: 24 / 24, attack 4, release 4,
    /// 4:1, GR meter, LN, linked, 100 % wet.
    fn default() -> Self {
        Settings {
            input: 24.0,
            output: 24.0,
            attack: 4.0,
            release: 4.0,
            ratio: Ratio::R4,
            meter: MeterMode::Gr,
            revision: Revision::Ln,
            link: true,
            mix: 1.0,
            sc_hpf_hz: 0.0,
            bypass: false,
        }
    }
}

#[cfg(test)]
mod tests;

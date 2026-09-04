//! The variable-mu model of the lab: the Fairchild 660 and 670.
//!
//! This module owns everything the engine needs from the parameters
//! ([`Settings`]) and the switch positions the panel prints; the parameter
//! ids, streams and the processor that hosts it live one level up in
//! [`crate::dsp`].
//!
//! The model follows `research/Fairchild-670.md` section 10. It opens the
//! **variable-mu** family, which is the last kind of gain element the lab
//! was missing, and it is unlike the other six in a way that is structural
//! rather than tonal: every other model has a part that attenuates and an
//! amplifier that colours, so the two can be tuned apart. Here there is
//! nothing between the transformers but the tubes, so gain reduction and
//! distortion are one curve read at two points and no honest model can offer
//! a drive control. What it offers instead is [`Settings::zero`], the
//! hardware's own ZERO screw, which moves the operating point and therefore
//! moves the standing gain, the available gain reduction and the standing
//! distortion together.
//!
//! | module | contents |
//! |---|---|
//! | [`triode`] | the remote-cutoff triode, now its own component crate |
//! | [`network`] | the six-position time-constant network, from the factory drawing |
//! | [`oversample`] | the half-band cascade, 4x / 8x / 16x |
//! | [`engine`] | the push-pull stage, the sidechain, the matrix, the meters, the static curve |
//! | this file | the switch positions and [`Settings`] |

pub mod engine;
pub mod network;
pub mod oversample;

/// The remote-cutoff triode lives in its own crate, because a valve is a
/// part and everything this module puts around it — four sections a side,
/// push-pull, the cathode resistors, the control injection, the timing
/// network — is the machine. The evidence went with it: the law is fitted to
/// plate current, its slope is a separate matter, and one parameter of the
/// published fit had to be refitted against the manufacturer's plate
/// characteristics before the slope was usable at all
/// (`research/Fairchild-670.md` section 4.3).
pub use noob_electrical_components::remote_cutoff_triode as triode;

pub use engine::Compressor;
pub use network::TIME_NAMES;

/// Which unit the model wears. The dossier's 1.3 reads the two schematics
/// side by side and finds less difference than the folklore claims: the same
/// timing network to the component value, and a genuinely different
/// operating point in the gain stage.
pub const MODEL_660: usize = 0;
pub const MODEL_670: usize = 1;
/// Labels of the unit switch, in parameter order.
pub const UNIT_NAMES: [&str; 2] = ["660", "670"];

/// S301, the AGC switch: two independent limiters, or a matrix on both sides
/// and the two channels working on lateral and vertical.
pub const AGC_LEFT_RIGHT: usize = 0;
pub const AGC_LAT_VERT: usize = 1;
/// Labels of the AGC switch, as the panel brackets them.
pub const AGC_NAMES: [&str; 2] = ["Left/Right", "Lat/Vert"];

/// S101, the METERING lever: the push side of the output stage, the centre
/// tap, the pull side. The panel prints `BAL ZERO BAL`; the names here are
/// POM Audio Design's, who describe the three positions in more detail than
/// any other source.
pub const METER_BAL_PUSH: usize = 0;
pub const METER_ZERO: usize = 1;
pub const METER_BAL_PULL: usize = 2;
pub const METER_NAMES: [&str; 3] = ["Bal Push", "Zero", "Bal Pull"];

/// The tube fitted. GE publish 4000 µmhos at the class-A1 point and JJ
/// publish 3000 for their modern replacement at the same point, which is a
/// real, published, sourced difference of 2.5 dB rather than a flavour.
pub const TUBE_GE_6386: usize = 0;
pub const TUBE_JJ_6386_LGP: usize = 1;
pub const TUBE_NAMES: [&str; 2] = ["GE 6386", "JJ 6386 LGP"];

/// Oversampling factor, ours and not the hardware's.
pub const OVERSAMPLE_NAMES: [&str; 3] = ["4x", "8x", "16x"];

/// AT101's range: a step attenuator, 1 dB per step, 21 detents printed
/// `0 2 4 … 20` with unnumbered dots between.
pub const INPUT_GAIN_MAX_DB: f32 = 20.0;
/// The threshold ring, printed `0 1 2 … 10`. **Not decibels**: the pot is
/// linear with a resistor on its centre tap, and what it sets jointly with
/// the DC threshold is a curve rather than a point.
pub const THRESHOLD_MAX: f32 = 10.0;
/// The ZERO screw's range in volts of standing grid bias, around
/// Raffensperger's −7.2 V.
pub const ZERO_MIN_V: f32 = -12.0;
pub const ZERO_MAX_V: f32 = -3.0;

/// Everything the engine needs from the parameters, read once per block.
///
/// Every value the hardware has twice is here twice, because the 670 is two
/// complete limiters and its two channels are meant to be set differently:
/// that is the whole point of the lateral-and-vertical mode, where
/// compressing the vertical component harder than the lateral is a
/// deliberate way of fitting a groove. The 660 is a mono unit, so when
/// [`Settings::model`] is [`MODEL_660`] both channels take the left row's
/// values and the AGC switch is out of circuit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// [`MODEL_660`] or [`MODEL_670`].
    pub model: usize,
    /// AT101 / AT201: attenuation in dB, 0 to 20 in whole steps.
    pub input_gain: [f32; 2],
    /// R115 / R215, in panel units 0 to 10.
    pub threshold: [f32; 2],
    /// S102 / S202, 0-based (the panel prints 1 to 6).
    pub time: [usize; 2],
    /// R117 / R217, the trimmer inside the chassis, as travel from fully
    /// anticlockwise. Exposed deliberately: it is the ratio and knee
    /// control, and every emulation that is any good brings it out.
    pub dc_threshold: [f32; 2],
    /// R142 / R242, the front-panel screwdriver marked ZERO, in volts of
    /// standing grid bias. A bias trim wearing a meter-calibration label.
    pub zero: [f32; 2],
    /// R105 / R205, the front-panel screwdriver marked BAL, −1 to +1.
    pub balance: [f32; 2],
    /// S101 / S201.
    pub meter: [usize; 2],
    /// S301.
    pub agc: usize,
    /// Ours: which 6386 is fitted.
    pub tube: usize,
    /// Ours: 0 = 4x, 1 = 8x, 2 = 16x.
    pub oversample: usize,
    /// Ours: one detector for both channels. **The hardware has no such
    /// thing** — its lateral-and-vertical mode is two matrices and two
    /// entirely independent limiters — so every preset of this model turns
    /// it off.
    pub link: bool,
    /// Ours: wet share, 0..1.
    pub mix: f32,
    /// Ours: side-chain high-pass corner in Hz, 0 = off.
    pub sc_hpf: f32,
    pub bypass: bool,
}

impl Settings {
    /// Cascade depth of the oversampler: 4x, 8x or 16x.
    pub fn depth(&self) -> usize {
        match self.oversample {
            0 => 2,
            2 => 4,
            _ => 3,
        }
    }
}

impl Default for Settings {
    /// The manual's own starting point: the unity-gain attenuator setting,
    /// the threshold fully clockwise, time constant 3 (*"merely a first
    /// suggestion for a general purpose timing circuit"*), the meter in
    /// ZERO, two independent limiters. The DC threshold's default is the
    /// factory-adjusted condition, fitted to the published curve 3.
    fn default() -> Self {
        Settings {
            model: MODEL_670,
            input_gain: [10.0; 2],
            threshold: [10.0; 2],
            time: [2; 2],
            dc_threshold: [0.07; 2],
            zero: [engine::V_BIAS_NOMINAL; 2],
            balance: [0.0; 2],
            meter: [METER_ZERO; 2],
            agc: AGC_LEFT_RIGHT,
            tube: TUBE_GE_6386,
            oversample: 1,
            link: false,
            mix: 1.0,
            sc_hpf: 0.0,
            bypass: false,
        }
    }
}

#[cfg(test)]
mod tests;

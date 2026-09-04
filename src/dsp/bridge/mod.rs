//! The diode-bridge family of the lab: the Neve 2254 and the 33609.
//!
//! `research/Neve-33609.md` is what this is built from. Three findings out
//! of it shape the whole module, and each is the difference between this
//! model and a generic compressor wearing a Neve faceplate.
//!
//! **The gain element's law is a hyperbolic tangent.** Four diodes with
//! two floating common nodes make a differential pair, so the bridge
//! current is `I·tanh(u/2ηV_T)` with no implicit resistive term to solve.
//! That is not the Wright omega function the survey recommending this unit
//! expected, and it is not an approximation of one: the omega form belongs
//! to a *single* diode shunting a resistor, where the resistance sits
//! inside the loop. The derivation validates itself, giving 25.01 dB of
//! open-bridge loss from three resistor values where Neve annotated 25 dB
//! on the same drawing. The law lives in the components repository, since
//! it is a part rather than a circuit; the divider around it is here.
//!
//! **The two sidechains listen at different points.** The compressor taps
//! the RV1 wiper, before the make-up amplifier; the limiter taps the 10640
//! output, after it. So raising the make-up gain drives the limiter harder
//! while leaving the compressor's own threshold exactly where it was. AMS
//! Neve sell that as a feature, and an emulation that taps both detectors
//! at one node loses the unit's most distinctive move. They combine as a
//! **maximum**, because TR9 and TR13 are emitter followers into a shared
//! load: whichever base is higher holds the node and the other turns off.
//!
//! **Distortion is set by the voltage across the bridge, not by the amount
//! of gain reduction.** More control current means less resistance, less
//! voltage across the bridge and a smaller `tanh` argument, so the bridge
//! itself distorts *less* as it works harder. The published figures do
//! rise with gain reduction, but that is sidechain ripple modulating the
//! gain and the transformers being driven, not the bridge waveshaping. The
//! detector is therefore deliberately under-smoothed rather than being
//! given a clean envelope, because a perfectly smoothed one would pass the
//! published distortion figures by cheating.
//!
//! # The ratios are not what the panel says
//!
//! The handbook publishes a calibration table with its own tolerances, and
//! it is the best ground truth any model in the lab has. Two of the five
//! printed ratios are wrong: 3:1 is really 2.86:1 and 6:1 is really
//! 6.67:1. [`RATIO_TRUE`] carries the real figures and [`RATIO_NAMES`] the
//! printed ones, and the model uses the former while the panel shows the
//! latter.
//!
//! Note that these are *closed-loop* ratios. This is a feedback design, so
//! a detector reading the compressed output turns an open-loop law of
//! slope `s` into a ratio of `s + 1`. The engine's law slopes are
//! therefore `RATIO_TRUE − 1`.
//!
//! # What is fitted rather than derived
//!
//! Two constants, both named at their definitions. The bridge's drive
//! level is calibrated against the published distortion rather than
//! against the block diagram's level annotation, because the two disagree
//! by about 20 dB and the dossier's section 4.5 flags that as unresolved.
//! And the control law's two slopes are fitted to the 2254 level diagram's
//! two published control voltages, because the divider contains a factory
//! preset whose position no drawing states.
//!
//! # Why there are no lookup tables here
//!
//! Every control on this unit is a switch, and all three of the ones with
//! real units step *linearly*: the limit threshold in 0.5 dB, the compress
//! threshold in 2 dB and the make-up in 2 dB. So the parameter's own range
//! and step count reproduce the switch exactly, and the framework samples
//! the staircase for the manifest itself. A sampled table would only add
//! interpolation error to a law that has none, which is why the other
//! models' tables have no counterpart here.

pub mod engine;

pub use engine::{Compressor, Network, Settings, VU_REFERENCE_DBFS};

/// Labels of `neve_model`.
pub const MODEL_NAMES: [&str; 3] = ["2254E", "33609J", "33609N"];
/// The 1969 valve-era unit, two rack units, with a switchable meter.
pub const MODEL_2254E: usize = 0;
/// The 1980s solid-state revision the handbook documents.
pub const MODEL_33609J: usize = 1;
/// The current revision, which adds a compressor attack switch.
pub const MODEL_33609N: usize = 2;

/// Labels of `neve_compress_ratio`, exactly as the panel prints them.
pub const RATIO_NAMES: [&str; 5] = ["1.5:1", "2:1", "3:1", "4:1", "6:1"];

/// What those five positions really are.
///
/// Derived from the 33609/J handbook's own calibration table, which states
/// the output change for a 10 dB input step at each position: 6.5, 5.0,
/// 3.5, 2.5 and 1.5 dB. Two of the printed labels are simply wrong, and
/// the model follows the measurements rather than the silkscreen.
pub const RATIO_TRUE: [f32; 5] = [1.54, 2.00, 2.86, 4.00, 6.67];

/// Labels of `neve_limit_attack`.
pub const LIMIT_ATTACK_NAMES: [&str; 2] = ["Slow", "Fast"];
/// Slow attack, R28 alone.
pub const LIMIT_ATTACK_SLOW: usize = 0;
/// Fast attack, R30 switched in.
pub const LIMIT_ATTACK_FAST: usize = 1;

/// Labels of `neve_compress_attack`. The /N only; the /J and the 2254 have
/// a fixed compressor attack and no control for it.
pub const COMPRESS_ATTACK_NAMES: [&str; 2] = ["Fast", "Slow"];

/// Labels of `neve_limit_recovery`.
pub const LIMIT_RECOVERY_NAMES: [&str; 6] = ["50 ms", "100 ms", "200 ms", "800 ms", "A1", "A2"];
/// Labels of `neve_compress_recovery`.
pub const COMPRESS_RECOVERY_NAMES: [&str; 6] =
    ["100 ms", "400 ms", "800 ms", "1500 ms", "A1", "A2"];
/// The first automatic recovery position, 100 ms over a 2 s platform.
pub const RECOVERY_AUTO1: usize = 4;
/// The second, 50 ms over a 5 s platform.
pub const RECOVERY_AUTO2: usize = 5;

/// Labels of `neve_meter_select`. The 2254/E only.
pub const METER_NAMES: [&str; 3] = ["In", "Control", "Out"];

#[cfg(test)]
mod tests;

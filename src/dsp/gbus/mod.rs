//! The bus compressor of the lab: the SSL 4000 G, built from
//! `research/SSL-Gbus.md`.
//!
//! Three findings out of that dossier shape the whole module, and each one
//! is the difference between this model and a generic VCA compressor
//! wearing an SSL faceplate.
//!
//! **It is a feedback compressor.** SSL's own card 82E27 splits the
//! detector's control voltage through three 100 kΩ resistors: R26 carries
//! it to the amplifier driving the *audio* VCAs and R27 carries the same
//! voltage to the amplifier driving the *sidechain* VCA, with only the
//! threshold pot's offset added to the second one. So the detector hears a
//! signal that has already been attenuated by exactly the amount the
//! compressor is currently attenuating the audio, and the loop is closed
//! (dossier 3.4, 5.2, 5.3). The audio path is still topologically
//! feedforward, which is why the latency is zero and no detector noise
//! reaches the signal. A team fitting grey-box models to 2528 hours of
//! recordings from a real module found their residual concentrated exactly
//! where a missing feedback path would put it, which is independent
//! corroboration from outside SSL.
//!
//! **The ratio rises with gain reduction and never straightens.** A linear
//! rectifier inside a feedback loop around a dB-domain VCA gives
//! `ratio(GR) = 1 + 0.11513·(GR + V_d/k)` (dossier 5.4). There is no fixed
//! slope anywhere on the curve and no corner at all: the curve bends for
//! its whole length. **So the knee cannot be a width parameter and this
//! module does not have one.** A model that exposes `knee_width_db` and
//! blends two straight lines can be tuned to match this box at one setting
//! and will be wrong at the next.
//!
//! **The automatic release is a two-section ladder whose charge is shared
//! unevenly.** Not one envelope with an adaptive coefficient: two RC
//! sections in series, 91 kΩ with 0.47 µF and 750 kΩ with 6.8 µF, charged
//! by the same current and decaying independently (dossier 7.4). A short
//! peak puts `C2/C1 = 14.5` times as much voltage on the fast section, so
//! it releases in about 43 ms; sustained compression lets the slow section
//! charge to its own equilibrium, where the resistors put 89 % of the
//! voltage on it and it releases over about 5 s. That is where the
//! programme dependence comes from, and it falls out of four component
//! values rather than being tuned.
//!
//! # The threshold is a diode, not a comparator
//!
//! Nothing in this circuit compares a level against a threshold. Gain
//! reduction begins when the rectified, ratio-scaled detector voltage
//! exceeds one diode drop on its way to the timing network, and a diode
//! turns on over a decade of current rather than at a corner. [`SOFTPLUS_V`]
//! is that turn-on, and it is the whole knee.
//!
//! # The RC network is simulated, not approximated by two coefficients
//!
//! The attack resistor, the release resistor and the capacitor are three
//! components in [`engine::Timing`], not an `attack_coeff` and a
//! `release_coeff`. That buys a real behaviour for free: the attack and
//! release resistors form a potential divider, so at the slowest attack
//! with the fastest release the network can only reach 40 % of the control
//! voltage it would otherwise reach, which is 8 dB of gain reduction lost.
//! No emulation with independent coefficients does that. The dossier flags
//! it as derived from the topology rather than measured, and it is the
//! least supported thing in this module.
//!
//! # What is estimated, and what it rests on
//!
//! Four things, all named at their definitions and all marked in the
//! README's table.
//!
//! - [`ratio_scaling`] — the control-bus volts per dB. SSL publish no
//!   measured transfer point for any ratio position, so this cannot be
//!   calibrated against a figure. It is derived instead from one
//!   convention, that the printed ratio is the ratio *at the knee*, which
//!   reproduces all three of the dossier's independently estimated values
//!   exactly.
//! - [`DETECTOR_SCALE`] — where the knee sits in absolute terms, anchored
//!   to the level the only measured recordings of this unit were made at.
//! - [`V_DIODE`] and [`SOFTPLUS_V`] — a silicon small-signal diode's drop
//!   and turn-on width.
//! - The second-harmonic coefficient in [`engine::BlackmerCell`], set from
//!   the THAT 2180A datasheet's own THD table.
//!
//! # Three places this module departs from the dossier's section 11
//!
//! Each is grounded in a published figure that section 11 itself cites, and
//! each is argued at the point where it happens.
//!
//! 1. **The threshold's sense is inverted from the dossier's equations.**
//!    Section 11.4 writes the sidechain gain as `T − GR`, which makes a
//!    higher `ssl_threshold` compress *more*. The panel prints THRESHOLD in
//!    dB, and the only published statement of the equivalence is SSL's own:
//!    the sidechain trims "increase the side chain level by 10dB —
//!    **effectively reducing the threshold** on that channel by 10dB". So a
//!    threshold reading and a sidechain gain run in opposite directions,
//!    and this module uses `−Θ − GR` so the knob reads as its legend does.
//!    The argument is set out in full at `sidechain_gain_db` in
//!    [`engine`].
//! 2. **The gain cell distorts on its input, not its output.** Section 11.3
//!    writes `x·gain + d2·(x·gain)²`. That form fits the THAT datasheet's
//!    first THD point and misses its second by a factor of seven, because
//!    the datasheet's distortion *rises* as the gain falls. Shaping before
//!    the gain fits both points within 27 %, which is inside the ±50 % the
//!    dossier's own test 24 allows. See [`engine::BlackmerCell`].
//! 3. **Oversampling offers 1× and 2×, not 4×.** Both nonlinearities here
//!    are exactly second order — a squarer and a product of two signals —
//!    so their output bandwidth is exactly twice their input bandwidth and
//!    2× already contains it with nothing left to fold. A 4× position could
//!    not differ audibly from 2×, and a control that cannot do anything is
//!    the dead ornament this repository has removed twice.

pub mod engine;

pub use engine::{BlackmerCell, Compressor, Settings, Timing};

/// Labels of `ssl_attack`, exactly as card 82E27's panel legend prints
/// them under `ATTACK mS` (dossier 7.1).
pub const ATTACK_NAMES: [&str; 6] = [".1", ".3", "1", "3", "10", "30"];
/// Labels of `ssl_release`, the console ladder under `RELEASE S`
/// (dossier 7.2). The last position is the two-section automatic network.
pub const RELEASE_NAMES: [&str; 5] = [".1", ".3", ".6", "1.2", "Auto"];
/// Index of the automatic release position.
pub const RELEASE_AUTO: usize = 4;
/// Labels of `ssl_ratio`, the console's three positions (dossier 6.3).
///
/// SSL's own plug-in guide prints "2:1, 4:1 and **20:1**" where every SSL
/// hardware panel prints 10:1. The dossier declines to correct either, and
/// notes that a unit whose ratio rises with gain reduction has no single
/// ratio, so both numbers can be true of one curve read at two operating
/// points. The panel's figure is what is printed here.
pub const RATIO_NAMES: [&str; 3] = ["2:1", "4:1", "10:1"];
/// The printed ratios, as numbers. These are the ratios **at the knee**;
/// every one of them rises with gain reduction (see [`ratio_at`]).
pub const RATIO_PRINTED: [f32; 3] = [2.0, 4.0, 10.0];
/// Labels of `ssl_hpf`, the sidechain filter on the 500-series module
/// (dossier 5.6).
///
/// SSL's product page says 106 Hz where SSL's own module panel and recall
/// sheet both print 105. The panel's figure is used.
pub const HPF_NAMES: [&str; 6] = ["Off", "30", "60", "105", "125", "185"];
/// Corner frequencies of `ssl_hpf` in Hz; 0 is Off.
pub const HPF_HZ: [f32; 6] = [0.0, 30.0, 60.0, 105.0, 125.0, 185.0];
/// Labels of `ssl_link`. The first is the hardware's own behaviour; the
/// other three are ours, after the modes SSL put on THE BUS+.
pub const LINK_NAMES: [&str; 4] = ["Dominant", "Sum", "Dual", "M/S"];
/// Labels of `ssl_oversample`.
pub const OVERSAMPLE_NAMES: [&str; 2] = ["1x", "2x"];

/// The timing capacitor on every fixed release position, farads
/// (card 82E27, dossier 7.1 and 7.2).
pub const TIMING_C: f32 = 0.47e-6;
/// The attack ladder, ohms: R1 to R6 on card 82E27 (dossier 7.1). The
/// sequence is the E24 preferred-value approximation to half-decade steps.
pub const ATTACK_R: [f32; 6] = [820.0, 2_700.0, 8_200.0, 27_000.0, 82_000.0, 270_000.0];
/// The fixed release ladder, ohms: R12, R11, R10 and R9 on card 82E27, in
/// panel order `.1 .3 .6 1.2` (dossier 7.2).
///
/// **The `.1` position looks wrong and is correct.** Its 180 kΩ gives
/// 84.6 ms, which is 1.18 times the panel figure where the other three are
/// 2.1 to 2.4 times theirs. The dossier reads R12 unambiguously at 16×
/// magnification and records the discrepancy rather than adjusting the
/// value to taste. It is in the drawing, not in this model.
pub const RELEASE_R: [f32; 4] = [180_000.0, 270_000.0, 560_000.0, 1_200_000.0];
/// The automatic release's fast section: C1 and R7 on card 82E27.
pub const AUTO_C1: f32 = 0.47e-6;
pub const AUTO_R7: f32 = 91_000.0;
/// The automatic release's slow section: C2 and R8 on card 82E27.
pub const AUTO_C2: f32 = 6.8e-6;
pub const AUTO_R8: f32 = 750_000.0;

/// D6's forward drop, volts. The part is a 1S44 on card 82E26; the voltage
/// is the clone builder's reading. **Estimate.**
pub const V_DIODE: f32 = 0.6;
/// The diode's turn-on width, `n·V_T` in volts, for a silicon small-signal
/// diode. **Estimate.** This is the entire knee: it has no corner and it
/// turns on over about a decade of current.
pub const SOFTPLUS_V: f32 = 45e-3;
/// `ln(10)/20`, the constant that makes the ratio rise (dossier 5.4).
pub const LN10_OVER_20: f32 = std::f32::consts::LN_10 / 20.0;

/// The detector stage's gain at each ratio position: `R_f / 20 kΩ` for the
/// three throws of SW1 on card 82E26, with R38 1 MΩ permanently in the
/// feedback path and R39 510 kΩ or R40 270 kΩ switched across it
/// (dossier 6.1).
///
/// **Which throw is which printed ratio is fixed by SSL's own sentence**,
/// not by taste: "Decreasing the RATIO setting **lowers the effective
/// threshold**." A higher detector gain reaches the diode drop at a lower
/// input, so the lowest ratio must take the highest gain.
pub const DETECTOR_GAIN: [f32; 3] = [50.0, 16.9, 10.6];

/// Volts per unit sample amplitude at the detector's input, folding the
/// summing gain and the rectifier's scaling.
///
/// **Estimate, and it is the one number that sets where the knee sits in
/// absolute terms.** SSL publish no measured transfer point for this box
/// at any setting, so there is nothing to calibrate against and the anchor
/// has to be an operating condition instead.
///
/// The one that is published, and the one used here, is the level the only
/// measured recordings of this unit were made at: the DAFx dataset drove a
/// real 500-series module with songs normalised to **−12 dB**. So this is
/// set so that the detector's output equals one diode drop when a
/// −12 dBFS signal arrives at 4:1 with the threshold centred, which puts
/// the unit's canonical three or four decibels of bus compression at the
/// middle of the threshold control's travel.
///
/// Solving `V_d = G·A·10^(L/20)` at `G` = 16.9 and `L` = −12 dBFS gives
/// 0.1413.
///
/// **Note that this is not where gain reduction begins.** D6 is a real
/// diode with a real exponential turn-on, so the onset is soft and sits a
/// few decibels below this point; the ideal-diode arithmetic above locates
/// the *centre* of that turn-on, not its foot. The dossier's own point,
/// that this compressor's curve has no corner anywhere, is the same
/// observation.
///
/// **The nominal level is not used as the anchor**, though it was at
/// first. SSL's "+4 dBu" is a VU reference and this detector is a peak
/// rectifier, so anchoring a peak detector to an average reference put the
/// knee about 12 dB too low and left the threshold control usable only
/// over its top third.
pub const DETECTOR_SCALE: f32 = 0.141_34;

/// The control-bus scaling `k` in volts per dB, for a printed ratio.
///
/// **Estimate, from one convention rather than six invented numbers.** The
/// dossier could not close the mapping from switch position to printed
/// ratio, because three of SW1's four poles are not on the drawing, and it
/// offers `k` as a calibration table marked estimate. Rather than carry
/// three loose numbers, this takes the convention that **the printed ratio
/// is the ratio at the knee** and derives `k` from the loop equation:
/// `ratio(0) = 1 + 0.11513·V_d/k = r`, so `k = 0.11513·V_d/(r − 1)`.
///
/// That reproduces all three of the dossier's independently chosen values
/// exactly — 69, 23 and 7.7 mV/dB at 2:1, 4:1 and 10:1 — which is the check
/// that the convention is the one the dossier was reasoning from. It also
/// makes the whole ratio law follow from one estimate instead of three, and
/// gives `ratio(GR) = r + 0.11513·GR` in closed form.
#[inline]
pub fn ratio_scaling(printed_ratio: f32) -> f32 {
    LN10_OVER_20 * V_DIODE / (printed_ratio - 1.0).max(1e-3)
}

/// The instantaneous compression ratio at `gr_db` of gain reduction, for a
/// switch printed with `printed_ratio` (dossier 5.4).
///
/// It rises by [`LN10_OVER_20`] per dB of gain reduction and never
/// straightens. This is a description of the model rather than a knob: no
/// part of the engine reads it, and it exists so the tests and the page can
/// state the curve's shape.
#[inline]
pub fn ratio_at(printed_ratio: f32, gr_db: f32) -> f32 {
    printed_ratio + LN10_OVER_20 * gr_db
}

/// Open-loop attack time constant at a switch position, seconds
/// (`R × C`, dossier 7.1).
#[inline]
pub fn attack_tau(i: usize) -> f32 {
    ATTACK_R[i.min(ATTACK_R.len() - 1)] * TIMING_C
}

/// Release time constant at a fixed switch position, seconds
/// (`R × C`, dossier 7.2). [`RELEASE_AUTO`] has no single constant.
#[inline]
pub fn release_tau(i: usize) -> f32 {
    RELEASE_R[i.min(RELEASE_R.len() - 1)] * TIMING_C
}

#[cfg(test)]
mod tests;

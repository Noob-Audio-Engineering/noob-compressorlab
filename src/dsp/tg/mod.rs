//! The EMI TG12413, the second unit in the lab's **diode** family and the
//! one that shows why the family is not called "diode bridge".
//!
//! `research/TG12413.md` is what this is built from. Four findings out of
//! it shape the whole module, and each is the difference between this
//! model and a Neve wearing a different faceplate.
//!
//! **It is not a bridge and it does not share the bridge's crate.** Neve's
//! element is a four-diode ring with two floating common nodes,
//! forward-biased by an injected current, one junction per arm. EMI's is
//! two branches of two diodes in series, both the same way up, sharing the
//! +20 V rail as their common node, and as drawn it is in reverse
//! breakdown rather than forward conduction. The dossier's section 4.5
//! lists thirteen rows of comparison and six of them are structural. So
//! [`element`] is built here, and the right-hand column of the dossier's
//! own constants table — the one headed "from the shared diode-bridge
//! component crate" — is empty. That empty table is the finding.
//!
//! What generalises is one level up, and [`element::Element`] is written
//! as that generalisation: *n* junctions per arm with a bulk resistance,
//! which becomes the Neve's law exactly at *n* = 1 and *r_b* = 0. Test 8
//! asserts that identity against the shipped crate to 1 × 10⁻⁹ relative,
//! which is the argument for a re-drawn component made executable rather
//! than argued.
//!
//! **Distortion goes the other way from the Neve's, because the element is
//! transparent when it is idle.** The Neve's bridge shunts a divider and
//! the voltage across it falls as the control current rises, so its own
//! distortion falls as it works harder. This element carries no current at
//! all until the sidechain drives it, and an element carrying no current
//! cannot bend a waveform. The dossier stakes the model on that difference
//! and test 17 asserts it.
//!
//! **Everything on the panel is a switch, and one of them is not
//! calibrated.** OUTPUT LEVEL is −10 to +10 in exact decibels and the
//! twenty-one resistors on the drawing really do deliver them; RECOVERY is
//! marked 1 to 6 with no times, on the drawing or anywhere else, because
//! Waves — who had the console — say the times are "very hard to put in
//! terms of exact milliseconds". The contrast between a calibrated control
//! and one that simply declines to say is most of this panel's
//! personality, and the model keeps it: [`engine::R_LADDER`] is
//! resistances and [`RECOVERY_NAMES`] is the numerals 1 to 6.
//!
//! **OUT is not a bypass.** The mode wafer selects a resistor rather than
//! opening the path, so in OUT the audio still passes through the element
//! and only the control is neutralised. The model gives you a separate
//! true bypass for A/B and marks it as an addition.
//!
//! # What is estimated, in one place
//!
//! More than for any other model in the lab, and the reason is worth
//! stating once: **no factory handbook, no specification and no
//! measurement of any kind has ever been published for this unit.** The
//! Neve model can be calibrated against a manufacturer's own table with
//! the manufacturer's own tolerances. This one has one photographed
//! blueprint, two companies' prose about their own recreations, and
//! arithmetic. Every constant in [`engine`] says which of those it came
//! from, and section 12.6 of the dossier lists eight things it refuses to
//! test because nothing supports a number. That refusal is honoured here:
//! there is no attack-time test, no threshold-in-dBu test, no maximum
//! gain-reduction test and no distortion-at-a-level test.

pub mod element;
pub mod engine;
pub mod oversample;

pub use engine::{Compressor, Settings, VU_REFERENCE_DBFS};

/// Labels of `tg_mode`, in EMI's printed order.
///
/// **From the S1 legend table on drawing TG12413-D101**, which prints
/// "POSITION 1 COMPRESS / 2 OUT / 3 LIMIT". Note the ordering: to get from
/// compress to limit you pass through out, which is a real ergonomic fact
/// and worth reproducing.
pub const MODE_NAMES: [&str; 3] = ["Compress", "Out", "Limit"];
/// The symmetric 62 K / 62 K pairing on the mode wafer.
pub const MODE_COMPRESS: usize = 0;
/// The 82 K / 82 K pairing: the sidechain biased but ineffective. **Not a
/// bypass** — the element stays in circuit.
pub const MODE_OUT: usize = 1;
/// The 120 K / 20 K pairing, asymmetric six to one.
pub const MODE_LIMIT: usize = 2;

/// Labels of `tg_recovery`.
///
/// Numerals and nothing else, because that is what the switch is marked
/// with on Chandler's recreation and in Waves' plug-in, and because no
/// time is printed on EMI's drawing or published anywhere. Putting
/// milliseconds on this control would be inventing a figure.
pub const RECOVERY_NAMES: [&str; 6] = ["1", "2", "3", "4", "5", "6"];

/// Labels of `tg_region`.
pub const REGION_NAMES: [&str; 2] = ["Breakdown", "Forward"];
/// Reverse breakdown, which is what the drawing shows and what three
/// independent sources call a Zener limiter.
pub const REGION_BREAKDOWN: usize = 0;
/// Ordinary forward conduction, the generous reading of the drawing, under
/// which the element is a tanh with twice the Neve's thermal scale.
pub const REGION_FORWARD: usize = 1;

/// Labels of `tg_oversample`.
pub const OVERSAMPLE_NAMES: [&str; 3] = ["1x", "2x", "4x"];

/// The factor each `tg_oversample` position selects.
pub const OVERSAMPLE_FACTORS: [usize; 3] = [1, 2, 4];

/// The oversampling factor for a switch index.
pub fn oversample_factor(i: usize) -> usize {
    OVERSAMPLE_FACTORS[i.min(2)]
}

#[cfg(test)]
mod tests;

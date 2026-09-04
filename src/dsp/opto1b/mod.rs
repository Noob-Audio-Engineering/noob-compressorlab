//! The variable-time optical model of the lab: the Tube-Tech CL 1B.
//!
//! `research/CL-1B.md` is what this is built from, and its section 9.1 is
//! the decision that shapes the whole module: **this engine must not
//! import [`crate::dsp::opto::model::Cell`]**.
//!
//! That is the opposite call from the LA-3A, and the reasoning is its
//! mirror image. The LA-3A shares the LA-2A's actual T4B module in the
//! same role, so duplicating the cell would have been two copies of one
//! physics. The CL 1B shares neither the cell nor the light source, and
//! decisively it does not put its time constants there at all: they live
//! in an op-amp, a 10 µF capacitor and two front-panel pots. Importing
//! `Cell` would import a 60 ms first-stage release, a half-second trap
//! and the programme memory into a machine whose Release knob is supposed
//! to run from 50 ms to ten seconds and whose Manual mode is supposed to
//! have no memory at all. The bottom half of the Release knob would stop
//! doing anything, Fix/Man would stop being distinguishable from Manual,
//! and this would quietly become a third LA-2A with extra knobs.
//!
//! What is shared is real, and it is precisely the general half of the
//! photocell crate, the half that belongs to any photoresistor rather
//! than to a T4: the photoconductor's own odd-order distortion law, and
//! its resistance law through `Photoresistor`, both called out of the
//! crate with this element's own numbers rather than the T4's. Alongside
//! them the lab's own filters, the VU reference and its constants, the
//! stereo link, the denormal flushing and the transfer solver.
//!
//! What is not shared is the cell, its electroluminescent panel law,
//! every one of its time constants, and its `CELL_GAMMA`. That last one
//! is worth naming because the research proposed borrowing it: the power
//! law has the same shape here, but its exponent is solved from the
//! published 2:1 ratio and comes out at 1.36, for the reason at
//! `engine::k::GRE_GAMMA`. Nor are the endpoints shared beyond the dark
//! resistance: this element's conductance scale comes from the service
//! manual's 10 dB calibration and its floor is a separate estimate,
//! which is the pair the crate's `Photoresistor` keeps apart and the T4
//! ties together.
//!
//! | element | LA-2A | LA-3A | CL 1B here |
//! |---|---|---|---|
//! | gain element | T4B cell | the same T4B | an undocumented potted GRE, modelled from its published response |
//! | where the timing lives | in the cell's physics | in the cell's physics | in the sidechain electronics, on two panel knobs |
//! | attack | about 10 ms, fixed | about 1.5 ms, fixed | 0.5 ms to 300 ms, or 1 ms fixed |
//! | release | two-stage with memory | two-stage with memory | a constant-slope ramp, 0.05 s to 10 s, memory only in Fix/Man |
//! | ratio | none | none | 2:1 to 10:1, and it is not really a ratio control |
//! | sidechain shaping | R37 shelf | contour and roll-off | **none**, which is why it hears 50 Hz as well as 1 kHz |
//! | bandwidth | 30 Hz to 15 kHz | 20 Hz to 20 kHz | 5 Hz to 25 kHz |
//!
//! Two things the research could not settle, recorded here because the
//! model has to choose something and a reader deserves to know which
//! numbers are anchored. Nobody outside Lydkraft knows what is inside the
//! GRE, so its drive-to-conductance law is fitted to the service manual's
//! two calibration points rather than derived from a circuit anybody has
//! seen. And the schematic's own component values do not reproduce the
//! published attack range, which the research says plainly it could not
//! resolve; the model therefore takes its *shapes* from the schematic (a
//! logarithmic attack pot, a linear release pot, a ramp-like discharge)
//! and its *numbers* from the manual.
//!
//! There is deliberately **no cell-wear control**, unlike the LA-2A's and
//! the LA-3A's. Lydkraft claim no long-term degradation of the element,
//! owners report units are alike, and nobody has published a contrary
//! observation. Inventing one would be inventing a fact.

pub mod engine;

pub use engine::{Compressor, Settings, VU_REFERENCE_DBFS};

/// Labels of `cl1b_mode`, in the panel's own left-to-right order.
pub const MODE_NAMES: [&str; 3] = ["Fixed", "Fix/Man", "Manual"];
/// Labels of `cl1b_meter`, in the panel's own left-to-right order.
pub const METER_NAMES: [&str; 3] = ["Input", "Compression", "Output"];
/// Labels of `cl1b_bus`. On the hardware this chooses which of two
/// side-chain busses the unit joins; here it selects the stereo link
/// group, which is what Softube did with it.
pub const BUS_NAMES: [&str; 3] = ["Off", "1", "2"];

/// Attack/release select positions.
pub const MODE_FIXED: usize = 0;
pub const MODE_FIXMAN: usize = 1;
pub const MODE_MANUAL: usize = 2;

/// Meter switch positions.
pub const METER_IN: usize = 0;
pub const METER_COMP: usize = 1;
pub const METER_OUT: usize = 2;

/// Number of points in a published lookup table, matching the
/// framework's own.
const TABLE_POINTS: usize = 65;

fn sample(f: impl Fn(f32) -> f32) -> Vec<f32> {
    (0..TABLE_POINTS)
        .map(|i| f(i as f32 / (TABLE_POINTS - 1) as f32))
        .collect()
}

/// The Gain pot's law in dB, sampled for the manifest.
///
/// These four tables are how the panel's real units reach the page
/// without anything reimplementing the pot laws on the other side of the
/// wire. The law lives once, in [`engine`]; the page reads the table.
pub fn gain_table() -> Vec<f32> {
    sample(engine::gain_db)
}

/// The Threshold pot's law in dBu. Descending, which the framework's
/// table interpolation handles.
pub fn threshold_table() -> Vec<f32> {
    sample(engine::threshold_dbu)
}

/// The Attack pot's law in milliseconds.
pub fn attack_table() -> Vec<f32> {
    sample(|p| engine::attack_s(p) * 1e3)
}

/// The Release pot's law in seconds.
pub fn release_table() -> Vec<f32> {
    sample(engine::release_s)
}

#[cfg(test)]
mod tests;

//! The optical model of the lab: the LA-2A. This module owns the grey-box
//! model of the T4 cell, the sidechain and the tube stage ([`model`]) and
//! the small filters it is built from ([`filters`]); the parameter ids,
//! streams and the processor that hosts it live one level up in
//! [`crate::dsp`]. `research/LA-2A.md` documents how the original works and
//! how the model was derived from it.
//!
//! | module | contents |
//! |---|---|
//! | [`model`] | [`Compressor`], [`Cell`], [`Settings`], the static solver behind the transfer curve |
//! | [`filters`] | one-pole, shelf and biquad sections |
//! | this file | the labels of the discrete knobs |

pub mod filters;
pub mod model;

pub use model::{
    CELL_SPEEDS, Cell, CellParams, Compressor, METER_GR, METER_OUT4, METER_OUT10, Settings,
    VU_REF_DBFS, attenuation_for, gr_db_for, makeup_db, resistance_for,
};

/// Labels of `opto_mode`.
pub const MODE_NAMES: [&str; 2] = ["Compress", "Limit"];
/// Labels of `opto_meter`.
pub const METER_NAMES: [&str; 3] = ["Gain Reduction", "Output +10", "Output +4"];
/// Labels of `opto_cell`.
pub const CELL_NAMES: [&str; 3] = ["Silver", "Gray", "LA-2"];

#[cfg(test)]
mod tests;

//! The true-RMS model of the lab: the dbx 160, with the 160A's OverEasy
//! and Infinity+ behind switches.
//!
//! `research/dbx-160.md` is what this is built from; section 10 is the
//! design and section 12 the test plan. It is a deliberate composite and
//! says so: the face, the ballistics and the hard knee are the original
//! 160's, and the two behaviours the original does not have arrive with
//! the 160A panel that `dbx_model` selects (research 2.5).
//!
//! # The two things that make it this box rather than a VCA compressor
//!
//! **The detector reads true RMS, and it is not a rectifier with a slow
//! attack.** It is David Blackmer's log-domain filter: a bilateral log
//! converter whose two diode junctions square the signal for free, a
//! capacitor charged through a junction and discharged by a constant
//! current, and a square root that is never computed because in the log
//! domain it is a division by two. Three consequences follow, and all
//! three are published behaviour. A falling signal is **rate-limited**,
//! decaying a fixed number of decibels per second rather than
//! exponentially. A rising one attacks **faster the bigger the step**,
//! because a bigger step opens the charging junction harder. And the two
//! **cannot be separated**, which is why a dbx 160 has no attack knob and
//! no release knob: THAT Corporation, who still make the descendant part,
//! say in as many words that separate attack and release adjustments are
//! not possible within the constraint of rms response.
//!
//! One number generates all of it. The detector's time constant is
//! [`engine::TAU_DEFAULT_S`], 35.3 ms, and it comes from two components printed
//! on dbx's own drawing which the drawing marks as a factory-matched
//! pair: R35 at 909 kΩ sets the discharge current and C15 at 22 µF is the
//! capacitor. Fed back through the filter's equations that one constant
//! reproduces dbx's published release rate, their three release times,
//! and their three attack times, seven figures with no free parameter
//! left over.
//!
//! **The ratio is one multiplication and the knee is a diode.** In the
//! decibel domain the detector's excess over threshold goes through a
//! rectifier and is then scaled by a single coefficient α before reaching
//! the gain cell, so the ratio is `R = 1/(1 − α)` exactly, with no gain
//! computer and no lookup. Infinity is not a mode: it is where α reaches
//! 1, and the pot passes through it the way it passes through 4:1. Past
//! it α exceeds 1, the cell pulls down more decibels than the input rose,
//! and the ratio goes negative — which dbx trademarked as Infinity+ and
//! which needed no new circuit, only a longer pot.
//!
//! The knee is the rectifier's diode. Inside an operational amplifier's
//! feedback loop its softness is divided by the open-loop gain and the
//! corner collapses to well under a thousandth of a decibel, which is the
//! original's hard knee and is why dbx could advertise it as
//! mathematically precise. Moved outside the loop the diode's own
//! exponential is exposed and becomes the knee, which is OverEasy. So one
//! function serves both and the button is a knee-width switch; see
//! [`engine::gain_reduction_db`].
//!
//! # What the panel says and what the box does
//!
//! **The ∞ mark is 120:1.** dbx published the number twice in one manual
//! and it is not infinity: "continuously variable from 1:1 to 120:1
//! (infinity)". The model leaves the residual slope in
//! ([`ALPHA_CEILING`]), so 40 dB of input rise above threshold still
//! lifts the output by a third of a decibel. A model that clamps to a
//! brick wall there has modelled the silkscreen rather than the circuit.
//!
//! **Two units, two pots, one parameter.** The original's THRESHOLD runs
//! 10 mV to 3 V, which is −37.8 to +11.8 dBu, while the 160A's runs −40
//! to +20 dBu; the original's COMPRESSION stops at ∞ where the 160A's
//! carries on to −1:1. The parameters carry the union of each pair, and
//! each faceplate maps its own pot's rotation onto the part of that range
//! its hardware has, so both dials are exact and neither unit gains a
//! range dbx did not give it. The engine clamps to the selected unit's
//! range as well, so a preset written on one face cannot smuggle the
//! other's range in ([`Settings::clamped`]).
//!
//! # Layout
//!
//! | module | contents |
//! |---|---|
//! | [`engine`] | the Blackmer cell, the log-domain detector, the static curve, the unit |
//! | this file | the constants, the dial laws, the switch labels and [`Settings`] |

pub mod engine;

pub use engine::{BlackmerCell, Compressor, RmsDetector, Settings};

/// Labels of `dbx_model`.
pub const MODEL_NAMES: [&str; 2] = ["160", "160A"];
/// The original, 1976: wood cheeks, a VU meter, a hard knee and nothing
/// past the ∞ mark.
pub const MODEL_160: usize = 0;
/// The current generation: black panel, two LED rows, OverEasy and
/// Infinity+.
pub const MODEL_160A: usize = 1;

/// Labels of `dbx_knee`.
pub const KNEE_NAMES: [&str; 2] = ["Hard", "OverEasy"];
/// The original's knee, and the 160A's HARD KNEE position.
pub const KNEE_HARD: usize = 0;
/// The 160A's OverEasy position. Not available on the original: OverEasy
/// is Leslie Tyler's 1978 invention, three years after the 160 shipped,
/// and dbx's own copy calls the hard knee "the classic 'Hard Knee' curve
/// popularized by the original dbx 160, 161 and 162".
pub const KNEE_OVEREASY: usize = 1;

/// Labels of `dbx_meter`.
///
/// These are the original's three METER push buttons. The 160A has no
/// such switch: its DISPLAY button moves the 19-LED row between the input
/// and the output level while the 12-LED row always shows gain reduction,
/// so that face drives the first two positions of this parameter and
/// treats the third as Output.
pub const METER_NAMES: [&str; 3] = ["Input", "Output", "Gain Change"];
/// INPUT LEVEL.
pub const METER_INPUT: usize = 0;
/// OUTPUT LEVEL.
pub const METER_OUTPUT: usize = 1;
/// GAIN CHANGE, the control voltage, which is the default the manual
/// suggests and the position the box is famous for being watched in.
pub const METER_GAIN_CHANGE: usize = 2;

// ---------------------------------------------------------------- levels

/// Level in dBu that 0 dBFS RMS stands for, at the default headroom.
///
/// **This is 22 and the research's table says 18.** The lab calibrates
/// every meter it has at 0 VU = +4 dBu = −18 dBFS RMS, so 0 dBFS RMS is
/// +22 dBu by arithmetic, and a model that mapped dBu to dBFS 4 dB
/// differently would put its own VU meter 4 dB out of step with the other
/// six faces for no gain. The range the research asks for is kept.
pub const HEADROOM_DEFAULT_DB: f32 = 22.0;
/// Lower end of `dbx_headroom` (research 10.9).
pub const HEADROOM_MIN_DB: f32 = 4.0;
/// Upper end of `dbx_headroom`.
pub const HEADROOM_MAX_DB: f32 = 28.0;

/// Lowest threshold the parameter carries, dBu: the 160A's published end.
pub const THRESHOLD_MIN_DBU: f32 = -40.0;
/// Highest, dBu: the 160A's published end.
pub const THRESHOLD_MAX_DBU: f32 = 20.0;

/// The original's THRESHOLD pot, fully anticlockwise: 10 mV, which its
/// specification prints as −38 dB and which is −37.78 dBu exactly.
pub const THRESHOLD_160_MIN_DBU: f32 = -37.78;
/// The original's THRESHOLD pot, fully clockwise: 3 V, printed +12 dB.
pub const THRESHOLD_160_MAX_DBU: f32 = 11.76;

/// The six voltages printed round the original's THRESHOLD dial, in the
/// order the drawing prints them anticlockwise to clockwise.
pub const THRESHOLD_MARK_LABELS: [&str; 6] = ["10mv", "30mv", "100mv", "300mv", "1V", "3V"];

/// Those six marks in dBu, `20·log10(V / 0.775)`.
///
/// dbx's factory procedure steps an oscillator "up and down in 10 db
/// steps verifying that the threshold level matches the input signal at
/// successive calibration marks", so these are calibration points rather
/// than decoration, and the dial is linear in decibels between them.
pub const THRESHOLD_MARK_DBU: [f32; 6] = [-37.78, -28.25, -17.78, -8.25, 2.22, 11.76];

/// Make-up gain, decibels either side of unity. dbx print ±20 dB on every
/// model in the family and mark the ends of R80's track "−20db" and
/// "+20db" on the drawing itself.
pub const OUTPUT_MIN_DB: f32 = -20.0;
pub const OUTPUT_MAX_DB: f32 = 20.0;

/// Rear-panel METER CALIBRATION trimmer, lowest 0 VU, dBu: the 160A's
/// −15 dBu (138 mV).
pub const METER_CAL_MIN_DBU: f32 = -15.0;
/// Highest, +10 dBu (2.45 V).
pub const METER_CAL_MAX_DBU: f32 = 10.0;
/// Factory setting: "The meter in the 160 and 161 is factory calibrated
/// to read '0' at +4dB (1.23V) output level."
pub const METER_CAL_DEFAULT_DBU: f32 = 4.0;

/// Where the gain-reduction display runs out, dB. dbx: "The lower row of
/// 12 LEDs displays up to 40dB of GAIN REDUCTION (although the 160A is
/// actually capable of delivering up to 60dB of gain reduction)."
pub const METER_GR_MAX_DB: f32 = 40.0;

// ----------------------------------------------------------- the ratio

/// The nine ratios printed round the original's COMPRESSION dial.
pub const RATIO_MARK_LABELS: [&str; 9] = ["1", "1.5", "2", "3", "4", "6", "10", "20", "∞"];

/// Those nine as the coefficient the circuit actually sets, `α = 1 − 1/R`.
///
/// This is the whole ratio law. Both the detector and the cell are
/// logarithmic with the same 6.1 mV/dB constant, so a volt is a decibel
/// everywhere in the sidechain and the COMPRESSION pot is simply the
/// fraction of the rectifier's output that reaches the control port.
pub const RATIO_MARK_ALPHA: [f32; 9] = [
    0.0,
    1.0 / 3.0,
    0.5,
    2.0 / 3.0,
    0.75,
    5.0 / 6.0,
    0.9,
    0.95,
    1.0,
];

/// Where those nine marks sit on the original's dial, as a fraction of
/// its own travel.
///
/// **Measured**, off dbx's own front-panel figure from the 160/161 instruction manual (archive.org/details/dbx_dbx_160, leaf 3), dbx's own front-panel
/// drawing enlarged. The knob's circle was found by scoring rings against
/// the drawing (centre 664, 462; radius 80 px, a perfect fit), and the
/// ticks were then read as angular clusters of dark pixels in the annulus
/// just outside it, at half-degree resolution. Separating them from the
/// figure's callout arrow needed a second pass on a wider annulus, where
/// only the arrow's shaft survives: it lies at 330° and the tick beside
/// it, at 316°, is the 3:1 mark.
///
/// The marks run from −155.2° to +150.5° about twelve o'clock, a sweep of
/// 305.7°, and the fractions below are each mark's share of it. The
/// research's own estimate of this table was taken from the *label*
/// centroids rather than the ticks and is quoted at ±0.03 of travel;
/// these agree with it everywhere except at the 3:1 and 4:1 marks, where
/// the labels sit noticeably off-radius.
///
/// dbx shaped this pot deliberately — "scale expansion at the subtle
/// lower ratios for easy, repeatable settings" — because α crams
/// everything above 4:1 into its last quarter. So unlike the threshold
/// and output dials, this taper is real and not drawing error, and it is
/// interpolated rather than fitted.
pub const RATIO_MARK_TRAVEL: [f32; 9] = [
    0.0, 0.099_45, 0.194_30, 0.363_76, 0.507_03, 0.698_07, 0.857_70, 0.939_48, 1.0,
];

/// Fraction of the *parameter's* travel at which α reaches 1, i.e. the
/// ∞:1 mark.
///
/// The parameter carries both dials, so the original's nine marks occupy
/// the part below this and the 160A's four negative marks the part above.
/// **Estimate**: the research reads dbx's 160A photograph as putting
/// α = 0 → 1 over "roughly five-sixths of the travel" with the negative
/// fan in the last sixth. The 160A's marks are printed labels with no
/// ticks, so unlike the original's dial there is nothing on that
/// photograph to measure, and α is taken as linear from 1 to 2 across
/// that last sixth for want of any evidence about its shape.
pub const INFINITY_TRAVEL: f32 = 5.0 / 6.0;

/// The largest α the pot can reach in the positive direction.
///
/// dbx publish the ∞ mark as **120:1**, twice: "continuously variable
/// from 1:1 to 120:1 (infinity)" and "infinite compression (approximately
/// 120:1)". `1 − 1/120` leaves a residual slope of one decibel out for
/// every 120 in, which is inaudible on programme and is the difference
/// between modelling the circuit and modelling the label.
pub const ALPHA_CEILING: f32 = 1.0 - 1.0 / 120.0;

/// Deepest gain reduction the model will apply, dB: dbx's ">60 dB maximum
/// compression".
pub const GR_MAX_DB: f32 = 60.0;

/// α for a fraction of the ratio pot's travel.
pub fn alpha_for_travel(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t >= INFINITY_TRAVEL {
        let u = (t - INFINITY_TRAVEL) / (1.0 - INFINITY_TRAVEL);
        1.0 + u
    } else {
        let u = t / INFINITY_TRAVEL;
        interp(&RATIO_MARK_TRAVEL, &RATIO_MARK_ALPHA, u)
    }
}

/// The inverse: where on the pot's travel a coefficient sits. The faces
/// use it to place their printed marks.
pub fn travel_for_alpha(alpha: f32) -> f32 {
    let a = alpha.clamp(0.0, 2.0);
    if a >= 1.0 {
        INFINITY_TRAVEL + (a - 1.0) * (1.0 - INFINITY_TRAVEL)
    } else {
        interp(&RATIO_MARK_ALPHA, &RATIO_MARK_TRAVEL, a) * INFINITY_TRAVEL
    }
}

/// Linear interpolation of `ys` against a monotonically rising `xs`.
fn interp(xs: &[f32], ys: &[f32], x: f32) -> f32 {
    if x <= xs[0] {
        return ys[0];
    }
    for i in 1..xs.len() {
        if x <= xs[i] {
            let span = xs[i] - xs[i - 1];
            let f = if span > 0.0 {
                (x - xs[i - 1]) / span
            } else {
                0.0
            };
            return ys[i - 1] + (ys[i] - ys[i - 1]) * f;
        }
    }
    ys[ys.len() - 1]
}

/// The ratio a coefficient really is: `R = 1/(1 − α)`, which runs through
/// infinity at α = 1 and comes back negative beyond it.
pub fn ratio_for_alpha(alpha: f32) -> f32 {
    1.0 / (1.0 - alpha)
}

/// What the panel would print for a coefficient. Used by the plug-in's
/// host display and by the faces.
pub fn ratio_label(alpha: f32) -> String {
    let d = 1.0 - alpha;
    if d.abs() < 1.0 / 999.0 {
        return "∞:1".to_string();
    }
    let r = 1.0 / d;
    let n = if r.abs() < 10.0 {
        format!("{r:.1}")
    } else {
        format!("{r:.0}")
    };
    // The panel prints a proper minus, not a hyphen.
    format!("{}:1", n.replace('-', "\u{2212}"))
}

/// Points the parameter's sampled taper is built from. The framework
/// interpolates between them, so this only has to be fine enough that the
/// nine measured marks land where they were measured.
const TAPER_POINTS: usize = 129;

/// The ratio pot's law as the manifest and the host see it: α against
/// travel.
pub fn ratio_table() -> Vec<f32> {
    (0..TAPER_POINTS)
        .map(|i| alpha_for_travel(i as f32 / (TAPER_POINTS - 1) as f32))
        .collect()
}

#[cfg(test)]
mod tests;

//! The resistances around the TG's gain element, and the node solve.
//!
//! The element itself is a shared component,
//! `noob-electrical-components-diode-arm-pair`, re-exported here as
//! [`DiodeArmPair`]. It carries equation (G1) of `research/TG12413.md` —
//! *n* junctions per arm with a bulk resistance, two arms in opposition —
//! together with the constants and the argument for why it is a separate
//! part from the diode bridge the Neve model uses. Read it there; nothing
//! about the part is restated here.
//!
//! What is here is the machine: **R14**, the 20 kΩ series arm the source
//! drives the element through, the divider those two make, and the Newton
//! solve of that divider's node equation. Those are the plug-in's because
//! they are the plug-in's: R14 is a resistor on EMI's drawing, the divider
//! is this module's gain structure, and a different unit built on the same
//! part would put different resistors around it. The Neve model draws the
//! same line at the same place, in `super::super::bridge::engine::Network`.
//!
//! There is no [`Network`] for the Neve's ring, because nothing puts R14
//! around a ring. The identity that test 8 asserts is between two *laws*,
//! so it takes [`DiodeArmPair::ring`] and needs no divider at all.
//!
//! # What the shared crate is used for and what it is not
//!
//! The plug-in takes the law, its slope, the small-signal resistance and
//! the inverse of that resistance. It does **not** take a thermal scale, a
//! solver or a divider from the diode-bridge crate, and the empty
//! right-hand column of the dossier's constants table in 11.6 is still
//! empty: the finding that the TG takes nothing from the bridge survived
//! the extraction, because what it now takes comes from its own component.

pub use noob_electrical_components::diode_arm_pair::{
    CURRENT_FLOOR, DiodeArmPair, JUNCTION_SCALE, N_JUNCTIONS,
};

/// R14, the series arm the source drives the element through, in ohms.
///
/// Read off TG12413-D101. This is the resistance that turns the element
/// into a divider, and it is the machine rather than the part, which is
/// why it is here and not in the component crate.
pub const R_SERIES: f32 = 20_000.0;

/// How close to an arm's bias current the Newton iterate may come.
///
/// The dossier asks for `|i| < I·(1 − 1e−6)`. The component applies the
/// same guard inside its own law; this one brackets the iterate, so that a
/// Newton step cannot walk outside the range the law is defined on and
/// come back through the clamp instead of through the arithmetic.
const HEADROOM: f32 = 1e-6;

/// The element as a shunt across a divider: R14, and the part it feeds.
///
/// The source drives the element through [`R_SERIES`] and the output is
/// the voltage across the element, so **no control current means unity
/// gain** and a large one shorts the signal away. Note what that makes of
/// the distortion: an element carrying no current cannot bend a waveform,
/// so this network is transparent when it is not working and dirtiest when
/// it is working hardest, which is the opposite of the Neve's bridge and
/// is the difference the dossier's section 9.2 stakes the model on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Network {
    /// The series arm, in ohms.
    pub r_series: f32,
    /// The gain element itself.
    pub element: DiodeArmPair,
}

impl Default for Network {
    /// The TG as drawn: two junctions per arm, in breakdown.
    fn default() -> Self {
        Network::breakdown()
    }
}

impl Network {
    /// R14 around a given element.
    pub fn new(element: DiodeArmPair) -> Self {
        Network {
            r_series: R_SERIES,
            element,
        }
    }

    /// The forward reading: `n` junctions per arm, no bulk term.
    ///
    /// `n = 2` is the TG under reading B of the dossier's section 4.3,
    /// which gives `i = I·tanh(u / 4ηV_T)` — the same function as the
    /// Neve's with the thermal scale doubled, and therefore four times
    /// less third harmonic at equal drive.
    pub fn forward(n: u32) -> Self {
        Network::new(DiodeArmPair::forward(n))
    }

    /// The breakdown reading, which is what the drawing shows.
    pub fn breakdown() -> Self {
        Network::new(DiodeArmPair::breakdown())
    }

    /// The divider's gain, 1 with the element open.
    #[inline]
    pub fn gain(&self, i_bias: f32) -> f32 {
        let r = self.element.resistance(i_bias);
        if r.is_finite() {
            r / (self.r_series + r)
        } else {
            1.0
        }
    }

    /// Gain reduction in dB, positive, for a bias current.
    pub fn gr_db(&self, i_bias: f32) -> f32 {
        -20.0 * self.gain(i_bias).log10()
    }

    /// The bias current giving `gr` dB of reduction, or `None` when the
    /// element's bulk floor puts that depth out of reach.
    ///
    /// Closed form, because the divider is and so is the element's
    /// `r = 2·r_b + 2·V_n/I`. The `None` comes from the component: below
    /// `2·r_b` there is no current that will do it, which is the floor on
    /// gain reduction that breakdown operation implies and that the
    /// forward reading does not have.
    pub fn current_for_gr_db(&self, gr: f32) -> Option<f32> {
        if gr <= 0.0 {
            return Some(0.0);
        }
        let a = 10f32.powf(-gr / 20.0);
        let r = a * self.r_series / (1.0 - a);
        self.element.current_for_resistance(r)
    }

    /// Solve the node equation for the voltage across the element.
    ///
    /// `(v_s − u) / R_s = i` with `u = u(i)` from (G1) is implicit in `i`,
    /// so this takes the linear seed the small-signal resistance gives and
    /// runs `steps` Newton corrections. One step is enough over the
    /// working range; the engine uses two when the drive control is past
    /// half, which is where the element is being pushed towards its
    /// asymptote on purpose.
    ///
    /// This is the caller's job rather than the component's, because R14
    /// is the machine and only the element is the part. Note also which
    /// variable it solves in: the bridge crate's law is explicit in
    /// voltage and its network solves in `u`, while (G1) is explicit in
    /// current and this solves in `i`. The two solvers look alike and are
    /// not the same code, which is one more reason the two parts are two
    /// crates.
    #[inline]
    pub fn solve(&self, v_s: f32, i_bias: f32, steps: u32) -> f32 {
        if i_bias.is_nan() || i_bias <= CURRENT_FLOOR {
            return v_s;
        }
        let (a, b) = (i_bias * (1.0 + self.m()), i_bias * (1.0 - self.m()));
        let lo = -a * (1.0 - HEADROOM);
        let hi = b * (1.0 - HEADROOM);
        let r0 = self.element.resistance(i_bias);
        let mut i = (v_s / (self.r_series + r0)).clamp(lo, hi);
        for _ in 0..steps.max(1) {
            let f = self.element.voltage(i, i_bias) - (v_s - self.r_series * i);
            let fp = self.element.slope(i, i_bias) + self.r_series;
            i = (i - f / fp).clamp(lo, hi);
        }
        v_s - self.r_series * i
    }

    /// The element's clamped imbalance, which the solve's brackets need.
    #[inline]
    fn m(&self) -> f32 {
        self.element.mismatch.clamp(0.0, 0.95)
    }
}

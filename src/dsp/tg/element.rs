//! The TG12413's gain element: equation (G1) of `research/TG12413.md`.
//!
//! Four HS2051 diodes in **two branches of two in series**, both the same
//! way up, sharing the +20 V rail as their common node. That is not the
//! Neve's ring, which is four diodes with two *floating* common nodes and
//! one junction per arm, and section 4.5 of the dossier lists six
//! structural differences between them rather than six different values.
//! So this element is built here rather than taken from
//! `noob-electrical-components-diode-bridge`, and the empty right-hand
//! column of the dossier's constants table in 11.6 is the finding.
//!
//! # The law
//!
//! Model each arm as *n* junctions in series with a bulk resistance, two
//! arms in opposition across the differential audio, biased at *I* each
//! and carrying `I + i` and `I − i`. Then
//!
//! ```text
//! u(i) = 2·r_b·i + 2·V_n·artanh( i / I )        (G1)
//! ```
//!
//! and three circuits fall out of it by choosing two constants:
//!
//! | circuit | n | V_n | r_b | reduces to |
//! |---|---|---|---|---|
//! | Neve ring, forward | 1 | η·V_T | 0 | `i = I·tanh(u / 2ηV_T)` |
//! | TG, forward | 2 | 2·η·V_T | ≈0 | `i = I·tanh(u / 4ηV_T)` |
//! | TG, breakdown | 2 | knee scale, **estimate** | **> 0** | a soft knee onto a resistive floor |
//!
//! [`Element::ring`] is the first row and it exists for exactly one
//! reason: test 8 asserts that it reproduces the shipped crate's law to
//! 1 × 10⁻⁹ relative. That is the whole argument of the dossier's section
//! 4.9 made executable — (G1) is the correct generalisation, and a crate
//! with the constant baked in cannot serve this unit.
//!
//! # Mismatch, and why the implementation is a logarithm rather than an
//! `artanh`
//!
//! EMI specify D1/D3 and D2/D4 as matched pairs on two separate drawings
//! and provide two adjust-on-test resistors to trim what is left, so the
//! balance between the two arms is a thing the factory adjusted by hand
//! and a thing that can be out. Writing the law as
//!
//! ```text
//! u(i) = 2·r_b·i + V_n·ln( (I_a + i) / (I_b − i) )
//! ```
//!
//! carries the two arm currents separately, becomes (G1) exactly when
//! `I_a == I_b`, and gives the even harmonics that an unbalanced pair
//! really does make. The `artanh` form cannot express it at all.
//!
//! # What is not modelled
//!
//! Temperature, junction capacitance, reverse recovery and the element's
//! own noise, all following the dossier's section 4.10. Temperature is the
//! interesting exclusion: a forward junction and a zener below 5 V and an
//! avalanche device above 6 V have three different signs of coefficient,
//! and since nobody has a datasheet for an HS2051 the sign is unknown.
//! Modelling it would mean inventing it.

use noob_electrical_components::diode_bridge as ring;

/// η·V_T for **one** junction, in volts: 45.4 mV.
///
/// Taken from the shipped crate's own fitted ideality and thermal voltage
/// rather than restated here, so that test 8's identity is exact against
/// the thing it is an identity with. Both are **estimates** — the crate's
/// note says they were fitted to a 1N4148, and neither HBX 31 nor HS2051
/// has a reachable datasheet.
pub const JUNCTION_SCALE: f32 = ring::IDEALITY * ring::THERMAL_VOLTAGE;

/// Junctions per arm on the TG: four diodes, two branches (dossier 4.2).
pub const N_JUNCTIONS: u32 = 2;

/// R14, the series arm the source drives the element through, in ohms.
///
/// Read off TG12413-D101. This is the resistance that turns the element
/// into a divider, and it is the machine rather than the part.
pub const R_SERIES: f32 = 20_000.0;

/// R16, in ohms, and the default for `r_b` in the breakdown region.
///
/// **A hint, not a measurement.** Two adjust-on-test resistors sit in
/// parallel opposite this fixed 24 Ω on the other branch, which is what
/// you build when you are trimming the balance between two branches that
/// must carry the same current. A device in breakdown presents a bulk
/// resistance in the ohms to tens of ohms, and you trim against ohms
/// because ohms is what the element presents (dossier 4.7).
pub const R_BALANCE: f32 = 24.0;

/// The knee scale of one arm in reverse breakdown, in volts.
///
/// **Estimate with no source at all.** The forward figure follows from
/// the crate's fitted η and V_T; this one does not follow from anything,
/// because breakdown is tunnelling below about 5 V and avalanche above
/// about 6 V and neither is the diode exponential. 120 mV is the
/// dossier's starting value in 11.6 and it is a calibration knob.
pub const V_N_BREAKDOWN: f32 = 0.120;

/// Below this control current the element is treated as an open circuit.
///
/// At 1 pA the element's resistance is 2.4 × 10¹¹ Ω, which is 140 dB above
/// the series arm, so the divider is unity to far better than f32 can
/// represent and the linear seed would only lose precision.
pub const CURRENT_FLOOR: f32 = 1e-12;

/// How close to an arm's bias current the signal current may come.
///
/// The dossier asks for `|i| < I·(1 − 1e−6)`; the logarithm form needs the
/// same guard on each arm separately, because with mismatch the two ends
/// are not at the same place.
const HEADROOM: f32 = 1e-6;

/// The two-branch diode gain element, as a shunt across a divider.
///
/// The source drives it through [`Element::r_series`] and the output is
/// the voltage across the element, so **no control current means unity
/// gain** and a large one shorts the signal away. Note what that makes of
/// the distortion: an element carrying no current cannot bend a waveform,
/// so this element is transparent when it is not working and dirtiest when
/// it is working hardest, which is the opposite of the Neve's bridge and
/// is the difference the dossier's section 9.2 stakes the model on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Element {
    /// `V_n` of (G1): the knee scale of one arm, in volts.
    pub v_n: f32,
    /// `r_b` of (G1): the bulk resistance of one arm, in ohms.
    pub r_b: f32,
    /// The series arm, in ohms.
    pub r_series: f32,
    /// Arm imbalance as a fraction of the bias current, 0 to 1: the arms
    /// carry `I·(1 + m)` and `I·(1 − m)`.
    pub mismatch: f32,
}

impl Default for Element {
    /// The TG as drawn: two junctions per arm, in breakdown.
    fn default() -> Self {
        Element::breakdown()
    }
}

impl Element {
    /// Neve's ring: one junction per arm, forward, no bulk term.
    ///
    /// Not used by the engine. It exists so that test 8 can assert that
    /// (G1) contains the shipped crate's law exactly.
    pub fn ring() -> Self {
        Element {
            v_n: JUNCTION_SCALE,
            r_b: 0.0,
            r_series: R_SERIES,
            mismatch: 0.0,
        }
    }

    /// The forward reading: `n` junctions per arm, no bulk term.
    ///
    /// `n = 2` is the TG under reading B of the dossier's section 4.3,
    /// which gives `i = I·tanh(u / 4ηV_T)` — the same function as the
    /// Neve's with the thermal scale doubled, and therefore four times
    /// less third harmonic at equal drive.
    pub fn forward(n: u32) -> Self {
        Element {
            v_n: n as f32 * JUNCTION_SCALE,
            r_b: 0.0,
            r_series: R_SERIES,
            mismatch: 0.0,
        }
    }

    /// The breakdown reading, which is what the drawing shows.
    pub fn breakdown() -> Self {
        Element {
            v_n: V_N_BREAKDOWN,
            r_b: R_BALANCE,
            r_series: R_SERIES,
            mismatch: 0.0,
        }
    }

    /// The two arm currents for a bias current, largest first.
    #[inline]
    fn arms(&self, i_bias: f32) -> (f32, f32) {
        let m = self.mismatch.clamp(0.0, 0.95);
        (i_bias * (1.0 + m), i_bias * (1.0 - m))
    }

    /// (G1): the differential voltage across the element for a signal
    /// current `i` at bias `i_bias`.
    ///
    /// With `mismatch == 0` this is `2·r_b·i + 2·V_n·artanh(i/I)` to the
    /// last bit, since `ln((I+i)/(I−i)) == 2·artanh(i/I)`.
    #[inline]
    pub fn voltage(&self, i: f32, i_bias: f32) -> f32 {
        let (a, b) = self.arms(i_bias);
        let num = (a + i).max(a * HEADROOM);
        let den = (b - i).max(b * HEADROOM);
        2.0 * self.r_b * i + self.v_n * (num / den).ln()
    }

    /// `du/di`, which the Newton step needs.
    #[inline]
    pub fn slope(&self, i: f32, i_bias: f32) -> f32 {
        let (a, b) = self.arms(i_bias);
        let num = (a + i).max(a * HEADROOM);
        let den = (b - i).max(b * HEADROOM);
        2.0 * self.r_b + self.v_n * (1.0 / num + 1.0 / den)
    }

    /// The small-signal resistance the element presents, in ohms.
    ///
    /// `2·r_b + 2·V_n / I` when the arms are matched. **The `2·r_b` term
    /// is a floor**, so the divider's loss is bounded and gain reduction
    /// stops increasing however hard the sidechain is driven. That is a
    /// property of breakdown operation and it is the mechanism the
    /// dossier offers for "not a brick-wall limiter: transients are
    /// expected to pass"; test 16 asserts it, and asserts that the
    /// forward reading with `r_b = 0` has no such floor.
    #[inline]
    pub fn resistance(&self, i_bias: f32) -> f32 {
        // The NaN test is not decoration: a control current that has gone
        // non-finite must leave the element open rather than fall through
        // to a logarithm, because `NaN <= x` is false on its own.
        if i_bias.is_nan() || i_bias <= CURRENT_FLOOR {
            return f32::INFINITY;
        }
        self.slope(0.0, i_bias)
    }

    /// The divider's gain, 1 with the element open.
    #[inline]
    pub fn gain(&self, i_bias: f32) -> f32 {
        let r = self.resistance(i_bias);
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
    /// bulk floor puts that depth out of reach.
    ///
    /// Closed form, because `r = 2·r_b + 2·V_n/I` is.
    pub fn current_for_gr_db(&self, gr: f32) -> Option<f32> {
        if gr <= 0.0 {
            return Some(0.0);
        }
        let a = 10f32.powf(-gr / 20.0);
        let r = a * self.r_series / (1.0 - a);
        let m = self.mismatch.clamp(0.0, 0.95);
        let top = 2.0 * self.v_n / (1.0 - m * m);
        let bottom = r - 2.0 * self.r_b;
        if bottom <= 0.0 {
            None
        } else {
            Some(top / bottom)
        }
    }

    /// Solve the node equation for the voltage across the element.
    ///
    /// `(v_s − u) / R_s = i` with `u = u(i)` from (G1) is implicit in `i`,
    /// so this takes the linear seed the small-signal resistance gives and
    /// runs `steps` Newton corrections. One step is enough over the
    /// working range; the engine uses two when the drive control is past
    /// half, which is where the element is being pushed towards its
    /// asymptote on purpose.
    #[inline]
    pub fn solve(&self, v_s: f32, i_bias: f32, steps: u32) -> f32 {
        if i_bias.is_nan() || i_bias <= CURRENT_FLOOR {
            return v_s;
        }
        let (a, b) = self.arms(i_bias);
        let lo = -a * (1.0 - HEADROOM);
        let hi = b * (1.0 - HEADROOM);
        let r0 = self.resistance(i_bias);
        let mut i = (v_s / (self.r_series + r0)).clamp(lo, hi);
        for _ in 0..steps.max(1) {
            let f = self.voltage(i, i_bias) - (v_s - self.r_series * i);
            let fp = self.slope(i, i_bias) + self.r_series;
            i = (i - f / fp).clamp(lo, hi);
        }
        v_s - self.r_series * i
    }
}

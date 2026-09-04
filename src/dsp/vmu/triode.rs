//! The remote-cutoff triode, kept in one place so it can be lifted out
//! whole.
//!
//! **Why this is a separate file and not a component crate.** The gain
//! element of a variable-mu limiter is a strong candidate for
//! `noob-electrical-components`, and `research/Fairchild-670.md` section 12
//! argues the case at length. It is deliberately **not** extracted: on the
//! rule that repository now uses, a part is admitted when two units are
//! documented to contain it, and today this has one built user and one
//! predicted one — which is exactly the footing the diode bridge was
//! admitted on, and the bridge's predicted second user turned out not to
//! contain one. So the part lives here, with nothing above it in this file
//! but the tube itself, and moving it later is a rename.
//!
//! **It is not the 610's triode with different numbers.** The 610 preamp's
//! stage in [`crate::dsp::pre`] was fitted for 12AX7-class valves, which
//! have no remote-cutoff characteristic: their plate current collapses over
//! a few volts of grid and their amplification factor barely moves on the
//! way. A 6386 is an automatic-gain-control tube whose grid is wound with
//! varying pitch, so it switches off progressively over tens of volts and
//! its mu is a function of bias rather than a number. Raffensperger says so
//! in as many words — *"Existing triode models were designed for tubes like
//! the 12AX7 which do not have the remote cutoff characteristic of the
//! 6386"* [18] — and the difference is in the functional form, not the
//! parameters (dossier 4.1).
//!
//! The law is Raffensperger's eight-parameter fit to General Electric's
//! published curves (dossier 4.3, constants 10.4):
//!
//! ```text
//!                  p1 · Vak^p2
//! Ia = ───────────────────────────────────────────
//!      (p3 − p4·Vgk)^p5 · [ p6 + exp(p7·Vak − p8·Vgk) ]
//! ```
//!
//! Grid current is assumed negligible, which holds while the grid stays
//! negative; the expression also diverges at `Vgk = +5 V`, where
//! `(p3 − p4·Vgk)` reaches zero, so the grid voltage is clamped well below
//! that ([`VGK_CLAMP`]).

/// Raffensperger's fitted parameters for the GE 6386 (dossier 10.4, all
/// marked **R**: they are his published values).
const P1: f32 = 3.981e-8;
const P2: f32 = 2.383;
const P3: f32 = 0.5;
const P4: f32 = 0.1;
const P5: f32 = 1.8;
const P6: f32 = 0.5;
const P7: f32 = -0.039_22;
const P8: f32 = 0.2;

/// Highest grid-to-cathode voltage the law is evaluated at.
///
/// `(p3 − p4·Vgk)` reaches zero at `Vgk = +5 V` and the expression blows up
/// there, so the fit is only meaningful for a negative grid. The clamp is
/// the dossier's (10.4): −0.5 V, which is five and a half volts below the
/// singularity and still above anything the Fairchild's grids reach, since
/// the standing bias sits some twenty volts down.
pub const VGK_CLAMP: f32 = -0.5;

/// Grid-to-plate capacitance of one section, pF (GE datasheet ET-T1113).
pub const C_GRID_PLATE_PF: f32 = 1.2;
/// Input capacitance of one section, pF (GE ET-T1113).
pub const C_INPUT_PF: f32 = 2.0;
/// Output capacitance of one section, pF (GE ET-T1113).
pub const C_OUTPUT_PF: f32 = 1.1;

/// A remote-cutoff triode section: a pure function of grid and plate
/// voltage, with no state at all.
///
/// The two constructors are the two parts that exist. `grid_scale` and
/// `grid_offset` stretch the grid axis so that a tube with the same plate
/// current and a different transconductance at the same operating point can
/// be expressed without refitting: the published difference between the two
/// is exactly that (see [`Self::jj_6386_lgp`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RemoteCutoffTriode {
    grid_scale: f32,
    grid_offset: f32,
}

impl Default for RemoteCutoffTriode {
    fn default() -> Self {
        Self::ge_6386()
    }
}

impl RemoteCutoffTriode {
    /// The General Electric 6386, which is what Fairchild fitted and what
    /// Raffensperger fitted the law to.
    pub const fn ge_6386() -> Self {
        RemoteCutoffTriode {
            grid_scale: 1.0,
            grid_offset: 0.0,
        }
    }

    /// The JJ Electronic 6386 LGP, the modern replacement.
    ///
    /// JJ publish typical characteristics at the same operating point GE
    /// use — `Ua = 100 V, Rk = 200 Ω, Ia = 9.6 mA` — with `S = 3 mA/V`
    /// against GE's 4 mA/V and `μ = 18` against 17. So the two parts carry
    /// the **same plate current at the same bias** and differ in the
    /// **slope** by a factor of 0.75, which is 2.5 dB. A tube with the same
    /// current and three quarters of the transconductance is the same curve
    /// stretched along the grid axis by 0.75, with the offset chosen to
    /// leave the operating point where it was:
    ///
    /// ```text
    /// 0.75 · (−1.92 V) + offset = −1.92 V   →   offset = −0.48 V
    /// ```
    ///
    /// That is an assumption about the *shape* away from the one published
    /// point, and it is stated rather than measured; what it reproduces
    /// exactly is the published transconductance ratio and the published
    /// plate current at the point where both are quoted.
    pub const fn jj_6386_lgp() -> Self {
        RemoteCutoffTriode {
            grid_scale: 0.75,
            grid_offset: -0.48,
        }
    }

    /// Anode current of one section, in amps.
    #[inline]
    pub fn anode_current(&self, vgk: f32, vak: f32) -> f32 {
        self.slopes(vgk, vak).0
    }

    /// Anode current and both its partial derivatives:
    /// `(Ia, ∂Ia/∂Vgk, ∂Ia/∂Vak)`.
    ///
    /// One evaluation gives all three, because the derivatives share every
    /// expensive term with the current. The cathode solve in the engine
    /// wants both slopes at the same point, so returning them together
    /// halves the transcendental count of the inner loop.
    ///
    /// Above the clamp the current is frozen and the grid slope is zero,
    /// which is what a clamp means; the engine never gets there in normal
    /// operation.
    #[inline]
    pub fn slopes(&self, vgk: f32, vak: f32) -> (f32, f32, f32) {
        let vak = vak.max(1.0);
        let raw = self.grid_scale * vgk + self.grid_offset;
        let g = raw.min(VGK_CLAMP);
        let c = P6 + (P7 * vak - P8 * g).exp();
        let ia = P1 * vak.powf(P2) / ((P3 - P4 * g).powf(P5) * c);
        let d_vak = ia * (P2 / vak - P7 * (c - P6) / c);
        if raw > VGK_CLAMP {
            return (ia, 0.0, d_vak);
        }
        // d(ln Ia)/dg, times the chain rule for the stretched grid axis.
        let d_vgk = ia * (P4 * P5 / (P3 - P4 * g) + P8 * (c - P6) / c) * self.grid_scale;
        (ia, d_vgk, d_vak)
    }

    /// Transconductance `∂Ia/∂Vgk` of one section, in siemens.
    ///
    /// Published for the metering block and for the gain-range check that
    /// the dossier's test 6 asks for.
    #[inline]
    pub fn transconductance(&self, vgk: f32, vak: f32) -> f32 {
        self.slopes(vgk, vak).1
    }

    /// Plate resistance `∂Vak/∂Ia` of one section, in ohms.
    pub fn plate_resistance(&self, vgk: f32, vak: f32) -> f32 {
        1.0 / self.slopes(vgk, vak).2
    }

    /// Amplification factor at a point: `gm · rp`, which for a remote-cutoff
    /// tube is a function of bias and not a number.
    pub fn mu(&self, vgk: f32, vak: f32) -> f32 {
        let (_, dg, da) = self.slopes(vgk, vak);
        dg / da
    }
}

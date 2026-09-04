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
//! 6386"* — and the difference is in the functional form, not the
//! parameters (dossier 4.1).
//!
//! The law is Raffensperger's eight-parameter fit to General Electric's
//! published curves (dossier 4.3, constants 10.4), **with its cut-off rate
//! corrected against the datasheet it was fitted to** — see [`P8_AS_PUBLISHED`]
//! for the measurement and the reason:
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

/// Raffensperger's fitted parameters for the GE 6386, **with one corrected
/// against the datasheet he fitted to**.
///
/// `p2` to `p7` are his published values. `p8` and the scale `p1` are not,
/// and this is the reason.
///
/// **His equation cuts the tube off far too early, and this unit operates in
/// exactly that region.** Read off General Electric's *plate*
/// characteristics — ET-T1113 page 5, lower figure, "AVERAGE PLATE
/// CHARACTERISTICS, EACH SECTION" — at 250 V of plate:
///
/// | Vgk | GE | as published | corrected |
/// |---|---|---|---|
/// | −12 V | 18.26 mA | −1.2 dB | −0.1 dB |
/// | −20 V | 8.85 | −1.0 | −0.2 |
/// | −30 V | 5.14 | −1.9 | −0.5 |
/// | −40 V | 3.61 | **−4.8** | −1.7 |
/// | −50 V | 1.60 | **−9.1** | +1.7 |
/// | −70 V | 0.60 | **−37.3** | −0.2 |
///
/// A remote-cutoff tube still passing half a milliamp at −70 V *is* the
/// point of the type, and as published the equation has it at one
/// hundredth of that. The Fairchild's grids sit about 22 V down at rest and
/// reach −70 V at the deepest limiting the published static curves show, so
/// the model would spend its whole working range on the wrong part of its
/// own tube law.
///
/// **The correction is one parameter.** `p8` sets the rate of the
/// exponential cut-off term, and it is the only part of the expression that
/// is wrong: everywhere shallower than about −30 V that term is negligible
/// and the power-law term carries the curve, which is why the published fit
/// looks right on the plots it was checked against. Refitting `p8` (and
/// renormalising `p1`, which is only a scale) against the nine readings
/// above plus the tabulated Class A₁ current takes the residual from 20.05
/// to 0.09 in the same least-squares cost. Letting `p4` and `p5` move as
/// well buys 0.03 more and is not taken, because one changed parameter with
/// a reason is easier to defend than four.
///
/// **The accuracy floor, stated rather than implied.** Only one datasheet
/// for the 6386 exists — General Electric's — so there is no second
/// manufacturer's curve to cross-check against and no measured floor. The
/// figure below is a **fit residual**: 0.89 dB RMS over nine readings taken
/// by one person off one 1953 graph. It says how well the curve was fitted,
/// not how right the curve is.
///
/// **How the original check missed it, which is the lesson.** The published
/// fit was validated against three points on the *transfer* characteristics
/// (page 4), where the whole family is crushed into the bottom few per cent
/// of a linear current axis below −30 V. Read there, −50 V looks like "half
/// to one milliamp" and the truth is 1.6. A check made on a plot that
/// cannot resolve the region it is checking can hardly fail. The plate
/// characteristics give every grid voltage its own line, so they resolve the
/// deep end, and [`super::tests`] asserts both ends.
const P1: f32 = 4.539_9e-8;
const P2: f32 = 2.383;
const P3: f32 = 0.5;
const P4: f32 = 0.1;
const P5: f32 = 1.8;
const P6: f32 = 0.5;
const P7: f32 = -0.039_22;
const P8: f32 = 0.131_87;

/// Raffensperger's published `p8`, kept so the tests can show what the
/// correction is worth rather than asserting it in the abstract.
pub const P8_AS_PUBLISHED: f32 = 0.2;
/// RMS residual of the corrected law against the nine plate-characteristic
/// readings, in dB. A fit residual, not a measured accuracy.
pub const FIT_RESIDUAL_DB: f32 = 0.89;

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
    ///
    /// **This is the one quantity the functional form cannot reproduce**, and
    /// it is recorded rather than hidden. Measured off GE's plate
    /// characteristics as the horizontal spacing of the grid curves at a
    /// fixed current — which is what an amplification factor *is*, and a far
    /// easier reading than a current near the baseline — it runs 16.5 near
    /// zero bias down to 5.8 at −30 V. That closes against GE's tabulated
    /// pair: 16.5 over a tabulated 4250 Ω of plate resistance is 3880 µmho
    /// against a tabulated 4000. This expression gives 9.7 at the same
    /// point, because its `Vak^p2` numerator with a grid-only denominator
    /// forces `μ ∝ Vak`, and the tube's falls as the plate rises. Nothing in
    /// the engine reads it: the audio path is a plate-current difference
    /// into a fixed plate voltage, so it never divides a load against a
    /// plate resistance. See the note on the gain path in
    /// [`super::engine`].
    pub fn mu(&self, vgk: f32, vak: f32) -> f32 {
        let (_, dg, da) = self.slopes(vgk, vak);
        dg / da
    }

    /// The law with an arbitrary scale and cut-off rate, so a test can show
    /// what the correction to `p8` is worth against GE's own curve rather
    /// than asserting it in the abstract.
    pub fn anode_current_with(vgk: f32, vak: f32, p1: f32, p8: f32) -> f32 {
        let vgk = vgk.min(VGK_CLAMP);
        let vak = vak.max(1.0);
        p1 * vak.powf(P2) / ((P3 - P4 * vgk).powf(P5) * (P6 + (P7 * vak - p8 * vgk).exp()))
    }
}

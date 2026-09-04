//! The dbx 160 engine: a Blackmer gain cell driven feedforward from a
//! true-RMS log-domain detector.
//!
//! `research/dbx-160.md` sections 5, 6 and 10 are the authority. Nothing
//! here is fitted except the one constant dbx published a target for (the
//! cell's symmetry residual, [`CELL_ASYMMETRY`]) and the one they never
//! published at all (the OverEasy knee width, which is a parameter with an
//! estimated default).

use crate::dsp::fet::oversample::{Downsampler, DryDelay, LATENCY, Upsampler};
use crate::dsp::flush;
use crate::dsp::opto::filters::OnePole;

use super::{
    ALPHA_CEILING, GR_MAX_DB, KNEE_OVEREASY, METER_GAIN_CHANGE, METER_INPUT, MODEL_160,
    THRESHOLD_160_MAX_DBU, THRESHOLD_160_MIN_DBU, THRESHOLD_MAX_DBU, THRESHOLD_MIN_DBU,
};

// ------------------------------------------------------------- constants

/// Thermal voltage at 300 K, from the descendant part's datasheet.
pub const V_T_MV: f32 = 25.9;

/// The Blackmer log constant, millivolts per decibel, on both the cell and
/// the detector: "6.1 mV/dB (6.0 min, 6.2 max)" measured over a −60 to
/// +40 dB gain range. dbx built the box out of two parts with the same
/// constant, which is why a volt is a decibel everywhere in the sidechain.
pub const K_MV_PER_DB: f32 = 6.1;

/// The junctions' ideality factor, implied by the datasheet's own two
/// numbers.
///
/// The detector's stored voltage is `2·n·V_T·ln(I_rms/I_S)`, so its scale
/// against a decibel of RMS level is `n·V_T·ln(10)/10`. Setting that equal
/// to the datasheet's measured 6.1 mV/dB, with `V_T` at its stated 25.9 mV,
/// gives n = 1.023. That is exactly where a silicon junction's ideality
/// belongs, and it is why the published scale factor is 6.1 rather than the
/// 5.96 bare thermal voltage alone would give.
pub const IDEALITY: f32 = K_MV_PER_DB * 10.0 / (V_T_MV * std::f32::consts::LN_10);

/// The **thermal decibel**: how many decibels one junction voltage is worth
/// in the sidechain. It is the natural unit of the log-domain filter, and
/// it sets both the release rate (`D/τ` decibels per second) and how much
/// faster a big step attacks than a small one.
///
/// # Why this is exactly `10/ln 10` and not `V_T/K`
///
/// **This is the one place the model departs from the research's own
/// arithmetic, and the reason is that the detector is either a true RMS
/// detector or it is not.**
///
/// The research divides the datasheet's 25.9 mV by its 6.1 mV/dB and gets
/// 4.246. Those two figures do not correspond: 6.1 is a measured typical
/// that carries the junctions' ideality with it (see [`IDEALITY`]) while
/// 25.9 is bare `kT/q`, so the quotient is 2 % small.
///
/// Doing the algebra instead: the log converter puts `2·n·V_T·ln(I/I_S)`
/// on the charging junction, and that junction's own current is
/// `exp((v_in − v_C)/(n·V_T))`. The capacitor settles where the mean of
/// that current equals the constant discharge current, which is where
/// `⟨(I/I_S)²⟩ = exp(v_C/(n·V_T))` — **the true mean of the square**, with
/// the ideality and the temperature both cancelling because the same kind
/// of junction does the logarithm and the averaging. The filter's decibel
/// unit is then `n·V_T / (n·V_T·ln10/10) = 10/ln 10`, exactly, whatever the
/// ideality and whatever the temperature.
///
/// So this constant is not a measurement to be rounded. It is the number
/// that makes the averaging an average of the square, and at any other
/// value the detector reads a slightly different mean — high on peaky
/// material at 4.246, which is the wrong sign against the datasheet's
/// crest-factor table as well as leaving a sine 2.98 dB rather than 3.01 dB
/// below its peak.
///
/// **What it costs.** dbx's published attack times and their published
/// release rate cannot both be met by any single-constant detector, and the
/// hardware is a single-constant detector; the research establishes that
/// their three attack figures alone imply time constants spanning 27 to
/// 40 ms. With the exact unit the release rate and two of the three attack
/// figures are met and the 20 dB attack point is 35 % slow. That miss is
/// asserted against dbx's own components rather than hidden, and
/// `README.md` records it.
pub const D_DB: f32 = 10.0 / std::f32::consts::LN_10;

/// The detector's time constant, seconds.
///
/// **Derived**, from two components printed on dbx's own drawing, which the
/// drawing marks as a factory-matched pair: `△ R35 AND C15 ARE MATCHED PER
/// DWG #164001`. R35 is 909 kΩ and sets the discharge current from the
/// +15 V rail, `I_T = 15 V / 909 kΩ = 16.50 µA`; C15 is 22 µF; and the log
/// domain filter's time constant is `τ = C·n·V_T/I_T`, the junction's own
/// incremental resistance times the capacitor.
///
/// ```text
/// τ = 22 µF × 1.023 × 25.9 mV / 16.50 µA = 35.3 ms
/// ```
///
/// which puts the release rate at `D/τ` = **123.0 dB/s**, between dbx's two
/// published figures of 120 dB/s for the 160 and 125 dB/s for the 160A. Two
/// components off a drawing and one datasheet constant, landing between two
/// published rates, is as good as this file gets.
///
/// The honest range is 34 to 39 ms, because the real drive is the rail less
/// whatever sits across the current source: the successor company's own
/// design procedure computes the same resistor from the rail plus two diode
/// drops.
pub const TAU_DEFAULT_S: f32 = 0.035_32;
/// Lowest `dbx_tau`, seconds.
pub const TAU_MIN_S: f32 = 0.020;
/// Highest.
pub const TAU_MAX_S: f32 = 0.060;

/// Default OverEasy knee width, dB.
///
/// **Estimate.** dbx never published a knee width for any model in the
/// family and it cannot be derived from the drawing: it is `V_θ/(G·K)`,
/// and while K is known and the diode's scaled thermal voltage is 39 to
/// 52 mV, the difference amplifier's gain G could not be read. That bounds
/// the width to roughly 2 to 9 dB. This is the largest hole in the
/// research and the reason the width is a parameter rather than a
/// constant.
pub const KNEE_WIDTH_DEFAULT_DB: f32 = 6.0;
/// Widest `dbx_knee_width`.
pub const KNEE_WIDTH_MAX_DB: f32 = 12.0;

/// The gain cell's even-order residual.
///
/// **Fitted**, to the one distortion magnitude dbx published for the
/// original: "0.075 % 2nd harmonic at infinite compression at +4dBm
/// output". The mechanism is the cell's, not a waveshaper's: the two
/// halves of the signal go through different transistors, so a matching
/// error amplifies them differently and an asymmetric transfer curve is an
/// even-order one. That is what the part's symmetry trim pin is for and
/// what dbx's factory procedure adjusts R27 against.
///
/// Modelled as `y = x + ε·|x|`, whose second harmonic is `4ε/(3π)` of the
/// fundamental, so ε = 0.00075·3π/4. Being a gain difference between the
/// two halves it is independent of level, of ratio, of time constant and
/// of frequency, which is exactly what dbx's own footnote says of the
/// second harmonic and exactly what separates it from the third.
///
/// **It is a constant, and it does not vary with gain reduction.** An
/// earlier reading of the descendant part's datasheet had the cell's
/// distortion rising fourfold with reduction; the research's author has
/// since withdrawn that, because the rows it rested on change the input
/// level and the gain together and so read a two-variable comparison as a
/// one-variable trend. Nothing here ever varied with reduction and nothing
/// here should start: a tabulated value at one operating point says
/// nothing about how a quantity varies.
pub const CELL_ASYMMETRY: f32 = 0.000_75 * 3.0 * std::f32::consts::PI / 4.0;

/// Input coupling corner, Hz. **Derived** from C12 = 0.15 µF into
/// R26 = 100 kΩ at the cell's virtual ground. dbx publish no frequency
/// response for the original at all, so this is two components rather than
/// a specification, and it is −1.1 dB at 20 Hz: the original had an
/// audible low-frequency tilt that the 160A's much larger coupling
/// capacitors removed.
pub const INPUT_HP_HZ: f32 = 10.6;

/// Output corner, Hz. **Derived** from C14 = 22 pF across R32 = 100 kΩ,
/// the transimpedance stage's compensation. The 160A's published −3 dB at
/// 90 kHz is a loose corroboration from the later board.
pub const OUTPUT_LP_HZ: f32 = 72_300.0;

/// Level, in dBFS, below which the detector's input is floored.
///
/// The excursion at every zero crossing is what generates the detector's
/// ripple, which is what generates dbx's published low-frequency third
/// harmonic, so this floor must be far enough down to be inaudible in that
/// mechanism rather than a smoother placed to tidy it away.
const POWER_FLOOR_DBFS: f64 = -200.0;

/// How far below the stored level the instantaneous one has to fall before
/// the general update is replaced by its exact asymptote. Below this
/// `exp(-q)` would overflow, and the asymptote is a straight line in
/// decibels, which is the release.
const RATE_LIMIT_Q: f64 = -40.0;

/// Longest `dbx_lookahead`, milliseconds.
pub const LOOKAHEAD_MAX_MS: f32 = 10.0;

/// Ring length for the look-ahead delay, in samples of the internal (that
/// is, possibly oversampled) rate. Ten milliseconds at 192 kHz doubled is
/// 3840, and the ring is allocated once at construction so the audio
/// thread never sees an allocation.
const LOOKAHEAD_MAX_SAMPLES: usize = 4096;

/// 0 VU is +4 dBu and −18 dBFS RMS, the calibration the whole lab uses.
pub const VU_REFERENCE_DBFS: f32 = -18.0;

/// Mean of `|sin|`, for turning a block's rectified average back into the
/// sine level a VU movement is calibrated against.
const SINE_MEAN_ABS: f32 = std::f32::consts::FRAC_2_PI;

// ---------------------------------------------------------- the gain cell

/// The Blackmer gain cell's control law, and nothing else.
///
/// Everything here is a property of the **part** — the dbx model 200 the
/// schematic calls out as `VCA (200)`, reference designator M1, and its
/// descendants down to the THAT 2180. Nothing here knows about R26 and
/// R32, the threshold, the ratio, the detector or the control-port
/// divider, because those are the dbx 160 and another box built from the
/// same cell would have different ones.
///
/// [`gain_db`](Self::gain_db) deliberately takes a **voltage**. The whole
/// reason this is a part rather than a multiply is the 6.1 mV/dB constant
/// with its tolerance and its temperature coefficient, and a caller that
/// passes decibels has already thrown those away.
#[derive(Clone, Copy, Debug)]
pub struct BlackmerCell {
    /// The log constant, mV/dB. Nominally [`K_MV_PER_DB`], 6.0 to 6.2
    /// across the part's tolerance.
    pub k_mv_per_db: f32,
    /// The even-order residual from the mismatch of the two half-wave
    /// paths; zero when the symmetry trim is perfect.
    pub symmetry: f32,
    /// Chip temperature, °C. The control constant carries +0.33 %/°C
    /// referenced to 27 °C, which is what the matched transistor dbx
    /// supplied with the module compensates.
    pub temp_c: f32,
}

impl Default for BlackmerCell {
    fn default() -> Self {
        BlackmerCell {
            k_mv_per_db: K_MV_PER_DB,
            symmetry: CELL_ASYMMETRY,
            temp_c: 27.0,
        }
    }
}

impl BlackmerCell {
    /// The log constant in force, including the temperature coefficient.
    #[inline]
    pub fn k(&self) -> f32 {
        self.k_mv_per_db * (1.0 + 0.0033 * (self.temp_c - 27.0))
    }

    /// Gain in decibels for a control-port voltage, in millivolts. The
    /// positive port; the negative one is `-K`.
    #[inline]
    pub fn gain_db(&self, v_ctrl_mv: f32) -> f32 {
        -v_ctrl_mv / self.k()
    }

    /// The control voltage in millivolts that asks for a gain in decibels.
    #[inline]
    pub fn control_mv(&self, gain_db: f32) -> f32 {
        -gain_db * self.k()
    }

    /// One sample through the cell at a linear gain, with the cell's own
    /// even-order residual. `dc` is a slow running mean of `|x|`, which
    /// the caller keeps: the residual carries a DC term that the real
    /// output coupling removes, and subtracting the mean is that removal
    /// without putting another pole in the audio path.
    #[inline]
    pub fn process(&self, x: f32, gain: f32, dc: f32) -> f32 {
        gain * (x + self.symmetry * (x.abs() - dc))
    }
}

// ----------------------------------------------------------- the detector

/// Blackmer's true-RMS detector as a log-domain filter.
///
/// The capacitor is charged through a junction whose current is the
/// antilogarithm of the difference between the log-domain signal and the
/// capacitor voltage, and discharged by a constant current. Writing the
/// stored level as `L` decibels and the instantaneous one as `L_inst`,
///
/// ```text
/// dL/dt = (D/τ) · ( exp( (L_inst − L) / D ) − 1 )
/// ```
///
/// which has an exact discrete solution for a held input over one sample
/// period, so [`step`](Self::step) costs one `exp` and one `ln` and is
/// unconditionally stable at any rate. There is no attack branch and no
/// release branch, because the circuit has neither: a rising signal
/// attacks faster the bigger the step, a falling one decays along a
/// straight line of `D/τ` decibels per second, and the two are one
/// constant seen from two sides.
#[derive(Clone, Copy, Debug)]
pub struct RmsDetector {
    /// Stored level in dB, kept in `f64` because it can sit a hundred
    /// decibels above the instantaneous one and the difference matters.
    level_db: f64,
    /// `exp(-h/τ)` for the sample period in force.
    a: f64,
    /// `(D/τ)·h`, the decibels one sample of rate-limited release costs.
    rate_step_db: f64,
    d: f64,
}

impl Default for RmsDetector {
    fn default() -> Self {
        let mut d = RmsDetector {
            level_db: POWER_FLOOR_DBFS,
            a: 0.0,
            rate_step_db: 0.0,
            d: D_DB as f64,
        };
        d.set(TAU_DEFAULT_S, 48_000.0);
        d
    }
}

impl RmsDetector {
    /// Retune to a time constant and a sample rate. This is the only
    /// rate-dependent coefficient in the whole detector, which is the
    /// pleasant consequence of solving the filter exactly rather than
    /// discretising it by hand.
    pub fn set(&mut self, tau_s: f32, sr: f32) {
        let h = 1.0 / sr.max(1.0) as f64;
        let tau = tau_s.max(1e-4) as f64;
        self.a = (-h / tau).exp();
        self.rate_step_db = self.d / tau * h;
    }

    /// Forget the stored level.
    pub fn reset(&mut self) {
        self.level_db = POWER_FLOOR_DBFS;
    }

    /// The stored level, dBFS.
    #[inline]
    pub fn level_db(&self) -> f32 {
        self.level_db as f32
    }

    /// Release rate in decibels per second, `D/τ`.
    pub fn release_rate_db_s(&self, tau_s: f32) -> f32 {
        D_DB / tau_s.max(1e-4)
    }

    /// One sample of the detector's power input.
    #[inline]
    pub fn step(&mut self, power: f32) -> f32 {
        let inst = if power > 1e-20 {
            10.0 * (power as f64).log10()
        } else {
            POWER_FLOOR_DBFS
        };
        let q0 = (inst - self.level_db) / self.d;
        if q0 < RATE_LIMIT_Q {
            // The charging junction is shut; the capacitor is discharged by
            // the constant current alone and the level falls along a
            // straight line. This is the exact asymptote of the line below,
            // not an approximation of it, and it is also what keeps
            // `exp(-q0)` from overflowing after loud material stops.
            self.level_db -= self.rate_step_db;
        } else {
            let m = 1.0 - (1.0 - (-q0).exp()) * self.a;
            self.level_db = inst + self.d * m.max(1e-300).ln();
        }
        self.level_db as f32
    }
}

// ------------------------------------------------------- the static curve

/// The softplus, `w·ln(1 + exp(x/w))`, with `w → 0` giving `max(x, 0)`.
///
/// This is the rectifier, and the knee width is what the OverEasy button
/// moves. With the diode inside the operational rectifier's feedback loop
/// its softness is divided by the amplifier's open-loop gain and the
/// corner collapses to under a ten-thousandth of a decibel, which is the
/// hard knee. Outside the loop the diode's exponential is exposed and
/// becomes the knee.
///
/// The exact solution of the circuit dbx's OverEasy patent draws is the
/// Wright omega function, and this is not it. Omega's asymptote drifts
/// logarithmically below the hard-knee line rather than merging with it,
/// so switching the button deep in compression would change the level —
/// whereas dbx draw the two curves merging and describe the threshold as
/// the point midway between the onset of processing and the point where
/// the curve "corresponds to the setting of the RATIO control", which
/// asserts that the ratio really is attained. The softplus has the same
/// shape, the same continuously rising first derivative the patent
/// specifies, and an exact asymptote. **The substitution is a modelling
/// choice, not a reading of the circuit**; below about 15 dB of gain
/// reduction the two are the same curve.
#[inline]
pub fn softplus_db(x: f32, w: f32) -> f32 {
    if w <= 1e-4 {
        return x.max(0.0);
    }
    let t = x / w;
    // Above this the softplus is its asymptote to within f32's resolution,
    // and `exp` would be on its way to overflowing.
    if t > 30.0 { x } else { w * t.exp().ln_1p() }
}

/// Gain reduction in decibels for an excess over threshold, a ratio
/// coefficient and a knee width. Positive for reduction.
///
/// Three multiplications and the rectifier, in that order, which is the
/// order of the circuit: the knee is in the rectifier, so its width in
/// decibels does not depend on the ratio while its depth does. That is why
/// dbx's OverEasy specification row reads "affected by THRESHOLD,
/// COMPRESSION RATIO settings" and their hard-knee row does not.
#[inline]
pub fn gain_reduction_db(excess_db: f32, alpha: f32, knee_db: f32) -> f32 {
    let a = effective_alpha(alpha);
    (a * softplus_db(excess_db, knee_db)).min(GR_MAX_DB)
}

/// The coefficient the circuit can actually reach.
///
/// dbx published the ∞ mark as 120:1, so the positive end is capped at
/// `1 − 1/120` and a residual slope of one decibel in 120 is left in.
/// Above 1 the coefficient passes through untouched, because Infinity+ is
/// not a mode either: it is the same pot continuing to turn.
#[inline]
pub fn effective_alpha(alpha: f32) -> f32 {
    if alpha <= 1.0 {
        alpha.clamp(0.0, ALPHA_CEILING)
    } else {
        alpha.min(2.0)
    }
}

// -------------------------------------------------------------- settings

/// Everything the engine needs from the parameters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// [`MODEL_160`] or [`super::MODEL_160A`].
    pub model: usize,
    /// THRESHOLD, dBu.
    pub threshold_dbu: f32,
    /// COMPRESSION, as the coefficient the pot sets: `α = 1 − 1/R`.
    pub alpha: f32,
    /// OUTPUT GAIN, dB.
    pub output_db: f32,
    /// [`super::KNEE_HARD`] or [`KNEE_OVEREASY`]. Forced hard on the
    /// original, which has no such switch.
    pub knee: usize,
    /// OverEasy knee width, dB. Ours: dbx never published one.
    pub knee_width_db: f32,
    /// The detector's time constant, seconds. Ours, and dbx's whole point
    /// is that you cannot adjust it; it is exposed because it is the one
    /// number the box is made of.
    pub tau_s: f32,
    /// METER, one of [`METER_INPUT`], [`super::METER_OUTPUT`],
    /// [`METER_GAIN_CHANGE`].
    pub meter: usize,
    /// The rear-panel METER CALIBRATION trimmer: what level reads 0 VU,
    /// dBu.
    pub meter_cal_dbu: f32,
    /// Anticipated compression, milliseconds. Not a control dbx fitted,
    /// but a use of the detector input they documented and drew in 1995.
    pub lookahead_ms: f32,
    /// The level in dBu that 0 dBFS RMS stands for.
    pub headroom_db: f32,
    /// The lab's shared stereo link, which here is dbx's True RMS Power
    /// Summing and the 160A's SLAVE button.
    pub link: bool,
    /// Wet share, 0..1. Not on the hardware.
    pub mix: f32,
    /// Side-chain high-pass in Hz, 0 = off. The lab's shared extra, and
    /// the nearest this plug-in comes to the detector input dbx put on the
    /// rear of the 160A.
    pub sc_hpf: f32,
    /// BYPASS.
    pub bypass: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            model: MODEL_160,
            threshold_dbu: 0.0,
            alpha: 0.75,
            output_db: 0.0,
            knee: super::KNEE_HARD,
            knee_width_db: KNEE_WIDTH_DEFAULT_DB,
            tau_s: TAU_DEFAULT_S,
            meter: METER_GAIN_CHANGE,
            meter_cal_dbu: super::METER_CAL_DEFAULT_DBU,
            lookahead_ms: 0.0,
            headroom_db: super::HEADROOM_DEFAULT_DB,
            link: false,
            mix: 1.0,
            sc_hpf: 0.0,
            bypass: false,
        }
    }
}

impl Settings {
    /// The settings with everything the selected unit cannot do taken away.
    ///
    /// The parameters carry the union of the two units' ranges so that one
    /// set of controls serves both faces, and this is where the original
    /// gets its own limits back: its THRESHOLD pot runs 10 mV to 3 V and
    /// no further, its COMPRESSION pot stops at the ∞ mark, and it has no
    /// OverEasy switch, because OverEasy is a 1978 invention that arrived
    /// three years after it shipped.
    pub fn clamped(mut self) -> Self {
        if self.model == MODEL_160 {
            self.threshold_dbu = self
                .threshold_dbu
                .clamp(THRESHOLD_160_MIN_DBU, THRESHOLD_160_MAX_DBU);
            self.alpha = self.alpha.min(1.0);
            self.knee = super::KNEE_HARD;
        } else {
            self.threshold_dbu = self
                .threshold_dbu
                .clamp(THRESHOLD_MIN_DBU, THRESHOLD_MAX_DBU);
            self.alpha = self.alpha.clamp(0.0, 2.0);
        }
        self
    }

    /// The knee width in force: zero unless the OverEasy switch is in.
    #[inline]
    pub fn knee_db(&self) -> f32 {
        if self.knee == KNEE_OVEREASY {
            self.knee_width_db.max(0.0)
        } else {
            0.0
        }
    }

    /// The threshold in dBFS RMS at the headroom in force.
    #[inline]
    pub fn threshold_dbfs(&self) -> f32 {
        self.threshold_dbu - self.headroom_db
    }
}

// ------------------------------------------------------------ one channel

/// One channel's state.
#[derive(Clone)]
struct Channel {
    det: RmsDetector,
    /// The ghost trace: what a peak detector with the same time constant
    /// would have made of the same signal. Not modelling anything in the
    /// hardware; it exists so that the argument this whole model rests on
    /// is visible on the page.
    ghost_db: f32,
    sc_hpf: OnePole,
    in_hp: OnePole,
    out_lp: OnePole,
    /// Slow mean of `|x|` at the cell, so its even-order residual carries
    /// no DC.
    cell_dc: OnePole,
    /// Anticipated compression, at the internal rate.
    look: Vec<f32>,
    look_w: usize,
    up: Upsampler,
    down: Downsampler,
    dry: DryDelay,
}

impl Channel {
    fn new(sr: f32) -> Self {
        let mut c = Channel {
            det: RmsDetector::default(),
            ghost_db: -120.0,
            sc_hpf: OnePole::default(),
            in_hp: OnePole::default(),
            out_lp: OnePole::default(),
            cell_dc: OnePole::default(),
            look: vec![0.0; LOOKAHEAD_MAX_SAMPLES],
            look_w: 0,
            up: Upsampler::new(),
            down: Downsampler::new(),
            dry: DryDelay::new(),
        };
        c.set_sample_rate(sr, sr < 88_200.0);
        c
    }

    fn set_sample_rate(&mut self, sr: f32, oversample: bool) {
        let internal = if oversample { sr * 2.0 } else { sr };
        self.in_hp.set(INPUT_HP_HZ, internal);
        self.out_lp.set(OUTPUT_LP_HZ.min(internal * 0.49), internal);
        self.cell_dc.set(5.0, internal);
        self.det.set(TAU_DEFAULT_S, internal);
        self.reset();
    }

    fn reset(&mut self) {
        self.det.reset();
        self.ghost_db = -120.0;
        self.sc_hpf.reset();
        self.in_hp.reset();
        self.out_lp.reset();
        self.cell_dc.reset();
        self.look.iter_mut().for_each(|v| *v = 0.0);
        self.look_w = 0;
        self.up.reset();
        self.down.reset();
        self.dry.reset();
    }

    /// Push one internal-rate sample into the look-ahead line and take out
    /// what went in `delay` samples ago.
    #[inline]
    fn delay(&mut self, x: f32, delay: usize) -> f32 {
        let n = self.look.len();
        self.look[self.look_w] = x;
        let r = (self.look_w + n - delay.min(n - 1)) % n;
        self.look_w = (self.look_w + 1) % n;
        self.look[r]
    }
}

// -------------------------------------------------------------- the unit

/// A dbx 160, or a 160A, depending on which panel `dbx_model` selects.
pub struct Compressor {
    sr: f32,
    settings: Settings,
    cell: BlackmerCell,
    ch: [Channel; 2],
    oversample: bool,
    /// Look-ahead in samples of the internal rate.
    look_samples: usize,
    // Per-block meter accumulators.
    gr_db: [f32; 2],
    in_peak: [f32; 2],
    out_peak: [f32; 2],
    in_abs: [f32; 2],
    out_abs: [f32; 2],
    det_db: [f32; 2],
    ghost_gr_db: f32,
    frames: usize,
}

impl Compressor {
    /// A unit at `sr` hertz with default settings.
    pub fn new(sr: f32) -> Self {
        let oversample = sr < 88_200.0;
        let mut c = Compressor {
            sr,
            settings: Settings::default(),
            cell: BlackmerCell::default(),
            ch: [Channel::new(sr), Channel::new(sr)],
            oversample,
            look_samples: 0,
            gr_db: [0.0; 2],
            in_peak: [0.0; 2],
            out_peak: [0.0; 2],
            in_abs: [0.0; 2],
            out_abs: [0.0; 2],
            det_db: [-120.0; 2],
            ghost_gr_db: 0.0,
            frames: 0,
        };
        c.retune();
        c
    }

    /// Change the sample rate, rebuilding everything that depends on it.
    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sr = sr;
        self.oversample = sr < 88_200.0;
        for c in &mut self.ch {
            c.set_sample_rate(sr, self.oversample);
        }
        self.retune();
        self.reset();
    }

    /// The internal rate: twice the host's below 88.2 kHz.
    #[inline]
    pub fn internal_rate(&self) -> f32 {
        if self.oversample {
            self.sr * 2.0
        } else {
            self.sr
        }
    }

    /// Recompute everything the settings and the rate together decide.
    fn retune(&mut self) {
        let internal = self.internal_rate();
        let s = self.settings;
        for c in &mut self.ch {
            c.det.set(s.tau_s, internal);
            if s.sc_hpf > 1.0 {
                c.sc_hpf.set(s.sc_hpf, internal);
            }
        }
        self.look_samples =
            ((s.lookahead_ms.clamp(0.0, LOOKAHEAD_MAX_MS) * 1e-3 * internal).round() as usize)
                .min(LOOKAHEAD_MAX_SAMPLES - 1);
    }

    /// Silence every state.
    pub fn reset(&mut self) {
        for c in &mut self.ch {
            c.reset();
        }
        self.gr_db = [0.0; 2];
        self.in_peak = [0.0; 2];
        self.out_peak = [0.0; 2];
        self.in_abs = [0.0; 2];
        self.out_abs = [0.0; 2];
        self.det_db = [-120.0; 2];
        self.ghost_gr_db = 0.0;
        self.frames = 0;
    }

    /// The settings in force, after the selected unit's own limits.
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// The gain cell.
    pub fn cell(&self) -> &BlackmerCell {
        &self.cell
    }

    /// Latency in samples at the host rate: the oversampler's round trip
    /// plus the look-ahead, which is reported so the host puts the track
    /// back where it belongs. The compression still arrives before the
    /// transient, which is the whole of the effect; only the plug-in's
    /// place in the session is restored.
    pub fn latency(&self) -> usize {
        let look = if self.oversample {
            self.look_samples / 2
        } else {
            self.look_samples
        };
        look + if self.oversample { LATENCY } else { 0 }
    }

    /// Apply new settings. Returns whether anything changed.
    pub fn configure(&mut self, s: Settings) -> bool {
        let s = s.clamped();
        let changed = s != self.settings;
        if changed {
            self.settings = s;
            self.retune();
        }
        changed
    }

    /// Gain reduction in dB (positive) on `channel`, from the last block.
    pub fn gain_reduction_db(&self, channel: usize) -> f32 {
        self.gr_db[channel.min(1)]
    }

    /// The detector's stored level in dBFS on `channel`, from the last
    /// block. This is the quantity the whole box is built around and the
    /// one every ballistics test measures.
    pub fn detector_db(&self, channel: usize) -> f32 {
        self.det_db[channel.min(1)]
    }

    /// What a peak detector with the same time constant would have asked
    /// for over the last block, in dB of reduction. The ghost trace.
    pub fn ghost_gr_db(&self) -> f32 {
        self.ghost_gr_db
    }

    /// How brightly the BELOW and ABOVE indicators sit, 0..1 each.
    ///
    /// dbx specify the interesting case: "A steady-state, sine-wave tone
    /// exactly at the threshold voltage causes both L.E.D.'s to remain
    /// dimly illuminated", and their factory procedure calibrates the
    /// threshold by turning the control until both are off. So the pair is
    /// a comparator with a small linear region either side of the
    /// threshold rather than two independent lamps, and at the threshold
    /// they are equal and half lit.
    pub fn threshold_lamps(&self) -> (f32, f32) {
        let excess = 0.5 * (self.det_db[0] + self.det_db[1]) - self.settings.threshold_dbfs();
        // One decibel either side is fully one or the other, which is the
        // window dbx's own test uses.
        let t = (0.5 + 0.5 * excess.clamp(-1.0, 1.0)).clamp(0.0, 1.0);
        (1.0 - t, t)
    }

    /// Whether the detector is inside the OverEasy region, 0..1: the third
    /// indicator the OverEasy models grew, which lights "when the signal is
    /// in the Over Easy region".
    pub fn overeasy_lamp(&self) -> f32 {
        let w = self.settings.knee_db();
        if w <= 0.0 {
            return 0.0;
        }
        let excess = 0.5 * (self.det_db[0] + self.det_db[1]) - self.settings.threshold_dbfs();
        // The knee is the band where the slope is between 5 % and 95 % of
        // its final value, which for the logistic is ±2.944·w.
        let edge = 2.944 * w;
        if excess.abs() <= edge {
            1.0 - (excess.abs() / edge)
        } else {
            0.0
        }
    }

    /// One internal-rate sample pair. Returns the two outputs.
    #[inline]
    fn tick(&mut self, xl: f32, xr: f32, k: &TickConst) -> (f32, f32) {
        // The detector listens to the input. Feedforward, always: dbx's own
        // argument for it was that a feedback loop's gain rises with the
        // ratio until it oscillates, which is why a 1176 stops at 20:1 and
        // this does not, and why Infinity+ is possible here at all.
        if !k.processing {
            // The 160A's BYPASS is a relay that works with the power off,
            // so nothing in the sidechain is running and the meters are
            // dead. The look-ahead line keeps turning so that the reported
            // latency stays honest and the bypassed signal comes back in
            // the same place the compressed one would have.
            self.gr_db = [0.0; 2];
            self.det_db = [-120.0; 2];
            self.ghost_gr_db = 0.0;
            let dl = self.ch[0].delay(xl, k.look);
            let dr = self.ch[1].delay(xr, k.look);
            return (dl, dr);
        }
        let (mut dl, mut dr) = (xl, xr);
        if k.use_hpf {
            dl = self.ch[0].sc_hpf.hp(dl);
            dr = self.ch[1].sc_hpf.hp(dr);
        }
        // True RMS Power Summing: the energies are added, not the signals,
        // "to prevent phase cancellation of the two signals from causing
        // unmusical compressor action". Two identical channels therefore
        // read 3.01 dB higher than either alone, and the effective
        // threshold drops by 3 dB when the link goes in. That is what the
        // hardware does and a model that compensated for it would not be
        // modelling the box.
        let (p0, p1) = if k.link {
            let p = dl * dl + dr * dr;
            (p, p)
        } else {
            (dl * dl, dr * dr)
        };

        let l0 = self.ch[0].det.step(p0);
        let l1 = if k.link { l0 } else { self.ch[1].det.step(p1) };

        let gr0 = gain_reduction_db(l0 - k.threshold_dbfs, k.alpha, k.knee_db);
        let gr1 = if k.link {
            gr0
        } else {
            gain_reduction_db(l1 - k.threshold_dbfs, k.alpha, k.knee_db)
        };

        // The ghost: the same threshold and ratio driven by a peak
        // follower with the same time constant.
        let peak = dl.abs().max(dr.abs());
        let peak_db = 20.0 * peak.max(1e-9).log10();
        let g = &mut self.ch[0].ghost_db;
        *g = if peak_db > *g {
            peak_db
        } else {
            (*g - k.ghost_fall_db).max(peak_db)
        };
        self.ghost_gr_db = gain_reduction_db(*g - k.threshold_dbfs, k.alpha, k.knee_db);

        let mut out = [0.0f32; 2];
        for (i, (x, gr)) in [(xl, gr0), (xr, gr1)].into_iter().enumerate() {
            let c = &mut self.ch[i];
            let d = c.delay(x, k.look);
            let a = c.in_hp.hp(d);
            let gain = 10f32.powf(-gr / 20.0) * k.make_up;
            let dc = c.cell_dc.lp(a.abs());
            out[i] = flush(c.out_lp.lp(self.cell.process(a, gain, dc)));
        }
        self.gr_db[0] = gr0;
        self.gr_db[1] = gr1;
        self.det_db[0] = l0;
        self.det_db[1] = l1;
        (out[0], out[1])
    }

    /// Process one block in place.
    pub fn process_block(&mut self, l: &mut [f32], r: &mut [f32]) {
        let n = l.len().min(r.len());
        let s = self.settings;
        let k = TickConst {
            threshold_dbfs: s.threshold_dbfs(),
            alpha: s.alpha,
            knee_db: s.knee_db(),
            make_up: 10f32.powf(s.output_db / 20.0),
            link: s.link,
            use_hpf: s.sc_hpf > 1.0,
            processing: !s.bypass,
            look: self.look_samples,
            ghost_fall_db: D_DB / s.tau_s.max(1e-4) / self.internal_rate(),
        };
        let oversample = self.oversample;
        let mix = s.mix.clamp(0.0, 1.0);

        self.in_peak = [0.0; 2];
        self.out_peak = [0.0; 2];
        self.in_abs = [0.0; 2];
        self.out_abs = [0.0; 2];
        let mut gr_sum = [0.0f32; 2];

        for i in 0..n {
            let (xl, xr) = (l[i], r[i]);
            self.in_peak[0] = self.in_peak[0].max(xl.abs());
            self.in_peak[1] = self.in_peak[1].max(xr.abs());
            self.in_abs[0] += xl.abs();
            self.in_abs[1] += xr.abs();

            let (mut yl, mut yr);
            if oversample {
                let al = self.ch[0].up.process(xl);
                let ar = self.ch[1].up.process(xr);
                let (b0, c0) = self.tick(al[0], ar[0], &k);
                let (b1, c1) = self.tick(al[1], ar[1], &k);
                yl = self.ch[0].down.process([b0, b1]);
                yr = self.ch[1].down.process([c0, c1]);
            } else {
                let (a, b) = self.tick(xl, xr, &k);
                yl = a;
                yr = b;
            }
            // The dry path is held back by the resampler's round trip so a
            // partial mix does not comb-filter itself. The look-ahead is
            // already inside the wet path and is reported as latency, so
            // both halves arrive together.
            let dl = self.ch[0].dry.process(xl);
            let dr = self.ch[1].dry.process(xr);
            if oversample {
                yl += (dl - yl) * (1.0 - mix);
                yr += (dr - yr) * (1.0 - mix);
            } else {
                yl += (xl - yl) * (1.0 - mix);
                yr += (xr - yr) * (1.0 - mix);
            }

            gr_sum[0] += self.gr_db[0];
            gr_sum[1] += self.gr_db[1];
            l[i] = yl;
            r[i] = yr;
            self.out_peak[0] = self.out_peak[0].max(yl.abs());
            self.out_peak[1] = self.out_peak[1].max(yr.abs());
            self.out_abs[0] += yl.abs();
            self.out_abs[1] += yr.abs();
        }

        if n > 0 {
            let inv = 1.0 / n as f32;
            self.gr_db = [gr_sum[0] * inv, gr_sum[1] * inv];
            self.in_abs = [self.in_abs[0] * inv, self.in_abs[1] * inv];
            self.out_abs = [self.out_abs[0] * inv, self.out_abs[1] * inv];
        }
        self.frames = n;
    }

    /// What the VU movement is chasing this block, in dB.
    ///
    /// The lab runs the movement itself, on the audio thread, so this is
    /// the level the needle is heading for and not where it is.
    fn meter_target_db(&self) -> f32 {
        let s = &self.settings;
        match s.meter {
            METER_GAIN_CHANGE => -0.5 * (self.gr_db[0] + self.gr_db[1]),
            m => {
                let abs = if m == METER_INPUT {
                    0.5 * (self.in_abs[0] + self.in_abs[1])
                } else {
                    0.5 * (self.out_abs[0] + self.out_abs[1])
                };
                // A VU movement is average-responding and sine-calibrated,
                // so the block's rectified mean is turned back into the
                // sine whose mean it is, and read against the trimmer.
                let rms = abs / SINE_MEAN_ABS / std::f32::consts::SQRT_2;
                let dbfs = 20.0 * rms.max(1e-9).log10();
                dbfs + s.headroom_db - s.meter_cal_dbu
            }
        }
    }

    /// `[in_l, in_r, out_l, out_r, gr_db, meter_target]` for the last
    /// block, with `gr_db` **positive** for reduction, which is the lab's
    /// frame convention.
    pub fn meter_frame(&self) -> [f32; 6] {
        [
            self.in_peak[0],
            self.in_peak[1],
            self.out_peak[0],
            self.out_peak[1],
            0.5 * (self.gr_db[0] + self.gr_db[1]),
            self.meter_target_db(),
        ]
    }

    /// `[below, above, ghost_gr_db, overeasy]`: the two threshold
    /// indicators, what a peak detector would have done, and the third
    /// indicator the OverEasy models grew.
    pub fn lamps_frame(&self) -> [f32; 4] {
        let (below, above) = self.threshold_lamps();
        [below, above, self.ghost_gr_db, self.overeasy_lamp()]
    }

    /// Steady-state gain reduction in dB (positive) for a sine of peak
    /// `amp_peak`.
    ///
    /// A direct evaluation rather than a fixed point, because the box is
    /// feedforward: the detector never sees the output, so there is
    /// nothing to solve.
    pub fn static_gr_db(&self, amp_peak: f32) -> f32 {
        if self.settings.bypass {
            return 0.0;
        }
        let mut rms = amp_peak / std::f32::consts::SQRT_2;
        if self.settings.link {
            // Both channels carrying the same sine doubles the power.
            rms *= std::f32::consts::SQRT_2;
        }
        let level = 20.0 * rms.max(1e-12).log10();
        gain_reduction_db(
            level - self.settings.threshold_dbfs(),
            self.settings.alpha,
            self.settings.knee_db(),
        )
    }

    /// The static transfer curve, output dBFS for `min_dbfs..max_dbfs` in.
    pub fn transfer_curve(&self, out: &mut [f32], min_dbfs: f32, max_dbfs: f32) {
        let n = out.len();
        if n == 0 {
            return;
        }
        let make_up = self.settings.output_db;
        for (i, o) in out.iter_mut().enumerate() {
            let t = if n == 1 {
                0.0
            } else {
                i as f32 / (n - 1) as f32
            };
            let db = min_dbfs + (max_dbfs - min_dbfs) * t;
            let amp = 10f32.powf(db / 20.0);
            *o = db - self.static_gr_db(amp) + make_up;
        }
    }
}

/// The per-block constants [`Compressor::tick`] needs, hoisted out of the
/// loop so nothing is recomputed per sample.
struct TickConst {
    threshold_dbfs: f32,
    alpha: f32,
    knee_db: f32,
    make_up: f32,
    link: bool,
    use_hpf: bool,
    processing: bool,
    look: usize,
    ghost_fall_db: f32,
}

//! The variable-mu engine: the Fairchild 660 and 670.
//!
//! `research/Fairchild-670.md` is what this is built from; section 10 is the
//! design and section 11 the test plan. Where the drawings leave a constant
//! under-determined it is fitted to a **published figure** and labelled as
//! fitted, and the figure it was fitted to is named.
//!
//! ## What is different about this engine, and why it could not be a reskin
//!
//! **There is no gain multiplier anywhere in it.** Every other model in the
//! lab computes a gain and multiplies the audio by it: a FET channel to
//! ground, a photocell in a divider, a Blackmer cell, a ring of diodes. Here
//! the gain element *is* the amplifier. The audio is amplified by eight 6386
//! triode sections a channel, and the control voltage reduces gain by
//! walking those same sections down their own remote-cutoff curve. Nothing
//! in the signal path attenuates. Search this file for `* gain` and you will
//! not find it: the output is the **difference of two tube currents**.
//!
//! Three things follow, and they are the model:
//!
//! 1. **Gain reduction and distortion are one curve read at two points.**
//!    Small-signal gain is proportional to the characteristic's slope and
//!    second-order distortion to its curvature, both evaluated at the point
//!    the control voltage sets. You cannot move one without the other, and
//!    Fairchild published a chart in March 1959 that shows exactly this:
//!    IM distortion against decibels of limiting at seven output levels
//!    (dossier 4.6). A model of this box that lets you have 15 dB of gain
//!    reduction cleanly is wrong.
//! 2. **The control voltage is common-mode, so it cancels at the output.**
//!    It is injected at the centre tap of the input transformer's secondary,
//!    so both grids move down together while the audio moves them apart.
//!    The output transformer takes the difference, and a common-mode step
//!    disappears in it. That is the mechanism behind the manual's first
//!    boast — *"the complete absence of audible thumps"* — and it means this
//!    engine needs no control-signal smoother at all.
//! 3. **The audio self-biases.** The two halves' currents move in opposite
//!    directions, so their *sum* is constant to first order and the cathode
//!    stays put; to second order the remote-cutoff curve is convex, so the
//!    sum rises with signal and the stage bends its own operating point down
//!    on loud material. That is a real, small, level-dependent gain change
//!    with no sidechain involved, and it is part of why people say the box
//!    "does something even at zero gain reduction".
//!
//! ## The cathode, and one place this departs from the dossier's block C
//!
//! The dossier's block C solves a cathode voltage **per half**, each half
//! self-biased through its own 680 Ω. That is right for the operating point
//! and wrong for the audio: `C101`/`C102`, the 4 µF that bridges the two
//! cathode nodes, ties them together for the differential signal, which is
//! the whole reason Raffensperger names that capacitor as the part that sets
//! the low-frequency response. Two independent solves would degenerate the
//! differential signal that the capacitor exists to un-degenerate, and would
//! take the stage's own nonlinearity — the thing this model is for — down
//! with it.
//!
//! So there is **one** cathode node here, carrying the sum of the two
//! halves' currents into `R_k`, which is what two tied cathodes with a
//! resistor each look like. The common-mode path keeps its degeneration, so
//! the operating point and the control taper are the circuit's; the
//! differential path has none, so the audio sees the tube's own law. The
//! capacitor's finite size shows up as the low corner, [`CATHODE_HP_HZ`].
//!
//! ## Where the numbers come from
//!
//! | constant | source |
//! |---|---|
//! | the tube law's eight parameters | Raffensperger [18], dossier 10.4 |
//! | `R_k`, `n_par`, the plate rail, the balance pot | the 670 schematic [4] and the 660 factory drawing [5] |
//! | the sidechain pad and `N_sc` | the 670 schematic [4] and Raffensperger |
//! | the dead-zone map, the clip, the rectifier, `I_max` | Raffensperger, all marked **E** in the dossier |
//! | every timing component | the 660 factory drawing [5], see [`super::network`] |
//! | [`A_V`] | **fitted** to the published input/output curve 3 |
//! | [`GRID_V_AT_PLUS24_DBM`] | **fitted** to the published IM chart |
//! | [`REST_GAIN_DB`] | published input/output curve 1 |
//! | [`CATHODE_HP_HZ`] | **fitted** to the published 40 Hz / −1 dB response |

use std::f32::consts::{PI, SQRT_2};

use crate::dsp::opto::filters::OnePole;
use crate::dsp::pre::filters::{Hp1, Hp2, Lp1};
use crate::dsp::vu::Vu;

use super::network::{Network, POSITIONS, position};
use super::oversample::{MAX_FACTOR, Resampler};
use super::triode::RemoteCutoffTriode;
use super::{AGC_LAT_VERT, METER_BAL_PULL, METER_BAL_PUSH, MODEL_660, Settings, TUBE_JJ_6386_LGP};

// ------------------------------------------------------------ calibration

/// 0 VU is +4 dBu and −18 dBFS RMS, the calibration the whole lab uses.
pub const VU_REFERENCE_DBFS: f32 = -18.0;
/// Peak amplitude of a sine at 0 VU, i.e. at +4 dBu.
pub const VU_REF_AMP: f32 = 0.125_892_54 * SQRT_2;

/// Volts on the 600 Ω line for one unit of plug-in amplitude.
///
/// The lab puts +4 dBu at −18 dBFS RMS, so an RMS amplitude of 0.125 892 54
/// is 1.227 644 V RMS and the ratio is 9.751 52 volts per unit. dBm into
/// 600 Ω and dBu are the same number, which is why the manual's dBm figures
/// can be used directly.
pub const VOLTS_PER_AMP: f32 = 9.751_52;

/// Peak amplitude of a sine at `x` dBm into 600 Ω.
pub fn dbm_amp(x: f32) -> f32 {
    VU_REF_AMP * 10f32.powf((x - 4.0) / 20.0)
}

/// A level in dBm from a peak amplitude.
pub fn amp_dbm(a: f32) -> f32 {
    20.0 * (a.max(1e-12) / VU_REF_AMP).log10() + 4.0
}

// --------------------------------------------------------------- the stage

/// Sections in parallel per push-pull half (V101–V104 a channel, one
/// section each side: the 670 schematic [4] and the 660 drawing [5]).
pub const N_PAR: f32 = 4.0;
/// Plate rail, from the schematic's own annotation (the transformer centre
/// tap is marked 240 V and the plate rail 230 V).
pub const V_PLATE: f32 = 230.0;
/// Cathode resistor per half on the 670: R103/R104 at 680 Ω plus half the
/// 100 Ω balance pot, which is Raffensperger's lumped `R_11 = R_21 = 705 Ω`.
pub const R_K_670: f32 = 705.0;
/// The 660's, from the factory drawing: R4/R5 at 1800 Ω plus half the 500 Ω
/// balance pot. **This is the one 660-versus-670 constant the dossier
/// trusts** (1.3, 3.3), and a factor of 2.6 on the cathode resistor is a
/// different operating point in the one stage that does all the work.
pub const R_K_660: f32 = 2050.0;
/// Half the balance pot, 670 and 660.
const R_BAL_670: f32 = 50.0;
const R_BAL_660: f32 = 250.0;
/// Standing grid bias the ZERO control sets, in volts: Raffensperger's
/// `V_bias`.
pub const V_BIAS_NOMINAL: f32 = -7.2;

/// Newton steps on the cathode node per oversampled sample.
///
/// The node is warm-started from the previous sample and moves slowly at
/// eight times the audio rate, so two steps put the residual far below a
/// millivolt; `the_cathode_solve_converges` measures it rather than assuming
/// it. The currents used for the output are the second step's, corrected to
/// first order by the step that followed, which buys the accuracy of a third
/// evaluation without paying for one.
const NEWTON_STEPS: usize = 2;

// ----------------------------------------------------------- the sidechain

/// Sidechain pad: four 150 Ω in series into two 680 Ω, off T102's second
/// secondary (670 schematic [4]; Raffensperger's `R_in = 600 Ω`,
/// `R_term = 1360 Ω` are the same components).
pub const G_PAD: f32 = 1360.0 / 1960.0;
/// T103's step-up, `N_p/N_s = 1/17` (Raffensperger).
pub const N_SC: f32 = 17.0;
/// Gain of sidechain stages two and three.
///
/// **Fitted**, to the published input/output curve 3 (dossier 7.2), not
/// taken from Raffensperger's 8.4. His figure is a fit to *his own SPICE
/// simulation* and the dossier marks it **E**; curve 3 is a factory
/// measurement of the hardware and marks the ratio at five input levels. The
/// two anchors are not of the same strength and this follows the stronger
/// one. With this value the model reproduces the five published points of
/// curve 3 to 0.16 dB RMS.
pub const A_V: f32 = 20.0;
/// Where stages two and three clip (Raffensperger, **E**).
const V_CLIP: f32 = 100.0;
/// Rectifier: germanium drop, softness and output resistance
/// (Raffensperger, all **E**; the dossier records that the diode type is
/// not established, Radiomuseum naming a silicon 1N538 for the 670).
const V_D: f32 = 0.3;
const LAMBDA: f32 = 10.0;
const R_O: f32 = 160.0;
/// The sidechain output stage's current limit (Raffensperger, **E**).
///
/// This is what makes the attack slew-limited and therefore proportional to
/// the timing capacitance, which is the only reading that reproduces all six
/// published attack times (dossier 5.6).
pub const I_MAX: f32 = 0.5;
/// R8 and R9 on the 660 drawing: the 24 kΩ resistors at the AC threshold
/// pot's centre tap that give it a kinked law.
const AC_TAP_R: f32 = 24_000.0;
/// The AC threshold pot, 100 kΩ linear on the 670 (R115a/b).
const AC_POT_R: f32 = 100_000.0;

// ---------------------------------------------------------- the two anchors

/// Small-signal gain of the whole unit at the reference attenuator setting,
/// in dB.
///
/// Published curve 1 of the December 1959 input/output chart, the straight
/// amplifier, reads **+2.0 dBm out at 0 dBm in** (dossier 7.2). The manual's
/// operating instruction says unity gain is "approx 10 db attenuation",
/// which is 2 dB away from the same setting; the chart is a measurement and
/// the instruction says "approx", so the chart wins and the difference is
/// recorded rather than split.
pub const REST_GAIN_DB: f32 = 2.0;
/// The attenuator setting the published curves were taken at, in dB. The
/// manual's own unity-gain setting, and the model's default.
pub const REF_ATTEN_DB: f32 = 10.0;
/// Grid half-swing, in volts, that produces +24 dBm at the output with no
/// limiting.
///
/// **Fitted**, to the published IM chart's curve 7 (+24 dBm out), which
/// reads about 3.8 % IM at zero limiting. It is the one constant that sets
/// how hard the tubes are driven for a given output, and fixing it fixes the
/// whole family: the model then reads 0.22 / 0.60 / 1.6 % at +12 / +16 /
/// +20 dBm against the chart's 0.2–0.3 / 0.5–0.7 / 1.6–1.7 %, which is six
/// published curves reproduced from one number.
pub const GRID_V_AT_PLUS24_DBM: f32 = 13.0;

/// The low corner, in Hz, of the first-order high-pass that stands for the
/// 4 µF cathode bridge.
///
/// **Fitted**, to the published response band: 40 Hz to 15 kHz ±1 dB. The
/// dossier says outright that this corner is *not established* (10.4),
/// because the impedance the capacitor works into depends on the operating
/// point, and instructs that it be set to meet the published figure and then
/// allowed to move. Modelling the shelf from the components instead — 705 Ω
/// against 4 µF, with the stage's own `gm·R_k` of about 1.04 — puts the
/// transition between 28 and 58 Hz and the response 3.1 dB down at 40 Hz,
/// which misses the specification by two decibels; that figure is recorded
/// in the README rather than used.
pub const CATHODE_HP_HZ: f32 = 16.5;
/// T101 and T102 as the dossier's block A asks: a second-order high-pass
/// well below the band and a first-order low-pass either side of it, the
/// pair landing on the published 15 kHz at −1 dB.
const XFMR_HP_HZ: f32 = 5.0;
const XFMR_HP_Q: f32 = 0.707;
const XFMR_LP_HZ: f32 = 60_000.0;

// ------------------------------------------------------------ control law

/// Entries in the tabulated control law, one per volt.
const LAW_POINTS: usize = 101;

/// The stage's small-signal behaviour against control voltage, tabulated
/// when the operating point moves.
///
/// The audio path never reads this — it evaluates the tube directly. It
/// exists for the three things that need an answer without running the loop:
/// the gain-reduction figure the meter stream carries, the plate-current
/// bridge the METERING switch reads, and the static transfer curve the page
/// draws. Rebuilt only when the bias, the tube or the unit changes, which is
/// what keeps it off the per-sample path.
#[derive(Clone)]
pub struct ControlLaw {
    /// Differential transconductance at each volt of control, in A/V.
    g: [f32; LAW_POINTS],
    /// Total plate current of both halves at each volt, in A.
    i_sum: [f32; LAW_POINTS],
}

impl ControlLaw {
    /// Tabulate for a tube, a cathode resistor and a standing bias.
    pub fn build(tube: &RemoteCutoffTriode, r_k: f32, bias: f32) -> Self {
        let mut law = ControlLaw {
            g: [0.0; LAW_POINTS],
            i_sum: [0.0; LAW_POINTS],
        };
        let mut vk = 14.0;
        for k in 0..LAW_POINTS {
            let vx = bias - k as f32;
            vk = quiescent_vk(tube, r_k, vx, vk);
            let (ia, gm, _) = tube.slopes(vx - vk, V_PLATE - vk);
            // Both halves in push-pull, four sections each: the differential
            // transconductance is 2·n_par·gm, and the current the meter
            // bridge sees is 2·n_par·Ia.
            law.g[k] = 2.0 * N_PAR * gm;
            law.i_sum[k] = 2.0 * N_PAR * ia;
        }
        law
    }

    /// Differential transconductance at `v` volts of control voltage.
    pub fn transconductance(&self, v: f32) -> f32 {
        self.lerp(&self.g, v)
    }

    /// Total plate current at `v` volts of control voltage.
    pub fn plate_current(&self, v: f32) -> f32 {
        self.lerp(&self.i_sum, v)
    }

    /// Gain reduction in dB (positive) at `v` volts of control voltage.
    pub fn gr_db(&self, v: f32) -> f32 {
        20.0 * (self.g[0] / self.transconductance(v).max(1e-12)).log10()
    }

    fn lerp(&self, t: &[f32; LAW_POINTS], v: f32) -> f32 {
        let x = v.clamp(0.0, (LAW_POINTS - 1) as f32);
        let i = x.floor() as usize;
        let f = x - i as f32;
        let a = t[i];
        let b = t[(i + 1).min(LAW_POINTS - 1)];
        a + (b - a) * f
    }
}

/// Solve the cathode node and return `(I_push − I_pull, Vk)`.
///
/// One node, not two: the 4 µF between the cathodes ties them for the
/// differential signal, so the common-mode path keeps its degeneration
/// (which sets the operating point and the control taper) while the audio
/// sees the tube's own law. See the module header.
#[inline]
fn stage(tube: &RemoteCutoffTriode, vg1: f32, vg2: f32, r_k: f32, vk0: f32) -> (f32, f32) {
    let g = r_k * N_PAR * 0.5;
    let mut vk = vk0;
    let mut i1 = 0.0;
    let mut i2 = 0.0;
    let mut s1 = 0.0;
    let mut s2 = 0.0;
    let mut step = 0.0;
    for _ in 0..NEWTON_STEPS {
        let (a, ag, aa) = tube.slopes(vg1 - vk, V_PLATE - vk);
        let (b, bg, ba) = tube.slopes(vg2 - vk, V_PLATE - vk);
        i1 = a;
        i2 = b;
        s1 = ag + aa;
        s2 = bg + ba;
        let f = vk - g * (a + b);
        let fp = 1.0 + g * (s1 + s2);
        step = f / fp;
        vk -= step;
    }
    // The currents above were taken before the last (tiny) move of the node;
    // correcting them to first order is worth a whole extra evaluation of
    // the tube and costs four multiplies.
    let idiff = N_PAR * ((i1 + s1 * step) - (i2 + s2 * step));
    (idiff, vk)
}

/// The sidechain, from the output volts to the current the timing network is
/// driven with (dossier 10.3, blocks E to H).
#[inline]
fn sidechain(v_out: f32, phi_ac: f32, width: f32, v_ctrl: f32) -> f32 {
    let v_pot = phi_ac * (G_PAD * N_SC * v_out) * 0.5;
    // The dead zone: a soft, symmetric window whose width is the DC
    // threshold. Small overshoots fall inside it and are not detected, large
    // ones clear it and are, which is the progressive ratio four sources
    // describe and only Raffensperger gives an equation for.
    let v1 = softplus(v_pot - width) - softplus(-v_pot - width);
    let v3 = (A_V * v1).clamp(-V_CLIP, V_CLIP);
    // A peak rectifier: the diodes conduct only while the rectified voltage
    // is above what the network already holds.
    let v_diff = v3.abs() - v_ctrl;
    let i_nom = (2.0 * V_D) / (LAMBDA * R_O) * softplus(LAMBDA * v_diff / (2.0 * V_D) - LAMBDA);
    // The hard current limit that makes the attack slew-limited, and
    // therefore proportional to the timing capacitance.
    i_nom - (I_MAX / 10.0) * softplus(10.0 * i_nom / I_MAX - 10.0)
}

/// The cathode voltage with no signal: the fixed point of
/// `Vk = R_k · n_par · Ia(Vx − Vk, Vplate − Vk)`, warm-started from `vk0`.
fn quiescent_vk(tube: &RemoteCutoffTriode, r_k: f32, vx: f32, vk0: f32) -> f32 {
    let mut vk = vk0;
    for _ in 0..24 {
        let (ia, dg, da) = tube.slopes(vx - vk, V_PLATE - vk);
        let f = vk - r_k * N_PAR * ia;
        let fp = 1.0 + r_k * N_PAR * (dg + da);
        let step = f / fp;
        vk -= step;
        if step.abs() < 1e-7 {
            break;
        }
    }
    vk.max(0.0)
}

// ---------------------------------------------------------------- helpers

/// `ln(1 + e^z)`, in the form that does not overflow.
#[inline]
fn softplus(z: f32) -> f32 {
    if z > 0.0 {
        z + (-z).exp().ln_1p()
    } else {
        z.exp().ln_1p()
    }
}

/// The AC threshold pot's law: panel 0–10 to the fraction of the sidechain
/// signal that reaches the first stage.
///
/// R115a/b is a 100 kΩ **linear** pot, but the 660 drawing shows R8 and R9,
/// 24 kΩ 5 %, hung on its centre tap, which is what Raffensperger means by
/// *"effectively a 76 kΩ potentiometer with a piecewise linear taper"*. A
/// resistor at the tap loads the midpoint, so the law has a kink there: the
/// lower half is compressed towards the bottom of the scale and the upper
/// half opens out. Working the divider both sides of the tap:
///
/// ```text
/// u ≤ ½ :  V_w / V = 2u · Z / (½R + Z)                Z = R_tap ∥ ½R
/// u > ½ :  V_w / V = ((u − ½)R + Z) / ((1 − u)R + (u − ½)R + Z)
/// ```
///
/// which is continuous at the tap and reaches 1 fully clockwise. **This is
/// why the threshold knob's numbers are not decibels** and why the panel
/// prints 0 to 10: the pot's law is a kinked line, and what the control
/// really sets, jointly with the DC threshold, is a curve.
pub fn ac_threshold_law(panel: f32) -> f32 {
    let u = (panel / 10.0).clamp(0.0, 1.0);
    let half = 0.5 * AC_POT_R;
    let z = AC_TAP_R * half / (AC_TAP_R + half);
    if u <= 0.5 {
        2.0 * u * z / (half + z)
    } else {
        let above = (u - 0.5) * AC_POT_R;
        let top = (1.0 - u) * AC_POT_R;
        (above + z) / (top + above + z)
    }
}

/// The DC threshold trimmer's law: panel travel to the width of the dead
/// zone Raffensperger's first sidechain stage has.
///
/// `φ'_DC = 12.2 (φ_DC + 0.1)` is his, and the **direction** is the
/// published curves': curve 4 is the DC control "slightly CCW from CW" and
/// plateaus at 0 dBm out, curve 5 is "slightly CW from CCW" and plateaus at
/// +10 dBm, so clockwise is *more* limiting, which is a *narrower* dead
/// zone. Raffensperger does not say which way his `φ_DC` runs; the chart
/// does, so the travel is inverted here and the reason is this paragraph.
pub fn dc_threshold_width(travel: f32) -> f32 {
    12.2 * ((1.0 - travel.clamp(0.0, 1.0)) + 0.1)
}

// --------------------------------------------------------------- channels

/// A fixed delay for the dry path, long enough for the deepest cascade.
#[derive(Clone, Copy, Debug)]
struct DryDelay {
    buf: [f32; MAX_DRY + 1],
    pos: usize,
    len: usize,
}

/// Deepest round trip the resampler can ask for.
const MAX_DRY: usize = 60;

impl DryDelay {
    fn new() -> Self {
        DryDelay {
            buf: [0.0; MAX_DRY + 1],
            pos: 0,
            len: MAX_DRY,
        }
    }

    fn set_len(&mut self, len: usize) {
        let len = len.min(MAX_DRY);
        if len != self.len {
            self.len = len;
            self.reset();
        }
    }

    fn reset(&mut self) {
        self.buf = [0.0; MAX_DRY + 1];
        self.pos = 0;
    }

    #[inline]
    fn process(&mut self, x: f32) -> f32 {
        let n = self.len + 1;
        self.buf[self.pos] = x;
        self.pos = (self.pos + 1) % n;
        self.buf[self.pos]
    }
}

/// One channel: one 660, or one half of a 670.
#[derive(Clone)]
struct Channel {
    net: Network,
    rs: Resampler,
    dry: DryDelay,
    /// T101 and T102.
    in_hp: Hp2,
    in_lp: Lp1,
    out_hp: Hp2,
    out_lp: Lp1,
    /// The 4 µF cathode bridge, on the difference current.
    cathode_hp: Hp1,
    /// The lab's own side-chain high-pass, which the hardware has not got.
    sc_hpf: OnePole,
    /// The cathode node, warm-started sample to sample.
    vk: f32,
    /// Block accumulators.
    gr_sum: f32,
    meter_sum: f32,
    frames: usize,
}

impl Channel {
    fn new(sr: f32, depth: usize) -> Self {
        let rs = Resampler::new(depth);
        let mut ch = Channel {
            net: Network::new(sr * rs.factor() as f32, 2),
            dry: DryDelay::new(),
            rs,
            in_hp: Hp2::bypassed(),
            in_lp: Lp1::default(),
            out_hp: Hp2::bypassed(),
            out_lp: Lp1::default(),
            cathode_hp: Hp1::default(),
            sc_hpf: OnePole::default(),
            vk: 14.0,
            gr_sum: 0.0,
            meter_sum: 0.0,
            frames: 0,
        };
        ch.retune(sr);
        ch
    }

    fn retune(&mut self, sr: f32) {
        let os = sr * self.rs.factor() as f32;
        self.net.set_sample_rate(os);
        self.in_hp = Hp2::new(os, XFMR_HP_HZ, XFMR_HP_Q);
        self.out_hp = Hp2::new(os, XFMR_HP_HZ, XFMR_HP_Q);
        self.in_lp.set(XFMR_LP_HZ, os);
        self.out_lp.set(XFMR_LP_HZ, os);
        self.cathode_hp.set(CATHODE_HP_HZ, os);
        self.dry.set_len(self.rs.latency());
    }

    fn reset(&mut self) {
        self.net.reset();
        self.rs.reset();
        self.dry.reset();
        self.in_hp.reset();
        self.out_hp.reset();
        self.in_lp.reset();
        self.out_lp.reset();
        self.cathode_hp.reset();
        self.sc_hpf.reset();
        self.vk = 14.0;
        self.gr_sum = 0.0;
        self.meter_sum = 0.0;
        self.frames = 0;
    }
}

// -------------------------------------------------------------- the unit

/// The Fairchild variable-mu limiter.
pub struct Compressor {
    sr: f32,
    settings: Settings,
    tube: RemoteCutoffTriode,
    law: ControlLaw,
    /// `K_out · G(0)`: volts out per volt of grid half-swing at rest, held
    /// at the reference configuration so that changing the tube, the ZERO
    /// screw or the unit changes the gain, as all three do on the hardware.
    k_out: f32,
    /// T101's step-up to each grid, from the two published anchors.
    n_in: f32,
    ch: [Channel; 2],
    vu: Vu,
    in_peak: [f32; 2],
    out_peak: [f32; 2],
    gr_db: [f32; 2],
    meter_db: [f32; 2],
}

impl Compressor {
    pub fn new(sr: f32) -> Self {
        let s = Settings::default();
        let tube = RemoteCutoffTriode::ge_6386();
        let law = ControlLaw::build(&tube, R_K_670, V_BIAS_NOMINAL);
        // The two published anchors, solved together. At the reference
        // attenuator setting the unit's small-signal gain is REST_GAIN_DB,
        // and at +24 dBm out the grids see GRID_V_AT_PLUS24_DBM volts:
        //   K·G0·N_in = 10^((rest + ref_atten)/20)
        //   K·G0      = peak(+24 dBm) / grid volts
        let k_g0 = dbm_amp(24.0) * VOLTS_PER_AMP / GRID_V_AT_PLUS24_DBM;
        let n_in = 10f32.powf((REST_GAIN_DB + REF_ATTEN_DB) / 20.0) / k_g0;
        let mut c = Compressor {
            sr,
            settings: s,
            tube,
            k_out: k_g0 / law.g[0],
            n_in,
            law,
            ch: [Channel::new(sr, 3), Channel::new(sr, 3)],
            vu: Vu::new(sr),
            in_peak: [0.0; 2],
            out_peak: [0.0; 2],
            gr_db: [0.0; 2],
            meter_db: [0.0; 2],
        };
        c.rebuild();
        c
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sr = sr;
        self.vu.set_sample_rate(sr);
        for ch in self.ch.iter_mut() {
            ch.retune(sr);
        }
        self.reset();
    }

    pub fn reset(&mut self) {
        let vk = quiescent_vk(&self.tube, self.r_k(), self.settings.zero[0], 14.0);
        for ch in self.ch.iter_mut() {
            ch.reset();
            ch.vk = vk;
        }
        self.vu.reset();
        self.in_peak = [0.0; 2];
        self.out_peak = [0.0; 2];
        self.gr_db = [0.0; 2];
        self.meter_db = [0.0; 2];
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// The tabulated control law, for the tests and the page.
    pub fn control_law(&self) -> &ControlLaw {
        &self.law
    }

    /// The tube in force.
    pub fn tube(&self) -> &RemoteCutoffTriode {
        &self.tube
    }

    /// The cathode resistor of the unit in force: 705 Ω on the 670,
    /// 2050 Ω on the 660 (dossier 1.3).
    pub fn r_k(&self) -> f32 {
        if self.settings.model == MODEL_660 {
            R_K_660
        } else {
            R_K_670
        }
    }

    /// Half the balance pot of the unit in force.
    fn r_bal(&self) -> f32 {
        if self.settings.model == MODEL_660 {
            R_BAL_660
        } else {
            R_BAL_670
        }
    }

    pub fn latency(&self) -> usize {
        self.ch[0].rs.latency()
    }

    /// Apply a settings snapshot; `true` if anything changed.
    pub fn configure(&mut self, s: Settings) -> bool {
        if s == self.settings {
            return false;
        }
        let depth_changed = s.oversample != self.settings.oversample;
        let law_changed = s.tube != self.settings.tube
            || s.model != self.settings.model
            || s.zero != self.settings.zero;
        self.settings = s;
        let depth = s.depth();
        let sr = self.sr;
        for ch in self.ch.iter_mut() {
            if depth_changed {
                ch.rs.set_depth(depth);
                ch.retune(sr);
            }
        }
        for (c, ch) in self.ch.iter_mut().enumerate() {
            // The 660 is mono: both channels take the left row's switch.
            let k = if s.model == MODEL_660 { 0 } else { c };
            // The switch moves; the capacitors keep their charge.
            ch.net.set_position(s.time[k].min(POSITIONS - 1));
            if s.sc_hpf > 1.0 {
                ch.sc_hpf.set(s.sc_hpf, sr * ch.rs.factor() as f32);
            }
        }
        if law_changed || depth_changed {
            self.rebuild();
        }
        true
    }

    /// Retabulate the control law for the tube, unit and ZERO in force.
    ///
    /// The law is built at the **left** channel's ZERO, which is what the
    /// meter and the transfer curve are drawn against; the audio path
    /// evaluates the tube per channel and does not read the table.
    fn rebuild(&mut self) {
        self.tube = if self.settings.tube == TUBE_JJ_6386_LGP {
            RemoteCutoffTriode::jj_6386_lgp()
        } else {
            RemoteCutoffTriode::ge_6386()
        };
        self.law = ControlLaw::build(&self.tube, self.r_k(), self.settings.zero[0]);
    }

    // ------------------------------------------------------------ the loop

    /// Process one stereo block in place.
    pub fn process_block(&mut self, l: &mut [f32], r: &mut [f32]) {
        let n = l.len().min(r.len());
        let s = self.settings;
        let matrix = s.agc == AGC_LAT_VERT && s.model != MODEL_660;
        let r_k = self.r_k();
        let r_bal = self.r_bal();
        // The 660 is a mono unit, so both channels take the left row's
        // settings and the matrix is not there to be switched in.
        let per = |i: usize| if s.model == MODEL_660 { 0 } else { i };
        let processing = !s.bypass;
        let use_hpf = s.sc_hpf > 1.0;

        // Fields are split into disjoint borrows so the inner loop can hold
        // a channel mutably while reading the tube and the control law.
        let law = &self.law;
        let tube = &self.tube;
        let chans = &mut self.ch;
        let n_in = self.n_in;
        let k_out = self.k_out;
        let sr = self.sr;
        let factor = chans[0].rs.factor();
        let os_rate = sr * factor as f32;
        let i_ref = law.plate_current(0.0);
        let g_ref = law.transconductance(0.0);

        let mut atten = [0.0f32; 2];
        let mut phi_ac = [0.0f32; 2];
        let mut width = [0.0f32; 2];
        let mut bias = [0.0f32; 2];
        let mut bal = [0.0f32; 2];
        for (c, ch) in chans.iter_mut().enumerate() {
            let k = per(c);
            atten[c] = 10f32.powf(-s.input_gain[k] / 20.0);
            phi_ac[c] = ac_threshold_law(s.threshold[k]);
            width[c] = dc_threshold_width(s.dc_threshold[k]);
            bias[c] = s.zero[k];
            // The balance pot splits the two cathode resistors, which puts
            // the two halves at different operating points. To first order
            // that is a differential grid offset: the standing current of
            // one half through the resistance the wiper moves.
            bal[c] = s.balance[k] * r_bal * 0.5 * i_ref;
            ch.gr_sum = 0.0;
            ch.meter_sum = 0.0;
            ch.frames = 0;
            // The low corner moves with gain reduction, because the cathode
            // impedance the 4 µF works into depends on the operating point.
            // Updated once a block from the control voltage in force.
            let v = ch.net.control_v();
            let deg = (1.0 + law.transconductance(v) * 0.5 * r_k) / (1.0 + g_ref * 0.5 * r_k);
            ch.cathode_hp.set(CATHODE_HP_HZ * deg, os_rate);
        }

        let mut in_peak = [0.0f32; 2];
        let mut out_peak = [0.0f32; 2];
        let mut up = [[0.0f32; MAX_FACTOR]; 2];
        let mut down = [[0.0f32; MAX_FACTOR]; 2];

        for i in 0..n {
            let x = [l[i], r[i]];
            for c in 0..2 {
                in_peak[c] = in_peak[c].max(x[c].abs());
            }
            if !processing {
                for (c, ch) in chans.iter_mut().enumerate() {
                    ch.dry.process(x[c]);
                }
                out_peak = in_peak;
                continue;
            }
            // The matrix: lateral is the sum and vertical the difference,
            // and what follows is two entirely independent limiters on them
            // — not a linked pair (dossier 6.3).
            let a = if matrix {
                [0.5 * (x[0] + x[1]), 0.5 * (x[0] - x[1])]
            } else {
                x
            };
            for (c, ch) in chans.iter_mut().enumerate() {
                ch.rs.up(a[c], &mut up[c]);
            }
            for k in 0..factor {
                let mut demand = [0.0f32; 2];
                let mut outs = [0.0f32; 2];
                for (c, ch) in chans.iter_mut().enumerate() {
                    // T101: the line, the step attenuator, the transformer.
                    let v_line = up[c][k] * VOLTS_PER_AMP * atten[c];
                    let v_sec = n_in * ch.in_lp.process(ch.in_hp.process(v_line));
                    // The control voltage arrives common-mode, one sample
                    // old, which is how the loop is closed. Both grids move
                    // together and the audio moves them apart.
                    let vx = bias[c] - ch.net.control_v();
                    let vg1 = vx + v_sec + bal[c];
                    let vg2 = vx - v_sec - bal[c];
                    let (idiff, vk) = stage(tube, vg1, vg2, r_k, ch.vk);
                    ch.vk = vk;
                    // The cathode bridge, then T102. **This is the whole
                    // compressor**: a difference of two tube currents.
                    let v_out = k_out * ch.cathode_hp.process(idiff);
                    let v_out = ch.out_lp.process(ch.out_hp.process(v_out));
                    outs[c] = v_out;
                    // The sidechain listens to the **output**: the pad hangs
                    // off T102's second secondary.
                    let sc_in = if use_hpf { ch.sc_hpf.hp(v_out) } else { v_out };
                    demand[c] = sidechain(sc_in, phi_ac[c], width[c], ch.net.control_v());
                }
                if s.link {
                    let m = demand[0].max(demand[1]);
                    demand = [m, m];
                }
                for (c, ch) in chans.iter_mut().enumerate() {
                    ch.net.step(demand[c]);
                    down[c][k] = outs[c] / VOLTS_PER_AMP;
                }
            }
            let mut y = [0.0f32; 2];
            for (c, ch) in chans.iter_mut().enumerate() {
                y[c] = ch.rs.down(&down[c]);
            }
            let y = if matrix {
                [y[0] + y[1], y[0] - y[1]]
            } else {
                y
            };
            for (c, ch) in chans.iter_mut().enumerate() {
                let dry = ch.dry.process(x[c]);
                let out = dry + (y[c] - dry) * s.mix;
                out_peak[c] = out_peak[c].max(out.abs());
                if c == 0 {
                    l[i] = out;
                } else {
                    r[i] = out;
                }
                let v = ch.net.control_v();
                ch.gr_sum += law.gr_db(v);
                // The METERING switch. In ZERO the meter sits in the centre
                // tap and reads the change in the total plate current from
                // its standing value; in the two BALANCE positions it reads
                // one leg. Moving the ZERO screw moves the standing point,
                // so it moves the needle — which is what the hardware does,
                // and why that control is a bias trim wearing a
                // meter-calibration label.
                let i_now = law.plate_current(v);
                let m = s.meter[per(c)];
                let reading = if m == METER_BAL_PUSH || m == METER_BAL_PULL {
                    let tilt = if m == METER_BAL_PUSH { 1.0 } else { -1.0 };
                    i_now * (1.0 + tilt * s.balance[per(c)]) / i_ref
                } else {
                    i_now / i_ref
                };
                ch.meter_sum += 20.0 * reading.max(1e-6).log10();
                ch.frames += 1;
            }
        }

        for c in 0..2 {
            let f = chans[c].frames.max(1) as f32;
            self.gr_db[c] = if processing { chans[c].gr_sum / f } else { 0.0 };
            self.meter_db[c] = if processing {
                chans[c].meter_sum / f
            } else {
                0.0
            };
        }
        self.in_peak = in_peak;
        self.out_peak = out_peak;
        let target = 0.5 * (self.meter_db[0] + self.meter_db[1]);
        self.vu.advance(target, n);
    }

    /// Gain reduction in dB (positive) of one channel over the last block.
    pub fn gain_reduction_db(&self, channel: usize) -> f32 {
        self.gr_db[channel.min(1)]
    }

    /// The control voltage one channel's timing network holds, in volts.
    pub fn control_v(&self, channel: usize) -> f32 {
        self.ch[channel.min(1)].net.control_v()
    }

    /// `[control volts, leg U charge, leg V charge]`: the state of the three
    /// timing capacitors, which positions 5 and 6 are incomprehensible
    /// without and obvious with.
    pub fn cell_state(&self) -> [f32; 3] {
        self.ch[0].net.charge_state()
    }

    /// `[in_l, in_r, out_l, out_r, gr_db, meter_vu]`, with `gr_db`
    /// **positive** for reduction, which is the lab's frame convention.
    pub fn meter_frame(&self) -> [f32; 6] {
        [
            self.in_peak[0],
            self.in_peak[1],
            self.out_peak[0],
            self.out_peak[1],
            0.5 * (self.gr_db[0] + self.gr_db[1]),
            self.vu.value(),
        ]
    }

    // --------------------------------------------------------- static curve

    /// Steady-state control voltage for a sine of peak amplitude `amp` at
    /// the input of channel `c`.
    ///
    /// Solved as a fixed point rather than by running the loop, because a
    /// release can be twenty-five seconds long. The rectifier only conducts
    /// near the peaks and the whole chain is symmetric, so the cycle average
    /// is the average over a quarter cycle.
    pub fn static_control_v(&self, c: usize, amp: f32) -> f32 {
        let s = &self.settings;
        let k = if s.model == MODEL_660 { 0 } else { c.min(1) };
        if s.bypass {
            return 0.0;
        }
        let phi_ac = ac_threshold_law(s.threshold[k]);
        let width = dc_threshold_width(s.dc_threshold[k]);
        let r_t = position(s.time[k].min(POSITIONS - 1)).r_t;
        let v_sec = self.n_in * amp * VOLTS_PER_AMP * 10f32.powf(-s.input_gain[k] / 20.0);
        const QUARTER: usize = 48;
        let mut lo = 0.0f32;
        let mut hi = (LAW_POINTS - 1) as f32;
        for _ in 0..28 {
            let v = 0.5 * (lo + hi);
            let peak = self.k_out * self.law.transconductance(v) * v_sec;
            let mut acc = 0.0;
            for j in 0..QUARTER {
                let th = 0.5 * PI * (j as f32 + 0.5) / QUARTER as f32;
                acc += sidechain(peak * th.sin(), phi_ac, width, v);
            }
            if acc / QUARTER as f32 > v / r_t {
                lo = v;
            } else {
                hi = v;
            }
        }
        0.5 * (lo + hi)
    }

    /// Steady-state gain reduction in dB (positive) for a sine of peak
    /// amplitude `amp`.
    pub fn static_gr_db(&self, amp: f32) -> f32 {
        self.law.gr_db(self.static_control_v(0, amp))
    }

    /// The static transfer curve, output dBFS for `min_dbfs..max_dbfs` in.
    pub fn transfer_curve(&self, out: &mut [f32], min_dbfs: f32, max_dbfs: f32) {
        let n = out.len();
        if n == 0 {
            return;
        }
        // The curve is drawn for the left channel, as the meter is.
        let make_up = REST_GAIN_DB + REF_ATTEN_DB - self.settings.input_gain[0];
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

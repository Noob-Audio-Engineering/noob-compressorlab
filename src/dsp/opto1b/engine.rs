//! The CL 1B engine: a feedback optical compressor whose time constants
//! are electronic rather than photochemical.
//!
//! Built from `research/CL-1B.md` section 9. The signal path, per channel
//! per sample:
//!
//! 1. the input transformer, 3.5 Hz to 30 kHz;
//! 2. the four-element attenuator ([`Network`]), which has **three** nodes
//!    because the Ratio rheostat sits between the node the detector
//!    listens to and the node the cell shunts;
//! 3. the audio node, and the photocell's own small odd-order term;
//! 4. the detector tap, which is **node B, not the audio node**, scaled by
//!    the Threshold pot;
//! 5. the shared side-chain high-pass (the lab's own extra, off by
//!    default, because the hardware's detector is flat);
//! 6. a 33.5 kHz pole and mean-absolute rectification;
//! 7. the control state, driven by whichever of the three timing modes is
//!    selected;
//! 8. the GRE's own lag and its static drive-to-conductance law;
//! 9. the Gain pot, applied **after** the attenuator so it cannot affect
//!    compression;
//! 10. a single-ended stage, a push-pull ceiling, the output
//!     transformer's low-frequency term, and a 30 kHz pole.
//!
//! Two calibrations from the service manual pin the model, and between
//! them they leave almost no freedom: +250.0 mV at the side-chain jack
//! gives exactly −10.0 dB, and the Gain control's maximum is exactly
//! +30.0 dB. A third, the panel's own −20 dot producing 1 dB of reduction
//! at −20 dBu, fixes the threshold offset. All three are solved
//! numerically in [`Calibration`] rather than written down as magic
//! numbers, so they stay true if a resistor value is corrected.

use super::{METER_COMP, METER_IN, METER_OUT, MODE_FIXED, MODE_FIXMAN};
use crate::dsp::fet::oversample::{Downsampler, DryDelay, LATENCY, Upsampler};
use crate::dsp::opto::filters::{Biquad, OnePole, flush};
// Deliberately narrow: `Cell` is *not* imported, and `tests::the_t4_cell_was_not_imported`
// exists to catch a future refactor that makes importing it look convenient.
use crate::dsp::opto::model::{R_DARK, SINE_MEAN_ABS, VU_REF_AMP, VU_REF_DBFS, distortion};

/// 0 VU, in dBFS. Shared with every other model, and Softube publish the
/// same reference for their own CL 1B.
pub const VU_REFERENCE_DBFS: f32 = VU_REF_DBFS;

/// Mean-absolute value of a sine at 0 VU.
pub const VU_REF_MEAN: f32 = SINE_MEAN_ABS * VU_REF_AMP;

/// Constants. Every value with an anchor names it; the rest are estimates
/// tuned against the test plan, and say so.
pub mod k {
    /// R2, the series arm of the attenuator. Schematic TE130-42.
    pub const R_SERIES: f32 = 100e3;
    /// P3, the Gain pot, shunting the audio node. Schematic.
    pub const R_POT: f32 = 100e3;
    /// P1, the Threshold pot, shunting the detector node. Schematic.
    pub const R_THR: f32 = 100e3;
    /// P2, the Ratio rheostat, between the two nodes. Schematic.
    pub const R_RATIO_MAX: f32 = 10e3;
    /// The GRE at full light. **Estimate**: no maximum reduction is
    /// published anywhere, so this is tuned for a plausible curve and
    /// there is deliberately no test asserting a maximum reduction.
    pub const R_GRE_MIN: f32 = 40.0;

    /// The control voltage that produces exactly 10 dB of reduction.
    /// Service manual: "+250,0 mV into the side chain jack ... the output
    /// level has dropped to −10,0 dB."
    pub const U_REF_10DB: f32 = 0.250;

    /// The GRE's own lag. **Estimate**, chosen well under the 0.5 ms
    /// fastest published attack so it never dominates it.
    pub const TAU_CELL_S: f32 = 200e-6;

    /// Published attack range, manual and specification sheet.
    pub const A_MIN_S: f32 = 0.5e-3;
    pub const A_MAX_S: f32 = 300e-3;
    /// Published fixed attack.
    pub const A_FIXED_S: f32 = 1.0e-3;
    /// Published release range.
    pub const R_MIN_S: f32 = 0.05;
    pub const R_MAX_S: f32 = 10.0;
    /// Published fixed release.
    pub const R_FIXED_S: f32 = 0.05;

    /// Maximum make-up gain. Service manual's basic-gain calibration.
    pub const GAIN_MAX_DB: f32 = 30.0;

    /// Input transformer's low end. Derived from the published 5 Hz at
    /// −3 dB.
    pub const IN_HP_HZ: f32 = 5.0;
    /// The high end, as **one Butterworth pair** rather than the cascaded
    /// first-order poles the other optical models use. Derived: the
    /// published −3 dB corner at 25 kHz and a near-flat 20 kHz cannot
    /// both hold for a first-order roll-off, which is 2.1 dB down at
    /// 20 kHz where the pair is 1.3 dB down.
    pub const HF_LP_HZ: f32 = 26e3;
    /// C1 100 pF across R3 47.5 kΩ on the side-chain sheet.
    pub const SC_LP_HZ: f32 = 33.5e3;

    /// The photocell's own odd-order term. A sixth of the LA-2A's,
    /// because the published THD+N is 0.15 % against the LA-2A's 0.9 to
    /// 4.2 %.
    pub const CELL_CUBIC: f32 = 0.1;
    pub const CELL_CUBIC_V0: f32 = 0.25;

    /// V1A, one single-ended ECC83 stage: second harmonic. **Estimate.**
    pub const ASYM: f32 = 0.004;
    /// The push-pull output stage's symmetric ceiling. Placed at the
    /// published +26 dBu maximum output. **Estimate** for the exponent.
    pub const CLIP_N: f32 = 6.0;

    /// The attack integrator's time constant is scaled by this before
    /// it is used. A feedback detector sees the *compressed* signal, so
    /// as the state rises its own target falls and the loop settles
    /// sooner than the pot's RC alone would: measured, about three and a
    /// half times sooner. The research says plainly that the schematic's
    /// component values do not reproduce the published attack range and
    /// that the model should therefore take its *shapes* from the
    /// schematic and its *numbers* from the manual, which is what this
    /// does. **Derived** by measurement against the published range.
    pub const ATTACK_LOOP_K: f32 = 3.4;

    /// The output transformer's low-frequency term. Its job is the
    /// specification's most distinctive property: a distortion figure
    /// quoted at 40 Hz that is *the same* at 0 dBu and +10 dBu. A plain
    /// cubic cannot do that, because its THD rises 20 dB per decade of
    /// level; this one saturates, so the figure rises to a plateau and
    /// then falls, and the plateau is placed between the two published
    /// levels. **Derived from the shape of the published figure**, not
    /// from a measurement of a transformer.
    pub const LF_K: f32 = 0.010;
    /// The level at which that term's distortion peaks, placed between
    /// the two published measurement levels so the figure is flat across
    /// them. **Derived** from the shape of the published figure.
    pub const LF_V0_DBU: f32 = -9.0;
    /// Corner separating "low frequency" from the rest, for that term.
    pub const LF_HZ: f32 = 100.0;

    /// The exponent of the GRE's drive-to-conductance law.
    ///
    /// The research proposed reusing the T4's `CELL_GAMMA` of 0.8, which
    /// comes from the CdS photoconductor literature. But its own section 4
    /// establishes that the CL 1B's element is **not** a T4 and that
    /// nobody outside Lydkraft knows what is inside it, so that figure is
    /// a guess about a different part rather than an anchor for this one.
    /// Meanwhile the manual publishes a real number this exponent
    /// controls: at the 2:1 stop, ten decibels more in gives five more
    /// out. In a feedback loop the output slope is `1/(1 − p)` where `p`
    /// is `dlog(attenuation)/dlog(drive)`, so 2:1 needs `p = −1`, and at
    /// 0.8 the loop settles at about 1.5:1 and cannot reach the published
    /// figure at any depth.
    ///
    /// **Solved from the published ratio** rather than borrowed, and the
    /// value that results sits just above the CdS range, which is
    /// unsurprising for a part that is not a CdS cell.
    pub const GRE_GAMMA: f32 = 1.36;

    /// Release constant of the peak-length envelope (see `Timing::env`).
    /// Short against the shortest delay the attack knob can set, so it
    /// never blurs the measurement it exists to make.
    pub const PEAK_ENV_S: f32 = 0.005;

    /// Parameter smoothing.
    pub const SMOOTH_S: f32 = 0.005;
    /// The panel's own measured dot positions, from Lydkraft's hi-res
    /// front photograph. Gain and Threshold share one artwork: their dot
    /// angles agree to within 2.5° at all six positions, because both are
    /// the same 100 kΩ log pot marked 10 dB per dot.
    ///
    /// The final Gain dot measured at 0.999 and is written 1.0 here, so
    /// that the service manual's "fully clockwise is +30.0 dB" holds
    /// exactly; a thousandth of a knob is well inside the measurement.
    pub const GAIN_DOTS: [(f32, f32); 5] = [
        (0.144, -10.0),
        (0.265, 0.0),
        (0.505, 10.0),
        (0.669, 20.0),
        (1.000, 30.0),
    ];
    pub const THR_DOTS: [(f32, f32); 5] = [
        (0.146, 0.0),
        (0.272, -10.0),
        (0.519, -20.0),
        (0.686, -30.0),
        (1.000, -40.0),
    ];
    /// Below the first Gain dot the knob runs to its `off` stop; the
    /// model fades over that span rather than stepping. **Estimate**: the
    /// panel prints only `off`.
    pub const GAIN_OFF_DB: f32 = -80.0;
}

/// Monotone piecewise-linear interpolation through the panel's measured
/// dots, extrapolating along the end segments.
///
/// The research measured the dot spacing as irregular and not monotone in
/// slope, and Softube, who had the hardware and the designer, say the
/// panel print "isn't very exact" and "the actual numbers on the panel
/// are very approximate". So the dots are reproduced exactly where they
/// were measured, and nothing smoother is claimed between them.
fn through_dots(dots: &[(f32, f32)], p: f32) -> f32 {
    let n = dots.len();
    if p <= dots[0].0 {
        let (x0, y0) = dots[0];
        let (x1, y1) = dots[1];
        return y0 + (p - x0) * (y1 - y0) / (x1 - x0);
    }
    for w in dots.windows(2) {
        let ((x0, y0), (x1, y1)) = (w[0], w[1]);
        if p <= x1 {
            return y0 + (p - x0) * (y1 - y0) / (x1 - x0);
        }
    }
    let (x0, y0) = dots[n - 2];
    let (x1, y1) = dots[n - 1];
    y0 + (p - x0) * (y1 - y0) / (x1 - x0)
}

/// Make-up gain in dB for the Gain knob, 0..1. Exactly +30.0 dB at the
/// clockwise stop, unity at the measured "0" dot, and fading to silence
/// over the span below the "−10" dot where the panel prints only `off`.
#[inline]
pub fn gain_db(p: f32) -> f32 {
    let p = p.clamp(0.0, 1.0);
    let first = k::GAIN_DOTS[0].0;
    if p < first {
        let t = p / first;
        return k::GAIN_OFF_DB + t * (k::GAIN_DOTS[0].1 - k::GAIN_OFF_DB);
    }
    through_dots(&k::GAIN_DOTS, p)
}

/// The Threshold knob's printed value in dBu, 0..1.
#[inline]
pub fn threshold_dbu(p: f32) -> f32 {
    through_dots(&k::THR_DOTS, p.clamp(0.0, 1.0))
}

/// Attack time in seconds. P4 is a 500 kΩ **log** pot, so the law is
/// geometric in rotation.
#[inline]
pub fn attack_s(p: f32) -> f32 {
    k::A_MIN_S * (k::A_MAX_S / k::A_MIN_S).powf(p.clamp(0.0, 1.0))
}

/// Release time in seconds, as a *full recovery from 10 dB*, which is how
/// the service manual measures it. P5 is a 500 kΩ **linear** pot, so the
/// law is affine in rotation. This one component value is what makes the
/// CL 1B a different machine from every compressor with a log release: at
/// the 10 o'clock setting Lydkraft recommend for vocals it gives about
/// 2.5 s, where a log taper would have given about 350 ms.
#[inline]
pub fn release_s(p: f32) -> f32 {
    k::R_MIN_S + p.clamp(0.0, 1.0) * (k::R_MAX_S - k::R_MIN_S)
}

/// The four-element attenuator, with its two nodes.
///
/// This is **not** [`crate::dsp::opto::model::Divider`], which models a
/// two-element network in which the detector and the audio see the same
/// node. Here the Ratio rheostat sits between them, so with the pot at
/// its clockwise stop the detector's view of the reduction saturates
/// while the audio's does not, and the loop stops fighting back. That is
/// the whole mechanism of the Ratio control, and it is why this model
/// needs its own network rather than the shared one.
#[derive(Clone, Copy, Debug)]
pub struct Network {
    r_ratio: f32,
    a_dark: f32,
    c_dark: f32,
}

impl Network {
    /// `ratio` is the knob, 0..1, mapped linearly onto the rheostat
    /// because P2 is a linear pot.
    pub fn new(ratio: f32) -> Self {
        let mut n = Network {
            r_ratio: k::R_RATIO_MAX * ratio.clamp(0.0, 1.0),
            a_dark: 1.0,
            c_dark: 1.0,
        };
        let (c, a) = n.raw(R_DARK);
        n.c_dark = c;
        n.a_dark = a;
        n
    }

    /// Un-normalised (audio, detector) gains for a cell resistance.
    #[inline]
    fn raw(&self, r_gre: f32) -> (f32, f32) {
        let z_c = r_gre * k::R_POT / (r_gre + k::R_POT);
        let z_b = k::R_THR * (self.r_ratio + z_c) / (k::R_THR + self.r_ratio + z_c);
        let a_raw = z_b / (k::R_SERIES + z_b);
        let c_raw = a_raw * z_c / (self.r_ratio + z_c);
        (c_raw, a_raw)
    }

    /// Normalised (audio, detector) gains: both unity with a dark cell.
    #[inline]
    pub fn nodes(&self, r_gre: f32) -> (f32, f32) {
        let (c, a) = self.raw(r_gre);
        (c / self.c_dark, a / self.a_dark)
    }

    /// Gain reduction of the audio path in dB, positive.
    #[inline]
    pub fn gr_db(&self, r_gre: f32) -> f32 {
        -20.0 * self.nodes(r_gre).0.max(1e-6).log10()
    }
}

/// The two calibrations from the service manual, solved rather than
/// written down.
#[derive(Clone, Copy, Debug)]
pub struct Calibration {
    /// Conductance scale of the GRE at the 10 dB point.
    pub k_g: f32,
    /// Side-chain gain offset in dB.
    pub g0_db: f32,
}

/// Cell resistance giving a target audio attenuation at the 2:1 stop,
/// by bisection. Monotone, so bisection is exact enough and cannot
/// diverge.
fn resistance_for_attenuation(net: &Network, target: f32) -> f32 {
    let (mut lo, mut hi) = (k::R_GRE_MIN, R_DARK);
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if net.nodes(mid).0 < target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

impl Calibration {
    /// Solve both anchors.
    ///
    /// The first is the service manual's compression-tracking adjustment:
    /// with the Ratio at 2:1, +250.0 mV of control gives exactly −10.0 dB.
    /// That fixes the conductance scale outright, with no freedom left.
    ///
    /// The second is the panel's own −20 dot: at that Threshold setting,
    /// 1 dB of reduction occurs at a 1 kHz input of −20 dBu, which is the
    /// manual's own definition of the control applied to its own scale.
    pub fn solve() -> Self {
        let net = Network::new(0.0);
        let r10 = resistance_for_attenuation(&net, 10f32.powf(-10.0 / 20.0));
        let k_g = 1.0 / r10 - 1.0 / R_DARK;

        let r1 = resistance_for_attenuation(&net, 10f32.powf(-1.0 / 20.0));
        let g1 = 1.0 / r1 - 1.0 / R_DARK;
        let u1 = k::U_REF_10DB * (g1 / k_g).powf(1.0 / k::GRE_GAMMA);

        // A 1 kHz sine at −20 dBu, seen at the detector node, which is
        // unity here because the reduction is only 1 dB and the anchor is
        // defined at the threshold of compression.
        let peak = VU_REF_AMP * 10f32.powf((-20.0 - 4.0) / 20.0);
        let mean = peak * SINE_MEAN_ABS * net.nodes(r1).1;
        let want = u1 / mean;
        let g0_db = 20.0 * want.log10() + threshold_dbu(0.519);
        Calibration { k_g, g0_db }
    }

    /// Side-chain gain for a Threshold knob position.
    ///
    /// Below the first printed dot the pot's wiper runs down to its stop,
    /// so the drive fades to nothing and the compressor is off, which is
    /// what the panel's `off` legend means.
    #[inline]
    pub fn thr_gain(&self, p: f32) -> f32 {
        let p = p.clamp(0.0, 1.0);
        let first = k::THR_DOTS[0].0;
        let full = 10f32.powf((self.g0_db - threshold_dbu(p.max(first))) / 20.0);
        if p < first { full * (p / first) } else { full }
    }

    /// The GRE's resistance for a control current.
    ///
    /// The research's pseudocode clamps the drive ratio to 1, which would
    /// cap the model at the 10 dB calibration point; its own
    /// `R_GRE_MIN` constant exists precisely to set the maximum
    /// reduction, so the clamp belongs on the resistance instead. Noted
    /// because it is a departure from the written design.
    #[inline]
    pub fn resistance(&self, i: f32) -> f32 {
        let drive = (i / k::U_REF_10DB).max(0.0);
        let g = 1.0 / R_DARK + self.k_g * drive.powf(k::GRE_GAMMA);
        (1.0 / g).clamp(k::R_GRE_MIN, R_DARK)
    }
}

/// The state of one timing circuit.
#[derive(Clone, Copy, Default)]
struct Timing {
    /// Control voltage on C3.
    u: f32,
    /// How long the state has been rising or holding, in seconds.
    held_s: f32,
    /// Fix/Man: seconds of fixed-rate release still owed.
    delay_left_s: f32,
    /// Whether the state is falling. Needed so the peak's length is
    /// measured once, at the falling edge: a sustained peak sits at
    /// `d == u`, and treating that as a release would restart the
    /// measurement every sample and make every peak look instantaneous.
    releasing: bool,
    /// A short envelope of the detector output, used **only** to measure
    /// how long a programme peak lasts for Fix/Man.
    ///
    /// The charge and discharge paths compare the instantaneous rectified
    /// signal against the stored voltage, as the schematic does. But a
    /// rectified sine crosses that stored voltage twice a cycle, so
    /// timing a "peak" by those crossings would measure a fraction of a
    /// cycle rather than the length of the programme peak the manual is
    /// talking about. This envelope is what the peak length is measured
    /// against instead.
    env: f32,
}

impl Timing {
    fn reset(&mut self) {
        *self = Timing::default();
    }
}

/// One channel's filters and resamplers.
#[derive(Clone, Default)]
struct Channel {
    in_hp: OnePole,
    sc_hpf: Biquad,
    sc_lp: OnePole,
    lf_hp: OnePole,
    hf_lp: Biquad,
    /// The output stage runs at twice the rate below 88.2 kHz: the
    /// ceiling and the low-frequency term both make high harmonics.
    up: Upsampler,
    down: Downsampler,
    dry: DryDelay,
}

impl Channel {
    fn reset(&mut self) {
        self.in_hp.reset();
        self.sc_hpf.reset();
        self.sc_lp.reset();
        self.lf_hp.reset();
        self.hf_lp.reset();
        self.up.reset();
        self.down.reset();
        self.dry.reset();
    }
}

/// A snapshot of the panel.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// Gain knob, 0..1. Never affects compression.
    pub gain: f32,
    /// Ratio knob, 0..1: the rheostat, 2:1 at 0 and 10:1 at 1.
    pub ratio: f32,
    /// Threshold knob, 0..1. Clockwise *lowers* the threshold.
    pub threshold: f32,
    /// Attack knob, 0..1. In Fix/Man this sets a delay, not an attack.
    pub attack: f32,
    /// Release knob, 0..1.
    pub release: f32,
    /// [`MODE_FIXED`], [`MODE_FIXMAN`] or [`super::MODE_MANUAL`].
    pub mode: usize,
    /// [`METER_IN`], [`METER_COMP`] or [`METER_OUT`].
    pub meter: usize,
    /// Side-chain bus: 0 = off, 1 and 2 are the two busses.
    pub bus: usize,
    /// The lab's shared link toggle. On this model it gates the panel's
    /// own bus switch rather than replacing it: the hardware links units
    /// through the bus, so a CL 1B with its bus off is not linked to
    /// anything, and the default is therefore unlinked where the other
    /// models default linked. That is the hardware's behaviour, not an
    /// oversight.
    pub link: bool,
    /// Wet share, 0..1. Not on the hardware; Softube, Universal Audio and
    /// Stam Audio all added one.
    pub mix: f32,
    /// Side-chain high-pass in Hz, 0 = off. The lab's shared extra. The
    /// hardware's detector is flat, so the default must be off, and test
    /// 30 depends on it.
    pub sc_hpf: f32,
    /// The panel's IN lever, which bypasses the whole compressor.
    pub bypass: bool,
    /// The panel's OFF/ON mains knob at the far right.
    ///
    /// **A deliberate divergence from the hardware, and the reason for
    /// it.** A real CL 1B with its mains off passes nothing at all,
    /// because its audio path runs through the tube stages. This model
    /// passes the input through instead and parks the meter, which is
    /// exactly what the 1176 in this same plug-in does when its METER
    /// switch is turned to OFF. Two power switches inside one product
    /// that behave differently would be worse than either choice on its
    /// own, and a user who clicks a panel switch and gets silence will
    /// reasonably think the plug-in has broken.
    pub power: bool,
}

impl Default for Settings {
    /// Lydkraft's own vocal setting, which is also the panel's shipped
    /// position: Manual, attack at 2 o'clock, release at 10 o'clock.
    fn default() -> Self {
        Settings {
            gain: 0.265,
            ratio: 0.375,
            threshold: 0.5,
            attack: 0.75,
            release: 0.25,
            mode: super::MODE_MANUAL,
            meter: METER_COMP,
            bus: 0,
            link: true,
            mix: 1.0,
            sc_hpf: 0.0,
            bypass: false,
            power: true,
        }
    }
}

/// The single-ended stage and the push-pull ceiling.
#[inline]
fn amp(w: f32, v_clip: f32) -> f32 {
    let w = w + k::ASYM * w * w;
    let u = (w / v_clip).abs().powf(k::CLIP_N);
    w / (1.0 + u).powf(1.0 / k::CLIP_N)
}

/// The stereo CL 1B.
pub struct Compressor {
    sr: f32,
    settings: Settings,
    cal: Calibration,
    net: Network,
    ch: [Channel; 2],
    timing: [Timing; 2],
    /// The GRE's own lag, one per channel.
    cell_i: [f32; 2],
    r_gre: [f32; 2],
    thr_gain: f32,
    makeup: f32,
    makeup_z: f32,
    mix_z: f32,
    smooth_a: f32,
    env_a: f32,
    a_attack: f32,
    a_fixed: f32,
    slew: f32,
    slew_fixed: f32,
    delay_s: f32,
    cell_a: f32,
    v_clip: f32,
    lf_v0: f32,
    oversample: bool,
    gr_db: [f32; 2],
    out_abs: [f32; 2],
    in_abs: [f32; 2],
    in_peak: [f32; 2],
    out_peak: [f32; 2],
}

impl Compressor {
    pub fn new(sr: f32) -> Self {
        let s = Settings::default();
        let mut c = Compressor {
            sr,
            settings: s,
            cal: Calibration::solve(),
            net: Network::new(s.ratio),
            ch: [Channel::default(), Channel::default()],
            timing: [Timing::default(); 2],
            cell_i: [0.0; 2],
            r_gre: [R_DARK; 2],
            thr_gain: 1.0,
            makeup: 1.0,
            makeup_z: 1.0,
            mix_z: 1.0,
            smooth_a: 0.0,
            env_a: 0.0,
            a_attack: 0.0,
            a_fixed: 0.0,
            slew: 0.0,
            slew_fixed: 0.0,
            delay_s: 0.0,
            cell_a: 0.0,
            v_clip: 1.0,
            lf_v0: 1.0,
            oversample: sr < 88_200.0,
            gr_db: [0.0; 2],
            out_abs: [0.0; 2],
            in_abs: [0.0; 2],
            in_peak: [0.0; 2],
            out_peak: [0.0; 2],
        };
        c.set_sample_rate(sr);
        c
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sr = sr;
        self.oversample = sr < 88_200.0;
        self.smooth_a = 1.0 - (-1.0 / (k::SMOOTH_S * sr)).exp();
        let s = self.settings;
        self.rebuild(s);
        self.reset();
    }

    /// Recompute everything a setting or the rate implies.
    fn rebuild(&mut self, s: Settings) {
        let sr = self.sr;
        let nyq = 0.45 * sr;
        for ch in &mut self.ch {
            ch.in_hp.set(k::IN_HP_HZ, sr);
            // The 33.5 kHz side-chain pole is above Nyquist at 44.1 kHz,
            // so it is clamped rather than allowed to alias.
            ch.sc_lp.set(k::SC_LP_HZ.min(nyq), sr);
            ch.lf_hp.set(k::LF_HZ, sr);
            ch.hf_lp.set_lowpass(k::HF_LP_HZ, sr);
            if s.sc_hpf > 1.0 {
                ch.sc_hpf.set_highpass(s.sc_hpf, sr);
            }
        }
        self.net = Network::new(s.ratio);
        self.thr_gain = self.cal.thr_gain(s.threshold);
        self.makeup = 10f32.powf(gain_db(s.gain) / 20.0);
        self.cell_a = 1.0 - (-1.0 / (k::TAU_CELL_S * sr)).exp();
        self.env_a = 1.0 - (-1.0 / (k::PEAK_ENV_S * sr)).exp();
        self.a_attack = 1.0 - (-1.0 / (attack_s(s.attack) * k::ATTACK_LOOP_K * sr)).exp();
        self.a_fixed = 1.0 - (-1.0 / (k::A_FIXED_S * k::ATTACK_LOOP_K * sr)).exp();
        // A full recovery from the 10 dB calibration point takes the
        // knob's time, which is how the service manual measures it.
        self.slew = k::U_REF_10DB / release_s(s.release);
        self.slew_fixed = k::U_REF_10DB / k::R_FIXED_S;
        self.delay_s = attack_s(s.attack);
        // Placed so that 1 % distortion arrives at the published +26 dBu
        // maximum output, which is a little below the hard ceiling.
        self.v_clip = VU_REF_AMP * 10f32.powf((29.0 - 4.0) / 20.0);
        self.lf_v0 = VU_REF_AMP * 10f32.powf((k::LF_V0_DBU - 4.0) / 20.0);
    }

    pub fn reset(&mut self) {
        for ch in &mut self.ch {
            ch.reset();
        }
        for t in &mut self.timing {
            t.reset();
        }
        self.cell_i = [0.0; 2];
        self.r_gre = [R_DARK; 2];
        self.makeup_z = self.makeup;
        self.mix_z = self.settings.mix;
        self.gr_db = [0.0; 2];
        self.out_abs = [0.0; 2];
        self.in_abs = [0.0; 2];
        self.in_peak = [0.0; 2];
        self.out_peak = [0.0; 2];
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// The calibration this model solved at construction.
    pub fn calibration(&self) -> Calibration {
        self.cal
    }

    /// The attenuator for the current Ratio setting.
    pub fn network(&self) -> Network {
        self.net
    }

    /// The resamplers' round trip, when they are running.
    pub fn latency(&self) -> usize {
        if self.oversample { LATENCY } else { 0 }
    }

    /// Apply a snapshot; `true` when anything changed.
    pub fn configure(&mut self, s: Settings) -> bool {
        if s == self.settings {
            return false;
        }
        self.settings = s;
        self.rebuild(s);
        true
    }

    /// Gain reduction in dB, positive.
    pub fn gain_reduction_db(&self, channel: usize) -> f32 {
        self.net.gr_db(self.r_gre[channel.min(1)])
    }

    /// `[control_volts, cell_resistance, drive_fraction]`.
    ///
    /// The other optical models publish the T4's three states here. This
    /// one has no such states, so it publishes what it does have: the
    /// control voltage the electronics hold and the resistance the
    /// element presents.
    pub fn cell_state(&self) -> [f32; 3] {
        [
            self.timing[0].u,
            self.r_gre[0],
            (self.cell_i[0] / k::U_REF_10DB).min(4.0),
        ]
    }

    /// Advance one timing circuit by one sample and return its control
    /// voltage.
    #[inline]
    fn step_timing(&mut self, c: usize, d: f32) -> f32 {
        let t_step = 1.0 / self.sr;
        let mode = self.settings.mode;
        let t = &mut self.timing[c];
        // The peak-length envelope: instant attack, short release.
        t.env = if d > t.env {
            d
        } else {
            t.env + self.env_a * (d - t.env)
        };
        let at_peak = t.env >= 0.9 * t.u;
        if d >= t.u {
            // Attacking, or holding at the peak. Fixed and Fix/Man both
            // use the fixed circuit; only Manual uses the knob. Getting
            // this wrong is the trap the manual warns about, and there is
            // a test for it.
            let a = if mode == MODE_FIXED || mode == MODE_FIXMAN {
                self.a_fixed
            } else {
                self.a_attack
            };
            t.u += a * (d - t.u);
            t.held_s += t_step;
            t.releasing = false;
        } else {
            if at_peak {
                // Still inside the programme peak, between two crossings
                // of the rectified waveform.
                t.held_s += t_step;
                t.releasing = false;
            } else if !t.releasing {
                // The falling edge, measured once. The attack knob buys a
                // stretch of the fixed release, but only for as long as
                // the peak was shorter than the knob's setting; a peak
                // longer than the setting gets the manual release from
                // the start, which is what the manual describes.
                t.releasing = true;
                if mode == MODE_FIXMAN {
                    t.delay_left_s = (self.delay_s - t.held_s).max(0.0);
                }
                t.held_s = 0.0;
            }
            let slew = match mode {
                MODE_FIXED => self.slew_fixed,
                MODE_FIXMAN if t.delay_left_s > 0.0 => {
                    t.delay_left_s = (t.delay_left_s - t_step).max(0.0);
                    self.slew_fixed
                }
                _ => self.slew,
            };
            t.u -= (t.u - d).min(slew * t_step);
        }
        t.u = flush(t.u.max(0.0));
        t.u
    }

    /// Process one stereo block in place. Real-time safe.
    pub fn process_block(&mut self, l: &mut [f32], r: &mut [f32]) {
        let n = l.len().min(r.len());
        let s = self.settings;
        let mut gr_sum = [0.0f32; 2];
        let mut out_abs = [0.0f32; 2];
        let mut in_abs = [0.0f32; 2];
        let mut in_peak = [0.0f32; 2];
        let mut out_peak = [0.0f32; 2];
        // The hardware links through the bus, so an instance whose bus
        // switch is off is not linked to anything.
        let linked = s.link && s.bus != 0;
        // The mains knob parks the machine without silencing it; see
        // `Settings::power`.
        let processing = !s.bypass && s.power;
        let oversample = self.oversample;
        let use_hpf = s.sc_hpf > 1.0;
        for i in 0..n {
            let x = [l[i], r[i]];
            for c in 0..2 {
                in_peak[c] = in_peak[c].max(x[c].abs());
                in_abs[c] += x[c].abs();
            }
            if !processing {
                for c in 0..2 {
                    if oversample {
                        self.ch[c].dry.process(x[c]);
                    }
                }
                out_peak = in_peak;
                out_abs[0] += x[0].abs();
                out_abs[1] += x[1].abs();
                continue;
            }
            let mut d = [0.0f32; 2];
            let mut y = [0.0f32; 2];
            let mut a_audio = [1.0f32; 2];
            for c in 0..2 {
                let (a_c, a_b) = self.net.nodes(self.r_gre[c]);
                a_audio[c] = a_c;
                let ch = &mut self.ch[c];
                // Input transformer.
                let xh = ch.in_hp.hp(x[c]);
                // Audio node, and the photocell's own small odd term. A
                // photoresistor distorts in proportion to the voltage
                // across it, which is why it is scaled by the reduction.
                let att = distortion(xh * a_c, a_c, k::CELL_CUBIC, k::CELL_CUBIC_V0);
                // Detector: node B, not the audio node. This is the whole
                // mechanism of the Ratio control.
                let mut sc = xh * a_b;
                if use_hpf {
                    sc = ch.sc_hpf.process(sc);
                }
                sc = ch.sc_lp.lp(sc);
                d[c] = sc.abs() * self.thr_gain;
                y[c] = att;
            }
            // One control state when the bus is joined, and it takes the
            // larger of the two: "the unit which performs the most
            // compression is controlling the others". The other optical
            // models average; this one does not, and there is a test.
            if linked {
                let dm = d[0].max(d[1]);
                let u = self.step_timing(0, dm);
                self.timing[1] = self.timing[0];
                for c in 0..2 {
                    self.cell_i[c] += self.cell_a * (u - self.cell_i[c]);
                    self.r_gre[c] = self.cal.resistance(self.cell_i[c]);
                }
            } else {
                for c in 0..2 {
                    let u = self.step_timing(c, d[c]);
                    self.cell_i[c] += self.cell_a * (u - self.cell_i[c]);
                    self.r_gre[c] = self.cal.resistance(self.cell_i[c]);
                }
            }
            self.mix_z += self.smooth_a * (s.mix - self.mix_z);
            self.makeup_z += self.smooth_a * (self.makeup - self.makeup_z);
            let v_clip = self.v_clip;
            let lf_v0 = self.lf_v0;
            for c in 0..2 {
                let ch = &mut self.ch[c];
                let w = y[c] * self.makeup_z;
                let shaped = if oversample {
                    let pair = ch.up.process(w);
                    ch.down
                        .process([amp(pair[0], v_clip), amp(pair[1], v_clip)])
                } else {
                    amp(w, v_clip)
                };
                // The output transformer's low-frequency term.
                let hp = ch.lf_hp.hp(shaped);
                let lf = shaped - hp;
                let lq = lf / lf_v0;
                let lq2 = lq * lq;
                let shaped = shaped + k::LF_K * lf * lq2 / (1.0 + lq2);
                let wet = ch.hf_lp.process(shaped);
                let dry_c = if oversample {
                    ch.dry.process(x[c])
                } else {
                    x[c]
                };
                let out = flush(self.mix_z * wet + (1.0 - self.mix_z) * dry_c);
                out_peak[c] = out_peak[c].max(out.abs());
                out_abs[c] += out.abs();
                gr_sum[c] += -20.0 * a_audio[c].max(1e-6).log10();
                if c == 0 { l[i] = out } else { r[i] = out }
            }
        }
        let inv = 1.0 / n.max(1) as f32;
        self.gr_db = [gr_sum[0] * inv, gr_sum[1] * inv];
        self.out_abs = [out_abs[0] * inv, out_abs[1] * inv];
        self.in_abs = [in_abs[0] * inv, in_abs[1] * inv];
        self.in_peak = in_peak;
        self.out_peak = out_peak;
    }

    /// `[in_l, in_r, out_l, out_r, gr_db, meter_vu]` for the last block,
    /// with `gr_db` positive (the lab negates it).
    ///
    /// The meter switch has no Off position on this machine, so unlike
    /// the LA-3A's there is nothing here that parks the needle.
    pub fn meter_frame(&self) -> [f32; 6] {
        let gr = 0.5 * (self.gr_db[0] + self.gr_db[1]);
        let out_mean = 0.5 * (self.out_abs[0] + self.out_abs[1]);
        let in_mean = 0.5 * (self.in_abs[0] + self.in_abs[1]);
        let vu = if !self.settings.power {
            // Parked: the lamp is out and the movement is unpowered.
            -60.0
        } else {
            match self.settings.meter {
                METER_IN => 20.0 * (in_mean / VU_REF_MEAN).max(1e-4).log10(),
                METER_OUT => 20.0 * (out_mean / VU_REF_MEAN).max(1e-4).log10(),
                _ => -gr,
            }
        };
        [
            self.in_peak[0],
            self.in_peak[1],
            self.out_peak[0],
            self.out_peak[1],
            gr,
            vu,
        ]
    }

    /// Steady-state gain reduction in dB (positive) for a 1 kHz sine of
    /// peak `amp_peak`.
    ///
    /// Solved as a damped fixed point rather than by running the loop,
    /// because the release can be ten seconds long and running it would
    /// cost half a million samples per point of the transfer curve.
    pub fn static_gr_db(&self, amp_peak: f32) -> f32 {
        if self.settings.bypass || !self.settings.power || self.thr_gain <= 0.0 {
            return 0.0;
        }
        let mut r = R_DARK;
        for _ in 0..400 {
            let (_, a_b) = self.net.nodes(r);
            let d = amp_peak * a_b * SINE_MEAN_ABS * self.thr_gain;
            let target = self.cal.resistance(d);
            let next = r + 0.35 * (target - r);
            if (next - r).abs() < 0.5 {
                r = next;
                break;
            }
            r = next;
        }
        self.net.gr_db(r)
    }

    /// Fill `out` with the static output level in dBFS for inputs from
    /// `min_dbfs` to `max_dbfs`.
    pub fn transfer_curve(&self, out: &mut [f32], min_dbfs: f32, max_dbfs: f32) {
        let n = out.len();
        if n == 0 {
            return;
        }
        let makeup = gain_db(self.settings.gain);
        for (i, o) in out.iter_mut().enumerate() {
            let x = min_dbfs + (max_dbfs - min_dbfs) * i as f32 / (n - 1).max(1) as f32;
            let gr = self.static_gr_db(10f32.powf(x / 20.0));
            let wet = x - gr + makeup;
            *o = if self.settings.bypass || !self.settings.power {
                x
            } else {
                let dry = 10f32.powf(x / 20.0);
                let w = 10f32.powf(wet / 20.0);
                20.0 * (dry + (w - dry) * self.settings.mix).max(1e-6).log10()
            };
        }
    }
}

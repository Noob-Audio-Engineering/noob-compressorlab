//! The EMI TG12413 engine.
//!
//! `research/TG12413.md` is what this is built from; section 11 is the
//! design and section 12 the test plan. Every constant below says where it
//! came from, because this unit is the hardest case in the lab for that:
//! **no factory handbook, no specification and no measurement of any kind
//! has ever been published for it** (8.1). What exists is one photographed
//! blueprint, two manufacturers' prose about their own recreations, and
//! arithmetic.
//!
//! # Where the sidechain listens, and why that is a ruling rather than a
//! reading
//!
//! Section 11.4 says the detector reads the "post-element,
//! post-output-ladder" signal. Section 12's test 4 says gain reduction
//! must not move by more than 0.1 dB when the output ladder is swept, and
//! calls that a circuit identity because "S3 sits after the gain element
//! in the signal path". Both cannot be true: a detector behind the ladder
//! is moved by the ladder.
//!
//! **This engine taps after the element and its post amplifier and before
//! the ladder.** That is still feedback, which is what 11.1 argues for on
//! three grounds, and it satisfies the identity. The identity is the
//! tighter statement and identities do not have tolerances, so it wins.
//!
//! # What is fitted, and against what
//!
//! Three constants, all named at their definitions, and the honest
//! position is that none of them has a source about *this* unit:
//!
//! * [`THRESHOLD_AMP`] — there is no threshold control on the hardware and
//!   no published threshold, so where the unit starts working is a choice.
//! * [`Compressor::k_i`] — fitted so that a full-scale sine settles at
//!   20 dB of gain reduction, which is the dossier's own instruction in
//!   11.6 and not a figure about the hardware.
//! * [`ELEMENT_DRIVE_V`] — EMI printed no level annotations anywhere on
//!   the sheet (3.5), so unlike the Neve there is nothing to derive this
//!   from. It is fitted to the only distortion figures that exist within
//!   reach of this circuit, which are the two ends of the THD scale
//!   Chandler print on the TG1's input knob.

use crate::dsp::flush;
use crate::dsp::opto::filters::OnePole;

use super::element::{Element, R_SERIES};
use super::oversample::{Chain, Delay, latency};
use super::{MODE_COMPRESS, MODE_LIMIT, MODE_OUT, REGION_FORWARD};

/// 0 VU is +4 dBu and −18 dBFS RMS, the calibration the whole lab uses.
pub const VU_REFERENCE_DBFS: f32 = -18.0;

/// Peak amplitude of a sine at 0 VU, i.e. at +4 dBu.
pub const VU_REF_AMP: f32 = 0.125_892_54 * std::f32::consts::SQRT_2;

/// Peak amplitude of a sine at `x` dBu.
pub fn dbu_amp(x: f32) -> f32 {
    VU_REF_AMP * 10f32.powf((x - 4.0) / 20.0)
}

/// A level in dBu from a peak amplitude.
pub fn amp_dbu(a: f32) -> f32 {
    20.0 * (a.max(1e-9) / VU_REF_AMP).log10() + 4.0
}

// ---------------------------------------------------------- output ladder

/// S3's twenty-one accumulated resistances, in ohms, from tag 1 upward.
///
/// **Read off drawing TG12413-D101**, assembly `TG12413 B203A`, and
/// summed from a tap to the top of the chain. The ladder is the input arm
/// of a virtual-earth stage, so the level goes as −20·log₁₀(R) exactly,
/// which is why these reproduce EMI's printed "1 dB STEPS" legend to
/// within a tenth of a decibel (dossier 3.4).
pub const R_LADDER: [f32; 21] = [
    32_110.0, 28_810.0, 25_510.0, 22_810.0, 20_410.0, 18_210.0, 16_210.0, 14_410.0, 12_810.0,
    11_510.0, 10_210.0, 9_110.0, 8_110.0, 7_200.0, 6_380.0, 5_700.0, 5_080.0, 4_570.0, 4_060.0,
    3_630.0, 3_300.0,
];

/// The index EMI's legend calls 0 dB.
pub const OUTPUT_UNITY: usize = 10;

/// The ladder's gain at a switch position.
///
/// **A ratio of resistances, never a decibel.** Interpolating decibels
/// would be smoother than the hardware and would throw away the ±0.09 dB
/// per-step error the real ladder has, which is free and which test 1
/// checks for.
pub fn output_gain(i: usize) -> f32 {
    R_LADDER[OUTPUT_UNITY] / R_LADDER[i.min(20)]
}

/// The ladder's real gain in dB at a switch position.
pub fn output_db(i: usize) -> f32 {
    20.0 * output_gain(i).log10()
}

/// What the panel is silkscreened with at that position: −10 to +10 in
/// exact decibels. The engine uses [`output_db`] and the page prints this,
/// which is what the hardware does.
pub fn output_marked_db(i: usize) -> f32 {
    i.min(20) as f32 - 10.0
}

// -------------------------------------------------------- recovery ladder

/// S2's six accumulated resistances, in ohms.
///
/// **Read off the drawing**, assembly `TG12413 B204A`: 47 K, 47 K, 130 K,
/// 220 K, 470 K and 1 M3 in a series chain, accumulated from the common
/// end. Their ratios are the only hard timing number in the whole
/// document, and test 3 asserts the ratios rather than the times because
/// Waves, who had the console, say the times "are very hard to put in
/// terms of exact milliseconds".
pub const R_RECOVERY: [f32; 6] = [
    47_000.0,
    94_000.0,
    224_000.0,
    444_000.0,
    914_000.0,
    2_214_000.0,
];

/// RV1, the HOLD preset, in ohms. **Read off the drawing.**
pub const R_HOLD: f32 = 10_000.0;

/// C18, the storage capacitor, in farads.
///
/// **Estimate.** Three capacitors on the sheet could be the store and the
/// drawing does not say which the recovery ladder discharges. 1 µF is
/// chosen on plausibility in 7.3: it is the only one of the three whose
/// range is sensible for a programme-dependent limiter, and it puts
/// positions 3, 4 and 5 — which Waves call the useful ones for mastering —
/// at 224 ms, 444 ms and 914 ms. **The ratios do not depend on this and
/// the absolutes do**, so moving this one number moves the whole column.
pub const C_STORE: f32 = 1e-6;

/// The recovery time constant in seconds at a switch position, with HOLD
/// at `hold` of its travel.
///
/// RV1 is 10 kΩ **in series with the ladder**, per 7.4, which is why HOLD
/// is worth 21 % at position 1 and under half a per cent at position 6
/// without any special-casing. Test 5 asserts both ends.
pub fn recovery_s(i: usize, hold: f32) -> f32 {
    (R_RECOVERY[i.min(5)] + R_HOLD * hold.clamp(0.0, 1.0)) * C_STORE
}

/// The attack time constant, in seconds.
///
/// **Estimate, and the dossier says so twice.** R41 39 K plus R47 8K2 into
/// C18 gives 47 ms, but all three candidate stores give a figure that is
/// far too slow for a limiter, which says the charge path is current-driven
/// by VT17 rather than resistively driven. So 47 ms is an upper bound and
/// 5 ms is the dossier's starting value. **No source publishes an attack
/// time for this unit and section 12 does not test one.**
pub const TAU_ATTACK: f32 = 0.005;

// --------------------------------------------------------- the sidechain

/// The forward drop of a germanium diode, in volts. Textbook, not measured.
pub const V_GE: f32 = 0.250;

/// D8, D9 and D10: three germanium diodes in a string. **Read off the
/// drawing.**
pub const N_GE_REF: f32 = 3.0;

/// The threshold reference, in volts.
///
/// The threshold is a **subtraction against a fixed reference**, not a
/// comparison against a control, because there is no threshold control on
/// this unit (5.3).
pub const V_REF: f32 = N_GE_REF * V_GE;

/// The law network's first slope, below [`V_LAW`]. **Estimate.**
///
/// # The one place this engine reverses the dossier
///
/// Section 11.6 starts the two slopes at `LAW_A = 1.0` and
/// `LAW_B = 0.35`, so the law is steep and then shallow. This engine has
/// them the other way round, keeping the dossier's ratio and inverting its
/// direction, and the reason is worth stating in full because it is a
/// departure.
///
/// The drawing carries six resistors in two rows of three: R31 2 K with
/// AOT 3 and AOT 4, and R32 820 with AOT 5 and AOT 6. Section 5.4 reads
/// that as two law segments with two selected components each, "one
/// setting where the law starts and one setting its slope", and says
/// plainly that the reading is inferred, that the drawing carries no
/// annotation for the block, and that **no value is given for any of the
/// four adjust-on-test parts**. So the slopes are the AOTs, which are
/// unknown, and R31 and R32 are the fixed heads. Nothing on the sheet
/// fixes which segment is the steep one.
///
/// What does bear on it is behaviour. The dossier's section 9.2 lists
/// "germanium rectification, so a softer onset — the TG should start
/// compressing earlier and **more gradually**" as one of the six
/// differences it stakes the model on, and 9.1's four manufacturer quotes
/// are consistent about "smooth", "squishy" and "warm open". A law that is
/// steep first and shallow after produces the opposite: a hard grab at the
/// threshold that relaxes. Inverted, it produces a gentle first decibel,
/// then a firmer middle, then the element's own law easing off at depth,
/// which is the shape all four quotes describe.
///
/// The ratio is the dossier's 0.35 and only the direction has moved.
pub const LAW_A: f32 = 0.35;

/// The law network's second slope, above [`V_LAW`]. **Estimate**, same
/// provenance and the same reversal.
pub const LAW_B: f32 = 1.0;

/// Where the law's two straight segments meet, in volts. **Estimate.**
///
/// Two straight segments and not a curve: that is what two rows of three
/// resistors builds (5.4), and smoothing it would be modelling something
/// EMI did not build.
pub const V_LAW: f32 = 0.100;

/// The mode wafer's positive-rail resistors, in ohms. **Read off the
/// drawing**: R48, R49, R50 around VT18.
pub const R_MODE_POS: [f32; 3] = [62_000.0, 82_000.0, 120_000.0];

/// The mode wafer's negative-rail resistors, in ohms. **Read off the
/// drawing**: R51, R52, R53 around VT19.
pub const R_MODE_NEG: [f32; 3] = [62_000.0, 82_000.0, 20_000.0];

/// The COMPRESS pair, which both poles share, used as the reference.
pub const R_MODE_REF: f32 = 62_000.0;

/// The detector's two half-cycle gains for a mode.
///
/// **This is the reading of section 6.2 that makes the modes different in
/// shape rather than in scale.** S1 is a two-pole three-way wafer picking
/// one resistor on each supply polarity, so it does not switch the
/// sidechain in and out — it re-scales the detector's drive on the two
/// polarities *independently*. COMPRESS is 62 K against 62 K and is
/// symmetric; LIMIT is 120 K against 20 K and is asymmetric six to one.
///
/// An asymmetric rectifier feeding a single store is what turns a soft law
/// into a hard one without touching the element, and it is also what puts
/// signal-frequency ripple on the control current where a symmetric one
/// puts only twice-signal-frequency ripple. That ripple is not filtered
/// away here (11.4).
///
/// OUT is the exception and it is not read off the resistors. 6.2 reads
/// the symmetric 82 K pair as leaving the sidechain "biased but
/// ineffective rather than disconnected", 11.6's own table gives OUT a
/// drive of zero, and test 14 requires exactly 0 dB of gain reduction with
/// the element still in circuit. So OUT is a dead detector and a live
/// element, which is precisely the distinction Chandler draw when they say
/// their equivalent mode "allows bypassing of the compressor/limiter
/// threshold but leaves all circuits in the signal path".
pub fn mode_gains(mode: usize) -> (f32, f32) {
    if mode == MODE_OUT {
        return (0.0, 0.0);
    }
    let m = mode.min(2);
    (R_MODE_POS[m] / R_MODE_REF, R_MODE_NEG[m] / R_MODE_REF)
}

/// Peak amplitude at which gain reduction begins in COMPRESS.
///
/// **Chosen, not derived.** There is no threshold control on the hardware
/// and no published threshold in dBu, and 12.6 lists that as one of the
/// eight things it refuses to test. −30 dBFS puts the onset far enough
/// below programme level that the unit is always doing something, which is
/// what a module with no threshold control has to be, and leaves room for
/// the calibration below to reach 20 dB of reduction inside a digital
/// full scale. `tg_input` moves it by ±12 dB.
pub const THRESHOLD_AMP: f32 = 0.031_623;

/// Volts at the detector for unit amplitude at the sidechain tap.
///
/// Set by [`THRESHOLD_AMP`]: for a large argument the soft rectifier
/// settles to `|a| − V_GE·ln2`, so the threshold is crossed when the
/// detector's drive reaches `V_REF + V_GE·ln2`.
pub const V_SC_DRIVE: f32 = (V_REF + V_GE * std::f32::consts::LN_2) / THRESHOLD_AMP;

/// The standing control current with the detector at rest, in amps.
///
/// Zero, so the element is an open circuit and the module is at unity
/// gain when it is not working. The drawing's DC operating point is not
/// derivable — R14 feeds the bias from +20 V but the control transistors
/// sit across it and EMI printed no node voltages (3.5) — so a rest state
/// had to be chosen and unity is the one that makes the mode switch honest.
pub const I_MIN: f32 = 0.0;

/// The input's gain-reduction target for the calibration of `K_I`.
pub const CAL_GR_DB: f32 = 20.0;

/// The input amplitude at which that target is met: digital full scale.
pub const CAL_INPUT_AMP: f32 = 1.0;

/// Volts across the element's source arm for unit amplitude at the input.
///
/// **Fitted, and it is the constant this model has least support for.**
/// The Neve's drawings print the signal level in dBu at six nodes and its
/// model is calibrated against that chain; EMI printed nothing of the kind
/// anywhere on TG12413-D101, which 3.5 calls the single biggest gap in the
/// evidence base.
///
/// So it is fitted to the nearest thing to a distortion figure that exists
/// for this circuit: **Chandler print a THD scale on the TG1's input knob
/// running from `.04%` to `2%`** (2.7). That is a figure about a licensed
/// recreation with its own added stages, not about EMI's module, and it is
/// used as a **range** rather than as a point target. The distinction the
/// build contract draws is between hardware and somebody's emulation: the
/// TG1 is a unit Chandler build under licence from EMI, not a plug-in, so
/// it is the nearest hardware there is. It is still not the hardware, and
/// the model is not tuned to match it. At this value the element's
/// third harmonic runs from about 0.04 % where the unit is just working to
/// about 0.13 % at 20 dB of reduction, and reaches 2 % at 20 dB of
/// reduction with `tg_drive` at maximum, which is the spoof's stand-in for
/// Chandler's THD mode.
///
/// Note what it does **not** disturb: the element's gain is a ratio of
/// resistances and is independent of level, so every calibration test in
/// section 12 holds whatever this is set to. Only the distortion moves.
pub const ELEMENT_DRIVE_V: f32 = 0.32;

/// The extra drive `tg_drive` puts on the element at maximum, in dB.
pub const DRIVE_MAX_DB: f32 = 12.0;

/// C1 4µ7 into R78 7K5: the input coupling corner, in hertz.
///
/// **Derived from the drawing.** 1/(2π·7500·4.7 µF) = 4.52 Hz, and test 6
/// asserts it. It is also the negative test for transformers: a
/// transformer-coupled model cannot be this flat or this
/// level-independent at the bottom, and the TG has no transformers
/// anywhere (3.3).
pub const F_IN_COUPLING: f32 = 4.516;

/// C23 470 µF into a 600 Ω load: the output coupling corner, in hertz.
///
/// **Derived from the drawing.** 0.56 Hz, so the output capacitor is
/// transparent and is there only to block the amplifier's standing DC.
pub const F_OUT_COUPLING: f32 = 0.564;

/// The DC blockers either side of the element, in hertz.
///
/// Hygiene rather than a component: 11.8 calls a DC block at the element
/// non-negotiable, because an offset biases the two arms asymmetrically
/// and manufactures the even harmonics the topology otherwise cancels.
/// Placed well below both coupling corners so that test 6 measures the
/// capacitors and not these.
pub const F_DC_BLOCK: f32 = 0.5;

/// `ln(2·cosh(x))`, computed so that it does not overflow.
///
/// Past twelve the correction term is below f32's epsilon, so the branch
/// is exact as well as cheap; the static solve leans on that, because most
/// of a transfer curve sits far above the rectifier's knee.
#[inline]
fn ln_2cosh(x: f32) -> f32 {
    let a = x.abs();
    if a > 12.0 {
        return a;
    }
    a + (1.0 + (-2.0 * a).exp()).ln()
}

/// A germanium full-wave rectifier with a soft knee of width `v`.
///
/// `softrect(a, V) = V·ln(2·cosh(a/V)) − V·ln2`, which is smooth, is
/// quadratic near zero and settles to `|a| − V·ln2` far from it.
/// Germanium rectification has a soft, low-threshold knee; a hard `abs()`
/// is a silicon rectifier and this unit does not have one (5.2).
#[inline]
pub fn softrect(a: f32, v: f32) -> f32 {
    v * (ln_2cosh(a / v) - std::f32::consts::LN_2)
}

/// The law network: two straight segments meeting at [`V_LAW`].
#[inline]
pub fn law(e0: f32) -> f32 {
    if e0 <= V_LAW {
        LAW_A * e0
    } else {
        LAW_A * V_LAW + LAW_B * (e0 - V_LAW)
    }
}

/// What the detector offers the store for one sample at the tap.
///
/// Rectify with the mode wafer's two polarities driven independently,
/// subtract the germanium reference, then the law network. The running
/// engine and the static solve both go through here, so the transfer curve
/// the page draws cannot drift away from the audio.
#[inline]
pub fn detector_e(x: f32, gains: (f32, f32)) -> f32 {
    let g = if x >= 0.0 { gains.0 } else { gains.1 };
    if g <= 0.0 {
        return 0.0;
    }
    law((softrect(V_SC_DRIVE * g * x, V_GE) - V_REF).max(0.0))
}

/// Points sampled across the conducting part of each half cycle by the
/// static solve.
const CYCLE_POINTS: usize = 12;

/// The store's steady state for a sine of peak `q` at the tap, with an
/// attack-to-release time-constant ratio of `ratio`.
///
/// **Not the peak of the rectified waveform, and the difference is this
/// unit's soft onset.** The store charges through R41 and D11 with one
/// time constant and discharges through the recovery ladder with another,
/// so in steady state it sits where the two areas balance:
///
/// ```text
/// mean( (e − s)+ ) = (τ_attack / τ_release) · mean( (s − e)+ )
/// ```
///
/// Just above the threshold the rectified signal crosses the germanium
/// reference for only a sliver of each cycle, so the store barely charges
/// however tall that sliver is, and gain reduction comes up gradually. A
/// peak-following model has no such region and gives this unit a knee it
/// does not have.
///
/// The sampling is over the conducting arc only, from where the rectified
/// sine first crosses the reference to the crest, because that is where
/// all the structure is; the silent remainder of the cycle enters as a
/// single weight. Each half cycle is taken separately, since in LIMIT the
/// mode wafer drives the two polarities six to one.
pub fn static_store(q: f32, gains: (f32, f32), ratio: f32) -> f32 {
    let mut e = [0.0f32; 2 * CYCLE_POINTS];
    let mut w = [0.0f32; 2 * CYCLE_POINTS];
    // The whole cycle's worth of weight that sits below the reference.
    let mut silent = 0.0f32;
    let mut hi0 = 0.0f32;
    for (half, &(sign, gain)) in [(1.0f32, gains.0), (-1.0f32, gains.1)].iter().enumerate() {
        // Where this half cycle first reaches the reference.
        let drive = V_SC_DRIVE * gain * q;
        let u = if drive > 0.0 {
            ((V_REF + V_GE * std::f32::consts::LN_2) / drive).min(1.0)
        } else {
            1.0
        };
        let th0 = u.clamp(-1.0, 1.0).asin();
        let arc = std::f32::consts::FRAC_PI_2 - th0;
        // Each half cycle is half the period, so its weights sum to 0.5.
        let below = 0.5 * th0 / std::f32::consts::FRAC_PI_2;
        silent += below;
        let each = (0.5 - below) / CYCLE_POINTS as f32;
        for k in 0..CYCLE_POINTS {
            let th = th0 + arc * (k as f32 + 0.5) / CYCLE_POINTS as f32;
            let v = detector_e(sign * q * th.sin(), gains);
            e[half * CYCLE_POINTS + k] = v;
            w[half * CYCLE_POINTS + k] = each;
            hi0 = hi0.max(v);
        }
    }
    if hi0 <= 0.0 {
        return 0.0;
    }
    let (mut lo, mut hi) = (0.0f32, hi0);
    for _ in 0..40 {
        let s = 0.5 * (lo + hi);
        let mut up = 0.0f32;
        let mut dn = silent * s;
        for k in 0..2 * CYCLE_POINTS {
            if e[k] > s {
                up += w[k] * (e[k] - s);
            } else {
                dn += w[k] * (s - e[k]);
            }
        }
        if up > ratio * dn {
            lo = s;
        } else {
            hi = s;
        }
    }
    0.5 * (lo + hi)
}

/// The recovery position the calibration of `K_I` is taken at.
///
/// The store's steady state depends on how fast it is allowed to fall
/// between peaks, so the depth the model reaches depends a little on the
/// recovery switch — which is real, and is part of why a fast recovery
/// sounds less compressed. The fit is taken at the default position so
/// that moving the switch moves the depth rather than the reverse.
pub const CAL_RECOVERY: usize = 2;

/// The attack-to-release ratio the static solve uses for a position.
pub fn tau_ratio(recovery: usize, hold: f32) -> f32 {
    TAU_ATTACK / recovery_s(recovery, hold)
}

// ---------------------------------------------------------------- settings

/// Everything the engine reads from the parameters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// [`MODE_COMPRESS`], [`MODE_OUT`] or [`MODE_LIMIT`].
    pub mode: usize,
    /// Recovery switch position, 0 to 5 for the panel's 1 to 6.
    pub recovery: usize,
    /// Output level switch position, 0 to 20 for the panel's −10 to +10.
    pub output: usize,
    /// RV1's travel, 0 to 1.
    pub hold: f32,
    /// Which operating region the element is in. A switch because the
    /// drawing is genuinely ambiguous (4.3) and the answer changes the
    /// sound; making it a control means the model can be corrected by
    /// moving a default rather than by rewriting the element.
    pub region: usize,
    /// Arm imbalance, 0 to 1 of the dossier's 5 % range.
    pub mismatch: f32,
    /// Input trim in dB. Not on EMI's module; modelled on Chandler's
    /// continuous input control.
    pub input_db: f32,
    /// Extra drive at the element only, 0 to 1. Not on the hardware.
    pub drive: f32,
    /// Oversampling factor: 1, 2 or 4.
    pub oversample: usize,
    /// The lab's shared stereo link, which on this unit is the GANG bus.
    pub link: bool,
    /// Wet share, 0 to 1. Not on the hardware.
    pub mix: f32,
    /// Side-chain high-pass in Hz, 0 = off. The lab's shared extra; the
    /// hardware has no sidechain filter.
    pub sc_hpf: f32,
    /// A true straight-through. **Not on the hardware**, and not what OUT
    /// is: OUT leaves the element in circuit (6.2).
    pub bypass: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            mode: MODE_COMPRESS,
            recovery: 2,
            output: OUTPUT_UNITY,
            hold: 0.0,
            region: 0,
            mismatch: 0.0,
            input_db: 0.0,
            drive: 0.0,
            oversample: 2,
            link: false,
            mix: 1.0,
            sc_hpf: 0.0,
            bypass: false,
        }
    }
}

/// What the detector needs that does not change per sample.
#[derive(Clone, Copy, Debug)]
pub struct DetectorConfig {
    /// The mode wafer's two half-cycle gains.
    pub gains: (f32, f32),
    /// One-pole coefficient for the charge path.
    pub k_attack: f32,
    /// One-pole coefficient for the recovery ladder.
    pub k_release: f32,
}

/// One channel's detector: the germanium rectifier, the threshold
/// reference, the law network and the store.
///
/// One detector and a mode switch, where the Neve has two detectors
/// fighting over one bridge. That is why the TG's gain reduction has one
/// slope in it and the Neve's has two (9.2, and test 20).
#[derive(Clone, Copy, Debug, Default)]
pub struct Detector {
    /// The store's voltage, C18.
    store: f32,
}

impl Detector {
    fn reset(&mut self) {
        self.store = 0.0;
    }

    /// The store's voltage.
    pub fn store(&self) -> f32 {
        self.store
    }

    /// One oversampled sample. `x` is the signal at the tap.
    #[inline]
    fn step(&mut self, x: f32, cfg: &DetectorConfig) -> f32 {
        // Rectify, with the mode wafer's two polarities driven
        // independently. In COMPRESS the two gains are equal and this is
        // an ordinary soft full-wave rectifier.
        let e = detector_e(x, cfg.gains);
        // D11 is in the charge path and D12 in the discharge path and
        // neither conducts backwards, so the store is one-way (5.5).
        let k = if e > self.store {
            cfg.k_attack
        } else {
            cfg.k_release
        };
        self.store = flush(self.store + (e - self.store) * k);
        self.store
    }
}

// ---------------------------------------------------------------- channel

#[derive(Clone)]
struct Channel {
    det: Detector,
    chain: Chain,
    dry: Delay,
    /// The tap, one oversampled sample old: the feedback path.
    z_tap: f32,
    /// The control current the element carried last sample, in amps.
    z_ctrl: f32,
    in_coupling: OnePole,
    out_coupling: OnePole,
    sc_hpf: OnePole,
    dc_in: OnePole,
    dc_out: OnePole,
}

impl Channel {
    fn new(sr: f32) -> Self {
        let mut c = Channel {
            det: Detector::default(),
            chain: Chain::new(),
            dry: Delay::new(),
            z_tap: 0.0,
            z_ctrl: 0.0,
            in_coupling: OnePole::default(),
            out_coupling: OnePole::default(),
            sc_hpf: OnePole::default(),
            dc_in: OnePole::default(),
            dc_out: OnePole::default(),
        };
        c.retune(sr, 2, 0.0);
        c
    }

    /// Set every filter for a host rate, an oversampling factor and a
    /// side-chain corner.
    fn retune(&mut self, sr: f32, factor: usize, sc_hpf: f32) {
        let os = sr * factor as f32;
        self.in_coupling.set(F_IN_COUPLING, sr);
        self.out_coupling.set(F_OUT_COUPLING, sr);
        self.dc_in.set(F_DC_BLOCK, os);
        self.dc_out.set(F_DC_BLOCK, os);
        self.sc_hpf.set(sc_hpf.max(1.0), os);
        self.chain.set_factor(factor);
        self.dry.set_len(latency(factor));
    }

    fn reset(&mut self) {
        self.det.reset();
        self.chain.reset();
        self.dry.reset();
        self.z_tap = 0.0;
        self.z_ctrl = 0.0;
        self.in_coupling.reset();
        self.out_coupling.reset();
        self.sc_hpf.reset();
        self.dc_in.reset();
        self.dc_out.reset();
    }
}

// ------------------------------------------------------------- the module

/// The EMI TG12413 limiter/compressor.
pub struct Compressor {
    sr: f32,
    settings: Settings,
    element: Element,
    k_i: f32,
    ch: [Channel; 2],
    gr_db: [f32; 2],
    in_peak: [f32; 2],
    out_peak: [f32; 2],
    ctrl_a: [f32; 2],
    frames: usize,
}

impl Compressor {
    /// A module at `sr` hertz with default settings.
    pub fn new(sr: f32) -> Self {
        let s = Settings::default();
        let element = element_for(&s);
        let mut c = Compressor {
            sr,
            settings: s,
            element,
            k_i: fit_k_i(&element),
            ch: [Channel::new(sr), Channel::new(sr)],
            gr_db: [0.0; 2],
            in_peak: [0.0; 2],
            out_peak: [0.0; 2],
            ctrl_a: [0.0; 2],
            frames: 0,
        };
        c.retune();
        c
    }

    fn retune(&mut self) {
        let (sr, f, hpf) = (self.sr, self.settings.oversample, self.settings.sc_hpf);
        for c in &mut self.ch {
            c.retune(sr, f, hpf);
        }
    }

    /// Change the sample rate, rebuilding everything that depends on it.
    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sr = sr;
        self.retune();
        self.reset();
    }

    /// Silence the state.
    pub fn reset(&mut self) {
        for c in &mut self.ch {
            c.reset();
        }
        self.gr_db = [0.0; 2];
        self.in_peak = [0.0; 2];
        self.out_peak = [0.0; 2];
        self.ctrl_a = [0.0; 2];
        self.frames = 0;
    }

    /// The settings in force.
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// The gain element as configured.
    pub fn element(&self) -> &Element {
        &self.element
    }

    /// Amps of control current per volt of store, as fitted.
    pub fn k_i(&self) -> f32 {
        self.k_i
    }

    /// Latency in samples at the host rate.
    pub fn latency(&self) -> usize {
        latency(self.settings.oversample)
    }

    /// Apply new settings. Returns whether anything changed.
    pub fn configure(&mut self, s: Settings) -> bool {
        let changed = s != self.settings;
        if !changed {
            return false;
        }
        let retune = s.oversample != self.settings.oversample || s.sc_hpf != self.settings.sc_hpf;
        self.settings = s;
        self.element = element_for(&s);
        self.k_i = fit_k_i(&self.element);
        if retune {
            self.retune();
        }
        true
    }

    /// Gain reduction in dB (positive) on `channel`, from the last block.
    pub fn gain_reduction_db(&self, channel: usize) -> f32 {
        self.gr_db[channel.min(1)]
    }

    /// The mean control current over the last block, in amps.
    ///
    /// The quantity the whole circuit is about, and the one 11.7 asks the
    /// page to show live because it is what makes the two modes legible
    /// side by side.
    pub fn control_a(&self, channel: usize) -> f32 {
        self.ctrl_a[channel.min(1)]
    }

    /// The detector's configuration for the settings in force.
    fn detector_config(&self, dt_os: f32) -> DetectorConfig {
        let s = &self.settings;
        let rel = recovery_s(s.recovery, s.hold);
        DetectorConfig {
            gains: mode_gains(s.mode),
            k_attack: 1.0 - (-dt_os / TAU_ATTACK).exp(),
            k_release: 1.0 - (-dt_os / rel).exp(),
        }
    }

    /// Process one block in place.
    pub fn process_block(&mut self, l: &mut [f32], r: &mut [f32]) {
        let n = l.len().min(r.len());
        let s = self.settings;
        let dt_os = 1.0 / (self.sr * s.oversample as f32);
        let cfg = self.detector_config(dt_os);
        let factor = self.ch[0].chain.factor();
        let trim = 10f32.powf(s.input_db / 20.0);
        let scale = ELEMENT_DRIVE_V * 10f32.powf(DRIVE_MAX_DB * s.drive / 20.0);
        // Two Newton steps once the element is being pushed towards its
        // asymptote on purpose, one otherwise (11.2).
        let steps = if s.drive > 0.5 { 2 } else { 1 };
        let out_gain = output_gain(s.output);
        let use_hpf = s.sc_hpf > 1.0;
        let element = self.element;
        let k_i = self.k_i;
        let processing = !s.bypass;

        let mut gr_sum = [0.0f32; 2];
        let mut ctrl_sum = [0.0f32; 2];
        self.in_peak = [0.0; 2];
        self.out_peak = [0.0; 2];

        for i in 0..n {
            let x = [l[i], r[i]];
            self.in_peak[0] = self.in_peak[0].max(x[0].abs());
            self.in_peak[1] = self.in_peak[1].max(x[1].abs());

            if !processing {
                for c in 0..2 {
                    self.ch[c].dry.process(x[c]);
                }
                l[i] = x[0];
                r[i] = x[1];
                self.out_peak[0] = self.out_peak[0].max(x[0].abs());
                self.out_peak[1] = self.out_peak[1].max(x[1].abs());
                continue;
            }

            // C1 into R78: the module's own input coupling, at the host
            // rate because it is in front of everything.
            let mut ups = [[0.0f32; 4]; 2];
            for c in 0..2 {
                let v = self.ch[c].in_coupling.hp(x[c] * trim);
                ups[c] = self.ch[c].chain.up(v);
            }

            let mut outs = [[0.0f32; 4]; 2];
            for k in 0..factor {
                // The detector reads the tap one oversampled sample ago,
                // which is what keeps the feedback loop causal.
                let mut ctrl = [0.0f32; 2];
                for c in 0..2 {
                    let tap = self.ch[c].z_tap;
                    let sc_in = if use_hpf {
                        self.ch[c].sc_hpf.hp(tap)
                    } else {
                        tap
                    };
                    ctrl[c] = I_MIN + k_i * self.ch[c].det.step(sc_in, &cfg);
                }
                // The GANG bus is a shared current rail, so whichever
                // module is working hardest holds it: a maximum over
                // currents, not over decibels (5.7).
                if s.link {
                    let m = ctrl[0].max(ctrl[1]);
                    ctrl = [m, m];
                }
                for c in 0..2 {
                    let v_s = self.ch[c].dc_in.hp(ups[c][k]) * scale;
                    let u = element.solve(v_s, ctrl[c], steps);
                    let y = self.ch[c].dc_out.hp(u / scale);
                    outs[c][k] = y;
                    self.ch[c].z_tap = y;
                    self.ch[c].z_ctrl = ctrl[c];
                    gr_sum[c] += element.gr_db(ctrl[c]);
                    ctrl_sum[c] += ctrl[c];
                }
            }

            for c in 0..2 {
                let wet = self.ch[c].chain.down(&outs[c]);
                // S3 is after the tap, so the ladder cannot move the gain
                // reduction; then C23 into the load.
                let wet = self.ch[c].out_coupling.hp(wet * out_gain);
                let dry = self.ch[c].dry.process(x[c]);
                let y = dry + (wet - dry) * s.mix;
                if c == 0 {
                    l[i] = y;
                } else {
                    r[i] = y;
                }
                self.out_peak[c] = self.out_peak[c].max(y.abs());
            }
        }

        if n > 0 {
            let inv = 1.0 / (n * factor.max(1)) as f32;
            self.gr_db = [gr_sum[0] * inv, gr_sum[1] * inv];
            self.ctrl_a = [ctrl_sum[0] * inv, ctrl_sum[1] * inv];
            self.frames = n;
        }
    }

    /// `[in_l, in_r, out_l, out_r, gr_db, meter_target]` for the last
    /// block, with `gr_db` **positive** for reduction, which is the lab's
    /// frame convention; the lab negates it on the way out.
    ///
    /// Slot 5 is the level the needle is **chasing**, not where it is. The
    /// lab owns the movement and runs it once, in the audio thread; a `Vu`
    /// here as well would put two cascaded movements in series. See
    /// `dsp::tests::every_needle_runs_one_ballistic`.
    pub fn meter_frame(&self) -> [f32; 6] {
        [
            self.in_peak[0],
            self.in_peak[1],
            self.out_peak[0],
            self.out_peak[1],
            0.5 * (self.gr_db[0] + self.gr_db[1]),
            -0.5 * (self.gr_db[0] + self.gr_db[1]),
        ]
    }

    /// `[control current in µA, element resistance in Ω, drive fraction]`.
    ///
    /// The lab's `cell` stream carries the T4's three states for the
    /// optical models; this machine has no cell, so it publishes the three
    /// quantities the element actually has, the way the CL-1B does.
    pub fn cell_state(&self) -> [f32; 3] {
        let i = 0.5 * (self.ctrl_a[0] + self.ctrl_a[1]);
        let r = self.element.resistance(i);
        [
            i * 1e6,
            if r.is_finite() { r } else { 1e9 },
            self.settings.drive,
        ]
    }

    /// Steady-state gain reduction in dB (positive) for a sine of peak
    /// `amp_peak`, before the output ladder.
    ///
    /// A bisection rather than a run of the loop, because the recovery can
    /// be two seconds long. The loop is a feedback one, so the detector
    /// sees the reduced signal and the answer is the store voltage at
    /// which the two agree.
    pub fn static_gr_db(&self, amp_peak: f32) -> f32 {
        if self.settings.bypass {
            return 0.0;
        }
        let gains = mode_gains(self.settings.mode);
        if gains.0.max(gains.1) <= 0.0 {
            return 0.0;
        }
        let trim = 10f32.powf(self.settings.input_db / 20.0);
        let p = (amp_peak * trim).max(0.0);
        let ratio = tau_ratio(self.settings.recovery, self.settings.hold);
        let demand = |store: f32| {
            let a = self.element.gain(I_MIN + self.k_i * store);
            static_store(p * a, gains, ratio) - store
        };
        if demand(0.0) <= 0.0 {
            return 0.0;
        }
        let (mut lo, mut hi) = (0.0f32, 1.0f32);
        while demand(hi) > 0.0 && hi < 1e6 {
            hi *= 4.0;
        }
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if demand(mid) > 0.0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        self.element.gr_db(I_MIN + self.k_i * (0.5 * (lo + hi)))
    }

    /// The static transfer curve, output dBFS for `min_dbfs..max_dbfs` in.
    pub fn transfer_curve(&self, out: &mut [f32], min_dbfs: f32, max_dbfs: f32) {
        let n = out.len();
        if n == 0 {
            return;
        }
        let make_up = output_db(self.settings.output) + self.settings.input_db;
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

/// The element the settings ask for.
pub fn element_for(s: &Settings) -> Element {
    let mut e = if s.region == REGION_FORWARD {
        Element::forward(super::element::N_JUNCTIONS)
    } else {
        Element::breakdown()
    };
    // The dossier's range for the imbalance is 0 to 5 % (11.3).
    e.mismatch = 0.05 * s.mismatch.clamp(0.0, 1.0);
    e.r_series = R_SERIES;
    e
}

/// Fit the control-current constant to the calibration target.
///
/// **This is the dossier's own instruction and not a figure about the
/// hardware**: 11.6 says to set `I_MIN` and `K_I` so that a full sidechain
/// gives about 20 dB, because no published gain-reduction range exists for
/// this unit. Fitting it rather than writing a number down means the
/// region switch and the mismatch control do not quietly change how deep
/// the unit goes, only how it sounds getting there.
pub fn fit_k_i(element: &Element) -> f32 {
    let gains = mode_gains(MODE_COMPRESS);
    let a = 10f32.powf(-CAL_GR_DB / 20.0);
    let store = static_store(CAL_INPUT_AMP * a, gains, tau_ratio(CAL_RECOVERY, 0.0));
    match element.current_for_gr_db(CAL_GR_DB) {
        Some(i) if store > 1e-9 => (i - I_MIN).max(0.0) / store,
        _ => 0.0,
    }
}

/// The three mode labels, in EMI's printed order, for a test that wants to
/// assert the order rather than the labels' spelling.
pub const MODE_ORDER: [usize; 3] = [MODE_COMPRESS, MODE_OUT, MODE_LIMIT];

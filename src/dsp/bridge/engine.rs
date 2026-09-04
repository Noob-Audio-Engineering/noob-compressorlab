//! The Neve diode-bridge engine.
//!
//! `research/Neve-33609.md` is what this is built from; section 11 is the
//! design and section 12 the test plan. The structure below is the
//! circuit's, and where the drawings leave a constant under-determined the
//! constant is fitted to a published figure and labelled as fitted.

use noob_electrical_components::diode_bridge as bridge;

use crate::dsp::fet::oversample::{Downsampler, DryDelay, LATENCY, Upsampler};
use crate::dsp::flush;
use crate::dsp::opto::filters::OnePole;
use crate::dsp::vu::Vu;

use super::{LIMIT_ATTACK_FAST, MODEL_2254E, MODEL_33609N, RATIO_TRUE, RECOVERY_AUTO1};

/// 0 VU is +4 dBu and −18 dBFS RMS, the calibration the whole lab uses.
pub const VU_REFERENCE_DBFS: f32 = -18.0;

/// Peak amplitude of a sine at 0 VU, i.e. at +4 dBu.
pub const VU_REF_AMP: f32 = 0.125_892_54 * std::f32::consts::SQRT_2;

/// Mean of `|sin|`, for turning the 2254/E meter's rectified average back
/// into the peak its VU scale is marked against.
const SINE_MEAN_ABS: f32 = std::f32::consts::FRAC_2_PI;

/// Peak amplitude of a sine at `x` dBu.
pub fn dbu_amp(x: f32) -> f32 {
    VU_REF_AMP * 10f32.powf((x - 4.0) / 20.0)
}

/// A level in dBu from a peak amplitude.
pub fn amp_dbu(a: f32) -> f32 {
    20.0 * (a.max(1e-9) / VU_REF_AMP).log10() + 4.0
}

// ---------------------------------------------------------------- network

/// The resistances around the bridge, and the gain structure either side.
///
/// Every value is from the 33609/J handbook's EX11475 except where noted.
/// The gain structure is the block diagram's own annotated chain, which
/// closes to unity: 0 dBu in, −6 at the T2 secondary, −31 at the bridge,
/// −25 after T1, −30 at the RV1 wiper, −8 after the 10640 and 0 after T3.
#[derive(Clone, Copy, Debug)]
pub struct Network {
    /// R48 + R49, the series arm.
    pub r_series: f32,
    /// R11, the linear shunt across the bridge.
    pub r_shunt: f32,
    /// The bridge's thermal scale, `2ηV_T`.
    pub k: f32,
}

impl Default for Network {
    fn default() -> Self {
        Network {
            r_series: 94_000.0,
            r_shunt: 5_600.0,
            k: bridge::THERMAL_SCALE,
        }
    }
}

impl Network {
    /// The divider's linear gain for a bridge small-signal resistance `r`.
    ///
    /// With the bridge open this is `R11 / (R48 + R49 + R11)`, which works
    /// out to −25.01 dB against Neve's own −6 dB to −31 dB annotation on
    /// the same drawing. That agreement between three resistor values and
    /// a level mark is the best validation in the document set, and it is
    /// what test `the_open_bridge_loses_25_db` asserts.
    pub fn gain_for_resistance(&self, r: f32) -> f32 {
        let r_p = if r.is_finite() {
            self.r_shunt * r / (self.r_shunt + r)
        } else {
            self.r_shunt
        };
        r_p / (self.r_series + r_p)
    }

    /// The divider's gain with the bridge open, i.e. no control current.
    pub fn open_gain(&self) -> f32 {
        self.gain_for_resistance(f32::INFINITY)
    }

    /// Gain reduction in dB (positive) for a control current in amps.
    pub fn gr_db(&self, control: f32) -> f32 {
        let r = bridge::small_signal_resistance(control, self.k);
        20.0 * (self.open_gain() / self.gain_for_resistance(r)).log10()
    }

    /// The control current that gives `gr` dB of reduction.
    ///
    /// Closed form, because the bridge's `r = k / I` is. One of the
    /// practical advantages of a bridge over a single shunt diode.
    pub fn control_for_gr_db(&self, gr: f32) -> f32 {
        if gr <= 0.0 {
            return 0.0;
        }
        let want = self.open_gain() / 10f32.powf(gr / 20.0);
        // Invert the divider for R_p, then the parallel pair for r.
        let r_p = want * self.r_series / (1.0 - want).max(1e-9);
        let inv = (1.0 / r_p - 1.0 / self.r_shunt).max(1e-12);
        bridge::control_for_resistance(1.0 / inv, self.k)
    }

    /// Solve the bridge's node equation for the differential voltage.
    ///
    /// The node equation `(v_s − u)/R_s = u/R_sh + I·tanh(u/k)` is
    /// implicit in `u`, so it takes a linear seed and one Newton step. The
    /// seed uses the bridge's small-signal resistance, which is already
    /// exact to first order, and `tanh` departs from identity by only a
    /// few per cent over the working range, so one step is enough. This is
    /// the caller's job rather than the component's, because the two
    /// resistors are the machine and only the bridge is the part.
    #[inline]
    pub fn solve_node(&self, v_s: f32, control: f32) -> f32 {
        let r = bridge::small_signal_resistance(control, self.k);
        let r_p = if r.is_finite() {
            self.r_shunt * r / (self.r_shunt + r)
        } else {
            self.r_shunt
        };
        let mut u = v_s * r_p / (self.r_series + r_p);
        if control > bridge::CONTROL_FLOOR {
            let f =
                u / self.r_shunt + bridge::current(u, control, self.k) - (v_s - u) / self.r_series;
            let fp = 1.0 / self.r_shunt + bridge::slope(u, control, self.k) + 1.0 / self.r_series;
            u -= f / fp;
        }
        u
    }
}

// ------------------------------------------------------------ calibration

/// Volts at the T2 secondary for a sine of unit peak amplitude at the
/// plug-in input.
///
/// **Fitted, and the one number in this engine I could not derive.** The
/// block diagram says the T2 secondary sits at −6 dBu for 0 dBu in, which
/// puts about 30 mV peak across the bridge and a `tanh` argument near
/// 0.34. At that argument the bridge's own third harmonic is about 0.96 %,
/// which is more than ten times the 0.075 % the handbook publishes for the
/// through path. The dossier's section 4.5 flags that as an open
/// discrepancy and concludes the real bridge must see substantially less
/// signal than the annotation implies, either because R47 loads the
/// transformer harder than the nominal level suggests or because the
/// annotations are nominal rather than measured.
///
/// So this is calibrated against the published distortion rather than
/// against the level annotation, and the gap between the two is recorded
/// rather than split. Note that it does not disturb the divider: the
/// bridge's *gain* is a ratio of resistances and is independent of level,
/// so the 25 dB open-bridge test holds whatever this is set to. Only the
/// distortion moves.
pub const BRIDGE_DRIVE_V: f32 = 0.003_1;

/// Control voltage at which gain reduction begins, in volts.
///
/// The compressor's law string starts with a 2.7 V zener (D12) and the
/// limiter's threshold reference is another (D13), so the control node
/// sits at a diode reference until the detector pushes it past. Taking
/// that as the foot of the control law is a physical reading rather than
/// a fit.
pub const V_CONTROL_FOOT: f32 = 2.7;

/// The two published control voltages, from the 2254/E level diagram
/// EB/20134: 3.5 V at 12 dB of gain reduction and 4.0 V at about 16.8 dB.
///
/// This is the only published statement anywhere of what this family's
/// sidechains produce, and it is what the control law below is fitted to.
pub const V_CONTROL_POINTS: [(f32, f32); 2] = [(3.5, 12.0), (4.0, 16.8)];

/// Softness of the control law's corner, in volts.
///
/// Four series diodes give about `4·η·V_T` of their own softness, which is
/// what rounds the corner where D10 begins to conduct across R29.
const V_LAW_SOFT: f32 = 0.18;

/// The law-correction network, as a two-segment map from control voltage
/// to bridge current.
///
/// The divider R36, RV2, R45, R29 with D10 across R29 is gentle until D10
/// conducts and then steeper, which is the "law correction characteristic
/// necessary to properly drive the gain control diode bridge" the handbook
/// names. The break point and the two slopes are **fitted to the two
/// published control voltages** rather than derived, because the divider's
/// preset RV2 is a factory trim whose position no drawing states.
#[derive(Clone, Copy, Debug)]
pub struct ControlLaw {
    foot: f32,
    brk: f32,
    slope_low: f32,
    slope_high: f32,
}

impl ControlLaw {
    /// Fit the law to the level diagram's two points through `net`.
    ///
    /// The two slopes are solved against the **rounded** hinges rather
    /// than against straight lines, so the law passes through both
    /// published points exactly instead of near them.
    pub fn fit(net: &Network) -> Self {
        let (v1, g1) = V_CONTROL_POINTS[0];
        let (v2, g2) = V_CONTROL_POINTS[1];
        let i1 = net.control_for_gr_db(g1);
        let i2 = net.control_for_gr_db(g2);
        // The first hinge, evaluated where each point sits on it.
        let a1 = soft_hinge(v1 - V_CONTROL_FOOT, V_LAW_SOFT);
        let a2 = soft_hinge(v2 - V_CONTROL_FOOT, V_LAW_SOFT);
        // The second hinge opens at the first point, so it contributes
        // nothing there and only shapes the run up to the second.
        let b2 = soft_hinge(v2 - v1, V_LAW_SOFT);
        let slope_low = i1 / a1.max(1e-6);
        ControlLaw {
            foot: V_CONTROL_FOOT,
            brk: v1,
            slope_low,
            slope_high: slope_low + (i2 - slope_low * a2) / b2.max(1e-6),
        }
    }

    /// Bridge control current in amps for a control voltage.
    #[inline]
    pub fn current(&self, v: f32) -> f32 {
        // A rounded hinge at the foot and another at the break, so the
        // corners have the diode string's own softness rather than being
        // the hard kinks a piecewise-linear map would give. Below the
        // foot the answer is exactly zero: the control node rests on the
        // 2.7 V zener and no current reaches the bridge at all.
        let a = soft_hinge(v - self.foot, V_LAW_SOFT);
        let b = soft_hinge(v - self.brk, V_LAW_SOFT);
        (self.slope_low * a + (self.slope_high - self.slope_low) * b).max(0.0)
    }

    /// The control voltage that produces a wanted bridge current: the
    /// exact inverse of [`ControlLaw::current`].
    ///
    /// This has to be exact rather than close. The sidechain works out how
    /// much reduction it wants and then charges toward the voltage that
    /// would deliver it, so if the two maps disagree the loop settles
    /// somewhere other than the ratio it was asked for. A straight-line
    /// inverse was the first version, and it cost about 30 % of the ratio
    /// at the 6:1 position: the handbook's 1.5 dB came out at 2.1 dB.
    ///
    /// Both hinges are quadratic then linear, so this is closed form: a
    /// square root in each rounded region and a division in each straight
    /// one.
    pub fn voltage_for_current(&self, i: f32) -> f32 {
        let s = V_LAW_SOFT;
        let d = self.slope_high - self.slope_low;
        if i <= 0.0 || self.slope_low <= 0.0 {
            return self.foot;
        }
        // The corners, in current.
        let i_a = self.slope_low * s; // end of the first rounded run
        let i_b = self.slope_low * (self.brk - self.foot - s); // the break
        let i_c = i_b + self.slope_low * s + d * s; // end of the second
        if i <= i_a {
            self.foot + 2.0 * (s * i / self.slope_low).sqrt()
        } else if i <= i_b {
            self.foot + s + i / self.slope_low
        } else if i <= i_c && d.abs() > 1e-12 {
            // (d/4s)u² + slope_low·u + (i_b − i) = 0, with u = v − brk.
            let a2 = d / (4.0 * s);
            let c2 = i_b - i;
            let disc = (self.slope_low * self.slope_low - 4.0 * a2 * c2).max(0.0);
            self.brk + (-self.slope_low + disc.sqrt()) / (2.0 * a2)
        } else if i <= i_c {
            self.foot + s + i / self.slope_low
        } else {
            (i + self.slope_low * (self.foot + s) + d * (self.brk + s)) / self.slope_high
        }
    }
}

/// A smooth hinge: exactly zero at and below zero, `x − s` well above,
/// and a quadratic joining the two over `0..2s`.
///
/// The obvious hinge is a softplus, and this was one until test 2 caught
/// what is wrong with it. `s·ln(1 + e^{x/s})` is never zero: at the corner
/// it is still `s·ln 2`, so a law built on it leaks current into the bridge
/// with both sections switched out and the handbook's chain no longer
/// closes to 0.0 dB. The same leak put gain reduction below the compressor
/// threshold and turned the limiter's 0.5 dB knee into 17 dB of reduction
/// at threshold, because a brickwall multiplies whatever leaks by 99.
///
/// This form is the standard soft knee instead. It is continuous in its
/// first derivative, which is all the rounding a corner needs, and it is
/// *identically* zero on the closed side, which is what a switched-out
/// section and a signal below threshold both require.
#[inline]
fn soft_hinge(x: f32, s: f32) -> f32 {
    if s <= 0.0 {
        x.max(0.0)
    } else if x <= 0.0 {
        0.0
    } else if x >= 2.0 * s {
        x - s
    } else {
        x * x / (4.0 * s)
    }
}

// ------------------------------------------------------------- sidechains

/// One detector: its law, its storage capacitor and its recovery.
///
/// Both sidechains are this shape. They differ in where they tap, in their
/// law (the compressor has a ratio ladder, the limiter is a brickwall) and
/// in their constants.
#[derive(Clone, Copy, Debug)]
pub struct Sidechain {
    /// Storage voltage, the control contribution this sidechain offers.
    v: f32,
    /// The auto-release platform's own state.
    platform: f32,
    /// Rectified level, smoothed only by the rectifier's own reservoir.
    det: f32,
}

impl Default for Sidechain {
    fn default() -> Self {
        Sidechain {
            v: V_CONTROL_FOOT,
            platform: V_CONTROL_FOOT,
            det: 0.0,
        }
    }
}

/// Where a sidechain is listening and what the bridge is doing, for one
/// sample.
///
/// The two sidechains differ in exactly one field of this — `tap_gain_db`
/// — and that one field is the unit's most distinctive behaviour.
struct Tap<'a> {
    /// The resistances around the bridge.
    net: &'a Network,
    /// The fitted law from control voltage to bridge current.
    law: &'a ControlLaw,
    /// What the bridge was reducing by one sample ago, in dB.
    applied_gr_db: f32,
    /// The gain between the bridge and this tap: zero for the compressor
    /// at the RV1 wiper, the make-up for the limiter at the 10640 output.
    tap_gain_db: f32,
    /// The sample period.
    dt: f32,
}

/// Everything a sidechain needs that does not change per sample.
#[derive(Clone, Copy, Debug)]
pub struct SidechainConfig {
    /// Peak amplitude at which this sidechain's threshold sits.
    pub threshold_amp: f32,
    /// Open-loop law slope, `R − 1` for a closed-loop ratio `R`.
    ///
    /// A feedback detector reads the *compressed* output, so a law of
    /// slope `s` closes to a ratio of `s + 1`. The published 2:1 needs an
    /// open-loop slope of 1 and the brickwall limiter needs 99.
    pub law_slope: f32,
    /// Knee width in dB, over which the law comes up to its full slope.
    pub knee_db: f32,
    /// Attack time constant, seconds.
    pub attack_s: f32,
    /// Fast recovery constant, seconds.
    pub release_s: f32,
    /// Slow platform constant for the auto positions, or `None`.
    pub platform_s: Option<f32>,
    /// Whether the sidechain is switched in at all.
    pub enabled: bool,
    /// First-order high-pass on the detector, Hz, 0 = off.
    pub hpf_hz: f32,
}

/// The platform's charge constant, seconds.
///
/// **Fitted**, to the only behaviour anyone published for it. The 47 µF
/// capacitor and its back-to-back BAX13 pair are on the drawings, but no
/// resistor list I can read fixes the charge path and no figure is given
/// for it. What is published is the behaviour: "recovery is rapid for
/// transient peaks but slower for persistent high levels". A second is
/// what separates those two — an isolated 100 ms burst charges the
/// platform less than a tenth of the way and releases on the fast state
/// alone, while a sustained passage charges it fully and releases on the
/// slow one. Test 24 is what it is fitted against.
const PLATFORM_CHARGE_S: f32 = 1.000;

/// How fast the rectifier's reservoir charges through the conducting
/// diode, in seconds.
///
/// Short, because a forward-biased diode from a low-impedance driver is
/// nearly a short circuit. What sets the section's attack is the storage
/// capacitor behind this, not the reservoir in front of it.
const DETECTOR_CHARGE_S: f32 = 0.000_05;

/// How fast the reservoir bleeds away between peaks, in seconds.
///
/// This is a **peak** rectifier, not an averaging one: the diode conducts
/// on the crests and the reservoir holds between them. That matters twice.
/// It is what the circuit is, a diode into a capacitor with a resistor
/// across it. And it decouples the ripple from the attack, which an
/// averaging detector cannot do — a reservoir long enough to stop the
/// ripple modulating the gain would be longer than the 2 ms attack the
/// handbook publishes, so an averaging detector cannot pass tests 7 and 20
/// at the same time.
///
/// It is deliberately not long enough to be a perfect hold. The dossier is
/// explicit that a *perfectly* smoothed detector would cheat the published
/// distortion figures, because much of the 33609's distortion under
/// compression is sidechain ripple modulating the gain rather than the
/// bridge's own waveshaping.
const DETECTOR_BLEED_S: f32 = 0.010;

impl Sidechain {
    fn reset(&mut self) {
        *self = Sidechain::default();
    }

    /// One sample. `x` is the signal at this sidechain's tap point, `net`
    /// and `law` the shared network, and `dt` the sample period.
    #[inline]
    fn step(&mut self, x: f32, cfg: &SidechainConfig, hpf: &mut OnePole, at: &Tap) -> f32 {
        let (net, law, dt) = (at.net, at.law, at.dt);
        let (applied_gr_db, tap_gain_db) = (at.applied_gr_db, at.tap_gain_db);
        if !cfg.enabled {
            self.v = V_CONTROL_FOOT;
            self.platform = V_CONTROL_FOOT;
            self.det = 0.0;
            return V_CONTROL_FOOT;
        }
        // Full-wave rectify, then the reservoir. The /N's slow position
        // puts a 100 Hz first-order high-pass ahead of it.
        let s = if cfg.hpf_hz > 1.0 { hpf.hp(x) } else { x };
        let a = s.abs();
        let k = if a > self.det {
            1.0 - (-dt / DETECTOR_CHARGE_S).exp()
        } else {
            1.0 - (-dt / DETECTOR_BLEED_S).exp()
        };
        self.det = flush(self.det + (a - self.det) * k);

        // How far this sidechain's tap sits above the threshold, in dB.
        // The threshold control is a sidechain *gain*, not a comparator
        // reference, so this is the same thing expressed the way the
        // circuit does it.
        //
        // Where this sidechain's tap sits, in dB above its threshold.
        //
        // Three things go into it, and the middle one is the whole design
        // of this unit. `tap_gain_db` is the make-up: zero for the
        // compressor, which taps the RV1 wiper *before* the make-up
        // amplifier, and the full make-up for the limiter, which taps the
        // 10640 output *after* it. That one term is why raising the
        // make-up drives the limiter and leaves the compressor's threshold
        // exactly where it was.
        //
        // `deficit` is how much more reduction the bridge is applying than
        // this sidechain asked for, which is only ever the *other*
        // sidechain's doing, because they share one bridge through a
        // shared load. A losing detector therefore goes on reading a
        // genuinely reduced signal, exactly as it does in the hardware,
        // while a winning one reads its own tap undisturbed.
        //
        // The reading is taken ahead of the bridge rather than after it,
        // with the deficit applied arithmetically. That is not a shortcut
        // around the feedback: it is how the loop delay is kept out. The
        // audio path carries an oversampler with 31 samples of group
        // delay, which is a modelling artefact and not a component, and a
        // detector reading through it is reading 31 samples of stale
        // signal. With a brickwall's loop gain that diverges, and it is
        // what collapsed the output to −161 dBu when the make-up first
        // drove the limiter.
        let own_gr = net.gr_db(law.current(self.v));
        let deficit = (applied_gr_db - own_gr).max(0.0);
        let over_db =
            20.0 * (self.det.max(1e-9) / cfg.threshold_amp).log10() + tap_gain_db - deficit;
        // The law: soft over the knee, then straight. The /N manual says
        // the true ratio is only attained more than 5 dB above threshold,
        // which is what the knee width carries.
        let shaped = softplus_db(over_db, cfg.knee_db);
        // `law_slope` is the open-loop slope `R − 1`: a detector reading
        // the compressed output turns a law of slope `s` into a ratio of
        // `s + 1`, which is the algebra a feedback design runs on. Since
        // the reading above has already been referred back past the
        // bridge, the reduction a ratio of `R` needs for `over` dB of
        // excess is `over · (R − 1)/R`, the same equilibrium written the
        // other way round.
        let want_gr = shaped * cfg.law_slope / (cfg.law_slope + 1.0);

        // Turn the wanted reduction into the control voltage that would
        // produce it, which is what the law network is for.
        let target = law.voltage_for_current(net.control_for_gr_db(want_gr));

        if target > self.v {
            // Attack: an emitter follower charging its storage capacitor,
            // so the rate is proportional to the difference rather than
            // being a fixed slope.
            let k = 1.0 - (-dt / cfg.attack_s).exp();
            self.v += (target - self.v) * k;
        } else {
            // Recovery toward the D11 reference, not toward zero.
            let k = 1.0 - (-dt / cfg.release_s).exp();
            self.v += (V_CONTROL_FOOT - self.v) * k;
        }
        self.v = self.v.max(V_CONTROL_FOOT);

        // The auto positions add a second, slower state behind a gate.
        // The auto positions add a second, slower state behind the 47 µF
        // capacitor and its back-to-back BAX13 pair. What makes it ignore
        // isolated transients is the **charge constant**, not a threshold:
        // at 300 ms a 100 ms burst moves it less than a third of the way
        // while a sustained passage charges it fully, which is exactly the
        // "rapid for transient peaks but slower for persistent high
        // levels" the manual describes.
        //
        // An earlier version gated it on the control sitting a diode drop
        // above the platform. That is what the diodes do, but it caps the
        // platform at one drop below the control, and a platform that can
        // only ever hold about 2 dB is not a second time constant at all:
        // the auto positions recovered in 119 ms where the handbook
        // publishes 1500.
        if let Some(plat_s) = cfg.platform_s {
            if target > self.platform {
                let k = 1.0 - (-dt / PLATFORM_CHARGE_S).exp();
                self.platform += (target - self.platform) * k;
            } else {
                let k = 1.0 - (-dt / plat_s).exp();
                self.platform += (V_CONTROL_FOOT - self.platform) * k;
            }
            self.platform = self.platform.max(V_CONTROL_FOOT);
            self.v.max(self.platform)
        } else {
            self.platform = V_CONTROL_FOOT;
            self.v
        }
    }
}

/// A soft hinge in dB: zero below threshold, `x` well above, rounded over
/// `knee` decibels between.
#[inline]
fn softplus_db(x: f32, knee: f32) -> f32 {
    // The knee is centred on the threshold, so it opens `knee/2` below it
    // and reaches its full slope `knee/2` above.
    soft_hinge(x + knee * 0.5, knee * 0.5)
}

// -------------------------------------------------------------- settings

/// Everything the engine reads from the parameters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// [`MODEL_2254E`], [`super::MODEL_33609J`] or [`MODEL_33609N`].
    pub model: usize,
    /// LIMIT IN.
    pub limit_in: bool,
    /// Limit threshold index, +4.0 to +15.0 dBu in 0.5 dB steps.
    pub limit_threshold: usize,
    /// 0 = slow, 1 = fast.
    pub limit_attack: usize,
    /// Limit recovery index: 50, 100, 200, 800 ms, a1, a2.
    pub limit_recovery: usize,
    /// COMPRESS IN.
    pub compress_in: bool,
    /// Compress threshold index, −20 to +10 dBu in 2 dB steps.
    pub compress_threshold: usize,
    /// Compress ratio index: 1.5, 2, 3, 4, 6 as printed.
    pub compress_ratio: usize,
    /// 0 = fast, 1 = slow. /N only; the /J and 2254 have no such control.
    pub compress_attack: usize,
    /// Compress recovery index: 100, 400, 800, 1500 ms, a1, a2.
    pub compress_recovery: usize,
    /// Gain make-up index, 0 to 20 dB in 2 dB steps.
    pub gain: usize,
    /// Meter switch, 2254/E only: 0 = in, 1 = control, 2 = out.
    pub meter_select: usize,
    /// Not on the hardware: extra drive into the bridge, 0..1.
    pub drive: f32,
    /// The lab's shared stereo link.
    pub link: bool,
    /// Wet share, 0..1. Not on the hardware.
    pub mix: f32,
    /// Side-chain high-pass in Hz, 0 = off. The lab's shared extra; the
    /// /N's own 100 Hz filter is separate and comes with its slow attack.
    pub sc_hpf: f32,
    /// The panel's BYPASS position, a true straight-through.
    pub bypass: bool,
    /// The mains switch. Off parks the meters and passes audio through.
    pub power: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            model: super::MODEL_33609J,
            limit_in: false,
            limit_threshold: 8,
            limit_attack: 0,
            limit_recovery: 1,
            compress_in: true,
            compress_threshold: 5,
            compress_ratio: 1,
            compress_attack: 0,
            compress_recovery: 1,
            gain: 0,
            meter_select: 1,
            drive: 0.0,
            link: false,
            mix: 1.0,
            sc_hpf: 0.0,
            bypass: false,
            power: true,
        }
    }
}

/// Limit threshold in dBu for its switch index.
pub fn limit_threshold_dbu(i: usize) -> f32 {
    4.0 + 0.5 * i.min(22) as f32
}

/// Compress threshold in dBu for its switch index.
pub fn compress_threshold_dbu(i: usize) -> f32 {
    -20.0 + 2.0 * i.min(15) as f32
}

/// Gain make-up in dB for its switch index.
pub fn gain_db(i: usize) -> f32 {
    2.0 * i.min(10) as f32
}

/// Limit recovery in seconds, and its platform if the position is auto.
pub fn limit_recovery_s(i: usize) -> (f32, Option<f32>) {
    match i {
        0 => (0.050, None),
        1 => (0.100, None),
        2 => (0.200, None),
        3 => (0.800, None),
        RECOVERY_AUTO1 => (0.100, Some(2.000)),
        _ => (0.050, Some(5.000)),
    }
}

/// Compress recovery in seconds, and its platform if the position is auto.
pub fn compress_recovery_s(i: usize) -> (f32, Option<f32>) {
    match i {
        0 => (0.100, None),
        1 => (0.400, None),
        2 => (0.800, None),
        3 => (1.500, None),
        RECOVERY_AUTO1 => (0.100, Some(2.000)),
        _ => (0.050, Some(5.000)),
    }
}

// -------------------------------------------------------------- the unit

/// One channel's state.
#[derive(Clone)]
struct Channel {
    comp: Sidechain,
    lim: Sidechain,
    comp_hpf: OnePole,
    lim_hpf: OnePole,
    sc_hpf: OnePole,
    /// The feedback taps, one sample old.
    z_pre: f32,
    z_post: f32,
    /// The control voltage the bridge held last sample.
    z_ctrl: f32,
    up: Upsampler,
    down: Downsampler,
    dry: DryDelay,
    /// DC blocking either side of the bridge, which Pines recommends for
    /// any diode gain element: an offset biases the tanh and manufactures
    /// even harmonics the hardware does not have.
    dc_in: OnePole,
    dc_out: OnePole,
}

impl Channel {
    fn new(sr: f32) -> Self {
        let mut ch = Channel {
            comp: Sidechain::default(),
            lim: Sidechain::default(),
            comp_hpf: OnePole::default(),
            lim_hpf: OnePole::default(),
            sc_hpf: OnePole::default(),
            z_pre: 0.0,
            z_post: 0.0,
            z_ctrl: V_CONTROL_FOOT,
            up: Upsampler::new(),
            down: Downsampler::new(),
            dry: DryDelay::new(),
            dc_in: OnePole::default(),
            dc_out: OnePole::default(),
        };
        ch.set_sample_rate(sr);
        ch
    }

    fn set_sample_rate(&mut self, sr: f32) {
        self.comp_hpf.set(100.0, sr);
        self.lim_hpf.set(100.0, sr);
        self.sc_hpf.set(100.0, sr);
        self.dc_in.set(2.0, sr);
        self.dc_out.set(2.0, sr);
    }

    fn reset(&mut self) {
        self.comp.reset();
        self.lim.reset();
        self.z_pre = 0.0;
        self.z_post = 0.0;
        self.z_ctrl = V_CONTROL_FOOT;
        self.up.reset();
        self.down.reset();
        self.dry.reset();
    }
}

/// The Neve diode-bridge limiter/compressor.
pub struct Compressor {
    sr: f32,
    settings: Settings,
    net: Network,
    law: ControlLaw,
    ch: [Channel; 2],
    oversample: bool,
    vu: Vu,
    // Per-block meter accumulators.
    gr_db: [f32; 2],
    in_peak: [f32; 2],
    out_peak: [f32; 2],
    in_abs: [f32; 2],
    out_abs: [f32; 2],
    ctrl_v: [f32; 2],
    comp_gr: [f32; 2],
    lim_gr: [f32; 2],
    frames: usize,
}

impl Compressor {
    /// A unit at `sr` hertz with default settings.
    pub fn new(sr: f32) -> Self {
        let net = Network::default();
        Compressor {
            sr,
            settings: Settings::default(),
            net,
            law: ControlLaw::fit(&net),
            ch: [Channel::new(sr), Channel::new(sr)],
            oversample: sr < 88_200.0,
            vu: Vu::new(sr),
            gr_db: [0.0; 2],
            in_peak: [0.0; 2],
            out_peak: [0.0; 2],
            in_abs: [0.0; 2],
            out_abs: [0.0; 2],
            ctrl_v: [V_CONTROL_FOOT; 2],
            comp_gr: [0.0; 2],
            lim_gr: [0.0; 2],
            frames: 0,
        }
    }

    /// Change the sample rate, rebuilding everything that depends on it.
    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sr = sr;
        self.oversample = sr < 88_200.0;
        for c in &mut self.ch {
            c.set_sample_rate(sr);
        }
        self.vu.set_sample_rate(sr);
        self.reset();
    }

    /// Silence the state. Both storage capacitors go back to the D11
    /// reference rather than to zero, so the first sample after a reset
    /// does not produce a gain jump.
    pub fn reset(&mut self) {
        for c in &mut self.ch {
            c.reset();
        }
        self.vu.reset();
        self.gr_db = [0.0; 2];
        self.in_peak = [0.0; 2];
        self.out_peak = [0.0; 2];
        self.in_abs = [0.0; 2];
        self.out_abs = [0.0; 2];
        self.ctrl_v = [V_CONTROL_FOOT; 2];
        self.comp_gr = [0.0; 2];
        self.lim_gr = [0.0; 2];
        self.frames = 0;
    }

    /// The settings in force.
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// The resistances around the bridge.
    pub fn network(&self) -> &Network {
        &self.net
    }

    /// The fitted control law.
    pub fn control_law(&self) -> &ControlLaw {
        &self.law
    }

    /// Latency in samples at the host rate.
    pub fn latency(&self) -> usize {
        if self.oversample { LATENCY } else { 0 }
    }

    /// Apply new settings. Returns whether anything changed.
    pub fn configure(&mut self, s: Settings) -> bool {
        let changed = s != self.settings;
        if changed {
            self.settings = s;
        }
        changed
    }

    /// Gain reduction in dB (positive) on `channel`, from the last block.
    pub fn gain_reduction_db(&self, channel: usize) -> f32 {
        self.gr_db[channel.min(1)]
    }

    /// Mean control voltage at the bridge's control node over the last
    /// block, in volts.
    ///
    /// This is the node the 2254/E level diagram annotates at 3.5 V and
    /// 4.0 V, and the only quantity inside either sidechain that anyone
    /// ever published a figure for.
    pub fn control_v(&self, channel: usize) -> f32 {
        self.ctrl_v[channel.min(1)]
    }

    /// What the **compressor** sidechain alone was asking for over the
    /// last block, in dB of reduction.
    ///
    /// The two sidechains combine by maximum, so this is not in general
    /// the reduction that happened. It exists so a test can tell which of
    /// the two moved, which is the whole point of a unit that taps them
    /// either side of the make-up amplifier.
    pub fn compress_gr_db(&self, channel: usize) -> f32 {
        self.comp_gr[channel.min(1)]
    }

    /// What the **limiter** sidechain alone was asking for, in dB.
    pub fn limit_gr_db(&self, channel: usize) -> f32 {
        self.lim_gr[channel.min(1)]
    }

    /// Gain reduction in dB for a control voltage, through the fitted law
    /// and the divider.
    fn gr_for_control_v(&self, v: f32) -> f32 {
        self.net.gr_db(self.law.current(v))
    }

    /// The compressor sidechain's configuration for the settings in force.
    fn compress_config(&self) -> SidechainConfig {
        let s = &self.settings;
        let (rel, plat) = compress_recovery_s(s.compress_recovery);
        // The /N is the only revision with a compressor attack switch, and
        // its slow position brings a 100 Hz first-order high-pass with it.
        let slow = s.model == MODEL_33609N && s.compress_attack == 1;
        SidechainConfig {
            threshold_amp: dbu_amp(compress_threshold_dbu(s.compress_threshold)),
            law_slope: RATIO_TRUE[s.compress_ratio.min(4)] - 1.0,
            knee_db: 5.0,
            attack_s: if s.model == MODEL_33609N {
                if slow { 0.006 } else { 0.003 }
            } else {
                0.0056
            },
            release_s: rel,
            platform_s: plat,
            enabled: s.compress_in,
            hpf_hz: if slow { 100.0 } else { 0.0 },
        }
    }

    /// The limiter sidechain's configuration for the settings in force.
    fn limit_config(&self) -> SidechainConfig {
        let s = &self.settings;
        let (rel, plat) = limit_recovery_s(s.limit_recovery);
        SidechainConfig {
            threshold_amp: dbu_amp(limit_threshold_dbu(s.limit_threshold)),
            // The published limit ratio is a 0.1 dB output change for a
            // 10 dB input step, so at least 50:1 and quoted elsewhere as
            // over 100:1. A brickwall, and the open-loop slope is R − 1.
            law_slope: 99.0,
            knee_db: 0.5,
            attack_s: if s.limit_attack == LIMIT_ATTACK_FAST {
                0.000_9
            } else {
                0.001_8
            },
            release_s: rel,
            platform_s: plat,
            enabled: s.limit_in,
            hpf_hz: 0.0,
        }
    }

    /// Process one block in place.
    pub fn process_block(&mut self, l: &mut [f32], r: &mut [f32]) {
        let n = l.len().min(r.len());
        let s = self.settings;
        let cc = self.compress_config();
        let lc = self.limit_config();
        let dt = 1.0 / self.sr;
        let dt_os = if self.oversample { dt * 0.5 } else { dt };
        // The handbook is specific: the make-up "alters the feedback in
        // amplifier 10640 **when the compress in switch is closed**". With
        // the compressor out it is not in circuit at all.
        let make_up_db = if s.compress_in { gain_db(s.gain) } else { 0.0 };
        let make_up = 10f32.powf(make_up_db / 20.0);
        let drive = 10f32.powf(12.0 * s.drive / 20.0);
        let open = self.net.open_gain();
        let processing = !s.bypass && s.power;
        let use_hpf = s.sc_hpf > 1.0;
        let oversample = self.oversample;

        let mut gr_sum = [0.0f32; 2];
        let mut ctrl_sum = [0.0f32; 2];
        let mut comp_sum = [0.0f32; 2];
        let mut lim_sum = [0.0f32; 2];
        self.in_peak = [0.0; 2];
        self.out_peak = [0.0; 2];
        self.in_abs = [0.0; 2];
        self.out_abs = [0.0; 2];

        for i in 0..n {
            let x = [l[i], r[i]];
            for (c, xc) in x.iter().enumerate() {
                self.in_peak[c] = self.in_peak[c].max(xc.abs());
                self.in_abs[c] += xc.abs();
            }

            if !processing {
                for c in 0..2 {
                    if oversample {
                        self.ch[c].dry.process(x[c]);
                    }
                }
                l[i] = x[0];
                r[i] = x[1];
                self.out_peak = self.in_peak;
                self.out_abs[0] += x[0].abs();
                self.out_abs[1] += x[1].abs();
                continue;
            }

            let mut ctrl = [V_CONTROL_FOOT; 2];
            for c in 0..2 {
                // The lab's shared side-chain high-pass, which the
                // hardware does not have. The /N's own 100 Hz filter is
                // separate and lives in the compressor's config.
                let sc_in = if use_hpf {
                    self.ch[c].sc_hpf.hp(x[c])
                } else {
                    x[c]
                };
                // What the bridge did to the signal one sample ago, which
                // is how each detector learns what the *other* one is up
                // to.
                let applied = self.gr_for_control_v(self.ch[c].z_ctrl);
                let mut hpf = self.ch[c].comp_hpf;
                let vc = self.ch[c].comp.step(
                    sc_in,
                    &cc,
                    &mut hpf,
                    &Tap {
                        net: &self.net,
                        law: &self.law,
                        applied_gr_db: applied,
                        tap_gain_db: 0.0,
                        dt,
                    },
                );
                self.ch[c].comp_hpf = hpf;
                let mut hpf = self.ch[c].lim_hpf;
                let vl = self.ch[c].lim.step(
                    sc_in,
                    &lc,
                    &mut hpf,
                    &Tap {
                        net: &self.net,
                        law: &self.law,
                        applied_gr_db: applied,
                        tap_gain_db: make_up_db,
                        dt,
                    },
                );
                self.ch[c].lim_hpf = hpf;
                // Two emitter followers into one shared load: the larger
                // holds the node and the other turns off. A maximum, not
                // a sum.
                ctrl[c] = vc.max(vl);
                comp_sum[c] += self.gr_for_control_v(vc);
                lim_sum[c] += self.gr_for_control_v(vl);
            }
            if s.link {
                let m = ctrl[0].max(ctrl[1]);
                ctrl = [m, m];
            }

            for c in 0..2 {
                ctrl_sum[c] += ctrl[c];
                let control = self.law.current(ctrl[c]);
                let gain = self.net.gain_for_resistance(
                    noob_electrical_components::diode_bridge::small_signal_resistance(
                        control, self.net.k,
                    ),
                );
                gr_sum[c] += 20.0 * (open / gain).log10();

                // Through the bridge. The signal is scaled into the
                // bridge's own volts, solved there, and scaled back, so
                // the divider is exact and only the nonlinearity sees the
                // absolute level.
                let scale = BRIDGE_DRIVE_V * drive;
                let dry = x[c];
                let v_in = self.ch[c].dc_in.hp(dry) * scale;
                let wet = if oversample {
                    let pair = self.ch[c].up.process(v_in);
                    let a = self.net.solve_node(pair[0], control);
                    let b = self.net.solve_node(pair[1], control);
                    let _ = dt_os;
                    self.ch[c].down.process([a, b])
                } else {
                    self.net.solve_node(v_in, control)
                } / scale;
                let wet = self.ch[c].dc_out.hp(wet) / open;

                // The RV1 wiper: after the bridge, before the make-up.
                let pre = wet;
                let post = pre * make_up;
                self.ch[c].z_ctrl = ctrl[c];
                self.ch[c].z_pre = flush(pre);
                self.ch[c].z_post = flush(post);

                let dry_c = if oversample {
                    self.ch[c].dry.process(dry)
                } else {
                    dry
                };
                let y = dry_c + (post - dry_c) * s.mix;
                if c == 0 {
                    l[i] = y
                } else {
                    r[i] = y
                }
                self.out_peak[c] = self.out_peak[c].max(y.abs());
                self.out_abs[c] += y.abs();
            }
        }

        if n > 0 {
            let inv = 1.0 / n as f32;
            self.gr_db = [gr_sum[0] * inv, gr_sum[1] * inv];
            self.in_abs[0] *= inv;
            self.in_abs[1] *= inv;
            self.out_abs[0] *= inv;
            self.out_abs[1] *= inv;
            self.ctrl_v = [ctrl_sum[0] * inv, ctrl_sum[1] * inv];
            self.comp_gr = [comp_sum[0] * inv, comp_sum[1] * inv];
            self.lim_gr = [lim_sum[0] * inv, lim_sum[1] * inv];
            self.frames = n;
            let target = self.meter_target();
            self.vu.advance(target, n);
        }
    }

    /// What the needle is pointing at, before ballistics.
    fn meter_target(&self) -> f32 {
        let gr = 0.5 * (self.gr_db[0] + self.gr_db[1]);
        // The 2254/E's meter switches between input, control and output;
        // the 33609's reads gain reduction only.
        let level = self.settings.model == MODEL_2254E && self.settings.meter_select != 1;
        if !self.settings.power {
            // Unpowered, so the movement falls back against its own stop.
            // Which stop that is depends on what the scale is: a gain
            // reduction scale rests at zero, a VU scale off the bottom.
            return if level { -60.0 } else { 0.0 };
        }
        if level {
            let vu_of = |mean: f32| 20.0 * ((mean / SINE_MEAN_ABS) / VU_REF_AMP).max(1e-4).log10();
            match self.settings.meter_select {
                0 => vu_of(0.5 * (self.in_abs[0] + self.in_abs[1])),
                _ => vu_of(0.5 * (self.out_abs[0] + self.out_abs[1])),
            }
        } else {
            -gr
        }
    }

    /// `[in_l, in_r, out_l, out_r, gr_db, meter_vu]` for the last block,
    /// with `gr_db` **positive** for reduction, which is the lab's frame
    /// convention; the lab negates it on the way out.
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

    /// Steady-state gain reduction in dB (positive) for a sine of peak
    /// `amp_peak`.
    ///
    /// Solved as a fixed point rather than by running the loop, because
    /// the recovery can be three seconds long. The loop is a feedback one,
    /// so the detector sees the compressed output and the solution is
    /// where the two agree.
    pub fn static_gr_db(&self, amp_peak: f32) -> f32 {
        let s = &self.settings;
        if s.bypass || !s.power {
            return 0.0;
        }
        let cc = self.compress_config();
        let lc = self.limit_config();
        let make_up = if s.compress_in {
            10f32.powf(gain_db(s.gain) / 20.0)
        } else {
            1.0
        };
        // Each detector refers its reading back through the bridge, so
        // each one's excess is over the level at its own tap point with
        // the reduction added back: the input for the compressor, the
        // input plus the make-up for the limiter. That makes this a
        // closed form rather than the fixed-point search it used to be.
        let demand = |cfg: &SidechainConfig, amp: f32| {
            if !cfg.enabled {
                return 0.0;
            }
            let over = 20.0 * (amp.max(1e-12) / cfg.threshold_amp).log10();
            softplus_db(over, cfg.knee_db) * cfg.law_slope / (cfg.law_slope + 1.0)
        };
        demand(&cc, amp_peak)
            .max(demand(&lc, amp_peak * make_up))
            .max(0.0)
    }

    /// The static transfer curve, output dBFS for `min_dbfs..max_dbfs` in.
    pub fn transfer_curve(&self, out: &mut [f32], min_dbfs: f32, max_dbfs: f32) {
        let n = out.len();
        if n == 0 {
            return;
        }
        let make_up = gain_db(self.settings.gain);
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

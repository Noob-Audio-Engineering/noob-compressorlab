//! The SSL 4000 G bus compressor's engine.
//!
//! The shape, from `research/SSL-Gbus.md` 11.1:
//!
//! ```text
//!                     ┌──────────────── GR[n−1] (dB) ───────────────┐
//!                     ↓                                             │
//! in_L ─┬─► × 10^((M − GR)/20) ───────────────────────────► out_L   │
//!       │                                                           │
//!       └─► × 10^((−Θ − GR)/20) ─► HPF ─► |·| ─┐                    │
//!                                               ├─► max ─► × G      │
//! in_R ─┬─► × 10^((−Θ − GR)/20) ─► HPF ─► |·| ─┘         │          │
//!       │                                                ↓          │
//!       └─► × 10^((M − GR)/20) ───────────────► out_R  diode D6     │
//!                                                        ↓          │
//!                                          attack R ─► RC network ──┘
//!                                                        ↓
//!                                                  GR = V/k  (dB)
//! ```
//!
//! The audio path is one multiply. There are no filters in it, no
//! transformer and no saturator; the only nonlinearity is inside the
//! multiply, in [`BlackmerCell`]. Latency is zero unless the oversampled
//! path is switched in.

use super::{
    ATTACK_R, AUTO_C1, AUTO_C2, AUTO_R7, AUTO_R8, DETECTOR_GAIN, DETECTOR_SCALE, HPF_HZ,
    LN10_OVER_20, RATIO_PRINTED, RELEASE_AUTO, RELEASE_R, SOFTPLUS_V, TIMING_C, V_DIODE,
    ratio_scaling,
};
use crate::dsp::fet::oversample::{Downsampler, LATENCY, Upsampler};
use crate::dsp::flush;

/// How the two channels' detectors are tied together.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Link {
    /// The hardware: both channels are rectified separately and **the
    /// louder one** controls the gain of both. Not a sum and not an
    /// average.
    #[default]
    Dominant,
    /// Ours: the mean of the two rectified channels, which is what a
    /// well-regarded emulation of this box does instead.
    Sum,
    /// Ours: two independent detectors and two independent gains. This is
    /// also what the lab's shared `link` parameter selects when it is off.
    Dual,
    /// Ours: independent detectors on the mid and side signals.
    MidSide,
}

impl Link {
    pub fn from_index(i: usize) -> Self {
        match i {
            0 => Link::Dominant,
            1 => Link::Sum,
            2 => Link::Dual,
            _ => Link::MidSide,
        }
    }
}

/// The Blackmer gain cell, and nothing else.
///
/// This is a `dbx 202C` on SSL's card 82E26, which is a module built from
/// ten paralleled `dbx 2150` cells around a common control buffer. The
/// numbers here are the THAT Corporation datasheet's, since THAT bought the
/// design and still publish it; the SSL drawing names the part and the
/// datasheet gives the law.
///
/// **The boundary is deliberately narrow.** This struct knows the control
/// law, its linearity tolerance, its temperature coefficient and its
/// symmetry residual. It does not know about the 68.1 kΩ input resistor,
/// the I/V converter, the detector, the threshold, or how the control
/// voltage was arrived at. Those are the circuit around it and they live in
/// [`Compressor`].
///
/// # Why the distortion is applied before the gain
///
/// The dossier's section 11.3 writes the cell as
/// `y = x·gain + d2·(x·gain)²`, shaping the *output*. The datasheet gives
/// two THD points, and they settle it: 0.005 % at 0 dBV with 0 dB of gain,
/// and **0.020 % at +10 dBV with −15 dB of gain**. The second point has a
/// *lower* output level than the first and four times the distortion, so
/// the distortion cannot be a function of the output. Shaping the input
/// fits the first point exactly by construction and the second to within
/// 27 %, which is inside the ±50 % the dossier's own test 24 allows;
/// shaping the output misses the second by a factor of seven. This is a
/// current-mode cell whose distortion is set by the current driven into it,
/// which is the input voltage across a resistor, so the datasheet and the
/// topology agree.
#[derive(Clone, Copy, Debug)]
pub struct BlackmerCell {
    /// Gain-control constant, volts per dB. −6.1 mV/dB typical on the
    /// datasheet, with −6.2 and −6.0 as the min and max.
    volts_per_db: f32,
    /// Second-harmonic coefficient, per unit sample amplitude at the
    /// cell's input.
    d2: f32,
    /// Running mean of the squared input, used to take the DC that a
    /// squarer necessarily produces back out again. A console strip blocks
    /// it with a coupling capacitor; taking it off the squared term alone
    /// leaves the linear path flat to floating-point precision, which is
    /// what the audio path having no filters in it means.
    dc: f32,
    /// One-pole coefficient for `dc`, about 2 Hz.
    dc_a: f32,
}

/// Volts per unit sample amplitude in the audio path.
///
/// 0 VU is [`crate::dsp::VU_REF_DBFS`] = −18 dBFS and, on this box, +4 dBu
/// = 1.228 V RMS. A sine of peak amplitude 10^(−18/20) therefore measures
/// 1.228 V RMS, so one unit of sample amplitude is 13.79 V.
pub const VOLTS_PER_SAMPLE: f32 = 13.794;

/// The second-harmonic coefficient at unity drive, per unit sample
/// amplitude.
///
/// From the THAT 2180A datasheet's "VIN = 0 dBV, 0 dB gain: **0.005 %**".
/// For `y = u + d2·u²` a sine of peak `U` gives a second harmonic of
/// `d2·U²/2` against a fundamental of `U`, so `THD = d2·U/2`. At 0 dBV,
/// `U = √2` volts, giving `d2 = 7.07e-5` per volt, which is
/// `7.07e-5 × 13.794 = 9.75e-4` per unit sample amplitude.
pub const D2_UNITY: f32 = 9.754e-4;

impl Default for BlackmerCell {
    fn default() -> Self {
        BlackmerCell {
            volts_per_db: -6.1e-3,
            d2: D2_UNITY,
            dc: 0.0,
            dc_a: 0.0,
        }
    }
}

impl BlackmerCell {
    pub fn new(sr: f32) -> Self {
        let mut c = BlackmerCell::default();
        c.set_sample_rate(sr);
        c
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.dc_a = (-2.0 * std::f32::consts::PI * 2.0 / sr.max(1.0)).exp();
    }

    pub fn reset(&mut self) {
        self.dc = 0.0;
    }

    /// The datasheet's gain-control constant, volts per dB (negative).
    pub fn volts_per_db(&self) -> f32 {
        self.volts_per_db
    }

    /// Set the second-harmonic coefficient as a multiple of [`D2_UNITY`].
    pub fn set_drive(&mut self, multiple: f32) {
        self.d2 = D2_UNITY * multiple.max(0.0);
    }

    /// The control voltage the cell's port needs for `gain_db` of gain.
    ///
    /// The engine works in decibels throughout, so nothing in the audio
    /// path calls this. It exists because it is the cell's actual
    /// interface, and it is what a component crate would take.
    #[inline]
    pub fn control_volts(&self, gain_db: f32) -> f32 {
        gain_db * self.volts_per_db
    }

    /// Linear gain for a control voltage at the cell's port.
    #[inline]
    pub fn gain_from_volts(&self, v: f32) -> f32 {
        10f32.powf(v / self.volts_per_db / 20.0)
    }

    /// Linear gain for a gain in dB. Exponential in dB by construction,
    /// which is what makes the control voltage linear in decibels and the
    /// gain-reduction meter's linear scale correct.
    #[inline]
    pub fn gain(&self, gain_db: f32) -> f32 {
        10f32.powf(gain_db / 20.0)
    }

    /// Advance the DC estimate and shape one sample. The linear term is
    /// untouched, so with the drive at rest this is `x` to within 1e-6.
    #[inline]
    pub fn shape(&mut self, x: f32) -> f32 {
        let sq = x * x;
        self.dc = flush(self.dc * self.dc_a + sq * (1.0 - self.dc_a));
        x + self.d2 * (sq - self.dc)
    }
}

/// A first-order high-pass, which is the only order published for anything
/// in this family.
#[derive(Clone, Copy, Debug, Default)]
struct OnePoleHp {
    a: f32,
    x1: f32,
    y1: f32,
    on: bool,
}

impl OnePoleHp {
    fn set(&mut self, hz: f32, sr: f32) {
        self.on = hz > 0.0;
        self.a = if self.on {
            (-2.0 * std::f32::consts::PI * hz / sr.max(1.0)).exp()
        } else {
            0.0
        };
    }

    fn reset(&mut self) {
        self.x1 = 0.0;
        self.y1 = 0.0;
    }

    #[inline]
    fn process(&mut self, x: f32) -> f32 {
        if !self.on {
            return x;
        }
        let y = self.a * (self.y1 + x - self.x1);
        self.x1 = x;
        self.y1 = flush(y);
        self.y1
    }
}

/// `s·ln(1 + exp(u/s))`, the diode's turn-on, computed without overflowing.
#[inline]
fn softplus(u: f32, s: f32) -> f32 {
    let t = u / s;
    if t > 20.0 {
        u
    } else if t < -20.0 {
        0.0
    } else {
        s * (1.0 + t.exp()).ln()
    }
}

/// The derivative of [`softplus`], which is the diode's incremental
/// conductance normalised out of its resistor.
#[inline]
fn sigmoid(u: f32, s: f32) -> f32 {
    let t = u / s;
    if t > 20.0 {
        1.0
    } else if t < -20.0 {
        0.0
    } else {
        1.0 / (1.0 + (-t).exp())
    }
}

/// The timing network: the attack resistor, the release capacitor and its
/// discharge resistor, as three components rather than two coefficients.
///
/// One section for the four fixed release positions and two for Auto. The
/// diode is not a branch: the same continuous equation
/// `C·dV/dt = i_D(d − V) − V/R` runs at every level, and it reduces exactly
/// to the dossier's charging solution when the diode conducts and to its
/// discharging solution when it does not. That is why there is no corner in
/// the transfer curve and no knee parameter anywhere in this file.
#[derive(Clone, Copy, Debug, Default)]
pub struct Timing {
    v1: f32,
    v2: f32,
    auto: bool,
    r_att: f32,
    r_rel: f32,
    c: f32,
    dt: f32,
}

impl Timing {
    pub fn new(sr: f32) -> Self {
        let mut t = Timing {
            r_att: ATTACK_R[2],
            r_rel: RELEASE_R[3],
            c: TIMING_C,
            ..Default::default()
        };
        t.set_sample_rate(sr);
        t
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.dt = 1.0 / sr.max(1.0);
    }

    /// A network with the release path opened, so a step response measures
    /// the bare attack constant `R × C`.
    ///
    /// The dossier's test 13 asks for the **open-loop** attack time
    /// constant, and in the assembled network the attack and release
    /// resistors form a divider that shortens it by up to 18 % at the
    /// slowest position. This is the instrument that test needs, and
    /// nothing in the audio path uses it.
    pub fn open_loop(sr: f32, attack: usize) -> Self {
        let mut t = Timing::new(sr);
        t.configure(attack, 0);
        t.r_rel = 1e12;
        t
    }

    pub fn reset(&mut self) {
        self.v1 = 0.0;
        self.v2 = 0.0;
    }

    /// Select the attack resistor and the release position.
    pub fn configure(&mut self, attack: usize, release: usize) {
        self.r_att = ATTACK_R[attack.min(ATTACK_R.len() - 1)];
        self.auto = release >= RELEASE_AUTO;
        if !self.auto {
            self.r_rel = RELEASE_R[release.min(RELEASE_R.len() - 1)];
            self.c = TIMING_C;
        }
    }

    /// The control voltage now.
    #[inline]
    pub fn voltage(&self) -> f32 {
        self.v1 + self.v2
    }

    /// One section of `C·dv/dt = i − v/R`, with the diode linearised about
    /// the present voltage and the result integrated exactly over the
    /// sample. Exact in both limits: with the diode fully on the time
    /// constant is `(R_att ∥ R)·C` and the target is the two resistors'
    /// divider, and with it off the time constant is `R·C` and the target
    /// is zero.
    #[inline]
    fn section(&self, v0: f32, i0: f32, sigma: f32, r: f32, c: f32) -> f32 {
        let g = sigma / self.r_att + 1.0 / r;
        let v_eq = (i0 + sigma * v0 / self.r_att) / g;
        let a = (-self.dt * g / c).exp();
        flush(v_eq + (v0 - v_eq) * a)
    }

    /// Advance one sample with `d` volts at the detector's output, and
    /// return the control voltage.
    #[inline]
    pub fn step(&mut self, d: f32) -> f32 {
        let v = self.v1 + self.v2;
        // **The diode's forward drop belongs here**, not only in the
        // steady-state law. D6 sits in series between the detector's
        // output on card 82E26 and this network on 82E27, so current
        // flows only once `d` exceeds the voltage already on the
        // capacitor *plus* one drop. Leaving it out lets the softplus's
        // own tail at zero drive charge the network: with the release at
        // 0.1 s that parked the model at 4.4 dB of gain reduction with no
        // signal at all, and every static test failed by that amount.
        // With it in, the equilibrium under conduction is
        // `V = (d − V_d)·R_rel/(R_att + R_rel)`, which is the dossier's
        // loop equation `k·GR + V_d = d` once the divider is unity.
        let u = d - v - V_DIODE;
        // The diode's current, and its slope for the linearisation. The
        // attack resistor is inside the softplus's argument and outside
        // its value, so this is `softplus(i·R_att)/R_att` written once.
        let i0 = softplus(u, SOFTPLUS_V) / self.r_att;
        let sigma = sigmoid(u, SOFTPLUS_V);
        if self.auto {
            // Two sections in series to ground, driven by one current and
            // decaying independently. A short peak splits the charge as
            // C2/C1 = 14.5 in favour of the fast one; a sustained tone
            // settles at R8/(R7+R8) = 89.2 % on the slow one. Neither
            // number is coded: both fall out of the four components.
            let v1 = self.section(self.v1, i0, sigma, AUTO_R7, AUTO_C1);
            let v2 = self.section(self.v2, i0, sigma, AUTO_R8, AUTO_C2);
            self.v1 = v1;
            self.v2 = v2;
        } else {
            self.v1 = self.section(self.v1, i0, sigma, self.r_rel, self.c);
            self.v2 = 0.0;
        }
        self.v1 + self.v2
    }

    /// Discharge only, with the detector **disconnected** rather than
    /// driven to zero.
    ///
    /// The IN switch opens the sidechain, so no current can reach the
    /// timing node at all. Feeding a zero driving voltage into
    /// [`step`](Self::step) instead leaves D6's own softplus tail charging
    /// the network, which parked the model at 7e-5 dB of gain reduction
    /// with the compressor switched out. Small, but the dossier's test 2
    /// says exactly zero and it is right to: an open switch is an open
    /// switch.
    #[inline]
    pub fn release_only(&mut self) -> f32 {
        if self.auto {
            let v1 = self.section(self.v1, 0.0, 0.0, AUTO_R7, AUTO_C1);
            let v2 = self.section(self.v2, 0.0, 0.0, AUTO_R8, AUTO_C2);
            self.v1 = v1;
            self.v2 = v2;
        } else {
            self.v1 = self.section(self.v1, 0.0, 0.0, self.r_rel, self.c);
            self.v2 = 0.0;
        }
        self.v1 + self.v2
    }

    /// The fast and slow sections separately, for the tests that fit the
    /// two exponentials of the Auto release.
    pub fn sections(&self) -> (f32, f32) {
        (self.v1, self.v2)
    }
}

/// Everything the engine needs from the parameters, read once per block.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// The hardware IN switch. **This is not a bypass.** With it off the
    /// audio still passes through the VCA and the make-up gain is still
    /// applied; only the sidechain is removed.
    pub sidechain_in: bool,
    /// THRESHOLD, dB, as the panel reads it. More negative compresses more.
    pub threshold_db: f32,
    /// MAKE UP, dB. Permanently in circuit, like the hardware's.
    pub makeup_db: f32,
    /// ATTACK switch, 0..5.
    pub attack: usize,
    /// RELEASE switch, 0..4, with 4 the automatic network.
    pub release: usize,
    /// RATIO switch, 0..2.
    pub ratio: usize,
    /// Sidechain HPF switch, 0..5, with 0 off.
    pub hpf: usize,
    /// How the detectors are tied together.
    pub link_mode: Link,
    /// Ours: scales the gain cell's second-harmonic term, 0..1.
    pub drive: f32,
    /// Ours: a ceiling on the gain reduction, dB.
    pub range_db: f32,
    /// Oversample the audio multiply.
    pub oversample: bool,
    /// The lab's shared stereo link. False forces [`Link::Dual`].
    pub link: bool,
    /// The lab's shared wet share, 0..1.
    pub mix: f32,
    /// The lab's shared sidechain high-pass in Hz, in series with the
    /// hardware's own switch. 0 is off.
    pub sc_hpf: f32,
    /// The plug-in's own sample-exact bypass.
    pub bypass: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            sidechain_in: true,
            threshold_db: 0.0,
            makeup_db: 0.0,
            // 1 ms, and Auto: SSL's own plug-in opens there.
            attack: 2,
            release: RELEASE_AUTO,
            ratio: 1,
            hpf: 0,
            link_mode: Link::Dominant,
            drive: 0.0,
            range_db: 20.0,
            oversample: true,
            link: true,
            mix: 1.0,
            sc_hpf: 0.0,
            bypass: false,
        }
    }
}

/// One audio channel's oversampling and dry-delay state.
#[derive(Clone)]
struct Channel {
    up: Upsampler,
    down: Downsampler,
    cell: BlackmerCell,
    dry: [f32; LATENCY + 1],
    dry_pos: usize,
    /// Last block's gain, so the gain can be interpolated across the
    /// oversampled pair rather than stepped.
    g_prev: f32,
    hpf_hw: OnePoleHp,
    hpf_lab: OnePoleHp,
    timing: Timing,
    gr_db: f32,
}

impl Channel {
    fn new(sr: f32) -> Self {
        Channel {
            up: Upsampler::new(),
            down: Downsampler::new(),
            cell: BlackmerCell::new(sr),
            dry: [0.0; LATENCY + 1],
            dry_pos: 0,
            g_prev: 1.0,
            hpf_hw: OnePoleHp::default(),
            hpf_lab: OnePoleHp::default(),
            timing: Timing::new(sr),
            gr_db: 0.0,
        }
    }

    fn set_sample_rate(&mut self, sr: f32) {
        self.cell.set_sample_rate(sr);
        self.timing.set_sample_rate(sr);
    }

    fn reset(&mut self) {
        self.up.reset();
        self.down.reset();
        self.cell.reset();
        self.dry = [0.0; LATENCY + 1];
        self.dry_pos = 0;
        self.g_prev = 1.0;
        self.hpf_hw.reset();
        self.hpf_lab.reset();
        self.timing.reset();
        self.gr_db = 0.0;
    }
}

/// The engine.
pub struct Compressor {
    sr: f32,
    settings: Settings,
    first: bool,
    ch: [Channel; 2],
    in_peak: [f32; 2],
    out_peak: [f32; 2],
    /// Gain reduction in dB, positive, as the meter reads it.
    gr_db: f32,
}

impl Compressor {
    pub fn new(sr: f32) -> Self {
        let mut c = Compressor {
            sr,
            settings: Settings::default(),
            first: true,
            ch: [Channel::new(sr), Channel::new(sr)],
            in_peak: [0.0; 2],
            out_peak: [0.0; 2],
            gr_db: 0.0,
        };
        let s = c.settings;
        c.configure(s);
        c
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sr = sr;
        for c in &mut self.ch {
            c.set_sample_rate(sr);
        }
        self.first = true;
        self.reset();
        let s = self.settings;
        self.configure(s);
    }

    pub fn reset(&mut self) {
        for c in &mut self.ch {
            c.reset();
        }
        self.in_peak = [0.0; 2];
        self.out_peak = [0.0; 2];
        self.gr_db = 0.0;
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Zero unless the oversampled path is switched in; the audio path
    /// never touches the detector, so nothing else can add delay.
    pub fn latency(&self) -> usize {
        if self.settings.oversample { LATENCY } else { 0 }
    }

    /// Apply a settings snapshot; `true` when anything changed.
    pub fn configure(&mut self, s: Settings) -> bool {
        let changed = self.first || s != self.settings;
        if !changed {
            return false;
        }
        self.first = false;
        self.settings = s;
        let hw = HPF_HZ[s.hpf.min(HPF_HZ.len() - 1)];
        for c in &mut self.ch {
            c.timing.configure(s.attack, s.release);
            c.hpf_hw.set(hw, self.sr);
            c.hpf_lab.set(s.sc_hpf, self.sr);
            // 0.005 % at rest, to about 0.3 % fully driven, which is past
            // the datasheet's worst grade. Ours, and marked so on the page.
            c.cell.set_drive(1.0 + s.drive.clamp(0.0, 1.0) * 63.0);
        }
        true
    }

    /// The effective link behaviour: the lab's shared switch, when it is
    /// off, means two independent detectors whatever the mode says.
    fn link_mode(&self) -> Link {
        if self.settings.link {
            self.settings.link_mode
        } else {
            Link::Dual
        }
    }

    /// The printed ratio in force.
    fn printed_ratio(&self) -> f32 {
        RATIO_PRINTED[self.settings.ratio.min(RATIO_PRINTED.len() - 1)]
    }

    /// The detector stage's gain in force.
    fn detector_gain(&self) -> f32 {
        DETECTOR_GAIN[self.settings.ratio.min(DETECTOR_GAIN.len() - 1)]
    }

    /// Gain reduction in dB, positive.
    pub fn gr_db(&self) -> f32 {
        self.gr_db
    }

    /// One channel's gain reduction in dB, positive. In the linked modes
    /// both channels carry the same value by construction.
    pub fn channel_gr_db(&self, channel: usize) -> f32 {
        self.ch[channel.min(1)].gr_db
    }

    /// The control voltage on one channel's timing network, volts.
    pub fn control_v(&self, channel: usize) -> f32 {
        self.ch[channel.min(1)].timing.voltage()
    }

    /// One channel's timing network, for the tests that fit the automatic
    /// release's two exponentials.
    pub fn timing(&self, channel: usize) -> &Timing {
        &self.ch[channel.min(1)].timing
    }

    /// `[in_l, in_r, out_l, out_r, gr_db, meter]`. The meter reads the
    /// control voltage on a linear 0 to 20 dB scale, and it is the level
    /// the needle chases, not where the needle is: the movement runs in
    /// [`crate::dsp::vu`] one level up.
    pub fn meter_frame(&self) -> [f32; 6] {
        [
            self.in_peak[0],
            self.in_peak[1],
            self.out_peak[0],
            self.out_peak[1],
            self.gr_db,
            self.gr_db,
        ]
    }

    /// The sidechain VCA's gain in dB: the audio VCA's gain plus the
    /// threshold offset, which is the whole architectural argument of this
    /// box in one line (dossier 3.4).
    ///
    /// **The threshold's sign.** SSL state the equivalence only once, of
    /// the XLogic's sidechain trims: they "increase the side chain level by
    /// 10dB — effectively reducing the threshold on that channel by 10dB".
    /// A threshold reading and a sidechain gain therefore run in opposite
    /// directions, so the panel's dB appear here negated and the knob reads
    /// as its legend does. The dossier's section 11.4 writes `+T`, which
    /// makes the same control compress harder as it is turned up; that is
    /// consistent only if `T` denotes the sidechain offset rather than the
    /// panel's threshold, and this parameter is the panel's.
    #[inline]
    fn sidechain_gain_db(&self, gr_db: f32) -> f32 {
        -self.settings.threshold_db - gr_db
    }

    /// Detector volts for one rectified channel amplitude.
    #[inline]
    fn detector_volts(&self, rectified: f32) -> f32 {
        self.detector_gain() * DETECTOR_SCALE * rectified
    }

    /// Gain reduction in dB from a control voltage, capped by the range
    /// control.
    #[inline]
    fn gr_from_volts(&self, v: f32) -> f32 {
        let k = ratio_scaling(self.printed_ratio());
        (v / k).clamp(0.0, self.settings.range_db.max(0.0))
    }

    /// Process one stereo block in place. Real-time safe.
    pub fn process_block(&mut self, l: &mut [f32], r: &mut [f32]) {
        let n = l.len().min(r.len());
        if n == 0 {
            return;
        }
        let s = self.settings;
        let mid_side = self.link_mode() == Link::MidSide;
        let mut pin = [0.0f32; 2];
        let mut pout = [0.0f32; 2];

        for i in 0..n {
            let xl = l[i];
            let xr = r[i];
            pin[0] = pin[0].max(xl.abs());
            pin[1] = pin[1].max(xr.abs());

            // The two signals the gain cells act on. In mid-side that is
            // the matrix; otherwise it is the channels themselves.
            let (a0, a1) = if mid_side {
                ((xl + xr) * 0.5, (xl - xr) * 0.5)
            } else {
                (xl, xr)
            };

            // ---- the sidechain, at the base rate -------------------------
            // Each channel's detector reads the input with the gain
            // reduction already applied, which is what closes the loop.
            let mut rect = [0.0f32; 2];
            for (c, x) in [a0, a1].into_iter().enumerate() {
                let sc = 10f32.powf(self.sidechain_gain_db(self.ch[c].gr_db) / 20.0);
                let mut v = x * sc;
                v = self.ch[c].hpf_hw.process(v);
                v = self.ch[c].hpf_lab.process(v);
                rect[c] = v.abs();
            }

            let gr = if !s.sidechain_in {
                // The sidechain is removed, so the detector drives nothing
                // and the capacitor simply discharges. The VCA and the
                // make-up gain stay in circuit, which is the thing about
                // this switch that is not what a plug-in author guesses.
                let v0 = self.ch[0].timing.release_only();
                let v1 = self.ch[1].timing.release_only();
                let g0 = self.gr_from_volts(v0);
                let g1 = self.gr_from_volts(v1);
                self.ch[0].gr_db = g0;
                self.ch[1].gr_db = g1;
                [g0, g1]
            } else {
                match self.link_mode() {
                    Link::Dominant | Link::Sum => {
                        let combined = if self.link_mode() == Link::Dominant {
                            rect[0].max(rect[1])
                        } else {
                            0.5 * (rect[0] + rect[1])
                        };
                        let d = self.detector_volts(combined);
                        let v = self.ch[0].timing.step(d);
                        let g = self.gr_from_volts(v);
                        // One control voltage drives both audio VCAs, so
                        // the two gains are identical by construction.
                        self.ch[0].gr_db = g;
                        self.ch[1].gr_db = g;
                        // Keep the idle network at rest so a switch to a
                        // dual mode does not jump.
                        self.ch[1].timing.release_only();
                        [g, g]
                    }
                    Link::Dual | Link::MidSide => {
                        let d0 = self.detector_volts(rect[0]);
                        let d1 = self.detector_volts(rect[1]);
                        let v0 = self.ch[0].timing.step(d0);
                        let v1 = self.ch[1].timing.step(d1);
                        let g0 = self.gr_from_volts(v0);
                        let g1 = self.gr_from_volts(v1);
                        self.ch[0].gr_db = g0;
                        self.ch[1].gr_db = g1;
                        [g0, g1]
                    }
                }
            };

            // ---- the audio path: one multiply ---------------------------
            let mut wet = [0.0f32; 2];
            let mut dry = [0.0f32; 2];
            for (c, x) in [a0, a1].into_iter().enumerate() {
                let g_db = s.makeup_db - gr[c];
                let g = self.ch[c].cell.gain(g_db);
                let ch = &mut self.ch[c];
                if s.oversample {
                    let pair = ch.up.process(x);
                    let g_mid = 0.5 * (ch.g_prev + g);
                    let y0 = ch.cell.shape(pair[0]) * g_mid;
                    let y1 = ch.cell.shape(pair[1]) * g;
                    wet[c] = ch.down.process([y0, y1]);
                } else {
                    wet[c] = ch.cell.shape(x) * g;
                }
                ch.g_prev = g;
                // The dry path is held back by the resampler's round trip
                // so that a mix or a bypass does not comb-filter itself.
                ch.dry[ch.dry_pos] = x;
                let read = if s.oversample {
                    (ch.dry_pos + 1) % (LATENCY + 1)
                } else {
                    ch.dry_pos
                };
                dry[c] = ch.dry[read];
                ch.dry_pos = (ch.dry_pos + 1) % (LATENCY + 1);
            }

            let (mut o0, mut o1) = if s.bypass {
                (dry[0], dry[1])
            } else {
                (
                    dry[0] + (wet[0] - dry[0]) * s.mix,
                    dry[1] + (wet[1] - dry[1]) * s.mix,
                )
            };
            if mid_side {
                let (m, sd) = (o0, o1);
                o0 = m + sd;
                o1 = m - sd;
            }
            l[i] = o0;
            r[i] = o1;
            pout[0] = pout[0].max(o0.abs());
            pout[1] = pout[1].max(o1.abs());
        }

        self.in_peak = pin;
        self.out_peak = pout;
        self.gr_db = if s.bypass {
            0.0
        } else {
            0.5 * (self.ch[0].gr_db + self.ch[1].gr_db)
        };
    }

    /// The resistance the timing capacitor discharges through, ohms. In
    /// the automatic position the two sections are in series, so at DC
    /// they add.
    fn shunt_resistance(&self) -> f32 {
        if self.settings.release >= RELEASE_AUTO {
            AUTO_R7 + AUTO_R8
        } else {
            RELEASE_R[self.settings.release.min(RELEASE_R.len() - 1)]
        }
    }

    /// Steady-state gain reduction in dB for a signal of peak amplitude
    /// `amp`.
    ///
    /// **This solves the network the engine actually runs, not the ideal
    /// loop equation.** The dossier's `k·GR + V_d = G·A·10^((L − Θ − GR)/20)`
    /// treats D6 as a perfect 0.6 V drop with nothing downstream. The real
    /// circuit has a soft diode and a discharge resistor, so at rest the
    /// diode passes exactly the current that resistor draws, which puts it
    /// a few tens of millivolts *below* the ideal drop rather than at it.
    /// Solving the ideal form instead disagreed with the engine by 4 dB at
    /// nominal level, which would have drawn a transfer curve the audio
    /// does not follow. The condition here is the engine's own:
    ///
    /// ```text
    /// softplus(d(GR) − k·GR − V_d) / R_att  =  k·GR / R_shunt
    /// ```
    ///
    /// Bisected rather than iterated: the map's own derivative is the loop
    /// gain, which exceeds one at every useful setting, so a fixed point
    /// would not converge.
    pub fn static_gr_db(&self, amp: f32) -> f32 {
        if amp <= 0.0 || !self.settings.sidechain_in {
            return 0.0;
        }
        let k = ratio_scaling(self.printed_ratio());
        let ga = self.detector_gain() * DETECTOR_SCALE * amp;
        let theta = self.settings.threshold_db;
        let r_att = ATTACK_R[self.settings.attack.min(ATTACK_R.len() - 1)];
        let r_shunt = self.shunt_resistance();
        // Charging current minus discharging current: strictly decreasing
        // in GR, since the detector falls and both loads rise.
        let h = |gr: f32| {
            let d = ga * 10f32.powf((-theta - gr) / 20.0);
            let v = k * gr;
            softplus(d - v - V_DIODE, SOFTPLUS_V) / r_att - v / r_shunt
        };
        let (mut lo, mut hi) = (0.0f32, 200.0f32);
        if h(hi) > 0.0 {
            return self.settings.range_db.max(0.0);
        }
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if h(mid) > 0.0 { lo = mid } else { hi = mid }
        }
        (0.5 * (lo + hi)).clamp(0.0, self.settings.range_db.max(0.0))
    }

    /// The static transfer curve: output level in dBFS for an input level
    /// in dBFS.
    ///
    /// Drawn from the model rather than from a formula, because in this box
    /// the curve is the finding: it bends for its whole length and never
    /// straightens.
    pub fn transfer_curve(&self, out: &mut [f32], min_dbfs: f32, max_dbfs: f32) {
        let n = out.len();
        if n == 0 {
            return;
        }
        let s = &self.settings;
        for (i, o) in out.iter_mut().enumerate() {
            let t = if n == 1 {
                0.0
            } else {
                i as f32 / (n - 1) as f32
            };
            let db = min_dbfs + (max_dbfs - min_dbfs) * t;
            if s.bypass {
                *o = db;
                continue;
            }
            let amp = 10f32.powf(db / 20.0);
            let wet_db = db - self.static_gr_db(amp) + s.makeup_db;
            // Mix in the amplitude domain, as the block loop does.
            let wet = 10f32.powf(wet_db / 20.0);
            *o = 20.0 * (amp + (wet - amp) * s.mix).max(1e-9).log10();
        }
    }

    /// The sidechain high-pass's magnitude response at `hz`, in dB
    /// relative to its passband.
    ///
    /// The filter that runs in the sidechain, measured directly rather than
    /// inferred from how much gain reduction survives: at a quarter and an
    /// eighth of the corner frequency the compressor's own ballistics ring
    /// at the test tone's period, and the dossier's test 23 needs the
    /// filter, not the loop.
    pub fn sidechain_response_db(&self, hz: f32) -> f32 {
        let fc = HPF_HZ[self.settings.hpf.min(HPF_HZ.len() - 1)];
        if fc <= 0.0 {
            return 0.0;
        }
        let mut f = OnePoleHp::default();
        f.set(fc, self.sr);
        let n = (self.sr / hz * 200.0) as usize;
        let step = 2.0 * std::f32::consts::PI * hz / self.sr;
        let mut peak = 0.0f32;
        for i in 0..n {
            let y = f.process((i as f32 * step).sin());
            // Only the last quarter, so the filter has settled.
            if i * 4 > n * 3 {
                peak = peak.max(y.abs());
            }
        }
        20.0 * peak.max(1e-12).log10()
    }

    /// The instantaneous ratio at `gr_db`, for the page and the tests.
    pub fn ratio_at(&self, gr_db: f32) -> f32 {
        self.printed_ratio() + LN10_OVER_20 * gr_db
    }
}

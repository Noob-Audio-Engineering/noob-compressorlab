//! The 610 stage: two tube gain stages with the Gain switch trading
//! attenuation for feedback between them, the shelving equaliser in the
//! second stage's feedback loop, and a transformer at each end
//! (`research/610.md` section 8).
//!
//! Calibration follows the rest of the lab: 0 dBFS is +22 dBu, so +4 dBu
//! (0 VU) is −18 dBFS and the preamp's +20 dBm maximum sits at −2 dBFS.
//! Every constant in [`Voicing`] is an **estimate** tuned against the test
//! plan in `research/610.md` section 9 unless a published figure is named.

use super::adaa::Adaa;
use super::filters::{Hp1, Hp2, Lp1, Shelf, Smooth, Vu, flush};
use super::{
    GAIN_STEPS_A_DB, GAIN_STEPS_DB, HF_FREQ_HZ, INPUT_OFFSET_DB, LF_FREQ_HZ, SHELF_GAIN_DB,
    Settings, level_to_db,
};
use crate::dsp::fet::oversample::{Downsampler, DryDelay, LATENCY, Upsampler};

/// 0 VU of the PRE meter, as the mean rectified value of a sine at
/// −18 dBFS.
pub const VU_REF_MEAN: f32 = 2.0 * 0.125_892_54 / std::f32::consts::PI;

/// The constants that differ between the 6176's own 610B and the 1958
/// 610A module (`research/610.md` 8.5).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Voicing {
    /// The five Gain switch positions, dB.
    pub gain_steps: [f32; 5],
    /// The microphone pad, dB.
    pub pad_db: f32,
    /// How much of each Gain step is a feedback change rather than an
    /// attenuation change: 1 would make the distortion at a fixed output
    /// rise 20 dB from −10 to +10, 0 would make the switch clean.
    pub kappa: f32,
    /// Input stage: bias offset (the asymmetry that makes the second
    /// harmonic dominate), knee sharpness, and the amplitude at which it
    /// saturates.
    pub b1: f32,
    pub n1: f32,
    pub x1: f32,
    /// Output stage: the same three, with a harder knee.
    pub b2: f32,
    pub n2: f32,
    pub x2: f32,
    /// Input transformer: high-pass corner and quality, and the top-end
    /// roll-off.
    pub in_hp_hz: f32,
    pub in_hp_q: f32,
    pub in_lp_hz: f32,
    /// The output transformer's own low-frequency roll-off. The research
    /// gives this the same corner as the flux integrator and then asserts
    /// the published +0 / −1 dB at 20 Hz from it alone, forgetting the
    /// second-order high-pass it specifies three blocks earlier; cascaded,
    /// the two are −2.47 dB at 20 Hz and the design as written cannot meet
    /// its own response figure. Separating the roll-off from the flux
    /// corner fixes it without changing how the core saturates.
    pub out_hp_hz: f32,
    /// Output transformer: the flux corner, the flux the core can carry,
    /// and the top-end roll-off.
    pub flux_hz: f32,
    pub flux_sat: f32,
    pub out_lp_hz: f32,
    /// Depth of the self-rectification that shifts the operating point
    /// after a loud passage.
    pub sag: f32,
    /// The A module has one corner per band instead of three.
    pub fixed_eq: bool,
}

/// The 610B: the preamp in the 6176 and the 2-610.
pub const B: Voicing = Voicing {
    gain_steps: GAIN_STEPS_DB,
    pad_db: -15.0,
    kappa: 0.8,
    b1: 0.12,
    n1: 2.5,
    // Tuned so that a microphone at +30 dB with the Gain switch at +10
    // reaches 1 % at the −12 dBu equivalent, the SOLO/610's figure, and
    // measured there rather than assumed: at 12.9 it gave 1.9 %.
    x1: 24.5,
    b2: 0.08,
    n2: 4.0,
    // The hard ceiling sits just above the published +20 dBm maximum.
    x2: 0.8,
    in_hp_hz: 7.0,
    in_hp_q: 0.6,
    in_lp_hz: 40_000.0,
    out_hp_hz: 6.0,
    flux_hz: 10.0,
    flux_sat: 0.085,
    out_lp_hz: 50_000.0,
    sag: 0.3,
    fixed_eq: false,
};

/// The 610A: the 1958 console module, darker and dirtier.
pub const A: Voicing = Voicing {
    gain_steps: GAIN_STEPS_A_DB,
    pad_db: -20.0,
    kappa: 0.7,
    b1: 0.20,
    n1: 2.5,
    x1: 24.5,
    b2: 0.12,
    n2: 3.5,
    x2: 0.63,
    in_hp_hz: 10.0,
    in_hp_q: 0.6,
    in_lp_hz: 22_000.0,
    out_hp_hz: 9.0,
    flux_hz: 16.0,
    flux_sat: 0.0425,
    out_lp_hz: 25_000.0,
    sag: 0.5,
    fixed_eq: true,
};

/// The voicing for a `pre_voice` index.
pub const fn voicing(i: usize) -> Voicing {
    if i == 0 { B } else { A }
}

/// What each input setting does to the response, on top of its gain: the
/// microphone taps load the transformer differently and the instrument
/// input loads a pickup differently, and `research/610.md` 8.4 gives the
/// figures. Without these the selector is "merely a label", which section 6
/// of that document names as one of the ways an emulation is judged wrong.
/// `(low shelf Hz, dB, high shelf Hz, dB)`; **estimates** from those
/// figures.
pub const INPUT_TILT: [(f32, f32, f32, f32); 5] = [
    // Line: the reference, flat.
    (0.0, 0.0, 0.0, 0.0),
    // Mic 500: a quarter of the impedance, so a little off both ends.
    (50.0, -0.5, 10_000.0, -0.7),
    // Mic 2.0K: the reference microphone tap.
    (0.0, 0.0, 0.0, 0.0),
    // Hi-Z 47K: the lower load damps a passive pickup's resonance.
    (0.0, 0.0, 5000.0, -1.0),
    // Hi-Z 2.2M: high enough to leave it alone.
    (0.0, 0.0, 0.0, 0.0),
];

/// How far the nonlinear blocks are oversampled.
///
/// The research asks for two, and the aliasing is not the reason this is
/// four. Its section 8.6 is right that the fix for a stage deep into its
/// knee is antiderivative anti-aliasing rather than a bigger factor: that
/// bought 24 dB where doubling from two to four bought two, and with it in
/// place the two factors are within 2 dB of each other at any sane setting.
///
/// The reason is the response. First-order anti-aliasing averages the
/// shaper across the segment between two samples, and that average is
/// itself a mild low-pass whose droop grows with frequency and with the
/// size of the step. At 2x it costs 1.7 dB at 12 kHz, against a published
/// +0 / −1 dB from 20 Hz to 20 kHz; at 4x the segments are half as long and
/// the droop a quarter as deep, and the response fits inside the
/// specification. The cost is the resamplers' round trip, which the plug-in
/// reports to the host.
pub const OVERSAMPLE: usize = 4;

/// The low cut of the LA-6176 and SOLO/610, Hz.
pub const HPF_HZ: f32 = 75.0;
/// The 1176 section's 600 Ω input loading rolls the top off a little.
pub const LOAD_600_LP_HZ: f32 = 28_000.0;
/// Self-rectification follower times, seconds.
pub const SAG_UP_S: f32 = 0.005;
pub const SAG_DOWN_S: f32 = 0.2;
/// Smoothing of the Level knob, seconds.
pub const LEVEL_SMOOTH_S: f32 = 0.005;

/// `S(v) = v / (1 + |v|^n)^(1/n)`: the tanh-like family the tube stages
/// are built from, and its slope at the origin's bias point.
#[inline]
pub fn s_curve(v: f32, n: f32) -> f32 {
    let a = v.abs().powf(n);
    v / (1.0 + a).powf(1.0 / n)
}

/// `S'(v) = (1 + |v|^n)^(−(n+1)/n)`.
#[inline]
pub fn s_slope(v: f32, n: f32) -> f32 {
    let a = v.abs().powf(n);
    (1.0 + a).powf(-(n + 1.0) / n)
}

/// One tube stage: `S(v + b) − S(b)`, normalised to unity small-signal
/// gain so the bias only bends the curve and does not change the gain.
#[inline]
pub fn tube(v: f32, b: f32, n: f32) -> f32 {
    (s_curve(v + b, n) - s_curve(b, n)) / s_slope(b, n)
}

/// Per-channel state.
#[derive(Clone, Default)]
struct Channel {
    in_hp: Hp2,
    /// What the input selector does to the response (see [`INPUT_TILT`]).
    tilt_lo: Shelf,
    tilt_hi: Shelf,
    in_lp: Lp1,
    lf: Shelf,
    hf: Shelf,
    flux: Lp1,
    out_hp: Hp1,
    out_lp: Lp1,
    load_lp: Lp1,
    cut: Hp2,
    /// Blocks 3 to 8 run at twice the rate below 88.2 kHz, as
    /// `research/610.md` 8.6 asks: two tube stages and a saturating core
    /// fold badly otherwise, and a 15 kHz tone into a hot microphone
    /// setting measured an alias at −9 dBFS without them.
    /// The two tube stages, integrated across each segment rather than
    /// sampled through (see [`super::adaa`]).
    shape_in: Adaa,
    shape_out: Adaa,
    up: Upsampler,
    up2: Upsampler,
    down: Downsampler,
    down2: Downsampler,
    dry: DryDelay,
    /// Follower behind the self-rectification.
    sag: f32,
    /// Rectified average of the stage output, for the PRE meter.
    out_abs: f32,
}

impl Channel {
    fn reset(&mut self) {
        self.in_hp.reset();
        self.tilt_lo.reset();
        self.tilt_hi.reset();
        self.shape_in.reset();
        self.shape_out.reset();
        self.up.reset();
        self.up2.reset();
        self.down.reset();
        self.down2.reset();
        self.dry.reset();
        self.in_lp.reset();
        self.lf.reset();
        self.hf.reset();
        self.flux.reset();
        self.out_hp.reset();
        self.out_lp.reset();
        self.load_lp.reset();
        self.cut.reset();
        self.sag = 0.0;
        self.out_abs = 0.0;
    }
}

/// The stereo 610 stage.
pub struct Stage {
    sr: f32,
    settings: Settings,
    v: Voicing,
    ch: [Channel; 2],
    level: Smooth,
    /// Everything ahead of the input stage, as one linear gain.
    front_gain: f32,
    /// The input stage's closed-loop gain and its relative feedback.
    a_in: f32,
    f_rel: f32,
    level_lin: f32,
    polarity: f32,
    sag_up: f32,
    sag_down: f32,
    vu: Vu,
    /// 2x below 88.2 kHz.
    oversample: bool,
    /// The PRE meter reading of the last block, in dB against 0 VU.
    pre_vu_db: f32,
    /// Peak drive of the input stage over the last block, 0..1-ish.
    drive: f32,
}

impl Stage {
    pub fn new(sr: f32) -> Self {
        let s = Settings::default();
        let mut st = Stage {
            sr,
            settings: s,
            v: voicing(s.voice),
            ch: [Channel::default(), Channel::default()],
            level: Smooth::default(),
            front_gain: 1.0,
            a_in: 1.0,
            f_rel: 1.0,
            level_lin: 1.0,
            polarity: 1.0,
            sag_up: 0.0,
            sag_down: 0.0,
            vu: Vu::default(),
            oversample: sr < 88_200.0,
            pre_vu_db: -60.0,
            drive: 0.0,
        };
        st.set_sample_rate(sr);
        st.apply(&s);
        st.rebuild(&s);
        st.reset();
        st
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sr = sr;
        self.oversample = sr < 88_200.0;
        let sr_p = if self.oversample {
            sr * OVERSAMPLE as f32
        } else {
            sr
        };
        self.level.set(sr, LEVEL_SMOOTH_S);
        // The self-rectification follower lives inside the oversampled
        // region, so its coefficients belong to that rate.
        self.sag_up = 1.0 - (-1.0 / (SAG_UP_S * sr_p)).exp();
        self.sag_down = 1.0 - (-1.0 / (SAG_DOWN_S * sr_p)).exp();
        self.vu.set_sample_rate(sr, 1.0 / sr);
        let s = self.settings;
        self.rebuild(&s);
        self.reset();
    }

    pub fn reset(&mut self) {
        for ch in &mut self.ch {
            ch.reset();
        }
        self.level.snap(self.level_lin);
        self.vu.reset();
        self.pre_vu_db = -60.0;
        self.drive = 0.0;
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    pub fn voicing(&self) -> &Voicing {
        &self.v
    }

    /// Apply a snapshot; `true` when anything changed.
    pub fn configure(&mut self, s: &Settings) -> bool {
        if *s == self.settings {
            return false;
        }
        let rebuild = s.voice != self.settings.voice
            || s.input != self.settings.input
            || s.lf_freq != self.settings.lf_freq
            || s.lf_gain != self.settings.lf_gain
            || s.hf_freq != self.settings.hf_freq
            || s.hf_gain != self.settings.hf_gain
            || s.hpf != self.settings.hpf
            || s.load != self.settings.load;
        self.apply(s);
        if rebuild {
            self.rebuild(s);
        }
        true
    }

    fn apply(&mut self, s: &Settings) {
        self.v = voicing(s.voice);
        let g_db = self.v.gain_steps[s.gain.min(4)];
        let pad = if s.pad && matches!(s.input, 1 | 2) {
            self.v.pad_db
        } else {
            0.0
        };
        let front_db = INPUT_OFFSET_DB[s.input.min(4)] + pad;
        self.front_gain = 10f32.powf(front_db / 20.0);
        self.a_in = 10f32.powf(g_db / 20.0);
        self.f_rel = 10f32.powf(-self.v.kappa * g_db / 20.0);
        self.level_lin = 10f32.powf(level_to_db(s.level) / 20.0);
        self.polarity = if s.polarity { -1.0 } else { 1.0 };
        self.settings = *s;
    }

    fn rebuild(&mut self, s: &Settings) {
        let v = voicing(s.voice);
        // The tables belong to the exponents, so they are rebuilt only when
        // the voicing changes them; this runs off the audio thread.
        for ch in &mut self.ch {
            ch.shape_in = Adaa::new(v.n1);
            ch.shape_out = Adaa::new(v.n2);
        }
        let (lf_hz, hf_hz) = if v.fixed_eq {
            (LF_FREQ_HZ[1], HF_FREQ_HZ[2])
        } else {
            (LF_FREQ_HZ[s.lf_freq.min(2)], HF_FREQ_HZ[s.hf_freq.min(2)])
        };
        // Blocks 3 to 8 run at the oversampled rate, so their sections are
        // designed there; the input transformer's high-pass and everything
        // after the resampler stay at the base rate.
        let sr_p = if self.oversample {
            self.sr * OVERSAMPLE as f32
        } else {
            self.sr
        };
        let lf = Shelf::new(sr_p, lf_hz, SHELF_GAIN_DB[s.lf_gain.min(10)], true);
        let hf = Shelf::new(sr_p, hf_hz, SHELF_GAIN_DB[s.hf_gain.min(10)], false);
        let in_hp = Hp2::new(self.sr, v.in_hp_hz, v.in_hp_q);
        let (tlo_hz, tlo_db, thi_hz, thi_db) = INPUT_TILT[s.input.min(4)];
        let tilt_lo = Shelf::new(self.sr, tlo_hz.max(1.0), tlo_db, true);
        let tilt_hi = Shelf::new(self.sr, thi_hz.max(1.0), thi_db, false);
        let cut = if s.hpf {
            Hp2::new(self.sr, HPF_HZ, std::f32::consts::FRAC_1_SQRT_2)
        } else {
            Hp2::bypassed()
        };
        // The transformer roll-offs sit above the audio band and often
        // above Nyquist. A one-pole is stable for any corner, and clamping
        // them to Nyquist would make every voicing identical, so they are
        // left where the circuit puts them and simply warp.
        for ch in &mut self.ch {
            ch.lf.set_from(&lf);
            ch.hf.set_from(&hf);
            ch.in_hp.set_from(&in_hp);
            ch.tilt_lo.set_from(&tilt_lo);
            ch.tilt_hi.set_from(&tilt_hi);
            ch.cut.set_from(&cut);
            ch.in_lp.set(v.in_lp_hz, sr_p);
            ch.out_lp.set(v.out_lp_hz, sr_p);
            ch.flux.set(v.flux_hz, sr_p);
            ch.out_hp.set(v.out_hp_hz, sr_p);
            ch.load_lp.set(
                if s.load == 1 {
                    LOAD_600_LP_HZ
                } else {
                    self.sr * 4.0
                },
                self.sr,
            );
        }
    }

    /// The round trip of the resamplers, when they are running.
    pub fn latency(&self) -> usize {
        if self.oversample { 2 * LATENCY } else { 0 }
    }

    /// Small-signal gain of the whole stage in dB, which is what the
    /// 6176's gain structure is set by.
    pub fn small_signal_db(&self) -> f32 {
        20.0 * (self.front_gain * self.a_in * self.level_lin)
            .max(1e-9)
            .log10()
    }

    /// Process one stereo block in place. Real-time safe.
    pub fn process_block(&mut self, l: &mut [f32], r: &mut [f32]) {
        let n = l.len().min(r.len());
        if n == 0 {
            return;
        }
        if self.settings.bypass {
            // Keep the meter honest and the filters warm, but pass through.
            for ch in &mut self.ch {
                ch.in_hp.reset();
            }
            self.vu.step(0.0, n as f32 / self.sr);
            self.pre_vu_db = -60.0;
            self.drive = 0.0;
            return;
        }
        let v = self.v;
        let oversample = self.oversample;
        let x1_scale = v.x1 * self.f_rel;
        let mut drive = 0.0f32;
        let mut abs_sum = 0.0f32;

        for i in 0..n {
            let level = self.level.process(self.level_lin);
            for c in 0..2 {
                let x = if c == 0 { l[i] } else { r[i] };
                let ch = &mut self.ch[c];

                // Input select, its response, the pad and the input
                // transformer's high-pass, all at the base rate.
                let mut y = x * self.front_gain;
                y = ch.tilt_lo.process(y);
                y = ch.tilt_hi.process(y);
                y = ch.in_hp.process(y);

                // Everything from here to the output transformer is where
                // the nonlinearities are, so it runs at twice the rate.
                let mut subs = [0.0f32; OVERSAMPLE];
                let count = if oversample {
                    let half = ch.up.process(y);
                    let a = ch.up2.process(half[0]);
                    let b = ch.up2.process(half[1]);
                    subs[0] = a[0];
                    subs[1] = a[1];
                    subs[2] = b[0];
                    subs[3] = b[1];
                    OVERSAMPLE
                } else {
                    subs[0] = y;
                    1
                };
                let mut shaped = [0.0f32; OVERSAMPLE];
                for k in 0..count {
                    let mut y = ch.in_lp.process(subs[k]);

                    // Input tube stage: the Gain switch raises the gain and
                    // lowers the feedback, so the drive rises twice over.
                    let u = y * self.a_in / x1_scale;
                    let au = u.abs();
                    if au > drive {
                        drive = au;
                    }
                    let a = if au > ch.sag {
                        self.sag_up
                    } else {
                        self.sag_down
                    };
                    ch.sag = flush(ch.sag + a * (au - ch.sag));
                    let bias = v.b1 * (1.0 + v.sag * ch.sag.min(4.0));
                    y = x1_scale * ch.shape_in.process(u, bias);

                    // Level, then the shelves (which sit in the output
                    // stage's feedback loop, so a boosted band drives it
                    // harder).
                    let mut y2 = y * level;
                    y2 = ch.lf.process(y2);
                    y2 = ch.hf.process(y2);

                    // Output tube stage.
                    let y3 = v.x2 * ch.shape_out.process(y2 / v.x2, v.b2);

                    // Output transformer: the core carries only so much
                    // flux, and what it cannot carry never reaches the
                    // secondary.
                    let phi = ch.flux.process(y3);
                    let excess = phi - v.flux_sat * s_curve(phi / v.flux_sat, 4.0);
                    let mut y4 = y3 - excess;
                    y4 = ch.out_hp.process(y4);
                    shaped[k] = ch.out_lp.process(y4);
                }
                let mut y4 = if oversample {
                    let a = ch.down2.process([shaped[0], shaped[1]]);
                    let b = ch.down2.process([shaped[2], shaped[3]]);
                    ch.down.process([a, b])
                } else {
                    shaped[0]
                };
                y4 = ch.load_lp.process(y4);
                y4 = ch.cut.process(y4) * self.polarity;

                ch.out_abs += y4.abs();
                if c == 0 { l[i] = y4 } else { r[i] = y4 }
            }
        }

        for ch in &mut self.ch {
            abs_sum = abs_sum.max(ch.out_abs / n as f32);
            ch.out_abs = 0.0;
        }
        let dt = n as f32 / self.sr;
        let needle = self.vu.step(abs_sum, dt);
        self.pre_vu_db = 20.0 * (needle / VU_REF_MEAN).max(1e-4).log10();
        self.drive = drive.min(8.0);
    }

    /// What the PRE meter reads, in dB against 0 VU (+4 dBm at the line
    /// output), for the last block.
    pub fn pre_vu_db(&self) -> f32 {
        self.pre_vu_db
    }

    /// Peak drive of the input tube stage over the last block: 1 is the
    /// stage's own saturation point.
    pub fn drive(&self) -> f32 {
        self.drive
    }

    /// The static output level in dBFS for a sine at `in_db`, used to
    /// place the compressor's transfer curve behind the preamp. Runs the
    /// same chain without the filters, which are flat in the middle of the
    /// band.
    pub fn static_out_db(&self, in_db: f32) -> f32 {
        let v = self.v;
        let amp = 10f32.powf(in_db / 20.0) * self.front_gain;
        let x1_scale = v.x1 * self.f_rel;
        let y1 = x1_scale * tube(amp * self.a_in / x1_scale, v.b1, v.n1);
        let y2 = y1 * self.level_lin;
        let y3 = v.x2 * tube(y2 / v.x2, v.b2, v.n2);
        20.0 * y3.abs().max(1e-9).log10()
    }
}

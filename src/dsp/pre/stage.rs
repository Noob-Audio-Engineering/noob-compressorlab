//! The 610 stage: two tube gain stages with the Gain switch trading
//! attenuation for feedback between them, the shelving equaliser in the
//! second stage's feedback loop, and a transformer at each end
//! (`research/610.md` section 8).
//!
//! The valve and the transformers are shared components rather than local
//! code: [`Triode`] is `noob-electrical-components-small-signal-triode` and
//! [`Rolloff`] and [`Core`] are `noob-electrical-components-transformer`.
//! What stays here is the machine around them — which voicing picks which
//! numbers, the feedback the Gain switch trades against attenuation, the
//! supply sag, the oversampling, and every filter the parts are realised
//! through.
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
pub use noob_electrical_components::transformer::{Core, Rolloff};

/// The valve, from the component crate: `S(v) = v / (1 + |v|^n)^(1/n)`, the
/// tanh-like family both gain stages are built from, and the stage law
/// `T(v) = (S(v + b) − S(b)) / S'(b)` that sits on it.
///
/// Re-exported under the 610's own vocabulary because this file and
/// [`Adaa`] both work in it, and because the component crate is the one
/// copy of the law: [`Adaa`] integrates exactly what [`tube`] evaluates,
/// and a second copy here would let the two drift apart.
///
/// It is the **small-signal** triode and not the remote-cutoff valve a
/// variable-mu unit uses. This law's bias sets the asymmetry of the curve
/// and can never set its gain, so it cannot be a gain element at all; the
/// component's own documentation says why the two are different parts.
pub use noob_electrical_components::small_signal_triode::{
    Triode, s_curve, s_slope, transfer as tube,
};

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
    /// Input stage: the valve, and the amplitude at which it saturates.
    ///
    /// The valve is the bias offset (the asymmetry that makes the second
    /// harmonic dominate) and the knee sharpness, which are the component's
    /// two numbers. `x1` is not the valve's: it is where this machine's
    /// gain structure puts the stage's saturation point, tuned against a
    /// published distortion figure, and the Gain switch moves it further.
    pub in_stage: Triode,
    pub x1: f32,
    /// Output stage: the same, with a harder knee.
    pub out_stage: Triode,
    pub x2: f32,
    /// Input transformer: its low-frequency roll-off, and the top-end
    /// roll-off.
    ///
    /// The research gives the two transformer roll-offs as 40 kHz and
    /// 50 kHz and says in as many words that they were "chosen to keep the
    /// B response within +0 / −1 dB from 20 Hz to 20 kHz". They never did:
    /// two first-order poles there spend 1.61 dB of that 1 dB budget at
    /// 20 kHz between them, before the anti-aliasing and the resamplers
    /// have taken their own share. So these are estimates picked for a
    /// stated purpose, by arithmetic that does not reach it, and the
    /// purpose is the better guide. They now sit where the published
    /// response puts them with the rest of the chain accounted for, which
    /// is what the research was trying to do. The A voicing keeps its own
    /// much lower corners: it is the 1958 module, it is meant to be darker,
    /// and the +0 / −1 dB figure is the 6176's rather than its.
    ///
    /// The top-end roll-off stays a bare number here rather than joining
    /// the component: as the paragraph above says, it was fitted to the
    /// whole chain's response with the resamplers accounted for, which
    /// makes it this machine's calibration rather than a property of a
    /// wound part.
    pub in_hp: Rolloff,
    pub in_lp_hz: f32,
    /// The output transformer's own low-frequency roll-off. The research
    /// gives this the same corner as the flux integrator and then asserts
    /// the published +0 / −1 dB at 20 Hz from it alone, forgetting the
    /// second-order high-pass it specifies three blocks earlier; cascaded,
    /// the two are −2.47 dB at 20 Hz and the design as written cannot meet
    /// its own response figure. Separating the roll-off from the flux
    /// corner fixes it without changing how the core saturates.
    pub out_hp: Rolloff,
    /// Output transformer: the core, and the top-end roll-off.
    pub core: Core,
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
    in_stage: Triode::new(0.12, 2.5),
    // Tuned so that a microphone at +30 dB with the Gain switch at +10
    // reaches 1 % at the −12 dBu equivalent, the SOLO/610's figure, and
    // measured there rather than assumed: at 12.9 it gave 1.9 %.
    x1: 24.5,
    out_stage: Triode::new(0.08, 4.0),
    // The hard ceiling sits just above the published +20 dBm maximum.
    x2: 0.8,
    in_hp: Rolloff::two_pole(7.0, 0.6),
    in_lp_hz: 80_000.0,
    out_hp: Rolloff::one_pole(6.0),
    core: Core::new(10.0, 0.085),
    out_lp_hz: 100_000.0,
    sag: 0.3,
    fixed_eq: false,
};

/// The 610A: the 1958 console module, darker and dirtier.
pub const A: Voicing = Voicing {
    gain_steps: GAIN_STEPS_A_DB,
    pad_db: -20.0,
    kappa: 0.7,
    in_stage: Triode::new(0.20, 2.5),
    x1: 24.5,
    out_stage: Triode::new(0.12, 3.5),
    x2: 0.63,
    in_hp: Rolloff::two_pole(10.0, 0.6),
    in_lp_hz: 22_000.0,
    out_hp: Rolloff::one_pole(9.0),
    core: Core::new(16.0, 0.0425),
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

/// The processing rate the stage aims for, Hz.
///
/// What matters to the anti-aliasing droop above is the rate the shaper
/// actually runs at, not the factor, so the factor has to follow the host.
/// This used to be a flag that simply stopped oversampling at and above
/// 88.2 kHz, which quietly made the response *worse* at high rates than at
/// low ones: it dropped the shaper to 88.2 kHz, below even the 2x case this
/// documentation rejects, and measured 4.9 dB down at 20 kHz where 48 kHz
/// was 2.1 dB down. Picking the factor from the rate keeps the shaper
/// between 176 and 192 kHz whatever the host runs at, so the response is
/// the same at every rate and no transformer corner has to be clamped
/// against a low Nyquist.
pub const TARGET_RATE_HZ: f32 = 176_400.0;

/// How far to oversample at `sr`: 4, 2 or 1, whichever first reaches
/// [`TARGET_RATE_HZ`].
pub fn oversample_factor(sr: f32) -> usize {
    if sr >= TARGET_RATE_HZ {
        1
    } else if sr * 2.0 >= TARGET_RATE_HZ {
        2
    } else {
        OVERSAMPLE
    }
}

/// The low cut of the LA-6176 and SOLO/610, Hz.
pub const HPF_HZ: f32 = 75.0;
/// The 1176 section's 600 Ω input loading rolls the top off a little.
pub const LOAD_600_LP_HZ: f32 = 28_000.0;
/// Self-rectification follower times, seconds.
pub const SAG_UP_S: f32 = 0.005;
pub const SAG_DOWN_S: f32 = 0.2;
/// Smoothing of the Level knob, seconds.
pub const LEVEL_SMOOTH_S: f32 = 0.005;

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
    factor: usize,
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
            factor: oversample_factor(sr),
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
        self.factor = oversample_factor(sr);
        let sr_p = sr * self.factor as f32;
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
            ch.shape_in = Adaa::new(v.in_stage.knee);
            ch.shape_out = Adaa::new(v.out_stage.knee);
        }
        let (lf_hz, hf_hz) = if v.fixed_eq {
            (LF_FREQ_HZ[1], HF_FREQ_HZ[2])
        } else {
            (LF_FREQ_HZ[s.lf_freq.min(2)], HF_FREQ_HZ[s.hf_freq.min(2)])
        };
        // Blocks 3 to 8 run at the oversampled rate, so their sections are
        // designed there; the input transformer's high-pass and everything
        // after the resampler stay at the base rate.
        let sr_p = self.sr * self.factor as f32;
        let lf = Shelf::new(sr_p, lf_hz, SHELF_GAIN_DB[s.lf_gain.min(10)], true);
        let hf = Shelf::new(sr_p, hf_hz, SHELF_GAIN_DB[s.hf_gain.min(10)], false);
        let in_hp = Hp2::new(self.sr, v.in_hp.hz, v.in_hp.q);
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
            ch.flux.set(v.core.integrator_hz, sr_p);
            ch.out_hp.set(v.out_hp.hz, sr_p);
            // The 600 Ω roll-off is a real in-band effect, about 2 dB down
            // at 20 kHz, but its corner sits above Nyquist at 48 kHz, so it
            // has to be designed at the oversampled rate to be placed at
            // all. The 15 kΩ position is no roll-off rather than a distant
            // one, which now matters: two corners past Nyquist would
            // prewarp to the same place and the switch would do nothing.
            if s.load == 1 {
                ch.load_lp.set(LOAD_600_LP_HZ, sr_p);
            } else {
                ch.load_lp.bypass();
            }
        }
    }

    /// The round trip of the resamplers, when they are running.
    pub fn latency(&self) -> usize {
        // One resampler pair per doubling.
        match self.factor {
            4 => 2 * LATENCY,
            2 => LATENCY,
            _ => 0,
        }
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
        let factor = self.factor;
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
                let count = match factor {
                    4 => {
                        let half = ch.up.process(y);
                        let a = ch.up2.process(half[0]);
                        let b = ch.up2.process(half[1]);
                        subs[0] = a[0];
                        subs[1] = a[1];
                        subs[2] = b[0];
                        subs[3] = b[1];
                        4
                    }
                    2 => {
                        let a = ch.up.process(y);
                        subs[0] = a[0];
                        subs[1] = a[1];
                        2
                    }
                    _ => {
                        subs[0] = y;
                        1
                    }
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
                    let bias = v.in_stage.bias * (1.0 + v.sag * ch.sag.min(4.0));
                    y = x1_scale * ch.shape_in.process(u, bias);

                    // Level, then the shelves (which sit in the output
                    // stage's feedback loop, so a boosted band drives it
                    // harder).
                    let mut y2 = y * level;
                    y2 = ch.lf.process(y2);
                    y2 = ch.hf.process(y2);

                    // Output tube stage.
                    let y3 = v.x2 * ch.shape_out.process(y2 / v.x2, v.out_stage.bias);

                    // Output transformer: the core carries only so much
                    // flux, and what it cannot carry never reaches the
                    // secondary.
                    let phi = ch.flux.process(y3);
                    let mut y4 = v.core.through(y3, phi);
                    y4 = ch.out_hp.process(y4);
                    // The 1176 section's input loading sits after the
                    // output transformer, and inside the oversampled block
                    // because its corner is above Nyquist at 48 kHz.
                    shaped[k] = ch.load_lp.process(ch.out_lp.process(y4));
                }
                let mut y4 = match factor {
                    4 => {
                        let a = ch.down2.process([shaped[0], shaped[1]]);
                        let b = ch.down2.process([shaped[2], shaped[3]]);
                        ch.down.process([a, b])
                    }
                    2 => ch.down.process([shaped[0], shaped[1]]),
                    _ => shaped[0],
                };
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
        let y1 = x1_scale * v.in_stage.shape(amp * self.a_in / x1_scale);
        let y2 = y1 * self.level_lin;
        let y3 = v.x2 * v.out_stage.shape(y2 / v.x2);
        20.0 * y3.abs().max(1e-9).log10()
    }
}

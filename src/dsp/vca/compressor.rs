//! The Distressor engine: a feedback VCA compressor computed in the dB
//! domain (`research/Distressor.md` section 7).
//!
//! ```text
//! in ─► ×g_in ─► VCA (×10^(G/20)) ─┬─► Dist 2 / Dist 3 ─► audio HP ─► ×g_out ─► mix ─► out
//!                                  │
//!                                  └─► band emphasis ─► SC high-pass ─► peak detector (dB)
//!                                        ─► per-ratio soft-knee curve ─► ballistics ─► G
//! ```
//!
//! The detector tap is **after** the gain cell, which is what makes the
//! ratios behave the way the manual describes: the loop halves the
//! effective time constants and softens every knee, and the curve table
//! below only has to describe the static shape. Everything runs per sample;
//! nothing allocates.

use super::filters::{BesselHp3, Biquad, OnePole, OnePoleHp, coefficient, flush};
use super::{
    HEADROOM_DEFAULT_DB, LinkMode, Ratio, Settings, attack_seconds, knob_to_db, release_seconds,
};
use crate::dsp::fet::oversample::{Downsampler, DryDelay, LATENCY, Upsampler};

/// Points in the static transfer curve (the lab draws every model on the
/// same grid).
pub const TRANSFER_POINTS: usize = 128;

/// How a position lets go of the gain reduction (`research/Distressor.md`
/// 7.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReleaseShape {
    /// One pole at the knob's time constant.
    Standard,
    /// The 10:1 position: a fast first stage, then a trap-like tail that
    /// stretches towards 20 s after long, deep compression.
    Opto,
    /// 20:1: quicker than the knob asks for.
    Fast,
    /// Nuke: fast at first, then slowing (logarithmic).
    Log,
}

/// The static curve and timing of one ratio button. Every number here is
/// an **estimate** from `research/Distressor.md` 7.4, tuned against the
/// tests in section 8 of the same document; the sources give the shapes and
/// the relative order, not the constants.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Curve {
    /// Threshold in dBFS at the default headroom.
    pub threshold_db: f32,
    /// Total knee width in dB (the 2:1 knee is the widest thing in the box).
    pub knee_db: f32,
    /// Effective slope, which the sources put above the printed label for
    /// every position from 2:1 to 10:1.
    pub ratio: f32,
    /// Attack multiplier: "fast attack at 2:1 is not the same as fast
    /// attack at 4:1".
    pub k_attack: f32,
    /// Release multiplier, before the shape below.
    pub k_release: f32,
    pub release: ReleaseShape,
}

/// The eight positions.
///
/// **A contradiction in the research, resolved here.** Its section 4.2
/// orders the thresholds so that higher ratios engage *earlier*, and its
/// test plan (8.1) asserts the reverse, that the threshold rises from 2:1
/// through 20:1. They cannot both hold. This table follows 4.2, because it
/// is the section that reasons about the circuit rather than restating a
/// summary, and because it matches what the box is known for: at 20:1 it
/// grabs sooner than at 2:1, not later. The ordering is pinned by a test
/// rather than left to chance.
pub const fn curve(r: Ratio) -> Curve {
    match r {
        // No gain reduction at all: the distortion modes on their own.
        Ratio::R1 => Curve {
            threshold_db: 1000.0,
            knee_db: 0.0,
            ratio: 1.0,
            k_attack: 1.0,
            k_release: 1.0,
            release: ReleaseShape::Standard,
        },
        Ratio::R2 => Curve {
            threshold_db: -6.0,
            knee_db: 30.0,
            ratio: 2.3,
            k_attack: 1.4,
            k_release: 1.2,
            release: ReleaseShape::Standard,
        },
        Ratio::R3 => Curve {
            threshold_db: -8.0,
            knee_db: 24.0,
            ratio: 3.3,
            k_attack: 1.25,
            k_release: 1.1,
            release: ReleaseShape::Standard,
        },
        Ratio::R4 => Curve {
            threshold_db: -12.0,
            knee_db: 12.0,
            ratio: 4.5,
            k_attack: 1.1,
            k_release: 1.0,
            release: ReleaseShape::Standard,
        },
        Ratio::R6 => Curve {
            threshold_db: -14.0,
            knee_db: 10.0,
            ratio: 6.5,
            k_attack: 1.0,
            k_release: 1.0,
            release: ReleaseShape::Standard,
        },
        Ratio::R10 => Curve {
            threshold_db: -16.0,
            knee_db: 8.0,
            ratio: 10.0,
            k_attack: 1.6,
            k_release: 1.0,
            release: ReleaseShape::Opto,
        },
        Ratio::R20 => Curve {
            threshold_db: -18.0,
            knee_db: 3.0,
            ratio: 20.0,
            k_attack: 0.8,
            k_release: 0.6,
            release: ReleaseShape::Fast,
        },
        Ratio::Nuke => Curve {
            // Below 20:1's, so the ordering holds all the way up: the
            // research puts Nuke at −16, which would have it engage nearly
            // three decibels after the ratio below it.
            threshold_db: -19.0,
            knee_db: 1.5,
            ratio: 40.0,
            k_attack: 0.7,
            k_release: 1.0,
            release: ReleaseShape::Log,
        },
    }
}

/// British mode's curve: a raised threshold and a slope in the 10 to 20
/// range, replacing whatever the ratio switch says
/// (`research/Distressor.md` 7.4a). **Estimate.**
pub const BRITISH: Curve = Curve {
    threshold_db: -8.0,
    knee_db: 6.0,
    ratio: 14.0,
    k_attack: 0.9,
    k_release: 0.6,
    release: ReleaseShape::Standard,
};

/// Constants of the model that are not per ratio. All **estimates** from
/// `research/Distressor.md` 7.3, 7.5 and 7.6 unless a measurement is named.
mod k {
    /// Overall scale on the knob's time constants. The loop normalisation
    /// in `ballistics` already gives the closed loop the knob's own time
    /// constant, which is what the published 50 µs and 50 ms figures
    /// describe, so this stays at 1 (7.3).
    pub const FEEDBACK_TIME: f32 = 1.0;
    /// How strongly an overshoot shortens the attack.
    pub const OVERSHOOT_SCALE_DB: f32 = 12.0;
    /// The overshoot at which the knob's own time constant applies. Without
    /// this the factor was always below one, so every attack came out
    /// faster than the map said and the published 30 ms at the top of the
    /// knob measured 17.75 ms. Now the map is what a gesture of this depth
    /// gets, and program dependence swings either side of it.
    pub const OVERSHOOT_REF_DB: f32 = 10.0;
    /// Bounds on that factor, so neither a huge transient nor a whisper can
    /// take the attack somewhere silly.
    pub const OVERSHOOT_FLOOR: f32 = 0.35;
    pub const OVERSHOOT_CEIL: f32 = 2.0;
    /// 10:1 release, first stage (seconds).
    pub const OPTO_FAST_S: f32 = 0.06;
    /// 10:1 release, tail with no history (seconds).
    pub const OPTO_SLOW_S: f32 = 0.5;
    /// 10:1 release, tail after long deep compression (seconds).
    pub const OPTO_SLOW_MAX_S: f32 = 20.0;
    /// How fast the 10:1 tail's memory charges and forgets (seconds).
    pub const OPTO_MEM_UP_S: f32 = 3.0;
    pub const OPTO_MEM_DOWN_S: f32 = 12.0;
    /// Gain reduction, in dB, that charges the memory fully.
    pub const OPTO_MEM_GR_DB: f32 = 12.0;
    /// Nuke's release: the knob's time constant is scaled by this at the
    /// start of the recovery and by the second one at the end.
    pub const NUKE_FAST: f32 = 0.25;
    pub const NUKE_SLOW: f32 = 3.0;
    /// British mode's onset lag: the detector charges through it before
    /// gain reduction starts (seconds).
    pub const BRITISH_LAG_S: f32 = 0.0015;
    /// British mode's extra distortion drive.
    pub const BRITISH_DRIVE: f32 = 1.6;
    /// Side-chain high-pass of the Detector switch's HP position, Hz.
    pub const DETECTOR_HP_HZ: f32 = 100.0;
    /// Band emphasis: a peaking boost into the side-chain.
    pub const BAND_HZ: f32 = 6000.0;
    pub const BAND_Q: f32 = 1.0;
    pub const BAND_DB: f32 = 8.0;
    /// Audio high-pass: the 80 Hz position is −3 dB at 65 Hz.
    pub const AUDIO_HP_F3_HZ: f32 = 65.0;
    /// Second-harmonic coefficient of Dist 2 at full drive: about 3 % THD.
    pub const DIST2_A2: f32 = 0.03;
    /// Third-harmonic coefficient of Dist 3 at full drive: about 20 % THD.
    pub const DIST3_A3: f32 = 0.2;
    /// Dist 3's smaller second-harmonic term.
    pub const DIST3_A2: f32 = 0.01;
    /// Amplitude that counts as full drive of the generator at the default
    /// headroom (1.0 = 0 dBFS).
    pub const DRIVE_REF: f32 = 1.0;
    /// The VCA's own distortion: well under 0.05 % at nominal level, from
    /// the THAT 2181 curves (7.5).
    pub const VCA_THD: f32 = 0.0025;
    /// THD, in per cent, at which the 1 % lamp lights and the redline lamp
    /// lights.
    pub const LAMP_THD_PCT: f32 = 1.0;
    pub const REDLINE_THD_PCT: f32 = 3.0;
    /// Smoothing of the input and output trims (seconds).
    pub const TRIM_SMOOTH_S: f32 = 0.005;
    /// Level below which a channel counts as silent for the dead-patch
    /// behaviour (linear peak over a block).
    pub const DEAD_PATCH_PEAK: f32 = 1e-4;
    /// Extra generator drive when a mono signal is linked to a silent
    /// channel (the "dead patch" trick, 7.9).
    pub const DEAD_PATCH_DRIVE: f32 = 1.4;
}

/// What the detector needs from the engine while a channel is borrowed.
#[derive(Clone, Copy, Debug)]
struct Ballistics {
    curve: Curve,
    tau_att: f32,
    tau_rel: f32,
    sr: f32,
}

/// Per-channel filters and detector state.
#[derive(Clone, Default)]
struct Channel {
    band: Biquad,
    det_hp: OnePoleHp,
    sc_hp: Biquad,
    audio_hp: BesselHp3,
    british_lag: OnePole,
    /// Gain reduction in dB, ≤ 0, one sample behind the audio (feedback).
    g_db: f32,
    /// Deepest gain reduction of the current gesture, for the release
    /// shapes that depend on how far the recovery has come.
    g_peak_db: f32,
    /// The 10:1 position's memory of how long and how deep it has been
    /// working, 0..1.
    opto_mem: f32,
    /// Running peak of the generator's drive over the block.
    drive_peak: f32,
    /// The harmonic generator is the only nonlinearity in the audio path,
    /// so it is the only thing that needs a higher rate to stay clean
    /// (`research/Distressor.md` 7.10).
    up: Upsampler,
    down: Downsampler,
    /// The dry path, held back by the resamplers' round trip so that a mix
    /// or a bypass does not comb-filter itself.
    dry: DryDelay,
}

impl Channel {
    fn reset(&mut self) {
        self.band.reset();
        self.det_hp.reset();
        self.sc_hp.reset();
        self.audio_hp.reset();
        self.british_lag.snap(-120.0);
        self.up.reset();
        self.down.reset();
        self.dry.reset();
        self.g_db = 0.0;
        self.g_peak_db = 0.0;
        self.opto_mem = 0.0;
        self.drive_peak = 0.0;
    }
}

/// The stereo Distressor.
pub struct Compressor {
    sr: f32,
    settings: Settings,
    ch: [Channel; 2],
    curve: Curve,
    /// Threshold after the headroom shift, dBFS.
    threshold_db: f32,
    g_in: OnePole,
    g_out: OnePole,
    mix: OnePole,
    /// Attack and release coefficients before the program-dependent and
    /// per-ratio factors.
    tau_att: f32,
    tau_rel: f32,
    opto_mem_up: f32,
    opto_mem_down: f32,
    british_lag_a: f32,
    /// Generator scale: smaller means more drive.
    drive_scale: f32,
    /// 2x the generator below 88.2 kHz, as the research asks.
    oversample: bool,
    /// Block telemetry.
    gr_db: f32,
    thd_pct: f32,
    drive: f32,
    dead_patch: bool,
}

impl Compressor {
    pub fn new(sr: f32) -> Self {
        let s = Settings::default();
        let mut c = Compressor {
            sr,
            settings: s,
            ch: [Channel::default(), Channel::default()],
            curve: curve(s.ratio),
            threshold_db: curve(s.ratio).threshold_db,
            g_in: OnePole::new(sr, k::TRIM_SMOOTH_S, 1.0),
            g_out: OnePole::new(sr, k::TRIM_SMOOTH_S, 1.0),
            mix: OnePole::new(sr, k::TRIM_SMOOTH_S, 1.0),
            tau_att: attack_seconds(s.attack),
            tau_rel: release_seconds(s.release),
            opto_mem_up: coefficient(sr, k::OPTO_MEM_UP_S),
            opto_mem_down: coefficient(sr, k::OPTO_MEM_DOWN_S),
            british_lag_a: coefficient(sr, k::BRITISH_LAG_S),
            drive_scale: k::DRIVE_REF,
            oversample: sr < 88_200.0,
            gr_db: 0.0,
            thd_pct: 0.0,
            drive: 0.0,
            dead_patch: false,
        };
        c.set_sample_rate(sr);
        c.apply(&s);
        c.reset();
        c
    }

    /// Retune to `sr` and start from rest.
    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sr = sr;
        self.oversample = sr < 88_200.0;
        self.g_in.set_tau(sr, k::TRIM_SMOOTH_S);
        self.g_out.set_tau(sr, k::TRIM_SMOOTH_S);
        self.mix.set_tau(sr, k::TRIM_SMOOTH_S);
        self.opto_mem_up = coefficient(sr, k::OPTO_MEM_UP_S);
        self.opto_mem_down = coefficient(sr, k::OPTO_MEM_DOWN_S);
        self.british_lag_a = coefficient(sr, k::BRITISH_LAG_S);
        let s = self.settings;
        self.rebuild_filters(&s);
        for ch in &mut self.ch {
            ch.british_lag.set_tau(sr, k::BRITISH_LAG_S);
        }
        self.reset();
    }

    pub fn reset(&mut self) {
        for ch in &mut self.ch {
            ch.reset();
        }
        self.g_in.snap(db_to_lin(knob_to_db(self.settings.input)));
        self.g_out.snap(db_to_lin(knob_to_db(self.settings.output)));
        self.mix.snap(self.settings.mix);
        self.gr_db = 0.0;
        self.thd_pct = 0.0;
        self.drive = 0.0;
        self.dead_patch = false;
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// The resamplers' round trip, when they are running.
    pub fn latency(&self) -> usize {
        if self.oversample { LATENCY } else { 0 }
    }

    /// Apply a snapshot; `true` when anything changed.
    pub fn configure(&mut self, s: &Settings) -> bool {
        if *s == self.settings {
            return false;
        }
        let filters = s.detector != self.settings.detector
            || s.audio != self.settings.audio
            || s.sc_hpf_hz != self.settings.sc_hpf_hz;
        self.apply(s);
        if filters {
            self.rebuild_filters(s);
        }
        true
    }

    fn apply(&mut self, s: &Settings) {
        self.curve = if s.british { BRITISH } else { curve(s.ratio) };
        self.threshold_db = self.curve.threshold_db + (s.headroom_db - HEADROOM_DEFAULT_DB);
        self.tau_att = attack_seconds(s.attack);
        self.tau_rel = release_seconds(s.release);
        // The generator's drive: harder with slow attack, fast release and
        // British mode, softer with more headroom (7.6). **Estimate.**
        let slow_attack = 1.0 + 0.35 * (s.attack / super::KNOB_MAX);
        let fast_release = 1.0 + 0.25 * (1.0 - s.release / super::KNOB_MAX);
        let british = if s.british { k::BRITISH_DRIVE } else { 1.0 };
        let headroom = db_to_lin(s.headroom_db - HEADROOM_DEFAULT_DB);
        self.drive_scale = k::DRIVE_REF * headroom / (slow_attack * fast_release * british);
        self.settings = *s;
    }

    fn rebuild_filters(&mut self, s: &Settings) {
        let band = Biquad::peaking(self.sr, k::BAND_HZ, k::BAND_Q, k::BAND_DB);
        let sc_hp = if s.sc_hpf_hz >= 10.0 {
            Biquad::highpass(self.sr, s.sc_hpf_hz, std::f32::consts::FRAC_1_SQRT_2)
        } else {
            Biquad::identity()
        };
        let audio_hp = if s.audio.hp() {
            BesselHp3::new(self.sr, k::AUDIO_HP_F3_HZ)
        } else {
            BesselHp3::bypassed()
        };
        let det_hz = if s.detector.hp() {
            k::DETECTOR_HP_HZ
        } else {
            0.0
        };
        for ch in &mut self.ch {
            ch.band.set_from(&band);
            ch.sc_hp.set_from(&sc_hp);
            ch.audio_hp.set_from(&audio_hp);
            ch.det_hp.set(det_hz, self.sr);
        }
    }

    /// The curve's target gain in dB (≤ 0) for a side-chain level of
    /// `x_db`, and the local slope of that curve (`research/Distressor.md`
    /// 7.4).
    ///
    /// The detector hears the **compressed** signal, so the law in the loop
    /// is not the feed-forward `1/R − 1`: a loop of slope `s` closes to an
    /// input-to-output ratio of `1 − s`, so the box's printed ratio `R`
    /// needs `s = −(R − 1)` in here. That is the high loop gain the
    /// hardware's control amplifier provides, and it is why a feedback
    /// design can limit at all.
    ///
    /// The knee widths in the table are what a measurement of the finished
    /// box would show, so they are input-referred: the knee starts half a
    /// width below the threshold and the width in the loop is compressed by
    /// the same `(R + 1) / 2` the loop stretches it by.
    #[inline]
    fn target_and_slope(&self, x_db: f32) -> (f32, f32) {
        let c = &self.curve;
        if c.ratio <= 1.0001 {
            return (0.0, 0.0);
        }
        let slope = -(c.ratio - 1.0);
        let onset = self.threshold_db - c.knee_db * 0.5;
        let knee = c.knee_db * 2.0 / (c.ratio + 1.0);
        let d = x_db - onset;
        if d <= 0.0 {
            (0.0, 0.0)
        } else if knee > 1e-6 && d < knee {
            (slope * d * d / (2.0 * knee), slope * d / knee)
        } else {
            (slope * (d - knee * 0.5), slope)
        }
    }

    /// One detector step for a channel: smooth `target` towards the state
    /// with the ballistics of the current position.
    ///
    /// `slope` is the curve's local slope, and the step is divided by
    /// `1 + |slope|` because the loop multiplies it back: with that
    /// normalisation the **closed** loop settles with exactly the time
    /// constant the knob asks for, which is what the published attack and
    /// release figures describe. Without it a fast attack at a high ratio
    /// would be a loop gain of twenty and would ring.
    #[inline]
    fn ballistics(ctx: &Ballistics, ch: &mut Channel, target: f32, slope: f32) -> f32 {
        let loop_gain = 1.0 + slope.abs();
        let g = ch.g_db;
        if target < g {
            // Attacking: a bigger overshoot is caught faster.
            let over = g - target;
            let f = ((1.0 + k::OVERSHOOT_REF_DB / k::OVERSHOOT_SCALE_DB)
                / (1.0 + over / k::OVERSHOOT_SCALE_DB))
                .clamp(k::OVERSHOOT_FLOOR, k::OVERSHOOT_CEIL);
            let tau = ctx.tau_att * ctx.curve.k_attack * f * k::FEEDBACK_TIME;
            let a = coefficient(ctx.sr, tau) / loop_gain;
            ch.g_db = flush(g + a * (target - g));
            if ch.g_db < ch.g_peak_db {
                ch.g_peak_db = ch.g_db;
            }
        } else {
            let tau = Self::release_tau(ctx, ch);
            let a = coefficient(ctx.sr, tau) / loop_gain;
            ch.g_db = flush(g + a * (target - g));
            if ch.g_db > -0.01 {
                ch.g_peak_db = ch.g_db.min(0.0);
            }
        }
        ch.g_db
    }

    /// The release time constant in force, which for two of the positions
    /// depends on how far the recovery has come (`research/Distressor.md`
    /// 7.3).
    #[inline]
    fn release_tau(ctx: &Ballistics, ch: &Channel) -> f32 {
        let base = ctx.tau_rel * ctx.curve.k_release * k::FEEDBACK_TIME;
        // How far this gesture has recovered, 0 at the deepest point and 1
        // back at unity gain.
        let p = if ch.g_peak_db < -0.05 {
            (1.0 - ch.g_db / ch.g_peak_db).clamp(0.0, 1.0)
        } else {
            1.0
        };
        match ctx.curve.release {
            ReleaseShape::Standard | ReleaseShape::Fast => base,
            ReleaseShape::Log => {
                // Quick at first, then slowing.
                base * (k::NUKE_FAST + (k::NUKE_SLOW - k::NUKE_FAST) * p)
            }
            ReleaseShape::Opto => {
                // A fast first stage to about half recovery, then a tail
                // that stretches with how hard the cell has been working.
                let slow = k::OPTO_SLOW_S
                    + (k::OPTO_SLOW_MAX_S - k::OPTO_SLOW_S) * ch.opto_mem.clamp(0.0, 1.0);
                let w = (p * 2.0).clamp(0.0, 1.0);
                k::OPTO_FAST_S + (slow - k::OPTO_FAST_S) * w
            }
        }
    }

    /// The harmonic generator (`research/Distressor.md` 7.6). Returns the
    /// shaped sample; `drive` is `|u|` for the lamps.
    #[inline]
    fn shape(x: f32, mode: u8, scale: f32) -> (f32, f32) {
        if mode == 0 {
            // The VCA's own small nonlinearity is always there.
            let u = x / scale;
            let y = x * (1.0 - k::VCA_THD * u * u);
            return (y, u.abs());
        }
        let u = x / scale;
        let y = match mode {
            // Chebyshev voicings with the DC term of T2 removed.
            2 => u + k::DIST2_A2 * 2.0 * u * u,
            _ => u + k::DIST3_A3 * (4.0 * u * u * u - 3.0 * u) + k::DIST3_A2 * 2.0 * u * u,
        };
        (y * scale, u.abs())
    }

    /// Total harmonic distortion of the generator, in per cent, for a
    /// sine of peak drive `u`.
    ///
    /// This is the closed form of the shapers above rather than a guess:
    /// for `u = A·sin`, `2u²` puts `a2·A²` on the second harmonic, and
    /// `4u³ − 3u` puts `−a3·A³` on the third while taking `3a3·A(1 − A²)`
    /// off the fundamental. An earlier version used the wrong second-order
    /// coefficient for Dist 3 and ignored what the shaper does to the
    /// fundamental, which put the lamps out by about a factor of two; the
    /// test that was supposed to catch that asserted on this very number,
    /// so it never could. It now checks the figure against a measured
    /// spectrum.
    fn thd_for(mode: u8, u: f32) -> f32 {
        let u = u.abs();
        match mode {
            // y = u + 2·a2·u²: second harmonic only, fundamental untouched.
            2 => 100.0 * k::DIST2_A2 * u,
            3 => {
                let h2 = k::DIST3_A2 * u * u;
                let h3 = k::DIST3_A3 * u * u * u;
                let fund = u * (1.0 + 3.0 * k::DIST3_A3 * (u * u - 1.0));
                100.0 * (h2 * h2 + h3 * h3).sqrt() / fund.abs().max(1e-9)
            }
            // The gain cell's own small cubic term.
            _ => 100.0 * k::VCA_THD * u * u,
        }
    }

    /// Process one stereo block in place. Real-time safe.
    pub fn process_block(&mut self, l: &mut [f32], r: &mut [f32]) {
        let n = l.len().min(r.len());
        if n == 0 {
            return;
        }
        let s = self.settings;
        let mode = s.audio.distortion();
        let g_in_t = db_to_lin(knob_to_db(s.input));
        let g_out_t = db_to_lin(knob_to_db(s.output));
        let band = s.detector.band();

        // Dead patch: a mono signal linked to a silent channel halves the
        // side-chain and drives the generator harder (7.9).
        let mut peak = [0.0f32; 2];
        for i in 0..n {
            peak[0] = peak[0].max(l[i].abs());
            peak[1] = peak[1].max(r[i].abs());
        }
        self.dead_patch = s.link
            && (peak[0] > k::DEAD_PATCH_PEAK) != (peak[1] > k::DEAD_PATCH_PEAK)
            && peak[0].max(peak[1]) > k::DEAD_PATCH_PEAK;
        let drive_scale = if self.dead_patch {
            self.drive_scale / k::DEAD_PATCH_DRIVE
        } else {
            self.drive_scale
        };

        for ch in &mut self.ch {
            ch.drive_peak = 0.0;
        }
        let mut gr_sum = 0.0f32;

        for i in 0..n {
            let g_in = self.g_in.process(g_in_t);
            let g_out = self.g_out.process(g_out_t);
            let mix = self.mix.process(s.mix);
            let dry = [l[i], r[i]];
            let x1 = [dry[0] * g_in, dry[1] * g_in];

            // The gain cell uses the state the detector left last sample:
            // this is the loop's one-sample delay.
            let mut g_lin = [db_to_lin(self.ch[0].g_db), db_to_lin(self.ch[1].g_db)];
            if s.link && matches!(s.link_mode, LinkMode::Image | LinkMode::Both) {
                // Image link: one gain-control signal for both channels.
                let g = 0.5 * (self.ch[0].g_db + self.ch[1].g_db);
                let lin = db_to_lin(g);
                g_lin = [lin, lin];
            }
            let x2 = [x1[0] * g_lin[0], x1[1] * g_lin[1]];

            // Side-chain, tapped after the cell.
            let mut sc = [0.0f32; 2];
            for c in 0..2 {
                let ch = &mut self.ch[c];
                let mut v = x2[c];
                if band {
                    v = ch.band.process(v);
                }
                v = ch.det_hp.process(v);
                v = ch.sc_hp.process(v);
                sc[c] = v.abs();
            }
            let det = if s.link && matches!(s.link_mode, LinkMode::Phase | LinkMode::Both) {
                let m = 0.5 * (sc[0] + sc[1]);
                [m, m]
            } else {
                sc
            };

            for c in 0..2 {
                let lvl_db = 20.0 * det[c].max(1e-9).log10();
                let lvl_db = if s.british {
                    // The onset lag: the detector charges through a short
                    // lag before the curve sees the level (7.4a).
                    let ch = &mut self.ch[c];
                    ch.british_lag.process(lvl_db)
                } else {
                    lvl_db
                };
                let (target, slope) = self.target_and_slope(lvl_db);
                let target = target.min(0.0);
                let ctx = Ballistics {
                    curve: self.curve,
                    tau_att: self.tau_att,
                    tau_rel: self.tau_rel,
                    sr: self.sr,
                };
                let (up, down) = (self.opto_mem_up, self.opto_mem_down);
                let ch = &mut self.ch[c];
                let g = Self::ballistics(&ctx, ch, target, slope);
                // The 10:1 memory charges while the cell works.
                let want = (-g / k::OPTO_MEM_GR_DB).clamp(0.0, 1.0);
                let a = if want > ch.opto_mem { up } else { down };
                ch.opto_mem = flush(ch.opto_mem + a * (want - ch.opto_mem));
            }
            gr_sum += self.ch[0].g_db.min(self.ch[1].g_db);

            // Generator, audio high-pass, output trim, mix. The generator
            // runs at twice the rate below 88.2 kHz: it is the only
            // nonlinearity here, and at 44.1 kHz Dist 3 on a 15 kHz tone
            // folds badly without it.
            let os = self.oversample;
            for c in 0..2 {
                let ch = &mut self.ch[c];
                let (mut y, u) = if os {
                    let pair = ch.up.process(x2[c]);
                    let mut shaped = [0.0f32; 2];
                    let mut peak = 0.0f32;
                    for (o, p) in shaped.iter_mut().zip(pair.iter()) {
                        let (v, d) = Self::shape(*p, mode, drive_scale);
                        *o = v;
                        peak = peak.max(d);
                    }
                    (ch.down.process(shaped), peak)
                } else {
                    Self::shape(x2[c], mode, drive_scale)
                };
                if u > ch.drive_peak {
                    ch.drive_peak = u;
                }
                y = ch.audio_hp.process(y);
                // The dry path is delayed to match, so mix and bypass stay
                // phase-aligned.
                let dry_c = if os { ch.dry.process(dry[c]) } else { dry[c] };
                let wet = y * g_out;
                let out = if s.bypass {
                    dry_c
                } else {
                    dry_c + (wet - dry_c) * mix
                };
                if c == 0 { l[i] = out } else { r[i] = out }
            }
        }

        self.gr_db = (gr_sum / n as f32).min(0.0);
        let u = self.ch[0].drive_peak.max(self.ch[1].drive_peak);
        self.drive = u.min(4.0);
        self.thd_pct = Self::thd_for(mode, self.drive);
    }

    /// Mean gain change of the last block in dB (≤ 0).
    pub fn gr_db(&self) -> f32 {
        self.gr_db
    }

    /// Estimated generator distortion of the last block, per cent.
    pub fn thd_pct(&self) -> f32 {
        self.thd_pct
    }

    /// `true` when the estimated distortion is past the redline lamp.
    pub fn redline(&self) -> bool {
        self.thd_pct >= k::REDLINE_THD_PCT
    }

    /// `true` when the estimated distortion is past the 1 % lamp.
    pub fn lamp_1pct(&self) -> bool {
        self.thd_pct >= k::LAMP_THD_PCT
    }

    /// Peak drive of the generator over the last block, 0..4.
    pub fn drive(&self) -> f32 {
        self.drive
    }

    /// `true` when a mono signal is linked to a silent channel.
    pub fn dead_patch(&self) -> bool {
        self.dead_patch
    }

    /// The steady-state gain change in dB for a sine of `level_db` dBFS at
    /// the input, solved by fixed-point iteration because the detector
    /// hears the compressed signal (the loop is a feedback one).
    pub fn static_gain_db(&self, level_db: f32) -> f32 {
        let g_in = knob_to_db(self.settings.input);
        // `target(x + g) − g` falls monotonically in `g`, so bisect: the
        // loop gain is far too high for a plain fixed-point iteration.
        let f = |g: f32| self.target_and_slope(level_db + g_in + g).0.min(0.0) - g;
        let (mut lo, mut hi) = (-200.0f32, 0.0f32);
        if f(hi) >= 0.0 {
            return 0.0;
        }
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if f(mid) > 0.0 { lo = mid } else { hi = mid }
        }
        0.5 * (lo + hi)
    }

    /// Fill `out` with the static output level in dBFS for inputs from
    /// `min_dbfs` to `max_dbfs`.
    pub fn transfer_curve(&self, out: &mut [f32], min_dbfs: f32, max_dbfs: f32) {
        let n = out.len();
        if n == 0 {
            return;
        }
        let g_in = knob_to_db(self.settings.input);
        let g_out = knob_to_db(self.settings.output);
        for (i, o) in out.iter_mut().enumerate() {
            let x = min_dbfs + (max_dbfs - min_dbfs) * i as f32 / (n - 1).max(1) as f32;
            let g = self.static_gain_db(x);
            let wet = x + g_in + g + g_out;
            *o = if self.settings.bypass {
                x
            } else {
                // Mix in the amplitude domain, as the other engines do.
                let dry = db_to_lin(x);
                let w = db_to_lin(wet);
                20.0 * (dry + (w - dry) * self.settings.mix).max(1e-6).log10()
            };
        }
    }
}

/// dB → linear amplitude.
#[inline]
pub fn db_to_lin(db: f32) -> f32 {
    if db <= -119.0 {
        0.0
    } else {
        10f32.powf(db / 20.0)
    }
}

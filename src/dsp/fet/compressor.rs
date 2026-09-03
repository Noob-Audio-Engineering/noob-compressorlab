//! The compressor engine: a grey-box model of a FET feedback limiter, per
//! `research/1176.md` section 7.
//!
//! ## Signal path (per channel, at the processing rate)
//!
//! ```text
//! in ─► ×g_in ─► input HP 15 Hz ─► ×2 up ─► FET divider ─► preamp (+24 dB, tanh) ─┬─► ×g_out ─► line amp (+24 dB, tanh + 2nd) ─► transformer HP 20 Hz ─► ×2 down ─► mix ─► out
//!                                                ▲                                   │ y_tap
//!                                                │ g_inst(v, x2)                     ▼
//!                                             FET law ◄── capacitor v ◄── diode ◄── ×k_r ◄── side-chain HPF
//! ```
//!
//! The sidechain is fed from the preamp output (`y_tap`), which makes this
//! a feedback compressor: the threshold is fixed inside the loop, the Input
//! knob drives gain reduction, and the effective attack, release and ratio
//! depend on how far above threshold the signal sits.
//!
//! ## Equations
//!
//! With `a_att = 1 − exp(−1 / (τ_att·fs_p))` and `a_rel` likewise:
//!
//! ```text
//! s[n] = k_r · y_tap[n−1]                       ratio ladder, one-sample loop delay
//! e[n] = max(0, |s[n]| − V_T,r)                 full-wave rectifier, diode bias = threshold
//! v[n] = v[n−1] + a_att·max(0, e[n] − v[n−1]) − a_rel·v[n−1]
//! G_dB = −G_max · (1 − exp(−S·v_eff / G_max))   FET law, v_eff = max(0, v − V_off)
//! g    = 10^(G_dB/20),  w = 1/g − 1
//! g_inst = 1 / (1 + w·(1 + a2·u + a3·u²)),  u = x2[n−1] / X0
//! ```
//!
//! The onset ratio is `R = 1 + 0.1151·S·V_T,r`, so the diode bias per
//! button is derived from the target ratio, and the ladder gain `k_r` places
//! the tap threshold at [`TAP_THRESHOLD_DB`].
//!
//! ## Constants (tuned against the tests; the research's estimates noted)
//!
//! | constant | value | note |
//! |---|---|---|
//! | `G_PRE_DB`, `G_LINE_DB` | 24 + 24 dB | unity at Input 24 / Output 24 |
//! | `S` (dB per volt) | 48 (research: 40) | with `G_MAX` 48 the law is `exp(−v)` |
//! | `G_MAX` | 48 dB (research: 40) | deeper plateau keeps the 20:1 slope within 20 % of nominal 6–16 dB above onset |
//! | onset ratios | 4, 8, 12, 20 | `V_T,r = (R − 1) / (0.1151·S)` |
//! | tap thresholds | −26, −24.5, −23, −20 dBFS (research: about a 6 dB spread) | with Input at 24 |
//! | all buttons | `G_MAX` 32, onset 16, `V_OFF` 1.2 V, ladder of 20:1 | bias shift, plateau, distortion ×5, sag |
//! | `X0` | 0.02 (−34 dBFS across the FET, −10 dBFS at the tap; research: 250 mV, about −14 dBFS) | FET nonlinearity reference; lower so the blue stripe distorts at normal drive |
//! | `a2`, `a3` | per revision ([`Circuit`]): LN family 0.04 / 0.10 at half FET swing, A 0.20 / 0.06, B 0.15 / 0.05 | even / odd FET terms |
//! | `x_pre`, `x_line` | 1.67 (A preamp 1.25, F and later line amp 2.2) | 0 dBFS is 1 dB into the tanh |
//! | `c_line` | 0.03 (0 for the push-pull stage of F and later) | line-amp second harmonic |
//! | `tilt_db` | A / B 1.2, C to E and LN 0.8, F 0.5, G / H 0.3 | high shelf at 6 kHz |
//! | `noise_dbfs` | A −70, B −72, C to E −74, F and LN −78, G / H −80 | output noise floor at Output 24 |
//! | transformer | 15 Hz Q 0.6 in, 20 Hz Q 0.707 out | second-order high-passes |
//!
//! Not modelled (deviations from the research): the transformer's
//! low-frequency core saturation and its ±0.5 dB spectral tilt (both flagged
//! as estimates to be measured first), and the 40 µs minimum attack of a
//! hardware stereo link.
//!
//! ## Real-time behaviour
//!
//! No allocation after construction. All states are `f32` with explicit
//! flushing of the detector, the FET memory and the filters' tiny values.
//! Below 88.2 kHz the gain multiplication and the waveshapers run at 2x
//! ([`oversample`](super::oversample)); above, at 1x. Time constants are
//! computed from the processing rate, so behaviour is sample-rate
//! independent.

use super::filters::{Biquad, OnePole, coefficient, flush};
use super::oversample::{Downsampler, LATENCY, Upsampler};
use super::{MeterMode, Ratio, Revision, Settings, attack_seconds, mark_to_db, release_seconds};

/// Points of the static transfer curve (input −60..0 dBFS).
pub const TRANSFER_POINTS: usize = 128;

const G_PRE_DB: f32 = 24.0;
const G_LINE_DB: f32 = 24.0;
/// FET law initial slope, dB per volt.
const S_DB_PER_V: f32 = 48.0;
/// FET law plateau, dB.
const G_MAX_DB: f32 = 48.0;
/// Plateau in all-buttons mode.
const G_MAX_ALL_DB: f32 = 32.0;
/// Bias shift in all-buttons mode (volts of the modelled sidechain).
const V_OFF_ALL: f32 = 1.2;
/// Onset ratios per button (4:1, 8:1, 12:1, 20:1) and for all buttons.
const ONSET_RATIO: [f32; 5] = [4.0, 8.0, 12.0, 20.0, 16.0];
/// Threshold at the preamp tap per button, dBFS peak, with Input at 24.
pub const TAP_THRESHOLD_DB: [f32; 4] = [-26.0, -24.5, -23.0, -20.0];
/// FET nonlinearity reference amplitude: −34 dBFS across the FET, which
/// is −10 dBFS at the preamp output (the signal across the FET is 24 dB
/// below the tap; the research quotes 250 mV, reached only when driven).
const X0: f32 = 0.02;
/// tanh knee of the preamp and line amp: 0 dBFS is 1 dB into the tanh.
const X_TANH: f32 = 1.67;
/// Corner of the revision's high-shelf tilt, Hz.
const TILT_HZ: f32 = 6000.0;
/// 20·log10(e) / 20: dB per neper.
const DB_PER_NEPER: f32 = 0.115_129_25;

/// The circuit character of one revision: every knob of the model that the
/// sources tie to a revision (research/1176.md §1.2, §3.8, §4.6). The README's
/// revision table lists the values and the source of each.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Circuit {
    /// FET even-order term: second harmonic, the blue-stripe signature.
    pub a2: f32,
    /// FET odd-order term.
    pub a3: f32,
    /// The LN circuit ("reduced voltage to the gain-reduction FET"): halves
    /// the signal swing the FET's nonlinearity sees.
    pub ln: bool,
    /// Preamp tanh knee; the FET preamp of Rev A clips earlier.
    pub x_pre: f32,
    /// Line-amp tanh knee; the class-AB stage of F and later has more headroom.
    pub x_line: f32,
    /// Line-amp asymmetry (second harmonic of a single-ended class-A stage;
    /// 0 for the push-pull stage).
    pub c_line: f32,
    /// Input transformer high-pass (corner Hz, Q); G and later are
    /// electronically balanced and only keep a DC blocker.
    pub in_hp: (f32, f32),
    /// Output transformer high-pass (corner Hz, Q).
    pub out_hp: (f32, f32),
    /// High-shelf tilt at 6 kHz (`TILT_HZ`), dB (the "bright" tilt of the sources).
    pub tilt_db: f32,
    /// Noise floor: white noise into the preamp, given as the output RMS in
    /// dBFS with Output at 24.
    pub noise_dbfs: f32,
}

/// Rev A: FET preamp, no LN circuit, Peerless / UA-5002 transformers.
const CIRCUIT_A: Circuit = Circuit {
    a2: 0.20,
    a3: 0.06,
    ln: false,
    x_pre: 1.25,
    x_line: X_TANH,
    c_line: -0.03,
    in_hp: (20.0, 0.6),
    out_hp: (25.0, 0.7),
    tilt_db: 1.2,
    noise_dbfs: -70.0,
};
/// Rev B: the bipolar preamp (and the AB resistor changes), still no LN circuit.
const CIRCUIT_B: Circuit = Circuit {
    a2: 0.15,
    a3: 0.05,
    x_pre: X_TANH,
    noise_dbfs: -72.0,
    ..CIRCUIT_A
};
/// Rev C / D / E: the LN circuit, class-A 1108-style output, both transformers.
const CIRCUIT_LN: Circuit = Circuit {
    a2: 0.04,
    a3: 0.10,
    ln: true,
    x_pre: X_TANH,
    x_line: X_TANH,
    c_line: -0.03,
    in_hp: (15.0, 0.6),
    out_hp: (20.0, 0.707),
    tilt_db: 0.8,
    noise_dbfs: -74.0,
};
/// Rev F: push-pull class-AB output stage, Bourns output transformer.
const CIRCUIT_F: Circuit = Circuit {
    x_line: 2.2,
    c_line: 0.0,
    out_hp: (18.0, 0.6),
    tilt_db: 0.5,
    noise_dbfs: -78.0,
    ..CIRCUIT_LN
};
/// Rev G (and H, cosmetic only): electronically balanced input.
const CIRCUIT_G: Circuit = Circuit {
    in_hp: (5.0, 0.5),
    tilt_db: 0.3,
    noise_dbfs: -80.0,
    ..CIRCUIT_F
};
/// The reissue: C / D / E with a modern noise floor.
const CIRCUIT_REISSUE: Circuit = Circuit {
    noise_dbfs: -78.0,
    ..CIRCUIT_LN
};

/// The circuit of a revision. Revisions the sources describe as
/// functionally identical share one set of constants.
pub const fn circuit(rev: Revision) -> Circuit {
    match rev {
        Revision::A => CIRCUIT_A,
        Revision::B => CIRCUIT_B,
        Revision::C | Revision::D | Revision::E => CIRCUIT_LN,
        Revision::F => CIRCUIT_F,
        Revision::G | Revision::H => CIRCUIT_G,
        Revision::Ln => CIRCUIT_REISSUE,
    }
}

/// Sidechain constants for one ratio setting.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Loop {
    /// Ladder gain: volts per unit of tap amplitude.
    k: f32,
    /// Diode bias (threshold), volts.
    v_t: f32,
    /// Bias offset the capacitor must charge through (all buttons).
    v_off: f32,
    /// FET plateau, dB.
    g_max: f32,
}

impl Loop {
    fn for_ratio(r: Ratio) -> Self {
        let bias = |ratio: f32, s: f32| (ratio - 1.0) / (DB_PER_NEPER * s);
        match r {
            Ratio::All => {
                let s = G_MAX_ALL_DB; // keep v0 = G_max / S = 1
                let total = bias(ONSET_RATIO[4], s);
                let k20 = Loop::for_ratio(Ratio::R20).k;
                Loop {
                    k: k20,
                    v_t: total - V_OFF_ALL,
                    v_off: V_OFF_ALL,
                    g_max: G_MAX_ALL_DB,
                }
            }
            _ => {
                let i = match r {
                    Ratio::R4 => 0,
                    Ratio::R8 => 1,
                    Ratio::R12 => 2,
                    _ => 3,
                };
                let v_t = bias(ONSET_RATIO[i], S_DB_PER_V);
                let thr = 10f32.powf(TAP_THRESHOLD_DB[i] / 20.0);
                Loop {
                    k: v_t / thr,
                    v_t,
                    v_off: 0.0,
                    g_max: G_MAX_DB,
                }
            }
        }
    }
}

/// FET gain in dB for a control voltage above the bias offset.
#[inline]
fn fet_db(v_eff: f32, g_max: f32) -> f32 {
    let s = g_max; // v0 = 1 V for every mode
    -g_max * (1.0 - (-s * v_eff / g_max).exp())
}

#[inline]
fn tanh_fast(x: f32) -> f32 {
    // Padé-style rational approximation, accurate to ~1e-4 over ±5, exact
    // limits beyond; cheaper than libm tanh and free of platform variance.
    if x > 5.0 {
        return 1.0;
    }
    if x < -5.0 {
        return -1.0;
    }
    let x2 = x * x;
    x * (27.0 + x2) / (27.0 + 9.0 * x2)
}

#[inline]
fn preamp(x: f32, x_pre: f32) -> f32 {
    x_pre * tanh_fast(x / x_pre)
}

#[inline]
fn line_amp(x: f32, x_line: f32, c_line: f32) -> f32 {
    let t = tanh_fast(x / x_line);
    x_line * (t + c_line * t * t)
}

/// The detector capacitor (one per channel, or one shared when linked).
#[derive(Clone, Copy, Default)]
struct Detector {
    v: f32,
    /// Slow average of `v` for the all-buttons "sag".
    v_slow: f32,
}

/// One channel's audio-path state.
#[derive(Clone)]
struct Channel {
    up: Upsampler,
    down: Downsampler,
    in_hp: Biquad,
    out_hp: Biquad,
    /// The revision's high-shelf tilt (oversampled rate).
    tilt: Biquad,
    sc_hp: Biquad,
    /// Noise generator state (xorshift32, never zero).
    rng: u32,
    /// Previous FET divider output (the signal across the FET).
    x2_prev: f32,
    /// Previous preamp output (the sidechain tap).
    y_tap_prev: f32,
    /// Dry path delay for mix / bypass (round-trip latency of the oversampler).
    dry: [f32; LATENCY + 1],
    dry_pos: usize,
}

impl Channel {
    fn new(fs: f32, fs_p: f32) -> Self {
        Channel {
            up: Upsampler::new(),
            down: Downsampler::new(),
            in_hp: Biquad::highpass(fs, 15.0, 0.6),
            out_hp: Biquad::highpass(fs_p, 20.0, 0.707),
            tilt: Biquad::identity(),
            sc_hp: Biquad::identity(),
            rng: 0x9E37_79B9,
            x2_prev: 0.0,
            y_tap_prev: 0.0,
            dry: [0.0; LATENCY + 1],
            dry_pos: 0,
        }
    }

    fn reset(&mut self) {
        self.up.reset();
        self.down.reset();
        self.in_hp.reset();
        self.out_hp.reset();
        self.tilt.reset();
        self.sc_hp.reset();
        self.x2_prev = 0.0;
        self.y_tap_prev = 0.0;
        self.dry = [0.0; LATENCY + 1];
        self.dry_pos = 0;
    }

    /// White noise, uniform in −1..1 (RMS 1/√3).
    #[inline]
    fn noise(&mut self) -> f32 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        x as f32 * (2.0 / 4_294_967_296.0) - 1.0
    }
}

/// Stereo FET compressor.
pub struct Compressor {
    fs_p: f32,
    oversample: bool,
    ch: [Channel; 2],
    det: [Detector; 2],
    settings: Settings,
    // Smoothed continuous values (per base-rate sample).
    g_in: OnePole,
    g_out: OnePole,
    k: OnePole,
    v_t: OnePole,
    v_off: OnePole,
    g_max: OnePole,
    mix: OnePole,
    a_att: f32,
    a_rel: f32,
    a_sag: f32,
    attack_off: bool,
    a2: f32,
    a3: f32,
    /// The selected revision's circuit and the revision whose filters are built.
    circuit: Circuit,
    circuit_rev: Option<Revision>,
    /// Noise injection gain at the preamp input.
    noise_gain: f32,
    all: bool,
    sc_hpf_hz: f32,
    // Block meters.
    gr_db: f32,
    meter_gr_db: f32,
    meter_sum_abs: f32,
    meter_n: u32,
    dirty: bool,
}

impl Compressor {
    /// A compressor for sample rate `fs`, at the default settings.
    pub fn new(fs: f32) -> Self {
        let oversample = fs < 88_200.0;
        let fs_p = if oversample { fs * 2.0 } else { fs };
        let s = Settings::default();
        let lp = Loop::for_ratio(s.ratio);
        let mut c = Compressor {
            fs_p,
            oversample,
            ch: [Channel::new(fs, fs_p), Channel::new(fs, fs_p)],
            det: [Detector::default(); 2],
            settings: s,
            g_in: OnePole::new(fs, 0.005, 10f32.powf(mark_to_db(s.input) / 20.0)),
            g_out: OnePole::new(fs, 0.005, 10f32.powf(mark_to_db(s.output) / 20.0)),
            k: OnePole::new(fs, 0.01, lp.k),
            v_t: OnePole::new(fs, 0.01, lp.v_t),
            v_off: OnePole::new(fs, 0.01, lp.v_off),
            g_max: OnePole::new(fs, 0.01, lp.g_max),
            mix: OnePole::new(fs, 0.005, s.mix),
            a_att: 0.0,
            a_rel: 0.0,
            a_sag: coefficient(fs_p, 0.1),
            attack_off: false,
            a2: 0.0,
            a3: 0.0,
            circuit: circuit(s.revision),
            circuit_rev: None,
            noise_gain: 0.0,
            all: false,
            sc_hpf_hz: -1.0,
            gr_db: 0.0,
            meter_gr_db: 0.0,
            meter_sum_abs: 0.0,
            meter_n: 0,
            dirty: true,
        };
        c.apply_settings(true);
        c
    }

    /// Change the sample rate: rebuilds the filters and clears the state.
    pub fn set_sample_rate(&mut self, fs: f32) {
        *self = Compressor::new(fs);
    }

    /// Clear every state (buffers, detector), keeping the settings.
    pub fn reset(&mut self) {
        for c in &mut self.ch {
            c.reset();
        }
        self.det = [Detector::default(); 2];
    }

    /// Latency in base-rate samples (the oversampler's round trip).
    pub fn latency(&self) -> usize {
        if self.oversample { LATENCY } else { 0 }
    }

    /// The current settings.
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Apply a settings snapshot. Continuous values are smoothed; switches
    /// take effect at once (with their loop constants crossfaded). Returns
    /// whether anything that changes the static curve changed.
    pub fn configure(&mut self, s: &Settings) -> bool {
        if *s == self.settings && !self.dirty {
            return false;
        }
        self.settings = *s;
        self.apply_settings(false);
        self.dirty = false;
        true
    }

    fn apply_settings(&mut self, snap: bool) {
        let s = self.settings;
        let lp = Loop::for_ratio(s.ratio);
        let g_in = 10f32.powf(mark_to_db(s.input) / 20.0);
        let g_out = 10f32.powf(mark_to_db(s.output) / 20.0);
        if snap {
            self.g_in.snap(g_in);
            self.g_out.snap(g_out);
            self.k.snap(lp.k);
            self.v_t.snap(lp.v_t);
            self.v_off.snap(lp.v_off);
            self.g_max.snap(lp.g_max);
            self.mix.snap(s.mix);
        }
        self.all = s.ratio == Ratio::All;
        match attack_seconds(s.attack) {
            Some(t) => {
                self.attack_off = false;
                self.a_att = coefficient(self.fs_p, t);
            }
            None => self.attack_off = true,
        }
        self.a_rel = coefficient(self.fs_p, release_seconds(s.release));
        let circ = circuit(s.revision);
        self.circuit = circ;
        self.a2 = if self.all { circ.a2 * 5.0 } else { circ.a2 };
        self.a3 = circ.a3;
        let os: f32 = if self.oversample { 2.0 } else { 1.0 };
        // RMS at the output = noise_dbfs with Output at 24 (the preamp adds
        // G_PRE_DB, the output pot takes it away, the line amp gives it
        // back); √3 for the uniform generator, √os for the decimator's band.
        self.noise_gain = 10f32.powf((circ.noise_dbfs - G_PRE_DB) / 20.0) * 3f32.sqrt() * os.sqrt();
        if snap || self.circuit_rev != Some(s.revision) {
            self.circuit_rev = Some(s.revision);
            let fs = self.fs_p / os;
            let in_hp = Biquad::highpass(fs, circ.in_hp.0, circ.in_hp.1);
            let out_hp = Biquad::highpass(self.fs_p, circ.out_hp.0, circ.out_hp.1);
            let tilt = Biquad::highshelf(self.fs_p, TILT_HZ, circ.tilt_db);
            for c in &mut self.ch {
                c.in_hp.set_from(&in_hp);
                c.out_hp.set_from(&out_hp);
                c.tilt.set_from(&tilt);
            }
        }
        if s.sc_hpf_hz != self.sc_hpf_hz {
            self.sc_hpf_hz = s.sc_hpf_hz;
            let bq = if s.sc_hpf_hz >= 5.0 {
                Biquad::highpass(self.fs_p, s.sc_hpf_hz, 0.707)
            } else {
                Biquad::identity()
            };
            for c in &mut self.ch {
                c.sc_hp.set_from(&bq);
            }
        }
    }

    /// Gain reduction in dB (≤ 0) at the end of the last block.
    pub fn gr_db(&self) -> f32 {
        self.gr_db
    }

    /// What the panel meter reads for the selected mode, in dB, for the
    /// last block: GR (all-buttons: from the raw capacitor voltage), or the
    /// VU of the mean rectified output referenced to +4 / +8 dBu, or −60
    /// when off. Resets the block accumulator.
    pub fn take_meter_reading(&mut self) -> f32 {
        let n = self.meter_n.max(1) as f32;
        let mean = self.meter_sum_abs / n;
        self.meter_sum_abs = 0.0;
        self.meter_n = 0;
        match self.settings.meter {
            MeterMode::Gr => self.meter_gr_db,
            MeterMode::Plus4 => vu_db(mean, -18.0),
            MeterMode::Plus8 => vu_db(mean, -14.0),
            MeterMode::Off => -60.0,
        }
    }

    /// Process one block in place (stereo). Reads the settings as configured.
    pub fn process(&mut self, l: &mut [f32], r: &mut [f32]) {
        let n = l.len().min(r.len());
        let a_att = self.a_att;
        let a_rel = self.a_rel;
        let a_sag = self.a_sag;
        let link = self.settings.link;
        let bypass = self.settings.bypass;
        let attack_off = self.attack_off;
        let all = self.all;
        let a2 = self.a2;
        let a3 = self.a3;
        let circ = self.circuit;
        let u_scale = if circ.ln { 0.5 / X0 } else { 1.0 / X0 };
        let noise_gain = self.noise_gain;
        let g_pre = 10f32.powf(G_PRE_DB / 20.0);
        let g_line = 10f32.powf(G_LINE_DB / 20.0);
        let os = if self.oversample { 2 } else { 1 };
        let mut last_g_db = 0.0f32;
        let mut last_meter_db = 0.0f32;

        for i in 0..n {
            let g_in = self
                .g_in
                .process(10f32.powf(mark_to_db(self.settings.input) / 20.0));
            let g_out = self
                .g_out
                .process(10f32.powf(mark_to_db(self.settings.output) / 20.0));
            let k = self.k.process(Loop::for_ratio(self.settings.ratio).k);
            let v_t = self.v_t.process(Loop::for_ratio(self.settings.ratio).v_t);
            let v_off = self
                .v_off
                .process(Loop::for_ratio(self.settings.ratio).v_off);
            let g_max = self
                .g_max
                .process(Loop::for_ratio(self.settings.ratio).g_max);
            let mix = self.mix.process(self.settings.mix);

            let xin = [l[i], r[i]];
            let mut up = [[0.0f32; 2]; 2];
            for c in 0..2 {
                let ch = &mut self.ch[c];
                // dry path, delayed by the oversampler's round trip
                ch.dry[ch.dry_pos] = xin[c];
                let x1 = ch.in_hp.process(xin[c] * g_in);
                up[c] = if os == 2 {
                    ch.up.process(x1)
                } else {
                    [x1, 0.0]
                };
            }
            let mut pair = [[0.0f32; 2]; 2];
            for j in 0..os {
                // --- sidechain from the previous tap outputs
                let mut e = [0.0f32; 2];
                for c in 0..2 {
                    let ch = &mut self.ch[c];
                    let s = k * ch.sc_hp.process(ch.y_tap_prev);
                    e[c] = (s.abs() - v_t).max(0.0);
                }
                if link {
                    let m = e[0].max(e[1]);
                    e = [m, m];
                }
                let mut g_db = [0.0f32; 2];
                let mut g_db_meter = [0.0f32; 2];
                for c in 0..2 {
                    let d = &mut self.det[if link { 0 } else { c }];
                    if c == 0 || !link {
                        if !attack_off {
                            d.v += a_att * (e[c] - d.v).max(0.0);
                        }
                        d.v -= a_rel * d.v;
                        d.v = flush(d.v);
                        if all {
                            d.v_slow += a_sag * (d.v - d.v_slow);
                        }
                    }
                    let v_off_eff = if all { v_off + 0.2 * d.v_slow } else { 0.0 };
                    let v_eff = (d.v - v_off_eff).max(0.0);
                    g_db[c] = if attack_off {
                        0.0
                    } else {
                        fet_db(v_eff, g_max)
                    };
                    g_db_meter[c] = if all { fet_db(d.v, g_max) } else { g_db[c] };
                }
                for c in 0..2 {
                    let ch = &mut self.ch[c];
                    let x = up[c][j];
                    // FET divider with signal-dependent resistance
                    let g_lin = 10f32.powf(g_db[c] / 20.0);
                    let w = 1.0 / g_lin - 1.0;
                    let u = ch.x2_prev * u_scale;
                    let a2_eff = if all {
                        a2 * (1.0 + 0.5 * self.det[if link { 0 } else { c }].v)
                    } else {
                        a2
                    };
                    let shape = (1.0 + a2_eff * u + a3 * u * u).clamp(0.5, 2.0);
                    let g_inst = if attack_off {
                        1.0
                    } else {
                        1.0 / (1.0 + w * shape)
                    };
                    let x2 = g_inst * x;
                    ch.x2_prev = flush(x2);
                    let y_tap = preamp((x2 + ch.noise() * noise_gain) * g_pre, circ.x_pre);
                    ch.y_tap_prev = flush(y_tap);
                    let y3 = y_tap * g_out;
                    let y4 = ch
                        .out_hp
                        .process(line_amp(y3 * g_line, circ.x_line, circ.c_line));
                    pair[c][j] = ch.tilt.process(y4);
                }
                last_g_db = g_db[0].min(g_db[1]);
                last_meter_db = g_db_meter[0].min(g_db_meter[1]);
            }
            for c in 0..2 {
                let ch = &mut self.ch[c];
                let wet = if os == 2 {
                    ch.down.process(pair[c])
                } else {
                    pair[c][0]
                };
                let dry_i = if self.oversample {
                    (ch.dry_pos + 1) % (LATENCY + 1)
                } else {
                    ch.dry_pos
                };
                let dry = ch.dry[dry_i];
                ch.dry_pos = (ch.dry_pos + 1) % (LATENCY + 1);
                let out = if bypass { dry } else { dry + (wet - dry) * mix };
                if c == 0 {
                    l[i] = out;
                } else {
                    r[i] = out;
                }
                self.meter_sum_abs += out.abs();
            }
            self.meter_n += 2;
        }
        self.gr_db = last_g_db;
        self.meter_gr_db = last_meter_db;
    }

    /// The static transfer curve at the current settings: output level in
    /// dBFS for a steady sine at each of [`TRANSFER_POINTS`] input levels
    /// from −60 to 0 dBFS. A fixed-point solution of the loop (the detector
    /// is assumed to track the tap peak), for display; the tests measure the
    /// real engine.
    pub fn transfer(&self, out: &mut [f32; TRANSFER_POINTS]) {
        let s = self.settings;
        let lp = Loop::for_ratio(s.ratio);
        let g_in = 10f32.powf(mark_to_db(s.input) / 20.0);
        let g_out = 10f32.powf(mark_to_db(s.output) / 20.0);
        let g_pre = 10f32.powf(G_PRE_DB / 20.0);
        let g_line = 10f32.powf(G_LINE_DB / 20.0);
        let circ = self.circuit;
        let attack_off = attack_seconds(s.attack).is_none();
        for (i, o) in out.iter_mut().enumerate() {
            let l_in = -60.0 + 60.0 * i as f32 / (TRANSFER_POINTS - 1) as f32;
            let a1 = 10f32.powf(l_in / 20.0) * g_in;
            // Solve v = k·A_tap(v) − V_T by bisection: the right-hand side falls
            // with v, so the difference changes sign exactly once.
            let tap_for = |v: f32| {
                preamp(
                    10f32.powf(fet_db((v - lp.v_off).max(0.0), lp.g_max) / 20.0) * a1 * g_pre,
                    circ.x_pre,
                )
            };
            let mut a_tap = preamp(a1 * g_pre, circ.x_pre);
            if !attack_off && lp.k * tap_for(0.0) - lp.v_t > 0.0 {
                let mut lo = 0.0f32;
                let mut hi = 40.0f32;
                for _ in 0..40 {
                    let mid = 0.5 * (lo + hi);
                    if lp.k * tap_for(mid) - lp.v_t > mid {
                        lo = mid
                    } else {
                        hi = mid
                    }
                }
                a_tap = tap_for(0.5 * (lo + hi));
            }
            let a_out = line_amp(a_tap * g_out * g_line, circ.x_line, circ.c_line)
                .abs()
                .max(1e-6);
            let wet_db = 20.0 * a_out.log10();
            *o = if s.bypass {
                l_in
            } else {
                // mix in the amplitude domain
                let dry = 10f32.powf(l_in / 20.0);
                20.0 * (dry + (a_out - dry) * s.mix).max(1e-6).log10()
            };
            let _ = wet_db;
        }
    }
}

/// VU reading in dB of a mean rectified value, for a reference sine of
/// `ref_rms_dbfs` RMS reading 0 VU (a full-wave average of a sine is
/// 0.637 of its peak).
#[inline]
fn vu_db(mean_abs: f32, ref_rms_dbfs: f32) -> f32 {
    let ref_peak = 10f32.powf(ref_rms_dbfs / 20.0) * std::f32::consts::SQRT_2;
    let ref_mean = std::f32::consts::FRAC_2_PI * ref_peak;
    if mean_abs <= 1e-9 {
        -60.0
    } else {
        (20.0 * (mean_abs / ref_mean).log10()).max(-60.0)
    }
}

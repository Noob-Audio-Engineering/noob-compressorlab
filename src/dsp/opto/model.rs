//! The grey-box model of an optical leveling amplifier, after
//! `research/LA-2A.md` section 7. The circuit topology is kept: a feedback
//! compressor whose gain element is a voltage divider with a photoresistor,
//! lit by an electroluminescent panel that a tube sidechain amplifier
//! drives; the constants are tuned to the published behaviour by the tests
//! in `tests.rs`.
//!
//! Signal path per channel, per sample:
//!
//! 1. input transformer high-pass (12 Hz, first order, so 30 Hz stays within
//!    the published −1 dB);
//! 2. the attenuator `y = x · A(R_cell)`, the divider of the schematic
//!    (`R_series = 70.7 kΩ`, `R_pot = 100 kΩ`) normalised so that a dark cell
//!    is unity;
//! 3. the optional photocell cubic (small odd-order distortion that grows
//!    with gain reduction);
//! 4. make-up: the Gain knob, a single-ended tube stage with slightly
//!    asymmetric soft clipping, the output transformer low-pass;
//! 5. `mix` between the processed and the dry signal, or bypass.
//!
//! Sidechain, per channel: the tap (Compress takes the attenuated signal,
//! Limit blends in `β` of the input, the R7 / (R6 + R7) divider of the
//! schematic, tuned up from 0.038 to 0.09 so Limit gives the extra reduction
//! the research derives), the modern side-chain high-pass, the Peak
//! Reduction gain, the sidechain tube's `tanh` saturation, the R37 low
//! shelf and the fixed tilt; then the electroluminescent panel (1 ms
//! smoothing of the rectified drive, Alfrey-Taylor light law) and the
//! photocell with traps ([`Cell`]), whose conductance closes the loop one
//! sample later. With `link` on, one panel and cell serve both channels,
//! driven by the mean of the two sidechains before rectification.
//!
//! Everything is `f32`, one-pole or explicit-Euler, and every state is
//! flushed to zero when it decays below `1e-12`, so nothing here can leave
//! a denormal after silence. No allocation, no locks.

use super::filters::{Biquad, OnePole, Shelf};

/// The photocell itself lives in its own crate, because the LA-3A shares
/// it exactly and the CL-1B deliberately does not, which is what
/// established where its edge lies. Re-exported here so this module still
/// reads as the whole of the LA-2A's circuit: what follows is the divider
/// around the cell, the sidechain that lights it and the amplifier after
/// it, which are this machine's and not the part's.
pub use noob_electrical_components::photocell::{
    CELL_GAMMA, CELL_SPEEDS, Cell, CellParams, EL_B, K_G, LA2_FAST_SHARE, LA2_FAST_SPEED, R_DARK,
    R_MIN, cell_params_for, resistance_for,
};

/// The divider's series resistance, `R6 + R7` of the schematic (ohms).
pub const R_SERIES: f32 = 70.7e3;
/// The Gain pot across the cell (ohms).
pub const R_POT: f32 = 100e3;
/// Limit mode's feed-forward share of the sidechain tap. The schematic's
/// `R7 / (R6 + R7)` is 0.038; with this model's sidechain gain that gave only
/// 2 dB more reduction than Compress at 20 dB, so it is tuned to 0.09, which
/// gives the 4 dB or more the research derives.
pub const BETA_LIMIT: f32 = 0.09;

/// dB per Peak Reduction unit (the fitted threshold span of the research).
pub const PR_DB_PER_UNIT: f32 = 0.55;
/// Peak Reduction setting at which 1 dB of gain reduction sits at 0 VU.
pub const PR_CALIBRATION: f32 = 30.0;
/// Gain-reduction the calibration point asks for, dB.
pub const CALIBRATION_GR_DB: f32 = 1.0;
/// 0 VU is this many dBFS RMS (+4 dBu).
pub const VU_REF_DBFS: f32 = -18.0;
/// Peak amplitude of a sine at 0 VU.
pub const VU_REF_AMP: f32 = 0.125_892_54 * std::f32::consts::SQRT_2;
/// Peak amplitude of a sine at +10 VU reference (+10 dBu, −12 dBFS RMS).
pub const VU10_REF_AMP: f32 = 0.251_188_64 * std::f32::consts::SQRT_2;
/// Average of `|sin|`, the rectifier's DC for a sine of unit amplitude.
pub const SINE_MEAN_ABS: f32 = std::f32::consts::FRAC_2_PI;

/// Sidechain amplifier saturation as a multiple of the sidechain amplitude
/// at the calibration onset (the 6AQ5 runs out of swing; this caps the
/// deepest gain reduction and tames the attack on big overshoots).
pub const V_SAT_OVER_ONSET: f32 = 10.0;
/// Photocell cubic distortion strength at full gain reduction.
pub const CELL_CUBIC: f32 = 0.6;
/// Reference amplitude for the photocell cubic.
pub const CELL_CUBIC_V0: f32 = 0.25;
/// The tube stage's softness. Set from the two published distortion
/// points, about 0.75 % at the +16 dBu equivalent and 0.3 % at +10 dBu,
/// and now measured at both rather than asserted in a comment: at 0.2 the
/// stage was half as dirty as the sources say.
pub const TUBE_K: f32 = 0.30;
/// Tube stage bias as a fraction of the clip level (second harmonic).
pub const TUBE_BIAS: f32 = 0.05;
/// R37 low shelf corner, Hz.
pub const R37_HZ: f32 = 1000.0;
/// R37 low shelf depth at full counter-clockwise, dB.
pub const R37_DEPTH_DB: f32 = -10.0;

/// The resistive divider the cell sits in. Both optical models use the
/// same arrangement and the same cell; only the three resistances differ,
/// and on the LA-3A they differ by a few per cent
/// (`research/LA-3A.md` 3.3 and 7.5). Parameterising it here is what lets
/// the LA-3A share this file's [`Cell`] instead of copying it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Divider {
    /// The series resistor ahead of the cell.
    pub r_series: f32,
    /// The pot the cell shunts.
    pub r_pot: f32,
    /// The cell's resistance in full light, which sets the maximum
    /// reduction.
    pub r_min: f32,
}

impl Divider {
    /// The LA-2A's: R1 70.7 kΩ, the 100 kΩ Gain pot, 500 Ω at full light.
    pub const LA2A: Divider = Divider {
        r_series: R_SERIES,
        r_pot: R_POT,
        r_min: R_MIN,
    };

    /// Cell resistance for `n_f` free carriers.
    #[inline]
    pub fn resistance(&self, n_f: f32) -> f32 {
        let k_g = 1.0 / self.r_min - 1.0 / R_DARK;
        let g = 1.0 / R_DARK + k_g * n_f;
        (1.0 / g).clamp(self.r_min, R_DARK)
    }

    /// Divider gain with a dark cell: the normalisation.
    #[inline]
    pub fn a_dark(&self) -> f32 {
        let r_p = R_DARK * self.r_pot / (R_DARK + self.r_pot);
        r_p / (self.r_series + r_p)
    }

    /// Divider gain for a cell resistance, normalised so a dark cell is
    /// unity.
    #[inline]
    pub fn attenuation(&self, r_cell: f32) -> f32 {
        let r_p = r_cell * self.r_pot / (r_cell + self.r_pot);
        (r_p / (self.r_series + r_p)) / self.a_dark()
    }

    /// Gain reduction in dB (positive) for `n_f` free carriers.
    #[inline]
    pub fn gr_db(&self, n_f: f32) -> f32 {
        -20.0 * self.attenuation(self.resistance(n_f)).log10()
    }
}

/// Divider gain for a cell resistance, normalised so a dark cell is unity.
#[inline]
pub fn attenuation_for(r_cell: f32) -> f32 {
    let r_p = r_cell * R_POT / (r_cell + R_POT);
    let a_raw = r_p / (R_SERIES + r_p);
    a_raw / A_DARK
}

/// The divider gain with a dark cell (the normalisation).
pub const A_DARK: f32 = {
    let r_p = R_DARK * R_POT / (R_DARK + R_POT);
    r_p / (R_SERIES + r_p)
};

/// Gain reduction in dB (positive) for `n_f` free carriers.
#[inline]
pub fn gr_db_for(n_f: f32) -> f32 {
    -20.0 * attenuation_for(resistance_for(n_f)).log10()
}

/// Make-up gain in dB for the Gain knob position `p` in 0..1: +40 dB at
/// full, unity at 0.32, falling steeply below that (a log-taper pot).
#[inline]
pub fn makeup_db(p: f32) -> f32 {
    40.0 * (1.0 + 2.02 * p.max(1e-4).log10())
}

/// Sidechain gain for a Peak Reduction setting, given the calibration
/// offset `g0_db` (see `Compressor::calibrate`): 0.55 dB per unit, and the
/// pot's end below 12 fades to nothing, so PR 0 does not compress at any
/// level (the research: no reduction up to +16 with the knob at 0).
#[inline]
pub fn pr_gain(pr: f32, g0_db: f32) -> f32 {
    let end = (pr / 12.0).clamp(0.0, 1.0);
    10f32.powf((g0_db + PR_DB_PER_UNIT * pr) / 20.0) * end * end
}

/// Meter modes.
pub const METER_GR: usize = 0;
pub const METER_OUT10: usize = 1;
pub const METER_OUT4: usize = 2;

/// Everything the audio thread needs, as one `Copy` snapshot compared by
/// value (the hosts rebuild it every block and only reconfigure on change).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// Gain knob, 0..100.
    pub gain: f32,
    /// Peak Reduction knob, 0..100.
    pub peak_reduction: f32,
    /// `true` = Limit, `false` = Compress.
    pub limit: bool,
    /// [`METER_GR`], [`METER_OUT10`] or [`METER_OUT4`].
    pub meter: usize,
    /// R37 sidechain emphasis, 0 (full HF emphasis) .. 1 (flat).
    pub emphasis: f32,
    /// Cell speed variant, index into [`CELL_SPEEDS`].
    pub cell: usize,
    /// The front-panel meter-zero trim, in dB. A real screw pot that
    /// drifts about a decibel in the first hour, so the panel has one and
    /// so does this: it moves the needle, not the audio.
    pub meter_zero: f32,
    /// Stereo link.
    pub link: bool,
    /// Wet / dry mix, 0..1.
    pub mix: f32,
    /// Side-chain high-pass corner in Hz; below 10 Hz = off.
    pub sc_hpf: f32,
    /// Hard bypass.
    pub bypass: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            gain: 32.0,
            peak_reduction: 40.0,
            limit: false,
            meter: METER_GR,
            emphasis: 1.0,
            cell: 1,
            meter_zero: 0.0,
            link: true,
            mix: 1.0,
            sc_hpf: 0.0,
            bypass: false,
        }
    }
}

/// Per-channel filters and the make-up stage.
#[derive(Clone, Copy, Debug, Default)]
struct Channel {
    in_hp: OnePole,
    sc_hpf: Biquad,
    r37: Shelf,
    tilt_lo: Shelf,
    tilt_hi: Shelf,
    out_lp: OnePole,
}

impl Channel {
    fn set_sample_rate(&mut self, sr: f32) {
        self.in_hp.set(12.0, sr);
        self.out_lp.set(40_000.0_f32.min(sr * 0.45), sr);
        // Fixed sidechain tilt: about −4 dB at 100 Hz and +3 dB at 6 kHz
        // relative to 1 kHz, as two gentle first-order shelves.
        self.tilt_lo.set(300.0, -4.0, true, sr);
        self.tilt_hi.set(3000.0, 3.0, false, sr);
    }

    fn reset(&mut self) {
        self.in_hp.reset();
        self.sc_hpf.reset();
        self.r37.reset();
        self.tilt_lo.reset();
        self.tilt_hi.reset();
        self.out_lp.reset();
    }

    /// The sidechain shaping's magnitude at 1 kHz, used to keep the static
    /// curve consistent with the sample-domain filters.
    fn sidechain_gain_1k(&self, sr: f32) -> f32 {
        self.r37.gain(1000.0, sr) * self.tilt_gain_1k(sr)
    }

    /// The fixed tilt alone at 1 kHz (the calibration point ignores R37 so
    /// that the emphasis trimmer changes low-frequency sensitivity, not the
    /// 1 kHz threshold).
    fn tilt_gain_1k(&self, sr: f32) -> f32 {
        self.tilt_lo.gain(1000.0, sr) * self.tilt_hi.gain(1000.0, sr)
    }
}

/// The tube make-up stage's soft, slightly asymmetric clipper, normalised
/// to unity small-signal gain.
#[inline]
pub fn tube(w: f32) -> f32 {
    let bias = TUBE_BIAS / TUBE_K;
    let tb = (TUBE_K * bias).tanh();
    ((TUBE_K * (w + bias)).tanh() - tb) / (TUBE_K * (1.0 - tb * tb))
}

/// The stereo compressor.
pub struct Compressor {
    sr: f32,
    settings: Settings,
    ch: [Channel; 2],
    cells: [Cell; 2],
    g0_db: f32,
    pr_gain: f32,
    /// Sidechain saturation in sidechain volts, derived from the calibration.
    v_sat: f32,
    makeup: f32,
    beta: f32,
    /// Smoothed make-up gain (linear) to avoid zipper noise.
    makeup_z: f32,
    mix_z: f32,
    smooth_a: f32,
    /// Last block's mean gain reduction per channel, dB (positive).
    gr_db: [f32; 2],
    /// Output rectifier average for the meter, per block.
    out_abs: [f32; 2],
    in_peak: [f32; 2],
    out_peak: [f32; 2],
    block_n: u32,
}

impl Compressor {
    pub fn new(sr: f32) -> Self {
        let mut c = Compressor {
            sr,
            settings: Settings::default(),
            ch: [Channel::default(); 2],
            cells: [Cell::new(CellParams::GRAY, sr); 2],
            g0_db: 0.0,
            pr_gain: 1.0,
            v_sat: 1.0,
            makeup: 1.0,
            beta: 0.0,
            makeup_z: 1.0,
            mix_z: 1.0,
            smooth_a: 0.0,
            gr_db: [0.0; 2],
            out_abs: [0.0; 2],
            in_peak: [0.0; 2],
            out_peak: [0.0; 2],
            block_n: 0,
        };
        c.set_sample_rate(sr);
        c.configure(Settings::default());
        c.makeup_z = c.makeup;
        c
    }

    /// Recompute every coefficient for a new sample rate; state is kept.
    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sr = sr;
        for ch in &mut self.ch {
            ch.set_sample_rate(sr);
        }
        for cell in &mut self.cells {
            cell.set_sample_rate(sr);
        }
        // 5 ms parameter smoothing.
        self.smooth_a = 1.0 - (-1.0 / (0.005 * sr)).exp();
        let s = self.settings;
        self.apply(s);
    }

    /// Forget all state (a transport restart).
    pub fn reset(&mut self) {
        for ch in &mut self.ch {
            ch.reset();
        }
        for cell in &mut self.cells {
            cell.reset();
        }
        self.makeup_z = self.makeup;
        self.mix_z = self.settings.mix;
    }

    /// Apply new settings; returns whether anything changed (the hosts use
    /// that to republish the transfer curve).
    pub fn configure(&mut self, s: Settings) -> bool {
        if s == self.settings && self.pr_gain > 0.0 {
            return false;
        }
        self.apply(s);
        true
    }

    fn apply(&mut self, s: Settings) {
        self.settings = s;
        let sr = self.sr;
        for ch in &mut self.ch {
            ch.sc_hpf.set_highpass(s.sc_hpf, sr);
            ch.r37.set(
                R37_HZ,
                R37_DEPTH_DB * (1.0 - s.emphasis.clamp(0.0, 1.0)),
                true,
                sr,
            );
        }
        let params = cell_params_for(s.cell);
        for cell in &mut self.cells {
            cell.set_params(params);
        }
        self.beta = if s.limit { BETA_LIMIT } else { 0.0 };
        self.makeup = 10f32.powf(makeup_db(s.gain / 100.0) / 20.0);
        let (g0_db, u_onset) = self.calibrate();
        self.g0_db = g0_db;
        self.v_sat = V_SAT_OVER_ONSET * u_onset / SINE_MEAN_ABS;
        self.pr_gain = pr_gain(s.peak_reduction, self.g0_db);
    }

    /// The sidechain gain offset that puts [`CALIBRATION_GR_DB`] of gain
    /// reduction at 0 VU when Peak Reduction is at [`PR_CALIBRATION`], in
    /// Compress, for a 1 kHz sine. Everything else follows from the physics
    /// constants, so retuning them keeps the panel calibrated.
    fn calibrate(&self) -> (f32, f32) {
        // Carriers for the calibration gain reduction, by bisection on the
        // monotonic attenuator law.
        let (mut lo, mut hi) = (0.0f32, 1.0f32);
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if gr_db_for(mid) < CALIBRATION_GR_DB {
                lo = mid
            } else {
                hi = mid
            }
        }
        let n = 0.5 * (lo + hi);
        // Light for those carriers, drive for that light (inverse of the EL law).
        let light = (n / CellParams::GRAY.k_gen).powf(1.0 / CELL_GAMMA);
        let u = (EL_B / -light.ln()).powi(2);
        // Attenuated 0 VU sine through the (Compress) tap and the sidechain
        // shaping at 1 kHz, rectified and averaged.
        let a = attenuation_for(resistance_for(n));
        let shaping = self.ch[0].tilt_gain_1k(self.sr);
        let drive = SINE_MEAN_ABS * VU_REF_AMP * a * shaping;
        let g = u / drive;
        (20.0 * g.log10() - PR_DB_PER_UNIT * PR_CALIBRATION, u)
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// The gain reduction in dB (positive) the cell of `channel` currently
    /// applies; with `link` on both are equal.
    pub fn gain_reduction_db(&self, channel: usize) -> f32 {
        gr_db_for(self.cells[channel.min(1)].conductance())
    }

    /// The cell state for the "inside the T4" display: light, free and
    /// trapped carriers (all 0..1), of the first cell.
    pub fn cell_state(&self) -> [f32; 3] {
        let c = &self.cells[0];
        [c.light, c.conductance(), c.n_t]
    }

    /// Process one stereo block in place and refresh the meter values.
    /// Real-time safe.
    pub fn process_block(&mut self, l: &mut [f32], r: &mut [f32]) {
        let n = l.len().min(r.len());
        let s = self.settings;
        let mut gr_sum = [0.0f32; 2];
        let mut out_abs = [0.0f32; 2];
        let mut in_peak = [0.0f32; 2];
        let mut out_peak = [0.0f32; 2];
        for i in 0..n {
            let x = [l[i], r[i]];
            in_peak[0] = in_peak[0].max(x[0].abs());
            in_peak[1] = in_peak[1].max(x[1].abs());
            if s.bypass {
                // Keep the filters and the cell warm, so un-bypassing
                // resumes from the state the signal would have left rather
                // than from a stale one.
                for c in 0..2 {
                    self.ch[c].in_hp.hp(x[c]);
                }
                self.cells[0].step(0.0);
                self.cells[1].step(0.0);
                out_peak = in_peak;
                out_abs[0] += x[0].abs();
                out_abs[1] += x[1].abs();
                continue;
            }
            // Attenuation from the cell(s) as updated by the previous sample.
            let a = if s.link {
                let a = attenuation_for(self.cells[0].resistance());
                [a, a]
            } else {
                [
                    attenuation_for(self.cells[0].resistance()),
                    attenuation_for(self.cells[1].resistance()),
                ]
            };
            let mut v = [0.0f32; 2];
            let mut y = [0.0f32; 2];
            // One step per sample, not one per channel: this used to sit
            // inside the channel loop and ran at twice its intended rate.
            self.makeup_z += self.smooth_a * (self.makeup - self.makeup_z);
            for c in 0..2 {
                let ch = &mut self.ch[c];
                let xh = ch.in_hp.hp(x[c]);
                let mut att = xh * a[c];
                // Photocell cubic: odd-order distortion growing with GR.
                let k = CELL_CUBIC * (1.0 - a[c]);
                let q2 = (att / CELL_CUBIC_V0) * (att / CELL_CUBIC_V0);
                att *= 1.0 - k * q2 / (1.0 + q2);
                // Sidechain tap and shaping.
                let tap = (1.0 - self.beta) * att + self.beta * xh;
                let mut sc = ch.sc_hpf.process(tap) * self.pr_gain;
                sc = self.v_sat * (sc / self.v_sat).tanh();
                sc = ch.r37.process(sc);
                sc = ch.tilt_hi.process(ch.tilt_lo.process(sc));
                v[c] = sc;
                // Make-up, tube, transformer.
                let w = att * self.makeup_z;
                let z = ch.out_lp.lp(tube(w));
                y[c] = z;
            }
            // Cells: shared (mean sidechain, polarity sensitive) or per channel.
            if s.link {
                self.cells[0].step(0.5 * (v[0] + v[1]));
                self.cells[1] = self.cells[0];
            } else {
                self.cells[0].step(v[0]);
                self.cells[1].step(v[1]);
            }
            self.mix_z += self.smooth_a * (s.mix - self.mix_z);
            for c in 0..2 {
                let out = self.mix_z * y[c] + (1.0 - self.mix_z) * x[c];
                out_peak[c] = out_peak[c].max(out.abs());
                out_abs[c] += out.abs();
                gr_sum[c] += -20.0 * a[c].max(1e-6).log10();
            }
            l[i] = self.mix_z * y[0] + (1.0 - self.mix_z) * x[0];
            r[i] = self.mix_z * y[1] + (1.0 - self.mix_z) * x[1];
        }
        let inv = 1.0 / n.max(1) as f32;
        self.gr_db = [gr_sum[0] * inv, gr_sum[1] * inv];
        self.out_abs = [out_abs[0] * inv, out_abs[1] * inv];
        self.in_peak = in_peak;
        self.out_peak = out_peak;
        self.block_n = self.block_n.wrapping_add(1);
    }

    /// `[in_peak_l, in_peak_r, out_peak_l, out_peak_r, gr_db, meter_vu]` for
    /// the last block: peaks linear, `gr_db` the mean gain reduction (positive
    /// dB), `meter_vu` what the panel meter shows in its current mode (dB
    /// relative to 0 VU, or the negated gain reduction in GR mode).
    pub fn meter_frame(&self) -> [f32; 6] {
        let gr = 0.5 * (self.gr_db[0] + self.gr_db[1]);
        let zero = self.settings.meter_zero;
        let vu = match self.settings.meter {
            METER_OUT10 => vu_of(0.5 * (self.out_abs[0] + self.out_abs[1]), VU10_REF_AMP),
            METER_OUT4 => vu_of(0.5 * (self.out_abs[0] + self.out_abs[1]), VU_REF_AMP),
            _ => -gr,
        };
        // The trim moves the needle only.
        let vu = vu + zero;
        [
            self.in_peak[0],
            self.in_peak[1],
            self.out_peak[0],
            self.out_peak[1],
            gr,
            vu,
        ]
    }

    /// Static gain reduction (dB, positive) for a 1 kHz sine of peak
    /// amplitude `amp` at the input, at the current settings, from the
    /// steady-state loop (used for the transfer curve and by the tests).
    pub fn static_gr_db(&self, amp: f32) -> f32 {
        let shaping = self.ch[0].sidechain_gain_1k(self.sr);
        let params = cell_params_for(self.settings.cell);
        let mut n = 0.0f32;
        for _ in 0..80 {
            let a = attenuation_for(resistance_for(n));
            let tap = (1.0 - self.beta) * amp * a + self.beta * amp;
            let mut sc = tap * self.pr_gain;
            sc = self.v_sat * (sc / self.v_sat).tanh();
            let u = SINE_MEAN_ABS * sc * shaping;
            let target = Cell::carriers_for(Cell::light_for(u), &params);
            n = 0.5 * n + 0.5 * target;
        }
        gr_db_for(n)
    }

    /// Fill `out` with the transfer curve: output level in dBFS (sine RMS)
    /// for input levels spread from `min_dbfs` to `max_dbfs`.
    pub fn transfer_curve(&self, out: &mut [f32], min_dbfs: f32, max_dbfs: f32) {
        let n = out.len();
        let makeup_db = makeup_db(self.settings.gain / 100.0);
        for (i, o) in out.iter_mut().enumerate() {
            let db = min_dbfs + (max_dbfs - min_dbfs) * i as f32 / (n.max(2) - 1) as f32;
            let amp = 10f32.powf(db / 20.0) * std::f32::consts::SQRT_2;
            let gr = if self.settings.bypass {
                0.0
            } else {
                self.static_gr_db(amp)
            };
            let wet = db - gr + makeup_db;
            let mix = self.settings.mix;
            // Blend in the linear domain.
            let lin = mix * 10f32.powf(wet / 20.0) + (1.0 - mix) * 10f32.powf(db / 20.0);
            *o = 20.0 * lin.max(1e-9).log10();
        }
    }
}

/// VU reading (dB relative to 0 VU) of an average-responding rectifier for
/// the block mean `|y|`, given the peak amplitude of a sine at 0 VU.
#[inline]
pub fn vu_of(mean_abs: f32, ref_amp: f32) -> f32 {
    20.0 * (mean_abs / (SINE_MEAN_ABS * ref_amp)).max(1e-6).log10()
}

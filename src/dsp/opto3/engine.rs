//! The LA-3A engine, following `research/LA-3A.md` section 7.
//!
//! ```text
//! in ─► input transformer ─► T4B divider ─┬─► Gain ─► class-AB stage ─► output LP ─► mix ─► out
//!                                                │
//!                                                └─► Peak Reduction ─► 100 Hz ─► 30 Hz
//!                                                      ─► HF Contour ─► fixed tilt
//!                                                      ─► driver clipping ─► the cell's panel
//! ```
//!
//! The one decision that shapes the rest: **this model does not get a cell
//! of its own.** The hardware uses the same T4B module in the same role and
//! the divider around it is the LA-2A's circuit to within a few per cent,
//! so the cell is imported from [`crate::dsp::opto::model`] and only the
//! constants around it change. The two devices do not differ in the cell;
//! they differ in how hard and how fast it is lit, and in what the audio
//! passes through on its way out.
//!
//! Two things in the sidechain are the whole personality. A 4.7 nF coupling
//! capacitor and the driver's autotransformer make the detector deaf below
//! about 100 Hz, so bass does not pump the gain, and a variable emitter
//! network lifts the top by up to 10 dB at 15 kHz. The mid-forward
//! reputation falls out of those two, not out of any filter in the audio
//! path, which stays flat within a decibel from 20 Hz to 20 kHz.

use super::{METER_GR, METER_OFF, METER_OUT};
use crate::dsp::fet::oversample::{Downsampler, DryDelay, LATENCY, Upsampler};
use crate::dsp::opto::filters::{Biquad, OnePole, Shelf, flush};
use crate::dsp::opto::model::{
    CELL_GAMMA, Cell, CellParams, Divider, EL_B, SINE_MEAN_ABS, VU_REF_AMP, VU_REF_DBFS, distortion,
};

/// 0 VU in dBFS: the same reference the rest of the lab uses.
pub const VU_REFERENCE_DBFS: f32 = VU_REF_DBFS;
/// Mean rectified value of a sine at 0 VU, which the Output position of
/// the meter is calibrated against.
pub const VU_REF_MEAN: f32 = SINE_MEAN_ABS * VU_REF_AMP;

/// Constants of the model (`research/LA-3A.md` 7.5). Published figures are
/// named; everything else is an **estimate** tuned against section 8.
pub mod k {
    use super::Divider;

    /// The divider: R1 68 kΩ, R3 plus the 100 kΩ Gain pot, and 400 Ω at
    /// full light, which is what gives the specified 40 dB of reduction.
    pub const DIVIDER: Divider = Divider {
        r_series: 68_000.0,
        r_pot: 101_300.0,
        r_min: 400.0,
    };
    /// Carrier generation at full light. Far above the LA-2A's 7, and the
    /// dossier's starting estimate of 12, because it is what makes the
    /// model both reach the published 40 dB of reduction and attack three
    /// times faster than the LA-2A on the same step: a cell that makes more
    /// carriers of the light it is given settles higher and gets there
    /// sooner. The dossier marks this an **estimate** and says to tune it
    /// until the threshold and attack tests both pass, which is what this
    /// value is.
    pub const K_GEN: f32 = 90.0;
    /// Panel smoothing, seconds: a quarter of the LA-2A's 1 ms, because a
    /// transistor stage behind a step-up transformer charges the panel far
    /// faster than a pentode plate through 10 kΩ.
    pub const TAU_U: f32 = 0.00025;
    /// Peak Reduction: about 46 dB across the 0..10 panel knob. The
    /// dossier derives 40 dB; the extra few decibels are what put the
    /// manual's own working point (Peak Reduction 4 on a +6 dBu programme
    /// gives 3 to 5 dB of reduction) where the manual puts it, once the
    /// threshold is pinned at the other end of the knob. **Estimate.**
    pub const PR_DB_PER_UNIT: f32 = 4.6;
    /// The bottom of the pot's travel fades the sidechain to nothing, so
    /// Peak Reduction 0 does not compress at any level.
    pub const PR_END: f32 = 1.2;
    /// The published threshold of limiting: 1 dB of reduction at −30 dBu
    /// with Peak Reduction at 10 and the pad out. −30 dBu is 34 dB below
    /// 0 VU. The 30 dB range's −10 dBm then falls out of the pad alone.
    pub const CAL_PR: f32 = 10.0;
    /// The panel knob is a tenth of the parameter.
    pub const KNOB_SCALE: f32 = 0.1;
    pub const CAL_LEVEL_VU: f32 = -34.0;
    pub const CAL_GR_DB: f32 = 1.0;
    /// Driver headroom before it clips, as a multiple of the drive that
    /// just turns the panel on. **Estimate**, tuned against the published
    /// 40 dB of maximum reduction: it is the driver's clip point that
    /// decides how bright the panel can ever get, and therefore where the
    /// reduction stops.
    pub const V_SAT_OVER_ONSET: f32 = 120.0;
    /// Limit blends a little of the uncompressed signal into the
    /// side-chain, so that the loop's feedback term stops holding the
    /// reduction back once it is deep. **This coefficient is tuned against
    /// the published ratios, not derived from the schematic**: the scan
    /// will not resolve which terminal of the Comp/Limit switch is the
    /// common one, and the obvious reading predicts a difference of about a
    /// tenth of a decibel between the two modes, which cannot be right
    /// (`research/LA-3A.md` 3.5). Do not mistake it for a measured value.
    pub const BETA_COMPRESS: f32 = 0.0;
    pub const BETA_LIMIT: f32 = 0.16;
    /// The two sidechain high-passes that make the detector deaf to bass:
    /// the 4.7 nF coupling capacitor and the driver's autotransformer.
    pub const SC_HP1_HZ: f32 = 100.0;
    pub const SC_HP2_HZ: f32 = 30.0;
    /// HF Contour. The published figure is 10 dB *more gain reduction* at
    /// 15 kHz than low down, which is not the same as 10 dB of shelf: the
    /// loop turns side-chain decibels into rather fewer decibels of
    /// reduction, so the shelf has to be steeper than the specification it
    /// is meant to produce. **Estimate**, tuned against that measurement.
    pub const CONTOUR_HZ: f32 = 6000.0;
    pub const CONTOUR_MAX_DB: f32 = 16.0;
    /// The tilt that is there whatever the trimmer does.
    pub const TILT_HZ: f32 = 3000.0;
    pub const TILT_DB: f32 = 3.0;
    /// The audio band: ±1 dB from 20 Hz to 20 kHz.
    pub const IN_HP_HZ: f32 = 7.0;
    pub const OUT_LP_HZ: f32 = 50_000.0;
    /// Make-up: 50 dB at the top of the knob, unity at 4.1.
    pub const GAIN_MAX_DB: f32 = 50.0;
    pub const GAIN_LOG_K: f32 = 2.583;
    /// The output stage: a ceiling at the +27 dBm equivalent with a shape
    /// that is nearly transparent until it, unlike the LA-2A's tube, which
    /// starts bending at once. In this calibration a sine of peak `A` is
    /// `18.99 + 20·log10(A)` dBu, so +27 dBm is a peak of 2.51; the ceiling
    /// used to sit at 1.78, which is +24 dBm, and put 4.5 % of distortion
    /// exactly where the manual promises less than 0.35 %.
    pub const V_CLIP: f32 = 2.51;
    pub const CLIP_N: f32 = 8.0;
    /// The class-AB crossover deadband, referred to the output. It is
    /// odd-order and proportionally worse as the level falls, the opposite
    /// of every other nonlinearity in the lab, and a real property of a
    /// diode-biased complementary pair.
    pub const XOVER: f32 = 0.001;
    pub const XOVER_SOFT: f32 = 0.5;
    /// A small even-order term, so the model puts a second harmonic on the
    /// spectrum at all: the hardware makes four overtones and the shipping
    /// plug-ins are criticised for making only odd ones.
    pub const ASYM: f32 = 0.0016;
    /// The photocell's own distortion, a third of the LA-2A's.
    pub const CELL_CUBIC: f32 = 0.2;
    pub const CELL_CUBIC_V0: f32 = 0.25;
    /// Smoothing of the make-up and mix, seconds.
    pub const SMOOTH_S: f32 = 0.005;
}

/// How much of the light a cell of each age turns into carriers: a
/// depleted T4 makes far less of what it is given (`research/LA-3A.md`
/// 4.7, which says a cell-age control is as defensible here as on the
/// LA-2A, where the lab already has one).
pub const CELL_WEAR: [f32; 3] = [1.0, 0.6, 0.2];

/// The cell this model uses: the LA-2A's, with the faster panel and the
/// hotter generation the LA-3A's driver gives it, aged by `wear`.
pub fn cell_params(wear: usize) -> CellParams {
    CellParams {
        k_gen: k::K_GEN * CELL_WEAR[wear.min(CELL_WEAR.len() - 1)],
        tau_u: k::TAU_U,
        ..CellParams::GRAY
    }
}

/// Everything the engine needs from the parameters, read once per block.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// Gain knob, 0..100 (the panel prints 0 to 10, unity at 4.1 and
    /// +50 dB at 10).
    pub gain: f32,
    /// Peak Reduction knob, 0..100 (the panel prints 0 to 10).
    pub peak_reduction: f32,
    /// `true` = Limit, `false` = Compress.
    pub limit: bool,
    /// [`METER_GR`], [`METER_OUT`] or [`METER_OFF`].
    pub meter: usize,
    /// Cell age, an index into [`CELL_WEAR`].
    pub cell: usize,
    /// HF Contour: 0 = flat (as the trimmer ships), 1 = the full 10 dB of
    /// lift at 15 kHz in the side-chain. This is the **opposite sense to
    /// the LA-2A's `emphasis`**, where 1 is flat, because the two are
    /// different circuits and their panels are labelled differently. A
    /// copy-paste from the other engine would invert it silently, so there
    /// is a test that asserts the direction on its own.
    ///
    /// **A contradiction in the research, resolved here.** Its section 4.5
    /// concludes that 1 should be flat, matching the LA-2A, and its section
    /// 7.3 concludes the opposite and writes the equations and constants
    /// that way. This follows 7.3: it is the section that reasons from the
    /// emitter network rather than from the panel legend, and the sources
    /// themselves disagree about which way the screw turns.
    pub emphasis: f32,
    /// Share one cell between the channels.
    pub link: bool,
    /// Wet share, 0..1.
    pub mix: f32,
    /// Side-chain high-pass corner in Hz, 0 = off (the lab's shared knob,
    /// on top of the hardware's own two).
    pub sc_hpf: f32,
    pub bypass: bool,
}

impl Default for Settings {
    /// Universal Audio's suggested starting point, with the trimmer as it
    /// ships. The panel knobs read 0 to 10; the parameters are 0 to 100 so
    /// that they match the LA-2A's, so 32 and 40 here are panel 3.2 and
    /// 4.0.
    fn default() -> Self {
        Settings {
            gain: 32.0,
            peak_reduction: 40.0,
            limit: false,
            meter: METER_GR,
            cell: 0,
            emphasis: 0.0,
            link: true,
            mix: 1.0,
            sc_hpf: 0.0,
            bypass: false,
        }
    }
}

/// Make-up gain in dB for the Gain knob, 0..10: +50 dB at the top, unity
/// at 4.1.
#[inline]
pub fn gain_db(p: f32) -> f32 {
    k::GAIN_MAX_DB * (1.0 + k::GAIN_LOG_K * (p / 10.0).max(1e-5).log10())
}

/// The solid-state output stage: a crossover deadband, a ceiling that is
/// nearly transparent until it is reached, and a small even-order term.
#[inline]
pub fn amp(w: f32) -> f32 {
    let w = if w.abs() < k::XOVER {
        w * k::XOVER_SOFT
    } else {
        w - w.signum() * k::XOVER * (1.0 - k::XOVER_SOFT)
    };
    let u = (w / k::V_CLIP).abs().powf(k::CLIP_N);
    let z = w / (1.0 + u).powf(1.0 / k::CLIP_N);
    z + k::ASYM * z * z
}

#[derive(Clone, Default)]
struct Channel {
    in_hp: OnePole,
    sc_hpf: Biquad,
    sc_hp1: OnePole,
    sc_hp2: OnePole,
    contour: Shelf,
    tilt: Shelf,
    out_lp: OnePole,
    /// The output stage runs at twice the rate below 88.2 kHz
    /// (`research/LA-3A.md` 7.7). Measured, a 15 kHz tone at the published
    /// maximum output folds an alias to −47 dBFS without it.
    up: Upsampler,
    down: Downsampler,
    dry: DryDelay,
}

impl Channel {
    fn reset(&mut self) {
        self.in_hp.reset();
        self.sc_hpf.reset();
        self.sc_hp1.reset();
        self.sc_hp2.reset();
        self.contour.reset();
        self.tilt.reset();
        self.out_lp.reset();
        self.up.reset();
        self.down.reset();
        self.dry.reset();
    }
}

/// The stereo LA-3A.
pub struct Compressor {
    sr: f32,
    settings: Settings,
    ch: [Channel; 2],
    cells: [Cell; 2],
    pr_gain: f32,
    /// Sidechain gain offset in dB, solved once against the published
    /// threshold of limiting.
    g0_db: f32,
    /// The driver's clip point, in sidechain volts.
    v_sat: f32,
    makeup: f32,
    makeup_z: f32,
    mix_z: f32,
    beta: f32,
    smooth_a: f32,
    oversample: bool,
    gr_db: [f32; 2],
    out_abs: [f32; 2],
    in_peak: [f32; 2],
    out_peak: [f32; 2],
}

impl Compressor {
    pub fn new(sr: f32) -> Self {
        let s = Settings::default();
        let mut c = Compressor {
            sr,
            settings: s,
            ch: [Channel::default(), Channel::default()],
            cells: [Cell::new(cell_params(s.cell), sr); 2],
            pr_gain: 0.0,
            g0_db: 0.0,
            v_sat: 1.0,
            makeup: 1.0,
            makeup_z: 1.0,
            mix_z: 1.0,
            beta: k::BETA_COMPRESS,
            smooth_a: 0.0,
            oversample: sr < 88_200.0,
            gr_db: [0.0; 2],
            out_abs: [0.0; 2],
            in_peak: [0.0; 2],
            out_peak: [0.0; 2],
        };
        c.set_sample_rate(sr);
        c.calibrate();
        c.apply(s);
        c.reset();
        c
    }

    /// Solve the sidechain offset against the published threshold of
    /// limiting: 1 dB of gain reduction at −30 dBu with the panel's Peak
    /// Reduction at 10 (`research/LA-3A.md` 7.4). UREI's two published
    /// thresholds differ by exactly the rear 20 dB pad, so pinning the
    /// 50 dB position pins both, and the pad itself does not need to be a
    /// parameter: the model is fixed at 50 dB and −30 dBm.
    fn calibrate(&mut self) {
        // Carriers for 1 dB of reduction, by bisection on the monotonic
        // attenuator law.
        let (mut lo, mut hi) = (0.0f32, 1.0f32);
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if k::DIVIDER.gr_db(mid) < k::CAL_GR_DB {
                lo = mid
            } else {
                hi = mid
            }
        }
        let n = 0.5 * (lo + hi);
        // Light for those carriers, and the drive that produces that light
        // (the inverse of the electroluminescent law).
        let params = cell_params(0);
        // The photoconductor's gamma, by name: it is the component's
        // constant, and a second copy of it here would be free to drift.
        let light = (n / params.k_gen).powf(1.0 / CELL_GAMMA);
        let u = (EL_B / -light.ln()).powi(2);
        self.v_sat = u * k::V_SAT_OVER_ONSET;
        // What the sidechain sees for the specified input: the attenuated
        // sine's rectified mean through the fixed tilt (the two high-passes
        // and the contour are flat at 1 kHz).
        let amp_peak = VU_REF_AMP * 10f32.powf(k::CAL_LEVEL_VU / 20.0);
        let a = k::DIVIDER.attenuation(k::DIVIDER.resistance(n));
        let tilt = 10f32.powf(k::TILT_DB / 20.0);
        let drive = SINE_MEAN_ABS * amp_peak * a * tilt;
        let g = u / drive.max(1e-12);
        let end = (k::CAL_PR / k::PR_END).clamp(0.0, 1.0);
        self.g0_db = 20.0 * g.log10() - k::PR_DB_PER_UNIT * k::CAL_PR - 40.0 * end.log10();
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sr = sr;
        self.oversample = sr < 88_200.0;
        self.smooth_a = 1.0 - (-1.0 / (k::SMOOTH_S * sr)).exp();
        for cell in &mut self.cells {
            cell.set_sample_rate(sr);
        }
        let s = self.settings;
        self.rebuild(s);
        self.reset();
    }

    pub fn reset(&mut self) {
        for ch in &mut self.ch {
            ch.reset();
        }
        for cell in &mut self.cells {
            cell.reset();
        }
        self.makeup_z = self.makeup;
        self.mix_z = self.settings.mix;
        self.gr_db = [0.0; 2];
        self.out_abs = [0.0; 2];
        self.in_peak = [0.0; 2];
        self.out_peak = [0.0; 2];
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// The resamplers' round trip, when they are running.
    pub fn latency(&self) -> usize {
        if self.oversample { LATENCY } else { 0 }
    }

    /// The divider this model uses.
    pub fn divider(&self) -> Divider {
        k::DIVIDER
    }

    /// Apply a snapshot; `true` when anything changed.
    pub fn configure(&mut self, s: Settings) -> bool {
        if s == self.settings {
            return false;
        }
        let rebuild = s.emphasis != self.settings.emphasis || s.sc_hpf != self.settings.sc_hpf;
        let recell = s.cell != self.settings.cell;
        self.apply(s);
        if rebuild {
            self.rebuild(s);
        }
        if recell {
            let p = cell_params(s.cell);
            for cell in &mut self.cells {
                cell.set_params(p);
            }
        }
        true
    }

    fn apply(&mut self, s: Settings) {
        let pr = s.peak_reduction * k::KNOB_SCALE;
        let end = (pr / k::PR_END).clamp(0.0, 1.0);
        let db = self.g0_db + k::PR_DB_PER_UNIT * pr;
        self.pr_gain = 10f32.powf(db / 20.0) * end * end;
        self.makeup = 10f32.powf(gain_db(s.gain * k::KNOB_SCALE) / 20.0);
        self.beta = if s.limit {
            k::BETA_LIMIT
        } else {
            k::BETA_COMPRESS
        };
        self.settings = s;
    }

    fn rebuild(&mut self, s: Settings) {
        for ch in &mut self.ch {
            ch.in_hp.set(k::IN_HP_HZ, self.sr);
            ch.out_lp.set(k::OUT_LP_HZ.min(self.sr * 0.45), self.sr);
            ch.sc_hp1.set(k::SC_HP1_HZ, self.sr);
            ch.sc_hp2.set(k::SC_HP2_HZ, self.sr);
            // 0 is flat and 1 is the full lift: see `Settings::emphasis`.
            ch.contour.set(
                k::CONTOUR_HZ,
                k::CONTOUR_MAX_DB * s.emphasis.clamp(0.0, 1.0),
                false,
                self.sr,
            );
            ch.tilt.set(k::TILT_HZ, k::TILT_DB, false, self.sr);
            // Below 10 Hz this makes itself an identity.
            ch.sc_hpf.set_highpass(s.sc_hpf, self.sr);
        }
    }

    /// Gain reduction of one channel, in positive dB.
    pub fn gain_reduction_db(&self, channel: usize) -> f32 {
        k::DIVIDER.gr_db(self.cells[channel.min(1)].conductance())
    }

    /// `[light, free_carriers, trapped_carriers]` of the cell.
    pub fn cell_state(&self) -> [f32; 3] {
        let c = &self.cells[0];
        [c.light, c.conductance(), c.n_t]
    }

    /// Process one stereo block in place. Real-time safe.
    pub fn process_block(&mut self, l: &mut [f32], r: &mut [f32]) {
        let n = l.len().min(r.len());
        let s = self.settings;
        let mut gr_sum = [0.0f32; 2];
        let mut out_abs = [0.0f32; 2];
        let mut in_peak = [0.0f32; 2];
        let mut out_peak = [0.0f32; 2];
        // The meter's Off position bypasses the processing, as it does on
        // the plug-in this borrows it from.
        let processing = !s.bypass && s.meter != METER_OFF;
        let oversample = self.oversample;
        for i in 0..n {
            let x = [l[i], r[i]];
            in_peak[0] = in_peak[0].max(x[0].abs());
            in_peak[1] = in_peak[1].max(x[1].abs());
            if !processing {
                for c in 0..2 {
                    self.ch[c].in_hp.hp(x[c]);
                    if oversample {
                        self.ch[c].dry.process(x[c]);
                    }
                }
                out_peak = in_peak;
                out_abs[0] += x[0].abs();
                out_abs[1] += x[1].abs();
                continue;
            }
            // What the cell was doing at the end of the last sample.
            let a = if s.link {
                let a = k::DIVIDER.attenuation(k::DIVIDER.resistance(self.cells[0].conductance()));
                [a, a]
            } else {
                [
                    k::DIVIDER.attenuation(k::DIVIDER.resistance(self.cells[0].conductance())),
                    k::DIVIDER.attenuation(k::DIVIDER.resistance(self.cells[1].conductance())),
                ]
            };
            let mut v = [0.0f32; 2];
            let mut y = [0.0f32; 2];
            for c in 0..2 {
                let ch = &mut self.ch[c];
                let xh = ch.in_hp.hp(x[c]);
                let mut att = xh * a[c];
                // The photocell's own odd-order distortion, a third of the
                // LA-2A's.
                att = distortion(att, a[c], k::CELL_CUBIC, k::CELL_CUBIC_V0);
                // Sidechain: the tap, the user's high-pass, the pot, the
                // two high-passes the hardware has, the contour and the
                // fixed tilt, and the driver's clip last of all, because on
                // this unit the driver is the final stage before the
                // transformer.
                let tap = (1.0 - self.beta) * att + self.beta * xh;
                let mut sc = ch.sc_hpf.process(tap) * self.pr_gain;
                sc = ch.sc_hp1.hp(sc);
                sc = ch.sc_hp2.hp(sc);
                sc = ch.contour.process(sc);
                sc = ch.tilt.process(sc);
                sc = self.v_sat * (sc / self.v_sat).tanh();
                v[c] = sc;
                // Make-up and the transistor output stage, at twice the
                // rate so the ceiling does not fold.
                let w = att * self.makeup_z;
                let shaped = if oversample {
                    let pair = ch.up.process(w);
                    ch.down.process([amp(pair[0]), amp(pair[1])])
                } else {
                    amp(w)
                };
                y[c] = ch.out_lp.lp(shaped);
            }
            if s.link {
                self.cells[0].step(0.5 * (v[0] + v[1]));
                self.cells[1] = self.cells[0];
            } else {
                self.cells[0].step(v[0]);
                self.cells[1].step(v[1]);
            }
            self.mix_z += self.smooth_a * (s.mix - self.mix_z);
            self.makeup_z += self.smooth_a * (self.makeup - self.makeup_z);
            for c in 0..2 {
                // The dry path waits for the resamplers, so mix and bypass
                // stay phase-aligned.
                let dry_c = if oversample {
                    self.ch[c].dry.process(x[c])
                } else {
                    x[c]
                };
                let out = flush(self.mix_z * y[c] + (1.0 - self.mix_z) * dry_c);
                out_peak[c] = out_peak[c].max(out.abs());
                out_abs[c] += out.abs();
                gr_sum[c] += -20.0 * a[c].max(1e-6).log10();
                if c == 0 { l[i] = out } else { r[i] = out }
            }
        }
        let inv = 1.0 / n.max(1) as f32;
        self.gr_db = [gr_sum[0] * inv, gr_sum[1] * inv];
        self.out_abs = [out_abs[0] * inv, out_abs[1] * inv];
        self.in_peak = in_peak;
        self.out_peak = out_peak;
    }

    /// `[in_l, in_r, out_l, out_r, gr_db, meter_vu]` for the last block,
    /// with `gr_db` positive (the lab negates it).
    pub fn meter_frame(&self) -> [f32; 6] {
        let gr = 0.5 * (self.gr_db[0] + self.gr_db[1]);
        let mean = 0.5 * (self.out_abs[0] + self.out_abs[1]);
        let vu = match self.settings.meter {
            METER_OUT => 20.0 * (mean / VU_REF_MEAN).max(1e-4).log10(),
            METER_OFF => -60.0,
            _ => -gr,
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
    /// peak `amp_peak`, by running the loop to rest.
    pub fn static_gr_db(&self, amp_peak: f32) -> f32 {
        if self.settings.peak_reduction <= 0.0 || self.settings.bypass {
            return 0.0;
        }
        let mut cell = Cell::new(cell_params(self.settings.cell), self.sr);
        let tilt = 10f32.powf(k::TILT_DB / 20.0);
        let x = amp_peak;
        let mut last = 0.0f32;
        let steps = (self.sr * 3.0) as usize;
        for i in 0..steps {
            let a = k::DIVIDER.attenuation(k::DIVIDER.resistance(cell.conductance()));
            let att = x * a;
            let tap = (1.0 - self.beta) * att + self.beta * x;
            // A sine drives the panel with its rectified mean; the two
            // sidechain high-passes and the contour are flat at 1 kHz.
            let sc = tap * SINE_MEAN_ABS * self.pr_gain * tilt;
            cell.step(self.v_sat * (sc / self.v_sat).tanh());
            if i.is_multiple_of(2048) {
                let now = k::DIVIDER.gr_db(cell.conductance());
                if i > 8192 && (now - last).abs() < 1e-4 {
                    break;
                }
                last = now;
            }
        }
        k::DIVIDER.gr_db(cell.conductance())
    }

    /// Fill `out` with the static output level in dBFS for inputs from
    /// `min_dbfs` to `max_dbfs`.
    pub fn transfer_curve(&self, out: &mut [f32], min_dbfs: f32, max_dbfs: f32) {
        let n = out.len();
        if n == 0 {
            return;
        }
        let makeup = gain_db(self.settings.gain * k::KNOB_SCALE);
        for (i, o) in out.iter_mut().enumerate() {
            let x = min_dbfs + (max_dbfs - min_dbfs) * i as f32 / (n - 1).max(1) as f32;
            let gr = self.static_gr_db(10f32.powf(x / 20.0));
            let wet = x - gr + makeup;
            *o = if self.settings.bypass {
                x
            } else {
                let dry = 10f32.powf(x / 20.0);
                let w = 10f32.powf(wet / 20.0);
                20.0 * (dry + (w - dry) * self.settings.mix).max(1e-6).log10()
            };
        }
    }
}

//! Filter helpers for the VCA model: RBJ biquad sections (high-pass,
//! peaking, low-pass), a first-order high-pass for the side-chain, and a
//! one-pole for smoothing. The other two engines carry their own versions
//! of these; the sections each model needs differ enough (this one needs a
//! peaking band and a Bessel cascade, the 1176 needs shelves) that keeping
//! them apart is cheaper than one shared file with every variant in it.

use std::f32::consts::PI;

/// Second-order section, transposed direct form II.
#[derive(Clone, Copy, Debug, Default)]
pub struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    z1: f32,
    z2: f32,
}

impl Biquad {
    /// Pass-through.
    pub fn identity() -> Self {
        Biquad {
            b0: 1.0,
            ..Default::default()
        }
    }

    /// RBJ high-pass at `fc` Hz with quality `q`.
    pub fn highpass(sr: f32, fc: f32, q: f32) -> Self {
        let w0 = 2.0 * PI * (fc / sr).clamp(1e-6, 0.49);
        let (s, c) = w0.sin_cos();
        let alpha = s / (2.0 * q);
        let a0 = 1.0 + alpha;
        Biquad {
            b0: (1.0 + c) / 2.0 / a0,
            b1: -(1.0 + c) / a0,
            b2: (1.0 + c) / 2.0 / a0,
            a1: -2.0 * c / a0,
            a2: (1.0 - alpha) / a0,
            z1: 0.0,
            z2: 0.0,
        }
    }

    /// RBJ low-pass at `fc` Hz with quality `q`.
    pub fn lowpass(sr: f32, fc: f32, q: f32) -> Self {
        let w0 = 2.0 * PI * (fc / sr).clamp(1e-6, 0.49);
        let (s, c) = w0.sin_cos();
        let alpha = s / (2.0 * q);
        let a0 = 1.0 + alpha;
        Biquad {
            b0: (1.0 - c) / 2.0 / a0,
            b1: (1.0 - c) / a0,
            b2: (1.0 - c) / 2.0 / a0,
            a1: -2.0 * c / a0,
            a2: (1.0 - alpha) / a0,
            z1: 0.0,
            z2: 0.0,
        }
    }

    /// RBJ peaking band of `gain_db` at `fc` Hz with quality `q`: the
    /// side-chain's Band Emphasis (`research/Distressor.md` 7.7).
    pub fn peaking(sr: f32, fc: f32, q: f32, gain_db: f32) -> Self {
        let a = 10f32.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * (fc / sr).clamp(1e-6, 0.49);
        let (s, c) = w0.sin_cos();
        let alpha = s / (2.0 * q);
        let a0 = 1.0 + alpha / a;
        Biquad {
            b0: (1.0 + alpha * a) / a0,
            b1: -2.0 * c / a0,
            b2: (1.0 - alpha * a) / a0,
            a1: -2.0 * c / a0,
            a2: (1.0 - alpha / a) / a0,
            z1: 0.0,
            z2: 0.0,
        }
    }

    /// Copy another section's coefficients, keeping this one's state.
    pub fn set_from(&mut self, other: &Biquad) {
        self.b0 = other.b0;
        self.b1 = other.b1;
        self.b2 = other.b2;
        self.a1 = other.a1;
        self.a2 = other.a2;
    }

    pub fn reset(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
    }

    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.z1;
        self.z1 = flush(self.b1 * x - self.a1 * y + self.z2);
        self.z2 = flush(self.b2 * x - self.a2 * y);
        y
    }
}

/// First-order high-pass, for the side-chain's 100 Hz / 6 dB per octave
/// position (`research/Distressor.md` 7.7).
#[derive(Clone, Copy, Debug, Default)]
pub struct OnePoleHp {
    a: f32,
    z: f32,
}

impl OnePoleHp {
    /// Set the corner; `hz` at or below zero makes it a pass-through.
    pub fn set(&mut self, hz: f32, sr: f32) {
        self.a = if hz <= 0.0 {
            0.0
        } else {
            (-2.0 * PI * (hz / sr).min(0.49)).exp()
        };
    }

    pub fn reset(&mut self) {
        self.z = 0.0;
    }

    /// `y = x − lp(x)`: the input minus a one-pole low-pass of it.
    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        if self.a == 0.0 {
            return x;
        }
        self.z = flush(self.z + (1.0 - self.a) * (x - self.z));
        x - self.z
    }
}

/// One-pole low-pass, used for parameter smoothing and meter ballistics.
#[derive(Clone, Copy, Debug, Default)]
pub struct OnePole {
    a: f32,
    y: f32,
}

impl OnePole {
    pub fn new(sr: f32, tau: f32, init: f32) -> Self {
        OnePole {
            a: coefficient(sr, tau),
            y: init,
        }
    }

    pub fn set_tau(&mut self, sr: f32, tau: f32) {
        self.a = coefficient(sr, tau);
    }

    pub fn snap(&mut self, v: f32) {
        self.y = v;
    }

    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        self.y = flush(self.y + self.a * (x - self.y));
        self.y
    }

    pub fn value(&self) -> f32 {
        self.y
    }
}

/// Third-order Bessel high-pass as a first-order section plus a biquad:
/// the Audio HP position, 80 Hz at 18 dB per octave
/// (`research/Distressor.md` 7.7).
#[derive(Clone, Copy, Debug, Default)]
pub struct BesselHp3 {
    real: OnePoleHp,
    pair: Biquad,
    active: bool,
}

/// Quality of the third-order Bessel complex pair (from the normalised
/// poles −1.8389 ± j1.7544: `|p| / 2·Re(p)`).
pub const BESSEL3_Q: f32 = 0.691;
/// The pair's corner as a fraction of the −3 dB frequency (**derived**:
/// the cascade is −3 dB at 1.449 times the pair's corner).
pub const BESSEL3_PAIR: f32 = 0.690;
/// The real pole's corner as a fraction of the −3 dB frequency
/// (**derived**: 1.0945 times the pair's, from the normalised poles).
pub const BESSEL3_REAL: f32 = 0.755;

impl BesselHp3 {
    /// A third-order Bessel high-pass that is −3 dB at `f3` Hz: a real pole
    /// and a complex pair, 18 dB per octave in the stop band
    /// (`research/Distressor.md` 7.7, where the 80 Hz switch position is
    /// −3 dB at 65 Hz and about −12 dB at 30 Hz).
    pub fn new(sr: f32, f3: f32) -> Self {
        let mut real = OnePoleHp::default();
        real.set(f3 * BESSEL3_REAL, sr);
        BesselHp3 {
            real,
            pair: Biquad::highpass(sr, f3 * BESSEL3_PAIR, BESSEL3_Q),
            active: true,
        }
    }

    pub fn bypassed() -> Self {
        BesselHp3 {
            real: OnePoleHp::default(),
            pair: Biquad::identity(),
            active: false,
        }
    }

    /// Copy another cascade's coefficients, keeping this one's state.
    pub fn set_from(&mut self, other: &BesselHp3) {
        self.real.a = other.real.a;
        self.pair.set_from(&other.pair);
        self.active = other.active;
    }

    pub fn reset(&mut self) {
        self.real.reset();
        self.pair.reset();
    }

    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        if !self.active {
            return x;
        }
        self.pair.process(self.real.process(x))
    }
}

/// `1 − exp(−1 / (tau·sr))`: the per-sample step of a one-pole with time
/// constant `tau` seconds; 1 for a zero time constant.
#[inline]
pub fn coefficient(sr: f32, tau: f32) -> f32 {
    if tau <= 0.0 {
        1.0
    } else {
        1.0 - (-1.0 / (tau * sr)).exp()
    }
}

/// The one flush, defined in [`crate::dsp::flush`].
pub use crate::dsp::flush;

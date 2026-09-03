//! Small filter helpers: an RBJ biquad (transposed direct form II) for the
//! transformer high-passes and the side-chain high-pass, and a one-pole for
//! parameter smoothing and slow control voltages.

use std::f32::consts::PI;

/// Second-order section in transposed direct form II.
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

    /// RBJ high-pass at `fc` Hz with quality `q`, for sample rate `sr`.
    pub fn highpass(sr: f32, fc: f32, q: f32) -> Self {
        let w0 = 2.0 * PI * (fc / sr).min(0.49);
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

    /// RBJ high shelf at `fc` Hz of `gain_db` (shelf slope 1): the
    /// transformer / amplifier tilt of the model.
    pub fn highshelf(sr: f32, fc: f32, gain_db: f32) -> Self {
        let a = 10f32.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * (fc / sr).min(0.49);
        let (s, c) = w0.sin_cos();
        let alpha = s / 2.0 * std::f32::consts::SQRT_2;
        let sa = 2.0 * a.sqrt() * alpha;
        let a0 = (a + 1.0) - (a - 1.0) * c + sa;
        Biquad {
            b0: a * ((a + 1.0) + (a - 1.0) * c + sa) / a0,
            b1: -2.0 * a * ((a - 1.0) + (a + 1.0) * c) / a0,
            b2: a * ((a + 1.0) + (a - 1.0) * c - sa) / a0,
            a1: 2.0 * ((a - 1.0) - (a + 1.0) * c) / a0,
            a2: ((a + 1.0) - (a - 1.0) * c - sa) / a0,
            z1: 0.0,
            z2: 0.0,
        }
    }

    /// RBJ low-pass at `fc` Hz with quality `q`.
    pub fn lowpass(sr: f32, fc: f32, q: f32) -> Self {
        let w0 = 2.0 * PI * (fc / sr).min(0.49);
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

    /// Replace the coefficients, keeping the state (for a corner that moves
    /// while running).
    pub fn set_from(&mut self, other: &Biquad) {
        self.b0 = other.b0;
        self.b1 = other.b1;
        self.b2 = other.b2;
        self.a1 = other.a1;
        self.a2 = other.a2;
    }

    /// Clear the state.
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

/// One-pole smoother / low-pass: `y += a·(x − y)`.
#[derive(Clone, Copy, Debug, Default)]
pub struct OnePole {
    a: f32,
    y: f32,
}

impl OnePole {
    /// Time constant `tau` seconds at sample rate `sr`, starting at `init`.
    pub fn new(sr: f32, tau: f32, init: f32) -> Self {
        OnePole {
            a: coefficient(sr, tau),
            y: init,
        }
    }

    pub fn set_tau(&mut self, sr: f32, tau: f32) {
        self.a = coefficient(sr, tau);
    }

    /// Jump to a value without smoothing.
    pub fn snap(&mut self, v: f32) {
        self.y = v;
    }

    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        self.y += self.a * (x - self.y);
        self.y
    }

    #[inline]
    pub fn value(&self) -> f32 {
        self.y
    }
}

/// `1 − exp(−1 / (tau·sr))`, the per-sample step of a one-pole with time
/// constant `tau` seconds; 1 for a zero time constant.
#[inline]
pub fn coefficient(sr: f32, tau: f32) -> f32 {
    if tau <= 0.0 {
        1.0
    } else {
        1.0 - (-1.0 / (tau * sr)).exp()
    }
}

/// Flush denormals and tiny values to zero (the audio thread runs in f32
/// and the detector decays exponentially towards zero).
#[inline]
pub fn flush(x: f32) -> f32 {
    if x.abs() < 1e-9 { 0.0 } else { x }
}

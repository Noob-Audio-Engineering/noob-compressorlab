//! Small first- and second-order filters used by the model: the transformer
//! band limits, the R37 low shelf, the fixed sidechain tilt and the modern
//! side-chain high-pass. All are transposed direct-form recursions with
//! coefficients recomputed from the sample rate, so behaviour is
//! rate-independent, and none allocates.

use std::f32::consts::PI;

/// First-order one-pole low-pass (or, through [`OnePole::hp`], high-pass).
#[derive(Clone, Copy, Debug, Default)]
pub struct OnePole {
    a: f32,
    z: f32,
}

impl OnePole {
    /// Set the corner frequency in Hz at `sr` (coefficient `1 − exp(−2πf/sr)`).
    pub fn set(&mut self, hz: f32, sr: f32) {
        self.a = 1.0 - (-2.0 * PI * hz / sr).exp();
    }

    /// Low-pass output for one sample.
    #[inline]
    pub fn lp(&mut self, x: f32) -> f32 {
        self.z += self.a * (x - self.z);
        self.z
    }

    /// High-pass output for one sample (input minus the low-pass).
    #[inline]
    pub fn hp(&mut self, x: f32) -> f32 {
        x - self.lp(x)
    }

    /// Forget the state.
    pub fn reset(&mut self) {
        self.z = 0.0;
    }

    /// Magnitude of the low-pass response at `hz` (for the static curve).
    pub fn lp_gain(&self, hz: f32, sr: f32) -> f32 {
        // H(z) = a / (1 - (1 - a) z^-1)
        let w = 2.0 * PI * hz / sr;
        let (c, s) = (w.cos(), w.sin());
        let re = 1.0 - (1.0 - self.a) * c;
        let im = (1.0 - self.a) * s;
        self.a / (re * re + im * im).sqrt()
    }
}

/// First-order shelving filter: `gain_db` below (`low` = true) or above the
/// corner, unity on the other side. Built as a one-pole split: the shelved
/// band is scaled by `g`, the rest passes.
#[derive(Clone, Copy, Debug, Default)]
pub struct Shelf {
    lp: OnePole,
    g: f32,
    low: bool,
}

impl Shelf {
    /// A low shelf (`low = true`) or high shelf with `gain_db` at `hz`.
    pub fn set(&mut self, hz: f32, gain_db: f32, low: bool, sr: f32) {
        self.lp.set(hz, sr);
        self.g = 10f32.powf(gain_db / 20.0);
        self.low = low;
    }

    /// One sample.
    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        let l = self.lp.lp(x);
        let h = x - l;
        if self.low {
            self.g * l + h
        } else {
            l + self.g * h
        }
    }

    /// Approximate magnitude at `hz` (the two bands are added as
    /// magnitudes; exact enough for the 1 kHz calibration point).
    pub fn gain(&self, hz: f32, sr: f32) -> f32 {
        let l = self.lp.lp_gain(hz, sr);
        let h = (1.0 - l * l).max(0.0).sqrt();
        if self.low {
            (self.g * l).hypot(h)
        } else {
            l.hypot(self.g * h)
        }
    }

    pub fn reset(&mut self) {
        self.lp.reset();
    }
}

/// Second-order Butterworth high-pass (the modern side-chain filter),
/// transposed direct form II.
#[derive(Clone, Copy, Debug, Default)]
pub struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    z1: f32,
    z2: f32,
    /// `true` when the filter is an identity (bypassed).
    pub identity: bool,
}

impl Biquad {
    /// Butterworth high-pass at `hz` (RBJ cookbook, Q = 1/√2); `hz` below
    /// 10 Hz makes the filter an identity.
    pub fn set_highpass(&mut self, hz: f32, sr: f32) {
        if hz < 10.0 {
            self.identity = true;
            return;
        }
        self.identity = false;
        let w0 = 2.0 * PI * hz / sr;
        let (s, c) = (w0.sin(), w0.cos());
        let alpha = s / (2.0 * std::f32::consts::FRAC_1_SQRT_2);
        let a0 = 1.0 + alpha;
        self.b0 = (1.0 + c) / 2.0 / a0;
        self.b1 = -(1.0 + c) / a0;
        self.b2 = (1.0 + c) / 2.0 / a0;
        self.a1 = -2.0 * c / a0;
        self.a2 = (1.0 - alpha) / a0;
    }

    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        if self.identity {
            return x;
        }
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        y
    }

    pub fn reset(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
    }
}

/// Flush a state variable that has decayed below `1e-12` to exactly zero, so
/// long silence after heavy compression cannot leave a denormal behind.
#[inline]
pub fn flush(x: f32) -> f32 {
    if x.abs() < 1e-12 { 0.0 } else { x }
}

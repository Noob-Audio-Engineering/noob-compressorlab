//! Sections the 610 stage is built from: the first-order shelf the
//! feedback equaliser behaves like, first-order low- and high-passes for
//! the transformers, a second-order high-pass for the low cut, a leaky
//! integrator for the output core's flux, and the VU ballistics of the
//! panel meter.

use std::f32::consts::PI;

/// A first-order shelf. The printed corner is the **half-gain** point,
/// which is the geometric mean of the zero and the pole, so a ±9 dB step
/// is ±4.5 dB at the number on the panel and reaches its full value about
/// a decade away (`research/610.md` 8.4). A 0 dB step is an exact
/// pass-through.
#[derive(Clone, Copy, Debug, Default)]
pub struct Shelf {
    b0: f32,
    b1: f32,
    a1: f32,
    g: f32,
    x1: f32,
    y1: f32,
    flat: bool,
}

impl Shelf {
    /// `low` picks a low shelf (gain at DC, unity above) rather than a high
    /// one (unity at DC, gain above).
    pub fn new(sr: f32, f0: f32, gain_db: f32, low: bool) -> Self {
        if gain_db.abs() < 1e-6 {
            return Shelf {
                b0: 1.0,
                flat: true,
                g: 1.0,
                ..Default::default()
            };
        }
        let a = 10f32.powf(gain_db / 20.0);
        // A high shelf of ratio `r`; a low shelf is the same shape with the
        // reciprocal ratio and an overall gain.
        let (r, g) = if low { (1.0 / a, a) } else { (a, 1.0) };
        let rs = r.sqrt();
        let w = 2.0 * PI * (f0 / sr).clamp(1e-6, 0.49);
        let kk = (w * 0.5).tan();
        // s → (1/K)(1 − z⁻¹)/(1 + z⁻¹), prewarped at the printed corner.
        let (ra, rb) = (rs / kk, (1.0 / rs) / kk);
        let d = 1.0 + rb;
        Shelf {
            b0: (1.0 + ra) / d,
            b1: (1.0 - ra) / d,
            a1: (1.0 - rb) / d,
            g,
            x1: 0.0,
            y1: 0.0,
            flat: false,
        }
    }

    pub fn set_from(&mut self, other: &Shelf) {
        self.b0 = other.b0;
        self.b1 = other.b1;
        self.a1 = other.a1;
        self.g = other.g;
        self.flat = other.flat;
    }

    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.y1 = 0.0;
    }

    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        if self.flat {
            return x;
        }
        let y = self.b0 * x + self.b1 * self.x1 - self.a1 * self.y1;
        self.x1 = x;
        self.y1 = flush(y);
        y * self.g
    }
}

/// First-order low-pass, for the transformer roll-offs.
#[derive(Clone, Copy, Debug, Default)]
pub struct Lp1 {
    g: f32,
    z: f32,
}

impl Lp1 {
    /// Set the corner, prewarped so the pole lands where the circuit puts
    /// it at every sample rate.
    ///
    /// This used to take the impulse-invariant coefficient
    /// `1 − exp(−2π·f/sr)`, which is a good match to the analogue pole only
    /// while the corner sits well below Nyquist. The transformer roll-offs
    /// do not: at 40 kHz they are a fifth of the rate when the stage
    /// oversamples and nearly half of it when it does not, so the same
    /// printed corner produced a different response at every rate, and the
    /// 20 kHz figure moved by more than 3 dB across the rates we support.
    /// A prewarped bilinear pole gives the analogue response at all of
    /// them.
    ///
    /// A corner at or above Nyquist cannot be represented, and prewarping
    /// it would diverge, so it is clamped just below. That is the right
    /// behaviour rather than a compromise: a pole above Nyquist does
    /// essentially nothing inside the audio band, which is what the clamped
    /// filter also does. The voicings stay distinct where it matters,
    /// because the rate is oversampled whenever the base rate is low enough
    /// for a clamp to reach into the band.
    pub fn set(&mut self, hz: f32, sr: f32) {
        let g = (PI * (hz / sr).clamp(0.0, 0.4999)).tan();
        self.g = g / (1.0 + g);
    }

    /// Exact pass-through, for a position that has no roll-off at all.
    ///
    /// Standing in for that with a corner far above the rate used to be
    /// harmless, but a prewarped pole clamps at Nyquist, so two different
    /// "very high" corners land in the same place and the switch between
    /// them does nothing. Saying pass-through outright avoids relying on a
    /// number that no longer means what it did.
    pub fn bypass(&mut self) {
        self.g = 1.0;
    }

    pub fn reset(&mut self) {
        self.z = 0.0;
    }

    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        let v = (x - self.z) * self.g;
        let y = v + self.z;
        self.z = flush(y + v);
        y
    }
}

/// First-order high-pass: `x − lp(x)`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Hp1 {
    lp: Lp1,
    active: bool,
}

impl Hp1 {
    pub fn set(&mut self, hz: f32, sr: f32) {
        self.active = hz > 0.0;
        if self.active {
            self.lp.set(hz, sr);
        }
    }

    pub fn reset(&mut self) {
        self.lp.reset();
    }

    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        if !self.active {
            return x;
        }
        x - self.lp.process(x)
    }
}

/// Second-order high-pass (RBJ), for the input transformer's 12 Hz corner
/// and the optional 75 Hz low cut.
#[derive(Clone, Copy, Debug, Default)]
pub struct Hp2 {
    g: f32,
    k: f32,
    a1: f32,
    a2: f32,
    a3: f32,
    ic1: f32,
    ic2: f32,
    active: bool,
}

impl Hp2 {
    /// Design the high-pass, in a form that survives a 12 Hz corner at
    /// 192 kHz.
    ///
    /// This was a cookbook biquad in transposed direct form, and at high
    /// rates it fell apart. A 12 Hz corner at 192 kHz puts the poles about
    /// three ten-thousandths inside the unit circle, and the direct form's
    /// `a1` and `a2` then differ from ±2 and 1 by less than a single-precision
    /// mantissa can resolve, so rounding moved the poles by an appreciable
    /// fraction of their own distance from the circle. Measured, the stage
    /// came out **24 dB up at 100 Hz** at 192 kHz, and the error grew with
    /// the rate: it was already visible at 96 kHz.
    ///
    /// The state-variable form below keeps its coefficients away from those
    /// cancellations, so the same corner is accurate at every rate we
    /// support. It is the standard remedy for exactly this failure.
    pub fn new(sr: f32, fc: f32, q: f32) -> Self {
        let g = (PI * (fc / sr).clamp(1e-9, 0.4999)).tan();
        let k = 1.0 / q.max(1e-4);
        let a1 = 1.0 / (1.0 + g * (g + k));
        Hp2 {
            g,
            k,
            a1,
            a2: g * a1,
            a3: g * (g * a1),
            ic1: 0.0,
            ic2: 0.0,
            active: true,
        }
    }

    pub fn bypassed() -> Self {
        Hp2 {
            active: false,
            ..Default::default()
        }
    }

    pub fn set_from(&mut self, other: &Hp2) {
        self.g = other.g;
        self.k = other.k;
        self.a1 = other.a1;
        self.a2 = other.a2;
        self.a3 = other.a3;
        self.active = other.active;
    }

    pub fn reset(&mut self) {
        self.ic1 = 0.0;
        self.ic2 = 0.0;
    }

    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        if !self.active {
            return x;
        }
        let v3 = x - self.ic2;
        let v1 = self.a1 * self.ic1 + self.a2 * v3;
        let v2 = self.ic2 + self.a2 * self.ic1 + self.a3 * v3;
        self.ic1 = flush(2.0 * v1 - self.ic1);
        self.ic2 = flush(2.0 * v2 - self.ic2);
        x - self.k * v1 - v2
    }
}

/// One-pole smoother for continuous parameters.
#[derive(Clone, Copy, Debug, Default)]
pub struct Smooth {
    a: f32,
    y: f32,
}

impl Smooth {
    pub fn set(&mut self, sr: f32, tau: f32) {
        self.a = if tau <= 0.0 {
            1.0
        } else {
            1.0 - (-1.0 / (tau * sr)).exp()
        };
    }

    pub fn snap(&mut self, v: f32) {
        self.y = v;
    }

    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        self.y = flush(self.y + self.a * (x - self.y));
        self.y
    }
}

/// A VU meter's needle: a second-order low-pass with the standard 300 ms
/// rise and a little overshoot, fed the rectified average.
#[derive(Clone, Copy, Debug, Default)]
pub struct Vu {
    y: f32,
    v: f32,
    w0: f32,
    zeta: f32,
    dt: f32,
}

impl Vu {
    pub fn set_sample_rate(&mut self, sr: f32, block: f32) {
        // Driven once per block of `block` seconds.
        self.dt = block.max(1.0 / sr);
        self.w0 = 13.0;
        self.zeta = 0.8;
    }

    pub fn reset(&mut self) {
        self.y = 0.0;
        self.v = 0.0;
    }

    /// Step the needle towards `target` (a linear rectified average) over
    /// `dt` seconds and return the new position.
    pub fn step(&mut self, target: f32, dt: f32) -> f32 {
        let dt = dt.clamp(1e-6, 0.1);
        let a = self.w0 * self.w0 * (target - self.y) - 2.0 * self.zeta * self.w0 * self.v;
        self.v += a * dt;
        self.y = flush(self.y + self.v * dt);
        self.y
    }

    pub fn value(&self) -> f32 {
        self.y
    }
}

/// The one flush, defined in [`crate::dsp::flush`].
pub use crate::dsp::flush;

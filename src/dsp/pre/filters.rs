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
    a: f32,
    z: f32,
}

impl Lp1 {
    pub fn set(&mut self, hz: f32, sr: f32) {
        // Stable for any positive corner, including the transformer
        // roll-offs that sit above Nyquist; clamping here would erase the
        // difference between the voicings.
        self.a = 1.0 - (-2.0 * PI * (hz / sr).max(0.0)).exp();
    }

    pub fn reset(&mut self) {
        self.z = 0.0;
    }

    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        self.z = flush(self.z + self.a * (x - self.z));
        self.z
    }

    /// The low-passed value alone (the flux of the output core).
    pub fn value(&self) -> f32 {
        self.z
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
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    z1: f32,
    z2: f32,
    active: bool,
}

impl Hp2 {
    pub fn new(sr: f32, fc: f32, q: f32) -> Self {
        let w0 = 2.0 * PI * (fc / sr).clamp(1e-6, 0.49);
        let (s, c) = w0.sin_cos();
        let alpha = s / (2.0 * q);
        let a0 = 1.0 + alpha;
        Hp2 {
            b0: (1.0 + c) / 2.0 / a0,
            b1: -(1.0 + c) / a0,
            b2: (1.0 + c) / 2.0 / a0,
            a1: -2.0 * c / a0,
            a2: (1.0 - alpha) / a0,
            z1: 0.0,
            z2: 0.0,
            active: true,
        }
    }

    pub fn bypassed() -> Self {
        Hp2 {
            b0: 1.0,
            active: false,
            ..Default::default()
        }
    }

    pub fn set_from(&mut self, other: &Hp2) {
        self.b0 = other.b0;
        self.b1 = other.b1;
        self.b2 = other.b2;
        self.a1 = other.a1;
        self.a2 = other.a2;
        self.active = other.active;
    }

    pub fn reset(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
    }

    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        if !self.active {
            return x;
        }
        let y = self.b0 * x + self.z1;
        self.z1 = flush(self.b1 * x - self.a1 * y + self.z2);
        self.z2 = flush(self.b2 * x - self.a2 * y);
        y
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

/// Flush denormals and tiny values to zero.
#[inline]
pub fn flush(x: f32) -> f32 {
    if x.abs() < 1e-9 { 0.0 } else { x }
}

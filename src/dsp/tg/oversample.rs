//! 1×, 2× and 4× oversampling for the TG's gain element and its sidechain.
//!
//! The lab's other models oversample at a fixed 2× decided by the host
//! rate, because their dossiers ask for one factor. `research/TG12413.md`
//! section 11.3 asks for a control with three positions and 11.8 explains
//! why: the element is a static nonlinearity fed with full-band audio, so
//! it aliases, and the dossier judges 2× plus a decent kernel the right
//! trade for a bus processor while leaving 4× available.
//!
//! # Why this is not the 1176's oversampler
//!
//! Two reasons, and the second is the real one.
//!
//! The 1176's stage is a single 2× half-band and there is no second
//! factor. Cascading it with itself would work, but its 63 taps put the
//! second stage's round trip at 15.5 samples of the host rate, so a 4×
//! chain would land on 46.5 samples and no whole-sample dry delay could
//! align with it. A **61**-tap kernel gives 30 samples at 2× and 45 at 4×,
//! both whole, and the same kernel serves both stages.
//!
//! That matters because the dry path has to be held back by exactly the
//! round trip or the mix control combs itself, and this model's mix is a
//! control the user turns.
//!
//! # The sidechain runs inside the loop
//!
//! Section 11.4 is explicit that the detector runs at the oversampled rate
//! and that its ripple is not to be filtered away, because the ripple
//! modulating the gain is a large part of what the manufacturers call
//! squishy. So the caller's per-sample work happens between [`Chain::up`]
//! and [`Chain::down`], not around them.

/// Filter length, odd so the group delay is a whole number of samples at
/// the rate the filter runs at, and ≡ 1 (mod 4) so that a two-stage chain
/// is a whole number of samples at the **host** rate as well.
pub const TAPS: usize = 61;

/// Round-trip latency in host samples at 2×.
pub const LATENCY_2X: usize = (TAPS - 1) / 2;

/// Round-trip latency in host samples at 4×: the first stage's 30 plus
/// the second stage's 15.
pub const LATENCY_4X: usize = LATENCY_2X + LATENCY_2X / 2;

/// The longest dry delay [`Delay`] has to cover.
const MAX_DELAY: usize = LATENCY_4X + 1;

/// Latency in host samples for an oversampling factor.
pub fn latency(factor: usize) -> usize {
    match factor {
        2 => LATENCY_2X,
        4 => LATENCY_4X,
        _ => 0,
    }
}

/// The half-band coefficients: `0.5·sinc((k − c) / 2)` under a Blackman
/// window, normalised to unity DC gain.
fn coefficients() -> [f32; TAPS] {
    let c = (TAPS - 1) as f32 / 2.0;
    let mut h = [0.0f32; TAPS];
    let mut sum = 0.0;
    for (k, hk) in h.iter_mut().enumerate() {
        let n = k as f32 - c;
        let sinc = if n == 0.0 {
            1.0
        } else {
            (std::f32::consts::PI * n / 2.0).sin() / (std::f32::consts::PI * n / 2.0)
        };
        let w = 0.42 - 0.5 * (2.0 * std::f32::consts::PI * k as f32 / (TAPS - 1) as f32).cos()
            + 0.08 * (4.0 * std::f32::consts::PI * k as f32 / (TAPS - 1) as f32).cos();
        *hk = 0.5 * sinc * w;
        sum += *hk;
    }
    for hk in h.iter_mut() {
        *hk /= sum;
    }
    h
}

#[derive(Clone)]
struct Fir {
    h: [f32; TAPS],
    buf: [f32; TAPS],
    pos: usize,
}

impl Fir {
    fn new() -> Self {
        Fir {
            h: coefficients(),
            buf: [0.0; TAPS],
            pos: 0,
        }
    }

    #[inline]
    fn push(&mut self, x: f32) -> f32 {
        self.buf[self.pos] = x;
        let mut acc = 0.0;
        let mut i = self.pos;
        for &hk in &self.h {
            acc += hk * self.buf[i];
            i = if i == 0 { TAPS - 1 } else { i - 1 };
        }
        self.pos = if self.pos + 1 == TAPS {
            0
        } else {
            self.pos + 1
        };
        acc
    }

    fn reset(&mut self) {
        self.buf = [0.0; TAPS];
        self.pos = 0;
    }
}

/// A one-in, `factor`-out and `factor`-in, one-out resampling chain.
///
/// At 1× both directions are the identity and nothing is filtered, so the
/// position really is off rather than transparent-by-accident.
#[derive(Clone)]
pub struct Chain {
    factor: usize,
    up1: Fir,
    up2: Fir,
    dn1: Fir,
    dn2: Fir,
}

impl Chain {
    pub fn new() -> Self {
        Chain {
            factor: 2,
            up1: Fir::new(),
            up2: Fir::new(),
            dn1: Fir::new(),
            dn2: Fir::new(),
        }
    }

    /// Set the factor (1, 2 or 4). Anything else is treated as 1.
    pub fn set_factor(&mut self, factor: usize) {
        let f = match factor {
            2 => 2,
            4 => 4,
            _ => 1,
        };
        if f != self.factor {
            self.factor = f;
            self.reset();
        }
    }

    /// The factor in force, and the number of meaningful entries in the
    /// arrays [`up`](Self::up) fills and [`down`](Self::down) reads.
    pub fn factor(&self) -> usize {
        self.factor
    }

    pub fn reset(&mut self) {
        self.up1.reset();
        self.up2.reset();
        self.dn1.reset();
        self.dn2.reset();
    }

    /// One host sample in, `factor` oversampled samples out (the rest of
    /// the array is left at zero).
    #[inline]
    pub fn up(&mut self, x: f32) -> [f32; 4] {
        let mut out = [0.0f32; 4];
        match self.factor {
            2 => {
                out[0] = 2.0 * self.up1.push(x);
                out[1] = 2.0 * self.up1.push(0.0);
            }
            4 => {
                let a0 = 2.0 * self.up1.push(x);
                let a1 = 2.0 * self.up1.push(0.0);
                out[0] = 2.0 * self.up2.push(a0);
                out[1] = 2.0 * self.up2.push(0.0);
                out[2] = 2.0 * self.up2.push(a1);
                out[3] = 2.0 * self.up2.push(0.0);
            }
            _ => out[0] = x,
        }
        out
    }

    /// `factor` oversampled samples in, one host sample out.
    #[inline]
    pub fn down(&mut self, y: &[f32; 4]) -> f32 {
        match self.factor {
            2 => {
                let a = self.dn1.push(y[0]);
                self.dn1.push(y[1]);
                a
            }
            4 => {
                let b0 = self.dn2.push(y[0]);
                self.dn2.push(y[1]);
                let b1 = self.dn2.push(y[2]);
                self.dn2.push(y[3]);
                let a = self.dn1.push(b0);
                self.dn1.push(b1);
                a
            }
            _ => y[0],
        }
    }
}

impl Default for Chain {
    fn default() -> Self {
        Self::new()
    }
}

/// A whole-sample delay of settable length, for the dry path.
///
/// The 1176's `DryDelay` is fixed at its own oversampler's round trip;
/// this one has three lengths to cover because the factor is a control.
#[derive(Clone, Copy, Debug)]
pub struct Delay {
    buf: [f32; MAX_DELAY + 1],
    len: usize,
    pos: usize,
}

impl Delay {
    pub fn new() -> Self {
        Delay {
            buf: [0.0; MAX_DELAY + 1],
            len: 0,
            pos: 0,
        }
    }

    /// Set the delay in samples, clamped to what the buffer holds.
    pub fn set_len(&mut self, len: usize) {
        let len = len.min(MAX_DELAY);
        if len != self.len {
            self.len = len;
            self.buf = [0.0; MAX_DELAY + 1];
            self.pos = 0;
        }
    }

    pub fn reset(&mut self) {
        self.buf = [0.0; MAX_DELAY + 1];
        self.pos = 0;
    }

    /// Push `x` and return what went in `len` samples ago.
    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        if self.len == 0 {
            return x;
        }
        let n = self.len + 1;
        self.buf[self.pos] = x;
        self.pos = (self.pos + 1) % n;
        self.buf[self.pos]
    }
}

impl Default for Delay {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Both factors must come back as the input delayed by exactly the
    /// latency this module publishes, or the mix control combs itself.
    #[test]
    fn the_round_trip_is_unity_at_the_stated_latency() {
        for factor in [1usize, 2, 4] {
            let mut c = Chain::new();
            c.set_factor(factor);
            let mut d = Delay::new();
            d.set_len(latency(factor));
            let n = 800;
            let sr = 48_000.0f32;
            let mut err: f32 = 0.0;
            for i in 0..n {
                let x = (std::f32::consts::TAU * 1000.0 * i as f32 / sr).sin();
                let up = c.up(x);
                let y = c.down(&up);
                let dry = d.process(x);
                if i > 400 {
                    err = err.max((y - dry).abs());
                }
            }
            assert!(err < 0.01, "{factor}x round trip error {err}");
        }
    }
}

//! 2x oversampling with a half-band FIR, as `research/1176.md` 7.7 recommends
//! for the gain multiplication (20 µs attacks) and the waveshapers.
//!
//! The same 31-tap windowed-sinc low-pass (cut-off at a quarter of the
//! oversampled rate, Blackman window) is used for interpolation and
//! decimation. Interpolation zero-stuffs and filters with gain 2;
//! decimation filters and drops every other sample. Each filter delays by
//! 15 oversampled samples, so a round trip costs [`LATENCY`] samples at the
//! base rate, which the plug-in reports to the host.

/// Filter length (odd, so the group delay is a whole number of samples).
pub const TAPS: usize = 31;
/// Round-trip latency in base-rate samples: `(TAPS − 1) / 2`.
pub const LATENCY: usize = (TAPS - 1) / 2;

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

/// FIR state shared by both directions.
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

/// Interpolator: one base-rate sample in, two oversampled samples out.
#[derive(Clone)]
pub struct Upsampler(Fir);

impl Upsampler {
    pub fn new() -> Self {
        Upsampler(Fir::new())
    }

    #[inline]
    pub fn process(&mut self, x: f32) -> [f32; 2] {
        [2.0 * self.0.push(x), 2.0 * self.0.push(0.0)]
    }

    pub fn reset(&mut self) {
        self.0.reset();
    }
}

impl Default for Upsampler {
    fn default() -> Self {
        Self::new()
    }
}

/// Decimator: two oversampled samples in, one base-rate sample out.
#[derive(Clone)]
pub struct Downsampler(Fir);

impl Downsampler {
    pub fn new() -> Self {
        Downsampler(Fir::new())
    }

    #[inline]
    pub fn process(&mut self, pair: [f32; 2]) -> f32 {
        let y = self.0.push(pair[0]);
        self.0.push(pair[1]);
        y
    }

    pub fn reset(&mut self) {
        self.0.reset();
    }
}

impl Default for Downsampler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_is_unity_with_the_stated_latency() {
        let mut up = Upsampler::new();
        let mut down = Downsampler::new();
        let n = 400;
        let sr = 48_000.0;
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let x = (2.0 * std::f32::consts::PI * 1000.0 * i as f32 / sr).sin();
            out.push(down.process(up.process(x)));
        }
        // After the transient, the output equals the input delayed by LATENCY.
        let mut err = 0.0f32;
        for i in 200..n {
            let x = (2.0 * std::f32::consts::PI * 1000.0 * (i - LATENCY) as f32 / sr).sin();
            err = err.max((out[i] - x).abs());
        }
        assert!(err < 0.01, "round-trip error {err}");
    }
}

//! Oversampling for the variable-mu engine: a cascade of half-band stages,
//! 4x, 8x or 16x.
//!
//! This is the 1176's oversampler ([`crate::dsp::fet::oversample`]) — the
//! same windowed-sinc half-band design, the same use for interpolation and
//! decimation — with two differences that the cascade forces.
//!
//! **It is 65 taps rather than 63, so that the round trip is a whole number
//! of samples at every factor.** A stage's filter delays `(N−1)/2` samples
//! at its own rate, so a stage running at `2^k` times the base rate costs
//! `(N−1)/2^k` base samples each way. With 63 taps that is 31, 15.5, 7.75 …
//! and a three-deep cascade lands on 54.25 base samples, which is not a
//! latency a host can be told. With 65 it is 32, 16, 8, 4 and every cascade
//! is exact.
//!
//! **The dossier asks for 8x and gives three reasons** (10.5), all of which
//! are about the feedback loop rather than about aliasing alone: the tube's
//! products fold, the bilinear transform warps, and — the one people forget
//! — the loop is closed with a one-sample delay that has to be short against
//! a 200 microsecond attack. At 44.1 kHz, 8x puts that delay at 2.8 µs
//! against 200 µs, which is 1.4 %; at 1x it would be 11 % and the fastest
//! time-constant positions would visibly slow down.
//!
//! | depth | factor | round trip, base samples |
//! |---|---|---|
//! | 2 | 4x | 48 |
//! | 3 | 8x | 56 |
//! | 4 | 16x | 60 |

/// Filter length. Odd, and `(TAPS − 1)` a multiple of 16, so that a cascade
/// four deep still delays a whole number of base-rate samples.
pub const TAPS: usize = 65;
/// Deepest cascade, i.e. 16x.
pub const MAX_DEPTH: usize = 4;
/// Largest factor, for stack buffers.
pub const MAX_FACTOR: usize = 1 << MAX_DEPTH;

/// Kaiser window shape. `β = 9` puts the stopband near 90 dB and, more to
/// the point here, keeps the transition band narrow enough to stay out of
/// the audio band.
const KAISER_BETA: f32 = 9.0;

/// Modified Bessel function of the first kind, order zero, by its series.
/// Only ever called while the coefficients are designed.
fn bessel_i0(x: f32) -> f32 {
    let mut term = 1.0f64;
    let mut sum = 1.0f64;
    let half = x as f64 / 2.0;
    for k in 1..40 {
        term *= (half / k as f64) * (half / k as f64);
        sum += term;
        if term < 1e-14 * sum {
            break;
        }
    }
    sum as f32
}

/// `0.5·sinc((k − c)/2)` under a Kaiser window, normalised to unity DC gain.
///
/// The 1176's filter is the same windowed sinc under a **Blackman** window,
/// and that window is why this one is not. A Blackman-windowed half-band of
/// this length has a transition band about 18 kHz wide about its 24 kHz
/// cutoff at 48 kHz, so its passband droop reaches down to 15 kHz — where
/// Fairchild specify the response to ±1 dB — and a round trip through the
/// cascade costs nearly 4 dB there. The 1176's README already records the
/// same droop as the reason its 610 preamp misses its response figure at
/// 44.1 kHz. A Kaiser window at `β = 9` halves the transition width for the
/// same 65 taps, which puts 15 kHz in the flat part.
fn coefficients() -> [f32; TAPS] {
    let c = (TAPS - 1) as f32 / 2.0;
    let mut h = [0.0f32; TAPS];
    let mut sum = 0.0;
    let denom = bessel_i0(KAISER_BETA);
    for (k, hk) in h.iter_mut().enumerate() {
        let n = k as f32 - c;
        let sinc = if n == 0.0 {
            1.0
        } else {
            (std::f32::consts::PI * n / 2.0).sin() / (std::f32::consts::PI * n / 2.0)
        };
        let t = n / c;
        let w = bessel_i0(KAISER_BETA * (1.0 - t * t).max(0.0).sqrt()) / denom;
        *hk = 0.5 * sinc * w;
        sum += *hk;
    }
    for hk in h.iter_mut() {
        *hk /= sum;
    }
    h
}

/// One direction of one stage.
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

/// A cascade of half-band stages, up on one side and down on the other.
#[derive(Clone)]
pub struct Resampler {
    up: Vec<Fir>,
    down: Vec<Fir>,
    depth: usize,
}

impl Resampler {
    /// A resampler `depth` stages deep, i.e. `2^depth` times oversampled.
    pub fn new(depth: usize) -> Self {
        let depth = depth.clamp(1, MAX_DEPTH);
        Resampler {
            up: (0..MAX_DEPTH).map(|_| Fir::new()).collect(),
            down: (0..MAX_DEPTH).map(|_| Fir::new()).collect(),
            depth,
        }
    }

    /// Change the factor. Every filter is cleared, because a cascade of a
    /// different depth is a different filter and its history means nothing.
    pub fn set_depth(&mut self, depth: usize) {
        let depth = depth.clamp(1, MAX_DEPTH);
        if depth != self.depth {
            self.depth = depth;
            self.reset();
        }
    }

    pub fn factor(&self) -> usize {
        1 << self.depth
    }

    /// Round-trip latency in base-rate samples: `Σ (TAPS − 1) / 2^k` over the
    /// stages, which the tap count is chosen to keep whole.
    pub fn latency(&self) -> usize {
        (1..=self.depth).map(|k| (TAPS - 1) >> k).sum()
    }

    pub fn reset(&mut self) {
        for f in self.up.iter_mut().chain(self.down.iter_mut()) {
            f.reset();
        }
    }

    /// One base-rate sample in, `factor()` oversampled samples into `out`.
    ///
    /// Each stage reads from a copy, because the filters are stateful and
    /// have to be fed **forward in time**. Expanding in place and walking
    /// backwards to avoid overwriting the source pushes the samples through
    /// in reverse order, which is not the same filter at all: it cost about
    /// 2 dB at 15 kHz before it was measured against the published response
    /// band.
    pub fn up(&mut self, x: f32, out: &mut [f32; MAX_FACTOR]) {
        let mut src = [0.0f32; MAX_FACTOR];
        src[0] = x;
        out[0] = x;
        let mut n = 1;
        for stage in 0..self.depth {
            for i in 0..n {
                out[2 * i] = 2.0 * self.up[stage].push(src[i]);
                out[2 * i + 1] = 2.0 * self.up[stage].push(0.0);
            }
            n *= 2;
            src[..n].copy_from_slice(&out[..n]);
        }
    }

    /// `factor()` oversampled samples in, one base-rate sample out.
    pub fn down(&mut self, xs: &[f32; MAX_FACTOR]) -> f32 {
        let mut buf = *xs;
        let mut n = self.factor();
        for stage in (0..self.depth).rev() {
            for i in 0..n / 2 {
                let y = self.down[stage].push(buf[2 * i]);
                self.down[stage].push(buf[2 * i + 1]);
                buf[i] = y;
            }
            n /= 2;
        }
        buf[0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The round trip is unity at the latency the module publishes, at every
    /// factor. Not a published figure: it is the resampler's own contract,
    /// and the test says so.
    #[test]
    fn the_cascade_round_trips_at_unity_with_the_stated_latency() {
        for depth in 1..=MAX_DEPTH {
            let mut r = Resampler::new(depth);
            let lat = r.latency();
            let sr = 48_000.0f32;
            let n = 600;
            let mut out = vec![0.0f32; n];
            let mut up = [0.0f32; MAX_FACTOR];
            for (i, o) in out.iter_mut().enumerate() {
                let x = (2.0 * std::f32::consts::PI * 1000.0 * i as f32 / sr).sin();
                r.up(x, &mut up);
                *o = r.down(&up);
            }
            let mut err = 0.0f32;
            for i in 300..n {
                let want = (2.0 * std::f32::consts::PI * 1000.0 * (i - lat) as f32 / sr).sin();
                err = err.max((out[i] - want).abs());
            }
            assert!(err < 0.02, "depth {depth}: round-trip error {err}");
        }
    }

    /// Every factor's latency is a whole number of base samples, which is
    /// why this module exists rather than the 1176's being cascaded.
    #[test]
    fn every_factor_has_an_integer_latency() {
        assert_eq!(Resampler::new(2).latency(), 48);
        assert_eq!(Resampler::new(3).latency(), 56);
        assert_eq!(Resampler::new(4).latency(), 60);
    }
}

//! First-order antiderivative anti-aliasing for the 610's two tube stages
//! (`research/610.md` 8.6).
//!
//! Oversampling moves a nonlinearity's harmonics further from Nyquist, but
//! it cannot help a stage that is tens of decibels into its knee: a hard
//! clip makes harmonics faster than any practical factor can outrun, which
//! is why doubling from 2x to 4x bought barely two decibels here. The
//! research says so and prescribes this instead.
//!
//! The trick is to integrate the shaper over the segment between two
//! samples rather than sampling it at a point. For a memoryless `f` with
//! antiderivative `F`,
//!
//! ```text
//! y[n] = (F(x[n]) − F(x[n−1])) / (x[n] − x[n−1])
//! ```
//!
//! which is the average of `f` across the step the signal actually took,
//! so the sharp corner is smeared over the segment instead of being
//! sampled through. When two inputs are too close together the quotient
//! loses its meaning, so it falls back to the midpoint.
//!
//! The shaper here is `S(v) = v / (1 + |v|^n)^(1/n)`, whose antiderivative
//! is elementary only for integer `n`; the stages use 2.5, 3.5 and 4, so
//! the antiderivative is tabulated once per stage when the voicing is set
//! and read back by interpolation. Above the table's range `S` is within a
//! fraction of a per cent of one, so the antiderivative continues as a
//! straight line of slope one.
//!
//! The shaper is not defined here. `S` and the stage law built on it are the
//! valve, which is a shared component, and this file integrates what that
//! component evaluates: the table below is a numerical integral of
//! [`s_curve`] and would go quietly wrong the moment a second copy of the
//! law existed for it to drift from. The anti-aliasing is a technique rather
//! than a part and stays here, which is the division the component's own
//! documentation draws.

use super::stage::{s_curve, tube};

/// Points in the table.
const POINTS: usize = 4096;
/// The table covers `|u|` up to here; beyond it the shaper is flat.
const U_MAX: f32 = 16.0;
/// Steps closer together than this fall back to the midpoint.
///
/// The quotient divides a difference of two nearly equal antiderivatives by
/// the step, so a table error of a millionth becomes a hundredth if the step
/// is a thousandth. Below this the midpoint is both more accurate and just
/// as correct: a signal moving that slowly has nothing near Nyquist to
/// alias, which is the only thing the quotient is here to fix.
const MIN_STEP: f32 = 0.01;

/// One tube stage's shaper, evaluated with first-order antiderivative
/// anti-aliasing.
#[derive(Clone)]
pub struct Adaa {
    /// `G(u) = ∫₀^u S(t, n) dt` for `u` in `0..U_MAX`; `G` is even because
    /// `S` is odd.
    table: Vec<f32>,
    n: f32,
    /// The last input, so the segment can be integrated.
    x1: f32,
}

impl Adaa {
    /// Tabulate the antiderivative for exponent `n`.
    pub fn new(n: f32) -> Self {
        let mut table = vec![0.0f32; POINTS];
        let h = U_MAX / (POINTS - 1) as f32;
        let mut acc = 0.0f64;
        let mut prev = 0.0f32;
        for (i, slot) in table.iter_mut().enumerate() {
            if i > 0 {
                let u = i as f32 * h;
                let cur = s_curve(u, n);
                // Trapezoid: the integrand is smooth and monotone.
                acc += 0.5 * (prev + cur) as f64 * h as f64;
                prev = cur;
            }
            *slot = acc as f32;
        }
        Adaa { table, n, x1: 0.0 }
    }

    pub fn reset(&mut self) {
        self.x1 = 0.0;
    }

    /// `G(u)`, by interpolation, continued as a straight line above the
    /// table.
    #[inline]
    fn g(&self, u: f32) -> f32 {
        let a = u.abs();
        let last = POINTS - 1;
        if a >= U_MAX {
            // `S` is within 1/(n·u^n) of one out here, so `G` runs straight.
            return self.table[last] + (a - U_MAX);
        }
        let t = a / U_MAX * last as f32;
        let i = t as usize;
        let f = t - i as f32;
        let lo = self.table[i];
        let hi = self.table[(i + 1).min(last)];
        lo + (hi - lo) * f
    }

    /// The antiderivative of the stage `tube(v, b, n)`, with the constant
    /// chosen so it is zero at rest.
    #[inline]
    fn antiderivative(&self, v: f32, b: f32) -> f32 {
        let s_b = s_curve(b, self.n);
        (self.g(v + b) - self.g(b) - s_b * v) / super::stage::s_slope(b, self.n)
    }

    /// One sample through the stage. `bias` may move between samples (the
    /// input stage's self-rectification does), in which case both ends of
    /// the segment are evaluated at the bias now in force.
    #[inline]
    pub fn process(&mut self, x: f32, bias: f32) -> f32 {
        let d = x - self.x1;
        let y = if d.abs() > MIN_STEP {
            (self.antiderivative(x, bias) - self.antiderivative(self.x1, bias)) / d
        } else {
            tube(0.5 * (x + self.x1), bias, self.n)
        };
        self.x1 = x;
        y
    }
}

impl Default for Adaa {
    fn default() -> Self {
        Adaa::new(2.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The table has to be the real antiderivative, or the anti-aliasing
    /// silently becomes a distortion of its own.
    #[test]
    fn the_table_integrates_the_shaper() {
        for n in [2.5f32, 3.5, 4.0] {
            let a = Adaa::new(n);
            // Against a fine trapezoid sum computed here.
            for probe in [0.25f32, 1.0, 3.0, 9.0] {
                let steps = 20_000;
                let h = probe / steps as f32;
                let mut acc = 0.0f64;
                let mut prev = 0.0f32;
                for i in 1..=steps {
                    let cur = s_curve(i as f32 * h, n);
                    acc += 0.5 * (prev + cur) as f64 * h as f64;
                    prev = cur;
                }
                let want = acc as f32;
                let got = a.g(probe);
                assert!(
                    (got - want).abs() < 1e-3 * want.max(1e-3),
                    "n {n}, u {probe}: table {got:.6} against {want:.6}"
                );
            }
            // Even, because the shaper is odd.
            assert!((a.g(-2.0) - a.g(2.0)).abs() < 1e-6);
        }
    }

    /// With a slowly changing input the segment average has to agree with
    /// the point evaluation, or the stage would be quietly mis-shaped.
    #[test]
    fn it_agrees_with_the_shaper_on_a_slow_signal() {
        let mut a = Adaa::new(2.5);
        let b = 0.12f32;
        let mut worst = 0.0f32;
        for i in 0..4000 {
            let x = 2.0 * (i as f32 * 0.0005).sin();
            let got = a.process(x, b);
            let want = tube(x, b, 2.5);
            worst = worst.max((got - want).abs());
        }
        assert!(worst < 0.01, "worst disagreement {worst:.5}");
    }

    #[test]
    fn it_survives_steps_and_stillness() {
        let mut a = Adaa::new(4.0);
        assert!(a.process(0.0, 0.08).abs() < 1e-6);
        assert!(a.process(0.0, 0.08).abs() < 1e-6);
        for x in [40.0f32, -40.0, 0.0, 1e-9, -1e-9] {
            let y = a.process(x, 0.08);
            assert!(y.is_finite(), "{x} gave {y}");
        }
    }
}

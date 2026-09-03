//! The panel meter's movement, shared by every model.
//!
//! A VU meter is not a level read-out, it is a physical needle on a spring,
//! and both the 1176 and the LA-2A research documents ask for the standard
//! response: **99 % of the deflection reached in 300 ms with 1 to 1.5 % of
//! overshoot**, from a second-order system with `omega_0` about 13 rad/s
//! and a damping ratio of about 0.80. Both also insist the ballistics live
//! in the audio thread, so that what the needle does cannot depend on how
//! often the page happens to repaint.
//!
//! So the engine publishes the needle's **position**, not the level it is
//! chasing: `meter_vu` in the `meter` stream is already smoothed, and a page
//! should draw it as it arrives rather than smoothing it again.
//!
//! The state is integrated once per sample from a target held constant over
//! the block, which is the same thing as stepping it per sample with a
//! zero-order-held input and costs a handful of flops per sample at
//! 13 rad/s.

/// Natural frequency of the movement, radians per second.
pub const OMEGA0: f32 = 13.0;
/// Damping ratio: 0.80 gives 1.5 % overshoot, where the framework's drawing
/// default of 0.62 would give about 8 %.
pub const ZETA: f32 = 0.80;
/// Where the needle rests when there is nothing to show, in dB.
pub const REST_DB: f32 = -60.0;

/// A VU movement: position and velocity, in whatever unit the target is
/// (every model in this lab drives it in dB).
#[derive(Clone, Copy, Debug)]
pub struct Vu {
    y: f32,
    v: f32,
    dt: f32,
}

impl Vu {
    pub fn new(sr: f32) -> Self {
        Vu {
            y: REST_DB,
            v: 0.0,
            dt: 1.0 / sr.max(1.0),
        }
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.dt = 1.0 / sr.max(1.0);
    }

    /// Park the needle at `y` with no velocity.
    pub fn snap(&mut self, y: f32) {
        self.y = y;
        self.v = 0.0;
    }

    pub fn reset(&mut self) {
        self.snap(REST_DB);
    }

    /// The needle's position now.
    pub fn value(&self) -> f32 {
        self.y
    }

    /// Advance `samples` steps towards a target held constant over them.
    #[inline]
    pub fn advance(&mut self, target: f32, samples: usize) {
        if !target.is_finite() {
            return;
        }
        let w2 = OMEGA0 * OMEGA0;
        let c = 2.0 * ZETA * OMEGA0;
        for _ in 0..samples {
            let a = w2 * (target - self.y) - c * self.v;
            self.v += a * self.dt;
            self.y += self.v * self.dt;
        }
        if !self.y.is_finite() {
            self.snap(target);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SR: f32 = 48_000.0;

    /// The standard the two research documents quote: 99 % of the way in
    /// about 300 ms, with 1 to 1.5 % of overshoot.
    #[test]
    fn the_movement_meets_the_vu_standard() {
        let mut vu = Vu::new(SR);
        vu.snap(0.0);
        let target = 1.0f32;
        let mut t99 = None;
        let mut peak = 0.0f32;
        let steps = (SR * 2.0) as usize;
        for i in 0..steps {
            vu.advance(target, 1);
            peak = peak.max(vu.value());
            if t99.is_none() && vu.value() >= 0.99 * target {
                t99 = Some(i as f32 / SR);
            }
        }
        let t99 = t99.expect("the needle must get there");
        assert!(
            (0.25..=0.35).contains(&t99),
            "99 % should arrive at about 300 ms, got {:.0} ms",
            t99 * 1000.0
        );
        let overshoot = (peak - target) / target * 100.0;
        assert!(
            (1.0..=1.6).contains(&overshoot),
            "overshoot should be 1 to 1.5 %, got {overshoot:.2} %"
        );
        // And it settles.
        assert!((vu.value() - target).abs() < 1e-3);
    }

    #[test]
    fn it_is_the_same_movement_at_every_rate() {
        for sr in [44_100.0f32, 48_000.0, 96_000.0] {
            let mut vu = Vu::new(sr);
            vu.snap(0.0);
            let n = (sr * 0.3) as usize;
            vu.advance(1.0, n);
            let at_300ms = vu.value();
            assert!(
                (at_300ms - 0.99).abs() < 0.02,
                "at {sr} Hz the needle should be at 99 % after 300 ms, got {at_300ms:.4}"
            );
        }
    }

    #[test]
    fn it_survives_a_target_that_is_not_a_number() {
        let mut vu = Vu::new(SR);
        vu.snap(-6.0);
        vu.advance(f32::NAN, 64);
        assert_eq!(vu.value(), -6.0);
        vu.advance(f32::INFINITY, 64);
        assert!(vu.value().is_finite());
    }
}

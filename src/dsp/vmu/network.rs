//! The six-position time-constant network, read off the factory drawing.
//!
//! This is the unit's signature and the one block that has to be built as a
//! circuit rather than as a switch statement. `research/Fairchild-670.md`
//! section 5.3 reads all fourteen component values off the original
//! Fairchild 660 drawing JL10866, where the switch positions are numbered
//! on the sheet, and finds that they agree with Raffensperger's published
//! table in all six positions — which also settles the one capacitor the
//! redrawn 670 sheet marks `???` as 2 µF / 200 V. The 670's network is the
//! same network with 1xx designators, so these values serve both units.
//!
//! ```text
//!            I_sc
//!             │
//!             ├──────┬───────────────┬───────────────┐
//!             │      │               │               │
//!            C_T    R_T             R_U             R_V
//!             │      │               │               │
//!             │      │              C_U             C_V
//!             │      │               │               │
//!            ─┴──────┴───────────────┴───────────────┴─
//! ```
//!
//! **Why the two extra legs are the whole point.** They are capacitors
//! behind resistors, so they charge on their own clock. While they are
//! empty they look like a near short to the return and pull the effective
//! resistance down, so the release is fast; once they are charged they stop
//! sinking current and the charge they hold has to come back out through
//! the same resistors, so the release becomes slow and grows a long tail.
//! That is the mechanism behind Fairchild's otherwise baffling claim that
//! position 6 releases in 0.3 seconds for individual peaks and 25 seconds
//! for consistently high programme level (dossier 5.5), and it only comes
//! out of a model that integrates the actual network.
//!
//! **The states are never reset by the switch.** A real rotary switch does
//! not discharge the capacitors: the legs that leave the circuit keep their
//! charge and the ones that join bring theirs. [`Network::set_position`]
//! therefore changes only the coefficients.

use crate::dsp::flush;

/// Number of positions on the TIME CONSTANT switch.
pub const POSITIONS: usize = 6;

/// The printed positions, which is all the panel says.
pub const TIME_NAMES: [&str; POSITIONS] = ["1", "2", "3", "4", "5", "6"];

/// One position of S102 (S2 on the 660), as component values.
///
/// `r_u` or `r_v` of [`f32::INFINITY`] is that leg out of circuit: its
/// capacitor is still there and still holds whatever charge it had, it just
/// has no path to the node.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    /// Ohms across the node: R137 (R32) alone, or in parallel with the
    /// resistor the upper or lower deck adds.
    pub r_t: f32,
    /// Farads on the node: C115 (C7) plus whatever the decks add.
    pub c_t: f32,
    pub r_u: f32,
    pub c_u: f32,
    pub r_v: f32,
    pub c_v: f32,
}

/// R137 on the 670, R32 on the 660: permanently across the node.
pub const R_FIXED: f32 = 220_000.0;
/// C115 on the 670 (marked `???` on the redraw), C7 on the 660: 2 µF/200 V.
pub const C_FIXED: f32 = 2e-6;
/// R37 / R141, switched in at position 1.
pub const R_POS1: f32 = 68_000.0;
/// R33 / R138, switched in at position 2.
pub const R_POS2: f32 = 470_000.0;
/// C11 / C113, switched in at positions 3, 4 and 5 (the deck ties them).
pub const C_POS345: f32 = 2e-6;
/// C9 / C111, added at position 4 only.
pub const C_POS4: f32 = 4e-6;
/// R34 + C8 (R139 + C110): the first programme-dependent leg, positions 5
/// and 6.
pub const R_LEG_U: f32 = 100_000.0;
pub const C_LEG_U: f32 = 8e-6;
/// R35 + C10 (R140 + C112): the second leg, position 6 only.
pub const R_LEG_V: f32 = 100_000.0;
pub const C_LEG_V: f32 = 20e-6;

/// Two resistors in parallel.
fn par(a: f32, b: f32) -> f32 {
    a * b / (a + b)
}

/// The six positions, folded out of the two switch decks (dossier 5.3).
///
/// Upper deck: position 1 adds R37; 3, 4 and 5 add C11; 6 adds R35 + C10.
/// Lower deck: position 2 adds R33; 4 adds C9; 5 and 6 add R34 + C8.
pub fn position(i: usize) -> Position {
    let out = f32::INFINITY;
    match i.min(POSITIONS - 1) {
        0 => Position {
            r_t: par(R_FIXED, R_POS1),
            c_t: C_FIXED,
            r_u: out,
            c_u: C_LEG_U,
            r_v: out,
            c_v: C_LEG_V,
        },
        1 => Position {
            r_t: par(R_FIXED, R_POS2),
            c_t: C_FIXED,
            r_u: out,
            c_u: C_LEG_U,
            r_v: out,
            c_v: C_LEG_V,
        },
        2 => Position {
            r_t: R_FIXED,
            c_t: C_FIXED + C_POS345,
            r_u: out,
            c_u: C_LEG_U,
            r_v: out,
            c_v: C_LEG_V,
        },
        3 => Position {
            r_t: R_FIXED,
            c_t: C_FIXED + C_POS345 + C_POS4,
            r_u: out,
            c_u: C_LEG_U,
            r_v: out,
            c_v: C_LEG_V,
        },
        4 => Position {
            r_t: R_FIXED,
            c_t: C_FIXED + C_POS345,
            r_u: R_LEG_U,
            c_u: C_LEG_U,
            r_v: out,
            c_v: C_LEG_V,
        },
        _ => Position {
            r_t: R_FIXED,
            c_t: C_FIXED,
            r_u: R_LEG_U,
            c_u: C_LEG_U,
            r_v: R_LEG_V,
            c_v: C_LEG_V,
        },
    }
}

/// The network, integrated with the trapezoidal rule.
///
/// The system is linear in its three node voltages and only the drive is
/// nonlinear, so the whole step folds into one 3 × 3 matrix that is
/// recomputed when the switch or the sample rate moves and applied with
/// nine multiplies per sample in between. That is cheaper than the branchy
/// programme-dependent logic a switched one-pole would have needed, which
/// is the dossier's own argument for building the circuit (5.5).
#[derive(Clone, Copy, Debug)]
pub struct Network {
    pos: usize,
    dt: f32,
    /// Node voltages on `C_T`, `C_U`, `C_V`.
    v: [f32; 3],
    /// The drive current of the previous step, which the trapezoidal rule
    /// needs.
    i_prev: f32,
    /// `(I − h·A/2)⁻¹ (I + h·A/2)`.
    m: [[f32; 3]; 3],
    /// `(I − h·A/2)⁻¹ · b · h/2`, applied to `I_n + I_{n−1}`.
    p: [f32; 3],
}

impl Network {
    pub fn new(sr: f32, pos: usize) -> Self {
        let mut n = Network {
            pos,
            dt: 1.0 / sr.max(1.0),
            v: [0.0; 3],
            i_prev: 0.0,
            m: [[0.0; 3]; 3],
            p: [0.0; 3],
        };
        n.rebuild();
        n
    }

    /// The switch position in force (0-based; the panel prints 1 to 6).
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Move the switch. **The capacitors keep their charge**, which is what
    /// a rotary switch does and what makes turning the knob during a
    /// passage behave the way this box is known for.
    pub fn set_position(&mut self, pos: usize) {
        let pos = pos.min(POSITIONS - 1);
        if pos != self.pos {
            self.pos = pos;
            self.rebuild();
        }
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.dt = 1.0 / sr.max(1.0);
        self.rebuild();
    }

    pub fn reset(&mut self) {
        self.v = [0.0; 3];
        self.i_prev = 0.0;
    }

    /// The control voltage across `C_T`, in volts, always ≥ 0. The grids see
    /// its negative.
    #[inline]
    pub fn control_v(&self) -> f32 {
        self.v[0]
    }

    /// The charge state of the three timing capacitors as a fraction of the
    /// node's own voltage, for the page's three little bars. Zeros while the
    /// node is empty.
    pub fn charge_state(&self) -> [f32; 3] {
        let d = self.v[0].max(1e-6);
        [
            self.v[0],
            (self.v[1] / d).clamp(0.0, 1.0),
            (self.v[2] / d).clamp(0.0, 1.0),
        ]
    }

    /// Advance one sample with a drive current of `i` amps into the node.
    #[inline]
    pub fn step(&mut self, i: f32) -> f32 {
        let s = i + self.i_prev;
        self.i_prev = i;
        let v = self.v;
        for k in 0..3 {
            self.v[k] = flush(
                self.m[k][0] * v[0] + self.m[k][1] * v[1] + self.m[k][2] * v[2] + self.p[k] * s,
            )
            .max(0.0);
        }
        self.v[0]
    }

    /// Rebuild the step matrix for the position and rate in force.
    fn rebuild(&mut self) {
        let p = position(self.pos);
        let gu = if p.r_u.is_finite() { 1.0 / p.r_u } else { 0.0 };
        let gv = if p.r_v.is_finite() { 1.0 / p.r_v } else { 0.0 };
        let a = [
            [-(1.0 / p.r_t + gu + gv) / p.c_t, gu / p.c_t, gv / p.c_t],
            [gu / p.c_u, -gu / p.c_u, 0.0],
            [gv / p.c_v, 0.0, -gv / p.c_v],
        ];
        let h = 0.5 * self.dt;
        // L = I − h·A, R = I + h·A, and the step is L⁻¹(R x + h·b·(iₙ+iₙ₋₁)).
        let mut l = [[0.0f32; 3]; 3];
        let mut r = [[0.0f32; 3]; 3];
        for (row, (lr, rr)) in l.iter_mut().zip(r.iter_mut()).enumerate() {
            for col in 0..3 {
                let e = if row == col { 1.0 } else { 0.0 };
                lr[col] = e - h * a[row][col];
                rr[col] = e + h * a[row][col];
            }
        }
        let inv = invert3(&l);
        for row in 0..3 {
            for col in 0..3 {
                self.m[row][col] = (0..3).map(|k| inv[row][k] * r[k][col]).sum();
            }
            self.p[row] = inv[row][0] * h / p.c_t;
        }
    }
}

/// Invert a 3 × 3 matrix by cofactors. Only ever called when the switch or
/// the rate moves, and the matrix is diagonally dominant by construction, so
/// there is nothing to be careful about beyond a zero determinant that
/// cannot arise.
fn invert3(m: &[[f32; 3]; 3]) -> [[f32; 3]; 3] {
    let c = |r: usize, s: usize| {
        let rows: Vec<usize> = (0..3).filter(|&x| x != r).collect();
        let cols: Vec<usize> = (0..3).filter(|&x| x != s).collect();
        let d =
            m[rows[0]][cols[0]] * m[rows[1]][cols[1]] - m[rows[0]][cols[1]] * m[rows[1]][cols[0]];
        if (r + s).is_multiple_of(2) { d } else { -d }
    };
    let det = m[0][0] * c(0, 0) + m[0][1] * c(0, 1) + m[0][2] * c(0, 2);
    let mut out = [[0.0f32; 3]; 3];
    for (row, o) in out.iter_mut().enumerate() {
        for (col, v) in o.iter_mut().enumerate() {
            // Transpose of the cofactor matrix, over the determinant.
            *v = c(col, row) / det;
        }
    }
    out
}

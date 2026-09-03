//! The standalone's demo signal: everything both compressors used to come
//! with, in one generator. Not band limited; it only has to give the
//! compressors something to chew on. Every kind plays at unity level.
//!
//! | `src_kind` | label | what it is |
//! |---|---|---|
//! | 0 | Vocal | a saw-ish tone with 5.5 Hz vibrato, a formant-like bump and syllables every 0.55 s |
//! | 1 | Bass | a plucked saw through a one-pole low-pass, walking a four-note eighth-note pattern from `freq` |
//! | 2 | Drums | a synthesized 120 BPM loop: kick on 1 and 3, snare on 2 and 4, hats on the eighths |
//! | 3 | Pink noise | Paul Kellet's refined pink filter on white noise |
//! | 4 | White noise | a xorshift generator |
//! | 5 | Saw | a naive sawtooth at `freq` |
//! | 6 | Sine | a sine at `freq` |

use std::f32::consts::PI;

/// Labels of `src_kind`, in parameter order.
pub const SOURCE_NAMES: [&str; 7] = [
    "Vocal",
    "Bass",
    "Drums",
    "Pink noise",
    "White noise",
    "Saw",
    "Sine",
];

/// The generator; one instance feeds both channels.
pub struct Source {
    phase: f32,
    phase2: f32,
    vib: f32,
    t: u32,
    env: f32,
    pink: [f32; 7],
    rng: u32,
    drum_t: u32,
    drum_env: f32,
    drum_phase: f32,
    snare_env: f32,
    hat_env: f32,
    bass_t: u32,
    bass_env: f32,
    bass_lp: f32,
    bass_phase: f32,
}

impl Source {
    pub fn new(seed: u32) -> Self {
        Source {
            phase: 0.0,
            phase2: 0.0,
            vib: 0.0,
            t: 0,
            env: 0.0,
            pink: [0.0; 7],
            rng: seed | 1,
            drum_t: 0,
            drum_env: 0.0,
            drum_phase: 0.0,
            snare_env: 0.0,
            hat_env: 0.0,
            bass_t: 0,
            bass_env: 0.0,
            bass_lp: 0.0,
            bass_phase: 0.0,
        }
    }

    #[inline]
    fn white(&mut self) -> f32 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        (x as f32 / u32::MAX as f32) * 2.0 - 1.0
    }

    /// Paul Kellet's refined pink filter on the white generator.
    #[inline]
    fn pink(&mut self) -> f32 {
        let w = self.white();
        let b = &mut self.pink;
        b[0] = 0.99886 * b[0] + w * 0.0555179;
        b[1] = 0.99332 * b[1] + w * 0.0750759;
        b[2] = 0.96900 * b[2] + w * 0.153_852;
        b[3] = 0.86650 * b[3] + w * 0.3104856;
        b[4] = 0.55000 * b[4] + w * 0.5329522;
        b[5] = -0.7616 * b[5] - w * 0.0168980;
        let pink = b[0] + b[1] + b[2] + b[3] + b[4] + b[5] + b[6] + w * 0.5362;
        b[6] = w * 0.115926;
        pink * 0.11
    }

    /// One sample of source `kind` (see [`SOURCE_NAMES`]) at unity level.
    #[inline]
    pub fn next(&mut self, kind: usize, freq: f32, sr: f32) -> f32 {
        self.t = self.t.wrapping_add(1);
        match kind {
            0 => {
                // Vocal: a saw-ish tone with 5.5 Hz vibrato, a formant-like
                // bump, and syllables every 0.55 s.
                self.vib += 5.5 / sr;
                if self.vib >= 1.0 {
                    self.vib -= 1.0;
                }
                let f = freq * (1.0 + 0.012 * (self.vib * 2.0 * PI).sin());
                self.phase += f / sr;
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                }
                self.phase2 += 3.0 * f / sr;
                if self.phase2 >= 1.0 {
                    self.phase2 -= 1.0;
                }
                let syllable = (self.t as f32 / (0.55 * sr)).fract();
                let target = if syllable < 0.7 {
                    1.0 - 0.4 * syllable
                } else {
                    0.0
                };
                self.env += (target - self.env) * if target > self.env { 0.002 } else { 0.0006 };
                let tone = (self.phase * 2.0 * PI).sin() * 0.7
                    + (self.phase2 * 2.0 * PI).sin() * 0.25
                    + (2.0 * self.phase - 1.0) * 0.15;
                tone * self.env * 0.8
            }
            1 => {
                // Plucked bass: eighth-note pattern over four notes.
                let step = (sr * 0.25) as u32;
                if self.bass_t.is_multiple_of(step) {
                    let n = (self.bass_t / step) % 8;
                    if n != 3 && n != 7 {
                        self.bass_env = 1.0;
                    }
                }
                let n = (self.bass_t / step.max(1)) % 8;
                let semis = [0.0, 0.0, 7.0, 0.0, 5.0, 3.0, 0.0, 0.0][n as usize];
                let f = freq * 2.0f32.powf(semis / 12.0);
                self.bass_t = self.bass_t.wrapping_add(1);
                self.bass_phase += f / sr;
                if self.bass_phase >= 1.0 {
                    self.bass_phase -= 1.0;
                }
                let saw = 2.0 * self.bass_phase - 1.0;
                let cutoff = 200.0 + 1800.0 * self.bass_env * self.bass_env;
                let a = 1.0 - (-2.0 * PI * cutoff / sr).exp();
                self.bass_lp += a * (saw - self.bass_lp);
                self.bass_env *= 1.0 - 4.0 / sr;
                self.bass_lp * (0.25 + 0.75 * self.bass_env)
            }
            2 => {
                // Drums at 120 BPM: kick on 1 and 3, snare on 2 and 4, hats
                // on every eighth.
                let step = (sr * 0.25) as u32;
                if self.drum_t.is_multiple_of(step) {
                    let eighth = (self.drum_t / step) % 8;
                    if eighth.is_multiple_of(4) {
                        self.drum_env = 1.0;
                        self.drum_phase = 0.0;
                    }
                    if eighth % 4 == 2 {
                        self.snare_env = 1.0;
                    }
                    self.hat_env = 0.6;
                }
                self.drum_t = self.drum_t.wrapping_add(1);
                let kick_f = 45.0 + 120.0 * self.drum_env * self.drum_env;
                self.drum_phase += kick_f / sr;
                let kick = (2.0 * PI * self.drum_phase).sin() * self.drum_env;
                self.drum_env *= 1.0 - 8.0 / sr;
                let snare = self.pink() * 2.5 * self.snare_env;
                self.snare_env *= 1.0 - 18.0 / sr;
                let hat = self.white() * 0.4 * self.hat_env;
                self.hat_env *= 1.0 - 60.0 / sr;
                (kick * 0.9 + snare + hat).clamp(-1.0, 1.0)
            }
            3 => self.pink() * 2.0,
            4 => self.white(),
            5 | 6 => {
                self.phase += freq / sr;
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                }
                if kind == 5 {
                    2.0 * self.phase - 1.0
                } else {
                    (2.0 * PI * self.phase).sin()
                }
            }
            _ => 0.0,
        }
    }
}

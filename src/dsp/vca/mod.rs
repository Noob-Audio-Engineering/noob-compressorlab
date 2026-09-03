//! The VCA model of the lab: the Distressor. This module owns everything
//! the engine needs from the parameters ([`Settings`]), its knob-to-time
//! maps and the labels of its switches; the parameter ids, streams and the
//! processor that hosts it live one level up in [`crate::dsp`].
//!
//! The model follows `research/Distressor.md` section 7: a **feedback** VCA
//! compressor computed in the dB domain, with one soft-knee curve per ratio
//! button (the 2:1 knee is 30 dB wide, the 20:1 and Nuke knees a couple of
//! dB), a branching detector whose attack and release depend on the ratio
//! and on how far the signal overshoots, the 10:1 position's two-stage
//! opto-like release and Nuke's logarithmic one, the switchable Dist 2 /
//! Dist 3 harmonic generator after the VCA, the side-chain's high-pass and
//! band emphasis, the 80 Hz Bessel audio high-pass, British mode, and the
//! three stereo link behaviours.
//!
//! | module | contents |
//! |---|---|
//! | [`compressor`] | the engine: detector, per-ratio curves, ballistics, distortion, meters, static transfer curve |
//! | [`filters`] | biquad, one-pole and Bessel sections |
//! | this file | knob maps, [`Ratio`], [`Detector`], [`AudioMode`], [`LinkMode`], [`Settings`] |

pub mod compressor;
pub mod filters;

pub use compressor::{Compressor, Curve, TRANSFER_POINTS, curve};

/// Labels of `dist_ratio`, in parameter order.
pub const RATIO_NAMES: [&str; 8] = ["1:1", "2:1", "3:1", "4:1", "6:1", "10:1", "20:1", "Nuke"];
/// Labels of `dist_detector`.
pub const DETECTOR_NAMES: [&str; 4] = ["Norm", "HP", "Band", "HP+Band"];
/// Labels of `dist_audio`.
pub const AUDIO_NAMES: [&str; 6] = ["Norm", "HP", "Dist 2", "Dist 3", "HP+Dist 2", "HP+Dist 3"];
/// Labels of `dist_link_mode`.
pub const LINK_MODE_NAMES: [&str; 3] = ["Phase", "Image", "Both"];

/// Panel range of the four continuous knobs: the Distressor's dials read
/// 0 to 10 with a little over-travel, and the panel numbers are arbitrary
/// (`research/Distressor.md` 2.1).
pub const KNOB_MAX: f32 = 10.5;
/// Fastest attack, seconds (50 µs, the published figure).
pub const ATTACK_MIN_S: f32 = 50e-6;
/// Attack at knob 10, seconds (30 ms, the published figure). The knob's
/// last half-mark runs past it, to about 41 ms.
pub const ATTACK_MAX_S: f32 = 30e-3;
/// Fastest release, seconds (50 ms, published).
pub const RELEASE_MIN_S: f32 = 50e-3;
/// Release at knob 10, seconds (3.5 s, published).
pub const RELEASE_MAX_S: f32 = 3.5;
/// Upper end of the shared side-chain high-pass; 0 turns it off.
pub const SC_HPF_MAX_HZ: f32 = 300.0;
/// Headroom range of the plug-in-only reference-level control, dB
/// (`research/Distressor.md` 7.2, after the UAD model).
pub const HEADROOM_MIN_DB: f32 = 4.0;
pub const HEADROOM_MAX_DB: f32 = 28.0;
/// Default headroom, dB.
pub const HEADROOM_DEFAULT_DB: f32 = 16.0;

/// Input / Output knob → gain in dB. The panel scale is arbitrary, so the
/// map is a piecewise-linear table with unity at 5 and about +25 dB fully
/// clockwise (`research/Distressor.md` 7.2; **estimate**, calibrated so
/// that "everything at 5" gives a few dB of gain reduction at 6:1 on a
/// −18 dBFS program).
pub const GAIN_TABLE_DB: [f32; 11] = [
    -60.0, -30.0, -20.0, -12.0, -6.0, 0.0, 5.5, 10.5, 15.0, 19.0, 23.5,
];

/// Interpolate [`GAIN_TABLE_DB`] at knob position `k` (0..[`KNOB_MAX`]);
/// below 0.1 the control is silent.
#[inline]
pub fn knob_to_db(k: f32) -> f32 {
    let k = k.clamp(0.0, KNOB_MAX);
    if k < 0.1 {
        return -120.0;
    }
    let i = (k.floor() as usize).min(GAIN_TABLE_DB.len() - 2);
    let f = k - i as f32;
    let a = GAIN_TABLE_DB[i];
    let b = GAIN_TABLE_DB[i + 1];
    // Above mark 10 the table runs out; keep the last slope.
    a + (b - a) * f
}

/// Attack knob → time constant in seconds, geometric between
/// [`ATTACK_MIN_S`] at 0 and [`ATTACK_MAX_S`] at 10
/// (`research/Distressor.md` 7.2).
#[inline]
pub fn attack_seconds(knob: f32) -> f32 {
    let k = knob.clamp(0.0, KNOB_MAX);
    ATTACK_MIN_S * (ATTACK_MAX_S / ATTACK_MIN_S).powf(k / 10.0)
}

/// Release knob → time constant in seconds, geometric between
/// [`RELEASE_MIN_S`] at 0 and [`RELEASE_MAX_S`] at 10.
#[inline]
pub fn release_seconds(knob: f32) -> f32 {
    let k = knob.clamp(0.0, KNOB_MAX);
    RELEASE_MIN_S * (RELEASE_MAX_S / RELEASE_MIN_S).powf(k / 10.0)
}

/// Ratio button. The eight positions are eight different curves, not one
/// curve with a slope control: the knee width, the effective slope and the
/// release shape all change (`research/Distressor.md` 7.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Ratio {
    /// 1:1, no gain reduction: the distortion modes on their own.
    R1,
    R2,
    R3,
    R4,
    /// The manufacturer's starting point.
    #[default]
    R6,
    /// The "opto" position: a two-stage release that stretches to seconds.
    R10,
    R20,
    /// Brick-wall limiting with a logarithmic release.
    Nuke,
}

impl Ratio {
    /// Every position in parameter order.
    pub const ALL: [Ratio; 8] = [
        Ratio::R1,
        Ratio::R2,
        Ratio::R3,
        Ratio::R4,
        Ratio::R6,
        Ratio::R10,
        Ratio::R20,
        Ratio::Nuke,
    ];

    /// From the parameter value / label index (clamped).
    pub fn from_index(i: usize) -> Self {
        Ratio::ALL.get(i).copied().unwrap_or(Ratio::R6)
    }

    /// The parameter value.
    pub fn index(self) -> usize {
        self as usize
    }

    /// The printed label ([`RATIO_NAMES`]).
    pub fn label(self) -> &'static str {
        RATIO_NAMES[self.index()]
    }
}

/// Detector switch: what the side-chain hears. Neither position touches
/// the audio path (`research/Distressor.md` 7.7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Detector {
    #[default]
    Norm,
    /// 100 Hz, 6 dB per octave into the side-chain: stops low-frequency
    /// pumping.
    Hp,
    /// A peaking boost around 6 kHz: the detector overreacts to sibilance.
    Band,
    HpBand,
}

impl Detector {
    pub fn from_index(i: usize) -> Self {
        match i {
            0 => Detector::Norm,
            1 => Detector::Hp,
            2 => Detector::Band,
            _ => Detector::HpBand,
        }
    }

    /// Is the side-chain high-pass in?
    pub fn hp(self) -> bool {
        matches!(self, Detector::Hp | Detector::HpBand)
    }

    /// Is the band emphasis in?
    pub fn band(self) -> bool {
        matches!(self, Detector::Band | Detector::HpBand)
    }
}

/// Audio switch: the 80 Hz high-pass and the two distortion modes, in the
/// combinations the hardware offers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AudioMode {
    #[default]
    Norm,
    Hp,
    /// Predominantly second harmonic, 0.05 % to about 3 %.
    Dist2,
    /// Predominantly third harmonic, up to about 20 %.
    Dist3,
    HpDist2,
    HpDist3,
}

impl AudioMode {
    pub fn from_index(i: usize) -> Self {
        match i {
            0 => AudioMode::Norm,
            1 => AudioMode::Hp,
            2 => AudioMode::Dist2,
            3 => AudioMode::Dist3,
            4 => AudioMode::HpDist2,
            _ => AudioMode::HpDist3,
        }
    }

    /// Is the audio high-pass in?
    pub fn hp(self) -> bool {
        matches!(
            self,
            AudioMode::Hp | AudioMode::HpDist2 | AudioMode::HpDist3
        )
    }

    /// Which harmonic generator is in: 0 none, 2 Dist 2, 3 Dist 3.
    pub fn distortion(self) -> u8 {
        match self {
            AudioMode::Dist2 | AudioMode::HpDist2 => 2,
            AudioMode::Dist3 | AudioMode::HpDist3 => 3,
            _ => 0,
        }
    }
}

/// How the two channels are tied together when the Link button is in
/// (`research/Distressor.md` 7.9).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LinkMode {
    /// The original EL8: the rectified side-chains are summed, so the gain
    /// reduction is common but the image can still shift.
    #[default]
    Phase,
    /// The EL8-X: the computed gain-control signals are summed, so both
    /// channels get identical gain reduction and the image is locked.
    Image,
    /// Both.
    Both,
}

impl LinkMode {
    pub fn from_index(i: usize) -> Self {
        match i {
            0 => LinkMode::Phase,
            1 => LinkMode::Image,
            _ => LinkMode::Both,
        }
    }
}

/// Everything the engine needs from the parameters, read once per block.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// Input knob, 0..[`KNOB_MAX`]. The internal threshold is fixed, so
    /// this is what sets how much compression there is.
    pub input: f32,
    /// Output knob, 0..[`KNOB_MAX`].
    pub output: f32,
    /// Attack knob, 0..[`KNOB_MAX`].
    pub attack: f32,
    /// Release knob, 0..[`KNOB_MAX`].
    pub release: f32,
    pub ratio: Ratio,
    pub detector: Detector,
    pub audio: AudioMode,
    /// The 1176 "all buttons in" treatment: a fixed curve with a raised
    /// threshold, sped-up timing, an attack lag and more grunge.
    pub british: bool,
    pub link_mode: LinkMode,
    /// Internal reference level in dB (plug-in only): how hard the
    /// distortion generator is driven for a given signal.
    pub headroom_db: f32,
    /// Share one detector between the channels (the Link button).
    pub link: bool,
    /// Wet share, 0..1.
    pub mix: f32,
    /// Side-chain high-pass corner in Hz, 0 = off (the lab's shared knob,
    /// on top of the Detector switch's own 100 Hz position).
    pub sc_hpf_hz: f32,
    pub bypass: bool,
}

impl Default for Settings {
    /// The manual's starting point: everything at 5, 6:1, no filters, no
    /// distortion.
    fn default() -> Self {
        Settings {
            input: 5.0,
            output: 5.0,
            attack: 5.0,
            release: 5.0,
            ratio: Ratio::R6,
            detector: Detector::Norm,
            audio: AudioMode::Norm,
            british: false,
            link_mode: LinkMode::Phase,
            headroom_db: HEADROOM_DEFAULT_DB,
            link: true,
            mix: 1.0,
            sc_hpf_hz: 0.0,
            bypass: false,
        }
    }
}

#[cfg(test)]
mod tests;

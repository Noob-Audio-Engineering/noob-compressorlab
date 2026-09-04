/**
 * Factory presets, one list per model (`{ id: plain }` maps; anything of
 * that model not listed loads at its default), and the user presets, which
 * live in the plug-in's UI store under one key per model
 * (`presets.user.fet`, `presets.user.opto`, `presets.user.la3a`,
 * `presets.user.vca`, `presets.user.pre6176`, `presets.user.cl1b`, `presets.user.bridge`,
 * `presets.user.dbx`, `presets.user.gbus`,
 * `presets.user.tg`) so they persist with the
 * plug-in state and every window of the instance sees them.
 *
 * A preset only ever touches its own model's parameters and the shared
 * extras (see `presetSkip` in `composables/useLab.js`); switching models
 * leaves the other model's settings as they were. The 6176 owns both its
 * `pre_*` section and the `fet_*` compressor it drives, so its presets set
 * both.
 * @typedef {{ name: string, description?: string, values: Record<string, number> }} Preset
 */
import { getClient } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';

/** @type {Record<'fet' | 'opto' | 'la3a' | 'vca' | 'pre6176' | 'cl1b' | 'bridge' | 'dbx' | 'tg' | 'gbus', Preset[]>} */
export const FACTORY_PRESETS = {
  // fet_ratio: 0 4:1, 1 8:1, 2 12:1, 3 20:1, 4 All. fet_revision: 0 A .. 7 H, 8 LN (see REVISIONS in models/fet/useFet.js).
  fet: [
    { name: 'Default', description: "The manufacturer's starting point: 24 / 24, attack 4, release 4, 4:1.", values: {} },
    {
      name: 'Vocal',
      description: 'Gentle 4:1 riding a lead vocal, medium attack, fast release, a little side-chain filtering.',
      values: { fet_input: 28, fet_output: 22, fet_attack: 3, fet_release: 6, fet_ratio: 0, fet_revision: 8, sc_hpf: 80 },
    },
    {
      name: 'Drums Punch',
      description: 'Slow attack lets the transient through, 8:1, fast release.',
      values: { fet_input: 30, fet_output: 20, fet_attack: 1.5, fet_release: 7, fet_ratio: 1 },
    },
    {
      name: 'Bass',
      description: '4:1 with a slower release so the low end does not pump, side-chain high-pass at 60 Hz.',
      values: { fet_input: 27, fet_output: 23, fet_attack: 4, fet_release: 2.5, fet_ratio: 0, sc_hpf: 60 },
    },
    {
      name: 'All-Button Smash',
      description: 'All buttons in, fast attack and release, driven hard. The room mic sound.',
      values: { fet_input: 34, fet_output: 16, fet_attack: 6, fet_release: 7, fet_ratio: 4, fet_revision: 0 },
    },
    {
      name: 'Parallel Crush',
      description: '20:1 slammed and blended in at 35 %.',
      values: { fet_input: 36, fet_output: 18, fet_attack: 7, fet_release: 7, fet_ratio: 3, mix: 35 },
    },
    {
      name: 'Blue Stripe Colour',
      description: 'Attack OFF: no compression, only the amplifiers and the transformers of the Rev A blue stripe.',
      values: { fet_input: 24, fet_output: 24, fet_attack: 0, fet_revision: 0 },
    },
    {
      name: 'Rev F Clean',
      description: 'The push-pull output stage of Rev F: the cleanest of the family, 8:1 on a mix bus.',
      values: { fet_input: 26, fet_output: 24, fet_attack: 2, fet_release: 4, fet_ratio: 1, fet_revision: 5 },
    },
  ],
  // opto_mode: 0 Compress, 1 Limit. opto_cell: 0 Silver, 1 Gray, 2 LA-2.
  opto: [
    { name: 'Init', description: 'Unity gain, gentle reduction, Compress.', values: {} },
    { name: 'Vocal', description: 'The classic: 4 to 8 dB of Compress on a lead vocal.', values: { opto_gain: 42, opto_peak_reduction: 55, opto_mode: 0, opto_emphasis: 1, opto_cell: 1 } },
    { name: 'Bass', description: 'Slower LA-2 cell, a little side-chain high-pass so the lows do not pump.', values: { opto_gain: 40, opto_peak_reduction: 60, opto_mode: 0, opto_cell: 2, sc_hpf: 60 } },
    { name: 'Mix Bus Glue', description: 'Barely touching the meter, blended in.', values: { opto_gain: 34, opto_peak_reduction: 35, opto_mode: 0, mix: 60 } },
    { name: 'Limit', description: 'Limit mode driven hard, for a wall.', values: { opto_gain: 50, opto_peak_reduction: 75, opto_mode: 1 } },
    { name: 'Airy', description: 'R37 pulled back so the sidechain ignores the lows.', values: { opto_gain: 40, opto_peak_reduction: 50, opto_emphasis: 0.2 } },
  ],
  // la3a_mode: 0 Compress, 1 Limit (a rear switch on the hardware).
  // la3a_emphasis is the rear HF Contour pot: 0 is flat, 1 makes the side-chain
  // hear 15 kHz about 10 dB hotter than the rest. Note the sense is the
  // opposite of the LA-2A's emphasis.
  la3a: [
    { name: 'Init', description: 'As the unit ships: Compress, contour flat, gain and reduction at a working setting.', values: {} },
    { name: 'Guitar', description: 'What the LA-3A is famous for: an electric guitar held in place, mid-forward.', values: { la3a_gain: 44, la3a_peak_reduction: 58, la3a_mode: 0, la3a_emphasis: 0 } },
    { name: 'Vocal, −3 to −5', description: "The manual's own operating point: raise Peak Reduction until the peaks read −3 to −5 dB.", values: { la3a_gain: 42, la3a_peak_reduction: 52, la3a_mode: 0, la3a_emphasis: 0.25 } },
    { name: 'De-Ess', description: 'HF Contour up, so the side-chain hears the top and rides sibilance instead of the body.', values: { la3a_gain: 40, la3a_peak_reduction: 55, la3a_emphasis: 0.9 } },
    { name: 'Room Mics', description: 'Heavy levelling on a drum room, side-chain high-passed.', values: { la3a_gain: 48, la3a_peak_reduction: 72, la3a_mode: 0, sc_hpf: 90 } },
    { name: 'Limit', description: 'The rear switch in LIMIT, which only shows itself once the unit is deep into compression.', values: { la3a_gain: 46, la3a_peak_reduction: 78, la3a_mode: 1 } },
    { name: 'Parallel', description: "UA's own trick: squash a copy hard and blend it back in at 40 %.", values: { la3a_gain: 52, la3a_peak_reduction: 85, mix: 40 } },
  ],
  // dist_ratio: 0 1:1, 1 2:1, 2 3:1, 3 4:1, 4 6:1, 5 10:1 (Opto), 6 20:1, 7 Nuke.
  // dist_audio: 0 Norm, 1 HP, 2 Dist 2, 3 Dist 3, 4 HP+Dist 2, 5 HP+Dist 3.
  vca: [
    { name: '5 5 5 5', description: "The manufacturer's own starting point: 6:1 with all four knobs at 5.", values: {} },
    {
      name: 'Vocal 4:1',
      description: 'A moderate 4:1 with the sidechain high-passed so plosives do not duck the whole line.',
      values: { dist_input: 6.2, dist_output: 5.4, dist_attack: 4, dist_release: 3.5, dist_ratio: 3, dist_detector: 1 },
    },
    {
      name: 'Drum Smash',
      description: 'Slow attack, fast release, 10:1 Opto, second-harmonic colour on the way out.',
      values: { dist_input: 7.5, dist_output: 4.5, dist_attack: 7, dist_release: 1.5, dist_ratio: 5, dist_audio: 2 },
    },
    {
      name: 'British Mode',
      description: 'The all-buttons character on 1:1, driven hard: the room-mic trick.',
      values: { dist_input: 8, dist_output: 4, dist_attack: 2.5, dist_release: 1, dist_ratio: 0, dist_british: 1, dist_audio: 3 },
    },
    {
      name: 'Nuke',
      description: 'Brick wall. Everything arrives at the same level, which is the point.',
      values: { dist_input: 8.5, dist_output: 4, dist_attack: 1, dist_release: 4, dist_ratio: 7 },
    },
    {
      name: 'Bass Opto',
      description: '10:1 Opto with the detector high-passed, so the low end drives it without pumping.',
      values: { dist_input: 6.5, dist_output: 5.5, dist_attack: 6, dist_release: 6, dist_ratio: 5, dist_detector: 1 },
    },
    {
      name: 'De-Harsh',
      description: 'Band emphasis in the sidechain: the compressor hears the top and rides it.',
      values: { dist_input: 6, dist_output: 5.5, dist_attack: 2, dist_release: 2.5, dist_ratio: 2, dist_detector: 2 },
    },
    {
      name: 'Mix Glue 2:1',
      description: 'The 30 dB knee doing invisible work on a bus, blended at 70 %.',
      values: { dist_input: 5.5, dist_output: 5.2, dist_attack: 5, dist_release: 5, dist_ratio: 1, mix: 70 },
    },
  ],
  // pre_gain: 0 −10 … 4 +10. pre_join: 0 Join, 1 BP, 2 1:1. pre_lf_gain / pre_hf_gain: 5 is 0 dB.
  pre6176: [
    { name: 'Unity', description: 'Clean preamp into the limiter at unity: gain 0, level 7, no EQ, 4:1.', values: {} },
    {
      name: 'Vocal Tube',
      description: 'Gain up and level back for tube colour, the top opened at 10 kHz, gentle 4:1.',
      values: { pre_gain: 3, pre_level: 5.5, pre_hf_freq: 2, pre_hf_gain: 7, pre_lf_freq: 0, pre_lf_gain: 4, fet_input: 27, fet_output: 23, fet_attack: 3, fet_release: 6 },
    },
    {
      name: 'Piano',
      description: "The manual's own recipe: +5 gain, a lift at 4.5 kHz, 8:1, moderate attack, short release.",
      values: { pre_gain: 3, pre_level: 7, pre_hf_freq: 0, pre_hf_gain: 7, fet_input: 28, fet_output: 22, fet_attack: 4, fet_release: 7, fet_ratio: 1 },
    },
    {
      name: 'DI Bass',
      description: 'Hi-Z in, low shelf at 70 Hz, the compressor holding it steady at 4:1.',
      values: { pre_input: 4, pre_gain: 2, pre_level: 8, pre_lf_freq: 0, pre_lf_gain: 7, fet_input: 27, fet_output: 23, fet_attack: 4, fet_release: 3, sc_hpf: 50 },
    },
    {
      name: 'Preamp Only',
      description: 'BP: the preamp straight to the output, the compressor out of the path. A tone box.',
      values: { pre_join: 1, pre_gain: 4, pre_level: 6 },
    },
    {
      name: 'Colour, No Squeeze',
      description: '1:1: no gain reduction, but the limiter amplifiers still colour the sound.',
      values: { pre_join: 2, pre_gain: 3, pre_level: 7, fet_input: 30, fet_output: 20 },
    },
    {
      name: 'Drum Room',
      description: 'Mic input driven hard, all buttons in on the compressor half.',
      values: { pre_input: 2, pre_gain: 4, pre_level: 8, fet_input: 33, fet_output: 17, fet_ratio: 4, fet_attack: 6, fet_release: 7 },
    },
  ],
  // cl1b: every continuous control runs 0..1 (the pot's own travel); the knobs sweep 239 degrees, so
  // 11 o'clock is 0.374 and 1 o'clock is 0.626. cl1b_mode: 0 Fixed, 1 Fix/Man, 2 Manual.
  cl1b: [
    {
      name: 'Vocal',
      description: "Lydkraft's own published vocal setting, which is where every control already starts: attack at 2 o'clock, release at 10 o'clock, meter on compression.",
      values: {},
    },
    {
      name: 'Bass, on the road',
      description: "Scotty Simpson's setting for the Oak Ridge Boys' bass rig: gain a shade above unity, attack around 11 o'clock, release around 1 o'clock, ratio near 2.5:1.",
      values: { cl1b_gain: 0.3, cl1b_ratio: 0.09, cl1b_threshold: 0.52, cl1b_attack: 0.374, cl1b_release: 0.626, cl1b_mode: 2 },
    },
    {
      name: 'Smooth and slow',
      description: 'Mine, not theirs: a low ratio and a long release for the glue this unit is known for, riding a couple of decibels and never letting go quickly.',
      values: { cl1b_gain: 0.34, cl1b_ratio: 0.15, cl1b_threshold: 0.62, cl1b_attack: 0.85, cl1b_release: 0.8, cl1b_mode: 2 },
    },
    {
      name: 'Fixed times',
      description: "Mine: the same idea with the attack and release knobs out of circuit, so the unit picks its own times from the programme. Ratio up for a firmer hand.",
      values: { cl1b_gain: 0.32, cl1b_ratio: 0.6, cl1b_threshold: 0.58, cl1b_mode: 0 },
    },
  ],
  // 33609: every value is a switch position, not a level. Thresholds count
  // detents, so limit index 8 is +8 dBu (23 stops from +4 in half decibels)
  // and compress index 5 is -10 dBu (16 stops from -20 in twos); gain index 2
  // is 4 dB (11 stops from 0 in twos); ratio 0..4 is 1.5:1 to 6:1.
  bridge: [
    {
      name: 'Bus',
      description: 'Where the unit is happiest: the compressor alone at 2:1, a gentle threshold and the 400 ms recovery, with the limiter out of circuit.',
      values: {},
    },
    {
      name: 'Bus, both in',
      description: 'The compressor doing the work with the limiter above it catching peaks. The two detectors listen at different points, so the make-up drives the limiter and leaves the compressor where it was.',
      values: { neve_limit_in: 1, neve_limit_threshold: 12, neve_limit_recovery: 4, neve_compress_threshold: -12, neve_compress_ratio: 1, neve_compress_recovery: 4, neve_gain: 4 },
    },
    {
      name: 'Firm',
      description: 'Mine: a lower threshold and 4:1 for a hand that stays on the material, with the automatic recovery so it lets go by programme rather than by clock.',
      values: { neve_compress_threshold: -16, neve_compress_ratio: 3, neve_compress_recovery: 5, neve_gain: 8 },
    },
    {
      name: 'Limiter only',
      description: 'The compressor out and the limiter in on its fast attack, which is the way the 2254 was used in front of a tape machine.',
      values: { neve_compress_in: 0, neve_limit_in: 1, neve_limit_attack: 1, neve_limit_threshold: 10, neve_limit_recovery: 1 },
    },
  ],
  // tg_mode: 0 Compress, 1 Out, 2 Limit. tg_recovery is the panel's 1 to 6
  // as 0 to 5, with no times attached to any of them, deliberately.
  // tg_output is the printed decibel, -10 to +10; the resistor ladder behind
  // it delivers 0.83 to 1.06 dB per step, which is the hardware's own error.
  /*
   * dbx's own application notes are the source for most of these: their
   * manual lists a starting point per source, and where it does the
   * description says so. `dbx_ratio` carries the coefficient the pot sets,
   * `alpha = 1 - 1/R`, so 4:1 is 0.75 and the infinity mark is 1.
   */
  dbx: [
    {
      name: 'Kick',
      description: "dbx's own kick-drum setting: 6:1 in OverEasy with the threshold set for about 15 dB of reduction. Their note says OverEasy takes slightly longer to react and so reduces the boominess of the body.",
      values: { dbx_model: 1, dbx_knee: 1, dbx_ratio: 5 / 6, dbx_threshold: -18, dbx_output: 6 },
    },
    {
      name: 'Vocal',
      description: "dbx's vocal starting point: a low-to-medium ratio around 4:1 with the threshold set for six to ten decibels of reduction.",
      values: { dbx_model: 1, dbx_knee: 1, dbx_ratio: 0.75, dbx_threshold: -10, dbx_output: 4, sc_hpf: 60 },
    },
    {
      name: 'Bass',
      description: "dbx's bass and electric-guitar setting: 4:1, threshold for ten to twelve decibels of reduction.",
      values: { dbx_ratio: 0.75, dbx_threshold: -14, dbx_output: 6 },
    },
    {
      name: 'Drum Bus',
      description: "dbx back the ratio off to 2:1 on a two-channel drum submix, in their words \u201cto avoid an excess of cymbal splattering\u201d. Linked, so the two channels sum their energies rather than their signals.",
      values: { dbx_ratio: 0.5, dbx_threshold: -12, dbx_output: 3, link: 1 },
    },
    {
      name: 'Sustain',
      description: "dbx's setting for sustain on a guitar or a synth pad: 10:1 to infinity, threshold to taste. This is the ten.",
      values: { dbx_ratio: 0.9, dbx_threshold: -20, dbx_output: 10 },
    },
    {
      name: 'Overload Guard',
      description: "dbx's digital-overload setting: hard knee at the infinity mark with the threshold a couple of decibels below clip. The mark is 120:1 rather than infinity, which dbx published and this model keeps, so a 40 dB overshoot still lifts the output by a third of a decibel.",
      values: { dbx_knee: 0, dbx_ratio: 1, dbx_threshold: 19, dbx_output: 0 },
    },
    {
      name: 'Line Amp',
      description: "dbx's own instruction for using it as a line amplifier: ratio fully anticlockwise at 1:1, threshold fully clockwise at 3 V, and the output gain to taste. Nothing compresses; you are hearing the audio path.",
      values: { dbx_ratio: 0, dbx_threshold: 11.76, dbx_output: 0 },
    },
    {
      name: 'Infinity+',
      description: "Past the infinity mark the coefficient exceeds one and the cell pulls down more than the input rose, so louder in is quieter out. dbx trademarked it and suggest striking a series of chords into it. The 160A only: the original's pot stops at the infinity mark.",
      values: { dbx_model: 1, dbx_ratio: 1.5, dbx_threshold: -20, dbx_output: 6 },
    },
  ],
  tg: [
    {
      name: 'Mastering',
      description: 'Waves, who built their model with Abbey Road, name recovery 3, 4 and 5 as the useful ones for mastering. This is the middle of those with the compressor in and nothing else touched.',
      values: {},
    },
    {
      name: 'Bus, driven',
      description: 'The module has no threshold, so how hard you drive it is how you set it. Six decibels in and three back out on the ladder, on the slower recovery so it holds between phrases.',
      values: { tg_input: 6, tg_output: -3, tg_recovery: 4 },
    },
    {
      name: 'Limit',
      description: 'The mode switch to LIMIT, which is a lower threshold on the same law rather than a harder ratio, and the fastest recovery. It is not a brick-wall limiter and transients are expected to pass.',
      values: { tg_mode: 2, tg_recovery: 0, tg_output: 2 },
    },
    {
      name: 'Dirty',
      description: 'Mine, and not the hardware: the drive control winds the gain element past where EMI ever ran it, and the arms are pushed out of balance so the even harmonics the matched pairs were meant to cancel come back.',
      values: { tg_input: 6, tg_drive: 60, tg_mismatch: 45, tg_recovery: 1, tg_output: -4 },
    },
  ],
  // ssl_attack is the panel's ladder as 0 to 5 (.1 .3 1 3 10 30 ms) and
  // ssl_release the console's as 0 to 4, with 4 the two-section automatic
  // network. ssl_ratio is 0 to 2 for 2:1, 4:1 and 10:1. The threshold is
  // marked as the panel marks it, so more negative compresses more.
  gbus: [
    {
      name: 'Bus',
      description: "The dossier's own defaults: 4:1 with the 1 ms attack and the automatic release, and the threshold centred. Where the box sits before anyone touches it.",
      values: {},
    },
    {
      name: 'Glue',
      description: 'The setting this compressor is famous for. A slow attack lets the transients through and pulls the body up behind them, and the automatic release lets go by programme rather than by clock.',
      values: { ssl_ratio: 0, ssl_attack: 5, ssl_release: 4, ssl_threshold: -6, ssl_makeup: 3 },
    },
    {
      name: 'Grab',
      description: 'Mine: the fastest attack and the highest ratio, with a quick release. The ratio rises as it works, so this bites harder the more you feed it.',
      values: { ssl_ratio: 2, ssl_attack: 0, ssl_release: 1, ssl_threshold: -12, ssl_makeup: 6 },
    },
    {
      name: 'No pumping',
      description: 'A fast release with the sidechain filter at 105 Hz, which is what the filter is there for: with a fast release a low tone modulates the gain at its own period and the result is intermodulation rather than pumping.',
      values: { ssl_ratio: 1, ssl_attack: 4, ssl_release: 0, ssl_hpf: 3, ssl_threshold: -8, ssl_makeup: 4 },
    },
  ],
};

/** The UI-store key of model `key`'s user presets. */
export const userKey = (key) => `presets.user.${key}`;

const list = (v) => (Array.isArray(v) ? v : []);

/**
 * @param {'fet' | 'opto'} key
 * @returns {Preset[]}
 */
export function loadUserPresets(key) {
  return list(getClient().store.get(userKey(key), []));
}

/**
 * @param {'fet' | 'opto'} key
 * @param {Preset[]} presets
 */
export function saveUserPresets(key, presets) {
  getClient().store.set(userKey(key), presets);
}

/** Re-run `fn` when any model's user presets change elsewhere (another window, a state restore). Returns an unsubscribe. */
export function onUserPresetsChange(fn) {
  return getClient().store.on('*', (k) => {
    if (k == null || String(k).startsWith('presets.user.')) fn();
  });
}

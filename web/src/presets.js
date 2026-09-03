/**
 * Factory presets, one list per model (`{ id: plain }` maps; anything of
 * that model not listed loads at its default), and the user presets, which
 * live in the plug-in's UI store under one key per model
 * (`presets.user.fet`, `presets.user.opto`, `presets.user.la3a`,
 * `presets.user.vca`, `presets.user.pre6176`) so they persist with the
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

/** @type {Record<'fet' | 'opto' | 'la3a' | 'vca' | 'pre6176', Preset[]>} */
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

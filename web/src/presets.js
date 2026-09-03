/**
 * Factory presets, one list per model (`{ id: plain }` maps; anything of
 * that model not listed loads at its default), and the user presets, which
 * live in the plug-in's UI store under one key per model
 * (`presets.user.fet`, `presets.user.opto`) so they persist with the
 * plug-in state and every window of the instance sees them.
 *
 * A preset only ever touches its own model's parameters and the shared
 * extras (see `presetSkip` in `composables/useLab.js`); switching models
 * leaves the other model's settings as they were.
 * @typedef {{ name: string, description?: string, values: Record<string, number> }} Preset
 */
import { getClient } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';

/** @type {Record<'fet' | 'opto', Preset[]>} */
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

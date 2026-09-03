/**
 * Noob CompressorLab specifics on top of the generic
 * `@noob-audio-engineering/noob-vst-webgui-framework/vue` bridge: the model switch, the handles
 * both models share, the page's one window-size instance, and the preset
 * helpers that only ever touch the active model's parameters.
 *
 * Everything here needs the manifest; call `useLab()` only once
 * `useNoobVstWebguiFramework().ready` is true (App.vue renders the page behind
 * `v-if="ready"`). Handles are cached by the framework, so every component
 * shares one subscription per parameter.
 */
import { computed, reactive } from 'vue';
import {
  getClient,
  hasParam,
  loadState as loadStateGeneric,
  stateToJson as stateToJsonGeneric,
  useParam,
  useNoobVstWebguiFramework,
  useWindowSize,
} from '@noob-audio-engineering/noob-vst-webgui-framework/vue';

export { getClient, hasParam, useParam, useNoobVstWebguiFramework };

/**
 * The models, in the order of the `model` parameter's steps. `key` is the
 * prefix of every parameter id the model owns (`fet_input`, `opto_gain`, …)
 * and the suffix of its store keys; `initPreset` is the name of its
 * factory default.
 * @type {{ key: 'fet' | 'opto', label: string, name: string, sub: string, initPreset: string }[]}
 */
export const MODELS = [
  { key: 'fet', label: '1176', name: 'NOOB 1176', sub: 'FET limiting amplifier', initPreset: 'Default' },
  { key: 'opto', label: 'LA-2A', name: 'NOOB LA-2A', sub: 'optical leveling amplifier', initPreset: 'Init' },
];

/** Smallest window the page lays out well in, `[width, height]` CSS pixels; `src/plugin.rs` clamps to the same. */
export const WINDOW_MIN = [900, 520];

let lab = null;

/**
 * The handles every model shares, resolved once: the model switch, the
 * extras (stereo link, mix, side-chain high-pass, bypass) and the
 * standalone's demo source when it is present. `active` is the entry of
 * `MODELS` the switch points at, `key` its prefix; both are computed refs.
 * @returns {{ model, active, key, link, mix, scHpf, bypass, source: null | { kind, level, freq } }}
 */
export function useLab() {
  if (lab) return lab;
  const model = useParam('model');
  const active = computed(() => MODELS[model.index] || MODELS[0]);
  lab = {
    model,
    active,
    key: computed(() => active.value.key),
    link: useParam('link'),
    mix: useParam('mix'),
    scHpf: useParam('sc_hpf'),
    bypass: useParam('bypass'),
    source: hasParam('src_kind') ? { kind: useParam('src_kind'), level: useParam('src_level'), freq: useParam('src_freq') } : null,
  };
  return lab;
}

/** Page-only state (not parameters): the preset name shown in the top bar, per model. */
export const ui = reactive({
  preset: Object.fromEntries(MODELS.map((m) => [m.key, m.initPreset])),
});

let win = null;

/**
 * The page's one `useWindowSize` instance (window size, resize requests,
 * fullscreen intent), created on first use from the root component so its
 * listeners live as long as the page; the top bar, the LA-2A view and the
 * grip share it. No aspect lock: each face keeps its own aspect and the
 * rest of the page takes what remains.
 */
export function useWindow() {
  win ??= useWindowSize({ min: WINDOW_MIN });
  return win;
}

// ---------------------------------------------------------------------------
// Presets: one list per model, applied to that model only
// ---------------------------------------------------------------------------

/**
 * What a preset of model `key` leaves alone: the model switch itself, the
 * other model's parameters, that model's meter selector (a view setting),
 * bypass and the demo source. The shared extras (link, mix, side-chain
 * high-pass) are part of a sound and load with it.
 * @param {'fet' | 'opto'} key
 * @returns {(id: string) => boolean}
 */
export function presetSkip(key) {
  return (id) =>
    id === 'model' ||
    id === 'bypass' ||
    id === `${key}_meter` ||
    id.startsWith('src_') ||
    MODELS.some((m) => m.key !== key && id.startsWith(`${m.key}_`));
}

/** `{ id: plain }` of the sound-defining parameters of model `key`. */
export function stateToJson(key) {
  return stateToJsonGeneric({ skip: presetSkip(key) });
}

/** Load `{ id: plain }` into model `key` in one frame, resetting the rest of that model's sound to defaults. */
export function loadState(key, values) {
  loadStateGeneric(values, { skip: presetSkip(key) });
}

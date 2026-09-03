/**
 * Noob CompressorLab specifics on top of the generic
 * `@noob-audio-engineering/noob-vst-webgui-framework/vue` bridge: the model switch, the handles
 * every model shares, the page's one window-size instance, and the preset
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
 * The models, in the order of the `model` parameter's steps. `key` names the
 * view, the store keys and the CSS class; `owns` lists the parameter-id
 * prefixes the model's sound lives in, and `meter` is its meter selector (a
 * view setting a preset leaves alone). The 6176 owns two prefixes: its own
 * 610 section and the 1176 half it drives, which is why ownership is a list
 * and not the key.
 * @type {{ key: 'fet' | 'opto' | 'la3a' | 'vca' | 'pre6176', label: string, name: string, sub: string, owns: string[], meter: string | null, initPreset: string }[]}
 */
export const MODELS = [
  { key: 'fet', label: '1176', name: 'NOOB 1176', sub: 'FET limiting amplifier', owns: ['fet_'], meter: 'fet_meter', initPreset: 'Default' },
  { key: 'opto', label: 'LA-2A', name: 'NOOB LA-2A', sub: 'optical leveling amplifier', owns: ['opto_'], meter: 'opto_meter', initPreset: 'Init' },
  { key: 'la3a', label: 'LA-3A', name: 'NOOB LA-3A', sub: 'solid-state optical leveler', owns: ['la3a_'], meter: 'la3a_meter', initPreset: 'Init' },
  { key: 'vca', label: 'Distressor', name: 'NOOB DISTRESSOR', sub: 'feedback VCA compressor', owns: ['dist_'], meter: null, initPreset: '5 5 5 5' },
  { key: 'pre6176', label: '6176', name: 'NOOB 6176', sub: 'tube preamp into the FET limiter', owns: ['pre_', 'fet_'], meter: 'pre_meter', initPreset: 'Unity' },
];

/** Smallest window the page lays out well in, `[width, height]` CSS pixels; `src/plugin.rs` clamps to the same. */
export const WINDOW_MIN = [900, 520];

let lab = null;

/**
 * The handles every model shares, resolved once: the model switch, the
 * extras (stereo link, mix, side-chain high-pass, bypass) and the
 * standalone's demo source when it is present. `active` is the entry of
 * `MODELS` the switch points at, `key` its view key; both are computed refs.
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
 * listeners live as long as the page; the top bar, the faces and the grip
 * share it. No aspect lock: each face keeps its own aspect and the rest of
 * the page takes what remains.
 */
export function useWindow() {
  win ??= useWindowSize({ min: WINDOW_MIN });
  return win;
}

// ---------------------------------------------------------------------------
// Presets: one list per model, applied to that model only
// ---------------------------------------------------------------------------

/** Every prefix any model claims, for deciding whether an id belongs to a model at all. */
const ALL_PREFIXES = [...new Set(MODELS.flatMap((m) => m.owns))];

/**
 * What a preset of model `key` leaves alone: the model switch itself, any
 * other model's parameters, this model's meter selector (a view setting),
 * bypass and the demo source. The shared extras (link, mix, side-chain
 * high-pass) are part of a sound and load with it. An id claimed by several
 * models — `fet_*`, which the 1176 and the 6176 share — is skipped only when
 * this model is not one of its owners.
 * @param {string} key
 * @returns {(id: string) => boolean}
 */
export function presetSkip(key) {
  const mine = MODELS.find((m) => m.key === key) || MODELS[0];
  const meters = MODELS.map((m) => m.meter).filter(Boolean);
  return (id) =>
    id === 'model' ||
    id === 'bypass' ||
    id.startsWith('src_') ||
    // this model's own meter selector, and every other model's selector too
    (meters.includes(id) && (id === mine.meter || !mine.owns.some((p) => id.startsWith(p)))) ||
    // a parameter that belongs to some model, but not to this one
    (ALL_PREFIXES.some((p) => id.startsWith(p)) && !mine.owns.some((p) => id.startsWith(p)));
}

/** `{ id: plain }` of the sound-defining parameters of model `key`. */
export function stateToJson(key) {
  return stateToJsonGeneric({ skip: presetSkip(key) });
}

/** Load `{ id: plain }` into model `key` in one frame, resetting the rest of that model's sound to defaults. */
export function loadState(key, values) {
  loadStateGeneric(values, { skip: presetSkip(key) });
}

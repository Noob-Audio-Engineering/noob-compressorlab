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
  useStoredRef,
  useWindowSize,
} from '@noob-audio-engineering/noob-vst-webgui-framework/vue';

export { getClient, hasParam, useParam, useNoobVstWebguiFramework, useStoredRef };

/**
 * Whether the development panel is shown, and whether it is expanded.
 *
 * It is meant to be there where debugging and demonstrating happen and
 * absent from a shipped plug-in, so it defaults on in offline design mode,
 * on the Vite dev server and under the standalone, and off inside a host.
 * The toggle beside SCOPE overrides that either way, and the choice lives
 * in the UI store, so it travels with the plug-in state and survives
 * reopening the editor.
 *
 * @returns {{ shown: import('vue').WritableComputedRef<boolean>, open: import('vue').WritableComputedRef<boolean>, byDefault: boolean }}
 */
export function useDebug() {
  if (debugState) return debugState;
  const stored = useStoredRef('debug.shown', null);
  const open = useStoredRef('debug.open', true);
  const byDefault = () => {
    const c = getClient();
    if (!c) return false;
    if (c.offline) return true;
    if (c.manifest && c.manifest.meta && c.manifest.meta.standalone) return true;
    return !!(import.meta && import.meta.env && import.meta.env.DEV);
  };
  debugState = {
    byDefault,
    shown: computed({
      get: () => (stored.value === null || stored.value === undefined ? byDefault() : !!stored.value),
      set: (v) => (stored.value = !!v),
    }),
    open: computed({ get: () => open.value !== false, set: (v) => (open.value = !!v) }),
  };
  return debugState;
}
let debugState = null;

/**
 * The models, in the order of the `model` parameter's steps. `key` names the
 * view, the store keys and the CSS class; `owns` lists the parameter-id
 * prefixes the model's sound lives in, and `meter` is its meter selector (a
 * view setting a preset leaves alone). The 6176 owns two prefixes: its own
 * 610 section and the 1176 half it drives, which is why ownership is a list
 * and not the key.
 * `family` groups them in the picker by the kind of compressor they are, and
 * `blurb` is the sentence that helps someone choose between them. Both are
 * declared here rather than inferred from `sub`, so a new model says what it
 * is instead of being parsed.
 * @type {{ key: 'fet' | 'opto' | 'la3a' | 'vca' | 'pre6176' | 'cl1b', label: string, name: string, sub: string, family: string, blurb: string, owns: string[], meter: string | null, initPreset: string }[]}
 */
export const MODELS = [
  { key: 'fet', label: '1176', name: 'NOOB 1176', sub: 'FET limiting amplifier', family: 'fet', blurb: 'Fast, forward and unmistakably aggressive. Ratio buttons, the all-buttons mode, and every revision from the blue stripe to the LN.', owns: ['fet_'], meter: 'fet_meter', initPreset: 'Default' },
  { key: 'opto', label: 'LA-2A', name: 'NOOB LA-2A', sub: 'optical leveling amplifier', family: 'optical', blurb: 'Slow, gentle and famously hard to misuse. One knob sets the amount and the T4 cell decides the rest.', owns: ['opto_'], meter: 'opto_meter', initPreset: 'Init' },
  { key: 'la3a', label: 'LA-3A', name: 'NOOB LA-3A', sub: 'solid-state optical leveler', family: 'optical', blurb: "The LA-2A's solid-state cousin: the same cell lit harder, quicker to grab and brighter with it.", owns: ['la3a_'], meter: 'la3a_meter', initPreset: 'Init' },
  { key: 'vca', label: 'Distressor', name: 'NOOB DISTRESSOR', sub: 'feedback VCA compressor', family: 'vca', blurb: 'Eight ratios from gentle to Nuke, two distortion modes, and a British setting that borrows the 1176 trick.', owns: ['dist_'], meter: null, initPreset: '5 5 5 5' },
  { key: 'pre6176', label: '6176', name: 'NOOB 6176', sub: 'tube preamp into the FET limiter', family: 'strip', blurb: 'A 610 tube preamp in front of the 1176, so the colour arrives before the compression does.', owns: ['pre_', 'fet_'], meter: 'pre_meter', initPreset: 'Unity' },
  { key: 'cl1b', label: 'CL-1B', name: 'NOOB CL 1B', sub: 'optical tube compressor', family: 'optical', blurb: 'The vocal one. Slow optical levelling with fixed or manual timing, and a ratio that rises as you push it.', owns: ['cl1b_'], meter: 'cl1b_meter', initPreset: 'Vocal' },
  { key: 'bridge', label: '33609', name: 'NOOB 33609', sub: 'diode-bridge limiter/compressor', family: 'bridge', blurb: 'A limiter and a compressor in series, each with its own detector. Every control is stepped, and the printed ratios are approximations.', owns: ['neve_'], meter: 'neve_meter_select', initPreset: 'Bus' },
];

/**
 * The families the picker groups by, in the order it shows them. A model
 * names its family; this names the family itself, so adding a seventh model
 * needs an entry in `MODELS` and nothing here unless it is a new kind.
 * @type {{ id: string, label: string, note: string }[]}
 */
export const FAMILIES = [
  { id: 'fet', label: 'FET', note: 'A field-effect transistor as the gain element: the fastest attack of the lot.' },
  { id: 'optical', label: 'Optical', note: 'A lamp and a photocell: slow, programme-dependent, and forgiving.' },
  { id: 'vca', label: 'VCA', note: 'A voltage-controlled amplifier: whatever ratio and timing you ask for.' },
  { id: 'strip', label: 'Channel strip', note: 'A preamp and a compressor in one box, in that order.' },
  { id: 'bridge', label: 'Diode bridge', note: 'Four diodes as the gain element: quiet about it until pushed, then unmistakable.' },
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
  /*
   * Whether the page is showing the browse view instead of the loaded
   * compressor. It is page state and not a parameter on purpose: browsing
   * must not touch the instance, so the engine never hears about it and the
   * compressor that is loaded keeps processing with its settings the whole
   * time the browser is up.
   */
  browsing: false,
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

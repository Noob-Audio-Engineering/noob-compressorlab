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
import { computed, reactive, watch } from 'vue';
import {
  getClient,
  hasParam,
  loadState as loadStateGeneric,
  stateToJson as stateToJsonGeneric,
  useParam,
  useNoobVstWebguiFramework,
  useStore,
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
 * @type {{ key: 'fet' | 'opto' | 'la3a' | 'vca' | 'pre6176' | 'cl1b' | 'bridge' | 'tg' | 'gbus', label: string, name: string, sub: string, family: string, blurb: string, uses: string, owns: string[], meter: string | null, initPreset: string }[]}
 */
export const MODELS = [
  { key: 'fet', label: '1176', name: 'NOOB 1176', sub: 'FET limiting amplifier', family: 'fet', blurb: 'Fast, forward and unmistakably aggressive. Ratio buttons, the all-buttons mode, and every revision from the blue stripe to the LN.', owns: ['fet_'], meter: 'fet_meter', uses: 'Snare, room mics, rock vocals and bass guitar. Hold all four ratio buttons down at once on a drum bus for the sound it is famous for.', initPreset: 'Default' },
  { key: 'opto', label: 'LA-2A', name: 'NOOB LA-2A', sub: 'optical leveling amplifier', family: 'optical', blurb: 'Slow, gentle and famously hard to misuse. One knob sets the amount and the T4 cell decides the rest.', owns: ['opto_'], meter: 'opto_meter', uses: 'Lead vocals, bass DI and acoustic guitar: anything you want levelled without hearing it happen.', initPreset: 'Init' },
  { key: 'la3a', label: 'LA-3A', name: 'NOOB LA-3A', sub: 'solid-state optical leveler', family: 'optical', blurb: "The LA-2A's solid-state cousin: the same cell lit harder, quicker to grab and brighter with it.", owns: ['la3a_'], meter: 'la3a_meter', uses: "Electric guitar, overheads and backing vocals. The LA-2A's move for material that needs it to let go sooner.", initPreset: 'Init' },
  { key: 'vca', label: 'Distressor', name: 'NOOB DISTRESSOR', sub: 'feedback VCA compressor', family: 'vca', blurb: 'Eight ratios from gentle to Nuke, two distortion modes, and a British setting that borrows the 1176 trick.', owns: ['dist_'], meter: null, uses: 'Drums first, then anything you want to hear working. Nuke across a room mic is its party trick.', initPreset: '5 5 5 5' },
  { key: 'pre6176', label: '6176', name: 'NOOB 6176', sub: 'tube preamp into the FET limiter', family: 'strip', blurb: 'A 610 tube preamp in front of the 1176, so the colour arrives before the compression does.', owns: ['pre_', 'fet_'], meter: 'pre_meter', uses: 'Tracking. Vocals and DI bass that want their colour printed on the way in rather than added afterwards.', initPreset: 'Unity' },
  { key: 'cl1b', label: 'CL-1B', name: 'NOOB CL 1B', sub: 'optical tube compressor', family: 'optical', blurb: 'The vocal one. Slow optical levelling with fixed or manual timing, and a ratio that rises as you push it.', owns: ['cl1b_'], meter: 'cl1b_meter', uses: 'Lead vocals and voiceover, where a slow hand and a ratio that rises under pressure hold a performance level without flattening it.', initPreset: 'Vocal' },
  { key: 'bridge', label: '33609', name: 'NOOB 33609', sub: 'diode-bridge limiter/compressor', family: 'diode', blurb: 'A limiter and a compressor in series, each with its own detector. Every control is stepped, and the printed ratios are approximations.', owns: ['neve_'], meter: 'neve_meter_select', uses: 'Mix bus, drum bus and broadcast. Two sections in series let you glue with one and catch the peaks with the other.', initPreset: 'Bus' },
  { key: 'dbx', label: '160', name: 'NOOB 160', sub: 'true-RMS VCA compressor', family: 'vca', blurb: 'The only one here that listens to power rather than peaks, so it lets the transient through and grabs the body. No attack or release: a true-RMS detector has one time constant and they are two sides of it. Past the infinity mark the ratio goes negative.', owns: ['dbx_'], meter: 'dbx_meter', uses: 'Kick, snare and bass, where you want the body squeezed and the stick left alone.', initPreset: 'Kick' },
  { key: 'tg', label: 'TG12413', name: 'NOOB TG 12413', sub: 'zener-diode limiter module', family: 'diode', blurb: 'A console module, not a rack unit. Three switches, no threshold and no ratio: you drive it and it decides, which is how the mastering desk at Abbey Road worked.', owns: ['tg_'], meter: null, uses: 'Mix bus and mastering, driven rather than dialled. You set the level going in and it decides how much, which is the point.', initPreset: 'Mastering' },
  { key: 'gbus', label: '4000 G', name: 'NOOB 4000 G', sub: 'stereo bus compressor', family: 'vca', blurb: 'The glue on the mix bus. A feedback design whose ratio rises as it works, so the curve bends for its whole length and never straightens, and an automatic release built from two RC sections that share their charge unevenly.', owns: ['ssl_'], meter: null, uses: 'The mix bus, and drums. The one you put across everything last and turn up until the track breathes.', initPreset: 'Bus' },
  { key: 'vmu', label: '670', name: 'NOOB 670', sub: 'variable-mu tube limiter', family: 'variablemu', blurb: 'The gain element is the amplifier. Eight tubes a channel do the amplifying and the limiting at once, so turning it up and dirtying it up are the same knob and no setting buys deep compression cleanly. Six time constants, two of which decide for themselves, and a stereo mode that is really mid-side.', owns: ['fc_'], meter: 'fc_meter_l', uses: 'Mix bus, drums and vocals, in that order. Set it for two or three decibels and it glues; push it and the colour arrives with the compression, because they are the same thing.', initPreset: 'Factory' },
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
  { id: 'variablemu', label: 'Variable-mu', note: 'The tube itself is the gain element: a remote-cutoff triode whose grid is wound with varying pitch, so a control voltage walks its transconductance down instead of switching it off. There is no attenuator anywhere in the audio path, which is why gain and distortion cannot be separated.' },
  { id: 'diode', label: 'Diode', note: 'Diodes as the gain element, in two forms that are not the same part. Neve close four of them into a ring with two floating nodes and one junction per arm, forward-biased by an injected current; EMI hang two branches of two off the supply rail and run them in reverse breakdown. Both are quiet until pushed. They are unmistakable in different ways.' },
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
  restorePresetNames();
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

/**
 * Page state that is not a parameter: the preset name shown in the top bar,
 * per model, and whether the browse view is up.
 *
 * **The preset names are persisted, and they have to be.** They are not
 * parameters — a preset's *name* changes no audio and belongs in no
 * automation lane — but a host closes and reopens the editor freely, and
 * each time it does the page is built again from nothing. Without this the
 * parameters came back correctly, because they are plug-in state, while the
 * top bar reset to every model's initial preset name: the panel said one
 * thing and the knobs another. So the map lives in the UI store, which is
 * serialised with the plug-in state, exactly as the user preset lists and
 * the debug toggle already are.
 */
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

/** Set once the preset names have been seeded from the store and hooked up to it. */
let presetsRestored = false;

/**
 * Seed [`ui.preset`] from the UI store and keep the store in step with it.
 *
 * Called from [`useLab`] rather than at module scope, because the store
 * needs a client and this module is imported before one exists. Names the
 * store does not carry keep the model's initial preset, so a project saved
 * before this existed opens with the old behaviour rather than a blank bar.
 */
function restorePresetNames() {
  if (presetsRestored) return;
  presetsRestored = true;
  const store = useStore();
  const stored = useStoredRef('preset.names', null);
  let seeded = false;

  /*
   * **Wait for the store before reading it, and only start writing after.**
   * The store arrives from the plug-in as a message, so at the moment a
   * component calls `useLab()` it is usually still empty. Seeding then would
   * read nothing, and attaching the writer then would save that nothing back
   * over the names the project actually holds — turning a display bug into a
   * data-loss one. `ready` is set when the plug-in's `store.all` lands, and
   * immediately in offline design mode.
   */
  const seed = () => {
    if (seeded) return;
    seeded = true;
    const saved = stored.value;
    if (saved && typeof saved === 'object') {
      for (const m of MODELS) if (typeof saved[m.key] === 'string') ui.preset[m.key] = saved[m.key];
    }
    // A shallow copy per change: the map is eleven short strings, and a deep
    // watcher on a reactive object would fire on its own writes.
    watch(
      () => ({ ...ui.preset }),
      (v) => {
        stored.value = v;
      },
    );
  };

  if (store.ready) seed();
  else {
    const stop = watch(
      () => store.ready,
      (r) => {
        if (!r) return;
        stop();
        seed();
      },
    );
  }
}

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

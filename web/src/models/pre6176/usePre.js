/**
 * The 6176 page's specifics: the handles of its 610 preamp section
 * (`pre_*`), the handles of the 1176 half it drives (the lab's existing
 * `fet_*` parameters, unchanged), and the scale mappings that let the
 * 6176's own knob markings drive them.
 *
 * The 6176 prints different numbers on the same circuit: Input and Output
 * read 0 to 10 with unity near 5 where a standalone 1176 prints attenuation
 * marks 0 to 48, and Attack and Release read 1 to 9 where the 1176 reads
 * 1 to 7. Both are page-side mappings only; the DSP is the lab's 1176
 * engine untouched, so a sound set here and a sound set on the 1176 face
 * mean the same thing to the compressor.
 *
 * Rules of use: `useControls()` looks parameters up by id, so call it (and
 * anything that uses it) only once `ready` is true.
 */
import { reactive } from 'vue';
import { useLab, useParam } from '../../composables/useLab.js';

let controls = null;

/**
 * The parameter handles of the panel, resolved once: the preamp section,
 * the compressor half and the shared extras.
 * @returns {object}
 */
export function useControls() {
  if (controls) return controls;
  const lab = useLab();
  controls = {
    // the 610 section
    join: useParam('pre_join'),
    gain: useParam('pre_gain'),
    inputSel: useParam('pre_input'),
    pad: useParam('pre_pad'),
    polarity: useParam('pre_polarity'),
    level: useParam('pre_level'),
    lfFreq: useParam('pre_lf_freq'),
    lfGain: useParam('pre_lf_gain'),
    hfFreq: useParam('pre_hf_freq'),
    hfGain: useParam('pre_hf_gain'),
    hpf: useParam('pre_hpf'),
    phantom: useParam('pre_phantom'),
    voice: useParam('pre_voice'),
    load: useParam('pre_load'),
    meter: useParam('pre_meter'),
    // the 1176 half, as it already exists
    input: useParam('fet_input'),
    output: useParam('fet_output'),
    attack: useParam('fet_attack'),
    release: useParam('fet_release'),
    ratio: useParam('fet_ratio'),
    revision: useParam('fet_revision'),
    // shared
    link: lab.link,
    mix: lab.mix,
    scHpf: lab.scHpf,
    bypass: lab.bypass,
    source: lab.source,
  };
  return controls;
}

/** Page state that is not a parameter: whether the analysis drawer is open. */
export const ui = reactive({
  scope: true,
});

// ---------------------------------------------------------------------------
// The 6176's own scales over the 1176's controls
// ---------------------------------------------------------------------------

/**
 * Input and Output: the 6176 prints 0 to 10 with unity at about 5, where
 * the 1176's dial prints attenuation marks 0 to 48 (mark m is m − 48 dB).
 * This table pairs the printed number with the mark it lands on
 * (research/610.md 8.4); between entries the mapping is linear.
 */
const SCALE_TABLE = [
  [0, 0],
  [1, 0],
  [2, 8],
  [3, 15],
  [4, 20],
  [5, 24],
  [6, 28],
  [7, 33],
  [8, 38],
  [9, 43],
  [10, 48],
];

function interp(table, x, from, to) {
  if (x <= table[0][from]) return table[0][to];
  for (let i = 1; i < table.length; i++) {
    const [a, b] = [table[i - 1], table[i]];
    if (x <= b[from]) return a[to] + ((x - a[from]) / (b[from] - a[from])) * (b[to] - a[to]);
  }
  return table[table.length - 1][to];
}

/** The 1176 mark a printed 6176 number (0..10) stands for. */
export const scaleToMark = (s) => interp(SCALE_TABLE, s, 0, 1);
/** The printed 6176 number of a 1176 mark. */
export const markToScale = (m) => interp(SCALE_TABLE, m, 1, 0);
/** Rotation fraction of a printed number on the Input / Output knob. */
export const scaleToRotation = (s) => s / 10;
export const rotationToScale = (r) => r * 10;

/**
 * Attack and Release: the 6176 prints 1 to 9 (Slow to Fast) where the 1176
 * prints 1 to 7 in the same direction. Attack keeps the detent past fully
 * counter-clockwise, which is the 1176's OFF.
 */
export const timeToScale = (v) => (v < 0.5 ? 0 : 1 + ((v - 1) * 8) / 6);
export const scaleToTime = (s) => (s < 0.5 ? 0 : 1 + ((s - 1) * 6) / 8);

/**
 * Rotation of the two time knobs. Attack keeps the click detent past fully
 * counter-clockwise that the 6176 prints as OFF, so its travel is that
 * detent and then 1 to 9 over the rest; Release has no OFF and uses the
 * whole sweep.
 */
export const attackToRotation = (v) => (v < 0.5 ? 0 : 0.12 + ((timeToScale(v) - 1) / 8) * 0.88);
export const rotationToAttack = (r) => (r < 0.06 ? 0 : scaleToTime(1 + ((Math.max(0.12, r) - 0.12) / 0.88) * 8));
export const releaseToRotation = (v) => (timeToScale(v) - 1) / 8;
export const rotationToRelease = (r) => scaleToTime(1 + r * 8);

export const SCALE_MARKS = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10].map((v) => ({ value: v, label: String(v) }));
export const TIME_MARKS = [1, 2, 3, 4, 5, 6, 7, 8, 9].map((v) => ({ value: v, label: String(v) }));

/**
 * The RATIO rotary of the 6176, which carries two positions a standalone
 * 1176 does not have: BP routes the preamp straight to the output, and 1 is
 * 1:1, no gain reduction but the limiter's amplifiers still in the path.
 * From 4 on it is the 1176's own ratio switch. Selecting a position
 * therefore writes the routing parameter, the ratio parameter, or both.
 * @type {{ label: string, join: number, ratio: number | null, hint: string }[]}
 */
export const RATIO_POSITIONS = [
  { label: 'BP', join: 1, ratio: null, hint: 'The preamp straight to the output: the compressor and its tone out of the path' },
  { label: '1', join: 2, ratio: null, hint: 'No gain reduction, but the limiter amplifiers still colour the sound' },
  { label: '4', join: 0, ratio: 0, hint: '4:1' },
  { label: '8', join: 0, ratio: 1, hint: '8:1' },
  { label: '12', join: 0, ratio: 2, hint: '12:1' },
  { label: '20', join: 0, ratio: 3, hint: '20:1' },
  { label: 'ALL', join: 0, ratio: 4, hint: 'All four buttons in: the overdriven all-buttons mode' },
];

/** Which RATIO position the current routing and ratio add up to. */
export function ratioIndex(join, ratio) {
  if (join === 1) return 0;
  if (join === 2) return 1;
  return 2 + Math.min(4, Math.max(0, ratio));
}

/** The printed 610 Gain steps, in the order of the `pre_gain` parameter. */
export const GAIN_MARKS = ['-10', '-5', '0', '+5', '+10'].map((label, value) => ({ value, label }));
/**
 * The eleven shelf steps. Every one gets a tick, but only the whole-number
 * ones get a numeral: eleven labels around a knob this size cannot be read
 * at the 900 px window the plug-in allows, and the exact step shows in the
 * read-out while the knob is turning.
 */
export const SHELF_MARKS = ['-9', '-6', '', '-3', '', '0', '', '+3', '', '+6', '+9'].map((label, value) => ({ value, label }));
/**
 * The input selector's five positions, printed as the hardware prints them:
 * short tokens around the knob with the "Mic" and "Hi-Z" group words on the
 * panel above. The parameter's order is Line, Mic 500, Mic 2.0K, Hi-Z 47K,
 * Hi-Z 2.2M, but the hardware's rotary runs 500, 2.0K, Line, 47K, 2.2M from
 * lower left to lower right, so the two are matched by placing each index at
 * its own rotation rather than by reordering the parameter.
 */
export const INPUT_MARKS = ['Line', '500', '2.0K', '47K', '2.2M'].map((label, value) => ({ value, label }));
/** Rotation fraction of each input-selector position, in panel order. */
const INPUT_ROTATION = [0.5, 0, 0.25, 0.75, 1];
export const inputSelToRotation = (index) => INPUT_ROTATION[Math.round(index)] ?? 0.5;
export const inputSelFromRotation = (rot) => {
  let best = 0;
  for (let i = 1; i < INPUT_ROTATION.length; i++) if (Math.abs(INPUT_ROTATION[i] - rot) < Math.abs(INPUT_ROTATION[best] - rot)) best = i;
  return best;
};
/** Input and Output print these numbers; the steps between them stay bare ticks. */
export const SCALE_LABELLED = new Set([0, 2, 4, 5, 6, 8, 10]);
/** The meter selector's three positions. */
export const METER_MARKS = ['PRE', 'GR', 'COMP'].map((label, value) => ({ value, label }));

/**
 * The 1176 page's specifics: its parameter handles grouped by panel
 * section (ids prefixed `fet_`, the shared extras from `useLab`), the knob
 * tapers of the original front panel, the revisions and their looks, and a
 * little UI state.
 *
 * Rules of use: `useControls()` looks parameters up by id, so call it
 * (and anything that uses it) only once `ready` is true.
 */
import { reactive } from 'vue';
import { useLab, useParam } from '../../composables/useLab.js';

/**
 * The revisions of the `fet_revision` parameter, in label order (mirrors
 * the revision names in `src/dsp/mod.rs`): the printed label, the
 * faceplate look the panel draws, and a short hint for the selector.
 * @type {{ label: string, look: 'bluestripe' | 'blackface' | 'silverface', hint: string }[]}
 */
export const REVISIONS = [
  { label: 'A', look: 'bluestripe', hint: 'Bluestripe, FET preamp' },
  { label: 'B', look: 'bluestripe', hint: 'Bluestripe, bipolar preamp' },
  { label: 'C', look: 'blackface', hint: 'Blackface, LN module' },
  { label: 'D', look: 'blackface', hint: 'Blackface, LN on board' },
  { label: 'E', look: 'blackface', hint: 'Blackface, as D' },
  { label: 'F', look: 'blackface', hint: 'Blackface, push-pull output' },
  { label: 'G', look: 'blackface', hint: 'Blackface, transformerless input' },
  { label: 'H', look: 'silverface', hint: 'Silverface, the G circuit' },
  { label: 'LN', look: 'blackface', hint: 'Reissue, C / D / E circuit' },
];

/** The faceplate look of a revision index. */
export const lookOf = (index) => (REVISIONS[index] || REVISIONS[REVISIONS.length - 1]).look;

let controls = null;

/**
 * The parameter handles of the panel, resolved once.
 * @returns {{ input, output, attack, release, ratio, meter, revision, link, mix, scHpf, bypass, source: { kind, freq, level } | null }}
 */
export function useControls() {
  if (controls) return controls;
  const lab = useLab();
  controls = {
    input: useParam('fet_input'),
    output: useParam('fet_output'),
    attack: useParam('fet_attack'),
    release: useParam('fet_release'),
    ratio: useParam('fet_ratio'),
    meter: useParam('fet_meter'),
    revision: useParam('fet_revision'),
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
// Knob tapers
// ---------------------------------------------------------------------------

/**
 * The Input / Output dial: printed marks are attenuation from full
 * clockwise (mark m is m − 48 dB) but the pot is not linear in angle. This
 * table (research/1176.md, 7.2) maps a mark to the fraction of the
 * rotation, so the page draws the marks where the panel prints them and a
 * drag feels like the real pot. Between the entries the mapping is linear.
 */
export const MARK_TAPER = [
  [0, 0.08],
  [6, 0.14],
  [12, 0.2],
  [18, 0.33],
  [24, 0.5],
  [30, 0.62],
  [36, 0.74],
  [42, 0.86],
  [48, 1.0],
];

function interp(table, x, from, to) {
  if (x <= table[0][from]) return table[0][to];
  for (let i = 1; i < table.length; i++) {
    const [a, b] = [table[i - 1], table[i]];
    if (x <= b[from]) return a[to] + ((x - a[from]) / (b[from] - a[from])) * (b[to] - a[to]);
  }
  return table[table.length - 1][to];
}

/** Rotation fraction (0..1) of a mark value on the Input / Output dial. */
export const markToRotation = (mark) => interp(MARK_TAPER, mark, 0, 1);
/** Mark value of a rotation fraction on the Input / Output dial. */
export const rotationToMark = (rot) => interp(MARK_TAPER, rot, 1, 0);

/** The Attack dial: OFF detent at the start, then 1..7 spread over the rest. */
export const attackToRotation = (v) => (v < 0.5 ? 0 : 0.12 + ((v - 1) / 6) * 0.88);
export const rotationToAttack = (r) => (r < 0.06 ? 0 : 1 + ((Math.max(0.12, r) - 0.12) / 0.88) * 6);

/** The Release dial: 1..7 linear. */
export const releaseToRotation = (v) => (v - 1) / 6;
export const rotationToRelease = (r) => 1 + r * 6;

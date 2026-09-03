/**
 * The Distressor page's specifics: its parameter handles, the tables the
 * faceplate prints (the eight ratios and their LED colours, the two
 * cycling filter selectors, the gain-reduction bargraph's steps) and the
 * knob taper of the original front panel.
 *
 * Rules of use: `useControls()` looks parameters up by id, so call it (and
 * anything that uses it) only once `ready` is true.
 */
import { reactive } from 'vue';
import { useStoredRef } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';
import { useLab, useParam } from '../../composables/useLab.js';

let controls = null;

/**
 * The parameter handles of the panel, resolved once.
 * @returns {{ input, output, attack, release, ratio, detector, audio, british, linkMode, headroom, link, mix, scHpf, bypass, source }}
 */
export function useControls() {
  if (controls) return controls;
  const lab = useLab();
  controls = {
    input: useParam('dist_input'),
    output: useParam('dist_output'),
    attack: useParam('dist_attack'),
    release: useParam('dist_release'),
    ratio: useParam('dist_ratio'),
    detector: useParam('dist_detector'),
    audio: useParam('dist_audio'),
    british: useParam('dist_british'),
    linkMode: useParam('dist_link_mode'),
    headroom: useParam('dist_headroom'),
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

/**
 * The two finishes the hardware has been sold in: the standard unit's black
 * and the anniversary edition's red. It changes nothing about the audio, so
 * it is not a parameter; it lives in the UI store, which travels with the
 * plug-in state, so a project remembers which one this instance wears.
 * @type {{ key: 'black' | 'red', label: string, hint: string }[]}
 */
export const FINISHES = [
  { key: 'black', label: 'BLACK', hint: 'The standard unit' },
  { key: 'red', label: 'RED', hint: 'The anniversary edition' },
];

let finishRef = null;

/** The finish this instance wears, remembered in the UI store. */
export function useFinish() {
  finishRef ??= useStoredRef('finish', 'black');
  return finishRef;
}

/**
 * The eight ratio LEDs, in the order of the `dist_ratio` parameter's steps.
 * The colours are the ones the hardware uses (research/Distressor.md 2.1):
 * green up to 4:1, amber for 6:1 and the 10:1 opto curve, red for 1:1 and
 * 20:1, and blue for Nuke, which the designer put there "just to be cool".
 * @type {{ label: string, colour: 'green' | 'amber' | 'red' | 'blue', hint: string }[]}
 */
export const RATIOS = [
  { label: '1:1', colour: 'red', hint: 'No compression — or the British curve with the toggle in' },
  { label: '2:1', colour: 'green', hint: 'The widest knee of the set, about 30 dB' },
  { label: '3:1', colour: 'green', hint: 'Gentle, standard knee' },
  { label: '4:1', colour: 'green', hint: 'The workhorse' },
  { label: '6:1', colour: 'amber', hint: 'Where the manual says to start' },
  { label: '10:1', colour: 'amber', hint: 'Opto: a two-stage release that stretches to twenty seconds' },
  { label: '20:1', colour: 'red', hint: 'Hard knee, near brick wall' },
  { label: 'Nuke', colour: 'blue', hint: 'Brick wall with a logarithmic release. The point is that everything arrives level' },
];

/**
 * The DETECTOR selector's four states and which of its lamps each lights.
 * The button cycles; the Link lamp is the shared stereo-link parameter, not
 * part of this selector.
 * @type {{ label: string, hp: boolean, band: boolean }[]}
 */
export const DETECTORS = [
  { label: 'Norm', hp: false, band: false },
  { label: 'HP', hp: true, band: false },
  { label: 'Band', hp: false, band: true },
  { label: 'HP+Band', hp: true, band: true },
];

/**
 * The AUDIO selector's six states: the audio-path high-pass and the two
 * distortion colours, second harmonic then third.
 * @type {{ label: string, hp: boolean, dist: 0 | 2 | 3 }[]}
 */
export const AUDIOS = [
  { label: 'Norm', hp: false, dist: 0 },
  { label: 'HP', hp: true, dist: 0 },
  { label: 'Dist 2', hp: false, dist: 2 },
  { label: 'Dist 3', hp: false, dist: 3 },
  { label: 'HP+Dist 2', hp: true, dist: 2 },
  { label: 'HP+Dist 3', hp: true, dist: 3 },
];

/**
 * The gain-reduction bargraph: sixteen lamps, silkscreened in dB from the
 * least reduction on the right to the most on the left. The colour bands
 * are the hardware's, with the green-to-amber boundary an estimate around
 * 5 to 6 dB (research/Distressor.md 2.1).
 * @type {{ db: number, colour: 'green' | 'amber' | 'red' }[]}
 */
export const GR_LAMPS = [26, 23, 20, 17, 14, 12, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1].map((db) => ({
  db,
  colour: db >= 12 ? 'red' : db >= 6 ? 'amber' : 'green',
}));

/** Step a cycling selector to its next state, the way the hardware's buttons do. */
export function cycle(p) {
  const n = p.labels?.length || 1;
  p.begin();
  p.setIndex((p.index + 1) % n);
  p.end();
}

// ---------------------------------------------------------------------------
// Knob taper
// ---------------------------------------------------------------------------

/**
 * The four big knobs are printed 0 to 10 and turn a little past ten, to
 * about 10.5; the scale is linear in angle, so the taper is the identity
 * over the parameter's range. The printed marks stop at 10.
 */
export const KNOB_MARKS = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10].map((v) => ({ value: v, label: String(v) }));
export const knobToRotation = (v) => v / 10.5;
export const rotationToKnob = (r) => r * 10.5;

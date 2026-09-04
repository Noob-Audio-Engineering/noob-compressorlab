/**
 * The Fairchild page's parameter handles and the scales its panel prints.
 *
 * Everything on the hardware's face is here and live, including the four
 * screwdriver adjustments: the two on the front panel that Fairchild put
 * there (ZERO and BAL) and the DC THRESHOLD that lives inside the chassis.
 * That last one is exposed deliberately. It is the ratio and knee control —
 * the manual's calibration procedure is entirely about setting it — and
 * every emulation that is any good brings it out to the front. Overloud say
 * so about their own in as many words.
 *
 * Three things about this panel need saying plainly.
 *
 * **It is a stereo unit whose two channels are meant to be set
 * differently.** Unlike the 33609, which is drawn here with both rows ganged
 * because the model has one set of parameters, the Fairchild has two of
 * everything and every one of them is its own parameter. That is the whole
 * point of the lateral-and-vertical mode: compressing the vertical component
 * harder than the lateral is a deliberate way of fitting a groove, and
 * Fairchild describe doing it.
 *
 * **The threshold's numbers are not decibels.** The ring is printed 0 to 10
 * and the pot is linear with a 24 kΩ resistor on its centre tap, so the law
 * has a kink in it; what the control sets, jointly with the DC threshold, is
 * a curve rather than a point. The engine owns that law
 * (`ac_threshold_law`), the panel prints what the metal prints, and neither
 * pretends it is a decibel scale.
 *
 * **The METERING lever is not a meter switch.** It reads plate current
 * through the output stage — the push leg, the centre tap, the pull leg —
 * and the ZERO screw beside it is a **bias trim wearing a
 * meter-calibration label**: adjusting it moves the operating point of all
 * eight 6386 sections, which is why it moves the needle. POM Audio Design,
 * who build new 670s, put it as "you are actually changing the Biasing of
 * the tubes, you are not changing the Vu calibration by design".
 *
 * Rules of use: `useControls()` looks parameters up by id, so call it (and
 * anything that uses it) only once `ready` is true.
 */
import { computed, reactive } from 'vue';
import { hasParam, useLab, useParam } from '../../composables/useLab.js';

let controls = null;

/**
 * The parameter handles of the panel, resolved once. Anything the engine may
 * not publish is resolved through `hasParam` and falls back to page state,
 * so the face never draws a control that writes nowhere.
 * @returns {object}
 */
export function useControls() {
  if (controls) return controls;
  const lab = useLab();
  const opt = (id) => (hasParam(id) ? useParam(id) : null);
  controls = {
    unit: opt('fc_model'),
    agc: opt('fc_agc'),
    tube: opt('fc_tube'),
    oversample: opt('fc_oversample'),
    inputGain: [useParam('fc_input_gain_l'), useParam('fc_input_gain_r')],
    threshold: [useParam('fc_threshold_l'), useParam('fc_threshold_r')],
    time: [useParam('fc_time_l'), useParam('fc_time_r')],
    dcThreshold: [useParam('fc_dc_threshold_l'), useParam('fc_dc_threshold_r')],
    zero: [useParam('fc_zero_l'), useParam('fc_zero_r')],
    balance: [useParam('fc_balance_l'), useParam('fc_balance_r')],
    meter: [useParam('fc_meter_l'), useParam('fc_meter_r')],
    link: lab.link,
    mix: lab.mix,
    scHpf: lab.scHpf,
    bypass: lab.bypass,
  };
  return controls;
}

/** Page state that is not a parameter: the analysis drawer, and the mains. */
export const ui = reactive({
  scope: true,
  powerOn: true,
});

// ---------------------------------------------------------------------------
// The scales, from research/Fairchild-670.md sections 2.2 and 2.3
// ---------------------------------------------------------------------------

/**
 * Sweeps in degrees.
 *
 * The dossier's 2.8 lists the knob styles among the things it could not
 * establish from a photograph, and it gives no sweeps either, so these are
 * read off `ref/fairchild-670-panel-inch-grid.png` and then rounded to the
 * index angle a rotary switch of that many ways actually comes in:
 *
 * - INPUT GAIN is a step attenuator of **21 detents**, 1 dB apart, and its
 *   printed numbers run from 20 at about seven o'clock round to 0 at about
 *   five. 21 ways at the standard 15° index is **300°**, which is where the
 *   photograph puts them.
 * - THRESHOLD is continuous, printed 0 to 10, and spans the same arc.
 * - TIME CONSTANT is **6 ways** and its numbers occupy a much smaller arc,
 *   1 at about ten o'clock and 6 at about two. Six ways at the other
 *   standard index, 30°, is **150°**.
 *
 * That the two rotary switches land on 15° and 30° is the check on the
 * reading: those are the plates the switches come in, and nothing was fitted
 * to make them come out that way.
 */
export const SWEEP = {
  inputGain: 300,
  threshold: 300,
  time: 150,
};

/** Build a scale: `n` detents, and a label at the indices that carry one. */
const scale = (n, pairs) => {
  const labels = {};
  for (const [i, l] of pairs) labels[i] = l;
  return Array.from({ length: n }, (_, i) => ({
    at: n === 1 ? 0.5 : i / (n - 1),
    label: labels[i] ?? null,
  }));
};

/**
 * INPUT GAIN: 21 detents of 1 dB, numbered on the evens and dotted between,
 * exactly as the panel prints it. The parameter is **attenuation**, so its
 * zero is fully clockwise and the ring is drawn from the wiper's travel.
 */
export const INPUT_GAIN_MARKS = scale(21, [
  [0, '0'], [2, '2'], [4, '4'], [6, '6'], [8, '8'], [10, '10'],
  [12, '12'], [14, '14'], [16, '16'], [18, '18'], [20, '20'],
]);

/** THRESHOLD: the ring is numbered 0 to 10 and the pot is continuous. */
export const THRESHOLD_MARKS = scale(11, [
  [0, '0'], [1, '1'], [2, '2'], [3, '3'], [4, '4'], [5, '5'],
  [6, '6'], [7, '7'], [8, '8'], [9, '9'], [10, '10'],
]);

/** TIME CONSTANT: six ways, numbered 1 to 6, which is all the panel says. */
export const TIME_MARKS = scale(6, [
  [0, '1'], [1, '2'], [2, '3'], [3, '4'], [4, '5'], [5, '6'],
]);

/**
 * What each time-constant position does, for the hint under the switch on
 * the extras strip. Fairchild's own paragraph is the most practical thing
 * anybody has written about this box, and these are its words plus the
 * published release times.
 */
export const TIME_HINTS = [
  '0.3 s — fast, for speech and popular music',
  '0.8 s — fast, for speech and popular music',
  '2 s — the manual’s general-purpose suggestion',
  '5 s — slow, for classical music',
  '2 s for individual peaks, lengthening to 10 s',
  '0.3 s for individual peaks, 10 s for multiple, 25 s for sustained',
];

/**
 * The mains button's handle: page state, because the hardware's ON switch is
 * a mains switch and the engine has no power parameter. It darkens the panel
 * and nothing else, which is stated here, in `FairToggle.vue` and in
 * `web/README.md` rather than left as an ornament.
 */
export function usePower() {
  return {
    real: false,
    on: computed(() => ui.powerOn),
    set: (v) => {
      ui.powerOn = v;
    },
  };
}

/**
 * The five published input/output curves of the December 1959 chart,
 * transcribed in the dossier's 7.2 and drawn behind the live one.
 *
 * Values are dBm out against dBm in, read off the chart by eye to about
 * ±0.5 dB. Curve 3 is the factory-adjusted condition, which is what the
 * plug-in's defaults reproduce.
 */
export const PUBLISHED_CURVES = [
  { id: 1, note: 'straight amplifier', points: [[0, 2.0], [5, 7.0], [10, 12.5], [15, 17.5], [20, 22.5]] },
  { id: 2, note: 'mild', points: [[0, 2.0], [5, 5.0], [10, 7.5], [15, 9.3], [20, 10.5]] },
  { id: 3, note: 'factory-adjusted', points: [[0, 2.0], [5, 4.3], [10, 5.3], [15, 5.7], [20, 5.9]] },
  { id: 4, note: 'maximum limiting', points: [[0, -4.5], [5, -2.5], [10, -1.0], [15, -0.3], [20, 0.0]] },
  { id: 5, note: 'high output', points: [[0, 2.0], [5, 7.0], [10, 9.5], [15, 10.0], [20, 10.2]] },
];

/**
 * The seven curves of the March 1959 IM chart, as the two numbers the
 * dossier's 4.6 reads off each: per cent IM at zero limiting, and the
 * decibels of limiting at which the curve reaches 9 %. Drawn as an
 * exponential through those two points, which is the shape the chart has.
 */
export const PUBLISHED_IM = [
  { out: 0, at0: 0.2, at9pct: 24 },
  { out: 4, at0: 0.2, at9pct: 20 },
  { out: 8, at0: 0.2, at9pct: 18 },
  { out: 12, at0: 0.2, at9pct: 16.5 },
  { out: 16, at0: 0.5, at9pct: 14 },
  { out: 20, at0: 1.6, at9pct: 10.5 },
  { out: 24, at0: 3.8, at9pct: 7 },
];

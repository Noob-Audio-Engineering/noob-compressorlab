/**
 * The Neve 33609 page's parameter handles and the tables its panel prints.
 *
 * Everything on the hardware's face is here and live: both threshold
 * switches, both recovery switches, the make-up gain, the ratio, the four
 * lever switches of the two processing blocks, the bypass, mono/stereo and
 * external-control levers of the centre block, and the mains button. Only
 * drive, mix and the side-chain high-pass are the lab's own, and those live
 * on the extras strip where we say so.
 *
 * Two things about this panel need saying plainly.
 *
 * **It is a stereo unit drawn with one set of parameters.** The hardware
 * has two complete channels, LIMIT 1 / COMPRESS 1 above and LIMIT 2 /
 * COMPRESS 2 below, each with its own switches. The model has one set, so
 * the face draws both rows as the panel does and both drive the same
 * handles: the two channels are permanently ganged, which is what a 33609
 * does with its MONO/STEREO lever in the ganged position and what a single
 * parameter set can honestly represent. Turning a knob in row one turns the
 * matching knob in row two because they really are one control here.
 *
 * **Every control is stepped.** AMS Neve make a feature of it, and the
 * research's section 2.3 read the true detent counts off the switch
 * assembly drawings rather than off the printed scales, which show fewer
 * numbers than there are positions. So the threshold switches have 23 and
 * 16 stops where the panel prints 12 and 9 numbers, and the make-up has 11
 * where the panel prints 6.
 *
 * Rules of use: `useControls()` looks parameters up by id, so call it (and
 * anything that uses it) only once `ready` is true.
 */
import { computed, reactive } from 'vue';
import { hasParam, useLab, useParam } from '../../composables/useLab.js';

let controls = null;

/**
 * The parameter handles of the panel, resolved once. Anything the engine
 * may not publish is resolved through `hasParam` and falls back to page
 * state, so the face never draws a control that writes nowhere.
 * @returns {object}
 */
export function useControls() {
  if (controls) return controls;
  const lab = useLab();
  const opt = (id) => (hasParam(id) ? useParam(id) : null);
  controls = {
    unit: opt('neve_model'),
    limitIn: opt('neve_limit_in'),
    limitThreshold: useParam('neve_limit_threshold'),
    limitAttack: opt('neve_limit_attack'),
    limitRecovery: useParam('neve_limit_recovery'),
    compressIn: opt('neve_compress_in'),
    compressThreshold: useParam('neve_compress_threshold'),
    compressRatio: useParam('neve_compress_ratio'),
    compressAttack: opt('neve_compress_attack'),
    compressRecovery: useParam('neve_compress_recovery'),
    gain: useParam('neve_gain'),
    meterSelect: opt('neve_meter_select'),
    drive: opt('neve_drive'),
    link: lab.link,
    mix: lab.mix,
    scHpf: lab.scHpf,
    bypass: lab.bypass,
  };
  return controls;
}

/** Page state that is not a parameter: the analysis drawer, and the mains fallback. */
export const ui = reactive({
  scope: true,
  powerOn: true,
  external: false,
});

/**
 * The mains button's handle: the engine's `neve_power` if it publishes one,
 * otherwise page state that only darkens the panel. Drawing a switch that
 * writes nowhere would be the ornament this project has removed twice, so
 * the fallback is stated in `NevePower.vue` and in `web/README.md`.
 */
let power = null;

export function usePower() {
  if (power) return power;
  if (hasParam('neve_power')) {
    const p = useParam('neve_power');
    power = { real: true, on: computed(() => p.on), set: (v) => { p.begin(); p.setOn(v); p.end(); } };
  } else {
    power = { real: false, on: computed(() => ui.powerOn), set: (v) => { ui.powerOn = v; } };
  }
  return power;
}

// ---------------------------------------------------------------------------
// The scales, from research/Neve-33609.md sections 2.2 and 2.3
// ---------------------------------------------------------------------------

/**
 * Sweep in degrees per control, measured off `ref/neve-33609n-front.jpg` by
 * reading the angle of every printed number about its knob's centre.
 *
 * Each switch then divides out to a whole number of degrees per detent, and
 * to one of the two standard rotary-switch index angles:
 *
 *   limit threshold     23 stops × 15°  = 330°
 *   compress threshold  16 stops × 15°  = 225°
 *   make-up gain        11 stops × 15°  = 150°
 *   both recoveries      6 stops × 30°  = 150°
 *   ratio                5 stops × 30°  = 120°
 *
 * That the measurements land on 15° and 30° is the check on them: those are
 * the index angles the switch plates come in, and nothing was fitted to make
 * them come out that way. The compress threshold's nine numbers sit on
 * detents 0, 2, 4 … 14 and 15, so its last printed step is half the width of
 * the others, and the photograph shows exactly that.
 *
 * My first pass guessed 270 / 270 / 240 / 220 from the look of the panel and
 * every one of those was wrong; the threshold knobs in particular turn much
 * further than they appear to, which is why their numbers crowded the
 * neighbouring knob before this was measured.
 */
export const SWEEP = {
  limitThreshold: 330,
  compressThreshold: 225,
  gain: 150,
  recovery: 150,
  ratio: 120,
};

/** Build a scale: `n` detents, and a label at the indices that carry one. */
const scale = (n, labels) =>
  Array.from({ length: n }, (_, i) => ({
    at: n === 1 ? 0.5 : i / (n - 1),
    label: labels[i] ?? null,
  }));

const sparse = (n, pairs) => {
  const labels = {};
  for (const [i, l] of pairs) labels[i] = l;
  return scale(n, labels);
};

/**
 * Limit threshold: ELMA SR10600 stopped at 23 ways, +4 to +15 dBu in half
 * decibel steps (drawing PL20238). The panel prints only the whole
 * decibels, so a label falls on every second detent.
 */
export const LIMIT_THRESHOLD_MARKS = sparse(23, [
  [0, '+4'], [2, '5'], [4, '6'], [6, '7'], [8, '8'], [10, '9'],
  [12, '10'], [14, '11'], [16, '12'], [18, '13'], [20, '14'], [22, '+15'],
]);

/** Limit recovery: six ways, the last two automatic (drawing PL20237). */
export const LIMIT_RECOVERY_MARKS = sparse(6, [
  [0, '50'], [1, '100'], [2, '200'], [3, '800'], [4, 'a1'], [5, 'a2'],
]);

/**
 * Compress threshold: sixteen ways, −20 to +10 dBu in 2 dB steps (drawing
 * PL20236). The panel prints nine of the sixteen, and prints them without
 * signs except at the ends, which is why the left half reads 16, 12, 8, 4
 * for negative values.
 */
export const COMPRESS_THRESHOLD_MARKS = sparse(16, [
  [0, '−20'], [2, '16'], [4, '12'], [6, '8'], [8, '4'],
  [10, '0'], [12, '4'], [14, '8'], [15, '+10'],
]);

/** Compress recovery: six ways, the last two automatic (drawing PL20235). */
export const COMPRESS_RECOVERY_MARKS = sparse(6, [
  [0, '100'], [1, '400'], [2, '800'], [3, '1500'], [4, 'a1'], [5, 'a2'],
]);

/** Make-up gain: eleven ways, 0 to 20 dB in 2 dB steps (drawing PL20234). */
export const GAIN_MARKS = sparse(11, [
  [0, '0'], [2, '4'], [4, '8'], [6, '12'], [8, '16'], [10, '20'],
]);

/**
 * Ratio: five ways (drawing PL20233), position one fully anticlockwise.
 *
 * The panel's numbers are approximations. The handbook's calibration table
 * gives the real slopes, and 3:1 is nearer 2.86:1 while 6:1 is nearer
 * 6.67:1 (research section 6). The engine models the real figures; the face
 * prints what the metal says, because that is what the metal says.
 */
export const RATIO_MARKS = sparse(5, [
  [0, '1.5:1'], [1, '2:1'], [2, '3:1'], [3, '4:1'], [4, '6:1'],
]);

/**
 * The gain-reduction meter's printed scale: decibels against degrees from
 * vertical, measured off `ref/neve-33609n-front.jpg`.
 *
 * These are not evenly spaced. Fitting a circle to the six ticks puts the
 * pivot 2.16 glass-heights below the top of the glass with the ticks on a
 * radius of 0.767 glass widths, residuals under a third of a pixel; read
 * against that centre the marks land here. The first 8 dB take 28.5 of the
 * 61.2 degrees of travel and the remaining 12 dB share the other 32.7, so
 * the working end of the scale is stretched and the deep end squeezed.
 *
 * Both movements measure the same to within a pixel although they sit at
 * different places in the frame, which is what rules out the lens. See the
 * header of `NeveMeter.vue`.
 */
export const METER_ANGLES = [
  [0, -30.8], [4, -17.2], [8, -2.3], [12, 8.3], [16, 19.5], [20, 30.4],
];

/**
 * The dbx 160 page's parameter handles, and the scales its two panels print.
 *
 * **Two panels, one set of controls.** `dbx_model` chooses which face is
 * drawn: the original 160, a wood-cheeked half-rack with three knobs, a VU
 * meter, three METER push buttons and two threshold indicators; or the 160A,
 * a black rack panel with red, blue and green knobs, four round buttons, a
 * third threshold indicator and two rows of LEDs. Both drive the same
 * parameters, because both units are the same three controls.
 *
 * That is why each dial here carries a `range` in normalised units. The two
 * hardware pots are not the same: the original's THRESHOLD runs 10 mV to
 * 3 V, which is −37.8 to +11.8 dBu, where the 160A's runs −40 to +20 dBu,
 * and the original's COMPRESSION stops at ∞ where the 160A's carries on to
 * −1:1. The parameter carries the union of each pair so that one control has
 * one meaning for a host, and each face maps its own pot's rotation onto the
 * part of it that unit has. Neither face gains a range dbx did not give it,
 * and the engine clamps to the same limits so a preset written on one face
 * cannot smuggle the other's in.
 *
 * **Where the numbers come from.** The original's dial positions were
 * measured off dbx's own front-panel figure from the 160/161 instruction manual (archive.org/details/dbx_dbx_160, leaf 3), dbx's own front-panel drawing
 * enlarged, by fitting circles to the knobs and then reading the tick marks
 * as angular clusters of dark pixels in the annulus just outside each. The
 * 160A's were read off dbx's own 160A product photograph (adn.harmanpro.com/product_attachments/product_attachments/614_1728149313/160Afront_lg_original.jpg), which prints
 * labels rather than ticks and so is a weaker source; that is said where it
 * matters. `Faceplate160.vue` carries the panel geometry and its own note on
 * how it was measured.
 *
 * Rules of use: `useControls()` looks parameters up by id, so call it (and
 * anything that uses it) only once `ready` is true.
 */
import { computed, onBeforeUnmount, onMounted, reactive, ref } from 'vue';
import { hasParam, useLab, useParam } from '../../composables/useLab.js';

let controls = null;

/**
 * The panel's handles, resolved once. Anything the engine may not publish
 * comes back `null` so the face never draws a control that writes nowhere.
 * @returns {object}
 */
export function useControls() {
  if (controls) return controls;
  const lab = useLab();
  const opt = (id) => (hasParam(id) ? useParam(id) : null);
  controls = {
    unit: opt('dbx_model'),
    threshold: useParam('dbx_threshold'),
    ratio: useParam('dbx_ratio'),
    output: useParam('dbx_output'),
    knee: opt('dbx_knee'),
    meter: opt('dbx_meter'),
    meterCal: opt('dbx_meter_cal'),
    kneeWidth: opt('dbx_knee_width'),
    tau: opt('dbx_tau'),
    lookahead: opt('dbx_lookahead'),
    headroom: opt('dbx_headroom'),
    // The 160A's SLAVE button is the strapping jack and its BYPASS is the
    // relay: the lab's shared controls are exactly those, so the panel's
    // buttons drive them rather than a second copy.
    link: lab.link,
    bypass: lab.bypass,
    mix: lab.mix,
    scHpf: lab.scHpf,
  };
  return controls;
}

/** Page state that is not a parameter: the analysis drawer. */
export const ui = reactive({ scope: true });

/** Which face is drawn. 0 is the original 160, 1 the 160A. */
export function useUnit() {
  const c = useControls();
  return computed(() => (c.unit ? c.unit.index : 0));
}

// ---------------------------------------------------------------------------
// The original 160's dials, measured off dbx's own drawing
// ---------------------------------------------------------------------------

/**
 * Sweeps in degrees, from dbx's own front-panel figure from the 160/161 instruction manual (archive.org/details/dbx_dbx_160, leaf 3).
 *
 * Each knob's circle was found by scoring rings against the drawing — the
 * fits are exact, so the drawing's circles really are circles — and the
 * ticks read as clusters in the annulus outside. The figure's callout arrows
 * cross two of the knobs and were separated by taking a second, wider
 * annulus, where only an arrow's shaft survives.
 *
 * | dial | ticks | first | last | sweep |
 * |---|---|---|---|---|
 * | THRESHOLD | 6 | −132.8° | +128.8° | 261.6° |
 * | COMPRESSION | 9 | −155.2° | +150.5° | 305.7° |
 * | OUTPUT GAIN | 9 | −146.8° | +149.2° | 296.0° |
 *
 * The threshold and output dials are drawn with visibly uneven tick
 * spacing — up to 4.5° and 12° off even — but both are linear in decibels
 * by construction, the threshold because dbx's factory procedure steps an
 * oscillator in equal decibels along its marks and the output because it is
 * a DC potentiometer across a trimmed rail. So their sweeps are measured and
 * their ticks drawn evenly, and the drawing's own scatter is recorded here
 * rather than copied. The compression dial is the opposite case: its taper
 * is real, dbx say so — "scale expansion at the subtle lower ratios" — so
 * its nine measured positions are used as measured.
 */
export const SWEEP_160 = { threshold: 261.6, compression: 305.7, output: 296.0 };

/** The 160A's dials. Its photograph prints labels and no ticks, so these
 * are read from the labels' angles and are correspondingly rough. */
export const SWEEP_160A = { threshold: 270, ratio: 300, output: 270 };

/** The six voltages dbx print round the original's THRESHOLD, and their dBu. */
export const THRESHOLD_MARKS_160 = [
  { at: 0, label: '10mv', dbu: -37.78 },
  { at: 1 / 5, label: '30mv', dbu: -28.25 },
  { at: 2 / 5, label: '100mv', dbu: -17.78 },
  { at: 3 / 5, label: '300mv', dbu: -8.25 },
  { at: 4 / 5, label: '1V', dbu: 2.22 },
  { at: 1, label: '3V', dbu: 11.76 },
];

/** The 160A's threshold scale, seven marks at 10 dB. */
export const THRESHOLD_MARKS_160A = [
  { at: 0, label: '−40dBu', dbu: -40 },
  { at: 1 / 6, label: '−30', dbu: -30 },
  { at: 2 / 6, label: '−20', dbu: -20 },
  { at: 3 / 6, label: '−10', dbu: -10 },
  { at: 4 / 6, label: '0', dbu: 0 },
  { at: 5 / 6, label: '+10', dbu: 10 },
  { at: 1, label: '+20dBu', dbu: 20 },
];

/**
 * The nine ratios printed round the original's COMPRESSION dial, with the
 * coefficient each one is and where it sits on the dial's own travel.
 *
 * `alpha = 1 − 1/R` is the whole ratio law: both the detector and the gain
 * cell are logarithmic with the same 6.1 mV/dB constant, so the pot is
 * simply the fraction of the rectifier's output that reaches the control
 * port. `at` is the measured fraction of this dial's 305.7° sweep.
 */
export const RATIO_MARKS_160 = [
  { at: 0.0, label: '1', alpha: 0 },
  { at: 0.09945, label: '1.5', alpha: 1 / 3 },
  { at: 0.1943, label: '2', alpha: 0.5 },
  { at: 0.36376, label: '3', alpha: 2 / 3 },
  { at: 0.50703, label: '4', alpha: 0.75 },
  { at: 0.69807, label: '6', alpha: 5 / 6 },
  { at: 0.8577, label: '10', alpha: 0.9 },
  { at: 0.93948, label: '20', alpha: 0.95 },
  { at: 1.0, label: '∞', alpha: 1 },
];

/**
 * The 160A's ratio scale. Ten labels, the last four squeezed into a fan at
 * the clockwise end, and their `at` values come from the parameter's own
 * taper rather than from the photograph, which prints no ticks to measure.
 */
export const RATIO_MARKS_160A = [
  { label: '1:1', alpha: 0 },
  { label: '2:1', alpha: 0.5 },
  { label: '3:1', alpha: 2 / 3 },
  { label: '4:1', alpha: 0.75 },
  { label: '6:1', alpha: 5 / 6 },
  { label: '10:1', alpha: 0.9 },
  { label: '∞:1', alpha: 1 },
  { label: '−5:1', alpha: 1.2 },
  { label: '−2:1', alpha: 1.5 },
  { label: '−1:1', alpha: 2 },
];

/** The original's OUTPUT GAIN: nine ticks 5 dB apart, five of them numbered. */
export const OUTPUT_MARKS_160 = [
  { at: 0, label: '−20', db: -20 },
  { at: 1 / 8, label: null, db: -15 },
  { at: 2 / 8, label: '−10', db: -10 },
  { at: 3 / 8, label: null, db: -5 },
  { at: 4 / 8, label: '0', db: 0 },
  { at: 5 / 8, label: null, db: 5 },
  { at: 6 / 8, label: '+10', db: 10 },
  { at: 7 / 8, label: null, db: 15 },
  { at: 1, label: '+20', db: 20 },
];

/** The 160A's OUTPUT GAIN, five marks. */
export const OUTPUT_MARKS_160A = [
  { at: 0, label: '−20dB', db: -20 },
  { at: 0.25, label: '−10', db: -10 },
  { at: 0.5, label: '0', db: 0 },
  { at: 0.75, label: '+10', db: 10 },
  { at: 1, label: '+20dB', db: 20 },
];

/**
 * The 160A's 19-LED level scale, exactly as the panel prints it: the minus
 * signs are implied on the left half, the twelve from −40 to −1 are yellow
 * and the seven from 0 to 20+ are red, and the 12-LED gain-reduction row
 * sits under the −40 to −1 marks and shares that scale.
 */
export const LEVEL_SCALE = [
  '40', '30', '20', '15', '10', '8', '6', '5', '4', '3', '2', '1',
  '0', '1', '2', '4', '6', '10', '20+',
];
/** How many of those are the yellow half, and so how wide the GR row is. */
export const LEVEL_YELLOW = 12;
/** Decibels each level LED stands for, negative on the yellow half. */
export const LEVEL_DB = [
  -40, -30, -20, -15, -10, -8, -6, -5, -4, -3, -2, -1,
  0, 1, 2, 4, 6, 10, 20,
];
/** Decibels of reduction each of the twelve gain-reduction LEDs stands for. */
export const GR_DB = [1, 2, 3, 4, 5, 6, 8, 10, 15, 20, 30, 40];

/**
 * Turn a list of `{ alpha }` or `{ dbu }` or `{ db }` marks into positions on
 * a dial, given the handle and the normalised span that dial covers.
 *
 * The taper lives in the parameter, so this asks the handle where a printed
 * value sits rather than keeping a second copy of the law on this side of
 * the wire. Two copies of one law is the drift an audit of this repository
 * punished across five plug-ins.
 *
 * @param {object} p the parameter handle
 * @param {Array} marks each with a `label` and one of `alpha`, `dbu`, `db`
 * @param {[number, number]} range the normalised span this dial covers
 */
export function placeMarks(p, marks, range) {
  const [lo, hi] = range;
  const span = hi - lo || 1;
  return marks.map((m) => {
    if (m.at !== undefined) return { at: m.at, label: m.label };
    const plain = m.alpha !== undefined ? m.alpha : m.dbu !== undefined ? m.dbu : m.db;
    return { at: Math.min(1, Math.max(0, (p.toNorm(plain) - lo) / span)), label: m.label };
  });
}

/**
 * The normalised span each dial covers on each face.
 *
 * The original's threshold pot ends at its 10 mV and 3 V marks and its
 * compression pot at the ∞ mark; the 160A's cover the whole parameter. The
 * output dial is ±20 dB on both units, so it covers the whole range either
 * way.
 */
export function ranges(c, unit) {
  const whole = [0, 1];
  if (unit !== 0) return { threshold: whole, ratio: whole, output: whole };
  return {
    threshold: [c.threshold.toNorm(-37.78), c.threshold.toNorm(11.76)],
    ratio: [0, c.ratio.toNorm(1)],
    output: whole,
  };
}

// ---------------------------------------------------------------------------
// Legibility
// ---------------------------------------------------------------------------

/**
 * Below this many CSS pixels a numeral shows nothing and looks broken.
 *
 * The rule the lab applies when a printed scale falls under it is to **drop
 * the numeral and keep the tick**: a mark with no number still shows a
 * position, where four-pixel type shows neither. It matters most on the
 * 160A, which is a 1U panel whose real silkscreen is about 1.4 mm tall.
 */
export const LEGIBLE_PX = 8;

/**
 * Whether a scale's numerals are big enough to be worth drawing, kept in
 * step with the element's rendered size.
 *
 * `measure(el)` returns the numerals' size in CSS pixels for the host
 * element. It is a function rather than a computed style because an SVG
 * scale's font size is in the viewBox's own units and only becomes pixels
 * after the box is laid out, so the two callers measure differently: the
 * knob multiplies its font by its box width, and the LED row reads its own
 * computed style.
 *
 * @param {import('vue').Ref<HTMLElement|SVGElement|null>} host
 * @param {(el: Element) => number} measure
 * @returns {import('vue').Ref<boolean>}
 */
export function useNumeralsFit(host, measure) {
  const fits = ref(true);
  let ro = null;
  const check = () => {
    const el = host.value;
    if (el) fits.value = measure(el) >= LEGIBLE_PX;
  };
  onMounted(() => {
    check();
    if (typeof ResizeObserver === 'undefined' || !host.value) return;
    ro = new ResizeObserver(check);
    ro.observe(host.value);
  });
  onBeforeUnmount(() => {
    ro?.disconnect();
    ro = null;
  });
  return fits;
}

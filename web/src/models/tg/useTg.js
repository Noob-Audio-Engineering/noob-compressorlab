/**
 * The EMI TG12413 page's parameter handles and the tables its strip prints.
 *
 * **Read `research/TG12413.md` section 2.9 before changing anything here.**
 * It says, plainly, that no photograph of a bare TG12413 module face exists.
 * The three other faces in this lab are measured off photographs of the
 * object; this one cannot be, and every geometric constant below therefore
 * says where it came from instead of pretending to a measurement.
 *
 * What *is* solid is the control set, because EMI drew all three switches on
 * the circuit diagram with every ladder resistor and every tag number:
 *
 * * **S1 mode**, three positions, COMPRESS / OUT / LIMIT, in that order,
 *   from the legend table printed on the sheet. OUT sits in the middle, so
 *   getting from compress to limit means passing through out.
 * * **S2 recovery**, six positions, marked 1 to 6 and nothing else. No time
 *   is printed on the drawing or published anywhere, and Waves, who had the
 *   console, say the times are "very hard to put in terms of exact
 *   milliseconds". The face does not invent one.
 * * **S3 output level**, twenty-one positions, −10 to +10 dB in the legend's
 *   own words "1 dB STEPS". The engine works from the twenty-one resistor
 *   values behind those markings, which really deliver 0.83 to 1.06 dB per
 *   step, so the silkscreen and the sound disagree by up to a tenth of a
 *   decibel exactly as the hardware's do.
 *
 * And **RV1 HOLD**, a 10 kΩ preset that is an internal screwdriver
 * adjustment rather than a panel control. It is drawn as one, on its own
 * small block beside the strip, because that is what it is.
 *
 * Rules of use: `useControls()` looks parameters up by id, so call it (and
 * anything that uses it) only once `ready` is true.
 */
import { reactive } from 'vue';
import { hasParam, useLab, useParam } from '../../composables/useLab.js';

let controls = null;

/**
 * The handles of the strip and its extras, resolved once.
 * @returns {object}
 */
export function useControls() {
  if (controls) return controls;
  const lab = useLab();
  const opt = (id) => (hasParam(id) ? useParam(id) : null);
  controls = {
    mode: useParam('tg_mode'),
    recovery: useParam('tg_recovery'),
    output: useParam('tg_output'),
    hold: opt('tg_hold'),
    region: opt('tg_region'),
    mismatch: opt('tg_mismatch'),
    input: opt('tg_input'),
    drive: opt('tg_drive'),
    oversample: opt('tg_oversample'),
    link: lab.link,
    mix: lab.mix,
    scHpf: lab.scHpf,
    bypass: lab.bypass,
  };
  return controls;
}

/** Page state that is not a parameter. */
export const ui = reactive({
  scope: true,
});

// ---------------------------------------------------------------------------
// The scales
// ---------------------------------------------------------------------------

/**
 * Sweep in degrees per switch.
 *
 * **Not measured, because there is nothing to measure them off** (2.9).
 * They are the standard rotary-switch index angles, chosen so that each
 * switch's detent count divides out to a whole one:
 *
 * ```text
 *   mode           3 stops x 45 deg =  90
 *   recovery       6 stops x 30 deg = 150
 *   output level  21 stops x 15 deg = 300
 * ```
 *
 * The mode switch takes the wider of the two, because its three positions
 * carry words rather than numerals and 30 degrees apart is not enough room
 * to print COMPRESS, OUT and LIMIT beside one another on a strip this
 * narrow. Switch decks come in 45 degrees as readily as 30.
 *
 * 15° and 30° are the angles switch index plates actually come in, and the
 * sibling Neve panel's five switches all landed on the same two when they
 * *were* measured off a photograph, which is the only corroboration
 * available here.
 */
export const SWEEP = {
  mode: 90,
  recovery: 150,
  output: 300,
};

/** Build a scale: `n` detents, with a label at the indices that carry one. */
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

/** S1: the legend table's three positions, in its order. */
export const MODE_MARKS = sparse(3, [
  [0, 'COMP'], [1, 'OUT'], [2, 'LIMIT'],
]);


/**
 * S2: six positions, numbered, with no times. The absence is the point —
 * see the module note above and section 2.2 of the dossier, which calls the
 * contrast between a calibrated control and one that declines to say "the
 * personality of the panel".
 */
export const RECOVERY_MARKS = sparse(6, [
  [0, '1'], [1, '2'], [2, '3'], [3, '4'], [4, '5'], [5, '6'],
]);

/**
 * S3: twenty-one positions, a number every fifth. Printing all twenty-one
 * would be unreadable at this size and is not what a panel does; EMI's own
 * legend table names positions 1, 11 and 21 and says "1 dB STEPS" for the
 * rest.
 */
export const OUTPUT_MARKS = sparse(21, [
  [0, '−10'], [5, '−5'], [10, '0'], [15, '+5'], [20, '+10'],
]);

/**
 * The gain-reduction meter's scale, in decibels.
 *
 * **This meter is an invention and the page says so.** Drawing TG12413-D101
 * has no meter on it at all. Chandler give their recreation two, scaled 0 to
 * +16 dB of reduction, and Waves give their module one scaled +9 to −20;
 * both are their own additions. This takes Chandler's range, rounded up to
 * the 20 dB the engine's control law reaches, and the strip's note says
 * whose idea it was.
 */
export const METER_MARKS = [0, 4, 8, 12, 16, 20];

/**
 * What the strip says about itself, which is as much a part of this face as
 * the switches are.
 *
 * The dossier's section 11.9 asks the page for an honest provenance note:
 * which controls are EMI's and which are the spoof's, that the operating
 * region is a switch because the drawing is ambiguous, and that no
 * measurement of this unit has ever been published. These are those
 * sentences, kept here so the face and the notes cannot drift apart.
 */
export const NOTES = [
  ['On the module', 'Three switches and one internal preset. Mode, recovery and output level are EMI’s, drawn from the circuit diagram; HOLD is a screwdriver adjustment inside the module, not a panel knob.'],
  ['Ours, not EMI’s', 'Input, drive, arm mismatch, oversampling, the meter, and the lab’s shared mix, link, side-chain filter and bypass. The module has no bypass: OUT leaves the gain element in the signal path and only neutralises the control.'],
  ['Why there is a region switch', 'The drawing is ambiguous about which side of the diodes’ characteristic the gain element works on, and the answer changes the sound. Rather than pick one silently, the model carries the choice.'],
  ['What nobody has measured', 'No factory handbook, no specification and no measurement of any kind has ever been published for this unit. Its calibration figures come from twenty-one resistor values on a photographed blueprint and from two manufacturers’ prose about their own recreations.'],
];

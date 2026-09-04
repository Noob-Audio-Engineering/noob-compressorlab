/**
 * The SSL 4000 G bus compressor page's parameter handles and the tables its
 * panel prints.
 *
 * Everything on the hardware's face is here and live: the threshold and
 * make-up pots, the attack, release, ratio and sidechain-filter switches,
 * and the IN switch. Only the detector-link mode past its first position,
 * the drive, the range, the oversampling, the mix and the shared side-chain
 * high-pass are the lab's own, and those live on the extras strip where we
 * say so.
 *
 * **The IN switch is not a bypass**, and it is the one control on this box
 * whose behaviour SSL state and a plug-in author would guess wrong. It
 * removes the *sidechain*. The audio still passes through the VCA and the
 * make-up gain is still applied, which is why a bypassed unit has excess
 * gain. The plug-in's own bypass is a separate, sample-exact thing and
 * lives on the extras strip with the other lab controls.
 *
 * **The panel is the 500-series module and the values on the switches are
 * the console's.** That is what `research/SSL-Gbus.md` section 2.5
 * instructs, and the reason is that SSL publish a high-resolution render and
 * a dimensioned recall sheet of the module and nothing legible of the
 * console, while card 82E27 gives the console's component values and
 * nothing gives the module's. So the release switch here prints
 * `.1 .3 .6 1.2 AUTO` where a 500-series module prints
 * `.1 .2 .4 .8 1.6 AUTO`, and the ratio prints three positions where the
 * module prints six. Drawing a panel nobody can photograph, or inventing
 * resistor values for the module's ladder, would both be worse.
 *
 * Rules of use: `useControls()` looks parameters up by id, so call it (and
 * anything that uses it) only once `ready` is true.
 */
import { reactive } from 'vue';
import { hasParam, useLab, useParam } from '../../composables/useLab.js';

let controls = null;

/**
 * The parameter handles of the panel, resolved once. Anything the engine
 * may not publish is resolved through `hasParam` and falls back to null, so
 * the face never draws a control that writes nowhere.
 * @returns {object}
 */
export function useControls() {
  if (controls) return controls;
  const lab = useLab();
  const opt = (id) => (hasParam(id) ? useParam(id) : null);
  controls = {
    in: opt('ssl_in'),
    threshold: useParam('ssl_threshold'),
    makeup: useParam('ssl_makeup'),
    attack: useParam('ssl_attack'),
    release: useParam('ssl_release'),
    ratio: useParam('ssl_ratio'),
    hpf: useParam('ssl_hpf'),
    linkMode: opt('ssl_link'),
    drive: opt('ssl_drive'),
    range: opt('ssl_range'),
    oversample: opt('ssl_oversample'),
    link: lab.link,
    mix: lab.mix,
    scHpf: lab.scHpf,
    bypass: lab.bypass,
  };
  return controls;
}

/** Page state that is not a parameter: the analysis drawer. */
export const ui = reactive({
  scope: true,
});

// ---------------------------------------------------------------------------
// Geometry, from research/SSL-Gbus.md section 2.2
// ---------------------------------------------------------------------------

/**
 * The module face measures 1429 x 2528 px in SSL's own render, an aspect of
 * 1 : 1.769. A single 500-series slot is 1.5 x 5.25 inches (1 : 3.5) and a
 * double slot 3.0 x 5.25 (1 : 1.75), so the measured figure matches a double
 * slot within 1 % and this is a two-slot module. Every fraction below is of
 * the panel's own width `W` and height `H`.
 *
 * **This is the only portrait faceplate in the lab**, which is why its view
 * puts the analysis panels beside the panel rather than under it: a
 * 1 : 1.769 plate in a 900 x 520 window would be 240 px wide if it were
 * given the full height, and the six knobs would be unreadable.
 */
export const PANEL_ASPECT = 1 / 1.769;

/** The thin white outline rectangle, x0 y0 x1 y1. */
export const OUTLINE = [0.127, 0.059, 0.873, 0.942];
/** The matt black meter escutcheon plate. */
export const ESCUTCHEON = [0.197, 0.091, 0.81, 0.384];
/** The lit scale window inside it. */
export const GLASS = [0.222, 0.092, 0.78, 0.321];
/** The IN switch cap. */
export const IN_CAP = [0.81, 0.417, 0.936, 0.495];

/** Knob column centres, as fractions of the panel width. */
export const COL = [0.348, 0.661];
/** Knob row centres, as fractions of the panel height. */
export const ROW = [0.509, 0.69, 0.871];

/** The four hex-socket panel screws: centres, and the head diameter in W. */
export const SCREW_X = [0.248, 0.752];
export const SCREW_Y = [0.036, 0.958];
export const SCREW_D = 0.069;

/**
 * The two knob sizes, and this is easy to miss. THRESHOLD and MAKE UP are
 * visibly smaller than the four switches below them, and the skirt-to-cap
 * ratio differs too, 1.23 against 1.43, so they are two different parts and
 * not one part at two sizes. Diameters as fractions of the panel width.
 */
export const POT = { cap: 0.102, skirt: 0.126, box: 0.26 };
export const SWITCH = { cap: 0.118, skirt: 0.168, box: 0.3 };

/**
 * The zero-adjust screw below the meter glass.
 *
 * **A conflict in the dossier, resolved towards the prose.** Its table gives
 * the slot at y 0.230, which falls inside the glass (0.092 to 0.321), while
 * the sentence beside the table says the screw is "in the escutcheon below
 * the glass". A zero-adjust screw behind glass could not be turned, so the
 * table's figure is taken as a slip and the screw is drawn between the
 * bottom of the glass and the bottom of the escutcheon. Recorded here rather
 * than silently corrected.
 */
export const ZERO_SCREW = [0.503, 0.352];

/**
 * The detent-dot sweep. Every switch and both pots carry a ring of small
 * white dots, one per position for the switches and eleven for the pots,
 * arranged over roughly 300 degrees with the gap at six o'clock.
 */
export const SWEEP = 300;

/** Build a ring of `n` dots, with a label at the indices that carry one. */
const dots = (n, pairs) => {
  const labels = {};
  for (const [i, l] of pairs) labels[i] = l;
  return Array.from({ length: n }, (_, i) => ({
    at: n === 1 ? 0.5 : i / (n - 1),
    label: labels[i] ?? null,
  }));
};

/**
 * THRESHOLD: a 50 kOhm linear pot, eleven dots, `0` at twelve o'clock and
 * the ends marked −20 and +20.
 *
 * The E card's own drawing annotates the pot's ends +15 dB and −15 dB, and
 * the Smart Research descendant specifies ±15 dBm, so the ±20 printed on
 * every modern unit is not the only figure SSL have used. The engine's range
 * is ±20 because that is what the panel being drawn prints.
 */
export const THRESHOLD_MARKS = dots(11, [[0, '−20'], [5, '0'], [10, '+20']]);

/**
 * MAKE UP: a 25 kOhm linear pot. The module prints only `0`, `−` and `+`.
 *
 * **The `0` does not sit at the end of the travel here, and on the hardware
 * it does.** The console's pot runs 0 to +20 dB with its zero fully
 * anticlockwise; this model takes SSL's own plug-in range of −5 to +15 dB
 * instead, so that the control can also take level away, and the zero
 * therefore falls a quarter of the way round. Both figures are published;
 * the difference is stated rather than hidden, and the mark is drawn where
 * the parameter's zero actually is.
 */
export const MAKEUP_MARKS = dots(11, [[0, '−'], [2, null], [10, '+']]);
/** Where 0 dB falls on the make-up pot's travel, for the extra `0` mark. */
export const MAKEUP_ZERO_AT = 0.25;

/** ATTACK: six positions, card 82E27's ladder, printed in milliseconds. */
export const ATTACK_MARKS = dots(6, [
  [0, '.1'], [1, '.3'], [2, '1'], [3, '3'], [4, '10'], [5, '30'],
]);

/** RELEASE: the console's five positions, printed in seconds. */
export const RELEASE_MARKS = dots(5, [
  [0, '.1'], [1, '.3'], [2, '.6'], [3, '1.2'], [4, 'AUTO'],
]);

/**
 * RATIO: the console's three positions.
 *
 * SSL's own plug-in guide prints "2:1, 4:1 and 20:1" where every SSL
 * hardware panel prints 10:1. A unit whose ratio rises with gain reduction
 * has no single ratio, so both numbers can be true of one curve read at two
 * operating points, and the dossier declines to correct either. The metal's
 * figure is what is printed here, because that is what the metal says.
 */
export const RATIO_MARKS = dots(3, [[0, '2'], [1, '4'], [2, '10']]);

/**
 * HPF: six positions, on the module only.
 *
 * SSL's product page says 106 Hz where SSL's own module panel and recall
 * sheet both print 105. The panel's figure is used, here and in the engine.
 */
export const HPF_MARKS = dots(6, [
  [0, 'OFF'], [1, '30'], [2, '60'], [3, '105'], [4, '125'], [5, '185'],
]);

/**
 * The compression meter's printed scale: 0 to 20 dB, left to right, on a
 * linear scale, with numbered ticks at 0, 4, 8, 12, 16 and 20 and
 * unnumbered ticks between them.
 *
 * The drive is documented by the clone builder, who read it off the same
 * circuit: the movement is fed from the buffered control voltage through a
 * series resistor, "linear scale, at about 50 µA/dB, making a 1 mA meter
 * showing 20 dB full-scale". So the meter reads the control voltage
 * linearly, not a decibel conversion of a measured gain, and this is the
 * rare case where the naive meter and the circuit meter agree, because a
 * Blackmer VCA's control voltage is linear in decibels.
 */
export const METER_MAX_DB = 20;
export const METER_LABELS = [0, 4, 8, 12, 16, 20];

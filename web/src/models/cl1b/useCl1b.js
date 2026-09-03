/**
 * The CL-1B page's parameter handles and its page-only state.
 *
 * Everything the hardware has is on the face: the three knobs of the top
 * row, the two of the bottom, both three-position lever switches, the IN
 * bypass toggle and the mains switch. Only Mix and the side-chain
 * high-pass are the lab's own, and those live on the extras strip where we
 * say so.
 *
 * Two of these need a word, because they are not what their names suggest.
 * `bypass` is the panel's IN switch: the hardware calls it In/Out and its
 * out position takes the compressor out of the signal path, so it is the
 * lab's shared bypass wearing the hardware's legend rather than a
 * `cl1b_` parameter of its own. And `bus` is the sidechain BUS SELECT,
 * whose off/1/2 positions choose a stereo link group on the hardware; in
 * the model it does the same job through the shared link, which is what
 * Softube did with it too.
 *
 * There is no cell-age control here, unlike the LA-2A and the LA-3A.
 * Lydkraft claim the element has no long-term degradation and owners report
 * their units are all alike, so inventing one would be inventing a fact.
 *
 * Rules of use: `useControls()` looks parameters up by id, so call it only
 * once `ready` is true.
 */
import { computed, reactive } from 'vue';
import { hasParam, useLab, useParam } from '../../composables/useLab.js';

/**
 * Page state that is not a parameter: whether the analysis drawer is open,
 * and the fallback position of the mains switch (see `power` below).
 */
export const ui = reactive({
  scope: true,
  powerOn: true,
});

/**
 * The mains switch's handle.
 *
 * The panel has a mains switch and the model may or may not have a
 * parameter for it, so this resolves to whichever exists. If the engine
 * publishes `cl1b_power` the switch drives that and the unit really does
 * power down; if it does not, the switch falls back to page state and only
 * darkens the panel, which is said plainly in `Cl1bPower.vue` and in
 * `web/README.md`. Both wear the same shape, so the face binds to one thing
 * either way and needs no change when the parameter lands.
 *
 * Note what powering down does here: it passes audio through rather than
 * silencing it, which is a deliberate divergence from a real CL-1B, whose
 * audio path runs through its tube stages and passes nothing with the mains
 * off. The 1176 in this same plug-in already chose pass-through for its own
 * power-down and the two are kept consistent. So nothing on this face may
 * imply the signal stops.
 */
function makePower() {
  if (hasParam('cl1b_power')) {
    const p = useParam('cl1b_power');
    return { real: true, on: computed(() => p.on), set: (v) => { p.begin(); p.setOn(v); p.end(); } };
  }
  return { real: false, on: computed(() => ui.powerOn), set: (v) => { ui.powerOn = v; } };
}

let controls = null;

/**
 * @returns {{ gain, ratio, threshold, attack, release, mode, meter, bus, link, mix, scHpf, bypass, power }}
 */
export function useControls() {
  if (controls) return controls;
  const lab = useLab();
  controls = {
    gain: useParam('cl1b_gain'),
    ratio: useParam('cl1b_ratio'),
    threshold: useParam('cl1b_threshold'),
    attack: useParam('cl1b_attack'),
    release: useParam('cl1b_release'),
    mode: useParam('cl1b_mode'),
    meter: useParam('cl1b_meter'),
    bus: useParam('cl1b_bus'),
    link: lab.link,
    mix: lab.mix,
    scHpf: lab.scHpf,
    bypass: lab.bypass,
    power: makePower(),
  };
  return controls;
}

/**
 * The scale dots as they sit on the metal, measured off Lydkraft's own
 * photograph in `research/CL-1B.md` section 2.2 and given as fractions of
 * the knob's travel. Gain and Threshold turn out to share one piece of
 * artwork, their dots agreeing to within 2.5 degrees at all six positions,
 * so the two lists differ only in what is printed beside them.
 *
 * The spacing is irregular and not monotone, which no smooth pot law
 * produces. That is the silkscreen being approximate rather than a
 * measurement error, and Softube say the same of the hardware they had in
 * front of them. So the dots go where they are on the metal and the
 * parameter follows its own smooth law underneath.
 */
export const GAIN_MARKS = [
  { at: 0.0, label: 'off' },
  { at: 0.144, label: '−10' },
  { at: 0.265, label: '0' },
  { at: 0.505, label: '+10' },
  { at: 0.669, label: '+20' },
  { at: 0.999, label: '+30' },
];

export const THRESHOLD_MARKS = [
  { at: 0.0, label: 'off' },
  { at: 0.146, label: '0' },
  { at: 0.272, label: '−10' },
  { at: 0.519, label: '−20' },
  { at: 0.686, label: '−30' },
  { at: 1.0, label: '−40' },
];

/** Ratio, Attack and Release carry two legends and nothing between them, which is honest of them. */
export const RATIO_MARKS = [
  { at: 0.0, label: '2:1' },
  { at: 1.0, label: '10:1' },
];

export const TIME_MARKS = [
  { at: 0.0, label: 'fast' },
  { at: 1.0, label: 'slow' },
];

/** Every knob sweeps 239 degrees, from −119 to +120, measured from the scale dots. */
export const SWEEP = 239;

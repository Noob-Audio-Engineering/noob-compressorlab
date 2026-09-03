/**
 * The LA-3A page's parameter handles and its page-only state.
 *
 * `meter` has the two positions the hardware's toggle has, GR and OUTPUT
 * (where 0 VU is +4 dBm), plus the Off that only the plug-ins have; the
 * face draws the toggle, the extras strip offers all three. `emphasis` is
 * the rear HF Contour pot and runs 0 = flat to 1 = full contour, which is
 * the opposite sense to the LA-2A's emphasis control.
 *
 * Rules of use: `useControls()` looks parameters up by id, so call it only
 * once `ready` is true.
 */
import { reactive } from 'vue';
import { useLab, useParam } from '../../composables/useLab.js';

/** Page state that is not a parameter: whether the analysis drawer is open. */
export const ui = reactive({
  scope: true,
});

let controls = null;

/**
 * @returns {{ gain, peakReduction, mode, meter, emphasis, cell, link, mix, scHpf, bypass }}
 */
export function useControls() {
  if (controls) return controls;
  const lab = useLab();
  controls = {
    gain: useParam('la3a_gain'),
    peakReduction: useParam('la3a_peak_reduction'),
    mode: useParam('la3a_mode'),
    meter: useParam('la3a_meter'),
    emphasis: useParam('la3a_emphasis'),
    cell: useParam('la3a_cell'),
    link: lab.link,
    mix: lab.mix,
    scHpf: lab.scHpf,
    bypass: lab.bypass,
  };
  return controls;
}

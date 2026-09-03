/**
 * The LA-2A page's specifics: its parameter handles grouped once (ids
 * prefixed `opto_`, the shared extras from `useLab`).
 *
 * Everything here needs the manifest; call `useOpto()` only once
 * `useNoobVstWebguiFramework().ready` is true.
 */
import { reactive } from 'vue';
import { hasParam, useLab, useParam } from '../../composables/useLab.js';

let panel = null;

/**
 * The panel's handles, resolved once and shared.
 * @returns {{ gain, peakReduction, mode, meter, emphasis, cell, link, mix, scHpf, bypass, source: null | { kind, level, freq } }}
 */
export function useOpto() {
  if (panel) return panel;
  const lab = useLab();
  panel = {
    gain: useParam('opto_gain'),
    peakReduction: useParam('opto_peak_reduction'),
    mode: useParam('opto_mode'),
    meter: useParam('opto_meter'),
    emphasis: useParam('opto_emphasis'),
    // A real front-panel trim, so the face draws it live as soon as the
    // engine publishes it; until then the panel keeps its plain screw.
    meterZero: hasParam('opto_meter_zero') ? useParam('opto_meter_zero') : null,
    cell: useParam('opto_cell'),
    link: lab.link,
    mix: lab.mix,
    scHpf: lab.scHpf,
    bypass: lab.bypass,
    source: lab.source,
  };
  return panel;
}

/**
 * Page state that is not a parameter: whether the analysis drawer is open.
 * The other five models already had this; the LA-2A gained it when the
 * bar's global half was standardised, because SCOPE is one of the controls
 * every model now carries.
 */
export const ui = reactive({
  scope: true,
});

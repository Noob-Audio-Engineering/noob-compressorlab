/**
 * The LA-2A page's specifics: its parameter handles grouped once (ids
 * prefixed `opto_`, the shared extras from `useLab`).
 *
 * Everything here needs the manifest; call `useOpto()` only once
 * `useNoobVstWebguiFramework().ready` is true.
 */
import { useLab, useParam } from '../../composables/useLab.js';

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
    cell: useParam('opto_cell'),
    link: lab.link,
    mix: lab.mix,
    scHpf: lab.scHpf,
    bypass: lab.bypass,
    source: lab.source,
  };
  return panel;
}

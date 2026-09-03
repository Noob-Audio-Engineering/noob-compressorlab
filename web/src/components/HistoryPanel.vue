<script setup>
/**
 * "Last 8 seconds", the history window both models share: the framework's
 * `Timeline` fed from the `meter` stream. Input and output peaks in dBFS
 * share one scale; the gain reduction (`gr_db`, at most 0) hangs from the
 * top on its own −24..0 dB scale (`range: [-24, 0]`, 0 at the top) and
 * draws the grid every 6 dB. The title carries the live reduction.
 *
 * The reduction series marks its own peaks and names them in callout boxes,
 * so the chart says how hard the compressor grabbed and when without the
 * reader tracing the scale. They sit faint until the pointer is over the
 * chart, which brings them to full strength. Only dips past 3 dB are worth
 * marking, two stay at least 400 ms apart so a busy passage does not become
 * a thicket, and the four deepest in the window are shown. A vertical line
 * every second gives them something to be read against.
 *
 * The panel is identical whichever model is active: the same chrome
 * (`.lab-panel` in `style.css`), typography, grid and series colours (the
 * LA-2A's workbench look, now the lab's: dim input, blue output, amber gain
 * reduction). Nothing here comes from the model. Props: none. Emits:
 * nothing.
 */
import { Timeline, useStreamValue } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';

const gr = useStreamValue('meter', { index: 4, unit: 'db' });
const series = [
  { stream: 'meter', index: 0, unit: 'linear', range: [-60, 6], color: 'rgba(231, 226, 216, 0.45)', width: 1, label: 'in' },
  { stream: 'meter', index: 2, unit: 'linear', range: [-60, 6], color: '#7cc6ff', width: 1.2, label: 'out' },
  {
    stream: 'meter',
    index: 4,
    unit: 'db',
    range: [-24, 0],
    color: '#e9a23b',
    width: 1.5,
    fill: true,
    fillTo: 0,
    label: 'gain reduction',
    // The reduction falls, so its peaks are minima. 1.5 dB of hysteresis
    // rides out the ripple an opto cell leaves on a sustained note without
    // missing a real second dip.
    peaks: { direction: 'min', threshold: -3, hysteresis: 1.5, minGapMs: 400, max: 4, format: (v) => `${v.toFixed(1)} dB` },
  },
];
</script>

<template>
  <div class="lab-panel lab-panel--history">
    <div class="lab-panel__title">
      <span>Last 8 seconds</span>
      <span class="lab-panel__val">GR {{ gr.toFixed(1) }} dB</span>
    </div>
    <div class="lab-panel__canvas"><Timeline :series="series" :seconds="8" :grid-series="2" :grid-step="6" time-grid /></div>
  </div>
</template>

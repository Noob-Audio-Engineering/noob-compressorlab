<script setup>
/**
 * "Last 8 seconds", the history window both models share: the framework's
 * `Timeline` fed from the `meter` stream. Input and output peaks in dBFS
 * share one scale; the gain reduction (`gr_db`, at most 0) hangs from the
 * top on its own −24..0 dB scale (`range: [-24, 0]`, 0 at the top) and
 * draws the grid every 6 dB. The title carries the live reduction.
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
  { stream: 'meter', index: 4, unit: 'db', range: [-24, 0], color: '#e9a23b', width: 1.5, fill: true, fillTo: 0, label: 'gain reduction' },
];
</script>

<template>
  <div class="lab-panel">
    <div class="lab-panel__title">
      <span>Last 8 seconds</span>
      <span class="lab-panel__val">GR {{ gr.toFixed(1) }} dB</span>
    </div>
    <div class="lab-panel__canvas"><Timeline :series="series" :seconds="8" :grid-series="2" :grid-step="6" /></div>
  </div>
</template>

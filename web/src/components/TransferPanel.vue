<script setup>
/**
 * The transfer panel both models share (the LA-2A's, kept as it was): the
 * static transfer curve (output level against input level for a 1 kHz
 * sine at the current settings) from the sticky `transfer` stream, drawn
 * with the framework's `LinePlot` with the dashed unity line for
 * reference, plus the live operating point from the `meter` stream's input
 * and output peaks (converted to sine RMS). The title carries the live
 * levels.
 *
 * The panel is identical whichever model is active: the same chrome
 * (`.lab-panel` in `style.css`), typography, ranges and colours (amber
 * curve, dim dashed unity). Nothing here comes from the model. Props:
 * none. Emits: nothing.
 */
import { computed } from 'vue';
import { LinePlot, useStreamValue } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';

const inPeak = useStreamValue('meter', { index: 0, unit: 'linear', initial: -200 });
const outPeak = useStreamValue('meter', { index: 2, unit: 'linear', initial: -200 });
const marker = computed(() => (inPeak.value > -90 ? [inPeak.value - 3.01, outPeak.value - 3.01] : null));
const series = [
  { stream: 'transfer', color: '#e9a23b', width: 2, label: 'transfer' },
  { xy: [[-60, -60], [0, 0]], color: 'rgba(231, 226, 216, 0.18)', dash: [4, 4], label: 'unity' },
];
const fmt = (v) => (v > -90 ? v.toFixed(1) : '–');
</script>

<template>
  <div class="lab-panel lab-panel--transfer">
    <div class="lab-panel__title">
      <span>Transfer</span>
      <span class="lab-panel__val">in {{ fmt(inPeak) }} · out {{ fmt(outPeak) }} dBFS</span>
    </div>
    <div class="lab-panel__canvas">
      <LinePlot :series="series" :x-range="[-60, 0]" :y-range="[-60, 12]" :x-step="12" :y-step="24" x-label="in dBFS" y-label="out dBFS" :marker="marker" />
    </div>
  </div>
</template>

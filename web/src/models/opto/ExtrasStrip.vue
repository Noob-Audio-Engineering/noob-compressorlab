<script setup>
/**
 * The modern extras that the original never had, kept off the faceplate:
 * R37 emphasis, the cell speed, stereo link, mix and the side-chain
 * high-pass, plus the standalone's demo source when it is present. Small
 * framework knobs and the unstyled `Segmented` / `Toggle` controls, styled
 * by `style.css` under `.bench`.
 */
import { Knob, Segmented, Toggle } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';
import { useOpto } from './useOpto.js';

const panel = useOpto();
const knob = { size: 42, color: '#e9a23b' };
</script>

<template>
  <div class="bench flex items-center gap-6 px-4 py-2">
    <div class="flex flex-col items-center gap-1">
      <Knob :p="panel.emphasis" v-bind="knob" label="Emphasis" />
    </div>
    <div class="flex flex-col items-center gap-1">
      <div class="bench-label">Cell</div>
      <Segmented :p="panel.cell" />
    </div>
    <div class="flex flex-col items-center gap-1">
      <div class="bench-label">Link</div>
      <Toggle :p="panel.link" :labels="['', 'stereo']" />
    </div>
    <div class="flex flex-col items-center gap-1">
      <Knob :p="panel.mix" v-bind="knob" label="Mix" />
    </div>
    <div class="flex flex-col items-center gap-1">
      <Knob :p="panel.scHpf" v-bind="knob" label="SC HPF" />
    </div>
    <div v-if="panel.source" class="ml-auto flex items-center gap-4 pl-4 border-l border-white/10">
      <div class="flex flex-col items-center gap-1">
        <div class="bench-label">Demo source</div>
        <Segmented :p="panel.source.kind" />
      </div>
      <Knob :p="panel.source.level" :size="36" color="#7cc6ff" label="Level" />
      <Knob :p="panel.source.freq" :size="36" color="#7cc6ff" label="Pitch" />
    </div>
  </div>
</template>

<script setup>
/**
 * The modern extras that the original never had, kept off the faceplate: the
 * cell speed, stereo link, mix and the side-chain high-pass, plus the
 * standalone's demo source when it is present. Small framework knobs and the
 * unstyled `Segmented` / `Toggle` controls, styled by `style.css` under
 * `.bench`.
 *
 * R37 is not here: it is a real screwdriver trim on the front panel, so it
 * lives on the faceplate where the hardware puts it.
 */
import { Knob, Segmented, Toggle } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';
import { ui, useOpto } from './useOpto.js';

const panel = useOpto();
const knob = { size: 42, color: '#e9a23b' };
</script>

<template>
  <div class="bench lab-bar">
    <div class="lab-bar__left">
      <div class="flex flex-col items-center gap-1">
        <div class="bench-label">Cell</div>
        <Segmented :p="panel.cell" />
      </div>
    </div>
    <div class="lab-bar__globals">
      <span class="lab-bar__tag" title="Ours, not the hardware's: every model carries these.">LAB</span>
      <div class="flex flex-col items-center gap-1">
        <Knob :p="panel.mix" v-bind="knob" label="Mix" />
      </div>
      <div class="flex flex-col items-center gap-1">
        <Knob :p="panel.scHpf" v-bind="knob" label="SC HPF" />
      </div>
      <div class="flex flex-col items-center gap-1">
        <div class="bench-label">Stereo</div>
        <Toggle :p="panel.link" :labels="['', 'stereo']" />
      </div>
      <template v-if="panel.source">
        <div class="flex flex-col items-center gap-1">
          <div class="bench-label">Demo source</div>
          <Segmented :p="panel.source.kind" />
        </div>
        <Knob :p="panel.source.level" :size="36" color="#7cc6ff" label="Level" />
        <Knob :p="panel.source.freq" :size="36" color="#7cc6ff" label="Pitch" />
      </template>
      <button class="bench-scope" :class="{ on: ui.scope }" title="Show or hide the analysis drawer" @click="ui.scope = !ui.scope">SCOPE</button>
    </div>
  </div>
</template>

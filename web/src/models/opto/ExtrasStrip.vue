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
import BarGlobals from '../../components/BarGlobals.vue';

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
    <BarGlobals v-model:scope="ui.scope" />
  
  </div>
</template>

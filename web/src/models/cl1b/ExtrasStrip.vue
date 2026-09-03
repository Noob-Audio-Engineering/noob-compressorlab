<script setup>
/**
 * The CL-1B's strip: only the lab's own additions, because unlike the LA-3A
 * this unit keeps nothing on its rear panel that a user would reach for.
 * Everything the hardware has is on the face.
 *
 * So Mix and the side-chain high-pass are here, and they are labelled as
 * ours. Neither is on a CL-1B; Softube, Universal Audio and Stam Audio all
 * added the same two, and the dossier notes the hardware's detector is flat,
 * which is why the high-pass defaults to off. Stereo link is here too: on
 * the hardware it is the sidechain bus, which the face already draws, so
 * this is the same idea in the lab's own terms.
 *
 * Reads / writes: `link`, `mix`, `sc_hpf`, `src_*`. Emits: nothing.
 */
import { Knob, Segmented, Toggle } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';
import { ui, useControls } from './useCl1b.js';
import BarGlobals from '../../components/BarGlobals.vue';

const c = useControls();
const knob = { size: 42, color: '#e8b04a' };
</script>

<template>
  <div class="extrascl1b lab-bar">
    <!-- nothing of this model's own belongs here: every control the CL-1B has is on its face -->
    <div class="lab-bar__left"></div>
    <BarGlobals v-model:scope="ui.scope" />
  
  </div>
</template>

<script setup>
/**
 * The discreet strip under the panel with the modern additions: the
 * revision selector (A to H and the reissue, with a hint of what changes),
 * stereo link, mix, side-chain high-pass, the scope drawer toggle, and, in
 * the standalone only, the demo source.
 *
 * Reads / writes: `fet_revision`, `link`, `mix`, `sc_hpf`, `src_*`. Emits: nothing.
 */
import { Segmented, Toggle } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';
import { REVISIONS, ui, useControls } from './useFet.js';
import BarGlobals from '../../components/BarGlobals.vue';
import Knob1176 from './Knob1176.vue';

const c = useControls();
const REVISION_LABELS = REVISIONS.map((r) => r.label);
const fmtHpf = (v) => (v < 5 ? 'OFF' : `${Math.round(v)} Hz`);
const fmtPct = (v) => `${Math.round(v)} %`;
const fmtLevel = (v) => `${Math.round(v * 100)} %`;
const fmtFreq = (v) => (v >= 1000 ? `${(v / 1000).toFixed(2)} kHz` : `${Math.round(v)} Hz`);
</script>

<template>
  <div class="extras1176 lab-bar">
    <div class="lab-bar__left">
      <div class="extras1176__item revision">
        <span class="extras1176__caption">REVISION</span>
        <Segmented :p="c.revision" :labels="REVISION_LABELS" />
        <span class="extras1176__hint">{{ (REVISIONS[c.revision.index] || REVISIONS[8]).hint }}</span>
      </div>
    </div>
    <BarGlobals v-model:scope="ui.scope" />
  
  </div>
</template>

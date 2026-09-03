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
    <div class="lab-bar__globals">
      <span class="lab-bar__tag" title="Ours, not the hardware's: every model carries these.">LAB</span>
    <div class="extras1176__item">
      <Knob1176 :p="c.mix" :size="46" label="MIX" :format="fmtPct" />
    </div>
    <div class="extras1176__item">
      <Knob1176 :p="c.scHpf" :size="46" label="SC HPF" :format="fmtHpf" />
    </div>
    <div class="extras1176__item">
      <span class="extras1176__caption">STEREO</span>
      <Toggle :p="c.link" :labels="['DUAL', 'LINK']" variant="rocker" />
    </div>
    <div v-if="c.source" class="extras1176__item source">
      <span class="extras1176__caption">DEMO SOURCE</span>
      <Segmented :p="c.source.kind" :labels="['VOCAL', 'BASS', 'DRUMS', 'PINK', 'WHITE', 'SAW', 'SINE']" />
      <Knob1176 :p="c.source.level" :size="42" label="LEVEL" :format="fmtLevel" :to-rotation="(v) => v" :from-rotation="(r) => r" />
      <Knob1176 :p="c.source.freq" :size="42" label="PITCH" :format="fmtFreq" :to-rotation="(v) => Math.log(v / 20) / Math.log(1000)" :from-rotation="(r) => 20 * Math.pow(1000, r)" />
    </div>
    <button class="extras1176__scope" :class="{ on: ui.scope }" title="Show or hide the analysis drawer" @click="ui.scope = !ui.scope">SCOPE</button>
    </div>
  </div>
</template>

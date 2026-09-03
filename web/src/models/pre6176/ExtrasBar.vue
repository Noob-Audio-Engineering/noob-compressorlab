<script setup>
/**
 * The strip under the 6176's panel with what the hardware keeps on its rear
 * or does not have at all: the compressor half's revision, the output
 * loading switch, stereo link, mix, the side-chain high-pass, the drawer
 * toggle and, in the standalone only, the demo source.
 *
 * The voicing selector is here rather than on the faceplate because the
 * hardware has no such switch: a 610A is a different box, not a setting, so
 * choosing between the two is ours and belongs where we say so.
 *
 * Reads / writes: `pre_voice`, `pre_load`, `pre_hpf`, `fet_revision`,
 * `link`, `mix`, `sc_hpf`, `src_*`. Emits: nothing.
 */
import { Segmented, Toggle } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';
import { REVISIONS } from '../fet/useFet.js';
import { ui, useControls } from './usePre.js';
import BarGlobals from '../../components/BarGlobals.vue';
import Knob1176 from '../fet/Knob1176.vue';

const c = useControls();
const REVISION_LABELS = REVISIONS.map((r) => r.label);
const fmtHpf = (v) => (v < 5 ? 'OFF' : `${Math.round(v)} Hz`);
const fmtPct = (v) => `${Math.round(v)} %`;
const fmtLevel = (v) => `${Math.round(v * 100)} %`;
const fmtFreq = (v) => (v >= 1000 ? `${(v / 1000).toFixed(2)} kHz` : `${Math.round(v)} Hz`);
</script>

<template>
  <div class="extras6176 lab-bar">
    <div class="lab-bar__left">
    <div class="extras6176__item">
      <span class="extras6176__caption">LIMITER REVISION</span>
      <Segmented :p="c.revision" :labels="REVISION_LABELS" />
    </div>
    <div class="extras6176__item">
      <span class="extras6176__caption">VOICING</span>
      <Segmented :p="c.voice" :labels="['610B', '610A']" />
      <span class="extras6176__hint">the 1958 module is browner and more asymmetric</span>
    </div>
    <div class="extras6176__item">
      <span class="extras6176__caption">LOW CUT</span>
      <Toggle :p="c.hpf" :labels="['OUT', 'IN']" variant="rocker" />
    </div>
    <div class="extras6176__item">
      <span class="extras6176__caption">INPUT LOADING</span>
      <Segmented :p="c.load" :labels="['15K', '600']" />
      <span class="extras6176__hint">the rear switch: 600 Ω is the older, duller load</span>
    </div>
    </div>
    <BarGlobals v-model:scope="ui.scope" />
  
  </div>
</template>

<script setup>
/**
 * The strip under the Distressor's panel with what the hardware does not
 * put on its face: the link mode the EL8-X toggle selects between (phase
 * summing, gain summing, or both), the operating headroom (the plug-in
 * world's version of the internal reference level), mix, the side-chain
 * high-pass, the drawer toggle and, in the standalone only, the demo
 * source.
 *
 * The finish selector is here too: which of the two liveries the panel
 * wears is a page setting rather than a parameter, since it changes nothing
 * about the audio, and it is kept in the UI store so a project remembers it.
 *
 * Reads / writes: `dist_link_mode`, `dist_headroom`, `mix`, `sc_hpf`,
 * `src_*`, and the stored `finish`. Emits: nothing.
 */
import { Segmented, Toggle } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';
import { FINISHES, ui, useControls, useFinish } from './useVca.js';
import KnobEL8 from './KnobEL8.vue';

const c = useControls();
const finish = useFinish();
const fmtHpf = (v) => (v < 5 ? 'OFF' : `${Math.round(v)} Hz`);
</script>

<template>
  <div class="extrasel8 lab-bar">
    <div class="lab-bar__left">
    <div class="extrasel8__item">
      <span class="extrasel8__caption">FINISH</span>
      <div class="extrasel8__finish">
        <button v-for="f in FINISHES" :key="f.key" type="button" :class="{ on: finish === f.key }" :aria-pressed="finish === f.key" :title="f.hint" @click="finish = f.key">{{ f.label }}</button>
      </div>
    </div>
    <div class="extrasel8__item">
      <span class="extrasel8__caption">LINK MODE</span>
      <Segmented :p="c.linkMode" :labels="['PHASE', 'IMAGE', 'BOTH']" />
      <span class="extrasel8__hint">how a linked pair shares its side-chain</span>
    </div>
    <div class="extrasel8__item">
      <KnobEL8 :p="c.headroom" size="46px" label="Headroom" :sweep="270" />
      <span class="extrasel8__value">{{ Math.round(c.headroom.plain) }} dB</span>
      <span class="extrasel8__caption">HEADROOM</span>
    </div>
    </div>
    <div class="lab-bar__globals">
      <span class="lab-bar__tag" title="Ours, not the hardware's: every model carries these.">LAB</span>
    <div class="extrasel8__item">
      <KnobEL8 :p="c.mix" size="46px" label="Mix" :sweep="270" />
      <span class="extrasel8__value">{{ Math.round(c.mix.plain) }} %</span>
      <span class="extrasel8__caption">MIX</span>
    </div>
    <div class="extrasel8__item">
      <KnobEL8 :p="c.scHpf" size="46px" label="Side-chain high-pass" :sweep="270" />
      <span class="extrasel8__value">{{ fmtHpf(c.scHpf.plain) }}</span>
      <span class="extrasel8__caption">SC HPF</span>
    </div>
    <div class="extrasel8__item">
      <span class="extrasel8__caption">STEREO</span>
      <Toggle :p="c.link" :labels="['', 'stereo']" />
    </div>
    <div v-if="c.source" class="extrasel8__item source">
      <span class="extrasel8__caption">DEMO SOURCE</span>
      <Segmented :p="c.source.kind" :labels="['VOCAL', 'BASS', 'DRUMS', 'PINK', 'WHITE', 'SAW', 'SINE']" />
    </div>
    <button class="extrasel8__scope" :class="{ on: ui.scope }" title="Show or hide the analysis drawer" @click="ui.scope = !ui.scope">SCOPE</button>
    </div>
  </div>
</template>

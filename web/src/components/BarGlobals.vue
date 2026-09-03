<script setup>
/**
 * The right-hand half of the bench bar: the controls that are ours rather
 * than any compressor's. Every model renders this one component, so the
 * group is identical by construction and not by six files agreeing.
 *
 * That is the point of it. These controls used to be assembled separately
 * in each model's bar, and they had drifted into three different knob
 * components — the framework `Knob` on the optical models, `Knob1176` on
 * the 1176 and the 6176, `KnobEL8` on the Distressor — plus two different
 * stereo controls. `Knob1176` draws no value arc at all, so pinning an
 * accent colour could not reach it: on those models there was no colour
 * around the knob because there was nothing to colour. The fix is one
 * component, not one variable.
 *
 * The model-specific knobs stay where they belong, on the faceplates, where
 * they are drawing that unit's real hardware. Anything in here is the lab's
 * own, so it looks the lab's own way: the framework controls, in the accent
 * pinned on `.lab-bar__globals`.
 *
 * What is deliberately **not** here is the demo source. Everything in this
 * bar changes real audio; the demo source only decides what audio there is
 * to change, and only where the page is generating its own. It lives in the
 * development panel now, with the rest of the standalone's diagnostics.
 *
 * Props: `scope` (whether the analysis drawer is open).
 * Emits: `update:scope`.
 */
import { Knob, Toggle } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';
import { useDebug, useLab } from '../composables/useLab.js';

defineProps({
  scope: { type: Boolean, default: true },
});
const emit = defineEmits(['update:scope']);

const lab = useLab();
const debug = useDebug();
const KNOB = { size: 42 };
</script>

<template>
  <div class="lab-bar__globals">
    <span class="lab-bar__tag" title="Ours, not the hardware's: every model carries these.">LAB</span>
    <div class="lab-bar__item"><Knob :p="lab.mix" v-bind="KNOB" label="Mix" /></div>
    <div class="lab-bar__item"><Knob :p="lab.scHpf" v-bind="KNOB" label="SC HPF" /></div>
    <div class="lab-bar__item">
      <span class="lab-bar__caption">STEREO</span>
      <Toggle :p="lab.link" :labels="['', 'stereo']" />
    </div>
    <button
      class="lab-bar__scope"
      :class="{ on: debug.shown.value }"
      type="button"
      title="Show or hide the development panel below the charts"
      @click="debug.shown.value = !debug.shown.value"
    >
      DEBUG
    </button>
    <button class="lab-bar__scope" :class="{ on: scope }" type="button" title="Show or hide the analysis drawer" @click="emit('update:scope', !scope)">
      SCOPE
    </button>
  </div>
</template>

<script setup>
/**
 * The LA-3A's rear panel and the plug-in's own additions, kept off the
 * face because that is where the hardware keeps them: the Comp / Limit
 * slide switch, the HF Contour pot (0 is flat; wound up, the side-chain
 * hears 15 kHz about 10 dB hotter than the rest, which is what makes this
 * unit ride sibilance), and the meter's third position, Off, which only the
 * plug-ins have. Then the lab's shared extras and, in the standalone, the
 * demo source.
 *
 * Note the contour's sense is the opposite of the LA-2A's emphasis control:
 * different circuits, so they do not share a component.
 *
 * The cell's age is here rather than on the panel for the same reason the
 * LA-2A's is: a T4 that has been in the rack twenty years is not a switch the
 * hardware gives you. A tired cell compresses about 2.7 dB less at a normal
 * operating point.
 *
 * Reads / writes: `la3a_mode`, `la3a_emphasis`, `la3a_cell`, `la3a_meter`,
 * `link`, `mix`, `sc_hpf`, `src_*`. Emits: nothing.
 */
import { Knob, Segmented, Toggle } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';
import { ui, useControls } from './useLa3a.js';
import BarGlobals from '../../components/BarGlobals.vue';

const c = useControls();
const knob = { size: 42, color: '#dda43f' };
</script>

<template>
  <div class="extrasla3a lab-bar">
    <div class="lab-bar__left">
      <span class="extrasla3a__rear">REAR PANEL</span>
    <div class="extrasla3a__item">
      <span class="extrasla3a__caption">MODE</span>
      <Segmented :p="c.mode" :labels="['COMPRESS', 'LIMIT']" />
      <span class="extrasla3a__hint">the two only part company deep into compression</span>
    </div>
    <div class="extrasla3a__item"><Knob :p="c.emphasis" v-bind="knob" label="HF Contour" /></div>
    <div class="extrasla3a__item">
      <span class="extrasla3a__caption">CELL</span>
      <Segmented :p="c.cell" :labels="['FRESH', 'USED', 'TIRED']" />
    </div>
    <div class="extrasla3a__item">
      <span class="extrasla3a__caption">METER</span>
      <Segmented :p="c.meter" :labels="['GR', 'OUT', 'OFF']" />
    </div>
    </div>
    <BarGlobals v-model:scope="ui.scope" />
  
  </div>
</template>

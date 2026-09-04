<script setup>
/**
 * The strip above the Fairchild's panel with what the hardware has no
 * control for: which unit of the pair the face wears, the DC threshold
 * trimmer that lives inside the chassis, which 6386 is fitted and how hard
 * the engine is oversampled — and then the lab's shared globals.
 *
 * **The DC threshold is on this strip and not on the faceplate**, because on
 * the hardware it is inside the box. It is not ours: it is R117, and it is
 * the ratio and knee control. Every emulation that is any good brings it out
 * where you can reach it, and Overloud say so about theirs; putting it here
 * rather than on the panel is how this face says that the metal has no hole
 * for it.
 *
 * The stereo link on the right belongs to the lab and not to this unit. The
 * hardware has no link at all — its lateral-and-vertical mode is two
 * matrices and two entirely independent limiters — so every preset of this
 * model turns it off, and the marker on the group says which controls are
 * ours.
 *
 * Reads / writes: `fc_model`, `fc_dc_threshold_l`, `fc_dc_threshold_r`,
 * `fc_tube`, `fc_oversample`, and through `BarGlobals` the shared handles.
 */
import { Segmented } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';
import { ui, useControls } from './useVmu.js';
import BarGlobals from '../../components/BarGlobals.vue';
import FairScrew from './FairScrew.vue';

const c = useControls();
</script>

<template>
  <div class="extrasfair lab-bar">
    <div class="lab-bar__left">
      <div v-if="c.unit" class="extrasfair__item">
        <span class="extrasfair__caption">UNIT</span>
        <Segmented :p="c.unit" />
        <span class="extrasfair__hint">the mono 660 or the stereo 670</span>
      </div>
      <div class="extrasfair__item">
        <span class="extrasfair__caption">DC THRESHOLD</span>
        <div class="extrasfair__pair">
          <FairScrew :p="c.dcThreshold[0]" size="34px" inside caption="L" />
          <FairScrew :p="c.dcThreshold[1]" size="34px" inside caption="R" />
        </div>
        <span class="extrasfair__hint">R117, the trimmer inside the chassis: the ratio and the knee</span>
      </div>
      <div v-if="c.tube" class="extrasfair__item">
        <span class="extrasfair__caption">TUBE</span>
        <Segmented :p="c.tube" />
        <span class="extrasfair__hint">GE’s 4000 µmhos against JJ’s 3000, which is 2.5 dB</span>
      </div>
      <div v-if="c.oversample" class="extrasfair__item">
        <span class="extrasfair__caption">OVERSAMPLE</span>
        <Segmented :p="c.oversample" />
        <span class="extrasfair__hint">ours; the loop needs it short against a 0.2 ms attack</span>
      </div>
    </div>
    <BarGlobals v-model:scope="ui.scope" />
  </div>
</template>

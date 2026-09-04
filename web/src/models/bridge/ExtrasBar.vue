<script setup>
/**
 * The strip above the Neve's panel with what the hardware has no control
 * for: which unit of the family the face wears, the drive that scales the
 * bridge input, and then the lab's shared globals.
 *
 * The unit selector is model-specific, so it sits on the left with the
 * other per-model controls, and drive with it. Mix, the side-chain
 * high-pass, the stereo link, the drawer and the development panel are the
 * lab's own and sit on the right in the fixed order every model uses,
 * behind the marker that says which controls are ours rather than the
 * hardware's.
 *
 * Reads / writes: `neve_model`, `neve_drive`, and through `BarGlobals` the
 * shared handles. Emits: nothing.
 */
import { Segmented } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';
import { ui, useControls } from './useBridge.js';
import BarGlobals from '../../components/BarGlobals.vue';
import NeveKnob from './NeveKnob.vue';

const c = useControls();
</script>

<template>
  <div class="extrasneve lab-bar">
    <div class="lab-bar__left">
      <div v-if="c.unit" class="extrasneve__item">
        <span class="extrasneve__caption">UNIT</span>
        <Segmented :p="c.unit" />
        <span class="extrasneve__hint">which of the family this face wears</span>
      </div>
      <div v-if="c.drive" class="extrasneve__item">
        <NeveKnob :p="c.drive" :sweep="270" size="46px" label="Drive" />
        <span class="extrasneve__value">{{ c.drive.text }}</span>
        <span class="extrasneve__caption">DRIVE</span>
      </div>
      <div v-if="c.meterSelect" class="extrasneve__item">
        <span class="extrasneve__caption">METER</span>
        <Segmented :p="c.meterSelect" />
        <span class="extrasneve__hint">the 2254's meter switch, which the 33609 has no room for</span>
      </div>
    </div>
    <BarGlobals v-model:scope="ui.scope" />
  </div>
</template>

<script setup>
/**
 * The strip above the TG's module with everything the hardware has no
 * control for.
 *
 * Five of them, and each is here for a stated reason rather than because it
 * is useful. **Input** and **drive** stand in for Chandler's continuous
 * input and their THD mode: this module has no threshold control at all, so
 * how hard you drive it is how you choose where it works. **Region** is not
 * a control on anything — the drawing is ambiguous about which side of the
 * diodes' characteristic the gain element sits on, and putting the choice
 * on the page is more honest than picking one silently. **Arm mismatch**
 * exists because ten adjust-on-test parts say EMI aligned every module by
 * hand, so being slightly out of balance is a feature of the object rather
 * than a defect. **Oversampling** is ours entirely.
 *
 * They sit on the left because they are model-specific; mix, the side-chain
 * high-pass, the stereo link, the drawer and the development panel are the
 * lab's own and sit on the right in the fixed order every model uses.
 *
 * Reads / writes: `tg_region`, `tg_input`, `tg_drive`, `tg_mismatch`,
 * `tg_oversample`, and through `BarGlobals` the shared handles.
 * Emits: nothing.
 */
import { Segmented } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';
import { ui, useControls } from './useTg.js';
import BarGlobals from '../../components/BarGlobals.vue';
import TgSwitch from './TgSwitch.vue';

const c = useControls();
</script>

<template>
  <div class="extrastg lab-bar">
    <div class="lab-bar__left">
      <div v-if="c.region" class="extrastg__item">
        <span class="extrastg__caption">REGION</span>
        <Segmented :p="c.region" />
        <span class="extrastg__hint">the drawing does not say which; this one is the reading it supports</span>
      </div>
      <div v-if="c.input" class="extrastg__item">
        <TgSwitch :p="c.input" :sweep="270" size="46px" label="Input" />
        <span class="extrastg__value">{{ c.input.text }}</span>
        <span class="extrastg__caption">INPUT</span>
      </div>
      <div v-if="c.drive" class="extrastg__item">
        <TgSwitch :p="c.drive" :sweep="270" size="46px" label="Drive" />
        <span class="extrastg__value">{{ c.drive.text }}</span>
        <span class="extrastg__caption">DRIVE</span>
      </div>
      <div v-if="c.mismatch" class="extrastg__item">
        <TgSwitch :p="c.mismatch" :sweep="270" size="46px" label="Arm mismatch" />
        <span class="extrastg__value">{{ c.mismatch.text }}</span>
        <span class="extrastg__caption">MISMATCH</span>
      </div>
      <div v-if="c.oversample" class="extrastg__item">
        <span class="extrastg__caption">OVERSAMPLE</span>
        <Segmented :p="c.oversample" />
      </div>
    </div>
    <BarGlobals v-model:scope="ui.scope" />
  </div>
</template>

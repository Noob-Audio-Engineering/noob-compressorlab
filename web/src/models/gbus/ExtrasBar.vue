<script setup>
/**
 * The strip above the SSL's panel with what the hardware has no control
 * for: the detector's link mode past its first position, the drive that
 * scales the gain cell's second-harmonic term, the ceiling on the control
 * voltage, and the oversampling. Then the lab's shared globals.
 *
 * Those four are model-specific, so they sit on the left with the other
 * per-model controls. Mix, the side-chain high-pass, the stereo link, the
 * drawer and the development panel are the lab's own and sit on the right in
 * the fixed order every model uses, behind the marker that says which
 * controls are ours rather than the hardware's.
 *
 * **The link mode's first position is the hardware and the other three are
 * not.** SSL state the stereo behaviour plainly: the channels are rectified
 * separately and the louder one controls the gain of both. Sum, Dual and
 * Mid/Side are after the modes SSL themselves put on a later unit, and they
 * are ours. The lab's shared stereo-link toggle still wins when it is off,
 * and forces two independent detectors whatever this says.
 *
 * Reads / writes: `ssl_link`, `ssl_drive`, `ssl_range`, `ssl_oversample`,
 * and through `BarGlobals` the shared handles. Emits: nothing.
 */
import { Segmented } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';
import { POT, SWEEP, ui, useControls } from './useGbus.js';
import BarGlobals from '../../components/BarGlobals.vue';
import SslKnob from './SslKnob.vue';

const c = useControls();
</script>

<template>
  <div class="extrasssl lab-bar">
    <div class="lab-bar__left">
      <div v-if="c.linkMode" class="extrasssl__item">
        <span class="extrasssl__caption">DETECTOR</span>
        <Segmented :p="c.linkMode" />
        <span class="extrasssl__hint">Dominant is the hardware</span>
      </div>
      <div v-if="c.drive" class="extrasssl__item">
        <SslKnob
          :p="c.drive"
          :sweep="SWEEP"
          :cap="POT.cap"
          :skirt="POT.skirt"
          :box="POT.box"
          label="Drive"
          style="width: 44px"
        />
        <span class="extrasssl__value">{{ c.drive.text }}</span>
        <span class="extrasssl__caption">DRIVE</span>
      </div>
      <div v-if="c.range" class="extrasssl__item">
        <SslKnob
          :p="c.range"
          :sweep="SWEEP"
          :cap="POT.cap"
          :skirt="POT.skirt"
          :box="POT.box"
          label="Range"
          style="width: 44px"
        />
        <span class="extrasssl__value">{{ c.range.text }}</span>
        <span class="extrasssl__caption">RANGE</span>
      </div>
      <div v-if="c.oversample" class="extrasssl__item">
        <span class="extrasssl__caption">OS</span>
        <Segmented :p="c.oversample" />
        <span class="extrasssl__hint">2x contains both</span>
      </div>
    </div>
    <BarGlobals v-model:scope="ui.scope" />
  </div>
</template>

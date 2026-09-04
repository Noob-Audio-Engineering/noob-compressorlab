<script setup>
/**
 * The strip above the dbx's panel: which unit of the family the face wears,
 * the rear-panel meter trimmer, and the three things dbx never gave anyone.
 * Then the lab's shared globals on the right, behind the marker that says
 * which controls are ours rather than the hardware's.
 *
 * **Everything on the left here is either on the rear panel or is ours.**
 * The meter calibration is a rear-panel trimmer on both units, so it is off
 * the face for the same reason the LA-3A's rear-panel switches are. The knee
 * width, the detector's time constant and the look-ahead are the lab's, and
 * the captions say so.
 *
 * The time constant is the interesting one to expose, and it is exposed on
 * purpose. dbx's whole argument is that a true-RMS detector's attack and
 * release are two sides of one constant and cannot be adjusted separately;
 * dragging this and hearing the release rate change while every attack time
 * changes with it is the clearest demonstration of that there is, and it is
 * a thing no hardware in this family will let anyone do.
 *
 * The knee width is here because dbx never published one for any model in
 * the family and it could not be derived from the drawing. Its default, 6 dB,
 * is an estimate and the hint says so.
 *
 * Reads / writes: `dbx_model`, `dbx_meter_cal`, `dbx_knee_width`, `dbx_tau`,
 * `dbx_lookahead`, and through `BarGlobals` the shared handles.
 */
import { Knob, Segmented } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';
import { ui, useControls } from './useDbx.js';
import BarGlobals from '../../components/BarGlobals.vue';

const c = useControls();
const knob = { size: 42, color: '#c13c37' };
</script>

<template>
  <div class="extrasdbx lab-bar">
    <div class="lab-bar__left">
      <div v-if="c.unit" class="extrasdbx__item">
        <span class="extrasdbx__caption">UNIT</span>
        <Segmented :p="c.unit" />
        <span class="extrasdbx__hint">the original, or the panel that added OverEasy and Infinity+</span>
      </div>
      <div v-if="c.meterCal" class="extrasdbx__item">
        <Knob :p="c.meterCal" v-bind="knob" label="Meter Cal" />
        <span class="extrasdbx__hint">rear panel</span>
      </div>
      <span
        class="extrasdbx__tag"
        title="Numbers dbx never gave anyone: one they never published, and one they built the box so you could not touch."
        >NOT ON THE BOX</span
      >
      <div v-if="c.kneeWidth" class="extrasdbx__item">
        <Knob :p="c.kneeWidth" v-bind="knob" label="OverEasy" />
        <span class="extrasdbx__hint">never published; 6 dB is an estimate</span>
      </div>
      <div v-if="c.tau" class="extrasdbx__item">
        <Knob :p="c.tau" v-bind="knob" label="Detector τ" />
        <span class="extrasdbx__hint">the one number the box is made of: move it and attack and release move together</span>
      </div>
      <div v-if="c.lookahead" class="extrasdbx__item">
        <Knob :p="c.lookahead" v-bind="knob" label="Look-ahead" />
        <span class="extrasdbx__hint">dbx drew this trick in 1995 and could not fit it</span>
      </div>
    </div>
    <BarGlobals v-model:scope="ui.scope" />
  </div>
</template>

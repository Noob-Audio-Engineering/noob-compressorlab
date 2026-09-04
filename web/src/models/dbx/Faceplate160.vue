<script setup>
/**
 * The original dbx 160's front panel, laid out from measurements taken off
 * dbx's own front-panel figure from the 160/161 instruction manual (archive.org/details/dbx_dbx_160, leaf 3) (Figure 1), enlarged, and checked
 * against `research/dbx-160.md` section 3.1.
 *
 * **How the numbers were found.** The drawing's long vertical runs give the
 * wood cheeks' four edges at x 88.5, 150.5, 1778 and 1846, and its long
 * horizontal runs the panel's top at y 169 and, from the cheek edges' own
 * length of 624 rows, its bottom at 793. So the panel face between the
 * cheeks is 1627.5 × 624 and every fraction below is of that box; the whole
 * plate including both cheeks is 1757.5 × 624, an aspect of 2.816.
 *
 * The three knobs were found by scoring rings against the drawing, which fit
 * exactly, so the figure's circles really are circles: centres (348, 460),
 * (664, 462) and (988, 456) with radii 61, 80 and 61. The two threshold
 * indicators came out the same way at (303, 323) and (393, 321), radius 11.
 * The meter bezel, the POWER switch and the three METER buttons are dark
 * rectangles found from their own edge runs, and the silkscreen rows and the
 * logo from bands of dark pixels inside each column.
 *
 * **What is measured and what is not.** Everything geometric here is
 * measured. Every *colour* is this plug-in's own: dbx's figure is a line
 * drawing, there is no colour photograph of a 160 in the reference set, and
 * the research says plainly that it will not state a panel colour. What the
 * drawing and the manual do establish is the indicator behaviour, and that
 * is followed: amber below threshold and red above, unlike every later model
 * in the family, which prints green / yellow / red.
 *
 * **The both-dim case is drawn, because dbx specified it.** "A steady-state,
 * sine-wave tone exactly at the threshold voltage causes both L.E.D.'s to
 * remain dimly illuminated", and their factory procedure calibrates the
 * threshold by turning the control until both are off. The engine publishes
 * the pair as one comparator's two sides, so at the threshold they are equal
 * and half lit.
 *
 * Reads: `dbx_threshold`, `dbx_ratio`, `dbx_output`, `dbx_meter`, and the
 * shared `bypass` for the POWER switch. Streams: `meter` through the VU, and
 * `lamps` through the two indicators.
 */
import { computed } from 'vue';
import { useStreamFrame } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';
import {
  OUTPUT_MARKS_160,
  RATIO_MARKS_160,
  SWEEP_160,
  THRESHOLD_MARKS_160,
  placeMarks,
  ranges,
  useControls,
} from './useDbx.js';
import DbxKnob from './DbxKnob.vue';
import VuFaceDbx from './VuFaceDbx.vue';

const c = useControls();
const r = computed(() => ranges(c, 0));
const lamps = useStreamFrame('lamps');

/** Fractions of the panel face → CSS, centred on the point. */
const at = (x, y) => ({ left: `${x * 100}%`, top: `${y * 100}%` });
/**
 * A knob's box, as a fraction of the panel.
 *
 * The SVG draws its knob at radius 25 in a 100-unit box, so the box is four
 * times the measured radius and the knob inside it comes out at exactly the
 * diameter that was measured; the rest of the box is the room the printed
 * scale needs. At the threshold knob's 0.0375 that is 0.15 of the panel, and
 * the numbers ride at 0.0555 from its centre against a 0.097 half-gap to the
 * compression knob's scale, so no two scales meet.
 */
const knobBox = (radius) => `${radius * 4 * 100}%`;

const thresholdMarks = computed(() => placeMarks(c.threshold, THRESHOLD_MARKS_160, r.value.threshold));
const ratioMarks = computed(() => placeMarks(c.ratio, RATIO_MARKS_160, r.value.ratio));
const outputMarks = computed(() => placeMarks(c.output, OUTPUT_MARKS_160, r.value.output));

/** The POWER switch. dbx fitted one; the lab's shared bypass is what it
 * drives, since a plug-in has no mains. */
const powered = computed(() => !c.bypass.on);
function togglePower() {
  c.bypass.begin();
  c.bypass.setOn(powered.value);
  c.bypass.end();
}

const below = computed(() => (lamps.value ? lamps.value[0] : 1));
const above = computed(() => (lamps.value ? lamps.value[1] : 0));
const meterMode = computed(() => (c.meter ? c.meter.index : 2));
function setMeter(i) {
  if (!c.meter) return;
  c.meter.begin();
  c.meter.setIndex(i);
  c.meter.end();
}
</script>

<template>
  <div class="face160" :class="{ off: !powered }">
    <div class="face160__cheek left" />
    <div class="face160__panel">
      <!-- the top legends -->
      <div class="face160__legend" :style="at(0.1425, 0.1771)">THRESHOLD</div>
      <div class="face160__legend" :style="at(0.3192, 0.1723)">COMPRESSION</div>
      <div class="face160__legend two" :style="at(0.5146, 0.1755)">OUTPUT<br />GAIN</div>

      <!-- the two indicators, amber below and red above, each at its own
           measured place rather than in a row centred on a guess -->
      <div class="face160__ledlabel" :style="at(0.0611, 0.2428)">BELOW</div>
      <span class="face160__led below" :style="{ ...at(0.0937, 0.2468), opacity: 0.18 + 0.82 * below }" />
      <span class="face160__led above" :style="{ ...at(0.1489, 0.2436), opacity: 0.18 + 0.82 * above }" />
      <div class="face160__ledlabel" :style="at(0.1963, 0.2436)">ABOVE</div>

      <!-- the three knobs, at their measured centres and diameters -->
      <div class="face160__knob" :style="{ ...at(0.1213, 0.4663), width: knobBox(0.0375) }">
        <DbxKnob :p="c.threshold" :marks="thresholdMarks" :sweep="SWEEP_160.threshold" :range="r.threshold" size="100%" label="Threshold" />
      </div>
      <div class="face160__knob" :style="{ ...at(0.3155, 0.4696), width: knobBox(0.0492) }">
        <DbxKnob :p="c.ratio" :marks="ratioMarks" :sweep="SWEEP_160.compression" :range="r.ratio" size="100%" label="Compression" />
      </div>
      <div class="face160__knob" :style="{ ...at(0.5146, 0.4599), width: knobBox(0.0375) }">
        <DbxKnob :p="c.output" :marks="outputMarks" :sweep="SWEEP_160.output" :range="r.output" size="100%" label="Output Gain" />
      </div>

      <!-- POWER, and the three METER buttons with their bracket -->
      <div class="face160__caption" :style="at(0.1269, 0.7019)">POWER</div>
      <button
        class="face160__power"
        :class="{ on: powered }"
        type="button"
        :aria-pressed="powered"
        title="The mains switch. Here it is the lab's shared bypass, because a plug-in has no mains."
        :style="at(0.1277, 0.7985)"
        @click="togglePower"
      />

      <div class="face160__caption" :style="at(0.3429, 0.7204)">INPUT</div>
      <div class="face160__caption" :style="at(0.4218, 0.7204)">OUTPUT</div>
      <div class="face160__caption two" :style="at(0.5045, 0.6985)">GAIN<br />CHANGE</div>
      <div class="face160__buttons" :style="at(0.4246, 0.7976)">
        <button
          v-for="(name, i) in ['INPUT', 'OUTPUT', 'GAIN CHANGE']"
          :key="name"
          class="face160__button"
          :class="{ on: meterMode === i }"
          type="button"
          :aria-pressed="meterMode === i"
          :title="`Meter: ${name}`"
          @click="setMeter(i)"
        />
      </div>
      <div class="face160__bracket" :style="at(0.4267, 0.9295)"><span>METER</span></div>

      <!-- the maker's mark, and the meter -->
      <div class="face160__logo" :style="at(0.7641, 0.7981)">noob</div>
      <div class="face160__sub" :style="at(0.8, 0.9239)">COMPRESSOR/LIMITER</div>
      <div class="face160__meter" :style="at(0.799, 0.397)">
        <VuFaceDbx :mode="meterMode" :lit="powered" />
      </div>
    </div>
    <div class="face160__cheek right" />
  </div>
</template>

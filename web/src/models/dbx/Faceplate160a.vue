<script setup>
/**
 * The dbx 160A's front panel, laid out from measurements taken off
 * dbx's own 160A product photograph (adn.harmanpro.com/product_attachments/product_attachments/614_1728149313/160Afront_lg_original.jpg), and checked
 * against `research/dbx-160.md` section 3.2.
 *
 * **How the numbers were found.** Thresholding the photograph on anything
 * darker than the white background gives the panel at x 6–1192, y 5–112,
 * that is 1186 × 107, an aspect of 11.08 against the 10.857 dbx publish for
 * a 1U 19-inch panel; the excess is the chassis the photograph includes.
 * Every fraction below is of that box.
 *
 * The three knob caps were found as contiguous runs of their own hue in the
 * knob row: red at 0.3141, blue at 0.4524 and green at 0.5430 across, all on
 * a centre line at 0.542 down, each cap 0.026 of the panel wide. The thin
 * blue rules that divide the panel came out as the most saturated blue
 * columns, at 0.153, 0.268, 0.414, 0.578 and 0.881. The LEDs were found as
 * lit pixels row by row: the BYPASS and SLAVE pair at 0.189 and 0.236, the
 * threshold trio at 0.369, 0.384 and 0.400, the INPUT and OUTPUT pair at
 * 0.600, and the 19-LED level row starting at 0.642 on a pitch of 0.0105 —
 * with the 12-LED gain-reduction row on the same pitch beneath it, ending
 * under the yellow half of the scale exactly as the research describes.
 *
 * **The colours are measured, unlike the original's.** This is a
 * photograph, so the panel field (#3d3b3c), the red, blue and green caps
 * (#c13c37, #446c87, #436c4d), the blue rules (#148ebf) and the three
 * indicator colours are sampled from it. `rms.css` says which is which.
 *
 * **Two buttons drive the lab's shared controls.** BYPASS is the 160A's
 * relay bypass and SLAVE is its strapping jack, and the lab already has both
 * as shared parameters, so the panel's buttons write those rather than a
 * second copy of each. DISPLAY moves the level row between input and output,
 * which is what dbx's own button does; the model's meter parameter carries
 * the original's third position too, and this face treats it as Output.
 *
 * Reads: `dbx_threshold`, `dbx_ratio`, `dbx_output`, `dbx_knee`,
 * `dbx_meter`, and the shared `bypass` and `link`. Streams: `meter` and
 * `lamps`.
 */
import { computed } from 'vue';
import { useStreamFrame } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';
import {
  OUTPUT_MARKS_160A,
  RATIO_MARKS_160A,
  SWEEP_160A,
  THRESHOLD_MARKS_160A,
  placeMarks,
  ranges,
  useControls,
} from './useDbx.js';
import DbxKnob from './DbxKnob.vue';
import LedDisplay from './LedDisplay.vue';

const c = useControls();
const r = computed(() => ranges(c, 1));
const lamps = useStreamFrame('lamps');
const meter = useStreamFrame('meter');

const at = (x, y) => ({ left: `${x * 100}%`, top: `${y * 100}%` });
const knobBox = (d) => `${d * 1.62 * 100}%`;

const thresholdMarks = computed(() => placeMarks(c.threshold, THRESHOLD_MARKS_160A, r.value.threshold));
const ratioMarks = computed(() => placeMarks(c.ratio, RATIO_MARKS_160A, r.value.ratio));
const outputMarks = computed(() => placeMarks(c.output, OUTPUT_MARKS_160A, r.value.output));

const below = computed(() => (lamps.value ? lamps.value[0] : 1));
const above = computed(() => (lamps.value ? lamps.value[1] : 0));
const overeasy = computed(() => (lamps.value ? lamps.value[3] : 0));
/** `meter[4]` is the gain change, which the lab publishes as ≤ 0. */
const grDb = computed(() => (meter.value ? Math.max(0, -meter.value[4]) : 0));
/** `meter[5]` is what the movement is chasing, in dB against 0 VU. */
const levelDb = computed(() => (meter.value ? meter.value[5] : -60));

/** DISPLAY moves the level row between input and output. */
const showingOutput = computed(() => (c.meter ? c.meter.index !== 0 : true));
function toggleDisplay() {
  if (!c.meter) return;
  c.meter.begin();
  c.meter.setIndex(showingOutput.value ? 0 : 1);
  c.meter.end();
}
const kneeOn = computed(() => (c.knee ? c.knee.index === 1 : false));
function toggleKnee() {
  if (!c.knee) return;
  c.knee.begin();
  c.knee.setIndex(kneeOn.value ? 0 : 1);
  c.knee.end();
}
const bypassed = computed(() => c.bypass.on);
function toggleBypass() {
  c.bypass.begin();
  c.bypass.setOn(!bypassed.value);
  c.bypass.end();
}
const slaved = computed(() => c.link.on);
function toggleSlave() {
  c.link.begin();
  c.link.setOn(!slaved.value);
  c.link.end();
}
</script>

<template>
  <div class="face160a" :class="{ bypassed }">
    <div class="face160a__ear left"><span class="slot" /><span class="slot low" /></div>
    <div class="face160a__panel">
      <span v-for="x in [0.153, 0.268, 0.414, 0.578, 0.881]" :key="x" class="face160a__rule" :style="{ left: `${x * 100}%` }" />

      <div class="face160a__brand" :style="at(0.075, 0.30)">noob</div>
      <div class="face160a__model" :style="at(0.075, 0.68)">160A</div>

      <!-- BYPASS and SLAVE, which are the lab's shared bypass and link -->
      <span class="face160a__led red" :class="{ on: bypassed }" :style="at(0.1893, 0.322)" />
      <span class="face160a__led yellow" :class="{ on: slaved }" :style="at(0.2361, 0.322)" />
      <div class="face160a__tiny" :style="at(0.1893, 0.514)">BYPASS</div>
      <div class="face160a__tiny" :style="at(0.2361, 0.514)">SLAVE</div>
      <button class="face160a__button" type="button" :aria-pressed="bypassed" title="Bypass: the lab's shared bypass" :style="at(0.1893, 0.75)" @click="toggleBypass" />
      <button class="face160a__button" type="button" :aria-pressed="slaved" title="Slave: the lab's shared stereo link, which here is True RMS Power Summing" :style="at(0.2361, 0.75)" @click="toggleSlave" />

      <!-- threshold, its three indicators and the OverEasy button -->
      <div class="face160a__knob" :style="{ ...at(0.3141, 0.542), width: knobBox(0.026) }">
        <DbxKnob :p="c.threshold" :marks="thresholdMarks" :sweep="SWEEP_160A.threshold" :range="r.threshold" cap="var(--dbx-red)" :scale-font="13" size="100%" label="Threshold" />
      </div>
      <div class="face160a__caption" :style="at(0.3141, 0.738)">THRESHOLD</div>

      <!--
        The three indicators, on the two rows dbx print them on: BELOW and
        ABOVE under their own lamps, OVEREASY a row lower and centred under
        the middle one, with a leader running from that lamp down through
        the word to its button. Flattening the three onto one row is what
        the panel does not do, and it is why they fit at all: the lamps are
        only 0.015 of the panel apart and the word OVEREASY is twice that
        wide.
      -->
      <span class="face160a__leader" :style="{ left: `${0.3836 * 100}%`, top: '34%', height: '38%' }" />
      <span class="face160a__led green" :style="{ ...at(0.3685, 0.322), opacity: 0.2 + 0.8 * below }" />
      <span class="face160a__led amber" :style="{ ...at(0.3836, 0.322), opacity: 0.2 + 0.8 * overeasy }" />
      <span class="face160a__led red on" :style="{ ...at(0.3997, 0.322), opacity: 0.2 + 0.8 * above }" />
      <div class="face160a__tiny" :style="at(0.3685, 0.407)">BELOW</div>
      <div class="face160a__tiny" :style="at(0.3997, 0.407)">ABOVE</div>
      <div class="face160a__tiny boxed" :style="at(0.3836, 0.523)">OVEREASY</div>
      <button class="face160a__button" type="button" :aria-pressed="kneeOn" title="OverEasy" :style="at(0.3836, 0.75)" @click="toggleKnee" />

      <!-- ratio, which carries on past infinity -->
      <div class="face160a__knob" :style="{ ...at(0.4524, 0.542), width: knobBox(0.026) }">
        <DbxKnob :p="c.ratio" :marks="ratioMarks" :sweep="SWEEP_160A.ratio" :range="r.ratio" cap="var(--dbx-blue)" :scale-font="13" size="100%" label="Compression Ratio" />
      </div>
      <div class="face160a__caption two" :style="at(0.4524, 0.82)">COMPRESSION<br />RATIO</div>

      <!-- output gain -->
      <div class="face160a__knob" :style="{ ...at(0.543, 0.542), width: knobBox(0.026) }">
        <DbxKnob :p="c.output" :marks="outputMarks" :sweep="SWEEP_160A.output" :range="r.output" cap="var(--dbx-green)" :scale-font="13" size="100%" label="Output Gain" />
      </div>
      <div class="face160a__caption two" :style="at(0.543, 0.82)">OUTPUT<br />GAIN</div>

      <!-- the display, its two indicators and the LED rows -->
      <!--
        Measured at 0.336 and 0.411 down; drawn 0.016 further apart than
        that so the two words clear each other. On the real panel that is
        under half a millimetre, and it is the one place on this face where
        a measured position was moved rather than the type shrunk further.
      -->
      <span class="face160a__led red" :class="{ on: !showingOutput }" :style="at(0.6, 0.32)" />
      <span class="face160a__led red" :class="{ on: showingOutput }" :style="at(0.6, 0.427)" />
      <div class="face160a__tiny left" :style="at(0.606, 0.32)">INPUT</div>
      <div class="face160a__tiny left" :style="at(0.606, 0.427)">OUTPUT</div>
      <button class="face160a__button" type="button" title="Display: which level the row shows" :style="at(0.6, 0.75)" @click="toggleDisplay" />
      <div class="face160a__caption" :style="at(0.6, 0.93)">DISPLAY</div>

      <div class="face160a__display" :style="at(0.6421, 0.5)">
        <LedDisplay :level-db="levelDb" :gr-db="grDb" :lit="!bypassed" />
      </div>
      <div class="face160a__sub" :style="at(0.79, 0.944)">C O M P R E S S O R / L I M I T E R</div>
    </div>
    <div class="face160a__ear right"><span class="slot" /><span class="slot low" /></div>
  </div>
</template>

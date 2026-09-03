<script setup>
/**
 * The front panel, laid out from reference photographs of the original
 * 2U unit: every element sits at a fraction of the faceplate's width and
 * height (`left` / `top` percentages, measured off the photos), inside a
 * box that keeps the panel's 5.2 : 1 aspect between the rack ears and
 * scales with the window. Sizes inside the panel use container-query
 * units (`cqw`, hundredths of the plate width), so lettering, knobs and
 * buttons grow with it.
 *
 * Left to right, as on the hardware: the two large INPUT and OUTPUT knobs
 * with their printed dB skirts and captions below, the small ATTACK knob
 * above the small RELEASE knob at one x, the vertical RATIO column of four
 * push buttons (20 / 12 / 8 / 4 printed beside them), the VU meter with
 * the model lettering under it, the vertical METER column (GR / +8 / +4 /
 * OFF, the red-capped OFF at the bottom) and the power toggle. The meter
 * and both button columns are centred on the axis of the big knobs. Rack
 * ears carry two slotted screws each; four small faceplate screws sit at
 * the panel corners.
 *
 * Three looks follow the `fet_revision` parameter (`lookOf` in
 * `useFet.js`):
 * - `bluestripe` (A, B): brushed silver plate, a blue block around the
 *   meter carrying the lettering in white, black knobs with black skirts;
 * - `blackface` (C to G, LN): black plate, silver-capped knobs with light
 *   skirt scales, the maker's badge above the meter and the model
 *   lettering under it;
 * - `silverface` (H): silver plate with the recessed left section and
 *   "PEAK LIMITER" above the big knobs, the blue badge at the right.
 *
 * Reads: `fet_input`, `fet_output`, `fet_attack`, `fet_release`, `fet_ratio`, `fet_meter`,
 * `fet_revision` (look), `bypass` (power).  Handles come from `useFet.js`. Streams: `meter` (through the VU
 * component).
 */
import { computed } from 'vue';
import { attackToRotation, lookOf, markToRotation, releaseToRotation, rotationToAttack, rotationToMark, rotationToRelease, useControls } from './useFet.js';
import Knob1176 from './Knob1176.vue';
import VuMeter1176 from './VuMeter1176.vue';
import RatioButtons from './RatioButtons.vue';
import MeterButtons from './MeterButtons.vue';
import PowerSwitch from './PowerSwitch.vue';

const c = useControls();
const look = computed(() => lookOf(c.revision.index));
const MARKS = [0, 6, 12, 18, 24, 30, 36, 42, 48].map((v) => ({ value: v, label: String(v) }));
const ATTACK_MARKS = [{ value: 0, label: 'OFF' }, ...[1, 2, 3, 4, 5, 6, 7].map((v) => ({ value: v, label: String(v) }))];
const RELEASE_MARKS = [1, 2, 3, 4, 5, 6, 7].map((v) => ({ value: v, label: String(v) }));
const fmtMark = (v) => String(Math.round(v));
const fmtAttack = (v) => (v < 0.5 ? 'OFF' : v.toFixed(1));
const fmtRelease = (v) => v.toFixed(1);

/** Position helper: fractions of the faceplate → CSS. */
const at = (x, y, extra = {}) => ({ left: `${x * 100}%`, top: `${y * 100}%`, ...extra });
/** Box helper: fractions of the faceplate → CSS. */
const box = (x, y, w, h) => ({ left: `${x * 100}%`, top: `${y * 100}%`, width: `${w * 100}%`, height: `${h * 100}%` });
</script>

<template>
  <section class="face1176" :class="look">
    <div class="face1176__ear left"><span class="screw big"></span><span class="screw big"></span></div>
    <div class="face1176__plate">
      <div class="face1176__panel">
        <!-- corner faceplate screws -->
        <span class="screw small abs" :style="at(0.02, 0.1)"></span>
        <span class="screw small abs" :style="at(0.02, 0.9)"></span>
        <span class="screw small abs" :style="at(0.98, 0.1)"></span>
        <span class="screw small abs" :style="at(0.98, 0.9)"></span>

        <!-- Bluestripe: the blue block behind and around the meter -->
        <div v-if="look === 'bluestripe'" class="face1176__block" :style="box(0.548, 0.04, 0.224, 0.92)"></div>
        <!-- Silverface: the recessed peak-limiter section -->
        <template v-if="look === 'silverface'">
          <div class="face1176__section" :style="box(0.022, 0.06, 0.305, 0.88)"></div>
          <div class="face1176__print abs" :style="at(0.17, 0.13)">PEAK LIMITER</div>
        </template>

        <div class="abs" :style="at(0.075, 0.55)">
          <Knob1176 :p="c.input" :marks="MARKS" :to-rotation="markToRotation" :from-rotation="rotationToMark" size="13cqw" :mark-size="7.5" :sweep="300" :format="fmtMark" bare />
        </div>
        <div class="abs" :style="at(0.25, 0.55)">
          <Knob1176 :p="c.output" :marks="MARKS" :to-rotation="markToRotation" :from-rotation="rotationToMark" size="13cqw" :mark-size="7.5" :sweep="300" :format="fmtMark" bare />
        </div>
        <div class="face1176__print abs" :style="at(0.075, 0.93)">INPUT</div>
        <div class="face1176__print abs" :style="at(0.25, 0.93)">OUTPUT</div>

        <div class="face1176__print abs" :style="at(0.4, 0.07)">ATTACK</div>
        <div class="abs" :style="at(0.4, 0.3)">
          <Knob1176 :p="c.attack" :marks="ATTACK_MARKS" :to-rotation="attackToRotation" :from-rotation="rotationToAttack" size="7.6cqw" :body="20" :mark-size="12.5" :sweep="270" :format="fmtAttack" bare />
        </div>
        <div class="face1176__print abs" :style="at(0.4, 0.53)">RELEASE</div>
        <div class="abs" :style="at(0.4, 0.77)">
          <Knob1176 :p="c.release" :marks="RELEASE_MARKS" :to-rotation="releaseToRotation" :from-rotation="rotationToRelease" size="7.6cqw" :body="20" :mark-size="12.5" :sweep="270" :format="fmtRelease" bare />
        </div>

        <div class="face1176__print abs" :style="at(0.51, 0.08)">RATIO</div>
        <div class="abs" :style="at(0.51, 0.5)"><RatioButtons :p="c.ratio" /></div>

        <!-- the maker's badge above the meter (black face), the small round logo (blue stripe) -->
        <div v-if="look === 'blackface'" class="face1176__badge abs" :style="at(0.66, 0.13)"><span>NOOB</span></div>
        <div v-if="look === 'bluestripe'" class="face1176__badge round abs" :style="at(0.66, 0.13)"><span>N</span></div>
        <div class="face1176__meter abs" :style="box(0.565, 0.26, 0.19, 0.48)">
          <VuMeter1176 :mode="c.meter" />
        </div>
        <div v-if="look !== 'silverface'" class="face1176__nameplate abs" :style="box(0.548, 0.78, 0.224, 0.16)">
          <b>NOOB 1176{{ look === 'blackface' ? 'LN' : '' }} LIMITING AMPLIFIER</b>
          <span>NOOB AUDIO · A SPOOF</span>
        </div>

        <div class="face1176__print abs" :style="at(0.805, 0.08)">METER</div>
        <div class="abs" :style="at(0.805, 0.5)"><MeterButtons :p="c.meter" /></div>

        <template v-if="look === 'silverface'">
          <div class="face1176__badge abs" :style="at(0.95, 0.3)"><span>NOOB</span></div>
          <div class="abs" :style="at(0.95, 0.74)"><PowerSwitch :p="c.bypass" /></div>
        </template>
        <div v-else class="abs" :style="at(0.95, 0.5)"><PowerSwitch :p="c.bypass" /></div>
      </div>
    </div>
    <div class="face1176__ear right"><span class="screw big"></span><span class="screw big"></span></div>
  </section>
</template>

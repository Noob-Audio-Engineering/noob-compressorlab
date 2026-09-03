<script setup>
/**
 * The CL-1B's front panel, laid out from `research/CL-1B.md` section 2.2,
 * which measured every control off Lydkraft's own hi-res photograph and
 * gives its centre as a fraction of the panel. I checked that table against
 * the same file before using it: the panel's bounding box measures an
 * aspect of 3.693 against the dossier's 3.695, and a column profile of the
 * dark controls puts them within 0.01 of every listed position.
 *
 * The panel is a full 19 inch rack width and **three** units tall, not two.
 * The specification sheet says two, but the same line gives 132 mm and
 * 5.2 inch, both of which are 3U, and the measured aspect agrees. So the
 * plate is drawn at 483 by 131 mm.
 *
 * Top row: the wordmark block, then GAIN, RATIO and THRESHOLD, then the
 * three-position METER lever, then the VU meter in its black bezel. Bottom
 * row: the IN switch under the wordmark, then ATTACK and RELEASE, then the
 * attack/release and sidechain levers, the pilot jewel and the mains
 * switch.
 *
 * Two things this face deliberately does not do. It prints no legend under
 * the meter saying what the selector is pointing at, unlike the 1176 and
 * LA-3A faces: this panel already prints input, compression and output
 * beside the lever, so a second copy would be silkscreen the hardware does
 * not have. And it draws no cell-age control, because the dossier withholds
 * one on the grounds that the element is claimed not to degrade.
 *
 * Reads: `cl1b_gain`, `cl1b_ratio`, `cl1b_threshold`, `cl1b_attack`,
 * `cl1b_release`, `cl1b_mode`, `cl1b_meter`, `cl1b_bus`, `bypass`, and the
 * mains handle. Streams: `meter`, through this model's own VU face.
 */
import { computed } from 'vue';
import { GAIN_MARKS, RATIO_MARKS, THRESHOLD_MARKS, TIME_MARKS, useControls } from './useCl1b.js';
import Cl1bKnob from './Cl1bKnob.vue';
import Cl1bLever from './Cl1bLever.vue';
import Cl1bToggle from './Cl1bToggle.vue';
import Cl1bPower from './Cl1bPower.vue';
import VuFaceCl1b from './VuFaceCl1b.vue';

const c = useControls();
const lit = computed(() => c.power.on.value);

/** Position helper: fractions of the panel → CSS, centred on the point. */
const at = (x, y) => ({ left: `${x * 100}%`, top: `${y * 100}%` });
/** Box helper: the centre of the box and its size, both as fractions of the panel. */
const box = (x, y, w, h) => ({ left: `${x * 100}%`, top: `${y * 100}%`, width: `${w * 100}%`, height: `${h * 100}%` });

/** A 36 mm knob on a 483 mm panel is 0.0745 of the width; the SVG's body is 0.52 of its own box. */
const KNOB = '14.3cqw';
</script>

<template>
  <section class="facecl1b" :class="{ unlit: !lit }">
    <div class="facecl1b__ear left"><span class="slot"></span><span class="slot"></span></div>
    <div class="facecl1b__plate">
      <div class="facecl1b__panel">
        <!-- the four chrome screws, inboard of the ears, top and bottom -->
        <span class="screwcl1b abs" :style="at(0.062, 0.14)"></span>
        <span class="screwcl1b abs" :style="at(0.062, 0.86)"></span>
        <span class="screwcl1b abs" :style="at(0.938, 0.14)"></span>
        <span class="screwcl1b abs" :style="at(0.938, 0.86)"></span>

        <!-- the wordmark block, three lines, left aligned in the leftmost sixth -->
        <div class="facecl1b__mark" :style="{ left: '4%', top: '44%' }">
          <b>NOOB-TECH</b>
          <span>COMPRESSOR&nbsp;&nbsp;CL 1B</span>
          <span>NOOB AUDIO&nbsp;&nbsp;DENMARK</span>
        </div>

        <!-- top row -->
        <div class="abs" :style="at(0.218, 0.268)"><Cl1bKnob :p="c.gain" :marks="GAIN_MARKS" :size="KNOB" label="Gain" /></div>
        <div class="facecl1b__cap abs" :style="at(0.218, 0.485)">GAIN</div>

        <div class="abs" :style="at(0.362, 0.272)"><Cl1bKnob :p="c.ratio" :marks="RATIO_MARKS" :size="KNOB" label="Ratio" /></div>
        <div class="facecl1b__cap abs" :style="at(0.362, 0.485)">RATIO</div>

        <div class="abs" :style="at(0.506, 0.268)"><Cl1bKnob :p="c.threshold" :marks="THRESHOLD_MARKS" :size="KNOB" label="Threshold" /></div>
        <div class="facecl1b__cap abs" :style="at(0.506, 0.485)">THRESHOLD</div>

        <div class="abs" :style="at(0.631, 0.30)">
          <Cl1bLever :p="c.meter" :legends="['input', 'compres-\nsion', 'output']" caption="METER" />
        </div>

        <div class="facecl1b__meter abs" :style="box(0.788, 0.271, 0.174, 0.323)">
          <VuFaceCl1b :lit="lit" />
        </div>

        <!-- bottom row -->
        <div class="abs" :style="at(0.111, 0.748)"><Cl1bToggle :p="c.bypass" /></div>

        <div class="abs" :style="at(0.216, 0.742)"><Cl1bKnob :p="c.attack" :marks="TIME_MARKS" :size="KNOB" label="Attack" /></div>
        <div class="facecl1b__cap abs" :style="at(0.216, 0.958)">ATTACK</div>

        <div class="abs" :style="at(0.360, 0.740)"><Cl1bKnob :p="c.release" :marks="TIME_MARKS" :size="KNOB" label="Release" /></div>
        <div class="facecl1b__cap abs" :style="at(0.360, 0.958)">RELEASE</div>

        <div class="abs" :style="at(0.516, 0.77)">
          <Cl1bLever :p="c.mode" :legends="['fixed', 'fix./man.', 'manual']" caption="attack/release" capline2="SELECT" />
        </div>

        <div class="abs" :style="at(0.631, 0.77)">
          <Cl1bLever :p="c.bus" :legends="['off', '1', '2']" caption="sidechain" capline2="BUS SELECT" />
        </div>

        <div class="facecl1b__jewel abs" :class="{ on: lit }" :style="at(0.735, 0.746)"></div>

        <div class="abs" :style="at(0.840, 0.752)"><Cl1bPower :p="c.power" /></div>
      </div>
    </div>
    <div class="facecl1b__ear right"><span class="slot"></span><span class="slot"></span></div>
  </section>
</template>

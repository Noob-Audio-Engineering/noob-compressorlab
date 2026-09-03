<script setup>
/**
 * The front panel, laid out from measurements of a reissue LA-2A photograph
 * (a straight-on rack shot; every position below is a fraction of the full
 * 19-inch width for x and of the 3U panel height for y, read off the photo
 * with a ruler, not a grid). The root keeps the 19 : 5.25 aspect at any
 * width; a ResizeObserver hands the width to the children so knobs, screws
 * and type scale with the panel.
 *
 * Left to right: the maker's block, the LIMIT / COMPRESS toggle with its
 * neighbouring bushing, the large GAIN knob, the VU meter in its bezel
 * under the diamond logo, the large PEAK REDUCTION knob, the meter
 * selector knob under its three labels, the POWER toggle; eight slotted
 * panel screws (three on the top edge, four on the bottom, one under the
 * meter), and rack ears with two
 * round rack screws each. Every control is a framework handle; every look
 * is this plug-in's.
 */
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import { useOpto } from './useOpto.js';
import BigKnob from './BigKnob.vue';
import VuFace from './VuFace.vue';
import SelectorKnob from './SelectorKnob.vue';
import ToggleLever from './ToggleLever.vue';
import TrimPot from './TrimPot.vue';

const panel = useOpto();
const root = ref(null);
const w = ref(1100);
let ro = null;
onMounted(() => {
  ro = new ResizeObserver(() => (w.value = root.value?.clientWidth || w.value));
  ro.observe(root.value);
  w.value = root.value.clientWidth;
});
onBeforeUnmount(() => ro?.disconnect());

/** Panel height for the width: 19 in × 5.25 in. */
const H = () => w.value / (19 / 5.25);
/** Absolute placement, centred on a fractional point; sizes in fractions of the width. */
const at = (x, y, wf, hf = wf, ratio = 1) => ({
  position: 'absolute',
  left: `${x * 100}%`,
  top: `${y * 100}%`,
  width: `${wf * w.value}px`,
  height: `${(hf * w.value) / ratio}px`,
  transform: 'translate(-50%, -50%)',
});
const text = (x, y, size = 0.0045, anchor = 'center') => ({
  position: 'absolute',
  left: `${x * 100}%`,
  top: `${y * 100}%`,
  transform: anchor === 'left' ? 'translate(0, -50%)' : 'translate(-50%, -50%)',
  fontSize: `${Math.max(7, size * w.value)}px`,
  whiteSpace: 'nowrap',
});

/** What the meter selector is pointing at, printed under the meter. */
const meterLegend = computed(() => ['GAIN REDUCTION', 'OUTPUT +10', 'OUTPUT +4'][panel.meter.index] || '');

const KNOB = 0.118; // number circle diameter, about 0.055 of the width in radius
/*
 * Measured off `scratchpad/ref/la2a-ref.jpg`. The photograph gives the screws
 * as fractions of the faceplate between the ears (0.051 and 0.952 along the
 * top; 0.030, 0.347, 0.656 and 0.974 along the bottom); these are the same
 * points expressed against the whole plate, which is what `at()` takes, so
 * each is 0.057 + 0.886 × the panel fraction. The one under the meter is the
 * bezel screw; it sits clear of the meter frame rather than on it.
 */
const panelScrews = [
  [0.102, 0.05],
  [0.617, 0.152],
  [0.9, 0.05],
  [0.084, 0.928],
  [0.364, 0.928],
  [0.638, 0.928],
  [0.92, 0.928],
];
const rackScrews = [
  [0.017, 0.29],
  [0.017, 0.74],
  [0.983, 0.29],
  [0.983, 0.74],
];
</script>

<template>
  <section ref="root" class="plate-root relative w-full select-none" :style="{ aspectRatio: '1920 / 520' }">
    <!-- rack ears -->
    <div class="ear absolute inset-y-0 left-0" :style="{ width: 0.057 * w + 'px' }"></div>
    <div class="ear absolute inset-y-0 right-0" :style="{ width: 0.057 * w + 'px' }"></div>
    <div v-for="([x, y], i) in rackScrews" :key="'r' + i" class="rack-screw" :style="at(x, y, 0.018)"></div>

    <!-- the faceplate between the ears -->
    <div class="faceplate absolute inset-y-0" :style="{ left: 0.057 * w + 'px', right: 0.057 * w + 'px' }"></div>
    <div v-for="([x, y], i) in panelScrews" :key="'s' + i" class="screw" :style="at(x, y, 0.011)"></div>

    <!-- maker's block: a red-bordered badge with a slanted right edge and twin top lines running off to the right; the model text starts clear of the slant -->
    <svg class="absolute" :style="{ left: 0.105 * w + 'px', top: 0.16 * H() + 'px', width: 0.28 * w + 'px', height: 0.2 * H() + 'px' }" viewBox="0 0 280 100" preserveAspectRatio="none">
      <path d="M 0 98 L 0 8 L 245 8" fill="none" stroke="#b5343a" stroke-width="1.4" vector-effect="non-scaling-stroke" />
      <path d="M 0 98 L 128 98 L 151 2 L 245 2" fill="none" stroke="#b5343a" stroke-width="1.4" vector-effect="non-scaling-stroke" />
      <path d="M 134 98 L 157 2" fill="none" stroke="#b5343a" stroke-width="1.4" vector-effect="non-scaling-stroke" />
    </svg>
    <div class="maker" :style="text(0.169, 0.253, 0.012)">NOOB COMPRESSOR</div>
    <div class="maker-sub" :style="text(0.169, 0.318, 0.0046)">SPOOF LEVELING CO.</div>
    <div class="engraved" :style="text(0.272, 0.205, 0.0058, 'left')">LEVELING AMPLIFIER</div>
    <div class="engraved" :style="text(0.272, 0.275, 0.0058, 'left')">MODEL NOOB-LA2A</div>




    <!-- diamond logo -->
    <svg class="absolute" :style="at(0.5, 0.106, 0.06, 0.018)" viewBox="0 0 120 36">
      <polygon points="18,2 46,2 58,18 46,34 18,34 6,18" fill="none" stroke="#3a3632" stroke-width="1.6" />
      <text x="32" y="23" text-anchor="middle" font-size="12" font-weight="800" fill="#3a3632" font-family="Inter, sans-serif">NC</text>
      <text x="90" y="23" text-anchor="middle" font-size="9" font-weight="700" letter-spacing="1" fill="#3a3632" font-family="Inter, sans-serif">SPOOFS</text>
    </svg>

    <!-- limit / compress -->
    <div class="engraved" :style="text(0.104, 0.583)">LIMIT</div>
    <div :style="at(0.104, 0.673, 0.03, 0.03)"><ToggleLever :p="panel.mode" :size="0.03 * w" /></div>
    <div class="engraved" :style="text(0.104, 0.813)">COMPRESS</div>
<!--
      R37, the limit-response pre-emphasis, is a real screwdriver trim in a hex
      bushing beside the toggle, so it is the working control here rather than
      an ornament with the parameter hidden away on a strip.
    -->
    <div :style="at(0.155, 0.692, 0.032)"><TrimPot :p="panel.emphasis" :size="0.032 * w" /></div>

    <!-- gain -->
    <div :style="at(0.289, 0.606, KNOB)"><BigKnob :p="panel.gain" :size="KNOB * w" label="Gain" /></div>
    <div class="engraved" :style="text(0.292, 0.89)">GAIN</div>

    <!-- meter -->
    <div :style="at(0.501, 0.404, 0.206, 0.206, 395 / 230)"><VuFace :width="0.206 * w" /></div>
<!-- the meter zero trim, through its hole below the meter -->
    <div v-if="panel.meterZero" :style="at(0.5, 0.763, 0.016)"><TrimPot :p="panel.meterZero" :size="0.016 * w" :hex="false" /></div>
    <div v-else class="screw" :style="at(0.5, 0.763, 0.011)"></div>
    <!-- what the meter is showing, printed on the panel where it has room to be read -->
    <div class="engraved" :style="text(0.501, 0.677, 0.0062)">{{ meterLegend }}</div>

    <!-- peak reduction -->
    <div :style="at(0.708, 0.606, KNOB)"><BigKnob :p="panel.peakReduction" :size="KNOB * w" label="Peak Reduction" /></div>
    <div class="engraved" :style="text(0.713, 0.89)">PEAK REDUCTION</div>

    <!-- meter selector -->
    <div class="engraved" :style="text(0.844, 0.12)">GAIN REDUCTION</div>
    <div class="engraved" :style="text(0.792, 0.173)">OUTPUT +10</div>
    <div class="engraved" :style="text(0.898, 0.173)">OUTPUT +4</div>
    <div :style="at(0.841, 0.346, 0.072)"><SelectorKnob :p="panel.meter" :size="0.072 * w" /></div>

    <!-- power -->
    <div class="engraved" :style="text(0.844, 0.567)">ON</div>
    <div :style="at(0.841, 0.702, 0.03, 0.03)"><ToggleLever :p="panel.bypass" :size="0.03 * w" inverted /></div>
    <div class="engraved" :style="text(0.844, 0.794)">POWER</div>
  </section>
</template>

<style scoped>
.plate-root {
  filter: drop-shadow(0 6px 14px rgba(0, 0, 0, 0.55));
}
.ear {
  background:
    repeating-linear-gradient(90deg, rgba(255, 255, 255, 0.04) 0 1px, rgba(0, 0, 0, 0.04) 1px 2px),
    linear-gradient(180deg, #cfcbc2, #b3ada2);
  border-radius: 4px;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.55), inset 0 -2px 3px rgba(0, 0, 0, 0.25);
}
.faceplate {
  background:
    repeating-linear-gradient(90deg, rgba(255, 255, 255, 0.05) 0 1px, rgba(0, 0, 0, 0.035) 1px 2px),
    linear-gradient(180deg, #dcd8cf 0%, #c9c4ba 45%, #b7b1a6 100%);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.7), inset 0 -2px 4px rgba(0, 0, 0, 0.25);
}
.rack-screw {
  border-radius: 50%;
  background: radial-gradient(circle at 40% 35%, #4a4540, #14120f 70%);
  box-shadow: inset 0 0 0 1px #08070699, 0 1px 1px rgba(255, 255, 255, 0.3);
}
.screw {
  border-radius: 50%;
  background: radial-gradient(circle at 35% 35%, #f4f1eb, #8e887c 70%, #5e594f);
  box-shadow: inset 0 0 0 1px rgba(0, 0, 0, 0.35);
}
.screw::after {
  content: '';
  position: absolute;
  left: 15%;
  right: 15%;
  top: 44%;
  height: 12%;
  background: #4a4640;
  transform: rotate(35deg);
}
.bushing {
  border-radius: 50%;
  background: radial-gradient(circle at 40% 35%, #b9b3a8, #6b665e 60%, #3d3934);
  box-shadow: inset 0 0 0 1px #2a2724, 0 1px 1px rgba(255, 255, 255, 0.4);
}
.bushing::after {
  content: '';
  position: absolute;
  inset: 32%;
  border-radius: 50%;
  background: radial-gradient(circle at 40% 35%, #d8d3c9, #7a746a 70%);
}
.engraved {
  color: #2a2724;
  text-shadow: 0 1px 0 rgba(255, 255, 255, 0.55);
  letter-spacing: 0.14em;
  text-transform: uppercase;
  font-weight: 600;
}
.maker {
  color: #b5343a;
  font-weight: 900;
  letter-spacing: -0.01em;
  font-stretch: condensed;
  text-shadow: 0 1px 0 rgba(255, 255, 255, 0.35);
}
.maker-sub {
  color: #b5343a;
  font-weight: 600;
  letter-spacing: 0.3em;
}
</style>

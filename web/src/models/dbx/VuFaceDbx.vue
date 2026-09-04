<script setup>
/**
 * The original 160's illuminated VU meter: a rectangular movement in a
 * rectangular bezel, a cream face with the VU scale above and the 0-to-100
 * percentage row below, a black needle on a pivot under the face, and the
 * zero-set screw beneath.
 *
 * dbx's front-panel drawing is monochrome and its meter face is blank, so
 * the *bezel and glass proportions* below are measured off it and everything
 * printed on the face is a standard VU scale in this plug-in's own style.
 * That is said here rather than presented as a reading of the drawing.
 *
 * Measured off dbx's own front-panel figure from the 160/161 instruction manual (archive.org/details/dbx_dbx_160, leaf 3), as fractions of the panel face
 * between the wood cheeks: the bezel spans x 0.635 to 0.963 and y 0.139 to
 * 0.655, and the glass inside it x 0.668 to 0.927 and y 0.231 to 0.563. So
 * the glass is 4.6 % of the panel width in from the bezel on the left and
 * 3.7 % on the right, which is the asymmetry a hand-drawn figure has and is
 * split evenly here.
 *
 * The needle follows element 5 of the `meter` stream, which is the level the
 * movement is chasing; the standard VU ballistics run in the audio thread
 * for every model in the lab, so this asks the framework's needle for
 * nothing but a short follow to bridge frames.
 *
 * On GAIN CHANGE the needle rests at 0 and swings left, which is what the
 * original does: it is showing the control voltage, and dbx's own
 * calibration puts 0 VU at no reduction.
 *
 * Props: `width` (px), `mode` (0 input, 1 output, 2 gain change), `lit`.
 */
import { computed } from 'vue';
import { useNeedle } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';

const props = defineProps({
  mode: { type: Number, default: 2 },
  lit: { type: Boolean, default: true },
});

/*
 * The scale runs −20 to +3 in both positions. dbx's metering range is 60 dB
 * and their movement is a VU, so the *movement* shows the top 23 dB of it
 * and the LED rows on the later models show the rest; this is the movement.
 */
const needle = useNeedle('meter', {
  index: 5,
  unit: 'db',
  mode: 'level',
  min: -20,
  max: 3,
  riseMs: 40,
  damping: 1,
  sweep: 78,
});
const marks = needle.marks([-20, -10, -7, -5, -3, -2, -1, 0, 1, 2, 3]);
const minor = needle.marks([-15, -8, -6, -4, -2.5, -1.5, -0.5, 0.5, 1.5, 2.5]);
const pct = needle.marks([-20, -13.98, -7.96, -4.44, -1.94, 0]);
const PCT_LABEL = { '-20': '0', '-13.98': '20', '-7.96': '40', '-4.44': '60', '-1.94': '80', 0: '100' };

const legend = computed(() => ['INPUT LEVEL', 'OUTPUT LEVEL', 'GAIN CHANGE'][props.mode] || 'GAIN CHANGE');

/*
 * Face geometry in a 535 x 298 viewBox, from the measured bezel and glass.
 *
 * The bezel is 0.3284 of the panel wide and 0.5153 of it tall; the panel is
 * 2.816 : 1, so in one set of units the bezel is 1.7945 : 1 and 535 across
 * makes it 298 down. The glass inside it lands at (53.9, 52.9), 421.1 by
 * 191.8, which is the measurement transferred rather than a proportion
 * chosen to look right.
 *
 * The pivot sits below the glass so a 78-degree sweep draws a shallow arc
 * across it: a radius of 285 puts the scale's chord at 85 % of the glass
 * width, its top 25 units below the glass top and its ends 103 above the
 * glass bottom.
 */
const CX = 267.5;
const CY = 363;
const R = 285;
function pt(deg, r) {
  const a = ((deg - 90) * Math.PI) / 180;
  return [CX + r * Math.cos(a), CY + r * Math.sin(a)];
}
function arc(r, a0, a1) {
  const [x0, y0] = pt(a0, r);
  const [x1, y1] = pt(a1, r);
  return `M ${x0.toFixed(1)} ${y0.toFixed(1)} A ${r} ${r} 0 0 1 ${x1.toFixed(1)} ${y1.toFixed(1)}`;
}
const SWEEP = 78;
const A0 = -SWEEP / 2;
const A1 = SWEEP / 2;
/** Where 0 VU sits, so the red band can start there. */
const zeroDeg = computed(() => A0 + ((0 - -20) / (3 - -20)) * SWEEP);

function tick(m, inner, outer) {
  const deg = A0 + m.frac * SWEEP;
  const [x1, y1] = pt(deg, inner);
  const [x2, y2] = pt(deg, outer);
  return { x1, y1, x2, y2 };
}
function label(m, r) {
  const deg = A0 + m.frac * SWEEP;
  const [x, y] = pt(deg, r);
  return { x, y };
}
const needleEnd = computed(() => {
  const deg = A0 + needle.frac.value * SWEEP;
  return pt(deg, R - 8);
});
</script>

<template>
  <div class="dbxvu" :class="{ dark: !lit }">
    <svg viewBox="0 0 535 298" class="dbxvu__svg">
      <!-- the bezel, then the glass, both at their measured places -->
      <rect x="1.5" y="1.5" width="532" height="295" rx="5" class="dbxvu__bezel" />
      <rect x="53.9" y="52.9" width="421.1" height="191.8" rx="2" class="dbxvu__glass" />

      <!-- the scale -->
      <path :d="arc(R, A0, zeroDeg)" class="dbxvu__arc" />
      <path :d="arc(R, zeroDeg, A1)" class="dbxvu__arc red" />
      <line v-for="(m, i) in marks" :key="'M' + i" v-bind="tick(m, R, R - 13)" class="dbxvu__tick" :class="{ red: m.value >= 0 }" />
      <line v-for="(m, i) in minor" :key="'m' + i" v-bind="tick(m, R, R - 8)" class="dbxvu__tick minor" :class="{ red: m.value >= 0 }" />
      <text
        v-for="(m, i) in marks"
        :key="'L' + i"
        v-bind="label(m, R + 15)"
        class="dbxvu__num"
        :class="{ red: m.value >= 0 }"
        text-anchor="middle"
      >{{ m.value > 0 ? `+${m.value}` : m.value }}</text>

      <!-- the percentage row, the same scale read in volts -->
      <line v-for="(m, i) in pct" :key="'p' + i" v-bind="tick(m, R - 24, R - 31)" class="dbxvu__tick pct" />
      <text v-for="(m, i) in pct" :key="'P' + i" v-bind="label(m, R - 40)" class="dbxvu__pct" text-anchor="middle">
        {{ PCT_LABEL[String(m.value)] }}
      </text>

      <text x="267.5" y="74" class="dbxvu__top" text-anchor="middle">VU</text>
      <text x="267.5" y="228" class="dbxvu__legend" text-anchor="middle">{{ legend }}</text>

      <!-- the needle, and the zero-set screw beneath the face -->
      <line x1="267.5" y1="363" :x2="needleEnd[0]" :y2="needleEnd[1]" class="dbxvu__needle" />
      <circle cx="267.5" cy="271" r="8" class="dbxvu__screw" />
      <line x1="261.5" y1="271" x2="273.5" y2="271" class="dbxvu__slot" />
    </svg>
  </div>
</template>

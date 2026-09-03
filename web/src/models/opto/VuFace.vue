<script setup>
/**
 * The panel's VU meter, drawn as an SVG face in this plug-in's own style
 * after the reissue's meter: a dark grey bezel with a bevelled inner edge,
 * a cream face with "VU LEVEL INDICATOR" printed above the scale, the
 * black scale with a red section past 0 and the VU numbers outside the
 * arc, the 0..100 percent row inside it, "VU" printed at both ends of the arc,
 * the maker's name and the mode legend below the arc, and a black needle
 * from a pivot below the face. The framework's `useNeedle` owns the
 * ballistics and the voltage-proportional scale maths (300 ms to 99 %).
 *
 * The needle follows element 5 of the `meter` stream, `meter_vu`: the
 * negated gain reduction in GR mode (the needle rests at 0 and swings left,
 * like the original) or the output level in dB relative to 0 VU in the
 * +10 / +4 modes. Which legend is printed under the scale follows the
 * `meter` parameter.
 *
 * Props: `width` (px); the bezel is 395 : 230 like the photograph.
 */
import { computed } from 'vue';
import { useNeedle } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';
import { useOpto } from './useOpto.js';

const props = defineProps({ width: { type: Number, default: 300 } });
const panel = useOpto();
/*
 * `meter[5]` is the needle's position, not the level it is chasing: the
 * standard VU movement runs in the audio thread for every model, at
 * 13 rad/s and damping 0.80, which is 99 % in 300 ms with about 1.5 %
 * overshoot. So this draws the field as it arrives and asks the framework's
 * needle for nothing but a short critically-damped follow, enough to bridge
 * the gap between frames without adding a second set of ballistics on top of
 * the engine's.
 */
const needle = useNeedle('meter', { index: 5, unit: 'db', mode: 'level', min: -20, max: 3, riseMs: 40, damping: 1, sweep: 78 });
const height = computed(() => Math.round(props.width * (230 / 395)));
const marks = needle.marks([-20, -10, -7, -5, -3, -2, -1, 0, 1, 2, 3]);
const minor = needle.marks([-15, -8, -6, -4, -2.5, -1.5, -0.5, 0.5, 1.5, 2.5]);
// The percentage row is the same scale read in volts, so 100 % sits on the
// 0 dB mark and each figure at 20*log10(pct/100).
const pct = needle.marks([-20, -13.98, -7.96, -4.44, -1.94, 0]);
const pctLabel = { '-20': '0', '-13.98': '20', '-7.96': '40', '-4.44': '60', '-1.94': '80', 0: '100' };
// The mode the selector is on is printed on the panel under the meter, not on
// the face: the real face carries only the small line above the scale and the
// maker's script below, and a third line here has nowhere legible to go.
void panel;
// Face geometry in a 395 x 230 viewBox: the face is the 325 x 144 window
// at (35, 43); the pivot sits well below it so the 78-degree sweep draws a
// shallow arc across the face's width, top at y 108, ends at y 158.
const CX = 197.5;
const CY = 333;
const R = 225;
function pt(deg, r) {
  const a = ((deg - 90) * Math.PI) / 180;
  return [CX + r * Math.cos(a), CY + r * Math.sin(a)];
}
function arc(r, a0, a1) {
  const [x0, y0] = pt(a0, r);
  const [x1, y1] = pt(a1, r);
  return `M ${x0.toFixed(1)} ${y0.toFixed(1)} A ${r} ${r} 0 0 1 ${x1.toFixed(1)} ${y1.toFixed(1)}`;
}
const zeroAngle = needle.marks([0])[0].angle;
const blackArc = arc(R, -39, zeroAngle);
const redArc = arc(R, zeroAngle, 39);
</script>

<template>
  <svg viewBox="0 0 395 230" :width="width" :height="height" class="vu">
    <defs>
      <linearGradient id="vuBezel" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0" stop-color="#5a5652" />
        <stop offset="1" stop-color="#2f2c29" />
      </linearGradient>
      <linearGradient id="vuGlass" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0" stop-color="#fff" stop-opacity="0.2" />
        <stop offset="0.5" stop-color="#fff" stop-opacity="0.02" />
        <stop offset="1" stop-color="#000" stop-opacity="0.12" />
      </linearGradient>
      <clipPath id="vuFaceClip">
        <rect x="35" y="43" width="325" height="144" />
      </clipPath>
    </defs>
    <rect x="0" y="0" width="395" height="230" rx="4" fill="url(#vuBezel)" />
    <polygon points="35,43 360,43 395,0 0,0" fill="#4d4945" />
    <polygon points="35,187 360,187 395,230 0,230" fill="#26231f" />
    <polygon points="0,0 35,43 35,187 0,230" fill="#3d3935" />
    <polygon points="395,0 360,43 360,187 395,230" fill="#3d3935" />
    <rect x="35" y="43" width="325" height="144" fill="#ede3c8" />
    <text x="197.5" y="63" text-anchor="middle" class="tiny">VU LEVEL INDICATOR</text>
    <text x="55" y="122" text-anchor="middle" class="vu-label">VU</text>
    <text x="340" y="122" text-anchor="middle" class="vu-label red">VU</text>
    <path :d="blackArc" class="arc" />
    <path :d="redArc" class="arc red" />
    <g class="ticks">
      <line v-for="m in minor" :key="'n' + m.value" :x1="pt(m.angle, R)[0]" :y1="pt(m.angle, R)[1]" :x2="pt(m.angle, R - 6)[0]" :y2="pt(m.angle, R - 6)[1]" />
      <line v-for="m in marks" :key="m.value" :x1="pt(m.angle, R)[0]" :y1="pt(m.angle, R)[1]" :x2="pt(m.angle, R - 11)[0]" :y2="pt(m.angle, R - 11)[1]" class="major" :class="{ red: m.value > 0 }" />
      <text v-for="m in marks" :key="'t' + m.value" :x="pt(m.angle, R + 14)[0]" :y="pt(m.angle, R + 14)[1]" text-anchor="middle" dominant-baseline="middle" :class="{ red: m.value > 0 }">
        {{ m.value > 0 ? '+' + m.value : m.value }}
      </text>
      <text v-for="m in pct" :key="'p' + m.value" :x="pt(m.angle, R - 22)[0]" :y="pt(m.angle, R - 22)[1]" text-anchor="middle" dominant-baseline="middle" class="pct">{{ pctLabel[m.value] }}</text>
    </g>
    <text x="197.5" y="172" text-anchor="middle" class="maker">NOOB COMPRESSOR</text>
    <g clip-path="url(#vuFaceClip)">
      <g :transform="`rotate(${needle.angle.value} ${CX} ${CY})`">
        <line :x1="CX" :y1="CY - 140" :x2="CX" :y2="CY - R - 4" class="needle" />
      </g>
    </g>
    <rect x="35" y="43" width="325" height="144" fill="url(#vuGlass)" pointer-events="none" />
  </svg>
</template>

<style scoped>
.vu {
  display: block;
}
.arc {
  fill: none;
  stroke: #1c1a17;
  stroke-width: 1.6;
}
.arc.red {
  stroke: #c3352c;
  stroke-width: 2.8;
}
.ticks line {
  stroke: #1c1a17;
  stroke-width: 0.9;
}
.ticks line.major {
  stroke-width: 1.5;
}
.ticks line.red {
  stroke: #c3352c;
}
.ticks text {
  font: 700 9.5px 'Inter', sans-serif;
  fill: #1c1a17;
}
.ticks text.red {
  fill: #c3352c;
}
.ticks text.pct {
  font: 600 6px 'Inter', sans-serif;
  fill: #3c3833;
}
.tiny {
  font: 600 9px 'Inter', sans-serif;
  fill: #3c3833;
  letter-spacing: 0.12em;
}
.vu-label {
  font: 900 12px 'Inter', sans-serif;
  fill: #1c1a17;
}
.vu-label.red {
  fill: #c3352c;
}
.maker {
  font: 900 12px 'Inter', sans-serif;
  fill: #1c1a17;
  letter-spacing: 0.06em;
}
.legend {
  font: 600 6px 'Inter', sans-serif;
  fill: #4a453e;
  letter-spacing: 0.18em;
}
.needle {
  stroke: #1a1410;
  stroke-width: 1.8;
  stroke-linecap: round;
}
</style>

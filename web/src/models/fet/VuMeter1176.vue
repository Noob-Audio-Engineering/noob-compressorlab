<script setup>
/**
 * The panel meter: a cream VU face behind glass with a black needle, in
 * SVG. The behaviour (scale mapping, 300 ms second-order ballistics) comes
 * from the framework's `useNeedle`; the look is this plug-in's.
 *
 * The plug-in publishes where the needle should be for the selected meter
 * mode as `meter[5]` (dB relative to the meter's zero: gain reduction in GR
 * mode, VU in +4 / +8, −60 when off), with the VU movement's ballistics
 * already applied in the audio thread, so this component draws that value on
 * a −20..+3 face; in GR mode the needle rests at 0 and swings left. The
 * "Off" mode dims the lamp.
 *
 * The face carries only what the hardware prints on it: the scale, the red
 * arc past zero and "VU". What the selector is pointing at is silkscreened on
 * the panel below the meter by each faceplate, not here, because inside the
 * arc it has nowhere legible to sit.
 *
 * Props:
 * - `mode` (object): the `meter` parameter handle (for the lamp).
 * - `offIndex` (number, default 3): the position that dims the lamp, or −1
 *   for a selector that has no off.
 *
 * Emits: nothing.
 */
import { computed } from 'vue';
import { useNeedle } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';

const props = defineProps({
  mode: { type: Object, required: true },
  offIndex: { type: Number, default: 3 },
});

const SWEEP = 96;
const CX = 150;
const CY = 205;
const R = 165;
/*
 * `meter[5]` is the needle's position, not the level it is chasing: the
 * standard VU movement runs in the audio thread for every model, at
 * 13 rad/s and damping 0.80, which is 99 % in 300 ms with about 1.5 %
 * overshoot. So this draws the field as it arrives and asks the framework's
 * needle for nothing but a short critically-damped follow, enough to bridge
 * the gap between frames without adding a second set of ballistics on top of
 * the engine's.
 */
const needle = useNeedle('meter', { index: 5, unit: 'db', mode: 'level', min: -20, max: 3, riseMs: 40, damping: 1, sweep: SWEEP });
const marks = needle.marks([-20, -10, -7, -5, -3, -2, -1, 0, 1, 2, 3]);
const minor = needle.marks([-15, -8, -6, -4, -2.5, -1.5, -0.5, 0.5, 1.5, 2.5]);
const pt = (deg, r) => {
  const a = (deg * Math.PI) / 180;
  return [CX + r * Math.sin(a), CY - r * Math.cos(a)];
};
const arc = (from, to, r) => {
  const [x1, y1] = pt(from, r);
  const [x2, y2] = pt(to, r);
  return `M ${x1} ${y1} A ${r} ${r} 0 0 1 ${x2} ${y2}`;
};
const zeroAngle = marks.find((m) => m.value === 0).angle;
const off = computed(() => props.mode.index === props.offIndex);
</script>

<template>
  <div class="vu1176" :class="{ off }">
    <svg viewBox="0 0 300 150" class="vu1176__svg">
      <defs>
        <linearGradient id="vuFace" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stop-color="#f6efd9" />
          <stop offset="1" stop-color="#e2d6b4" />
        </linearGradient>
        <linearGradient id="vuGlass" x1="0" y1="0" x2="0.3" y2="1">
          <stop offset="0" stop-color="rgba(255,255,255,0.28)" />
          <stop offset="0.35" stop-color="rgba(255,255,255,0.04)" />
          <stop offset="1" stop-color="rgba(0,0,0,0.08)" />
        </linearGradient>
        <clipPath id="vuClip"><rect x="0" y="0" width="300" height="150" rx="6" /></clipPath>
      </defs>
      <g clip-path="url(#vuClip)">
        <rect x="0" y="0" width="300" height="150" fill="url(#vuFace)" />
        <!-- scale arcs -->
        <path :d="arc(-SWEEP / 2, zeroAngle, R * 0.86)" class="vu1176__arc" />
        <path :d="arc(zeroAngle, SWEEP / 2, R * 0.86)" class="vu1176__arc red" />
        <!-- minor ticks -->
        <line v-for="m in minor" :key="'m' + m.value" :x1="pt(m.angle, R * 0.86)[0]" :y1="pt(m.angle, R * 0.86)[1]" :x2="pt(m.angle, R * 0.82)[0]" :y2="pt(m.angle, R * 0.82)[1]" :class="['vu1176__tick', { red: m.value > 0 }]" />
        <!-- major ticks and numbers -->
        <g v-for="m in marks" :key="'M' + m.value">
          <line :x1="pt(m.angle, R * 0.86)[0]" :y1="pt(m.angle, R * 0.86)[1]" :x2="pt(m.angle, R * 0.79)[0]" :y2="pt(m.angle, R * 0.79)[1]" :class="['vu1176__tick', 'major', { red: m.value > 0 }]" />
          <text :x="pt(m.angle, R * 0.72)[0]" :y="pt(m.angle, R * 0.72)[1]" text-anchor="middle" dominant-baseline="central" :class="['vu1176__num', { red: m.value > 0 }]">{{ m.value > 0 ? '+' + m.value : m.value }}</text>
        </g>
        <text x="150" y="120" text-anchor="middle" class="vu1176__vu">VU</text>
        <!-- needle -->
        <g :transform="`rotate(${needle.angle.value} ${CX} ${CY})`">
          <line :x1="CX" :y1="CY" :x2="CX" :y2="CY - R" class="vu1176__needle" />
        </g>
        <rect x="0" y="0" width="300" height="150" fill="url(#vuGlass)" />
      </g>
    </svg>
  </div>
</template>

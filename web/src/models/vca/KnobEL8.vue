<script setup>
/**
 * One of the four big knobs of the Distressor's front panel: an ivory body
 * with a ringed skirt and a dark pointer notch, the numbers 0 to 10 printed
 * on the black panel around its top. Drawn in SVG; the gestures come from
 * the framework's `useKnobGesture` and the value lives in a `useParam`
 * handle.
 *
 * Props:
 * - `p` (object, required): the parameter handle.
 * - `marks` (array, default []): `[{ value, label }]` printed around the
 *   knob at the position `toRotation(value)` gives.
 * - `toRotation`, `fromRotation` (functions): the dial's taper between the
 *   parameter's plain value and the rotation fraction 0..1 (default linear
 *   over the parameter range).
 * - `size` (number or string, default 120): px, or any CSS length (the
 *   faceplate passes container-query units so the knob scales with it).
 * - `sweep` (number, default 300): degrees between the end stops.
 * - `label` (string): the caption the panel prints above the knob.
 * - `endLabels` (array, default []): `[{ value, label }]` small italic
 *   notes under the skirt, for the Opto(10) and Opto(0) marks the hardware
 *   prints at the ends of Attack and Release.
 *
 * Emits: nothing. Gestures: vertical drag (Shift = fine), wheel,
 * double-click resets, arrow keys / Home / End when focused.
 */
import { computed } from 'vue';
import { useKnobGesture } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';

const props = defineProps({
  p: { type: Object, required: true },
  marks: { type: Array, default: () => [] },
  toRotation: { type: Function, default: null },
  fromRotation: { type: Function, default: null },
  size: { type: [Number, String], default: 120 },
  sweep: { type: Number, default: 300 },
  label: { type: String, default: null },
  endLabels: { type: Array, default: () => [] },
});

const toRotPlain = (v) => (props.toRotation ? props.toRotation(v) : (v - props.p.min) / (props.p.max - props.p.min));
const fromRotPlain = (r) => (props.fromRotation ? props.fromRotation(r) : props.p.min + r * (props.p.max - props.p.min));
const { handlers, dragging } = useKnobGesture(props.p, {
  rotation: { toRotation: (norm) => toRotPlain(props.p.toPlain(norm)), fromRotation: (rot) => props.p.toNorm(fromRotPlain(rot)) },
});

const width = computed(() => (typeof props.size === 'number' ? `${props.size}px` : props.size));
const angle = computed(() => -props.sweep / 2 + toRotPlain(props.p.plain) * props.sweep);
const BODY = 30;
const place = (value, radius) => {
  const a = ((-props.sweep / 2 + toRotPlain(value) * props.sweep) * Math.PI) / 180;
  return { x: 50 + radius * Math.sin(a), y: 50 - radius * Math.cos(a) };
};
const markItems = computed(() => props.marks.map((m) => ({ ...m, ...place(m.value, BODY + 8.5), tick: place(m.value, BODY + 3), tick2: place(m.value, BODY + 5.5) })));
const endItems = computed(() => props.endLabels.map((m) => ({ ...m, ...place(m.value, BODY + 16) })));
const text = computed(() => props.p.plain.toFixed(1));
</script>

<template>
  <div class="knobel8" :style="{ width }">
    <svg viewBox="0 0 100 100" class="knobel8__dial" tabindex="0" role="slider" :aria-label="label || p.name" :aria-valuetext="text" v-on="handlers">
      <defs>
        <radialGradient id="el8Body" cx="38%" cy="30%" r="78%">
          <stop offset="0" stop-color="#fbf8ee" />
          <stop offset="0.55" stop-color="#e6e1d0" />
          <stop offset="1" stop-color="#b8b3a2" />
        </radialGradient>
        <linearGradient id="el8Skirt" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stop-color="#d7d2c2" />
          <stop offset="1" stop-color="#8e8a7c" />
        </linearGradient>
      </defs>
      <!-- printed scale on the panel -->
      <g class="knobel8__marks">
        <line v-for="m in markItems" :key="'t' + m.value" :x1="m.tick.x" :y1="m.tick.y" :x2="m.tick2.x" :y2="m.tick2.y" />
        <text v-for="m in markItems" :key="'l' + m.value" :x="m.x" :y="m.y" text-anchor="middle" dominant-baseline="central">{{ m.label }}</text>
      </g>
      <g class="knobel8__ends">
        <text v-for="m in endItems" :key="m.label" :x="m.x" :y="m.y" text-anchor="middle" dominant-baseline="central">{{ m.label }}</text>
      </g>
      <!-- body: skirt ring, ivory face, the ring the pointer notch sits in -->
      <circle cx="50" cy="50" :r="BODY" fill="url(#el8Skirt)" />
      <circle cx="50" cy="50" :r="BODY - 2" fill="url(#el8Body)" />
      <circle cx="50" cy="50" :r="BODY - 9" class="knobel8__inner" />
      <g :transform="`rotate(${angle} 50 50)`">
        <rect x="48.6" :y="50 - BODY + 1.5" width="2.8" height="8" rx="1.2" class="knobel8__notch" />
      </g>
      <circle cx="50" cy="50" r="3" class="knobel8__pin" />
    </svg>
    <div v-if="dragging" class="knobel8__value">{{ text }}</div>
  </div>
</template>

<script setup>
/**
 * A front-panel knob in the style of the original: the printed marks
 * around a skirt, a black body with a cap (silver on the black-face
 * revisions, black on the blue stripe) and a pointer, and the value under
 * the label. Drawn in SVG; the gestures come from the framework's
 * `useKnobGesture` in rotation space, converting through this dial's taper
 * so the pointer moves at a constant angular rate and the printed marks
 * stay under it; the value lives in a `useParam` handle.
 *
 * Props:
 * - `p` (object, required): the parameter handle.
 * - `marks` (array, default []): `[{ value, label }]` printed around the
 *   skirt at the position `toRotation(value)` gives.
 * - `toRotation`, `fromRotation` (functions): the dial's taper between the
 *   parameter's plain value and the rotation fraction 0..1 (default linear
 *   over the parameter range).
 * - `size` (number or string, default 120): px, or any CSS length (the
 *   faceplate passes container-query units so the dial scales with the
 *   panel).
 * - `body` (number, default 32): body radius in the 100-unit drawing; the
 *   marks sit outside it, so a smaller body puts the numbers further out,
 *   as on the small attack and release knobs of the hardware.
 * - `markSize` (number, default 6.5): mark font size in drawing units.
 * - `label` (string): caption under the knob (default the parameter name).
 * - `sweep` (number, default 270): degrees of rotation between the end stops.
 * - `format` (function): plain → text for the read-out (default `p.text`).
 * - `bare` (boolean, default false): no caption and no read-out except
 *   while dragging (the panel prints its own captions).
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
  body: { type: Number, default: 32 },
  markSize: { type: Number, default: 6.5 },
  label: { type: String, default: null },
  sweep: { type: Number, default: 270 },
  format: { type: Function, default: null },
  bare: { type: Boolean, default: false },
});

// Plain-value taper (the printed marks) → the normalized-space adapter the framework wants.
const toRotPlain = (v) => (props.toRotation ? props.toRotation(v) : (v - props.p.min) / (props.p.max - props.p.min));
const fromRotPlain = (r) => (props.fromRotation ? props.fromRotation(r) : props.p.min + r * (props.p.max - props.p.min));
const { handlers, dragging } = useKnobGesture(props.p, {
  rotation: { toRotation: (norm) => toRotPlain(props.p.toPlain(norm)), fromRotation: (rot) => props.p.toNorm(fromRotPlain(rot)) },
});

const width = computed(() => (typeof props.size === 'number' ? `${props.size}px` : props.size));
const rotation = computed(() => toRotPlain(props.p.plain));
const angle = computed(() => -props.sweep / 2 + rotation.value * props.sweep);
// Skirt geometry from the body radius: ticks just outside the body, numbers outside the ticks.
const geo = computed(() => {
  const r = props.body;
  const gap = r >= 28 ? 3 : 4;
  return { rim: r, face: r - 2, cap: r * 0.7, t1: r + gap, t2: r + gap + 4, text: r + gap + 4 + props.markSize * 0.85 };
});
const markItems = computed(() =>
  props.marks.map((m) => {
    const a = ((-props.sweep / 2 + toRotPlain(m.value) * props.sweep) * Math.PI) / 180;
    const g = geo.value;
    const s = Math.sin(a);
    const c = Math.cos(a);
    return { ...m, x1: 50 + g.t1 * s, y1: 50 - g.t1 * c, x2: 50 + g.t2 * s, y2: 50 - g.t2 * c, tx: 50 + g.text * s, ty: 50 - g.text * c };
  }),
);
const text = computed(() => (props.format ? props.format(props.p.plain) : props.p.text));
</script>

<template>
  <div class="knob1176" :style="{ width }">
    <svg
      viewBox="0 0 100 100"
      class="knob1176__dial"
      tabindex="0"
      role="slider"
      :aria-label="label || p.name"
      :aria-valuetext="text"
      v-on="handlers"
    >
      <defs>
        <radialGradient id="knobFace" cx="40%" cy="35%" r="70%">
          <stop offset="0" stop-color="#4a4d52" />
          <stop offset="0.6" stop-color="#26282c" />
          <stop offset="1" stop-color="#121316" />
        </radialGradient>
        <radialGradient id="knobCap" cx="40%" cy="35%" r="70%">
          <stop offset="0" stop-color="#3a3d42" />
          <stop offset="1" stop-color="#1a1b1e" />
        </radialGradient>
        <linearGradient id="knobCapSilver" x1="0" y1="0" x2="0.4" y2="1">
          <stop offset="0" stop-color="#f2f3f5" />
          <stop offset="0.5" stop-color="#b9bcc2" />
          <stop offset="1" stop-color="#e6e8eb" />
        </linearGradient>
        <linearGradient id="knobRim" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stop-color="#8a8d93" />
          <stop offset="1" stop-color="#2b2d31" />
        </linearGradient>
      </defs>
      <!-- printed skirt -->
      <g class="knob1176__marks" :style="{ fontSize: markSize + 'px' }">
        <line v-for="m in markItems" :key="'t' + m.value" :x1="m.x1" :y1="m.y1" :x2="m.x2" :y2="m.y2" />
        <text v-for="m in markItems" :key="'l' + m.value" :x="m.tx" :y="m.ty" text-anchor="middle" dominant-baseline="central">{{ m.label }}</text>
      </g>
      <!-- knob body: rim, black face, cap -->
      <circle cx="50" cy="50" :r="geo.rim" fill="url(#knobRim)" />
      <circle cx="50" cy="50" :r="geo.face" fill="url(#knobFace)" />
      <circle cx="50" cy="50" :r="geo.cap" class="knob1176__cap" />
      <g :transform="`rotate(${angle} 50 50)`">
        <path :d="`M 50 ${50 - geo.face + 1} L 47 ${50 - geo.cap - 1} L 53 ${50 - geo.cap - 1} Z`" class="knob1176__tip" />
        <line x1="50" :y1="50 - geo.cap" x2="50" :y2="50 - geo.cap * 0.35" class="knob1176__ptr" stroke-width="2.4" stroke-linecap="round" />
      </g>
      <circle cx="50" cy="50" r="2.5" class="knob1176__pin" />
    </svg>
    <div v-if="!bare" class="knob1176__label">{{ label || p.name }}</div>
    <div v-if="!bare || dragging" class="knob1176__value" :class="{ float: bare }">{{ text }}</div>
  </div>
</template>

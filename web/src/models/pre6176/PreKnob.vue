<script setup>
/**
 * A knob of the 6176's front panel: a glossy black body with a single white
 * pointer line, the marks printed on the black inset panel around it. The
 * same component draws the continuous LEVEL knob and the stepped rotary
 * switches (Gain, the input selector, the shelf steps, Meter, Ratio); a
 * stepped one snaps, and its `marks` are its positions.
 *
 * Props:
 * - `p` (object, required): the parameter handle.
 * - `marks` (array, default []): `[{ value, label }]` printed around the
 *   knob at the position `toRotation(value)` gives. Every mark gets a tick;
 *   only the ones with a non-empty `label` get a numeral, so a switch with
 *   eleven steps can print seven of them and stay readable at the smallest
 *   window the plug-in allows.
 * - `toRotation`, `fromRotation` (functions): the dial's taper between the
 *   parameter's plain value and the rotation fraction 0..1 (default linear
 *   over the parameter range, which is what a stepped switch wants).
 * - `size` (number or string, default 100), `body` (number, default 26):
 *   the body radius in the 100-unit drawing, so a smaller body pushes the
 *   printed marks further out.
 * - `markSize` (number, default 6): mark font size in drawing units.
 * - `sweep` (number, default 280): degrees between the end stops.
 * - `label` (string): the accessible name (the panel prints its own caption).
 * - `dots` (boolean, default false): print a small dot between each pair of
 *   marks, as the LEVEL knob has.
 * - `endLabels` (array, default []): `[{ value, label }]` drawn outside the
 *   scale at those positions, for the Slow and Fast the panel prints at the
 *   ends of each time knob.
 *
 * Emits: nothing. Gestures come from the framework's `useKnobGesture`:
 * vertical drag (Shift = fine), wheel, double-click resets, arrow keys when
 * focused.
 */
import { computed } from 'vue';
import { useKnobGesture } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';

const props = defineProps({
  p: { type: Object, required: true },
  marks: { type: Array, default: () => [] },
  toRotation: { type: Function, default: null },
  fromRotation: { type: Function, default: null },
  size: { type: [Number, String], default: 100 },
  body: { type: Number, default: 26 },
  markSize: { type: Number, default: 6 },
  sweep: { type: Number, default: 280 },
  label: { type: String, default: null },
  dots: { type: Boolean, default: false },
  endLabels: { type: Array, default: () => [] },
});

const span = () => Math.max(1e-6, props.p.max - props.p.min);
const toRotPlain = (v) => (props.toRotation ? props.toRotation(v) : (v - props.p.min) / span());
const fromRotPlain = (r) => (props.fromRotation ? props.fromRotation(r) : props.p.min + r * span());
const { handlers, dragging } = useKnobGesture(props.p, {
  rotation: { toRotation: (norm) => toRotPlain(props.p.toPlain(norm)), fromRotation: (rot) => props.p.toNorm(fromRotPlain(rot)) },
});

const width = computed(() => (typeof props.size === 'number' ? `${props.size}px` : props.size));
const angle = computed(() => -props.sweep / 2 + toRotPlain(props.p.plain) * props.sweep);
const place = (value, radius) => {
  const a = ((-props.sweep / 2 + toRotPlain(value) * props.sweep) * Math.PI) / 180;
  return { x: 50 + radius * Math.sin(a), y: 50 - radius * Math.cos(a) };
};
const markItems = computed(() =>
  props.marks.map((m) => ({
    ...m,
    ...place(m.value, props.body + 4 + props.markSize * 0.9),
    t1: place(m.value, props.body + 1.5),
    t2: place(m.value, props.body + 3.5),
  })),
);
const dotItems = computed(() => {
  if (!props.dots || props.marks.length < 2) return [];
  const out = [];
  for (let i = 1; i < props.marks.length; i++) out.push(place((props.marks[i - 1].value + props.marks[i].value) / 2, props.body + 5));
  return out;
});
const endItems = computed(() => props.endLabels.map((m) => ({ ...m, ...place(m.value, props.body + 6 + props.markSize * 1.9) })));
const text = computed(() => props.p.text);
</script>

<template>
  <div class="preknob" :style="{ width }">
    <svg viewBox="0 0 100 100" class="preknob__dial" tabindex="0" role="slider" :aria-label="label || p.name" :aria-valuetext="text" v-on="handlers">
      <defs>
        <radialGradient id="preBody" cx="36%" cy="28%" r="80%">
          <stop offset="0" stop-color="#4c4e54" />
          <stop offset="0.45" stop-color="#1d1e21" />
          <stop offset="1" stop-color="#000" />
        </radialGradient>
      </defs>
      <g class="preknob__marks" :style="{ fontSize: markSize + 'px' }">
        <line v-for="m in markItems" :key="'t' + m.value" :x1="m.t1.x" :y1="m.t1.y" :x2="m.t2.x" :y2="m.t2.y" class="preknob__tick" />
        <template v-for="m in markItems" :key="'l' + m.value">
          <text v-if="m.label" :x="m.x" :y="m.y" text-anchor="middle" dominant-baseline="central">{{ m.label }}</text>
        </template>
        <circle v-for="(d, i) in dotItems" :key="'d' + i" :cx="d.x" :cy="d.y" r="0.8" class="preknob__dot" />
      </g>
      <g class="preknob__ends">
        <text v-for="m in endItems" :key="m.label" :x="m.x" :y="m.y" text-anchor="middle" dominant-baseline="central">{{ m.label }}</text>
      </g>
      <circle cx="50" cy="50" :r="body" fill="url(#preBody)" />
      <circle cx="50" cy="50" :r="body" class="preknob__rim" />
      <g :transform="`rotate(${angle} 50 50)`">
        <line x1="50" :y1="50 - body + 2" x2="50" :y2="50 - body * 0.28" class="preknob__ptr" />
      </g>
    </svg>
    <div v-if="dragging" class="preknob__value">{{ text }}</div>
  </div>
</template>

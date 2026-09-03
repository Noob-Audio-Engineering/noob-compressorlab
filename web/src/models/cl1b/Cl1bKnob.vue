<script setup>
/**
 * One of the CL-1B's five large knobs: a glossy black body with a faceted
 * polygonal skirt and a domed top, a single white index line running from
 * near the centre down the sloping face, and its scale printed on the blue
 * panel around it rather than on the knob.
 *
 * The dots go where `research/CL-1B.md` section 2.2 measured them on the
 * metal, which is not where a smooth pot law would put them; the parameter
 * runs on its own law underneath. All five knobs sweep 239 degrees.
 *
 * Props: `p` (the handle, required), `marks` (the scale, `{ at, label }`
 * with `at` a fraction of travel), `size` (number or CSS length),
 * `label` (accessible name; the panel prints its own caption).
 * Emits: nothing. Gestures come from the framework's `useKnobGesture`.
 */
import { computed } from 'vue';
import { useKnobGesture } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';
import { SWEEP } from './useCl1b.js';

const props = defineProps({
  p: { type: Object, required: true },
  marks: { type: Array, default: () => [] },
  size: { type: [Number, String], default: 100 },
  label: { type: String, default: null },
});

/** The knob body, as a radius in the 100-unit box; the scale sits outside it. */
const BODY = 26;
/** Ten flats around the skirt, as the hardware has. */
const FLATS = 10;

const { handlers, dragging } = useKnobGesture(props.p);

const width = computed(() => (typeof props.size === 'number' ? `${props.size}px` : props.size));
/*
 * Rotation follows the pot's travel, `p.norm`, not its plain value. Today
 * every one of these parameters is a plain 0..1 travel so the two agree,
 * but the read-out beside the knob wants decibels and seconds, and the way
 * to get those without this page duplicating the pot laws is for the engine
 * to publish each parameter in its real unit with a table carrying the
 * law. The moment it does, `plain` stops being linear in travel while
 * `norm` stays linear, which is what a knob turns by, and the measured
 * scale dots below are travel fractions too. So this reads `norm` and keeps
 * working either way.
 */
const angle = computed(() => -SWEEP / 2 + Math.min(1, Math.max(0, props.p.norm)) * SWEEP);

const place = (at, radius) => {
  const a = ((-SWEEP / 2 + at * SWEEP) * Math.PI) / 180;
  return { x: 50 + radius * Math.sin(a), y: 50 - radius * Math.cos(a) };
};
const dots = computed(() => props.marks.map((m) => ({ ...m, ...place(m.at, BODY + 5) })));
const legends = computed(() => props.marks.map((m) => ({ ...m, ...place(m.at, BODY + 13.5) })));

/** The faceted skirt: a regular polygon, rotated with the knob. */
const skirt = computed(() =>
  Array.from({ length: FLATS }, (_, i) => {
    const a = ((i / FLATS) * 2 * Math.PI) - Math.PI / 2;
    return `${(50 + BODY * Math.cos(a)).toFixed(2)},${(50 + BODY * Math.sin(a)).toFixed(2)}`;
  }).join(' '),
);
</script>

<template>
  <div class="cl1bknob" :style="{ width }">
    <svg viewBox="0 0 100 100" class="cl1bknob__dial" tabindex="0" role="slider" :aria-label="label || p.name" :aria-valuetext="p.text" v-on="handlers">
      <defs>
        <radialGradient :id="`cl1bBody${p.id}`" cx="36%" cy="26%" r="82%">
          <stop offset="0" stop-color="#4a5150" />
          <stop offset="0.45" stop-color="#202726" />
          <stop offset="1" stop-color="#0a0f0e" />
        </radialGradient>
      </defs>

      <!-- the scale, printed on the panel around the knob -->
      <g class="cl1bknob__marks">
        <circle v-for="(d, i) in dots" :key="'d' + i" :cx="d.x" :cy="d.y" r="1.15" />
        <text v-for="(l, i) in legends" :key="'l' + i" :x="l.x" :y="l.y" text-anchor="middle" dominant-baseline="central">{{ l.label }}</text>
      </g>

      <!-- the body: faceted skirt, domed top, one index line -->
      <g :transform="`rotate(${angle} 50 50)`">
        <polygon :points="skirt" class="cl1bknob__skirt" />
        <circle cx="50" cy="50" :r="BODY - 3.5" :fill="`url(#cl1bBody${p.id})`" />
        <circle cx="50" cy="50" :r="BODY - 3.5" class="cl1bknob__rim" />
        <rect x="49.1" :y="50 - BODY + 4" width="1.8" height="12" rx="0.9" class="cl1bknob__index" />
      </g>
    </svg>
    <div v-if="dragging" class="cl1bknob__value">{{ p.text }}</div>
  </div>
</template>

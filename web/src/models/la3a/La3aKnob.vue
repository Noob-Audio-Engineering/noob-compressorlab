<script setup>
/**
 * One of the LA-3A's two knobs: a light cream body with a single pointer
 * notch, and the 0 to 10 scale printed on the black panel around it — 0 at
 * the lower left, 5 at top dead centre, 10 at the lower right, with a small
 * dot between each pair of numerals, over about 300 degrees. The numbers
 * are arbitrary on the hardware and they are arbitrary here.
 *
 * Props: `p` (the handle, required), `size` (number or CSS length, default
 * 100), `label` (accessible name; the panel prints its own caption).
 * Emits: nothing. Gestures come from the framework's `useKnobGesture`.
 */
import { computed } from 'vue';
import { useKnobGesture } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';

const props = defineProps({
  p: { type: Object, required: true },
  size: { type: [Number, String], default: 100 },
  label: { type: String, default: null },
});

const SWEEP = 300;
const BODY = 27;
const { handlers, dragging } = useKnobGesture(props.p);

const width = computed(() => (typeof props.size === 'number' ? `${props.size}px` : props.size));
/** The parameter runs 0..100; the panel prints 0..10. */
const toRot = (v) => (v - props.p.min) / Math.max(1e-6, props.p.max - props.p.min);
const angle = computed(() => -SWEEP / 2 + toRot(props.p.plain) * SWEEP);
const place = (frac, radius) => {
  const a = ((-SWEEP / 2 + frac * SWEEP) * Math.PI) / 180;
  return { x: 50 + radius * Math.sin(a), y: 50 - radius * Math.cos(a) };
};
const numbers = computed(() => Array.from({ length: 11 }, (_, i) => ({ label: String(i), ...place(i / 10, BODY + 8.5) })));
const dots = computed(() => Array.from({ length: 10 }, (_, i) => place((i + 0.5) / 10, BODY + 4.5)));
const text = computed(() => (props.p.plain / 10).toFixed(1));
</script>

<template>
  <div class="la3aknob" :style="{ width }">
    <svg viewBox="0 0 100 100" class="la3aknob__dial" tabindex="0" role="slider" :aria-label="label || p.name" :aria-valuetext="text" v-on="handlers">
      <defs>
        <radialGradient id="la3aBody" cx="38%" cy="30%" r="78%">
          <stop offset="0" stop-color="#fdfcf6" />
          <stop offset="0.6" stop-color="#eae5d6" />
          <stop offset="1" stop-color="#bab5a6" />
        </radialGradient>
      </defs>
      <g class="la3aknob__marks">
        <text v-for="n in numbers" :key="n.label" :x="n.x" :y="n.y" text-anchor="middle" dominant-baseline="central">{{ n.label }}</text>
        <circle v-for="(d, i) in dots" :key="'d' + i" :cx="d.x" :cy="d.y" r="0.7" />
      </g>
      <circle cx="50" cy="50" :r="BODY" fill="url(#la3aBody)" />
      <circle cx="50" cy="50" :r="BODY" class="la3aknob__rim" />
      <g :transform="`rotate(${angle} 50 50)`">
        <rect x="48.8" :y="50 - BODY + 1" width="2.4" height="7.5" rx="1" class="la3aknob__notch" />
      </g>
    </svg>
    <div v-if="dragging" class="la3aknob__value">{{ text }}</div>
  </div>
</template>

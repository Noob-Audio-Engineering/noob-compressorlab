<script setup>
/**
 * One of the two large panel knobs (Gain, Peak Reduction): a printed 0..100
 * skirt around a black bakelite body with a white pointer line, the way
 * the reissue's knobs look, bound to a handle through the framework's
 * `useKnobGesture` (drag, wheel, double-click to reset, keys). The
 * caption is printed on the panel, not here. Purely this plug-in's look.
 *
 * Proportions from the photograph: body diameter two thirds of the skirt's
 * number circle; the scale sweeps 300 degrees from 0 at seven o'clock to
 * 100 at five o'clock.
 *
 * Props:
 * - `p` (handle, required): a 0..100 parameter.
 * - `label` (string): accessible name.
 * - `size` (number, default 190): outer diameter in px.
 * - `marks` (number[]): printed scale values (default 0, 10, ... 100).
 */
import { computed } from 'vue';
import { useKnobGesture } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';

const props = defineProps({
  p: { type: Object, required: true },
  label: { type: String, default: '' },
  size: { type: Number, default: 190 },
  marks: { type: Array, default: () => [0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100] },
});

const SWEEP = 300;
const { handlers, dragging } = useKnobGesture(props.p, { sensitivity: 260 });
const angle = computed(() => -SWEEP / 2 + SWEEP * props.p.norm);
const ticks = computed(() =>
  Array.from({ length: 51 }, (_, i) => {
    const a = -SWEEP / 2 + (SWEEP * i) / 50;
    return { a, major: i % 5 === 0 };
  }),
);
function polar(r, deg) {
  const a = ((deg - 90) * Math.PI) / 180;
  return [100 + r * Math.cos(a), 100 + r * Math.sin(a)];
}
const markPos = computed(() =>
  props.marks.map((m) => {
    const a = -SWEEP / 2 + (SWEEP * (m - props.p.min)) / (props.p.max - props.p.min);
    const [x, y] = polar(86, a);
    return { m, x, y, a };
  }),
);
</script>

<template>
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 200 200"
    class="big-knob"
    :class="{ dragging }"
    tabindex="0"
    role="slider"
    :aria-valuemin="p.min"
    :aria-valuemax="p.max"
    :aria-valuenow="p.plain"
    :aria-label="label"
    :title="`${label} ${p.plain.toFixed(0)}`"
    v-on="handlers"
  >
    <g class="scale">
      <line v-for="t in ticks" :key="t.a" :x1="polar(t.major ? 64 : 67, t.a)[0]" :y1="polar(t.major ? 64 : 67, t.a)[1]" :x2="polar(72, t.a)[0]" :y2="polar(72, t.a)[1]" :stroke-width="t.major ? 1.8 : 0.9" />
      <text v-for="m in markPos" :key="m.m" :x="m.x" :y="m.y" text-anchor="middle" dominant-baseline="middle" :transform="`rotate(${m.a} ${m.x} ${m.y})`">{{ m.m }}</text>
    </g>
    <circle cx="100" cy="103" r="55" class="shadow" />
    <circle cx="100" cy="100" r="54" class="skirt" />
    <circle cx="100" cy="100" r="50" class="body" />
    <g :transform="`rotate(${angle} 100 100)`">
      <rect x="97.5" y="52" width="5" height="34" rx="2.5" class="pointer" />
    </g>
    <circle cx="100" cy="100" r="16" class="cap" />
  </svg>
</template>

<style scoped>
.big-knob {
  cursor: ns-resize;
  outline: none;
  display: block;
}
.scale line {
  stroke: #1b1a17;
}
.scale text {
  font: 700 11.5px 'Inter', sans-serif;
  fill: #1b1a17;
}
.shadow {
  fill: rgba(0, 0, 0, 0.4);
}
.skirt {
  fill: #2b2724;
  stroke: #0e0d0b;
  stroke-width: 1;
}
.body {
  fill: #1c1917;
  stroke: #3d3733;
  stroke-width: 1.5;
}
.pointer {
  fill: #f4efe4;
}
.cap {
  fill: #0d0c0b;
}
.big-knob.dragging .body {
  stroke: #7cc6ff;
}
</style>

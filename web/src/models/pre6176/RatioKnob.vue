<script setup>
/**
 * The 6176's RATIO rotary. On the hardware this is a knob like its
 * neighbours, marked BP, 1, 4, 8, 12, 20 and ALL, and it is the one control
 * on this face that drives two parameters: BP and 1 are routing settings
 * (`pre_join`), and 4 through ALL are the limiter's own ratio
 * (`fet_ratio`). That is why it has its own component rather than using
 * `PreKnob`, which binds to a single handle.
 *
 * Props: `size` (number or CSS length), `body`, `markSize`, `sweep`.
 * Emits: nothing. Gestures: vertical drag, wheel, click to step, arrow keys
 * and Home / End when focused.
 */
import { computed, ref } from 'vue';
import { RATIO_POSITIONS, ratioIndex, useControls } from './usePre.js';

const props = defineProps({
  size: { type: [Number, String], default: 100 },
  body: { type: Number, default: 20 },
  markSize: { type: Number, default: 13 },
  sweep: { type: Number, default: 300 },
});

const c = useControls();
const dragging = ref(false);
const LAST = RATIO_POSITIONS.length - 1;
const index = computed(() => ratioIndex(c.join.index, c.ratio.index));

/** Write whichever of the two parameters the position belongs to. */
function set(i) {
  const next = Math.min(LAST, Math.max(0, Math.round(i)));
  if (next === index.value) return;
  const p = RATIO_POSITIONS[next];
  c.join.begin();
  c.join.setIndex(p.join);
  c.join.end();
  if (p.ratio != null) {
    c.ratio.begin();
    c.ratio.setIndex(p.ratio);
    c.ratio.end();
  }
}

const width = computed(() => (typeof props.size === 'number' ? `${props.size}px` : props.size));
const angle = computed(() => -props.sweep / 2 + (index.value / LAST) * props.sweep);
const place = (i, radius) => {
  const a = ((-props.sweep / 2 + (i / LAST) * props.sweep) * Math.PI) / 180;
  return { x: 50 + radius * Math.sin(a), y: 50 - radius * Math.cos(a) };
};
const markItems = computed(() =>
  RATIO_POSITIONS.map((r, i) => ({
    ...r,
    i,
    ...place(i, props.body + 4 + props.markSize * 0.9),
    t1: place(i, props.body + 1.5),
    t2: place(i, props.body + 3.5),
  })),
);

/** A drag of about 200 px covers the whole travel, as the framework's knobs do. */
let from = 0;
let start = 0;
function onPointerdown(e) {
  dragging.value = true;
  from = e.clientY;
  start = index.value;
  e.currentTarget.setPointerCapture?.(e.pointerId);
  e.preventDefault();
}
function onPointermove(e) {
  if (!dragging.value) return;
  set(start + ((from - e.clientY) / 200) * LAST);
}
function onPointerup(e) {
  if (!dragging.value) return;
  dragging.value = false;
  e.currentTarget.releasePointerCapture?.(e.pointerId);
}
function onWheel(e) {
  e.preventDefault();
  set(index.value + (e.deltaY < 0 ? 1 : -1));
}
function onKeydown(e) {
  const k = e.key;
  if (k === 'ArrowUp' || k === 'ArrowRight') set(index.value + 1);
  else if (k === 'ArrowDown' || k === 'ArrowLeft') set(index.value - 1);
  else if (k === 'Home') set(0);
  else if (k === 'End') set(LAST);
  else return;
  e.preventDefault();
}
</script>

<template>
  <div class="preknob" :style="{ width }">
    <svg
      viewBox="0 0 100 100"
      class="preknob__dial"
      tabindex="0"
      role="slider"
      aria-label="Ratio"
      :aria-valuetext="RATIO_POSITIONS[index].label"
      :title="RATIO_POSITIONS[index].hint"
      @pointerdown="onPointerdown"
      @pointermove="onPointermove"
      @pointerup="onPointerup"
      @pointercancel="onPointerup"
      @wheel="onWheel"
      @keydown="onKeydown"
    >
      <defs>
        <radialGradient id="ratioBody" cx="36%" cy="28%" r="80%">
          <stop offset="0" stop-color="#4c4e54" />
          <stop offset="0.45" stop-color="#1d1e21" />
          <stop offset="1" stop-color="#000" />
        </radialGradient>
      </defs>
      <g class="preknob__marks" :style="{ fontSize: markSize + 'px' }">
        <line v-for="m in markItems" :key="'t' + m.i" :x1="m.t1.x" :y1="m.t1.y" :x2="m.t2.x" :y2="m.t2.y" class="preknob__tick" />
        <text v-for="m in markItems" :key="'l' + m.i" :x="m.x" :y="m.y" text-anchor="middle" dominant-baseline="central">{{ m.label }}</text>
      </g>
      <circle cx="50" cy="50" :r="body" fill="url(#ratioBody)" />
      <circle cx="50" cy="50" :r="body" class="preknob__rim" />
      <g :transform="`rotate(${angle} 50 50)`">
        <line x1="50" :y1="50 - body + 2" x2="50" :y2="50 - body * 0.28" class="preknob__ptr" />
      </g>
    </svg>
    <div v-if="dragging" class="preknob__value">{{ RATIO_POSITIONS[index].label }}</div>
  </div>
</template>

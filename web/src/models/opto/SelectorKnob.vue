<script setup>
/**
 * The meter selector: a small black rotary switch with a white pointer
 * line and three position marks on the panel, bound to the discrete
 * `meter` handle. The handle's steps are Gain Reduction, Output +10 and
 * Output +4; the pointer angles put each position under the caption
 * printed for it on the faceplate (+10 up-left, Gain Reduction straight
 * up, +4 up-right). Drag or wheel through the framework's
 * `useKnobGesture`; a click steps to the next position.
 *
 * Props: `p` (handle, required), `size` (px, the mark circle's diameter).
 */
import { computed } from 'vue';
import { useKnobGesture } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';

const props = defineProps({ p: { type: Object, required: true }, size: { type: Number, default: 60 } });
const { handlers, dragging } = useKnobGesture(props.p, { sensitivity: 120 });
/** Pointer angle per handle index: GR, +10, +4. */
const ANGLES = [0, -47, 51];
const angle = computed(() => ANGLES[props.p.index] ?? 0);
function pt(deg, r) {
  const a = ((deg - 90) * Math.PI) / 180;
  return [50 + r * Math.cos(a), 50 + r * Math.sin(a)];
}
function next(e) {
  if (e.detail && e.detail > 1) return;
  props.p.begin();
  props.p.setIndex((props.p.index + 1) % 3);
  props.p.end();
}
</script>

<template>
  <svg :width="size" :height="size" viewBox="0 0 100 100" class="selector" :class="{ dragging }" tabindex="0" role="slider" aria-valuemin="0" aria-valuemax="2" :aria-valuenow="p.index" aria-label="Meter" v-on="handlers" @click="next">
    <circle v-for="a in ANGLES" :key="a" :cx="pt(a, 47)[0]" :cy="pt(a, 47)[1]" r="2.6" class="mark" />
    <circle cx="50" cy="52" r="37" class="shadow" />
    <circle cx="50" cy="50" r="36" class="body" />
    <circle cx="50" cy="50" r="24" class="top" />
    <g :transform="`rotate(${angle} 50 50)`">
      <rect x="46.5" y="14" width="7" height="34" rx="3.5" class="pointer" />
    </g>
  </svg>
</template>

<style scoped>
.selector {
  cursor: pointer;
  outline: none;
  display: block;
}
.mark {
  fill: #2a2724;
}
.shadow {
  fill: rgba(0, 0, 0, 0.35);
}
.body {
  fill: #1a1715;
  stroke: #3a3531;
  stroke-width: 1.5;
}
.top {
  fill: #26221f;
}
.pointer {
  fill: #f2eee6;
}
.selector.dragging .body {
  stroke: #7cc6ff;
}
</style>

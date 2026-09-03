<script setup>
/**
 * A bat-handle toggle switch: a chrome bushing with a lever that points up
 * for "on" and down for "off", like the original's LIMIT / COMPRESS and
 * POWER switches, bound to a toggle handle. `inverted` flips the mapping
 * (the power switch is up when `bypass` is off). Clicking sends a full
 * gesture.
 *
 * Props: `p` (handle, required), `size` (px, the bushing diameter),
 * `inverted` (boolean). The drawing is centred on the bushing; the lever
 * reaches 0.9 × `size` above or below it.
 */
import { computed } from 'vue';

const props = defineProps({
  p: { type: Object, required: true },
  size: { type: Number, default: 30 },
  inverted: { type: Boolean, default: false },
});
const up = computed(() => (props.inverted ? !props.p.on : props.p.on));
function flip() {
  props.p.begin();
  props.p.setOn(!props.p.on);
  props.p.end();
}
</script>

<template>
  <svg :width="size * 2" :height="size * 3" viewBox="0 0 80 120" class="lever" :style="{ margin: `${-size}px 0 0 ${-size / 2}px` }" role="switch" :aria-checked="up" tabindex="0" @click="flip" @keydown.enter.space.prevent="flip">
    <ellipse cx="40" cy="62" rx="20" ry="8" class="ring-shadow" />
    <circle cx="40" cy="60" r="19" class="bushing" />
    <circle cx="40" cy="60" r="10" class="collar" />
    <g :transform="`rotate(${up ? -12 : 192} 40 60)`">
      <rect x="35" y="14" width="10" height="48" rx="5" class="handle" />
      <circle cx="40" cy="16" r="6.5" class="tip" />
    </g>
  </svg>
</template>

<style scoped>
.lever {
  cursor: pointer;
  outline: none;
  display: block;
  overflow: visible;
}
.ring-shadow {
  fill: rgba(0, 0, 0, 0.28);
}
.bushing {
  fill: #9a948a;
  stroke: #3b3732;
  stroke-width: 1.2;
}
.collar {
  fill: #4a4540;
}
.handle {
  fill: #dedad2;
  stroke: #6b665c;
  stroke-width: 0.8;
}
.tip {
  fill: #f2eee6;
  stroke: #6b665c;
  stroke-width: 0.8;
}
</style>

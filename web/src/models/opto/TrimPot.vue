<script setup>
/**
 * One of the LA-2A's two front-panel screwdriver trims: R37, the limit
 * response pre-emphasis in its hex bushing beside the Limit / Compress
 * toggle, and the meter zero below the meter. Both are real controls on the
 * hardware, so both are live here rather than decoration; each is a slotted
 * head in a hex nut, and the slot turns with the value.
 *
 * Props: `p` (the parameter handle, required), `size` (px), `hex` (boolean,
 * default true: draw the bushing around it). Emits: nothing. Gestures:
 * vertical drag, wheel, double-click resets, arrow keys when focused.
 */
import { computed } from 'vue';
import { useKnobGesture } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';

const props = defineProps({
  p: { type: Object, required: true },
  size: { type: Number, default: 24 },
  hex: { type: Boolean, default: true },
});

const { handlers, dragging } = useKnobGesture(props.p, { sensitivity: 160 });
/** A screwdriver slot turns through about 270 degrees over the range. */
const angle = computed(() => -135 + ((props.p.plain - props.p.min) / Math.max(1e-6, props.p.max - props.p.min)) * 270);
</script>

<template>
  <div class="trimpot" :style="{ width: `${size}px`, height: `${size}px` }">
    <svg viewBox="0 0 100 100" tabindex="0" role="slider" :aria-label="p.name" :aria-valuetext="p.text" v-on="handlers">
      <defs>
        <radialGradient id="trimHead" cx="38%" cy="32%" r="72%">
          <stop offset="0" stop-color="#dcdfe4" />
          <stop offset="0.6" stop-color="#a9adb4" />
          <stop offset="1" stop-color="#6e7278" />
        </radialGradient>
      </defs>
      <polygon v-if="hex" points="50,4 90,27 90,73 50,96 10,73 10,27" class="trimpot__hex" />
      <circle cx="50" cy="50" r="27" fill="url(#trimHead)" />
      <circle cx="50" cy="50" r="27" class="trimpot__rim" />
      <g :transform="`rotate(${angle} 50 50)`"><rect x="46" y="28" width="8" height="44" rx="2" class="trimpot__slot" /></g>
    </svg>
    <div v-if="dragging" class="trimpot__value">{{ p.text }}</div>
  </div>
</template>

<script setup>
/**
 * A screwdriver adjustment: a slotted brass head in a small recess, with its
 * legend engraved above it on the panel.
 *
 * **These are live, and that is the point.** The 670 has four of them and
 * the dossier calls the position of one — the DC THRESHOLD, inside the
 * chassis — "the single most consequential fact" about the control list,
 * because it is the ratio and knee control and every emulation that is any
 * good brings it out. The other two on the front panel are ZERO, which is a
 * bias trim wearing a meter-calibration label, and BAL, which sets how much
 * of the push-pull's even-harmonic cancellation survives.
 *
 * A screwdriver slot turns; so does this. The gesture is the framework's
 * knob gesture in rotation space, so the slot stays under the pointer.
 *
 * Props: `p` (the handle, required), `caption` (engraved above), `size`,
 * `sweep` (degrees of slot travel), `inside` (drawn recessed and dimmer,
 * for the trimmer the hardware puts inside the chassis).
 * Emits: nothing.
 */
import { computed } from 'vue';
import { useKnobGesture } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';

const props = defineProps({
  p: { type: Object, required: true },
  caption: { type: String, default: '' },
  size: { type: [Number, String], default: 100 },
  sweep: { type: Number, default: 260 },
  inside: { type: Boolean, default: false },
});

const { handlers, dragging } = useKnobGesture(props.p, { rotation: true });
const width = computed(() => (typeof props.size === 'number' ? `${props.size}px` : props.size));
const angle = computed(() => -props.sweep / 2 + Math.min(1, Math.max(0, props.p.norm)) * props.sweep);
</script>

<template>
  <div class="fairscrew" :class="{ inside }" :style="{ width }">
    <div v-if="caption" class="fairscrew__caption">{{ caption }}</div>
    <svg
      viewBox="0 0 100 100"
      class="fairscrew__head"
      tabindex="0"
      role="slider"
      :aria-label="p.name"
      :aria-valuetext="p.text"
      v-on="handlers"
    >
      <circle cx="50" cy="50" r="44" class="fairscrew__recess" />
      <circle cx="50" cy="50" r="34" class="fairscrew__body" />
      <g :transform="`rotate(${angle} 50 50)`">
        <rect x="18" y="45" width="64" height="10" rx="2" class="fairscrew__slot" />
      </g>
      <circle cx="50" cy="50" r="34" class="fairscrew__rim" />
    </svg>
    <div v-if="dragging" class="fairscrew__value">{{ p.text }}</div>
  </div>
</template>

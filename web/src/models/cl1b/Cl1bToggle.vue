<script setup>
/**
 * The IN switch: a small black bat-handle toggle on a chrome hex bushing,
 * with IN silkscreened above it, at the far left of the bottom row under
 * the wordmark.
 *
 * This is the hardware's bypass. The manual is plain about it: the lever
 * switches the compressor in and out of the signal path, and the out
 * position bypasses the entire compressor. So it drives the lab's shared
 * `bypass` parameter rather than a `cl1b_` id of its own, and it reads
 * inverted, because IN means bypass is off.
 *
 * Props: `p` (the bypass handle, required), `legend` (default IN).
 * Emits: nothing.
 */
import { computed } from 'vue';

const props = defineProps({
  p: { type: Object, required: true },
  legend: { type: String, default: 'IN' },
});

/** True when the compressor is in the path, which is bypass off. */
const isIn = computed(() => !props.p.on);

function flip() {
  props.p.begin();
  props.p.setOn(isIn.value);
  props.p.end();
}
</script>

<template>
  <div class="cl1btog">
    <span class="cl1btog__leg">{{ legend }}</span>
    <button
      class="cl1btog__body"
      type="button"
      role="switch"
      :aria-checked="isIn"
      :aria-label="`${legend} (compressor in the signal path)`"
      @click="flip"
    >
      <span class="cl1btog__bat" :class="isIn ? 'up' : 'down'"></span>
      <span class="cl1btog__bush"></span>
    </button>
  </div>
</template>

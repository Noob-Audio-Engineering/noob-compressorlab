<script setup>
/**
 * One of the LA-3A's two small bat toggles, with its legends silkscreened
 * to the left and the right of the bat rather than above and below: the
 * meter switch (GR / OUTPUT) and the power switch (POWER / ON). Two
 * positions only, as the hardware has.
 *
 * Props: `p` (the handle, required), `left` / `right` (the legends),
 * `invert` (boolean, default false) for a switch whose right position means
 * the parameter is off, which is how the power switch sits over bypass.
 * Emits: nothing.
 */
import { computed } from 'vue';

const props = defineProps({
  p: { type: Object, required: true },
  left: { type: String, default: '' },
  right: { type: String, default: '' },
  invert: { type: Boolean, default: false },
});

/** 0 is the left position, 1 the right. */
const index = computed(() => {
  const raw = props.p.isToggle ? (props.p.on ? 1 : 0) : Math.min(1, props.p.index);
  return props.invert ? 1 - raw : raw;
});
function flip() {
  const next = index.value === 1 ? 0 : 1;
  const raw = props.invert ? 1 - next : next;
  props.p.begin();
  if (props.p.isToggle) props.p.setOn(raw === 1);
  else props.p.setIndex(raw);
  props.p.end();
}
</script>

<template>
  <div class="la3atog">
    <span class="la3atog__leg">{{ left }}</span>
    <button class="la3atog__body" type="button" role="switch" :aria-checked="index === 1" :aria-label="p.name" :aria-valuetext="p.text" @click="flip">
      <span class="la3atog__bat" :class="`at${index}`"></span>
    </button>
    <span class="la3atog__leg">{{ right }}</span>
  </div>
</template>

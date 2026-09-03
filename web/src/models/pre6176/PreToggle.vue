<script setup>
/**
 * One of the small bat toggles on the 6176's panels: two positions for the
 * pad, the polarity and the low cut, three for the shelf frequencies. The
 * bat sits at the top, middle or bottom, the panel prints the legends
 * around it, and clicking (or Space / Enter, or the arrow keys) steps it.
 *
 * Props: `p` (the handle, required), `positions` (2 or 3, default 2),
 * `up` (string) / `mid` (string) / `down` (string): the legends the panel
 * prints beside the bat, and `invert` (boolean, default false) for a
 * two-position switch whose up position means the parameter is off — the
 * power switch, which is up when the plug-in is not bypassed.
 * Emits: nothing.
 *
 * The parameter's index is the position, counting from the bottom, so a
 * toggle parameter's "on" is up unless `invert` says otherwise.
 */
import { computed } from 'vue';

const props = defineProps({
  p: { type: Object, required: true },
  positions: { type: Number, default: 2 },
  up: { type: String, default: '' },
  mid: { type: String, default: '' },
  down: { type: String, default: '' },
  invert: { type: Boolean, default: false },
});

const index = computed(() => {
  if (!props.p.isToggle) return props.p.index;
  return props.p.on !== props.invert ? 1 : 0;
});
const set = (i) => {
  const n = props.positions;
  const next = ((i % n) + n) % n;
  props.p.begin();
  if (props.p.isToggle) props.p.setOn(props.invert ? next === 0 : next === 1);
  else props.p.setIndex(next);
  props.p.end();
};
const step = () => set(index.value + 1);
</script>

<template>
  <div class="pretog" :class="`pos${positions}`">
    <span v-if="up" class="pretog__leg up">{{ up }}</span>
    <button class="pretog__body" type="button" :aria-label="p.name" :aria-valuetext="p.text" @click="step" @keydown.up.prevent="set(index + 1)" @keydown.down.prevent="set(index - 1)">
      <span class="pretog__bat" :class="`at${index}`"></span>
    </button>
    <span v-if="mid" class="pretog__leg mid">{{ mid }}</span>
    <span v-if="down" class="pretog__leg down">{{ down }}</span>
  </div>
</template>

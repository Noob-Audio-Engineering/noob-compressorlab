<script setup>
/**
 * The RATIO column: four push buttons, top to bottom 20, 12, 8, 4 as on
 * the panel, the values printed beside them, and the "all buttons in"
 * trick: Shift-click (or Alt-click) any button to push all four in;
 * clicking a single button again releases the others. Bound to the
 * `ratio` handle (0 = 4:1 … 3 = 20:1, 4 = All).
 *
 * Props: `p` (object, required): the ratio handle.
 * Emits: nothing; clicks send full gestures.
 */
const props = defineProps({ p: { type: Object, required: true } });
const buttons = [
  { index: 3, label: '20' },
  { index: 2, label: '12' },
  { index: 1, label: '8' },
  { index: 0, label: '4' },
];
function pressed(i) {
  return props.p.index === i || props.p.index === 4;
}
function push(i, e) {
  props.p.begin();
  props.p.setIndex(e.shiftKey || e.altKey ? 4 : i);
  props.p.end();
}
</script>

<template>
  <div class="column1176">
    <div v-for="b in buttons" :key="b.index" class="column1176__row left">
      <span class="column1176__label">{{ b.label }}</span>
      <button type="button" class="push1176" :class="{ in: pressed(b.index) }" :aria-pressed="pressed(b.index)" :title="`${b.label}:1 (Shift-click: all buttons in)`" @click="push(b.index, $event)"></button>
    </div>
  </div>
</template>

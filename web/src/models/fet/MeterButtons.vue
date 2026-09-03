<script setup>
/**
 * The METER column: four push buttons, top to bottom GR, +8, +4, OFF as
 * printed on the original (OFF has the red cap), with the values printed
 * beside the buttons. Bound to the `meter` handle (0 GR, 1 +4, 2 +8, 3 Off).
 *
 * Props: `p` (object, required): the meter handle. Emits: nothing.
 */
const props = defineProps({ p: { type: Object, required: true } });
const buttons = [
  { index: 0, label: 'GR' },
  { index: 2, label: '+8' },
  { index: 1, label: '+4' },
  { index: 3, label: 'OFF', red: true },
];
function push(i) {
  props.p.begin();
  props.p.setIndex(i);
  props.p.end();
}
</script>

<template>
  <div class="column1176">
    <div v-for="b in buttons" :key="b.index" class="column1176__row right">
      <button type="button" class="push1176" :class="{ in: p.index === b.index, red: b.red }" :aria-pressed="p.index === b.index" :title="`Meter: ${b.label}`" @click="push(b.index)"></button>
      <span class="column1176__label">{{ b.label }}</span>
    </div>
  </div>
</template>

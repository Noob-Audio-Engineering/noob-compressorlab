<script setup>
/**
 * One of the two small bat toggles the EL8-X adds between the big knobs:
 * Stereo Image Link above Attack, British Mode between Attack and Release.
 * Up is on, and a lamp beside the bat says so. Bound to a toggle parameter
 * handle; clicking (or Space / Enter) flips it.
 *
 * Props: `p` (the handle, required), `lamp` ('red' | 'amber', default
 * 'amber'). Emits: nothing.
 */
const props = defineProps({
  p: { type: Object, required: true },
  lamp: { type: String, default: 'amber' },
});

function flip() {
  props.p.begin();
  props.p.setOn(!props.p.on);
  props.p.end();
}
</script>

<template>
  <button class="el8tog" :class="{ on: p.on }" type="button" role="switch" :aria-checked="p.on" :aria-label="p.name" @click="flip">
    <span class="el8tog__body"><span class="el8tog__bat"></span></span>
    <span class="el8lamp" :class="[lamp, { on: p.on }]"></span>
  </button>
</template>

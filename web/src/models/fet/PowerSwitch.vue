<script setup>
/**
 * The small power toggle under the badge at the right of the panel. It is
 * the inverse of the `bypass` parameter: up (power on) means the
 * compressor is in the signal path; down bypasses it.
 *
 * Props: `p` (object, required): the bypass handle.
 * Emits: nothing.
 */
const props = defineProps({ p: { type: Object, required: true } });
function flip() {
  props.p.begin();
  props.p.setOn(!props.p.on);
  props.p.end();
}
</script>

<template>
  <div class="power1176">
    <button type="button" class="power1176__lever" :class="{ on: !p.on }" role="switch" :aria-checked="!p.on" :title="p.on ? 'Bypassed: click to power on' : 'Powered: click to bypass'" @click="flip">
      <span class="power1176__bat"></span>
    </button>
    <div class="power1176__caption">{{ p.on ? 'OFF' : 'POWER' }}</div>
  </div>
</template>

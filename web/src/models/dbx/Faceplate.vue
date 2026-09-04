<script setup>
/**
 * Which dbx face is drawn. `dbx_model` chooses, and the two are different
 * enough that they are separate components rather than one with branches:
 * the original is a wood-cheeked half-rack with three plain knobs, a VU
 * movement and two indicators, and the 160A is a black 1U panel with three
 * coloured knobs, four buttons, three indicators and thirty-one LEDs.
 *
 * Both drive the same parameters, because both units are the same three
 * controls; `useDbx.js` explains how one parameter serves two pots whose
 * ranges differ.
 */
import { computed } from 'vue';
import { useUnit } from './useDbx.js';
import Faceplate160 from './Faceplate160.vue';
import Faceplate160a from './Faceplate160a.vue';

const unit = useUnit();
const face = computed(() => (unit.value === 1 ? Faceplate160a : Faceplate160));
</script>

<template>
  <component :is="face" />
</template>

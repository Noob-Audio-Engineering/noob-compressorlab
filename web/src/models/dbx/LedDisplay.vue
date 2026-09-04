<script setup>
/**
 * The 160A's two LED rows, read off dbx's own 160A product photograph (adn.harmanpro.com/product_attachments/product_attachments/614_1728149313/160Afront_lg_original.jpg).
 *
 * **LEVEL**, 19 LEDs, with the printed scale beneath reading −40, 30, 20,
 * 15, 10, 8, 6, 5, 4, 3, 2, 1, 0, 1, 2, 4, 6, 10, 20+. The minus signs are
 * implied on the left half, which is why the panel prints bare numbers
 * there. The twelve from −40 to −1 are yellow and the seven from 0 to 20+
 * are red, sampled at #faf7a7 and #d2392b in the photograph's lit LEDs.
 *
 * **GAIN REDUCTION**, 12 LEDs, all red, sitting under the −40 to −1 marks
 * and sharing that half of the scale.
 *
 * The row shows the input or the output level according to `dbx_meter`,
 * which is what the 160A's DISPLAY button moves. dbx are explicit that the
 * make-up gain is "not displayed by the GAIN REDUCTION LEDs" but is
 * "reflected in the OUTPUT LEVEL display", and the model publishes those two
 * separately, so that holds here by construction.
 *
 * The display reaches 40 dB of reduction and no further, which is the
 * hardware: dbx note the row "displays up to 40dB of GAIN REDUCTION
 * (although the 160A is actually capable of delivering up to 60dB)".
 *
 * Props: `levelDb` (dB relative to 0 VU on the level row), `grDb` (dB of
 * reduction, positive), `lit`.
 */
import { computed, ref } from 'vue';
import { GR_DB, LEVEL_DB, LEVEL_SCALE, LEVEL_YELLOW, useNumeralsFit } from './useDbx.js';

const props = defineProps({
  levelDb: { type: Number, default: -60 },
  grDb: { type: Number, default: 0 },
  lit: { type: Boolean, default: true },
});

/*
 * A bar display, not a dot: every LED at or below the level is on, which is
 * what the photograph of a working unit shows and what "level display"
 * means. The lowest one lights as soon as there is a signal at all.
 */
const level = computed(() =>
  LEVEL_DB.map((db, i) => ({
    i,
    on: props.lit && props.levelDb >= db,
    yellow: i < LEVEL_YELLOW,
    label: LEVEL_SCALE[i],
  })),
);
const reduction = computed(() =>
  GR_DB.map((db, i) => ({ i, on: props.lit && props.grDb >= db })).reverse(),
);

/*
 * The printed scale between the two rows is dropped when it would render
 * below the legibility floor, and the lamps stay. On a 1U panel that scale
 * is about a millimetre tall, so below roughly a 1500-pixel window it is
 * four or five pixels: a row of grey smudges that says less than nothing.
 * The lamps still show the level, which is what the row is for.
 */
const scale = ref(null);
const numerals = useNumeralsFit(scale, (el) => parseFloat(getComputedStyle(el).fontSize) || 0);
</script>

<template>
  <div class="dbxleds">
    <div class="dbxleds__row">
      <span v-for="d in level" :key="'l' + d.i" class="dbxleds__led" :class="{ on: d.on, yellow: d.yellow }" />
      <span class="dbxleds__caption">LEVEL</span>
    </div>
    <div ref="scale" class="dbxleds__scale" :class="{ blank: !numerals }">
      <span v-for="d in level" :key="'s' + d.i">{{ numerals ? d.label : '' }}</span>
      <span class="dbxleds__spacer" />
    </div>
    <div class="dbxleds__row gr">
      <span v-for="d in reduction" :key="'g' + d.i" class="dbxleds__led" :class="{ on: d.on }" />
      <span class="dbxleds__caption wide">GAIN REDUCTION</span>
    </div>
  </div>
</template>

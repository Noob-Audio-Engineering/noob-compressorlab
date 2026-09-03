<script setup>
/**
 * The sixteen-lamp gain-reduction bargraph across the top of the panel,
 * silkscreened 26 down to 1 dB from left to right: every lamp whose printed
 * value the live reduction has reached lights up, so the bar fills leftwards
 * as the compressor works. Green to about 5 dB, amber to 10, red beyond,
 * which is the hardware's own code for "radical settings" — and the reason
 * the manual says not to be shy about hitting it.
 *
 * Reads the `meter` stream's `gr_db` (at most 0). Props: none.
 */
import { computed } from 'vue';
import { useStreamValue } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';
import { GR_LAMPS } from './useVca.js';

const gr = useStreamValue('meter', { index: 4, unit: 'db' });
const reduction = computed(() => Math.max(0, -gr.value));
const lamps = computed(() => GR_LAMPS.map((l) => ({ ...l, on: reduction.value >= l.db })));
</script>

<template>
  <div class="el8bar">
    <div class="el8bar__scale">
      <span v-for="l in GR_LAMPS" :key="l.db">{{ l.db }}</span>
    </div>
    <div class="el8bar__lamps">
      <span v-for="l in lamps" :key="l.db" class="el8lamp" :class="[l.colour, { on: l.on }]" :title="`${l.db} dB`"></span>
    </div>
    <div class="el8bar__title">GAIN REDUCTION</div>
  </div>
</template>

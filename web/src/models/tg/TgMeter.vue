<script setup>
/**
 * A vertical gain-reduction meter for the TG12413.
 *
 * **This is an invention and the panel says so out loud.** Drawing
 * TG12413-D101 has no meter anywhere on it; whatever indication the console
 * gave, it was not on this module's sheet. Chandler give their recreation
 * two meters scaled 0 to +16 dB of reduction and Waves give their module one
 * scaled +9 to −20, and both are the manufacturers' own additions for users
 * who expect one. So is this. It is drawn beside the strip rather than on
 * it, so that the module keeps the three switches and nothing else that
 * section 2.8 asks for.
 *
 * It is a bar and not a needle, deliberately: a moving-coil face would
 * imitate a part EMI did not fit, where a column of segments reads as
 * instrumentation the page has added.
 *
 * `meter[5]` is the needle's **position**, not the level it chases, because
 * the ballistics run in the audio thread for every model in this lab. This
 * draws what arrives and asks the framework only for a short critically
 * damped follow between frames; a second set of ballistics here would double
 * the engine's.
 *
 * Props: `label`. Emits: nothing.
 */
import { computed } from 'vue';
import { useNeedle } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';
import { METER_MARKS } from './useTg.js';

defineProps({ label: { type: String, default: 'Gain reduction' } });

const FULL = METER_MARKS[METER_MARKS.length - 1];

const needle = useNeedle('meter', {
  index: 5,
  mode: 'reduction',
  scale: 'linear',
  riseMs: 40,
  damping: 1,
});

/** Decibels of reduction, positive, clamped to the printed scale. */
const db = computed(() => Math.min(FULL, Math.max(0, -needle.position.value)));
/** How much of the column is lit, 0 at the top. */
const fill = computed(() => (db.value / FULL) * 100);

const marks = computed(() =>
  METER_MARKS.map((m) => ({ db: m, y: (m / FULL) * 100 })),
);
</script>

<template>
  <div class="tgmeter" role="img" :aria-label="`${label}: ${db.toFixed(1)} decibels`">
    <div class="tgmeter__caption">GAIN REDUCTION</div>
    <div class="tgmeter__body">
      <div class="tgmeter__column">
        <div class="tgmeter__fill" :style="{ height: `${fill}%` }" />
      </div>
      <div class="tgmeter__scale">
        <div
          v-for="m in marks"
          :key="m.db"
          class="tgmeter__mark"
          :style="{ top: `${m.y}%` }"
        >{{ m.db }}</div>
      </div>
    </div>
    <div class="tgmeter__unit">dB</div>
    <div class="tgmeter__ours">ours, not EMI’s</div>
  </div>
</template>

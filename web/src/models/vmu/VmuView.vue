<script setup>
/**
 * The Fairchild page: the strip with the lab's own controls, the front
 * panel, and the analysis drawer — the shared history and transfer panels,
 * identical under every face, with this model's own timing-and-charts panel
 * beside them the way the LA-2A puts its T4 panel there.
 *
 * **This face has a layout problem the other seven do not, and it is the
 * hardware's.** The 670 is 19 × 14 inches, which is 8U and very nearly
 * square, where the next tallest face here is 19 × 5.25. Fitted to the
 * height of a 900 × 520 window it would be a few hundred pixels wide and its
 * silkscreen unreadable. So the panel is capped from the height as every
 * face is, but with a floor under it, and the drawer gives way first: below
 * the height at which both fit, the drawer closes and the panel takes the
 * room. Nothing scrolls and nothing is drawn too small to read.
 *
 * The 660 is half the panel and gets the height back, so its aspect is read
 * from the unit switch rather than fixed.
 */
import { computed } from 'vue';
import { useDebug, useWindow } from '../../composables/useLab.js';
import { ui, useControls } from './useVmu.js';
import HistoryPanel from '../../components/HistoryPanel.vue';
import TransferPanel from '../../components/TransferPanel.vue';
import Charts from './Charts.vue';
import Faceplate from './Faceplate.vue';
import ExtrasBar from './ExtrasBar.vue';
import DebugPanel from '../../components/DebugPanel.vue';

const win = useWindow();
const c = useControls();

/**
 * Vertical space the page needs besides the panel: the top bar, the
 * paddings and gaps, and the extras strip, which carries two rows of
 * controls on this model. Measured at the 900 px minimum.
 */
const CHROME_PX = 178;
/** The least drawer worth showing; below this it closes instead. */
const BENCH_PX = 190;
/** The panel is never drawn narrower than this, whatever the window does. */
const PLATE_MIN = 520;

const aspect = computed(() => (c.unit && c.unit.index === 0 ? 19 / 7 : 19 / 14));
const room = computed(() => Math.max(120, win.height.value - CHROME_PX));
const showBench = computed(() => ui.scope && (room.value - BENCH_PX) * aspect.value >= PLATE_MIN);
const plateMax = computed(() => {
  const h = showBench.value ? room.value - BENCH_PX : room.value;
  return `${Math.max(PLATE_MIN, Math.round(h * aspect.value))}px`;
});
const debug = useDebug();
</script>

<template>
  <main class="lab-model lab-model--vmu">
    <ExtrasBar />
    <div class="shrink-0 px-3 pt-3">
      <div class="w-full mx-auto" :style="{ maxWidth: plateMax }"><Faceplate /></div>
    </div>
    <section v-if="showBench" class="lab-bench has-charts">
      <div class="lab-panel lab-panel--charts"><Charts /></div>
      <HistoryPanel />
      <TransferPanel />
    </section>
    <DebugPanel v-if="debug.shown.value" v-model:open="debug.open.value" />
  </main>
</template>

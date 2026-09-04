<script setup>
/**
 * The dbx 160 page: the strip with the lab's own controls and the rear
 * panel's trimmer, the front panel, and the analysis drawer (the shared
 * history and transfer panels, identical under every face). Mounted by
 * `LabPage.vue` while the model switch says 160.
 *
 * The two faces have different shapes, so the cap on the plate's width
 * follows the one being drawn: the original is a wood-cheeked half-rack at
 * 2.82 : 1, wider than it is tall but nothing like a rack panel, while the
 * 160A is 1U at 10.86 : 1 and would be a ribbon if it filled the width of a
 * large window. Both are capped from the window height so a wide, shallow
 * window never pushes the drawer off the bottom.
 */
import { computed } from 'vue';
import { useDebug, useWindow } from '../../composables/useLab.js';
import { ui, useUnit } from './useDbx.js';
import HistoryPanel from '../../components/HistoryPanel.vue';
import TransferPanel from '../../components/TransferPanel.vue';
import Faceplate from './Faceplate.vue';
import ExtrasBar from './ExtrasBar.vue';
import DebugPanel from '../../components/DebugPanel.vue';

const win = useWindow();
const unit = useUnit();
/**
 * Vertical space the page needs besides the panel: the top bar, the
 * paddings and gaps, the extras band and the least drawer height worth
 * having. Measured at the 900 px minimum with the bar on one row.
 */
const CHROME_PX = 344;
const aspect = computed(() => (unit.value === 1 ? 19 / 1.75 : 2.816));
const plateMax = computed(() => {
  const room = Math.round((win.height.value - CHROME_PX) * aspect.value);
  return `${Math.max(720, Math.min(2400, room))}px`;
});
const debug = useDebug();
</script>

<template>
  <main class="lab-model lab-model--dbx">
    <ExtrasBar />
    <div class="shrink-0 px-3 pt-3">
      <div class="w-full mx-auto" :style="{ maxWidth: plateMax }"><Faceplate /></div>
    </div>
    <section v-if="ui.scope" class="lab-bench">
      <HistoryPanel />
      <TransferPanel />
    </section>
    <DebugPanel v-if="debug.shown.value" v-model:open="debug.open.value" />
  </main>
</template>

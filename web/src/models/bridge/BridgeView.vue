<script setup>
/**
 * The Neve 33609 page: the strip with the lab's own controls, the two-unit
 * front panel, and the analysis drawer (the shared history and transfer
 * panels, identical under every face). Mounted by `LabPage.vue` while the
 * model switch says 33609.
 *
 * The panel keeps its 19 : 3.5 aspect and fills the width, capped from the
 * window height so a wide, shallow window never pushes the drawer off the
 * bottom. This face is two rack units where the Distressor is one, so the
 * cap binds sooner and `CHROME_PX` is measured accordingly rather than
 * copied from a neighbour.
 */
import { computed } from 'vue';
import { useDebug, useWindow } from '../../composables/useLab.js';
import { ui } from './useBridge.js';
import HistoryPanel from '../../components/HistoryPanel.vue';
import TransferPanel from '../../components/TransferPanel.vue';
import Faceplate from './Faceplate.vue';
import ExtrasBar from './ExtrasBar.vue';
import DebugPanel from '../../components/DebugPanel.vue';

const win = useWindow();

/**
 * Vertical space the page needs besides the panel: the top bar, the
 * paddings and gaps, the extras band and the least drawer height worth
 * having. Measured at the 900 px minimum with the bar on one row.
 */
const CHROME_PX = 322;
const PLATE_ASPECT = 19 / 3.5;
const plateMax = computed(() => `${Math.max(760, Math.round((win.height.value - CHROME_PX) * PLATE_ASPECT))}px`);
const debug = useDebug();
</script>

<template>
  <main class="lab-model lab-model--bridge">
    <ExtrasBar />
    <div class="shrink-0 px-3 pt-3"><div class="w-full mx-auto" :style="{ maxWidth: plateMax }"><Faceplate /></div></div>
    <section v-if="ui.scope" class="lab-bench">
      <HistoryPanel />
      <TransferPanel />
    </section>
    <DebugPanel v-if="debug.shown.value" v-model:open="debug.open.value" />
  </main>
</template>

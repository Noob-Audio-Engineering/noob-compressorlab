<script setup>
/**
 * The LA-3A page: the half-rack front panel, the strip carrying the rear
 * panel's switches and the lab's additions, and the analysis drawer (the
 * shared history and transfer panels, identical under every face). Mounted
 * by `LabPage.vue` while the model switch says LA-3A.
 *
 * The panel is 8.5 by 3.5 inches, half a rack, so unlike the other faces it
 * does not fill the width: it keeps its own proportions and sits centred,
 * with a cap from the window height so a wide window never pushes the
 * drawer off the bottom.
 */
import { computed } from 'vue';
import { useDebug, useWindow } from '../../composables/useLab.js';
import { ui } from './useLa3a.js';
import HistoryPanel from '../../components/HistoryPanel.vue';
import TransferPanel from '../../components/TransferPanel.vue';
import Faceplate from './Faceplate.vue';
import ExtrasStrip from './ExtrasStrip.vue';
import DebugPanel from '../../components/DebugPanel.vue';

const win = useWindow();
/*
 * What the page needs besides the panel at the smallest window: the top bar,
 * the paddings, the rear-panel strip and the workbench row's own minimum. The
 * panel yields to that rather than the other way round, so the charts stay
 * legible at 900 x 520.
 */
const CHROME_PX = 382;
const PLATE_ASPECT = 19 / 3.5;
/** Half a rack at a comfortable size, but never taller than the space left for the drawer. */
const plateMax = computed(() => `${Math.max(700, Math.min(2400, Math.round((win.height.value - CHROME_PX) * PLATE_ASPECT)))}px`);
const debug = useDebug();
</script>

<template>
  <main class="lab-model lab-model--la3a">
    <ExtrasStrip />
    <div class="shrink-0 px-3 pt-3"><div class="w-full mx-auto" :style="{ maxWidth: plateMax }"><Faceplate /></div></div>
    <section v-if="ui.scope" class="lab-bench">
      <HistoryPanel />
      <TransferPanel />
    </section>
    <DebugPanel v-if="debug.shown.value" v-model:open="debug.open.value" />
  </main>
</template>

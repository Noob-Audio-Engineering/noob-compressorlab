<script setup>
/**
 * The 1176 page: the front panel (in the look of the selected revision),
 * the strip with the modern additions, and the analysis drawer (the lab's
 * shared history and transfer panels, identical under both faces). The panel
 * follows the width up to the height cap in `fet.css`, the drawer takes
 * the rest. Mounted by `LabPage.vue` while the model switch says 1176.
 */
import { computed } from 'vue';
import { useDebug, useWindow } from '../../composables/useLab.js';
import { ui } from './useFet.js';
import HistoryPanel from '../../components/HistoryPanel.vue';
import TransferPanel from '../../components/TransferPanel.vue';
import Faceplate from './Faceplate.vue';
import ExtrasBar from './ExtrasBar.vue';
import DebugPanel from '../../components/DebugPanel.vue';


/*
 * The height budget: the app's top bar, this model's own bar, the
 * workbench's minimum and the paddings between them. The bar sits above the
 * faceplate and grows as it wraps, so this is sized from the tallest it
 * gets, which is at the 900 px minimum width where it wraps most. Measured
 * there rather than guessed.
 */
const win = useWindow();
const CHROME_PX = 384;
/** Two units in a full rack: the 1176's plate is 5.2 to 1. */
const PLATE_ASPECT = 5.2;
const plateMax = computed(() => `${Math.max(640, Math.round((win.height.value - CHROME_PX) * PLATE_ASPECT))}px`);
const debug = useDebug();
</script>

<template>
  <div class="lab-model lab-model--fet">
    <ExtrasBar />
    <div class="shrink-0 pt-3"><div class="w-full mx-auto" :style="{ maxWidth: plateMax }"><Faceplate /></div></div>
    <section v-if="ui.scope" class="lab-bench">
      <HistoryPanel />
      <TransferPanel />
    </section>
    <DebugPanel v-if="debug.shown.value" v-model:open="debug.open.value" />
  </div>
</template>

<script setup>
/**
 * The LA-2A page: the silver faceplate (Gain, the VU meter with its
 * selector and the Limit / Compress lever, Peak Reduction, the power
 * switch) and the dark workbench below with the modern displays (inside
 * the T4, and the lab's shared history and transfer panels, identical
 * under both faces) and the extras strip. Mounted by
 * `LabPage.vue` while the model switch says LA-2A.
 *
 * The view follows the window in both directions: the faceplate keeps its
 * 19 : 5.25 aspect and fills the width, capped from the window height so a
 * very wide window never pushes the workbench off the bottom; the
 * workbench takes whatever height remains (the charts grow with it); the
 * extras strip is a fixed band.
 */
import { computed } from 'vue';
import { useDebug, useWindow } from '../../composables/useLab.js';
import HistoryPanel from '../../components/HistoryPanel.vue';
import TransferPanel from '../../components/TransferPanel.vue';
import Faceplate from './Faceplate.vue';
import T4Panel from './T4Panel.vue';
import { ui } from './useOpto.js';
import ExtrasStrip from './ExtrasStrip.vue';
import DebugPanel from '../../components/DebugPanel.vue';

const win = useWindow();

/**
 * Vertical space the page needs besides the faceplate: the top bar (40),
 * the main padding (24), the two gaps (24), the extras band (about 74) and
 * the least workbench height worth having (100). The faceplate's width is
 * capped so its height never eats into that.
 */
const CHROME_PX = 331;
const PLATE_ASPECT = 19 / 5.25;
const plateMax = computed(() => `${Math.max(600, Math.round((win.height.value - CHROME_PX) * PLATE_ASPECT))}px`);
const debug = useDebug();
</script>

<template>
  <main class="lab-model lab-model--opto">
    <ExtrasStrip />
    <div class="shrink-0 px-3 pt-3"><div class="w-full mx-auto" :style="{ maxWidth: plateMax }"><Faceplate /></div></div>
    <section v-if="ui.scope" class="lab-bench has-t4">
      <div class="lab-panel lab-panel--t4"><T4Panel /></div>
      <HistoryPanel />
      <TransferPanel />
    </section>
    <DebugPanel v-if="debug.shown.value" v-model:open="debug.open.value" />
  </main>
</template>

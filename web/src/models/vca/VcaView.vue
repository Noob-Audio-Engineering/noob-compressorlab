<script setup>
/**
 * The Distressor page: the single-rack-unit front panel, the strip with the
 * additions the hardware has no room for, and the analysis drawer (the
 * lab's shared history and transfer panels, identical under every face).
 * Mounted by `LabPage.vue` while the model switch says Distressor.
 *
 * The view follows the window in both directions: the panel keeps its
 * 19 : 1.75 aspect and fills the width, capped from the window height so a
 * very wide window never pushes the drawer off the bottom; the drawer takes
 * whatever height remains.
 */
import { computed } from 'vue';
import { useWindow } from '../../composables/useLab.js';
import { ui } from './useVca.js';
import HistoryPanel from '../../components/HistoryPanel.vue';
import TransferPanel from '../../components/TransferPanel.vue';
import Faceplate from './Faceplate.vue';
import ExtrasBar from './ExtrasBar.vue';

const win = useWindow();

/**
 * Vertical space the page needs besides the panel: the top bar, the
 * paddings and gaps, the extras band and the least drawer height worth
 * having. A one-unit panel is short, so this rarely binds; it does once the
 * window is wide and shallow.
 */
const CHROME_PX = 338;
const PLATE_ASPECT = 19 / 1.75;
const plateMax = computed(() => `${Math.max(760, Math.round((win.height.value - CHROME_PX) * PLATE_ASPECT))}px`);
</script>

<template>
  <main class="lab-model lab-model--vca">
    <ExtrasBar />
    <div class="shrink-0 px-3 pt-3"><div class="w-full mx-auto" :style="{ maxWidth: plateMax }"><Faceplate /></div></div>
    <section v-if="ui.scope" class="lab-bench">
      <HistoryPanel />
      <TransferPanel />
    </section>
  </main>
</template>

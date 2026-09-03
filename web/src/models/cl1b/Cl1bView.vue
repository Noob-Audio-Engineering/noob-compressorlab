<script setup>
/**
 * The CL-1B page: the full-width front panel, the strip carrying the lab's
 * own additions, and the analysis drawer (the shared history and transfer
 * panels, identical under every face). Mounted by `LabPage.vue` while the
 * model switch says CL-1B.
 *
 * The panel is a full 19 inch rack width and three units tall, so it fills
 * the window's width like the 1176 and the Distressor rather than sitting
 * centred as the half-rack LA-3A does. Being 3U it is the tallest face in
 * the lab for its width, so the plate is capped from the window height as
 * well, and the workbench row keeps its own minimum: at 900 by 520 the
 * charts stay legible and the panel yields, not the other way round.
 */
import { computed } from 'vue';
import { useWindow } from '../../composables/useLab.js';
import { ui } from './useCl1b.js';
import HistoryPanel from '../../components/HistoryPanel.vue';
import TransferPanel from '../../components/TransferPanel.vue';
import Faceplate from './Faceplate.vue';
import ExtrasStrip from './ExtrasStrip.vue';

const win = useWindow();
/** The top bar, the paddings, the extras strip and the workbench's own minimum. */
const CHROME_PX = 283;
/** 483 by 131 mm: a 19 inch panel three units tall (section 2.1). */
const PLATE_ASPECT = 483 / 131;
const plateMax = computed(() => `${Math.max(760, Math.min(2600, Math.round((win.height.value - CHROME_PX) * PLATE_ASPECT)))}px`);
</script>

<template>
  <main class="lab-model lab-model--cl1b">
    <ExtrasStrip />
    <div class="shrink-0 px-3 pt-3"><div class="w-full mx-auto" :style="{ maxWidth: plateMax }"><Faceplate /></div></div>
    <section v-if="ui.scope" class="lab-bench">
      <HistoryPanel />
      <TransferPanel />
    </section>
  </main>
</template>

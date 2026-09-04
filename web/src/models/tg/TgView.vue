<script setup>
/**
 * The EMI TG12413 page: the strip with the lab's own controls, the module in
 * its frame, and the analysis drawer (the shared history and transfer
 * panels, identical under every face). Mounted by `LabPage.vue` while the
 * model switch says TG12413.
 *
 * This face is the odd one in the lab because the object is: every other
 * model here is a rack unit that fills the width, and this is a console
 * module about one fifth as wide as it is tall. So the layout is the other
 * way round — the strip takes the height it is given and the frame beside it
 * takes the width — and the height cap is on the strip rather than on a
 * plate aspect.
 */
import { computed } from 'vue';
import { useDebug, useWindow } from '../../composables/useLab.js';
import { ui } from './useTg.js';
import HistoryPanel from '../../components/HistoryPanel.vue';
import TransferPanel from '../../components/TransferPanel.vue';
import Faceplate from './Faceplate.vue';
import ExtrasBar from './ExtrasBar.vue';
import DebugPanel from '../../components/DebugPanel.vue';

const win = useWindow();

/**
 * Vertical space the page needs besides the module: the top bar, the
 * paddings and gaps, the extras band and the least drawer height worth
 * having. Measured at the 900 px minimum with the bar on one row.
 */
const CHROME_PX = 344;
/** The strip never grows past this, or it dwarfs the frame beside it. */
const STRIP_MAX_PX = 520;
const stripHeight = computed(
  () => `${Math.max(230, Math.min(STRIP_MAX_PX, win.height.value - CHROME_PX))}px`,
);
const debug = useDebug();
</script>

<template>
  <main class="lab-model lab-model--tg">
    <ExtrasBar />
    <div class="shrink-0 px-3 pt-3" :style="{ '--tg-strip-h': stripHeight }">
      <Faceplate />
    </div>
    <section v-if="ui.scope" class="lab-bench">
      <HistoryPanel />
      <TransferPanel />
    </section>
    <DebugPanel v-if="debug.shown.value" v-model:open="debug.open.value" />
  </main>
</template>

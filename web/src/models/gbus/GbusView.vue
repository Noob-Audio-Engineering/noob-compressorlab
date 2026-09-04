<script setup>
/**
 * The SSL 4000 G bus compressor page: the strip with the lab's own
 * controls, the 500-series module, and the shared history and transfer
 * panels, identical under every face. Mounted by `LabPage.vue` while the
 * model switch says 4000 G.
 *
 * **This is the only portrait face in the lab, and the layout follows from
 * that.** The module is a double 500-series slot, 3.0 by 5.25 inches, an
 * aspect of 1 : 1.769. Stacking the analysis panels under it the way the
 * rack faces do would leave the panel 240 px wide in a 900 x 520 window,
 * with six knobs on it, and the printed detent numbers would be illegible.
 * So the panel takes the height and the panels sit beside it, which is what
 * a landscape window has spare when the hardware is portrait. The bench bar
 * keeps its place directly under the top bar, as every model does.
 *
 * With the drawer closed the panel is centred and takes the whole stage.
 */
import { useDebug } from '../../composables/useLab.js';
import { ui } from './useGbus.js';
import HistoryPanel from '../../components/HistoryPanel.vue';
import TransferPanel from '../../components/TransferPanel.vue';
import Faceplate from './Faceplate.vue';
import ExtrasBar from './ExtrasBar.vue';
import DebugPanel from '../../components/DebugPanel.vue';

/*
 * The stage takes what the column has left rather than a height computed
 * from the window: the top bar, the extras band and the development panel
 * are all items in the same flex column, and an explicit height only fights
 * them. A zero `min-height` on the stage is what lets the panel shrink to
 * fit instead of pushing the drawer off the bottom, and the panel's own
 * `aspect-ratio` turns whatever height it gets into a width.
 */
const debug = useDebug();
</script>

<template>
  <main class="lab-model lab-model--gbus">
    <ExtrasBar />
    <section class="gbus-stage">
      <div class="gbus-stage__panel"><Faceplate /></div>
      <div v-if="ui.scope" class="gbus-stage__bench">
        <HistoryPanel />
        <TransferPanel />
      </div>
    </section>
    <DebugPanel v-if="debug.shown.value" v-model:open="debug.open.value" />
  </main>
</template>

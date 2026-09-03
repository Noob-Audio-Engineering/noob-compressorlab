<script setup>
/**
 * The 1176 page: the front panel (in the look of the selected revision),
 * the strip with the modern additions, and the analysis drawer (the lab's
 * shared history and transfer panels, identical under both faces). The panel
 * follows the width up to the height cap in `fet.css`, the drawer takes
 * the rest. Mounted by `LabPage.vue` while the model switch says 1176.
 */
import { ui } from './useFet.js';
import HistoryPanel from '../../components/HistoryPanel.vue';
import TransferPanel from '../../components/TransferPanel.vue';
import Faceplate from './Faceplate.vue';
import ExtrasBar from './ExtrasBar.vue';
</script>

<template>
  <div class="lab-model lab-model--fet">
    <Faceplate />
    <ExtrasBar />
    <section v-if="ui.scope" class="lab-bench">
      <HistoryPanel />
      <TransferPanel />
    </section>
  </div>
</template>

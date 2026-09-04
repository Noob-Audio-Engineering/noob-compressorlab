<script setup>
/**
 * The page once the manifest is in: the shared top bar, the active
 * model's view (any of the six compressors the lab can be) and the
 * framework's resize grip. The top bar's compressor button swaps that view
 * for the browse view; the engine hears nothing about it, so whatever is
 * loaded keeps processing with its settings while the user looks. Which view shows follows the `model` parameter,
 * so the choice is per instance and saved with the host's project;
 * switching re-mounts the view, and each view keeps its own colours (the
 * `.lab--*` classes only tint the shell).
 *
 * Window: every view scales with the window in both dimensions; the grip
 * in the bottom-right corner lets the user resize the plug-in window from
 * 900 × 520 up (through the one `useWindowSize` instance in `useLab.js`).
 * There is no fullscreen control: the framework supports it and this page
 * chooses not to offer it.
 */
import { computed } from 'vue';
import { ResizeGrip } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';
import { WINDOW_MIN, ui, useLab, useWindow } from '../composables/useLab.js';
import TopBar from './TopBar.vue';
import ModelBrowser from './ModelBrowser.vue';
import FetView from '../models/fet/FetView.vue';
import OptoView from '../models/opto/OptoView.vue';
import La3aView from '../models/la3a/La3aView.vue';
import VcaView from '../models/vca/VcaView.vue';
import Pre6176View from '../models/pre6176/Pre6176View.vue';
import Cl1bView from '../models/cl1b/Cl1bView.vue';
import BridgeView from '../models/bridge/BridgeView.vue';

const VIEWS = { fet: FetView, opto: OptoView, la3a: La3aView, vca: VcaView, pre6176: Pre6176View, cl1b: Cl1bView, bridge: BridgeView };
const lab = useLab();
useWindow();
const key = lab.key;
const view = computed(() => VIEWS[key.value] || FetView);
</script>

<template>
  <div class="lab" :class="`lab--${key}`">
    <TopBar />
    <ModelBrowser v-if="ui.browsing" />
    <component :is="view" v-else :key="key" />
    <ResizeGrip class="lab-grip" :min="WINDOW_MIN" title="Drag to resize the window" />
  </div>
</template>

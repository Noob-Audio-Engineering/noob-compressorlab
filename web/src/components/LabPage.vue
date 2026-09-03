<script setup>
/**
 * The page once the manifest is in: the shared top bar, the active
 * model's view (the 1176's face, extras and scope drawer, or the LA-2A's
 * face and workbench) and the framework's resize grip. Which view shows
 * follows the `model` parameter, so the choice is per instance and saved
 * with the host's project; switching re-mounts the view, and each view
 * keeps its own colours (`.lab--fet` / `.lab--opto` only tint the shell).
 *
 * Window: every view scales with the window in both dimensions; the grip
 * in the bottom-right corner lets the user resize the plug-in window from
 * 900 × 520 up, and the top bar's fullscreen button asks the host for the
 * monitor's work area (both through the one `useWindowSize` instance in
 * `useLab.js`).
 */
import { computed } from 'vue';
import { ResizeGrip } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';
import { WINDOW_MIN, useLab, useWindow } from '../composables/useLab.js';
import TopBar from './TopBar.vue';
import FetView from '../models/fet/FetView.vue';
import OptoView from '../models/opto/OptoView.vue';

const VIEWS = { fet: FetView, opto: OptoView };
const lab = useLab();
useWindow();
const key = lab.key;
const view = computed(() => VIEWS[key.value] || FetView);
</script>

<template>
  <div class="lab" :class="`lab--${key}`">
    <TopBar />
    <component :is="view" :key="key" />
    <ResizeGrip class="lab-grip" :min="WINDOW_MIN" title="Drag to resize the window" />
  </div>
</template>

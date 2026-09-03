<script setup>
/**
 * Noob CompressorLab: the root. Once the manifest is in (`ready`) the page
 * (`LabPage.vue`: the top bar with the model switch, the active model's
 * face and workbench, the resize grip) renders; before that a short status
 * line shows what the client is doing. In development the offline
 * manifest makes that immediate.
 *
 * Keyboard: Ctrl+Z / Ctrl+Shift+Z (or Ctrl+Y) undo and redo through the
 * framework's history, Ctrl+B toggles A/B.
 */
import { onBeforeUnmount, onMounted } from 'vue';
import { useNoobVstWebguiFramework } from './composables/useLab.js';
import LabPage from './components/LabPage.vue';

const { ready, connected, history } = useNoobVstWebguiFramework();

function onKey(e) {
  const t = e.target;
  if (t && (t.tagName === 'INPUT' || t.tagName === 'SELECT' || t.tagName === 'TEXTAREA')) return;
  const mod = e.ctrlKey || e.metaKey;
  const k = e.key.toLowerCase();
  if (mod && k === 'z' && !e.shiftKey) history.undo();
  else if ((mod && k === 'y') || (mod && e.shiftKey && k === 'z')) history.redo();
  else if (mod && k === 'b') history.toggleAB();
  else return;
  e.preventDefault();
}
onMounted(() => window.addEventListener('keydown', onKey));
onBeforeUnmount(() => window.removeEventListener('keydown', onKey));
</script>

<template>
  <LabPage v-if="ready" />
  <div v-else class="lab">
    <div class="lab__wait">{{ connected ? 'loading the manifest' : 'connecting to the lab' }}</div>
  </div>
</template>

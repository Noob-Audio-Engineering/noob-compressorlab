<script setup>
/**
 * The strip above the face, shared by both models: the model switch (the
 * `model` parameter, so the choice is per instance and saved with the
 * project), the active model's presets (previous / next, a menu with the
 * factory and user presets, Save As, Delete), undo / redo / A-B from the
 * framework's `History`, bypass, and the read-outs (edit→echo
 * round trip, reported latency, connection).
 *
 * Presets are per model: loading one touches only that model's parameters
 * and the shared extras, and the user lists live under `presets.user.fet`
 * / `presets.user.opto` in the UI store. Emits: nothing.
 */
import { computed, onBeforeUnmount, ref } from 'vue';
import { ContextMenu } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';
import { loadState, stateToJson, ui, useLab, useNoobVstWebguiFramework, useWindow } from '../composables/useLab.js';
import { FACTORY_PRESETS, loadUserPresets, onUserPresetsChange, saveUserPresets } from '../presets.js';

const { history, historyState, connected, stats, status, modified, client } = useNoobVstWebguiFramework();
const lab = useLab();
// The window handle is still created here so the page has exactly one; the
// framework's fullscreen support is untouched, this page simply has no
// control for it.
useWindow();
const key = lab.key;
const active = lab.active;
const version = ref(0);
const offStore = onUserPresetsChange(() => version.value++);
onBeforeUnmount(offStore);
const user = computed(() => {
  void version.value;
  return loadUserPresets(key.value);
});
const all = computed(() => [...FACTORY_PRESETS[key.value].map((p) => ({ ...p, factory: true })), ...user.value.map((p) => ({ ...p, factory: false }))]);
const presetName = computed(() => ui.preset[key.value]);
const menu = ref({ open: false, x: 0, y: 0, items: [] });

function load(p) {
  loadState(key.value, p.values || {});
  ui.preset[key.value] = p.name;
}
function step(dir) {
  const list = all.value;
  if (!list.length) return;
  let i = list.findIndex((p) => p.name === presetName.value);
  i = ((i < 0 ? 0 : i + dir) + list.length) % list.length;
  load(list[i]);
}
function saveAs() {
  const name = window.prompt('Preset name', presetName.value === active.value.initPreset ? 'My Setting' : presetName.value);
  if (!name) return;
  const list = user.value.filter((p) => p.name !== name);
  list.push({ name, values: stateToJson(key.value) });
  saveUserPresets(key.value, list);
  version.value++;
  ui.preset[key.value] = name;
  modified.value = false;
}
function remove(name) {
  saveUserPresets(
    key.value,
    user.value.filter((p) => p.name !== name),
  );
  version.value++;
}
function openMenu(e) {
  const r = e.currentTarget.getBoundingClientRect();
  const current = presetName.value;
  const items = [
    ...FACTORY_PRESETS[key.value].map((p) => ({ label: p.name, hint: p.description, checked: current === p.name, action: () => load(p) })),
    { divider: true },
    ...(user.value.length
      ? user.value.map((p) => ({ label: p.name, checked: current === p.name, hint: 'user preset', action: () => load(p) }))
      : [{ label: 'No user presets yet', disabled: true }]),
    { divider: true },
    { label: 'Save As…', action: saveAs },
    ...(user.value.some((p) => p.name === current) ? [{ label: `Delete “${current}”`, action: () => remove(current) }] : []),
  ];
  menu.value = { open: true, x: r.left, y: r.bottom + 4, items };
}
function toggleBypass() {
  lab.bypass.begin();
  lab.bypass.setOn(!lab.bypass.on);
  lab.bypass.end();
}
const fmt = (ms) => (Number.isNaN(ms) || ms == null ? '–' : ms < 1 ? `${(ms * 1000).toFixed(0)} µs` : `${ms.toFixed(2)} ms`);
const latency = computed(() => (status.value?.latency_ms != null ? `${status.value.latency_ms.toFixed(2)} ms` : '–'));
const offline = computed(() => client.offline === true);

</script>

<template>
  <header class="labbar">
    <div class="labbar__brand">
      <span class="dot" :class="{ on: connected }" :title="connected ? 'connected' : offline ? 'design mode: no plug-in connected' : 'connecting'"></span>
      <span class="labbar__name">NOOB COMPRESSORLAB</span>
      <button
      class="labbar__model"
      type="button"
      :aria-pressed="ui.browsing"
      title="Change compressor: browse by family without disturbing what is playing"
      @click="ui.browsing = !ui.browsing"
    >
      <span class="labbar__modelname">{{ active.label }}</span>
      <span class="labbar__modelcue">{{ ui.browsing ? 'browsing' : 'change' }}</span>
    </button>
      <span class="labbar__sub">{{ active.sub }} · an affectionate spoof</span>
    </div>
    <div class="labbar__presets">
      <button class="tb" title="Previous preset" @click="step(-1)">‹</button>
      <button class="tb wide" title="Presets" @click="openMenu">{{ presetName }}<span v-if="modified"> *</span></button>
      <button class="tb" title="Next preset" @click="step(1)">›</button>
    </div>
    <div class="labbar__tools">
      <button class="tb hist" :disabled="!historyState.canUndo" title="Undo (Ctrl+Z)" @click="history.undo()">↶</button>
      <button class="tb hist" :disabled="!historyState.canRedo" title="Redo (Ctrl+Shift+Z)" @click="history.redo()">↷</button>
      <button class="tb hist" title="Toggle A / B (Ctrl+B)" @click="history.toggleAB()"><b :class="{ dim: historyState.ab !== 'A' }">A</b>/<b :class="{ dim: historyState.ab !== 'B' }">B</b></button>
      <button class="tb hist" title="Copy this state to the other slot" @click="history.copyToOther()">⧉</button>
      <button class="tb" :class="{ on: lab.bypass.on }" :title="lab.bypass.on ? 'Bypassed: click to put the compressor back in' : 'Bypass the compressor'" @click="toggleBypass">BYPASS</button>
      <span class="labbar__stat echo">edit→echo <b>{{ fmt(stats.echoAvgMs) }}</b></span>
      <span class="labbar__stat">latency <b>{{ latency }}</b></span>
    </div>
    <ContextMenu :open="menu.open" :x="menu.x" :y="menu.y" :items="menu.items" @close="menu.open = false" />

  </header>
</template>

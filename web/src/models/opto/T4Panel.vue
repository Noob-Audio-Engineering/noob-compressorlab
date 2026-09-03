<script setup>
/**
 * "Inside the T4": three bars from the `cell` stream, the electroluminescent
 * light, the free carriers (the photocell's conductance, what sets the
 * gain right now) and the trapped carriers (the memory that makes the
 * release slow after long, hard compression). Read at animation-frame rate
 * through the framework's `useStreamFrame`.
 *
 * Light and free carriers span decades (the light law is exponential, and
 * the model works in physical units: the light sits around 1e-6..1e-4 at
 * working levels, the carriers around 1e-3), so the light bar is
 * logarithmic over eight decades and the free carriers over five; the
 * trapped carriers are shown linearly, magnified a hundredfold, because a
 * percent of full traps is already a long tail.
 */
import { computed } from 'vue';
import { useStreamFrame } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';

const frame = useStreamFrame('cell');
const logN = (v, decades) => (v <= 10 ** -decades ? 0 : Math.min(1, 1 + Math.log10(v) / decades));
const bars = computed(() => {
  const f = frame.value || [0, 0, 0];
  const light = f[0] || 0;
  const free = f[1] || 0;
  const trapped = f[2] || 0;
  return [
    { label: 'Light', v: logN(light, 8), cls: 'light' },
    { label: 'Free', v: logN(free, 5), cls: 'free' },
    { label: 'Trapped', v: Math.min(1, trapped * 100), cls: 'trapped' },
  ];
});
const pct = (v) => `${Math.round(Math.min(1, Math.max(0, v)) * 100)}%`;
</script>

<template>
  <div class="h-full flex flex-col t4-root">
    <div class="bench-label mb-2">Inside the T4</div>
    <div class="flex-1 flex items-end justify-around gap-3 px-1">
      <div v-for="b in bars" :key="b.label" class="flex flex-col items-center gap-1 h-full justify-end">
        <div class="bar" :class="b.cls">
          <div class="fill" :style="{ height: pct(b.v) }"></div>
        </div>
        <div class="text-[9px] tracking-widest uppercase text-plate-100/50">{{ b.label }}</div>
      </div>
    </div>
    <div class="blurb text-[10px] text-plate-100/40 mt-2 leading-snug">Light drives the photocell. Trapped carriers empty slowly, so the release lengthens the harder and longer the unit has worked.</div>
  </div>
</template>

<style scoped>
.t4-root {
  container-type: size;
}
/* Too short for the explanation: keep the bars. */
@container (max-height: 170px) {
  .blurb {
    display: none;
  }
}
.bar {
  width: 22px;
  height: 100%;
  border-radius: 4px;
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.08);
  display: flex;
  align-items: flex-end;
  overflow: hidden;
}
.fill {
  width: 100%;
  border-radius: 3px 3px 0 0;
  transition: height 60ms linear;
}
.light .fill {
  background: linear-gradient(180deg, #fff0c0, #e9a23b);
  box-shadow: 0 0 12px rgba(233, 162, 59, 0.6);
}
.free .fill {
  background: linear-gradient(180deg, #9fe2ff, #3aa0d0);
}
.trapped .fill {
  background: linear-gradient(180deg, #d9a8ff, #8a4bc9);
}
</style>

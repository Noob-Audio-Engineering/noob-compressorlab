<script setup>
/**
 * One of the CL-1B's three-position lever switches: a black lever with a
 * white index stripe on a chrome ball bushing, its three legends printed on
 * the panel to the left, above and to the right of it.
 *
 * All three of the hardware's switches of this kind have three positions,
 * not two: METER is input / compression / output, attack/release SELECT is
 * fixed / fix./man. / manual, and sidechain BUS SELECT is off / 1 / 2. The
 * lever leans left for position 0, stands up for 1 and leans right for 2.
 *
 * Clicking a legend goes straight to that position; clicking the lever
 * steps to the next and wraps, which is what a lever does when you flick
 * it.
 *
 * Props: `p` (the handle, required), `legends` (three strings, left, top,
 * right; the top one may carry a newline to set on two lines as the panel
 * does with "compres-sion"), `caption` / `capline2` (the control's name,
 * printed below).
 * Emits: nothing.
 */
import { computed } from 'vue';

const props = defineProps({
  p: { type: Object, required: true },
  legends: { type: Array, required: true },
  caption: { type: String, default: '' },
  capline2: { type: String, default: '' },
});

const index = computed(() => Math.min(2, Math.max(0, props.p.index)));
const topLines = computed(() => String(props.legends[1] ?? '').split('\n'));

function go(next) {
  const i = Math.min(2, Math.max(0, next));
  if (i === props.p.index) return;
  props.p.begin();
  props.p.setIndex(i);
  props.p.end();
}
</script>

<template>
  <div class="cl1blev">
    <div class="cl1blev__legends">
      <button class="cl1blev__leg left" :class="{ on: index === 0 }" type="button" @click="go(0)">{{ legends[0] }}</button>
      <button class="cl1blev__leg top" :class="{ on: index === 1 }" type="button" @click="go(1)">
        <span v-for="(l, i) in topLines" :key="i">{{ l }}</span>
      </button>
      <button class="cl1blev__leg right" :class="{ on: index === 2 }" type="button" @click="go(2)">{{ legends[2] }}</button>
    </div>
    <button
      class="cl1blev__body"
      type="button"
      role="slider"
      :aria-label="p.name"
      :aria-valuetext="p.text"
      :aria-valuemin="0"
      :aria-valuemax="2"
      :aria-valuenow="index"
      @click="go((index + 1) % 3)"
      @keydown.left.prevent="go(index - 1)"
      @keydown.right.prevent="go(index + 1)"
    >
      <span class="cl1blev__bat" :class="`at${index}`"></span>
      <span class="cl1blev__bush"></span>
    </button>
    <div v-if="caption" class="cl1blev__caption">
      <span>{{ caption }}</span>
      <b v-if="capline2">{{ capline2 }}</b>
    </div>
  </div>
</template>

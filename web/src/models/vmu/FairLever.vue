<script setup>
/**
 * The METERING control, which is **not a knob**: a tall black lever with a
 * white arrow tip, of the kind used on rotary lever switches, with
 * `BAL  ZERO  BAL` arced above it on the panel.
 *
 * Its three positions are the push side of the output stage, the centre tap,
 * and the pull side — not input, gain reduction and output. POM Audio
 * Design, who build new 670s, describe them better than anyone: *"it is a
 * bit like a tube tester… the 'Push' side current of the output stage by
 * flicking the 'BAL' switch to the Left, the overall current that is flowing
 * through the centre tap of the output transformer in the 'ZERO' position,
 * the 'Pull' side current by flicking it to the Right"*.
 *
 * Universal Audio removed these positions from their emulation because "the
 * software version of the meters have the benefit of not requiring
 * recalibrating". That is defensible and this makes the opposite choice: the
 * three positions work, because this is the one place the hardware admits
 * that its meter is a valve tester rather than a gain-reduction movement.
 *
 * Clicking a legend goes straight to that position; clicking the lever steps
 * to the next and wraps, which is what a lever does when you flick it.
 *
 * Props: `p` (the handle, required), `caption`, `capline2`.
 * Emits: nothing.
 */
import { computed } from 'vue';

const props = defineProps({
  p: { type: Object, required: true },
  caption: { type: String, default: '' },
  capline2: { type: String, default: '' },
});

const index = computed(() => Math.min(2, Math.max(0, props.p.index)));

function go(next) {
  const i = Math.min(2, Math.max(0, next));
  if (i === props.p.index) return;
  props.p.begin();
  props.p.setIndex(i);
  props.p.end();
}
</script>

<template>
  <div class="fairlev">
    <div v-if="caption" class="fairlev__caption">
      <span>{{ caption }}</span>
      <span v-if="capline2">{{ capline2 }}</span>
    </div>
    <div class="fairlev__legends">
      <button class="fairlev__leg left" :class="{ on: index === 0 }" type="button" @click="go(0)">BAL</button>
      <button class="fairlev__leg mid" :class="{ on: index === 1 }" type="button" @click="go(1)">ZERO</button>
      <button class="fairlev__leg right" :class="{ on: index === 2 }" type="button" @click="go(2)">BAL</button>
    </div>
    <button
      class="fairlev__body"
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
      <span class="fairlev__bat" :class="`at${index}`"><i></i></span>
      <span class="fairlev__bush"></span>
    </button>
  </div>
</template>

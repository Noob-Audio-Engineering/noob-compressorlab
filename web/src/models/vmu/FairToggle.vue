<script setup>
/**
 * A bat-handle toggle in a chrome bushing: the mains switch at the top left,
 * marked `ON`, and the AGC switch between the two channel rows.
 *
 * The AGC switch is the one that matters. Its two ends are bracketed on the
 * panel as `LEFT / RIGHT` and `LAT / VERT`, and what it does is throw ten
 * wafers at once to put a sum-and-difference matrix in front of both
 * channels and another behind them. It is not a stereo link and it is not a
 * mode switch on a linked pair: in the lateral-and-vertical position the two
 * limiters are still entirely independent, and they are now working on mid
 * and side. Fairchild built it for cutting stereo lacquers, and made the
 * musical argument for mid-side bus compression in 1959 almost as an aside:
 * *"such limiting will retain the spatial distribution of instruments and
 * soloists as originally recorded without producing any annoying image
 * drift"*.
 *
 * Props: `p` (the handle; when it is null the toggle drives `fallback`),
 * `up` / `down` (the legends beside each end of the throw, two lines each,
 * newline-separated), `vertical` (the mains switch, which throws up and
 * down with one legend above it), `fallback` (`{ on, set }`).
 * Emits: nothing.
 */
import { computed } from 'vue';

const props = defineProps({
  p: { type: Object, default: null },
  up: { type: String, default: '' },
  down: { type: String, default: '' },
  label: { type: String, default: '' },
  vertical: { type: Boolean, default: false },
  fallback: { type: Object, default: null },
});

const on = computed(() => (props.p ? props.p.index >= 1 : !!props.fallback?.on?.value));
const upLines = computed(() => String(props.up).split('\n'));
const downLines = computed(() => String(props.down).split('\n'));

function toggle() {
  if (props.p) {
    props.p.begin();
    props.p.setIndex(on.value ? 0 : 1);
    props.p.end();
  } else if (props.fallback) {
    props.fallback.set(!props.fallback.on.value);
  }
}
</script>

<template>
  <div class="fairtog" :class="{ vertical }">
    <div v-if="label" class="fairtog__label">{{ label }}</div>
    <div v-if="!vertical" class="fairtog__row">
      <span class="fairtog__leg left" :class="{ on: !on }">
        <b v-for="(l, i) in upLines" :key="i">{{ l }}</b>
      </span>
      <button
        class="fairtog__body"
        type="button"
        role="switch"
        :aria-checked="on"
        :aria-label="p ? p.name : label"
        @click="toggle"
      >
        <span class="fairtog__bat" :class="on ? 'right' : 'left'"></span>
        <span class="fairtog__bush"></span>
      </button>
      <span class="fairtog__leg right" :class="{ on }">
        <b v-for="(l, i) in downLines" :key="i">{{ l }}</b>
      </span>
    </div>
    <button
      v-else
      class="fairtog__body up"
      type="button"
      role="switch"
      :aria-checked="on"
      :aria-label="label || 'Power'"
      @click="toggle"
    >
      <span class="fairtog__bat" :class="on ? 'up' : 'down'"></span>
      <span class="fairtog__bush"></span>
    </button>
  </div>
</template>

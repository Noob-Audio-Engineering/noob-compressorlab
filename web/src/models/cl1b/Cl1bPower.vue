<script setup>
/**
 * The mains switch at the far right of the bottom row: a small black knob
 * with a white index line, not a toggle, with OFF printed above and to its
 * left and ON above and to its right, exactly as the panel has it.
 *
 * It binds to whatever `power` in `useCl1b.js` resolved to. When the engine
 * publishes `cl1b_power` the switch really does power the unit down; until
 * then it falls back to page state and only darkens the panel, and says so
 * in its tooltip rather than pretending. Nothing here changes when the
 * parameter lands.
 *
 * **Powering down passes audio through, it does not silence it.** A real
 * CL-1B with the mains off passes nothing, because its audio path runs
 * through the tube stages, but the 1176 in this same plug-in already chose
 * pass-through for its own power-down and the two are kept consistent
 * inside one product. So the meter parks and the jewel goes out, and
 * nothing on this panel suggests the signal has stopped.
 *
 * Props: `p` (the power handle from `useCl1b.js`, required).
 * Emits: nothing.
 */
import { computed } from 'vue';

const props = defineProps({
  p: { type: Object, required: true },
});

const on = computed(() => props.p.on.value);
const hint = computed(() => {
  if (!props.p.real) {
    return on.value
      ? 'Mains on'
      : 'Mains off: the panel goes dark. This model has no power parameter yet, so the audio is unaffected.';
  }
  return on.value ? 'Mains on' : 'Mains off: the unit passes audio through, as the 1176 here does when powered down.';
});
</script>

<template>
  <div class="cl1bpwr">
    <span class="cl1bpwr__leg off">OFF</span>
    <span class="cl1bpwr__leg on">ON</span>
    <button
      class="cl1bpwr__body"
      type="button"
      role="switch"
      :aria-checked="on"
      aria-label="Mains switch"
      :title="hint"
      @click="p.set(!on)"
    >
      <svg viewBox="0 0 100 100" class="cl1bpwr__dial">
        <circle cx="50" cy="50" r="42" class="cl1bpwr__skirt" />
        <circle cx="50" cy="50" r="35" class="cl1bpwr__cap" />
        <g :transform="`rotate(${on ? 38 : -38} 50 50)`">
          <rect x="48.6" y="13" width="2.8" height="26" rx="1.4" class="cl1bpwr__index" />
        </g>
      </svg>
    </button>
  </div>
</template>

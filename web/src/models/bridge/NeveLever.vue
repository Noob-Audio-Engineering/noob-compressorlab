<script setup>
/**
 * One of the 33609/N's bat-handle toggles: a chrome lever with a pale
 * paddle tip on a black bushing, its two legends printed above and below on
 * the panel.
 *
 * All of this panel's levers are two-position, unlike the CL-1B's
 * three-position ones: the /N replaced the /J's illuminated pushbuttons
 * with toggles, and each still names its two states above and below rather
 * than beside. Up is the upper legend and down the lower, so `limit` sits
 * above `in` and the lever points at whichever is true.
 *
 * Optional `lamps` draws the pair of indicator LEDs the centre block's
 * levers carry to their right, the upper lit in the up position and the
 * lower in the down.
 *
 * Props: `p` (a toggle handle, required), `up` / `down` (the two legends;
 * either may carry a newline to set on two lines, as `internal control`
 * does), `lamps` (boolean).
 * Emits: nothing.
 */
import { computed } from 'vue';

const props = defineProps({
  p: { type: Object, required: true },
  up: { type: String, default: '' },
  down: { type: String, default: '' },
  lamps: { type: Boolean, default: false },
});

/**
 * True when the lever stands up, at the upper legend. The handle's `on` is
 * the upper legend in every use here: bypass on is "bypass" and off is "in",
 * link on is "mono" and off is "stereo". An `invert` prop used to sit here
 * and every one of its three call sites needed it the other way round, so
 * both bypass levers and the mono/stereo lever drew the wrong position.
 */
const isUp = computed(() => props.p.on);
const upLines = computed(() => String(props.up).split('\n'));
const downLines = computed(() => String(props.down).split('\n'));

function flip() {
  props.p.begin();
  props.p.setOn(!isUp.value);
  props.p.end();
}
</script>

<template>
  <div class="nevelev" :class="{ 'has-lamps': lamps }">
    <div class="nevelev__leg up">
      <span v-for="(l, i) in upLines" :key="i">{{ l }}</span>
    </div>
    <div class="nevelev__row">
      <button
        class="nevelev__body"
        type="button"
        role="switch"
        :aria-checked="isUp"
        :aria-label="p.name"
        @click="flip"
        @keydown.up.prevent="!isUp && flip()"
        @keydown.down.prevent="isUp && flip()"
      >
        <span class="nevelev__bat" :class="isUp ? 'up' : 'down'">
          <i class="nevelev__paddle"></i>
        </span>
      </button>
      <span v-if="lamps" class="nevelev__lamps">
        <i :class="{ lit: isUp }"></i>
        <i :class="{ lit: !isUp }"></i>
      </span>
    </div>
    <div class="nevelev__leg down">
      <span v-for="(l, i) in downLines" :key="i">{{ l }}</span>
    </div>
  </div>
</template>

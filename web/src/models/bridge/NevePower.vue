<script setup>
/**
 * The 33609/N's mains button: a large round illuminated red push, in its
 * own narrow block at the far right of the panel, with a vertical white bar
 * across the lens. The /J had a rectangular illuminated pushbutton here;
 * the /N replaced it with this, and the photograph is of an /N.
 *
 * It drives `neve_power` if the engine publishes one, and page state if it
 * does not, which only darkens the panel. The fallback is stated here
 * rather than left for someone to discover, because a control that writes
 * nowhere while looking live is the ornament this project has removed
 * twice.
 *
 * Note what powering down does: it passes audio through rather than
 * silencing it. That is a deliberate divergence from the hardware, kept
 * consistent with the 1176 and the CL-1B in this same plug-in, so nothing
 * on this face may imply the signal stops.
 *
 * Props: none. Emits: nothing.
 */
import { usePower } from './useBridge.js';

const power = usePower();
</script>

<template>
  <button
    class="nevepwr"
    type="button"
    role="switch"
    :aria-checked="power.on.value"
    aria-label="Mains"
    :title="power.real ? 'Mains' : 'Mains (panel only: the engine publishes no power parameter)'"
    :class="{ on: power.on.value }"
    @click="power.set(!power.on.value)"
  >
    <span class="nevepwr__lens"><i class="nevepwr__bar"></i></span>
  </button>
</template>

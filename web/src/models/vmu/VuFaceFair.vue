<script setup>
/**
 * The Fairchild's VU movement: a cream face in a deep black moulded bezel,
 * recessed into the panel, drawn from the two photographs the dossier's 2.3
 * and 2.5 read.
 *
 * The scale is the standard VU one — `-20 -10 -7 -5 -3 -2 -1 0 +1 +2 +3` on
 * the upper arc and `0 20 40 60 80 100` on the lower, with the arc from 0 to
 * +3 in red and `VU` printed at **both** ends, which is unusual and is what
 * the metal does. The maker's name sits across the lower half of the face.
 *
 * **What the needle is reading is not a programme level and not gain
 * reduction.** The meter is a valve-current bridge across the push-pull
 * output stage: in ZERO it reads the current in the centre tap, and in the
 * two BALANCE positions one leg each. Once the unit is balanced and zeroed,
 * deflection during operation is the *change* in control-valve current,
 * which is gain reduction — but as a consequence of the bridge going out of
 * balance rather than because anything measures it. Moving the ZERO screw
 * moves the standing current, so it moves the needle, which is exactly what
 * it does on the hardware.
 *
 * `meter[5]` is the needle's position, not the level it is chasing: the
 * standard VU movement runs in the audio thread for every model here. So
 * this draws the field as it arrives and asks the framework's needle only
 * for a short critically-damped follow to bridge the gap between frames.
 *
 * Props: `lit` (the meter lamp, which the mains switch darkens), `label`.
 * Emits: nothing.
 */
import { computed } from 'vue';
import { useNeedle } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';

defineProps({
  lit: { type: Boolean, default: true },
  label: { type: String, default: 'VU meter' },
});

const SWEEP = 84;
const CX = 150;
const CY = 250;
const R = 218;
const R_NUM = R * 0.965;
const R_TICK = R * 0.905;
const R_TICK_IN = R * 0.85;
const R_MINOR_IN = R * 0.878;
const R_BAND = R * 0.93;
const R_PCT = R * 0.78;
const R_PCT_IN = R * 0.758;
const R_PCT_NUM = R * 0.72;

const needle = useNeedle('meter', {
  index: 5,
  unit: 'db',
  mode: 'level',
  min: -20,
  max: 3,
  riseMs: 40,
  damping: 1,
  sweep: SWEEP,
});

const marks = needle.marks([-20, -10, -7, -5, -3, -2, -1, 0, 1, 2, 3]);
const minor = needle.marks([-15, -8, -6, -4, -2.5, -1.5, -0.5, 0.5, 1.5, 2.5]);

const PERCENTS = [0, 20, 40, 60, 80, 100];
const percents = computed(() =>
  PERCENTS.map((p) => {
    const db = p <= 0 ? -20 : Math.max(-20, 20 * Math.log10(p / 100));
    return { label: String(p), angle: needle.marks([db])[0].angle };
  }),
);

const pt = (deg, r) => {
  const a = (deg * Math.PI) / 180;
  return [CX + r * Math.sin(a), CY - r * Math.cos(a)];
};
const arc = (from, to, r) => {
  const [x1, y1] = pt(from, r);
  const [x2, y2] = pt(to, r);
  return `M ${x1} ${y1} A ${r} ${r} 0 0 1 ${x2} ${y2}`;
};
const zero = marks.find((m) => m.value === 0).angle;
const left = -SWEEP / 2;
const right = SWEEP / 2;
const label = (v) => (v > 0 ? `+${v}` : String(v));
</script>

<template>
  <div class="fairvu" :class="{ unlit: !lit }" role="img" :aria-label="label">
    <svg viewBox="0 0 300 210" preserveAspectRatio="xMidYMid meet">
      <rect x="0" y="0" width="300" height="210" rx="6" class="fairvu__face" />

      <!-- the dB arc, black to zero and red past it -->
      <path :d="arc(left, zero, R_BAND)" class="fairvu__arc" />
      <path :d="arc(zero, right, R_BAND)" class="fairvu__arc red" />

      <g class="fairvu__ticks">
        <line
          v-for="m in marks"
          :key="'M' + m.value"
          :x1="pt(m.angle, R_TICK)[0]"
          :y1="pt(m.angle, R_TICK)[1]"
          :x2="pt(m.angle, R_TICK_IN)[0]"
          :y2="pt(m.angle, R_TICK_IN)[1]"
          :class="{ red: m.value > 0 }"
        />
        <line
          v-for="m in minor"
          :key="'m' + m.value"
          class="minor"
          :x1="pt(m.angle, R_TICK)[0]"
          :y1="pt(m.angle, R_TICK)[1]"
          :x2="pt(m.angle, R_MINOR_IN)[0]"
          :y2="pt(m.angle, R_MINOR_IN)[1]"
          :class="{ red: m.value > 0 }"
        />
      </g>

      <g class="fairvu__nums">
        <text
          v-for="m in marks"
          :key="'n' + m.value"
          :x="pt(m.angle, R_NUM)[0]"
          :y="pt(m.angle, R_NUM)[1]"
          text-anchor="middle"
          dominant-baseline="central"
          :class="{ red: m.value > 0 }"
        >
          {{ label(m.value) }}
        </text>
      </g>

      <!-- the percentage arc, the same needle read the other way -->
      <g class="fairvu__pct">
        <line
          v-for="p in percents"
          :key="'p' + p.label"
          :x1="pt(p.angle, R_PCT)[0]"
          :y1="pt(p.angle, R_PCT)[1]"
          :x2="pt(p.angle, R_PCT_IN)[0]"
          :y2="pt(p.angle, R_PCT_IN)[1]"
        />
        <text
          v-for="p in percents"
          :key="'pn' + p.label"
          :x="pt(p.angle, R_PCT_NUM)[0]"
          :y="pt(p.angle, R_PCT_NUM)[1]"
          text-anchor="middle"
          dominant-baseline="central"
        >
          {{ p.label }}
        </text>
      </g>

      <!-- VU at both ends of the scale, which is what the metal prints -->
      <text class="fairvu__vu" x="46" y="34">VU</text>
      <text class="fairvu__vu" x="254" y="34">VU</text>
      <text class="fairvu__maker" x="150" y="150" text-anchor="middle">NOOB</text>

      <line
        class="fairvu__needle"
        :x1="CX"
        :y1="CY"
        :x2="pt(needle.angle.value, R * 0.99)[0]"
        :y2="pt(needle.angle.value, R * 0.99)[1]"
      />
      <circle class="fairvu__hub" :cx="CX" :cy="CY" r="12" />
    </svg>
  </div>
</template>

<script setup>
/**
 * The module's compression meter: a moving-coil movement reading 0 to 20 dB
 * of gain reduction, left to right, on a **linear** scale, with numbered
 * ticks at 0, 4, 8, 12, 16 and 20 and unnumbered ticks between them. The
 * pointer pivots below the visible window and rests at 0 at the far left.
 *
 * **The scale is linear in decibels and that is not a simplification.** The
 * clone builder read the drive off the same circuit: the movement is fed
 * from the buffered control voltage through a series resistor, "linear
 * scale, at about 50 µA/dB, making a 1 mA meter showing 20 dB full-scale".
 * So the needle follows the control voltage, not a decibel conversion of a
 * measured gain. This is the rare case where the naive meter and the circuit
 * meter agree, because a Blackmer VCA's control voltage is itself linear in
 * decibels, and the dossier says so rather than building machinery it does
 * not need.
 *
 * **The ballistics are not here.** `meter_vu` at index 5 of the `meter`
 * stream is where the needle *is*, not the level it is chasing: the movement
 * runs in the audio thread in `src/dsp/vu.rs` for every model in the lab.
 * The `useNeedle` model below is a light frame-rate follower that turns that
 * value into an angle; smoothing it properly here would apply the
 * ballistics twice.
 *
 * Reads: the `meter` stream, index 5. Props: `lit`. Emits: nothing.
 */
import { computed } from 'vue';
import { useNeedle } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';
import { METER_LABELS, METER_MAX_DB } from './useGbus.js';

defineProps({ lit: { type: Boolean, default: true } });

/*
 * The glass measures 0.558 of the panel's width by 0.229 of its height,
 * which at the panel's 1 : 1.769 aspect is 0.558 by 0.405 in width units,
 * so the window is 1.38 times as wide as it is tall. The viewBox is a
 * hundred units across and 72.6 down to match.
 *
 * The pivot sits below the window, as the hardware's does, and it sits a
 * long way below: at 150 units the scale curved so hard that the 0 and the
 * 4 crowded together at the left end. A moving-coil meter this shape draws
 * a shallow arc, so the pivot goes to 260 and the sweep narrows to 20
 * degrees to keep the same travel across the glass. The ticks then rise
 * only 3.6 units from the middle of the scale to its ends.
 */
const VIEW_H = 72.6;
const CX = 50;
const CY = 260;
const R_TICK_OUT = 226;
const R_TICK_IN = 214;
const R_MINOR_IN = 219;
/*
 * The numerals sit **outside** the tick arc, which is where the render puts
 * them and, more to the point, where they do not land on the same line as
 * the `dB` and `COMPRESSION` legends low in the window. Inside the arc at a
 * 97-unit radius they drew across both.
 */
const R_NUM = 237;
const NEEDLE_TIP = 228;
const SWEEP = 20;

const needle = useNeedle('meter', {
  index: 5,
  unit: 'db',
  mode: 'level',
  min: 0,
  max: METER_MAX_DB,
  riseMs: 40,
  damping: 1,
  sweep: SWEEP,
});

const MINORS = [2, 6, 10, 14, 18];
const major = needle.marks(METER_LABELS);
const minor = needle.marks(MINORS);

const pt = (deg, r) => {
  const a = (deg * Math.PI) / 180;
  return [CX + r * Math.sin(a), CY - r * Math.cos(a)];
};

const majorTicks = computed(() =>
  major.map((m) => {
    const [x1, y1] = pt(m.angle, R_TICK_IN);
    const [x2, y2] = pt(m.angle, R_TICK_OUT);
    const [nx, ny] = pt(m.angle, R_NUM);
    return { value: m.value, x1, y1, x2, y2, nx, ny };
  }),
);
const minorTicks = computed(() =>
  minor.map((m) => {
    const [x1, y1] = pt(m.angle, R_MINOR_IN);
    const [x2, y2] = pt(m.angle, R_TICK_OUT);
    return { value: m.value, x1, y1, x2, y2 };
  }),
);
const tip = computed(() => pt(needle.angle.value, NEEDLE_TIP));
const hub = computed(() => pt(needle.angle.value, R_TICK_IN - 100));
</script>

<template>
  <div class="sslmeter" :class="{ unlit: !lit }">
    <svg :viewBox="`0 0 100 ${VIEW_H}`" class="sslmeter__svg" role="img" aria-label="Compression, dB">
      <rect x="0" y="0" width="100" :height="VIEW_H" class="sslmeter__glass" />
      <g class="sslmeter__scale">
        <line v-for="t in minorTicks" :key="'n' + t.value" :x1="t.x1" :y1="t.y1" :x2="t.x2" :y2="t.y2" class="minor" />
        <line v-for="t in majorTicks" :key="'t' + t.value" :x1="t.x1" :y1="t.y1" :x2="t.x2" :y2="t.y2" />
        <text
          v-for="t in majorTicks"
          :key="'l' + t.value"
          :x="t.nx"
          :y="t.ny"
          text-anchor="middle"
          dominant-baseline="central"
        >{{ t.value }}</text>
      </g>
      <text x="50" y="57" text-anchor="middle" class="sslmeter__unit">dB</text>
      <text x="50" y="67" text-anchor="middle" class="sslmeter__caption">COMPRESSION</text>
      <line :x1="hub[0]" :y1="hub[1]" :x2="tip[0]" :y2="tip[1]" class="sslmeter__needle" />
    </svg>
  </div>
</template>

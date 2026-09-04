<script setup>
/**
 * One of the 33609's two gain-reduction movements, drawn from measurements
 * taken off `ref/neve-33609n-front.jpg` rather than from the dossier's prose.
 *
 * The part is three pieces stacked, not one box: a clear-bezelled **glass**
 * carrying the black scale plate (42.4 % of the unit's height), the bezel's
 * bright bottom **lip** (4.8 %), and below them the black crinkle-finished
 * **can** the movement lives in (52.8 %), with its slotted zero-adjust screw
 * at the centre. The screw is on the can, not on the scale, which is where I
 * had drawn it before I measured.
 *
 * It reads the other way round from the VU faces elsewhere in this plug-in.
 * A VU rests at the right of its scale and falls back as it is driven; this
 * rests at 0 on the **left** and swings right as reduction deepens, so more
 * needle means more compression.
 *
 * **The printed scale is not linear in decibels.** Fitting a circle to the
 * six tick marks in the photograph puts the pivot 2.16 glass-heights below
 * the top of the glass and 0.496 of the way across it, with the ticks on a
 * radius of 0.767 glass widths, and the residuals are under a third of a
 * pixel. Read against that centre the marks fall at −30.8°, −17.2°, −2.3°,
 * +8.3°, +19.5° and +30.4° from vertical: the 0–8 dB end is stretched and
 * everything above 8 dB is squeezed into even ~10.8° steps. Both meters give
 * the same figures to within a pixel although they sit at different places
 * in the frame, which rules out lens distortion. So the marks are drawn
 * where they are printed and the needle is interpolated through the same
 * table, and a reading taken off this face is the reading the metal gives.
 *
 * `meter[5]` is the needle's position rather than the level it chases,
 * because the ballistics run in the audio thread for every model here. So
 * this draws what arrives and asks the framework's needle only for a short
 * critically-damped follow to bridge the gap between frames, which is what
 * `riseMs: 40, damping: 1` is for. Laying a second set of ballistics over
 * the engine's would double them.
 *
 * The movement's own type markings, a "19" beside the recall symbol and a
 * "2.5" in the far corner, are left off: they measure a third of the height
 * of the "dB" and would be noise at every size this panel is drawn at.
 *
 * Props: `label` (for assistive technology; the visible caption is panel
 * silkscreen and belongs to `Faceplate.vue`), `lit` (the meter lamp, which
 * the mains button darkens).
 * Emits: nothing.
 */
import { computed } from 'vue';
import { useNeedle } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';
import { METER_ANGLES } from './useBridge.js';

const props = defineProps({
  label: { type: String, default: '' },
  lit: { type: Boolean, default: true },
});

/*
 * The glass, in a viewBox scaled from the 179 x 68 px it measures in the
 * photograph: 300 wide, so every measured pixel is worth 1.676 units.
 */
const K = 300 / 179;
const CX = 88.8 * K;   // 148.8: the pivot, all but centred across the glass
const CY = 147.1 * K;  // 246.5: and far below it, inside the can
const R_OUT = 137.3 * K;  // the ticks' outer ends
const R_IN = 124.8 * K;   // and their inner ends
const R_NUM = 142.1 * K;  // the printed numbers, outside the ticks
const R_NEEDLE = 136.3 * K;

/**
 * Decibels of reduction → degrees from vertical, through the measured marks.
 * Linear between them and clamped at the ends, because the needle stops.
 */
const degFor = (db) => {
  const v = Math.min(METER_ANGLES[METER_ANGLES.length - 1][0], Math.max(0, db));
  for (let i = 1; i < METER_ANGLES.length; i += 1) {
    const [d0, a0] = METER_ANGLES[i - 1];
    const [d1, a1] = METER_ANGLES[i];
    if (v <= d1) return a0 + ((v - d0) / (d1 - d0)) * (a1 - a0);
  }
  return METER_ANGLES[METER_ANGLES.length - 1][1];
};

/*
 * `mode: 'reduction'` signs the arriving figure negative, so the scale runs
 * from 0 at the left to −20 at the right and `position` is negative
 * decibels of reduction. `scale: 'linear'` keeps the framework's own
 * voltage-proportional VU spacing, which is right for the other six models
 * here, out of a face that is not a VU; the angle comes from the measured
 * table above, not from the framework's `angle`.
 */
const needle = useNeedle('meter', {
  index: 5,
  unit: 'db',
  mode: 'reduction',
  scale: 'linear',
  min: 0,
  max: -20,
  riseMs: 40,
  damping: 1,
});

/*
 * The movement parks at 0 when the mains button is out, because an unpowered
 * meter's needle rests at the left stop. Powering down does not silence the
 * signal here, it passes it through, which is the same deliberate divergence
 * the CL-1B makes; so the needle stopping is the meter losing its lamp and
 * its coil, not the audio stopping. `NevePower.vue` and `web/README.md` both
 * say so, and nothing on this face suggests otherwise.
 */
const reduction = computed(() => (props.lit ? Math.abs(needle.position.value) : 0));
const angle = computed(() => degFor(reduction.value));

/** Point at `deg` from the pivot, `r` out. */
const pt = (deg, r) => {
  const a = (deg * Math.PI) / 180;
  return { x: CX + r * Math.sin(a), y: CY - r * Math.cos(a) };
};

const marks = computed(() =>
  METER_ANGLES.map(([db, deg]) => ({
    db,
    a: pt(deg, R_OUT),
    b: pt(deg, R_IN),
    t: pt(deg, R_NUM),
  })),
);

const tip = computed(() => pt(angle.value, R_NEEDLE));
</script>

<template>
  <div class="nevemeter" :class="{ unlit: !lit }">
    <div class="nevemeter__glass">
      <svg
        viewBox="0 0 300 114"
        class="nevemeter__svg"
        preserveAspectRatio="none"
        role="img"
        :aria-label="`${label || 'Gain reduction'}: ${reduction.toFixed(1)} dB of reduction`"
      >
        <defs>
          <clipPath :id="`neveGlass${label.replace(/\W/g, '')}`">
            <rect x="0" y="0" width="300" height="114" />
          </clipPath>
        </defs>
        <rect x="0" y="0" width="300" height="114" class="nevemeter__face" />

        <g class="nevemeter__scale">
          <line v-for="m in marks" :key="'t' + m.db" :x1="m.a.x" :y1="m.a.y" :x2="m.b.x" :y2="m.b.y" />
          <text v-for="m in marks" :key="'n' + m.db" :x="m.t.x" :y="m.t.y" text-anchor="middle" dominant-baseline="central">{{ m.db }}</text>
        </g>

        <text x="150" y="84" text-anchor="middle" class="nevemeter__unit">dB</text>
        <text x="29" y="98" text-anchor="middle" class="nevemeter__recall">↻</text>

        <g :clip-path="`url(#neveGlass${label.replace(/\W/g, '')})`">
          <line :x1="CX" :y1="CY" :x2="tip.x" :y2="tip.y" class="nevemeter__needle" />
        </g>
      </svg>
    </div>
    <div class="nevemeter__lip"></div>
    <div class="nevemeter__can"><i class="nevemeter__zero"></i></div>
  </div>
</template>

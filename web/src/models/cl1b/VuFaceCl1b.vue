<script setup>
/**
 * The CL-1B's VU meter: a rectangular movement in a black moulded bezel
 * with a cream face, drawn from `research/CL-1B.md` section 2.1.
 *
 * It carries more than the 1176's does, which is why it is its own
 * component rather than that one reused. The upper arc is the standard VU
 * dB scale, black to zero and red past it, with a heavy red band from 0 to
 * +3, a small black minus at the left end and a red plus at the right. A
 * second, smaller arc below it carries the percentage scale, which is the
 * same movement read the other way: 100 % is 0 VU and each step down is
 * 20·log10 of the ratio, so the two scales are one needle position
 * expressed twice. Across the middle sits the maker's script over a rule,
 * and VU in bold sans at the lower right rather than centred.
 *
 * `meter[5]` is the needle's position, not the level it is chasing: the
 * standard VU movement runs in the audio thread for every model in this
 * plug-in. So this draws the field as it arrives and asks the framework's
 * needle only for a short critically-damped follow, enough to bridge the
 * gap between frames without laying a second set of ballistics over the
 * engine's.
 *
 * Props: `lit` (boolean, default true) — the meter lamp, which the mains
 * switch darkens. Emits: nothing.
 */
import { computed } from 'vue';
import { useNeedle } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';

defineProps({
  lit: { type: Boolean, default: true },
});

/*
 * The movement's geometry. A real VU's arc is shallow and its numbers sit
 * outside the tick arc rather than inside it, with the percentage scale
 * tucked below the ticks, so the radii below run numbers, ticks, then
 * percentages inward. The pivot sits below the face and is clipped, as the
 * hardware's is.
 */
const SWEEP = 80;
const CX = 150;
const CY = 245;
const R = 210;
const R_NUM = R * 0.955;
const R_ARC = R * 0.9;
const R_TICK_IN = R * 0.845;
const R_MINOR_IN = R * 0.872;
const R_BAND = R * 0.925;
const R_PCT = R * 0.775;
const R_PCT_IN = R * 0.755;
const R_PCT_NUM = R * 0.715;

const needle = useNeedle('meter', { index: 5, unit: 'db', mode: 'level', min: -20, max: 3, riseMs: 40, damping: 1, sweep: SWEEP });

const marks = needle.marks([-20, -10, -7, -5, -3, -2, -1, 0, 1, 2, 3]);
const minor = needle.marks([-15, -8, -6, -4, -2.5, -1.5, -0.5, 0.5, 1.5, 2.5]);

/**
 * The percentage arc. 100 % is 0 VU and the rest follow 20·log10(p/100);
 * 0 % is minus infinity, so it sits at the left end of the travel.
 */
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
const zeroAngle = marks.find((m) => m.value === 0).angle;
const endLeft = -SWEEP / 2;
const endRight = SWEEP / 2;
</script>

<template>
  <div class="cl1bvu" :class="{ dark: !lit }">
    <svg viewBox="0 0 300 150" class="cl1bvu__svg">
      <defs>
        <linearGradient id="cl1bVuFace" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stop-color="#f8e6bd" />
          <stop offset="1" stop-color="#eed7a4" />
        </linearGradient>
        <linearGradient id="cl1bVuGlass" x1="0" y1="0" x2="0.3" y2="1">
          <stop offset="0" stop-color="rgba(255,255,255,0.26)" />
          <stop offset="0.35" stop-color="rgba(255,255,255,0.04)" />
          <stop offset="1" stop-color="rgba(0,0,0,0.1)" />
        </linearGradient>
        <clipPath id="cl1bVuClip"><rect x="0" y="0" width="300" height="150" rx="4" /></clipPath>
      </defs>

      <g clip-path="url(#cl1bVuClip)">
        <rect x="0" y="0" width="300" height="150" fill="url(#cl1bVuFace)" />

        <!-- the dB arc: black to zero, red past it, with the heavy band above -->
        <path :d="arc(endLeft, zeroAngle, R_ARC)" class="cl1bvu__arc" />
        <path :d="arc(zeroAngle, endRight, R_ARC)" class="cl1bvu__arc red" />
        <path :d="arc(zeroAngle, endRight, R_BAND)" class="cl1bvu__band" />

        <line
          v-for="m in minor"
          :key="'m' + m.value"
          :x1="pt(m.angle, R_ARC)[0]"
          :y1="pt(m.angle, R_ARC)[1]"
          :x2="pt(m.angle, R_MINOR_IN)[0]"
          :y2="pt(m.angle, R_MINOR_IN)[1]"
          :class="['cl1bvu__tick', { red: m.value > 0 }]"
        />
        <g v-for="m in marks" :key="'M' + m.value">
          <line
            :x1="pt(m.angle, R_ARC)[0]"
            :y1="pt(m.angle, R_ARC)[1]"
            :x2="pt(m.angle, R_TICK_IN)[0]"
            :y2="pt(m.angle, R_TICK_IN)[1]"
            :class="['cl1bvu__tick', 'major', { red: m.value > 0 }]"
          />
          <text
            :x="pt(m.angle, R_NUM)[0]"
            :y="pt(m.angle, R_NUM)[1]"
            text-anchor="middle"
            dominant-baseline="central"
            :class="['cl1bvu__num', { red: m.value > 0 }]"
          >
            {{ m.value }}
          </text>
        </g>

        <!-- the ends: a black minus at the left, a red plus at the right -->
        <text :x="pt(endLeft - 3.5, R_ARC)[0]" :y="pt(endLeft - 3.5, R_ARC)[1]" text-anchor="middle" dominant-baseline="central" class="cl1bvu__end">−</text>
        <text :x="pt(endRight + 3.5, R_BAND)[0]" :y="pt(endRight + 3.5, R_BAND)[1]" text-anchor="middle" dominant-baseline="central" class="cl1bvu__end red">+</text>

        <!-- the percentage arc below it, the same needle read the other way -->
        <g v-for="p in percents" :key="'p' + p.label">
          <line
            :x1="pt(p.angle, R_PCT)[0]"
            :y1="pt(p.angle, R_PCT)[1]"
            :x2="pt(p.angle, R_PCT_IN)[0]"
            :y2="pt(p.angle, R_PCT_IN)[1]"
            class="cl1bvu__tick pct"
          />
          <text :x="pt(p.angle, R_PCT_NUM)[0]" :y="pt(p.angle, R_PCT_NUM)[1]" text-anchor="middle" dominant-baseline="central" class="cl1bvu__pct">{{ p.label }}</text>
        </g>

        <!-- the maker's script over its rule, and VU at the lower right -->
        <text x="150" y="122" text-anchor="middle" class="cl1bvu__maker">NOOB</text>
        <line x1="108" y1="130" x2="192" y2="130" class="cl1bvu__rule" />
        <text x="266" y="141" text-anchor="middle" class="cl1bvu__vu">VU</text>

        <g :transform="`rotate(${needle.angle.value} ${CX} ${CY})`">
          <line :x1="CX" :y1="CY" :x2="CX" :y2="CY - R * 0.83" class="cl1bvu__needle" />
        </g>
        <rect x="0" y="0" width="300" height="150" fill="url(#cl1bVuGlass)" />
      </g>
    </svg>
  </div>
</template>

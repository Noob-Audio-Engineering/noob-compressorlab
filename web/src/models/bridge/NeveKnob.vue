<script setup>
/**
 * One of the 33609's twelve switch knobs: a flat matte blue cap on a bright
 * knurled aluminium skirt, a white index line across the cap, and its scale
 * printed on the panel around it.
 *
 * **The skirt is grey, not black.** The research's colour table used to call
 * it "near-black, knurled" at #1B1F20. Reading `ref/neve-33609n-front.jpg`
 * radially out from a knob centre gives the blue cap to about 0.9 of the cap
 * radius, then #9BA2A1 at 1.15 and #606B6F through 1.3 to 1.6 before the
 * panel takes over, and the close crop shows an unmistakably bright machined
 * knurl. The dossier's section 2.4 now carries the measured values and the
 * correction; this draws what the photograph shows.
 *
 * Sizes, in fractions of the panel's width, from the same photograph: the
 * cap measures 0.0257 across and the skirt 0.0392, so the cap is 0.61 of the
 * skirt; the scale's ticks ring the knob at 0.0219 from its centre and the
 * printed numbers at 0.0272. This box is 7.4 % of the panel, and the
 * constants below put every one of those where it was measured.
 *
 * Every control on this unit is a rotary switch, so the knob steps between
 * detents rather than turning smoothly, and the scale is drawn as one short
 * radial tick per detent, which is how the panel prints it. Only some ticks
 * carry a number: the limit threshold has twenty-three stops and twelve
 * numbers. The detent counts come from the switch assembly drawings quoted
 * in the research's section 2.3, not from the photograph, which does not
 * resolve individual ticks at this size.
 *
 * Props: `p` (the handle, required), `marks` (`{ at, label }`, `at` a
 * fraction of travel), `sweep` (degrees), `size`, `label`.
 * Emits: nothing. Gestures come from the framework's `useKnobGesture`.
 */
import { computed } from 'vue';
import { useKnobGesture } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';

const props = defineProps({
  p: { type: Object, required: true },
  marks: { type: Array, default: () => [] },
  sweep: { type: Number, default: 270 },
  size: { type: [Number, String], default: 100 },
  label: { type: String, default: null },
});

/** The blue cap's radius in the 100-unit box: 0.0257 of the panel, halved. */
const CAP = 17;
/** The knurled skirt's outer radius, 1.53 caps as measured. */
const SKIRT = 26;
/** The printed tick ring, and the numbers outside it. */
const TICK_IN = 27.5;
const TICK_OUT = 29.5;
const TICK_LONG = 30.5;
/**
 * The numbers are centred on a ring, which is how the panel sets them: the
 * measured centres of the gain knob's "20" and the ratio knob's "1.5:1" sit
 * at 2.60 and 2.42 per cent of the panel from their own knobs, and the two
 * clear each other by 0.42 across a 5.84 gap between the knobs. Marks near
 * the top or bottom of the sweep go a little further out, because there they
 * grow across the radius rather than along it. Anchoring them outward from
 * the ticks instead, which is the other way panels do this, pushes the long
 * labels far enough out to touch the next knob's.
 */
const NUM_SIDE = 34;
const NUM_END = 36.5;
/** Teeth around the knurl. */
const TEETH = 36;

const { handlers, dragging } = useKnobGesture(props.p, { discrete: true });

const width = computed(() => (typeof props.size === 'number' ? `${props.size}px` : props.size));

/*
 * Rotation follows the pot's travel rather than its plain value: these are
 * stepped switches whose plain value is a decibel figure or a millisecond
 * time, and neither is linear in the angle the shaft turns through. `norm`
 * is, and so are the measured mark positions.
 */
const angle = computed(() => -props.sweep / 2 + Math.min(1, Math.max(0, props.p.norm)) * props.sweep);

/*
 * The read-out shown while dragging. The framework prints a stepped
 * parameter as `Math.round(plain)`, which is right for whole-decibel
 * switches and wrong for the limiter's threshold: that one steps in half
 * decibels, so +8.5 and +9.0 dBu would both read "9". The framework's own
 * note says product-specific formatting belongs in the page, so this adds a
 * decimal exactly when the switch's step is not a whole number and leaves
 * every other control to the shared rule.
 */
const readout = computed(() => {
  const s = props.p.spec || {};
  const steps = Number(s.steps) || 0;
  if (steps < 2 || !Number.isFinite(s.min) || !Number.isFinite(s.max)) return props.p.text;
  const step = (s.max - s.min) / (steps - 1);
  if (Number.isInteger(step)) return props.p.text;
  const v = props.p.plain.toFixed(1);
  return s.unit ? `${v} ${s.unit}` : v;
});

const place = (at, radius) => {
  const a = ((-props.sweep / 2 + at * props.sweep) * Math.PI) / 180;
  return { x: 50 + radius * Math.sin(a), y: 50 - radius * Math.cos(a) };
};

/** One tick per detent, run a little longer where a number sits on it. */
const ticks = computed(() =>
  props.marks.map((m, i) => ({
    i,
    a: place(m.at, TICK_IN),
    b: place(m.at, m.label ? TICK_LONG : TICK_OUT),
    long: !!m.label,
  })),
);
const legends = computed(() =>
  props.marks
    .filter((m) => m.label)
    .map((m, i) => {
      const rad = ((-props.sweep / 2 + m.at * props.sweep) * Math.PI) / 180;
      const side = Math.abs(Math.sin(rad)) > 0.3;
      return { i, label: m.label, ...place(m.at, side ? NUM_SIDE : NUM_END) };
    }),
);

/*
 * The knurl: fine flutes machined into the skirt, drawn wholly inside its
 * outer circle. Running them out to the rim instead gives the knob a
 * sawtooth silhouette and it reads as a gear, which is not what the
 * photograph shows: the skirt's edge there is a clean circle with the
 * fluting inside it.
 */
const teeth = computed(() =>
  Array.from({ length: TEETH }, (_, i) => {
    const a = ((i / TEETH) * 2 * Math.PI) - Math.PI / 2;
    return {
      x1: 50 + (SKIRT - 4.5) * Math.cos(a),
      y1: 50 + (SKIRT - 4.5) * Math.sin(a),
      x2: 50 + (SKIRT - 1) * Math.cos(a),
      y2: 50 + (SKIRT - 1) * Math.sin(a),
    };
  }),
);
</script>

<template>
  <div class="neveknob" :style="{ width }">
    <svg
      viewBox="0 0 100 100"
      class="neveknob__dial"
      tabindex="0"
      role="slider"
      :aria-label="label || p.name"
      :aria-valuetext="p.text"
      v-on="handlers"
    >
      <defs>
        <linearGradient :id="`neveSkirt${p.id}`" x1="0" y1="0" x2="0.3" y2="1">
          <stop offset="0" stop-color="#c2c8c8" />
          <stop offset="0.5" stop-color="#98a1a2" />
          <stop offset="1" stop-color="#5d686c" />
        </linearGradient>
      </defs>

      <!-- the scale, printed on the panel: a tick per detent, a number on some -->
      <g class="neveknob__marks">
        <line
          v-for="t in ticks"
          :key="'t' + t.i"
          :x1="t.a.x"
          :y1="t.a.y"
          :x2="t.b.x"
          :y2="t.b.y"
          :class="{ long: t.long }"
        />
        <text v-for="l in legends" :key="'l' + l.i" :x="l.x" :y="l.y" text-anchor="middle" dominant-baseline="central">{{ l.label }}</text>
      </g>

      <!-- the body: grey knurled skirt under a flat blue cap with one index line -->
      <g :transform="`rotate(${angle} 50 50)`">
        <circle cx="50" cy="50" :r="SKIRT" :fill="`url(#neveSkirt${p.id})`" />
        <g class="neveknob__knurl">
          <line v-for="(t, i) in teeth" :key="'k' + i" :x1="t.x1" :y1="t.y1" :x2="t.x2" :y2="t.y2" />
        </g>
        <circle cx="50" cy="50" :r="SKIRT" class="neveknob__skirtrim" />
        <circle cx="50" cy="50" :r="CAP" class="neveknob__cap" />
        <circle cx="50" cy="50" :r="CAP" class="neveknob__caprim" />
        <rect x="49.1" :y="50 - CAP + 2" width="1.8" height="15" rx="0.9" class="neveknob__index" />
      </g>
    </svg>
    <div v-if="dragging" class="neveknob__value">{{ readout }}</div>
  </div>
</template>

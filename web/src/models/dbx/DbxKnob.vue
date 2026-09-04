<script setup>
/**
 * A dbx 160 or 160A knob, with its scale printed on the panel around it.
 *
 * **The `range` prop is the point of this component.** The two units' pots
 * are not the same — the original's THRESHOLD runs 10 mV to 3 V where the
 * 160A's runs −40 to +20 dBu, and the original's COMPRESSION stops at the ∞
 * mark where the 160A's carries on past it to −1:1 — but they are one
 * parameter, so each face passes the normalised span its own hardware covers
 * and the knob maps its rotation across that. Full anticlockwise on the
 * original's threshold knob is the 10 mV mark, and turning it cannot reach a
 * setting that pot does not have.
 *
 * That is what the framework's knob gesture calls a `rotation` mapping: a
 * dial whose printed scale is not the parameter's own range turns at a
 * constant rate under the pointer and converts back through the mapping. So
 * the drag, the wheel, the arrow keys and double-click-to-reset all come
 * from the framework and only the mapping lives here.
 *
 * Two bodies. The original's is a plain pointer knob with an index line from
 * the centre, drawn from dbx's **monochrome** front-panel figure, so its
 * colour is this plug-in's own and is not a measurement. The 160A's is a
 * coloured cap with a white pointer, and those colours are sampled from
 * dbx's own product photograph.
 *
 * Props: `p` (the handle, required), `marks` (`{ at, label }`, `at` a
 * fraction of *this dial's* travel), `sweep` (degrees), `range` (the
 * normalised span this pot covers), `size`, `cap` (a CSS colour for the
 * 160A's cap, or null for the original's), `label`.
 * Emits: nothing.
 */
import { computed, ref } from 'vue';
import { useKnobGesture } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';
import { useNumeralsFit } from './useDbx.js';

const props = defineProps({
  p: { type: Object, required: true },
  marks: { type: Array, default: () => [] },
  sweep: { type: Number, default: 270 },
  range: { type: Array, default: () => [0, 1] },
  size: { type: [Number, String], default: 100 },
  cap: { type: String, default: null },
  label: { type: String, default: null },
  /*
   * Font size of the printed scale, in the SVG's own 100-unit box. The
   * default suits the original's half-rack panel; the 160A passes a larger
   * one, because at 1U its knobs are a twentieth of the panel wide and the
   * scale at true size would be under six pixels in any window this page
   * opens at.
   */
  scaleFont: { type: Number, default: 9.5 },
});

const lo = computed(() => props.range[0]);
const span = computed(() => props.range[1] - props.range[0] || 1);
const clamp01 = (v) => Math.min(1, Math.max(0, v));

const { handlers, dragging } = useKnobGesture(props.p, {
  rotation: {
    toRotation: (n) => clamp01((n - lo.value) / span.value),
    fromRotation: (r) => lo.value + clamp01(r) * span.value,
  },
});

const width = computed(() => (typeof props.size === 'number' ? `${props.size}px` : props.size));

/*
 * The printed numbers are dropped when they would render below the
 * legibility floor, and the ticks are kept. A scale mark with no number
 * still shows a position; four-pixel type shows neither, and it looks
 * broken. This binds on the 160A, whose knobs are a twentieth of a 1U panel
 * wide, and not on the original's half-rack face.
 *
 * The font is in the SVG's own 100-unit box, so its size in pixels is that
 * font times the box's rendered width.
 */
const host = ref(null);
const numerals = useNumeralsFit(host, (el) => (props.scaleFont / 100) * el.getBoundingClientRect().width);
/** Where the pointer sits on this dial's own travel, 0..1. */
const travel = computed(() => clamp01((props.p.norm - lo.value) / span.value));
const angle = computed(() => -props.sweep / 2 + travel.value * props.sweep);

const RADIUS = 25;
const TICK_IN = 27.5;
const TICK_OUT = 31;
const TICK_SHORT = 29.5;
/*
 * Numbers ride a little further out at the ends of the sweep, where they
 * grow across the radius rather than along it. Anchoring every label at one
 * radius instead lets the long ones at top and bottom reach the neighbouring
 * knob's scale.
 */
const NUM_SIDE = 39.5;
const NUM_END = 42;

function place(at, r) {
  const a = ((-props.sweep / 2 + at * props.sweep) * Math.PI) / 180;
  return { x: 50 + r * Math.sin(a), y: 50 - r * Math.cos(a) };
}

const ticks = computed(() =>
  props.marks.map((m, i) => ({
    i,
    a: place(m.at, TICK_IN),
    b: place(m.at, m.label ? TICK_OUT : TICK_SHORT),
    long: !!m.label,
  })),
);
const legends = computed(() =>
  props.marks
    .map((m, i) => ({ m, i }))
    .filter(({ m }) => m.label)
    .map(({ m, i }) => {
      const rad = ((-props.sweep / 2 + m.at * props.sweep) * Math.PI) / 180;
      // Near the top and bottom of the sweep a label grows across the
      // radius rather than along it, so it needs the further ring. The
      // original's threshold dial is the case that decides this: its 100 mV
      // and 300 mV marks sit 52 degrees apart either side of twelve
      // o'clock, and at the nearer ring their labels touch.
      const side = Math.abs(Math.sin(rad)) > 0.55;
      return { i, label: m.label, ...place(m.at, side ? NUM_SIDE : NUM_END) };
    }),
);
</script>

<template>
  <div ref="host" class="dbxknob" :style="{ width }">
    <svg
      viewBox="0 0 100 100"
      class="dbxknob__dial"
      :class="{ active: dragging }"
      tabindex="0"
      role="slider"
      :aria-label="label || p.name"
      :aria-valuetext="p.text"
      v-on="handlers"
    >
      <!-- the scale, printed on the panel around the knob -->
      <g class="dbxknob__marks" :style="{ fontSize: `${scaleFont}px` }">
        <line
          v-for="t in ticks"
          :key="'t' + t.i"
          :x1="t.a.x"
          :y1="t.a.y"
          :x2="t.b.x"
          :y2="t.b.y"
          :class="{ long: t.long }"
        />
        <text
          v-for="l in numerals ? legends : []"
          :key="'l' + l.i"
          :x="l.x"
          :y="l.y"
          text-anchor="middle"
          dominant-baseline="central"
        >{{ l.label }}</text>
      </g>

      <g :transform="`rotate(${angle} 50 50)`">
        <circle cx="50" cy="50" :r="RADIUS" class="dbxknob__body" :style="cap ? { fill: cap } : null" />
        <circle cx="50" cy="50" :r="RADIUS" class="dbxknob__rim" />
        <!-- the original draws a line from the centre out; the 160A a short
             pointer near the rim, which is what each shows -->
        <line v-if="!cap" x1="50" y1="50" x2="50" :y2="50 - RADIUS + 2" class="dbxknob__index" />
        <rect v-else x="49.1" :y="50 - RADIUS + 3" width="1.8" height="10" rx="0.9" class="dbxknob__pointer" />
      </g>
    </svg>
    <div v-if="dragging" class="dbxknob__value">{{ p.text }}</div>
  </div>
</template>

<script setup>
/**
 * One of the Fairchild's three knob styles: a large black skirted knob with
 * a white pointer dot on the skirt and a smaller concentric black cap on
 * top, with its scale printed on the panel around it.
 *
 * **What the photograph shows and what it does not.** The dossier's 2.8 puts
 * the knob styles among the things it could not establish: the big knobs are
 * "large black skirted knobs with a white pointer dot on the skirt, and
 * there is a second, smaller concentric black cap on top of each; whether
 * that is a two-part knob or a knob plus a retaining cap I cannot tell from
 * a photograph". So this draws the two-part silhouette that both photographs
 * agree on and does not invent a knurl, a chamfer or a flute that neither
 * shows. The TIME CONSTANT control is a smaller skirted knob with a pointer
 * wing rather than a dot, which is `wing`.
 *
 * The scale is drawn as one short radial tick per detent with a number on
 * the ones that carry one, which is how the panel prints all three: 21
 * detents and 11 numbers on INPUT GAIN, 11 numbers on the continuous
 * THRESHOLD ring, 6 and 6 on TIME CONSTANT.
 *
 * Props: `p` (the handle, required), `marks` (`{ at, label }`), `sweep`
 * (degrees), `size`, `label`, `wing` (the small knob's pointer),
 * `reverse` (the scale runs anticlockwise, which INPUT GAIN's does because
 * its parameter is attenuation and the panel prints level).
 * Emits: nothing. Gestures come from the framework's `useKnobGesture`.
 */
import { computed } from 'vue';
import { useKnobGesture } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';

const props = defineProps({
  p: { type: Object, required: true },
  marks: { type: Array, default: () => [] },
  sweep: { type: Number, default: 300 },
  size: { type: [Number, String], default: 100 },
  label: { type: String, default: null },
  wing: { type: Boolean, default: false },
  /**
   * Radius of the printed number ring in the 100-unit box. The three
   * controls have three different rings — 3.1, 2.6 and 1.7 inch across
   * against knobs of 1.5, 1.5 and 0.9 — so it is measured per control
   * rather than shared.
   */
  ring: { type: Number, default: 43 },
  reverse: { type: Boolean, default: false },
  discrete: { type: Boolean, default: false },
});

/**
 * The skirt, the cap and the pointer, in a 100-unit box. The box is sized by
 * the caller so that the skirt lands on the diameter the panel measures: the
 * two big knobs are 1.5 inch across and the small one 0.9, and the skirt
 * here is 0.42 of the box.
 */
const SKIRT = 21;
const CAP = 11;
const DOT = 16;

const { handlers, dragging } = useKnobGesture(props.p, { rotation: true, discrete: props.discrete });

const width = computed(() => (typeof props.size === 'number' ? `${props.size}px` : props.size));

/** Travel to degrees, anticlockwise when the printed scale runs that way. */
const at = (t) => (props.reverse ? props.sweep / 2 - t * props.sweep : -props.sweep / 2 + t * props.sweep);
const angle = computed(() => at(Math.min(1, Math.max(0, props.p.norm))));

const place = (t, radius) => {
  const a = (at(t) * Math.PI) / 180;
  return { x: 50 + radius * Math.sin(a), y: 50 - radius * Math.cos(a) };
};

const ticks = computed(() =>
  props.marks.map((m, i) => ({
    i,
    a: place(m.at, props.ring - 9),
    b: place(m.at, props.ring - (m.label ? 4 : 6)),
    long: !!m.label,
  })),
);
const legends = computed(() =>
  props.marks.filter((m) => m.label).map((m, i) => ({ i, label: m.label, ...place(m.at, props.ring) })),
);
</script>

<template>
  <div class="fairknob" :style="{ width }">
    <svg
      viewBox="0 0 100 100"
      class="fairknob__dial"
      tabindex="0"
      role="slider"
      :aria-label="label || p.name"
      :aria-valuetext="p.text"
      v-on="handlers"
    >
      <!-- the scale, printed on the panel around the knob -->
      <g class="fairknob__marks">
        <line
          v-for="t in ticks"
          :key="'t' + t.i"
          :x1="t.a.x"
          :y1="t.a.y"
          :x2="t.b.x"
          :y2="t.b.y"
          :class="{ long: t.long }"
        />
        <text v-for="l in legends" :key="'l' + l.i" :x="l.x" :y="l.y" text-anchor="middle" dominant-baseline="central">
          {{ l.label }}
        </text>
      </g>

      <!-- the body: a black skirt carrying the index, a smaller cap on top -->
      <g :transform="`rotate(${angle} 50 50)`">
        <circle cx="50" cy="50" :r="SKIRT" class="fairknob__skirt" />
        <circle cx="50" cy="50" :r="SKIRT" class="fairknob__skirtrim" />
        <template v-if="wing">
          <path :d="`M 50 ${50 - SKIRT - 3} l 3.4 6 h -6.8 z`" class="fairknob__wing" />
        </template>
        <template v-else>
          <circle cx="50" :cy="50 - DOT" r="2.6" class="fairknob__dot" />
        </template>
        <circle cx="50" cy="50" :r="CAP" class="fairknob__cap" />
        <circle cx="50" cy="50" :r="CAP" class="fairknob__caprim" />
      </g>
    </svg>
    <div v-if="dragging" class="fairknob__value">{{ p.text }}</div>
  </div>
</template>

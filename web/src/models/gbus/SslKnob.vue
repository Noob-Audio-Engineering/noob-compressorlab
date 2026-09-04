<script setup>
/**
 * One of the module's six knobs: a blue cap on a dark metal skirt, a single
 * white radial index bar from the cap's centre to its rim, and a ring of
 * small white dots printed on the panel around it.
 *
 * **There are two knob parts here, not one part at two sizes.** THRESHOLD
 * and MAKE UP take a 0.102 W cap on a 0.126 W skirt; the four switches below
 * take 0.118 on 0.168. The skirt-to-cap ratios differ too, 1.23 against
 * 1.43, which is what rules out a single part scaled twice. Both are passed
 * in as fractions of the panel width so the caller keeps the measurements in
 * one place.
 *
 * The dots are the panel's silkscreen and not part of the knob: one per
 * position on a switch, eleven on a pot, over a 300 degree sweep with the
 * gap at six o'clock. On the switches the selected position also shows a
 * short dark bar cut into the skirt at that angle, which is how the module
 * shows its detent.
 *
 * Props: `p` (the handle, required), `marks` (`{ at, label }`, `at` a
 * fraction of travel), `sweep` (degrees), `cap` and `skirt` (diameters as
 * fractions of the panel width), `box` (the component's width as a fraction
 * of the panel width), `discrete`, `label`, and `extraMark` for the make-up
 * pot's `0`, which does not fall on a dot.
 * Emits: nothing. Gestures come from the framework's `useKnobGesture`.
 */
import { computed } from 'vue';
import { useKnobGesture } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';

const props = defineProps({
  p: { type: Object, required: true },
  marks: { type: Array, default: () => [] },
  sweep: { type: Number, default: 300 },
  cap: { type: Number, required: true },
  skirt: { type: Number, required: true },
  box: { type: Number, required: true },
  discrete: { type: Boolean, default: false },
  label: { type: String, default: null },
  extraMark: { type: Object, default: null },
});

const { handlers, dragging } = useKnobGesture(props.p, { discrete: props.discrete });

/* Radii in the 100-unit viewBox: a diameter that is `d` of the panel width
 * inside a box that is `box` of the panel width occupies `d / box` of the
 * box, so its radius is fifty times that. */
const capR = computed(() => (props.cap / props.box) * 50);
const skirtR = computed(() => (props.skirt / props.box) * 50);
/*
 * The printed dot ring, and the numbers outside it. The numbers sit ten
 * units out rather than thirteen because the rows are 0.181 of the panel
 * height apart and the caption above each knob has to clear the topmost
 * number: at thirteen the RELEASE and RATIO captions touched their own
 * twelve-o'clock legends.
 */
const dotR = computed(() => skirtR.value + 4.5);
const numR = computed(() => skirtR.value + 10);

const angle = computed(
  () => -props.sweep / 2 + Math.min(1, Math.max(0, props.p.norm)) * props.sweep,
);

const place = (at, radius) => {
  const a = ((-props.sweep / 2 + at * props.sweep) * Math.PI) / 180;
  return { x: 50 + radius * Math.sin(a), y: 50 - radius * Math.cos(a) };
};

const printed = computed(() => props.marks.map((m, i) => ({ i, ...place(m.at, dotR.value) })));

const legends = computed(() => {
  const out = props.marks
    .filter((m) => m.label)
    .map((m) => ({ label: m.label, ...place(m.at, numR.value) }));
  if (props.extraMark) {
    out.push({ label: props.extraMark.label, ...place(props.extraMark.at, numR.value) });
  }
  return out.map((l, i) => ({ i, ...l }));
});

/** The detent bar cut into a switch's skirt at the selected angle. */
const detent = computed(() => (props.discrete ? place(props.p.norm, skirtR.value - 2) : null));
</script>

<template>
  <div class="sslknob">
    <svg
      viewBox="0 0 100 100"
      class="sslknob__dial"
      tabindex="0"
      role="slider"
      :aria-label="label || p.name"
      :aria-valuetext="p.text"
      v-on="handlers"
    >
      <defs>
        <linearGradient :id="`sslCap${p.id}`" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stop-color="#6e93b6" />
          <stop offset="0.5" stop-color="#5884a9" />
          <stop offset="1" stop-color="#4a7091" />
        </linearGradient>
      </defs>

      <!-- the silkscreen: one dot per position, and the printed numbers -->
      <g class="sslknob__marks">
        <circle v-for="d in printed" :key="'d' + d.i" :cx="d.x" :cy="d.y" r="1.15" />
        <text
          v-for="l in legends"
          :key="'l' + l.i"
          :x="l.x"
          :y="l.y"
          text-anchor="middle"
          dominant-baseline="central"
        >{{ l.label }}</text>
      </g>

      <!-- the body -->
      <circle cx="50" cy="50" :r="skirtR" class="sslknob__skirt" />
      <circle v-if="detent" :cx="detent.x" :cy="detent.y" r="1.6" class="sslknob__detent" />
      <g :transform="`rotate(${angle} 50 50)`">
        <circle cx="50" cy="50" :r="capR" :fill="`url(#sslCap${p.id})`" />
        <circle cx="50" cy="50" :r="capR" class="sslknob__caprim" />
        <rect
          :x="49.3"
          :y="50 - capR + 1"
          width="1.4"
          :height="capR - 1"
          rx="0.7"
          class="sslknob__index"
        />
      </g>
    </svg>
    <div v-if="dragging" class="sslknob__value">{{ p.text }}</div>
  </div>
</template>

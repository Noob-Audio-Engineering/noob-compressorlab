<script setup>
/**
 * One of the TG12413's three rotary switches.
 *
 * Two knob families sit on the EMI desk and both are used here, because the
 * photographs of the restored TG12345 show them unambiguously even though
 * they show no limiter module: **black skirted knobs with a pointed
 * indicator and a bright metal centre boss**, and **red skirted knobs capped
 * with a cream pointer top**, which are the most distinctive objects on the
 * console. The mode switch gets the red one, because on a limiter the mode
 * is the control you want to find without looking; recovery and output level
 * get the black.
 *
 * **Everything on this module is a switch**, so the knob steps between
 * detents and the scale is drawn as one short radial tick per detent, with a
 * number only where the panel would print one. `discrete: true` on the
 * gesture is what makes it feel stepped rather than merely land on steps,
 * which section 2.8 asks for in as many words.
 *
 * The geometry below is not measured off anything. Section 2.9 says no
 * photograph of a bare TG12413 face exists, so the proportions are the ones
 * a 1960s console module is built with — a skirt about 1.5 knob-caps across,
 * ticks just outside it, numbers outside those — and the panel is laid out
 * from the control set rather than from an image.
 *
 * Props: `p` (the handle, required), `marks` (`{ at, label }`, `at` a
 * fraction of travel), `sweep` (degrees), `size`, `label`, `accent`.
 * Emits: nothing. Gestures come from the framework's `useKnobGesture`.
 */
import { computed } from 'vue';
import { useKnobGesture } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';

const props = defineProps({
  p: { type: Object, required: true },
  marks: { type: Array, default: () => [] },
  sweep: { type: Number, default: 150 },
  /** Fixed width; leave unset to let the stylesheet size it. */
  size: { type: [Number, String], default: null },
  label: { type: String, default: null },
  /** `'red'` for the console's cream-capped red knob, `'black'` otherwise. */
  accent: { type: String, default: 'black' },
});

/** The cap, the skirt, the tick ring and the number ring, in a 100-unit box. */
const CAP = 15;
const SKIRT = 23;
const TICK_IN = 25;
const TICK_OUT = 28;
const TICK_LONG = 30;
const NUM = 37;
/** Legends anchored to one side come in, so a long word stays on the strip. */
const NUM_SIDE = 32;
/** Teeth around the knurled skirt. */
const TEETH = 30;

const { handlers, dragging } = useKnobGesture(props.p, { discrete: true });

/*
 * Undefined when no size is given, so Vue leaves the inline style off and
 * the stylesheet decides. Returning '100%' instead put an inline width on
 * every switch, which beat the strip's own rule and overflowed the module
 * in a short window.
 */
const width = computed(() =>
  props.size == null ? undefined : typeof props.size === 'number' ? `${props.size}px` : props.size,
);

/*
 * Rotation follows travel, not the plain value: the output switch's plain
 * value is a decibel figure and the mode's is an index, and only `norm` is
 * linear in the angle the shaft turns through.
 */
const angle = computed(
  () => -props.sweep / 2 + Math.min(1, Math.max(0, props.p.norm)) * props.sweep,
);

const place = (at, radius) => {
  const a = ((-props.sweep / 2 + at * props.sweep) * Math.PI) / 180;
  return { x: 50 + radius * Math.sin(a), y: 50 - radius * Math.cos(a) };
};

const ticks = computed(() =>
  props.marks.map((m, i) => ({
    i,
    a: place(m.at, TICK_IN),
    b: place(m.at, m.label ? TICK_LONG : TICK_OUT),
    long: !!m.label,
  })),
);

/**
 * The printed legends.
 *
 * A number sits centred above its detent, but a **word** cannot: the mode
 * switch's three legends are COMPRESS, OUT and LIMIT, and centring all three
 * on a 92-pixel strip runs them straight through one another. So a legend
 * more than 30 degrees off vertical is anchored away from the knob and
 * pulled in a little, which is how a panel sets legends that flank a switch
 * rather than crowning it. The first draft centred every one and the mode
 * switch was unreadable at every window size.
 */
const numbers = computed(() =>
  props.marks
    .map((m, i) => {
      const deg = -props.sweep / 2 + m.at * props.sweep;
      const side = Math.abs(deg) > 30;
      const anchor = !side ? 'middle' : deg < 0 ? 'end' : 'start';
      return { i, label: m.label, anchor, ...place(m.at, side ? NUM_SIDE : NUM) };
    })
    .filter((m) => m.label),
);

const teeth = computed(() =>
  Array.from({ length: TEETH }, (_, i) => (i * 360) / TEETH),
);
</script>

<template>
  <div class="tgswitch" :style="{ width }">
    <svg
      viewBox="0 0 100 100"
      class="tgswitch__dial"
      tabindex="0"
      role="slider"
      :aria-label="label || p.name"
      :aria-valuetext="p.text"
      v-on="handlers"
    >
      <g class="tgswitch__scale">
        <line
          v-for="t in ticks"
          :key="'t' + t.i"
          :x1="t.a.x"
          :y1="t.a.y"
          :x2="t.b.x"
          :y2="t.b.y"
          :class="['tgswitch__tick', { 'tgswitch__tick--long': t.long }]"
        />
        <text
          v-for="m in numbers"
          :key="'n' + m.i"
          :x="m.x"
          :y="m.y"
          :text-anchor="m.anchor"
          dominant-baseline="central"
          class="tgswitch__num"
        >{{ m.label }}</text>
      </g>
      <g :transform="`rotate(${angle} 50 50)`">
        <circle cx="50" cy="50" :r="SKIRT" :class="['tgswitch__skirt', `tgswitch__skirt--${accent}`]" />
        <line
          v-for="(a, i) in teeth"
          :key="'k' + i"
          :x1="50"
          :y1="50 - SKIRT"
          :x2="50"
          :y2="50 - SKIRT + 2.6"
          class="tgswitch__tooth"
          :transform="`rotate(${a} 50 50)`"
        />
        <circle cx="50" cy="50" :r="CAP" :class="['tgswitch__cap', `tgswitch__cap--${accent}`]" />
        <!-- The pointed indicator: a wedge from the boss to the cap's edge. -->
        <path
          :d="`M 50 ${50 - CAP} L ${50 - 3.2} ${50 - 2} L ${50 + 3.2} ${50 - 2} Z`"
          :class="['tgswitch__pointer', `tgswitch__pointer--${accent}`]"
        />
        <circle cx="50" cy="50" r="3.4" class="tgswitch__boss" />
      </g>
    </svg>
    <div v-if="dragging" class="tgswitch__value">{{ p.text }}</div>
  </div>
</template>

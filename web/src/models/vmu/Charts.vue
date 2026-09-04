<script setup>
/**
 * The three blocks this model adds to its drawer: the state of the three
 * timing capacitors, and Fairchild's own two 1959 measurement charts with
 * the unit's live operating point on them.
 *
 * **One component, three parts, chosen by the `part` prop.** They began as
 * a single panel holding all three, which at 292 px wide drew the bars a
 * few pixels across and both charts as unreadable thumbnails. They are now
 * three panels of their own, each with its own title, which is what a user
 * asked for and the only way any of them is legible. The data and the
 * geometry stay here in one place rather than being copied three times, so
 * a curve transcribed once is drawn once.
 *
 * It is here because the dossier's 10.6 asks for exactly this and gives the
 * reason: **positions 5 and 6 are incomprehensible without seeing the
 * capacitors and obvious with it**, and putting the model's behaviour on top
 * of the manufacturer's own measurement is the single most educational thing
 * this plug-in could put on screen. The shared history and transfer panels
 * are untouched and identical under every model, as they must be; this sits
 * beside them the way the LA-2A's T4 panel does.
 *
 * **The grey curves are not drawn by the model.** They are the five
 * input/output curves from page 13 of the December 1959 manual and the seven
 * IM curves from the March 1959 chart, transcribed in the dossier's 7.2 and
 * 4.6 and held in `useVmu.js`. What moves is the dot: where the unit
 * actually is, read from the `meter` stream. So the panel shows the model
 * against the hardware rather than against itself.
 *
 * The IM chart's axes are Fairchild's: decibels of limiting across, per cent
 * IM up, one curve per output level. The dot's *height* is not a measurement
 * of this model's distortion — nothing measures that in real time — it sits
 * on the published curve for the output level in force, at the gain
 * reduction in force, which is what the chart says the hardware would do.
 * That is stated here and on the panel rather than left to look like a
 * read-out.
 */
import { computed } from 'vue';
import { useStreamFrame, useStreamValue } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';
import { PUBLISHED_CURVES, PUBLISHED_IM } from './useVmu.js';

const props = defineProps({
  /** Which block to draw: `network`, `io` or `im`. */
  part: { type: String, default: 'network' },
});

const cell = useStreamFrame('cell');
const inPeak = useStreamValue('meter', { index: 0, unit: 'linear', initial: 0 });
const outPeak = useStreamValue('meter', { index: 2, unit: 'linear', initial: 0 });
const gr = useStreamValue('meter', { index: 4, unit: 'db', initial: 0 });

/** 0 VU is +4 dBu at −18 dBFS, so a peak amplitude maps to dBm like this. */
const VU_REF_AMP = 0.12589254 * Math.SQRT2;
const dbm = (a) => (a > 1e-9 ? 20 * Math.log10(a / VU_REF_AMP) + 4 : -60);

/**
 * The three timing capacitors. The first bar is the control voltage itself,
 * over the fifty volts the sidechain can develop; the other two are how full
 * each slow leg is relative to the node it hangs on, which is what decides
 * whether the release is the fast one or the slow one.
 */
const bars = computed(() => {
  const f = cell.value || [0, 0, 0];
  return [
    { label: 'C_T', v: Math.min(1, (f[0] || 0) / 50), cls: 'ct', hint: 'the control voltage' },
    { label: 'C_U', v: Math.min(1, f[1] || 0), cls: 'cu', hint: '8 µF behind 100 kΩ' },
    { label: 'C_V', v: Math.min(1, f[2] || 0), cls: 'cv', hint: '20 µF behind 100 kΩ' },
  ];
});
const pct = (v) => `${Math.round(Math.min(1, Math.max(0, v)) * 100)}%`;

// --- the input/output chart -------------------------------------------------

const IO = { x0: -10, x1: 25, y0: -10, y1: 25, w: 200, h: 130 };
const iox = (v) => ((v - IO.x0) / (IO.x1 - IO.x0)) * IO.w;
const ioy = (v) => IO.h - ((v - IO.y0) / (IO.y1 - IO.y0)) * IO.h;
const ioPath = (points) => points.map(([x, y], i) => `${i ? 'L' : 'M'} ${iox(x).toFixed(1)} ${ioy(y).toFixed(1)}`).join(' ');
const curves = PUBLISHED_CURVES.map((c) => ({ ...c, d: ioPath(c.points) }));
const live = computed(() => ({
  x: iox(Math.max(IO.x0, Math.min(IO.x1, dbm(inPeak.value)))),
  y: ioy(Math.max(IO.y0, Math.min(IO.y1, dbm(outPeak.value)))),
  show: inPeak.value > 1e-5,
}));

// --- the IM chart -----------------------------------------------------------

const IM = { x1: 20, y1: 12, w: 200, h: 130 };
const imx = (v) => (v / IM.x1) * IM.w;
const imy = (v) => IM.h - (Math.min(v, IM.y1) / IM.y1) * IM.h;
/**
 * Each published curve is drawn through its two read points — the per cent
 * at zero limiting and the decibels at which it reaches nine per cent —
 * as the exponential the chart's own curves are.
 */
const imPath = (c) => {
  const k = Math.log(9 / c.at0) / c.at9pct;
  const pts = [];
  for (let i = 0; i <= 24; i++) {
    const x = (i / 24) * IM.x1;
    const y = c.at0 * Math.exp(k * x);
    if (y > IM.y1) break;
    pts.push(`${i ? 'L' : 'M'} ${imx(x).toFixed(1)} ${imy(y).toFixed(1)}`);
  }
  return pts.join(' ');
};
const imCurves = PUBLISHED_IM.map((c) => ({ ...c, d: imPath(c) }));
/** Which published curve the unit is on, and how far along it. */
const imLive = computed(() => {
  const out = dbm(outPeak.value);
  const g = Math.min(IM.x1, Math.max(0, -gr.value));
  const near = PUBLISHED_IM.reduce((a, b) => (Math.abs(b.out - out) < Math.abs(a.out - out) ? b : a));
  const k = Math.log(9 / near.at0) / near.at9pct;
  const y = near.at0 * Math.exp(k * g);
  return { x: imx(g), y: imy(y), out: near.out, pct: y, show: outPeak.value > 1e-5 && y <= IM.y1 };
});
</script>

<template>
  <div class="fairchart" :class="`fairchart--${props.part}`">
    <template v-if="props.part === 'network'">
      <div class="bench-label mb-2">The timing network</div>
      <div class="fairchart__bars">
        <div v-for="b in bars" :key="b.label" class="fairchart__bar">
          <div class="bar" :class="b.cls"><div class="fill" :style="{ height: pct(b.v) }"></div></div>
          <div class="fairchart__barlabel">{{ b.label }}</div>
          <div class="fairchart__barhint">{{ b.hint }}</div>
        </div>
      </div>
      <p class="fairchart__note">
        The two slow legs charge on their own clock. Empty, they pull the release fast; full, they hold the
        node up for tens of seconds. That is the whole of positions 5 and 6.
      </p>
    </template>

    <template v-else-if="props.part === 'io'">
      <div class="bench-label mb-2">Input against output, December 1959</div>
      <div class="fairchart__plot">
        <svg :viewBox="`-22 -8 ${IO.w + 34} ${IO.h + 26}`" role="img" aria-label="Published input versus output curves with the live operating point">
          <g class="grid">
            <line v-for="v in [-10, 0, 10, 20]" :key="'gx' + v" :x1="iox(v)" :y1="0" :x2="iox(v)" :y2="IO.h" />
            <line v-for="v in [-10, 0, 10, 20]" :key="'gy' + v" :x1="0" :y1="ioy(v)" :x2="IO.w" :y2="ioy(v)" />
          </g>
          <path v-for="c in curves" :key="c.id" :d="c.d" class="pub" />
          <g class="axis">
            <text v-for="v in [-10, 0, 10, 20]" :key="'lx' + v" :x="iox(v)" :y="IO.h + 12" text-anchor="middle">{{ v }}</text>
            <text v-for="v in [0, 10, 20]" :key="'ly' + v" :x="-4" :y="ioy(v)" text-anchor="end" dominant-baseline="central">{{ v }}</text>
            <text :x="IO.w / 2" :y="IO.h + 24" text-anchor="middle">dBm in</text>
          </g>
          <circle v-if="live.show" class="live" :cx="live.x" :cy="live.y" r="4" />
        </svg>
      </div>
      <p class="fairchart__note">
        Grey is Fairchild&rsquo;s, measured in 1959; the dot is where this unit is now.
      </p>
    </template>

    <template v-else>
      <div class="bench-label mb-2">Intermodulation against limiting, March 1959</div>
      <div class="fairchart__plot">
        <svg :viewBox="`-22 -8 ${IM.w + 34} ${IM.h + 26}`" role="img" aria-label="Published intermodulation curves with the live operating point">
          <g class="grid">
            <line v-for="v in [0, 5, 10, 15, 20]" :key="'ix' + v" :x1="imx(v)" :y1="0" :x2="imx(v)" :y2="IM.h" />
            <line v-for="v in [0, 5, 10]" :key="'iy' + v" :x1="0" :y1="imy(v)" :x2="IM.w" :y2="imy(v)" />
          </g>
          <path v-for="c in imCurves" :key="c.out" :d="c.d" class="pub" />
          <g class="axis">
            <text v-for="v in [0, 5, 10, 15, 20]" :key="'ax' + v" :x="imx(v)" :y="IM.h + 12" text-anchor="middle">{{ v }}</text>
            <text v-for="v in [5, 10]" :key="'ay' + v" :x="-4" :y="imy(v)" text-anchor="end" dominant-baseline="central">{{ v }}%</text>
            <text :x="IM.w / 2" :y="IM.h + 24" text-anchor="middle">dB of limiting</text>
          </g>
          <circle v-if="imLive.show" class="live" :cx="imLive.x" :cy="imLive.y" r="4" />
        </svg>
      </div>
      <p class="fairchart__note">
        The dot sits on the published curve for the output level in force, at the limiting in force &mdash; what
        the hardware did &mdash; rather than on a measurement of our own.
      </p>
    </template>
  </div>
</template>

<script setup>
/**
 * The Fairchild 670 front panel, laid out from the measurements in
 * `research/Fairchild-670.md` section 2.2, which were taken off
 * `ref/fairchild-670-panel-inch-grid.png` — a photograph of a real unit
 * rectified with a projective homography onto a 19 × 14 inch rectangle at
 * 100 pixels to the inch — rather than from prose.
 *
 * **The panel is 19 × 14 inches, which is 8U**, from the manual's own
 * specification page. Sound On Sound, Vintage King and Heritage Audio all
 * say 6U and all three are wrong about the original; the manual's line is
 * OCR'd as `1)"`, `1h!` and `1h"` in three separate scans of the same
 * document and the glyph is a 4. It is by a long way the tallest face in
 * this lab, and the view below caps it from the window height accordingly.
 *
 * **How much to trust the positions.** The rectification is anchored on
 * corners located to about ±0.1 inch, and the right-hand side of the plate
 * was the far side of the shot. Positions left of x = 16 inches are good to
 * roughly ±0.15 inch; the two TIME CONSTANT columns at the right edge to
 * about ±0.3. Every fraction below is that measurement divided by 19 or 14.
 *
 * **Two deliberate asymmetries, kept because the metal has them.** The TIME
 * CONSTANT knobs are noticeably smaller than the others and sit vertically
 * offset from their own rows — the upper one about 0.6 inch high and the
 * lower one 0.8 inch low — which opens the gap in the right-hand column that
 * the AGC toggle sits in. And the two channels' controls are *not* on
 * identical x centres: the lower row sits between 0.1 and 0.4 inch to the
 * right of the upper. On a hand-drilled 1959 panel that is tolerance rather
 * than intent, but it is what the metal does and reproducing it is free.
 *
 * **The 660 is drawn as one row of the 670 on a shorter panel, and this says
 * so.** The dossier found no photograph square-on enough to measure a 660
 * and recommends exactly that (2.7). What the Abbey Road photograph does
 * confirm is the family resemblance and the differences: one meter rather
 * than two, the same BAL/ZERO/BAL lever, the same three controls in the same
 * order, ON at the left, the same script logo at the bottom right, and no
 * AGC toggle because there is nothing to matrix.
 *
 * The engraving says **LONG ISLAND CITY**, which is what is on the metal and
 * on the manual's cover; the drawing office was in Whitestone and the
 * component-layout drawing's title block says so, but the panel does not.
 *
 * Reads: every `fc_` handle. Streams: `meter`, through the two movements.
 */
import { computed } from 'vue';
import {
  INPUT_GAIN_MARKS,
  SWEEP,
  THRESHOLD_MARKS,
  TIME_MARKS,
  useControls,
  usePower,
} from './useVmu.js';
import FairKnob from './FairKnob.vue';
import FairLever from './FairLever.vue';
import FairScrew from './FairScrew.vue';
import FairToggle from './FairToggle.vue';
import VuFaceFair from './VuFaceFair.vue';

const c = useControls();
const power = usePower();
const lit = computed(() => power.on.value && !c.bypass.on);
const mono = computed(() => !!c.unit && c.unit.index === 0);

/** Inches to fractions of the plate. On the 660 the panel is half as tall. */
const H = computed(() => (mono.value ? 7 : 14));
const fx = (x) => x / 19;
const fy = (y) => y / H.value;
const at = (x, y) => ({ left: `${fx(x) * 100}%`, top: `${fy(y) * 100}%` });
const box = (x0, y0, x1, y1) => ({
  left: `${fx(x0) * 100}%`,
  top: `${fy(y0) * 100}%`,
  width: `${fx(x1 - x0) * 100}%`,
  height: `${fy(y1 - y0) * 100}%`,
});

/**
 * The two channel rows, in inches down the 670's panel. Each carries its own
 * measured y values rather than one offset applied twice, because the
 * captions and the TIME CONSTANT knob do not mirror.
 */
const ROWS = computed(() => {
  const upper = {
    n: 0,
    name: 'LEFT',
    lat: 'LEFT-LAT',
    channel: 'LEFT CHANNEL',
    cap1: 0.78,
    cap2: 1.08,
    arc: 2.42,
    lever: [7.3, 3.2],
    zero: [6.2, 6.2],
    bal: [8.35, 6.25],
    screwCap: 5.75,
    gain: [10.6, 3.4],
    thresh: [13.9, 3.5],
    time: [17.2, 2.8],
    timeCap1: 0.78,
    timeCap2: 1.08,
    meter: [1.725, 0.875, 5.675, 5.925],
    dial: [2.175, 1.4, 5.225, 3.9],
  };
  if (mono.value) return [upper];
  return [
    upper,
    {
      n: 1,
      name: 'RIGHT',
      lat: 'RIGHT-VERT',
      channel: 'RIGHT CHANNEL',
      cap1: 7.48,
      cap2: 7.78,
      arc: 8.82,
      lever: [7.5, 9.6],
      zero: [6.3, 13.0],
      bal: [8.5, 13.05],
      screwCap: 12.55,
      gain: [10.7, 9.7],
      thresh: [14.1, 9.7],
      time: [17.6, 10.5],
      timeCap1: 8.95,
      timeCap2: 9.25,
      meter: [1.775, 7.8, 5.625, 12.9],
      dial: [2.175, 8.35, 5.225, 10.85],
    },
  ];
});

/**
 * Knob box widths, as fractions of the panel. The skirt is 0.42 of its own
 * box, so a 1.5 inch knob wants a 3.57 inch box and a 0.9 inch knob a 2.14
 * inch one; the printed number rings then sit where they were measured, at
 * 3.1, 2.6 and 1.7 inches across.
 */
const BIG = `${(3.57 / 19) * 100}cqw`;
const SMALL = `${(2.14 / 19) * 100}cqw`;
const RING = { gain: 43, thresh: 36, time: 40 };
/** The screwdriver heads measure 0.35 inch across in a 0.6 inch box. */
const SCREW = `${(0.6 / 19) * 100}cqw`;
</script>

<template>
  <section class="facefair" :class="{ unlit: !lit, mono }">
    <div class="facefair__ear left"><span class="slot"></span><span class="slot"></span></div>
    <div class="facefair__plate" :style="{ aspectRatio: `19 / ${H}` }">
      <div class="facefair__panel">
        <!-- the mains toggle, top left, and the fuse below the meters -->
        <div class="abs facefair__on" :style="at(0.9, mono ? 2.0 : 3.3)">
          <div class="facefair__onlabel">ON</div>
          <FairToggle vertical label="Power" :fallback="power" />
        </div>
        <span class="facefair__screw abs" :style="at(0.85, mono ? 5.3 : 6.7)"></span>
        <div class="abs facefair__fuse" :style="at(0.95, mono ? 6.1 : 10.2)">
          <span class="facefair__fusecap">FUSE</span>
          <span class="facefair__fuseamp">5A</span>
        </div>

        <template v-for="row in ROWS" :key="row.n">
          <!-- the meter, recessed in its moulded bezel -->
          <div class="facefair__bezel abs" :style="box(...row.meter)"></div>
          <div class="facefair__dial abs" :style="box(...row.dial)">
            <VuFaceFair :lit="lit" :label="`${row.lat} meter`" />
          </div>
          <span class="facefair__screw abs" :style="at((row.meter[0] + row.meter[2]) / 2, row.dial[3] + 1.1)"></span>

          <!-- METERING: a lever, not a knob, with BAL ZERO BAL arced above -->
          <div class="facefair__cap abs" :style="at(row.lever[0], row.cap1)">{{ row.lat }}</div>
          <div class="facefair__cap abs" :style="at(row.lever[0], row.cap2)">METERING</div>
          <div class="abs facefair__lever" :style="at(row.lever[0], row.lever[1])">
            <FairLever :p="c.meter[row.n]" />
          </div>

          <!-- the two front-panel screwdriver adjustments -->
          <div class="facefair__mini abs" :style="at(row.zero[0] + 0.05, row.screwCap)">ZERO</div>
          <div class="abs" :style="at(...row.zero)">
            <FairScrew :p="c.zero[row.n]" :size="SCREW" />
          </div>
          <div class="facefair__mini abs" :style="at(row.bal[0] + 0.05, row.screwCap)">BAL</div>
          <div class="abs" :style="at(...row.bal)">
            <FairScrew :p="c.balance[row.n]" :size="SCREW" />
          </div>

          <!-- INPUT GAIN: a step attenuator, 1 dB a detent -->
          <div class="facefair__cap abs" :style="at(row.gain[0], row.cap1)">{{ row.channel }}</div>
          <div class="facefair__cap abs" :style="at(row.gain[0], row.cap2)">INPUT GAIN</div>
          <div class="abs" :style="at(...row.gain)">
            <FairKnob
              :p="c.inputGain[row.n]"
              :marks="INPUT_GAIN_MARKS"
              :sweep="SWEEP.inputGain"
              :ring="RING.gain"
              :size="BIG"
              reverse
              discrete
              :label="`${row.channel} input gain`"
            />
          </div>

          <!-- THRESHOLD: printed 0 to 10, and not decibels -->
          <div class="facefair__cap abs" :style="at(row.thresh[0], row.cap1)">{{ row.lat }}</div>
          <div class="facefair__cap abs" :style="at(row.thresh[0], row.cap2)">THRESHOLD</div>
          <div class="abs" :style="at(...row.thresh)">
            <FairKnob
              :p="c.threshold[row.n]"
              :marks="THRESHOLD_MARKS"
              :sweep="SWEEP.threshold"
              :ring="RING.thresh"
              :size="BIG"
              :label="`${row.lat} threshold`"
            />
          </div>

          <!-- TIME CONSTANT: smaller, and offset from its own row -->
          <div class="facefair__cap abs" :style="at(row.time[0], row.timeCap1)">{{ row.lat }}</div>
          <div class="facefair__cap abs" :style="at(row.time[0], row.timeCap2)">TIME CONSTANT</div>
          <div class="abs" :style="at(...row.time)">
            <FairKnob
              :p="c.time[row.n]"
              :marks="TIME_MARKS"
              :sweep="SWEEP.time"
              :ring="RING.time"
              :size="SMALL"
              wing
              discrete
              :label="`${row.lat} time constant`"
            />
          </div>
        </template>

        <!-- the matrix switch, alone in the gap between the two rows -->
        <div v-if="!mono && c.agc" class="abs facefair__agc" :style="at(17.3, 6.0)">
          <FairToggle :p="c.agc" up="LEFT&#10;RIGHT" down="LAT&#10;VERT" label="AGC" />
        </div>

        <!-- the maker's name across the bottom right -->
        <div class="abs facefair__logo" :style="at(11.65, mono ? 6.1 : 12.55)">Noob</div>
        <div class="abs facefair__address" :style="at(13.6, mono ? 6.05 : 12.5)">
          <span>RECORDING EQPT. CORP.</span>
          <span>LONG ISLAND CITY 1, N.Y.</span>
        </div>
        <div class="abs facefair__model" :style="at(15.15, mono ? 6.15 : 12.6)">
          MODEL {{ mono ? '660' : '670' }}
        </div>
      </div>
    </div>
    <div class="facefair__ear right"><span class="slot"></span><span class="slot"></span></div>
  </section>
</template>

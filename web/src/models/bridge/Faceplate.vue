<script setup>
/**
 * The Neve 33609/N front panel, laid out from measurements I took off
 * `ref/neve-33609n-front.jpg` (2000 × 512) rather than from the research's
 * prose, and checked against `research/Neve-33609.md` section 2.2.
 *
 * How the numbers below were found: the panel is the blue-grey field in that
 * photograph, so thresholding on it gives a bounding box of x 28–1969,
 * y 130–478, that is 1942 × 349 px, an aspect of 5.56 against a 19 inch 2U
 * panel's 5.43. The excess is the chassis lip the photograph includes above
 * and below, so 2U is right, as the dossier says. Every fraction here is of
 * that box. The white rules came from columns and rows that are bright over
 * most of their length; the knob centres from the blue caps' extent, column
 * by column; the silkscreen rows from runs of bright pixels inside each
 * block; the meters from the dark rectangles and a circle fitted to their
 * tick marks.
 *
 * **Three things the dossier or my first pass had wrong.**
 *
 * 1. *The knob skirt is grey, not black.* The dossier's colour table gave
 *    #1B1F20, "near-black, knurled". Sampling radially out from a knob
 *    centre gives the blue cap to 0.9 of the cap radius, then #9BA2A1 at
 *    1.15 and #606B6F from 1.3 to 1.6, and the close crop shows an
 *    unmistakably bright aluminium knurl. The skirts here are grey and
 *    section 2.4 of the dossier now records the correction.
 *
 * 2. *The block rules are at 0.039 and 0.959 of the panel height*, not the
 *    0.075 and 0.925 I first drew. The blocks are taller than I had them.
 *
 * 3. *The two channel rows are not vertical mirrors of one another.* Their
 *    blocks are the same height to within a thousandth, but the lower
 *    block's contents sit higher inside it: its captions are 0.029 below its
 *    top rule where the upper block's are 0.085, while both titles sit 0.033
 *    above their bottom rule. The knob rows are 0.4217 apart and the rules
 *    0.4756. So the rows carry their own measured y values here rather than
 *    one shared offset.
 *
 * **Both channel rows drive the same parameters.** The hardware has two
 * complete channels; the model has one set of controls, so the rows are
 * permanently ganged and moving a knob in one moves its twin in the other.
 * That is a 33609 with its channels linked, which is what one parameter set
 * can honestly show, and it is said here rather than hidden.
 *
 * Reads: every `neve_` handle plus the shared `bypass` and `link`. Streams:
 * `meter`, through this model's two gain-reduction movements.
 */
import { computed } from 'vue';
import {
  COMPRESS_RECOVERY_MARKS,
  COMPRESS_THRESHOLD_MARKS,
  GAIN_MARKS,
  LIMIT_RECOVERY_MARKS,
  LIMIT_THRESHOLD_MARKS,
  RATIO_MARKS,
  SWEEP,
  useControls,
  usePower,
  ui,
} from './useBridge.js';
import NeveKnob from './NeveKnob.vue';
import NeveLever from './NeveLever.vue';
import NeveMeter from './NeveMeter.vue';
import NevePower from './NevePower.vue';

const c = useControls();
const power = usePower();
const lit = computed(() => power.on.value);

/** Fractions of the panel → CSS, centred on the point. */
const at = (x, y) => ({ left: `${x * 100}%`, top: `${y * 100}%` });
/** A box given its edges as fractions. */
const box = (x0, y0, x1, y1) => ({
  left: `${x0 * 100}%`,
  top: `${y0 * 100}%`,
  width: `${(x1 - x0) * 100}%`,
  height: `${(y1 - y0) * 100}%`,
});

/** The measured knob columns. */
const LIM_X = [0.0991, 0.1592];
const COMP_X = [0.3027, 0.3612, 0.4190, 0.4774];
/** The levers, after the knobs in each block. */
const LIM_LEV = [0.2160, 0.2470];
const COMP_LEV = [0.5270, 0.5580];
/** The centre block's two lever columns. */
const SW_LEV = [0.5985, 0.6315];

/** A knob cap is 0.0257 of the panel width, and 0.34 of this box. */
const KNOB = '7.4cqw';

/**
 * The two channels: caption row, knob row, block title and centre-block
 * lever, each measured in its own half of the panel rather than mirrored.
 */
const CHANNELS = [
  { n: 1, cap: 0.1232, y: 0.3195, title: 0.4505, lev: 0.2850 },
  { n: 2, cap: 0.5430, y: 0.7412, title: 0.9269, lev: 0.7050 },
];
</script>

<template>
  <section class="faceneve" :class="{ unlit: !lit }">
    <div class="faceneve__ear left"><span class="slot"></span><span class="slot"></span></div>
    <div class="faceneve__plate">
      <div class="faceneve__panel">
        <!-- the four fixing screws, inboard of the ears -->
        <span class="screwneve abs" :style="at(0.0455, 0.20)"></span>
        <span class="screwneve abs" :style="at(0.0455, 0.80)"></span>
        <span class="screwneve abs" :style="at(0.9545, 0.20)"></span>
        <span class="screwneve abs" :style="at(0.9545, 0.80)"></span>

        <!-- the seven bordered blocks, on the measured rules -->
        <div class="faceneve__block" :style="box(0.0669, 0.0387, 0.2616, 0.4842)"></div>
        <div class="faceneve__block" :style="box(0.0669, 0.5143, 0.2616, 0.9585)"></div>
        <div class="faceneve__block" :style="box(0.2670, 0.0387, 0.5778, 0.4842)"></div>
        <div class="faceneve__block" :style="box(0.2670, 0.5143, 0.5778, 0.9585)"></div>
        <div class="faceneve__block" :style="box(0.5832, 0.0387, 0.6578, 0.9585)"></div>
        <div class="faceneve__block" :style="box(0.6630, 0.0387, 0.8777, 0.9585)"></div>
        <div class="faceneve__block" :style="box(0.8829, 0.0387, 0.9328, 0.9585)"></div>

        <template v-for="ch in CHANNELS" :key="ch.n">
          <!-- LIMIT block -->
          <div class="faceneve__cap abs" :style="at(LIM_X[0], ch.cap)">threshold dBu</div>
          <div class="abs" :style="at(LIM_X[0], ch.y)">
            <NeveKnob :p="c.limitThreshold" :marks="LIMIT_THRESHOLD_MARKS" :sweep="SWEEP.limitThreshold" :size="KNOB" :label="`Limit ${ch.n} threshold`" />
          </div>
          <div class="faceneve__cap abs" :style="at(LIM_X[1], ch.cap)">recovery ms</div>
          <div class="abs" :style="at(LIM_X[1], ch.y)">
            <NeveKnob :p="c.limitRecovery" :marks="LIMIT_RECOVERY_MARKS" :sweep="SWEEP.recovery" :size="KNOB" :label="`Limit ${ch.n} recovery`" />
          </div>
          <div class="abs" :style="at(LIM_LEV[0], ch.y)">
            <NeveLever v-if="c.limitIn" :p="c.limitIn" up="limit" down="in" />
          </div>
          <div class="abs" :style="at(LIM_LEV[1], ch.y)">
            <NeveLever v-if="c.limitAttack" :p="c.limitAttack" up="attack&#10;fast" down="slow" />
          </div>
          <div class="faceneve__title abs" :style="at(0.1504, ch.title)">LIMIT {{ ch.n }}</div>

          <!-- COMPRESS block -->
          <div class="faceneve__cap abs" :style="at(COMP_X[0], ch.cap)">threshold dBu</div>
          <div class="abs" :style="at(COMP_X[0], ch.y)">
            <NeveKnob :p="c.compressThreshold" :marks="COMPRESS_THRESHOLD_MARKS" :sweep="SWEEP.compressThreshold" :size="KNOB" :label="`Compress ${ch.n} threshold`" />
          </div>
          <div class="faceneve__cap abs" :style="at(COMP_X[1], ch.cap)">recovery ms</div>
          <div class="abs" :style="at(COMP_X[1], ch.y)">
            <NeveKnob :p="c.compressRecovery" :marks="COMPRESS_RECOVERY_MARKS" :sweep="SWEEP.recovery" :size="KNOB" :label="`Compress ${ch.n} recovery`" />
          </div>
          <div class="faceneve__cap abs" :style="at(COMP_X[2], ch.cap)">gain</div>
          <div class="abs" :style="at(COMP_X[2], ch.y)">
            <NeveKnob :p="c.gain" :marks="GAIN_MARKS" :sweep="SWEEP.gain" :size="KNOB" :label="`Compress ${ch.n} make-up gain`" />
          </div>
          <div class="faceneve__cap abs" :style="at(COMP_X[3], ch.cap)">ratio</div>
          <div class="abs" :style="at(COMP_X[3], ch.y)">
            <NeveKnob :p="c.compressRatio" :marks="RATIO_MARKS" :sweep="SWEEP.ratio" :size="KNOB" :label="`Compress ${ch.n} ratio`" />
          </div>
          <div class="abs" :style="at(COMP_LEV[0], ch.y)">
            <NeveLever v-if="c.compressIn" :p="c.compressIn" up="compress" down="in" />
          </div>
          <div class="abs" :style="at(COMP_LEV[1], ch.y)">
            <NeveLever v-if="c.compressAttack" :p="c.compressAttack" up="attack&#10;fast" down="slow" />
          </div>
          <div class="faceneve__title abs" :style="at(0.3919, ch.title)">COMPRESS {{ ch.n }}</div>
        </template>

        <!-- centre block: bypass and mono/stereo above, bypass and external below -->
        <div class="abs" :style="at(SW_LEV[0], CHANNELS[0].lev)">
          <NeveLever :p="c.bypass" up="bypass" down="in" lamps />
        </div>
        <div class="abs" :style="at(SW_LEV[1], CHANNELS[0].lev)">
          <NeveLever :p="c.link" up="mono" down="stereo" lamps />
        </div>
        <div class="abs" :style="at(SW_LEV[0], CHANNELS[1].lev)">
          <NeveLever :p="c.bypass" up="bypass" down="in" lamps />
        </div>
        <div class="abs faceneve__ext" :style="at(SW_LEV[1], CHANNELS[1].lev)">
          <div class="nevelev__leg up"><span>external</span></div>
          <div class="nevelev__row">
            <button class="nevelev__body" type="button" role="switch" :aria-checked="ui.external" aria-label="External control" @click="ui.external = !ui.external">
              <span class="nevelev__bat" :class="ui.external ? 'up' : 'down'"><i class="nevelev__paddle"></i></span>
            </button>
            <span class="nevelev__lamps"><i :class="{ lit: ui.external }"></i><i :class="{ lit: !ui.external }"></i></span>
          </div>
          <div class="nevelev__leg down"><span>internal</span><span>control</span></div>
        </div>

        <!-- meter block: the two staggered movements, the mark, the model line -->
        <div class="faceneve__cap abs" :style="at(0.7189, 0.0960)">gain reduction 1</div>
        <div class="abs faceneve__meter" :style="box(0.6730, 0.1719, 0.7647, 0.6246)">
          <NeveMeter label="gain reduction 1" :lit="lit" />
        </div>
        <div class="faceneve__cap abs" :style="at(0.8229, 0.3700)">gain reduction 2</div>
        <div class="abs faceneve__meter" :style="box(0.7765, 0.4441, 0.8692, 0.8911)">
          <NeveMeter label="gain reduction 2" :lit="lit" />
        </div>
        <div class="abs faceneve__mark" :style="at(0.8226, 0.1530)">
          <span class="faceneve__bolt">N</span><span class="faceneve__word">Noob</span>
        </div>
        <div class="abs faceneve__ident" :style="at(0.6660, 0.8374)">
          <span>NOOB 33609 DISCRETE</span>
          <span>PRECISION STEREO</span>
          <span>LIMITER/COMPRESSOR</span>
        </div>

        <!-- mains -->
        <div class="abs" :style="at(0.9109, 0.5088)"><NevePower /></div>
      </div>
    </div>
    <div class="faceneve__ear right"><span class="slot"></span><span class="slot"></span></div>
  </section>
</template>

<script setup>
/**
 * The SSL 4000 G bus compressor's 500-series module face, laid out from the
 * measured fractions in `research/SSL-Gbus.md` section 2.2 rather than from
 * its prose, and coloured from the medians in section 2.3.
 *
 * **Why this panel and not the console's.** SSL publish a high-resolution
 * product render of the module and a dimensioned recall sheet of the same
 * panel with every legend and every detent dot on it. No photograph of the
 * G Series console's centre section is legible enough to draw from, and the
 * dossier says so plainly: drawing a panel I cannot see and inventing its
 * silkscreen is exactly the failure this repository's research standard
 * exists to prevent. So the module is drawn, and the *values* on its
 * switches follow the console, because card 82E27 gives the console's
 * component values and nothing gives the module's. That is the dossier's
 * own instruction in section 2.5, and it is said again in `useGbus.js` and
 * in the README.
 *
 * **The proportions check themselves.** The face measures 1429 x 2528 px in
 * the render, an aspect of 1 : 1.769. A double 500-series slot is 3.0 x 5.25
 * inches, 1 : 1.75, so the measurement lands within 1 % of a real double
 * slot and the module is two slots wide. Nothing was fitted to make that
 * come out.
 *
 * **Two knob parts, not one at two sizes.** The pots take a 0.102 W cap on a
 * 0.126 W skirt and the switches 0.118 on 0.168, so the skirt-to-cap ratios
 * are 1.23 and 1.43. SSL used Sifam knobs and a Sifam AL29 movement; the
 * clone builder gives Farnell 969-746 and RS 225-704 as the current
 * catalogue equivalents.
 *
 * It is an affectionate spoof, so the panel says NOOB where the metal says
 * SSL. Everything else on it is transcribed as printed.
 *
 * Reads: every `ssl_` handle on the hardware's face. Streams: `meter`,
 * through `SslMeter.vue`.
 */
import { computed } from 'vue';
import {
  ATTACK_MARKS,
  COL,
  ESCUTCHEON,
  GLASS,
  HPF_MARKS,
  IN_CAP,
  MAKEUP_MARKS,
  MAKEUP_ZERO_AT,
  OUTLINE,
  POT,
  RATIO_MARKS,
  RELEASE_MARKS,
  ROW,
  SCREW_D,
  SCREW_X,
  SCREW_Y,
  SWEEP,
  SWITCH,
  THRESHOLD_MARKS,
  ZERO_SCREW,
  useControls,
} from './useGbus.js';
import SslKnob from './SslKnob.vue';
import SslMeter from './SslMeter.vue';

const c = useControls();

/** Centre something on the panel at a fraction of its width and height. */
const at = (x, y) => ({ left: `${x * 100}%`, top: `${y * 100}%` });
/*
 * A knob's box, centred on the panel. The width has to be the knob's own
 * fraction of the panel: a box declared 100 % wide and pulled back by half
 * of itself reaches 0.5 of the panel either side of its centre, which made
 * the panel scroll horizontally by 32 px at the largest window size.
 */
const knobAt = (x, y, k) => ({ ...at(x, y), width: `${k.box * 100}%` });
/** A box from two corners, both as fractions. */
const box = ([x0, y0, x1, y1]) => ({
  left: `${x0 * 100}%`,
  top: `${y0 * 100}%`,
  width: `${(x1 - x0) * 100}%`,
  height: `${(y1 - y0) * 100}%`,
});

/**
 * The caption above a knob, and its unit below, as offsets in panel-height
 * fractions from the knob's centre.
 *
 * **The unit sits inside the dot ring, in the gap at six o'clock.** The
 * dots run over 300 degrees with a 60 degree gap at the bottom, and that
 * gap is what the `dB`, `ms`, `s` and `Hz` legends are for: there is no
 * room for them outside the ring, because the rows are only 0.181 of the
 * panel height apart and the bottom row would push its legend into the
 * strap line. Putting them in the gap is also what the module does.
 *
 * The caption clears the topmost printed number, which reaches 0.064 out
 * once the knob's numbers were brought in.
 */
const CAP_DY = 0.085;
const UNIT_DY = 0.06;

/** The panel is dark when the plug-in's own bypass is engaged. */
const lit = computed(() => !(c.bypass && c.bypass.on));
</script>

<template>
  <section class="facessl" :class="{ unlit: !lit }">
    <div class="facessl__panel">
      <!-- the thin white outline the module is printed inside -->
      <div class="facessl__outline" :style="box(OUTLINE)"></div>

      <!-- the four hex-socket fixing screws -->
      <template v-for="x in SCREW_X" :key="'sx' + x">
        <span
          v-for="y in SCREW_Y"
          :key="'s' + x + y"
          class="facessl__screw"
          :style="{ ...at(x, y), width: `${SCREW_D * 100}%` }"
        ></span>
      </template>

      <!--
        Top row, **either side of** the upper screws, which is what the
        render shows and what the two legends have to clear: the screw
        heads are 0.069 W across and centred at 0.248 and 0.752, so they
        occupy 0.214 to 0.283 and 0.717 to 0.786. Centring the legends and
        hoping was not enough; they are anchored to the inner edge of each
        screw instead.
      -->
      <div class="facessl__brand left" :style="at(0.208, 0.036)">NOOB</div>
      <div class="facessl__brand right" :style="at(0.792, 0.036)">
        <b>G</b><span>-COMP</span>
      </div>

      <!-- the meter, in its matt escutcheon -->
      <div class="facessl__escutcheon" :style="box(ESCUTCHEON)"></div>
      <div class="facessl__glass" :style="box(GLASS)"><SslMeter :lit="lit" /></div>
      <span class="facessl__zero" :style="at(ZERO_SCREW[0], ZERO_SCREW[1])"></span>

      <!-- the IN switch: the one bright object on the panel, and not a bypass -->
      <button
        v-if="c.in"
        class="facessl__in"
        :class="{ on: c.in.on }"
        :style="box(IN_CAP)"
        type="button"
        role="switch"
        :aria-checked="c.in.on"
        aria-label="Compressor in"
        @click="c.in.begin(); c.in.setOn(!c.in.on); c.in.end()"
      >IN</button>

      <!-- row 1: the two pots -->
      <div class="facessl__cap" :style="at(COL[0], ROW[0] - CAP_DY)">THRESHOLD</div>
      <div class="facessl__knob" :style="knobAt(COL[0], ROW[0], POT)">
        <SslKnob
          :p="c.threshold"
          :marks="THRESHOLD_MARKS"
          :sweep="SWEEP"
          :cap="POT.cap"
          :skirt="POT.skirt"
          :box="POT.box"
          label="Threshold"
        />
      </div>
      <div class="facessl__unit" :style="at(COL[0], ROW[0] + UNIT_DY)">dB</div>

      <div class="facessl__cap" :style="at(COL[1], ROW[0] - CAP_DY)">MAKE UP</div>
      <div class="facessl__knob" :style="knobAt(COL[1], ROW[0], POT)">
        <SslKnob
          :p="c.makeup"
          :marks="MAKEUP_MARKS"
          :extra-mark="{ at: MAKEUP_ZERO_AT, label: '0' }"
          :sweep="SWEEP"
          :cap="POT.cap"
          :skirt="POT.skirt"
          :box="POT.box"
          label="Make-up gain"
        />
      </div>
      <div class="facessl__unit" :style="at(COL[1], ROW[0] + UNIT_DY)">dB</div>

      <!-- row 2: attack and release -->
      <div class="facessl__cap" :style="at(COL[0], ROW[1] - CAP_DY)">ATTACK</div>
      <div class="facessl__knob" :style="knobAt(COL[0], ROW[1], SWITCH)">
        <SslKnob
          :p="c.attack"
          :marks="ATTACK_MARKS"
          :sweep="SWEEP"
          :cap="SWITCH.cap"
          :skirt="SWITCH.skirt"
          :box="SWITCH.box"
          discrete
          label="Attack"
        />
      </div>
      <div class="facessl__unit" :style="at(COL[0], ROW[1] + UNIT_DY)">ms</div>

      <div class="facessl__cap" :style="at(COL[1], ROW[1] - CAP_DY)">RELEASE</div>
      <div class="facessl__knob" :style="knobAt(COL[1], ROW[1], SWITCH)">
        <SslKnob
          :p="c.release"
          :marks="RELEASE_MARKS"
          :sweep="SWEEP"
          :cap="SWITCH.cap"
          :skirt="SWITCH.skirt"
          :box="SWITCH.box"
          discrete
          label="Release"
        />
      </div>
      <div class="facessl__unit" :style="at(COL[1], ROW[1] + UNIT_DY)">s</div>

      <!-- row 3: ratio and the sidechain filter -->
      <div class="facessl__cap" :style="at(COL[0], ROW[2] - CAP_DY)">RATIO</div>
      <div class="facessl__knob" :style="knobAt(COL[0], ROW[2], SWITCH)">
        <SslKnob
          :p="c.ratio"
          :marks="RATIO_MARKS"
          :sweep="SWEEP"
          :cap="SWITCH.cap"
          :skirt="SWITCH.skirt"
          :box="SWITCH.box"
          discrete
          label="Ratio"
        />
      </div>

      <div class="facessl__cap" :style="at(COL[1], ROW[2] - CAP_DY)">HPF</div>
      <div class="facessl__knob" :style="knobAt(COL[1], ROW[2], SWITCH)">
        <SslKnob
          :p="c.hpf"
          :marks="HPF_MARKS"
          :sweep="SWEEP"
          :cap="SWITCH.cap"
          :skirt="SWITCH.skirt"
          :box="SWITCH.box"
          discrete
          label="Sidechain high-pass"
        />
      </div>
      <div class="facessl__unit" :style="at(COL[1], ROW[2] + UNIT_DY)">Hz</div>

      <!-- bottom, centred between the lower screws -->
      <div class="facessl__strap" :style="at(0.5, 0.958)">STEREO BUS COMPRESSOR</div>
    </div>
  </section>
</template>

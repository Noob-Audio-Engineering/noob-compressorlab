<script setup>
/**
 * The LA-3A's front panel, laid out from the description in
 * `research/LA-3A.md` section 2.1, which was read off the original UREI
 * datasheet photograph and the reissue manual's line drawing.
 *
 * The unit is **half a rack wide** and two units tall, 8.5 by 3.5 inches,
 * so on its own it cannot fill a rack. The dossier gives the answer the
 * hardware gives: the SR-3A mounting kit puts one LA-3A in a 19 inch,
 * 3.5 inch panel with a blank plate beside it. That is what this face
 * draws, which is why it has rack ears and rails like the others and yet
 * only half of it carries controls. Elements inside the unit sit at
 * fractions of the unit's own box and are sized in container-query units
 * (`cqw`) of it, so the panel scales with the window.
 *
 * Top row: the GAIN knob at the left with its scale printed on the panel,
 * the VU meter in the centre under the maker's wordmark, and PEAK REDUCTION
 * at the right. Bottom row: the two-position GR / OUTPUT meter toggle under
 * Gain, the nameplate in the centre, and the POWER / ON toggle under Peak
 * Reduction with the gain-reduction zero-set trim peeking through its hole
 * just above the P of POWER, exactly as the reissue has it.
 *
 * The Comp / Limit switch and the HF Contour pot are not here: on the
 * hardware they live on the rear panel, so the page keeps them on the
 * extras strip. Reads: `la3a_gain`, `la3a_peak_reduction`, `la3a_meter`,
 * `bypass`. Streams: `meter`, through the 1176 face's VU component.
 */
import { computed } from 'vue';
import { useControls } from './useLa3a.js';
import La3aKnob from './La3aKnob.vue';
import La3aToggle from './La3aToggle.vue';
import VuMeter1176 from '../fet/VuMeter1176.vue';

const c = useControls();
/** What the two-position meter switch is on, plus the plug-in's own Off. */
const meterLegend = computed(() => ['GAIN REDUCTION', 'OUTPUT', 'OFF'][c.meter.index] || '');

/** Position helper: fractions of the unit's panel → CSS, centred on the point. */
const at = (x, y, extra = {}) => ({ left: `${x * 100}%`, top: `${y * 100}%`, ...extra });
/** Box helper: the centre of the box and its size, both as fractions of the unit's panel. */
const box = (x, y, w, h) => ({ left: `${x * 100}%`, top: `${y * 100}%`, width: `${w * 100}%`, height: `${h * 100}%` });
</script>

<template>
  <section class="facela3a">
    <div class="facela3a__ear left"><span class="screw big"></span><span class="screw big"></span></div>
    <div class="facela3a__plate">
      <div class="facela3a__rack">
        <!-- the blank half of the SR-3A kit, with its own panel screws -->
        <span class="screw small abs" :style="at(0.58, 0.12)"></span>
        <span class="screw small abs" :style="at(0.58, 0.88)"></span>
        <span class="screw small abs" :style="at(0.97, 0.12)"></span>
        <span class="screw small abs" :style="at(0.97, 0.88)"></span>
        <div class="facela3a__blankmark">SR-3A BLANK</div>

        <!-- the unit itself, on the left half -->
        <div class="facela3a__unit">
          <div class="facela3a__panel">
            <!-- the four cover screws along the top edge of the chassis behind the panel -->
            <span class="screw small abs" :style="at(0.12, 0.045)"></span>
            <span class="screw small abs" :style="at(0.37, 0.045)"></span>
            <span class="screw small abs" :style="at(0.63, 0.045)"></span>
            <span class="screw small abs" :style="at(0.88, 0.045)"></span>

            <div class="facela3a__print abs" :style="at(0.17, 0.115)">GAIN</div>
            <div class="abs" :style="at(0.17, 0.4)"><La3aKnob :p="c.gain" size="19cqw" label="Gain" /></div>

            <div class="facela3a__logo abs" :style="at(0.5, 0.12)">NOOB</div>
            <div class="facela3a__meter abs" :style="box(0.5, 0.42, 0.3, 0.42)">
              <VuMeter1176 :mode="c.meter" :off-index="2" />
            </div>
            <!-- what the meter is showing, on the panel rather than the face -->
            <div class="facela3a__legend abs" :style="at(0.5, 0.665)">{{ meterLegend }}</div>

            <div class="facela3a__print abs" :style="at(0.83, 0.07)">PEAK</div>
            <div class="facela3a__print abs" :style="at(0.83, 0.155)">REDUCTION</div>
            <div class="abs" :style="at(0.83, 0.4)"><La3aKnob :p="c.peakReduction" size="19cqw" label="Peak Reduction" /></div>

            <div class="abs" :style="at(0.17, 0.79)"><La3aToggle :p="c.meter" left="GR" right="OUTPUT" /></div>

            <div class="facela3a__nameplate abs" :style="at(0.5, 0.79)">
              <span>LEVELING AMPLIFIER</span>
              <b>NOOB AUDIO LA-3A</b>
            </div>

            <div class="facela3a__trim abs" :style="at(0.755, 0.68)" title="Gain-reduction zero set: the trim pot the reissue lets you reach through the panel"></div>
            <div class="abs" :style="at(0.83, 0.79)"><La3aToggle :p="c.bypass" invert left="POWER" right="ON" /></div>
          </div>
        </div>
      </div>
    </div>
    <div class="facela3a__ear right"><span class="screw big"></span><span class="screw big"></span></div>
  </section>
</template>

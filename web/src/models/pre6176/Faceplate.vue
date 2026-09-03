<script setup>
/**
 * The 6176's front panel, measured from a photograph of a real unit in a
 * rack (`scratchpad/ref/ua-band0.png` and `ua-band1.png`, cropped from
 * `ua-aes124.jpg`): two rack units of brushed aluminium carrying two black
 * inset panels with a narrow bare strip between them. Everything sits at a
 * fraction of the plate and is sized in container-query units (`cqw`), so
 * the panel scales with the window.
 *
 * The arrangement follows the photograph, and the two halves label
 * themselves differently, which is the thing to keep hold of when moving
 * anything:
 *
 * - **Left half, the 610 preamp**: captions sit *above* their controls
 *   (GAIN, LEVEL, HIGH, LOW), clear of the scale so no caption crosses a
 *   numeral. The GAIN switch and the input selector share a column, the pad
 *   and polarity toggles and the Hi-Z jack the next, the big LEVEL knob the
 *   middle, then the two shelf-frequency toggles and the two shelf-step
 *   knobs.
 * - **Middle strip**: the maker's diamond at the top, the violet jewel lamp
 *   under it, and two small toggles at the bottom, each with one short
 *   label. Nothing else: on the hardware this strip is nearly empty, and
 *   the model text belongs in the badge block on the right.
 * - **Right half, the 1176 limiter**: captions sit *below* their controls,
 *   with "Slow" and "Fast" flanking the caption on the same line. ATTACK,
 *   RELEASE and METER above; INPUT, OUTPUT and RATIO below; the VU meter at
 *   the right with the model block beneath it.
 *
 * RATIO is a rotary here, not a row of buttons: the hardware marks BP, 1,
 * 4, 8, 12, 20 and ALL around a knob like its neighbours. It is the one
 * control that drives two parameters, so it has its own component.
 *
 * The right half drives the lab's existing 1176 parameters through the
 * 6176's own printed scales (`usePre.js`): Input and Output read 0 to 10
 * where the 1176 dial reads its attenuation marks, and Attack and Release
 * read 1 to 9 where it reads 1 to 7.
 *
 * Reads: `pre_*`, `fet_*`, `bypass`. Streams: `meter` (through the VU).
 */
import {
  GAIN_MARKS,
  INPUT_MARKS,
  METER_MARKS,
  SCALE_LABELLED,
  SCALE_MARKS,
  SHELF_MARKS,
  TIME_MARKS,
  attackToRotation,
  inputSelFromRotation,
  inputSelToRotation,
  releaseToRotation,
  rotationToAttack,
  rotationToRelease,
  markToScale,
  scaleToMark,
  scaleToRotation,
  scaleToTime,
  timeToScale,
  rotationToScale,
  useControls,
} from './usePre.js';
import PreKnob from './PreKnob.vue';
import PreToggle from './PreToggle.vue';
import RatioKnob from './RatioKnob.vue';
import VuMeter1176 from '../fet/VuMeter1176.vue';

const c = useControls();

/** Position helper: fractions of the faceplate → CSS, centred on the point. */
const at = (x, y, extra = {}) => ({ left: `${x * 100}%`, top: `${y * 100}%`, ...extra });
/** Box helper: the centre of the box and its size, both as fractions of the faceplate. */
const box = (x, y, w, h) => ({ left: `${x * 100}%`, top: `${y * 100}%`, width: `${w * 100}%`, height: `${h * 100}%` });

// The 1176 half through the 6176's printed scales.
const scaleTaper = { toRotation: (mark) => scaleToRotation(markToScale(mark)), fromRotation: (r) => scaleToMark(rotationToScale(r)) };
const scaleMarks = SCALE_MARKS.map((m) => ({ value: scaleToMark(m.value), label: SCALE_LABELLED.has(Number(m.label)) ? m.label : '' }));
const attackTaper = { toRotation: attackToRotation, fromRotation: rotationToAttack };
const releaseTaper = { toRotation: releaseToRotation, fromRotation: rotationToRelease };
const timeMarks = TIME_MARKS.map((m) => ({ value: scaleToTime(m.value), label: m.label }));
/** Attack prints the detent the hardware prints past its counter-clockwise stop. */
const attackMarks = [{ value: 0, label: 'OFF' }, ...timeMarks];
</script>

<template>
  <section class="face6176">
    <div class="face6176__ear left"><span class="screw big"></span><span class="screw big"></span></div>
    <div class="face6176__plate">
      <div class="face6176__panel">
        <!-- the two black inset panels, and the bare strip between them -->
        <div class="face6176__inset" :style="box(0.03, 0.07, 0.35, 0.875)"></div>
        <div class="face6176__inset" :style="box(0.5, 0.07, 0.47, 0.875)"></div>

        <!-- ------------------------------------------------ the 610 half -->
        <div class="face6176__print abs" :style="at(0.064, 0.126)">GAIN</div>
        <div class="abs" :style="at(0.064, 0.32)">
          <PreKnob :p="c.gain" :marks="GAIN_MARKS" size="7cqw" :body="20" :mark-size="15.5" :sweep="240" label="Preamp gain" />
        </div>

        <div class="face6176__print small abs" :style="at(0.036, 0.605)">Mic</div>
        <div class="face6176__print small abs" :style="at(0.092, 0.605)">Hi-Z</div>
        <div class="abs" :style="at(0.064, 0.775)">
          <PreKnob
            :p="c.inputSel"
            :marks="INPUT_MARKS"
            :to-rotation="inputSelToRotation"
            :from-rotation="inputSelFromRotation"
            size="7cqw"
            :body="20"
            :mark-size="12.5"
            :sweep="240"
            label="Input select"
          />
        </div>

        <div class="abs" :style="at(0.124, 0.26)"><PreToggle :p="c.pad" up="-15dB" down="PAD" /></div>
        <div class="abs" :style="at(0.124, 0.6)"><PreToggle :p="c.polarity" up="OUT ⌀" down="IN ⌀" /></div>
        <div class="face6176__jack abs" :style="at(0.124, 0.83)"><span></span></div>
        <div class="face6176__print small abs" :style="at(0.124, 0.925)">Hi-Z</div>

        <div class="face6176__print abs" :style="at(0.19, 0.155)">LEVEL</div>
        <div class="abs" :style="at(0.19, 0.52)">
          <PreKnob :p="c.level" :marks="SCALE_MARKS" size="11cqw" :body="24" :mark-size="9.5" :sweep="290" dots label="Preamp level" />
        </div>

        <div class="face6176__print abs" :style="at(0.253, 0.115)">HIGH</div>
        <div class="abs" :style="at(0.253, 0.31)"><PreToggle :p="c.hfFreq" :positions="3" up="7K" mid="10K" down="4.5K" /></div>
        <div class="face6176__print abs" :style="at(0.253, 0.555)">LOW</div>
        <div class="abs" :style="at(0.253, 0.75)"><PreToggle :p="c.lfFreq" :positions="3" up="100" mid="200" down="70" /></div>

        <div class="abs" :style="at(0.335, 0.315)">
          <PreKnob :p="c.hfGain" :marks="SHELF_MARKS" size="7.4cqw" :body="20" :mark-size="14.5" :sweep="280" label="High shelf" />
        </div>
        <div class="abs" :style="at(0.335, 0.755)">
          <PreKnob :p="c.lfGain" :marks="SHELF_MARKS" size="7.4cqw" :body="20" :mark-size="14.5" :sweep="280" label="Low shelf" />
        </div>

        <!-- ------------------------- the centre strip, nearly empty as it is -->
        <div class="face6176__logo abs" :style="at(0.44, 0.2)">NB</div>
        <div class="face6176__jewel abs" :class="{ off: c.bypass.on }" :style="at(0.44, 0.47)"></div>
<!--
          The two toggles the hardware puts here. JOIN sends the preamp into
          the limiter and SPLIT makes the halves independent, which is the
          routing parameter the RATIO switch's BP position also reaches, so
          the two agree rather than each having a path of its own.
        -->
        <div class="abs" :style="at(0.398, 0.79)"><PreToggle :p="c.phantom" class="on-alu" down="+48v" /></div>
        <div class="abs" :style="at(0.44, 0.79)"><PreToggle :p="c.bypass" class="on-alu" invert up="ON" down="OFF" /></div>
        <div class="abs" :style="at(0.482, 0.79)"><PreToggle :p="c.join" class="on-alu" :positions="2" up="JOIN" down="SPLIT" /></div>

        <!-- ----------------------------------------------- the 1176 half -->
        <div class="abs" :style="at(0.542, 0.28)">
          <PreKnob :p="c.attack" :marks="attackMarks" :to-rotation="attackTaper.toRotation" :from-rotation="attackTaper.fromRotation" size="8cqw" :body="20" :mark-size="13.5" :sweep="280" label="Attack" />
        </div>
        <div class="face6176__print tiny abs" :style="at(0.507, 0.475)">Slow</div>
        <div class="face6176__print abs" :style="at(0.542, 0.475)">ATTACK</div>
        <div class="face6176__print tiny abs" :style="at(0.577, 0.475)">Fast</div>

        <div class="abs" :style="at(0.629, 0.28)">
          <PreKnob :p="c.release" :marks="timeMarks" :to-rotation="releaseTaper.toRotation" :from-rotation="releaseTaper.fromRotation" size="8cqw" :body="20" :mark-size="13.5" :sweep="280" label="Release" />
        </div>
        <div class="face6176__print tiny abs" :style="at(0.594, 0.475)">Slow</div>
        <div class="face6176__print abs" :style="at(0.629, 0.475)">RELEASE</div>
        <div class="face6176__print tiny abs" :style="at(0.664, 0.475)">Fast</div>

        <div class="abs" :style="at(0.722, 0.28)">
          <PreKnob :p="c.meter" :marks="METER_MARKS" size="8cqw" :body="20" :mark-size="13" :sweep="120" label="Meter" />
        </div>
        <div class="face6176__print abs" :style="at(0.722, 0.475)">METER</div>

        <div class="abs" :style="at(0.542, 0.683)">
          <PreKnob :p="c.input" :marks="scaleMarks" :to-rotation="scaleTaper.toRotation" :from-rotation="scaleTaper.fromRotation" size="8cqw" :body="20" :mark-size="13.5" :sweep="280" label="Input" />
        </div>
        <div class="face6176__print abs" :style="at(0.542, 0.875)">INPUT</div>
        <span class="face6176__lamp abs" :style="at(0.586, 0.875)" title="Meter lamp"></span>

        <div class="abs" :style="at(0.629, 0.683)">
          <PreKnob :p="c.output" :marks="scaleMarks" :to-rotation="scaleTaper.toRotation" :from-rotation="scaleTaper.fromRotation" size="8cqw" :body="20" :mark-size="13.5" :sweep="280" label="Output" />
        </div>
        <div class="face6176__print abs" :style="at(0.629, 0.875)">OUTPUT</div>

        <div class="abs" :style="at(0.722, 0.683)"><RatioKnob size="8cqw" :body="20" :mark-size="13" :sweep="280" /></div>
        <div class="face6176__print abs" :style="at(0.722, 0.875)">RATIO</div>

        <div class="face6176__meter abs" :style="box(0.856, 0.252, 0.126, 0.33)">
          <VuMeter1176 :mode="c.meter" :off-index="-1" />
        </div>
        <div class="face6176__nameplate abs" :style="at(0.856, 0.71)">
          <b>NOOB AUDIO</b>
          <em>6176</em>
          <span>610B Tube Preamplifier</span>
          <span>1176LN Limiting Amplifier</span>
        </div>

        <span class="screw small abs" :style="at(0.014, 0.16)"></span>
        <span class="screw small abs" :style="at(0.014, 0.84)"></span>
        <span class="screw small abs" :style="at(0.986, 0.16)"></span>
        <span class="screw small abs" :style="at(0.986, 0.84)"></span>
      </div>
    </div>
    <div class="face6176__ear right"><span class="screw big"></span><span class="screw big"></span></div>
  </section>
</template>

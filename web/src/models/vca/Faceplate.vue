<script setup>
/**
 * The Distressor's front panel, laid out from the manufacturer's own
 * photograph of an EL8-X: a single rack unit, so everything is one long
 * row. Elements sit at fractions of the plate's width and height measured
 * off that photo, inside a box that keeps the panel's 19 : 1.75 aspect and
 * scales with the window; sizes inside use container-query units (`cqw`,
 * hundredths of the plate width) so the lettering, lamps and knobs grow
 * with it.
 *
 * Left to right, as on the hardware: the maker's initials on the rack ear,
 * the Distressor script, the BYPASS button, the sixteen-lamp GAIN REDUCTION
 * bargraph above them, the clipping-waveform icon with the REDLINE and
 * 1% THD lamps, the eight RATIO lamps with their cycling button, the
 * DETECTOR and AUDIO lamp columns with theirs, the four big ivory knobs
 * (INPUT, ATTACK, RELEASE, OUTPUT) with the two EL8-X toggles between them,
 * and the power rocker with the EL8-X badge.
 *
 * The lamps are live controls as well as indicators: clicking a ratio
 * selects it, clicking a filter lamp sets that bit of its selector, and the
 * Link lamp is the lab's shared stereo link. The buttons cycle, as they do
 * on the hardware.
 *
 * Reads: `dist_*`, `link` (the Link lamp), `bypass` (the power rocker).
 * Streams: `meter` through the bargraph, `lamps` for the distortion lamps.
 */
import { computed } from 'vue';
import { useStreamValue } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';
import { AUDIOS, DETECTORS, KNOB_MARKS, RATIOS, cycle, knobToRotation, rotationToKnob, useControls, useFinish } from './useVca.js';
import KnobEL8 from './KnobEL8.vue';
import GrBargraph from './GrBargraph.vue';
import MiniToggle from './MiniToggle.vue';

const c = useControls();
const finish = useFinish();
const thd = useStreamValue('lamps', { index: 0 });
const redline = useStreamValue('lamps', { index: 1 });

const det = computed(() => DETECTORS[c.detector.index] || DETECTORS[0]);
const aud = computed(() => AUDIOS[c.audio.index] || AUDIOS[0]);

/** Position helper: fractions of the faceplate → CSS. */
const at = (x, y, extra = {}) => ({ left: `${x * 100}%`, top: `${y * 100}%`, ...extra });
/** Box helper: the centre of the box and its size, both as fractions of the faceplate. */
const box = (x, y, w, h) => ({ left: `${x * 100}%`, top: `${y * 100}%`, width: `${w * 100}%`, height: `${h * 100}%` });

function setIndex(p, i) {
  p.begin();
  p.setIndex(i);
  p.end();
}
/** The detector lamps set their own bit of the four-state selector. */
function setDetector(bit) {
  const next = { hp: det.value.hp, band: det.value.band, [bit]: !det.value[bit] };
  setIndex(c.detector, DETECTORS.findIndex((d) => d.hp === next.hp && d.band === next.band));
}
/** The audio lamps do the same across the six states: the high-pass is a bit, the two colours are exclusive. */
function setAudio(what) {
  const hp = what === 'hp' ? !aud.value.hp : aud.value.hp;
  const dist = what === 'hp' ? aud.value.dist : aud.value.dist === what ? 0 : what;
  setIndex(c.audio, AUDIOS.findIndex((a) => a.hp === hp && a.dist === dist));
}
function toggleLink() {
  c.link.begin();
  c.link.setOn(!c.link.on);
  c.link.end();
}
function togglePower() {
  c.bypass.begin();
  c.bypass.setOn(!c.bypass.on);
  c.bypass.end();
}
</script>

<template>
  <section class="faceel8" :class="`is-${finish}`">
    <div class="faceel8__ear left"><span class="screw"></span><span class="screw"></span></div>
    <div class="faceel8__plate">
      <div class="faceel8__panel">
        <!-- the maker's initials, at the left as on the hardware -->
        <div class="faceel8__eli abs" :style="at(0.032, 0.48)">nb</div>

        <!-- gain reduction, across the top -->
        <div class="abs" :style="box(0.225, 0.2, 0.263, 0.34)"><GrBargraph /></div>

        <div class="faceel8__logo abs" :style="at(0.126, 0.46)">Noobtressor</div>
        <div class="faceel8__sig abs" :style="at(0.126, 0.6)">a fond spoof</div>

        <button class="faceel8__bypass abs" :class="{ on: c.bypass.on }" :style="at(0.214, 0.47)" type="button" :aria-pressed="c.bypass.on" title="Hard bypass; the meter stays live" @click="togglePower">
          <span class="el8lamp red" :class="{ on: c.bypass.on }"></span>
        </button>
        <div class="faceel8__print tiny abs" :style="at(0.214, 0.64)">BY PASS</div>

        <!-- the clipping-waveform icon and the two distortion lamps -->
        <svg class="faceel8__wave abs" :style="at(0.276, 0.45)" viewBox="0 0 40 18">
          <path d="M1 9 Q 6 1 10 9 T 19 9" fill="none" />
          <path d="M19 9 h3 v-6 h6 v12 h6 v-6 h5" fill="none" class="clip" />
        </svg>
        <button class="faceel8__lampbtn abs" :style="at(0.332, 0.36)" type="button" title="Redline: about 3 % distortion, or the output clipping">
          <span class="el8lamp red" :class="{ on: redline > 0.5 }"></span><span class="faceel8__print tiny">REDLINE</span>
        </button>
        <button class="faceel8__lampbtn abs" :style="at(0.332, 0.58)" type="button" title="The hardware silkscreens 1 % here but lights it at a quarter of that; this lights at 1 %">
          <span class="el8lamp amber" :class="{ on: thd >= 1 }"></span><span class="faceel8__print tiny">1% THD</span>
        </button>

        <!-- the eight ratios -->
        <div class="faceel8__ratios abs" :style="at(0.222, 0.79)">
          <button v-for="(r, i) in RATIOS" :key="r.label" class="faceel8__ratio" :class="{ on: c.ratio.index === i }" type="button" :aria-pressed="c.ratio.index === i" :title="r.hint" @click="setIndex(c.ratio, i)">
            <span class="el8lamp" :class="[r.colour, { on: c.ratio.index === i }]"></span>
            <span class="faceel8__ratiolabel" :class="{ nuke: r.label === 'Nuke' }">{{ r.label }}</span>
          </button>
        </div>
        <div class="faceel8__brit abs" :style="at(0.146, 0.7)">Brit</div>
        <div class="faceel8__opto abs" :style="at(0.253, 0.955)">Opto</div>
        <div class="faceel8__print tiny abs" :style="at(0.196, 0.955)">RATIO</div>
        <button class="faceel8__btn abs" :style="at(0.332, 0.79)" type="button" title="Step through the eight ratios" @click="cycle(c.ratio)">RATIO</button>

        <!-- detector: what the compressor listens to -->
        <button class="faceel8__lampbtn abs" :style="at(0.386, 0.13)" type="button" :aria-pressed="det.hp" title="High-pass the side-chain so the lows stop pumping it" @click="setDetector('hp')">
          <span class="el8lamp green" :class="{ on: det.hp }"></span><span class="faceel8__print tiny">HP</span>
        </button>
        <button class="faceel8__lampbtn abs" :style="at(0.386, 0.36)" type="button" :aria-pressed="det.band" title="Band emphasis: the side-chain hears the top, so it rides sibilance" @click="setDetector('band')">
          <span class="el8lamp amber" :class="{ on: det.band }"></span><span class="faceel8__print tiny">Band</span>
        </button>
        <button class="faceel8__lampbtn abs" :style="at(0.386, 0.59)" type="button" :aria-pressed="c.link.on" title="Stereo link" @click="toggleLink">
          <span class="el8lamp red" :class="{ on: c.link.on }"></span><span class="faceel8__print tiny">Link</span>
        </button>
        <button class="faceel8__btn abs" :style="at(0.394, 0.82)" type="button" title="Step through the side-chain filters" @click="cycle(c.detector)">DETECTOR</button>

        <!-- audio: what comes out -->
        <button class="faceel8__lampbtn abs" :style="at(0.452, 0.13)" type="button" :aria-pressed="aud.hp" title="High-pass the audio: 80 Hz, 18 dB per octave" @click="setAudio('hp')">
          <span class="el8lamp green" :class="{ on: aud.hp }"></span><span class="faceel8__print tiny">HP</span>
        </button>
        <button class="faceel8__lampbtn abs" :style="at(0.452, 0.36)" type="button" :aria-pressed="aud.dist === 2" title="Dist 2: second-harmonic colour, a few per cent" @click="setAudio(2)">
          <span class="el8lamp amber" :class="{ on: aud.dist === 2 }"></span><span class="faceel8__print tiny">Dist 2</span>
        </button>
        <button class="faceel8__lampbtn abs" :style="at(0.452, 0.59)" type="button" :aria-pressed="aud.dist === 3" title="Dist 3: third harmonic, up to twenty per cent" @click="setAudio(3)">
          <span class="el8lamp red" :class="{ on: aud.dist === 3 }"></span><span class="faceel8__print tiny">Dist 3</span>
        </button>
        <button class="faceel8__btn abs" :style="at(0.458, 0.82)" type="button" title="Step through the audio-path options" @click="cycle(c.audio)">AUDIO</button>

        <!-- the four knobs -->
        <div class="faceel8__print big abs" :style="at(0.515, 0.1)">INPUT</div>
        <div class="abs" :style="at(0.515, 0.56)">
          <KnobEL8 :p="c.input" :marks="KNOB_MARKS" :to-rotation="knobToRotation" :from-rotation="rotationToKnob" size="8.8cqw" label="Input" />
        </div>
        <div class="faceel8__print big abs" :style="at(0.624, 0.1)">ATTACK</div>
        <div class="abs" :style="at(0.624, 0.56)">
          <KnobEL8 :p="c.attack" :marks="KNOB_MARKS" :to-rotation="knobToRotation" :from-rotation="rotationToKnob" size="8.8cqw" label="Attack" :end-labels="[{ value: 10, label: 'Opto(10)' }]" />
        </div>
        <div class="faceel8__print big abs" :style="at(0.735, 0.1)">RELEASE</div>
        <div class="abs" :style="at(0.735, 0.56)">
          <KnobEL8 :p="c.release" :marks="KNOB_MARKS" :to-rotation="knobToRotation" :from-rotation="rotationToKnob" size="8.8cqw" label="Release" :end-labels="[{ value: 0, label: 'Opto(0)' }]" />
        </div>
        <div class="faceel8__print big abs" :style="at(0.843, 0.1)">OUTPUT</div>
        <div class="abs" :style="at(0.843, 0.56)">
          <KnobEL8 :p="c.output" :marks="KNOB_MARKS" :to-rotation="knobToRotation" :from-rotation="rotationToKnob" size="8.8cqw" label="Output" />
        </div>

        <!-- the two toggles the X adds, in the gaps between the knobs -->
        <div class="faceel8__toglabel abs" :style="at(0.5695, 0.3)">Stereo<br />Image Link</div>
        <div class="abs" :style="at(0.5695, 0.72)"><MiniToggle :p="c.link" lamp="red" /></div>
        <div class="faceel8__toglabel abs" :style="at(0.6795, 0.3)">British<br />Mode (1:1)</div>
        <div class="abs" :style="at(0.6795, 0.72)"><MiniToggle :p="c.british" lamp="amber" /></div>

        <!-- power and the badge -->
        <div class="faceel8__print big abs" :style="at(0.917, 0.1)">POWER</div>
        <button class="faceel8__power abs" :class="{ off: c.bypass.on }" :style="at(0.917, 0.58)" type="button" :aria-pressed="!c.bypass.on" title="Power" @click="togglePower">
          <span class="faceel8__rocker"></span>
        </button>
<template v-if="finish === 'red'">
          <div class="faceel8__badge abs" :style="at(0.958, 0.3)">ANNIVERSARY EDITION</div>
          <div class="faceel8__model abs" :style="at(0.958, 0.58)">NB8</div>
          <div class="faceel8__badge abs" :style="at(0.958, 0.85)">XXX · NOOB AUDIO</div>
        </template>
        <template v-else>
          <div class="faceel8__badge abs" :style="at(0.958, 0.42)">Noob<br />Audio</div>
          <div class="faceel8__model abs" :style="at(0.958, 0.72)">NB8</div>
        </template>
      </div>
    </div>
    <div class="faceel8__ear right"><span class="screw"></span><span class="screw"></span></div>
  </section>
</template>

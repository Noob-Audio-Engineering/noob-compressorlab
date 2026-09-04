<script setup>
/**
 * The TG12413's face, drawn as what it is: a **vertical module strip in a
 * console frame**, not a nineteen-inch rack panel.
 *
 * The unit has no front panel of its own. On the recording desk it is part
 * of a dual-channel microphone cassette; on the transfer console it is one
 * of five strips in a rack of modules. Its connections on the drawing are
 * numbered tags rather than connectors, and the whole thing lands on an edge
 * connector. Drawing it as a rack face would be the first lie on the page,
 * so the strip is roughly one to five and sits in a frame with the things
 * the module does not itself carry.
 *
 * # Colour
 *
 * The two sources disagree and the dossier's section 2.6 declines to resolve
 * them, so this sits between them and says so. The Creative Commons
 * photographs of the restored TG12345 at Abbey Road measure the console's
 * module field at `#69767E` and `#656969`, a mid grey with a blue-green
 * cast; Waves, who built their model with Abbey Road and had the *transfer*
 * console, draw their strips in a slate blue between `#47647C` and
 * `#42576D`. They are different consoles and there is no reason their
 * modules were finished alike. The field here is between the two.
 *
 * White silkscreen throughout, which both sources agree on.
 *
 * # The wordmark
 *
 * The console silkscreens `EMI` at the head of every module strip. **The
 * spoof must not**, so the head of this strip carries the lab's own name and
 * the foot carries the model number and what the module is, which is how
 * Waves label theirs.
 *
 * Reads: `tg_mode`, `tg_recovery`, `tg_output`, `tg_hold`, and the `cell`
 * stream for the live control current. Emits: nothing.
 */
import { computed } from 'vue';
import { useStreamFrame } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';
import { MODE_MARKS, NOTES, OUTPUT_MARKS, RECOVERY_MARKS, SWEEP, useControls } from './useTg.js';
import TgSwitch from './TgSwitch.vue';
import TgMeter from './TgMeter.vue';

const c = useControls();

/**
 * `[control current in microamps, element resistance in ohms, drive]`.
 *
 * The control current is the quantity the whole circuit is about, and
 * section 11.9 asks the page to show it live, because it is what makes the
 * two modes legible side by side: the same gain reduction in COMPRESS and in
 * LIMIT is reached at very different currents.
 */
const cell = useStreamFrame('cell');

const microamps = computed(() => {
  const v = cell.value?.[0] ?? 0;
  return Number.isFinite(v) ? v : 0;
});

/** The element's resistance, printed in ohms or kilohms as it deserves. */
const resistance = computed(() => {
  const r = cell.value?.[1] ?? 0;
  // An element carrying no control current is an open circuit, and the
  // engine publishes a very large number for it. Zero means the same thing
  // and arrives before the first block, so both read as open rather than as
  // a dead short, which is what a bare zero would look like.
  if (!Number.isFinite(r) || r <= 0 || r >= 1e8) return '∞';
  if (r >= 1000) return `${(r / 1000).toFixed(1)} kΩ`;
  return `${Math.round(r)} Ω`;
});
</script>

<template>
  <div class="tgframe">
    <!-- The module itself: three switches and nothing else. -->
    <div class="tgstrip">
      <div class="tgstrip__head">NOOB</div>

      <div class="tgstrip__control">
        <div class="tgstrip__legend">MODE</div>
        <TgSwitch :p="c.mode" :marks="MODE_MARKS" :sweep="SWEEP.mode" accent="red" label="Mode" />
        <div class="tgstrip__value">{{ c.mode.text }}</div>
      </div>

      <div class="tgstrip__control">
        <div class="tgstrip__legend">RECOVERY</div>
        <TgSwitch :p="c.recovery" :marks="RECOVERY_MARKS" :sweep="SWEEP.recovery" label="Recovery" />
        <div class="tgstrip__value">{{ c.recovery.text }}</div>
      </div>

      <div class="tgstrip__control">
        <div class="tgstrip__legend">OUTPUT LEVEL</div>
        <TgSwitch :p="c.output" :marks="OUTPUT_MARKS" :sweep="SWEEP.output" label="Output level" />
        <div class="tgstrip__value">{{ c.output.text }}</div>
      </div>

      <div class="tgstrip__foot">
        <span>TG 12413</span>
        <span>LIMITER</span>
      </div>
    </div>

    <!-- The frame around it: the internal preset, the added meter, the live
         control current, and what the page owes the reader. -->
    <div class="tgrack">
      <div class="tgrack__row">
        <div v-if="c.hold" class="tgpreset">
          <div class="tgpreset__caption">RV1 HOLD</div>
          <TgSwitch :p="c.hold" :sweep="270" size="52px" label="Hold preset" />
          <div class="tgpreset__value">{{ c.hold.text }}</div>
          <div class="tgpreset__note">a 10 kΩ screwdriver preset inside the module, worth a fifth of the fastest recovery and almost nothing at the slowest</div>
        </div>

        <TgMeter />

        <div class="tgreadout">
          <div class="tgreadout__caption">CONTROL CURRENT</div>
          <div class="tgreadout__value">{{ microamps.toFixed(1) }}<span class="tgreadout__unit"> µA</span></div>
          <div class="tgreadout__caption">ELEMENT</div>
          <div class="tgreadout__value tgreadout__value--small">{{ resistance }}</div>
          <div class="tgreadout__note">The current into the four diodes, and the resistance they present. Gain reduction is nothing but that resistance against the 20 kΩ series arm.</div>
        </div>
      </div>

      <dl class="tgnotes">
        <template v-for="[term, text] in NOTES" :key="term">
          <dt class="tgnotes__term">{{ term }}</dt>
          <dd class="tgnotes__text">{{ text }}</dd>
        </template>
      </dl>
    </div>
  </div>
</template>

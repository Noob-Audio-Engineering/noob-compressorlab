<script setup>
/**
 * The Fairchild page: the strip with the lab's own controls, the front
 * panel, and the analysis drawer.
 *
 * **The drawer is two rows on this face, and only on this face.** The
 * shared history runs full width across the top; beneath it sit this
 * model's three blocks — the timing network, and each of Fairchild's two
 * 1959 charts — with the shared transfer panel at the end. They used to be
 * one 292 px column holding all three, where the capacitor bars were a few
 * pixels wide and both charts were thumbnails nobody could read. A user
 * asked for them as separate blocks with the history above, and that is
 * what this is.
 *
 * The history and transfer panels are the same components under this face
 * as under every other, which is a standing instruction. Only where they
 * sit changes.
 *
 * **This face has a layout problem the other seven do not, and it is the
 * hardware's.** The 670 is 19 × 14 inches, which is 8U and very nearly
 * square, where the next tallest face here is 19 × 5.25. Fitted to the
 * height of a 900 × 520 window it would be a few hundred pixels wide and its
 * silkscreen unreadable. So the panel is capped from the height as every
 * face is, but with a floor under it, and the drawer gives way first: below
 * the height at which both fit, the drawer closes and the panel takes the
 * room. Nothing scrolls and nothing is drawn too small to read.
 *
 * The 660 is half the panel and gets the height back, so its aspect is read
 * from the unit switch rather than fixed.
 */
import { computed } from 'vue';
import { useDebug, useWindow } from '../../composables/useLab.js';
import { ui, useControls } from './useVmu.js';
import HistoryPanel from '../../components/HistoryPanel.vue';
import TransferPanel from '../../components/TransferPanel.vue';
import Charts from './Charts.vue';
import Faceplate from './Faceplate.vue';
import ExtrasBar from './ExtrasBar.vue';
import DebugPanel from '../../components/DebugPanel.vue';

const win = useWindow();
const c = useControls();

/**
 * Vertical space the page needs besides the panel: the top bar, the
 * paddings and gaps, and the extras strip, which carries two rows of
 * controls on this model. Measured at the 900 px minimum.
 */
const CHROME_PX = 156;
/**
 * The least drawer worth showing; below this it closes instead.
 *
 * Two rows need more room than the one row this replaced, but the number
 * did not rise as far as the layout might suggest: 300 px covers a 90 px
 * history, a 174 px lower row and the gaps, and it is deliberately the
 * largest value that keeps every window size behaving exactly as it did.
 * At 1100 x 620 the 660 still opens its drawer and the 670 still closes
 * one, which is what each did before.
 */
const BENCH_PX = 300;
/** The panel is never drawn narrower than this, whatever the window does. */
const PLATE_MIN = 430;

const aspect = computed(() => (c.unit && c.unit.index === 0 ? 19 / 7 : 19 / 14));
const room = computed(() => Math.max(120, win.height.value - CHROME_PX));
/**
 * How much of the page the drawer keeps.
 *
 * It used to be exactly BENCH_PX, so the panel took every pixel above the
 * minimum and the drawer never grew. That was tolerable with one row of
 * charts and is not with two: at 1900 x 1000 the panel was 544 px tall with
 * 500 px of unused width beside it, while the drawer sat at its 300 px
 * floor and the blocks underneath were half the height they could have had.
 * So the drawer takes a share of a tall window instead, still never less
 * than the floor and never more than it can use.
 */
const benchRoom = computed(() =>
  Math.min(520, Math.max(BENCH_PX, Math.round(room.value * 0.4))),
);
const showBench = computed(() => ui.scope && (room.value - BENCH_PX) * aspect.value >= PLATE_MIN);
const plateMax = computed(() => {
  const h = showBench.value ? room.value - benchRoom.value : room.value;
  return `${Math.max(PLATE_MIN, Math.round(h * aspect.value))}px`;
});
const debug = useDebug();
</script>

<template>
  <main class="lab-model lab-model--vmu">
    <ExtrasBar />
    <div class="shrink-0 px-3 pt-3">
      <div class="w-full mx-auto" :style="{ maxWidth: plateMax }"><Faceplate /></div>
    </div>
    <section v-if="showBench" class="lab-bench-vmu">
      <HistoryPanel />
      <div class="lab-bench-vmu__row">
        <div class="lab-panel lab-panel--charts"><Charts part="network" /></div>
        <div class="lab-panel lab-panel--charts"><Charts part="io" /></div>
        <div class="lab-panel lab-panel--charts"><Charts part="im" /></div>
        <TransferPanel />
      </div>
    </section>
    <DebugPanel v-if="debug.shown.value" v-model:open="debug.open.value" />
  </main>
</template>

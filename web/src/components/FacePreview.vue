<script setup>
/**
 * A real faceplate, shown small and inert, as the preview in the browse
 * view. It is the model's own `Faceplate.vue` rather than a drawing of one,
 * so a preview cannot drift from the panel it represents.
 *
 * **Rendered wide, then scaled down.** The faces size themselves in
 * container-query units, so putting one in a narrow card does not shrink
 * it, it re-lays it out: the small-size branches kick in, legends that
 * only fit on a wide panel collapse onto each other, and the LA-3A's blank
 * rack plate swallows the card. A thumbnail should be the panel made
 * smaller, not a different panel. So each preview is laid out at
 * `RENDER_W`, the width these faces are designed around, and the result is
 * scaled into the card with a transform. The internal layout is then
 * identical to the real thing at that width.
 *
 * **Mounted only once it scrolls into view.** Every faceplate carries live
 * meters, and a meter subscribes to the meter stream and runs an animation
 * frame, so mounting six at once would put six needles on the page to draw
 * the two the user can see.
 *
 * It is inert: pointer events are off and it is hidden from assistive
 * technology, because the card around it is the control.
 *
 * Props: `modelKey` (one of the `MODELS` keys).
 */
import { computed, onBeforeUnmount, onMounted, ref, shallowRef } from 'vue';
import FetFace from '../models/fet/Faceplate.vue';
import OptoFace from '../models/opto/Faceplate.vue';
import La3aFace from '../models/la3a/Faceplate.vue';
import VcaFace from '../models/vca/Faceplate.vue';
import Pre6176Face from '../models/pre6176/Faceplate.vue';
import Cl1bFace from '../models/cl1b/Faceplate.vue';
import BridgeFace from '../models/bridge/Faceplate.vue';
import DbxFace from '../models/dbx/Faceplate.vue';
import TgFace from '../models/tg/Faceplate.vue';
import GbusFace from '../models/gbus/Faceplate.vue';
import VmuFace from '../models/vmu/Faceplate.vue';

const props = defineProps({
  modelKey: { type: String, required: true },
});

/** The width the faces are laid out at before scaling: a comfortable full-size panel. */
const RENDER_W = 1100;

/*
 * **The portrait faces, which need both a layout height and a scale of
 * their own.**
 *
 * Every landscape face is driven by width: the inner box is `RENDER_W`
 * across, the panel fills it, and the height falls out. The 4000 G is a
 * double 500-series module and is driven the other way, `height: 100%` with
 * the width following from its aspect ratio. In the live view the stage
 * gives it a definite height; here the inner box is out of flow with no
 * height, so its height resolves to `auto`, the width comes from an aspect
 * ratio off that, both axes end up indefinite and the panel collapses to
 * nothing. `offsetHeight` then reads zero and the card drew an empty row,
 * which is what a user reported.
 *
 * So a portrait face is laid out at `RENDER_W` of **its own long axis**,
 * which keeps the two bases comparable: a 19-inch panel and a 3-inch module
 * are each drawn at their full size and then scaled down, and the type in
 * the two thumbnails lands within a fraction of a pixel of the same size.
 * The layout height is set in the model's own stylesheet, since it is a
 * fact about that panel's proportions.
 *
 * Scaling it by card width over `RENDER_W` the way a landscape face is
 * scaled would give a row twelve hundred pixels tall, so a portrait preview
 * is scaled to a target height instead: a third of the card, which stands
 * it a little taller than the landscape panels beside it, as the hardware
 * is.
 */
const PORTRAIT = new Set(['gbus']);
const PORTRAIT_H_FRACTION = 1 / 3;

/*
 * One entry per model in `MODELS`. A key missing here draws an empty card
 * rather than failing, so a new model has to be added in both this map and
 * `LabPage.vue`'s.
 */
const FACES = {
  fet: FetFace,
  opto: OptoFace,
  la3a: La3aFace,
  vca: VcaFace,
  pre6176: Pre6176Face,
  cl1b: Cl1bFace,
  bridge: BridgeFace,
  dbx: DbxFace,
  tg: TgFace,
  gbus: GbusFace,
  vmu: VmuFace,
};
const face = computed(() => FACES[props.modelKey] || null);

const host = ref(null);
const inner = ref(null);
const shown = shallowRef(false);
const scale = ref(0.3);
const height = ref(0);
/*
 * How far to push the laid-out box across. Zero for a landscape face, which
 * fills the card; a portrait one is a fraction of the card wide and hugging
 * the left edge of an otherwise empty row reads as a mistake rather than as
 * a small module.
 */
const offsetX = ref(0);
let io = null;
let ro = null;

function measure() {
  if (!host.value) return;
  const w = host.value.clientWidth;
  // `offsetHeight` ignores the transform, so this is the panel's own height
  // at its layout size; the box it needs is that times the scale. Measuring
  // the transformed rect instead reads back the scaled value and, before the
  // face has laid out, a short one, which clipped the preview.
  const h = inner.value ? inner.value.offsetHeight : 0;
  if (PORTRAIT.has(props.modelKey)) {
    if (w > 0 && h > 0) {
      const target = w * PORTRAIT_H_FRACTION;
      scale.value = target / h;
      height.value = target;
      offsetX.value = Math.max(0, (w - RENDER_W * scale.value) / 2);
    }
    return;
  }
  offsetX.value = 0;
  if (w > 0) scale.value = w / RENDER_W;
  if (h > 0) height.value = h * scale.value;
}

onMounted(() => {
  if (!host.value) return;
  if (typeof ResizeObserver === 'function') {
    ro = new ResizeObserver(() => measure());
    ro.observe(host.value);
  }
  if (typeof IntersectionObserver !== 'function') {
    shown.value = true;
    requestAnimationFrame(measure);
    return;
  }
  io = new IntersectionObserver(
    (entries) => {
      if (entries.some((e) => e.isIntersecting)) {
        shown.value = true;
        io.disconnect();
        io = null;
        requestAnimationFrame(() => {
          measure();
          if (inner.value && ro) ro.observe(inner.value);
        });
      }
    },
    { rootMargin: '200px' },
  );
  io.observe(host.value);
});
onBeforeUnmount(() => {
  if (io) io.disconnect();
  if (ro) ro.disconnect();
  io = null;
  ro = null;
});
</script>

<template>
  <div ref="host" class="facepv" :class="`facepv--${modelKey}`" :style="{ height: height ? `${Math.round(height)}px` : null }" aria-hidden="true">
    <div ref="inner" class="facepv__inner" :style="{ width: `${RENDER_W}px`, left: `${Math.round(offsetX)}px`, transform: `scale(${scale})` }">
      <component :is="face" v-if="shown && face" />
    </div>
  </div>
</template>

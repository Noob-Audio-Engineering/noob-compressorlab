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

const props = defineProps({
  modelKey: { type: String, required: true },
});

/** The width the faces are laid out at before scaling: a comfortable full-size panel. */
const RENDER_W = 1100;

const FACES = { fet: FetFace, opto: OptoFace, la3a: La3aFace, vca: VcaFace, pre6176: Pre6176Face, cl1b: Cl1bFace };
const face = computed(() => FACES[props.modelKey] || null);

const host = ref(null);
const inner = ref(null);
const shown = shallowRef(false);
const scale = ref(0.3);
const height = ref(0);
let io = null;
let ro = null;

function measure() {
  if (!host.value) return;
  const w = host.value.clientWidth;
  if (w > 0) scale.value = w / RENDER_W;
  // `offsetHeight` ignores the transform, so this is the panel's own height
  // at RENDER_W; the box it needs is that times the scale. Measuring the
  // transformed rect instead reads back the scaled value and, before the
  // face has laid out, a short one, which clipped the preview.
  const h = inner.value ? inner.value.offsetHeight : 0;
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
    <div ref="inner" class="facepv__inner" :style="{ width: `${RENDER_W}px`, transform: `scale(${scale})` }">
      <component :is="face" v-if="shown && face" />
    </div>
  </div>
</template>

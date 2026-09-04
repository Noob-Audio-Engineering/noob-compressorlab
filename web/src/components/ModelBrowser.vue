<script setup>
/**
 * The browse view: what the lab can be, grouped by the kind of compressor
 * each one is. It replaces the faceplate and the workbench while it is up,
 * and the top bar stays, so it is always clear what is loaded and where you
 * are.
 *
 * **Browsing does not touch the audio.** The compressor that is loaded keeps
 * processing with its own settings the whole time this view is open: the
 * `model` parameter is written only when a card is chosen. Nothing here
 * previews by switching, which would be audible, would push entries onto
 * the undo history and would fight automation on that parameter. Leaving by
 * Escape, by the close button or by choosing the compressor already loaded
 * writes nothing at all.
 *
 * Each entry shows the model's real faceplate (small and inert, mounted
 * lazily; see `FacePreview.vue`), its name, its family and a sentence of
 * character, because the point is choosing rather than switching.
 *
 * A new model needs an entry in `MODELS` and nothing here: families come
 * from the registry and the grid flows.
 */
import { computed, onBeforeUnmount, onMounted } from 'vue';
import { FAMILIES, MODELS, ui, useLab } from '../composables/useLab.js';
import FacePreview from './FacePreview.vue';

const lab = useLab();
const current = computed(() => lab.model.index);
/** `lab.active` is a ref on a plain object, so it does not unwrap in the template. */
const loaded = computed(() => lab.active.value);

/** Families that actually have models, so one never shows up empty. */
const groups = computed(() =>
  FAMILIES.map((f) => ({
    ...f,
    models: MODELS.map((m, index) => ({ ...m, index })).filter((m) => m.family === f.id),
  })).filter((g) => g.models.length > 0),
);

function choose(index) {
  if (index !== current.value) {
    lab.model.begin();
    lab.model.setIndex(index);
    lab.model.end();
  }
  ui.browsing = false;
}
/** Leave without choosing: nothing has been written, so there is nothing to undo. */
function cancel() {
  ui.browsing = false;
}
function onKey(e) {
  if (e.key === 'Escape') {
    e.preventDefault();
    cancel();
  }
}
onMounted(() => window.addEventListener('keydown', onKey));
onBeforeUnmount(() => window.removeEventListener('keydown', onKey));
</script>

<template>
  <main class="browse" role="region" aria-label="Change compressor">
    <header class="browse__head">
      <div>
        <h2 class="browse__title">Change compressor</h2>
        <p class="browse__lede">
          Your current compressor is still running while you look. Nothing changes until you pick one, and each compressor keeps its own
          settings, so you can try another and come back to find this one exactly as you left it.
        </p>
      </div>
      <button class="browse__close" type="button" title="Keep the compressor I have (Escape)" @click="cancel">Keep {{ loaded.label }}</button>
    </header>

    <div class="browse__scroll">
      <section v-for="g in groups" :key="g.id" class="browse__group">
        <div class="browse__family">
          <span class="browse__familyname">{{ g.label }}</span>
          <span class="browse__familynote">{{ g.note }}</span>
        </div>
        <div class="browse__grid">
          <div v-for="m in g.models" :key="m.key" class="browse__cell">
          <button
            class="browse__card"
            :class="{ on: m.index === current }"
            type="button"
            :aria-current="m.index === current ? 'true' : undefined"
            @click="choose(m.index)"
          >
            <span class="browse__preview"><FacePreview :model-key="m.key" /></span>
            <span class="browse__meta">
              <span class="browse__name">{{ m.label }}<span v-if="m.index === current" class="browse__badge">loaded</span></span>
              <span class="browse__sub">{{ m.sub }}</span>
              <span class="browse__blurb">{{ m.blurb }}</span>
              <span class="browse__uses"><b>Good for</b> {{ m.uses }}</span>
            </span>
          </button>
          </div>
        </div>
      </section>
    </div>
  </main>
</template>

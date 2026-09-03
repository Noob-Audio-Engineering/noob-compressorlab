<script setup>
/**
 * The development panel: this plug-in's diagnostic window onto the bridge,
 * shown below the workbench.
 *
 * It is deliberately **not** part of any of the six units. Nothing here is
 * modelled on hardware; it exists so that the bridge, the streams and the
 * parameters can be watched while the plug-in is being built or
 * demonstrated, and it says so in its own footer.
 *
 * **It costs nothing when it is not on screen.** The values are read by
 * polling, and the poll only runs while the panel is both enabled and
 * expanded; collapsing it or turning it off clears the timer, so a hidden
 * panel does no work at all. Polling a few times a second rather than every
 * frame is the honest rate: these are diagnostics, not meters, and the
 * numbers are legible only if they hold still long enough to read.
 *
 * What it shows that nothing else on the page does: the bridge's own
 * statistics, every stream's sequence number and current values laid out by
 * the layout string in its meta, and every parameter's raw normalised value
 * beside its plain value. That last pair is the point: the two disagreeing
 * is exactly the class of fault this project's audits kept finding.
 *
 * The demo source lives here too, controls and state together. It belongs
 * with the diagnostics rather than in the bench bar: it is a test-signal
 * generator, it is not automatable, and it is compiled into the standalone
 * only, so in a host it does not exist at all. Everything left in the bench
 * bar changes real audio; this only decides what audio there is.
 *
 * Props: `open` (whether it is expanded). Emits: `update:open`.
 */
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { Knob, Segmented } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';
import { MODELS, getClient, useLab, useNoobVstWebguiFramework } from '../composables/useLab.js';

const props = defineProps({
  open: { type: Boolean, default: true },
});
const emit = defineEmits(['update:open']);

const { connected, stats, status, client } = useNoobVstWebguiFramework();
const lab = useLab();

/** Everything the panel draws, refreshed by the poll below. */
const snap = ref({ streams: [], params: [], input: null });
const POLL_MS = 250;
let timer = null;

const active = computed(() => MODELS[lab.model.index] || MODELS[0]);
/** The active model's own ids plus the ones every model shares. */
const SHARED = ['model', 'link', 'mix', 'sc_hpf', 'bypass', 'src_kind', 'src_level', 'src_freq'];

const SOURCES = ['VOCAL', 'BASS', 'DRUMS', 'PINK', 'WHITE', 'SAW', 'SINE'];
/**
 * Which generators the pitch control actually reaches. Vocal and bass build
 * their notes from it and the saw and sine are simply at it; the drum loop is
 * a fixed 120 BPM pattern and neither noise has a pitch to set. Saying so is
 * the difference between a diagnostic and a row of knobs.
 */
const PITCHED = new Set([0, 1, 5, 6]);

const dbfs = (a) => (a > 1e-6 ? 20 * Math.log10(a) : -Infinity);

function fmt(v, dp = 3) {
  if (v === null || v === undefined || Number.isNaN(v)) return '–';
  if (!Number.isFinite(v)) return String(v);
  return Math.abs(v) >= 1000 ? v.toFixed(0) : v.toFixed(dp);
}

function sample() {
  const c = getClient();
  if (!c || !c.manifest) return;
  const now = performance.now();
  const owns = active.value.owns;

  const streams = (c.streams || []).map((s) => {
    const layout = (s.meta && s.meta.layout ? String(s.meta.layout) : '').split(',').map((x) => x.trim()).filter(Boolean);
    const d = s.data || [];
    const values = [];
    for (let i = 0; i < d.length && i < 12; i++) values.push({ name: layout[i] || `[${i}]`, v: d[i] });
    return {
      id: s.id,
      kind: s.kind,
      seq: s.seq,
      ageMs: s.ts ? Math.max(0, Math.round(now - s.ts)) : null,
      len: d.length,
      capacity: s.capacity,
      // a curve is too long to list, so show its span and ends instead
      curve: s.kind === 'curve' && d.length > 2 ? { first: d[0], last: d[d.length - 1] } : null,
      values: s.kind === 'curve' ? [] : values,
    };
  });

  const params = (c.params || [])
    .filter((p) => SHARED.includes(p.id) || owns.some((prefix) => p.id.startsWith(prefix)))
    // `text` and `steps` are not fields on the client's parameter object:
    // the display string comes from `format()` and the step count lives on
    // the spec. Reading them as plain properties left both columns blank.
    .map((p) => ({
      id: p.id,
      norm: p.norm,
      plain: p.plain,
      text: typeof p.format === 'function' ? p.format() : '',
      steps: (p.spec && p.spec.steps) || 0,
    }));

  // What is actually arriving at the input, whoever is producing it. This is
  // the only honest answer to "is the source feeding anything": the setting
  // says what should be generated, this says what the compressor received.
  let input = null;
  const meter = (c.streams || []).find((st) => st.meta && String(st.meta.layout || '').includes('in_l'));
  if (meter && meter.data && meter.data.length) {
    const names = String(meter.meta.layout).split(',').map((x) => x.trim());
    const l = meter.data[names.indexOf('in_l')];
    const r = meter.data[names.indexOf('in_r')];
    const peak = Math.max(Math.abs(l || 0), Math.abs(r || 0));
    input = { peak, db: dbfs(peak), fresh: meter.ts ? now - meter.ts < 1000 : false };
  }

  snap.value = { streams, params, input };
}

function start() {
  stop();
  sample();
  timer = setInterval(sample, POLL_MS);
}
function stop() {
  if (timer) clearInterval(timer);
  timer = null;
}
watch(() => props.open, (o) => (o ? start() : stop()), { immediate: true });
onBeforeUnmount(stop);

/**
 * The demo source's state, not just its controls. In the plug-in the three
 * parameters do not exist at all — `plugin.rs` never registers them, the
 * generator is compiled only into the standalone — so there is nothing to
 * show and nothing to imply: the host is the input, and the panel says so.
 */
const source = computed(() => {
  const src = lab.source;
  const input = snap.value.input;
  if (!src) return { present: false, input };
  const kind = Math.round(src.kind.index ?? 0);
  return {
    present: true,
    kind,
    name: SOURCES[kind] || '?',
    level: src.level.plain,
    levelDb: dbfs(src.level.plain),
    freq: src.freq.plain,
    pitched: PITCHED.has(kind),
    input,
  };
});

/** dBFS for display, with a real minus sign and an honest floor. */
function db(v) {
  if (v === null || v === undefined || Number.isNaN(v)) return '–';
  if (!Number.isFinite(v)) return '−∞ dBFS';
  return `${v < 0 ? '−' : ''}${Math.abs(v).toFixed(1)} dBFS`;
}

/** Whether anything is actually reaching the input, and from where. */
const feeding = computed(() => {
  const s = source.value;
  const i = s.input;
  if (!i) return s.present ? 'no meter stream to check against' : 'the host feeds the input; no meter stream to check against';
  if (!i.fresh) return 'the meter has stopped updating — nothing is running';
  if (i.peak < 1e-5) return `silent — the input is reading ${db(i.db)}`;
  if (s.present) return `yes — the compressor's input is at ${db(i.db)}`;
  // No generator in this build: whatever is arriving comes from outside it.
  return client.offline
    ? `design-mode frames are feeding the input, at ${db(i.db)}`
    : `the host is feeding the input, at ${db(i.db)}`;
});

const bridge = computed(() => ({
  state: client.offline ? 'offline (design mode)' : connected.value ? 'live' : 'connecting',
  url: (client && client.url) || '–',
  protocol: (client.manifest && client.manifest.protocol) || '–',
  standalone: !!(client.manifest && client.manifest.meta && client.manifest.meta.standalone),
  sampleRate: (client.manifest && client.manifest.meta && client.manifest.meta.sample_rate) || null,
}));
</script>

<template>
  <section class="dbg" :class="{ open }">
    <button class="dbg__bar" type="button" :aria-expanded="open" @click="emit('update:open', !open)">
      <span class="dbg__caret">{{ open ? '▾' : '▸' }}</span>
      <span class="dbg__title">Development panel</span>
      <span class="dbg__hint">{{ bridge.state }}<template v-if="!open"> · collapsed, not polling</template></span>
    </button>

    <div v-if="open" class="dbg__body">
      <div class="dbg__grid">
        <section class="dbg__card">
          <h4>Bridge</h4>
          <dl>
            <dt>state</dt><dd>{{ bridge.state }}</dd>
            <dt>url</dt><dd class="wrap">{{ bridge.url }}</dd>
            <dt>protocol</dt><dd>{{ bridge.protocol }}</dd>
            <dt>standalone</dt><dd>{{ bridge.standalone ? 'yes' : 'no' }}</dd>
            <dt>sample rate</dt><dd>{{ bridge.sampleRate ? `${bridge.sampleRate} Hz` : '–' }}</dd>
            <dt>edit→echo</dt><dd>{{ Number.isNaN(stats.echoAvgMs) ? '–' : `${(stats.echoAvgMs * 1000).toFixed(0)} µs` }}</dd>
            <dt>frames/s</dt><dd>{{ fmt(stats.fps, 1) }}</dd>
            <dt>bytes in</dt><dd>{{ stats.bytesIn ?? '–' }}</dd>
            <dt>frames in</dt><dd>{{ stats.framesIn ?? '–' }}</dd>
          </dl>
        </section>

        <section class="dbg__card">
          <h4>Model</h4>
          <dl>
            <dt>loaded</dt><dd>{{ active.label }}</dd>
            <dt>key</dt><dd>{{ active.key }}</dd>
            <dt>family</dt><dd>{{ active.family }}</dd>
            <dt>owns</dt><dd>{{ active.owns.join(', ') }}</dd>
            <dt>latency</dt><dd>{{ status?.latency_ms != null ? `${status.latency_ms.toFixed(2)} ms` : '–' }}</dd>
          </dl>
        </section>
      </div>

      <section class="dbg__card">
        <h4>Demo source <span class="dim">— the standalone's test signal, not part of any of these units</span></h4>
        <template v-if="source.present">
          <div class="dbg__srcrow">
            <Segmented :p="lab.source.kind" :labels="SOURCES" />
            <Knob :p="lab.source.level" :size="40" label="Level" />
            <Knob :p="lab.source.freq" :size="40" label="Pitch" />
          </div>
          <dl class="dbg__srcstate">
            <dt>generator</dt><dd>{{ source.name }}</dd>
            <dt>level</dt><dd>{{ fmt(source.level, 2) }} · {{ db(source.levelDb) }}</dd>
            <dt>pitch</dt>
            <dd>
              {{ fmt(source.freq, 1) }} Hz
              <span v-if="!source.pitched" class="dim">— {{ source.name }} does not use it</span>
            </dd>
            <dt>feeding</dt><dd>{{ feeding }}</dd>
          </dl>
        </template>
        <p v-else class="dbg__note">
          The generator is compiled into the standalone only, so in a host these three parameters do not exist and there is nothing here to
          set: {{ feeding }}.
        </p>
      </section>

      <section class="dbg__card">
        <h4>Streams</h4>
        <table class="dbg__table">
          <thead>
            <tr><th>id</th><th>kind</th><th>seq</th><th>age</th><th>len</th><th>values</th></tr>
          </thead>
          <tbody>
            <tr v-for="s in snap.streams" :key="s.id">
              <td>{{ s.id }}</td>
              <td class="dim">{{ s.kind }}</td>
              <td>{{ s.seq }}</td>
              <td class="dim">{{ s.ageMs === null ? '–' : `${s.ageMs} ms` }}</td>
              <td class="dim">{{ s.len }}/{{ s.capacity }}</td>
              <td>
                <template v-if="s.curve">first {{ fmt(s.curve.first, 2) }} · last {{ fmt(s.curve.last, 2) }}</template>
                <span v-for="v in s.values" v-else :key="v.name" class="dbg__val"><i>{{ v.name }}</i>{{ fmt(v.v) }}</span>
              </td>
            </tr>
          </tbody>
        </table>
      </section>

      <section class="dbg__card">
        <h4>Parameters <span class="dim">— {{ active.label }} and the shared ones, raw beside plain</span></h4>
        <table class="dbg__table">
          <thead>
            <tr><th>id</th><th>norm</th><th>plain</th><th>shown</th><th>steps</th></tr>
          </thead>
          <tbody>
            <tr v-for="p in snap.params" :key="p.id">
              <td>{{ p.id }}</td>
              <td>{{ fmt(p.norm) }}</td>
              <td>{{ fmt(p.plain) }}</td>
              <td class="dim">{{ p.text }}</td>
              <td class="dim">{{ p.steps || '' }}</td>
            </tr>
          </tbody>
        </table>
      </section>

      <p class="dbg__foot">
        A development view, not part of any of these units: no CL-1B, 1176 or LA-2A has a panel like this, and none of them generates its own
        signal. It reads the bridge a few times a second while it is open, and stops entirely when collapsed or switched off.
      </p>
    </div>
  </section>
</template>

/**
 * Design-time manifest for Noob CompressorLab: what the plug-in publishes,
 * described up front so the page renders before (or without) the plug-in.
 * Ids, ranges, labels, defaults and stream layouts mirror `src/dsp/mod.rs`
 * exactly; keep them in step. Only development builds load this (see
 * `main.js`), and the client hands over to the real server the moment
 * `/ws` answers.
 *
 * The frame generators follow the model switch the way the plug-in does:
 * under the 1176 the meter breathes with a drum loop, under the LA-2A with
 * vocal-like syllables and the T4 cell lights up; the sticky transfer
 * curve is republished whenever the model changes. `gr_db` is at most 0
 * (a gain change in dB) and `meter_vu` is what the active model's needle
 * shows for the selected meter mode, both as the contract with the Rust
 * side says.
 */
import { getClient } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';

/** The plain value of a parameter, read from the (offline) client at frame time. */
function plain(id, fallback = 0) {
  try {
    const p = getClient().param(id);
    return p ? p.plain : fallback;
  } catch {
    return fallback;
  }
}
const db = (a) => (a > 0 ? 20 * Math.log10(a) : -120);
/** The VU reference: +4 dBu reads 0 VU at −18 dBFS. */
const VU_REF_DBFS = -18;

export const offline = {
  name: 'noob-compressorlab',
  meta: { vendor: 'Ely Erin Fox', version: 'dev', sample_rate: 48000, vu_ref_dbfs: VU_REF_DBFS, transfer_points: 128, standalone: true },
  params: [
    { id: 'model', name: 'Model', labels: ['1176', 'LA-2A'], default: 0, group: 'lab', automatable: false },

    { id: 'fet_input', name: 'Input', min: 0, max: 48, default: 24, group: '1176' },
    { id: 'fet_output', name: 'Output', min: 0, max: 48, default: 24, group: '1176' },
    { id: 'fet_attack', name: 'Attack', min: 0, max: 7, default: 4, group: '1176' },
    { id: 'fet_release', name: 'Release', min: 1, max: 7, default: 4, group: '1176' },
    { id: 'fet_ratio', name: 'Ratio', labels: ['4', '8', '12', '20', 'All'], default: 0, group: '1176' },
    { id: 'fet_meter', name: 'Meter', labels: ['GR', '+4', '+8', 'Off'], default: 0, group: '1176', automatable: false },
    { id: 'fet_revision', name: 'Revision', labels: ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'LN'], default: 8, group: '1176', automatable: false },

    { id: 'opto_gain', name: 'Gain', min: 0, max: 100, default: 32, group: 'LA-2A' },
    { id: 'opto_peak_reduction', name: 'Peak Reduction', min: 0, max: 100, default: 40, group: 'LA-2A' },
    { id: 'opto_mode', name: 'Mode', labels: ['Compress', 'Limit'], default: 0, group: 'LA-2A' },
    { id: 'opto_meter', name: 'Meter', labels: ['Gain Reduction', 'Output +10', 'Output +4'], default: 0, group: 'LA-2A', automatable: false },
    { id: 'opto_emphasis', name: 'Emphasis (R37)', min: 0, max: 1, default: 1, group: 'LA-2A' },
    { id: 'opto_cell', name: 'Cell', labels: ['Silver', 'Gray', 'LA-2'], default: 1, group: 'LA-2A', automatable: false },

    { id: 'link', name: 'Stereo Link', toggle: true, default: 1, group: 'extras' },
    { id: 'mix', name: 'Mix', min: 0, max: 100, default: 100, unit: '%', group: 'extras' },
    { id: 'sc_hpf', name: 'Side-chain HPF', min: 0, max: 300, default: 0, unit: 'Hz', group: 'extras' },
    { id: 'bypass', name: 'Bypass', toggle: true, default: 0, group: 'extras' },

    { id: 'src_kind', name: 'Source', labels: ['Vocal', 'Bass', 'Drums', 'Pink noise', 'White noise', 'Saw', 'Sine'], default: 0, group: 'source', automatable: false },
    { id: 'src_level', name: 'Source Level', min: 0, max: 1, default: 0.4, group: 'source', automatable: false },
    { id: 'src_freq', name: 'Source Frequency', min: 20, max: 20000, default: 110, taper: 'log', unit: 'Hz', group: 'source', automatable: false },
  ],
  streams: [
    { id: 'meter', name: 'Meter', kind: 'meter', capacity: 6, channels: 2, meta: { layout: 'in_l,in_r,out_l,out_r,gr_db,meter_vu', vu_ref_dbfs: VU_REF_DBFS, sample_rate: 48000 } },
    { id: 'cell', name: 'T4 cell', kind: 'raw', capacity: 3, channels: 1, meta: { layout: 'light,free_carriers,trapped_carriers' } },
    { id: 'transfer', name: 'Transfer curve', kind: 'curve', capacity: 128, channels: 1, sticky: true, meta: { in_db: [-60, 0], unit: 'dBFS' } },
  ],
  frames: {
    meter: (t) => {
      const opto = Math.round(plain('model')) === 1;
      let inl;
      let gr;
      if (opto) {
        // vocal-like syllables, a slow optical release
        const syllable = (t % 0.55) / 0.55;
        const env = syllable < 0.7 ? 1 - 0.4 * syllable : 0.15;
        gr = -(4 + 6 * env * (0.6 + 0.4 * Math.sin(t * 0.7)) + 0.4 * Math.abs(Math.sin(t * 5)));
        inl = 0.35 * env * (0.9 + 0.1 * Math.sin(t * 13));
      } else {
        // a drum loop, fast FET grabs
        const beat = Math.max(0, Math.sin(t * 2 * Math.PI * 1.9)) ** 8;
        inl = 0.18 + 0.6 * beat;
        gr = -12 * beat - 1.5 * Math.abs(Math.sin(t * 0.7));
      }
      const outl = inl * 10 ** (gr / 20) * (opto ? 1.6 : 1);
      const outDb = db(outl);
      // what the needle shows: the reduction in GR modes, the output level against the meter's zero otherwise
      const mode = Math.round(plain(opto ? 'opto_meter' : 'fet_meter'));
      const vu = opto
        ? [gr, outDb - (VU_REF_DBFS + 6), outDb - VU_REF_DBFS][mode] // +10 reads 6 dB lower than +4
        : [gr, outDb - VU_REF_DBFS, outDb - (VU_REF_DBFS + 4), -60][mode]; // +8 reads 4 dB lower than +4, Off parks the needle
      return [inl, inl * 0.96, outl, outl * 0.96, gr, vu ?? gr];
    },
    cell: (() => {
      let dark = false; // under the 1176 the cell publishes zeros once, then nothing
      return (t) => {
        if (Math.round(plain('model')) !== 1) {
          if (dark) return null;
          dark = true;
          return [0, 0, 0];
        }
        dark = false;
        // the model's own units: light around 1e-5..1e-4 at working levels, carriers around 1e-3
        const gr = 4 + 6 * (0.6 + 0.4 * Math.sin(t * 0.7));
        const light = 3e-6 * 10 ** (3 * Math.min(1, gr / 20));
        return [light, 1.2e-3 * (0.7 + 0.6 * Math.min(1, light / 1e-4)), 1e-3 + 0.5e-3 * Math.sin(t * 0.2)];
      };
    })(),
    transfer: (() => {
      let last = -1;
      return () => {
        const m = Math.round(plain('model'));
        if (m === last) return null; // sticky: publish once per model
        last = m;
        const [knee, ratio, width] = m === 1 ? [-30, 3, 8] : [-26, 4, 6];
        const out = new Float32Array(128);
        for (let i = 0; i < 128; i++) {
          const x = -60 + (60 * i) / 127;
          const over = Math.max(0, x - knee);
          out[i] = x - over * (1 - 1 / ratio) * (1 - Math.exp(-over / width));
        }
        return out;
      };
    })(),
  },
  timeoutMs: 1200,
};

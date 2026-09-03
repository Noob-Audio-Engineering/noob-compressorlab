/**
 * Design-time manifest for Noob CompressorLab: what the plug-in publishes,
 * described up front so the page renders before (or without) the plug-in.
 * Ids, ranges, labels, defaults and stream layouts mirror `src/dsp/mod.rs`
 * exactly; keep them in step. Only development builds load this (see
 * `main.js`), and the client hands over to the real server the moment
 * `/ws` answers.
 *
 * The frame generators follow the model switch the way the plug-in does:
 * under the 1176 the meter breathes with a drum loop, under the two optical
 * models with vocal-like syllables and the T4 cell lights up, under the
 * Distressor with a fast VCA grab whose depth follows the ratio, and under
 * the 6176 with the preamp's own VU on top of the 1176's behaviour. The
 * sticky transfer curve is republished whenever the model (or the
 * Distressor's ratio) changes. `gr_db` is at most 0 (a gain change in dB)
 * and `meter_vu` is what the active model's needle shows for the selected
 * meter mode, both as the contract with the Rust side says.
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

/** With `?nosource`, the parameter set a host would publish. */
function dropSource(list) {
  const off = typeof location !== 'undefined' && new URLSearchParams(location.search).has('nosource');
  return off ? list.filter((p) => p.group !== 'source') : list;
}
/** The VU reference: +4 dBu reads 0 VU at −18 dBFS. */
const VU_REF_DBFS = -18;
/** The model switch as a key, in the order of the `model` parameter's steps. */
const MODEL_KEYS = ['fet', 'opto', 'la3a', 'vca', 'pre6176', 'cl1b'];
const modelKey = () => MODEL_KEYS[Math.round(plain('model'))] || 'fet';

/**
 * The Distressor's eight ratios as the offline meter and transfer curve see
 * them: the effective slope and the knee width of `research/Distressor.md`
 * 7.4, rounded. Estimates, like everything in that table.
 */
const DIST_RATIOS = [
  { ratio: 1, threshold: 0, width: 1, depth: 0 },
  { ratio: 2.3, threshold: -6, width: 30, depth: 0.35 },
  { ratio: 3.3, threshold: -8, width: 24, depth: 0.5 },
  { ratio: 4.5, threshold: -12, width: 12, depth: 0.7 },
  { ratio: 6.5, threshold: -14, width: 10, depth: 0.85 },
  { ratio: 10, threshold: -16, width: 8, depth: 1 },
  { ratio: 20, threshold: -18, width: 3, depth: 1.2 },
  { ratio: 40, threshold: -16, width: 1.5, depth: 1.6 },
];

/**
 * Give every labelled parameter the range the plug-in publishes for it:
 * 0 to the number of steps less one, the index of its label. The framework's
 * offline mock derives the default's normalized position from `min` and
 * `max`, and a labelled parameter left without them is read as 0 to 1, so
 * any default past the first step lands on the last one. Stating the range
 * is what the real manifest does anyway.
 */
const stepped = (list) => list.map((p) => (p.labels && p.max == null ? { min: 0, max: p.labels.length - 1, ...p } : p));

export const offline = {
  name: 'noob-compressorlab',
  meta: { vendor: 'Noob Audio Engineering', version: 'dev', sample_rate: 48000, vu_ref_dbfs: VU_REF_DBFS, transfer_points: 128, standalone: true },
  // `?nosource` drops the three demo-source parameters, reproducing what a
  // host sees: `plugin.rs` never registers them, so the development panel's
  // source card has to say the host is the input rather than offer controls.
  // It exists so that presentation can be checked without a host.
  params: dropSource(stepped([
    { id: 'model', name: 'Model', labels: ['1176', 'LA-2A', 'LA-3A', 'Distressor', '6176', 'CL-1B'], default: 0, group: 'lab', automatable: false },

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
    { id: 'opto_meter_zero', name: 'Meter Zero', min: -2, max: 2, default: 0, unit: 'dB', group: 'LA-2A', automatable: false },

    { id: 'la3a_gain', name: 'Gain', min: 0, max: 100, default: 32, group: 'LA-3A' },
    { id: 'la3a_peak_reduction', name: 'Peak Reduction', min: 0, max: 100, default: 40, group: 'LA-3A' },
    { id: 'la3a_mode', name: 'Mode', labels: ['Compress', 'Limit'], default: 0, group: 'LA-3A' },
    { id: 'la3a_meter', name: 'Meter', labels: ['Gain Reduction', 'Output', 'Off'], default: 0, group: 'LA-3A', automatable: false },
    { id: 'la3a_emphasis', name: 'HF Contour', min: 0, max: 1, default: 0, group: 'LA-3A' },
    { id: 'la3a_cell', name: 'Cell', labels: ['Fresh', 'Used', 'Tired'], default: 0, group: 'LA-3A', automatable: false },
    // CL-1B. Every continuous control is the pot's own travel, 0..1, because the panel print is
    // approximate and the displayed value comes from the pot's law (research/CL-1B.md 9.3). The
    // defaults are Lydkraft's own published vocal setting.
    { id: 'cl1b_gain', name: 'Gain', min: 0, max: 1, default: 0.265, group: 'CL-1B' },
    { id: 'cl1b_ratio', name: 'Ratio', min: 0, max: 1, default: 0.375, group: 'CL-1B' },
    { id: 'cl1b_threshold', name: 'Threshold', min: 0, max: 1, default: 0.5, group: 'CL-1B' },
    { id: 'cl1b_attack', name: 'Attack', min: 0, max: 1, default: 0.75, group: 'CL-1B' },
    { id: 'cl1b_release', name: 'Release', min: 0, max: 1, default: 0.25, group: 'CL-1B' },
    { id: 'cl1b_mode', name: 'Attack/Release Select', labels: ['Fixed', 'Fix/Man', 'Manual'], default: 2, group: 'CL-1B' },
    { id: 'cl1b_meter', name: 'Meter', labels: ['Input', 'Compression', 'Output'], default: 1, group: 'CL-1B', automatable: false },
    { id: 'cl1b_bus', name: 'Sidechain Bus', labels: ['Off', '1', '2'], default: 0, group: 'CL-1B' },
    { id: 'cl1b_power', name: 'Power', toggle: true, default: 1, group: 'CL-1B', automatable: false },

    { id: 'dist_input', name: 'Input', min: 0, max: 10.5, default: 5, group: 'Distressor' },
    { id: 'dist_output', name: 'Output', min: 0, max: 10.5, default: 5, group: 'Distressor' },
    { id: 'dist_attack', name: 'Attack', min: 0, max: 10.5, default: 5, group: 'Distressor' },
    { id: 'dist_release', name: 'Release', min: 0, max: 10.5, default: 5, group: 'Distressor' },
    { id: 'dist_ratio', name: 'Ratio', labels: ['1:1', '2:1', '3:1', '4:1', '6:1', '10:1', '20:1', 'Nuke'], default: 4, group: 'Distressor' },
    { id: 'dist_detector', name: 'Detector', labels: ['Norm', 'HP', 'Band', 'HP+Band'], default: 0, group: 'Distressor' },
    { id: 'dist_audio', name: 'Audio', labels: ['Norm', 'HP', 'Dist 2', 'Dist 3', 'HP+Dist 2', 'HP+Dist 3'], default: 0, group: 'Distressor' },
    { id: 'dist_british', name: 'British Mode', toggle: true, default: 0, group: 'Distressor' },
    { id: 'dist_link_mode', name: 'Link Mode', labels: ['Phase', 'Image', 'Both'], default: 0, group: 'Distressor', automatable: false },
    { id: 'dist_headroom', name: 'Headroom', min: 4, max: 28, default: 16, unit: 'dB', group: 'Distressor', automatable: false },

    { id: 'pre_join', name: 'Routing', labels: ['Join', 'BP', '1:1'], default: 0, group: '6176' },
    { id: 'pre_gain', name: 'Gain', labels: ['-10', '-5', '0', '+5', '+10'], default: 2, group: '6176' },
    { id: 'pre_input', name: 'Input', labels: ['Line', 'Mic 500', 'Mic 2.0K', 'Hi-Z 47K', 'Hi-Z 2.2M'], default: 0, group: '6176', automatable: false },
    { id: 'pre_pad', name: 'Pad', toggle: true, default: 0, group: '6176' },
    { id: 'pre_polarity', name: 'Polarity', toggle: true, default: 0, group: '6176' },
    { id: 'pre_level', name: 'Level', min: 0, max: 10, default: 7, group: '6176' },
    { id: 'pre_lf_freq', name: 'Low Freq', labels: ['70', '100', '200'], default: 1, group: '6176' },
    { id: 'pre_lf_gain', name: 'Low', labels: ['-9', '-6', '-4.5', '-3', '-1.5', '0', '+1.5', '+3', '+4.5', '+6', '+9'], default: 5, group: '6176' },
    { id: 'pre_hf_freq', name: 'High Freq', labels: ['4.5k', '7k', '10k'], default: 2, group: '6176' },
    { id: 'pre_hf_gain', name: 'High', labels: ['-9', '-6', '-4.5', '-3', '-1.5', '0', '+1.5', '+3', '+4.5', '+6', '+9'], default: 5, group: '6176' },
    { id: 'pre_hpf', name: 'Low Cut', toggle: true, default: 0, group: '6176' },
    { id: 'pre_voice', name: 'Voicing', labels: ['610B', '610A'], default: 0, group: '6176', automatable: false },
    { id: 'pre_load', name: 'Output Load', labels: ['15k', '600'], default: 0, group: '6176', automatable: false },
    { id: 'pre_meter', name: 'Meter', labels: ['PRE', 'GR', 'COMP'], default: 1, group: '6176', automatable: false },
    { id: 'pre_phantom', name: 'Phantom Power', toggle: true, default: 0, group: '6176', automatable: false },

    { id: 'link', name: 'Stereo Link', toggle: true, default: 1, group: 'extras' },
    { id: 'mix', name: 'Mix', min: 0, max: 100, default: 100, unit: '%', group: 'extras' },
    { id: 'sc_hpf', name: 'Side-chain HPF', min: 0, max: 300, default: 0, unit: 'Hz', group: 'extras' },
    { id: 'bypass', name: 'Bypass', toggle: true, default: 0, group: 'extras' },

    { id: 'src_kind', name: 'Source', labels: ['Vocal', 'Bass', 'Drums', 'Pink noise', 'White noise', 'Saw', 'Sine'], default: 0, group: 'source', automatable: false },
    { id: 'src_level', name: 'Source Level', min: 0, max: 1, default: 0.4, group: 'source', automatable: false },
    { id: 'src_freq', name: 'Source Frequency', min: 20, max: 20000, default: 110, taper: 'log', unit: 'Hz', group: 'source', automatable: false },
  ])),
  streams: [
    // `meter_vu` is where the needle already is: the VU movement's ballistics
    // (13 rad/s, damping 0.80) run in the audio thread, so a page draws the
    // field rather than smoothing it again.
    { id: 'meter', name: 'Meter', kind: 'meter', capacity: 6, channels: 2, meta: { layout: 'in_l,in_r,out_l,out_r,gr_db,meter_vu', vu_ref_dbfs: VU_REF_DBFS, sample_rate: 48000, ballistics: 'engine' } },
    { id: 'cell', name: 'T4 cell', kind: 'raw', capacity: 3, channels: 1, meta: { layout: 'light,free_carriers,trapped_carriers' } },
    { id: 'lamps', name: 'Lamps', kind: 'raw', capacity: 4, channels: 1, meta: { layout: 'thd_pct,redline,pre_vu_db,drive' } },
    { id: 'transfer', name: 'Transfer curve', kind: 'curve', capacity: 128, channels: 1, sticky: true, meta: { in_db: [-60, 0], unit: 'dBFS' } },
  ],
  frames: {
    meter: (t) => {
      const key = modelKey();
      const optical = key === 'opto' || key === 'la3a' || key === 'cl1b';
      let inl;
      let gr;
      if (optical) {
        // vocal-like syllables; the LA-3A grabs harder and lets go sooner
        const fast = key === 'la3a';
        const syllable = (t % 0.55) / 0.55;
        const env = syllable < 0.7 ? 1 - 0.4 * syllable : 0.15;
        const depth = fast ? 8 : 6;
        gr = -(4 + depth * env * (0.6 + 0.4 * Math.sin(t * (fast ? 1.1 : 0.7))) + 0.4 * Math.abs(Math.sin(t * 5)));
        inl = 0.35 * env * (0.9 + 0.1 * Math.sin(t * 13));
      } else if (key === 'vca') {
        // a fast VCA on a drum loop: the ratio sets how deep it goes
        const r = DIST_RATIOS[Math.round(plain('dist_ratio', 4))] || DIST_RATIOS[4];
        const beat = Math.max(0, Math.sin(t * 2 * Math.PI * 2.1)) ** 6;
        inl = 0.2 + 0.6 * beat;
        gr = -(14 * r.depth * beat + 2 * r.depth * Math.abs(Math.sin(t * 0.9)));
      } else {
        // a drum loop, fast FET grabs (the 1176 and the 6176's compressor half)
        const beat = Math.max(0, Math.sin(t * 2 * Math.PI * 1.9)) ** 8;
        inl = 0.18 + 0.6 * beat;
        gr = -12 * beat - 1.5 * Math.abs(Math.sin(t * 0.7));
      }
      const outl = inl * 10 ** (gr / 20) * (optical ? 1.6 : 1);
      const outDb = db(outl);
      // what the needle (or the bargraph) shows: the reduction in the GR modes,
      // the output level against the meter's zero otherwise
      let vu;
      if (key === 'vca') {
        vu = gr;
      } else if (key === 'la3a') {
        // two hardware positions, GR and OUTPUT (0 VU is +4 dBm), plus the plug-in's own Off
        const mode = Math.round(plain('la3a_meter'));
        vu = [gr, outDb - VU_REF_DBFS, -60][mode];
      } else if (optical) {
        const mode = Math.round(plain('opto_meter'));
        vu = [gr, outDb - (VU_REF_DBFS + 6), outDb - VU_REF_DBFS][mode]; // +10 reads 6 dB lower than +4
      } else if (key === 'cl1b') {
        // the hardware's three positions: input, compression, output (0 VU is +4 dBu)
        const mode = Math.round(plain('cl1b_meter', 1));
        vu = [db(inl) - VU_REF_DBFS, gr, outDb - VU_REF_DBFS][mode];
      } else if (key === 'pre6176') {
        const mode = Math.round(plain('pre_meter', 1));
        vu = [preVuDb(), gr, outDb - VU_REF_DBFS][mode];
      } else {
        const mode = Math.round(plain('fet_meter'));
        vu = [gr, outDb - VU_REF_DBFS, outDb - (VU_REF_DBFS + 4), -60][mode]; // +8 reads 4 dB lower, Off parks the needle
      }
      return [inl, inl * 0.96, outl, outl * 0.96, gr, vu ?? gr];
    },
    cell: (() => {
      let dark = false; // away from the optical models the cell publishes zeros once, then nothing
      return (t) => {
        const key = modelKey();
        if (key !== 'opto' && key !== 'la3a') {
          if (dark) return null;
          dark = true;
          return [0, 0, 0];
        }
        dark = false;
        // the model's own units: light around 1e-5..1e-4 at working levels, carriers around 1e-3.
        // The LA-3A drives the same cell harder, so its light sits a decade up.
        const drive = key === 'la3a' ? 3 : 1;
        const gr = 4 + 6 * (0.6 + 0.4 * Math.sin(t * 0.7));
        const light = 3e-6 * drive * 10 ** (3 * Math.min(1, gr / 20));
        return [light, 1.2e-3 * (0.7 + 0.6 * Math.min(1, light / 1e-4)), 1e-3 + 0.5e-3 * Math.sin(t * 0.2)];
      };
    })(),
    lamps: (() => {
      let dark = false; // only the Distressor and the 6176 have lamps to light
      return (t) => {
        const key = modelKey();
        if (key !== 'vca' && key !== 'pre6176') {
          if (dark) return null;
          dark = true;
          return [0, 0, 0, 0];
        }
        dark = false;
        if (key === 'vca') {
          // the generator's THD: Dist 2 reaches a few per cent, Dist 3 goes to twenty
          const audio = Math.round(plain('dist_audio'));
          const kind = [0, 0, 2, 3, 2, 3][audio] || 0;
          const drive = Math.min(1, Math.max(0, (plain('dist_input', 5) - 3) / 6));
          const base = kind === 0 ? 0.06 : kind === 2 ? 3 : 18;
          const thd = base * (0.25 + 0.75 * drive) * (0.85 + 0.15 * Math.sin(t * 3.1));
          return [thd, thd >= 3 ? 1 : 0, 0, drive];
        }
        // the 610 half: the PRE needle and how hard the input stage is driven
        const drive = Math.min(1, Math.max(0, (plain('pre_gain', 2) * 5 - 10 + plain('pre_level', 7) * 2) / 28));
        return [0.05 + 2.4 * drive ** 2, 0, preVuDb(), drive];
      };
    })(),
    transfer: (() => {
      let last = '';
      return () => {
        const key = modelKey();
        const ratio = Math.round(plain('dist_ratio', 4));
        const stamp = `${key}:${key === 'vca' ? ratio : ''}`;
        if (stamp === last) return null; // sticky: publish once per model (and per Distressor ratio)
        last = stamp;
        let knee;
        let slope;
        let width;
        if (key === 'vca') {
          const r = DIST_RATIOS[ratio] || DIST_RATIOS[4];
          [knee, slope, width] = [r.threshold, r.ratio, r.width / 3];
        } else if (key === 'opto') {
          [knee, slope, width] = [-30, 3, 8];
        } else if (key === 'la3a') {
          [knee, slope, width] = [-28, 4.5, 5];
        } else {
          [knee, slope, width] = [-26, 4, 6];
        }
        const out = new Float32Array(128);
        for (let i = 0; i < 128; i++) {
          const x = -60 + (60 * i) / 127;
          const over = Math.max(0, x - knee);
          out[i] = x - over * (1 - 1 / slope) * (1 - Math.exp(-over / Math.max(0.5, width)));
        }
        return out;
      };
    })(),
  },
  timeoutMs: 1200,
};

/** The 610 section's PRE needle in dB: the gain switch and the level knob against 0 VU. */
function preVuDb() {
  const gain = plain('pre_gain', 2) * 5 - 10; // −10 … +10 dB
  const level = plain('pre_level', 7);
  return gain + (level - 5) * 2.2 - 2;
}

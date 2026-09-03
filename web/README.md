# Noob CompressorLab · the page

The front panels of [Noob CompressorLab](../README.md), a Vue 3 + Tailwind
single-page app rendered inside the plug-in's native web view (or a browser
tab), talking to the Rust DSP over
[noob-vst-webgui-framework](https://github.com/Noob-Audio-Engineering/noob-vst-webgui-framework). One instance is one compressor at a
time: the `model` parameter picks the 1176 or the LA-2A, the page shows
that model's face, extras and workbench, and because the choice is a
parameter it is saved with the host's project and can differ per instance.

Everything you see is this plug-in's own look, twice over: the 1176's
black (or silver, or blue-striped) panel with its machined knobs, push
buttons and cream VU face; the LA-2A's brushed plate with its bakelite
knobs, bat-handle levers, rotary selector and bevelled meter. The framework
supplies behaviour only: parameter handles, knob gestures in rotation space
(`useKnobGesture` with the `rotation` option, so a printed taper stays under
the pointer), the needle's ballistics and scale maths, the history and
transfer charts, presets in the plug-in-persisted store, undo / redo / A-B,
window resizing and fullscreen intent.

## Dev workflow

```sh
npm install
npm run build                                  # writes dist/, which the standalone serves and the plug-in embeds
NOOB_VST_WEBGUI_FRAMEWORK_PORT=4244 npm run dev         # hot reload on 5175; proxies /ws and /instance* to the standalone
```

Vite serves `src/` on port 5175 and proxies the WebSocket and the discovery
endpoints to the standalone (`vite.config.js`). Build `dist/` before
building the plug-in with `--features plugin`.

### Design mode

`src/dev/manifest.js` describes what the plug-in publishes (parameter ids,
ranges, labels, defaults, the three streams) and generates synthetic frames
that follow the model switch: a drum loop with fast FET grabs under the
1176, vocal-like syllables with a slow optical release and a lit T4 cell
under the LA-2A, and a transfer curve republished whenever the model
changes. `main.js` hands it to the client with `configureClient({ offline })`
in development builds only; when no real server answers within about a
second the page renders against it, edits stay local, and the moment a
standalone or plug-in connects the client hands over. Keep the manifest in
step with `param_specs` and `streams` in `src/dsp/mod.rs`.

## Component tree

```
App.vue                         root: the wait screen, then LabPage; Ctrl+Z / Ctrl+Shift+Z / Ctrl+Y / Ctrl+B
└── components/LabPage.vue      the shell: top bar, the active model's view (re-mounted on switch), the framework's ResizeGrip (`.lab-grip`)
    ├── components/TopBar.vue   model switch (framework Segmented on `model`), presets of the active model, undo / redo / A-B, BYPASS, fullscreen, edit→echo and latency
    ├── components/HistoryPanel.vue   "Last 8 seconds": framework Timeline over `meter` (in, out, gain reduction), identical under both models
    ├── components/TransferPanel.vue  the transfer curve: framework LinePlot over the sticky `transfer` stream with the live operating point, identical under both models
    ├── models/fet/FetView.vue        the 1176: faceplate, extras bar, and the scope drawer (the two shared panels)
    │   ├── Faceplate.vue             the 5.2 : 1 panel between rack ears in three looks by revision; everything at fractions of the plate, sizes in cqw
    │   │   ├── Knob1176.vue          SVG knob with printed marks along a taper (useKnobGesture, rotation option)
    │   │   ├── RatioButtons.vue      the RATIO column, 20 / 12 / 8 / 4 (Shift-click pushes all in)
    │   │   ├── MeterButtons.vue      the METER column, GR / +8 / +4 / OFF
    │   │   ├── VuMeter1176.vue       the cream VU face and needle (useNeedle on meter[5])
    │   │   └── PowerSwitch.vue       the power toggle (the inverse of `bypass`)
    │   └── ExtrasBar.vue             REVISION (A to H, LN), STEREO, MIX, SC HPF, the demo source (standalone only), SCOPE
    └── models/opto/OptoView.vue      the LA-2A: faceplate, the workbench (T4 panel and the two shared panels), extras strip
        ├── Faceplate.vue             the 19 : 5.25 panel: rack ears, screws, logotype and captions placed by fractions measured from a photograph
        │   ├── BigKnob.vue           Gain and Peak Reduction: printed 0..100 scale, black body, white pointer (useKnobGesture)
        │   ├── VuFace.vue            the bevelled VU meter, face and needle (useNeedle on meter[5]); legend follows the meter mode
        │   ├── SelectorKnob.vue      the meter selector, three positions (useKnobGesture, click steps)
        │   └── ToggleLever.vue       bat-handle toggles for Limit / Compress and Power
        ├── T4Panel.vue               light, free and trapped carriers from the `cell` stream
        └── ExtrasStrip.vue           emphasis, cell, link, mix, side-chain HPF, the demo source
```

Composables and data:

| file | contents |
|---|---|
| `composables/useLab.js` | the facade over `@noob-audio-engineering/noob-vst-webgui-framework/vue`: `MODELS`, `useLab()` (the model switch and the shared handles: link, mix, side-chain HPF, bypass, demo source), `useWindow()` (the page's one `useWindowSize`), the per-model preset helpers (`presetSkip`, `stateToJson`, `loadState`) and the `ui` state |
| `models/fet/useFet.js` | the 1176's handles (`useControls()`, ids `fet_*`), the revisions and their looks, the dial tapers |
| `models/opto/useOpto.js` | the LA-2A's handles (`useOpto()`, ids `opto_*`) |
| `presets.js` | factory presets per model and the `presets.user.<model>` store helpers |
| `dev/manifest.js` | the design-mode manifest |

## The model switch

The top bar's 1176 / LA-2A keys are the framework's `Segmented` bound to
the `model` parameter (non-automatable, saved in the plug-in state).
`LabPage.vue` mounts the view for the active model and re-mounts it on a
switch; both models' parameters exist all the time, so each keeps its
settings while the other is showing, and a preset of one model never
touches the other (`presetSkip` in `useLab.js` leaves the model switch, the
other model's ids, the meter selector, bypass and the demo source alone).
The Rust side runs only the active engine and republishes the transfer
curve on every switch.

## What binds to what

| control | parameter | notes |
|---|---|---|
| 1176 / LA-2A keys | `model` | framework `Segmented`, styled as `.labbar__model` |
| BYPASS (top bar), POWER (both faces) | `bypass` | the levers are inverted |
| INPUT, OUTPUT | `fet_input`, `fet_output` | marks 0..48 on the original's taper |
| ATTACK, RELEASE | `fet_attack`, `fet_release` | ATTACK has the OFF detent before 1 |
| ratio buttons | `fet_ratio` | 0..3 = 4 / 8 / 12 / 20, 4 = all in |
| meter buttons | `fet_meter` | GR / +8 / +4 / OFF |
| REVISION | `fet_revision` | `REVISIONS` in `useFet.js` gives each index its look and hint |
| GAIN, PEAK REDUCTION | `opto_gain`, `opto_peak_reduction` | |
| LIMIT / COMPRESS | `opto_mode` | |
| meter selector | `opto_meter` | Gain Reduction / Output +10 / Output +4 |
| EMPHASIS, CELL | `opto_emphasis`, `opto_cell` | extras strip |
| STEREO / LINK, MIX, SC HPF | `link`, `mix`, `sc_hpf` | shared by both models |
| DEMO SOURCE | `src_kind`, `src_level`, `src_freq` | standalone only (`hasParam`) |
| both needles | stream `meter[5]` | what the active model's meter reads for its mode, in dB |
| LAST 8 SECONDS | stream `meter[0, 2, 4]` | in and out peaks (dBFS), gain reduction (dB, at most 0) |
| TRANSFER | stream `transfer`, marker from `meter[0, 2]` | sticky curve, republished on change |
| INSIDE THE T4 | stream `cell` | light, free and trapped carriers (LA-2A only) |

### The shared panels

`components/HistoryPanel.vue` and `components/TransferPanel.vue` are
identical under both faces: the same card (`.lab-panel` in `style.css`,
the LA-2A's workbench look, now the lab's), the same typography, grid and
series colours (dim input, blue output, amber gain reduction hanging from
the top of a −24..0 dB scale with a line every 6 dB; the amber transfer
curve over −60..0 dBFS in against −60..+12 out, the dashed unity line and
the live operating point). The framework's chart variables are fixed on
the panel itself, so neither model's root can tint them, and the row the
panels sit in (`.lab-bench`, 12 px gaps and padding; the LA-2A adds the
T4 panel as a first column) is shared too. Nothing about these panels
differs per model; the faceplates and the extras strips keep their own
looks.

## Styling

`src/style.css` holds the Tailwind v4 setup (`@import`, the two model
files, `@source` pointing at the framework's Vue directory, every `@theme`
token) and the shell: the frame, the top bar, the model switch, the grip, the shared
workbench row and panel chrome. The two looks live side by side:

* `models/fet/fet.css` is the 1176's styling as it was (faceplate finishes
  by revision, knobs, push buttons, meter, power lever, extras strip), with
  its amber renamed `--color-fet-amber`;
* `models/opto/opto.css` is the LA-2A's workbench styling (`.bench-panel`,
  the framework's `Segmented` and `Toggle` under `.bench`), with its amber
  renamed `--color-opto-amber`; the faceplate, knobs, levers, selector and
  meter keep their scoped styles.

That rename was the only token collision; the rest (`panel-*`, `silver`,
`cream` on one side, `plate-*`, `bench-*`, `lamp` on the other) never
overlapped. Each model's root (`.lab-model--fet`, `.lab-model--opto`)
paints its own background; the shell's accent (`--lab-accent`) follows the
model too. The shared panels sit outside that: `.lab-panel` fixes the
`--noob-vst-webgui-framework-*` chart variables and every colour of the two panels
to one lab-wide set. The one rule to keep in
mind: `.abs` (the 1176's centring helper) must stay in `fet.css`, before
the meter and nameplate rules that override its transform.

### The 1176's looks

`models/fet/Faceplate.vue` puts one class on the panel from the selected
revision (`lookOf(index)`), and the "finishes by revision" block at the end
of `fet.css` draws it:

| look | revisions | what it draws |
|---|---|---|
| `bluestripe` | A, B | brushed silver plate, a blue block behind and around the meter with the lettering in white, black knobs with black caps and dark skirt scales |
| `blackface` | C, D, E, F, G, LN | black anodised plate, light lettering, silver-capped knobs with light skirt scales, the badge above the meter, the model lettering under it |
| `silverface` | H | silver plate with the recessed left section and "PEAK LIMITER", silver caps, the blue badge at the right |

## Window size and fullscreen

Both views scale with the window in both dimensions, from 900 × 520 up
(`WINDOW_MIN` in `useLab.js`, the same limits `src/plugin.rs` gives the
editor). The 1176's panel follows the width up to a cap from the height
(`max-width: calc(5.2 * 50vh)`), its extras strip wraps in a narrow window
and the drawer takes the rest; the LA-2A's faceplate keeps its 19 : 5.25
aspect and is capped from the height by `OptoView.vue` (`CHROME_PX`), its
workbench takes what remains. The top bar hides the subtitle and the
edit-echo read-out in narrow windows so it never wraps. Nothing scrolls.

Two framework pieces drive the host window, both through the one
`useWindowSize` instance that `useWindow()` creates:

* **Resize grip**: the framework's unstyled `ResizeGrip` sits fixed in the
  bottom-right corner (`.lab-grip`, three diagonal ridges that take the
  model's accent on hover), `min` 900 × 520 and no aspect lock. Dragging it
  sends coalesced `resize` messages; the adapter resizes the host window
  and web view, remembers the size under the `window` store key and reopens
  at it. In a browser tab (the standalone) the grip renders nothing.
* **Fullscreen**: the ⛶ button in the top bar calls `toggleFullscreen()`
  and lights up while fullscreen. In a host the adapter sizes the editor to
  the monitor's work area and restores the previous size afterwards; in a
  tab the browser's Fullscreen API does the same for the tab.

## Adding a control

1. Declare the parameter in `src/dsp/mod.rs` (`param_specs`) and in
   `src/plugin.rs`, prefixed `fet_` or `opto_` if one model owns it.
2. Mirror it in `src/dev/manifest.js`.
3. Add a handle in `useControls()` (1176) or `useOpto()` (LA-2A), or in
   `useLab()` if both models share it, and bind a control on that model's
   face or extras strip; style it in that model's CSS file.
4. If it is part of a sound, leave it out of `presetSkip`; if it is a view
   setting (like the meter selectors), add it there.

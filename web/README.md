# Noob CompressorLab · the page

The front panels of [Noob CompressorLab](../README.md), a Vue 3 + Tailwind
single-page app rendered inside the plug-in's native web view (or a browser
tab), talking to the Rust DSP over
[noob-vst-webgui-framework](https://github.com/Noob-Audio-Engineering/noob-vst-webgui-framework). One instance is one compressor at a
time: the `model` parameter picks one of five, the page shows that model's
face, extras and workbench, and because the choice is a parameter it is
saved with the host's project and can differ per instance.

Everything you see is this plug-in's own look, five times over: the 1176's
black (or silver, or blue-striped) panel with its machined knobs, push
buttons and cream VU face; the LA-2A's brushed plate with its bakelite
knobs, bat-handle levers, rotary selector and bevelled meter; the LA-3A's
flat black half-rack panel with two cream knobs and no chrome at all; the
Distressor's charcoal single unit with its ivory knobs and rows of coloured
lamps; and the 6176's brushed aluminium carrying two black inset panels,
the tube preamp on the left and the limiter on the right. The framework
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
ranges, labels, defaults, the four streams) and generates synthetic frames
that follow the model switch: a drum loop with fast FET grabs under the
1176 and the 6176, vocal-like syllables with an optical release and a lit
T4 cell under the LA-2A and the LA-3A, a faster and deeper grab whose depth
follows the ratio under the Distressor, the lamps under the two models that
have them, and a transfer curve republished whenever the model (or the
Distressor's ratio) changes. `main.js` hands it to the client with
`configureClient({ offline })` in development builds only; when no real
server answers within about a second the page renders against it, edits
stay local, and the moment a standalone or plug-in connects the client
hands over. Keep the manifest in step with `param_specs` and `streams` in
`src/dsp/mod.rs`.

One detail worth keeping: every labelled parameter here states `min: 0` and
`max: steps − 1` (the `stepped()` helper fills them in). The framework's
offline mock works the default's normalized position out from `min` and
`max`, so a labelled parameter left without them is read as 0 to 1 and any
default past the first step lands on the last one — a stepped switch would
open at its final position instead of its default. The real manifest states
the range anyway, so this only makes design mode match it.

## Component tree

```
App.vue                         root: the wait screen, then LabPage; Ctrl+Z / Ctrl+Shift+Z / Ctrl+Y / Ctrl+B
└── components/LabPage.vue      the shell: top bar, the active model's view (re-mounted on switch), the framework's ResizeGrip (`.lab-grip`)
    ├── components/TopBar.vue   model switch (framework Segmented on `model`), presets of the active model, undo / redo / A-B, BYPASS, fullscreen, edit→echo and latency
    ├── components/HistoryPanel.vue   "Last 8 seconds": framework Timeline over `meter` (in, out, gain reduction), identical under every model
    ├── components/TransferPanel.vue  the transfer curve: framework LinePlot over the sticky `transfer` stream with the live operating point, identical under every model
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
    │   ├── T4Panel.vue               light, free and trapped carriers from the `cell` stream
    │   └── ExtrasStrip.vue           emphasis, cell, link, mix, side-chain HPF, the demo source
    ├── models/la3a/La3aView.vue      the LA-3A: the half-rack faceplate, the rear-panel strip, the drawer
    │   ├── Faceplate.vue             the SR-3A rack: a 19 : 3.5 panel with the black half-rack unit and a blank plate
    │   ├── La3aKnob.vue              Gain and Peak Reduction: cream body, the 0..10 scale printed on the panel
    │   ├── La3aToggle.vue            the two bat toggles, legends left and right (GR / OUTPUT, POWER / ON)
    │   └── ExtrasStrip.vue           the rear panel (Comp / Limit, HF Contour, meter Off) and the lab's additions
    ├── models/vca/VcaView.vue        the Distressor: the one-unit faceplate, the extras bar, the drawer
    │   ├── Faceplate.vue             the 19 : 1.75 panel in either finish; its lamps are controls as well as indicators
    │   ├── KnobEL8.vue               the four ivory knobs, 0..10 printed on the panel around them
    │   ├── GrBargraph.vue            the sixteen-lamp gain-reduction bargraph, 1 to 26 dB
    │   ├── MiniToggle.vue            the two toggles the EL8-X adds between the knobs
    │   └── ExtrasBar.vue             finish, link mode, headroom, mix, side-chain HPF, the demo source, SCOPE
    └── models/pre6176/Pre6176View.vue  the 6176: the two-unit faceplate, the extras bar, the drawer
        ├── Faceplate.vue             the 19 : 3.5 aluminium panel with two black insets; reuses the 1176's VU
        ├── PreKnob.vue               the glossy black knobs, continuous and stepped alike
        ├── RatioKnob.vue             the RATIO rotary, the one control that drives two parameters
        └── PreToggle.vue             the small bat toggles, two or three positions
```

Composables and data:

| file | contents |
|---|---|
| `composables/useLab.js` | the facade over `@noob-audio-engineering/noob-vst-webgui-framework/vue`: `MODELS`, `useLab()` (the model switch and the shared handles: link, mix, side-chain HPF, bypass, demo source), `useWindow()` (the page's one `useWindowSize`), the per-model preset helpers (`presetSkip`, `stateToJson`, `loadState`) and the `ui` state |
| `models/fet/useFet.js` | the 1176's handles (`useControls()`, ids `fet_*`), the revisions and their looks, the dial tapers |
| `models/opto/useOpto.js` | the LA-2A's handles (`useOpto()`, ids `opto_*`) |
| `models/la3a/useLa3a.js` | the LA-3A's handles (`useControls()`, ids `la3a_*`) and its drawer state |
| `models/vca/useVca.js` | the Distressor's handles (ids `dist_*`), the ratio / detector / audio tables, the bargraph steps, the knob taper |
| `models/pre6176/usePre.js` | the 6176's handles (`pre_*` plus the `fet_*` half it drives) and the mappings between the 6176's printed numbers and the 1176's |
| `presets.js` | factory presets per model and the `presets.user.<model>` store helpers |
| `dev/manifest.js` | the design-mode manifest |

## The model switch

The top bar's five keys are the framework's `Segmented` bound to the
`model` parameter (non-automatable, saved in the plug-in state).
`LabPage.vue` mounts the view for the active model and re-mounts it on a
switch; every model's parameters exist all the time, so each keeps its
settings while another is showing, and a preset of one model never touches
another (`presetSkip` in `useLab.js` leaves the model switch, the other
models' ids, the meter selectors, bypass and the demo source alone). The
Rust side runs only the active engine and republishes the transfer curve on
every switch.

`MODELS` in `useLab.js` is the registry: each entry carries the view `key`,
the label on the switch, the parameter-id prefixes the model `owns` and its
meter selector. Ownership is a list rather than one prefix because the 6176
owns two, its own `pre_*` section and the `fet_*` limiter it drives. That is
what lets a 6176 preset set both halves while a 1176 preset still leaves
`pre_*` alone.

### The 6176's scales over the 1176's controls

The 6176 prints different numbers on the same circuit, so the right half of
its face is a page-side mapping onto the lab's existing `fet_*` parameters
and the DSP is untouched. Input and Output read 0 to 10 with unity near 5
where a standalone 1176 prints attenuation marks 0 to 48; the table in
`usePre.js` pairs them, so 5 lands on the 1176's "24". Attack and Release
read 1 to 9 where the 1176 reads 1 to 7. The RATIO rotary carries two
positions a standalone 1176 has not got, BP and 1, which write the routing
parameter `pre_join` instead of `fet_ratio`; from 4 on it is the ratio
switch again.

## What binds to what

| control | parameter | notes |
|---|---|---|
| the five model keys | `model` | framework `Segmented`, styled as `.labbar__model` |
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
| GAIN, PEAK REDUCTION (LA-3A) | `la3a_gain`, `la3a_peak_reduction` | printed 0..10, arbitrary on the hardware too |
| CELL (LA-3A) | `la3a_cell` | Fresh / Used / Tired; the extras strip, as the LA-2A's is, because the age of a T4 is not a switch the hardware gives you |
| R37, METER ZERO (LA-2A) | `opto_emphasis`, `opto_meter_zero` | both are screwdriver trims on the real front panel, so both are live on the faceplate rather than on the strip |
| +48v (6176) | `pre_phantom` | on the panel because the hardware has it. It cannot be audible — phantom power feeds a microphone and the model starts at the preamp input — but there is no working version of it hidden elsewhere, so it latches as panel state and is saved with the rest |
| JOIN / SPLIT (6176) | `pre_join` | the same routing parameter the RATIO knob's BP position reaches, so switch and knob agree instead of each having a path |
| GR / OUTPUT toggle | `la3a_meter` | the hardware's two positions; the third, Off, is on the extras strip |
| MODE, HF CONTOUR | `la3a_mode`, `la3a_emphasis` | rear-panel controls, so they live on the strip; the contour is 0 = flat, the opposite sense to the LA-2A's emphasis |
| INPUT, ATTACK, RELEASE, OUTPUT (Distressor) | `dist_input`, `dist_attack`, `dist_release`, `dist_output` | 0 to 10.5, as the knobs turn |
| the eight ratio lamps | `dist_ratio` | clicking a lamp selects it; the RATIO button cycles |
| DETECTOR lamps and button | `dist_detector` | HP and Band are bits of the four-state selector; the Link lamp is the shared `link` |
| AUDIO lamps and button | `dist_audio` | HP is a bit, Dist 2 and Dist 3 are exclusive |
| British Mode toggle | `dist_british` | the EL8-X toggle between Attack and Release |
| LINK MODE, HEADROOM | `dist_link_mode`, `dist_headroom` | extras bar |
| FINISH | stored `finish` | black or the anniversary red; a page setting, not a parameter, kept in the UI store so a project remembers it |
| GAIN, LEVEL, the shelves, PAD, polarity, input select (6176) | `pre_gain`, `pre_level`, `pre_lf_*`, `pre_hf_*`, `pre_pad`, `pre_polarity`, `pre_input` | the 610 half |
| RATIO rotary (6176) | `pre_join` + `fet_ratio` | a knob marked BP, 1, 4, 8, 12, 20, ALL, as the hardware has it; BP and 1 write the routing, 4 to ALL the ratio (`RatioKnob.vue`) |
| METER (6176) | `pre_meter` | PRE / GR / COMP |
| LO CUT, voicing, input loading | `pre_hpf`, `pre_voice`, `pre_load` | the centre strip and the extras bar |
| STEREO / LINK, MIX, SC HPF | `link`, `mix`, `sc_hpf` | shared by both models |
| DEMO SOURCE | `src_kind`, `src_level`, `src_freq` | standalone only (`hasParam`) |
| every needle | stream `meter[5]` | **where the needle already is**, in dB against the meter's zero. The VU movement runs in the audio thread for all five models (13 rad/s, damping 0.80: 99 % in 300 ms, about 1.5 % overshoot), so a face draws this field rather than smoothing it again; the framework needle is asked for nothing but a short critically-damped follow to bridge the gap between frames |
| LAST 8 SECONDS | stream `meter[0, 2, 4]` | in and out peaks (dBFS), gain reduction (dB, at most 0) |
| TRANSFER | stream `transfer`, marker from `meter[0, 2]` | sticky curve, republished on change |
| INSIDE THE T4 | stream `cell` | light, free and trapped carriers (the two optical models) |
| REDLINE, 1% THD, the PRE needle | stream `lamps` | `thd_pct, redline, pre_vu_db, drive`; published while the Distressor or the 6176 is active |

### The shared panels

`components/HistoryPanel.vue` and `components/TransferPanel.vue` are
identical under every face: the same card (`.lab-panel` in `style.css`,
the LA-2A's workbench look, now the lab's), the same typography, grid and
series colours (dim input, blue output, amber gain reduction hanging from
the top of a −24..0 dB scale with a line every 6 dB; the amber transfer
curve over −60..0 dBFS in against −60..+12 out, the dashed unity line and
the live operating point). The framework's chart variables are fixed on
the panel itself, so no model's root can tint them, and the row the panels
sit in (`.lab-bench`, 12 px gaps and padding; the LA-2A adds the T4 panel
as a first column) is shared too. Nothing about these panels differs per
model; the faceplates and the extras strips keep their own looks.

## Styling

`src/style.css` holds the Tailwind v4 setup (`@import`, the five model
files, `@source` pointing at the framework's Vue directory, every `@theme`
token) and the shell: the frame, the top bar, the model switch, the grip, the shared
workbench row and panel chrome. The two looks live side by side:

* `models/fet/fet.css` is the 1176's styling as it was (faceplate finishes
  by revision, knobs, push buttons, meter, power lever, extras strip), with
  its amber renamed `--color-fet-amber`;
* `models/opto/opto.css` is the LA-2A's workbench styling (`.bench-panel`,
  the framework's `Segmented` and `Toggle` under `.bench`), with its amber
  renamed `--color-opto-amber`; the faceplate, knobs, levers, selector and
  meter keep their scoped styles;
* `models/la3a/la3a.css` is the LA-3A's flat black panel, its cream knobs
  and its two bat toggles, plus the rear-panel strip;
* `models/vca/vca.css` is the Distressor's charcoal panel, ivory knobs, the
  lamp code (`.el8lamp` green / amber / red / blue) and the bargraph;
* `models/pre6176/pre6176.css` is the 6176's brushed aluminium, its two
  black insets, the glossy knobs, the bat toggles and the jewel lamp.

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

Every view scales with the window in both dimensions, from 900 × 520 up
(`WINDOW_MIN` in `useLab.js`, the same limits `src/plugin.rs` gives the
editor). Each faceplate keeps the aspect of the hardware it draws and is
capped from the window height by its view (`CHROME_PX`), so a wide, shallow
window never pushes the drawer off the bottom; the drawer takes whatever
height is left and the charts grow with it.

| model | panel | aspect |
|---|---|---|
| 1176 | two units, full rack | 5.2 : 1 |
| LA-2A | three units | 19 : 5.25 |
| LA-3A | two units, in the SR-3A rack kit | 19 : 3.5 |
| Distressor | one unit, full rack | 19 : 1.75 |
| 6176 | two units, full rack | 19 : 3.5 |

The LA-3A is the odd one: the unit itself is only half a rack wide, so its
face draws the SR-3A mounting kit the hardware is sold with — a full
19 inch panel carrying the unit on one side and a blank plate beside it.
That is why it has ears and rails like the others and yet only half of it
carries controls.

The top bar sheds its content in stages as the window narrows so that it
never wraps and never overflows: the subtitle first, then the read-outs,
then the transport, then the product name, leaving BYPASS and fullscreen
standing longest. The workbench row keeps a minimum height and, below about
640 px of window, shows the history panel alone at full width rather than
two charts too short to read. Nothing scrolls.

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

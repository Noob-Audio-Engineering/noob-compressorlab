# Noob CompressorLab build contract: the invariant half

Every model in this plug-in is built to this contract. What changes from model to model is the
dossier it reads and the three or four findings that define that unit; everything here is the same
every time.

It exists because these instructions kept failing to arrive. Each one below is a rule that was
learned by getting it wrong, and several had to be restated after a build had already gone the other
way. When a build is briefed, this file is cited rather than paraphrased, because paraphrasing it is
how the detail that mattered got dropped the first time.

Repo: `C:\Users\elyci\RustroverProjects\noob-compressorlab`, a free plug-in by Noob Audio Engineering
on the noob-vst-webgui-framework. One instance is one compressor, chosen by a per-instance `model`
parameter, and every knob of every model is a parameter of the instance.

## Read first, in this order

1. `research/<MODEL>.md`, the dossier. Its recommended-DSP-design section and its test-plan section
   are the authority for everything the contract marks "from the dossier". Do not substitute your own
   judgement for a figure the dossier sourced.
2. `src/dsp/mod.rs`, `src/plugin.rs`, `web/README.md`.
3. The closest existing model of each shape, engine and face, named in the model-specific half.

## Identity and the model index

- Parameter prefix `<prefix>_`. Engine at `src/dsp/<engine>/`, face at `web/src/models/<face>/`.
- **Append at the end of `MODEL_NAMES` and `Model::ALL`, and never insert before an existing
  variant.** `Model` takes its value from declaration order, so appending is self-consistent whatever
  order the parallel builds land in. Renumbering an existing entry breaks every saved project, which
  is the one unrecoverable mistake available here.
- It is an affectionate spoof, not a parity replacement. The panel carries NOOB names, never the real
  trademark. Follow the naming the shipped models use.

## Shared files, when builds run in parallel

`src/dsp/mod.rs`, `src/plugin.rs`, `src/bin/benchmark.rs`, `web/src/composables/useLab.js`,
`web/src/dev/manifest.js`, the presets file, `README.md` and `docs/BENCHMARK.md` are touched by every
build. **Re-read each one immediately before each edit and make a small surgical insertion.** No
wholesale rewrite, no reformatting of neighbouring entries. An entry that was not there when you last
looked is a parallel builder working, not a corrupted tree: leave it and append after it. Do the
shared-file wiring last, after the engine and faceplate are written and tested in their own new
directories, so the window where two builds are in one file stays short. A genuine conflict gets
messaged to the coordinator, never resolved by overwriting.

## Parameters

Take every id, range, taper and default from the dossier's parameter table rather than inventing one.
Where a control duplicates a shared lab parameter (`link`, `mix`, `sc_hpf`, `bypass`, `src_*`), use
the shared one rather than adding a second, and say so to the coordinator.

**Publish a display form wherever a control's printed marking is not simply its value.** Four models
needed that fixed retrospectively.

**Model the real figures and print the panel's.** Where a manufacturer's calibration table shows the
panel labels are approximations, that gap is a finding to implement and state, not to round away.

Use `Taper::Table` / `with_table` to publish a pot law, so the law lives once in the engine rather
than being re-guessed on the page.

## Anything the hardware has, anything we add

Every control the hardware has is live and in its real place. Anything we add that the hardware lacks
goes on the extras strip, where we say plainly it is ours. That rule caught an invented power switch
on the 1176 and a dead ornament on the LA-2A.

## Rust

- `src/dsp/<engine>/` in the usual shape: `Settings`, the engine, `reset`, `configure`, `process`,
  `transfer`, and a `tests.rs`.
- Extend `Model`, `Settings`, `param_specs()`, `read_settings`, `Processor`, `streams()`, `latency()`
  and `src/plugin.rs`. The 20 ms crossfade on model change stays.
- VU ballistics run in the audio thread in `src/dsp/vu.rs` for every model. Publish `meter_vu` as the
  **needle's position**, not the level it chases, and do not smooth it again on the page.
- Reuse the `meter`, `cell`, `transfer` and `lamps` streams. Publish `cell` only if the model has an
  element that justifies it, zeros otherwise.
- Oversample where the dossier asks. If a stage sits deep in its knee, prefer antiderivative
  antialiasing over more oversampling: on the 610 that bought 24 dB where doubling the factor bought 2.
- Add rows to `src/bin/benchmark.rs` so the model appears in `docs/BENCHMARK.md`.
- `README.md` gains a section: what it is, the control table, its numbers with their sources, and its
  misses if any.

## Components

**Do not extract a component crate to `noob-electrical-components`, ever, on your own authority.**
Build the part in one small separable place inside your engine and tell the coordinator what it is.
Extraction is the coordinator's decision and needs evidence from more than one build.

The rule that repository now uses: a part is admitted when **two units are documented to contain it**,
on drawings, not when one does and another is expected to. That clause is stricter than it was, and it
is stricter because the looser version has already been wrong once. The diode bridge was admitted on
the expectation that the EMI TG12413 would be its second user; the TG12413 turned out not to contain a
bridge at all, and the crate sits there with one user.

Two failures to keep in view, because they are the two ways this goes wrong. Three "tube stages" were
three different circuits sharing a word, and were rightly refused. A component called "a VCA" would
have been the fourth, and was renamed to the Blackmer gain cell, which is a part rather than a
category, and admitted with two documented users. The test is always whether the units share an
**equation**, never whether they share a **word**.

## Web

- `web/src/models/<face>/` with its own view, faceplate and controls, measured from the reference
  images the dossier's panel section names, not eyeballed from prose.
- The shared history and transfer panels stay **identical** across every model. That is a standing
  instruction from the user, and structurally-shared-but-different has already been rejected once.
- The bench bar keeps its arrangement: it sits under the topmost bar, model-specific controls on the
  left, the shared globals on the right in the fixed order, with the marker saying which are ours.
- Per-model presets and `presetSkip` as for the others; `web/src/dev/manifest.js` gains the
  parameters with frame generators; the browse view gains the model in its family, one per row.

## Testing standard, which is not negotiable

**Every test that exists to check a real figure asserts that figure, with its source named in a
comment.** If the model cannot meet one, fix the model, or record the miss at the test and in the
README's table. Never widen an assertion until it passes, and never assert a value the model itself
produced. An audit found nine such tests across five plug-ins, one of which compared an estimate with
itself; fixing four of them immediately exposed five real faults. Measure with a probe built outside
the repo where that is the only honest way.

A figure from another vendor's emulation is not a target for us. Benchmark against the hardware.

## Verification

`cargo test`, `cargo clippy --all-targets`, `cargo clippy --features plugin` in dev and `--release`,
`cargo doc --no-deps`, `cargo fmt --check`, the SPA build, `scratchpad/ui-audit.mjs` across every
model at 900x520, 1100x620 and 1900x1000, and the benchmark regenerated.

**Build the plug-in library last.** Building a binary without `--features plugin` recompiles the
library without it and leaves a stripped artefact of the same name with no `GetPluginFactory`. That
has installed a broken bundle three times. Verify the installed size and exports afterwards.

## Rules

Names only as `noob-vst-webgui-framework` and `Noob Audio Engineering`, never a bare fragment; check
with a case-insensitive grep for the old names. **Nothing published to crates.io or npm.** First
person prose, never "the author", and no bylines in documentation. The framework holds generics only,
headless and uncoloured: every look, face and colour stays in the plug-in. Bash commands under 8 KB;
perl substitutions never with `|` as the delimiter when the pattern contains a pipe. Screenshots into
the session scratchpad. Do not commit; the coordinator integrates.

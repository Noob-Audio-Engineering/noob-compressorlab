# What `noob-compressorlab` should model next: a survey of the classic compressors and limiters

A reasoned build order, not a list of famous boxes. Fifteen candidates assessed. Every document named
below was fetched during this survey unless the text says otherwise, and where a document is
unreachable I say so rather than cite it from memory.

---

## 1. Where the holes are

The lab has six models in four families. Sorting them by gain element rather than by badge makes the
gaps obvious:

| family | gain element | what we have | engine |
|---|---|---|---|
| FET | JFET as a voltage-controlled resistor, shunt, feedback | 1176 (nine revisions), and the 1176 half of the 6176 | `dsp::fet` |
| optical | photoresistor driven by an electroluminescent panel or LED | LA-2A, LA-3A, CL 1B | `dsp::opto`, `dsp::opto3`, `dsp::opto1b` |
| VCA | Blackmer-type log-antilog multiplier, dB-domain feedback | Distressor | `dsp::vca` |
| channel strip | a tube preamp in front of one of the above | 6176 (610 into the 1176) | `dsp::pre` + `dsp::fet` |
| **variable-mu** | **remote-cutoff triode, grid bias moves the gain** | **nothing** | — |
| **diode bridge** | **forward-biased diodes as a voltage-variable attenuator** | **nothing** | — |
| **mainstream VCA** | **VCA, but feedforward and console-shaped** | **nothing** | — |

Three families are missing. The third is a softer gap than the first two: we do own a VCA, but the
Distressor is an outlier inside its own family, a dB-domain feedback design with program-dependent
ballistics and eight hand-drawn curves. It teaches the codebase nothing about the ordinary
feedforward console compressor that most engineers actually mean by "a VCA".

Two of the missing families are also the two that would most change how the lab sounds. Every gain
element we have reduces gain by *shunting* signal to ground through something whose resistance
falls: a FET channel, a photocell, or a multiplier cell. Nothing in the lab reduces gain by pushing
current through a diode junction, and nothing reduces gain by moving a tube's operating point along
a remote-cutoff curve. Those two mechanisms distort in ways ours cannot: the diode bridge's
distortion rises with gain reduction and is a property of the attenuator itself, and the variable-mu
tube's gain and distortion are the same curve read at two points.

---

## 2. How I ranked them

Four criteria, in the order that decided ties:

1. **Does it fill a family?** A seventh unit in a family we already model reuses code and teaches
   nothing. A first unit in an empty family forces a new gain element, a new detector, and usually a
   new way of thinking about the loop.
2. **Can it be modelled honestly?** This decided the order. A unit with no reachable schematic
   produces a dossier of adjectives. The six existing dossiers establish the gain element, the
   detector and the ballistics from circuit drawings and published measurements, and I will not
   recommend a unit that cannot meet that bar.
3. **Does it actually sound different from what we have?** A unit that would land within a few
   decibels of our 1176 on every test is a faceplate, not a model, and I say so where that is true.
4. **What could it be benchmarked against?** Which commercial plug-ins model it, whether any publish
   measurements, and whether the hardware itself has ever been measured in public.

A fifth consideration, effort and reuse, breaks ties inside a family.

**One finding that applies to every candidate, so I will say it once here.** No unit in this survey
has an independent published laboratory measurement of the hardware that I could reach. Audio
Science Review does not measure studio compressors. No plug-in vendor I checked publishes a null
test against hardware, and several that claim "component-level modelling" publish no numbers at all.
The nearest thing to a measurement available anywhere is a factory specification *with tolerances*,
and on that axis one candidate is far ahead of the rest.

---

## 3. The candidates

### 3.1 Neve 2254 and 33609 — diode bridge — **recommended first**

**Family and gain element.** Diode bridge, a family we do not have. Gain reduction happens in a
balanced ring of four diodes sitting between two transformers. A DC control voltage forward-biases
the diodes, which changes their dynamic resistance and so the loss through the bridge. The AMS Neve
handbook states it plainly, naming the parts:

> the ... balanced diode bridge D14 to D17. A DC Control voltage, derived b[y] the limiter or
> compressor sidechain sections, is used to forward bias the diodes in the bridge and thereby alter
> the effective [impedance] ... signal remains in a linear region of the diode characteristic.

The 2254 is the discrete original, a Rupert Neve design with Marinair transformers; the 33609 is its
two-channel descendant, still manufactured today. They share the bridge. I have not verified the
2254's introduction date from a primary source: the schematic I read is the 2254/E revision, drawn
20 September 1972 and revised to 11 July 1974, and the commonly repeated 1969 date for the original
is not on any document I could reach. The dossier should establish it or say it could not.

**Why it is worth modelling.** Two reasons beyond the empty family. First, the distortion mechanism
is unlike anything in the lab: the bridge is the attenuator *and* the nonlinearity, so harmonic
content rises with gain reduction rather than being added by a separate amplifier stage, and it is
transformer-bracketed on both sides. Second, the 33609 runs an independent limiter section and
compressor section into the same bridge simultaneously, each with its own threshold, ratio and
recovery. Nothing we own has two detectors fighting over one gain element. That is a genuinely new
shape for `Processor` to host, not a variation on a shape it already knows.

**Whether it can be modelled honestly.** This is the strongest documentation position of any
candidate in the survey, and it is not close.

- **Neve 2254/E schematic set**, `https://archive.org/details/neve-2254-schematics`. Photographs of
  the original blueprints. The main drawing is L/10,004/E, "2254/E — LIMITER AND COMPRESSOR", The
  Neve Group of Companies, dated 20-9-72 with revisions through 11 July 1974. I opened it and read
  it: it is legible at 2288 × 1712. It shows the input transformer, the CONTROL block (the bridge)
  fed from T2, separate LIMIT SIDE CHAIN and COMP SIDE CHAIN blocks, the compress ratio switch with
  its resistor ladder annotated INITIAL LAW and TOP END LAW, the limit and compress recovery
  switches, the threshold pots, the meter select, and component values throughout. It also carries a
  design note that is worth a paragraph of the dossier on its own: *"PINS J & L TO BE LINKED
  EXTERNALLY TO GIVE 100µS ATTACK. FOR LONGER ATTACK TIMES A SERIES RESISTANCE SHOULD BE INSERTED
  (470Ω FOR 1MS)."* The same item carries separate schematics for the BA185, BA191, BA192 and BA283
  amplifier cards, both switch decks, the front panel, and the alignment block.
- **AMS Neve 33609/J Limiter/Compressor Technical Handbook**, 3rd edition, 2002, document 527-149
  Issue 3, `https://archive.org/details/neve-33609-j-technical-handbook`. A factory service
  document. It gives a transistor-by-transistor description of both sidechains (TR2, TR5, TR8, TR9
  for the compressor; TR6, TR7, TR10 to TR13 for the limiter), identifies R28 as the limiter's
  attack time constant with R30 paralleled in the fast position, and includes the drawing set for
  the 11475 motherboard, the 10640 power amplifier and the switch assemblies.

**Published measurement.** The handbook's specification section is the best calibration anchor I
found anywhere in this survey, because it does not give a ratio number, it gives a *measured
transfer point per ratio setting with a tolerance*:

Compress ratio calibration, recovery 100 ms, threshold −20 dBu, 1 kHz at 0 dBu, input raised 10 dB:

| ratio switch | output level change | tolerance | implied ratio |
|---|---|---|---|
| 1.5:1 | 6.5 dB | ±1 dB | 1.54:1 |
| 2:1 | 5.0 dB | ±1 dB | 2.00:1 |
| 3:1 | 3.5 dB | ±1 dB | 2.86:1 |
| 4:1 | 2.5 dB | ±0.5 dB | 4.00:1 |
| 6:1 | 1.5 dB | ±0.5 dB | 6.67:1 |

Note that the panel labels are approximations: 3:1 is really 2.86:1 and 6:1 is really 6.67:1. That
is exactly the kind of fact the CL 1B dossier went hunting for and mostly could not find, and here
it is stated by the manufacturer. The same section gives:

| quantity | published value |
|---|---|
| attack, slow | 4 ms ±1 ms |
| attack, fast | 2 ms ±1 ms |
| limit recovery | 50, 100, 200, 800 ms, AUTO 1 1500 ms, AUTO 2 3000 ms, all ±50 % |
| compress recovery | 100, 400, 800, 1500 ms, AUTO 1 800 ms, AUTO 2 1500 ms |
| limit ratio | 10 dB input step gives 0.1 dB ±0.1 dB output change |
| frequency response | 20 Hz to 20 kHz ±0.5 dB at 0 dBu relative to 1 kHz |
| distortion, bypassed | 0.075 % max at +9 dBu, 1 kHz |
| distortion, compress 6:1, make-up max, recovery 800 ms, threshold −18 dBu | 0.2 % max |
| distortion, limit in, recovery 800 ms, +22 dBu, threshold −18 dBu | 0.45 % max |
| noise | −75 dBu bypassed; −55 dBu compress in with make-up at maximum |

The attack figures come with their own definition, which matters: "the time taken for a signal to
return to within 1 dB of its original value, using an increase in input level of 10 dB on an input
adjusted to +10 dBu at 1 kHz". A test can be written directly from that sentence.

**The decisive document.** Coriander V. Pines, *"Real-Time Virtual Analog Modelling of Diode-Based
VCAs"*, DAFx-25, Ancona, September 2025,
`https://www.dafx.de/paper-archive/2025/DAFx25_paper_44.pdf`. Eight pages, open access under CC BY
4.0. The paper's introduction names the Neve 2254/E and the Neve 33609/J as two of its four
canonical diode-VCA examples, and its references [6] and [7] are *the exact two archive.org items
above*. It states that "a careful review of prior literature did not reveal any existing digital
models of diode-based VCAs", then supplies one: a nonlinear, explicit, **stateless** model solved in
closed form with the Wright omega function, so no iteration and no Newton solver. It covers gain
control, comparison against SPICE, an analysis of harmonic distortion, and four sections of
implementation guidance (signal contribution to effective gain, bias parameter magnitude, DC
blocking, antialiasing). Section 5.2 works a full resistor-diode *bridge* in closed form.

The honest caveat: section 5.2's bridge is derived from the Dolby A301's current-controlled
topology, not from Neve's transformer-coupled bridge. The paper gives the method and the family of
solutions; deriving Neve's particular bridge from L/10,004/E and the 11475 description is still
work. But it is work with a published, peer-reviewed, real-time-capable starting point, which is
more than any other candidate offers.

**Benchmarks.** Arturia Comp DIODE-609 models the 33609 'C', "an earlier fully-discrete model", and
claims it is "faithfully emulated down to the behavior of individual components"; it exposes a
calibration control that "adjusts diode-bridge emulation, reducing headroom", which is a testable
claim. UAD, Waves, Softube and Brainworx all ship 33609 models. None publishes measurements or a
null test. Uniquely among the top candidates, **the hardware is still manufactured by AMS Neve**, so
a real unit is obtainable and comparable rather than a museum piece.

**Effort and reuse.** Moderate, and lower than it looks. The dB-domain detector scaffolding, stereo
link, sidechain high-pass, meter and transfer-curve streams all come from `dsp::vca` and the shared
extras. The bridge is new but algebraically closed-form, so it needs no iterative solver and no tube
stage. Transformers can borrow the 610's model in `dsp::pre`. A second diode unit afterwards would
reuse nearly all of it.

---

### 3.2 Fairchild 660 and 670 — variable-mu tube — **recommended second**

**Family and gain element.** Variable-mu, the other empty family. Eight General Electric 6386
remote-cutoff twin triodes per unit sit in the audio path in a push-pull class-A stage; the
sidechain drives their grids negative, walking the tubes down a remote-cutoff curve so that
amplification and distortion change together. Designed by Rein Narma; Raffensperger dates the
design to the 1950s, and the earliest dated Fairchild document he cites is the 1959 instruction
manual.

**Why it is worth modelling.** It is the most famous compressor ever built, and more usefully it is
the only one whose gain element is also its amplifier. In every model we own, the thing that reduces
gain and the thing that colours the signal are separate blocks; here they are one, which means gain
reduction and harmonic content cannot be tuned independently. That constraint is the whole point of
the box and it would force a genuinely new engine shape. The 670 also matrixes to mid-side
internally, which nothing in the lab does.

**Whether it can be modelled honestly.** Yes, and the chain of documents closes completely. All of
the following are in the John Leimseider archive on archive.org and I confirmed each downloads:

| document | identifier |
|---|---|
| Fairchild 670 stereo limiting amplifier schematic | `JL10883` |
| Fairchild 660 mono limiting amplifier schematic | `JL10866` |
| Fairchild 660 Drawn | `JL10865` |
| Fairchild 663 mono compressor schematic | `JL10873` |
| Fairchild 670 component layout | `JL10876` |
| Fairchild 670 Limiter manual | `JL10878` |
| Fairchild 670 stereo limiting amplifier instructions | `JL10882` |
| Fairchild 670 owner's manual (24.5 MB) | `Fairchild_670_owners_manual` |

The owner's manual publishes a real specification sheet:

| quantity | published value |
|---|---|
| attack | 0.2 ms in time-constant positions 1, 2 and 6; 0.4 ms in positions 3, 4 and 5 |
| release, position 1 | 0.3 s |
| release, position 2 | 0.8 s |
| release, position 3 | 2 s |
| release, position 4 | 5 s |
| release, position 5 | program-dependent: 2 s individual peaks, 10 s multiple peaks |
| release, position 6 | program-dependent: 0.3 s individual, 10 s multiple, 25 s consistently high |
| compression ratio | variable 1:1 to 20:1 above a predetermined level, factory-set to +2 dBm |
| frequency response | 40 c/s to 15 kc ±1 dB |
| distortion | under 1 % at any level to +18 dBm out, no limiting; under 1 % at 10 dB limiting and +12 dBm out |
| separation | 60 dB left-right, 0 dB vertical-lateral |
| tube complement | eight 6386, plus a 5651, two 12AX7 and two 12BH7 (the remaining entry is illegible in the scan) |

**The tube law is available as data, not description.** The General Electric 6386 datasheet,
ET-T1113, dated August 1954, is at `https://frank.pocnet.net/sheets/142/6/6386.pdf`. Six pages. I
opened it: "6386 TWIN TRIODE, Five-Star Tube, FOR REMOTE-CUTOFF CASCODE-AMPLIFIER APPLICATIONS,
REMOTE-CUTOFF CHARACTERISTIC, 9-PIN MINIATURE, MEDIUM-MU", with interelectrode capacitances, basing,
and the plate characteristic curves. Finding it took brute force; it is filed under volume 142 and
is not turned up by the obvious paths.

**And the circuit has already been modelled in the literature.** Peter Raffensperger, *"Toward a
Wave Digital Filter Model of the Fairchild 670 Limiter"*, DAFx-12, York, September 2012,
`https://www.dafx.de/paper-archive/2012/papers/dafx12_submission_9.pdf`. It establishes that the 670
is a **feedback** design (the sidechain watches the output), and it publishes a fitted remote-cutoff
triode model, which existing 12AX7-shaped triode models cannot represent:

```
Ia = p1 · Vak^p2 / [ (p3 − p4·Vgk)^p5 · (p6 + exp(p7·Vak − p8·Vgk)) ]

p1 = 3.981e−8   p2 = 2.383   p3 = 0.5    p4 = 0.1
p5 = 1.8        p6 = 0.5     p7 = −0.03922   p8 = 0.2
```

fitted by Levenberg-Marquardt to the GE datasheet curves. The paper uses 8× oversampling at 44.1 kHz
and argues why.

**The honest caveat.** Raffensperger validates the model against **SPICE simulation, not against
hardware**. He states outright that nobody had published a model of the 670 before him, and he
publishes no measurement of a real unit. So the Fairchild's documentation is deep on circuit and
thin on ground truth: we would be checking a model against a model. The two program-dependent
time-constant positions are described qualitatively in the manual and nowhere quantitatively.

**Benchmarks.** Universal Audio's Fairchild Tube Limiter Collection claims "accurate circuit models
of 'golden-reference' units from legendary Ocean Way Studios" and specifically that "the models
follow the variable-mu tube gain control and transformer behavior of the originals, not just their
broad compression curve"; it adds Wet/Dry, a sidechain filter and a Headroom control that the
hardware lacks. Waves PuigChild is the other well-known one. No vendor publishes measurements.
Raffensperger's paper is the only benchmark with numbers in it.

**Effort and reuse.** The highest of the top four. Twenty tubes, fourteen transformers, a push-pull
variable-mu stage that must be oversampled, and a feedback loop around a nonlinearity. The 610's
triode and transformer work in `dsp::pre` transfers, and the 1176's oversampler and feedback-loop
structure transfer, but the gain stage itself is new and expensive. This is why it is second and not
first: it is the better story and the harder build, and its ground truth is weaker.

---

### 3.3 SSL 4000 G bus compressor — mainstream VCA — **recommended third**

**Family and gain element.** VCA, which we technically have, but a different animal from the
Distressor. Blackmer-type multiplier (dbx 202XT / 2150 originally, THAT 2180 or 2181 in every clone
and later revision), feedforward, with a "dominant channel" stereo detector.

**Why it is worth modelling.** It does not sound like our Distressor and it is not built like it.
SSL's own module guide describes three things we model nowhere:

> The main VCA is permanently in circuit; the compressor sidechain is enabled by the IN switch.
>
> It should be noted that the knee point of the compressor, set with the THRESHOLD control,
> purposely changes depending on the setting of the RATIO control. Decreasing the RATIO setting
> lowers the effective threshold, hence maintaining the perceived 'loudness' of the compressed
> signal.
>
> The compressor features a classic 'dominant' sidechain architecture. The left and right channels
> are independently rectified using a true peak full wave detector circuit, and the dominant, ie.
> louder channel, controls the gain reduction of the overall stereo level.

A threshold that moves with the ratio control on purpose, and a dominant-channel rather than summed
or linked stereo detector, are both new behaviours for the lab. The famous "Auto" release is two
time constants in parallel, which is different from the Distressor's program dependence.

**Whether it can be modelled honestly.** Yes, but from clone documentation rather than from SSL.

- **Gyraf Audio's GSSL page**, `https://www.gyraf.dk/gy_pd/ssl/ssl.htm`. Jakob Erland worked from the
  original SSL4000E desk schematics, cards 82E26 and 82E27, and publishes the values: attack 0.1,
  0.3, 1, 3, 10, 30 ms; release 0.1, 0.3, 0.6, 1.2, 2.4 s and Auto; ratios 2:1, 4:1, 10:1; make-up
  0 to 20 dB; threshold −20 to +20 dBm. He gives the Auto release as two constants combined,
  91k + 6µ8 and 750k + µ47, and describes the full-wave rectifier built from two TL074 stages, the
  NE5534s driving the VCAs in current mode through 27k, and the parallel "tracking dummy" sidechain
  VCA. This is the single most useful document for the unit, and it is a clone builder's page rather
  than a manufacturer's, which is worth stating in the dossier.
- **SSL 500-series G Comp module user guide**,
  `https://solidstatelogic.com/assets/uploads/downloads/SSL_500_Series_G_Comp_Module_User_Guide.pdf`.
  SSL's own words on the architecture, quoted above, plus the modern module's sidechain high-pass
  frequencies (30, 60, 106, 125, 185 Hz) and the added 1.5, 3 and 10 ratios.
- **THAT 2181 datasheet**, `https://www.thatcorp.com/datashts/THAT_2181-Series_Datasheet.pdf`. This
  gives the gain element as a specification rather than a guess: gain-control constant 6.0 / 6.1 /
  6.2 mV per dB (min, typical, max) over −60 dB < gain < +40 dB at 25 °C; temperature coefficient
  +0.33 %/°C referenced to a 27 °C chip; gain-control linearity 0.5 % typical, 2 % maximum across
  that 100 dB span; dynamic range over 120 dB; trimmed THD 0.01 %, 0.02 % or 0.05 % by grade at 1 V,
  1 kHz, 0 dB; explicitly pin-compatible with the 2150 series that the original used. The datasheet
  also notes that the symmetry null for minimum distortion drifts with frequency in the presence of
  stray control-path pickup, which is a real mechanism for the unit-to-unit variation people report.
  The THAT 2180 datasheet is at the sibling URL and also downloads.

SSL themselves publish no attack, release, ratio or distortion figures for the hardware that I could
find. That is a genuine weakness relative to the Neve.

**Benchmarks.** The best benchmarking position of any candidate, because **the manufacturer ships
its own model**: SSL Native Bus Compressor 2. Waves' SSL G-Master Buss Compressor is "developed
under license from Solid State Logic". Cytomic's The Glue is the most technically candid of the
third-party models, describing "an analog modelled dual diode envelope follower, which is solved
using optimised nodal analysis" and "high quality algorithms used in circuit simulators, but
optimised to run fast". UAD and Brainworx bx_townhouse also model it. None publishes a null test.

**Effort and reuse.** The lowest of the top four. `dsp::vca` already has a dB-domain loop, a
Blackmer-ish cell and a detector; the work is a feedforward path, the dominant-channel detector, the
moving knee and the dual-constant Auto release. This is the cheapest genuinely-different model on
the list, which is why it ranks third despite not filling an empty family.

---

### 3.4 dbx 160 and 160A — VCA, true RMS, feedforward — **recommended fourth**

**Family and gain element.** VCA again, but the *other* pole of the family: David Blackmer's own
company, a true-RMS detector rather than a peak detector, and the OverEasy soft knee.

**Why it is worth modelling.** Nothing in the lab uses an RMS detector. Every detector we have is a
rectifier with a time constant. A true-RMS detector responds to power rather than to peaks, which
changes what the compressor hears on dense material in a way no amount of attack-time tuning
reproduces. The OverEasy knee is a named, specified, published curve, and the 160's "infinity
through to negative ratio" range does something none of our models can: at negative ratios the
output goes *down* as the input goes up.

**Whether it can be modelled honestly.** Yes on behaviour, partly on circuit.

- **dbx 160A operation manual**, `https://archive.org/details/DbxMODEL160AManual600dpi_201608`. Its
  specification page is unusually good on ballistics, and like the Neve it publishes program
  dependence as a table rather than a single number:

  | quantity | published value |
  |---|---|
  | attack | program-dependent; typically 15 ms for 10 dB, 5 ms for 20 dB, 3 ms for 30 dB |
  | release | program-dependent; typically 8 ms for 1 dB, 80 ms for 10 dB, 400 ms for 50 dB; 125 dB/s rate |
  | ratio | variable 1:1 to ∞:1 and through to −1:1; over 60 dB maximum compression |
  | threshold | −40 dBu to +20 dBu |
  | knee | selectable OverEasy or hard |
  | THD | under 0.2 % typical at any amount of compression up to 40 dB at 1 kHz |
  | noise | under −90 dBu unweighted, 20 Hz to 20 kHz |
  | dynamic range | over 113 dB |
  | stereo | True RMS Power Summing |

- **dbx 160 / 161 / 162 Preliminary Technical Service Manual**, dbx Professional Products, 1 May
  1991, `https://archive.org/details/dbx_160-161-162_Service_Manual`. Honest caveat: this is a
  **test and alignment manual, not a full schematic set**. It is still useful, because the alignment
  procedure names the trims and thereby the topology (R25 RMS symmetry, R61 VCA symmetry, R68 Log,
  R33 threshold, and a discrete "RMS module" with its own trim pins), and it gives factory
  distortion targets. But it will not substitute for a schematic the way the Neve documents will.
- **David E. Blackmer, US 3,681,618, "RMS circuits with bipolar logarithmic converter"**, filed 29
  March 1971, granted 1 August 1972, `https://patents.google.com/patent/US3681618A/en`. The primary
  source for the detector: an op-amp with opposite-polarity feedback junctions, ±2 gain stages, half-
  wave rectifiers and a constant-current source into a storage capacitor, driving the long-term
  average of the rectifier currents into equality with the source current. This is the detector, from
  its inventor, in a document with legal force behind its precision.

**Benchmarks.** Waves dbx 160, UAD dbx 160, and Arturia Comp VCA-65 (which models the 165, the
OverEasy sibling). No published measurements or null tests.

**Effort and reuse.** Low to moderate, and it drops further if the SSL is built first: both are
Blackmer cells with feedforward detectors, so the cell, the control law and the meter path are
shared. The new work is the true-RMS detector and the OverEasy knee.

---

### 3.5 Universal Audio 175B and 176 — variable-mu — **fifth, and only after the Fairchild**

**Family and gain element.** Variable-mu, using a 6BC8 twin triode rather than a 6386. A Bill Putnam
design; I did not find a dated primary source for its introduction, so the dossier should establish
the year rather than repeat the commonly quoted one.

**Why it is worth modelling.** Narrative, and it is a strong one. The 176 is the tube limiter that
Putnam replaced with the 1176, and the 1176's attack and release knobs are direct descendants of the
175B's. We already model the 1176 in nine revisions and the 610 preamp in the 6176. Adding the 176
completes a Putnam lineage that no other plug-in tells in one instance: the tube limiter, the tube
preamp, and the FET limiter that superseded them. It is also a much smaller, more aggressive
variable-mu box than the Fairchild, so it would not duplicate it.

**Whether it can be modelled honestly.** Better than I expected. The **Universal Audio 175B
operating manual**, `https://archive.org/details/ua-175-bmanual`, is a 15.5 MB scan that contains the
full schematic, drawing C-10476, titled "SCHEMATIC LIMITING AMPLIFIER MODEL 175/176" — that is, it
covers **both** models — plus "CIRCUIT BOARD DETAILS LIMITING AMPLIFIER MODEL 175 AND 176" and the
component location drawings B-10038 and B-10073. The drawing notes are the sort of thing that makes
a dossier: "ALL POTENTIOMETERS ARE LINEAR TAPER", "ALL RESISTORS ARE ½ W AND 5 %", and DC operating
voltages annotated for the no-signal condition at the 12:1 ratio. Published specifications include a
12:1 compression ratio above threshold, attack adjustable from 100 to 1000 µs, release adjustable
from 27 to 527 ms, signal-to-noise 80 dB, and a tube complement of one each 6BC8, 12AX7, 12BH7,
6AL5, OB2 and GZ34.

**Effort and reuse.** Small *if* the Fairchild is built first, because the variable-mu stage, the
oversampling and the feedback loop already exist by then. Built first, it would cost nearly as much
as the Fairchild for a less famous box. That dependency is why it sits at five and not two.

---

### 3.6 EMI TG12413 / Chandler TG1 — diode — **sixth, and only after the Neve**

**Family.** Diode-based limiter, the Abbey Road console section behind most of what EMI recorded
from the late 1960s onward. Its documentation is a single but real item: **"EMI TG Schematics"**,
`https://archive.org/details/JL11440`, a 17 MB scan covering the TG12411 input, TG12412 EQ,
**TG12413 compressor/limiter**, TG12414 filter and TG12417 fader. There is no service manual, no
published specification and no calibration data that I could find, so the ballistics would have to
be derived from the schematic alone with nothing to check them against. That is a real risk, and it
is only acceptable as a *second* unit in a family whose detector and bridge have already been
validated against the Neve's published tolerances. Alone, it would be a dossier of adjectives.

### 3.7 API 2500 — VCA — **assessed, ranks below the SSL**

Genuinely well specified for a modern unit: Sound On Sound's review establishes four THAT 2180 VCAs
and one THAT 2252 RMS level detector per channel, seven attack steps from 0.03 ms to 30 ms, six
release presets from 0.05 s to 2 s plus variable to 3 s, ratios from 1.5:1 to 10:1 and infinity in
seven steps, selectable hard, medium or soft knee, an Old/New switch that is literally feedback
versus feedforward sensing, and the Thrust sidechain filter with a 2 dB/octave slope on Med and
4 dB/octave on Loud. Universal Audio's page claims "exclusive access to API's schematics" and that
they "analyzed two classic 'golden unit' API 2500s", which is a benchmark worth having.

It ranks below the SSL because it is a third VCA rather than a second, API publishes no schematic,
and `apiaudio.com` refused a TLS handshake from this machine so I could not reach API's own manual.
Its Old/New switch would be a good thing to own eventually, since it makes feedback and feedforward
a user-facing parameter, which is a nice teaching artefact for the lab.

### 3.8 Altec 436C — variable-mu — **assessed, thin**

The Motown compressor, and historically important out of proportion to its price. Documentation is
one file: `https://archive.org/details/altec-lansing-436-c-compressor-amplifier-schematic`, a single
47 KB GIF of the schematic. No manual, no specification, no measurement. A schematic with nothing to
check it against is half a dossier. Worth revisiting only after a variable-mu engine exists.

---

## 4. Candidates I assessed and would advise against

**Gates Sta-Level — despite its fame.** This is the one I would explicitly warn off. It is a famous,
much-loved 1956 variable-mu limiter, it is on many "best compressors ever" lists, and it has a
current officially-endorsed recreation. But I could not reach a schematic or a service manual for
the 1956 original anywhere: archive.org has nothing under any spelling I tried, and neither do the
broadcast archives. Worse, **Retro Instruments' own product page for their recreation will not say
what the original's gain-reduction tube was.** It says only that *their* version "uses a pair of
easily-obtainable 6BA6 gain reduction tubes", which is a statement about the recreation and a
conspicuous silence about the original. Their linked Kazrog plug-in page is now only a redirect. A
Sta-Level dossier would be built from a modern reinterpretation and secondhand description, which is
exactly the failure mode the lab's research standard exists to prevent. Skip it until a schematic
surfaces.

**Manley Variable Mu.** Manley publishes good numbers on their product page: 5670 × 2 standard or
6BA6 × 4 with the T-BAR mod, variable attack 25 to 70 ms, five recovery steps at 0.2, 0.4, 0.6, 4
and 8 s, ratios 1.5:1 compress and 4:1 to 20:1 limit, a sidechain high-pass at −3 dB / 100 Hz, and
20 Hz to 25 kHz ±0.1 dB. But they publish **no schematic and no manual** that I could find; their
manuals page links out to dealers. It is also a current, actively-sold product from a small
independent manufacturer, which is a different proposition from spoofing a 1959 Fairchild. I would
leave it.

**UREI LA-4.** The manual exists (`https://archive.org/details/studio_UREI-LA-4_manual`) and it is a
good box, but it is a fourth optical compressor. We have three. It would share the photocell, the
divider and most of the sidechain with `dsp::opto3`, and the audible difference would be smaller than
the difference between our existing LA-2A cell options. It fills nothing.

**Urei 1178, Purple MC77, Warm WA76, Klark Teknik 1176 and every other 1176.** These are the
"faceplate, not a model" case. Our `dsp::fet` already covers nine revisions. A tenth would be a
different silkscreen over the same transfer curve. The 1178's only real addition is stereo linking,
which the lab already has as a shared `link` parameter.

**Summit TLA-100A, Joemeek SC2, Avalon VT-737.** All optical or optical-ish. Same objection as the
LA-4.

**Focusrite Red 3.** A VCA with a good reputation and effectively no public circuit documentation.

**Siemens U273 and Dolby A301.** Both are diode-based, and both are documented on archive.org
(`studio_Siemens_U273_Limiter` and `JL11202`) and cited by the DAFx-2025 paper. But neither is a
compressor anyone asks for by name; the A301 is a noise-reduction system whose VCA merely happens to
be a diode bridge. They are useful as *supporting sources* for a Neve dossier, not as models.

---

## 5. Recommended order

1. **Neve 2254 / 33609 (diode bridge).** Fills an empty family, best documented, closed-form gain
   element, still-manufactured hardware to compare against.
2. **Fairchild 660 / 670 (variable-mu tube).** Fills the other empty family, deep circuit
   documentation and a published tube model, but expensive and with weak ground truth.
3. **SSL 4000 G bus compressor (mainstream VCA).** Cheapest genuinely-different model on the list,
   and the manufacturer ships its own emulation to benchmark against.
4. **dbx 160 / 160A (true-RMS VCA).** The first RMS detector in the lab, with a published
   program-dependent ballistics table and the inventor's own patent for the detector.
5. **Universal Audio 176 (variable-mu).** Cheap once the Fairchild exists, and it completes the
   Putnam lineage against our 1176 and 610.
6. **EMI TG12413 (diode).** Cheap once the Neve exists, but has no published specification, so build
   it only on top of a validated bridge.

### Why the Neve is first, stated plainly

Three reasons, in order.

**It is the only candidate where the documents that make an honest dossier possible are the same
documents a recent peer-reviewed paper used to derive a real-time model of the exact gain element.**
Pines' DAFx-2025 paper names the Neve 2254/E and 33609/J in its introduction and cites, as
references [6] and [7], the two archive.org items I have verified download. So the dossier can
establish the bridge from a Neve blueprint, and then solve it in closed form with a published,
stateless, non-iterative method. Every other candidate requires either inventing the modelling
approach (the SSL, the dbx) or importing one validated against a simulator rather than the circuit
(the Fairchild). That alignment does not exist anywhere else in this survey.

**Its ground truth is the best available anywhere in the category.** The lab's research standard
prefers calibration points to specification prose. The CL 1B dossier has a whole section on exactly
that, because Lydkraft's numbers were so thin. AMS Neve publish a per-ratio transfer table with
tolerances, attack times with tolerances *and* a stated measurement procedure, six recovery times,
and three distortion figures each with its own operating conditions. Test 10.1-style calibration
tests can be written straight out of the handbook, and they will have real tolerances rather than
invented ones. As a bonus, the table reveals that the panel labels are approximations, which is a
finding the dossier can state and defend.

**It fills the family that most changes how the lab sounds, for the least new machinery.** The
diode bridge distorts as a function of gain reduction, inside the attenuator, between two
transformers. Nothing we own does that. And unlike the Fairchild it needs no oversampled push-pull
tube stage and no iterative solver, so the cost is a new gain element and a second sidechain rather
than a new engine architecture.

The 33609's two independent sidechains driving one bridge is also the most interesting new *shape*
on the list, and it is the one thing here that would teach `Processor` something it does not already
know.

---

## 6. What I could not reach, said plainly

- **`apiaudio.com`** refuses a TLS handshake from this machine (`SEC_E_ILLEGAL_MESSAGE`), so API's
  own 2500 manual is unread. The Sound On Sound review substituted.
- **Sound On Sound's search is JavaScript-driven** and returns a default article list to a plain
  fetch. Most review slugs I guessed return HTTP 410, including `soundonsound.com/reviews/fairchild-670`.
  Only the API 2500 review resolved. There may be SoS reviews of the 33609, the SSL and the dbx 160
  that I simply could not locate.
- **No Gates Sta-Level schematic or service manual** exists at any address I tried.
- **Manley publishes no schematic or manual** for the Variable Mu.
- **DuckDuckGo and Mojeek both refuse `curl`**, so every document above was found by constructing
  URLs, walking directory listings, or querying the Internet Archive's search API. The GE 6386
  datasheet was found by probing all 164 volume directories on `frank.pocnet.net`; it is in volume
  142 and no guessed path found it.
- **`funkwerkes.com` returns 403 on directory listings**, though direct PDF links still work. That is
  the host the CL 1B service manual came from, so it remains usable if you know the exact filename.
- **No independent laboratory measurement of any candidate's hardware** was found. This is a
  category-wide gap, not a failure of a particular search.

---

## 7. Sources

Everything below was fetched during this survey. Manufacturer documents are cited as manufacturer
claims; the clone-builder page is cited as a clone builder's reading of a schematic he had and I do
not.

**Primary circuit documents**

1. Neve 2254/E Limiter and Compressor schematic set, drawing L/10,004/E, The Neve Group of Companies,
   20 September 1972 rev. 11 July 1974, with BA185, BA191, BA192 and BA283 card schematics, switch
   decks, front panel and alignment block. https://archive.org/details/neve-2254-schematics
2. AMS Neve plc, *33609/J Limiter/Compressor Technical Handbook*, 3rd edition, doc. 527-149 Issue 3,
   2002. https://archive.org/details/neve-33609-j-technical-handbook
3. Neve 33609/N owner's manual. https://archive.org/details/Neve_33609-N_owners_manual
4. Fairchild 670 stereo limiting amplifier schematic. https://archive.org/details/JL10883
5. Fairchild 660 mono limiting amplifier schematic. https://archive.org/details/JL10866
6. Fairchild 663 mono compressor schematic. https://archive.org/details/JL10873
7. Fairchild 670 component layout. https://archive.org/details/JL10876
8. Fairchild 670 owner's manual, with the full specification sheet.
   https://archive.org/details/Fairchild_670_owners_manual
9. Fairchild 670 Limiter manual and instructions. https://archive.org/details/JL10878 and
   https://archive.org/details/JL10882
10. General Electric, *6386 Twin Triode*, datasheet ET-T1113, August 1954.
    https://frank.pocnet.net/sheets/142/6/6386.pdf
11. Universal Audio, *Model 175B Limiting Amplifier operating manual*, containing schematic C-10476
    for the Models 175 and 176 and their circuit board details.
    https://archive.org/details/ua-175-bmanual
12. dbx Professional Products, *Model 160/161/162 Preliminary Technical Service Manual*, 1 May 1991.
    https://archive.org/details/dbx_160-161-162_Service_Manual
13. dbx, *Model 160A operation manual*, with the specification page.
    https://archive.org/details/DbxMODEL160AManual600dpi_201608
14. EMI TG schematics: TG12411 input, TG12412 EQ, TG12413 compressor/limiter, TG12414 filter, TG12417
    fader. https://archive.org/details/JL11440
15. Altec Lansing 436C compressor amplifier schematic.
    https://archive.org/details/altec-lansing-436-c-compressor-amplifier-schematic
16. UREI LA-4 manual. https://archive.org/details/studio_UREI-LA-4_manual
17. Siemens U273 limiter documentation. https://archive.org/details/studio_Siemens_U273_Limiter
18. David E. Blackmer, "RMS circuits with bipolar logarithmic converter", US 3,681,618, filed 29 March
    1971, granted 1 August 1972. https://patents.google.com/patent/US3681618A/en

**Component data**

19. THAT Corporation, *THAT 2181-Series Blackmer Pre-Trimmed IC Voltage Controlled Amplifiers*
    datasheet, doc. 600030 rev. 03. https://www.thatcorp.com/datashts/THAT_2181-Series_Datasheet.pdf
20. THAT Corporation, *THAT 2180-Series* datasheet.
    https://www.thatcorp.com/datashts/THAT_2180-Series_Datasheet.pdf
21. Radiomuseum, 6386 tube entry: remote-cutoff medium-mu twin triode derived from the 2C51, mu 17.
    https://www.radiomuseum.org/tubes/tube_6386.html

**Modelling literature**

22. Coriander V. Pines, "Real-Time Virtual Analog Modelling of Diode-Based VCAs", *Proc. DAFx-25*,
    Ancona, September 2025. https://www.dafx.de/paper-archive/2025/DAFx25_paper_44.pdf
23. Peter Raffensperger, "Toward a Wave Digital Filter Model of the Fairchild 670 Limiter",
    *Proc. DAFx-12*, York, September 2012.
    https://www.dafx.de/paper-archive/2012/papers/dafx12_submission_9.pdf
24. Julian Parker, "A Simple Digital Model of the Diode-Based Ring-Modulator", *Proc. DAFx-11*, Paris,
    2011. https://www.dafx.de/paper-archive/2011/Papers/66_e.pdf
25. Alec Wright and Vesa Välimäki, "Grey-Box Modelling of Dynamic Range Compression", *Proc. DAFx-22*,
    Vienna. https://www.dafx.de/paper-archive/2022/papers/DAFx20in22_paper_35.pdf
26. DAFx paper archive search, used to establish what has and has not been published on each family.
    https://www.dafx.de/paper-archive/search.php?q=diode

**Manufacturer and clone documentation**

27. Solid State Logic, *500-Series G Comp Module User Guide*, revision V2.0, June 2020.
    https://solidstatelogic.com/assets/uploads/downloads/SSL_500_Series_G_Comp_Module_User_Guide.pdf
28. Solid State Logic, Stereo Bus Compressor Module product page.
    https://solidstatelogic.com/products/stereo-bus-compressor-module
29. Solid State Logic, SSL Native Bus Compressor 2.
    https://www.solidstatelogic.com/products/ssl-native-bus-compressor-2
30. Jakob Erland (Gyraf Audio), "GSSL" stereo bus compressor project, built from SSL4000E cards 82E26
    and 82E27. https://www.gyraf.dk/gy_pd/ssl/ssl.htm
31. Manley Laboratories, Variable Mu Limiter Compressor.
    https://www.manley.com/products/pro-audio/dynamics/variable-mu
32. Retro Instruments, Sta-Level. https://www.retroinstruments.com/product.php?product_id=stalevel

**Emulations, for benchmarking**

33. Arturia, Comp DIODE-609 (Neve 33609 'C').
    https://www.arturia.com/products/software-effects/comp-diode-609/overview
34. Universal Audio, Fairchild Tube Limiter Collection.
    https://www.uaudio.com/products/fairchild-tube-limiter-collection
35. Universal Audio, API 2500 Bus Compressor.
    https://www.uaudio.com/products/api-2500-bus-compressor
36. Waves, SSL G-Master Buss Compressor, "developed under license from Solid State Logic".
    https://www.waves.com/plugins/ssl-g-master-buss-compressor
37. Waves, dbx 160 Compressor / Limiter.
    https://www.waves.com/plugins/dbx-160-compressor-limiter
38. Waves, PuigChild Compressor. https://www.waves.com/plugins/puigchild-compressor
39. Cytomic, The Glue. https://www.cytomic.com/product/glue/
40. Cytomic technical papers, checked for compressor or VCA content and found to contain none.
    https://www.cytomic.com/technical-papers/

**Background and taxonomy**

41. Sound On Sound, "Classic Compressors". Useful as a family taxonomy and for confirming which units
    are bridged-diode, variable-mu, optical, FET or VCA. Cited for taxonomy only: it gives no
    quantified attack, release, ratio or frequency-response figures for any unit.
    https://www.soundonsound.com/techniques/classic-compressors
42. Sound On Sound, API 2500 review, for the THAT 2180 and 2252 complement, the Thrust slopes and the
    control ranges. https://www.soundonsound.com/reviews/api-2500
43. The John Leimseider archive on the Internet Archive, the collection holding most of the Fairchild
    material. https://archive.org/details/john-leimseider-archive

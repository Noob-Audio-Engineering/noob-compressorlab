# The Tube-Tech CL 1B opto compressor: research notes for the CL 1B side of `noob-compressorlab`

Research dossier for the CL 1B model of the `noob-compressorlab` example plug-in of noob-vst-webgui-framework.
The example is a humorous, affectionate spoof of the Lydkraft / Tube-Tech CL 1B Opto Compressor. It is not a
product, it is not endorsed by anybody, and it does not use the CL 1B, Tube-Tech or Lydkraft names as its own
name. Trademarks below belong to their owners and are used only to identify the device and the products
discussed. This model sits behind the same per-instance `model` switch that already selects the 1176, LA-2A,
LA-3A, Distressor and 6176 behaviours; see [[1176]], [[610]], [[LA-2A]], [[LA-3A]] and [[Distressor]]. The 6176
is the 610 and the 1176 together and has no dossier of its own.

Conventions (kept the same as the other dossiers so they all read alike):

- Citations are `[n]`; the numbered list in section 11 gives the URL for every source, and reference-style link
  definitions at the very end make the `[n]` markers clickable.
- Numbers that come from a manufacturer specification, a manual, a service manual, a schematic or a measurement
  are attributed. Numbers that are my own derivation or assumption are labelled **estimate** or **derived**.
  Nothing labelled a measurement was invented.
- "GR" is gain reduction. "GRE" is Lydkraft's own name for the gain-reduction element, the optical attenuator.
  "THD" is total harmonic distortion. dBm and dBu are used as the sources use them; on this device, into
  600 Ω, they are the same number. dBFS is digital full scale.
- The CL 1B is a spoof target, not a parity goal. I want the *feel* of the thing: the way it takes six decibels
  off a vocal without anybody noticing, the very long release that never sounds like it is letting go, and the
  ratio control that does not really set a ratio. I do not want a component-accurate clone and I am not trying
  to beat Softube, who have had a licence and the actual designer's help since 2006 [14] [15].
- **A warning that colours the whole file.** Nobody outside Lydkraft has published a single independent
  measurement of a CL 1B. No THD-versus-level curve, no measured frequency response, no measured attack or
  release, no gain-reduction curves, no ratio curves. I looked, and so did a second pass; Audio Science Review,
  the one venue that routinely bench-tests pro audio hardware, has nothing on it at all [76]. Every number in
  circulation traces back to Lydkraft's own copy. So section 10's tests assert *manufacturer specifications and
  the service manual's own calibration procedure*, and where neither exists I say so instead of inventing a
  bound. That is a real constraint on this model and I would rather state it once, loudly, than let a reader
  assume the figures are better established than they are.

**The one thing to carry away before reading any of it.** The CL 1B looks like an LA-2A with more knobs. It is
not. On the LA-2A and the LA-3A the *cell is the compressor*: the T4's carriers and traps are what produce the
attack, the two-stage release and the programme-dependent memory, which is why `src/dsp/opto/model.rs` is
mostly cell physics and why the LA-3A model could reuse it whole. On the CL 1B the timing lives in an op-amp
sidechain, in a 10 µF capacitor with a diode-and-resistor charge path and a resistive pull-up discharge path,
and the optical element is asked only to convert a control current into a resistance as quickly and as boringly
as it can. Reusing `opto::model::Cell` here would import the T4's memory into a machine that does not have any,
and would quietly break the one thing the CL 1B is famous for: a release you can actually set. Section 9.1 makes
that decision explicitly and section 10's test 27 exists to catch anybody who undoes it.

---

## 1. What the CL 1B is

### 1.1 Lydkraft, and John G. Petersen

- **Lydkraft** ("sound power" in Danish) was founded in Denmark in **1977** by **John G. Petersen** and two
  partners. It served the PA market until **1980**, when Petersen took the company over and turned it towards
  direct boxes, mixing desks and speaker systems. [10]
- Petersen was trained as an electronics engineer at the Danish Post and Telegraph in **1969** and worked from
  **1972** for the Danish Broadcasting Company as a maintenance engineer, factory-trained by Solid State Logic,
  EMT, Studer, Sony, Neumann, NTP and Lyrec; he designed the playback and record amplifiers for Lyrec's tape
  machines. [10]
- The origin story is a Pultec-and-LA-2A story: "During the early eighties, John came across several classic
  tube processors such as the Pultec equalizers, and Teletronics compressor LA2A. John liked the high quality
  and the simplicity of these units and started working his own tube designs." He decided to develop his own
  units in **1984**, and the first Tube-Tech product, the **PE 1A** program equaliser, shipped in **1985** from
  his private garage. [10]
- The company is still Lydkraft ApS, Mose Allé 20, 2610 Rødovre, Denmark. The 1993 schematics carry the older
  address, Ved Damhussøen 38, DK 2720 Vanløse. [2] [9]
- Lydkraft's own round number: "TUBE-TECH has been around for 40 years. And over the years, more than 15,000
  Blue Units have been shipped from Lydkraft, Denmark." That is the whole range, not the CL 1B alone. [10]

### 1.2 CL 1A, CL 1B, CL 2A and CM 1A: the family, and exactly what differs

Lydkraft's own product timeline, verbatim [10]:

| Year | Product | Their words |
|---|---|---|
| 1987 | **CL 1A** | "CL 1A Opto 'LA2A type' Compressor. Hard wired." |
| **1991** | **CL 1B** | "Equal to CL1A but featuring Circuit Boards" |
| 1993 | LCA 2B | "Stereo Compressor (With a tube as VCA)" |
| 1998 | **CL 2A** | "Two Channel Opto Tube Compressor" |
| 2000 | SMC 2B | "The World's first Tube Multiband Compressor" |
| 2008 | **CM 1A** | "Opto Compressor module" (RM series), since discontinued [49] |

Two things in that table matter. First, **Tube-Tech themselves call the CL 1A an "LA2A type" compressor**, which
settles the lineage question before anybody has to argue about it. Sound On Sound put it the same way in 2023:
"the CL‑1A was based on what at that time was the discontinued UREI/Teletronix LA‑2A valve‑amplified optical
compressor". [19]

Second, **the CL 1B is 1991, not 1987**. This trips people up constantly, because 1987 is the CL 1A's year and
because the two are electrically identical. Lydkraft's own FAQ, on the CL 1B product page: "FAQ: Opto
Compressors CL 1A and CL 1B difference — The circuit and functions are exactly the same. The amplifier section
in the CL 1A is hardwired. In the CL 1B the transformers and the tubes are placed inside the unit on a printed
circuit board. The control circuit is on a PCB in both units." [1] Attack Magazine's compressor listicle gets
this wrong and dates the CL 1B to 1987 [21]; Bonedo and Sound On Sound both get it right [20] [19]. Use 1991.
Sound On Sound in December 2023: it "has been in continuous production since 1991", thirty-two years and
counting. [19]

The two siblings, for contrast, both from Lydkraft's own specification lists:

| | **CL 1B** [1] [3] | **CL 2A** [12] | **CM 1A** [13] |
|---|---|---|---|
| Channels | 1 | 2, linkable | 1 (RM-series module) |
| Ratio | 2:1 to 10:1 | **1.5:1 to 10:1** | 2:1 to 10:1 |
| Threshold | off to −40 dBu | **off to −20 dBu** | **off to −30 dBu** |
| Output gain | off to +30 dBu | **off to +10 dBu** | off to +30 dBu |
| Response, −3 dB | 5 Hz to 25 kHz | **5 Hz to 60 kHz** | 5 Hz to 25 kHz |
| Noise | < −75 dBu at 30 dB gain | **< −80 dBu at 10 dB gain** | < −75 dBu at 30 dB gain |
| Metering | VU: input / compression / output | VU: **output or compression** | **LED** display |
| Busses | Off, Bus 1, Bus 2 | Off, Bus 1, Bus 2, plus a Link switch | Off, Link 1, Link 2 |

The CM 1A is the interesting one, because Lydkraft say out loud what changed and it is exactly the thing this
dossier cares about: "It features **exactly the same tube circuit** as our famous CL 1B Opto Compressor, but the
action is different due to a **different, carefully selected optical element** and a fine-tuning of the control
circuit. This provides the CM 1A with an attack that enhances transients in a more punchy way ... The different
optical element does NOT seriously change the 'classic' characteristics, known from the CL1B. The action of the
CM1A places it somewhere between the LCA2B (Tube VCA) and the classic CL1B." [13] So Lydkraft's own position is
that the tube amplifier is generic Tube-Tech and **the optical element is the personality**. That is a strong
hint about where to spend the modelling effort, and section 4 takes it seriously.

### 1.3 What it took from the LA-2A, and what it added

This is the point of the model, so it gets its own table. Sources and detail are in sections 3 to 5.

| Element | LA-2A | CL 1B |
|---|---|---|
| Gain element | T4 cell shunting a resistive divider, in the **audio path** | a proprietary optical **GRE** shunting a resistive divider, in the **audio path** [19] |
| Detector topology | feedback | **feedback, with a variable amount of it** — that is what the Ratio knob does (3.4) |
| Audio amplifier | 12AX7A voltage amp, 12BH7A cathode follower | **ECC83** voltage amp plus cathodyne phase splitter, **ECC82** push-pull output, output transformer [2] [9] [20] |
| Sidechain | a 12AX7A into a 6AQ5A driving the panel from a high-voltage rail | **two LF347 quad JFET op-amps, one BC337 NPN, one BF245A JFET**, on ±15 V [2] [9] |
| Where the time constants live | **in the cell**: carriers, traps, memory | **in the sidechain**: one 10 µF capacitor with switchable charge and discharge paths (3.7) |
| Attack | 10 ms typical, not adjustable | **1 ms fixed, or 0.5 to 300 ms by knob** [2] [3] |
| Release | 60 ms to 50 %, then 0.5 to 5 s, not adjustable | **50 ms fixed, or 0.05 to 10 s by knob, or a combination** [2] [3] |
| Ratio | about 3:1 Compress, ∞:1 Limit | **continuously variable 2:1 to 10:1** [2] [3] |
| Threshold | implicit in Peak Reduction | **an explicit control, marked in dBu**, defined as the 1 dB point [2] |
| Make-up gain | 40 dB | **off to +30 dB**, and the service manual pins it at exactly +30.0 dB [2] [9] |
| Bandwidth | +0 / −1 dB, 30 Hz to 15 kHz | **5 Hz to 25 kHz at −3 dB** [1] [3] |
| Distortion | 0.9 % to 4.2 % measured across six units | **0.15 % THD+N at 40 Hz**, at both 0 and +10 dBu [3] |
| Linking | none | **two sidechain busses, up to ten units** [2] |
| Size | 19 inch, 2U, heavy | 19 inch, **3U**, 4.1 kg net [3] [19] |

Read that as one sentence: **Lydkraft kept the LA-2A's optical attenuator and its all-tube audio path, threw
away everything that made its timing uncontrollable, and put a proper compressor's front panel on the result.**
Sound On Sound says the same thing from the other end: "Lydkraft had added a full set of conventional
compressor controls (attack, release, ratio, threshold and make‑up gain), plus bypass and attack/release timing
selector switches — this innovative switched circuit determines whether the attack/release timings are fixed,
manually controlled or determined by a combination of those two modes, delivering a fixed attack time and a
programme‑dependent release. An additional innovation was the two switch‑selectable side‑chain busses that
allowed multiple units to be linked and controlled when required." [19]

### 1.4 Why it is famous

- Lydkraft's own headline is not subtle: "CL1B: The Industry Standard Vocal Compressor!" and "the rule is that
  the majority of all top singers and musicians demands the CL1B for their recordings regardless of the genre
  they perform. As an example, it's well known secret that all the big Hip Hop / Rap stars depend heavily on the
  CL1B for their vocal performance." [1]
- The strongest evidence for that claim is not a testimonial but a description of a whole city's default chain.
  Kesha Lee, the Atlanta engineer behind Lil Uzi Vert, Migos, Young Thug and Travis Scott, in Sound On Sound's
  Inside Track: "The standard for Atlanta seems to be to use Yamaha NS10s or Augspurger monitors, and the
  recording signal chain consists of an Avalon VT-737sp mic pre and a **Tube-Tech CL 1B compressor**. The
  microphone is normally a Neumann U87." [28] That is the CL 1B as regional infrastructure.
- Kanye West's *Yeezus* (2013) went through one for every vocal, tracked and live, by Noah Goldstein; Lydkraft
  note with visible delight that West "normally doesn't want to have his name connected to any specific piece of
  gear" and made an exception. [26]
- Richard Furch ran every lead vocal on Tyrese Gibson's *Black Rose* through his: "if I just tap the Tube-Tech
  CL1B (just 1dB to 3dB of gain reduction) it makes the vocals leaner and beautifully highlights all of their
  details. **It's like it puts a satin glove on the high end.** ... All-in-all, the Tube-Tech CL1B puts a
  'blanket of awesome' on vocals." He used "a medium attack time with a mild ratio in the neighborhood of 3:1"
  and "configured all of his settings in manual mode". [22] [23]
- Marcello Spiridioni, mastering at Forward Studios in Rome: "There is almost no compressor I can think of that
  glues and smoothens out vocals like the CL 1B does; it makes them sit perfectly in the mix while still
  sounding so natural and even." [27]
- Attack Magazine rank it **fourth** in their twenty best hardware compressors ever made, and put the character
  in one line: "the CL-1B is not noted as a character piece; it does warmth and it does smooth, and it does both
  transparently" and it is "one of the favourite go-to compressors for vocals (often in series with an 1176 or
  LA-2A)". [21]
- It gets used where a whole show has to survive: the CL 1B took the lead vocals on the Danish X-Factor,
  mixed on an SSL C200, alongside an SMC 2B on the master bus [47], and it sits in three teaching rooms at
  Full Sail University in Orlando [48].
- It is not only a vocal box. Scotty Simpson takes one on the road for the Oak Ridge Boys' bass rig, 150 nights
  a year: "It's like the Rolls Royce of compressors ... And it's definitely colored. If you took away the
  Tube-Tech, you'd notice a big difference." His settings, which are a useful anchor for the tapers: input gain
  slightly above unity, "the attack around 11 o'clock, the release around 1 o'clock, and the ratio near 2.5:1".
  [24] [25]

### 1.5 Production, price, and the serial-number question

- In continuous production since 1991 and still made, which is unusual enough that Sound On Sound remarked on
  it. [19]
- Price has climbed steeply. A dealer quoted a new MAP of **$3150** around 2008-09, with used units then "between
  $2100-$2500", and remembered them at **$1000-$1150 used** in the early Gearspace years. [39] A dated step:
  "They went from $3190 to $3690 this week, so a $500 price increase ... the 1st of April". [38] Bonedo listed
  **EUR 3273 (UVP)** in 2013 [20]; Thomann currently list **£3444** and out of stock [43]; Bonedo's live buy link
  now shows **€3990** [20].
- **How many were made is not published anywhere.** The only public number that bounds it at all is the service
  manual's own serial breakpoint for the GRE selection code: "For serial no. up to 15891 ... For serial no.
  15892 and onwards" [2]. That implies at least about 15,900 units of *something* had passed through that serial
  sequence by the changeover. I am not confident the sequence is CL 1B-only rather than shared across Tube-Tech
  models, and Lydkraft's "more than 15,000 Blue Units" figure is for the whole range [10], so treat this as a
  curiosity, not a production figure. **Estimate, and a weak one.**

---
## 2. Controls, the front panel and their real ranges

### 2.1 The front panel, close enough to draw

Described from Lydkraft's own **hi-res front photograph** `CL1BFront.jpg` [6], saved locally as
`ref/cl1b-front-hires.jpg`, cross-checked against their **recall sheet** [5], which is a clean line drawing of
the same panel with every legend set in type, and their **brochure** [4], which shows the panel in colour at
smaller size. Every figure in section 2.2 was measured off that photograph by me; every legend below was read
off the recall sheet, where the type is unambiguous.

**Faceplate.** A **blue** powder-coated steel panel, **19 inch wide and 3U tall** (section 2.2 measures the
aspect ratio and gets 131 mm, which is 3U; the specification sheet's "Height: 2 units" is a typographical error,
because the same line gives 132 mm and 5.2 inch, both of which are 3U, and Sound On Sound explicitly contrast the
WA-1B's 2U with "the CL‑1B's 3U" [3] [19]). All lettering is **white**, a plain grotesque sans in caps for the
control names and lower case for the switch positions. Two rack ears, each with two rounded oval slots, and four
chrome pan-head Phillips screws set inboard of the ears, top and bottom.

**Top row, left to right.**

- **TUBE-TECH** in large white caps at the far left, with a long hyphen that reads almost as an en-dash. Under
  it, smaller and bolder, **COMPRESSOR   CL 1B**, and under that **LYDKRAFT   DENMARK**. Three lines, left
  aligned, occupying the leftmost sixth of the panel.
- **GAIN.** A large black knob, glossy, with a faceted polygonal skirt (about ten flats) and a domed top, and a
  single white index line running from near the centre down the sloping face to the skirt. Around it, printed on
  the panel, small white dots with white legends outside them: **off** at the counter-clockwise stop, then
  **−10, 0, +10, +20, +30**, with +10 at top dead centre.
- **RATIO.** The same knob. Only two marks: **2:1** at the counter-clockwise stop and **10:1** at the clockwise
  stop. Nothing in between, which is honest of them.
- **THRESHOLD.** The same knob. **off** at the counter-clockwise stop, then **0, −10, −20, −30, −40**, with −20
  at top dead centre. Note the direction: clockwise *lowers* the threshold and therefore compresses more, which
  is the same sense as the LA-2A's Peak Reduction and the opposite of a modern threshold knob's dB reading.
- **METER.** A three-position black lever switch with a white index stripe, on a ball bushing. Legends above it:
  **input** to the left, **compres-** over **sion** on two lines at the top, **output** to the right.
- **The VU meter**, right of centre. A rectangular movement in a black moulded bezel with a cream/ivory face. The
  upper arc carries the standard VU dB scale, **−20, −10, −7, −5, −3, −2, −1, 0** in black and then **1, 2, 3**
  in red, with a heavy red arc band running from 0 to +3, a small black **−** at the left end and a red **+** at
  the right. A second, smaller arc below it carries the percentage scale **0, 20, 40, 60, 80, 100**. Across the
  middle, **LYDKRAFT** in a serif face over a horizontal rule; **VU** in bold sans at the lower right. The
  pointer is black and rests at the far left. A small manufacturer's code sits in the bottom left corner of the
  face, too small to read in the press photograph.

**Bottom row, left to right.**

- **IN.** A small black bat-handle toggle with a chrome hex bushing, below the wordmark block, with **IN**
  silkscreened above it. This is the bypass: "This leverswitch switches the compressor in and out of the signal
  path. The out position bypasses the entire compressor." [2]
- **ATTACK.** The same large black knob, with **fast** at the counter-clockwise stop and **slow** at the
  clockwise stop, and nothing in between.
- **RELEASE.** Identical, **fast** to **slow**.
- **attack/release SELECT.** A three-position lever switch identical to the meter switch. Legends above it:
  **fixed** left, **fix./man.** top, **manual** right. The control name is printed below in two lines,
  **attack/release** in lower case over **SELECT** in caps.
- **sidechain BUS SELECT.** The same switch. Legends above: **off** left, **1** top, **2** right. Name below:
  **sidechain** over **BUS SELECT**.
- **The pilot lamp.** A round red faceted jewel in a chrome bezel. Bonedo call it what it is: "Der
  Betriebszustand wird stilecht von einer Glühlampe mit 'Rubin-Abdeckung' verkündet" — an incandescent lamp
  under a ruby cover. [20] The schematic calls it B1, 12 V / 0.1 A, fed through R16 68.1 Ω. [9]
- **OFF / ON.** The mains switch, at the far right. It is a small black knob with a white index line, not a
  toggle, with **OFF** printed above and to its left and **ON** above and to its right. Bonedo: "der
  Drehschalter rechts außen setzt die Kiste entsprechend unter Strom." [20]

### 2.2 Measured geometry, from Lydkraft's own photograph

Everything in this table is **my measurement** of `CL1BFront.jpg` [6], done by locating the panel's bounding box
and then each control's non-blue blob, and expressed as a fraction of the panel's width and height so the face
can be drawn at any size. Physical millimetres assume a 483 mm panel, which is the specification's own width [3].
The panel's measured aspect ratio is **3.695**, which at 483 mm wide implies **131 mm tall**; the specification
says 132 mm, so the photograph and the specification agree to under one per cent and both say 3U.

| Feature | x (fraction of width) | y (fraction of height) | Size |
|---|---|---|---|
| GAIN knob centre | 0.218 | 0.268 | ⌀ 36 mm |
| RATIO knob centre | 0.362 | 0.272 | ⌀ 36 mm |
| THRESHOLD knob centre | 0.506 | 0.268 | ⌀ 35 mm |
| METER lever bushing | 0.631 | ≈ 0.27 | — |
| VU meter bezel centre | 0.788 | 0.271 | 0.174 × 0.323 of the panel, ≈ 84 × 42 mm |
| ATTACK knob centre | 0.216 | 0.742 | ⌀ 36 mm |
| RELEASE knob centre | 0.360 | 0.740 | ⌀ 35 mm |
| attack/release SELECT bushing | 0.516 | ≈ 0.74 | — |
| sidechain BUS SELECT bushing | 0.631 | ≈ 0.74 | — |
| Pilot jewel centre | 0.735 | 0.746 | ⌀ 19 mm including the chrome bezel |
| OFF/ON switch knob centre | 0.840 | 0.752 | ⌀ 22 mm |
| IN toggle centre | 0.111 | 0.748 | — |
| Wordmark block, left edge | ≈ 0.04 | 0.33 to 0.55 | three lines |

The two lever-switch y positions are marked "≈" because the connected-component measurement bounds the whole
lever, which hangs below the bushing; the bushing centre is my **estimate** from the same image.

**Knob travel, measured from the scale dots.** I located every white scale dot around each of the five knobs and
computed its angle from top dead centre. All five knobs sweep **239°**, from **−119°** (about 8 o'clock) to
**+120°** (about 4 o'clock). Measured dot angles:

| Knob | Legends, counter-clockwise stop to clockwise stop | Angles from 12 o'clock | Knob fractions |
|---|---|---|---|
| GAIN | off, −10, 0, +10, +20, +30 | −118.0, −83.6, −54.7, +2.6, +42.0, +120.7 | 0.000, 0.144, 0.265, 0.505, 0.669, 0.999 |
| THRESHOLD | off, 0, −10, −20, −30, −40 | −119.3, −84.4, −54.3, +4.8, +44.8, +120.7 | 0.000, 0.146, 0.272, 0.519, 0.686, 1.000 |
| RATIO | 2:1, 10:1 | −117.4, +120.0 | 0.000, 1.000 |
| ATTACK | fast, slow | −119.1, +119.5 | 0.000, 1.000 |
| RELEASE | fast, slow | −119.8, +120.4 | 0.000, 1.000 |

**Two findings fall straight out of that table.**

First, **the GAIN and THRESHOLD scales are the same artwork**. Their dot angles agree to within 2.5° at every
one of the six positions. That is not a coincidence: both are the same 100 kΩ log potentiometer (section 3.2),
both are marked 10 dB per dot, and somebody laid the dots out once from the pot's actual law and reused the
drawing, mirroring the numbers. So a faceplate only needs one scale curve, drawn twice.

Second, **the spacing is irregular and not monotone**. Between GAIN's dots the slope runs 82.6, 41.7, 61.0 and
30.3 dB per unit of knob travel, which no single smooth pot law produces. Either my angles carry a couple of
degrees of error, or the silkscreen is simply approximate. Softube, who have had the hardware and the designer,
say it is the latter, and it is worth quoting in full because it is the most honest sentence any plug-in maker
has written about a panel: "Why don't the read-outs and panel print match? We chose to be faithful to the
original unit, the markings on the panel match the original hardware, and the knob positions match the original
hardware. But **the panel print isn't very exact**. For example, the knob extend further than what the start and
end points on the panel indicates, and **the actual numbers on the panel are very approximate**." [15]

So: draw the dots where I measured them, because that is where they are on the metal, and fit a smooth monotone
law for the parameter (section 9.4), because that is what the pot is actually doing.

### 2.3 The colour question, which does not resolve cleanly

Tube-Tech publish the answer in their own FAQ: "The code for the TUBE-TECH blue colour is: **RAL 5001**." [11]
RAL 5001 is "Green blue", and the swatch I sampled renders at about **#0E4C64** — a dark, muted, slightly green
teal. [45]

Their own photographs are nowhere near it. I sampled the blue in three Lydkraft press images:

| Image | Median blue |
|---|---|
| `CL1BFront.jpg`, front, hi-res [6] | **#005EB6** |
| `cl1bny2.jpg`, perspective [8] | **#006EAC** |
| `CL-1B-Rear.jpg`, rear, hi-res [7] | **#0046A6** |

All three are far brighter and far bluer than RAL 5001, and they disagree with each other by more than the
difference between two adjacent RAL blues. The front and rear images are CMYK press files, which explains part
of it; direct flash on gloss powder coat explains more.

**What I would do for the faceplate:** take the manufacturer's stated RAL 5001 as the base and lighten it
towards the photographs, landing somewhere around **#0F5A8C**, and say in a comment that this is a compromise
between a stated paint code and four photographs that contradict it and each other. Do not treat #005EB6 as the
colour of a CL 1B; it is the colour of a CL 1B under a photographer's strobe in a brochure. **Estimate.**

Other colours, sampled from the same photograph [6] and labelled as photographed rather than as painted:

| Element | As photographed | Note |
|---|---|---|
| Knobs | ≈ #151D1A | Black gloss; the greenish cast is the blue panel reflecting in them |
| VU face | ≈ #F5DDA9 | Warm cream; a fair part of that warmth is the meter lamp behind it |
| VU red band and "1 2 3" | saturated red, clipped to ≈ #FE5C3C in the file | Orange-red |
| Pilot jewel | clipped to ≈ #FE1235 | Blown out by the flash; a real ruby jewel is deeper |
| Silkscreen | white | |

### 2.4 The rear panel

Read from Lydkraft's hi-res rear photograph [7], saved as `ref/cl1b-rear-hires.jpg`, and confirmed against the
interconnection schematic [9]. Left to right as you look at the back:

- **The mains transformer**, standing proud of the chassis and painted the same blue, with its winding table
  silkscreened on the cover. The rear photo's cover reads **TR234/3**; the 1993 interconnection sheet names the
  part **TR234/1**. [7] [9]
- **CE** mark and **CSA LR 109998 NRTL/C**.
- Fuse legend on two lines: **T 0,1A/250 V (230 V~)** over **T 0,2A/250 V (115 V~)**, then the rotary voltage
  selector and the fuse holder, then a standard **IEC** inlet.
- **CAUTION: RISK OF FIRE / REPLACE FUSES AS MARKED**, and **115/230 V~ 22W (50 - 60 Hz)**.
- **SIDECHAIN**: two ¼ inch **TRS** jack sockets stacked vertically with a "not equal" glyph between them. They
  are wired in parallel and both are input *and* output; tip is bus 1, ring is bus 2. [2]
- **OUTPUT**: a male XLR.
- The **LYDKRAFT** identification plate, with **TYPE** and **SERIAL No** fields.
- **INPUT**: a female XLR.

There is no rear-panel trimmer, no pad, no sidechain filter and no link switch. Everything the user can reach is
on the front, which is one of the reasons the box is described as intuitive.

### 2.5 Control table

| Control | Hardware | Range and behaviour | Sources |
|---|---|---|---|
| **Gain** | P3, 100 kΩ **log**, a divider after the GRE | "used to 'make up' for the gain loss ... It is placed after the gain-reduction circuit and therefore **has no influence on the threshold setting**. The gain-control is continuously variable from off to +30 dB." The service manual pins the maximum at exactly +30.0 dB. | [2] [9] |
| **Ratio** | P2, 10 kΩ **linear**, a rheostat in series between the detector tap and the GRE | "varies the ratio by which the input signal is compressed. If the ratio selected is to 2:1, and the input signal increases 10 dB, the output signal is only increased by 5 db. ... continuously variable from 2:1 to 10:1." Section 3.4 shows what it is really doing. | [2] [9] |
| **Threshold** | P1, 100 kΩ **log**, tapping the divider ahead of the GRE and feeding the sidechain | "the point where the compressor begins its action. It is **defined as the point where the gain is reduced by 1 dB**." Range: the owner's manual and Softube say **+20 dBu to −40 dBu**; the specification sheet, the brochure and the web page say **off to −40 dBu**; the panel is marked off, 0, −10, −20, −30, −40. See 6.4. | [1] [2] [3] [15] |
| **Attack** | P4, 500 kΩ **log**, in the charge path of the timing capacitor | "chooses how fast/slow the compressor responds to an increase in the input signal ... continuously variable from 0.5 to 300 milliseconds." In Fix/Man it stops being an attack control and becomes a delay (5.3). | [2] [9] |
| **Release** | P5, 500 kΩ **linear**, in the discharge path | "continuously variable from 0,05 to 10 seconds." The linear taper is on the schematic and it matters (5.1). | [2] [9] |
| **Attack/release select** | SW1a and SW1b, three positions | **Fixed** (1 ms / 50 ms), **Manual** (the knobs), **Fix/man** (fixed attack, combined releases). | [2] |
| **Meter** | SW3a and SW3b, three positions, with R3 3.65 kΩ | **Input**, **Compression** (rest at 0 VU, reduction shown as a deflection to the left), **Output** (0 VU = +4 dBu). "Leave the meter switch in position compression as it might introduce distortion if left in the input or output position." | [2] [9] |
| **In** | a lever switch, "clickless" | Hard bypass of the whole compressor. Lydkraft advertise it as a "Clickless In/Out switch". | [1] [2] |
| **Sidechain bus select** | SW2, three positions, with R4 100 Ω | Off, 1, 2. Up to **ten** units share a bus over a standard ¼ inch TRS cable, tip bus 1, ring bus 2. "The interconnection implies, that the unit which performs the most compression is controlling the others." | [2] [9] |
| **Off / On** | SW1 on the mains, a rotary | Power. Lights B1, the ruby pilot, and B2, the meter lamp; both 12 V / 0.1 A. | [9] |

### 2.6 The clock-position problem

Lydkraft's suggested settings, and every user report, are given in clock positions. The knobs' physical stops
are at **8 o'clock and 4 o'clock** (section 2.2), so the honest mapping from an hour `h` to a knob fraction is

```
p(h) = ((h - 8) mod 12) / 8          →  8:00 = 0.000   9:00 = 0.125  10:00 = 0.250  11:00 = 0.375
                                        12:00 = 0.500   1:00 = 0.625   2:00 = 0.750   3:00 = 0.875   4:00 = 1.000
```

That maps every hour Lydkraft use except one. Their heaviest setting calls for the attack at **7 o'clock**,
which is past the counter-clockwise stop. I read it as their shorthand for "fully counter-clockwise", i.e.
`p = 0`, which is consistent with the surrounding text ("the attack control has reached the full CCW position")
and with the knob being marked **fast** there. [2] Getting this wrong shifts every published setting by one
hour, which is 12.5 % of the travel, so it is worth stating.

### 2.7 Lydkraft's own starting points

From the owner's manual's "Suggested applications" page, converted to knob fractions with the mapping above and
to times with the tapers of section 9.4. The dB figures are Lydkraft's; the times in the last two columns are
**derived** from the published ranges and the schematic's pot tapers.

| Application | GR wanted | Select | Attack | Release | Ratio |
|---|---|---|---|---|---|
| Final mix | **3-4 dB** | Fix/man | 2 o'clock (p 0.750, ≈ 61 ms) | 10 o'clock (p 0.250, ≈ 2.5 s) | 9 o'clock (p 0.125) |
| Bass, piano, guitar, keyboards, **vocals** | **4-5 dB** | Manual | 2 o'clock (p 0.750, ≈ 61 ms) | 10 o'clock (p 0.250, ≈ 2.5 s) | 10-2 o'clock (p 0.250-0.750) |
| Line guitar and piano, heavy | **10 dB** | Manual | 7 o'clock (p 0.000, 0.5 ms) | 1 o'clock (p 0.625, ≈ 6.3 s) | 3 o'clock (p 0.875) |
| Snare and bass drum | **2-3 dB** | Fixed | — | — | 9-12 o'clock (p 0.125-0.500) |

Two things are worth noticing. The vocal setting, the one the box is famous for, is a **slow** attack of about
60 ms and a **very** long release of about 2.5 s, at a ratio barely off its minimum. And Lydkraft only reach for
Fixed on drums, and only for two or three decibels. Bonedo agree from the other side: "Drums und sehr schnelle
Transienten eher nicht, wenn dann nur für die letzten 2-3 dB RMS." [20]

Scotty Simpson's bass setting, from a third party rather than the manual, sits in the same territory: attack
around 11 o'clock (p 0.375, ≈ 5.5 ms), release around 1 o'clock (p 0.625, ≈ 6.3 s), ratio near 2.5:1. [24]

### 2.8 What the plug-ins put on the panel

| Product | Controls | Additions the hardware does not have | Sources |
|---|---|---|---|
| TC Electronic CL 1B for PowerCore / TDM (2007-08) | Gain, Ratio, Threshold, Attack, Release, Attack/Release select, Meter select, Sidechain/Link | Sidechain/Link "reproduces the dynamic master/slave relationship between the left and right channels"; 28 presets | [14] |
| Softube CL 1B Mk I (native, 2009) | The same, with Sidechain Bus Select recast as **Internal / External** | External sidechain from the host; automatic stereo linking | [15] |
| Softube CL 1B Mk II (2018) | The Mk I set plus a **Generation Switch** (Mk I / Mk II voicing) | **Sidechain Low Cut** ("the main signal path is not filtered") and **Parallel compression** | [15] [16] [17] |
| Universal Audio Tube-Tech CL 1B MkII (UAD) | Softube's Mk II model under licence, "fully endorsed by Tube-Tech, Denmark" | Parallel Compression, Sidechain Low Cut, Generation Switch | [18] |
| Warm Audio WA-1B (hardware, 2U) | The full CL 1B set including the two busses | Front-panel meter calibration trim; 41-position detented pots | [19] [40] |
| Stam Audio SA-1B / SA-1B5 (hardware) | "meticulously reverse engineered replica" | **Dry/Wet blend**, a **5-position sidechain HPF**, bypass, stereo link | [41] |
| Kiive Audio KC1 (software, unlicensed) | Not named as a CL 1B on the page: "Inspired by a legendary studio staple" | Not documented; the page carries no specification table | [42] |

The pattern is unmissable: **everybody who is allowed to add controls adds the same two**, a wet/dry blend and a
sidechain high-pass. Softube added both in the Mk II, Universal Audio ship the same two, and Stam Audio put them
on the metal. The tribute should add exactly those two and no more, because that is the consensus of everybody
who has actually lived with the thing.

---
## 3. Signal path and circuit behaviour

Everything in this section is read from the **Lydkraft service manual** [9], which contains the complete
four-sheet drawing set and which I fetched, rendered at 300 dpi and read sheet by sheet. Unlike the Distressor
([[Distressor]] section 3.8), the CL 1B has a published schematic; unlike the LA-2A and LA-3A, one part of it is
a deliberate black box, and section 4 is about that part.

The drawings:

| Sheet | Title | Drawing | PCB | Date |
|---|---|---|---|---|
| 1 of 4 | TUBE-TECH COMPRESSOR CL 1B (interconnection) | TE130-40 rev 1.0 | — | 23 April 1993 |
| 2 of 4 | TUBE-TECH FRONT PCB | TE130-42 rev 1.0 | 870314-2 | 12 April 1993 |
| — | TUBE-TECH CL 1B, ME 1B, PE 1C AMPLIFIER | TE 100/41 rev 1.0 | 900621-2 | 12 April 1993 |
| 4 of 4 | TUBE-TECH CL 1B sidechain | TE130-43 rev 1.0 | 870316-2 | 12 April 1993 |

Note that the amplifier board is shared with the ME 1B and PE 1C equalisers, which is why Lydkraft can say the
CM 1A has "exactly the same tube circuit" [13] and why the CL 1B's audio path has nothing compressor-specific in
it at all: the compressor is the divider plus the sidechain, and the amplifier is Tube-Tech's house amplifier.

### 3.1 Block diagram in words

```
XLR IN ─► input transformer TR1 (TR230-2)
            │
            ▼
        R1 68.1 kΩ shunt ─► R2 100 kΩ series ──┬── node B ──[ RATIO P2, 0..10 kΩ ]── node C ──┬── GRE (shunt to 0 V)
                                                │                                              │
                                     THRESHOLD P1 100 kΩ log                            GAIN P3 100 kΩ log
                                                │  wiper                                       │  wiper
                                                ▼                                              ▼
                                          SIDECHAIN PCB                            ECC83 V1A voltage amp
                                                │                                       ECC83 V1B cathodyne
                                                │                                  ECC82 V2A/V2B push-pull
                                                │                                  output transformer TR2
                                                │                                              │
                                                │                                            XLR OUT
                                                ▼
   U1D ×2 buffer ─► precision full-wave rectifier (U1C, D1, D2) ─► U2A summer
        ─► U2B and C3 10 µF, the timing capacitor
              ├─ charge:    D3, R11 274 Ω, ATTACK P4 500 kΩ log
              └─ discharge: +15 V, P1 470 kΩ trim, R10 274 kΩ, RELEASE P5 500 kΩ lin, R9 47.5 kΩ, D4
        ─► U2C (MANUAL) and U2D (FIXED), diode-ORed by SW1 ─► CONTROL
        ─► R17, C8, D11, P2 ─► U1A servo ─► Q2 BF245A ─► Q1 BC337 ─► R15 100 Ω ─► the GRE's light source
        ─► U1B, P3, P4 ─► the VU meter in the Compression position
```

The one sentence version: **a feedback opto compressor whose detector tap sits one resistor upstream of the
optical cell, and whose ballistics are a single capacitor in an op-amp sidechain.**

### 3.2 The input transformer and the audio divider

The audio enters the amplifier board on TR1, a Lydkraft part numbered **TR230-2** for the CL 1B (the same
drawing lists TR200-1 for the ME 1B and PE 1C). Its secondary is loaded by **R2 1 MΩ** on the amplifier board
and leaves for the front board as "TO FRONT PCB". [9] The input impedance is specified as **600 Ω** and the
input and output are "balanced as well as fully floating" with "a static-screen between the primary and
secondary wirings". [1] [3]

On the front board, the signal meets a four-element network that is the whole compressor as far as the audio is
concerned [9]:

| Part | Value | Job |
|---|---|---|
| R1 | 68.1 kΩ | shunt at the transformer secondary, loading it |
| R2 | 100 kΩ | the series arm of the attenuator |
| P1 THRESHOLD | 100 kΩ log | shunt at node B, and its wiper feeds the sidechain |
| P2 RATIO | 10 kΩ linear, wired as a **rheostat** | in series between node B and node C |
| P3 GAIN | 100 kΩ log | shunt at node C, and its wiper feeds the tube amplifier |
| GRE | variable | shunt at node C, to 0 V |

So the attenuator is **exactly the LA-2A's arrangement**: a fixed series resistor, a photoresistive cell
shunting the node, and the make-up gain pot across the cell. That is why `opto::model::Divider` transfers to
this model unchanged as a *structure* (section 9.2), and it is why Sound On Sound describe the CL 1B family as
belonging to the small group in which "the optical attenuator forms part of the audio signal path", as against
"the majority of modern optical compressors, in which an optical element such as a light‑dependent resistor
(LDR) forms part of the side‑chain that controls a VCA‑based gain cell". [19] [72]

Because P3 is wired as a divider rather than a rheostat, it presents a constant 100 kΩ to node C whatever its
setting, which is precisely why the manual can promise that Gain "has no influence on the threshold setting".
[2] That is a small thing but it is a real design decision and the model should reproduce it: turning the
tribute's Gain knob must not change the gain reduction by so much as a tenth of a decibel.

### 3.3 The gain-reduction element in the circuit

On the sidechain sheet, the GRE is drawn as an **opaque rectangular module with eleven pins down each side**, of
which five are used [9]:

- one pin takes **TO GRE**, the audio node C;
- one takes **0V-1**, the audio ground;
- two on the other side are bracketed to **+15 V**;
- one on the other side goes to the **collector of Q1**, a BC337 NPN.

Q1's emitter runs to 0 V through **R15, 100 Ω** [55]. Its base is driven by **Q2, a BF245A JFET** source follower [56],
whose gate is driven by **U1A**, one quarter of an LF347 quad JFET op-amp [54], with **R16 10 kΩ** loading the base
node and the feedback taken from that node back to U1A's inverting input. That is a servoed voltage-to-current
converter: the loop forces the Q1 base node to follow the control voltage, so the current through the GRE's
light source is **(V_control − V_be) / 100 Ω**, with the JFET in the loop only to supply base current without
loading the op-amp's output through a diode drop. **R14, 100 Ω**, links the audio ground 0V-1 to the sidechain
ground 0V-2; it is a ground-lift resistor, not part of the divider.

That is everything the schematic says about the GRE, and it is all anybody outside Lydkraft has. Section 4 is
about what can and cannot be inferred from it.

### 3.4 The Ratio control is not a ratio control

This is the most interesting thing in the whole box and I want to be careful about it, so: **the topology below
is read directly off the schematic; the numbers below it are my own derivation from that topology, and are
labelled derived.**

The Ratio pot is a 0-10 kΩ rheostat sitting *between* the node the detector listens to (node B) and the node the
optical cell shunts (node C). [9] The consequence is that the detector cannot see all of the gain reduction:

- With the Ratio pot at **0 Ω** (fully counter-clockwise, marked 2:1), nodes B and C are the same node. The
  detector sees **exactly** the reduction the audio sees. The feedback loop is complete.
- With the Ratio pot at **10 kΩ** (fully clockwise, marked 10:1), node B can never fall below the divider formed
  by 100 kΩ and 10 kΩ, no matter how hard the cell clamps node C. The detector's view of the reduction
  **saturates**, the loop stops fighting back, and the audio reduction runs away.

I computed the divider exactly for a range of cell resistances, taking R2 = 100 kΩ, P1 = 100 kΩ, P3 = 100 kΩ and
a 20 MΩ dark cell (**derived**):

| Cell resistance | GR at node B / node C, ratio pot 0 Ω | at 2 kΩ | at 5 kΩ | at 10 kΩ |
|---|---|---|---|---|
| 300 kΩ | 0.90 / 0.90 dB | 0.87 / 0.92 | 0.82 / 0.96 | 0.75 / 1.01 |
| 30 kΩ | 6.48 / 6.48 | 6.05 / 6.59 | 5.49 / 6.76 | 4.73 / 7.03 |
| 10 kΩ | 12.72 / 12.72 | 11.34 / 12.90 | 9.76 / 13.14 | 7.91 / 13.52 |
| 3 kΩ | 21.65 / 21.65 | 17.49 / 21.86 | 13.89 / 22.15 | 10.49 / 22.59 |
| 1 kΩ | 30.70 / 30.70 | 21.49 / 30.92 | 16.02 / 31.23 | 11.62 / 31.69 |
| → 0 Ω | **unbounded** | **24.8 dB** | **17.4 dB** | **12.3 dB** |

The last row is the whole mechanism. The detector's apparent reduction ceilings at **12.3 dB** with the ratio
pot fully clockwise, at **17.4 dB** at half, and never at all when the pot is at zero.

**What that does to the ratio.** In a feedback compressor whose sidechain has an open-loop slope `s` decibels of
reduction per decibel of detected level, and whose detector sees a fraction `β = dGR_B/dGR_C` of the reduction,
the closed-loop ratio is

```
ratio = (1 + s·β) / (1 + s·β − s)     (derived; the standard feedback case [67] [72])
```

Two consequences, both testable and both audible:

1. **At the 2:1 end the ratio is exact and constant.** There `β = 1`, so `ratio = (1+s)/1 = 1+s`. A
   photoconductor whose resistance goes as the inverse first power of the drive gives `s = 1` in the
   deep-reduction asymptote, and therefore **exactly 2:1**. I solved the full loop numerically with that law and
   got a local ratio between 1.87:1 and 1.97:1 across a 60 dB input sweep — flat, and matching the manual's own
   worked example ("if the ratio selected is to 2:1, and the input signal increases 10 dB, the output signal is
   only increased by 5 db" [2]) to within the resolution of the arithmetic. **Derived**, and it is a pleasing
   result: Lydkraft could not offer a ratio below 2:1 because full feedback around a first-power photocell *is*
   2:1.
2. **At every other setting the ratio rises with depth.** `β` falls as the cell clamps, so the same numerical
   solve gives a local ratio near **2.4:1 at one decibel of reduction** with the knob fully clockwise, about
   **4:1 at ten decibels**, and **10:1 or more only past twenty**. **Derived.**

That second result is, I think, the single most important characteristic of the machine, and it explains three
things that are otherwise loose talk. It explains why Softube say the panel numbers are "very approximate" [15]:
the ratio marked on the panel is an asymptote, not an operating point. It explains Bonedo's "Man ist darüber
hinaus oftmals sehr verwundert, wie viel dB Gain-Reduction gerade stattfinden, ohne dass die Quellen dabei
gequetscht wirken" — one is often astonished how many decibels of reduction are happening without the source
sounding squashed. [20] And it explains why Sound On Sound can call 10:1 "effectively being limiting" [19] while
users report it as gentle: at three decibels it is not limiting at all.

**Caveats I am not going to hide.** The slope `s` is not on the schematic; it depends on the GRE's exponent,
which nobody has published (section 4.2). The numerical solve above assumed a first-power law and a linear
detector, both of which are reasonable and neither of which is documented. And the loop's actual gain is set by
the Threshold pot, which moves the whole family of curves along the input axis. So take the *shape* — flat 2:1
at one end, a knee that steepens with depth at the other — as the finding, and the exact decibel figures as
starting values to tune against section 10.

### 3.5 The Threshold control and where the detector listens

P1, 100 kΩ log, sits from node B to ground and its wiper is the entire input to the sidechain. [9] So Threshold
is a plain sidechain gain control, exactly as Peak Reduction is on an LA-2A, and the panel's dBu markings are a
calibration of that gain against the level at which one decibel of reduction happens. The manual is unusually
precise about the definition: "The threshold is the point where the compressor begins its action. It is defined
as the point where **the gain is reduced by 1 dB**." [2]

There is no coupling capacitor, no high-pass and no shelf anywhere between node B and the rectifier. The only
frequency shaping in the sidechain is two 100 pF capacitors: **C1** across U1D's 47.5 kΩ feedback resistor,
which rolls off above about **33.5 kHz**, and **C2** across U2A's 20 kΩ feedback resistor, which rolls off above
about **80 kHz**. [9] Both are above the audio band and both are there to keep the op-amps stable, not to shape
anything.

**The CL 1B's detector is therefore flat.** That is a real, structural difference from both of the optical
models already in the lab: the LA-2A has R37 shaping its sidechain, and the LA-3A is deliberately deaf below
100 Hz and can be tilted by ten decibels at 15 kHz ([[LA-3A]] sections 3.5 and 4.5). The CL 1B has none of that,
and section 10's test 30 asserts the difference on the same input.

That flatness has a consequence people describe without naming it. Bonedo, on full-range material: "Von der
frequenzselektiven Arbeitsweise der Opto-Kompression profitieren gerade 'Full-Range'-Signale, wie beispielsweise
Bässe: Unten herum wird gut kontrolliert komprimiert, während obenrum die 'Schnelligkeit' weitestgehend erhalten
bleibt." [20] A flat detector with a very long release does control the bottom end firmly, and the top end
survives because the reduction is not moving fast enough to modulate it.

### 3.6 The detector

Read from sheet 4 of 4 [9]. Both quad op-amps are **LF347N** [54], JFET-input types chosen, the manual says, because
"they do not affect the sound reproduction, second they have a high slew rate, which is of importance for the
performance of the compressor and third they don't take up much room". [2]

1. **U1D**, a non-inverting amplifier: R1 47.5 kΩ into the non-inverting input, R2 47.5 kΩ from ground to the
   inverting input, R3 47.5 kΩ feedback with C1 100 pF across it. Gain **×2**, rolling off above 33.5 kHz.
2. **U1C plus D1 and D2 (1N4148)**, a precision half-wave rectifier, with R4 10 kΩ as its input resistor and
   R5 10 kΩ as its feedback.
3. **U2A**, an inverting summer with R6 20 kΩ from the buffered signal, R7 10 kΩ from the rectifier, R8 20 kΩ
   feedback and C2 100 pF across it. The 20 : 10 : 10 : 20 ratio is the textbook precision **full-wave
   rectifier** [73]: the direct path contributes −1 of the signal, the rectified path −2 of the half-wave, and the sum
   is the absolute value.
4. **R22, 10 MΩ from +15 V** into U2A's summing node. That is a deliberate 1.5 µA bias, worth about 30 mV at the
   output, which keeps the following stage sitting just below conduction when there is no signal.

So the CL 1B's detector is a **mean-absolute (average-responding) detector of the instantaneous waveform**, flat
across the audio band, with no peak hold and no RMS: the first of the three canonical detector types [68]. This matters for the model: a sine and a square of the same
peak level will not produce the same reduction, and neither will a sine and pink noise. The 100 pF capacitors
are the only anti-aliasing there is.

### 3.7 The three time-constant circuits

This is where the CL 1B stops being an LA-2A. The manual states the architecture plainly: "The compressor
contains two time constants circuits: 1. Fixed attack and release times. 2. Variable attack and release times.
The **attack/release select** switch makes it possible to use these two circuits separately or combine their
functions." [2]

On the schematic [9], the storage node is **C3, 10 µF / 50 V**, sitting at U2B's inverting input. Everything
else is a path into or out of it:

- **U2B** compares the rectified detector output (its non-inverting input) against the stored voltage (its
  inverting input, which is the C3 node) and drives its output accordingly.
- **The charge path**, i.e. attack: U2B's output → **D3 (1N4148)** → **R11, 274 Ω** → the `ATTACK-1` pin → the
  front panel's **ATTACK pot P4, 500 kΩ log** → the `ATTACK-2` pin, which is the C3 node.
- **The discharge path**, i.e. release: **+15 V** → **P1, a 470 kΩ trimmer** (the service manual's "Rel. 10
  Sec." adjustment) → **R10, 274 kΩ** → the `RELEASE-1` pin → the front panel's **RELEASE pot P5, 500 kΩ
  linear** → the `RELEASE-2` pin → **R9, 47.5 kΩ** → U2B, with **D4 (1N4148)** steering into the C3 node.
- **U2C** and **U2D** are two more LF347 sections, each with a **47.5 kΩ** feedback resistor bypassed by a diode
  (D7 and D5) and each feeding the switch through an output diode (D8 and D6). U2C's output is the `MANUAL`
  line; U2D's is the `FIXED` line. SW1a and SW1b select which of them reaches the `CONTROL` line, and in the
  middle position both do, so the diodes make the result a whichever-is-larger combination.

Two derivations follow, and both are load-bearing for the model.

**The release is close to a constant-slope ramp, not an exponential.** The discharge path is a resistive
pull-up from a **fixed +15 V supply** into the capacitor, not a resistor to the node the capacitor is heading
for. While the stored voltage is small compared with 15 V, the current through it barely changes, so the control
voltage falls at an almost constant volts-per-second. That is a genuinely different shape from the LA-2A's
exponential-with-a-tail and it is consistent with how people describe the CL 1B's release: long, even, and
unhurried rather than fast-then-lingering. **Derived**, and it is the derivation I would most like somebody to
check on real hardware.

**The release taper is linear in knob rotation.** P5 is a **500 kΩ linear** pot in a resistive path, so the
total resistance, and therefore the time, is affine in rotation: `t(p) = t_min + p·(t_max − t_min)`. With the
published 0.05 s to 10 s that gives about **2.5 seconds at 10 o'clock**, which is the setting Lydkraft recommend
for vocals and for the mix bus (section 2.7). A log taper would have given about 350 ms there — seven times
faster and a completely different machine. This single component value changes the character of every published
setting, and it is why I read the pot codes off the schematic rather than assuming. **Derived from P5's stated
taper and the published range.**

By contrast **P4, the attack pot, is 500 kΩ log**, so the attack is logarithmic in rotation: `t(p) = t_min ·
(t_max/t_min)^p`, giving about **61 ms at 2 o'clock**, which is the vocal setting. Again: slow.

**One thing I cannot make come out.** With C3 at 10 µF and R11 at 274 Ω, the fastest attack the charge path can
produce as a simple RC is 2.7 ms, not the published 0.5 ms, and 300 ms would want about 30 kΩ rather than the
pot's full 500 kΩ. Either the published times are settling times of the whole loop rather than time constants of
this node, or the op-amp's ability to drive the node hard changes the arithmetic, or I have misread a component
value on a thirty-year-old scan. I have not resolved it, I am not going to pretend I have, and the model is
therefore calibrated to the **published times**, which is what section 10 asserts. The schematic is used for the
*shapes* — log attack, linear release, ramp-like discharge — and the manual for the *numbers*.

### 3.8 The control amplifier and the meter drive

The `CONTROL` line returns from the switch through **R17, 20 kΩ**, to a node carrying **C8, 1 nF** and **D11, a
1N4002**, then through **P2, 100 kΩ**, into U1A, which drives the GRE through Q2 and Q1 as described in 3.3. P2
is the service manual's "compression tracking" trimmer: the procedure sets it so that **+250.0 mV DC injected at
the sidechain jack produces exactly −10.0 dB** of gain reduction. [2] That is the single most useful published
number in the whole document set, because it pins the sidechain-voltage-to-gain-reduction map at one exact point
with no interpretation required.

The meter's Compression reading is generated from the same control voltage, not from the audio: **R18 2.74 kΩ**
and **R19 15.4 kΩ** set the scaling, **P3, 100 kΩ**, is the trimmer, **U1B** with **D13**, **R20 47.5 kΩ** and
**D14** shapes it, and **R21 4.75 kΩ** feeds the `+VU` line; **D12**, a 5.6 V zener, and **P4, 500 kΩ**, set the
`−VU` reference. [9] So in Compression the needle is showing what the sidechain thinks it is doing, calibrated
by trimmers, not a measurement of the attenuator. The service manual's tolerance: "The VU-meter accuracy should
be within ±0,5 dB when reading compression." [2]

### 3.9 The tube amplifier and the transformers

From the amplifier sheet, drawing TE 100/41, PCB 900621-2 [9], and confirmed by Bonedo, who opened one: "Die
Röhrenstufe besteht aus genau zwei Röhren, wobei die erste für Vorverstärkung und Phasenteilung zuständig ist
(ECC 83) und die andere die Ausgangsverstärkung übernimmt (ECC 82)." [20]

| Stage | Parts |
|---|---|
| **V1A, ECC83** [57] | voltage amplifier. R3 200 kΩ plate load, R4 2.26 kΩ cathode bypassed by C1 10 µF, P1 470 Ω preset in the cathode (the "preset GAIN" of the service procedure), R26 47.5 kΩ, R2 1 MΩ grid leak |
| **C2** | 22 nF / 400 V coupling to the splitter |
| **V1B, ECC83** | **cathodyne phase splitter**: R6 100 kΩ plate, R8 100 kΩ cathode, R7 2.61 kΩ, R5 1 MΩ grid leak |
| **C3, C6** | 0.33 µF / 250 V, coupling the two phases out |
| **V2A, V2B, ECC82** [58] | **push-pull output**, R11 and R12 1 MΩ grid leaks, R13 and R14 4.7 kΩ 2 W cathode resistors each bypassed by 47 µF / 63 V |
| **TR2** | output transformer **TR242/1**, centre-tapped primary fed from +270 V, with R15 1 kΩ and C9 3.3 nF / 400 V across the secondary as a snubber |

The power supply is worth a line because Lydkraft advertise it: 270 V AC through F1 (T100 mA) and a B380C800
bridge into a three-section RC filter (1 kΩ / 100 µF, 1.5 kΩ / 100 µF, 2.2 kΩ / 100 µF) for the output stage's
+270 V, then a discrete series regulator (T1 BUX85, T2 and T3 BUX87, D3 a 9.1 V zener, C13 2.2 µF / 385 V)
producing a **stabilised +240 V for the preamplifier and phase splitter**; separately, 15 V AC through F2
(T630 mA) and a B40C800 bridge into an **LM7812**, giving the two valves **stabilised 12 V DC heaters**. [9] The
manual states both facts in words. [2]

**What that means for distortion.** The single-ended V1A stage generates second harmonic; the push-pull ECC82
output stage cancels even harmonics and leaves odd; the two transformers add their own low-frequency
nonlinearity when the core is worked. That is exactly why the specification quotes THD+N **at 40 Hz** and why it
is the *same* 0.15 % at 0 dBu and at +10 dBu [3] — a figure that flat with level is a transformer figure, not a
tube figure. The tribute's distortion model needs a second-harmonic term, a symmetric soft ceiling, and a
low-frequency term that dominates at 40 Hz. Section 9.5 says how.

### 3.10 Metering

Three positions, SW3a and SW3b, with R3 3.65 kΩ setting the sensitivity [9]:

- **Input**: the level at the input socket.
- **Compression**: "The VU-meter is reading gain reduction. Its rest position is '0 VU', and the amount of
  compression is shown as a decreasing deflection in dB." [2]
- **Output**: "'0 VU' is equivalent to +4 dBU." [2]

And a warning that tells you the meter is bridged across the audio in two of the three positions: "**Leave the
meter switch in position compression as it might introduce distortion if left in the input or output
position.**" [2] That is a lovely, slightly alarming detail and the tribute should keep it as a joke rather than
as behaviour.

Softube's calibration, which is also the reference the rest of this lab already uses: "The meter and the plugin
are calibrated so that a sine wave showing 0 VU at the output corresponds to a −18 dBFS output signal.
Correspondingly, a 18 dBFS sine at the input will show 0 VU if the meter is set at showing the input signal."
[15]

### 3.11 The bus system

Two ¼ inch TRS sockets on the rear, wired in parallel, both input and output, tip for bus 1 and ring for bus 2;
the front switch selects off, 1 or 2. Up to **ten** units can share a bus. "The interconnection implies, that
the unit which performs the most compression is controlling the others." And the practical warning: "Remember to
set the ratio control and the gain control in the same position on the 'slaves'. Otherwise the stereo image
could be shifted during compression. The attack/release-control on the slaves will have no effect." [2]

Sound On Sound could not find a use for it — "I can't think of a recording situation that I've ever been in
where I would have needed to chain multiples of mono compressors together" [19] — and Softube quietly replaced
it with an Internal / External sidechain selector and automatic stereo linking [15]. The tribute should do what
Softube did, because a plug-in has hosts and buses of its own, and keep the three-position switch on the face
because it is part of the picture.

### 3.12 What the schematic does not show

One thing, and it is the important thing: the inside of the GRE. That is section 4.

---

## 4. The optical element, and why it is not a T4

The brief for this dossier asks specifically what the CL 1B's optical cell can and cannot borrow from the T4
model in `src/dsp/opto/model.rs`. This section is the answer, and it is the section I would ask a reviewer to
read first.

### 4.1 What the lab already has

`opto::model::Cell` is a physical model of a **T4**, built for the LA-2A and reused unchanged by the LA-3A
([[LA-2A]] section 7.2, [[LA-3A]] section 7.1). It has three states and models three things:

| State | What it is | Governed by |
|---|---|---|
| `u` | the smoothed drive on the **electroluminescent panel** | `tau_u` |
| `n_f` | **free carriers** in the CdS photoconductor, which is the conductance | `tau_f0`, `l_a`, `tau_r1`, `k_gen` |
| `n_t` | **trapped carriers**, which is the memory | `capture`, `tau_t0`, `k_m` |

`Cell::light_for` is the Alfrey-Taylor electroluminescent law, `exp(−b/√u)`, with `EL_B = 5.0`.
`Cell::carriers_for` is the photoconductive law, conductance proportional to light to the power `CELL_GAMMA =
0.8`. And `Cell::step` integrates all three states, producing the LA-2A's 10 ms attack, its 60 ms first-stage
release, its 0.5 to 5 s second stage and its programme-dependent memory **out of the physics**, without a single
time constant being set by a user control.

That is the correct model of a T4 and it is the wrong model of a GRE.

### 4.2 What the GRE is, as far as anybody outside Lydkraft knows

**Established.** It is a passive optical attenuator in the audio path, driven directly by the sidechain's control
current, in the LA-2A tradition rather than the LDR-controls-a-VCA tradition. [19] It is a discrete potted module
with two audio pins and a light source fed from +15 V through a transistor current sink. [9] It is graded and
marked: the service manual's adjustment procedure requires that "For serial no. up to 15891 **THE GRE SHALL BE
MARKED BETWEEN 1.225-1.285**. For serial no. 15892 and onwards **THE GRE SHALL BE MARKED BETWEEN W1.20-
W1.50**." [2] Lydkraft claim it "in itself has a very low harmonic distortion and none of the non-linearity
problems involved when using most semiconductor elements. Furthermore **there is no long-term degradation of the
element thus giving it almost infinite life**." [2]

**Not established: what is inside it.** The one Gearspace thread that asks the question directly gets no
definitive answer in twenty years. The best reply explains the category and then concedes: "The T4B used in the
beloved LA2A is a non semiconductor opto element, built from an electroluminescent panel and two photocells. The
Vac Rac uses VACTROLS which ARE semiconductor opto elements since they are built around a diode and a photocell,
and readily available as electronic parts. **I'm not sure about what's inside the CL1B but, probably they
developed their own opto element.**" [36] A builder attempting a clone puts the same point from the schematic's
side: "The schematics for the cl1b can be found if you search the internet for a little while. **The only thing
this schematic does not reveal is the proprietary opto cell circuit and parts.**" [37]

**What the wording does rule out.** Lydkraft's phrase is "non-semiconductor element". An LED is a semiconductor.
An electroluminescent panel is not, and neither is an incandescent lamp. So their own description argues against
an LED-plus-LDR module and towards something in the T4 family, though nobody has published a teardown that
confirms it and I am not going to state it as fact. Note that Bonedo's review describes opto compression in
general as "im Allgemeinen eine Leuchtdiode mit angeschlossenem Lichtsensor" [20], an LED with a light sensor —
that is the reviewer explaining the category, not a claim about this box, and it should not be cited as
evidence of an LED.

**On ageing.** There is no public report of a CL 1B's GRE failing. That is weak evidence, not strong: an absence
of forum threads is not a measurement, and the place such a thread would most likely live could not be searched.
What owners do report as normal maintenance is **valve** swapping, not GRE work. Two things follow for the
model. There should be no cell-wear parameter, unlike the LA-2A's `opto_cell` and the LA-3A's `la3a_cell`,
because the manufacturer explicitly claims no degradation and nobody has contradicted them. And unit-to-unit
consistency should be high, which is what owners say: "Every one is virtually the same. Built like a tank and
quality sound" — against the LA-2A, of which the same poster says "no two sound alike, well maybe every third
one sound like another, but they vary a lot". [29]

### 4.3 The decisive difference: where the time constants live

This is the whole section in one table.

| | LA-2A / LA-3A | CL 1B |
|---|---|---|
| What sets the attack | the photoconductor's carrier generation rate | **an RC charge path with a 500 kΩ log pot in it** [9] |
| What sets the release | free-carrier recombination, then trap emptying | **an RC discharge path with a 500 kΩ linear pot in it** [9] |
| Programme dependence | emerges from trap occupancy; **not switchable** | a **switch position**, Fix/man, that ORs two release circuits [2] |
| Range of release | 60 ms to 50 %, then 0.5-5 s. Fixed. | **0.05 s to 10 s, by knob** [2] [3] |
| Range of attack | about 10 ms. Fixed. | **0.5 ms to 300 ms, by knob**, or 1 ms fixed [2] [3] |
| What the cell must do | be the compressor | get out of the way |

A CL 1B set to a 300 ms attack and a 10 s release is doing something no T4 can do, and a CL 1B in Fixed mode is
doing something with a 50 ms release that a T4's trap memory would smear into seconds. If the tribute's CL 1B
imported `opto::model::Cell`, the T4's `tau_r1 = 60 ms` and `tau_t0 = 0.5 s` with `k_m = 12` would sit in series
with the electronic release and would dominate everywhere below about a second, so the Release knob's whole
lower half would do nothing. Users would notice within a minute, and so would test 27.

### 4.4 What to do instead

Model the GRE as what the circuit asks it to be: a **fast, memoryless, static map from control current to
resistance**, plus one small first-order lag for the physical cell's own response, which is what the published
photoconductor models reduce to when the emitter is fast and the drive is a current [59] [60] [63] [65]. That is:

```
R_gre = f(i_control)        static, monotone, a photoconductive power law
i_control = i_smoothed      one pole, tau_cell
```

with `tau_cell` small enough that it never becomes the dominant time constant at any knob setting. The
sidechain's own fastest published time is 0.5 ms, so `tau_cell` must be well under that; I take **0.2 ms** as an
**estimate** and section 10's test 14 checks that the fastest attack setting still reaches its published time.

What survives from the existing code is the **static** half of the cell model and the divider around it, not the
dynamic half. Section 9.2 lists the imports item by item.

---

## 5. Fixed, Manual and Fix/Man

### 5.1 The published times

From the owner's manual's attack/release select page, which is the only place all three are stated together [2]:

| Position | Attack | Release |
|---|---|---|
| **Fixed** | **1 ms** | **50 ms** |
| **Manual** | **0.5 ms to 300 ms** | **0.05 s to 10 s** |
| **Fix/man** | as Fixed, i.e. 1 ms | "combines the release times of fixed and manual mode" |

The specification sheet gives the manual ranges as "Attack 0,5 ms to 300 ms" and "Release: 50 ms to 10 s", which
is the same thing written differently. [3] Sound On Sound reproduce all of it for the WA-1B clone and are
scrupulous that it is inference, not measurement: "Although Warm Audio have not released attack and release
times, assuming that the WA‑1B is a faithful recreation of the original ... **it is probable** that these times
... are 1ms attack and 50ms release in Fixed mode; continuously variable 0.5 to 300 ms attack and 0.05 to 10
second release in Manual mode". [19]

**An ambiguity nobody resolves.** Lydkraft never say whether "1 ms" and "50 ms" are time constants, 63 % times,
10-90 % times or settling times. The one place the manual gets quantitative about a release, it uses a *full
recovery* time: in the release calibration, with the threshold set for 10 dB of reduction, attack fast and
release slow, "Switch off the 1 kHz and observe that the VU meter moves to 0 VU in approx. **10 sec.**" [2] So
at least the 10 s end of the range is a full recovery from 10 dB, not a time constant. I model it that way, and
section 10's tests assert the published quantity under the published conditions rather than a time constant I
would have had to invent.

### 5.2 Fix/man, in the manual's own words

The manual describes this at more length than anything else in the document, which tells you Lydkraft thought it
was the interesting part [2]:

> The fix/man mode always has a fast attack, but it is possible to obtain a release time depending on the input
> signal, e.g. get a fast release when the peak disappears, then superseded shortly thereafter by the release
> time selected by the release control.
>
> From the time the peak disappears, until the selected release time takes over, is dependent upon the setting
> of the attack control. **That is, the attack control changes function from a pure attack control, to a control
> of delay with the same time range.**
>
> The more CW the attack control is turned, the longer time before the release control takes over. The more CCW
> the attack control is turned, the shorter time before the release control takes over.
>
> This function is valid only if the time of the peak is shorter than the setting of the attack control. If the
> peak of the program is longer than the setting of the attack control, or if the attack control has reached the
> full CCW position, it will respond as in the manual mode.
>
> The fix/man mode acts as an automatic release function with a constant fast attack time and fast release time
> for short peaks and a longer release times for longer peaks. This setting is mainly intended for use on
> program material (overall compression).

That is a complete specification of a state machine, and section 9.5 implements it literally. Note the two
clauses that make it more than a dual release: the delay is *set by the attack knob over the same 0.5 to 300 ms
range*, and the whole feature *switches itself off* if the peak outlasts the delay.

Sound On Sound's reviewer, meeting the idea for the first time on the clone, described the experience well:
"having the (new to me) ability to delay the onset of what I began to think of conceptually as the 'second
phase' of release really did offer precise and more creative control of the release envelope." [19]

### 5.3 What the circuit does, and where I stop

U2C and U2D each take the storage node through a 47.5 kΩ resistor bypassed by a diode and out through another
diode onto the `MANUAL` and `FIXED` lines; SW1a and SW1b connect one or both of those lines to `CONTROL`. [9]
With both connected, the output diodes make the control voltage the larger of the two, which is the right
mechanism for "fast release for short peaks, long release for long ones". Which lug of the three-position switch
is the common one, and therefore exactly which combination the middle position makes, I cannot resolve from the
scan. So I model the manual's *description*, which is unambiguous and detailed, rather than the diode network,
and I say so in the code. That is the same call the LA-3A model made about its Comp/Limit switch
([[LA-3A]] section 7.6) and for the same reason.

### 5.4 The attack knob's double life, and why it is a trap

In Manual the attack knob is an attack time. In Fix/man it is a **delay** before the slow release takes over,
over the same numeric range, while the attack itself is pinned at 1 ms. In Fixed it does nothing at all.

That is three behaviours on one control, and it is the thing most likely to be implemented as "attack time,
always" by somebody working from the specification sheet rather than the manual. Lydkraft's own suggested
setting for the mix bus uses Fix/man with the attack at 2 o'clock [2], which under the wrong reading would give
a 61 ms attack on a mix bus and would sound obviously wrong. Section 10's test 19 asserts the difference
explicitly.

---
## 6. Published measurements

### 6.1 The specification sheet

Reproduced in full from Lydkraft's own sheet [3], with the web page and brochure as cross-checks [1] [4]. All
figures are at RL = 600 Ω, and Lydkraft reserve the right to alter them without notice.

| | |
|---|---|
| Input impedance | 600 Ω |
| Output impedance | < 60 Ω |
| Frequency response, −3 dB | **5 Hz to 25 kHz** |
| THD+N at 40 Hz, 0 dBu | **0.15 %** |
| THD+N at 40 Hz, +10 dBu | **0.15 %** |
| Maximum output | **+26 dBu at < 1 %** |
| Maximum input | **+21 dBu at < 1 %** |
| Noise, Rg = 200 Ω, 22 Hz-22 kHz | < −85 dBu at 0 dB gain; **< −75 dBu at +30 dB gain** |
| Noise, CCIR-468-4 | < −75 dBu at 0 dB gain; < −65 dBu at +30 dB gain |
| Crosstalk at 10 kHz | < −60 dB |
| CMRR at 10 kHz | > 60 dB |
| Gain | **off to +30 dB** |
| Ratio | **2:1 to 10:1** |
| Threshold | **off to −40 dBu** |
| Attack | **0.5 ms to 300 ms** |
| Release | **50 ms to 10 s** |
| Valves | ECC82 ×1, ECC83 ×1 |
| Dimensions | 132 mm (5.2 in) × 483 mm (19.0 in) × 170 mm (6.7 in) |
| Weight | 4.1 kg net, 5.9 kg shipping |
| Power | 115/230 V, 50-60 Hz, **22 W** |

### 6.2 The calibration points, which are better than the specifications

The service manual's adjustment procedure gives four exact, reproducible operating points, and for modelling
purposes they are worth more than the whole specification sheet, because each of them pins a specific transfer
function at a specific value [2].

1. **Maximum gain is exactly +30.0 dB.** "Apply a signal of 1 kHz, −30,0 dBU into the input. Turn the GAIN
   control fully clockwise. Set the RATIO control at 2:1. Adjust the preset GAIN (located on amp/psu PCB) to an
   output-reading of 0,0 dBU."
2. **+250.0 mV DC at the sidechain jack gives exactly −10.0 dB of gain reduction.** With the threshold fully
   counter-clockwise, the ratio at 2:1, the bus at 1, a 1 kHz 0.0 dBu input and the gain set for 0.0 dBu out:
   "Apply a DC-voltage of +250,0 mV into the side chain jack socket (tip) and observe that the output level has
   dropped to −10,0 dB."
3. **The meter reads compression to ±0.5 dB.** Trim for 0 VU with no reduction, then for −10.0 VU with the
   250 mV applied. "The VU-meter accuracy should be within +/− 0,5 dB when reading compression."
4. **The slowest release recovers from 10 dB in about 10 seconds.** Meter to compression, select to manual,
   1 kHz at 0.0 dBu, threshold adjusted for a −10 VU reading, attack fast, release slow: "Switch off the 1 kHz
   and observe that the VU meter moves to 0 VU in approx. 10 sec."

Plus two housekeeping figures: the unit must warm up for **at least 15 minutes** before adjustment, and the DC
offset at the sidechain jack with the threshold off must not exceed **±15 mV** in either the fixed or the manual
position. [2]

### 6.3 What has never been measured

Nobody outside Lydkraft has published a measurement of a CL 1B. Not a THD-versus-level curve, not a measured
frequency response, not a measured attack or release, not a static transfer curve, not a ratio curve. Audio
Science Review, the obvious venue, has nothing [76]. Sound On Sound have never reviewed the CL 1B or any of its
plug-ins — their only Tube-Tech reviews are of the MEC 1A, MMC 1A and MP 2A, and Lydkraft's own
review index links to a retailer's user reviews rather than to a magazine [50] — and the closest thing to a
technical treatment in English is their review of the Warm Audio clone, in which the timing figures are
explicitly labelled inference. [19]

The nearest anybody has come to empirical work is listening comparison with published files: Bonedo recorded a
large set of before-and-after examples through an RME UFX at 44.1 kHz / 24 bit, level-matched by ear before
dithering [20], and a Gearspace user posted a hardware-versus-Softube comparison with the raw files available
[35]; Lydkraft publish their own before-and-after set too [46]. Those are useful for judging character. They
are not measurements.

**Consequence for section 10.** Every test asserts a manufacturer figure or a service-manual calibration point,
and I name the figure and its source for each. Where no published figure exists I say so and do not propose a
bound, because a loose bound that the model trivially satisfies is worse than no test: it looks like evidence
and is not. The quantities with **no published figure at all** are:

- the **maximum gain reduction**, which nothing states;
- the **knee shape** and the static curve at any setting;
- the **ratio at a given depth**, as against the two end labels;
- the **attack and release definitions** (63 %, 10-90 % or settling), except for the 10 s release, which the
  service manual gives as a full recovery;
- the **GRE's exponent, minimum resistance and dark resistance**;
- **THD at 1 kHz** — every distortion figure Lydkraft publish is at 40 Hz;
- the **distortion spectrum**, i.e. which harmonics and in what proportion;
- **unit-to-unit variation**, beyond owners' impression that there is very little of it.

### 6.4 Where the sources contradict each other

Six, and each of them matters to somebody:

1. **Threshold range.** Owner's manual and service manual page 3: "continuously variable from +20dBU to
   −40 dBU". Specification sheet, brochure and web page: "Threshold off to −40 dBU". Softube's manual repeats
   "+20 dB to −40 dB". The **panel** is marked off, 0, −10, −20, −30, −40, with no +20 anywhere. [1] [2] [3]
   [15] I take the panel as authoritative for the marked range and treat the counter-clockwise stop as "off",
   and I note that a +20 dBu threshold would be above the specified +21 dBu maximum input, so as a usable range
   it is close to meaningless anyway.
2. **Height.** The specification sheet says "Height: 2 units" on the same line as 132 mm and 5.2 inch, which are
   both 3U. Sound On Sound, Bonedo and my own measurement of the manufacturer's photograph all say 3U. [3] [19]
   [20] The words are wrong; the numbers are right.
3. **Frequency response.** The specification sheet's own table reads "Frequency response @ -3dB **5 Hz to
   25 Hz**". The brochure, the web page and the CM 1A sheet all say 25 kHz. [1] [3] [4] [13] A typographical
   error in the sheet.
4. **CMRR sign.** The specification sheet writes "CMMR @ 10 kHz **< − 60 dB**"; the web page and brochure say
   "CMRR: **> 60 dB** @ 10 kHz". [1] [3] [4] The web page is right and the sheet has both a transposed acronym
   and an inverted inequality.
5. **Weight.** Specification sheet: 4.1 kg net. Bonedo, who had one on the bench: "4,8 kg schweren
   Stahlblech-Kiste". [3] [20] Not important, but if a source says 4.8 kg it is not wrong, it is Bonedo.
6. **Ratio range in the TC Electronic plug-in.** TC's own specification page said 2:1 to 10:1; a magazine
   review of the same product said 1:1 to 10:1. Unresolved; the hardware is 2:1 to 10:1 and that is what
   matters. [14]

And one non-contradiction worth flagging because it looks like one: the manual says release is "0,05 sec to 10
sec" and the specification sheet says "50 ms to 10 s". Those are the same number.

---

## 7. Sound character, and what an emulation must get right

The CL 1B is not described the way the LA-2A is. Nobody calls it thick, or fat, or a "vibe box". The words that
recur are **clean, bright, modern, forgiving, and invisible**, and the recurring complaint is that it is
*undramatic*. Any emulation that sounds obviously compressed has already failed.

**On the character itself.**

- Bonedo, on the whole approach: "Er ist also eher **Skalpell als Axt**" — a scalpel rather than an axe — "man
  sollte also nicht verwundert sein, wenn der CL-1B auf den ersten 'Horch' nicht ganz so stark arbeitet, wie es
  die Regler vermuten lassen." [20]
- And the observation that matters most for the static curve: "Man ist darüber hinaus oftmals sehr verwundert,
  wie viel dB Gain-Reduction gerade stattfinden, ohne dass die Quellen dabei gequetscht wirken." [20] That is
  section 3.4's rising ratio, heard rather than derived.
- Attack Magazine: "the CL-1B is not noted as a character piece; it does warmth and it does smooth, and it does
  both transparently". [21]
- Sound On Sound, on the clone but describing the design: "a 'forgiving' compressor — one that's essentially
  incapable of delivering a bad performance ... even at stupidly high levels of compression on highly dynamic
  tracks"; and "although its valves and transformers do add a touch of vintage analogue warmth, [it] possesses a
  **speed and clarity** that can also add a more modern edge". [19]
- An owner of both, comparing directly: "**The CL1B has less distortion and a bit leaner/clearer in tone.** The
  LA2A is great on bass." [29]
- Another, in the same thread: "**The CL1B has this uncanny ability to help great singers sound better and bad
  singers to sound worse.** For tracking, it is best used with very subtle compression, just a few db max." [29]
- A blind clip comparison against two LA-2As, in which listeners picked it out without being told: "Sample A
  seems like the most versatile, and it stands out from B and C as being **brighter and cleaner**. I am guessing
  this is Tubetech." [33]
- And the dissent, which should be in the file too: "theres just not a ton of mojo with the Cl1b, **what it does
  is solidifies the bass and softens the highs**, but other compressors do that as well. It's a rather clean
  effect in my opinion." [34]

**On what it is used for.** Vocals, overwhelmingly, and then bass, guitar, keys and mix bus. "**CL1B = Clarity
and Brightness with lots of flexibility.**" [31] "La2a will be 'best' for 80% of vocalists if used right. CL1B
will be 'best' for the other 20%, if used right." [32] "The LA2a has the slight edge with male vocals IME ...
**The CL1b has a definite strong edge with femvox**." [30] Not drums: Lydkraft's own suggested setting for
snare and bass drum asks for 2-3 dB [2] and Bonedo says the same [20]. An owner on Audiofanzine puts the
range plainly: "it is a delicate work or 'muscled' on demand". [44]

**So what must an emulation get right to be recognisable?** Six things, in the order I would fix them.

1. **The ratio must grow with depth.** Section 3.4. At one decibel every ratio setting behaves nearly the same;
   at twenty they diverge enormously. This is the thing people are describing when they say it does not sound
   squashed. A model with a fixed ratio per knob position will be wrong everywhere and will sound wrong first.
2. **The release must be settable and long.** Lydkraft's own vocal setting is about 2.5 seconds because the pot
   is linear (section 3.7). If a model gives 350 ms there, every published setting is wrong.
3. **Fix/man must be a delay, not a blend.** Section 5.2. It is the feature the designer is proudest of and it
   is the only programme-dependent behaviour in the box.
4. **The detector must be flat.** No sidechain shaping (section 3.5). Adding a low-frequency roll-off "because
   opto compressors have one" imports the LA-3A's personality into a machine that does not have it, and is the
   single most likely accidental error given what is already in this repository.
5. **It must be clean.** 0.15 % THD+N at 40 Hz, at both 0 and +10 dBu [3], against the LA-2A's measured 0.9 % to
   4.2 %. If the tribute's CL 1B is dirtier than its LA-2A, it is not a CL 1B.
6. **The Gain knob must not touch the compression.** Section 3.2. It is a one-line property of the circuit and a
   one-line test, and getting it wrong makes the box feel like a channel strip.

And one thing an emulation must *not* do: give it a cell-wear control. Section 4.2.

---

## 8. Existing emulations

The CL 1B has been emulated far less than the LA-2A, for a reason a builder put plainly: "their IP has not
escaped into the world, and they are entitled to sell their product as long as they can." [37] Lydkraft licensed
one developer and everybody else has had to work around the trademark.

| Product | Year | Licensed | What it adds | Sources |
|---|---|---|---|---|
| **TC Electronic CL 1B for PowerCore**, later TDM | 2007, TDM Apr 2008 | Collaboration with Lydkraft and Softube | "based upon highly advanced component emulation technology"; Sidechain/Link; 28 presets. PowerCore development ceased 2011 | [14] |
| **Softube Tube-Tech CL 1B** (Mk I) | native from 2009 | Yes | External sidechain; automatic stereo link | [14] [15] |
| **Softube Tube-Tech CL 1B mk II** | 2018 | "Created in partnership with and endorsed by Tube-Tech" | **Generation Switch** (Mk I/Mk II), **Sidechain Low Cut**, **Parallel compression**. "component modeling" | [15] [16] [17] [51] [52] |
| **Universal Audio Tube-Tech CL 1B MkII** (UAD) | — | "fully endorsed by Tube-Tech, Denmark"; it is Softube's model, "reimagined with Softube's latest modeling and signal processing technology" | Parallel Compression, Sidechain Low Cut, Generation Switch | [18] |
| **Warm Audio WA-1B** (hardware) | 2023 [53] | No; "Scandinavian", "1B-style" | 2U instead of 3U; front-panel meter trim; 41-position detented pots; TRS as well as XLR | [19] [40] |
| **Stam Audio SA-1B / SA-1B5** (hardware) | — | No; "meticulously reverse engineered replica" | **Dry/Wet blend**, **5-position sidechain HPF**, bypass, stereo link | [41] |
| **Kiive Audio KC1** (software) | — | No; "Inspired by a legendary studio staple" | Not documented | [42] |

**Confirmed absent**, from enumerating whole catalogues rather than spot checks: Antelope Audio (199 products,
no CL 1B; their opto is an LA-2A model), Analog Obsession, Waves (424 products; "cl1b" appears only as a search
tag on their LA-2A and 1176 models), IK Multimedia, Arturia, Plugin Alliance and Brainworx, SKnote, Klanghelm,
Nembrini, United Plugins, Audified, Overloud, Pulsar. Acustica Audio cannot be strictly excluded because they
name products by colour codenames and do not identify the hardware they sample, but nothing on their site names
Tube-Tech.

**What the criticism says they get wrong.** Universal Audio's own review corpus is the only place with volume
criticism, and two themes are sonic rather than defect reports. The first is that the Mk II model changed the
character: "Compared to v1 this comp is WAY too aggressive. I don't know what went wrong but v1 is way more
smooth." The second is pumping at depth: "pumps badly anything over -3 db GR at any setting", corroborated by
"anymore than -3 dB GR and it starts to pump, I've tried all settings.. very sensitive to set up compared to
other compressors". [18] Both are consistent with a model whose ratio does not grow smoothly with depth, which
is section 3.4's point from the failure side. A Mk I-era complaint says the same thing differently: "The
behavior of this plug was erratic; i.e its goes from too subtle to being extreme ... I have not used the HW but
I can't imagine the HW would behave this way." [18]

**What the good comparison says they get right.** A hardware owner who posted files: "**Being a relatively clean
compressor the CL1B has been a fairly easy comp for softube to nail when using sensible amounts of GR, once you
get into the extremes I prefer the HW.** If you don't use extreme amounts of GR the plug-in is actually
amazingly close to the hardware IME." And: "There's kind of a lower midrange push with the Softube that isn't my
favorite thing." [35]

That is a very usable summary of the state of the art: **the easy part is the first six decibels and the hard
part is what happens after twenty.** Which is section 3.4 again.

---

## 9. Recommended DSP design (44.1 to 96 kHz, real time)

### 9.1 The one decision that shapes everything else

**The CL 1B model must not reuse `opto::model::Cell`.** It reuses the divider, the static photoconductive law,
the filters, the metering and the hygiene, and it brings its own control-voltage state.

This is the opposite of the call the LA-3A made ([[LA-3A]] section 7.1), and the reasoning is the mirror image
of it. The LA-3A shares the LA-2A's *actual T4B module* in the same role, so duplicating the cell would have been
two copies of one physics. The CL 1B does not share the cell, does not share the light source, and — decisively
— does not put its time constants there at all (section 4.3). Importing `Cell` would import `tau_r1 = 60 ms`,
`tau_t0 = 0.5 s`, `k_m = 12` and the trap memory into a machine whose Release knob is supposed to run from
50 ms to 10 seconds and whose Manual mode is supposed to have no memory at all. The bottom half of the Release
knob would stop doing anything, Fix/man would stop being distinguishable from Manual, and the model would
quietly become a third LA-2A with extra knobs.

What it does share is real and worth sharing:

| Existing item | CL 1B treatment |
|---|---|
| `opto::model::Divider` and its `resistance` / `attenuation` / `gr_db` | **reused as a type**, with CL 1B values, and extended by one field for the Ratio rheostat (9.2) |
| `Cell::carriers_for`'s power law and `CELL_GAMMA` | **reused as the static** light-to-conductance law |
| `Cell` itself, `Cell::step`, `n_t`, `capture`, `tau_t0`, `k_m` | **not used** |
| `Cell::light_for` and `EL_B` (the Alfrey-Taylor panel law) | **not used**: the GRE's emitter is not a documented EL panel and its drive is a current source, not a high-voltage swing |
| `R_DARK` | **reused** |
| `opto::filters::{OnePole, Shelf, Biquad, flush}` | **reused** |
| `VU_REF_DBFS`, `VU_REF_AMP`, `SINE_MEAN_ABS`, `vu_of` | **reused**, and Softube publish the same −18 dBFS reference [15] |
| the VU ballistics, stereo link, denormal flushing, static solver and `transfer_curve` | **reused** |
| `pr_gain`, `makeup_db`, `tube` | **replaced** |

One alternative I am deliberately not taking: a black-box neural or state-space model of the whole box, which
is where the literature has gone for optical compressors [64] [66] [69] [70]. It would very likely sound closer
than this grey-box model, and it would teach a reader of this repository nothing about why a CL 1B behaves the
way it does, which is the point of the example.

The cleanest structure is a sibling module `src/dsp/opto1b/` alongside `opto` and `opto3`, importing
`crate::dsp::opto::{filters, model::{Divider, R_DARK, CELL_GAMMA, VU_REF_DBFS, VU_REF_AMP, SINE_MEAN_ABS}}` and
nothing else from `model`. **The load-bearing requirement is that it must not import `Cell`.** If a future
refactor makes that import look convenient, test 27 will fail, and it should.

### 9.2 Block diagram in words, per channel per sample

1. **Input transformer**: first-order high-pass at 3.5 Hz and first-order low-pass at 35 kHz, together giving
   the published 5 Hz to 25 kHz at −3 dB when combined with the output transformer (9.6).
2. **The divider**, with the cell resistance from the previous sample. Three nodes, computed exactly:
   `Z_c`, `Z_b`, the audio attenuation `A_c` and the detector attenuation `A_b`. This is the only place the
   Ratio knob appears.
3. **Audio node**: `y = A_c · x`.
4. **Photocell nonlinearity**: the same small odd-order term the other optical models use, at a low strength,
   because the published distortion is very low and Lydkraft claim the element "in itself has a very low
   harmonic distortion" [2].
5. **Detector tap**: `d_in = A_b · x`, multiplied by the Threshold pot's gain. Note it is **not** the audio node.
6. **User side-chain high-pass** (`sc_hpf`, the lab's shared extra; not on the hardware, but Softube, Universal
   Audio and Stam Audio all added one [15] [18] [41]).
7. **Detector**: one pole at 33.5 kHz, then mean-absolute rectification.
8. **Timing**: the control state `u`, driven by whichever of the three modes is selected (9.4).
9. **GRE**: one pole at `TAU_CELL`, then the static drive-to-conductance law, then `R_gre`.
10. **Make-up**: the Gain knob, off to +30 dB, applied **after** the divider.
11. **Tube amplifier**: a single-ended asymmetric stage (second harmonic), then a symmetric soft ceiling for the
    push-pull output (third harmonic).
12. **Output transformer**: a low-frequency nonlinearity that dominates the 40 Hz distortion figure, then a
    first-order low-pass.
13. **Mix / bypass / meter / stereo link**, all shared with the other models.

### 9.3 Parameter table

Ids use the `cl1b_` prefix as briefed. Shared ids keep the names they already have in the lab.

| id | Label | Range / labels | Taper | Default | Notes |
|---|---|---|---|---|---|
| `cl1b_gain` | Gain | 0.0 to 1.0, displayed off / −10 to +30 dB | log pot, see `gain_db` in 9.4 | **0.265** | Unity at 0.265, the measured position of the "0" dot (2.2). Maximum exactly **+30.0 dB** [2]. Never affects compression [2]. |
| `cl1b_ratio` | Ratio | 0.0 to 1.0, displayed 2:1 to 10:1 | **linear** (P2 is a linear rheostat) | **0.375** | 0 = the pot at 0 Ω, full feedback, a flat 2:1; 1 = 10 kΩ. The displayed ratio is a label, not a slope (3.4). |
| `cl1b_threshold` | Threshold | 0.0 to 1.0, displayed off / 0 to −40 dBu | log pot, shared curve with Gain (2.2) | **0.5** | Defined as the level at which **1 dB** of reduction occurs [2]. Clockwise lowers it. |
| `cl1b_attack` | Attack | 0.0 to 1.0, displayed 0.5 to 300 ms | **log** (P4 is a log pot) | **0.75** | 2 o'clock, Lydkraft's own vocal setting [2]. In Fix/man this is a delay, not an attack (5.4). |
| `cl1b_release` | Release | 0.0 to 1.0, displayed 0.05 to 10 s | **linear** (P5 is a linear pot) | **0.25** | 10 o'clock, Lydkraft's own vocal setting [2]; about **2.5 s**, not 350 ms (3.7). |
| `cl1b_mode` | Attack/Release Select | Fixed, Fix/Man, Manual | switch | **Manual** | Lydkraft's own vocal and instrument settings both use Manual [2]. |
| `cl1b_meter` | Meter | Input, Compression, Output | switch, not automatable | **Compression** | "Leave the meter switch in position compression" [2]. Output: 0 VU = +4 dBu. |
| `cl1b_bus` | Sidechain Bus | Off, 1, 2 | switch | **Off** | Painted on the face for the joke; in the model it selects the stereo link group, which is what Softube did with it [15]. |
| `link` | Stereo Link | toggle | — | on | Shared. Matches the hardware's bus behaviour: the unit compressing most controls the others [2]. |
| `mix` | Mix | 0 to 100 % | linear | 100 % | Not on the hardware. Softube, Universal Audio and Stam Audio all added it [15] [18] [41]. |
| `sc_hpf` | Side-chain HPF | 0 (off) to 300 Hz | linear | 0 | Shared extra; the same three added one [15] [18] [41]. Section 3.5: the hardware's detector is flat, so the default must be **off**. |
| `bypass` | In | toggle | — | off | The hardware's "clickless" In/Out switch [1]. |

**No cell-wear parameter**, unlike the LA-2A's `opto_cell` and the LA-3A's `la3a_cell`. Lydkraft claim "no
long-term degradation of the element thus giving it almost infinite life" [2], owners report units are all
alike [29], and nobody has published a contrary observation (4.2). Inventing one would be inventing a fact.

### 9.4 Equations per block

Let `fs` be the sample rate and `T = 1/fs`. A first-order section with time constant `tau` uses
`a = 1 − exp(−T/tau)`.

**Calibration.** 0 VU = −18 dBFS RMS = +4 dBu, the reference the lab already uses and the one Softube publish
[15]. `VU_REF_AMP = 10^(−18/20)·√2` is the sine peak amplitude at 0 VU.

**Input transformer.**

```
x = HighPass(x_in; IN_HP_HZ)        // 3.5 Hz
x = LowPass(x;    IN_LP_HZ)         // 35 kHz
```

**The divider** (section 3.2 and 3.4; `r_gre` is the previous sample's cell resistance):

```
R_SERIES = 100_000        // R2
R_THR    = 100_000        // P1 track
R_POT    = 100_000        // P3 track
R_RATIO  = R_RATIO_MAX * cl1b_ratio       // P2, 0 .. 10_000, LINEAR

Z_c   = r_gre * R_POT / (r_gre + R_POT)
Z_b   = R_THR * (R_RATIO + Z_c) / (R_THR + R_RATIO + Z_c)
a_raw = Z_b / (R_SERIES + Z_b)                     // detector node B
c_raw = a_raw * Z_c / (R_RATIO + Z_c)              // audio node C
```

Both are normalised by the same expressions evaluated at `r_gre = R_DARK`, computed once per Ratio change:

```
A_audio = c_raw / c_dark
A_det   = a_raw / a_dark
y       = A_audio * x
```

**Photocell nonlinearity** (reused from the other optical models, weakened; a photoresistor's distortion grows
with the voltage across it, which is why the term is scaled by the attenuation [61]):

```
k   = CELL_CUBIC * (1 - A_audio)          // CELL_CUBIC = 0.1  (LA-2A 0.6, LA-3A 0.2)
q2  = (y / CELL_CUBIC_V0)^2
y  *= 1 - k * q2 / (1 + q2)
```

**Detector.**

```
s = A_det * x
s = UserScHpf(s)                          // sc_hpf, off by default
s = LowPass(s; SC_LP_HZ)                  // 33.5 kHz, C1 across R3
d = |s| * thr_gain(cl1b_threshold)        // mean-absolute; the rectifier is exact
```

with the Threshold law fitted to the panel's own dot positions (2.2) and calibrated by the anchor in 9.5:

```
thr_gain(p) = 10^((G0 + THR_DB(p)) / 20)
THR_DB(p)   = -40 * f(clamp((p - 0.145) / 0.855, 0, 1))     // f from the measured dots, monotone fit
```

**Timing.** One state `u`, in control volts. `d` is the rectified, scaled detector output.

```
Manual:
    if d > u:  u += a_attack(cl1b_attack) * (d - u)
    else:      u -= min(u - d, RELEASE_SLEW(cl1b_release) * T)      // ramp, not exponential (3.7)

Fixed:
    if d > u:  u += a_attack_fixed * (d - u)                        // 1 ms
    else:      u -= min(u - d, RELEASE_SLEW_FIXED * T)              // 50 ms equivalent

Fix/Man:
    attack is always a_attack_fixed                                 // 1 ms
    on the falling edge, start a timer:
        peak_len  = how long u was rising or held
        delay     = t_attack(cl1b_attack)                           // 0.5 .. 300 ms, the SAME range
        if peak_len >= delay:  release with the Manual slew immediately
        else:                  release with the Fixed slew for (delay - peak_len), then the Manual slew
```

with

```
t_attack(p)     = A_MIN * (A_MAX / A_MIN)^p                 // 0.5 ms .. 300 ms, LOG pot
t_release(p)    = R_MIN_S + p * (R_MAX_S - R_MIN_S)         // 0.05 s .. 10 s, LINEAR pot
RELEASE_SLEW(p) = U_REF_10DB / t_release(p)                 // volts per second: full recovery from
                                                            // 10 dB takes t_release(p) [2]
```

`U_REF_10DB` is the control voltage that produces 10 dB of reduction, which is exactly the quantity the service
manual's 250 mV calibration pins (9.5). Defining the slew that way makes the 10 s figure assertable as the
manual states it — a full recovery from 10 dB — rather than as a time constant nobody published.

**The GRE.**

```
i = LowPass(u; TAU_CELL)                                     // 0.2 ms, the cell's own lag
g = 1/R_DARK + K_G * clamp(i / I_REF, 0, 1)^CELL_GAMMA       // CELL_GAMMA reused, 0.8
r_gre = clamp(1/g, R_GRE_MIN, R_DARK)
```

**Make-up and the tube amplifier.**

```
gain_db(p) = the same monotone log-pot curve as THR_DB, scaled: off at 0, +30.0 dB at 1, unity at 0.265
w  = y * 10^(gain_db(cl1b_gain) / 20)
w  = w + ASYM * w * w                                        // V1A, single-ended: second harmonic
w  = V_CLIP * w / (1 + |w / V_CLIP|^N)^(1/N)                 // V2A/V2B push-pull: symmetric, odd
w  = w + LF_NONLIN(w)                                        // TR2 core, dominates the 40 Hz THD
out = LowPass(w; OUT_LP_HZ)
```

`LF_NONLIN` is a low-frequency-weighted odd term: high-pass the signal at 100 Hz, subtract to get the
low-frequency content, apply a cubic to that alone, and add it back. That reproduces the specification's most
distinctive property — a distortion figure that is quoted at 40 Hz and is the *same* at 0 dBu and +10 dBu [3] —
which a level-dependent tube nonlinearity alone cannot do. **Derived from the shape of the published figure**,
not from a measurement of a transformer.

### 9.5 The two calibrations, and why they are better anchors than the LA-2A had

The LA-2A model had to be calibrated to a soft target because no absolute threshold was ever published
([[LA-2A]] section 7.2). The CL 1B has two hard ones, both from the service manual [2].

**Calibration A, the sidechain-to-reduction map.** Threshold fully counter-clockwise, Ratio at 2:1, 1 kHz at
0.0 dBu, Gain trimmed for 0.0 dBu out; then **+250.0 mV DC at the sidechain jack gives exactly −10.0 dB**. In
the model, the sidechain bus injection point is the control node, so this fixes `U_REF_10DB` and, through it,
`K_G` and `I_REF`: solve so that a control state equivalent to 250 mV produces `A_audio` 10.0 dB down at Ratio
0. There is no freedom left in that equation, which is unusual and welcome.

**Calibration B, the make-up gain.** 1 kHz at −30.0 dBu in, Gain fully clockwise, Ratio at 2:1, output exactly
**0.0 dBu**. So `gain_db(1.0) = +30.0` exactly, and the taper is anchored at the top as well as at unity.

The **threshold** offset `G0` is then solved so that the panel's own marks are true: with `cl1b_threshold` at
the measured "−20" dot position (p = 0.519), one decibel of reduction occurs at a 1 kHz input of −20 dBu. That
is the manual's own definition of the control [2] applied to the panel's own scale, and it is the model's
primary calibration.

### 9.6 Constants

Values with a source are anchored; the rest are **estimates** to tune against section 10. The LA-2A and LA-3A
columns are there so a reviewer can see at a glance what is shared and what is not.

| Constant | LA-2A | LA-3A | **CL 1B** | Anchor |
|---|---|---|---|---|
| `R_SERIES` | 70.7 kΩ | 68 kΩ | **100 kΩ** | R2 on the schematic [9] |
| `R_POT` | 100 kΩ | 101.3 kΩ | **100 kΩ** | P3, the Gain pot [9] |
| `R_THR` | — | — | **100 kΩ** | P1, the Threshold pot [9] — **new** |
| `R_RATIO_MAX` | — | — | **10 kΩ** | P2, the Ratio pot [9] — **new** |
| `R_DARK` | 2 MΩ | 2 MΩ | 2 MΩ | shared |
| `R_GRE_MIN` | 500 Ω | 400 Ω | **200 Ω** | **estimate**: no maximum reduction is published (6.3), so this is tuned against test 6's shape, not against a figure |
| `CELL_GAMMA` | 0.7 | 0.7 | 0.7 | CdS gamma 0.6-0.9 [59] [60] [62], and 0.7 is the LA-2A design table's pick inside that range. This row read 0.8 and **shared** until 2026-09-03: the implementation held 0.8 with nothing written behind it, and "shared" cited the code rather than a source. No T4 measurement exists either way |
| `EL_B` | 5.0 | 5.0 | **not used** | the GRE's emitter is undocumented (4.2) |
| `tau_f0`, `tau_r1`, `tau_t0`, `k_m`, `capture`, `k_gen` | the T4's | the T4's | **not used** | the timing is electronic (4.3) |
| `TAU_CELL` | — | — | **0.2 ms** | **estimate**, chosen well under the 0.5 ms fastest published attack — **new** |
| `A_MIN`, `A_MAX` | — | — | **0.5 ms, 300 ms** | published attack range [2] [3] — **new** |
| `A_FIXED` | — | — | **1 ms** | published fixed attack [2] — **new** |
| `R_MIN_S`, `R_MAX_S` | — | — | **0.05 s, 10 s** | published release range [2] [3] — **new** |
| `R_FIXED_S` | — | — | **50 ms** | published fixed release [2] — **new** |
| attack taper | — | — | **logarithmic** | P4 is a 500 kΩ **log** pot [9] — **derived** |
| release taper | — | — | **linear** | P5 is a 500 kΩ **linear** pot [9] — **derived**, and it is the finding that changes every published setting (3.7) |
| release shape | exponential, two-stage | exponential, two-stage | **constant slew** | resistive pull-up from a fixed supply [9] — **derived** (3.7) |
| `U_REF_10DB` | — | — | solved | +250.0 mV gives −10.0 dB [2] — **new** |
| `GAIN_MAX_DB` | 40 | 50 | **30.0** | service manual calibration [2] |
| gain / threshold taper | — | — | one shared monotone curve | fitted to the measured dot angles (2.2) — **derived** |
| `IN_HP_HZ`, `OUT_LP_HZ` | 12 Hz, 40 kHz | 7 Hz, 50 kHz | **3.5 Hz, 30 kHz** | 5 Hz to 25 kHz at −3 dB [3] — **derived** |
| `SC_LP_HZ` | — | — | **33.5 kHz** | C1 100 pF across R3 47.5 kΩ [9] — **derived** |
| sidechain shaping | R37 shelf | 100 Hz + 30 Hz + contour | **none** | nothing in the schematic [9] (3.5) |
| `CELL_CUBIC` | 0.6 | 0.2 | **0.1** | 0.15 % THD+N [3] against the LA-2A's 0.9-4.2 % |
| `ASYM` | 0.05 bias | 0.0016 | small, second harmonic | one single-ended ECC83 stage [9] — **estimate** |
| `V_CLIP`, `N` | tanh, k 0.2 | 2.51, N 8 | ceiling at the +26 dBu equivalent, N ≈ 6 | max output +26 dBu at < 1 % [3] — **estimate** |
| `LF_NONLIN` | — | — | cubic on the sub-100 Hz content | THD+N specified at 40 Hz and flat with level [3] — **derived** (9.4) |
| VU reference | −18 dBFS | −18 dBFS | −18 dBFS | shared, and Softube publish it [15] |
| cell wear | 3 positions | 3 positions | **none** | "no long-term degradation" [2] (4.2) |

### 9.7 Sample rate, oversampling, stereo, hygiene

- Every time constant is in seconds and every coefficient is recomputed on `set_sample_rate`. The fastest state
  is `TAU_CELL` at 0.2 ms, which is 8.8 samples at 44.1 kHz, so the integration stays comfortable; the 33.5 kHz
  sidechain pole is the one thing that genuinely wants 96 kHz to behave, and at 44.1 kHz it should simply be
  clamped below Nyquist rather than allowed to alias.
- **Oversampling.** The gain loop does not need it. The output stage does, mildly, because the `N ≈ 6` ceiling
  and the low-frequency cubic generate higher-order harmonics; 2× over the make-up and output stage only, as the
  LA-3A model already does, reusing `fet::oversample`.
- **Stereo** follows the hardware's bus: with `link` on, one control state is driven by the larger of the two
  channels' detector outputs, because "the unit which performs the most compression is controlling the others"
  [2]. That is a **maximum**, not a mean, and it differs from the LA-2A and LA-3A models, which average. Note the
  hardware's own warning that Ratio and Gain must match across linked units or the image shifts [2]; in a
  plug-in they always do.
- **Denormals**: the control state, the cell lag, the detector pole and the transformer poles all flush below
  `1e-12`. The long release means the control state can sit at a very small non-zero value for many seconds
  after a passage, which is exactly the condition that produces denormals, so it needs the flush most.

### 9.8 What the page should show

The framework stays headless and uncoloured; every face and colour belongs to the example
([[feedback-framework-vs-plugin]]). For this model the face is: a **blue 3U panel** (section 2.3 on the colour),
five large black knobs with white index lines and panel-printed scales, three black lever switches with white
stripes, a cream VU meter with a warm lamp behind it and LYDKRAFT-shaped lettering replaced by the tribute's own
name, a red jewel, and a small black rotary for the mains. The measured proportions of section 2.2 are enough to
lay it out.

Two display ideas that are specific to this model and worth the effort. First, the **cell stream** already
exists for the other optical models; here it should show the control voltage and the cell resistance, not the
T4's three states, because that is what the CL 1B has. Second, and better: a **live static curve** with the
current operating point marked on it, because section 3.4's rising ratio is invisible on a meter and obvious on
a curve, and it is the single most interesting thing about the machine.

---
## 10. Test plan

Each test drives the DSP core offline at 44.1, 48 and 96 kHz. **Every test below names the published figure it
asserts and the source of that figure**, because an audit of this repository found tests across five models that
asserted the model's own output back to itself, which proves nothing. Where a real published number does not
exist, the entry says so and proposes no bound at all rather than a loose one; those entries are grouped in
10.6 so nobody can mistake them for coverage.

Unless stated, settings are: Attack/Release Select = Manual, `mix` 100 %, `sc_hpf` off, `link` off, `cl1b_bus`
off, and levels are referred to **0 VU = −18 dBFS RMS = +4 dBu** [15]. "The manual" is the CL 1B owner's manual
[2]; "the sheet" is the specification sheet [3].

### 10.1 Static behaviour and calibration

1. **Bypass and unity.** `bypass` on: output equals input to 1e-6. `bypass` off, Threshold off, `cl1b_gain` at
   0.265: output level equals input level within ±0.3 dB from −40 to +10 dBu.
   *Figure asserted:* the Gain scale's "0" mark sits at knob fraction 0.265. *Source:* my measurement of
   Lydkraft's own front photograph, section 2.2 [6]. **This is the one test in the file whose reference figure
   is mine rather than a manufacturer's, and it is a measurement of a photograph, not of a unit.**

2. **Maximum gain is exactly +30 dB.** 1 kHz sine at −30.0 dBu, `cl1b_gain` at 1.0, `cl1b_ratio` at 0.0
   (2:1), Threshold off: output is **0.0 dBu ±0.3 dB**.
   *Figure:* "Apply a signal of 1 kHz, −30,0 dBU ... Turn the GAIN-control fully clockwise. Set the RATIO-control
   at 2:1. Adjust the preset GAIN ... to an output-reading of 0,0 dBU." *Source:* service manual, Adjustment of
   basic gain [2]. Assert at all three sample rates.

3. **The sidechain calibration point.** Threshold off, `cl1b_ratio` 0.0, 1 kHz at 0.0 dBu, `cl1b_gain` trimmed
   for 0.0 dBu out; inject the model's equivalent of **+250.0 mV DC** at the sidechain bus: output drops to
   **−10.0 dB ±0.3 dB**.
   *Figure:* "Apply a DC-voltage of +250,0 mV into the side chain jack socket (tip) and observe that the output
   level has dropped to −10,0 dB." *Source:* service manual, Adjustment of compression tracking [2]. This is the
   model's primary calibration and the tightest anchor in the document set.

4. **The threshold definition.** With `cl1b_threshold` at each of the five measured dot positions (0.146, 0.272,
   0.519, 0.686, 1.000), a 1 kHz sine produces **exactly 1 dB** of gain reduction at **0, −10, −20, −30 and
   −40 dBu** respectively, ±2 dB on the input level.
   *Figure:* "The threshold ... is defined as the point where the gain is reduced by 1 dB" and the panel's own
   scale. *Sources:* manual [2] for the definition, my dot measurement (2.2) of [6] for the positions.
   *Tolerance:* ±2 dB is deliberately loose because Softube, who had the hardware and the designer, state that
   "the actual numbers on the panel are very approximate" [15]. Asserting ±0.5 dB against an approximate
   silkscreen would be asserting a precision the source does not have.

5. **The Gain control does not affect compression.** At a fixed Threshold and input giving 6 dB of reduction,
   sweep `cl1b_gain` across its whole range: the gain reduction changes by **less than 0.1 dB**.
   *Figure:* "It is placed after the gain-reduction circuit and therefore **has no influence on the threshold
   setting**." *Source:* manual [2]. This is a circuit property (P3 is a divider, section 3.2) and should be
   exact, so the tolerance is tight.

6. **Ratio at the 2:1 stop.** `cl1b_ratio` 0.0, Threshold set so a 1 kHz sine at −10 dBu gives 6 dB of
   reduction; raise the input by **10 dB**: the output rises by **5 dB ±1 dB**.
   *Figure:* "If the ratio selected is to 2:1, and the input signal increases 10 dB, the output signal is only
   increased by 5 db." *Source:* manual, Ratio [2]. Repeat the 10 dB step from three starting points spanning
   3 dB to 20 dB of reduction and assert **5 dB ±1 dB at every one of them**, because at this setting the ratio
   is derived to be flat with depth (3.4).

7. **Ratio at the 10:1 stop is not 10:1 near the threshold.** `cl1b_ratio` 1.0: the local slope at 1 dB of
   reduction must be **shallower than 4:1**, and the local slope must **increase monotonically** with depth.
   *Figure:* there is no published ratio-versus-depth curve, so this test asserts a **direction**, not a number,
   and the direction comes from two published statements: Softube's "the actual numbers on the panel are very
   approximate" [15], and Bonedo's "Man ist darüber hinaus oftmals sehr verwundert, wie viel dB Gain-Reduction
   gerade stattfinden, ohne dass die Quellen dabei gequetscht wirken" [20]. **I have no published number for the
   ratio at a given depth and I am not going to invent one.** The 4:1 figure is my derivation (3.4), labelled as
   such in the test, and it is a sanity bound, not evidence.

8. **Ratio monotonicity.** Sweeping `cl1b_ratio` from 0 to 1 at a fixed input and Threshold: the gain reduction
   is **monotonically non-decreasing**, and at 20 dB of reduction the 10:1 setting gives **at least 6 dB more**
   reduction than the 2:1 setting.
   *Figure:* the panel's own two labels, 2:1 and 10:1, and Sound On Sound's reading of the clockwise end as
   "effectively being limiting". *Sources:* [2] [19]. The 6 dB figure is **derived** from section 3.4's divider
   arithmetic and is labelled so.

### 10.2 Frequency response and distortion

9. **Frequency response.** Threshold off, `cl1b_gain` at unity, 5 Hz to 30 kHz: the **−3 dB points are at
   5 Hz ±2 Hz and 25 kHz ±3 kHz**, and the response is within ±1 dB from 20 Hz to 20 kHz.
   *Figure:* "Frequency response @ -3 dB: 5 Hz to 25 kHz." *Source:* web page, brochure and specification sheet
   [1] [3] [4]. Note that the sheet itself misprints the upper figure as "25 Hz" (6.4); the other three sources
   agree on 25 kHz. At 44.1 kHz the upper assertion must be skipped, since 25 kHz is above Nyquist.

10. **Distortion at 40 Hz, both levels.** A **40 Hz** sine, Threshold off, `cl1b_gain` at unity: THD+N is
    **0.15 % ±0.08 %** at **0 dBu**, and **0.15 % ±0.08 %** at **+10 dBu**.
    *Figure:* "Distortion THD+N @ 40Hz — 0 dBU 0,15 % — +10 dBU 0,15 %." *Source:* specification sheet [3].
    Both halves matter: the *value* and the fact that it **does not change with level** over that 10 dB. A model
    whose distortion rises with level will pass the first and fail the second, and the second is the reason the
    output-transformer term exists at all (9.4). Do **not** write a 1 kHz version of this test: **Lydkraft
    publish no 1 kHz distortion figure**, and asserting one would be inventing it.

11. **Maximum output and input.** Threshold off: the level at which THD reaches **1 %** at the output is
    **+26 dBu ±2 dB**, and the input level at which it reaches 1 % is **+21 dBu ±2 dB**.
    *Figure:* "Max. output: +26 dBU <1 % — Max. input: +21 dBU <1 %." *Source:* specification sheet [3].

12. **The output must be clean where it matters.** With **6 dB** of reduction at 0 VU, THD stays **below
    0.5 %**. *Figure:* **there is no published distortion figure under compression.** This test asserts only
    that the compressed figure does not exceed the uncompressed 0.15 % at 40 Hz [3] by more than a factor of
    three, which is my
    bound and is labelled as mine. **Stated as an explicit gap: Lydkraft publish distortion only with the
    compressor idle.**

### 10.3 Dynamics

13. **Fixed attack.** Select = Fixed, Threshold set for 10 dB of steady-state reduction, a 1 kHz tone stepping
    up by 18 dB: the gain reduction reaches 63 % of its final value in **0.5 to 3 ms**.
    *Figure:* "Fixed. Attack time: 1 msec." *Source:* manual, Attack/release select [2]. The bracket is wide
    on purpose: **Lydkraft do not say whether "1 ms" is a time constant, a 63 % time or a settling time**, and
    the bracket spans all three readings of the same published number rather than picking one silently.

14. **Manual attack range.** Select = Manual. At `cl1b_attack` 0.0 the 63 % time is **0.3 to 1.5 ms**; at 1.0 it
    is **150 to 600 ms**.
    *Figure:* "The attack control is continuously variable from 0.5 to 300 milliseconds." *Source:* manual [2],
    repeated on the sheet [3]. Same ±2× bracket, same reason as test 13.

15. **The attack taper is logarithmic.** With `cl1b_attack` at 0.0, 0.25, 0.5, 0.75 and 1.0, the measured attack
    times form a **geometric** progression: each ratio of successive times is within ±40 % of the others.
    *Figure:* P4 is a **500 kΩ log** potentiometer. *Source:* schematic TE130-42 [9]. **Derived**, and the
    ±40 % is loose because a real audio-taper pot is a two-segment approximation to a logarithm, not a logarithm.

16. **The release taper is linear, which is the whole point.** With `cl1b_release` at 0.0, 0.25, 0.5, 0.75 and
    1.0, the recovery times from 10 dB form an **arithmetic** progression from **0.05 s to 10 s**, and the value
    at 0.25 is **2.5 s ±0.6 s** — *not* the 0.35 s a logarithmic taper would give.
    *Figures:* the range from "The release control is continuously variable from 0,05 to 10 seconds" [2] and
    "Release: 50 ms to 10 s" [3]; the taper from P5 being a **500 kΩ linear** pot on the schematic [9]. This is
    the test that stops somebody assuming a log taper because every other compressor has one (3.7).

17. **The slowest release, exactly as the service manual measures it.** Select = Manual, `cl1b_attack` 0.0
    (fast), `cl1b_release` 1.0 (slow), 1 kHz at 0.0 dBu, Threshold adjusted for **10 dB** of reduction; remove
    the tone: the reduction returns to **0 dB in 10 s ±2 s**.
    *Figure:* "Switch off the 1 kHz and observe that the VU meter moves to 0 VU in approx. 10 sec." *Source:*
    service manual, Adjustment of the release control [2]. Note this is a **full recovery**, not a time
    constant, and the test must measure it that way.

18. **Fixed release.** Select = Fixed, same procedure as test 17: the reduction returns to 0 dB in a time
    consistent with **50 ms**, measured as 63 % recovery in **20 to 120 ms**.
    *Figure:* "Fixed. ... release time: 50 msec." *Source:* manual [2]. Same definitional bracket as test 13.

19. **Fix/Man has the fixed attack, not the knob's.** Select = Fix/Man, `cl1b_attack` at 1.0 (which in Manual
    would be 300 ms): the attack on an 18 dB step is **within a factor of 3 of the Fixed mode's**, i.e. still
    about 1 ms, and **at least 50× faster** than Manual mode's attack at the same knob position.
    *Figure:* "Fix/man. This setting combines the release times of fixed and manual mode. **The attack time is
    as in the fixed mode.**" *Source:* manual [2]. This is the trap of section 5.4 and it is the test that
    catches it.

20. **Fix/Man's delay is set by the attack knob.** Select = Fix/Man, a **5 ms** burst at 10 dB of reduction
    followed by silence, with `cl1b_release` at 1.0 (slow). With `cl1b_attack` at 0.0 the reduction is more than
    half recovered within **100 ms**; with `cl1b_attack` at 1.0 it is **less than a quarter** recovered at
    100 ms. Sweeping the attack knob must move the crossover point monotonically.
    *Figure:* "the attack control changes function from a pure attack control, to a control of delay with the
    same time range. The more CW the attack control is turned, the longer time before the release control takes
    over." *Source:* manual [2].

21. **Fix/Man switches itself off for long peaks.** Select = Fix/Man, `cl1b_attack` at 0.25 (about 5 ms of
    delay), `cl1b_release` at 1.0. A **1 second** burst at 10 dB must release at the *Manual* rate from the
    start; its recovery to 50 % must be **at least 10× slower** than the 5 ms burst of test 20 at the same
    settings.
    *Figure:* "This function is valid only if the time of the peak is shorter than the setting of the attack
    control. If the peak of the program is longer than the setting of the attack control ... it will respond as
    in the manual mode." *Source:* manual [2].

22. **Fix/Man is what Lydkraft recommend for a mix.** Select = Fix/Man, `cl1b_attack` 0.750, `cl1b_release`
    0.250, `cl1b_ratio` 0.125, on a full-band programme item with the Threshold trimmed by the test: the average
    gain reduction lands in **3 to 4 dB** with the Threshold at the setting the test solves for, and the model
    reaches that window at some Threshold setting.
    *Figure:* "FINAL MIX — COMPRESSION NEEDED: 3-4 dB — Attack/release select: Fix/man — Attack: 2 o'clock —
    Release: 10 o'clock — Ratio: 9 o'clock." *Source:* manual, Suggested applications [2], with the clock
    mapping of section 2.6.

23. **The vocal setting.** Select = Manual, `cl1b_attack` 0.750, `cl1b_release` 0.250, `cl1b_ratio` between
    0.250 and 0.750: the model reaches **4-5 dB** of average reduction on a sustained vocal-like signal at some
    Threshold setting, and the attack at that setting measures **40 to 100 ms** and the release **1.9 to 3.1 s**.
    *Figure:* "BASS, PIANO, GUITAR, KEYBOARDS AND VOCALS — COMPRESSION NEEDED: 4-5 dB — Manual — Attack: 2
    o'clock — Release: 10 o'clock — Ratio: 10-2 o'clock." *Source:* manual [2]. The time windows are
    **derived** from the published ranges and the schematic's tapers (2.7) and are labelled so.

### 10.4 Metering, stereo and hygiene

24. **Meter accuracy in Compression.** The Compression reading matches the model's actual attenuation to within
    **±0.5 dB** at steady state, from 1 to 20 dB of reduction.
    *Figure:* "The VU-meter accuracy should be within +/− 0,5 dB when reading compression." *Source:* service
    manual [2].

25. **Meter calibration in Output, and VU ballistics.** In Output, a steady 1 kHz tone at **+4 dBu reads 0 VU
    ±0.3 dB**; in Input, a **−18 dBFS** sine reads 0 VU ±0.3 dB. VU ballistics: 99 % of reading in **300
    ±30 ms** with 1 to 1.5 % overshoot.
    *Figures:* "'0 VU' is equivalent to +4 dBU" [2]; "a sine wave showing 0 VU at the output corresponds to a
    −18 dBFS output signal. Correspondingly, a 18 dBFS sine at the input will show 0 VU" [15]; the 300 ms and
    overshoot from the VU standard [74] [75].

26. **Stereo link takes the maximum, not the mean.** With `link` on and hard-panned material, the gain reduction
    on both channels equals the *larger* of the two unlinked reductions, within 0.3 dB.
    *Figure:* "The interconnection implies, that **the unit which performs the most compression is controlling
    the others**." *Source:* manual, Compressor interconnection [2]. This differs from the LA-2A and LA-3A
    models, which average, and the difference is documented, not a preference.

27. **The T4 cell was not imported.** *This is the structural test and it is the reason section 9.1 exists.*
    Select = Manual, `cl1b_release` 0.0 (fastest). Compare a **100 ms** burst and a **20 second** burst, both
    held at 20 dB of reduction. Their 90 % recovery times must agree within **20 %**. Then run the identical
    comparison on the LA-2A engine: its long-burst recovery must be **at least twice** its short-burst recovery.
    *Figures:* the CL 1B's release range is fully specified as a function of one knob, 0.05 s to 10 s [2] [3],
    with no programme dependence stated anywhere for Manual mode; the LA-2A's memory is documented in its own
    dossier and manual ([[LA-2A]] section 4.3). If somebody makes the CL 1B import `opto::model::Cell`, the trap
    memory will break the first half of this test and nothing else in the suite will notice.

28. **Numerical robustness.** Ten minutes of digital silence after 30 s at 20 dB of reduction with
    `cl1b_release` at 1.0 (10 s): no denormals, no NaN or infinity for inputs of ±10.0, DC and silence, and every
    state inside its bounds. The control state must flush below `1e-12`; it is the state most exposed, because a
    10 s release leaves it very small and non-zero for a long time (9.7).
    *Figure:* **none, and there cannot be one.** This is a numerical-hygiene test, not a behavioural one; it
    asserts a property of the implementation, and the only external anchor is the release range that makes the
    condition likely, 0.05 s to 10 s [2] [3].

29. **Sample-rate consistency.** Gain-reduction envelopes at 44.1, 48 and 96 kHz agree within **0.2 dB** for the
    same input, and the static curves within 0.1 dB.
    *Figure:* **none.** Rate invariance is a property of the implementation and no manufacturer publishes it.
    The 0.2 dB is my tolerance, chosen to be tighter than any published tolerance in this file, and labelled so.

### 10.5 CL 1B against LA-2A and LA-3A, same input, same file

These run two or three engines from the same buffer and assert a *difference*. Each has a published direction
and a published magnitude; none is a taste judgement.

30. **The detector's frequency response.** Calibrate all three models to 8 dB of reduction on a 1 kHz sine, then
    feed **50 Hz** at the same level. The **CL 1B must produce within 2 dB** of the reduction it produced at
    1 kHz; the **LA-3A must produce at least 4 dB less**.
    *Figures:* the CL 1B's sidechain contains no coupling capacitor, no high-pass and no shelf, only two 100 pF
    stabilising capacitors at 33.5 kHz and 80 kHz — schematic TE130-43 [9]; the LA-3A's low-frequency deafness
    is documented in its own dossier and manual. **This is the strongest structural differentiator in the file**
    and it is the one most likely to be broken by copying sidechain code from `opto3`.

31. **Attack range.** With the CL 1B in Manual at `cl1b_attack` 1.0, its attack on an 18 dB step must be **at
    least 20× slower** than the LA-2A's on the same step, and at `cl1b_attack` 0.0 it must be **at least 5×
    faster**.
    *Figures:* CL 1B 0.5 ms to 300 ms [2] [3]; LA-2A about 10 ms, LA-3A "1.5 ms or less" or "250 µs to 0.5 ms",
    both from their own dossiers. No LA-2A or LA-3A can be made to attack in 300 ms; that is the point.

32. **Release range.** With `cl1b_release` at 1.0, the CL 1B's recovery from 10 dB is **10 s** [2]; the LA-2A's
    first stage reaches 50 % in **60 ms**. Assert that the CL 1B's time to 50 % recovery exceeds the LA-2A's by
    **at least a factor of 20** at that setting, and that with `cl1b_release` at 0.0 the two are **within a
    factor of 3**.
    *Figures:* CL 1B 0.05 s to 10 s [2] [3]; LA-2A "60 ms to 50 %" from its own manual. The second half is the
    interesting one: at its fastest the CL 1B is genuinely in LA-2A territory, and the model should show that.

33. **Bandwidth.** With no reduction, the CL 1B's −3 dB points must lie **outside** both other models' at both
    ends: at or below 5 Hz and at or above 25 kHz, against the LA-2A's 30 Hz to 15 kHz and the LA-3A's 20 Hz to
    20 kHz.
    *Figures:* "Frequency response @ -3 dB: 5 Hz to 25 kHz" [1] [3]; the other two from their own dossiers and
    manuals.

34. **Distortion.** At 6 dB of reduction and 0 dBu, at **40 Hz**, the CL 1B's THD+N must be **at least 12 dB
    lower** than the LA-2A's.
    *Figures:* CL 1B 0.15 % at 40 Hz [3]; LA-2A measured at 0.9 % to 4.2 % across six units [71]. A factor of
    six is 15.6 dB, so 12 dB is the conservative reading of the pair.

35. **Make-up gain range.** The CL 1B's maximum make-up must be **+30.0 dB ±0.5 dB**; the LA-2A's is 40 dB and
    the LA-3A's 50 dB or 30 dB. Assert all three from the same test so a change to one is visible.
    *Figures:* the CL 1B's from the service manual's basic-gain calibration [2] and "Gain off to +30 dB" [3];
    the other two from their own dossiers.

36. **Ratio control.** At 20 dB of reduction, sweeping `cl1b_ratio` end to end must change the gain reduction by
    at least 6 dB (test 8); the LA-2A and LA-3A have **no ratio control at all**, so assert that neither exposes
    one. A trivial test, and it exists because the obvious way to build this model is to clone `opto3` and
    rename things, and this is the first thing that clone would get wrong.
    *Figures:* "Variable ratio from 2:1 to 10:1" [1] [3]; the absence of any ratio control on the other two,
    from their own manuals. The 6 dB is **derived** (3.4), not published.

37. **Programme dependence is switchable.** In Fix/Man the CL 1B must show a release that depends on burst
    length (test 20 and 21); in Manual it must not (test 27). The LA-2A and LA-3A must show it **in every mode**,
    because their memory is in the cell and there is no switch.
    *Figures:* the CL 1B's three-position select switch and its described behaviour [2]; the LA-2A's and LA-3A's
    lack of any such control, from their own manuals. This is section 4.3 stated as an assertion, and together
    with test 27 it is what keeps the three optical models three different machines.

### 10.6 Tests I am not writing, and why

The brief for this repository asks that where a real published number cannot be reached, I say so explicitly
rather than proposing a loose bound. These are the ones.

- **Maximum gain reduction.** Nothing published states it. The LA-2A has "40 dB" and the LA-3A has "40 dB"; the
  CL 1B has nothing. `R_GRE_MIN` is therefore an estimate tuned for a plausible curve, and there is **no test**
  asserting a maximum reduction.
- **Knee.** No published knee figure, no published static curve. Test 7 asserts a *direction* only, and says so.
- **Ratio versus depth.** No published curve. See test 7.
- **THD at 1 kHz.** Every distortion figure Lydkraft publish is at 40 Hz. There is no 1 kHz test.
- **Distortion spectrum.** No source states which harmonics the CL 1B produces or in what proportion. The LA-3A
  dossier could assert "both a second and a third harmonic above −90 dBc" because a listening comparison
  reported four overtones; nothing equivalent exists for the CL 1B, so there is **no harmonic-content test**.
- **Noise.** The sheet gives < −85 dBu at 0 dB gain and < −75 dBu at +30 dB, 22 Hz to 22 kHz, and CCIR-468-4
  figures 10 dB worse [3]. The model has **no noise source**, so these are not assertable. If a noise generator
  is ever added, these four figures are ready and should be used.
- **CMRR and crosstalk.** > 60 dB at 10 kHz and < −60 dB respectively [1] [3]. Neither has meaning for a
  plug-in with no common-mode path and no analogue channel adjacency. Not tested.
- **Unit-to-unit variation.** Owners say there is very little ("Every one is virtually the same" [29]) but
  nobody has measured two units. No test, and no cell-wear parameter (9.3).
- **Attack and release definitions.** Lydkraft never define whether their millisecond figures are time
  constants, 63 % times or settling times, except for the 10 s release, which the service manual gives as a full
  recovery [2]. Tests 13, 14 and 18 therefore use ±2× brackets that span every reading of the published number,
  and test 17 measures the one figure that is unambiguous.
- **Anything at all from an independent measurement.** There are none (6.3). Every figure in this test plan
  comes from Lydkraft, from Softube, or from my own measurement of a Lydkraft photograph, and each entry says
  which.

---

## 11. References

Everything below was fetched and read while writing this file. The schematics were located through a
third-party technical archive, because Lydkraft publish the owner's manual but not the service manual. Forum
threads are cited for what a person said, not as authorities; manufacturer documents are cited as manufacturer
claims, which on this device carries more weight than usual because there are no independent measurements at
all (6.3).

1. Tube-Tech (Lydkraft ApS), "CL 1B Opto Compressor" product page: features, specifications, the CL 1A / CL 1B
   FAQ, and the links to every document below. https://www.tube-tech.com/cl-1b-opto-compressor/
2. Lydkraft ApS, "Owners manual TUBE-TECH CL 1B Compressor", 8 pages, revision marks 140919 to 180515: the
   description, sidechain, interconnection, controls, attack/release select, suggested applications, adjustment
   procedure and sidechain PCB trimmer layout. The single most-cited source in this file.
   http://www.tube-tech.com/wp-content/uploads/2020/05/Tube-Tech-Manual-CL-1B-200513.pdf
3. Lydkraft ApS, "SPECIFICATIONS for TUBE-TECH CL 1B", one page.
   http://www.tube-tech.com/wp-content/uploads/2017/11/cl1bspecs.pdf
4. Tube-Tech, "MONO OPTO COMPRESSOR CL 1B" brochure page.
   http://www.tube-tech.com/wp-content/uploads/2017/11/Brochure-CL1B.pdf
5. Tube-Tech, CL 1B recall sheet: a line drawing of the front panel with every legend set in type.
   http://www.tube-tech.com/wp-content/uploads/2017/11/RE-CL1B.pdf
6. Tube-Tech, CL 1B hi-res front photograph, 2715 × 810, CMYK. Saved as `ref/cl1b-front-hires.jpg`; all of
   section 2.2's geometry and 2.3's colours were measured from it.
   http://www.tube-tech.com/wp-content/uploads/2017/11/CL1BFront.jpg
7. Tube-Tech, CL 1B hi-res rear photograph. Saved as `ref/cl1b-rear-hires.jpg`; section 2.4 is read from it.
   http://www.tube-tech.com/wp-content/uploads/2017/11/CL-1B-Rear.jpg
8. Tube-Tech, CL 1B perspective photograph. Saved as `ref/cl1b-perspective.jpg`; used only in 2.3.
   http://www.tube-tech.com/wp-content/uploads/2017/11/cl1bny2.jpg
9. Lydkraft ApS, "TUBE-TECH CL 1B compressor" service manual, 14 pages, with the complete four-sheet drawing
   set: TE130-40 interconnection (23 April 1993), TE130-42 front PCB 870314-2, TE 100/41 amplifier PCB
   900621-2, TE130-43 sidechain PCB 870316-2 (all 12 April 1993). Every component value in section 3.
   https://funkwerkes.com/web/wp-content/techdocs/MixedProAudio/Tube-Tech-CL1B-Compressor-SM.pdf
10. Tube-Tech, "The LYDKRAFT Story": company history, Petersen's biography, and the product timeline that dates
    the CL 1A to 1987 and the CL 1B to 1991. http://www.tube-tech.com/the-lydkraft-story/
11. Tube-Tech, "GENERAL TUBE-TECH FAQ": the valve policy, and "The code for the TUBE-TECH blue colour is: RAL
    5001". http://www.tube-tech.com/general-tube-tech-faq/
12. Tube-Tech, "CL 2A Dual Opto Compressor". https://www.tube-tech.com/cl-2a-dual-opto-compressor/
13. Tube-Tech, "CM1A Optical Tube Amplified Compressor (Discontinued)": "exactly the same tube circuit as our
    famous CL 1B ... but the action is different due to a different, carefully selected optical element".
    https://www.tube-tech.com/cm1a-optical-tube-amplified-compressor/
14. Tube-Tech, "CL 1B Plugin": the TC Electronic and Softube history, and the link to Universal Audio.
    https://www.tube-tech.com/cl-1b-plugin/
15. Softube, "Tube-Tech CL 1B & CL 1B Mk II Compressors" user manual: Petersen's foreword, the control
    descriptions, the −18 dBFS calibration, the Mk II additions, and the admission that "the panel print isn't
    very exact". https://www.softube.com/user-manuals/tube-tech-cl-1b-and-cl-1b-mk-ii-compressors
16. Softube, "Tube-Tech CL 1B mk II" product page: component modelling, the Generation Switch, 2006 and 2018.
    https://www.softube.com/plug-ins/tube-tech-cl-1b-mk-ii
17. Softube, "Tube-Tech Complete Collection 2": "officially licensed", parallel blend, side-chain low cut.
    https://www.softube.com/plug-ins/tube-tech-complete-collection-2
18. Universal Audio, "Tube-Tech CL 1B MkII Compressor" and its customer review corpus: "fully endorsed by
    Tube-Tech, Denmark", and the criticism quoted in section 8.
    https://www.uaudio.com/products/tube-tech-cl-1b-mkii-compressor
19. Bob Thomas, "Warm Audio WA-1B: Valve Optical Compressor", Sound On Sound, December 2023. The best English
    technical writing on the CL 1B topology, because the WA-1B is a clone: the 1987/1991 lineage, the GRE in the
    audio path, the valve complement, the inferred timings, and the 3U height.
    https://www.soundonsound.com/reviews/warm-audio-wa-1b
20. Felix Klostermann, "Lydkraft Tube-Tech CL 1B Test", Bonedo, 16 November 2013, 5/5. The one full hardware
    review I could reach: 3U, 4.8 kg, ECC83 and ECC82, the semiconductor-free audio path, "Skalpell als Axt",
    the recommendation against drums, and a large set of before-and-after audio examples.
    https://www.bonedo.de/artikel/lydkraft-tube-tech-cl-1b-test/
21. Attack Magazine, "Top 20 Best Hardware Compressors Ever Made": the CL 1B at number 4. **Cited for character
    only: it dates the CL 1B to 1987, which is the CL 1A's year and is wrong.**
    https://www.attackmagazine.com/reviews/the-best/top-20-best-hardware-compressors-ever-made/17/
22. ProSoundWeb, "Audio Engineer Richard Furch Mixes Tyrese Gibson With Tube-Tech": the "satin glove" and
    "blanket of awesome" quotes and Furch's settings.
    https://www.prosoundweb.com/audio-engineer-richard-furch-mixes-tyrese-gibson-with-tube-tech/
23. Tube-Tech, "Top Mixer Richard Furch Uses CL 1B": the same story on the maker's own site.
    http://tube-tech.com/top-mixer-richard-furch-uses-cl-1b/
24. ProSoundWeb, "Scotty Simpson On The Road With Tube-Tech And The Oak Ridge Boys": the bass settings quoted
    in 1.4 and 2.7. https://www.prosoundweb.com/scotty-simpson-on-the-road-with-tube-tech-and-the-oak-ridge-boys/
25. Tube-Tech, "Scotty Simpson Takes the CL1B on the Road".
    https://www.tube-tech.com/scotty-simpson-takes-the-cl1b-on-the-road/
26. Tube-Tech, "CL1B Is the One and Only for Kanye West": Yeezus, 2013, engineer Noah Goldstein.
    https://www.tube-tech.com/cl1b-is-the-one-and-only-for-kanye-west/
27. Tube-Tech, "Marcello Spiridioni | The Italian Mastering Engineer of Reference".
    https://www.tube-tech.com/marcello-spiridioni-the-italian-mastering-engineer-of-reference/
28. Sound On Sound, "Inside Track: Lil Uzi Vert": Kesha Lee on the Atlanta standard signal chain.
    https://www.soundonsound.com/techniques/inside-track-lil-uzi-vert
29. Gearspace, "CL1B or LA2A": the great-singers quote, the less-distortion comparison, and the unit-to-unit
    consistency contrast with the LA-2A. https://gearspace.com/threads/cl1b-or-la2a.892647/
30. Gearspace, "Tubetech CL1B worth it": opto-then-FET convention, male versus female vocals, "euphonic".
    https://gearspace.com/threads/tubetech-cl1b-worth-it.901825/
31. Gearspace, "Retro Sta-Level or CL1B for vocals in really dense mixes".
    https://gearspace.com/threads/retro-sta-level-or-cl1b-for-vocals-in-really-dense-mixes.1202193/
32. Gearspace, "LA2A or Tube-Tech CL1B": the 80/20 split on vocalists.
    https://gearspace.com/threads/la2a-or-tube-tech-cl1b.486618/
33. Gearspace, "Help: LA2A vs CL1B clips, have to decide within days": a blind clip comparison in which
    listeners identified the CL 1B as the brighter, cleaner sample.
    https://gearspace.com/threads/help-la2a-vs-cl1b-clips-have-to-decide-within-days.1183326/
34. Gearspace, "Avalon 737 vs CL1B": the dissenting view, and valve swapping as the maintenance lever.
    https://gearspace.com/threads/avalon-737-vs-cl1b.1448863/
35. Gearspace, "Tubetech CL1B vs Softube CL1B MkII": a hardware owner's comparison with downloadable files.
    https://gearspace.com/threads/tubetech-cl1b-vs-softube-cl1b-mkii.1326347/
36. Gearspace, "CL 1B: what does non-semiconductor opto element mean?": the thread that asks what is inside the
    GRE and does not find out. https://gearspace.com/threads/cl-1b-what-does-non-semiconductor-opto-element-mean.499932/
37. Gearspace, "Puzzled newbie: CL1B clone": "the only thing this schematic does not reveal is the proprietary
    opto cell circuit and parts", and the tertiary output-transformer winding as a second barrier to cloning.
    https://gearspace.com/threads/puzzled-newbie-cl1b-clone.1340949/
38. Gearspace, "Holy sh*t, you see that Tube-Tech CL1B price increase?": the dated $3190 to $3690 step.
    https://gearspace.com/threads/holy-sh-t-you-see-that-tube-tech-cl1b-price-increase.919694/
39. Gearspace, "What is with the Tube-Tech CL1B prices?": a dealer's MAP figure and the earlier used market.
    https://gearspace.com/threads/what-is-with-the-tube-tech-cl1b-prices.441060/
40. Warm Audio, "WA-1B": "Scandinavian", "1B-style", the fix/man description and the bus selector.
    https://www.warmaudio.com/wa-1b
41. Stam Audio, "SA-1B": "a meticulously reverse engineered replica of the legendary Danish 1B opto
    compressor", plus the added dry/wet blend and five-position sidechain high-pass.
    https://stamaudio.com/shop/compressors/sa-1b/
42. Kiive Audio, "KC1 | Tube Compressor": an unlicensed software model that does not name its subject.
    https://kiiveaudio.com/products/kc1
43. Thomann, "Tube-Tech CL 1B": current retail price and stock. https://www.thomann.de/gb/tube_tech_cl_1b.htm
44. Audiofanzine, "Tube-Tech CL1B" user reviews.
    https://en.audiofanzine.com/studio-compressor/tube-tech/CL1B/user_reviews/
45. RAL Colour Chart, "RAL 5001 Green blue": the swatch sampled in 2.3.
    https://www.ralcolorchart.com/ral-classic/ral-5001-green-blue
46. Tube-Tech, "CL 1B Sound Demos": Lydkraft's own before-and-after files on vocals, guitar and bass.
    https://www.tube-tech.com/cl-1b-sound-demos/
47. Tube-Tech, "Taking The Mix of X-FACTOR to Another Level": the CL 1B on lead vocals for the Danish X-Factor.
    https://www.tube-tech.com/taking-the-mix-of-x-factor-to-another-level/
48. Tube-Tech, "TUBE-TECH at Full Sail University in Orlando".
    https://www.tube-tech.com/tube-tech-at-full-sail-university-in-orlando/
49. Tube-Tech, DISCONTINUED product tag: the CM 1A's discontinuation and the rest of the retired range.
    http://www.tube-tech.com/tag/discontinued/
50. Tube-Tech, "TUBE-TECH Reviews": the review index, which links to Sweetwater user reviews for the CL 1B and
    to no magazine review at all, which is itself evidence for 6.3. http://www.tube-tech.com/tube-tech-reviews/
51. KVR Audio, "Tube-Tech CL 1B mk II by Softube": version numbering and user reviews.
    https://www.kvraudio.com/product/tube-tech-cl-1b-mk-ii-by-softube
52. Softube, release notes index. https://www.softube.com/release-notes
53. Gearspace, "Warm Audio introduces the WA-1B compressor": reproduces Warm Audio's launch copy in full.
    https://gearspace.com/threads/warm-audio-introduces-the-wa-1b-compressor.1415269/
54. Texas Instruments, LF347 wide-bandwidth quad JFET-input operational amplifier datasheet: U1 and U2 of the
    sidechain. https://www.ti.com/lit/ds/symlink/lf347.pdf
55. onsemi, BC337 NPN transistor datasheet: Q1, the GRE current sink.
    https://www.onsemi.com/pdf/datasheet/bc337-d.pdf
56. Vishay, BF245A N-channel JFET datasheet: Q2, in the GRE servo loop.
    https://www.vishay.com/docs/70216/bf245.pdf
57. ECC83 (12AX7) double triode datasheet: V1, the voltage amplifier and cathodyne splitter.
    https://frank.pocnet.net/sheets/030/e/ECC83.pdf
58. ECC82 (12AU7) double triode datasheet: V2, the push-pull output stage.
    https://frank.pocnet.net/sheets/030/e/ECC82.pdf
59. Clairex Corporation, photoconductive cells catalogue: rise and decay against illumination, and the gamma
    convention reused for the GRE's static law. Internet Archive full text.
    https://archive.org/stream/TNM_Clairex_photoconductive_cells_-_Clairex_Corp_20180514_0017/TNM_Clairex_photoconductive_cells_-_Clairex_Corp_20180514_0017_djvu.txt
60. PerkinElmer Optoelectronics, "Photoconductive Cells" application note: rise and decay definitions, speed
    against light level, light history, gamma.
    https://cdn-learn.adafruit.com/assets/assets/000/010/129/original/APP_PhotocellIntroduction.pdf
61. Wikipedia, "Resistive opto-isolator": turn-on against turn-off asymmetry, and distortion against the voltage
    across the cell. https://en.wikipedia.org/wiki/Resistive_opto-isolator
62. GL5528 CdS photoconductive cell datasheet: the gamma definition and typical values behind `CELL_GAMMA`.
    https://pi.gate.ac.uk/pages/airpi-files/PD0001.pdf
63. F. Eichas and U. Zölzer, "Modeling of an Optocoupler-Based Audio Dynamic Range Control Circuit", SPIE 9948,
    2016. https://www.hsu-hh.de/ant/wp-content/uploads/sites/699/2017/10/Eichas-Modeling-of-an-optocoupler-based-audio-dynamic-range-control-circuit-99480W.pdf
64. F. Eichas, E. Gerat and U. Zölzer, "Virtual Analog Modeling of Dynamic Range Compression Systems", AES 142,
    2017. https://aes.org/publications/elibrary-page/?id=18628
65. J. Najnudel, R. Müller, T. Hélie and D. Roze, "Power-Balanced Dynamic Modeling of Vactrols", DAFx23.
    https://www.dafx.de/paper-archive/2023/DAFx23_paper_50.pdf
66. A. Wright and V. Välimäki, "Grey-Box Modelling of Dynamic Range Compression", DAFx20in22.
    https://www.dafx.de/paper-archive/2022/papers/DAFx20in22_paper_35.pdf
67. D. Giannoulis, M. Massberg and J. D. Reiss, "Digital Dynamic Range Compressor Design: A Tutorial and
    Analysis", JAES 60(6), 2012: the feedback-topology analysis behind section 3.4's ratio formula, and the
    knee and curve conventions. https://www.aes.org/e-lib/download.cfm?ID=16354
68. U. Zölzer (ed.), "DAFX: Digital Audio Effects", Wiley: the dynamics chapter, and the detector conventions.
    https://www.dafx.de/DAFX_Book_Page_2nd_edition/index.html
69. C. J. Steinmetz and J. D. Reiss, "Efficient neural networks for real-time modeling of analog dynamic range
    compression", AES 152, 2022. Listed because it is the obvious alternative approach and I am not taking it.
    https://ar5iv.labs.arxiv.org/html/2102.06200
70. R. Simionato and S. Fasciani, "Modeling Time-Variant Responses of Optical Compressors with Selective State
    Space Models", JAES 73(3), 2025. https://arxiv.org/html/2408.12549
71. A. Moore, "Objective Analysis and Perceptual Evaluation of LA-2A Compressors and Vocal Recordings",
    University of Huddersfield: the six-unit LA-2A measurement study whose 0.9-4.2 % distortion range test 34
    compares against. https://pure.hud.ac.uk/ws/portalfiles/portal/140787498/AAM.pdf
72. Sound On Sound, "Classic Compressors": the standing reference for feedback versus feed-forward topology.
    https://www.soundonsound.com/techniques/classic-compressors
73. Analog Devices, "Precision Full-Wave Rectifier": the 20 k : 10 k : 10 k : 20 k absolute-value topology the
    CL 1B's detector uses. Cited as the textbook circuit, not as a Lydkraft source.
    https://www.analog.com/en/resources/technical-articles/precision-fullwave-rectifier-dual-supply.html
74. Prism Sound glossary, "VU Meter": the ANSI C16.5-1942 ballistics used in test 25.
    http://www.prismsound.com/define.php?term=VU_Meter
75. EDN, "Analog VU Meters & Quick Pointers". https://www.edn.com/analog-vu-meters-quick-pointers/
76. Audio Science Review, forum search for Tube-Tech and CL 1B, returning no substantive results. Cited as the
    evidence for 6.3's negative finding, not for any figure.
    https://www.audiosciencereview.com/forum/index.php?search/search&keywords=Tube-Tech+CL+1B

[1]: https://www.tube-tech.com/cl-1b-opto-compressor/
[2]: http://www.tube-tech.com/wp-content/uploads/2020/05/Tube-Tech-Manual-CL-1B-200513.pdf
[3]: http://www.tube-tech.com/wp-content/uploads/2017/11/cl1bspecs.pdf
[4]: http://www.tube-tech.com/wp-content/uploads/2017/11/Brochure-CL1B.pdf
[5]: http://www.tube-tech.com/wp-content/uploads/2017/11/RE-CL1B.pdf
[6]: http://www.tube-tech.com/wp-content/uploads/2017/11/CL1BFront.jpg
[7]: http://www.tube-tech.com/wp-content/uploads/2017/11/CL-1B-Rear.jpg
[8]: http://www.tube-tech.com/wp-content/uploads/2017/11/cl1bny2.jpg
[9]: https://funkwerkes.com/web/wp-content/techdocs/MixedProAudio/Tube-Tech-CL1B-Compressor-SM.pdf
[10]: http://www.tube-tech.com/the-lydkraft-story/
[11]: http://www.tube-tech.com/general-tube-tech-faq/
[12]: https://www.tube-tech.com/cl-2a-dual-opto-compressor/
[13]: https://www.tube-tech.com/cm1a-optical-tube-amplified-compressor/
[14]: https://www.tube-tech.com/cl-1b-plugin/
[15]: https://www.softube.com/user-manuals/tube-tech-cl-1b-and-cl-1b-mk-ii-compressors
[16]: https://www.softube.com/plug-ins/tube-tech-cl-1b-mk-ii
[17]: https://www.softube.com/plug-ins/tube-tech-complete-collection-2
[18]: https://www.uaudio.com/products/tube-tech-cl-1b-mkii-compressor
[19]: https://www.soundonsound.com/reviews/warm-audio-wa-1b
[20]: https://www.bonedo.de/artikel/lydkraft-tube-tech-cl-1b-test/
[21]: https://www.attackmagazine.com/reviews/the-best/top-20-best-hardware-compressors-ever-made/17/
[22]: https://www.prosoundweb.com/audio-engineer-richard-furch-mixes-tyrese-gibson-with-tube-tech/
[23]: http://tube-tech.com/top-mixer-richard-furch-uses-cl-1b/
[24]: https://www.prosoundweb.com/scotty-simpson-on-the-road-with-tube-tech-and-the-oak-ridge-boys/
[25]: https://www.tube-tech.com/scotty-simpson-takes-the-cl1b-on-the-road/
[26]: https://www.tube-tech.com/cl1b-is-the-one-and-only-for-kanye-west/
[27]: https://www.tube-tech.com/marcello-spiridioni-the-italian-mastering-engineer-of-reference/
[28]: https://www.soundonsound.com/techniques/inside-track-lil-uzi-vert
[29]: https://gearspace.com/threads/cl1b-or-la2a.892647/
[30]: https://gearspace.com/threads/tubetech-cl1b-worth-it.901825/
[31]: https://gearspace.com/threads/retro-sta-level-or-cl1b-for-vocals-in-really-dense-mixes.1202193/
[32]: https://gearspace.com/threads/la2a-or-tube-tech-cl1b.486618/
[33]: https://gearspace.com/threads/help-la2a-vs-cl1b-clips-have-to-decide-within-days.1183326/
[34]: https://gearspace.com/threads/avalon-737-vs-cl1b.1448863/
[35]: https://gearspace.com/threads/tubetech-cl1b-vs-softube-cl1b-mkii.1326347/
[36]: https://gearspace.com/threads/cl-1b-what-does-non-semiconductor-opto-element-mean.499932/
[37]: https://gearspace.com/threads/puzzled-newbie-cl1b-clone.1340949/
[38]: https://gearspace.com/threads/holy-sh-t-you-see-that-tube-tech-cl1b-price-increase.919694/
[39]: https://gearspace.com/threads/what-is-with-the-tube-tech-cl1b-prices.441060/
[40]: https://www.warmaudio.com/wa-1b
[41]: https://stamaudio.com/shop/compressors/sa-1b/
[42]: https://kiiveaudio.com/products/kc1
[43]: https://www.thomann.de/gb/tube_tech_cl_1b.htm
[44]: https://en.audiofanzine.com/studio-compressor/tube-tech/CL1B/user_reviews/
[45]: https://www.ralcolorchart.com/ral-classic/ral-5001-green-blue
[46]: https://www.tube-tech.com/cl-1b-sound-demos/
[47]: https://www.tube-tech.com/taking-the-mix-of-x-factor-to-another-level/
[48]: https://www.tube-tech.com/tube-tech-at-full-sail-university-in-orlando/
[49]: http://www.tube-tech.com/tag/discontinued/
[50]: http://www.tube-tech.com/tube-tech-reviews/
[51]: https://www.kvraudio.com/product/tube-tech-cl-1b-mk-ii-by-softube
[52]: https://www.softube.com/release-notes
[53]: https://gearspace.com/threads/warm-audio-introduces-the-wa-1b-compressor.1415269/
[54]: https://www.ti.com/lit/ds/symlink/lf347.pdf
[55]: https://www.onsemi.com/pdf/datasheet/bc337-d.pdf
[56]: https://www.vishay.com/docs/70216/bf245.pdf
[57]: https://frank.pocnet.net/sheets/030/e/ECC83.pdf
[58]: https://frank.pocnet.net/sheets/030/e/ECC82.pdf
[59]: https://archive.org/stream/TNM_Clairex_photoconductive_cells_-_Clairex_Corp_20180514_0017/TNM_Clairex_photoconductive_cells_-_Clairex_Corp_20180514_0017_djvu.txt
[60]: https://cdn-learn.adafruit.com/assets/assets/000/010/129/original/APP_PhotocellIntroduction.pdf
[61]: https://en.wikipedia.org/wiki/Resistive_opto-isolator
[62]: https://pi.gate.ac.uk/pages/airpi-files/PD0001.pdf
[63]: https://www.hsu-hh.de/ant/wp-content/uploads/sites/699/2017/10/Eichas-Modeling-of-an-optocoupler-based-audio-dynamic-range-control-circuit-99480W.pdf
[64]: https://aes.org/publications/elibrary-page/?id=18628
[65]: https://www.dafx.de/paper-archive/2023/DAFx23_paper_50.pdf
[66]: https://www.dafx.de/paper-archive/2022/papers/DAFx20in22_paper_35.pdf
[67]: https://www.aes.org/e-lib/download.cfm?ID=16354
[68]: https://www.dafx.de/DAFX_Book_Page_2nd_edition/index.html
[69]: https://ar5iv.labs.arxiv.org/html/2102.06200
[70]: https://arxiv.org/html/2408.12549
[71]: https://pure.hud.ac.uk/ws/portalfiles/portal/140787498/AAM.pdf
[72]: https://www.soundonsound.com/techniques/classic-compressors
[73]: https://www.analog.com/en/resources/technical-articles/precision-fullwave-rectifier-dual-supply.html
[74]: http://www.prismsound.com/define.php?term=VU_Meter
[75]: https://www.edn.com/analog-vu-meters-quick-pointers/
[76]: https://www.audiosciencereview.com/forum/index.php?search/search&keywords=Tube-Tech+CL+1B

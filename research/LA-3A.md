# The LA-3A audio leveler: research notes for the LA-3A side of `noob-compressorlab`

Research dossier for the LA-3A model of the `noob-compressorlab` example plug-in of noob-vst-webgui-framework.
The example is a humorous, affectionate spoof of the Teletronix / UREI / Universal Audio LA-3A Audio Leveler. It
is not a product and does not use the LA-3A, Teletronix, UREI or Universal Audio names as its own name.
Trademarks below belong to their owners and are used only to identify the device and the products discussed.
This model sits behind the same per-instance `model` switch that already selects the 1176, LA-2A and Distressor
behaviours; see [[1176]], [[LA-2A]] and [[Distressor]].

Conventions (kept the same as the 1176, LA-2A and Distressor dossiers so they all read alike):

- Citations are `[n]`; the numbered list in section 9 gives the URL for every source, and reference-style link
  definitions at the very end make the `[n]` markers clickable.
- Numbers that come from a manufacturer specification, a manual, a schematic or a measurement are attributed.
  Numbers that are my own derivation or assumption are labelled **estimate** or **derived**. Nothing labelled a
  measurement was invented; where sources disagree, both figures are given, and on this device they disagree a
  lot.
- "GR" is gain reduction. "PR" is the Peak Reduction control. "THD" is total harmonic distortion. dBm in the old
  UREI documents means dBu into 600 ohms. dBFS is digital full scale.
- The LA-3A is a spoof target, not a parity goal. I want the *feel* of the thing: the fast grab, the two-stage
  let-go, the way it hears the top end before the bottom, and the mid-forward push people keep describing. I do
  not want a component-accurate clone, and I am not trying to beat anybody's plug-in.
- One piece of good luck makes this model cheap to build: the LA-3A uses the **same T4B electro-optical
  attenuator as the LA-2A**. The cell simulation in `src/dsp/opto/model.rs` is therefore reused as-is, and only
  the drive, the amplifier and the output stage change. Section 7 says exactly what is shared and what is not.

---

## 1. What the LA-3A is

### 1.1 Origin

- The **Teletronix LA-3A Audio Leveler** made its debut at the **1969 New York AES show** and marked the
  departure from the tube design of the LA-2A. Universal Audio's own history says it "incorporated components
  and design concepts from UA's solid-state driven 1176LN Limiting Amplifier, while also harnessing the LA-2A's
  optical compression design, giving the LA-3A its own distinctive sound and expanded versatility". [4]
- UA credits the design to **Brad Plunkett**, released 1969. Plunkett is the same engineer who invented the
  wah-wah pedal and who designed the low-noise modification that turned the 1176 into the 1176**LN**. [5] [31]
  (The 1176LN provenance is in the LA-3A hardware manual's own historical notes: "The first major modification
  to the 1176 circuit was designed by Brad Plunkett in an effort to reduce noise -- hence the birth of the
  1176LN". [1]) UA's blog frames the LA-3A as "essentially a solid-state iteration of the tube-driven LA-2A with
  a significantly faster attack capability", giving "smooth, optical compression, but with the aggression of a
  transistor circuit". [5]
- The original datasheet is blunter and less romantic: "The LA-3A Solid State Leveling Amplifier is the solid
  state successor to the well known Teletronix LA-2A. The unique characteristics of the T4A electro-optical
  attenuator have been maintained." [2] Note that the same page then says the unit uses a **T4B**, not a T4A;
  the two sentences contradict each other and I flag it in section 3.7.
- Corporate context, from the LA-2A side of the story: Jim Lawrence's Teletronix was sold to **Babcock
  Electronics** in 1965; in **1967** Bill Putnam's Studio Electronics (renamed **UREI** shortly after) bought
  Babcock's broadcast division including the Teletronix brand. LA-2A production stopped **around 1969**, as the
  solid-state LA-3A, LA-4 and LA-5 took over. [32] [34] [35] The LA-3A is therefore the machine that killed the
  LA-2A, which is a good joke to keep in the tribute.
- Selling points on the original sheet were practical, not sonic: "Improvements in overload characteristics and
  signal to noise ratio", and "The new 1/2 rack size allows installation of two LA-3A Leveling Amplifiers in only
  3 1/2 inches of rack space. Contemporary styling complements existing studio equipment." [2] The unit was
  designed for broadcast; Tape Op's reissue review notes it "became central to the classic rock sound of 1970s
  FM radio". [11]

### 1.2 Versions, badges and how to tell them apart

There is no revision cult here of the kind the 1176 has ([[1176]] section 1.2). A Gearspace regular who has
owned several put it plainly: "I could be wrong but I was under the impression an LA3A was an LA3A. They didn't
change much. And if they did, it didn't sonically matter." [19] With that caveat, the badges people do see:

| Era | Faceplate | Notes | Sources |
|---|---|---|---|
| 1969 onward, Teletronix / UREI | UREI logo top centre; "LEVELING AMPLIFIER" over "TELETRONIX LA-3A" bottom centre | The datasheet imprint I have was distributed by Taber Manufacturing and Engineering, San Leandro; the UREI footer reads "UNITED RECORDING ELECTRONICS INDUSTRIES, 11922 Valerio Street, No. Hollywood, California 91605" | [2] |
| 1970s, UREI | Same layout, "UREI / Universal Audio" wording instead of Teletronix on some units | Owners report both wordings and cannot correlate either to a sound | [19] |
| Late UREI / JBL-Harman era | "Universal Audio" bottom centre | Same observation | [19] |
| 2005 onward, Universal Audio reissue | Silkscreen "UNIVERSAL AUDIO, INC. SANTA CRUZ, CALIFORNIA" at the bottom; two-position GR / OUTPUT meter switch; POWER ON | Adds the **MOD** gain switch and an IEC inlet with XLRs alongside the barrier strip | [1] [7] [11] |

Within-era variation is real but mechanical rather than architectural. One long-time user: "I've used a bunch of
different vintage LA-3As. They are all a little bit different. Some get gain reduction easier than others, some
are on the bright or aggressive side, some are more chilled out ... Some of them have a built in de-esser, some
have the comp/limit switch on the front, some have the stereo/mono controls." [19] The schematic I worked from
is UREI drawing **C11186, issue E, dated 4-14-70**, and it carries five revision notes of its own, of which the
only one with a stated value is "'A' R13 WAS 82K" (the gain-reduction meter matching resistor, now 68 kΩ). [3]

Clones and descendants that come up constantly and are worth knowing exist, because they set the community's
expectations of what an LA-3A *is*: Serpent Audio **SA-3A**, LaZ Electronics **LA3A** and its 500-series
version, Golden Age Project **Comp-3A**, AudioScape **V3A** (500 series), Anthony DeMaria Labs **ADL 1600**,
J-Labs **JLA-3**, ADK **CLA-1**, and Stam Audio's take. [12] [19] [69]

### 1.3 What it kept from the LA-2A, and what it threw away

This is the whole point of the model, so it gets its own table. Details and citations are in sections 3 and 4;
the short version is the one the forums give whenever somebody asks [73].

| Element | LA-2A | LA-3A |
|---|---|---|
| Gain element | T4 / T4B cell shunting a resistive divider | **the same T4B cell, same job** [2] [3] |
| Detector topology | feedback (sidechain fed from the gain-reduced signal) | **feedback**, explicitly: "The LA-3A is a feedback style compressor" [1] |
| Audio amplifier | 12AX7A voltage amp, 12BH7A cathode follower, ~275 V rails | dual **2N5089** differential pair, 2N5087 drivers, complementary **2N3053 / 2N4037** class-AB output on a single ~25 V rail [3] |
| Sidechain amplifier | 12AX7A into a 6AQ5A driving the EL panel directly from a high-voltage rail | **2N5089** into a discrete driver, coupled through an **autotransformer (B11184)** that steps ~10 V up to the panel [3] [20] |
| Sidechain shaping | R37 "limit response" trimmer, flat by default | **HF CONTOUR** trimmer plus a fixed low-frequency roll-off that is there whatever the trimmer does [1] [17] |
| Attack | 10 ms typical | **1.5 ms or less** (UA) or **250 µs to 0.5 ms** (UREI) [1] [2] |
| Release | 0.06 s to 50 %, then 0.5-5 s | **identical wording**: 60 ms to 50 %, then 0.5-5 s [1] [2] |
| Ratio | about 3:1 Compress, ∞:1 Limit | about 3:1 Compress, ∞:1 Limit (UA) or "approaching 50:1" (UREI) [1] [4] [2] |
| Gain | 40 dB ±1 dB | **50 dB or 30 dB ±1 dB**, switched at the rear [2] |
| Max output | +10 dBm nominal, +16 dBm peaks | **+24 dBm (+27 dBm peaks)** [2] |
| Bandwidth | +0 / −1 dB, 30 Hz-15 kHz | **±1 dB, 20 Hz-20 kHz** (UREI) or ±0.5 dB (UA) [1] [2] |
| Size | 19 inch, 3.5 inch, heavy | **half rack**, 3.5 inch, 6.5 lb [1] [2] |

Read that table as one sentence: **the cell stayed, everything around it got faster, louder, wider and
smaller**. That is the model, and it is why the framework's existing T4 cell is the right starting point.

### 1.4 Why it is famous

- It is the one people reach for when the LA-2A is too slow and the 1176 is too obvious. UA's marketing does not
  even pretend otherwise: "faster limiting than the tube-driven smooth operator Teletronix LA-2A with the
  clarity of the iconic solid-state 1176", "bold, mid-forward compression". [6] [77] A Gearspace thread title puts
  the same thought as a question: "Is an LA-3A the perfect midpoint between LA-2A and 1176?" [67]
- Its faster attack made it work where the LA-2A did not: UA lists "cymbals, room mics, drums, and percussion"
  and calls it a "'secret weapon' compressor with a unique character capable of moving sounds right to the front
  of your speakers". [4] [5]
- Craig Schumacher, reviewing the reissue in Tape Op 49 (Sep/Oct 2005): "less squishy and a lot more
  transparent" than the LA-2A; "you can make this thing compress like crazy, and it never sounds overdone"; on
  lead vocals, "when you get it right, you don't have to ride the fader at all during mixdown"; and it "works
  fantastically on bass, guitar, piano, vocals, and pretty much anything you put through it". [11]
- Electric guitar is the folk-use it is best known for, to the point that one of the more-read Gearspace threads
  about it is titled "WTF is the UA LA3A doing to my guitar tracks?" [68] Joe Chiccarelli is quoted by UA on its
  hybrid tube-versus-solid-state character; UA's artist preset list for the plug-in includes Mark Needham,
  Damian Taylor, Eric J Dubowsky, Joe Chiccarelli, Chris Coady and Chuck Zwicky. [4] [5]
- Chris Lord-Alge "considers these among his favorites of all vintage compressors", which is why Waves modelled
  his unit as the CLA-3A. [8]

---

## 2. Controls, the front panel and their real ranges

### 2.1 The front panel, close enough to draw

Described from the original UREI datasheet photograph and its single- and dual-rack mounting drawings [2], and
from the UA reissue manual's front-panel line drawing and recall sheet [1]. Proportions are my estimates scaled
to the stated 8.5 inch panel width; UREI's stated size is **3.5 inch vertical, 8.5 inch horizontal, 9.25 inch
depth behind the panel, 6.5 pounds** [2] (UA repeats 2RU, half of 19 inch, 9.25 inch depth, 6.5 lb [1]).

**Faceplate.** A flat **black** panel, half rack width, two rack units tall, with four visible cover screws
along the top edge of the chassis behind it. All lettering and scale marks are **white**, a plain condensed
sans-serif, mostly small. The look is deliberately plain: no bezel around the panel, no chrome, no colour except
the meter.

**Top row, left to right.**

- **GAIN**, upper-left. A round light-cream/white knob with a single pointer notch, roughly 0.85 inch diameter
  (estimate). The word "GAIN" is silkscreened in small caps *above* the knob. The scale is printed on the panel
  around the knob, not on the knob: **0 at the lower left, then 1, 2, 3, 4, 5 at top dead centre, 6, 7, 8, 9,
  10 at the lower right**, with small dots between the numerals. About 300 degrees of rotation.
- **The meter**, centre. A rectangular VU meter, roughly 2.4 by 1.5 inch (estimate), with a **cream/white
  face**, a black arc scale and a black pointer that rests at the left. Standard VU markings: −20 to 0 in black
  with the usual crowding, 0 to +3 in red, "VU" in a small box below the arc. Two lamps behind the face
  (section 3.9). Above the meter, centred on the panel, the **UREI logo** (the squared-off "UREI" wordmark).
- **PEAK REDUCTION**, upper-right. Identical knob and identical 0-10 scale, with "PEAK REDUCTION" silkscreened
  above it on two lines, "PEAK" over "REDUCTION".

**Bottom row, left to right.**

- **GR — [toggle] — OUTPUT**, lower-left, under the Gain knob. A small bat-handle toggle with "GR" to its left
  and "OUTPUT" to its right, both in small caps. This is the whole meter switch: **two positions, not three.**
- **LEVELING AMPLIFIER** over **TELETRONIX LA-3A**, bottom centre, in two lines; the first line lighter, the
  second bolder and letter-spaced. On the UA reissue this block is replaced by **UNIVERSAL AUDIO, INC. SANTA
  CRUZ, CALIFORNIA** in a single small line. [1] [2]
- **POWER — [toggle] — ON**, lower-right, under the Peak Reduction knob. Same small toggle, "POWER" to its left
  and "ON" to its right. On the UA reissue, the **GR zero-set trim pot** peeks through "a small hole on the
  front panel just above the P of the word POWER". [1] I love that detail and the tribute should keep it.

**A correction I have to make to my own brief.** I went looking for a three-position GR / +10 / +4 meter switch
because that is what the LA-2A has, and it is not there. Both the 1969 datasheet photo [2] and the UA reissue
manual [1] show a **two-position GR / OUTPUT** toggle, and in the OUTPUT position "a meter reading of 0
corresponds to an output level of **+4 dBm** at the LA-3A output" [1]. There is no +10 position on the LA-3A.
The UA plug-in adds a third position, **Off**, which bypasses processing and dims the meter, and Waves' CLA-3A
uses In / GR / Out instead. [4] [8] I keep GR / Output / Off in the model and say why.

**Rack accessories.** Single-unit kit **SR-3A** puts one LA-3A in a 19 inch, 3.5 inch panel with a blank half;
dual kit **DR-3A** straps two together. [2] UA ships the brackets in the box and sells the dual strapping plates
separately. [1] For the tribute's page this matters only as a joke: the face is half a rack, so it can sit next
to the 1176's face.

### 2.2 The rear panel

The original rear panel, read from the datasheet photograph [2] and confirmed by the reissue manual's
description [1]. Left to right as you look at the back:

- **Left**: the mains transformer standing proud of the chassis, the fuse holder, and the voltage-selector
  slide. Silkscreen: **"115V 3AG 1/8 AMP S.B."** above **"230V 3AG 1/16 AMP S.B."**. The reissue adds a standard
  IEC inlet. [1] [2]
- **Middle, upper**: **STEREO ADJ**, a screwdriver-slot pot with an arc marked and **MONO** at the
  counter-clockwise end.
- **Middle, lower**: **HF CONTOUR**, a second screwdriver-slot pot with **FLAT** marked at one end and an arrow.
- **Middle, right**: a **GAIN** slide switch marked **30dB** and **50dB**, and a **COMPRESS / LIMIT** slide
  switch. On the reissue these are joined by the **MOD** switch.
- **Right**: the Jones barrier terminal strip, six screws, silkscreened **IN, COM, CHASSIS, STEREO, COM, OUT**.
  The reissue adds XLR **LINE INPUT** and **LINE OUTPUT** with pin 2 hot, keeps the strip, and warns that "the
  600 ohm resistor across screws 5 & 6 must not be removed for proper operation of the output terminals AND the
  Output XLR jack". [1] Owners rediscovering these switches years after buying the unit is a recurring forum
  genre [71].

### 2.3 Control table

| Control | Where | Hardware | Range and behaviour | Sources |
|---|---|---|---|---|
| **Gain** | front | R4, 100 kΩ pot after the attenuator, feeding the amplifier | Make-up only; "does not affect the amount of compression". Knob 0-10 is arbitrary and "do not reflect any particular dB value". Maximum system gain 50 dB or 30 dB ±1 dB depending on the rear switch. | [1] [3] [4] [2] |
| **Peak Reduction** | front | R21, 100 kΩ pot at the sidechain input | "Controls the gain of the side-chain circuit. The greater the gain of this circuit, the lower the threshold and the greater the amount of compression." Sets threshold and amount together. Knob 0-10, arbitrary, non-linear. At minimum, "no compression (or limiting) occurs but the signal is still colored by the circuitry". | [1] [4] [8] |
| **Meter switch** | front | S3, two-position toggle | **GR**: needle rests at 0 and swings left, showing gain reduction in dB. **OUTPUT**: 0 VU = **+4 dBm** at the output. | [1] [2] |
| **Power** | front | S1 toggle with the mains fuse F1 | On/off. Two 1819 lamps light the meter. | [3] [18] |
| **GR zero set** | front (reissue), through a hole above the "P" of POWER | trim pot | Warm up 5 minutes, meter to GR, Peak Reduction fully counter-clockwise, adjust for 0 dB. | [1] |
| **Comp / Limit** | rear (front on some units) | slide switch on the divider tap | "When in the COMPRESS position, the curve is gentler, and presents a low compression ratio. A higher compression ratio results when the switch is set to the LIMIT position. **The difference in these two modes is only present when the LA-3A is in deep compression.** Most users leave the LA-3A in COMPRESS mode." UREI: limiter is "a compression ratio approaching 50:1". | [1] [2] [19] |
| **50 / 30 dB Gain** | rear | 20 dB input T-pad (R41 510 Ω, R42 510 Ω, R43 130 Ω) around the input transformer | "An input pad to prevent high output devices from overdriving the input transformer. Setting the switch to the 30 dB position will attenuate the incoming signal by 20 dB." Threshold of limiting moves with it: **−10 dBm at 30 dB, −30 dBm at 50 dB**. | [1] [2] [3] |
| **MOD** | rear, reissue only | switchable version of a common vintage modification | "Disengages the 20 dB input pad and drops the output amp's gain by 24 dB. In this configuration, the unit's gain is +26 dB and will be less noisy than when in normal mode with the 50/30 dB switch at 30. In addition, **the MOD switch lowers the threshold point where compression begins and allows a greater amount of overall compression**." | [1] [11] [17] |
| **Stereo Adj** | rear | R31, 10 kΩ pot in the sidechain driver | Balances the gain reduction of two linked units. Fully clockwise to start, then trim the unit reading the most GR until both meters agree at −5 dB. | [1] [3] |
| **HF Contour** | rear | R28, 10 kΩ pot in the sidechain amplifier's emitter network, with R29 1.5 kΩ and C7 4.7 nF | "A high frequency boost of the signal feeding the gain reduction circuit. This control is NOT an EQ of the audible program material ... At the full clockwise position, the LA-3A will attenuate 15 kHz **10 dB more** than the lower frequencies." Sense of rotation is disputed; see section 4.5. | [1] [2] [3] [13] [14] |

### 2.4 Manufacturer's starting points

UA's suggested defaults, which are also the sanest defaults for the plug-in: **Comp/Limit down (Compress); MOD
and 30/50 dB switches up; Stereo Adj and HF Contour fully counter-clockwise, as shipped.** Set Gain and Peak
Reduction to 0, bring Gain up to a working level, then raise Peak Reduction "until the LA-3A meter reads between
−3 and −5 on the meter during volume peaks". [1] UA repeats the −3 to −5 dB figure three times in the manual,
so I treat it as the intended operating point and calibrate the model's defaults to it.

Universal Audio also volunteers two "tricks" that are worth stealing for preset names: put an 1176 set fast in
front of an LA-3A (or the other way round), and split a signal, compress one copy at −5 to −10 dB, and blend.
[1] The second is exactly what the plug-in's Mix control does, and UA's own plug-in added a Dry/Wet Mix for the
same reason. [4]

### 2.5 What the plug-ins put on the panel

| Product | Controls | Notes | Sources |
|---|---|---|---|
| UA Teletronix LA-3A (UAD-2) / LA-3A Compressor (UADx) | Peak Reduction, Gain (0-10, arbitrary), Comp/Lim, Meter Select GR / Output / **Off**, **HF** set-screw, **Mix** set-screw | HF and Mix are additions; Mix "does not exist on the original hardware". Meter Off also disables processing. | [4] |
| Waves CLA-3A | Gain 0-10 (unity at **4.08**, initial 5.50), Peak Reduction 0-10 (initial 4.00), Comp/Limiter, **HiFreq** 0-100 (flat at 100, initial **50**), **Analog** Off/50 Hz/60 Hz, VU In/GR/Out, Mix 0-100 %, Trim ±18 dB | Modelled at **−18 dBFS = +4 dBu = 0 VU**. Stereo component uses "one detector for both channel paths". | [8] [9] |
| Black Rooster Audio VLA-3A | Peak Reduction 0-10, Gain 0-10, Limit/Compress, VU In / GR / Out | Claims "real-time, SPICE-style circuit simulation" of transformers, discrete sidechain, audio amplifier, HF contour network and the T4B. | [10] |
| Bomb Factory LA-3A (Digidesign, discontinued) | Peak Reduction, Gain, Comp/Limit, meter | The first widely used LA-3A plug-in; still shows up in "which LA-3A plug-in" threads as the baseline people grew up with. | [70] [72] |

---

## 3. Signal path and circuit behaviour

Everything in this section is read from UREI schematic **C11186, issue E, 4-14-70** [3] unless another source is
cited. Unlike the Distressor ([[Distressor]] section 3.8), the LA-3A has a published schematic, so there is very
little guesswork here. Where I do interpret the drawing rather than read it, I say so.

### 3.1 Block diagram in words

```
IN (600 Ω, Jones barrier strip or XLR)
  -> 20 dB T-pad, switched  (R41 510 Ω, R42 510 Ω, R43 130 Ω; the "50 / 30 dB GAIN" switch)
  -> input transformer T1  (B11178, dual 150 Ω primaries in series = 600 Ω, ~15 kΩ secondary)
  -> R2 39 kΩ shunt across the secondary
  -> R1 68 kΩ series, with C15 100 pF across it
  -> node X, shunted to common by the T4B AUDIO PHOTOCELL          <-- the gain element
  -> R3 1.3 kΩ (marked "NOM", selected on test)
  -> GAIN pot R4 100 kΩ (front panel)
  -> R5 3.3 kΩ, C1 0.22 µF
  -> AMPLIFIER: Q1/Q2 dual 2N5089 differential pair
               -> Q3 2N5087 -> Q4 2N5087
               -> complementary output Q5 2N3053 (NPN) / Q6 2N4037 (PNP),
                  R18 / R19 4.3 Ω emitter resistors, CR1 / CR2 1N4003 bias string,
                  output node TP-1 biased to 12-13 V by R7 500 kΩ
               feedback: R14 220 kΩ from TP-1 to Q2's base, shunt leg R12 5.1 kΩ
                         with R44 15 kΩ / C14 220 µF; C4 12 pF and R15 1 kΩ compensate
  -> C5 400 µF -> output transformer T2 (B11148) -> 600 Ω balanced OUT

SIDECHAIN (feedback: driven by the gain-reduced signal)
  tap from the attenuator, selected by the COMPRESSOR / LIMITER switch
  -> PEAK REDUCTION pot R21 100 kΩ (front panel)
  -> C6 4.7 nF                                                     <-- the low-frequency deafness
  -> Q7 2N5089, base bias R25 4.7 MΩ / R26 1 MΩ,
     emitter network R28 10 kΩ (rear-panel HF CONTOUR) with R29 1.5 kΩ + C7 4.7 nF
                                                                   <-- the high-frequency emphasis
  -> Q8 2N3417 or 2N3391, R27 33 kΩ, R30 3.3 kΩ
  -> STEREO ADJ R31 10 kΩ, R32 1.8 kΩ, C8 10 µF   -> STEREO terminal (link)
  -> driver Q10 2N3417/2N3391 with Q9 2N3053 and Q11 2N4037, R34 4.7 kΩ, R35 68 kΩ
  -> C9 80 µF
  -> AUTOTRANSFORMER T4 (B11184, taps marked .31 / 2.5 / 5)        <-- steps ~10 V up to ~100 V
  -> EL PANEL inside the T4B  ->  light  ->  the audio photocell, and a matched meter photocell

METER: matched cell -> R13 68 kΩ (82 kΩ before revision A) -> S3 GR / OUTPUT -> VU meter,
       R22 47 kΩ and R23 250 kΩ set the sensitivity, R36 3.9 kΩ the output-level path.
POWER: T3 B11147 -> bridge CR3-CR6 1N4003 -> C10 1000 µF -> series pass Q12 2N3053 with
       CR7 27 V zener -> C12 2100 µF; two 1819 lamps (28 V) light the meter.
```

The important structural point: **this is the LA-2A's architecture with the tubes cut out**. The divider, the
cell, the feedback sidechain and the two-knob interface are unchanged. The amplifier, the sidechain amplifier
and the panel driver are all transistor stages on a single low-voltage rail, and that is where the difference in
sound comes from.

### 3.2 The input pad and transformer

- **T1 = B11178**, the same input transformer as UREI's 1109 preamp, "dual 150 ohm primaries in series for 600
  [ohms], and a 15K (?) secondary", per a builder who chased the JBL Pro documentation for it. [20] A 600 Ω to
  15 kΩ transformer is a **1:5 voltage step-up, about +14 dB** (**derived**).
- The **50 / 30 dB GAIN** switch inserts R41 510 Ω and R42 510 Ω in series with R43 130 Ω shunt around the
  primary. A textbook 20 dB T-pad for 600 Ω wants series arms of 600·(10−1)/(10+1) = **491 Ω** and a shunt of
  2·600·10/(10²−1) = **121 Ω** (**derived**), so 510 / 510 / 130 is exactly that pad built from stock values.
  The manual's "attenuate the incoming signal by 20 dB" is therefore literal. [1] [3]
- Because the pad sits **ahead of everything**, moving it moves the threshold with it, and UREI published both
  numbers: **threshold of limiting −10 dBm at the 30 dB position, −30 dBm at the 50 dB position**. [2] That is
  the single most useful calibration fact in this whole dossier: unlike the LA-2A, the LA-3A has a *published*
  absolute threshold. Section 7.4 anchors the model to it.
- C15, 100 pF across R1, is a small treble bypass around the series arm of the attenuator (**interpretation**:
  it slightly reduces the attenuator's high-frequency insertion loss and its effect grows as the cell resistance
  falls, which is one plausible mechanical origin of the "gets brighter as it works" reports in section 4.5).

### 3.3 The attenuator, and why the LA-2A's divider code transfers

With the cell resistance `Rc`, the node ahead of the Gain pot sees

```
Z    = Rc ∥ (R3 + R_pot)  =  Rc ∥ 101.3 kΩ
A_raw = Z / (R1 + Z)      =  Z / (68 kΩ + Z)
A     = A_raw / A_dark
```

Numbers (**derived**, with `R_dark = 2 MΩ`):

| Cell resistance | `A_raw` | Insertion loss | Gain reduction re. dark |
|---|---|---|---|
| 2 MΩ (dark) | 0.586 | −4.6 dB | 0 dB |
| 100 kΩ | 0.424 | −7.5 dB | 2.8 dB |
| 20 kΩ | 0.198 | −14.1 dB | 9.4 dB |
| 5 kΩ | 0.0654 | −23.7 dB | 19.0 dB |
| 1 kΩ | 0.01435 | −36.9 dB | 32.2 dB |
| 500 Ω | 0.00726 | −42.8 dB | 38.1 dB |
| 400 Ω | 0.00584 | −44.7 dB | 40.0 dB |

So the published "Max Gain Reduction 40 dB" [1] wants a cell that reaches about **400 Ω** under full light
(**derived**), which is in the range the LA-2A literature gives for a bright T4 cell (under 1 kΩ, 0.68-2 kΩ
quoted by different sources; see [[LA-2A]] section 3.3). The 4.6 dB dark insertion loss is absorbed by the
amplifier.

**The useful surprise**: the LA-2A model in `src/dsp/opto/model.rs` uses `R_SERIES = 70.7 kΩ` and
`R_POT = 100 kΩ`; the LA-3A's equivalent numbers are `68 kΩ` and `101.3 kΩ`. That is a 4 % difference in the
series arm and a 1.3 % difference in the shunt, which moves the dark insertion loss by 0.14 dB and the 40 dB
point by under 1 dB (**derived**). **The attenuator maths is the same circuit.** `attenuation_for()` can be
reused verbatim with per-model constants, and I should resist the temptation to invent differences that are not
there. The two devices sound different because of the *sidechain*, the *amplifier* and the *speed*, not because
of the divider.

### 3.4 The amplifier

A four-transistor, single-rail, class-AB line amplifier with heavy global feedback, and it is the reason the
LA-3A measures better than the LA-2A everywhere except noise-at-low-gain.

- **Input pair**: Q1 and Q2, drawn as one package containing two 2N5089 devices, i.e. a monolithic or matched
  pair. R8 68 kΩ and R9 4.7 kΩ are the collector loads, R10 180 kΩ the tail. Q1's base takes the signal through
  R5 3.3 kΩ and C1 0.22 µF; Q2's base is the feedback summing node.
- **DC servo by hand**: R7, a 500 kΩ trimmer, feeds Q1's base through R6 220 kΩ with C2 1 µF decoupling, and the
  drawing carries the instruction **"ADJ BIAS FOR 12-13 VOLTS AT TP-1"**. That is the entire calibration
  procedure of the unit, and a GroupDIY regular confirms it in practice: "Basically you just adjust R7 until you
  measure 12-13 V at TP1. In reality you have to wait a while to adjust it & even then it drifts around a bit."
  [22]
- **Drivers**: Q3 and Q4, both 2N5087 PNP, with R11 10 kΩ and R17 100 Ω.
- **Output**: Q5 2N3053 NPN above, Q6 2N4037 PNP below, each with a 4.3 Ω emitter resistor (R18, R19), biased by
  two 1N4003 diodes (CR1, CR2) in the classic cheap-and-cheerful arrangement. The junction, TP-1, sits at half
  the rail and is coupled out through C5, 400 µF, into the output transformer.
- **Feedback and gain**: R14 220 kΩ returns TP-1 to Q2's base; the shunt leg is R12 5.1 kΩ, with R44 15 kΩ
  bypassed by C14 220 µF so the AC leg is about 3.8 kΩ. Closed-loop gain ≈ 1 + 220/3.8 ≈ **59, or +35.4 dB**
  (**derived**). With T1's +14 dB and the attenuator's −4.6 dB that is +44.8 dB before the output transformer,
  against a **50 dB ±1 dB** specification [2], so the output transformer must contribute roughly +5 dB
  (**estimate**; I have no turns data for the B11148).
- **A confirmation from the DIY community that the feedback reading is right**: the widely circulated "LA3A
  Mods" list, which is what UA turned into the reissue's MOD switch, says "Add 15k resistor in parallel with R14
  (220k)". [29] Dropping R14 from 220 kΩ to about 14 kΩ takes the closed-loop gain from ≈59 to ≈4.7, a **22 dB
  reduction** (**derived**); UA's manual describes the MOD switch as dropping "the output amp's gain by 24 dB"
  and giving "+26 dB" of unit gain with the pad out. [1] Two independent descriptions land within 2 dB of each
  other, which is as much confirmation as I am going to get without a unit on the bench.
- **Consequences for the sound.** A single ~25 V rail with a 12-13 V output node gives roughly ±12 V of
  available swing at TP-1 (**derived**), and the specification is +24 dBm nominal with +27 dBm peaks into 600 Ω
  [2], i.e. 12.3 V RMS / 17.4 V peak at the *secondary*. The output transformer therefore steps up, and the
  amplifier clips symmetrically against the rails rather than softly the way a triode does. That is the
  headroom story: enormous clean output, then a fairly abrupt solid-state ceiling. Section 4.6 has the distortion
  numbers.

### 3.5 The sidechain, and where the LA-3A's ears are

This is the part that makes an LA-3A an LA-3A, so I go through it component by component.

**1. The tap and the Peak Reduction pot.** The sidechain is taken from the attenuator, not from the input:
"The LA-3A is a feedback style compressor. This is due to the fact that the signal that is used to drive the
side-chain circuit is affected by the gain-reduced signal." [1] The tap feeds the top of **R21, a 100 kΩ pot**,
whose wiper is the sidechain input. Turning it up feeds more of the (already compressed) signal to the detector,
which raises the loop gain, which lowers the threshold and deepens the reduction: "The greater the gain of this
circuit, the lower the threshold and the greater the amount of compression." [1]

**2. C6, 4.7 nF — the low-frequency deafness.** The wiper couples into Q7's base through a **4.7 nF** capacitor.
That is a very small coupling capacitor for a sidechain. Loaded by the stage's input resistance (R25 4.7 MΩ in
parallel with R26 1 MΩ is 826 kΩ, but the transistor's own input resistance dominates and I do not know it), the
corner lands somewhere between about **40 Hz and 350 Hz** (**estimate**, from 1/(2π·R·4.7 nF) for R between
826 kΩ and 100 kΩ). I take **about 100 Hz** as the working value, because that is what users hear: an owner of
two original units describes the LA-3A as having "a nice roll off in the side chain below 100 Hz". [17] The
LA-2A has nothing equivalent in the audio-to-detector path; its low-frequency behaviour is dominated by the EL
panel's coupling capacitor instead ([[LA-2A]] section 4.5).

This one capacitor is a large part of the answer to "why does it sit differently on guitars and vocals". A
compressor that barely hears below 100 Hz does not duck the whole track when the bass or the kick arrives; it
rides the midrange and the top.

**3. The HF Contour network — the LA-3A's ears above 1 kHz.** Q7's emitter goes to **R28, a 10 kΩ rear-panel
pot**, and a series network of **R29 1.5 kΩ and C7 4.7 nF** bridges from a tap on R28 back to the collector
side. As frequency rises, C7's impedance falls and the network progressively removes emitter degeneration, so
the stage's gain rises with frequency. The transition sits at roughly

```
f ≈ 1 / (2π · C7 · (R29 + R_E,tapped))
  = 1 / (2π · 4.7 nF · 11.5 kΩ)  ≈ 2.9 kHz     (pot at maximum, derived)
```

falling to no boost at all when the tap shorts. UREI's own words for the resulting behaviour: "Limiting
frequency response is adjustable to allow as much as **10 dB increase in gain reduction at 15 kHz compared to
frequencies below 1 kHz**." [2] UA's reissue manual says the same in studio language: "Turning the control ...
will increase compression of frequencies above 1 kHz. At the full ... position, the LA-3A will attenuate 15 kHz
10 dB more than the lower frequencies." [1] So: **a first-order high shelf in the sidechain, corner near 1 to
3 kHz, up to +10 dB, continuously variable, and set flat at the factory.** Which end is flat is genuinely
disputed and gets its own subsection (4.5).

**4. The second stage and the link.** Q8 (2N3417 or 2N3391, the drawing offers either) with R27 33 kΩ and R30
3.3 kΩ raises the level again. **R31, the 10 kΩ STEREO ADJ pot**, then feeds R32 1.8 kΩ and C8 10 µF, and this
node is brought out to the **STEREO** terminal on the barrier strip. Two units joined here share the same
control voltage, which is why UREI could say "The electro-optical attenuators of two LA-3A units may be
connected in tandem for stereo operation" [2] and why the calibration procedure is "turn the PEAK REDUCTION
knob on just one of the two units until the meter reads −5 dB; **both units should respond**" [1].

**5. The driver and the autotransformer.** Q10 (2N3417/2N3391) with Q9 2N3053 and Q11 2N4037 forms the panel
driver; R34 4.7 kΩ, R35 68 kΩ and R37 220 kΩ set it up, C13 0.01 µF and R33 2.2 kΩ shape it. Its output passes
through **C9, 80 µF**, into **T4, the B11184 autotransformer**, whose taps are marked **.31, 2.5 and 5** on the
drawing. The EL panel of the T4B hangs on the high-turns end.

This is the single biggest circuit difference from the LA-2A, and it is worth stating plainly. The LA-2A drives
its EL panel from a **6AQ5 pentode plate sitting on a ~275 V rail**, through a 10 kΩ plate resistor: a
high-impedance, high-voltage source ([[LA-2A]] section 3.4). The LA-3A drives its panel from a **low-voltage
transistor stage through a step-up autotransformer**: a low-impedance source behind a piece of iron. That means

- the drive can rise fast, because the driver is not slew-limited by a 10 kΩ plate resistor charging the panel's
  1-2 nF, which is a plausible mechanism for the specified attack of 1.5 ms or less against the LA-2A's 10 ms
  (**interpretation**);
- the autotransformer imposes its own **low-frequency roll-off and its own saturation**, which is a second,
  independent reason the LA-3A ignores bass, on top of C6;
- and the drive is **AC-coupled twice** (C9 and the transformer), so there is no DC path to the panel at all.

A builder rebuilding this stage measured the primary side: "The 10 V on the primary is very reasonable from
testing. I think the highest I measured was **14 V with GR max with 1.23 V input @ 1 kHz**", and specified a
replacement autotransformer as **100 Ω : 10 kΩ (10 V / 100 V), 2.5 W** — a **1:10 voltage step-up**. [20] So the
panel sees roughly **100 V rising to 140 V** at maximum gain reduction (**derived** from that measurement),
which is the right neighbourhood for an EL panel designed for 120 V mains ([[LA-2A]] section 3.3). Replacement
parts confirm the ratio: Sowter's equivalent is labelled "blue common, pink 1:, red :10". [21] Studio
Electronics (David Kulka), Hairball Audio (EA-11184) and Cinemag (CM-2511) all sell replicas. [21] [63] [64]
[65] The stage is distinctive enough that builders ask about transplanting it into tube units [27], and sourcing
the electroluminescent panel it drives is its own long-running thread [30].

**6. The Compressor / Limiter switch: what I can and cannot read.** The drawing shows a small slide switch
labelled **COMPRESSOR / LIMITER** with three net labels beneath it, **D**, **E** and **L**, and two spare lugs.
Net **D** goes to the attenuator node ahead of R3; the other two go down into the audio wiring around the Gain
pot and the sidechain. At the scan resolution available to me I **cannot** resolve with confidence which
terminal is the common and which net carries the sidechain feed, and I will not pretend otherwise. What I can
say:

- On the reading where the switch simply moves the sidechain tap from one side of R3 to the other, the two taps
  differ by 20·log10(101.3/100) = **0.11 dB regardless of the cell resistance** (**derived**), so that reading
  predicts *no* audible difference, and it must therefore be wrong or incomplete.
- The manual's functional description is unambiguous and is what the model must reproduce: "When in the
  COMPRESS position, the curve is gentler, and presents a low compression ratio. A higher compression ratio
  results when the switch is set to the LIMIT position. **The difference in these two modes is only present when
  the LA-3A is in deep compression.**" [1]
- The two published ratio pairs are "approximately 3:1" and "approximately infinity:1" (UA) [4] and "linear
  gain reduction" versus "a compression ratio approaching **50:1**" (UREI) [2]. Waves says 3:1 and about
  **100:1**. [8]
- The LA-2A's equivalent switch is understood by the DIY community as mixing a few per cent of the *un*-reduced
  signal into the feedback tap, which turns the loop progressively feed-forward as the reduction deepens and
  makes the ratio climb steeply ([[LA-2A]] section 3.4). That mechanism produces exactly the behaviour the
  LA-3A manual describes, and the LA-2A model already implements it as a blend coefficient.

So section 7 implements Limit as **the same blend coefficient with a larger value**, tuned by test rather than
derived from the drawing, and the dossier flags the topology as unresolved. That is an honest place to land: the
tests in section 8 pin the *behaviour*, which is what the spoof needs.

### 3.6 The T4B as the LA-3A draws it

The dashed box on the schematic labelled **T4B** has **seven numbered pins** and, on my reading of the drawing,
**four photocell symbols** plus the EL panel:

- a cell between pins **5** and **6**, wired out to R22 47 kΩ and the R23 250 kΩ trimmer, i.e. the **meter
  cell**;
- the **EL panel** (drawn as a capacitor, which is what it electrically is) between the pin **3** node and pin
  **2**;
- a cell from pin **1** up to the panel's top node;
- **two cells in parallel** between pin **7** and the pin **4** rail.

A GroupDIY poster noticed the same thing and asked about it: "why the T4B circuit drawn on the original
schematic, in the LA3A manual from the seventies, seems to have four photocells, isn't it just two photocells in
the T4B's built today?" [24] Nobody answered him, and the honest reading is that **the drawing shows the early
T4B, the one with the extra fast Clairex CL-705 cell paralleled with the main pair** that the LA-2A literature
describes for units up to about 1969 ([[LA-2A]] section 1.2). Later T4Bs dropped the third cell, and Ken Kantor
of Kenetek concluded that the overall response "is dominated by the response of the slower photocell" anyway.
[25] I treat the parallel pair as **the audio attenuator** and the extra cell as the fast one, label it
**interpretation**, and note in section 7 that the existing single-cell model with its two-time-constant carrier
and trap states already produces the fast-plus-slow behaviour that a parallel fast cell would give, so no new
state is needed.

Two more T4B facts that matter for the model:

- The datasheet's opening paragraph says "the unique characteristics of the **T4A** electro-optical attenuator
  have been maintained", and the very next paragraph says "the **T4B** electro-optical attenuator is used". [2]
  I read this as marketing continuity with the LA-2A rather than a real T4A fitment, and everyone who has opened
  a unit reports a T4B, with the panel voltages to match. [3] [23] [25] [26] [56]
- The **matching resistor**: T4Bs shipped with a resistor to calibrate the meter, and on the LA-3A it is
  **R13**, 68 kΩ (82 kΩ before revision A). A GroupDIY member with a batch of UREI-stamped spares: "All of them
  came with a 33k resistor if I recall correctly, which I suspect wasn't really correct. It's quite easy to work
  out what it should be by subbing in a preset pot ... I think the resistor in question is R13. It's R25 on an
  LA-2A." Another notes the supplied manual "describes the 33K resistor as an LA-2A R25. If it is 3A, this
  resistor will not be needed", and a third points out that on the LA-2A drawing "R25 can be anything between
  27k & 100k", i.e. adjust on test. [23] The tribute should therefore expose a **cell-match trim** and let it be
  slightly wrong, because on real units it usually is.

### 3.7 Metering

- In **GR** mode the meter is driven from the matched second cell through R13 68 kΩ, with R22 47 kΩ and the R23
  250 kΩ trimmer setting sensitivity; the needle rests at 0 and swings left. In **OUTPUT** mode R36 3.9 kΩ feeds
  it from the output, and 0 VU corresponds to **+4 dBm**. [1] [3]
- Calibration, from someone who does it: inject a sine, meter to OUTPUT, Peak Reduction down, set Gain for 0 VU;
  raise Peak Reduction until the output drops to −10 VU; switch to GR and trim until it reads −10. [23]
- The reissue's front-panel **GR zero set** is done warm, after five minutes, with Peak Reduction fully
  counter-clockwise. [1]
- Diagnostic folklore worth keeping as a joke in the tribute: "With that switch on 50 you can get the meter to
  show gain reduction with the opto removed from the circuit", which is how people convince themselves a dead
  T4B is working. [19]

### 3.8 Power supply, and the two little lamps

T3 (B11147) feeds a 1N4003 bridge (CR3-CR6) and C10 1000 µF; Q12, a 2N3053, is the series pass device with R38
27 Ω, R39 1 kΩ, R40 330 Ω and **CR7, a 27 V zener**, setting the rail, smoothed by C12, 2100 µF. Consumption is
**6 watts** on the original sheet [2] and "35 watts maximum" on the reissue [1] — the reissue presumably counts
the transformer's rating rather than the draw. Fuses: 3AG 1/8 A slow-blow at 115 V, 1/16 A at 230 V. [1] [2] [3]

The meter is lit by **two type 1819 lamps, 28 V**, marked I1 and I2 on the drawing and confirmed by owners
hunting replacements ("According to the schematic ... there are two 1819 bulbs rated at 28 volts each"). [3]
[18] [76] The tribute's faceplate should glow warm behind the meter, and a "lamp" toggle costs nothing and is funny.

### 3.9 Stereo

Two units are joined by a shielded cable between the **CHASSIS** and **STEREO** terminals, which bridges the
sidechain driver nodes so both panels see the same drive. **STEREO ADJ** (R31) trims the balance. UA's procedure:
set Stereo Adj fully clockwise, match the outputs at 0 dB with the Gain controls, then raise Peak Reduction on
one unit only to −5 dB GR, and trim the unit showing the most reduction until the meters agree. [1] [3] In
normal use "the stereo image should remain solid and both units should compress the stereo signal equally,
regardless of which side (left or right) is triggering the gain reduction". [1]

---

## 4. Measured and published behaviour

A warning before the numbers. The LA-2A has an academic study behind it: Moore measured six units at
Huddersfield and published attack, release, THD and frequency-response spreads [55]. **Nothing comparable exists
for the LA-3A.** What I have is two manufacturer specification sheets that disagree with each other, one
carefully run null-test shootout by a hobbyist with two hardware units [12], and a large amount of consistent
listening testimony. I have not invented a single measurement to fill the gap, and where the model needs a
number that nobody published, section 7 says so and labels it an estimate.

### 4.1 Published specifications

| Quantity | UREI / Teletronix datasheet [2] | UA reissue manual [1] | LA-2A, for contrast [31] |
|---|---|---|---|
| Input impedance | 600 Ω floating | 600 Ω floating | 600 Ω balanced |
| Max input level | +20 dBm at 30 dB gain, 0 dBm at 50 dB gain | "−0 dBm @ 50 dBm gain setting" (sic) | — |
| Output load / source | 600 Ω floating, damping factor 8 | "600 floating", also "approximately 50 Ω" internal | 600 Ω balanced |
| Max output level | **+24 dBm nominal, +27 dBm peaks** | +20 dBm nominal, +27 dBm peaks | +10 dBm nominal, +16 dBm peaks |
| Gain | **50 dB or 30 dB, ±1 dB**, switched at the rear | 50 dB ±1 dB | 40 dB ±1 dB |
| Frequency response | **±1 dB, 20 Hz to 20 kHz** | ±0.5 dB, 20 Hz to 20 kHz | +0/−1 dB, 30 Hz to 15 kHz |
| Signal to noise | > 80 dB at the threshold of limiting, 30 Hz-15 kHz bandwidth | noise floor −80 dB at the limiting threshold | 70 dB below +10 dBm |
| **Threshold of limiting** | **−10 dBm at the 30 dB position, −30 dBm at the 50 dB position** | not stated | not stated |
| Distortion | < 0.5 % THD, 30 Hz to 20 kHz; worst case 0.7 % at 50 Hz with 15 dB GR; typically < 0.3 % across the band at 20 dB GR | < 0.35 % THD at +24 dBm | < 0.5 % at +10 dBm |
| Max gain reduction | not stated (40 dB implied) | **40 dB** | 40 dB |
| **Attack** | **less than 250 µs to 0.5 ms, program dependent** | **1.5 ms or less, program dependent** | 10 ms |
| **Release** | 500 ms to 5.0 s, "depending on the duration of the peak causing the onset of limiting" | **stage 1: 60 ms (50 % release); stage 2: 0.5 to 5 s, program dependent** | 0.06 s for 50 %, 0.5-5 s complete |
| Power | 110-125 V, 50/60 Hz, **6 watts**; switchable 220-250 V | 115 / 230 V, 35 watts maximum | — |
| Size, weight | 3.5 in × 8.5 in, 9.25 in deep, 6.5 lb (8 lb shipping) | 2RU half rack, 9.25 in deep, 6.5 lb | 19 in, 3.5 in |

The reissue's specification block is visibly sloppy: it lists "Internal Output Impedance" twice with two
different answers, writes "50 dBm" where it means the 50 dB gain position, and gives "Recommended Minimum Load
50". I quote it because it is the manufacturer's document, not because it is careful.

### 4.2 Attack: the number nobody agrees on

- **UREI, 1970: "Less than 250 microseconds to 0.5 milliseconds depending on program material."** [2]
- **UA, reissue: "1.5 ms or less, program dependent."** [1]
- These differ by a factor of three to six, and both are honest in the sense that neither states its criterion.
  There is no industry standard for defining a compressor's attack time, a point Gregory Scott makes at length
  in the LA-2A literature ([[LA-2A]] section 4.2).
- Users are sceptical of the slower figure in the other direction. On Gearspace, asked where the numbers come
  from: "The LA3A sure sounds slower than 1.5 ms. I wonder if that spec is for a certain amount of gain
  reduction." [15] That is almost certainly the resolution: an optical cell's rise time shortens as the light
  gets brighter (Clairex type-5 material goes from 1.1 s at 0.01 footcandles to 2 ms at 100 fc; PerkinElmer:
  "all material types show faster speed at higher light levels"; [36] [37]), so the "attack time" of an optical
  compressor is a function of how hard you hit it and how much reduction you ask for. A single number is a
  category error, which is why both manufacturers append "depending on program material".
- Whatever the criterion, **the LA-3A is between four and forty times faster than the LA-2A's 10 ms**, and that
  is the headline. UA: "significantly faster attack capability", suited to "cymbals, room mics, drums, and
  percussion" where the LA-2A was less effective. [5]
- For the model I treat the two specifications as bracketing a level-dependent curve: **about 1.5 ms at light
  reduction, falling toward a few hundred microseconds when the panel is driven hard** (**estimate**, but an
  estimate that both published figures and the photocell physics support).

### 4.3 Release: two stages, and a memory

Both manufacturers give the same two-stage story, and the reissue manual's text is lifted almost verbatim from
the LA-2A manual with the model number changed:

> "After the light is removed from the cell, it releases quickly (**40-80 milliseconds**) to approximately half
> of its off resistance. The remainder of its release can take place over as much as several seconds." [1]

and the memory:

> "The amount of time it takes for the cell to recover after the light is removed depends on how long light had
> been shining on it and how bright the light. In the case of the LA-3A this results in behavior where **the
> release time is slower if the unit has either been in compression for a while, or the amount of compression is
> large**. This signal dependent release characteristic is critical to the sound of the unit." [1]

and the operating consequence, in the applications chapter:

> "The release time will be quick with occasional light compression (−3 dB) but slower if driven hard
> continuously (−7 to −10 dB). Also, the release time has two stages. It releases quickly at first and then
> slower." [1]

UREI's 1970 sheet quotes only the slow stage — "Varies from **500 milliseconds to 5.0 seconds** depending on the
duration of the peak causing the onset of limiting" [2] — which is the same 0.5-5 s window, and its phrasing
("depending on the duration of the peak") is the clearest published statement anywhere that the release depends
on exposure history rather than on level alone.

The physics is the same as the LA-2A's and is already in the model: a fast free-carrier recombination and a slow
trap-controlled tail, with the trap population acting as the memory ([[LA-2A]] sections 3.3, 4.3, 7.2, and
[40] [45]).

### 4.4 Ratio, knee and threshold

| Source | Compress | Limit |
|---|---|---|
| UREI datasheet [2] | "linear gain reduction" | "a compression ratio approaching **50:1**" |
| UA plug-in manual [4] | "approximately **3:1**" | "approximately **infinity:1**"; "the compression ratios are nonlinear and frequency dependent, so these figures are not absolute" |
| UA reissue manual [1] | "the curve is gentler, and presents a low compression ratio" | "a higher compression ratio"; the difference "is only present when the LA-3A is in deep compression" |
| Waves CLA-3A [8] | "approximately **3:1**" | "approximately **100:1**" |
| Black Rooster VLA-3A [10] | not stated | not stated |

Points of agreement, which are the ones the model must honour:

1. **There is no threshold control and no ratio control.** Peak Reduction moves both at once: "the Peak
   Reduction knob controls both the threshold and the amount of compression". [1]
2. **The knee is soft and there is no corner**, because the curve is the product of an EL panel's light law, a
   photocell's gamma and a feedback loop, exactly as on the LA-2A.
3. **The ratio is not a constant.** It grows with drive and it varies with frequency [4], in the same words UA
   uses of the LA-2A [33].
4. **The two modes coincide until you push it.** [1] [19]
5. **The threshold is absolute and published**: −10 dBm with the 30 dB gain switch, −30 dBm with the 50 dB
   switch [2], and the MOD switch on the reissue "lowers the threshold point where compression begins and allows
   a greater amount of overall compression" [1]. Nothing else in this family of devices gives me a hard number
   like that, and section 7.4 calibrates the model to it.
6. **The knob taper is deliberately odd.** Waves: "The scale is not linear and has been adjusted to conform to
   the exact scaling of the modeled unit. Thus, there may be more compression than expected at certain steps, as
   with analog gear." [8]

A behaviour that comes up repeatedly and is worth reproducing: past roughly two-thirds of the Peak Reduction
travel, the output level starts falling as well as the dynamics tightening. An owner of an ADL 1600 (an LA-3A
type) describing his first time with one: "after approximately 12 o'clock it starts to behave different, then it
starts to reduce output at the same time as the meter moves closer and closer to −20 dB", and was told by both
the seller and the builder that this is normal. [24] That is what a feedback opto does when the loop gain gets
large: it stops behaving like a compressor with make-up and starts behaving like a fader being pulled down.

### 4.5 Frequency dependence, and the direction of the HF Contour

**The built-in tilt.** Even with the trimmer flat, the LA-3A hears the top of the spectrum better than the
bottom. Three independent mechanisms in the circuit push the same way (section 3.5): the 4.7 nF sidechain
coupling capacitor C6, the autotransformer's own low-frequency limit, and the emitter network's residual
high-frequency lift. An owner of two originals describes the result from the listening chair: the unit "has a
nice roll off in the side chain below 100 Hz", which he treats as a feature when tracking bass. [17] This is the
most important difference from the LA-2A for the way the two sit on a source, and it is why an LA-3A on a
distorted guitar or a full drum bus does not pump on the low end the way an LA-2A does.

**The trimmer.** UREI: "as much as 10 dB increase in gain reduction at 15 kHz compared to frequencies below
1 kHz. This is advantageous in FM and TV transmission where pre-emphasis is used." [2] UA: "make the compressor
more sensitive to high and high mid frequency content ... Typically, this control is left in the FLAT position",
and at the extreme "the LA-3A will attenuate 15 kHz 10 dB more than the lower frequencies". [1] Everybody agrees
on the *amount*. Nobody agrees on the *direction*:

| Source | Which way is flat | Which way is more HF sensitivity |
|---|---|---|
| UA reissue hardware manual [1] | full **counter-clockwise** | clockwise |
| UA plug-in manual [4] | full **clockwise** (the default) | counter-clockwise |
| Waves CLA-3A [8] | **100** (clockwise) | toward 0 |
| The circulated "LA3A Mods" list [29] | — | "Set HF boost rear panel control to MAX. **FULLY CCW**" |
| Gearspace owners [13] [14] | "Mine are all set to flat" | "Counter clock wise means it will compress more high frequencies. Kinda like a de-esser" |

Two community sources and both plug-ins put maximum high-frequency sensitivity at the **counter-clockwise** end
and flat at the clockwise end; the UA hardware manual is alone on the other side and also contradicts UA's own
plug-in. I therefore treat **clockwise = flat** as the convention, note the hardware manual's disagreement, and
match the LA-2A model's existing `emphasis` parameter, where 1 is flat, so that the two models' controls behave
the same way on the page. This is exactly the R37 situation on the LA-2A, where clockwise is also flat
([[LA-2A]] section 2), so the convention is at least internally consistent.

**The upper-mid push.** Something in the LA-3A reads as an EQ move around the presence region, and people go
looking for it. A mixer trying to recreate an old LA-3A vocal: "the LA-3A vocal has some kind of push EQ-wise in
I think the upper mids. Does anyone have any numbers of where that might be exactly?", answered by a long-time
owner: "I always feel like it's somewhere @ 2.5 to 4 k but it seems to be dynamic. I have never been able to
just EQ it in." [16] UA sells the same impression as "bold, mid-forward compression" [6] and Black Rooster calls
its emulation "faster, brighter, mid-forward". [10] Nobody I can find has published a swept response that
demonstrates a static bump, and the "it seems to be dynamic" observation points at the real cause: a compressor
that is more sensitive above 1 kHz and deaf below 100 Hz will duck the lows less and the highs more, and when
the gain comes back the midrange rides forward. **I model it as an emergent consequence of the sidechain shape,
not as a static EQ**, and section 8 has a test that checks it emerges.

### 4.6 Distortion and frequency response

- UREI's own numbers are careful and unusually informative: "Less than 0.5 % T.H.D. from 30 Hz to 20 kHz", with
  the note "**The low frequency Total Harmonic Distortion in a limiter is a function of release time.** Under
  worst case conditions (a predominant low frequency energy envelope causing 15 dB of gain reduction) the 50 Hz
  THD will not exceed **0.7 %**. Typical THD over the program spectrum bandwidth, with 20 dB of gain reduction
  is less than **0.3 %**." [2] That is a manufacturer explicitly documenting sidechain ripple: when the detector
  tracks within a cycle of a low-frequency waveform, the gain modulates at twice the signal frequency and
  generates harmonics.
- The reissue quotes "< 0.35 % THD @ +24 dBm". [1]
- **Compare with the LA-2A: 0.9 % to 4.2 % THD at only 6 dB of gain reduction across six units, third harmonic
  10-20 dB above the second** [55]. The LA-3A is roughly an order of magnitude cleaner while compressing three
  times harder. That is the whole point of the solid-state amplifier, and any emulation that makes the LA-3A as
  dirty as an LA-2A has got it wrong.
- Bandwidth is wider at both ends: ±1 dB from 20 Hz to 20 kHz [2] against the LA-2A's +0/−1 dB from 30 Hz to
  15 kHz, where Moore measured up to −5.5 dB at the extremes on one unit and blamed the transformers [55].
- **The one piece of independent harmonic data I have** is the Gearspace null-test shootout [12], which is worth
  reporting exactly. On a sustained sine at about 10 dB of gain reduction, in Compress, with the reduction
  settled, the hardware "shows **four distinct overtones** being produced". Against that: "The UAD has too much
  3rd harmonic, no 2nd, and too much 4th it seems. The CLA and rooster only seem to add the 3rd harmonic, and
  not nearly enough." The author's summary of all three plug-ins: "The harmonic overtones that the plugins
  produce are not correct." [12]
- A second observation from the same test, about dynamics rather than distortion: "It's interesting to compare
  the behaviour on a loud kick drum signal. The plugins seem to **choke/hard-limit even on 'compress' mode**
  relative to the real thing. The real LA-3A is all over the place after the initial hit — doesn't look like the
  input signal at all." [12] I read that as the plug-ins running out of drive and clamping, where the real cell
  keeps moving. It is a good thing to test for (section 8, test 14).

### 4.7 Unit-to-unit variation

The LA-3A is **much more consistent than the LA-2A**, and that is a real, measured finding rather than an
impression. From the null-test shootout, on two original Ureis, not stereo-linked:

> "To avoid the conclusion that each HW unit is different ... I ran the test signal through both of my LA-3As
> (no stereo link) and subtracted the two signals ... **It was very easy to bring the two so close** — the gain
> reduction knobs were set to 7 vs 6.3 or so. The gain knobs were very similar, at 3.4 each." [12]

Two units, 0.7 of a knob mark apart on Peak Reduction and identical on Gain, nulling well. Compare Moore's LA-2A
sample, where attack ranged 33-81 ms, release 449-1670 ms, THD 0.9-4.2 %, and one unit could not exceed 9 dB of
gain reduction at all. [55] The explanation is obvious in hindsight: the LA-2A's spread comes from tubes *and*
cells, the LA-3A's only from cells, and the LA-3A's heavy global feedback swamps device tolerance in the audio
path.

That said, owners do hear differences: "They are all a little bit different. Some get gain reduction easier than
others, some are on the bright or aggressive side, some are more chilled out." [19] [58] And T4B ageing is the same
problem it is on the LA-2A: Waves found "depleted T4 devices result in **up to 80 % less compression**" and that
"up to **90 % of T4 components in use today have never been replaced**" [8] [57]; owners are warned that a unit with
the rear switch on 50 dB can appear to show gain reduction on the meter with the opto out of circuit entirely
[19]. A "cell age" control is therefore as defensible on the LA-3A as it is on the LA-2A, and it is funnier here,
because the LA-3A's reputation for consistency makes a worn one more of a surprise.

### 4.8 Reissue versus vintage

Tape Op's verdict on the 2005 reissue: Universal Audio "have done an amazing job" recreating the original; the
unit is "less squishy and a lot more transparent" than the tube LA-2A; "you can make this thing compress like
crazy, and it never sounds overdone"; it "works fantastically on bass, guitar, piano, vocals, and pretty much
anything you put through it"; and on lead vocals, "when you get it right, you don't have to ride the fader at
all during mixdown". The review notes UA "included the emphasis adjustment pot on the rear panel and added a
gain modification stage" that had been a third-party upgrade on vintage units, at $1,500 street. [11] So the
reissue is the vintage circuit plus the mod everyone was already doing, which is the right thing to have done
and gives the model a natural "MOD" toggle.

---

## 5. Sound character, and what makes an emulation right or wrong

Sound On Sound's survey of the classics files the LA-2A as the extremely soft-knee, slow-attack member of the
family [62]. The LA-3A is the other end of the same idea, and people describe it accordingly.

How people describe it, collected:

- **Faster than an LA-2A and cleaner than either neighbour.** "Less squishy and a lot more transparent" [11];
  "smooth, optical compression, but with the aggression of a transistor circuit" [5]; "faster limiting than the
  tube-driven smooth operator ... with the clarity of the iconic solid-state 1176" [6]; an attack that "catches
  transients the LA-2A would sail past" [10].
- **Mid-forward.** "Bold, mid-forward compression" [6]; "faster, brighter, mid-forward" [10]; an upper-mid push
  around 2.5-4 kHz that owners cannot EQ back in [16].
- **It brings things to the front.** "A 'secret weapon' compressor with a unique character capable of moving
  sounds right to the front of your speakers" [4]; "This, coupled with the attack and release character,
  produces great results that bring vocals forward" [1].
- **It survives being hit hard.** "You can make this thing compress like crazy, and it never sounds overdone"
  [11]; but UA's own caveat, which is more honest than most marketing: "When driven hard (−10 or greater) the
  LA-3A may not be the proper choice for fast tempo music and quick 'staccato' sounds if natural transparency is
  desired." [1]
- **Guitars and bass above all.** UA lists "acoustic/electric guitars, bass, and drums, valued for its midrange
  character" [5]; the reissue manual devotes sections to vocals and bass guitar and says the preset attack and
  release "work well for letting some transients through and then shaping the release in a smooth way" [1].

What the evidence says an emulation must get right, in the order I care about:

1. **Feedback topology with an emergent curve.** No threshold, no ratio, no knee parameter. The shape comes out
   of the EL law, the cell gamma and the loop gain, exactly as in [[LA-2A]] section 5. A feed-forward
   compressor with a ratio knob will not do it and will be obvious on programme material.
2. **A fast, level-dependent attack.** Between 250 µs and 1.5 ms depending on how hard the panel is lit [1] [2],
   which is four to forty times faster than the LA-2A, and which must get *faster* as the reduction deepens
   because that is how CdS cells behave [36] [37].
3. **A two-stage, history-dependent release.** 40-80 ms to half, then 0.5-5 s, longer after long or deep
   compression. [1] [2] Already solved in the framework's cell.
4. **A sidechain that is deaf below about 100 Hz and gets brighter above about 1 kHz**, before the trimmer does
   anything. [2] [17] This is the difference that makes the LA-3A sit differently on a guitar, and it is the
   thing I would most expect a lazy emulation to omit.
5. **A ten-decibel, variable high shelf in the sidechain** at up to 15 kHz, wired so that one end is genuinely
   flat. [1] [2]
6. **Low distortion that rises with reduction and with falling frequency.** Under 0.5 % broadband, 0.7 % at
   50 Hz with 15 dB of reduction, 0.3 % typical at 20 dB. [2] Not the LA-2A's 1-4 %. And the harmonic *series*
   matters, not just the total: the hardware makes four overtones and the shipping plug-ins do not match them.
   [12]
7. **A wide, flat band**, 20 Hz to 20 kHz within a decibel [2], not the LA-2A's transformer-limited 30 Hz to
   15 kHz.
8. **Enormous output headroom**, +24 dBm nominal, then a solid-state ceiling rather than a tube's gradual
   softening. [2]
9. **Consistency.** Two real units null against each other easily [12]. An LA-3A model should not be given the
   LA-2A's unit-to-unit lottery as a default, though a "worn cell" option is fair game as a joke.

And the things a spoof is allowed to get wrong on purpose: the exact harmonic series (nobody has published it),
the Comp/Limit topology (nobody has explained it), and the panel's absolute calibration (I pick one and document
it).

---

## 6. How the LA-3A and optical compressors are simulated

### 6.1 Physical and grey-box models of photoresistors and opto cells

The LA-3A uses the same cell as the LA-2A, so the whole literature from [[LA-2A]] section 6.1 applies unchanged,
and the framework already implements the parts of it that matter. In brief, and with the LA-3A angle noted:

- **Parker and D'Angelo, DAFx 2013** (Buchla lowpass gate, VTL5C3): a one-pole on the control light whose time
  constant switches on the sign of the derivative and is "modulated further by the current output value of the
  vactrol model, so that it responds quicker when at high values"; LED-current-to-resistance law
  `R = A·I^−1.4 + B`. [44] The "quicker when at high values" term is precisely the mechanism that turns the
  LA-2A's 10 ms into the LA-3A's 1.5 ms when the panel is driven harder.
- **Eichas and Zölzer, SPIE 2016** and **Eichas, Gerat and Zölzer, AES 142** (VTL5C2 in a dynamics circuit):
  measured turn-on below 1 kΩ within 5 ms at 10 mA and turn-off above 1 MΩ only after 500 ms, "100 times
  longer"; the digital model is a peak detector, a measured static level-to-gain table, and a smoothing block of
  three first-order low-passes with separate attack and release coefficients and blending weights, fitted by
  Levenberg-Marquardt to error-to-signal ratios of 1-3 %. [42] [43]
- **Najnudel, Müller, Hélie and Roze, DAFx 2023**: a port-Hamiltonian, passive, physical vactrol model with two
  carrier populations obeying Shockley-Read-Hall recombination whose identified rate constants differ by two
  orders of magnitude, a softplus LED law with an explicit threshold, and a dual-slope optical coupling. Their
  divider-compressor simulation shows "the higher the input voltage, the shorter the attack and the longer the
  release", and that "ratio and knee are set by the divider resistances". [45] Both of those conclusions are
  load-bearing for the LA-3A: the first is its attack specification, the second is why section 3.3's divider
  table is the transfer curve.
- **Wright and Välimäki, DAFx 2022** (grey-box, LA-2A, SignalTrain data): a log-domain compressor whose
  threshold, ratio and knee are predicted from the control setting by a small network, with a level detector
  that is a one-pole, a switching one-pole or an RNN-modulated one-pole. The RNN detector learned to make the
  time constant "very small when the input signal is large" — a neural network rediscovering the level-dependent
  attack from data. [46]
- **Yu and Fazekas, AES AIMLA 2025**: a five-parameter feed-forward compressor fitted per Peak Reduction setting
  by Newton-Raphson; they note the device "is technically a feedback compressor", that modelling the two-stage
  release explicitly should improve results, and that the fitted attack and release times "vary exponentially
  with the peak reduction". [47]
- **Underlying device physics**: photocurrent decay in CdS has a fast recombination component of about a
  millisecond and a slow trap-controlled component of about ten seconds, with the slow one dominating at low
  light [41]; time-averaged EL brightness follows the Alfrey-Taylor relation `B = B0·exp(−b/√V)` [40]; CdS
  gammas are 0.6-0.9 [38] [39]; resistive opto-isolators turn on about ten times faster than they turn off, and
  their distortion is below about 0.01 % under 100-300 mV across the cell, rising with the square of the voltage
  above that [38].

### 6.2 Black-box neural models

All of the published black-box work targets the LA-2A, not the LA-3A, because SignalTrain is an LA-2A dataset.
[48] It is still the right place to learn what a *sufficient* model needs: dilated temporal convolutional
networks needed about **300 ms of receptive field** before they matched the hardware [49]; a state-space (S6)
model of about a thousand parameters runs stereo at 48 kHz under 200 MFLOPS with 64-sample latency and its
authors state flatly that "the release time for the LA-2A cannot be known a priori, as it is highly dependent on
the signal's history" [50] [51] [52]. Nobody has published a neural LA-3A. If somebody trains one, the 300 ms
figure is the number to beat, and it argues that the framework's trap-state memory is not optional.

### 6.3 What the emulations say they model

| Product | Claimed content | Notes | Sources |
|---|---|---|---|
| **Universal Audio Teletronix LA-3A** (UAD-2) / **LA-3A Compressor** (UADx) | "Modeled from a unit in UA's vintage collection"; refreshed interface; adds an **HF Emphasis** sidechain filter presented as the hardware's own control, and a **Dry/Wet Mix** that the hardware does not have; Comp ≈ 3:1, Lim ≈ ∞:1, "nonlinear and frequency dependent"; Meter GR / Output / Off | The reference emulation. Measured in [12] as having "too much 3rd harmonic, no 2nd, and too much 4th" | [4] [6] [12] |
| **Waves CLA-3A** | Chris Lord-Alge's unit; THD, "variable release times ... lasting several seconds", 50/60 Hz hum, T4 depletion; HiFreq "increases voltage amplifier gain in the peak reduction circuit, for frequencies above 1 kHz, leaving lower frequencies unaffected", usable "as sort of a de-esser"; 3:1 and about 100:1; **−18 dBFS = +4 dBu = 0 VU**; stereo component uses one detector for both channels | The only source that publishes a calibration reference. Measured in [12] as adding only a third harmonic, "not nearly enough" | [8] [12] |
| **Black Rooster Audio VLA-3A** | "Real-time, SPICE-style circuit simulation" of the whole path: input and output transformers, discrete sidechain stages, audio amplifier stages, the **high-frequency contour filter network**, and the **T4B optical cell** with "level-dependent, self-adjusting behaviour"; describes the hardware as taking "the optical attenuator idea from the LA-2A and dropp[ing] the tube amplifier in favour of discrete solid-state circuitry" | The clearest public statement of what a component-level LA-3A model contains. Its dating ("the early 1970s") is a year or two late | [10] [12] |
| **Bomb Factory LA-3A** (Digidesign, long discontinued) | The first widely used LA-3A plug-in | Still the baseline in "which LA-3A plug-in" threads | [70] [72] |
| **Tim Petherick "Opto 3a"** (Nebula library) | Sampled/convolution-based | Suggested for inclusion in the null-test shootout but not tested | [12] |
| **Softube Opto Compressor** | "An iconic early 1960s T4 opto cell tube compressor/limiter" plus a Time control | An LA-2A, not an LA-3A; listed so the reader does not mistake it for one | [60] |
| **IK Multimedia T-RackS White 2A** | "No electronic circuitry involved with the compression itself. It's just a tube amp with photo-resistors, lighted by a fluorescent panel driven by the output signal" | Also an LA-2A. **I could not find an LA-3A model in the T-RackS range**, and I would rather say so than invent one | [61] |

**What each one gets right and wrong.** Only one public test puts the plug-ins next to the hardware [12], so
this is a short list rather than a confident ranking, and every negative below traces to that test or to the
product's own documentation rather than to my ears.

- **UA Teletronix LA-3A.** Right: it is modelled from a real unit, it is the only emulation that documents the
  ratio pair and the frequency dependence in the same breath ("nonlinear and frequency dependent, so these
  figures are not absolute" [4]), and it exposes the HF Emphasis trimmer, which most owners of the hardware
  never touch and most emulations never offer. Wrong: the harmonic series. The null test found "too much 3rd
  harmonic, no 2nd, and too much 4th" [12] — a model producing no second harmonic at all is a structural
  choice, not a tuning error, and it is why section 7.4 puts a small even-order term in the output stage.
  Ambiguous: its HF control rotates the opposite way to the hardware manual's description (section 4.5).
- **Waves CLA-3A.** Right: it is the only product that publishes its calibration (−18 dBFS = +4 dBu = 0 VU
  [8]), the only one that documents the T4 depletion problem honestly, and the only one that admits the Peak
  Reduction taper is deliberately non-linear "to conform to the exact scaling of the modeled unit" [8]. Its
  HiFreq description is also the most precise account anyone publishes of what the trimmer does electrically:
  it "increases voltage amplifier gain in the peak reduction circuit, for frequencies above 1 kHz, leaving
  lower frequencies unaffected" [8], which is exactly the emitter network of section 3.5. Wrong: harmonics
  again, "only seem to add the 3rd harmonic, and not nearly enough" [12]; and modelling one named engineer's
  unit means inheriting that unit's cell.
- **Black Rooster VLA-3A.** Right: the most complete public statement of what a component-level LA-3A model
  contains — transformers, discrete sidechain, audio amplifier, the HF contour network and a T4B with
  "level-dependent, self-adjusting behaviour" [10] — and it is the only one to name the contour filter as part
  of the circuit rather than as a bonus feature. Wrong: same harmonic shortfall as Waves [12], and its dating
  ("the early 1970s") is a year or two late against the 1969 AES debut [4] [5].
- **All three, together.** The two failures they share are the ones worth designing against. Harmonics: none
  matched the hardware's four overtones [12]. Extremes: "The plugins do not attack and release like the real
  thing, especially from 0 to extreme gain reduction levels", and on a loud kick they "choke/hard-limit even on
  'compress' mode" where the real unit "is all over the place after the initial hit" [12]. Both are failures at
  the ends of the range, which is where a feedback opto is hardest and where a static level-to-gain table fitted
  in the middle will always let go. That is the argument for keeping the physical cell and the real loop rather
  than fitting a curve, and it is what tests 15 and 16 are for.

Hardware clones describe the same ingredients from the other direction: Serpent Audio's SA-3A, LaZ Electronics'
LA3A and its 500-series version, Golden Age's Comp-3A, AudioScape's V3A and Anthony DeMaria's ADL 1600 all
centre on a T4B, an autotransformer-fed panel driver and a discrete line amp, and the DIY community's shopping
list for a build is exactly the parts in section 3: a B11178-equivalent input transformer, a B11148-equivalent
output transformer, a B11184-equivalent autoformer (Studio Electronics, Hairball, Sowter, Cinemag) and a T4B
(Kenetek, AudioScape, DeMaria). [12] [19] [20] [21] [25] [58] [59] [63] [64]

The overall state of the art, from the one person who tested three of them side by side against two real units
(a looser three-plug-in comparison reaches the same conclusion less rigorously [66]):
plug-ins get the level right and the harmonics and the extremes wrong. "The plugins do not attack and release
like the real thing, especially from 0 to extreme gain reduction levels." [12] That is a low bar and a useful
one, because it tells me where to spend effort in section 7: on the drive law and the extremes, not on the
distortion.

---

## 7. Recommended DSP design (44.1 to 96 kHz, real time)

### 7.1 The one decision that shapes everything else

**The LA-3A model reuses the LA-2A's T4 cell simulation. It does not get a cell of its own.** The hardware uses
the same T4B module in the same role [2] [3], the divider around it is the same circuit to within 4 %
(section 3.3), and the release specification is word-for-word identical [1]. Duplicating
`src/dsp/opto/model.rs`'s `Cell` would create two copies of the same physics that would drift apart, and it
would be wrong: the two devices do not differ in the cell, they differ in **how hard and how fast the cell is
lit**, and in **what the light passes through on its way to the loudspeaker**.

Concretely, in the existing code:

| Existing item | LA-3A treatment |
|---|---|
| `Cell`, its `n_f` / `n_t` / `u` states and `step()` | **reused unchanged** |
| `CellParams` | **reused**, with one new field, `tau_u` (section 7.5), and an `LA3A` preset |
| `Cell::light_for` and `EL_B` | **reused unchanged**: the electroluminescent panel is the same part |
| `resistance_for`, `attenuation_for`, `gr_db_for`, `A_DARK` | **reused**, parameterised by `R_SERIES` / `R_POT` per voicing |
| `pr_gain`, `makeup_db`, `tube` | **replaced** per voicing |
| `Compressor`, `Settings`, the static solver, metering, stereo, denormal flushing | **shared**, with a `Voicing` enum selecting the constant set |

The cheapest structure is a `Voicing { La2a, La3a }` inside the existing `opto` module, because the divider, the
cell, the static-curve solver, the VU ballistics, the stereo link and the denormal hygiene are all identical and
are already written and tested; what the voicing selects is a small constants block plus three swapped
functions. A separate sibling module is equally acceptable **provided it imports `opto::model::Cell` rather
than copying it**. That is the load-bearing requirement, and it is the only one I would refuse to compromise
on: two hand-tuned copies of the same photocell will drift, and tests 23 and 24 in section 8 exist precisely
to catch that drift.

### 7.2 Block diagram in words, per channel per sample

1. **Input**: no pad. The rear 50/30 dB switch is **not** a parameter; the model is fixed in the **50 dB
   position**, pad out. That is the studio setting, and it is the position UREI's −30 dBm threshold figure
   refers to. Section 7.3 says what was dropped and why.
2. **Input transformer**: first-order high-pass at 7 Hz. (Wider than the LA-2A's 12 Hz; see section 7.5.)
3. **Attenuator**: `y_att = x_hp · A(R_cell)`, the divider of section 3.3, with the cell state from the previous
   sample.
4. **Photocell non-linearity**: the same small odd-order term the LA-2A model already applies, at about a third
   of the LA-2A's strength (the LA-3A's published THD is an order of magnitude lower; section 4.6).
5. **Sidechain tap**: `tap = (1 − β)·y_att + β·x_hp`, `β = 0` in Compress, `β = BETA_LIMIT` in Limit.
6. **User side-chain high-pass** (the shared `sc_hpf` extra, not on the hardware).
7. **Peak Reduction**: `v = g_pr(pr) · tap`.
8. **C6, the low-frequency deafness**: first-order high-pass at 100 Hz.
9. **The autoformer's low-frequency limit**: a second first-order high-pass at 30 Hz.
10. **HF Contour**: first-order high shelf, corner 2 kHz, `+10·emphasis` dB, where `emphasis = 0` is flat and
    `emphasis = 1` is the full 10 dB. This is the **opposite sense to the LA-2A's `opto_emphasis`**, where 1 is
    flat; section 7.3 explains the reversal.
11. **Fixed residual tilt**: a gentle high shelf, +3 dB above 3 kHz, present whatever the trimmer does.
12. **Driver saturation**: `v = V_sat · tanh(v / V_sat)`. **Last** in the chain, unlike the LA-2A model, because
    on the LA-3A the driver is the final stage before the transformer.
13. **Cell**: `cell.step(v)` — the shared `Cell`, with a shorter panel smoothing time constant.
14. **Make-up**: the Gain knob alone, reaching +50 dB. The MOD switch is not modelled (section 7.3).
15. **Output amplifier**: a solid-state soft clipper with a small crossover deadband, replacing `tube()`.
16. **Output transformer**: first-order low-pass at 50 kHz.
17. **Mix / bypass / meter / stereo link**, all shared with the LA-2A path.

### 7.3 Parameter table

Ids use the `la3a_` prefix as briefed. Note that the LA-2A model in the repository uses `opto_` rather than
`la2a_`; if the ids are ever harmonised, `opto_` should become `la2a_` and this table stays as it is. Ids are
stable API, so this is a decision to make once, before release.

| id | Label | Range / labels | Taper | Default | Notes |
|---|---|---|---|---|---|
| `la3a_gain` | Gain | 0.0 to 10.0 | audio (log), see `gain_db` in 7.4 | **5.0** | Unity at **4.1** (Waves' 4.08 [8]); +50 dB at 10. Make-up only, never affects compression [1] [4]. |
| `la3a_peak_reduction` | Peak Reduction | 0.0 to 10.0 | linear knob, exponential sidechain gain | **4.0** | Waves' initial value [8]. Sets threshold and depth together. At 0 there is no compression at any level. |
| `la3a_mode` | Mode | Compress, Limit | switch | **Compress** | "Most users leave the LA-3A in COMPRESS mode." [1] |
| `la3a_meter` | Meter | Gain Reduction, Output, Off | switch, not automatable | **Gain Reduction** | Output: 0 VU = +4 dBm [1]. Off dims the meter and bypasses, as the UA plug-in does [4]. |
| `la3a_emphasis` | HF Contour | 0.0 to 1.0 | linear | **0.0** (flat) | **0 = flat, 1 = the full +10 dB at 15 kHz** in the sidechain. This is the **opposite sense to the LA-2A's `opto_emphasis`**, where 1 is flat. See 4.5 for the rotation dispute and the note below for the reversal. |
| `la3a_cell` | Cell | Fresh, Used, Tired | switch, not automatable | **Fresh** | Scales `k_gen` to 1.0 / 0.6 / 0.2, after Waves' "up to 80 % less compression" from a depleted T4 [8]. Default Fresh because real LA-3As null against each other [12]. |
| `link` | Stereo Link | toggle | — | on | Shared with the other models. |
| `mix` | Mix | 0 to 100 % | linear | 100 % | Not on the hardware; UA added the same control [4]. |
| `sc_hpf` | Side-chain HPF | 0 (off) to 300 Hz | linear | 0 | Shared extra. Stacks on top of the built-in 100 Hz roll-off; builders add external side-chain inputs to real units for the same reason [28]. |
| `bypass` | Bypass | toggle | — | off | Shared. |

**Two hardware controls are deliberately absent, and one has its sense flipped.**

- **The rear 50/30 dB pad is not a parameter.** The model is fixed in the 50 dB position, pad out. The pad is a
  20 dB attenuator ahead of everything (section 3.2), so in a plug-in it does nothing that the host's own gain
  staging does not already do, while costing a control that would confuse anyone who did not read the manual.
  Fixing it also removes an ambiguity from the calibration: UREI's two published thresholds, −10 dBm and
  −30 dBm, differ by exactly the pad [2], and pinning the 50 dB position pins the model to the −30 dBm figure
  with nothing left to interpret.
- **The MOD switch is not a parameter either.** With the pad fixed out, MOD's only remaining effect is a 24 dB
  cut in make-up (section 3.4), which is the Gain knob's job. Its interesting half, forcing the pad out and so
  lowering the threshold, is already the model's permanent state. Keeping a toggle whose whole function is
  "turn the output down" would be a joke with no punchline.
- **`la3a_emphasis` runs 0 = flat to 1 = full, the opposite way round to `opto_emphasis`.** On the LA-2A the
  control is R37, whose factory setting is fully clockwise and whose panel legend reads as a trim you back
  *off*, so 1 = flat reads correctly there. On the LA-3A the control is a rear-panel HF Contour with **FLAT**
  silkscreened at one end (section 2.2) and the manuals describe it as a boost you *add*: "a high frequency
  **boost** of the signal feeding the gain reduction circuit" [1], "as much as 10 dB **increase** in gain
  reduction at 15 kHz" [2]. A parameter called emphasis should therefore read 0 for none and 1 for all of it.
  Note that neither shipping plug-in numbers it this way round: UA puts flat at fully clockwise and Waves puts
  flat at 100 [4] [8], so both count downwards from flat. I follow the hardware's legend and the name, not
  their numbering. The default is 0 because the factory setting is flat and so is every owner's [13] [14],
  and because the modification list that treats the control as a feature still calls its other end the
  maximum [29]. **This is the
  single most likely thing to get wrong when copying code from the LA-2A engine**, so section 8's test 7
  asserts the sense explicitly.

### 7.4 Equations per block

Let `fs` be the sample rate and `T = 1/fs`. A first-order section with time constant `tau` uses
`a = 1 − exp(−T/tau)`.

**Calibration.** 0 VU = **−18 dBFS RMS = +4 dBu**, the same reference the LA-2A model already uses
(`VU_REF_DBFS`) and the one Waves publishes for the CLA-3A [8]. Sine peak amplitude at 0 VU is
`VU_REF_AMP = 10^(−18/20)·√2`.

**Input transformer.** The pad is out and stays out (section 7.3), so there is nothing to switch:

```
x_hp = HighPass(x; 7 Hz, first order)                  // T1, B11178
```

**Attenuator** (section 3.3; the same code path as the LA-2A with different constants):

```
R_SERIES_LA3A = 68_000        // R1
R_POT_LA3A    = 101_300       // R3 (1.3 k) + Gain pot (100 k)
R_DARK        = 2_000_000
R_MIN         = 400           // gives the specified 40 dB (derived, section 3.3)

R_p   = R_cell * R_POT / (R_cell + R_POT)
A_raw = R_p / (R_SERIES + R_p)
A     = A_raw / A_DARK
y_att = x_hp * A
```

**Photocell non-linearity** (reused, weakened):

```
k   = CELL_CUBIC_LA3A * (1 - A)          // CELL_CUBIC_LA3A = 0.2 (LA-2A uses 0.6)
q2  = (y_att / CELL_CUBIC_V0)^2
y_att *= 1 - k * q2 / (1 + q2)
```

**Sidechain.**

```
tap = (1 - beta) * y_att + beta * x_hp           // beta = 0 or BETA_LIMIT
s   = UserScHpf(tap)
v   = g_pr(pr) * s
v   = HighPass(v; 100 Hz, first order)           // C6, 4.7 nF        <-- new
v   = HighPass(v; 30 Hz,  first order)           // autoformer        <-- new
v   = HighShelf(v; 2 kHz, +10 * emphasis dB)     // HF Contour, 0 = flat
v   = HighShelf(v; 3 kHz, +3 dB)                 // fixed residual tilt
v   = V_sat * tanh(v / V_sat)                    // driver clipping, last
cell.step(v)
```

with the Peak Reduction law

```
g_pr(pr) = 10^((G0 + PR_DB_PER_UNIT * pr) / 20) * end^2,    end = clamp(pr / 1.2, 0, 1)
PR_DB_PER_UNIT = 4.0     // 40 dB across the 0..10 knob
```

The `end^2` term reproduces "when Peak Reduction is set to its minimum value, no compression (or limiting)
occurs" [4] by fading the sidechain to nothing over the bottom 12 % of the travel, exactly as the LA-2A model
already does.

**Where `G0` comes from.** This is the one place where the LA-3A is better documented than the LA-2A, so it is
worth being precise. The LA-2A had to be calibrated to a soft target
("PR 30 gives 1 dB of gain reduction at 0 VU") because no absolute threshold was ever published. The LA-3A has
one. UREI states **"THRESHOLD OF LIMITING: −10 dBm at 30 dB position, −30 dBm at 50 dB position"** [2]. I read
that as *the input level at which limiting begins with the sidechain at full drive* (**interpretation**, since
UREI does not say at what Peak Reduction setting), and I solve `G0` so that

```
with pr = 10, mode = Compress, emphasis = 0 (flat):
    1 dB of gain reduction occurs at a 1 kHz sine of −30 dBu   ( = −34 dB re 0 VU = −52 dBFS RMS )
```

UREI's other number is then a consistency check rather than a second calibration. Inserting the 20 dB pad would
move the same point to **−10 dBu**, which is exactly the published 30 dB figure, so the two thresholds differ
by precisely the pad and one constant fixes both. Because the model is fixed in the 50 dB position (section
7.3), only the −30 dBu figure is exposed, and section 8's test 3 asserts it. That is still a much stronger
anchor than anything available for the LA-2A, where no absolute threshold was ever published.

The same bisection routine the LA-2A model already uses (`Compressor::calibrate`) does the work: bisect on the
cell's free carriers for 1 dB of reduction, invert the electroluminescent law for the drive that produces that
light, and set `G0` so the sidechain delivers that drive for the specified input at `pr = 10`. Nothing new to
write.

**Cell.** Unchanged, except that `CellParams` gains a `tau_u` field (the panel-plus-driver smoothing that
`Cell::set_sample_rate` currently hardwires to 1 ms):

```
self.a_u = 1.0 - (-self.dt / params.tau_u).exp();
```

`tau_u` is **1.0 ms for the LA-2A** (unchanged behaviour) and **0.25 ms for the LA-3A**. The justification is
physical, not fudged: the LA-2A drives its panel from a 6AQ5 plate through a 10 kΩ resistor, the LA-3A from a
low-impedance transistor stage through a step-up transformer (section 3.5), so the panel's charging time
constant is several times shorter. Everything else about the attack difference comes out of the existing law
`tau = tau_f0 / (1 + light / l_a)` being fed a brighter light, because the LA-3A's sidechain gain is higher: the
cell gets faster because it is being hit harder, which is what the physics says [36] [37] [45] and what both
manufacturers mean by "depending on program material" [1] [2].

**Make-up and the output amplifier.**

```
gain_db(p) = 50.0 * (1.0 + 2.583 * log10(max(p / 10.0, 1e-5)))    // +50 dB at 10, 0 dB at 4.1
w = y_att * 10^(gain_db / 20)
```

then the solid-state stage, which replaces `tube()`:

```
// 1. crossover deadband: class-AB with diode bias and 4.3 Ω emitter resistors
w = if |w| < XOVER { w * XOVER_SOFT } else { w - sign(w) * XOVER * (1 - XOVER_SOFT) }

// 2. a hard-ish symmetric ceiling, much cleaner than tanh below it
z = w / (1 + |w / V_CLIP|^N)^(1/N)                                 // N = 5

// 3. a small even-order term so the model produces a second harmonic at all
z += ASYM * z * z

z = LowPass(z; 50 kHz, first order)                                // T2, B11148
```

Three notes on that stage:

- The `(1 + |w/V|^N)^(1/N)` shape with `N = 5` is essentially transparent until close to the ceiling and then
  turns hard, which is what a feedback amplifier clipping against its rails does, and it is the opposite of the
  LA-2A's `tanh`, which starts bending immediately. Tuned so that a sine at the +24 dBm equivalent gives
  **0.35 % THD** [1], the model is an order of magnitude cleaner than the LA-2A model at the same reduction,
  which is what the specifications demand (section 4.6).
- The **crossover deadband** is the one piece of colour that is specific to this circuit and that I have not
  seen in any emulation. It generates odd harmonics that get *proportionally worse as the level falls*, the
  opposite of every other non-linearity in the lab, and it is a real property of a diode-biased complementary
  pair. Keep `XOVER` tiny (of order a millivolt referred to the output) so it lives under the noise on loud
  material and adds a faint edge on quiet material.
- The `ASYM` term exists because the one independent measurement I have says the hardware makes four overtones
  while the shipping plug-ins make one or two, and specifically that the UAD model has "no 2nd" [12]. A model
  that produces only odd harmonics is provably wrong here. `ASYM` is small and its only job is to put a second
  harmonic on the spectrum.

**Meter.** Unchanged VU ballistics from the LA-2A model (99 % of reading in 300 ms, 1.0-1.5 % overshoot). GR
mode reads the cell; **Output** mode reads the output with **0 VU = +4 dBu** [1]; **Off** dims the meter and
disables processing [4]. The `cell` parameter's mismatch, and the `R13`-style match trim discussed in section
3.6, can be exposed as a hidden ±0.5 dB offset between the audio cell and the meter cell, because on real units
the meter is only as honest as somebody's soldering.

### 7.5 Constants

Values marked with a source are anchored; the rest are **estimates** to be tuned against section 8. LA-2A
values are given alongside so a reviewer can see at a glance what actually changed.

| Constant | LA-2A (existing) | LA-3A | Anchor |
|---|---|---|---|
| `R_SERIES` | 70.7 kΩ | **68 kΩ** | R1 on the schematic [3] |
| `R_POT` | 100 kΩ | **101.3 kΩ** | R3 + Gain pot [3] |
| `R_DARK` | 2 MΩ | 2 MΩ | same cell |
| `R_MIN` | 500 Ω | **400 Ω** | gives the specified 40 dB max GR (derived, 3.3) |
| `EL_B` | 5.0 | 5.0 | same panel; Alfrey-Taylor [40] |
| `CELL_GAMMA` | 0.7 | 0.7 | CdS gamma 0.6-0.9 [38] [39], and 0.7 is the LA-2A design table's pick inside that range. This row read 0.8 and "same cell" until 2026-09-03, which cited the implementation rather than a source; the implementation had no justification recorded for 0.8. No T4 measurement exists either way |
| `tau_u` (panel smoothing) | 1.0 ms | **0.25 ms** | low-impedance driver behind a transformer (3.5) — estimate |
| `tau_f0` (cell attack, dim light) | 40 ms | 40 ms | same cell; the speed comes from the drive |
| `l_a` (light at which attack halves) | 0.05 | 0.05 | same cell |
| `tau_r1` (first release stage) | 60 ms | 60 ms | "60 ms (50 % release)" [1], identical to the LA-2A |
| `tau_t0` (slow release, empty traps) | 0.5 s | 0.5 s | "0.5 to 5 seconds" [1] [2] |
| `k_m` (trap slowing) | 12 | 12 | gives the 5 s tail |
| `capture` | 1/0.3 s | 1/0.3 s | same cell |
| `k_gen` | 7.0 | **12.0** | hotter drive; tune so test 3's threshold and test 4's attack both pass — estimate |
| `PR_DB_PER_UNIT` | 0.55 dB (knob 0-100) | **4.0 dB** (knob 0-10) | 40 dB span, from UREI's threshold pair and the recommended operating point (derived) |
| `PR calibration` | 1 dB GR at 0 VU with PR 30 | **1 dB GR at −30 dBu with PR 10** | UREI "threshold of limiting" [2] |
| `BETA_LIMIT` | 0.09 | **0.16** | tuned so Limit reaches an effective 40:1 or more where Compress is at 3:1 ([2] "approaching 50:1", [8] "100:1") — estimate |
| `V_SAT_OVER_ONSET` | 10.0 | 14.0 | more drive headroom before the driver clips — estimate |
| sidechain HP 1 (C6) | none | **100 Hz, first order** | 4.7 nF coupling [3]; "roll off in the side chain below 100 Hz" [17] — estimate within a derived 40-350 Hz bracket |
| sidechain HP 2 (autoformer) | none | **30 Hz, first order** | transformer coupling [3] — estimate |
| HF Contour shelf | 1 kHz, −10 dB low shelf, 1 = flat | **2 kHz, +10 dB high shelf, 0 = flat** | "10 dB at 15 kHz vs below 1 kHz" [1] [2]; corner from R29/C7 (derived, 2.9 kHz). Note the reversed sense (7.3) |
| fixed sidechain tilt | −4 dB @ 300 Hz, +3 dB @ 3 kHz | **+3 dB @ 3 kHz only** | the low end is now a real high-pass, not a shelf |
| input transformer HP | 12 Hz | **7 Hz** | ±1 dB at 20 Hz [2] (derived) |
| output transformer LP | 40 kHz | **50 kHz** | ±1 dB at 20 kHz [2] (derived) |
| make-up law | 40 dB, unity at 0.32 | **50 dB, unity at 4.1** | 50 dB ±1 dB [2]; Waves' unity 4.08 [8] |
| pad, MOD switch | — | **not modelled** | fixed 50 dB position, pad out (7.3); the hardware values are −20 dB [1] [3] and −24 dB [1] [29] should they ever be wanted |
| `V_CLIP`, `N` | `tanh`, k = 0.2 | ceiling at the +27 dBm equivalent, `N = 5` | 0.35 % THD at +24 dBm [1]; +27 dBm peaks [2] — estimate |
| `XOVER`, `XOVER_SOFT` | — | ≈ 1 mV referred to output, 0.5 | class-AB crossover [3] — estimate |
| `ASYM` | bias 0.05 (tube) | small, second harmonic ≈ −70 dBc at 0 VU | four overtones observed [12] — estimate |
| `CELL_CUBIC` | 0.6 | **0.2** | < 0.5 % THD [2] against the LA-2A's 1-4 % [55] |
| VU reference | −18 dBFS = +4 dBu | same | [8] |

### 7.6 Compress versus Limit in the model

Only `beta` changes, exactly as in the LA-2A model, and for the same reason: at light reduction the
feed-forward term is swamped by the feedback term and the two modes coincide, and as the reduction deepens the
feed-forward term keeps growing while the feedback term is clamped, so the effective ratio runs away. That
reproduces the manual's "the difference in these two modes is only present when the LA-3A is in deep
compression" [1] without needing to know which lug of the switch is the common (section 3.5, point 6).

`BETA_LIMIT` is larger than the LA-2A's because the published Limit ratio is much more extreme: UREI's
"approaching 50:1" [2] and Waves' "approximately 100:1" [8] against the LA-2A's disputed 4:1 to 100:1
([[LA-2A]] section 4.4). Tune it against test 6, not against the schematic.

### 7.7 Sample rate, oversampling, stereo, hygiene

- Every time constant is in seconds and every coefficient is recomputed on `set_sample_rate`, so the model is
  rate-independent to first order. The fastest state is now `tau_u = 0.25 ms`, which is still 11 samples at
  44.1 kHz, so the Euler integration stays comfortable.
- **Oversampling.** The gain loop does not need it: its bandwidth is far below 1 kHz. The output stage now
  does, mildly, because the hard `N = 5` ceiling and the crossover deadband generate higher-order harmonics
  than the LA-2A's `tanh`. Offer **2× oversampling of the make-up and output stage only**, defaulted on above
  −6 dBFS peaks or simply defaulted on, since it is one biquad pair per channel.
- **Stereo** is the same shared-cell arrangement as the LA-2A: with `link` on, one cell is driven by the mean of
  the two sidechains before rectification, which is what joining the STEREO terminals does on the hardware
  (section 3.9).
- **Denormals**: unchanged. The new sidechain high-passes are two more states that must be flushed below
  `1e-12`, and they are the ones most likely to ring quietly forever after a bass note, so they need it most.

### 7.8 What the page should show

The framework stays headless and uncoloured; every face and colour belongs to the example
([[feedback-framework-vs-plugin]]). For this model the face is: a **black half-rack panel**, two cream knobs
with panel-printed 0-10 scales, a cream VU meter with two warm lamps behind it, a `GR / OUTPUT` toggle, a
`POWER / ON` toggle, and the `LEVELING AMPLIFIER` / model-name block between them (section 2.1). The two rear-panel
controls the model keeps, `Comp/Limit` and `HF Contour`, belong on a flip-around back panel or a drawer,
because that is where they are on the real thing and finding them is half the joke. The 50/30 dB and MOD
switches can be painted on and left dead: they are part of the picture, not part of the model (section 7.3). The existing `cell` stream
(`[light, free_carriers, trapped_carriers]`) already exists and should be reused so the "inside the T4" display
works for both optical models; the difference the viewer will see is that the LA-3A's light spikes far faster.

---

## 8. Test plan

Each test drives the DSP core offline at 44.1, 48 and 96 kHz and asserts against a tolerance. Tests 1 to 15 pin
the LA-3A against its own published behaviour; **tests 16 to 22 run the LA-2A and LA-3A engines on identical
input and assert that they differ in the documented directions**, which is the part that stops the second model
from quietly becoming a re-badged copy of the first. Where a hardware figure exists it is cited; where the
expected value is mine it says so. Unless stated, settings are Compress, HF Contour flat
(`la3a_emphasis` 0), `mix` 100 %, `sc_hpf` off, and levels are referred to 0 VU = −18 dBFS RMS = +4 dBu.

**Static behaviour**

1. **Bypass and unity.** `bypass` on: output equals input to 1e-6. `bypass` off, Peak Reduction 0, Gain 4.1:
   output level equals input level within ±0.15 dB from −40 to +10 dBu, and total harmonic distortion at 0 VU is
   below 0.1 %. Peak Reduction 0 must give **no gain reduction at any level up to the +27 dBm equivalent** [2]
   [4].
2. **Frequency response.** Peak Reduction 0, 20 Hz to 20 kHz: response within **±1 dB** [2], and within ±0.5 dB
   from 40 Hz to 15 kHz. Assert the −3 dB points lie below 10 Hz and above 40 kHz.
3. **Threshold of limiting.** Peak Reduction 10, `la3a_emphasis` 0, 1 kHz sine: 1 dB of gain reduction occurs
   at **−30 dBu ±1.5 dB**, UREI's figure for the 50 dB position, which is the position the model is fixed in
   [2]. This is the model's primary calibration and it must be asserted at all three sample rates.
4. **Recommended operating point.** Peak Reduction 4.0, a 1 kHz sine at +6 dBu: gain reduction lands in the
   **3 to 5 dB** window UA recommends [1] (±1.5 dB tolerance on the window edges).
5. **Static curve, monotonicity and knee.** 1 kHz sines from −50 to +20 dBu at Peak Reduction
   {0, 2, 4, 6, 8, 10}: gain reduction is monotonically non-decreasing in both level and Peak Reduction; maximum
   gain reduction reaches **at least 38 dB and no more than 42 dB** at Peak Reduction 10 and +20 dBu ("Max Gain
   Reduction 40 dB" [1]); and the curve has no corner, i.e. the second derivative of the input/output curve stays
   under a fixed bound, as a soft-knee compressor must [53] [54].
6. **Compress versus Limit.** Below 3 dB of gain reduction the two modes differ by **less than 0.3 dB** ("the
   difference in these two modes is only present when the LA-3A is in deep compression" [1]). At the input level
   that gives 20 dB of reduction in Compress, Limit gives **at least 8 dB more**, and the local slope of the
   Limit curve in the 20-35 dB region exceeds **20:1** ("approaching 50:1" [2], "approximately 100:1" [8]).
7. **Control senses and defaults.** With every parameter at its default the model must be in the state the
   hardware ships in: Compress, HF Contour flat, meter reading Gain Reduction, and 3 to 5 dB of reduction on
   test 4's signal. Then assert the emphasis sense explicitly: **`la3a_emphasis = 0` gives a flat sidechain
   response** (400 Hz and 15 kHz sines at equal level produce gain reductions within 4 dB of each other) and
   **`la3a_emphasis = 1` gives the full 10 dB** of extra sensitivity at 15 kHz. This test exists because the
   control runs the opposite way round to the LA-2A's `opto_emphasis` (section 7.3), and a copy-paste from that
   engine would invert it silently while every other test still passed.

**Dynamics**

8. **Attack.** A 1 kHz tone stepping from −24 dBFS to −3 dBFS at Peak Reduction 6: 63 % of the final gain
   reduction is reached in **0.2 to 3 ms**, bracketing UREI's "less than 250 microseconds to 0.5 milliseconds"
   [2] and UA's "1.5 ms or less" [1]. A 6 dB step must attack **measurably slower** than an 18 dB step (level
   dependence [36] [37] [45]), by at least 20 % of the smaller time.
9. **Two-stage release.** After a 2 s burst held at 10 dB of gain reduction: reduction falls to 50 % in
   **40 to 120 ms** ("60 ms (50 % release)" [1], "40-80 milliseconds" [1]) and to 10 % in **0.5 to 5 s** [1]
   [2].
10. **Memory.** Compare a 100 ms burst with a 20 s burst, both at 20 dB of reduction: time to 90 % recovery
    after the long burst is **at least twice** that after the short one, and the long-burst tail to 99 % exceeds
    5 s [1] [2]. Re-running the same passage without resetting state must produce a different gain-reduction
    trace [8].
11. **Programme-dependent release, the manual's own claim.** Release after a −3 dB reduction is **faster** than
    release after a sustained −7 to −10 dB reduction, by at least a factor of 1.5 [1].

**Frequency-dependent behaviour**

12. **The built-in low-frequency deafness.** Equal-level 50 Hz and 1 kHz sines at Peak Reduction 6: the 50 Hz
    tone produces **at least 4 dB less** gain reduction than the 1 kHz tone (from the sidechain shape of
    section 7.4, itself derived from C6 and the autoformer; user report "a nice roll off in the side chain below
    100 Hz" [17]). This is the single most LA-3A-ish assertion in the file.
13. **HF Contour range.** With the contour flat, a 15 kHz sine and a 400 Hz sine at equal level produce gain
    reductions within 4 dB of each other. With the contour at maximum, the 15 kHz tone produces **10 ±2 dB more**
    gain reduction than the sub-1 kHz tone [1] [2], and the flat setting must be genuinely flat: sweeping the
    control from flat to maximum changes the 400 Hz gain reduction by less than 1.5 dB.
14. **The mid-forward push is emergent, not baked in.** With pink noise at 8 dB of average reduction, the
    output-to-input magnitude ratio averaged over the burst shows a rise of **1 to 4 dB in the 2 to 4 kHz band**
    relative to 200 Hz [16], and switching the compressor out (Peak Reduction 0) removes it entirely. Assert
    that the static frequency response with no reduction is flat (test 2), so the push cannot come from a fixed
    filter.

**Distortion, level and metering**

15. **Distortion.** 1 kHz at 0 VU with 6 dB of reduction: THD **below 0.5 %** [2]. At the +24 dBm equivalent
    with no reduction: THD **0.2 to 0.5 %** ("< 0.35 % THD @ +24 dBm" [1]). A 50 Hz tone driven to 15 dB of
    reduction: THD **below 1.0 %** ("will not exceed 0.7 %" [2], with tolerance). Broadband programme at 20 dB
    of reduction: THD below 0.4 % ("less than 0.3 %" [2]). The harmonic spectrum at 10 dB of reduction must
    contain **both a second and a third harmonic** above −90 dBc, because the hardware shows four overtones and
    the shipping plug-ins are criticised for producing only odd ones [12].
16. **The kick-drum test.** A loud kick sample at Peak Reduction 8 in Compress: the model must not flat-top.
    Assert that the output's crest factor after the initial hit stays above a floor and that the gain-reduction
    trace keeps moving for at least 300 ms rather than sitting at a rail, after the observation that "the
    plugins seem to choke/hard-limit even on 'compress' mode ... The real LA-3A is all over the place after the
    initial hit" [12].
17. **Meter.** VU ballistics: 99 % of reading in **300 ±30 ms** with 1.0-1.5 % overshoot [74] [75]. In Output
    mode a steady tone at **+4 dBu reads 0 VU ±0.3 dB** [1]. In GR mode the reading matches the attenuator's
    reduction within 0.5 dB at steady state. Off dims and bypasses [4].

**LA-2A versus LA-3A, same input, same file**

These run both engines from the same buffer and assert a *difference*. Each has a documented direction; none of
them is a taste judgement.

18. **Attack.** Calibrate both models to 10 dB of steady-state reduction on a 1 kHz tone, then step the input by
    18 dB. The LA-3A must reach 63 % of the new reduction **at least three times faster** than the LA-2A
    (1.5 ms or less against 10 ms [1] [2] and the LA-2A's own specification).
19. **Low-frequency sensitivity.** Calibrate both to 8 dB of reduction at 1 kHz, then feed 50 Hz at the same
    level. The LA-3A must produce **at least 4 dB less** gain reduction than the LA-2A [17], and the difference
    must survive setting both models' emphasis trimmers flat.
20. **Distortion.** At 6 dB of reduction and 0 VU, the LA-3A's THD must be **at least 6 dB lower** than the
    LA-2A's, and in absolute terms below 0.5 % where the LA-2A model sits at 0.8 % or above (measured LA-2A
    range 0.9-4.2 % [55]; LA-3A specification < 0.5 % [2]).
21. **Bandwidth.** With no reduction, the LA-3A's −1 dB points must lie **outside** the LA-2A's at both ends: at
    or below 20 Hz and at or above 20 kHz for the LA-3A [2], against 30 Hz and 15 kHz for the LA-2A.
22. **Headroom and output.** The level at which each model reaches 1 % THD must be **at least 10 dB higher** for
    the LA-3A (+24 dBm nominal, +27 dBm peaks [2], against +10 dBm nominal, +16 dBm peaks for the LA-2A).
23. **Release parity.** The two models' first-stage release must agree within 25 % (both specify 60 ms to 50 %
    [1]), and both must show the memory effect of test 10. This is a *negative* differentiator: it asserts the
    shared cell was not accidentally re-tuned, and it is the test that will fail first if somebody "improves"
    the cell for one model only.
24. **Release tail.** After identical 20 s bursts at 20 dB, the two models' 99 % recovery times must agree
    within a factor of 1.5. Same reasoning as test 23.

**Hygiene**

25. **Numerical robustness.** Ten minutes of digital silence after 30 s at 30 dB of reduction: no denormals
    (check the flush-to-zero flag and the per-block time), no NaN or infinity for inputs of ±10.0, DC, and
    silence; every state stays inside its bounds. The two new sidechain high-pass states must flush below
    `1e-12` (section 7.7).
26. **Sample-rate consistency.** Gain-reduction envelopes at 44.1, 48 and 96 kHz agree within **0.2 dB** for the
    same input, and the static curves within 0.1 dB.
27. **Stereo.** Identical mono material on both channels gives identical gain reduction. With `link` on, the
    reduction on hard-panned material lies between the two unlinked values, matching the hardware's "both units
    should compress the stereo signal equally, regardless of which side ... is triggering the gain reduction"
    [1] [79].
28. **Cell age.** Setting `la3a_cell` to Tired must reduce the gain reduction at a fixed Peak Reduction and
    level by **60 to 85 %** relative to Fresh ("up to 80 % less compression" [8]).
29. **Performance.** A ten-minute stereo render at 96 kHz with 2× oversampling of the output stage completes in
    under 5 % of real time on the reference machine, and the per-sample cost stays bounded (two transcendental
    calls, a `tanh`, a handful of one-poles).

---

## 9. References

Everything below was fetched and read while writing this file, except where an entry says otherwise. The two
manuals were located through Universal Audio's own manual index [78]. Forum
threads are cited for what a named person said, not as authorities; manufacturer documents are cited as
manufacturer claims, which is not the same thing as measurement.

1. Universal Audio, "Model LA-3A Audio Leveler" user manual, Universal Audio Manual Number 65-1301, revision
   1.53 (specifications, front and rear panel, stereo setup, compressor theory of operation, the T4 cell, the
   side-chain circuit, historical notes).
   https://media.uaudio.com/assetlibrary/l/a/la-3a-manual.pdf
2. Teletronix / UREI (United Recording Electronics Industries), "Model LA-3A Leveling Amplifier" product
   datasheet, two pages, with front and rear photographs and full technical specifications; the scan carries a
   Taber Manufacturing and Engineering Co. distributor imprint. John Leimseider archive, Internet Archive.
   https://archive.org/details/JL10842
3. UREI, "SCHEMATIC LA-3A", drawing number C11186, issue E, dated 4-14-70, Los Angeles, California.
   http://www.waltzingbear.com/Schematics/Urei/LA_3A.htm
4. Universal Audio Support, "Teletronix LA-3A Compressor Manual" (the plug-in manual; controls, history, HF
   Emphasis, Mix, ratios).
   https://help.uaudio.com/hc/en-us/articles/13925934548884-Teletronix-LA-3A-Compressor-Manual
5. Universal Audio blog, "Origins of the Teletronix LA-3A" (Brad Plunkett, 1969, the 1176 connection).
   https://www.uaudio.com/blogs/ua/teletronix-la-3a-origins
6. Universal Audio, "Teletronix LA-3A Classic Audio Leveler" plug-in product page.
   https://www.uaudio.com/products/uad-la-3a
7. Universal Audio Support, "LA-3A" (the hardware support page that links the manual).
   https://help.uaudio.com/hc/en-us/articles/206371653-LA-3A
8. Waves, "CLA-3A User Guide" (modelling notes, control ranges, −18 dBFS = +4 dBu reference, T4 depletion).
   https://assets.wavescdn.com/pdf/plugins/cla-3a-compressor-limiter.pdf
9. Waves, "CLA-3A Compressor / Limiter" product page.
   https://www.waves.com/plugins/cla-3a-compressor-limiter
10. Black Rooster Audio, "VLA-3A" product page (SPICE-style circuit simulation claims).
    https://blackroosteraudio.com/en/products/vla-3a
11. C. Schumacher, "Universal Audio LA-3A Audio Leveler reissue", Tape Op 49, September/October 2005.
    https://tapeop.com/reviews/gear/49/la-3a-audio-leveler-reissue
12. Gearspace forum, "LA3A shootout: dual UREI hardware vs UAD, Waves, Black Rooster plugins, null tests"
    (two-unit null test, harmonic comparison, kick-drum observation).
    https://gearspace.com/threads/la3a-shootout-dual-urei-hardware-vs-uad-waves-black-rooster-plugins-null-tests.1159435/
13. Gearspace forum, "LA3A HF contour".
    https://gearspace.com/threads/la3a-hf-contour.37923/
14. Gearspace forum, "Compressor side chain pre-emphasis: LA2A, LA3A".
    https://gearspace.com/threads/compressor-side-chain-pre-emphasis-la2a-la3a.343176/
15. Gearspace forum, "Attack and release times of LA-2A and LA-3A".
    https://gearspace.com/threads/attack-and-release-times-of-la-2a-and-la-3a.130340/
16. Gearspace forum, "UREI LA-3A midrange EQ effect: where is it?".
    https://gearspace.com/threads/urei-la-3a-midrange-eq-effect-where-is-it.641591/
17. Gearspace forum, "LA3A owners: MOD gain switch usage, vocal tips".
    https://gearspace.com/threads/la3a-owners-mod-gain-switch-usage-vocal-tips-la-3a.143664/
18. Gearspace forum, "Where to source replacement lamps for a vintage UREI LA-3A" (two 1819 lamps, 28 V).
    https://gearspace.com/threads/where-to-source-replacement-lamps-for-a-vintage-urei-la-3a.139106/
19. Gearspace forum, "UREI LA-3A identification: which version is what".
    https://gearspace.com/threads/urei-la-3a-indentification-which-version-is-what.479004/
20. GroupDIY forum, "LA-3A design thread: autotransformer, with pics" (B11178 data, autoformer measurements and
    the 100 Ω : 10 kΩ replacement specification).
    https://groupdiy.com/threads/la-3a-design-thread-autotransformer-with-pics.13566/
21. GroupDIY forum, "LA3A T4 autoformer info" (B11184 taps, Sowter, Studio Electronics and Hairball
    replacements).
    https://groupdiy.com/threads/la3a-t4-autoformer-info.88660/
22. GroupDIY forum, "LA3A calibration procedure" (R7, 12-13 V at TP-1).
    https://groupdiy.com/threads/la3a-calibration-procedure.89404/
23. GroupDIY forum, "T4B matching resistor for LA3A, and T4B versions comparison" (R13, meter calibration
    procedure, the 33 kΩ supplied resistor).
    https://groupdiy.com/threads/t4b-matching-resistor-for-la3a-and-t4b-versions-comparison.90915/
24. GroupDIY forum, "Questions for the LA3A gurus out there" (the four photocells on the original drawing; the
    output falling at high Peak Reduction).
    https://groupdiy.com/threads/questions-for-the-la3a-gurus-out-there.51117/
25. GroupDIY forum, "New Kenetek T4B opto-attenuators for your LA-2A, LA-3A and similar builds".
    https://groupdiy.com/threads/new-kenetek-t4b-opto-attenuators-for-your-la-2a-la-3a-and-similar-builds.72265/
26. GroupDIY forum, "Voltages for T4B in LA3A".
    https://groupdiy.com/threads/voltages-for-t4b-in-la3a.30911/
27. GroupDIY forum, "LA-3A driver amplifier" (on transplanting the transistor panel driver into a tube unit).
    https://groupdiy.com/threads/la-3a-driver-amplifier.66855/
28. GroupDIY forum, "Adding side-chain input to LA3A".
    https://groupdiy.com/threads/adding-sidechain-input-to-la3a.66765/
29. GroupDIY forum, "LA3A mods" (the circulated modification list: relocate Comp/Limit, link and power to the
    front, HF boost fully counter-clockwise for maximum, 50 dB position, 15 kΩ across R14).
    https://groupdiy.com/threads/la3a-mods.55052/
30. GroupDIY forum, "Electroluminescent panels for LA3A, T4B etc".
    https://groupdiy.com/threads/electroluminescent-panels-for-la3a-t4b-etc.45922/
31. Universal Audio, "Model LA-2A Leveling Amplifier, User's Guide" (reissue manual; the T4 text the LA-3A
    manual reuses, and the LA-2A specifications quoted for contrast).
    https://media.uaudio.com/assetlibrary/l/a/la-2a_manual.pdf
32. Universal Audio Support, "A History of the Teletronix LA-2A Leveling Amplifier" (Teletronix, Babcock, UREI,
    the end of LA-2A production).
    https://help.uaudio.com/hc/en-us/articles/215779663-A-History-of-the-Teletronix-LA-2A-Leveling-Amplifier
33. Universal Audio Support, "Teletronix LA-2A Leveler Collection Manual" (the LA-2A plug-in's ratio and
    frequency-dependence language, quoted for comparison).
    https://help.uaudio.com/hc/en-us/articles/4419496124180-Teletronix-LA-2A-Leveler-Collection-Manual
34. Wikipedia, "LA-2A Leveling Amplifier".
    https://en.wikipedia.org/wiki/LA-2A_Leveling_Amplifier
35. Wikipedia, "UREI".
    https://en.wikipedia.org/wiki/UREI
36. Clairex Corporation, photoconductive cells catalogue (CL-505L and CL-705 data; rise and decay against
    illumination). Internet Archive full text.
    https://archive.org/stream/TNM_Clairex_photoconductive_cells_-_Clairex_Corp_20180514_0017/TNM_Clairex_photoconductive_cells_-_Clairex_Corp_20180514_0017_djvu.txt
37. PerkinElmer Optoelectronics, "Photoconductive Cells" application note (rise and decay definitions, speed
    against light level, light history, gamma).
    https://cdn-learn.adafruit.com/assets/assets/000/010/129/original/APP_PhotocellIntroduction.pdf
38. Wikipedia, "Resistive opto-isolator" (turn-on against turn-off asymmetry; distortion against voltage across
    the cell).
    https://en.wikipedia.org/wiki/Resistive_opto-isolator
39. GL5528 CdS photoconductive cell datasheet (gamma definition and typical values).
    https://pi.gate.ac.uk/pages/airpi-files/PD0001.pdf
40. "ZnS:Sm and ZnS:Cu,Sm electroluminescent phosphors", Bulletin of Materials Science (the Alfrey-Taylor
    brightness relation used for the panel law).
    https://www.ias.ac.in/article/fulltext/boms/005/05/0405-0415
41. "On the relationship between photocurrent decay time and trap distribution in CdS and CdSe
    photoconductors", Solid-State Electronics.
    https://www.sciencedirect.com/science/article/abs/pii/0038110165900055
42. F. Eichas and U. Zölzer, "Modeling of an Optocoupler-Based Audio Dynamic Range Control Circuit",
    Proc. SPIE 9948, 2016.
    https://www.hsu-hh.de/ant/wp-content/uploads/sites/699/2017/10/Eichas-Modeling-of-an-optocoupler-based-audio-dynamic-range-control-circuit-99480W.pdf
43. F. Eichas, E. Gerat and U. Zölzer, "Virtual Analog Modeling of Dynamic Range Compression Systems",
    AES Convention 142, 2017.
    https://aes.org/publications/elibrary-page/?id=18628
44. J. Parker and S. D'Angelo, "A Digital Model of the Buchla Lowpass-Gate", DAFx-13.
    https://dafx.de/paper-archive/2013/papers/44.dafx2013_submission_56.pdf
45. J. Najnudel, R. Müller, T. Hélie and D. Roze, "Power-Balanced Dynamic Modeling of Vactrols: Application to
    a VTL5C3/2", DAFx23.
    https://www.dafx.de/paper-archive/2023/DAFx23_paper_50.pdf
46. A. Wright and V. Välimäki, "Grey-Box Modelling of Dynamic Range Compression", DAFx20in22.
    https://www.dafx.de/paper-archive/2022/papers/DAFx20in22_paper_35.pdf
47. C.-Y. Yu and G. Fazekas, "Sound Matching an Analogue Levelling Amplifier Using the Newton-Raphson Method",
    AES AIMLA 2025.
    https://arxiv.org/pdf/2509.10706
48. S. H. Hawley, B. Colburn and S. I. Mimilakis, "SignalTrain: Profiling Audio Compressors with Deep Neural
    Networks", AES Convention 147, 2019.
    https://arxiv.org/abs/1905.11928
49. C. J. Steinmetz and J. D. Reiss, "Efficient neural networks for real-time modeling of analog dynamic range
    compression", AES Convention 152, 2022.
    https://ar5iv.labs.arxiv.org/html/2102.06200
50. R. Simionato and S. Fasciani, "Deep Learning Conditioned Modeling of Optical Compression", DAFx20in22.
    https://dafx2020.mdw.ac.at/proceedings/papers/DAFx20in22_paper_6.pdf
51. R. Simionato and S. Fasciani, "Fully Conditioned and Low-latency Black-box Modeling of Analog
    Compression", DAFx23.
    https://www.dafx.de/paper-archive/2023/DAFx23_paper_10.pdf
52. R. Simionato and S. Fasciani, "Modeling Time-Variant Responses of Optical Compressors with Selective State
    Space Models", JAES 73(3), 2025.
    https://arxiv.org/html/2408.12549
53. D. Giannoulis, M. Massberg and J. D. Reiss, "Digital Dynamic Range Compressor Design: A Tutorial and
    Analysis", JAES 60(6), 2012 (knee and curve definitions used in test 5).
    https://www.aes.org/e-lib/download.cfm?ID=16354
54. U. Zölzer (ed.), "DAFX: Digital Audio Effects", Wiley (dynamics processing chapter; the reference text for
    the detector and curve conventions used here).
    https://www.dafx.de/DAFX_Book_Page_2nd_edition/index.html
55. A. Moore, "Objective Analysis and Perceptual Evaluation of LA-2A Compressors and Vocal Recordings",
    University of Huddersfield (six-unit LA-2A measurement study, quoted throughout for contrast).
    https://pure.hud.ac.uk/ws/portalfiles/portal/140787498/AAM.pdf
56. I. Sobczyk, IGS Audio, "T4Bx photocell: learn how 'the sound' is created".
    https://igsaudio.com/wp-content/uploads/2024/05/qXh3AjYh.pdf
57. ProReplicas, "T4B Opto-Attenuator".
    https://www.proreplicas.com/t4b_cell.html
58. AudioScape Engineering, "Why do we make our own T4B Optical Cells?".
    https://www.audio-scape.com/news/t4b
59. DIYRE wiki, "Kenetek T4B Opto-Attenuator Cell".
    https://wiki.diyrecordingequipment.com/projects/kenetek-t4b-opto-attenuator-cell/
60. Softube, "OPTO Compressor" user manual (an LA-2A model, listed so it is not mistaken for an LA-3A one).
    https://www.softube.com/user-manuals/opto-compressor
61. IK Multimedia, "T-RackS White 2A Leveling Amplifier" (likewise an LA-2A model; I found no LA-3A model in
    the T-RackS range).
    https://www.ikmultimedia.com/products/trwhite2a/
62. Sound On Sound, "Classic Compressors".
    https://www.soundonsound.com/techniques/classic-compressors
63. Hairball Audio, "EA-11184 Autoformer" (the LA-3A side-chain autotransformer replacement).
    https://www.hairballaudio.com/catalog/parts-store/audio-transformers/ea-11184-autoformer-
64. Studio Electronics (David Kulka), LA-3A autoformer replacement part.
    https://www.studioelectronics.biz/sunshop/index.php?l=product_detail&p=1169
65. Cinemag, transformer catalogue (CM-2511, cited by builders as an LA-3A autoformer substitute; see [21]).
    https://www.cinemag.biz/
66. Gearspace forum, "LA-3A shootout: three plugins compared".
    https://gearspace.com/threads/la-3a-shootout-three-plugins-compared.857747/
67. Gearspace forum, "Is an LA-3A the perfect midpoint between LA-2A and 1176?".
    https://gearspace.com/threads/is-an-la-3a-the-perfect-midpoint-between-la-2a-and-1176.1386091/
68. Gearspace forum, "WTF is the UA LA3A doing to my guitar tracks?".
    https://gearspace.com/threads/wtf-is-the-ua-la3a-doing-to-my-guitar-tracks.186507/
69. Gearspace forum, "Best LA3A clone currently available".
    https://gearspace.com/threads/best-la3a-clone-currently-available.1318430/
70. Gearspace gear database, "Bomb Factory LA-3A 3.0 HD".
    https://gearspace.com/gear/bomb-factory-bomb-factory-bomb-factory-la-3a-3-0-hd.14371/
71. Gearspace forum, "Universal Audio LA-3A: switches on rear".
    https://gearspace.com/threads/universal-audio-la-3a-switches-on-rear.440034/
72. Gearspace forum, "LA3A plugin vs hardware shootout".
    https://gearspace.com/threads/la3a-plugin-vs-hardware-shootout.1412967/
73. Gearspace forum, "What are the differences btw LA2A and LA3A".
    https://gearspace.com/threads/what-are-the-differences-btw-la2a-and-la3a.1182252/
74. Prism Sound glossary, "VU Meter" (ANSI C16.5-1942 ballistics).
    http://www.prismsound.com/define.php?term=VU_Meter
75. EDN, "Analog VU Meters & Quick Pointers".
    https://www.edn.com/analog-vu-meters-quick-pointers/
76. CML Innovative Technologies, CM1819 lamp datasheet (the 28 V meter lamp; cited through [18], not fetched
    directly).
    http://www.cml-it.com/pdf/2-54.pdf
77. Universal Audio blog, "The Best Compressor Plug-Ins".
    https://www.uaudio.com/blogs/ua/best-compressor-plugins
78. Universal Audio Support, "Analog Hardware Manuals" (the index that locates the LA-3A and LA-2A manual PDFs).
    https://help.uaudio.com/hc/en-us/articles/12264250115476-Analog-Hardware-Manuals
79. Gearspace forum, "Stereo LA3As: linked or unlinked".
    https://gearspace.com/threads/stereo-la3as-linked-or-unlinked.466922/

[1]: https://media.uaudio.com/assetlibrary/l/a/la-3a-manual.pdf
[2]: https://archive.org/details/JL10842
[3]: http://www.waltzingbear.com/Schematics/Urei/LA_3A.htm
[4]: https://help.uaudio.com/hc/en-us/articles/13925934548884-Teletronix-LA-3A-Compressor-Manual
[5]: https://www.uaudio.com/blogs/ua/teletronix-la-3a-origins
[6]: https://www.uaudio.com/products/uad-la-3a
[7]: https://help.uaudio.com/hc/en-us/articles/206371653-LA-3A
[8]: https://assets.wavescdn.com/pdf/plugins/cla-3a-compressor-limiter.pdf
[9]: https://www.waves.com/plugins/cla-3a-compressor-limiter
[10]: https://blackroosteraudio.com/en/products/vla-3a
[11]: https://tapeop.com/reviews/gear/49/la-3a-audio-leveler-reissue
[12]: https://gearspace.com/threads/la3a-shootout-dual-urei-hardware-vs-uad-waves-black-rooster-plugins-null-tests.1159435/
[13]: https://gearspace.com/threads/la3a-hf-contour.37923/
[14]: https://gearspace.com/threads/compressor-side-chain-pre-emphasis-la2a-la3a.343176/
[15]: https://gearspace.com/threads/attack-and-release-times-of-la-2a-and-la-3a.130340/
[16]: https://gearspace.com/threads/urei-la-3a-midrange-eq-effect-where-is-it.641591/
[17]: https://gearspace.com/threads/la3a-owners-mod-gain-switch-usage-vocal-tips-la-3a.143664/
[18]: https://gearspace.com/threads/where-to-source-replacement-lamps-for-a-vintage-urei-la-3a.139106/
[19]: https://gearspace.com/threads/urei-la-3a-indentification-which-version-is-what.479004/
[20]: https://groupdiy.com/threads/la-3a-design-thread-autotransformer-with-pics.13566/
[21]: https://groupdiy.com/threads/la3a-t4-autoformer-info.88660/
[22]: https://groupdiy.com/threads/la3a-calibration-procedure.89404/
[23]: https://groupdiy.com/threads/t4b-matching-resistor-for-la3a-and-t4b-versions-comparison.90915/
[24]: https://groupdiy.com/threads/questions-for-the-la3a-gurus-out-there.51117/
[25]: https://groupdiy.com/threads/new-kenetek-t4b-opto-attenuators-for-your-la-2a-la-3a-and-similar-builds.72265/
[26]: https://groupdiy.com/threads/voltages-for-t4b-in-la3a.30911/
[27]: https://groupdiy.com/threads/la-3a-driver-amplifier.66855/
[28]: https://groupdiy.com/threads/adding-sidechain-input-to-la3a.66765/
[29]: https://groupdiy.com/threads/la3a-mods.55052/
[30]: https://groupdiy.com/threads/electroluminescent-panels-for-la3a-t4b-etc.45922/
[31]: https://media.uaudio.com/assetlibrary/l/a/la-2a_manual.pdf
[32]: https://help.uaudio.com/hc/en-us/articles/215779663-A-History-of-the-Teletronix-LA-2A-Leveling-Amplifier
[33]: https://help.uaudio.com/hc/en-us/articles/4419496124180-Teletronix-LA-2A-Leveler-Collection-Manual
[34]: https://en.wikipedia.org/wiki/LA-2A_Leveling_Amplifier
[35]: https://en.wikipedia.org/wiki/UREI
[36]: https://archive.org/stream/TNM_Clairex_photoconductive_cells_-_Clairex_Corp_20180514_0017/TNM_Clairex_photoconductive_cells_-_Clairex_Corp_20180514_0017_djvu.txt
[37]: https://cdn-learn.adafruit.com/assets/assets/000/010/129/original/APP_PhotocellIntroduction.pdf
[38]: https://en.wikipedia.org/wiki/Resistive_opto-isolator
[39]: https://pi.gate.ac.uk/pages/airpi-files/PD0001.pdf
[40]: https://www.ias.ac.in/article/fulltext/boms/005/05/0405-0415
[41]: https://www.sciencedirect.com/science/article/abs/pii/0038110165900055
[42]: https://www.hsu-hh.de/ant/wp-content/uploads/sites/699/2017/10/Eichas-Modeling-of-an-optocoupler-based-audio-dynamic-range-control-circuit-99480W.pdf
[43]: https://aes.org/publications/elibrary-page/?id=18628
[44]: https://dafx.de/paper-archive/2013/papers/44.dafx2013_submission_56.pdf
[45]: https://www.dafx.de/paper-archive/2023/DAFx23_paper_50.pdf
[46]: https://www.dafx.de/paper-archive/2022/papers/DAFx20in22_paper_35.pdf
[47]: https://arxiv.org/pdf/2509.10706
[48]: https://arxiv.org/abs/1905.11928
[49]: https://ar5iv.labs.arxiv.org/html/2102.06200
[50]: https://dafx2020.mdw.ac.at/proceedings/papers/DAFx20in22_paper_6.pdf
[51]: https://www.dafx.de/paper-archive/2023/DAFx23_paper_10.pdf
[52]: https://arxiv.org/html/2408.12549
[53]: https://www.aes.org/e-lib/download.cfm?ID=16354
[54]: https://www.dafx.de/DAFX_Book_Page_2nd_edition/index.html
[55]: https://pure.hud.ac.uk/ws/portalfiles/portal/140787498/AAM.pdf
[56]: https://igsaudio.com/wp-content/uploads/2024/05/qXh3AjYh.pdf
[57]: https://www.proreplicas.com/t4b_cell.html
[58]: https://www.audio-scape.com/news/t4b
[59]: https://wiki.diyrecordingequipment.com/projects/kenetek-t4b-opto-attenuator-cell/
[60]: https://www.softube.com/user-manuals/opto-compressor
[61]: https://www.ikmultimedia.com/products/trwhite2a/
[62]: https://www.soundonsound.com/techniques/classic-compressors
[63]: https://www.hairballaudio.com/catalog/parts-store/audio-transformers/ea-11184-autoformer-
[64]: https://www.studioelectronics.biz/sunshop/index.php?l=product_detail&p=1169
[65]: https://www.cinemag.biz/
[66]: https://gearspace.com/threads/la-3a-shootout-three-plugins-compared.857747/
[67]: https://gearspace.com/threads/is-an-la-3a-the-perfect-midpoint-between-la-2a-and-1176.1386091/
[68]: https://gearspace.com/threads/wtf-is-the-ua-la3a-doing-to-my-guitar-tracks.186507/
[69]: https://gearspace.com/threads/best-la3a-clone-currently-available.1318430/
[70]: https://gearspace.com/gear/bomb-factory-bomb-factory-bomb-factory-la-3a-3-0-hd.14371/
[71]: https://gearspace.com/threads/universal-audio-la-3a-switches-on-rear.440034/
[72]: https://gearspace.com/threads/la3a-plugin-vs-hardware-shootout.1412967/
[73]: https://gearspace.com/threads/what-are-the-differences-btw-la2a-and-la3a.1182252/
[74]: http://www.prismsound.com/define.php?term=VU_Meter
[75]: https://www.edn.com/analog-vu-meters-quick-pointers/
[76]: http://www.cml-it.com/pdf/2-54.pdf
[77]: https://www.uaudio.com/blogs/ua/best-compressor-plugins
[78]: https://help.uaudio.com/hc/en-us/articles/12264250115476-Analog-Hardware-Manuals
[79]: https://gearspace.com/threads/stereo-la3as-linked-or-unlinked.466922/

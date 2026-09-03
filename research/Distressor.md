# The Empirical Labs Distressor: research notes for the Distressor side of `noob-compressorlab`

Research dossier for the Distressor model of the `noob-compressorlab` example plug-in of noob-vst-webgui-framework.
The example is a humorous, affectionate spoof of the Empirical Labs EL8 Distressor (and its EL8-X variant with
British Mode and Image Link). It is not a product and does not use the Distressor, Empirical Labs, Arousor, 1176 or
LA-2A names as its own name. Trademarks below belong to their owners and are used only to identify the device and
the products discussed. This model sits behind the same per-instance `model` switch that already selects the 1176
and LA-2A behaviours; see [[1176]] and [[LA-2A]].

Conventions (kept the same as the 1176 and LA-2A dossiers so the three read alike):

- Citations are `[n]`; the numbered list in section 9 gives the URL for every source, and reference-style link
  definitions at the very end make the `[n]` markers clickable.
- Numbers that come from a manufacturer specification, a manual or a measurement are attributed. Numbers that are my
  own derivation or assumption are labelled **estimate** or **derived**. Nothing labelled a measurement was invented;
  where sources disagree, both figures are given.
- "GR" is gain reduction. "THD" is total harmonic distortion. "VCA" is a voltage-controlled amplifier. dBu is
  0.775 V RMS; dBFS is digital full scale.
- The Distressor is a spoof target, not a parity goal. I want the *feel* of the eight ratios, the knee, the two
  distortion colours and the Nuke/Opto/British tricks, not a component-accurate clone. Empirical Labs never published
  a schematic, and asked for the reverse-engineered ones to be taken down [42][43], so a lot of the circuit detail
  below is inference, clearly marked as such.

---

## 1. What the Distressor is

### 1.1 Origin and the designer

The Distressor was designed by **Dave Derr**, founder of Empirical Labs of Lake Hiawatha (later New Providence),
New Jersey. Derr was a music major in college (classical piano, composition, theory), then a professional musician
for twelve years in the Philadelphia-area band Jack of Diamonds, signed to Ransom Records and distributed by RCA
[19]. He taught himself electronics as a technician at a medical company, then joined Eventide around 1986 as an
engineer, where over roughly nine years he worked on the analog section, interface and presets of the H3000
Ultra-Harmonizer alongside Ken Bogdanowicz and Bob Belcher [19][11]. He ran a 24-track analog studio in Garfield,
New Jersey, and it was there, around 1989, that a more experienced engineer at the studio, **John Patterson**,
steered him toward a Valley People Gain Brain, a pair of UREI 1176LNs and an LA-2A. Derr calls these his "mojo
compressors": boxes that did not merely level the signal but "added excitement, presence and size" [15]. The
Distressor is his attempt to build one modern, consistent, reliable box that captures what he loved about all three
[15][22][3].

The name is a portmanteau of **distortion** and **compressor** [14]. The faceplate subtitle is literally
"CLASSIC KNEE COMPRESSION" [1, cover]. John Patterson, who coined the name "Nuke", died before the manual was
printed; the manual is dedicated to his memory [1, p. 15].

### 1.2 Dates and how it caught on

The sources disagree on the year, and it is worth stating the spread rather than picking one:

- Derr says he spent "three or four years tweaking" the Distressor after leaving Eventide, using it on his own
  sessions before he trusted it on clients [19].
- Wikipedia and the SonicScoop interview: first units **sold in 1995**, Empirical Labs **founded in 1996** [14][15].
- Reverb (from an interview with Derr): "first prototyped in 1994 and officially released in 1996" [17].
- Universal Audio's plug-in copy and the SonicScoop review say "originally released in **1993**" [11][16]; I treat
  1993 as the design-start year, not the ship date, because it conflicts with Derr's own "sold in 1995" [15][19].
- The turning point was a 1996 review in *Mix* magazine, pushed for by George Peterson, plus early adoption by
  George Massenburg and Mutt Lange, and Fletcher of Mercenary Audio insisting Derr show it at the 1995 AES show in
  New York. Distribution was handled by Gil Griffith [15][19]. Sales climbed from "one here, one there" into the
  dozens, hundreds, and by 1999 the thousands [15].

Empirical Labs now quotes **over 38,000 units** in the field (the 2024 product page and Mixdown) and one interview
cites **42,000** by 2022 [3][22][13]. It entered the TECnology Hall of Fame in 2016 [14]. A "prominent recording
engineer recently wagered that there probably was not a top 40 record made in the last five years that didn't have
at least one Distressor on it" [3].

### 1.3 Versions

| Model | What it is | Sources |
|---|---|---|
| **EL8** | The original mono, single-channel 1U unit. Transformerless in and out. | [1][3] |
| **EL8-S** | A pair of EL8s matched at the factory as a stereo pair, shipped with the link cables. | [3] |
| **EL8-X** | EL8 plus two factory modifications: **British Mode** (a toggle that applies an 1176 "all-buttons-in" character to any ratio, with 1:1 becoming the dedicated "British" ratio) and **Stereo Image Link** (a third linking mode). New metal faceplate. An EL8 can be upgraded to an EL8-X for a fee. | [1, p. 12-13][3][20] |
| **EL8X-S** | A stereo pair of EL8-X units. | [3] |
| **EL8-XXX** | 30th-anniversary limited edition, announced NAMM 2026. All EL8-X features, plus a **Triad HS-56 input transformer** (drops input impedance to 600 ohms, adds low-end "size" and earlier saturation), different discrete parts so it "saturates earlier than the classic EL8", and a red faceplate carrying the logo from Derr's 1994 prototype. | [3][29][30] |
| **Arousor** | Empirical Labs' own native plug-in. Not a straight emulation: Derr shifted the ratio names up one slot to "idealise" them (see 4.2), added 1.5:1 and 2:1 ratios, "Rivet" (a Nuke-derived brick-wall), AtMod (attack modification), a Soft Clipper, a two-band detector sidechain EQ, and two Opto modes modelling an early-1980s "T4A" and a 2018 "T4B" optocoupler. | [7][8][24] |

The Universal Audio **UAD EL8 Distressor** plug-in (2018) is the only third-party emulation officially endorsed by,
and co-developed with, Empirical Labs [3][11][17].

### 1.4 Why it is famous

It is the "Swiss Army knife, Desert Island compressor" [15]: eight ratios each with its own knee and detector
behaviour, two flavours of programmable distortion, sidechain filtering, and a range that runs from "no compression,
just warmth" (1:1) to brick-wall room-mic destruction (Nuke). It is digitally controlled analog, so the settings are
repeatable and unit-to-unit consistent, which the vintage boxes it emulates are not [3][22]. F. Reid Shippen called
it "the best thing to happen to compression in the last twenty years" [20]; *Mix* wrote "this product is actually a
classic already" [1, p. 14]. The four big white knobs are patented and trademarked and are instantly recognisable in
studio photos [22].

---

## 2. Controls, front panel and their real ranges

The EL8-X front panel, left to right along the top: the **Distressor** logo; a small grey **BYPASS** button; a
16-LED **GAIN REDUCTION** bargraph; a clipping-waveform icon with **REDLINE** and **1% THD** LEDs; the **RATIO**
button with its eight ratio LEDs; the **DETECTOR** button with HP / Band-emphasis / Link LEDs; the **AUDIO** button
with HP / Dist 2 / Dist 3 LEDs; the **POWER** switch. Along the bottom: four large white knobs, **INPUT**,
**ATTACK**, **RELEASE**, **OUTPUT**, numbered 0 to 10 (and physically a little past 10, to about 10.5) [2][33]. On
the EL8-X, two toggle switches sit between the knobs: **Stereo Image Link** (yellow silkscreen) above the Attack
knob, and **British Mode (1:1)** (yellow silkscreen) between Attack and Release; the Attack knob is marked
**Opto(10)** at its slow end and the Release knob **Opto(0)** at its fast end [faceplate photos, Leeds guide 56].

The UAD plug-in adds two controls that the hardware does not have: a small **HR** (Headroom) set-screw at far left
and a **MIX** knob at far right [11].

| Control | Range as published | Direction | What it does |
|---|---|---|---|
| Input | Knob 0-10.5 (arbitrary scale) | CW = more level, more compression | There is no threshold control. The internal threshold is fixed and Input drives the signal up over it, setting the amount of GR. The amount of threshold shift is relative to the selected ratio. [1][11][26] |
| Attack | 50 µs (0, fastest) to 30 ms (10, slowest) | CW = slower | Sets time to onset of GR. At 0 with 2:1/3:1/4:1 the attack can go even faster than 50 µs. Time constants interact with the ratio. [1][11][33] |
| Release | 0.05 s (0, fastest) to 3.5 s (10) | CW = slower | Sets recovery time. In 10:1 Opto, release can extend to 20 s from program-dependent behaviour. [1][11][26] |
| Output | Knob 0-10.5 | CW = louder | Make-up gain after the compressor and distortion. For a quick +4 dBu tape level try Output at 8. [1] |
| Ratio | 1:1, 2:1, 3:1, 4:1, 6:1, 10:1 (Opto), 20:1, Nuke | button cycles | Each ratio sets both a threshold offset and a ratio/knee. 2:1, 10:1 and Nuke use special detector circuitry. See 4.2. [1][11] |
| Detector | Norm, HP, Band Emphasis, Link, and combinations (8 states) | button cycles | Sidechain filtering; does not filter the audio, only what the compressor "sees". [1][11] |
| Audio | Norm, HP, Dist 2, Dist 3, and HP combinations (6 states) | button cycles | Audio-path high-pass and the two distortion colours. [1][11] |
| Bypass | on/off | dedicated button | Hard relay bypass, input wired straight to output. GR meter stays live. [1][11] |
| Stereo Image Link (EL8-X) | toggle | up = on | Adds the gain-control-summing link mode. See 3.7. [1, p. 13] |
| British Mode (EL8-X) | toggle | up = on | Applies the 1176 all-buttons character to the current ratio; 1:1 is the dedicated British ratio. See 3.6. [1, p. 12] |
| Headroom (UAD only) | 4, 8, 12, 16, 20, 24, 28 dB; default 16 | set-screw | Internal operating-reference level. Lower dB value = pushed harder = more colour. Not on the hardware. [11] |
| Mix (UAD only) | Dry to Comp, default Comp | knob | Wet/dry parallel blend, phase-accurate. Not on the hardware. [11] |

### 2.1 LED colours (for the faceplate)

From the front-panel photographs and the reviews:

- **Gain reduction bargraph**: 16 LEDs silkscreened, from the right (least) to left (most): 1, 2, 3, 4, 5, 6, 7, 8,
  9, 10, 12, 14, 17, 20, 23, 26 (dB). SonicScoop's review reads the colours as 1-6 dB green, 7-10 dB yellow, 12-26 dB
  red [16]. The photo shows roughly 1-5 green, 6-10 yellow/amber, 12-26 red/orange [faceplate photo]; I treat the
  boundary between green and yellow as **estimate** around 5-6 dB.
- **BYPASS**: grey button, red LED when bypassed [1][16].
- **REDLINE**: red LED, lights at ~3% THD and also indicates output clipping. **1% THD**: yellow LED. On the
  hardware the manual text says this yellow LED lights at 0.25% THD even though the silkscreen reads "1% THD"; the
  UAD plug-in lights its 1% LED at ~1% THD [1, p. 5][11]. Flag this inconsistency in the tribute rather than hide it.
- **Ratio LEDs**: 1:1 red, 2:1 green, 3:1 green, 4:1 green, 6:1 yellow, 10:1 (Opto) yellow/amber, 20:1 red, **Nuke
  blue** ("just to be cool" [18]) [faceplate photo].
- **Detector LEDs**: HP green, Band Emphasis yellow, Link red [11][faceplate photo].
- **Audio LEDs**: HP green, Dist 2 yellow, Dist 3 red [1, p. 5][11].
- **Stereo Image Link** toggle: red LED. **British Mode** toggle: orange/amber LED [faceplate photo, Leeds guide 56].

The manufacturer's own note on the colour scheme: "The color encoded indicator LED's were arranged in an easy to
read pattern, with Red LEDs usually indicating radical or distorted settings" [1, p. 10].

### 2.2 Where to start (manufacturer's advice)

"Where to start - 5 5 5 5": 6:1 ratio, all four knobs at 5, then raise Input for the GR you want; only the 6:1 LED
should be lit [1][11]. Caution repeated everywhere: cycling through 1:1 while the unit is working turns compression
off and the signal swells to peak level, "possibly becoming dangerously loud", so wait for a pause before changing
ratio [1][11].

---

## 3. Signal path and circuit behaviour

### 3.1 Block diagram (manual, figure on p. 6; UAD's simplified version)

Main audio path, in order:

```
Balanced In -> Dif Amp -> Input Gain -> VCA -> Distortion Generator -> 80 Hz High Pass Filter
   -> Output Amp -> (Master Bypass) -> Output Gain -> Active Outputs (XLR / 1/4")
```

Detector (sidechain) circuit, fed from the audio *after* the gain change:

```
tap -> Band Emphasis (In/Out) -> Sidechain HP Filter (In/Out) -> Detector and Envelope Gen (diode)
   -> Control Voltage -> VCA
Link Sum <- Link In;  Detector/envelope -> front-panel bargraph;  Image Link Mod switch (EL8-X)
```

Two structural facts matter for the model. First, the detector is fed from the compressed signal, so the Distressor
is a **feedback** compressor, like the 1176 and LA-2A [16][14 (sound-freqs)][26]. Second, the **distortion generator
is a separate block after the VCA**, not a by-product of the gain cell; the audio 80 Hz high-pass sits after it, and
the whole colour/filter chain is switchable in and out [1, p. 6][11].

### 3.2 The gain cell (a VCA), and why I say "THAT/dbx-style"

Empirical Labs describes the Distressor as using "a custom designed gain control circuit" [3] and, more plainly, as
a VCA soft-knee compressor [26]. No schematic is public; Empirical Labs had the reverse-engineered versions removed
from GroupDIY [42][43]. What can be said:

- It is **digitally controlled analog**: a microcontroller reads the buttons and sets the analog switching, and a
  non-volatile capacitor (a "gold cap") stores the last front-panel settings for about four weeks; older units also
  took two batteries, newer ones wait six seconds before committing a change to memory [26][43][6].
- Derr repeatedly frames the gain element as a **VCA** (contrasting it with the optocoupler "VCA" of an LA-2A) and
  says the Dist 3 third harmonic is "induced by increasing VCA output level" [1, p. 5][8].
- The industry-standard exponential-control VCA of that era, used in almost every VCA compressor, is the
  Blackmer-topology **THAT 2180/2181** (successor to the dbx 202/2150). Its control law is exponential in decibels at
  about **-6.1 mV/dB** on the negative control port, gain range >130 dB, dynamic range >120 dB, THD ~0.005% (~0.0025%
  for the 2181A grade), 20 MHz bandwidth, with a symmetry-trim pin to null the distortion [45]. GroupDIY threads
  routinely assume a THAT/dbx VCA for the Distressor [44]. I therefore model the gain cell as a **THAT/dbx-style
  Blackmer VCA** driven by a control voltage in the log (dB) domain. This is a strong, standard inference, labelled
  as such, not a confirmed part choice.

The consequence for the model: gain reduction is naturally computed and applied in the **dB/log domain**, and the VCA
adds its own gentle THD that rises with drive (the 2181 THD-versus-level curves [45]) on top of the deliberate
distortion generator.

### 3.3 The detector and program-dependent time constants

The block diagram draws the detector as a diode "Detector and Envelope Gen" fed from the post-VCA signal, i.e. a
diode (peak/quasi-peak) envelope follower rather than a true-RMS detector [1, p. 6]. Attack and Release set the
charge/discharge of the envelope, but their effect is deliberately **program- and ratio-dependent**: "the effective
attack/release characteristics change with the ratio", so fast attack at 2:1 is not the same as fast attack at 4:1
[33][2]. Several ratios (2:1, 10:1, Nuke) "employ special detector circuitry" that gives them their own knee and
release shape [1, p. 4]. The Nuke release is explicitly **logarithmic** (fast at first, then slowing), which Derr
calls "a big part of the Distressor's sound" [1]. The Opto release stretches to 20 s and is program dependent,
echoing the LA-2A's two-stage recovery [1][26][3]; see [[LA-2A]] section 4.3 for the physics I borrow there.

Because the loop is a feedback loop, the same "ratio collapse" and program dependence I documented for the 1176 apply
here (see [[1176]] section 3.7): the effective ratio and the attack lag emerge from loop gain rather than from a gain
computer, which is exactly why Derr found the measured ratios ran higher than the labels (4.2). Floru's THAT papers
give the feedback-loop math directly: a feedback compressor halves the effective RMS-detector integration time
constant compared with the feedforward case, and the log-domain detector's release rate of about 120 dB/s "provides
the best sound without introducing noticeable distortion" [48][49].

### 3.4 The ratio circuitry: threshold and knee both move

"Each 'ratio mode' of the Distressor sets both the threshold and the ratio, in the standard sense of the word" [1,
p. 4]. Raising the ratio button also moves the threshold and hardens the knee. The soft knee is defined by Empirical
Labs as "a compression curve where the first few dB of gain reduction occur at very low ratios, gradually increasing
as the signal increases ... The knee usually extends for a few dB and gradually flattens out toward a final ratio"
[1, p. 4]. The important quantitative claims:

- 1:1: no compression at all; audio just passes through the warming/distortion circuits.
- 2:1 and 3:1: "parabolic" knees, very gentle, no hard limiting and therefore no overload protection. The 2:1 knee
  can be "as long as 30 dB, depending on attack and decay settings", with a nominal "+15 dB knee" quoted in the
  quick-start [1, p. 4].
- 4:1 and 6:1: steeper knees, moving toward hard limiting; 6:1 is the general-purpose vocal/bass/acoustic setting.
- 6:1 and 10:1 Opto: "shorter knee limiting, reminiscent of some old classics from the 60's and 70's" [1].
- 20:1 and Nuke: dominant (hard) knees, brick-wall, "keeping any normal signal within 1 dB or so" [1].

### 3.5 The distortion generator (Dist 2 / Dist 3) and the audio high-pass

The distortion is a deliberate, trimmed generator, not amp clipping. Empirical Labs' framing: the Distressor is "a
modern digitally controlled analog device that attempts to offer some of the 'musical non-linearities' exhibited by
the older tube, class A discrete, and magnetic tape mediums" [1, p. 5]. Three modes:

| Mode | LED | THD range | Character |
|---|---|---|---|
| Normal (clean) | none | 0.025% to 0.3% | No induced distortion. [1, p. 5] |
| Dist 2 | yellow | 0.05% to 3% | Emphasised 2nd harmonic; "Class A" tube-like warmth; mostly 2nd harmonic when compressing. [1, p. 5] |
| Dist 3 | red | 0.1% to 20% | 3rd harmonic increased (plus some 2nd); tape-like; flattens the top and bottom of the waveform; "induced by increasing VCA output level". [1, p. 5] |

Derr's own account of the maths: symmetrical soft clipping produces only odd harmonics, so Dist 3 (symmetric, third
harmonic) came almost for free, and the team "spent a bit of time getting the even harmonics to sound right" for
Dist 2, because "older analog gear usually added some even harmonics from power supply and component variations"
[22]. The tube analogy is triode curves ("smooth triode curves that are the most tube-like") [22]. The distortion is
intentionally modest: it is not guitar-amp distortion but the "grungier circuits from the early days of audio, when
designers had to kick and scrap to get 1% THD out of a tube or transistor" [3]. You make it more obvious by slowing
the attack (peaks hit the generator harder) or quickening the release (signal is sucked back up to hot levels) [3].

The **audio high-pass** (Audio HP) is an 80 Hz, 18 dB/octave Bessel filter, about 3 dB down at 65 Hz and 12 dB down
at 30 Hz [1, p. 6][11]. It rolls "sub" mud out of the audio, and can be combined with either distortion mode.

### 3.6 British Mode (the 1176 "all-buttons-in" trick)

The original 1176 all-buttons behaviour is documented in [[1176]] section 4.5; British Mode is the Distressor's
switchable version of it. Empirical Labs' history: renegade engineers found that pressing all four 1176 ratio buttons
at once left them "in", giving "a very, very aggressive sound that had some elements of the unit's 20:1 ratio, but
with an unusual knee and new envelope shape" [4]. On the EL8-X a dedicated toggle applies this aggressive character
to **any** ratio, and 1:1 becomes the dedicated "British" ratio [1, p. 12][3].

Details that matter for modelling:

- Installing the mod changes what 1:1 does *even with the switch off*: 1:1 is no longer 1:1 but roughly **10:1**,
  "depending on the attack, decay", because the curve that now lives in the 1:1 slot is designed to work with the
  toggle. Derr confirmed this personally to a user; the original behaviour can be restored with an internal jumper
  [38]. To get harmonic distortion with no compression on a modded unit you use 1:1, British Mode on, Attack 10
  (Opto), which "interacts to make the threshold much much higher" [38].
- To keep the 1176LN character you must keep Attack under 3 or 4; above that the unit incurs "a rise in some grunge
  (distortion)" and the THD LEDs light more, and it "will no longer behave smoothly, nor like an 1176" [4].
- The behaviour is "a really non-linear beast, with sped-up behavior. Very hard to quantify ... almost a different
  product when engaged" [17]. It is best used to "skim peaks": mostly transparent, but when a peak hits it pushes
  back smoothly and gets out of the way quickly [4].

### 3.7 Stereo linking and Image Link

The original EL8 link uses a "summing and phase detection method", which locks gain reduction but allows the stereo
image to shift; this is often desirable ("thickening" on room mics) but a problem when absolute L/R balance must be
kept [1, p. 13][3]. The EL8-X adds **Stereo Image Link**, which sums the gain-control signals between units and locks
the image, giving three link modes: the original phase link, the new Image Link, and the combination of the two [3].

Linking uses TRS 1/4" cables, link-out to link-in, chained in a loop for more than two units; there is "no limit ...
in theory" but long cables add noise [3][6]. Two hardware tricks fall out of the link circuit and are worth keeping
in the spoof:

- **Dead-patch distortion**: engaging Link on a single unit with nothing plugged in averages the input with a
  non-existent channel 2, halving the sidechain signal, which raises the threshold and increases harmonic distortion.
  UAD models this "dead patch" behaviour exactly [1][11][26].
- **Master/slave and mismatched ratios**: putting one unit in 1:1 with attack/release at 10 lets the other unit
  drive both timing circuits for a longer attack "a la SSL type compression"; mismatching the two units' ratios
  (e.g. 2:1 and Nuke) yields whole new combined curves, with the faster attack/decay setting generally winning [5].

### 3.8 What has no schematic, and what I therefore infer

There is no public schematic [42][43]. The VCA part number, the exact detector topology, the ratio-switching network
and the distortion-generator circuit are all inferred. My model treats the box as: an exponential dB-domain VCA in a
feedback loop; a diode/peak envelope detector with ratio-switched, program-dependent time constants; a per-ratio
table of (threshold offset, knee width, target slope, release shape); and a separate post-VCA waveshaper for the two
distortion colours. Every constant below is an **estimate** unless a source is cited.

---

## 4. Measured and published behaviour

### 4.1 Specifications

From the Empirical Labs manual and product page [1][3] unless noted:

| Quantity | Value | Notes / disagreements |
|---|---|---|
| Frequency response | 2 Hz to 160 kHz, +0/-3 dB, clean mode | Shaped in Dist 2/Dist 3. The 1997 Sound On Sound review quotes 5 Hz to 160 kHz [18]. |
| Dynamic range | 110 dB (max to min output, 1:1) | > 100 dB S/N in Dist 3 mode. [3][18] |
| Distortion | 0.02% to 20% | Depends on distortion mode and release. Per mode: clean 0.025-0.3%, Dist 2 0.05-3%, Dist 3 0.1-20% [1, p. 5]. SOS quotes 0.01% to 20% [18]. |
| Attack | 50 µs to 30 ms | Manual body and UAD say 30 ms [1][11]; the empiricallabs.com spec bullet says **50 ms** [3]. I treat 30 ms as the working figure and note the 50 ms discrepancy. |
| Release | 0.05 s to 3.5 s; up to 20 s in 10:1 Opto | Program dependent; time constants depend on ratio. [1][3][11] |
| I/O | DC-coupled in and out; transformerless (except EL8-XXX); XLR balanced + 1/4" TRS; pin 2 hot, user-changeable to pin 3 | EL8-XXX adds a Triad HS-56 input transformer, 600 ohm input. [1][3][29] |
| Power | 10 W typical, 14 W max (manual) / 15 W max (site) | 115/230 V switchable, 0.5 A (1/5 A) fuse. [1][3] |
| Size / weight | 1U (1.75" H x 10" D x 19" W), 12.1 lb mono | [1][3] |
| Memory | Gold cap holds settings ~4 weeks; older units 2 batteries; newer units 6 s write delay | [1][6][43] |

### 4.2 Ratios, knees and the "ratios run higher than the labels" problem

The single most important measured fact, and it comes from the designer: **the ratios measure higher than their
labels**. Derr: "Over a decade ago I realized that the ratios of the Distressor were generally quite a bit higher at
typical settings than their name suggests. That wasn't a big deal in itself, as the ratio curves change with attack
and decay settings and there is some 'play' in determining a ratio to begin with, especially ones with knees" [17].
When he built the Arousor he shifted every ratio name up one slot to bring the labels closer to the measurement, so
that **a Distressor ratio equals the next-higher Arousor ratio** (2:1 Distressor = 3:1 Arousor; 6:1 Distressor = 8:1
Arousor), except 20:1 and Nuke which stayed the same [7][17][22]. So when I calibrate the model I calibrate the
*effective slope I measure between 6 and 16 dB above threshold*, not the nominal ratio number.

Reading the published descriptions into approximate constants (**derived / estimate**, to be tuned against 8.1):

| Ratio | Threshold behaviour | Knee | Nominal vs likely measured slope | Detector |
|---|---|---|---|---|
| 1:1 | highest | n/a | 1:1 (no compression); ~10:1 if British mod installed, switch off [38] | standard; warming/distortion only |
| 2:1 | high | parabolic, "up to 30 dB", nominal +15 dB | ~2:1 rising through the knee; measures a little higher | special detector circuitry [1] |
| 3:1 | high | parabolic, gentle | ~3:1, measures higher | standard |
| 4:1 | mid | steeper | ~4:1, measures higher | standard |
| 6:1 | mid | steeper, short-knee limiting | ~6:1, measures higher | standard; general-purpose start point |
| 10:1 (Opto) | mid | short knee | opto curve, release to 20 s | special "opto" detector [1][3] |
| 20:1 | low-mid | dominant/hard | brick-wall, within ~1 dB | special detector [1] |
| Nuke | medium | dominant/hard | brick-wall; logarithmic release | special detector [1] |

The knee width for the gentle ratios is genuinely enormous: a 30 dB-wide soft knee on 2:1 is far wider than the
Giannoulis/Massberg/Reiss tutorial's usual few-dB knee [50], and it is a defining part of the Distressor's
"invisible first 6 dB" feel [15].

### 4.3 Attack and release, and the knob taper

No independent oscilloscope measurement against a stated criterion was found; what exists is the manufacturer range,
the interaction rules, and user reports:

- Range 50 µs to 30 ms attack, 0.05 s to 3.5 s release, 0 = fastest, and the knob goes slightly past 10 (to ~10.5)
  [1][2][33]. The 50 µs figure is 0.05 ms, so the whole attack knob spans 0.05 ms to 30 ms [2].
- The taper is undocumented and users could not pin exact numbers to knob positions; the recurring answer from the
  community and implied by Empirical Labs is "use your ears" [1][2][33]. The knob past 10 is where one engineer lives
  for vocal attack [2].
- Time constants interact with the ratio and are program dependent (3.3). At Attack 0 with 2:1/3:1/4:1 the attack can
  beat 50 µs [11].
- Opto (10:1, Attack 10, Release 0, Det HP on) emulates the LA-2A: slow-ish apparent attack, fast first release stage
  then a long tail to 20 s [1][3][35]. Community consensus is that the Opto release "lets off kinda quick at first,
  then starts to release much slower" and is "nothing like an 1176" [4][35].
- Nuke release is logarithmic (fast then slow) [1].

Because the taper is unpublished I use geometric (log) mappings in the model (7.2), same reasoning as for the 1176.

### 4.4 Distortion behaviour

- THD is set by mode, level, gain reduction and attack/release, not by a separate depth control [18]. The amber
  (1% THD, effectively 0.25% on the hardware) and REDLINE (3%) LEDs guide the user [1][11]. On individual instruments
  "3% distortion sounds 'fat' and 'analog' and isn't heard as distortion at all", while on full mixes it becomes
  obvious sooner [1, p. 5].
- Dist 2 = predominantly 2nd harmonic at "relatively small amounts (around 3%)", an overdriven-Class-A-valve
  analogue [18]. Dist 3 = 3rd harmonic (plus 2nd), tape/Class-B-like, flattening both ends of the waveform [1][18].
- Even in 1:1 with no distortion mode, a modded (British) unit generates harmonics and can even show GR because "loud
  harmonic distortion can sound like compression" [38]; owners routinely used 1:1 as a pure distortion generator
  [8][38].

### 4.5 Nuke and 20:1 (brick-wall)

Nuke has a medium threshold but "a nuclear blast won't budge the output level"; it and 20:1 keep any normal signal
within about 1 dB [1]. Nuke was developed for drum room mics (the "John Bonham thing"), 10-25 ft from the kit, slam
the meters, 15-20 dB of GR "starting to sound about right", output runs a little lower than other ratios, and 20 dB
of GR raises the noise floor by 20 dB so quiet preamps matter [1]. The logarithmic release is the signature: it can
release very fast without crackling, even on bass [1].

### 4.6 Unit-to-unit consistency (the opposite of the LA-2A)

Unlike the vintage boxes it emulates, the Distressor is deliberately uniform: "Precise factory calibration assures
that if you go from one Distressor to another, these settings will all sound the same" [1]. This is a design goal, so
the tribute does not need an "age" or "unit variation" control the way the LA-2A tribute does (see [[LA-2A]] 4.7).
Derr even abandoned an in-house opto compressor because "most opto compressors ... could not be made to match within
1-3 dB, so we gave up on it. We like things extremely consistent" [22].

---

## 5. Sound character, and what makes emulations right or wrong

Descriptions from users and reviewers:

- "It soon becomes obvious that this compressor wants to be heard ... at its best adding smack to kick drums,
  thickness and attack to bass guitars, and solidity to vocals" [18]. Reminiscent of the 1176LN and LA-2A "but with a
  unique clarity and edge" [20].
- The Opto mode is "very convincing" but "still sounds fairly aggressive ... not creamy" next to a real LA-2A; owners
  who compared them side by side agree the Distressor is "really really good, but not creamy" and would still reach
  for an LA-2A, ELOP or CL1B for a silky vocal [3 (opto thread)][35]. British Mode "injects a welcome aggression" and
  is a favourite on electric guitars [20].
- In a hardware-versus-plug-in shootout the hardware "grabbed the signal better esp in the low end" and had a
  softer, more pleasing compression than a competing plug-in, which had more "bite" [41].

What the sources say an emulation must get right:

1. **Feedback VCA with emergent, higher-than-labelled ratios and soft knees.** There is no threshold or ratio knob;
   the curve comes from loop gain, a fixed internal threshold, the per-ratio offsets and the huge soft knees. Reverb's
   summary: a good Distressor plug-in must capture "the way the unit's feedback design changes the attack and release
   curves depending on program material, the way the ratio buttons interact with the input drive, and the subtle
   low-frequency distortion that creeps in as you push the gain reduction meter past 6 or 7 dB" [17][14 (sound-freqs)].
2. **Eight genuinely different curves**, including the special-detector 2:1 (30 dB knee), 10:1 Opto (two-stage,
   20 s release) and Nuke/20:1 (brick-wall, logarithmic release) [1][17].
3. **The two distortion colours as a separate, level- and GR-dependent generator**: Dist 2 mostly 2nd to ~3%, Dist 3
   3rd (plus 2nd) up to ~20%, driven harder by slow attack / fast release, with the 1% and Redline meters [1][18].
4. **The detector filters** (HP to stop LF pumping, 6 kHz band emphasis to catch harshness) acting only on the
   sidechain [1][11][22].
5. **British Mode** as a sped-up, raised-threshold, extra-distortion 1176-all-buttons state, not a fifth ratio [4][17].
6. **The ultra-fast attack.** UAD's own marketing says to "contrast the original hardware's ultra-fast attack time to
   quickly hear where most plug-ins fail" [11]; the Reverb shootout singled out UAD's accurate attack as the thing
   other emulations missed [17].

It is judged "wrong" when it is a generic FET/VCA compressor with the Distressor's knob labels, when the ratios are
literal instead of emergent, when the distortion is amp-like instead of the gentle "vintage bite", or when the eight
ratios all feel the same.

---

## 6. How the Distressor and VCA compressors are simulated

### 6.1 Digital compressor design literature

- **Giannoulis, Massberg and Reiss, JAES 2012** [50]: the standard tutorial. The soft-knee gain computer with
  threshold `T`, ratio `R` and knee width `W` (restated in [[1176]] section 6.1) is the natural way to build the
  eight per-ratio curves, with `W` set very wide for 2:1/3:1 and narrow for 20:1/Nuke. The paper's branching and
  smooth-decoupled peak detectors give the envelope. It recommends feed-forward for predictability; I use a feedback
  loop here to get the emergent ratios, and fit a feed-forward curve to it for the tests.
- **Floru, THAT AES preprints 4054 (1995) and 4703 (1998)** [48][49]: the maths of RMS-based compressor time
  constants, including the feedback case. Key results I use: the feedback loop halves the effective detector
  integration time constant versus feed-forward; the log-domain release rate of ~120 dB/s "provides the best sound
  without introducing noticeable distortion"; the RMS/level detector control constant is 5.96 mV/dB at room
  temperature, in the 6.1-6.5 mV/dB range, matching the THAT VCA's -6.1 mV/dB so detector and VCA track [48][49][45].
- **THAT Design Notes 00A and 107/111** [46][47]: complete worked compressor/limiter circuits using a THAT VCA and a
  2252 RMS detector, including a soft-knee variant where an open-loop diode replaces the hard threshold detector
  (giving exactly the "first few dB at low ratio, gradually increasing" knee the Distressor manual describes), and
  `RATIO = 1/(1-R)` from a compression pot. These are the closest public analog analogue of the Distressor's block
  structure and give real component-scale numbers for the detector timing and the 6 mV/dB scaling.
- **Zölzer, DAFX 2nd edition, dynamics chapter** [51]: the reference implementation of limiter, compressor, expander,
  noise gate and de-esser with attack/release smoothing; the source for the standard envelope and gain-computer
  code I build the eight curves on.
- **Le Brun, "Digital Waveshaping Synthesis", JAES 1979** [52]: Chebyshev-polynomial waveshaping, where the kth
  Chebyshev polynomial turns a sinusoid into its kth harmonic, so a weighted sum of polynomials gives a shaper with a
  chosen harmonic balance. This is the cleanest way to voice Dist 2 (weight T2 for the 2nd harmonic) and Dist 3
  (weight T3 plus some T2), and to hit the manual's target THD percentages by scaling drive.

### 6.2 Commercial emulations and what they say they model

| Product | What the vendor says is modelled | Notes |
|---|---|---|
| Universal Audio UAD EL8 Distressor [11][12][17] | "A full component model of the Distressor circuit in all eight ratios ... plus the multiple distortion modes and filters in the detector sidechain, and those in the audio path"; three units analysed for consistency; the ultra-fast attack; the mono dead-patch link behaviour. | Officially endorsed and co-developed with Empirical Labs. Adds Headroom and Mix. Notably **omits British Mode** (an EL8-X-only feature). Detector HP 100 Hz 6 dB/oct, Band Emphasis ~6 kHz, Audio HP 80 Hz 18 dB/oct Bessel. [11][17] |
| Empirical Labs Arousor [7][8][24] | Not an emulation but "an evolution": ratio names shifted up one slot toward the measurement, added 1.5:1/2:1 and Rivet; AtMod (adjustable attack-curve modification); a Soft Clipper merging both distortion modes with continuous control; two-band detector sidechain EQ; two Opto modes modelling an early-80s and a 2018 T4 optocoupler. Distortion 0.004%-30%, attack 50 µs-40 ms, release 50 ms-3 s. | The designer's own reference for what is adjustable behind the Distressor's fixed behaviour. Zero-latency, one sample in/out. [8] |
| Slate Digital FG-Stress [31][32] | "Licensed digital replication of the iconic Empirical Labs Distressor"; every ratio modelled ("every ratio is essentially a different compressor"); Nuke as a limiting/clipping curve; detector HP 86 Hz 6 dB/oct, Bell +6 kHz, Link; Audio HP 80 Hz 18 dB/oct; five audio states (HP, HP+Dist2, Dist2, Dist3, HP+Dist3); adds a Mix knob. Ratios 0-10.5 default 6:1; input/output 0-10.5. | Reviewers found good top-end sizzle and lows but a bit less depth and more low-end rattle in Nuke than the hardware. [17][31] |
| SKnote Disto / Disto-S [17] | Distressor-style, includes a "UK Mode" (British). | "Digital sizzle" when pushed; British hard to nail. |
| Sly-Fi Deflector (Gregory Scott / UBK) [17] | "My twisted take on it", not a strict emulation; warmer, rounder, extra distortion. | Ratios unlike the hardware. |
| Cocell SOR8 [17] | Cheap Distressor-style GUI, quick heavy distortion. | Very digital distortion. |

**Diode-bridge context (a deliberate contrast).** The task brief mentioned the Arturia Comp DIODE-609 for
"diode-bridge context". That plug-in models the **Neve 33609 / 2254 diode-bridge** compressor, a completely different
gain element from the Distressor's VCA: in a diode bridge the audio itself is attenuated by a ring of diodes whose
resistance the control current sets, and the distinctive sound is the diode nonlinearity in the signal path [53]. The
Distressor is a VCA design, so I do **not** model a diode bridge in the signal path; I note it only to be clear about
what the Distressor is not. The one place a diode appears in the Distressor is the sidechain envelope detector, not
the audio path.

### 6.3 Oversampling

The Distressor's deliberate distortion generator, the VCA's own THD and the gain multiplication at fast attack all
create content above Nyquist, exactly as for the 1176 (see [[1176]] section 6.3). The distortion generator is the
strongest aliasing source here because Dist 3 can reach 20% THD. I run the distortion generator and the gain
multiply at 2x oversampling below 88.2 kHz, 1x at and above it.

---

## 7. Recommended DSP design for noob-vst-webgui-framework (44.1 to 96 kHz, real time)

The design is a **feedback VCA compressor in the log (dB) domain** with a per-ratio curve table, a program-dependent
diode-style envelope detector, a separate Chebyshev-voiced distortion generator, and the sidechain/audio filters. It
reuses the envelope and metering blocks of the 1176 and LA-2A models behind the shared `model` switch; only the
per-ratio table, the distortion generator and the filters are Distressor-specific. Everything runs per sample; the
distortion generator and gain multiply are 2x oversampled below 88.2 kHz. All constants are **estimates** unless a
source is cited, to be tuned against section 8.

### 7.1 Block diagram in words (per channel)

Main path, at the processing rate:

1. **Input trim**: `x1 = x * 10^(A_in/20)`, `A_in` from the Input knob taper (7.2). Because the internal threshold is
   fixed, this is what drives the amount of compression.
2. **Dif-amp / input conditioning**: first-order high-pass ~2 Hz (DC-coupled, essentially flat) [1]; optional Triad
   HS-56 transformer voicing (a gentle low-frequency saturation and a small high-frequency lift) as an "XXX" flavour
   option [29]. Estimate.
3. **VCA**: `x2 = g_inst[n] * x1`, where `g_inst = 10^(G_dB[n]/20)` and `G_dB` (<= 0) is the gain reduction from the
   detector/curve (7.3-7.4). The VCA also contributes a small level-dependent THD (7.5).
4. **Distortion generator** (7.6): the switchable Dist 2 / Dist 3 waveshaper, `y = shape(x2, mode, drive)`.
5. **Audio high-pass** (7.7): 80 Hz, 18 dB/oct Bessel, switchable.
6. **Output amp + output trim**: `x3 = y * 10^(A_out/20)`, with a gentle output-amp saturation. Estimate.
7. Output.

Sidechain (feedback), tapped from the VCA output `x2` (before the distortion generator, matching the block diagram
where the detector is fed from the gain-changed audio) [1, p. 6]:

1. **Band emphasis** (switchable): +6 kHz bell into the sidechain (7.7).
2. **Sidechain HP** (switchable): high-pass into the sidechain (7.7).
3. **Level detector**: diode/peak envelope in the dB domain (7.3).
4. **Curve/gain computer**: per-ratio soft-knee curve giving target `G_dB` (7.4).
5. **Ballistics**: ratio-switched, program-dependent attack/release smoothing of `G_dB` (7.3).
6. **Link summing** for stereo (7.9); meter feed for the bargraph (7.8).

### 7.2 Control mappings

- **Input / Output** knobs, 0 to 10.5. The panel scale is arbitrary; map to dB with a piecewise-linear table.
  Estimate: Input 0 -> -inf (or a large attenuation), 5 -> 0 dB, 10.5 -> about +25 dB of drive; Output symmetric.
  Calibrate so that at 6:1 with all knobs at 5 and a -18 dBFS program, 1 LED lights and GR is a few dB [1][11].
- **Attack** knob `a` in [0, 10.5], geometric map (taper undocumented, so log like the 1176):
  `tau_att(a) = 50us * (30ms/50us)^(a/10)` giving 50 µs at 0 and 30 ms at 10 (about 55 ms at 10.5). At `a=0` with
  ratios 2:1/3:1/4:1 allow the effective attack to drop below 50 µs [11]. **Estimate.**
- **Release** knob `r` in [0, 10.5], geometric: `tau_rel(r) = 50ms * (3.5s/50ms)^(r/10)` giving 50 ms at 0 and 3.5 s
  at 10. In 10:1 Opto the release target is overridden by a two-stage curve up to 20 s (7.4). **Estimate.**
- **Ratio**: selects a row of the per-ratio table (7.4). **Detector** and **Audio**: cycle the filter/distortion
  states (7.6, 7.7). **British Mode**, **Stereo Image Link**: toggles (7.4, 7.9).
- **Headroom** and **Mix**: model the UAD-only controls (Headroom scales the internal reference level 4-28 dB,
  default 16; Mix is a phase-accurate wet/dry blend), because they are genuinely useful and the tribute is a plug-in
  [11]. Off by default in a "hardware-faithful" sub-mode.

### 7.3 Detector and program-dependent ballistics (feedback, dB domain)

Tap `s = x2` (post-VCA). Rectify and convert to dB: `x_dB = 20*log10(max(|s_filtered|, eps))`, where `s_filtered` is
`s` after the optional band emphasis and sidechain HP. Compute the target gain `G_tgt` from the curve (7.4), then
smooth with a branching detector whose coefficients depend on the ratio and on the overshoot (program dependence):

```
a_att = 1 - exp(-1/(tau_att_eff * fs_proc))
a_rel = 1 - exp(-1/(tau_rel_eff * fs_proc))
if G_tgt < G[n-1]:  G[n] = G[n-1] + a_att*(G_tgt - G[n-1])   # attacking (more GR)
else:               G[n] = G[n-1] + a_rel*(G_tgt - G[n-1])   # releasing
```

Program dependence (the thing every source stresses): make `tau_att_eff` and `tau_rel_eff` depend on both the ratio
and the overshoot. **Derived / estimate:**

- `tau_att_eff = tau_att(a) * k_att_ratio(ratio) * f(overshoot)`, where larger overshoot shortens the attack (a big
  transient is caught faster), reproducing "fast attack at 2:1 is not the same as fast attack at 4:1" [33].
- Feedback halves the effective time constant versus a feed-forward reading [49]; fold this into `k_att_ratio`.
- For **Nuke**, override the release with a **logarithmic** two-segment curve: a fast first segment then a slow tail,
  so it "lets off quickly at first and then slows" [1]. Implement as two release poles blended by how far the
  envelope has recovered.
- For **10:1 Opto**, override with a **two-stage, memory-dependent** release borrowed from the LA-2A model (see
  [[LA-2A]] 7.2): a fast pole (~60 ms) to roughly half recovery, then a slow trap-like pole that stretches to 20 s
  after long/deep compression [1][3]. The Opto attack is set slow (knob 10) and the fast light-cell attack is not
  exposed.

### 7.4 Static curve per ratio (the eight curves)

Use the standard soft-knee gain computer [50] in dB, with per-ratio threshold offset `T_r`, knee width `W_r` and
target slope giving ratio `R_r`, and calibrate `R_r` to the **measured** slope (higher than the label, 4.2):

```
x = x_dB                                  # sidechain level in dB
if 2*(x - T_r) < -W_r:      G_tgt = 0
elif 2*|x - T_r| <= W_r:    G_tgt = (1/R_r - 1) * (x - T_r + W_r/2)^2 / (2*W_r)
else:                       G_tgt = (T_r + (x - T_r)/R_r) - x     # <= 0
```

Per-ratio constant table (**derived / estimate**; `T_r` in dB relative to the internal reference, `W_r` in dB, `R_r`
the effective slope, tune against 8.1):

| Ratio | `T_r` (dB) | `W_r` (dB) | `R_r` (effective) | Release shape | Detector variant |
|---|---|---|---|---|---|
| 1:1 | +inf | - | 1 | - | none (distortion only); British: `T` low, `R_r`~10, sped-up (7.4a) |
| 2:1 | -6 | 30 | ~2.3 | standard | "special" wide parabolic knee [1] |
| 3:1 | -8 | 24 | ~3.3 | standard | standard |
| 4:1 | -12 | 12 | ~4.5 | standard | standard |
| 6:1 | -14 | 10 | ~6.5 | standard | standard; default |
| 10:1 (Opto) | -16 | 8 | ~10 | two-stage, to 20 s | opto (7.3) |
| 20:1 | -18 | 3 | ~20 | fast | "special" hard knee [1] |
| Nuke | -16 | 1.5 | ~40 (brick-wall) | logarithmic (7.3) | "special" hard knee [1] |

The 30 dB knee on 2:1 is the widest and is what makes the first several dB "invisible" [15]; the 20:1/Nuke knees are
narrow and dominant [1]. Absolute placement of `T_r` and the reference level is an **estimate** to be calibrated so
that -18 dBFS program at Input 5 gives gentle compression at 6:1 (a few dB) and heavy compression as Input rises,
matching the "5 5 5 5" advice [1][11].

#### 7.4a British Mode

Following the 1176 all-buttons treatment (see [[1176]] section 7.6) and the Empirical Labs description [4][17]:

1. On the 1:1 slot, replace the 1:1 curve with a British curve of `R_r` ~10-20 and a raised threshold, "sped up"
   (shorter effective time constants) [38][17].
2. The dedicated British toggle applies the same offset/speed-up and extra loop gain to whatever ratio is selected.
3. Add a bias offset the envelope must charge through before GR starts (the "reverse look-ahead" attack lag), and
   make the effective attack fast and program-dependent.
4. Increase the distortion-generator drive with GR, emphasised at low frequencies, and require Attack under ~3-4 to
   keep the "1176 character"; above that, let grunge and the THD LEDs rise, exactly as the manual warns [4].
5. Modelled as a state, not a fifth ratio. All numbers **estimate**, tune by ear and against 8.3.

### 7.5 VCA nonlinearity

Give the VCA a small level-dependent THD on top of the deliberate generator, from the THAT 2181 THD-versus-level
curves [45]: `x2 *= 1 - kappa_vca * (x2/X0)^2` with `kappa_vca` tuned for well under 0.05% THD at nominal level and
rising toward the clip point. **Estimate.** Keep it subtle; the audible distortion is the generator (7.6), not this.

### 7.6 Distortion generator (Dist 2 / Dist 3)

A separate post-VCA waveshaper, voiced with Chebyshev polynomials [52] so the harmonic balance and THD match the
manual:

```
u = x2 / X_d                                   # drive; X_d scales with Headroom and GR
Dist2:  y = u + a2*T2(u)                        # T2(u)=2u^2-1 -> 2nd harmonic
Dist3:  y = u + a3*T3(u) + a2b*T2(u)            # T3(u)=4u^3-3u -> 3rd (plus some 2nd)
y *= X_d
```

Target THD from the manual/measurements [1][18]: Dist 2 predominantly 2nd harmonic, 0.05% to ~3%; Dist 3 3rd harmonic
(with 2nd), 0.1% up to 20%. Drive `X_d` rises when the attack is slow, the release is fast, GR is present, or Link is
engaged (dead-patch), and is scaled by Headroom [1][3][11]. Light the **1% THD** LED at ~1% (model) or 0.25%
(hardware-faithful sub-mode) and the **REDLINE** LED at ~3% [1][11]. `a2 ~ 0.03` gives ~3% 2nd; `a3` tuned for up to
20% at full drive. **Estimates.**

### 7.7 Filters

- **Audio HP**: 80 Hz, 18 dB/oct Bessel, ~3 dB down at 65 Hz, ~12 dB at 30 Hz [1][11]. A 3rd-order Bessel high-pass.
- **Detector HP**: 100 Hz, 6 dB/oct (first-order) into the sidechain [11]; Slate uses 86 Hz [32]. Stops LF pumping.
- **Band Emphasis**: peaking boost around 6 kHz into the sidechain, makes the detector overreact to harsh mids /
  sibilance [1][11]; Mixdown describes the audible effect as compressing "from around 3 kHz" up [22]. Estimate the
  gain at +6 to +10 dB, Q ~1.
- The audio path is never EQ'd by the detector filters; they only shape what the compressor reacts to [1].

### 7.8 Metering

- **16-LED gain-reduction bargraph**, silkscreen 1..26 dB, colours green (~1-5/6), yellow (~6/7-10), red (12-26)
  [16][faceplate]. Drive it from `G_dB` with fast attack and a slightly slower release so it "deflects much faster
  than the old VU's" [1]; that is why the manual says don't be afraid to hit it hard (10-20 dB on peaks in Opto) [1].
- **1% THD** (yellow) and **REDLINE** (red) LEDs from the distortion-generator THD estimate (7.6).
- Send the smoothed values to the web UI at 30-60 Hz; keep the ballistics in the audio thread.

### 7.9 Stereo and the three link modes

- **Unlinked**: two independent detectors.
- **Phase link** (original EL8): sum-and-phase detection; sum the two rectified sidechains before the detector so GR
  is common but the image can shift [1][3].
- **Image Link** (EL8-X): sum the two gain-control signals (the computed `G_dB`) so both channels get identical GR and
  the image is locked [1, p. 13][3].
- **Both**: the combination.
- **Dead patch**: Link on with a single (mono) channel averages the input with a silent channel 2, halving the
  sidechain (raising threshold) and increasing distortion-generator drive [1][11][26]. Model this explicitly; it is a
  known trick.
- **Master/slave and mismatched ratios** (7.3): a stereo option where one channel drives both timing circuits, and
  where mismatched per-channel ratios blend two curves with the faster time constant winning [5].

### 7.10 Defaults and parameter list

| Parameter | Range | Default | Source of default |
|---|---|---|---|
| Input | 0 to 10.5 (dB via taper) | 5 | [1][11] |
| Output | 0 to 10.5 | 5 | [1] |
| Attack | 0 to 10.5 (50 µs to ~55 ms) | 5 | [1] |
| Release | 0 to 10.5 (50 ms to 3.5 s) | 5 | [1] |
| Ratio | 1:1, 2:1, 3:1, 4:1, 6:1, 10:1, 20:1, Nuke | 6:1 | [1][11] |
| Detector | Norm, HP, Band, Link, + combos (8) | Norm | [1][11] |
| Audio | Norm, HP, Dist 2, Dist 3, + HP combos (6) | Norm | [1][11] |
| British Mode | on/off | off | [1, p. 12] |
| Stereo Image Link | phase / image / both / off | off (mono), phase (stereo) | [1, p. 13] |
| Headroom (plug-in) | 4-28 dB | 16 | [11] |
| Mix (plug-in) | Dry-Comp | Comp | [11] |
| Oversampling | auto | 2x below 88.2 kHz | 6.3 |

Smooth Input/Output gains over a few ms; switch ratio/mode instantly but crossfade `T_r`, `W_r`, `R_r` over a few ms
to avoid clicks; warn (or auto-duck) on the 1:1 swell the manual flags [1][11].

---

## 8. Test plan

Unit and integration tests for the Rust core, run offline at 44.1, 48 and 96 kHz. Tolerances are proposals; tighten
after tuning. Where hardware evidence exists the expected value is cited.

### 8.1 Static gain reduction per ratio

For each ratio, sweep a 1 kHz sine from -60 to 0 dBFS in 1 dB steps, hold 2 s, measure steady-state output over the
last 200 ms:

- below threshold, output tracks input within 0.1 dB (plus fixed gain);
- threshold (input for 1 dB GR) increases monotonically from 2:1 through 20:1 [1];
- the measured slope 6-16 dB above threshold matches the effective `R_r` in the 7.4 table within 20%, and is
  **higher than the nominal label** for 2:1-10:1 (Derr's finding) [17];
- 20:1 and Nuke rise less than 1 dB over that range (brick-wall, within ~1 dB) [1];
- soft knee: 2:1 shows measurable GR at least 15-30 dB below its final-ratio threshold (the +15 to 30 dB knee) [1];
- 1:1 shows zero GR clean, but non-zero GR once a distortion mode drives it hard [38].

### 8.2 Attack and release timing

Tone burst -40 to -6 dBFS at each ratio, hold 1 s, back down; divide output envelope by input envelope for the gain
trajectory:

- attack (time to 63% of final GR) monotonic in the knob and within a factor of 1.5 of the mapped `tau_att`; a bigger
  step attacks faster than a smaller one (program dependence) [33];
- fast attack at 2:1 differs measurably from fast attack at 4:1 [33];
- release (63% recovery) within 25% of the mapped `tau_rel` in normal ratios;
- **Nuke** release is logarithmic: recovers faster in its first half than its second [1];
- **10:1 Opto** release is two-stage: ~50% recovery in tens of ms then a tail that, after a long deep burst, exceeds
  several seconds and can approach 20 s [1][3].

### 8.3 British Mode

- threshold higher and effective ratio 10-20 with the toggle on 1:1 [4][38];
- time to first 1 dB of GR longer than 20:1 for the same burst (attack lag) [17];
- with Attack under 4, THD stays modest; above 4, THD (and the 1%/Redline meters) rise sharply [4];
- a modded unit at 1:1 with the toggle off still compresses (~10:1) [38].

### 8.4 Distortion

- clean mode, 1 kHz at -18 dBFS: THD 0.025% to 0.3% [1];
- Dist 2, 1 kHz, a few dB GR: THD around 0.05-3%, **2nd harmonic dominant** [1][18];
- Dist 3, driven hard: THD up to ~20%, **3rd harmonic dominant** (with some 2nd) [1];
- slow attack raises THD versus fast attack at the same GR (peaks hit the generator harder) [3];
- Link dead-patch on a mono channel raises THD versus Link off [11];
- Audio HP: -3 dB at ~65 Hz, -12 dB at ~30 Hz, 18 dB/oct [1];
- aliasing: 15 kHz at -6 dBFS, Dist 3, fastest attack at 44.1 kHz: no aliased component above -70 dBFS below 10 kHz;
  compare with 96 kHz.

### 8.5 Detector filters

- Detector HP on: a 40 Hz tone that pumps the compressor with HP off produces markedly less GR with HP on (100 Hz
  6 dB/oct) [1][11];
- Band Emphasis on: a 6 kHz tone triggers more GR than an equal-level 1 kHz tone [1][11];
- the audio spectrum is unchanged by either detector filter (sidechain only) [1].

### 8.6 Numerical robustness

- 10 s burst then 30 s silence: no NaN/inf, envelope reaches zero, no denormals (FTZ);
- DC, full-scale square waves at 20 Hz and 20 kHz, impulses at every knob extreme: output finite and bounded;
- alternate ratio buttons every 100 samples: no discontinuity larger than the implied threshold change (crossfade
  test); the 1:1 swell is bounded or auto-ducked [1].

### 8.7 Metering

- GR bargraph reads the settled GR within 0.5 dB and lights the right colour band per 2.1;
- 1% LED lights at ~1% THD (model) / 0.25% (hardware sub-mode); REDLINE at ~3% [1][11];
- meter stays live in Bypass, dark when Power is off [11].

### 8.8 Stereo / link

- Unlinked: right channel shows no GR when only left is driven;
- Phase link: common GR, image may shift; Image Link: identical GR both channels, image locked [1][3];
- Dead patch (mono, Link on): higher THD and higher effective threshold than Link off [11].

### 8.9 Sample-rate invariance and performance

- Run 8.1, 8.2, 8.4 at 44.1/48/88.2/96 kHz: thresholds within 0.3 dB, time constants within 5%, THD within a factor
  of 1.5 (oversampling-filter differences);
- one stereo instance at 2x/96 kHz under a small fixed CPU budget (**estimate**: under 2% of one 2020-class core),
  no audio-thread allocations.

---

## 9. References

1. Empirical Labs, "Distressor EL8-X Users Manual" (Features & Specs, ratios & curves, audio/detector modes, block
   diagram, British Mod, Stereo Image Link, warranty/dedication). https://www.empiricallabs.com/wp-content/uploads/distressor_manual.pdf
2. Empirical Labs, older "Distressor Manual" (barryrudolph.com mirror; features, specs, ratios, curves, audio and
   detector modes). https://www.barryrudolph.com/recall/manuals/distressor1.pdf
3. Empirical Labs, "Distressor EL8x" product page (specifications, versions EL8/EL8-S/EL8-X/EL8X-S/EL8-XXX, distortion
   modes, opto, British Mode, Image Link, FAQs, testimonials). https://www.empiricallabs.com/distressor/
4. Empirical Labs, "Brit Mode Tips and Tricks". https://www.empiricallabs.com/brit-mode-tips-and-tricks/
5. Empirical Labs, "Stereo Image Link Tips and Tricks". https://www.empiricallabs.com/stereo-image-link-tips-and-tricks/
6. Empirical Labs, "FAQs" (memory cap, linking, 1:1, soft knee, DocDerr ~5:1 knee comparison). https://www.empiricallabs.com/faqs/
7. Empirical Labs, "Arousor" product page (AtMod, Soft Clipper, detector sidechain EQ, Rivet, Opto modes, ratios).
   https://www.empiricallabs.com/product/arousor/
8. Empirical Labs, "Arousor Rev 2.0 Manual" (features/specs, block diagram, compressor/AtMod/soft-clip/blend/Det HP/
   Det parametric EQ sections; attack 50 µs-40 ms, release 50 ms-3 s, distortion 0.004%-30%). https://www.empiricallabs.com/wp-content/uploads/arousor_manual_rev2_v1.pdf
9. Empirical Labs, "What's New in Arousor Rev 3" (two Opto modes modelling an early-80s T4A and a 2018 T4B). https://www.empiricallabs.com/whats-new-in-arousor-rev-3/
10. Empirical Labs, "Manuals" index. https://www.empiricallabs.com/manuals/
11. Universal Audio, "Empirical Labs EL8 Distressor Manual" (controls, ratio/detector/audio tables, filters, headroom,
    mix, dead-patch link, Operation Notes from Dave Derr). https://help.uaudio.com/hc/en-us/articles/18741515014676-Empirical-Labs-EL8-Distressor-Manual
12. Universal Audio, "Empirical Labs EL8 Distressor Compressor" product page. https://www.uaudio.com/products/empirical-labs-el8-distressor-compressor
13. Universal Audio (hookup.co.jp mirror), "UAD Powered Plug-Ins Manual 151: Empirical Labs EL8 Distressor" (Japanese;
    controls, ratio/detector/audio tables, block diagram). https://hookup.co.jp/assets/upload/support/attachments/2023/12/4469/Empirical-Labs-EL8-Distressor_JP0803.pdf
14. Wikipedia, "Empirical Labs Distressor" (history, name, 10:1 opto, EL8-X British Mode/Image Link, TECnology Hall of
    Fame, notable users). https://en.wikipedia.org/wiki/Empirical_Labs_Distressor
15. SonicScoop, D. Weiss, "Inventor Insights: Dave Derr's Tips & Tricks for The Empirical Labs Distressor" (history,
    Gain Brain / 1176 / LA-2A influence, per-source tips, link modes). https://sonicscoop.com/inventor-insights-dave-derrs-tips-tricks-for-the-empirical-labs-distressor/
16. SonicScoop, R. Crescenti, "New Software Review: Universal Audio EL8 Distressor" (feedback topology, ratios, knee,
    detector 100 Hz/6 kHz, audio HP 80 Hz, 1993 date, GR LED colours). https://sonicscoop.com/new-software-review-universal-audio-el8-distressor
17. Reverb, S. Templeton, "The Empirical Labs Distressor and Its Emulations" (Derr interview: ratios run higher than
    labels, Brit Mode "non-linear beast", six-plug-in shootout). https://reverb.com/news/the-empirical-labs-distressor-and-its-emulations-hear-how-6-plugins-compare-to-the-original
18. Sound On Sound, P. White, "Empirical Labs Distressor" review, Dec 1997 (VCA soft-knee, ratios, Nuke blue LED,
    detector 80 Hz / 6 kHz, Dist 2 ~3% 2nd harmonic, dead-patch trick, specs). https://www.soundonsound.com/reviews/empirical-labs-distressor
19. Tape Op, W. Szalva, "Dave Derr: Behind The Gear with Empirical Labs", issue #33 (biography, Eventide/H3000,
    Garfield studio, three or four years tweaking, Fletcher/Massenburg launch). https://tapeop.com/interviews/33/dave-derr
20. Tape Op, F. Reid Shippen, "Empirical Labs EL-8X Distressor" review, issue #32, Nov/Dec 2002 (1:1 harmonic
    enhancement, 10:1 opto, Nuke brick-wall, British Mode, Image Link, link-without-second-unit trick). https://tapeop.com/reviews/gear/32/el-8x-distressor
21. Gearshoot, "Dave Derr - Empirical Labs - Interview" (design philosophy, ratios idealised on Arousor, distortion
    triode curves / odd vs even harmonics, AtMod, discarded opto compressor). https://gearshoot.com/interviews/dave-derr-empirical-labs-interview/
22. Mixdown, A. Lloyd-Russell, "Gear Icons: Empirical Labs Distressor" (history, ratios/knee, detector HP/band/link,
    audio HP 80 Hz Bessel, Dist 2/3, British Mode, settings). https://mixdownmag.com.au/features/gear-icons-empirical-labs-distressor/
23. Mixonline, "Review: Universal Audio Empirical Labs EL8 Distressor". https://www.mixonline.com/recording/review-universal-audio-empirical-labs-el8-distressor
24. Mixonline, "Product of the Week: Empirical Labs Arousor Rev 3" (two Opto modes, soft-clipper expert panel, 6.5:1
    and 7:1 ratios). https://www.mixonline.com/technology/product-of-the-week-empirical-labs-arousor-rev-3
25. Audiogearz, "Review: Empirical Labs EL8 Distressor Compressor/Limiter" (history, features overview). https://www.audiogearz.com/gear/outboard/el8-distressor-compressor-limiter/
26. Sound-Freqs, "Distressor Plugin: Emulations, Best Alternatives & Comparison" (feedback design, ratio interaction,
    LF distortion past 6-7 dB GR, UAD/FG-Stress/Arturia notes). https://sound-freqs.com/reviews/distressor-plugin-emulations-best-alternatives-and-comparison/
27. Vintage Technology Archive, "Empirical Labs Distressor - Specs & History" (production 1995-2023, ~40,000 units,
    versions, specs, MSRP). https://vintagetechnologyarchive.com/audio/empirical-labs/distressor/
28. Vintage Digital, "The Amazing Empirical Labs Distressor from 1996" (history, Gain Brain influence, EL8-X, Brit
    Mode, Image Link). https://www.vintagedigital.com.au/empirical-labs-distressor/
29. Production Expert, "Empirical Labs Marks 30 Years of the Distressor with Limited EL8-XXX Edition" (Triad HS-56
    transformer, discrete parts, red faceplate, NAMM 2026). https://www.production-expert.com/production-expert-1/empirical-labs-marks-30-years-of-the-distressor-with-limited-el8-xxx-edition
30. Vintage King, "Empirical Labs EL8-XXX Distressor 30th Anniversary Edition". https://vintageking.com/empirical-labs-el-8-xxx-distressor-30th-anniversary-edition
31. Slate Digital, "FG-Stress Distressor Plugin" product page (licensed replication, every ratio a different
    compressor, Nuke, Mix). https://slatedigital.com/fg-stress-distressor-plugin/
32. Slate Digital docs, "FG-Stress" (input/output/attack/release 0-10.5, Nuke, detector HP 86 Hz/Bell 6 kHz/Link,
    audio HP 80 Hz 18 dB/oct, five audio states, GR 16 LEDs, 1%/REDLINE at 0.25%/3%). https://docs.slatedigital.com/VMR/FG-Stress.html
33. Gearspace, "Scaling of Distressor Attack and Release knob" (time constants change with ratio; Dave Derr returned a
    user's call about the British 1:1 change). https://gearspace.com/board/high-end/17105-scaling-distressor-attack-release-knob.html
34. Gearspace, "Distressor Attack times" (0 fast/10 slow, 50 µs=0.05 ms to 30 ms, knob past 10, taper not documented).
    https://gearspace.com/board/so-much-gear-so-little-time/1050264-distressor-attack-times.html
35. Gearspace, "Distressor 'opto' mode question" (opto attack 10/release 0, two-stage release, LA-2A comparison,
    separate circuit). https://gearspace.com/threads/distressor-opto-mode-question.43014/
36. Gearspace, "Question about the Distressor's Opto setting and the 1176" (opto is LA-2A-like not 1176-like; release
    lets off fast then slow). https://gearspace.com/threads/question-about-the-distressors-opto-setting-and-the-1176.457447/
37. Gearspace, "This is Cool, Dave Derr tells the Story of the Distressor" (2001 Record Plant Remote video). https://gearspace.com/board/so-much-gear-so-little-time/963774-cool-dave-derr-tells-story-distressor.html
38. Gearspace, "Distressor 1:1 weirdness?" (Dave Derr explains British-modded 1:1 becomes ~10:1; jumper to restore;
    1:1 + BM on + Attack 10 for distortion without compression). https://gearspace.com/board/so-much-gear-so-little-time/6683-distressor-1-1-weirdness.html
39. Gearspace, "Distressor Guide" (usage, opto on vocals, dist 2/3, HP detector). https://gearspace.com/threads/distressor-guide.61591/
40. Gearspace, "Distressor Tricks" (settings: opto vocals, Nuke drum sub, 20:1 vocal peak catcher, British on drums).
    https://gearspace.com/threads/distressor-tricks.43532/
41. Gearspace, "Distressor (hardware) Versus Smack! (plugin)" shootout (hardware softer/more low-end, plugin more
    bite). https://gearspace.com/board/gear-shoot-outs-sound-file-comparisons-audio-tests/859363-distressor-hardware-versus-smack-plugin.html
42. GroupDIY, "Distressor Schematic" (no public schematic; reverse-engineered ones removed at Empirical Labs' request).
    https://groupdiy.com/threads/distressor-schematic.4026/
43. GroupDIY, "Distressor schematic please!" (control part non-trivial; supercap holds settings during power-down).
    https://groupdiy.com/threads/distressor-schematic-please.75814/
44. GroupDIY, "THAT 2181 VCA question" (2180 vs 2181 grades, trimming, control-voltage AC coupling). https://groupdiy.com/threads/that-2181-vca-question.87727/
45. THAT Corporation, "2181-Series Trimmable Blackmer VCA" datasheet (exponential dB control, -6.1 mV/dB, >130 dB gain
    range, >120 dB dynamic range, ~0.005%/0.0025% THD, 20 MHz bandwidth, symmetry trim). https://www.thatcorp.com/datashts/THAT_2181-Series_Datasheet.pdf
46. THAT Corporation, Design Note 00A, "Basic Compressor/Limiter Design" (218x VCA + 2252 RMS; above-threshold and
    soft-knee circuits; RATIO=1/(1-R); 6 mV/dB; ~35 ms integration). https://thatcorp.com/datashts/dn00A.pdf
47. THAT Corporation, Design Note 107/111, "A simple, effective soft-knee compressor/limiter" (2180C VCA + 2252 RMS;
    open-loop diode soft-knee threshold; 120 dB/s release; KGAIN 6.1 mV/dB). https://thatcorp.com/datashts/dn107.pdf
48. THAT Corporation, F. Floru, "Attack and Release Time Constants in RMS-Based Compressors and Limiters", AES
    preprint 4054, 99th Convention 1995 (log-domain detector, ripple, 5.96 mV/dB at 300 K, 6.1-6.5 mV/dB range). https://www.thatcorp.com/datashts/AES4054_Attack_and_Release_Time_Constants_II.pdf
49. THAT Corporation, F. Floru, "Attack and Release Time Constants in RMS-Based Feedback Compressors", AES preprint
    4703, 104th Convention 1998 (linear- and log-domain feedforward vs feedback transfer functions; feedback halves
    the detector integrator time constant). https://thatcorp.com/datashts/AES4703_Attack_and_Release_Time_Constants_I.pdf
50. D. Giannoulis, M. Massberg, J. D. Reiss, "Digital Dynamic Range Compressor Design: A Tutorial and Analysis", JAES
    60(6), 2012 (soft-knee gain computer, branching/decoupled peak detectors). https://www.eecs.qmul.ac.uk/~josh/documents/2012/GiannoulisMassbergReiss-dynamicrangecompression-JAES2012.pdf
51. U. Zölzer (ed.), "DAFX: Digital Audio Effects", 2nd ed., dynamics-processing chapter (limiter, compressor,
    expander, noise gate, de-esser). https://www.dafx.de/DAFX_Book_Page/chapter5.html
52. M. Le Brun, "Digital Waveshaping Synthesis", JAES 27(4), 1979 (Chebyshev-polynomial waveshaping for prescribed
    harmonic spectra). https://aes.org/e-lib/browse.cfm?elib=3212
53. Arturia, "Comp DIODE-609" (Neve 33609 diode-bridge compressor; diode-bridge topology and its distortion; contrast
    with the Distressor's VCA). https://www.arturia.com/products/software-effects/comp-diode-609/overview
54. Bobby Owsinski's Inner Circle Podcast, "Episode 623 - Creator of the Distressor Dave Derr" (analog compression,
    the birth of Nuke mode, trademarked knobs). https://bobbyowsinskiblog.com/creator-of-the-distressor-dave-derr-on-my-latest-podcast/
55. Recording Studio Rockstars, "RSR489 - Dave Derr - Empirical Labs" (Distressor, Arousor, Fatso, Pump design). https://recordingstudiorockstars.com/rsr489-dave-derr-empirical-labs-creator-of-the-distressor-arouser-big-freq-fatso-and-pump/
56. Leeds Conservatoire, "Using the Empirical Labs Distressor" user guide, 2024 (panel walkthrough, ratio/detector/
    audio LED states, British Mode 1:1, THD onset percentages). https://students.leedsconservatoire.ac.uk/wp-content/uploads/2025/08/Empirical-Labs-Distressor-User-Guide.pdf
57. UMLSRT (mirror), "Empirical Labs EL8X Distressor - Dynamics" manual scan. https://umlsrt.com/wp-content/uploads/Studio%20Documents/EmpericalLabs_DistressorEL8X_Dynamics.pdf
58. MusicRadar, "Universal Audio Empirical Labs EL8 Distressor" review (control layout, ratios, headroom, British Mode
    absent from plug-in). https://www.musicradar.com/reviews/universal-audio-empirical-labs-el8-distressor
59. Ask.Audio, "Review: Universal Audio Empirical Labs EL8 Distressor" (null test to ~60 dB against hardware, opto
    10:1 settings). https://ask.audio/articles/review-universal-audio-empirical-labs-el8-distressor
60. Nail The Mix, "The UAD Distressor Plugin: Your Metal Mix Swiss Army Knife" (ratios, Nuke, British, Dist 2/3,
    suggested settings). https://www.nailthemix.com/uad-distressor-plugin
61. THAT Corporation, "Design Notes" index. https://thatcorp.com/design-notes/
62. Empirical Labs, "Arousor Rev 3" page (Opto A/B, ALT ratios 6.5:1 and 7:1, listen mode). https://www.empiricallabs.com/arousor-rev-3/

[1]: https://www.empiricallabs.com/wp-content/uploads/distressor_manual.pdf
[2]: https://www.barryrudolph.com/recall/manuals/distressor1.pdf
[3]: https://www.empiricallabs.com/distressor/
[4]: https://www.empiricallabs.com/brit-mode-tips-and-tricks/
[5]: https://www.empiricallabs.com/stereo-image-link-tips-and-tricks/
[6]: https://www.empiricallabs.com/faqs/
[7]: https://www.empiricallabs.com/product/arousor/
[8]: https://www.empiricallabs.com/wp-content/uploads/arousor_manual_rev2_v1.pdf
[9]: https://www.empiricallabs.com/whats-new-in-arousor-rev-3/
[10]: https://www.empiricallabs.com/manuals/
[11]: https://help.uaudio.com/hc/en-us/articles/18741515014676-Empirical-Labs-EL8-Distressor-Manual
[12]: https://www.uaudio.com/products/empirical-labs-el8-distressor-compressor
[13]: https://hookup.co.jp/assets/upload/support/attachments/2023/12/4469/Empirical-Labs-EL8-Distressor_JP0803.pdf
[14]: https://en.wikipedia.org/wiki/Empirical_Labs_Distressor
[15]: https://sonicscoop.com/inventor-insights-dave-derrs-tips-tricks-for-the-empirical-labs-distressor/
[16]: https://sonicscoop.com/new-software-review-universal-audio-el8-distressor
[17]: https://reverb.com/news/the-empirical-labs-distressor-and-its-emulations-hear-how-6-plugins-compare-to-the-original
[18]: https://www.soundonsound.com/reviews/empirical-labs-distressor
[19]: https://tapeop.com/interviews/33/dave-derr
[20]: https://tapeop.com/reviews/gear/32/el-8x-distressor
[21]: https://gearshoot.com/interviews/dave-derr-empirical-labs-interview/
[22]: https://mixdownmag.com.au/features/gear-icons-empirical-labs-distressor/
[23]: https://www.mixonline.com/recording/review-universal-audio-empirical-labs-el8-distressor
[24]: https://www.mixonline.com/technology/product-of-the-week-empirical-labs-arousor-rev-3
[25]: https://www.audiogearz.com/gear/outboard/el8-distressor-compressor-limiter/
[26]: https://sound-freqs.com/reviews/distressor-plugin-emulations-best-alternatives-and-comparison/
[27]: https://vintagetechnologyarchive.com/audio/empirical-labs/distressor/
[28]: https://www.vintagedigital.com.au/empirical-labs-distressor/
[29]: https://www.production-expert.com/production-expert-1/empirical-labs-marks-30-years-of-the-distressor-with-limited-el8-xxx-edition
[30]: https://vintageking.com/empirical-labs-el-8-xxx-distressor-30th-anniversary-edition
[31]: https://slatedigital.com/fg-stress-distressor-plugin/
[32]: https://docs.slatedigital.com/VMR/FG-Stress.html
[33]: https://gearspace.com/board/high-end/17105-scaling-distressor-attack-release-knob.html
[34]: https://gearspace.com/board/so-much-gear-so-little-time/1050264-distressor-attack-times.html
[35]: https://gearspace.com/threads/distressor-opto-mode-question.43014/
[36]: https://gearspace.com/threads/question-about-the-distressors-opto-setting-and-the-1176.457447/
[37]: https://gearspace.com/board/so-much-gear-so-little-time/963774-cool-dave-derr-tells-story-distressor.html
[38]: https://gearspace.com/board/so-much-gear-so-little-time/6683-distressor-1-1-weirdness.html
[39]: https://gearspace.com/threads/distressor-guide.61591/
[40]: https://gearspace.com/threads/distressor-tricks.43532/
[41]: https://gearspace.com/board/gear-shoot-outs-sound-file-comparisons-audio-tests/859363-distressor-hardware-versus-smack-plugin.html
[42]: https://groupdiy.com/threads/distressor-schematic.4026/
[43]: https://groupdiy.com/threads/distressor-schematic-please.75814/
[44]: https://groupdiy.com/threads/that-2181-vca-question.87727/
[45]: https://www.thatcorp.com/datashts/THAT_2181-Series_Datasheet.pdf
[46]: https://thatcorp.com/datashts/dn00A.pdf
[47]: https://thatcorp.com/datashts/dn107.pdf
[48]: https://www.thatcorp.com/datashts/AES4054_Attack_and_Release_Time_Constants_II.pdf
[49]: https://thatcorp.com/datashts/AES4703_Attack_and_Release_Time_Constants_I.pdf
[50]: https://www.eecs.qmul.ac.uk/~josh/documents/2012/GiannoulisMassbergReiss-dynamicrangecompression-JAES2012.pdf
[51]: https://www.dafx.de/DAFX_Book_Page/chapter5.html
[52]: https://aes.org/e-lib/browse.cfm?elib=3212
[53]: https://www.arturia.com/products/software-effects/comp-diode-609/overview
[54]: https://bobbyowsinskiblog.com/creator-of-the-distressor-dave-derr-on-my-latest-podcast/
[55]: https://recordingstudiorockstars.com/rsr489-dave-derr-empirical-labs-creator-of-the-distressor-arouser-big-freq-fatso-and-pump/
[56]: https://students.leedsconservatoire.ac.uk/wp-content/uploads/2025/08/Empirical-Labs-Distressor-User-Guide.pdf
[57]: https://umlsrt.com/wp-content/uploads/Studio%20Documents/EmpericalLabs_DistressorEL8X_Dynamics.pdf
[58]: https://www.musicradar.com/reviews/universal-audio-empirical-labs-el8-distressor
[59]: https://ask.audio/articles/review-universal-audio-empirical-labs-el8-distressor
[60]: https://www.nailthemix.com/uad-distressor-plugin
[61]: https://thatcorp.com/design-notes/
[62]: https://www.empiricallabs.com/arousor-rev-3/

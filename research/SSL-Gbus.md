# The SSL 4000 G bus compressor: research notes for the mainstream-VCA side of `noob-compressorlab`

Research dossier for the SSL bus compressor model of the `noob-compressorlab` example plug-in of
noob-vst-webgui-framework. The example is a humorous, affectionate spoof of the Solid State Logic
G Series stereo bus compressor. It is not a product, it is not a parity replacement, and it does not
use the SSL, Solid State Logic, G Series or Bus Compressor names as its own name. Trademarks below
belong to their owners and are used only to identify the device and the products discussed. This
model sits behind the same per-instance `model` switch that already selects the 1176, LA-2A, LA-3A,
CL 1B, 6176 and Distressor behaviours; see [[Distressor]] [56], which is its nearest relative in the
lab, and [[Neve-33609]] [57], which is the dossier this one is written to match.

Conventions, kept the same as the six existing dossiers so they read alike:

- Citations are `[n]`; the numbered list in section 14 gives the URL for every source, and
  reference-style link definitions at the very end make the `[n]` markers clickable.
- Numbers that come from a manufacturer specification, a manual, a schematic or a published
  measurement are attributed. Numbers that are my own derivation or assumption are labelled
  **derived** or **estimate**. Nothing labelled a measurement was invented; where sources disagree,
  both figures are given and the disagreement is stated rather than resolved.
- "GR" is gain reduction. "THD" is total harmonic distortion. "CV" is the sidechain control voltage.
  "VCA" is a voltage-controlled amplifier. dBu is 0.775 V RMS; dBFS is digital full scale.
- Component designators like `R39` and `C1` refer to the SSL console card drawings 82E26 and 82E27
  unless another drawing is named.
- I measured colours and proportions from photographs. Every such figure names the image it came
  from, and the images are listed in section 14 with their sources.

**The one thing to hold in mind while reading.** This is the third VCA compressor family the lab has
looked at and the second it would build, so the question this file has to answer is not "what is a
VCA compressor" but "what does *this* one do that the Distressor does not". Section 12 answers that
directly and is not optional reading: an audit of this codebase found three tube stages that were
three different circuits wearing one word, and the same trap is open here, because "VCA compressor"
covers two machines that share a chip and share almost nothing else.

---

## 1. What this unit is, and where its documents are

### 1.0 Where the documents are, and what I could not reach

The survey ranked this unit third and said its documentation position was "yes, but from clone
documentation rather than from SSL" [55]. That was right about SSL and wrong about the ceiling. Four
things turned up that the survey did not have, and two of them change the dossier.

**The console card schematics are real SSL drawings, and they are legible.** Jakob Erland's GSSL page
links two images that are not his own clone drawings at all: `ssl_82e26.gif` and `ssl_82e27.gif`
[16] [1] [2]. They are photographs of SSL's own card schematics from an SL 4044 E desk. Card 82E26
carries the title block "82E26 QUAD BUS MIX AMPS, PATCH RETURNS, VCA, COMPRESSOR SIDE CHAIN", dated
**19-8-80**, revision **G**; card 82E27 carries "CF82E27 COMPRESSOR TIME CONSTANTS, QUAD FADER,
AUTOFADE" and the drawing number **82E27-710-7911-0**. At 800 × 555 and 800 × 538 they are small,
but every component value in the timing and sidechain sections reads cleanly at 10× magnification,
and sections 3, 5, 6 and 7 below are read off them directly. **This is a manufacturer schematic, not
a clone builder's redraw**, and the survey did not know it was there.

**There is a peer-reviewed paper about this exact unit, published three months ago.** Yicheng Gu,
Runsong Zhang, Lauri Juvela and Zhizheng Wu, "Solid State Bus-Comp: A Large-Scale and Diverse Dataset
for Dynamic Range Compressor Virtual Analog Modeling", DAFx-25, Ancona, 2–5 September 2025, pages
55–60 [30]. It is a dataset paper built by recording **2528 hours** of real audio through a real
**SSL 500 G-Bus** module in 220 parameter combinations, and section 10.2 below argues it is the
single most useful document in this file, because it contains the only published comparison of
commercial emulations against measured hardware that exists anywhere in this category.

**SSL's own emulation is documented, and its documentation contradicts SSL's own hardware.** The
*SSL Native V6.5 User Guide* [9] gives the Bus Compressor plug-in's ratio positions as "2:1, 4:1 and
20:1". Every piece of SSL hardware I found says 10:1, not 20:1. Section 6.3 takes that seriously
rather than assuming a typo.

**SSL publishes a real electronic specification, but for the wrong unit.** The *XLogic Multichannel
Compressor Owner's Manual* [8] carries a full performance specification and, more usefully, a
factory calibration procedure with voltages and tolerances. It is a SuperAnalogue design, not a
G Series card, so its noise and distortion figures do not describe the box we are modelling. Its
calibration voltages, though, are the only published statement anywhere of what an SSL compressor
sidechain's control voltage actually is, and section 8.3 uses them.

**What I could not reach, said plainly.**

- **WebSearch was unavailable for this whole session** (the budget was exhausted before I started),
  so every document here was found by constructing URLs, following links out of documents I already
  had, walking the Internet Archive's search API, or querying the DAFx paper archive's own search
  endpoint. The DAFx paper was found by searching that archive for the string `SSL`, which is a lucky
  collision: the paper uses "SSL" for *self-supervised learning* as well as for Solid State Logic.
- **The *Bus Compressor 2 User Guide*** is listed as a download on SSL's own product page [10] but
  the page does not carry its URL, and every filename I constructed under
  `solidstatelogic.com/assets/uploads/downloads/plug-ins/` returned **HTTP 403**. The *SSL Native
  V6.5 User Guide* at the same path prefix downloads fine, so this is a per-file block rather than a
  blanket one. SSL's downloads page [12] is a JavaScript search form that returns "Sorry, we couldn't
  find anything!" to a plain fetch, and its product selector does not list any legacy product.
- **No SSL schematic for the G Series card.** Everything circuit-level in this file is from the
  **E Series** cards 82E26 and 82E27. Section 1.2 is honest about what that costs.
- **`cytomic.com` returns HTTP 403 to `curl`**, though `WebFetch` reaches it; the Glue quotations in
  10.3 come through that route.
- **`patents.google.com` returned HTTP 503**, so David Blackmer's VCA patent is unread. The THAT
  Corporation datasheets [24] [25] [26] cover the gain law with more precision than the patent would
  have, so nothing is lost except the primary-source pedigree.
- **No photograph of the G Series console centre section at a resolution that shows the bus
  compressor's silkscreen.** I fetched the two largest Wikimedia Commons photographs of SL 4064 G+
  and G+ 4000 consoles [50] [51] and cropped the centre section of each at full resolution; in one
  the compressor panel is out of frame and in the other it is behind a Total Recall terminal. Section
  2.5 says what follows from that.
- **No independent laboratory measurement of frequency response, distortion or noise for the G Series
  console compressor.** This is the category-wide gap the survey identified [55] and it is still
  there. What is *not* missing any more is a published error measurement of the emulations against
  hardware, which is a different and in some ways more useful thing (10.2).

### 1.1 Solid State Logic, and how a bus compressor ended up in a console

Solid State Logic was founded by **Colin Sanders in 1969**, and its first business was not audio at
all: it made solid-state control systems for pipe organs, and Sanders coined the name to explain
transistor and FET switching to organ builders [47]. He also owned Acorn Studios in Stonesfield,
Oxfordshire, and when he could not buy a console with the routing flexibility and setting recall he
wanted, he built two, designating them **SL 4000 A** [47].

The line that matters runs:

| year | event | source |
|---|---|---|
| 1969 | SSL founded by Colin Sanders | [47] |
| 1976 | **SL 4000 B**: the in-line design plus a computer for fader automation and transport auto-location. Six built, the first to Abbey Road, then Le Studio, Townhouse and Tocano | [47] |
| 1979 | **SL 4000 E**: a four-band EQ developed with George Martin, setting recall, and "the first mixer to feature a compressor/gate on every channel **as well as the master bus compressor**" | [47] |
| 1987 | **SL 4000 G**, introduced at the AES New York convention, with a redesigned EQ "among other improvements" | [47] |
| 2003 | **XLogic**: the first outboard boxes carrying console circuitry | [47] |
| 2005 | **G Series Compressor** outboard and the **X-Rack**, using "SSL's classic G Series center compressor design elements within a SuperAnalogue design topology" | [47] |
| 2006 | **Duende**, a DSP platform, "Additionally, the system offers the SSL Stereo Bus Compressor" — the first digital version | [47] |

The SL 4000 ran **from 1976 to 2002**, and the E Series is described in the same terms in the
dedicated article: "the **first console** to offer a compressor/gate on every channel **as well as a
master bus compressor**" [48]. The same article carries the Listen Mic story — Hugh Padgham
discovering gated reverb on a Townhouse B Series in 1980 because SSL's talkback circuit "employed
gating and **extreme compression**" [48] — which is worth knowing because the Listen Mic compressor is
a different SSL circuit that people sometimes confuse with this one. It also records that in 1996
Billboard's Studio Action Chart "reported that **83 % of number-one singles that year had been
produced using an SSL board**" [48], which is the commercial fact behind the reputation.

Two things in that table are worth pausing on. First, Wikipedia's SL 4000 E entry credits the master
bus compressor to the **E Series in 1979**, not the G [47]. The box is universally called "the G bus
compressor", and SSL's own current marketing says "the centre section compressor from the 1980's
Solid State Logic G-Series analogue console" [7] and "The centre section compressor from SSL's
1980's G Series analogue console" [10]. The compressor is older than the name it is sold under.
Second, the earliest hard date I have on the *circuit* is the title block of card 82E26:
**19 August 1980, revision G** [1] — where "revision G" is the card revision letter and has nothing
to do with the G Series console, a coincidence worth flagging because it is exactly the kind of thing
that gets misquoted.

**What I did not establish.** I found no primary source naming the engineer who designed the bus
compressor, and no SSL document giving the year the circuit was drawn. The card is dated 1980; the
console it came from is an SL 4044 E "dating back from '85" according to the person who took the
schematics out of it [16]. I am not going to assert a designer or a design year from that.

### 1.2 E Series and G Series, and the honest problem with calling this "the G"

**The circuit in this file is the E Series circuit.** Cards 82E26 and 82E27 came out of an SL 4044 E
desk [16]. I could not reach a G Series card drawing, and I did not find any SSL document that states
what changed in the bus compressor between the two consoles. That is a real gap and it sits under
every component value in sections 3 to 7.

What can be said about the difference is thin and I will not pad it:

- SSL's 1987 announcement of the G Series is described as "a redesigned EQ, among other improvements"
  [47]. The EQ change is documented; the compressor change, if there was one, is not.
- The E card already has the **Auto release position** (pin 9 of the release switch, silkscreened `A`,
  with the two-section network of 7.4) [2]. So the famous Auto release is **not** a G Series
  addition, which is the most common thing people assume.
- The E card's threshold pot ends are annotated **+15 dB** and **−15 dB** on the drawing [2], while
  every later SSL unit — the 500-series module [3], the recall sheet [4], the XLogic [8] and the
  Native plug-in [9] — gives **−20 dB to +20 dB**. That is a documented change, though not one I can
  date to the E-to-G transition specifically.
- SSL's own modern hardware, THE BUS+, offers a **4K MODE** which "changes the operation of the VCA
  from balanced to unbalanced (**matching how the Bus Compressor in a 4000-series console was
  implemented**)" [11]. That is SSL, in 2023, telling us a specific circuit fact about the 4000-series
  compressor: its VCA ran unbalanced. It is the only statement of its kind I found from SSL.

**So what do I call the model?** The lab should call it the G, because that is the name the sound is
sold under and the spoof is of a reputation as much as of a circuit. But the dossier must not pretend
the drawing is a G drawing, and section 11 marks every constant that comes from the E card as such.

### 1.3 The outboard descendants, and why there are three different control sets

The same compressor has been sold with three different sets of switch values, and confusing them is
the easiest mistake available here.

| unit | ratios | attack (ms) | release (s) | sidechain HPF | source |
|---|---|---|---|---|---|
| **SL 4000 console centre section** (cards 82E26/82E27) | 2, 4, 10 | .1, .3, 1, 3, 10, 30 | .1, .3, .6, 1.2, Auto | none | [16] [1] [2] |
| **Alan Smart C1** | as C2 minus Crush and sidechain in | as C2 | .1, .3, .6, 1.2, 2.4 | external cable option, 150 Hz 6 dB/oct | [22] |
| **Alan Smart C2** | 1.5, 2, 3, 4, 10, **Limit** | **0**, .1, .3, 1, 3, 10, 30 | .1, .3, .6, 1.2, 2.4 | external cable option | [22] |
| **XLogic Multichannel Compressor** (2004) | 1.5, 2, 3, 4, 5, 10 | .1, .3, 1, 3, 10, 30 | .1, **.2, .4, .8, 1.6**, Auto | LFE only, 120 Hz low-pass | [8] |
| **500-series G-Comp module** (current) | 1.5, 2, 3, 4, 5, 10 | .1, .3, 1, 3, 10, 30 | .1, .2, .4, .8, 1.6, Auto | Off, 30, 60, 105, 125, 185 Hz | [3] [4] [7] |
| **SSL Native Bus Compressor** plug-in | 2, 4, **20** | .1, .3, 1, 3, 10, 30 | .1, .3, .6, 1.2, Auto | yes, unspecified | [9] |

Three observations fall straight out of that table.

**The release values changed and nobody says why.** The console runs 0.1 / 0.3 / 0.6 / 1.2 s; every
modern SSL unit runs 0.1 / 0.2 / 0.4 / 0.8 / 1.6 s, a clean binary ladder with an extra step. SSL's
own plug-in keeps the *console* values, which suggests SSL themselves treat the console values as
the canonical ones for an emulation. Section 7.5 takes that as the model's default.

**SSL's marketing bullet for the 500-series module is wrong, or at least not self-consistent.** The
product page lists as a feature "Additional compression ratio settings **1.5 / 3 / 10**" [7]. The
module's own panel [3] and recall sheet [4] show 1.5, 2, 3, 4, 5, 10. If the classic set is 2, 4, 10,
the additions are 1.5, 3 and **5**, not 10. The survey repeated SSL's bullet as "the added 1.5, 3 and
10 ratios" [55]; it should not be repeated again.

**Alan Smart is not a third-party cloner.** Alan Smart was an SSL commissioning and service engineer
who "engineer[ed] at SSL's 'Huge' studios in Oxford", took "a position as commissioning and service
engineer for SSL", and commissioned or serviced Townhouse 1 and 2, Sarm West, Wessex, Abbey Road and
Air Studios during the period the 4000 series was spreading [23]. The C1 and C2 are therefore closer
to a factory descendant than to a clone, and their published figures (8.5) are worth more than a
random clone's. Erland lists the Alan Smart C2 and the "SSL Logic FXG 384" as aliases of the same
box [16].

### 1.4 THE BUS+, which is SSL telling us about the old circuit by selling a new one

THE BUS+ (2023) is a current SSL bus compressor whose colouration switches are, in effect, a list of
things the G Series compressor did that a modern clean design does not. SSL describe three [11]:

> **LOW THD MODE**: introduces a special circuit modification in the side-chain, helping to limit the
> amount of low frequency distortion compression can create, especially with fast release times.
>
> **F/B (FEED-BACK) MODE**: takes the signal feeding the side-chain from a feed-back position (i.e.
> after the main gain-reduction VCA in the audio path). This results in a more 'relaxed' style of
> compression, in contrast to the traditional 'grab' of the Bus Compressor.
>
> **4K MODE**: changes the operation of the VCA from balanced to unbalanced (matching how the Bus
> Compressor in a 4000-series console was implemented). It also introduces a variable amount of
> harmonic distortion via the VCA.

It also adds "Negative Ratios — for creative pumping effects", four stereo modes (Classic Stereo,
Σ S/C Stereo, Dual Mono, Mid-Side) and "New Attack and Release options, including the new 'Auto 2'
setting" [11].

The F/B quotation is the one that matters, and section 5.3 is built around resolving it against the
schematic, because taken at face value it says the classic bus compressor is **not** a feedback
design, and the schematic says something more interesting than either answer.

### 1.5 Why it is famous

SSL's own copy is unusually plain about what the box is for:

> It is a simple unit with a simple purpose; it makes complete mixes sound bigger, with more power,
> punch and drive. It brings cohesion and strength to your mix without compromising clarity. [10]

and

> "Sticks your mix like audio glue." — That's how we often hear the SSL Stereo Bus Compressor
> described, along with — "You strap it across your mix — and it sounds like a record." [10]

Cytomic named their emulation *The Glue* after that description [42]. The reputation is not for
transparency and not for character in the 1176 sense; it is for a specific, narrow trick — a couple
of decibels of gain reduction across a whole mix that makes the mix sound like a finished record —
and an emulation is judged almost entirely on whether it does that trick. Section 9 turns that into
a list of things the model must get right.
---

## 2. Controls, the front panel, and enough geometry to draw a faceplate

### 2.1 The controls, and what each one really does

| control | type | range / values | what it does |
|---|---|---|---|
| **THRESHOLD** | continuous pot, 50 kΩ linear | −20 to +20 dB on every modern unit [3] [4] [8]; the E card's drawing annotates its ends **+15 dB** and **−15 dB** [2] | Does **not** set a comparator reference. It adds a DC offset to the *sidechain* VCA's control port only (3.4), so it changes how hot a signal the detector sees. The real threshold is a diode drop (5.1). |
| **MAKE UP** | continuous pot, 25 kΩ linear [2] | 0 to +20 dB [16]; the module panel prints only `0`, `−` and `+` [3] | Summed into the *audio* VCAs' control voltage, and **permanently in circuit**: "On the original SSL compressor the makeup gain pot is active all the time, so when bypassed there's excess gain" [16]. |
| **ATTACK** | 6-position rotary | 0.1, 0.3, 1, 3, 10, 30 ms | Selects one of six resistors, 820 Ω to 270 kΩ, that charge the timing capacitor (7.1). |
| **RELEASE** | 5-position on the console, 6 on modern units | .1, .3, .6, 1.2, Auto (console) / .1, .2, .4, .8, 1.6, Auto (modern) | Selects the timing capacitor **and** its discharge resistor together; Auto substitutes a two-section network (7.4). |
| **RATIO** | 3-position on the console (4-pole), 6 on modern units | 2, 4, 10 / 1.5, 2, 3, 4, 5, 10 | Changes the sidechain gain **and**, deliberately, the threshold (6.2). |
| **HPF** | 6-position rotary, later units only | Off, 30, 60, 105, 125, 185 Hz [3] [4] [7] | High-passes the sidechain only. Not on the console compressor. |
| **IN** | latching switch | — | "The main VCA is permanently in circuit; the compressor sidechain is enabled by the IN switch" [3]. So this is not a bypass: the audio still goes through the VCA, and the make-up gain still applies. |
| **COMPRESSION meter** | moving-coil, 0 to 20 dB | linear scale | Reads gain reduction, driven from the control voltage at about 50 µA/dB [16]. |

**The IN switch deserves its own sentence**, because it is the one control on this box whose
behaviour is stated by SSL and is not what a plug-in author would guess. It does not remove the gain
element. It removes the *sidechain*. The audio still passes through the VCA and the make-up gain is
still applied, which is why the clone builder added an option to disconnect the make-up when bypassed
and wrote "I don't think you will" prefer the original scheme [16]. A faithful model's `ssl_in`
switch must therefore leave make-up gain applied when it is off, and the plug-in's own bypass has to
be a separate, honest, sample-exact bypass.

### 2.2 The front panel, measured

I have SSL's own product render of the **500-series G-Comp module** at 1618 × 2697 [5], and SSL's own
**recall sheet** [4], which is a line drawing of the same panel with every legend and every detent
dot. Everything below is measured off the render unless it says otherwise. The image is saved as
`ref/ssl-SSL_500_G-Comp._1685.png` and the recall-sheet render as `ref/ssl-recall-p1.png`.

**Overall proportions.** The module face measures 1429 × 2528 px, an aspect ratio of **1 : 1.769**.
A single 500-series slot is 1.5 × 5.25 inches (1 : 3.5); a **double** slot is 3.0 × 5.25 inches
(1 : 1.75). The measured 1.769 matches the double within 1 %, so the module is two slots wide and
the scale is **476 px per inch horizontally, 482 px per inch vertically**. Every fraction below is
of the panel's own width `W` and height `H`, which is the form the faceplate code wants.

| feature | x (fraction of W) | y (fraction of H) | source |
|---|---|---|---|
| panel face | 0 → 1 | 0 → 1 | measured [5] |
| thin white outline rectangle | 0.127 → 0.873 | 0.059 → 0.942 | measured [5] |
| meter escutcheon (matt black plate) | 0.197 → 0.810 | 0.091 → 0.384 | measured [5] |
| meter glass (the lit scale window) | 0.222 → 0.780 | 0.092 → 0.321 | measured [5] |
| meter zero-adjust screw slot | centred 0.503 | ≈ 0.230 | measured [5] |
| IN switch cap | 0.810 → 0.936 | 0.417 → 0.495 | measured [5] |
| knob column, left | centre 0.348 | — | measured [5] |
| knob column, right | centre 0.661 | — | measured [5] |
| knob row 1 (THRESHOLD, MAKE UP) | — | centre 0.509 | measured [5] |
| knob row 2 (ATTACK, RELEASE) | — | centre 0.690 | measured [5] |
| knob row 3 (RATIO, HPF) | — | centre 0.871 | measured [5] |
| panel screws (four, hex socket) | centres 0.248 and 0.752 | ≈ 0.036 and ≈ 0.958 | measured [5] |
| screw head diameter | 0.069 W | — | measured [5] |

**Two knob sizes, and this is easy to miss.** THRESHOLD and MAKE UP are visibly smaller than the four
switches below them:

| knob | coloured cap diameter | metal skirt diameter |
|---|---|---|
| THRESHOLD, MAKE UP (pots) | **0.102 W** | **0.126 W** |
| ATTACK, RELEASE, RATIO, HPF (switches) | **0.118 W** | **0.168 W** |

At 3.0 inches of panel width that is a 7.8 mm cap on a 9.6 mm skirt for the pots, and a 9.0 mm cap on
a 12.8 mm skirt for the switches. The skirt-to-cap ratio also differs, 1.23 for the pots against 1.43
for the switches, so they are two different knob parts and not one part at two sizes. Erland notes
that SSL used **Sifam** knobs and **Sifam AL29** moving-coil meters, and gives Farnell 969-746 and
RS 225-704 as the part numbers to look like the original [16] — worth having in the faceplate
comments even though those are current catalogue numbers rather than SSL's own.

**Silkscreen, exactly as printed** (render [5] cross-checked against the recall sheet [4], which
differs in two places, noted):

- Top row, either side of the two upper screws: **`SSL`** in bold, then **`G`** in bold followed by
  **`-COMP`** in light weight. The recall sheet writes it `SSL` … `G COMP` with a space and no
  hyphen.
- Bottom, centred between the two lower screws: **`STEREO BUS COMPRESSOR`**. The recall sheet has
  **`Solid State Logic`** there instead, in a script face.
- Meter face: **`dB`** in bold on the upper line, **`COMPRESSION`** below it, both centred low in the
  window. Scale numerals **`0 4 8 12 16 20`**.
- Knob 1: **`THRESHOLD`** above, **`0`** at twelve o'clock, **`-20`** and **`+20`** at the extremes,
  **`dB`** below.
- Knob 2: **`MAKE UP`** above, **`0`** at the nine-o'clock end, **`-`** and **`+`** at the extremes,
  **`dB`** below. Note the asymmetry: MAKE UP's zero is at one end of its travel, THRESHOLD's is in
  the middle, and their detent-dot rings are correspondingly rotated relative to each other.
- Knob 3: **`ATTACK`** above; detents **`.1 .3 1 3 10 30`**; **`ms`** below. Recall sheet: `ATTACK - mS`.
- Knob 4: **`RELEASE`** above; detents **`.1 .2 .4 .8 1.6 AUTO`**; **`s`** below. Recall sheet:
  `RELEASE - S`.
- Knob 5: **`RATIO`** above; detents **`1.5 2 3 4 5 10`**; nothing below.
- Knob 6: **`HPF`** above; detents **`OFF 30 60 105 125 185`**; **`Hz`** below. Recall sheet:
  `HPF - Hz`.
- Switch: **`IN`** on the cap.

**The detent dots.** Every switch and both pots carry a ring of small white dots, one per position for
the switches and eleven for the pots, arranged over roughly 300° with the gap at six o'clock. The
pointer is a single white radial bar from the cap's centre to its rim. On the switches the currently
selected position also shows a short black bar cut into the skirt at that angle.

### 2.3 Colours, measured from the render

Medians over sampled patches of `ref/ssl-SSL_500_G-Comp._1685.png` [5]:

| element | measured | note |
|---|---|---|
| panel face, brushed aluminium | **`#535353`** | vertical brushed grain; sampled between the knob rows and down the left margin, both give 82–84 in all three channels |
| module edge / surround | **`#000000`** | |
| knob cap, blue | **`#5884A9`** (88, 132, 169) | median over every blue pixel on the panel; the caps are lit from above so the top of a cap runs to about `#6E93B6` and the bottom to about `#4A7091` |
| knob skirt, metal | **`#323232`** | |
| meter glass | **`#010101`** | effectively black |
| meter escutcheon plate | **`#302F2D`** | a matt, slightly warmer black than the glass |
| IN cap | **`#C6C6C6`** | a light grey plastic cap, the only bright object on the panel |
| all silkscreen | **`#FFFFFF`** | pure white, both text and detent dots |

The whole design is therefore **one accent colour on a grey ground**: a single blue for the six knob
caps, white for everything printed, and a light grey for the one switch. Which is exactly the kind of
restraint the framework rule says belongs in the example plug-in and not in the framework: the lab's
faceplate should carry *this* palette because it is *this* unit's face, and nothing about it should
leak into shared code.

### 2.4 The meter

A moving-coil gain-reduction meter reading **0 to 20 dB, left to right, on a linear scale**, with
numbered ticks at 0, 4, 8, 12, 16 and 20 and unnumbered ticks between them [5] [4]. The pointer
pivots below the visible window and rests at 0 at the far left. There is a zero-adjust screw in the
escutcheon below the glass.

The drive is documented by the clone builder, who read it off the same circuit: the meter is fed from
the buffered control voltage through a series resistor, "This is linear scale, at about **50 µA/dB**,
making a **1 mA meter showing 20 dB full-scale**", with a 2 kΩ series resistor for a 1 mA movement
and a 1 kΩ substitution if you want 10 dB full scale [16]. That is a useful, checkable statement: it
says the meter reads the **control voltage linearly**, not a dB conversion of a measured gain, so a
model that computes 20·log10 of its own gain reduction and paints that on the meter will be right
only to the extent that the control voltage is linear in dB — which, for a Blackmer VCA, it is
(4.2). This is the rare case where the naive meter and the circuit meter agree, and the dossier
should say so rather than build machinery it does not need.

### 2.5 The console centre section, and what I could not confirm

I could not obtain a photograph in which the G Series console's bus compressor panel is legible
(1.0). What I can say about it comes only from the card drawings and from the control list in 1.3:
the console panel carries **THRESHOLD, MAKE UP, ATTACK, RATIO, RELEASE** and an **IN** switch, plus
the same compression meter, and it has **no sidechain high-pass**. The switch values are the console
values of 1.3. The E card's threshold annotation of ±15 dB [2] means I cannot even assert that the
console panel printed −20/+20.

**What follows for the plug-in.** Draw the 500-series module. It is the version SSL still sells, SSL
publish a high-resolution render and a dimensioned recall sheet of it, it carries the whole control
set including the sidechain filter, and it is the exact unit the DAFx dataset was recorded through
[30], so the faceplate and the ground truth describe the same box. Then let the *values* on the
switches follow the console (1.3, 7.5), and say so in the plug-in's own help text. Drawing a panel I
cannot see and inventing its silkscreen would be precisely the failure this repository's research
standard exists to prevent.
---

## 3. Signal path and circuit, read from SSL's own card drawings

Everything in this section is read from the two card images at 5× to 16× magnification. Where a value
is at the edge of legibility I say so. Crops are saved beside the originals in the session `ref/`
folder with names beginning `z-82e26-` and `z-82e27-`.

### 3.1 The two cards, and how the compressor is split across them

The console bus compressor is not one module. It is spread over two cards in the main mix section,
and the split is functional, which is why both are needed to understand it:

| card | title block | what is on it |
|---|---|---|
| **82E26** | "82E26 QUAD BUS MIX AMPS, PATCH RETURNS, VCA, COMPRESSOR SIDE CHAIN", date **19-8-80**, rev **G** [1] | the quad bus mix amplifiers, the **audio VCA**, the **sidechain VCA**, the full-wave rectifier, the **RATIO switch network**, the ±12 V subregulators |
| **82E27** | "CF82E27 COMPRESSOR TIME CONSTANTS, QUAD FADER, AUTOFADE", drawing **82E27-710-7911-0** [2] | the **ATTACK and RELEASE networks**, the timing buffer, the **MAKE UP** and **THRESHOLD** summing, the control-voltage distribution to both sets of VCAs, and the quad fader and autofade machinery the compressor shares the VCAs with |

That last column is the structural fact that explains the whole design. **The compressor does not own
its VCAs.** The same VCAs are the console's quad master fader and its autofade, so the compressor has
to reduce gain by adding to a control voltage that the fader and the automation are also using. Erland
puts it exactly: the sidechain VCA exists "in the original design, the 4000E console, to use the
single set of main VCA's for compression, fader, computer and autofader at the same time — without
having to resort to less predictable feed-forward compression schemes" [16].

Card 82E27 also carries a hand-lettered note that shows the compressor's position in the console was
a build option:

> FIT R26 FOR COMPRESSOR POST FADER. OMIT R26 FOR COMPRESSOR PRE FADER. [2]

### 3.2 The audio path

Short, and that is the point. Per channel, on card 82E26 [1]:

```
mix bus  →  bus mix amp (NE5534)  →  R12 68K1 (0·5 %)  →  A1  →  T3 (NE5534) I/V  →  out
                                       current in       dbx 202C      R13 4K7, R15 470R, C10
```

- **R12 is 68.1 kΩ and carries the drawing's `*` mark, which its own note defines as "0·5 % Tol."**
  [1]. It is the only tolerance called out anywhere near the gain element, which tells you SSL cared
  about the VCA's input current scaling and about channel matching.
- **A1 is a dbx 202C**, drawn as a six-pin block annotated "BOTTOM VIEW", with ±15 V rails and C22 /
  C23 (10 µF) decoupling [1]. Section 4.1 is about that part.
- **RV1, a 50 kΩ trimmer marked `DISTORTION NULL`**, runs between +15 V (pin 31) and −15 V (pin 32)
  and feeds the VCA's symmetry pin through **R14, 1 MΩ** [1].
- **T3 is an NE5534** current-to-voltage converter, feedback **R13 4K7** with **C10**, and **R15
  470R** on its non-inverting input [1].

The drawing's own general notes read "**All Diodes 1S44**" and "**Last used R58, C29, D11, T8, TR4,
A2**" [1], which is a useful sanity check: two active devices lettered A (the two VCAs), eight
op-amp positions, and one FET.

There are no transformers, no tubes and no deliberate distortion stage anywhere in the audio path.
The only nonlinearity in the signal chain **is the VCA itself**. Hold that thought until section 12,
because it is the sharpest difference between this box and the Distressor.

### 3.3 The sidechain

Also on card 82E26 [1], and this is where the design gets interesting:

```
L + R summed  →  TR4 (E175 JFET)  →  A2 (dbx 202C, the "dummy" VCA)  →  T5 (741) I/V
                                       R31 4K7          R32 33K ∥ C17 22p, R33 470R
      →  C18 6µ8  →  ┬─ R34 20K → T6 (741) half-wave rectifier, R35 20K with D2/D3 in the
                     │                                       feedback  → R36 10K ──┐
                     └─────────────── R37 20K ─────────────────────────────────────┤
                                                                                    ↓
                                              T7 (741), feedback R38 1M with D4/D5 across it
                                                                     ↓
                                              RATIO SW1 (pole 36, throws 37/38/39)
                                                                     ↓
                                                    D6  →  pin 42  →  card 82E27
```

- **The rectifier is a textbook precision full-wave rectifier.** T6 with R34/R35 at 20 kΩ each is a
  unity-gain inverting half-wave stage; its output reaches the T7 summing node through **R36, 10 kΩ**,
  while the unrectified signal reaches the same node through **R37, 20 kΩ**. The 2 : 1 ratio between
  those two resistors is what makes |x| out of x, and it is exactly the standard value pair. Erland
  describes the same thing from the same drawing: "The sidechain signal that is obtained after the VCA
  is then full-wave rectified by two TL074-stages" [16] — TL074 in his clone, 741s on SSL's card.
- **D4 and D5 sit across R38, T7's 1 MΩ feedback resistor**, anti-parallel. That makes T7's gain fall
  once its output exceeds a diode drop, which is a soft nonlinearity right in the middle of the
  detector. I read the connection as anti-parallel across the feedback; at this scan resolution I
  cannot rule out that they are two series diodes to a clamp rail instead, and I flag it as **a
  reading I could not close**.
- **D6 sits between T7's output and pin 42**, which is the wire to the timing card. That single diode
  is the compressor's real threshold, and Erland says so in as many words: "The 'real' threshold to
  be overcome before charging the attack/release caps, is the **0.6 V** AK voltage across the diode
  placed between the fullwave rectifier and the A/R timing" [16]. Section 5.1 makes that the centre of
  the model.
- **SSL's own words describe the same detector at the system level:** "The left and right channels are
  independently rectified using a **true peak full wave detector circuit**, and the dominant, ie.
  louder channel, controls the gain reduction of the overall stereo level" [3]. Section 5.5.

### 3.4 The control voltage, and where it goes

On card 82E27 [2], reading left to right from the timing network:

```
pin 42 in  →  ATTACK resistor  →  RELEASE cap/resistor  →  pin 11  →  T1 (LF351N)
                                        R13 3M3 to ground, R14 470K feedback
   →  D2 (1S44)  →  R16 4K7  →  node clamped by D3 (12 V zener)
   →  MAKE UP pot, 25K linear (pins 21 / 12 / 10)  →  wiper  →  R20 470K ─┐
                                                          R25 100K ───────┤
                                                                          ↓
                                     T2 (¼ 3403) summing amp  →  R51 100R  →  pin 15
                                                                  C12 0·1   AUDIO VCA'S
                                                                            ("QUAD FADER")
   and, in parallel:
                    R21 100K ─┬─ R26 100K → the T2 node above
                              └─ R27 100K ─┐
   THRESHOLD pot (R17 3K9 from +12, end marked "+15dB"; R18 360R, end marked "−15dB")
        wiper pin 24  →  R22 360K ─────────┤
                                           ↓
                                     T3 (351)  →  R44 100R  →  pin 32  SIDE CHAIN VCA'S
```

**This is the whole architectural argument of the unit, in one paragraph.** The detector's control
voltage arrives at a node and is split by three 100 kΩ resistors, R21, R26 and R27. R26 sends it to
the amplifier that drives the **audio** VCAs; R27 sends it to the amplifier that drives the
**sidechain** VCA. The **THRESHOLD** pot adds a DC offset through R22 (360 kΩ) into the sidechain
amplifier's summing node **and nowhere else**. The **MAKE UP** pot adds its offset into the audio
amplifier's node **and nowhere else**.

So:

- **the sidechain VCA's gain = the audio VCA's gain, plus a threshold offset.** The detector therefore
  hears the input *as gain-reduced as the audio is*, offset by the threshold setting.
- **make-up gain is applied by the same control voltage that does the compressing**, on the audio side
  only, which is why it is live even with the sidechain switched out (2.1).
- **there is no comparator anywhere.** Nothing in this circuit compares a level against a threshold.
  The threshold is a pot that changes a gain, and a diode drop that has to be overcome.

The 100 Ω series resistors R51 and R44 with C12 (0.1 µF) at the outputs are there to keep the control
buses quiet; the THAT datasheet warns that "stray control-path pickup" is a real distortion mechanism
in these parts [26], so the low output impedance is deliberate rather than incidental.

### 3.5 What the drawings do not show

- **The RATIO switch is a four-pole, three-position type** — the drawing labels it "RATIO SW1 **A-D**"
  [1], and Erland's parts list for the clone calls for a "Lorlin or similar rotary switch, **4sw x
  3positions**" [17]. Card 82E26 shows **one** of those four poles. The other three go somewhere I
  cannot see, and section 6 says what that costs.
- **No G Series card**, so no way to check any of the above against the console the box is named for
  (1.2).
- **No stereo detail.** The cards are drawn per-channel and the "dominant channel" comparison SSL
  describe [3] is not visible on either drawing at this resolution.
- **No published block diagram from SSL** for the console compressor, of the kind AMS Neve print for
  the 33609. SSL's operational prose [3] is the nearest equivalent and it is three paragraphs long.

---

## 4. The VCA, and its control law

### 4.1 Which part, and why that matters

**The console used a dbx 202C.** Both A1 (audio) and A2 (sidechain) on card 82E26 are lettered
`DBX 202C` [1]. That is a *module*, not a monolithic chip, and Erland reports reverse-engineering
one: "it turned out to consist of **ten paralleled 2150's with a common low-impedance buffer for the
control inputs**" [16]. Ten cells in parallel divides the noise and the offsets by √10 and is exactly
what you would do to make a fader-quality VCA out of what was then a commodity part.

That immediately gives the lineage:

| part | what it is | in this box | source |
|---|---|---|---|
| **dbx 202C / 202XT** | a potted module: ten paralleled dbx 2150 cells with a common control buffer | the original console part | [16] [1] |
| **dbx 2150 series** | David Blackmer's log-antilog VCA as a monolithic IC | the cell inside the 202, and the part SSL used in the *channel* compressors | [16] [26] |
| **THAT 2180** | pre-trimmed replacement for the 2150; "Pin-Compatible with 2150-Series" | the modern substitute; pin 4 (distortion trim) is simply cut off | [16] [24] |
| **THAT 2181** | the same with an external trim pin | the other modern substitute | [16] [25] |

THAT's own 2150 datasheet says the parts are "**Based on dbx technology**" [26], so the substitution
is a continuation rather than an approximation. Erland's practical verdict, from having built the
same circuit with all three: "the 'sound' doesn't seem to be that different, only the 202 may tend to
be a little more transparent than the 2150 — but that's not always a good thing in compressors" [16].
He also notes a real consequence of the substitution: "the 100 K resistor marked has to be replaced by
a **127 K** resistor to compensate for higher input current sensitivity on the new chips" [16].

### 4.2 The gain law, as a specification rather than a guess

The Blackmer cell's law is exponential in decibels, and THAT publish it with tolerances [24] [25]
[26]:

| quantity | 2180A / B / C | condition |
|---|---|---|
| **Gain-control constant, EC−** | **−6.2 / −6.1 / −6.0 mV/dB** (min / typ / max) | TA = 25 °C, TCHIP 35 °C |
| Gain-control constant, EC+ | +6.0 / 6.1 / 6.2 mV/dB | same |
| **Gain-control temperature coefficient** | **+0.33 %/°C**, referenced to TCHIP = 27 °C | −60 dB < gain < +40 dB |
| **Gain-control linearity** | **0.5 % typical, 2 % maximum** | over −60 dB to +40 dB, a 100 dB span |
| Gain at 0 V control voltage | 0.0 dB, ±0.1 / ±0.15 / ±0.2 dB by grade | EC− = 0 mV |
| Dynamic range | > 120 dB | |
| Gain range | > 130 dB | |
| Gain-bandwidth | 20 MHz | |
| Slew rate | 12 V/µs | RIN = ROUT = 20 kΩ |
| Off isolation at 1 kHz | 110 dB min, 115 dB typical | EC+ = −360 mV, EC− = +360 mV |
| **THD, no external trim** | **0.005 / 0.010 / 0.030 %** typical by grade | VIN = 0 dBV, 0 dB gain, 1 kHz |
| THD, no external trim | 0.020 / 0.030 / 0.040 % typical | VIN = +10 dBV, −15 dB gain |
| THD, no external trim | 0.020 / 0.030 / 0.040 % typical | VIN = −5 dBV, +15 dB gain |
| Output noise | −98 dBV typical | 20 Hz–20 kHz, ROUT = 20 kΩ, 0 dB gain |
| Output noise | −88 dBV typical | +15 dB gain |

The 2150 datasheet adds the symmetry specification the console's `DISTORTION NULL` trimmer exists to
set: **symmetry control voltage −1.6 to +1.6 mV** for the A grade at 0 dB gain with THD below 0.07 %
[26], and typical THD of 0.004 % once trimmed.

Two of those numbers do more work than the rest.

**−6.1 mV/dB is the entire reason this model is computed in dB.** The control port is linear in
decibels of gain by construction, so a control voltage *is* a gain in dB up to a scale factor. Every
equation in section 11 is in dB, and that is not a convenience, it is what the hardware does.

**+0.33 %/°C is why two units never quite match.** A 10 °C difference between two chips is a 3.3 %
difference in dB-per-volt, which at 10 dB of gain reduction is a third of a decibel. THAT also note
that the symmetry null "drifts with frequency in the presence of stray control-path pickup" [26],
which is a second, independent mechanism for unit-to-unit variation. I would not model either — see
9.3 — but they are the honest explanation for why engineers insist their unit sounds different from
the one down the hall, and the tribute can put a line about it in the help text.

### 4.3 Balanced or unbalanced, and what SSL say about it now

The THAT parts have two control ports, EC+ and EC−, and the datasheet notes it is "possible (and
sometimes advantageous)" to drive both [24]. SSL's THE BUS+ offers a switch that "changes the
operation of the VCA from **balanced to unbalanced** (matching how the Bus Compressor in a 4000-series
console was implemented). It also introduces a variable amount of harmonic distortion via the VCA"
[11].

Read carefully, that is SSL saying three things about the console box: its VCA ran **unbalanced**;
running it unbalanced produces **more harmonic distortion** than the balanced arrangement; and SSL
consider that distortion a desirable, sellable part of the sound. Card 82E26 shows the control input
arriving on a single pin from R51/R44 through a single-ended bus [1] [2], which is consistent.

So the model's distortion budget is not "add a saturator after the VCA". It is "the gain cell itself
is imperfect, and its imperfection is second-harmonic and rises with drive". Erland, who has measured
his own builds: "the distortion doesn't really have an annoying character: it tends to be **almost
exclusively second harmonic** as far as I can measure... and hear", and he reports that a
manufacturer building units on this design deliberately trimmed the VCAs "a bit off-center — just to
get a little more 'sound'" [16]. His published figure for the finished clone is "**About −75 dB
unadjusted. Mostly 2nd harmonic**" [16], which is 0.018 %.
---

## 5. The sidechain, and the question the whole model turns on

### 5.1 The threshold is a diode, not a comparator

There is no comparator in this compressor. Gain reduction begins when the rectified, ratio-scaled
detector voltage exceeds **one diode drop**, D6, on its way from card 82E26 to the timing network on
82E27 [16] [1]. Below that the timing capacitor simply is not charged and the control voltage is
whatever the make-up pot alone puts on it.

That single fact does more to shape the sound than anything else in the box, because a diode does not
switch on at 0.6 V, it turns on over a decade or two of current. The transfer curve therefore has no
corner at all: it bends. Every description of this compressor as having a soft knee, and SSL's own
statement that the knee point moves (6.2), traces back to this one component.

### 5.2 The tracking dummy VCA, and what it is for

The second dbx 202C, A2 on card 82E26, is not in the audio path. It sits in the sidechain, fed from
the summed left and right inputs through TR4 (an E175 JFET) and R31, and it is driven by the **same
control voltage as the audio VCAs**, plus the threshold offset (3.4) [1] [2].

Erland's account, from the same drawing:

> The purpose of this VCA is to act like a tracking "dummy" VCA, paralleling the GR action of the main
> VCA's, and thereby making it possible (in the original design, the 4000E console) to use the single
> set of main VCA's for compression, fader, computer and autofader at the same time — without having
> to resort to less predictable feed-forward compression schemes. So this is a combination of a
> feed-forward and a feed-back architecture, **acting mostly as a feed-back compressor** [16]

and, on the threshold:

> The DC sidechain control signal is also summed with another DC voltage — coming from the "threshold"
> pot — and used to control the sidechain VCA. In this way the added gain in the sidechain VCA **looks
> to the rectifier like there's more signal coming in**, changing the threshold this way [16]

I did not take that on trust. Section 3.4 traces it on SSL's own drawing: R21 splits the detector's
control voltage, R26 carries it to the audio-VCA amplifier and R27 carries it to the sidechain-VCA
amplifier, and only R22 (from the threshold pot) is added to the second one [2]. **Erland's reading
is correct and it is confirmed by the manufacturer's drawing.**

### 5.3 Feedforward or feedback? SSL's own product line says one thing, the drawing says another

This matters more than any other single question in the file, because it decides the engine's shape.

**The survey called it feedforward** [55]. **SSL's THE BUS+ implies feedforward**, by offering a
"F/B (FEED-BACK) MODE" that "takes the signal feeding the side-chain from a feed-back position (i.e.
after the main gain-reduction VCA in the audio path)" and describing the result as "a more 'relaxed'
style of compression, **in contrast to the traditional 'grab' of the Bus Compressor**" [11]. If the
classic box were already a feedback design, that mode would not be a change.

**The drawing says the loop is closed.** The sidechain VCA's gain equals the audio VCA's gain plus a
fixed threshold offset (3.4, 5.2). The rectifier therefore sees a signal that has been attenuated by
exactly the amount the compressor is currently attenuating the audio. Whether the sidechain tap is
physically before or after the audio VCA makes no difference to the mathematics: the detector's input
level is `L − GR + T`, and `GR` is the thing the detector is computing. **That is a feedback loop.**

The resolution, and I think it is the interesting answer rather than a fudge:

- Topologically the audio path is **feedforward** — the audio never passes through anything the
  detector has touched, so there is no latency, no detector noise in the audio, and no stability
  problem in the signal path.
- Behaviourally the control law is **feedback** — the detector's input is gain-reduced, so the loop
  gain shapes the ratio (5.4) and the ballistics (7.3).
- SSL's F/B MODE on THE BUS+ is a statement about **THE BUS+**, a 2023 design whose 4K MODE also has
  to be switched on to match how the 4000-series VCA was wired [11]. Its default is not the console
  circuit, so its F/B switch is not evidence about the console circuit.

**And there is independent corroboration from outside SSL entirely.** The DAFx-25 team, fitting
grey-box models to 2528 hours of recordings from a real module, report that the residual error is
concentrated exactly where a missing feedback path would put it:

> a noticeable performance gap is observed between seen and unseen parameter settings. This can also
> be attributed to the changing compressor curve in the analog module, making it hard for **grey-box
> models without explicit feedback mechanisms** to capture that information. [30]

A team with no stake in the argument, fitting models to measurements rather than reading a drawing,
found that models lacking a feedback mechanism cannot reproduce this compressor's changing curve.
That is the strongest evidence available and it agrees with the drawing.

**Decision for the model: build it as a feedback compressor**, with the detector reading
`input − current gain reduction + threshold`, and say so in the code comment with a pointer here.

### 5.4 What feedback plus a linear rectifier plus a dB-domain VCA actually produces

This is the derivation that makes the box make sense, and I have not seen it written down anywhere,
so everything in this subsection is **derived** and labelled as such.

Write, in steady state:

- `L` — input level, dB
- `GR` — gain reduction, dB, positive
- `T` — the threshold pot's offset, dB, applied to the sidechain VCA only
- `A` — a constant folding in the input summing gain and the rectifier's scaling, volts per unit
  amplitude
- `G` — the detector stage's gain, set by the RATIO switch (6.1)
- `V_d` — D6's forward drop, about 0.6 V [16]
- `k` — volts of control voltage per dB of gain reduction at the audio VCA, set by the R21/R26/R27
  divider and the VCA's own 6.1 mV/dB (4.2)

The sidechain VCA's output is `A · 10^((L − GR + T)/20)`. The detector multiplies by `G`, D6 subtracts
`V_d`, and the result is the control voltage, which is `k · GR`:

```
k·GR + V_d = G·A · 10^((L − GR + T)/20)
```

Differentiate with respect to `L`, using `d/dL [10^(u/20)] = (ln10/20)·10^(u/20)·du/dL` and
`ln10/20 = 0.11513`:

```
k · dGR/dL = 0.11513 · (k·GR + V_d) · (1 − dGR/dL)
```

Divide by `k` and define the **loop gain**

```
γ  :=  0.11513 · ( GR + V_d/k )
```

then

```
dGR/dL = γ / (1 + γ)          and          dL_out/dL_in = 1 / (1 + γ)
```

so the **instantaneous compression ratio is**

```
ratio(GR) = 1 + γ = 1 + 0.11513 · ( GR + V_d/k )
```

Four consequences, and they are the model:

1. **The ratio is a function of the gain reduction, and it never stops rising.** There is no fixed
   slope anywhere on the curve. This is not a soft knee that settles into a straight line; it is a
   curve all the way up. That is what "soft and progressive" means here, and it is a genuinely
   different shape from the Distressor's per-ratio knee tables.
2. **The ratio at threshold is already greater than 1**, by `0.11513 · V_d/k`. The compressor is
   never at 1:1 once it is doing anything at all.
3. **`G` cancelled.** Under this reading the ratio switch's detector gain sets *where* compression
   starts, not how steep it is — which is precisely the behaviour SSL describe when they say the
   ratio control changes the threshold (6.2). The printed ratios must therefore come from the switch
   changing `k` or the diode arrangement as well, through the three poles the drawing does not show
   (3.5, 6.1).
4. **The knee width is not a free parameter.** It falls out of `V_d/k`, which is the same reason the
   DAFx team found that "static gain with a **soft knee** generally performs better", noting that
   this "aligns with the analog design of the SSL G-Bus compressor, which employs a soft knee where
   the **knee width is automatically computed based on the threshold and ratio**" [30].

To get a feel for the numbers: with `V_d = 0.6 V`, a ratio of 2:1 at threshold needs `V_d/k = 8.7 dB`
and so `k ≈ 69 mV/dB`; 4:1 needs `k ≈ 23 mV/dB`; 10:1 needs `k ≈ 7.7 mV/dB`. All three are plausible
control-bus scalings — the XLogic's fader bus runs at 50 mV/dB (8.3) — and their 9 : 3 : 1 spread is
the spread the switch has to produce.

### 5.5 The dominant-channel detector

SSL state the stereo behaviour plainly, and it is not what most compressors do:

> The compressor features a classic 'dominant' sidechain architecture. The left and right channels are
> independently rectified using a true peak full wave detector circuit, and **the dominant, ie. louder
> channel, controls the gain reduction of the overall stereo level** via the user selected time
> constants. [3]

So: rectify each channel separately, take the **maximum**, and apply that one control voltage to both
audio VCAs. Not a sum, not an average, not a mid-side matrix. SSL's own six-channel XLogic makes the
same mechanism visible on the front panel: "The MAX display consists of 6 bi-colour LEDs... When the
compressor is active **the LED corresponding to the channel that is applying the most gain reduction
will turn red**" [8].

Cytomic describe The Glue doing something adjacent but not identical — "This results in a 'soft
maximum', as the smoothed sum of both the left and right detected amplitude is used to control the
stereo level" [42] — which is a sum, softened, rather than a maximum. Worth noting as a place where
a well-regarded emulation deliberately departs.

Nothing in the lab does a dominant-channel detector. The Distressor's three link modes sum control
voltages or sum gain reduction [Distressor 3.7]; a maximum is a different operator and a different
behaviour on material where one channel is consistently hotter.

### 5.6 The sidechain high-pass, on the units that have one

Not on the console compressor. On the 500-series module and every modern SSL version: a six-position
switch, **Off / 30 / 60 / 105 / 125 / 185 Hz** [3] [4] [7]. SSL describe it as "The compressor now
features an HPF (High Pass Filter) in the sidechain, which is controlled by a multi-position switch"
[3], and on the plug-in as a way to "Remove low-frequency content from the compression sidechain to
prevent pumping or breathing... when processing tracks with low-frequency elements like kick or bass"
[10]. No order or slope is published anywhere I could reach; the Smart Research equivalent, supplied
as an external cable rather than built in, is "150 Hz **−6 dB/octave**" [22], i.e. first order, and
that is the only slope figure available for anything in this family.

SSL's THE BUS+ adds a related but different thing, a **LOW THD MODE** which "introduces a special
circuit modification in the side-chain, helping to limit the amount of low frequency distortion
compression can create, especially with fast release times" [11]. That is a description of the
failure mode the high-pass exists to avoid: with a fast release, a low-frequency signal modulates the
gain at its own period, and the result is intermodulation, not pumping. Worth a line in the tribute's
help text.

---

## 6. The RATIO switch, and the threshold that moves with it

### 6.1 What the drawing shows, and what it cannot show

Card 82E26 carries one pole of a four-pole, three-position switch labelled `RATIO SW1 A-D`, with the
pole at pin 36 and throws at pins 37, 38 and 39 [1] [17]. The pole connects to **T7's inverting
summing node**, which is the detector's virtual earth. Around it:

| designator | value | between |
|---|---|---|
| R38 | 1 MΩ | T7 output ↔ summing node (the permanent feedback) |
| R39 | 510 kΩ | T7 output ↔ pin 38 |
| R40 | 270 kΩ | T7 output ↔ pin 39 |
| R41 | 68 kΩ | T7 output ↔ the D7 / R45 node |
| R42 | 1.2 MΩ | pin 37 ↔ −12 V |
| R43 | 1.5 MΩ | pin 38 ↔ −12 V |
| R44 | 3.9 MΩ | pin 39 ↔ −12 V |
| R45 | 620 kΩ | the R41 node ↔ +12 V |
| D7 | 1S44 | between the pin-39 node and the R41/R45 node |

Two things follow immediately, both **derived**.

**Each position sets the detector's gain.** With R37 (20 kΩ) as the input resistor for the direct half
of the full-wave rectifier (3.3), the stage gain is `R_f / 20 kΩ`:

| throw | feedback | R_f | detector gain |
|---|---|---|---|
| pin 37 | R38 alone | 1.000 MΩ | **50.0** |
| pin 38 | R38 ∥ R39 | 337.7 kΩ | **16.9** |
| pin 39 | R38 ∥ R40 | 212.6 kΩ | **10.6** |

**Each position also injects a different DC current from the −12 V rail into the same virtual earth**,
which appears at T7's output as a fixed offset, `12 V · R_f / R_rail`:

| throw | rail resistor | offset at T7's output |
|---|---|---|
| pin 37 | R42 1.2 MΩ | **10.0 V** |
| pin 38 | R43 1.5 MΩ | **2.70 V** |
| pin 39 | R44 3.9 MΩ | **0.654 V** |

That second table is the mechanism behind SSL's statement in 6.2: **the ratio switch moves the
threshold on purpose, and the drawing shows how.** I cannot fix the sign, because D6's orientation is
below the resolution of the scan, so I cannot say whether these offsets raise or lower the threshold —
only that they differ by a factor of fifteen across the switch, which is far too large to be
incidental.

**R41 and D7 are the knee-shaping network.** R41 (68 kΩ) hangs off T7's output permanently, into a
node biased toward +12 V through R45 (620 kΩ), with D7 bridging to the pin-39 node. Once the voltage
across D7 passes a drop, D7 conducts and R41 joins the feedback path, collapsing the detector gain
from 10.6 or 16.9 to something near `68 kΩ ∥ R_f / 20 kΩ` ≈ 2.8. That is a level-dependent detector
gain, which is a knee. **I offer this as an inference and not as a reading**: at 800 × 555 I am
confident of the component values and of which nodes they touch, and less confident of D7's polarity.

**What the drawing cannot give.** Three of SW1's four poles are elsewhere. Since the derivation in 5.4
shows that a change in detector gain alone moves the threshold rather than the slope, the printed
ratios must be produced with help from those poles — most likely by scaling the control voltage on
card 82E27 or by switching the diode network. **I could not close the ratio law from the documents I
have, and I am not going to guess it.** Section 11 therefore treats the per-position law as a
calibration table with `k` as its free constant, marked **estimate**, and section 13 pins it with
tests written against the only published ratio figures that exist.

### 6.2 What SSL say the ratio control does, in their own words

> It should be noted that the knee point of the compressor, set with the THRESHOLD control, **purposely
> changes depending on the setting of the RATIO control**. Decreasing the RATIO setting **lowers the
> effective threshold**, hence maintaining the perceived 'loudness' of the compressed signal. [3]

Three things are being said there. The knee point moves. It moves *deliberately*. And the direction is
that a lower ratio gives a lower threshold, which is the direction that keeps output loudness roughly
constant as you turn the ratio down — because a lower ratio removes less level, so to keep the
perceived result the same you have to start compressing sooner.

Note also that SSL call it a **knee point**, not a threshold, in the same sentence in which they say
the THRESHOLD control sets it. That is consistent with 5.1 and 5.4: there is no threshold in the
comparator sense, only a knee whose position moves.

The DAFx-25 team cite this same SSL document for the same behaviour, twice, once for the knee — "a
soft knee where the knee width is automatically computed based on the threshold and ratio" — and once
for the difficulty it causes their models — "the changing compressor curve in the analog module" [30].

### 6.3 What the ratios really are, and the 20:1 problem

**No SSL document I could reach publishes a measured transfer point for any ratio position.** There is
nothing here like the AMS Neve table that anchors the 33609 dossier [Neve-33609 8.1]. That is the
single biggest weakness of this unit's documentation and section 13 is shaped around it.

What there is:

- **The console has three ratios: 2, 4, 10** [16] [1].
- **Modern SSL hardware has six: 1.5, 2, 3, 4, 5, 10** [3] [4] [8].
- **SSL's own plug-in has three: "2:1, 4:1 and 20:1"** [9].

That last one is a genuine problem, not a typo I am entitled to correct. The *SSL Native V6.5 User
Guide* is SSL's own document for SSL's own emulation of SSL's own hardware, and it says 20:1 where
every SSL hardware panel says 10:1. Two readings are possible and I cannot choose between them from
the documents:

- It is an error in the guide, carried forward through revisions.
- SSL's emulation genuinely implements a steeper top position than the hardware's panel claims —
  which, given 5.4, would not even be surprising: if the top position's real behaviour is a curve
  passing through something like 20:1 at high gain reduction, then "10:1" and "20:1" can both be true
  statements about the same curve read at two different points.

The second reading is worth taking seriously precisely because the derivation in 5.4 predicts exactly
that ambiguity: a unit whose ratio rises with gain reduction has no single ratio, and whoever prints
a number on a panel has chosen an operating point. This is the same finding the Neve dossier made
about the 33609's silkscreen [Neve-33609 6.4], arrived at from the opposite direction.

**Recommendation for the model:** implement the console's three positions as the primary set, offer
the modern six as an option, label the top position `10:1` after the hardware, and record this
discrepancy in the code comment. Do not implement a 20:1.

### 6.4 The knee, and why it cannot be a width parameter

Everything above converges on one design instruction. The knee of this compressor is not a
region-of-N-decibels around a corner. It is the shape produced by a diode drop inside a feedback loop,
it has no corner, and its apparent width changes when the ratio or the threshold changes because
`V_d/k` and the operating gain reduction both move. A model that exposes `knee_width_db` and blends
between two straight lines can be tuned to match this box at one setting and will be wrong at the
next, which is precisely the failure the DAFx team measured in grey-box models that lacked the
feedback term [30].
---

## 7. Attack, release, and the Auto release

Everything in 7.1 to 7.4 is read directly off card 82E27 [2] and the crops are saved as
`z-82e27-att.png`, `z-82e27-rel.png`, `z-82e27-relvals.png` and `z-82e27-r12.png`.

### 7.1 The attack ladder, exactly

Six resistors from the timing bus (pin 41) to six switch contacts, with the panel legend
**`ATTACK mS`** printed beside them and the position numbers engraved under each contact:

| switch pin | panel | designator | value | τ = R × 0.47 µF |
|---|---|---|---|---|
| 43 | `.1` | R1 | **820 Ω** | 385 µs |
| 44 | `.3` | R2 | **2.7 kΩ** | 1.27 ms |
| 45 | `1` | R3 | **8.2 kΩ** | 3.85 ms |
| 46 | `3` | R4 | **27 kΩ** | 12.7 ms |
| 47 | `10` | R5 | **82 kΩ** | 38.5 ms |
| 48 | `30` | R6 | **270 kΩ** | 127 ms |

The resistor sequence is 820 / 2K7 / 8K2 / 27K / 82K / 270K, which is the E24 preferred-value
approximation to a decade-and-a-half ladder in half-decade steps. The panel prints the *ideal*
sequence 0.1 / 0.3 / 1 / 3 / 10 / 30.

### 7.2 The release ladder, exactly

Each release position selects **both** a capacitor and its own discharge resistor. All four fixed
positions use a 0.47 µF tantalum:

| switch pin | panel | cap | resistor | τ = R × C |
|---|---|---|---|---|
| 5 | `.1` | C6, 0.47 µF | R12, **180 kΩ** | **84.6 ms** |
| 6 | `.3` | C5, 0.47 µF | R11, **270 kΩ** | **127 ms** |
| 7 | `.6` | C4, 0.47 µF | R10, **560 kΩ** | **263 ms** |
| 8 | `1.2` | C3, 0.47 µF | R9, **1.2 MΩ** | **564 ms** |
| 9 | `A` | C1 0.47 µF and C2 6.8 µF | R7 **91 kΩ** and R8 **750 kΩ** | see 7.4 |

The common node (pin 11) feeds **T1, an LF351N**, with **R13 3.3 MΩ** and **R14 470 kΩ** around it
[2]. If R13 shunts the timing node to ground it is in parallel with every release resistor and
shortens the times by 5 % at the fast end and 27 % at the slow end; if it is the lower leg of a
divider after a series R14, it does not. **I could not settle which from the scan** and the values in
the table above are the bare R × C, without R13.

### 7.3 The panel labels are not the RC constants, and here is why

**Attack.** Divide the time constant by the panel number:

| panel | τ | τ / panel |
|---|---|---|
| 0.1 ms | 385 µs | 3.85 |
| 0.3 ms | 1.27 ms | 4.23 |
| 1 ms | 3.85 ms | 3.85 |
| 3 ms | 12.7 ms | 4.23 |
| 10 ms | 38.5 ms | 3.85 |
| 30 ms | 127 ms | 4.23 |

The alternation between 3.85 and 4.23 is entirely explained by the preferred-value approximation of
7.1 and is not a property of the circuit. **The panel figure is one quarter of the RC time constant,
at every position, to within 6 %.**

A factor of four is not a coincidence, and 5.4 supplies it. In a feedback loop the closed-loop
settling time is the open-loop time constant divided by one plus the loop gain, and 5.4's loop gain
is `γ = ratio − 1`. At the **4:1** position `γ = 3`, so `τ_closed = τ_open / 4`. **Derived:** the
numbers SSL printed on the attack switch are the *effective* attack times with the loop closed at
about 4:1, and the resistors implement open-loop constants four times longer.

That derivation makes a prediction I have no measurement to check: **attack time should shorten as
the ratio rises**, roughly by `(1+γ)`, so the `1` position would behave like 1.9 ms at 2:1 and like
0.4 ms at 10:1. If that is wrong, the alternative is simply that SSL printed convenient round numbers
with no RC meaning, which is also possible and would be a duller answer. Section 13 turns it into a
test that asserts the panel figure at 4:1, which is true under either reading.

**Release.** The same arithmetic, the other way up:

| panel | τ | panel / τ |
|---|---|---|
| 0.1 s | 84.6 ms | **1.18** |
| 0.3 s | 127 ms | **2.36** |
| 0.6 s | 263 ms | **2.28** |
| 1.2 s | 564 ms | **2.13** |

Three of the four cluster at about 2.2 to 2.4, which is `ln 9 = 2.20`, the time to recover 90 % of the
gain reduction — a completely ordinary way to quote a release time. **The `.1` position is the
outlier at 1.18 and I cannot explain it.** R12 reads unambiguously as 180 kΩ at 16× magnification
(`z-82e27-r12.png`); 90 kΩ would fit the pattern and it is not what is drawn. I record this as an
**unresolved discrepancy** rather than adjusting the value to taste. Note, without claiming it means
anything, that under the 2.2 convention 180 kΩ would be labelled **0.2 s**, and 0.2 s is a value that
appears on every later SSL release switch (1.3).

**Why attack and release scale differently.** Because the loop is only closed on the way in. During
attack, D6 conducts, the detector drives the capacitor, and the falling sidechain-VCA gain fights the
charging — so the loop divides the time constant. During release, D6 is reverse-biased, the detector
is disconnected, and the capacitor simply discharges through its resistor with nothing opposing it —
so the release time is the bare RC. **The asymmetry between the two panel conventions is a
consequence of a single diode**, which is a satisfying thing for a model to reproduce rather than
to hard-code.

### 7.4 The Auto release is a two-section ladder, and the best-known description of it has the pairs the wrong way round

The Auto position, pin 9, replaces the single RC with two RC sections in series from the timing node
to ground [2]:

```
timing node ──┬── C1 0.47 µF ──┬── C2 6.8 µF ── ground
              └── R7  91 kΩ  ──┴── R8 750 kΩ ── ground
                 section 1        section 2
```

Because the buffer draws no current, no current flows through the string once the detector stops
driving it, so the two sections decay **independently**:

```
τ₁ = R7·C1 = 91 kΩ × 0.47 µF = 42.8 ms
τ₂ = R8·C2 = 750 kΩ × 6.8 µF = 5.10 s

V_cv(t) = V₁(0)·e^(−t/τ₁) + V₂(0)·e^(−t/τ₂)
```

**And the split between them is what makes it program-dependent.** While charging, the same current
flows through both sections, so both capacitors receive the same charge, and the voltage each takes
is inversely proportional to its capacitance:

```
ΔV₁ / ΔV₂ = C2 / C1 = 6.8 / 0.47 = 14.5
```

A short peak puts fourteen and a half times as much voltage on the fast section as on the slow one,
so it releases with `τ₁ ≈ 43 ms`. Sustained compression lets section 2 charge toward its own
equilibrium, where the split is set by the resistors instead — `V₁/V₂ = 91/750 = 0.121`, so **89 % of
the control voltage sits on the slow section** — and it releases with `τ₂ ≈ 5.1 s`. That is exactly
what the Auto release is famous for, derived from four components.

Erland describes the behaviour correctly — "This will give short time constants for short programme
peaks, but if compression is going on for a longer time, the slow time constant will set in. Right
after the book" — but the parenthesis in the same sentence pairs the components the other way round:
"combining two release time constants (**91k+6u8 and 750k+u47**)" [16]. Those pairings give 619 ms and
353 ms, two nearly equal constants that would produce **no program dependence at all** and would
contradict his own description. The drawing shows C1 (0.47 µF) across R7 (91 kΩ) and C2 (6.8 µF)
across R8 (750 kΩ), and the physics agrees with the drawing. **This is a slip in the single most-cited
document about this compressor, and anyone building from that sentence will build the wrong Auto
release.**

SSL's own descriptions of Auto are qualitative and consistent with 43 ms / 5.1 s: "release time is
dependant upon duration of signal peak" [9], and "an automatic release mode that optimises the
release based on the content of the signal" [10].

### 7.5 Which set of values the model should use

Use the **console** values from card 82E27: attack 0.1 / 0.3 / 1 / 3 / 10 / 30 ms, release 0.1 / 0.3 /
0.6 / 1.2 s and Auto. SSL's own plug-in uses those [9] even though SSL's own current hardware does
not [3], which is as close to an endorsement as this question is going to get. Offer the modern
0.1 / 0.2 / 0.4 / 0.8 / 1.6 ladder as an alternative behind a switch if the lab wants the 500-series
faceplate to be literally true, and label it as the later revision.

---

## 8. Published measurements, with their tolerances and their conditions

### 8.1 For the console compressor, SSL publish nothing

No attack time, no release time, no ratio calibration, no distortion figure, no noise figure, no
frequency response, no measurement procedure. The survey said so [55] and I confirm it: I read SSL's
500-series module user guide [3], the module's product page [7], the recall sheet [4], the Native
plug-in guide [9], the Bus Compressor 2 product page [10] and THE BUS+ page [11], and between them
they publish exactly two quantitative facts about the compressor — the switch values, and the
sidechain filter frequencies.

That is the difference between this dossier and the Neve one, and it should be stated rather than
papered over. The Neve dossier's calibration section rests on a factory table with per-position
tolerances and a stated measurement procedure [Neve-33609 8.1]. **There is no equivalent for the SSL,
and section 13 is written around its absence.**

### 8.2 The XLogic Multichannel Compressor specification, and what it does and does not describe

SSL's *XLogic Multichannel Compressor Owner's Manual* [8] carries a full performance specification.
The unit is a six-channel 2U box that "utilises classic SSL Centre Section compressor design elements
within a **SuperAnalogue design topology**", and "The compressor design is very similar to that found
in the **XL 9000 console centre section**", "electronically balanced and DC coupled throughout" [8].

**So these figures describe a modern SuperAnalogue circuit that shares a control law with the G, not
the G's own audio path.** Its noise and distortion are those of a 2004 design and are certainly better
than a 1980 card's. I list them because they are the only SSL-published numbers of their kind, and
section 13 uses only the ones that are about behaviour rather than about audio quality.

Conditions: source impedance 50 Ω unless stated; all measurements RMS through a 22 Hz to 22 kHz filter
unless stated [8].

| quantity | published value | conditions |
|---|---|---|
| Noise | **< −99 dBu** | input terminated 50 Ω, compressor switched in, 0 dB gain |
| Headroom | **> +26 dBu** output | *defined as* the output level at which THD exceeds 1 %, 20 Hz–20 kHz, 0 dB gain |
| Dynamic range | **> 125 dB** | |
| Frequency response | **±0.05 dB, 20 Hz–20 kHz** | reference 1 kHz, any gain setting |
| Frequency response | **+0.2 / −0.5 dB, 10 Hz–96 kHz** | |
| Frequency response | **+0.2 / −2 dB, 10 Hz–200 kHz** | |
| CMRR | **> 50 dB, 20 Hz–10 kHz** | input +20 dBu, ground referenced |
| THD + N | **< 0.006 %, typically 0.003 %** | 0 dB gain, +4 dBu in, master fader in, **compressor out**, 10 Hz–20 kHz in an 80 kHz filter |
| THD + N | **< 0.01 %, typically < 0.006 %** | as above at +20 dBu in |
| THD + N, compressor in | *not specified* | "THD with the compressor switched in is **dependent on attack and release times and signal content**" |
| Crosstalk | **< −105 dB at 50 Hz**, **< −90 dB at 1 kHz** | +20 dBu into another channel, all channels 0 dB |
| Input impedance | **> 10 kΩ** | |
| Output impedance | **< 40 Ω** | |

The refusal to quote THD with the compressor in is itself informative: SSL are saying that in this
design the distortion **is** the compression, which is 4.3's point restated by the manufacturer.

### 8.3 The XLogic calibration figures, which are the useful ones

Appendix D of the same manual is a factory calibration procedure with target voltages and tolerances
[8]. These are the only published numbers anywhere describing what an SSL compressor's control
voltages actually are, and three of them earn their place in section 11.

| adjustment | target | what it tells us |
|---|---|---|
| Master Fader, measured at TP5 | **−2.00 V ±1 mV, "which equates to 40 dB of attenuation"** | the control bus runs at **50 mV/dB** — about eight times the VCA's own 6.1 mV/dB (4.2), exactly the "ten times higher" scaling THAT recommend to keep control-path noise out of the signal [26] |
| Output level check, same setting | **−20 dBu ±0.1 dB with +20 dBu in, "which equates to 40 dB of attenuation"** | confirms the 50 mV/dB figure at the audio output rather than at a test point |
| Side chain trim at indent | **−0.45 V ±10 mV** | the sidechain gain trims sit at a defined offset, not at zero |
| Side chain gain | **+2.00 V ±1 mV** | |
| Control voltage breakthrough null | **0 V ±10 mV**, and must stay within 10 mV as the threshold pot is swept end to end | SSL trim the threshold pot's feedthrough into the audio path to a tenth of a per cent of the 2 V full-scale bus |
| VCA distortion null | **minimum THD, < 0.007 %**, at 1 kHz, +24 dBu, per channel and per polarity | the factory target for the `DISTORTION NULL` trimmer of 3.2 |

And one operational statement that is a measurement in disguise, and is SSL confirming the derivation
of 5.4 in their own words:

> When fully clockwise they [the S-CHAIN TRIM controls] **increase the side chain level by 10dB —
> effectively reducing the threshold on that channel by 10dB**. [8]

That is SSL saying, explicitly, that **N decibels of extra sidechain gain is N decibels of threshold
reduction**. It is the same identity 5.4 derived from the loop equation, and it is why the threshold
control in this box is a sidechain gain and not a comparator reference.

Two more from the same manual: the LFE sidechain "has **10dB more gain** than the other channels" to
compensate for the Dolby LFE convention, with an `LFE 0DB` switch to remove it; and the LFE low-pass
filter is **120 Hz** [8]. The MAX signal-present LEDs light "at approximately **−36 dBu**" [8].

### 8.4 The clone's published figures

Erland publishes a specification for the finished GSSL [16]. It is a clone's specification, measured
by its designer on his own build, and it is cited as exactly that:

| quantity | value |
|---|---|
| Ratio | 2:1, 4:1, 10:1 |
| Attack | 0.1, 0.3, 1, 3, 10, 30 ms |
| Release | 0.1, 0.3, 0.6, 1.2 s, 2.4 (Auto) |
| Make-up gain | 0 to 20 dB |
| Threshold | −20 to +20 dBm |
| Input | 50 kΩ electronically balanced |
| Output | 100 Ω electronically balanced |
| Frequency response | less than 15 Hz to more than 35 kHz within 3 dB |
| Noise | less than −80 dB |
| **Distortion** | **about −75 dB unadjusted. Mostly 2nd harmonic** |

−75 dB is **0.018 %**, which sits between the THAT part's untrimmed 0.005–0.03 % (4.2) and the
XLogic's < 0.007 % factory target (8.3), and is the only distortion figure available for anything
built on the console's own topology.

### 8.5 The Smart Research C1 and C2, which are the closest thing to a factory descendant

Alan Smart's provenance is in 1.3. His published specifications [22]:

| quantity | **C1** | **C2** |
|---|---|---|
| Ratios | 1.5, 2, 3, 4, 10 | 1.5, 2, 3, 4, 10, **Limit** |
| Attack, ms | 0.1, 0.3, 1, 3, 10, 30 | **0**, 0.1, 0.3, 1, 3, 10, 30 |
| Release, s | 0.1, 0.3, 0.6, 1.2, 2.4 | 0.1, 0.3, 0.6, 1.2, 2.4 |
| Noise floor | below **−92 dBm** flat 20 Hz–20 kHz | below **−104 dBm** flat 20 Hz–20 kHz |
| Distortion | below **0.025 %** (1 kHz / +4, THD 20 Hz–20 kHz) | below **0.005 %** |
| Frequency response | within **½ dB, 20 Hz–20 kHz** | within **½ dB, 20 Hz–100 kHz** |
| Main input | balanced, 10 kΩ per leg | balanced, 1.8 kΩ per leg |
| Output | unbalanced, above **+21 dBm** | discrete transistor, above **+26 dBm** into 600 Ω |
| **Threshold** | **−15 to +15 dBm** | **−20 to +20 dBm** |
| Make-up gain | 0 to +15 dBm | 0 to +20 dBm |
| Sidechain input | none | balanced, 10 kΩ per leg |
| Sidechain HPF | optional cable, 150 Hz **−6 dB/octave** | same |

**The C1's threshold range is −15 to +15 dBm**, and card 82E27's threshold pot has its ends annotated
**+15 dB** and **−15 dB** [2]. Two independent documents, one an SSL console drawing and one a
specification written by a former SSL service engineer, agree on a range that no modern SSL unit
uses. That is a real cross-check and it is the strongest evidence I have that ±15 dB is the original
figure and ±20 dB is a later change (1.2).

The C2's distortion of 0.005 % against the C1's 0.025 % is a five-fold improvement across one
generation of the same design, which puts a useful bracket around how much of this family's
distortion is the VCA and how much is the amplifiers around it.

### 8.6 What has never been measured

- **No independent laboratory measurement of the G Series console compressor's frequency response,
  distortion, noise or ballistics.** Nobody has published one and I could not find one.
- **No published ratio calibration**, of the kind AMS Neve print for the 33609.
- **No published knee curve**, though SSL confirm the knee moves (6.2).
- **No null test by any plug-in vendor**, which is what the survey predicted [55].

**But the category-wide gap the survey identified is no longer total**, because the DAFx-25 dataset
paper publishes error measurements of four commercial emulations against a real unit. That is section
10.2 and it is the most important thing in this file after the schematics.
---

## 9. How it is described as sounding, and what an emulation must get right

### 9.1 The published descriptions

**SSL, on what it is for:**

> It is a simple unit with a simple purpose; it makes complete mixes sound bigger, with more power,
> punch and drive. It brings cohesion and strength to your mix without compromising clarity. [10]
>
> The Stereo Bus Compressor brings cohesion and strength to your mix unlike any other processor. To
> this day it remains a key element of the SSL sound and has become not just a tool for production but
> a part of top engineers and producers creative process. [7]

**Alan Smart, on the mechanism** — this is the single most technically useful description I found,
because it names a time window and a cause [22]:

> The characteristic sound of this unit arises mostly from the **fast response at the onset of
> compression**, which when **delayed to occur around the mid range attack settings (after 0.3 to 3
> milliseconds)** results in a **window through which transients can still pass**. When used across a
> mix, or with any dynamic programme, this has the effect of adding 'punch', as the sidechain
> 'breathes' around these transients, while still controlling overall levels.

That describes something quite specific: the loop is fast *once it starts*, and the attack switch
delays the start rather than slowing the whole response. A transient shorter than the selected attack
gets through intact and then the gain slams. It also identifies **0.3 to 3 ms** as the useful band,
which is exactly the band the DAFx team's professional mastering engineers chose when asked for
"widely used parameter combinations" — their attack grid is **0.1, 0.3, 1, 3 ms**, four of the six
positions, all at the fast end [30].

**Cytomic, on what makes it smooth** [42]:

> Forming the core of the smooth behaviour of The Glue is an analog modelled **dual diode envelope
> follower**, which is solved using **optimised nodal analysis**. ... **Diodes are used as a way to
> smoothly switch between not attacking, and attacking, based on the level of the input signal level
> across the diodes.**

Independent confirmation, from the most technically candid vendor in the category, that the diodes in
the detector path are where the character lives — which is 5.1's argument arrived at by someone who
built a working model.

**Sound On Sound, on where it sits among its rivals** [49]:

> the API 2500 and the SSL G-series console master compressor. Both of these feature fairly clean,
> VCA-based gain reduction and are good at **increasing average levels without trampling on the punch
> of transients or robbing too much low end**. However, the SSL also has a reputation for adding a
> dose of **desirable mid-range crunch when more heavily provoked**

The same article records Tom Lord-Alge reaching for it specifically as a transient tool — "If the
beginnings of the words then have too much attack, I'll put the vocals through an SSL compressor with
a **really fast attack**, to take off or smooth out the extra attack that the LA3A adds" — and Jason
Goldstein on why the console itself is part of the reputation: "many guys still like to mix on the
SSL 4000 — those consoles are always **just shy of distorting**. It adds to the overall aggression of
the mix" [49]. Sound On Sound also calls Smart Research's C2 an "emulation" of the SSL in as many
words, which is a third party agreeing with 1.3's reading of the relationship [49].

**SSL, on the failure mode** [11]: low-frequency distortion "compression can create, especially with
fast release times", which the modern LOW THD MODE exists to suppress. The classic box does not
suppress it. That is part of the sound, not a defect to design out.

### 9.2 What an emulation must get right, in priority order

1. **The feedback loop.** Not because it is elegant but because everything else follows from it: the
   ratio law (5.4), the four-times-shorter attack (7.3), and the "changing compressor curve" that
   defeated grey-box models without it (5.3) [30]. Get this wrong and no amount of curve fitting
   fixes it.
2. **The soft, never-straightening knee.** `ratio = 1 + 0.11513·(GR + V_d/k)` (5.4). A hard knee with
   a blend region is the wrong shape, and the DAFx grey-box results say a soft knee whose width comes
   from the threshold and ratio is what fits [30].
3. **The Auto release as two independent exponentials, 43 ms and 5.1 s, sharing charge in the ratio
   14.5 : 1 on transients and 1 : 7.4 at equilibrium** (7.4). This is the unit's signature and it is
   fully determined by four component values.
4. **The transient window.** Fast onset, delayed by the attack switch; a 3 ms attack must let a 1 ms
   transient through substantially intact (9.1) [22].
5. **The dominant-channel detector, a maximum and not a sum** (5.5) [3].
6. **The moving threshold**: turning the ratio down must lower the effective threshold (6.2) [3].
7. **Second-harmonic distortion from the gain cell that rises with drive**, around 0.005 % to 0.02 %
   at moderate levels, and no separate saturator (4.3, 8.4) [16] [24].
8. **Make-up gain that stays applied when the sidechain is switched out** (2.1) [3] [16].

### 9.3 What I would not bother with

- **VCA temperature drift and the symmetry-null-versus-frequency effect** (4.2). Real, published,
  and inaudible except as the explanation for why two units differ. Put it in the help text, not in
  the DSP.
- **A model of the E175 JFET buffer** in front of the sidechain VCA (3.3). It is a buffer.
- **The quad fader and autofade machinery** on card 82E27. It shares the VCAs and it is why the
  architecture is what it is (3.1), but it is console plumbing and nothing about it is audible in an
  outboard box.
- **Transformers.** There are none, anywhere, in any version.
- **A separate distortion stage.** There isn't one, and adding one would be the exact mistake section
  12 warns about.

---

## 10. Existing emulations, and the one published comparison

### 10.1 SSL's own: Bus Compressor 2

SSL make and sell their own model, which is unusual and is why the survey ranked this unit third
[55]. Their claim:

> Engineered by SSL, the SSL Bus Compressor 2 plug-in is an **accurate emulation** of the legendary
> G-Series Bus Compressor, featuring the same compression characteristics and sound quality as the
> original unit. [10]

Its published feature list [10]: "Attack, Release and Compressor Ratios", "Renowned 'Auto' release",
"**2X & 4X Oversampling**", "External side-chain and side-chain HPF", "Mix control", and control from
the SSL 360° Plug-in Mixer and the UC1 hardware. The *SSL Native V6.5 User Guide* [9] gives its
parameter set: attack 0.1 / 0.3 / 1 / 3 / 10 / 30 ms; release 0.1 / 0.3 / 0.6 / 1.2 s or Auto with
"release time is dependant upon duration of signal peak"; **ratio 2:1, 4:1 and 20:1** (6.3);
threshold continuously variable **−20 to +20 dB**; make-up continuously variable **−5 to +15 dB**; a
sidechain high-pass; and a Mix control with a Mix Lock that "Excludes the MIX control from the preset
management system".

Two things are worth extracting for the lab. First, **make-up runs −5 to +15 dB in SSL's own model**,
not 0 to +20 as the clone's specification says [16] — a 20 dB span shifted down by 5 dB, presumably
because a plug-in's unity is not a console bus's unity. Second, SSL themselves ship oversampling as a
user choice at 2× and 4×, which is a manufacturer's admission that the gain cell aliases.

**What SSL's emulation documents about the hardware**, which is what the survey asked for: it fixes
the canonical parameter set as the *console's* values rather than the current hardware's (1.3, 7.5);
it describes Auto in terms of peak duration rather than a fixed pair of times (7.4); it confirms the
±20 dB threshold; and its ratio list raises the 20:1 question of 6.3. It does not publish a
schematic, a measurement, a block diagram or a null test.

### 10.2 The DAFx-2025 benchmark, which is the closest thing to a null test this category has

Yicheng Gu, Runsong Zhang, Lauri Juvela and Zhizheng Wu, *"Solid State Bus-Comp: A Large-Scale and
Diverse Dataset for Dynamic Range Compressor Virtual Analog Modeling"*, DAFx-25, Ancona, 2–5 September
2025, pages 55–60, open access under CC BY 4.0 [30].

**What they did.** Recorded 175 unmastered songs from the Cambridge Multitrack Library through a real
**SSL 500 G-Bus** module in 220 parameter combinations, giving **2528 hours** of paired input/output
audio at **44.1 kHz** [30]. The rig: Reaper as the DAW, an **RME Fireface UFX+** as the interface, a
ReaScript driving ReaInsert to send and receive automatically. Level matching: "we normalized all
songs to **−12 dB** and applied a **5 dB input boost and a 5 dB output attenuation**" [30].

**The parameter grid**, chosen "after consulting six professional mastering engineers" [30]:

| parameter | values recorded |
|---|---|
| threshold | −28, −24, −20, −16 |
| attack | 0.1, 0.3, 1, 3 ms |
| release | 0.1, 0.4, 0.8 s, **auto** |
| ratio | 2, 4, 10 |

144 combinations, plus 76 more chosen at random "as supplementary edge cases". The full ranges given
in their comparison table are threshold **[−40, 0] dB**, attack **[0.1, 30] ms**, release
**[0.1, 1.6] s**, ratio **1:[1.5, 10]** [30].

**The result that matters.** Their Table 4 benchmarks four commercial plug-ins against the recorded
hardware, reporting L1 error in the time domain and multi-resolution STFT error in the frequency
domain, on both the seen and the unseen parameter distributions. Lower is better:

| plug-in | L1, seen | L1, unseen | M-STFT, seen | M-STFT, unseen |
|---|---|---|---|---|
| Solid State Logic (Bus Compressor 2) | 0.0322 | 0.0175 | 0.4489 | 0.2943 |
| Softube (Bus Processor) | 0.0448 | 0.0237 | 0.7069 | 0.4546 |
| Overloud (Comp G) | 0.0326 | 0.0176 | 0.4738 | 0.3253 |
| **PSPaudioware (BussPressor)** | **0.0269** | **0.0145** | **0.3047** | **0.2184** |

**PSPaudioware's BussPressor is closer to the hardware than SSL's own emulation on every one of the
four metrics**, and Softube's is furthest away on all four by a wide margin. I have no stake in that
result and neither did the authors, who are an academic group at Aalto and CUHK-Shenzhen benchmarking
against a dataset they built themselves and released.

**Caveats, stated plainly.** These are L1 and multi-resolution STFT errors, not a null test; they are
measured on one physical module, not a population; the four plug-ins were run at whatever settings
the authors matched to the hardware's panel and the paper does not describe that matching in detail;
and none of the four is being judged on the thing people actually buy it for. But it is a published,
peer-reviewed, quantitative comparison of emulations against measured hardware, and **nothing like it
exists for any other unit in this lab.** It is the reason this dossier can have a test plan with real
targets in it at all (section 13).

**Three more findings from the same paper** that bear directly on section 11:

- Grey-box models did best with **"static gain with a soft knee"**, which the authors tie to "the
  analog design of the SSL G-Bus compressor, which employs a soft knee where the knee width is
  automatically computed based on the threshold and ratio" [30].
- Among level detectors, **"the switching one-pole filter achieves the best overall performance,
  followed by the standard one-pole filter"**, while an RNN-modulated one-pole did worse — the
  authors attributing this to "the relatively simple design of the VCA compressor's level detection
  circuit, which is different from the LA-2A that has strong non-linear distortion due to its optical
  components" [30]. A switching one-pole with separate attack and release coefficients is precisely
  what card 82E27 is (7.1, 7.2), so this is a measurement agreeing with a schematic.
- Adding "a simple phase inversion module would damage the model performance **since there are no
  phasers in the actual analog module**" [30] — a small, useful negative result: nothing in this box
  moves phase.

### 10.3 The rest of the field

| plug-in | claim | what it publishes |
|---|---|---|
| **Universal Audio**, SSL 4000 G Bus Compressor Collection | "an expert end-to-end circuit emulation" of "the iconic SSL G Bus Compressor", "**Fully endorsed by Solid State Logic**"; adds an internal sidechain filter, a Mix control and a Headroom control that the hardware does not have; includes an Auto Fade of up to 60 seconds | no measurements, no schematic, no null test [40] |
| **Waves**, SSL G-Master Buss Compressor | "'Glue' tracks into a smooth, cohesive mix"; the product page's own copy is a single line and the survey's description of it as "developed under license from Solid State Logic" [55] is not on the page I fetched | nothing [41] |
| **Cytomic**, The Glue | "based on the classic 80's British big console buss compressor"; "an analog modelled **dual diode envelope follower**, which is solved using **optimised nodal analysis**"; "the same high quality algorithms used in circuit simulators, but optimised to run fast"; a "soft maximum" stereo detector from the smoothed sum; a **Range** knob controlling "the maximum dynamic swing in changes of compression" to "emulate older units"; minimum-, intermediate- and linear-phase oversampling options | no measurements; by far the most specific technical description of any vendor [42] |
| **Softube**, Bus Processor | "a luxurious-sounding bus compressor plugin that provides musical glue for any bus, making your mix sound like a record. It's a glue compressor with **ultramodern algorithms**"; **the page never names SSL or the G Series at all** | nothing; worst of the four measured (10.2), and see the note below [43] |
| **Overloud**, Comp G | "The Revolutionary Simulation of the British VCA Master Bus Compressor", "Hyper-realistic simulation of the original unit thanks to the 4th generation dsp technology"; does not name the hardware | nothing [44] |
| **PSPaudioware**, BussPressor | "combines the sound of classic VCA compression with wide tuning capabilities", with a sidechain high-pass and parallel compression | nothing on the page; **best of the four measured** (10.2) [45] |
| **Brainworx**, bx_townhouse | named by the survey [55]; not re-checked here | — |

**One fairness note about the benchmark table.** Softube came last on all four metrics, and Softube's
own page for Bus Processor **never claims to model the SSL**: it describes "a glue compressor with
ultramodern algorithms, compression, saturation, sidechain, spatialization" [43]. Measuring it against
an SSL and reporting that it is furthest away is a fair measurement of a thing it did not set out to
be. The three that do claim an emulation are SSL's own, Overloud's and PSP's, and among those SSL's
own is second.

Cytomic's Range control is worth a moment because it is the only vendor feature that maps onto a
circuit fact: capping how far the control voltage can swing is what an ageing electrolytic in the
timing path or a weaker rail would do, and "emulate older units" is a fair description of the result.
If the lab wants one spoof extra beyond mix and drive, that is the one with a story behind it.

### 10.4 What none of them does

None publishes a schematic. None publishes a measurement of the hardware. None publishes a null test.
None of the vendor pages states whether its model is feedforward or feedback, which is the single
question that decides what the thing is (5.3). And none of them draws the transfer curve the way 5.4
says it actually behaves — a ratio that keeps rising with gain reduction — because every plug-in
interface in the list presents ratio as a switch with a fixed number on it.
---

## 11. Recommended DSP design

Target: 44.1 to 192 kHz, real time, one `Processor` hosting the model behind the lab's existing
`model` switch, reusing `dsp::vca`'s dB-domain scaffolding and shared extras where they genuinely
fit — which is less than it looks, and section 12 is the argument.

### 11.1 The shape of the engine

```
                        ┌──────────────── GR[n−1] (dB) ────────────────┐
                        ↓                                              │
 in_L ─┬─► × 10^((M − GR)/20) ──────────────────────────────► out_L    │
       │                                                               │
       └─► × 10^((T − GR)/20) ─► HPF ─► |·| ─┐                         │
                                              ├─► max ─► × G_ratio     │
 in_R ─┬─► × 10^((T − GR)/20) ─► HPF ─► |·| ─┘            │            │
       │                                                  ↓            │
       └─► × 10^((M − GR)/20) ──────────────► out_R    diode D6        │
                                                          ↓            │
                                            attack R ─► RC network ────┘
                                                          ↓
                                                    GR = V/k  (dB)
                                                          ↓
                                                       meter
```

Five properties of that diagram are the model:

- **The audio path is one multiply.** No filters, no saturator, no transformer, no oversampled stage
  in the signal path itself (3.2). The only nonlinearity is inside the multiply (11.3).
- **The detector reads `input − GR + threshold`**, which closes the loop (5.3). Use `GR[n−1]`; the
  one-sample delay is the standard resolution of an instantaneous feedback loop and it costs nothing
  at any sample rate this runs at.
- **`max`, not sum** (5.5).
- **The RC network is simulated, not approximated by two coefficients.** The attack resistor, the
  release resistor and the capacitor are three components, and the interaction between them (11.5)
  is a real behaviour that a two-coefficient envelope follower cannot produce.
- **The diode is the threshold** (5.1). There is no comparator and no knee-width parameter anywhere.

Latency is zero. The audio path never touches the detector.

### 11.2 Parameter table

All parameters prefixed `ssl_`. Stepped controls are integer indices because every control that is
stepped on the hardware is a rotary switch with detents.

| parameter | type | range / values | default | source of the range |
|---|---|---|---|---|
| `ssl_revision` | enum | `console`, `module` | `console` | 1.3; picks which switch legends and values are live |
| `ssl_bypass` | bool | — | false | the plug-in's own sample-exact bypass, **not** the hardware's IN |
| `ssl_in` | bool | — | true | the hardware IN switch: false removes the **sidechain only**, leaving the VCA and make-up in circuit (2.1) [3] [16] |
| `ssl_threshold` | float | **−20 to +20 dB**, 0.1 dB steps | 0 | [3] [4] [9]; the E card annotates ±15 dB and the Smart C1 specifies ±15 dBm (8.5), offered as a `console` variant |
| `ssl_makeup` | float | **−5 to +15 dB** | 0 | SSL's own plug-in [9]; the clone says 0 to +20 [16], offered as the `console` variant |
| `ssl_attack` | index 0–5 | 0.1, 0.3, 1, 3, 10, 30 ms | index 2 (1 ms) | card 82E27 [2], and identical on every unit ever made (1.3) |
| `ssl_release` | index 0–4 (`console`) | 0.1, 0.3, 0.6, 1.2 s, **Auto** | index 4 (Auto) | card 82E27 [2] and SSL's own plug-in [9] |
| `ssl_release` | index 0–5 (`module`) | 0.1, 0.2, 0.4, 0.8, 1.6 s, **Auto** | index 5 (Auto) | [3] [4] [8] |
| `ssl_ratio` | index 0–2 (`console`) | 2:1, 4:1, 10:1 | index 1 (4:1) | [16] [1] |
| `ssl_ratio` | index 0–5 (`module`) | 1.5, 2, 3, 4, 5, 10 | index 3 (4:1) | [3] [4] [8] |
| `ssl_hpf` | index 0–5 | Off, 30, 60, 105, 125, 185 Hz | index 0 (Off) | [3] [4] [7]; greyed out on `console`, which has none (5.6) |
| `ssl_link` | enum | `dominant`, `sum`, `dual`, `mid-side` | `dominant` | `dominant` is the hardware [3]; the other three are SSL's own THE BUS+ modes [11] and are spoof extras here |
| **spoof extras, not on the classic hardware** | | | | |
| `ssl_mix` | float | 0 to 1 | 1 | SSL's own plug-in has one [10]; so do UAD [40] and PSP [45] |
| `ssl_drive` | float | 0 to 1, scales the VCA's second-harmonic term | 0 | after THE BUS+'s 4K MODE, which "introduces a variable amount of harmonic distortion via the VCA" [11] |
| `ssl_range` | float | 0 to 20 dB cap on the control voltage | 20 (no cap) | after Cytomic's Range, which "controls the maximum dynamic swing" to "emulate older units" [42] |
| `ssl_oversample` | enum | 1×, 2×, 4× | 2× | SSL ship 2× and 4× on their own model [10] |
| `ssl_sc_ext` | bool | — | false | external sidechain; on the C2 [22], SSL's plug-in [10] and PSP's [45] |

**Not offered, deliberately:** no `knee` control (6.4), no separate `attack shape`, no `feedback /
feedforward` switch (5.3 settles it; THE BUS+'s F/B mode is a different circuit), and no ratio above
10:1 (6.3).

### 11.3 The gain cell

The VCA is exponential in dB by construction (4.2), so the "model" of the gain cell is a `powf`, and
the only thing worth modelling about it is its imperfection.

```
g_db  = M − GR                                     // make-up minus gain reduction, both dB
gain  = 10^(g_db / 20)
y     = x · gain  +  d2 · (x · gain)²  · sgn_hint  // second-harmonic term
```

with the second-harmonic coefficient set so that, at 0 dBV and unity gain, THD is **0.005 %** (the
THAT 2180A typical, which is the grade a fader-quality module would use) rising to **0.02 %** at
+10 dBV and −15 dB gain, both from the datasheet's own table (4.2) [24]. `ssl_drive` scales `d2`
upward toward the 0.05 % C grade and beyond, which is what THE BUS+'s 4K MODE does [11].

**Use an asymmetric, even-order term and nothing else.** Erland measured the real thing: "it tends to
be **almost exclusively second harmonic** as far as I can measure... and hear" [16]. A symmetric
soft-clip would be the wrong harmonic family, and a third-harmonic term would make it sound like the
Distressor's Dist 3, which is a different box doing a different thing on purpose (12.3).

Antialias the squared term with the antiderivative method already in `dsp::pre::adaa`, or oversample.
At `ssl_oversample` = 2× the second harmonic of a 20 kHz tone lands at 40 kHz and folds to 8.1 kHz at
44.1 kHz without it, which is audible; SSL offering oversampling on their own model (10.1) is a
manufacturer conceding the same point.

### 11.4 The sidechain, in equations

Per sample, per channel, before the detector.

```
// 1. the tracking dummy VCA: same gain as the audio VCA, plus the threshold offset
sc_gain_db = T − GR[n−1]                                  // T = ssl_threshold, dB
v_L = x_L[n] · 10^(sc_gain_db / 20)
v_R = x_R[n] · 10^(sc_gain_db / 20)

// 2. the sidechain high-pass, later revisions only: first order, −6 dB/octave
v_L = hpf(v_L, f_c[ssl_hpf])                              // one-pole, 5.6
v_R = hpf(v_R, f_c[ssl_hpf])

// 3. true peak full-wave rectification, per channel
r_L = |v_L| ; r_R = |v_R|

// 4. the dominant channel wins
r = max(r_L, r_R)                                          // 5.5

// 5. the ratio switch's detector gain
d = G[ssl_ratio] · r                                       // 6.1
```

`ssl_link` replaces step 4: `sum` uses `(r_L + r_R)/2` — Cytomic's "soft maximum" [42] would be a
smooth-max here — `dual` keeps two independent detectors and two independent gains, and `mid-side`
detects on `(L+R)/2` and `(L−R)/2`.

**The one-sample delay in step 1 is the feedback loop.** Comment it, and point at 5.3.

### 11.5 The diode and the RC network, which is where the character lives

Do not implement `attack_coeff` and `release_coeff`. Implement the three components.

```
// the diode: a smooth turn-on, not a hard max(0, ·)
i_drive = (d − V_cv) / R_att[ssl_attack]                   // current the detector can push
i       = softplus(i_drive · R_att, V_scale) / R_att       // D6's exponential turn-on
```

where `softplus(u, s) = s · ln(1 + exp(u/s))` and `V_scale = n·V_T ≈ 45 mV` for a silicon
small-signal diode. That single line is the knee (5.1, 6.4): it has no corner, it turns on over about
a decade of current, and it produces the `1 + 0.11513·(GR + V_d/k)` law of 5.4 without that law ever
being written down.

**The fixed release positions** are a series R, shunt RC, solved exactly over one sample:

```
τ_chg   = (R_att ∥ R_rel) · C                              // charging, diode conducting
V_targ  = d · R_rel / (R_att + R_rel)                      // the divider the diode charges toward
τ_dis   = R_rel · C                                        // discharging, diode off

if diode conducting:  V ← V_targ + (V − V_targ)·exp(−1/(fs·τ_chg))
else:                 V ← V · exp(−1/(fs·τ_dis))
```

**That divider is a real and surprising behaviour, and it is derived, not assumed.** The attack
resistor and the release resistor form a potential divider, so the *achievable* control voltage
depends on the attack setting:

| attack | release 0.1 s (180 kΩ) | release 1.2 s (1.2 MΩ) |
|---|---|---|
| 0.1 ms (820 Ω) | ×0.995 (−0.04 dB) | ×0.999 |
| 1 ms (8.2 kΩ) | ×0.956 (−0.4 dB) | ×0.993 |
| 10 ms (82 kΩ) | ×0.687 (−3.3 dB) | ×0.936 |
| 30 ms (270 kΩ) | **×0.400 (−8.0 dB)** | ×0.816 |

At the slowest attack with the fastest release the circuit loses **60 % of its control voltage**, and
therefore most of its gain reduction. No emulation with independent attack and release coefficients
does that, and it is the kind of behaviour engineers describe as "the slow attack settings don't seem
to do much". I have **no measurement to confirm it** — it is a consequence of the topology in 7.1 and
7.2 and it is labelled as derived, but it costs nothing to implement because it is what simulating
the network gives you for free.

**The Auto position** is two sections, integrated independently (7.4):

```
i     = the diode current above, common to both sections
V1 ← V1·exp(−1/(fs·τ₁)) + i·R7·(1 − exp(−1/(fs·τ₁)))       // τ₁ = R7·C1 = 42.8 ms
V2 ← V2·exp(−1/(fs·τ₂)) + i·R8·(1 − exp(−1/(fs·τ₂)))       // τ₂ = R8·C2 = 5.10 s
V   = V1 + V2
```

The 14.5 : 1 transient split and the 1 : 7.4 steady-state split (7.4) both fall out of this without
being coded, which is the test that the implementation is right rather than merely tuned.

Finally:

```
GR = min(V / k[ssl_ratio], ssl_range)                      // dB
```

### 11.6 The ratio, honestly

Section 6.1 could not close the mapping from switch position to printed ratio, so `k[ssl_ratio]` is a
**calibration table of three (or six) numbers, marked estimate**, chosen so that the model's measured
ratio at the operating point matches the printed label. The starting values from 5.4, with
`V_d = 0.6 V`:

| position | required `V_d/k` | `k` | note |
|---|---|---|---|
| 2:1 | 8.69 dB | **69 mV/dB** | |
| 4:1 | 26.1 dB | **23 mV/dB** | |
| 10:1 | 78.2 dB | **7.7 mV/dB** | close to the VCA's own 6.1 mV/dB (4.2), i.e. the top position drives the control port nearly directly |

All three are **estimates**. What is *not* an estimate is the shape they produce: `ratio` rises
linearly with gain reduction at 0.115 per dB in every position, which is 5.4 and is what section 13
tests.

### 11.7 Constants

**Where each number comes from is the point of this table.** S = SSL card drawing 82E26 or 82E27,
read directly; M = a manufacturer document; C = the clone builder's published reading; D = my
derivation from the values in this table; E = my estimate.

| symbol | value | source |
|---|---|---|
| **Attack ladder (82E27)** | | |
| R1–R6 | **820 Ω, 2.7 kΩ, 8.2 kΩ, 27 kΩ, 82 kΩ, 270 kΩ** | **S** [2] |
| τ_attack, open loop | **385 µs, 1.27, 3.85, 12.7, 38.5, 127 ms** | **D** |
| panel label ÷ τ | **≈ 1/4 at every position** | **D**, 7.3 |
| **Release ladder (82E27)** | | |
| C3–C6 | **0.47 µF** each | **S** [2] |
| R9–R12 | **1.2 MΩ, 560 kΩ, 270 kΩ, 180 kΩ** | **S** [2] |
| τ_release | **564, 263, 127, 84.6 ms** | **D** |
| panel label ÷ τ | 2.13, 2.28, 2.36, **1.18** — the last is unexplained | **D**, 7.3 |
| **Auto release (82E27)** | | |
| C1, R7 (fast section) | **0.47 µF, 91 kΩ** | **S** [2] |
| C2, R8 (slow section) | **6.8 µF, 750 kΩ** | **S** [2] |
| τ₁, τ₂ | **42.8 ms, 5.10 s** | **D** |
| transient charge split ΔV₁/ΔV₂ | **14.5** = C2/C1 | **D** |
| equilibrium split V₂/(V₁+V₂) | **0.892** = R8/(R7+R8) | **D** |
| **Timing buffer (82E27)** | | |
| T1 | **LF351N** | **S** [2] |
| R13, R14 | 3.3 MΩ, 470 kΩ | **S** [2]; configuration unresolved (7.2) |
| **Control-voltage distribution (82E27)** | | |
| R21, R26, R27 | **100 kΩ** each | **S** [2] |
| make-up pot | **25 kΩ linear**, via R20 470 kΩ | **S** [2] |
| make-up clamp | D2 1S44, R16 4.7 kΩ, D3 **12 V zener** | **S** [2] |
| threshold pot feed | R17 3.9 kΩ from +12 V, R18 360 Ω, wiper via R22 **360 kΩ** | **S** [2] |
| threshold pot end legends | **+15 dB / −15 dB** | **S** [2], corroborated by the Smart C1 (8.5) [22] |
| control-bus series R, decoupling | R51, R44 **100 Ω**; C12 **0.1 µF** | **S** [2] |
| **control bus scaling k** | **50 mV/dB** | **M**, XLogic calibration [8] — for the *fader* bus of a later unit, used here as the order of magnitude |
| **Audio VCA (82E26)** | | |
| A1 | **dbx 202C** | **S** [1] |
| input resistor R12 | **68.1 kΩ, 0.5 %** | **S** [1] |
| symmetry trim | RV1 **50 kΩ** between ±15 V, via R14 **1 MΩ**, marked `DISTORTION NULL` | **S** [1] |
| I/V converter | T3 **NE5534**, R13 4.7 kΩ, R15 470 Ω | **S** [1] |
| dbx 202 internals | **ten paralleled dbx 2150 cells, common control buffer** | **C** [16] |
| **VCA law** | | |
| gain-control constant | **−6.1 mV/dB** typical, −6.2 / −6.0 min / max | **M** [24] [25] [26] |
| temperature coefficient | **+0.33 %/°C** ref. 27 °C chip | **M** [24] |
| gain-control linearity | **0.5 % typ, 2 % max** over −60 to +40 dB | **M** [24] |
| THD untrimmed, 0 dBV, 0 dB gain | **0.005 / 0.010 / 0.030 %** by grade | **M** [24] |
| THD untrimmed, +10 dBV, −15 dB gain | **0.020 / 0.030 / 0.040 %** | **M** [24] |
| symmetry null window | **±1.6 mV** for THD < 0.07 % | **M**, 2150 datasheet [26] |
| second-harmonic dominance | "almost exclusively second harmonic" | **C** [16] |
| measured clone distortion | **−75 dB ≈ 0.018 %** | **C** [16] |
| **Sidechain (82E26)** | | |
| A2 | **dbx 202C**, driven by the same CV plus threshold | **S** [1] [2] |
| buffer | TR4 **E175** JFET, R31 4.7 kΩ | **S** [1] |
| I/V | T5 741, R32 **33 kΩ**, C17 22 pF, R33 470 Ω | **S** [1] |
| coupling | C18 **6.8 µF** | **S** [1] |
| half-wave stage | T6 741, R34 **20 kΩ** in, R35 **20 kΩ** feedback, D2/D3 | **S** [1] |
| summing | R36 **10 kΩ** (rectified), R37 **20 kΩ** (direct) — the 2:1 pair that makes \|x\| | **S** [1] |
| output stage | T7 741, R38 **1 MΩ** feedback, D4/D5 across it | **S** [1] |
| threshold diode | **D6**, drop ≈ **0.6 V** | **S** [1] for the part, **C** [16] for the voltage |
| all diodes | **1S44** | **S** [1] |
| **Ratio network (82E26)** | | |
| R39, R40, R41 | **510 kΩ, 270 kΩ, 68 kΩ** from T7 output | **S** [1] |
| R42, R43, R44 | **1.2 MΩ, 1.5 MΩ, 3.9 MΩ** to −12 V | **S** [1] |
| R45 | **620 kΩ** to +12 V | **S** [1] |
| detector gain per throw | **50.0, 16.9, 10.6** | **D**, 6.1 |
| DC offset per throw | **10.0 V, 2.70 V, 0.654 V** | **D**, 6.1; sign unresolved |
| switch type | **4 poles, 3 positions**; one pole drawn | **S** [1], **C** [17] |
| **Derived law** | | |
| ln10/20 | **0.11513** | **D** |
| ratio(GR) | **1 + 0.11513·(GR + V_d/k)** | **D**, 5.4 |
| loop gain γ | **ratio − 1** | **D**, 5.4 |
| closed-loop attack | **τ_open / (1 + γ)** | **D**, 7.3 |
| k per ratio position | **69, 23, 7.7 mV/dB** | **E**, 11.6 |
| **Sidechain filter** | | |
| frequencies | **Off, 30, 60, 105, 125, 185 Hz** | **M** [3] [4] [7] |
| order | **first, −6 dB/octave** | **E**, from the only slope published for the family, Smart Research's 150 Hz cable [22] |
| **Diode physics** | | |
| n·V_T | **≈ 45 mV** | **E**, silicon small-signal; the lab already carries 1N4148 constants in `dsp::bridge` |
| **Meter** | | |
| scale | **0 to 20 dB, linear**, from the control voltage | **M** [5] [4], **C** [16] |
| sensitivity | **≈ 50 µA/dB, 1 mA full scale = 20 dB** | **C** [16] |
| **Operating point** | | |
| nominal level | **+4 dBu** | **M**, the 500-series module guide: "the nominal input/output level is +4dBu" [3] |
| DAFx recording reference | songs normalised to **−12 dB**, **+5 dB in / −5 dB out** | **M** [30] |

### 11.8 Meter, oversampling, hygiene

**Meter.** `V / k` in dB, painted on a linear 0–20 scale (2.4). Ballistic damping: a moving-coil meter
of this size is roughly a second-order system with a rise time of a couple of hundred milliseconds;
the lab's existing `dsp::vu` already models one and this is a case where reuse is genuinely free
(12.2).

**Oversampling.** 2× by default, 1× and 4× offered, matching SSL's own model [10]. The nonlinearities
are the VCA's second-harmonic term and the detector diode; the detector runs at the base rate in any
case, so only the audio multiply needs the oversampled path.

**Sample rate.** All time constants are seconds, converted with `exp(−1/(fs·τ))` per sample. Nothing
in the model has a sample-rate-dependent constant. The fastest time constant is 385 µs, seventeen
samples at 44.1 kHz, which the exponential form handles exactly.

**Denormals.** The Auto release's 5.1 s section decays for a very long time; flush it.

**Reset.** On reset, `V1 = V2 = 0` and `GR = 0`. Because the loop reads `GR[n−1]`, a cold start is
one sample of open loop, which is inaudible and correct.

### 11.9 What the page should show

The transfer curve, drawn from the model rather than from a formula, because in this box the curve is
the finding: it should visibly bend for its whole length and never straighten (5.4, 6.4). Draw it at
all three ratio settings on one grid so the moving knee point (6.2) is visible. Add a second plot of
the control voltage over time under a two-second burst so the Auto release's two exponentials can be
seen separating (7.4) — that is a picture no other plug-in in this family shows, and it is the thing
the unit is famous for.
---

## 12. Reuse, assessed honestly

An audit of this codebase found **three tube stages that were three different circuits wearing one
word**, and the components crate's README records the same lesson from the other side: the CL 1B
earned the photocell crate its boundary by *refusing* the T4's timing while still sharing the
photoconductor's distortion law [58]. The SSL is the same test applied to the word "VCA", and it is a
harder test than it looks, because the Distressor and the SSL really do share a gain element and
really do not share anything else.

### 12.1 What is genuinely shared

**The dB-domain feedback loop.** Both boxes compute gain in decibels and both close a loop around the
gain element. `dsp::vca`'s scaffolding for that — carrying the previous sample's gain reduction,
converting dB to linear, ordering the loop so the detector reads a gain-reduced signal — transfers
directly and is the reason this is the cheapest genuinely-different model on the survey's list [55].

**The Blackmer cell's control law, and only the law.** The Distressor dossier reasons its way to a
"THAT/dbx-style Blackmer VCA driven by a control voltage in the log (dB) domain", explicitly labelled
"a strong, standard inference, labelled as such, not a confirmed part choice" [Distressor 3.2]. The
SSL needs no inference: card 82E26 has `DBX 202C` lettered on it in SSL's own hand (4.1) [1]. So the
two units share a part, one by evidence and one by inference, and the shared thing is exactly what
the components crate's rule admits: the exponential −6.1 mV/dB law, its ±0.1 mV/dB window, its
+0.33 %/°C coefficient, its 0.5 % linearity over 100 dB, and its published THD-versus-level-and-gain
table (4.2) [24] [25] [26].

**The components crate has been waiting for exactly this.** Its README lists as a coming candidate
"**VCA.** The Distressor's, likewise, and shared with every mainstream VCA compressor", and says each
candidate "waits for a second real user" [58]. The SSL is that second user. Its arrival is a stronger
justification than the diode bridge's was, because the bridge "has one user today, and a second... is
next but one in the plug-in's build order" [58], while the VCA would have two users on the day it
lands.

**And the boundary the crate should draw**, following the photocell precedent exactly: the crate holds
the **part** — the control law, the tolerances, the temperature coefficient, the distortion surface,
the symmetry-null concept — and holds **nothing** about the 68.1 kΩ current-mode input resistor, the
NE5534 I/V converter, the detector, the threshold, or how the control voltage is derived. Those are
the machine, and they differ completely between the two units. The crate should also record the
asymmetry of evidence: the SSL is a documented user, the Distressor an inferred one.

**Ordinary infrastructure.** The VU/meter ballistics in `dsp::vu`, the oversampler, the one-pole and
biquad sections in `dsp::vca::filters`, the transfer-curve stream and the stereo plumbing. None of
these is a component and all of them transfer without argument.

### 12.2 What only looks shared, item by item

| word | Distressor | SSL bus compressor | verdict |
|---|---|---|---|
| **ratio** | eight buttons, each selecting a hand-drawn curve with its own knee width, threshold offset, slope and release shape; `Curve` is a struct of tuned constants per position [Distressor 7.4] | a switch that changes a detector gain and a DC offset; the resulting slope is `1 + 0.11513·(GR + V_d/k)` and is a function of gain reduction, not a constant (5.4) | **not shared.** Reusing `Curve` forces a knee width the SSL does not have (6.4). |
| **knee** | per-position widths from 30 dB at 2:1 down to a couple of dB at Nuke [Distressor 7.4] | not a parameter at all; it is a diode's turn-on inside a loop (5.1) | **not shared**, and this is the sharpest one. |
| **attack / release** | continuous knobs, 50 µs–30 ms and 50 ms–3.5 s, with a branching detector whose times depend on the ratio *and* on how far the signal overshoots [Distressor 7.3] | two resistors and a capacitor; the times are RC products and the two resistors form a divider that costs gain reduction (11.5) | **not shared.** A common ballistics struct would strip the Distressor's branching or the SSL's network. |
| **threshold** | there isn't one; the Input knob drives a fixed internal threshold [Distressor 2] | a pot that adds gain to a *sidechain* VCA, which SSL confirm is equivalent to moving the threshold (8.3) [8] | **not shared**, and neither is a comparator reference. |
| **link** | three modes that sum control voltages or sum gain reduction [Distressor 3.7] | a **maximum** over independently rectified channels (5.5) [3] | **not shared.** Different operator. |
| **distortion** | a **separate generator block after the VCA**, switchable in and out, with an 80 Hz Bessel high-pass after it; Dist 2 and Dist 3 are deliberate colours [Distressor 3.1] | **the VCA is the distortion**; there is no separate stage anywhere in the audio path (3.2, 4.3) | **not shared, and this is the tube-stage trap in its exact original shape.** One box adds a colour; the other's gain element is imperfect. Same word, opposite architecture. |
| **detector** | band emphasis, sidechain high-pass, and a branching envelope generator [Distressor 3.3] | a precision full-wave rectifier into a passive RC, and the DAFx team measured that a *more* complex detector fits it **worse** [30] | **not shared.** |
| **auto release** | the 10:1 Opto position's two-stage release and Nuke's logarithmic one [Distressor 7.3] | two RC sections sharing charge in the ratio C2/C1 (7.4) | **not shared** — and note that `dsp::bridge`'s Neve auto release is a *third* mechanism, a gated platform capacitor [Neve-33609 7.4]. Three units, three "auto releases", three circuits. |

### 12.3 What about the diode bridge and the photocell?

Neither applies. There is no photocell and no diode bridge in this box. The detector's D6 is a single
small-signal diode used as a threshold, not a bridge used as an attenuator, and the components
crate's diode-bridge crate models "four matched diodes whose floating common nodes make its law a
hyperbolic tangent" [58] — a completely different object doing a completely different job. Pulling D6
into that crate would be the same error in miniature.

### 12.4 Verdict

- **New module, `dsp::gbus`.** Not a variant inside `dsp::vca`.
- **Extract a VCA cell into the components crate**, holding the Blackmer control law and its published
  tolerances and nothing else, with the SSL as its documented user and the Distressor as its inferred
  one. This is the crate's own stated trigger condition and it is now met [58].
- **Share `dsp::vu`, the oversampler and the filter sections** without ceremony.
- **Share nothing named ratio, knee, threshold, link, distortion, detector or auto release.** Every one
  of those is a different circuit in the two boxes, and the audit that found three tube stages under
  one word found it because somebody made exactly this trade for exactly these reasons.

The honest summary: this unit is cheap to build **because the loop scaffolding and the gain cell
transfer**, and it is worth building **because nothing else does**.

---

## 13. Test plan

**Two standards this repository enforces, and both bite hard here.** Every test asserts a **published
figure**, names it, and cites it. Where no real number is reachable, the test says so and asserts a
*circuit identity* or a *direction* rather than an invented bound.

**And this unit is the hardest case the lab has had for the first standard.** SSL publish no ratio
calibration, no ballistics measurement and no distortion figure for the compressor (8.1). So the tests
below draw on four kinds of figure, and each test says which kind it is using:

- **(P)** a figure printed by SSL — switch values, panel ranges, filter frequencies, operational
  statements.
- **(S)** a component value on SSL's own card drawing, with the arithmetic on it stated. A time
  constant computed from a resistor and a capacitor SSL specified is a published figure once the
  arithmetic is shown, in the same way the Neve dossier's 25 dB bridge attenuation is [Neve-33609
  12.1].
- **(M)** a figure from a component manufacturer's datasheet or from SSL's own calibration procedure.
- **(C)** a figure published by the clone builder or by Smart Research, cited as such.

Unless a test says otherwise: 1 kHz sine, `ssl_revision` = `console`, `ssl_link` = `dominant`,
`ssl_oversample` = 2×, run at 44.1, 48 and 96 kHz.

### 13.1 Structure and static behaviour

1. **Bypass is exact.** `ssl_bypass` on: output equals input to 1e-6.
   *Figure:* none needed; this is an identity the plug-in owes its user.

2. **The IN switch is not a bypass.** With `ssl_in` off and `ssl_makeup` = +10 dB, the output is
   **10 dB above the input, ±0.1 dB**, and gain reduction is exactly zero.
   *Figure:* "The main VCA is permanently in circuit; the compressor sidechain is enabled by the IN
   switch" **(P)** [3], and "On the original SSL compressor the makeup gain pot is active all the
   time, so when bypassed there's excess gain" **(C)** [16]. **This is the test that catches the
   commonest wrong assumption about this box.**

3. **Unity at zero.** `ssl_in` off, `ssl_makeup` = 0: output equals input to **±0.1 dB**.
   *Figure:* the THAT parts' "Gain at 0 V Control Voltage: 0.0 dB, **±0.1 dB**" for the A grade
   **(M)** [24]. The tolerance is the manufacturer's, not mine.

4. **Make-up is exact across its range.** `ssl_in` on, threshold high enough for no gain reduction:
   sweeping `ssl_makeup` from −5 to +15 dB changes the output by exactly the set amount, **±0.1 dB**.
   *Figure:* the range is SSL's own plug-in specification, "**−5dB to +15dB**" **(P)** [9]; the
   tolerance is test 3's.

5. **Nothing moves phase.** A swept sine through the model, compressor in and out, shows **no phase
   shift beyond that of the sidechain high-pass**, which is not in the audio path at all.
   *Figure:* "adding a simple phase inversion module would damage the model performance since **there
   are no phasers in the actual analog module**" **(M)** [30]. A small negative result, published,
   and worth asserting because a model that filters the audio path would fail it.

6. **The audio path has no filters.** With the compressor out, frequency response is flat within
   **±0.05 dB from 20 Hz to 20 kHz**.
   *Figure:* SSL's XLogic specification, "20Hz to 20kHz **±0.05dB**", measured at any gain setting
   **(M)** [8]. **Stated limitation:** that figure describes a 2004 SuperAnalogue unit, not a 1980
   console card. It is used here only as a bound on *the model's own* audio path, which has no
   filters in it at all and should therefore be flat to floating-point precision; the SSL figure is
   the loosest bound that is still a published one. The clone's "**less than 15 Hz to more than 35 kHz
   within 3 dB**" **(C)** [16] is the corresponding real-hardware figure and is far looser.

### 13.2 The feedback architecture

7. **The threshold control is a sidechain gain.** With a fixed input and a fixed ratio, raising
   `ssl_threshold` by **10 dB** must produce the same gain reduction as raising the input by 10 dB
   with the threshold unchanged, to **±0.5 dB**.
   *Figure:* SSL's XLogic manual, of the sidechain trims: "When fully clockwise they **increase the
   side chain level by 10dB — effectively reducing the threshold on that channel by 10dB**" **(P)**
   [8]. This is the only place SSL state the equivalence numerically, and it is the test that proves
   the model built a sidechain gain rather than a comparator.

8. **The detector sees a gain-reduced signal.** Instrument the model: with 10 dB of gain reduction
   established, the level at the rectifier input must be **10 dB below** what a feedforward detector
   would see, ±0.1 dB.
   *Figure:* a **circuit identity** read from card 82E27 — R26 and R27 carry the same control voltage
   to the audio and sidechain VCAs, and only R22 adds the threshold offset **(S)** [2] — plus
   Erland's independent reading, "acting mostly as a feed-back compressor" **(C)** [16]. **No number
   is published for this**, so the test asserts the identity rather than a measurement, and says so.

9. **A model without the feedback term fails on unseen settings.** Build the same engine with the
   detector reading the *input* instead of `input − GR`, and confirm the two disagree by more than
   1 dB of gain reduction somewhere in the parameter grid.
   *Figure:* the DAFx team's finding that the residual is attributable to "the changing compressor
   curve in the analog module, making it hard for **grey-box models without explicit feedback
   mechanisms** to capture that information" **(M)** [30]. This is a *differential* test: it asserts
   that the feedback term matters, which is what the published result says, without claiming to
   reproduce their error figures.

### 13.3 The ratio and the knee

10. **The slope rises with gain reduction and never straightens.** Measure the local slope at 1, 3, 5,
    10 and 20 dB of gain reduction at each ratio setting. The slope must **increase monotonically**
    at every setting, and the increase must be **0.115 per dB of gain reduction, ±20 %**.
    *Figure:* `ratio(GR) = 1 + 0.11513·(GR + V_d/k)`, derived in 5.4 from the loop equation, with
    `ln10/20 = 0.11513` **(S, via derivation)**; corroborated by SSL's "soft knee" description **(P)**
    [3] and by the DAFx team's "a soft knee where the **knee width is automatically computed based on
    the threshold and ratio**" **(M)** [30]. **The ±20 % tolerance is mine**, chosen because the
    derivation is exact but `V_d/k` is not, and labelled as an estimate.

11. **The knee point moves with the ratio, in the direction SSL state.** At a fixed threshold setting,
    measure the input level at which gain reduction first reaches 0.5 dB, at each ratio. **Lowering
    the ratio must lower that level.**
    *Figure:* "the knee point of the compressor, set with the THRESHOLD control, purposely changes
    depending on the setting of the RATIO control. **Decreasing the RATIO setting lowers the effective
    threshold**" **(P)** [3]. **No magnitude is published**, so this test asserts the *direction* SSL
    state and nothing more. Saying "and by about 3 dB" would be inventing a number.

12. **There is no ratio calibration test, and this is why.** SSL publish no measured transfer point
    for any ratio position, with or without a tolerance, in any document I could reach (8.1). The
    Neve dossier's primary calibration test [Neve-33609 12.1 test 4] has **no counterpart here**. The
    `k` table of 11.6 is therefore a set of **estimates** and must be labelled so in the code; the
    only thing tests 10 and 11 can pin is the law's *shape* and its *direction*. **A test asserting
    "5 dB ±1 dB at 4:1" would be asserting my own tuning, which is the failure the audit found in
    five plug-ins, and I am not writing one.**

### 13.4 Ballistics

13. **The attack resistors are what the drawing says.** Instrument the model's open-loop attack time
    constant at each position and assert **385 µs, 1.27 ms, 3.85 ms, 12.7 ms, 38.5 ms, 127 ms**,
    ±2 %.
    *Figure:* R1–R6 = 820 Ω, 2.7 kΩ, 8.2 kΩ, 27 kΩ, 82 kΩ, 270 kΩ across C = 0.47 µF, all on card
    82E27 **(S)** [2], with the arithmetic in 7.1. The ±2 % is a floating-point margin, not a
    physical tolerance.

14. **The effective attack matches the panel.** Closed loop at `ssl_ratio` = 4:1, measure the time for
    gain reduction to reach 63 % of its final value after a step: assert the **panel figure** at each
    position, **±30 %**.
    *Figure:* the panel legend `ATTACK mS` with 0.1 / 0.3 / 1 / 3 / 10 / 30 **(P)** [2] [9], and the
    derivation `τ_closed = τ_open/(1+γ)` with `γ = 3` at 4:1 (7.3) **(S, via derivation)**. **The
    ±30 % is mine** and is wide on purpose: the derivation predicts a factor of 3.85–4.23 against a
    panel that prints round numbers, and I have no measurement to tighten it against.

15. **The release resistors are what the drawing says.** Assert release time constants of **564 ms,
    263 ms, 127 ms, 84.6 ms** at the 1.2 / 0.6 / 0.3 / 0.1 positions, ±2 %.
    *Figure:* R9–R12 = 1.2 MΩ, 560 kΩ, 270 kΩ, 180 kΩ across 0.47 µF **(S)** [2], 7.2.
    **This test will look wrong and it is correct.** The 0.1 s position's constant is 84.6 ms, which
    is 1.18 times the panel figure while the other three are 2.1 to 2.4 times theirs (7.3). The
    discrepancy is in the drawing, not in the model, and the test comment must say so and point here.

16. **The Auto release is two exponentials, 42.8 ms and 5.10 s.** Drive the model into 10 dB of gain
    reduction with a 20 ms burst, release, and fit two exponentials to the control voltage: assert
    **τ₁ = 42.8 ms ±5 %** and, after a 10 s sustained tone, **τ₂ = 5.10 s ±5 %**.
    *Figure:* R7 91 kΩ with C1 0.47 µF, and R8 750 kΩ with C2 6.8 µF, on card 82E27 **(S)** [2],
    7.4. **This is the most valuable test in the file**, because it is the unit's signature and it is
    fully determined by four component values SSL specified.

17. **The Auto charge split.** After a short burst, the fast section must hold **14.5 times** the
    voltage of the slow one (ratio C2/C1); after a long sustained tone, the slow section must hold
    **89.2 %** of the total (ratio R8/(R7+R8)). Both ±5 %.
    *Figure:* the same four components **(S)** [2], with the arithmetic in 7.4. This is the test that
    catches an Auto release that was tuned to sound right rather than built from the network.

18. **Attack and release interact through the divider.** At `ssl_attack` = 30 ms and
    `ssl_release` = 0.1 s, the steady-state gain reduction on a constant tone must be **8.0 dB less**
    than at `ssl_attack` = 0.1 ms with the same release, ±1 dB.
    *Figure:* the potential divider R_rel/(R_att + R_rel) = 180 kΩ/450 kΩ = 0.400 **(S, via
    derivation)** [2], 11.5. **No measurement of this exists anywhere** and I flag it as the least
    supported test in the plan: it follows from the topology, and if a real unit does not do it, the
    topology reading in 7.1–7.2 is wrong and this test is how we would find out.

19. **Auto release survives a rate change.** Every time constant above holds at 44.1, 48, 96 and
    192 kHz, to the same tolerances.
    *Figure:* none needed; a sample-rate-dependent time constant is a bug.

### 13.5 Stereo, filter and gain cell

20. **The detector takes the maximum, not the sum.** Feed −20 dBFS into the left channel and −40 dBFS
    into the right: the gain reduction must equal the gain reduction produced by −20 dBFS into both,
    **±0.1 dB**. With `ssl_link` = `sum` it must not.
    *Figure:* "the **dominant, ie. louder channel**, controls the gain reduction of the overall stereo
    level" **(P)** [3], and SSL's own six-channel implementation lighting "the LED corresponding to the
    channel that is applying the most gain reduction" **(P)** [8]. This test is a clean pass/fail on
    an operator, so the tolerance can be tight.

21. **Both channels get the same gain.** In `dominant` mode the left and right gains are **bit-identical**.
    *Figure:* the same sentence — one control voltage drives "the overall stereo level" **(P)** [3].

22. **The sidechain filter is in the sidechain only.** With `ssl_hpf` = 185 Hz and the compressor
    doing nothing, the audio-path response at 30 Hz is unchanged to **1e-6**; with the compressor
    working on a 30 Hz tone, gain reduction is measurably lower than with `ssl_hpf` = Off.
    *Figure:* "an HPF (High Pass Filter) **in the sidechain**" **(P)** [3], with the switch positions
    "30Hz / 60Hz / 106Hz / 125Hz / 185Hz" **(P)** [7]. **Note the discrepancy:** SSL's product page
    says **106 Hz** and SSL's own module panel and recall sheet both print **105** [5] [4]. The model
    uses 105 and the test comment records the disagreement.

23. **The filter is first order.** The sidechain response 1 octave below each corner is **−6 dB ±1 dB**
    relative to the passband.
    *Figure:* Smart Research's sidechain filter, "150Hz **−6dB/octave**" **(C)** [22]. **Stated
    limitation:** this is the *only* slope figure published for anything in this family, and it is for
    a different unit's outboard cable, not for SSL's built-in filter. The test asserts it as an
    explicitly borrowed figure and the comment says so.

24. **The gain cell's distortion is second-harmonic and it rises with drive.** At 0 dBV and unity
    gain, THD is **0.005 % ±50 %**, and the second harmonic is at least **20 dB above** the third. At
    +10 dBV and −15 dB gain, THD is **0.020 % ±50 %**.
    *Figure:* the THAT 2180A typical THD table, "VIN = 0 dBV, 0 dB gain: **0.005 %**" and "VIN =
    +10 dBV, −15 dB gain: **0.020 %**" **(M)** [24]; the harmonic family from Erland's measurement,
    "almost exclusively second harmonic" **(C)** [16]. **The ±50 % is mine**, because the datasheet
    gives typicals with a maximum but no distribution, and the second-to-third ratio is a **direction**
    rather than a published number.

25. **The control law is exponential and linear in dB.** Sweep the control voltage over a 100 dB range
    of gain and assert the model's dB-per-volt is constant to **0.5 %**, with a maximum deviation
    under **2 %**.
    *Figure:* THAT's "Gain-Control Linearity: **0.5 % typical, 2 % maximum**, −60 dB to +40 dB gain"
    **(M)** [24]. This is a published tolerance over a published span and it is the cleanest
    calibration figure available anywhere in this file.

26. **The meter reads the control voltage on a linear 0–20 dB scale.** With 10 dB of gain reduction
    the meter sits at **half scale, ±2 %**.
    *Figure:* the module's printed scale, `0 4 8 12 16 20` evenly spaced **(P)** [5] [4], and
    "linear scale, at about **50 µA/dB**, making a **1 mA meter showing 20 dB full-scale**" **(C)**
    [16].

### 13.6 What I will not test, and why

- **Ratio calibration** (13.3 test 12). No published figure exists.
- **Noise.** The XLogic's "< −99 dBu" [8] and Smart Research's "−104 dBm" [22] describe different,
  later, better circuits, and the clone's "less than −80 dB" [16] describes a homebuilt one. A
  floating-point model's noise floor is a design choice, and asserting any of those three against it
  would be theatre.
- **Absolute distortion of the whole box.** Test 24 asserts the *gain cell's* published figure. The
  box's own figure is 0.018 % from a clone [16], 0.025 % from a C1 and 0.005 % from a C2 [22], and
  none of those is the console card.
- **Temperature drift and symmetry-null-versus-frequency** (9.3). Real, published, and not modelled.
- **Anything against the DAFx dataset.** Their L1 and M-STFT figures (10.2) are the right kind of
  target, but the dataset is 2528 hours behind a Google Drive link and the paper does not publish the
  per-setting matching procedure needed to reproduce their plug-in comparison. **If the lab ever wants
  a real external benchmark, that dataset is where it is**, and this is the note that says so.
---

## 14. References

Everything below was fetched and read while writing this file, except where the entry says otherwise.
**WebSearch was unavailable for the whole session** (1.0), so every item was reached by constructing
a URL, following a link out of a document already in hand, or querying a search API directly.

Manufacturer documents are cited as manufacturer claims. The two card schematics are cited
**separately**, because they carry different title blocks, different dates and different halves of the
circuit, and merging them would hide that the timing network and the sidechain are on different cards
(3.1). The clone builder's page is cited as a clone builder's reading of a drawing he had; where his
prose disagrees with the drawing, both are cited and the disagreement is stated (7.4).

**SSL's own circuit drawings**

These are photographs of Solid State Logic card schematics, taken out of an SL 4044 E desk and
published by Jakob Erland as reference material for his clone [16]. They are not clone drawings.

1. Solid State Logic, **card 82E26**, "82E26 QUAD BUS MIX AMPS, PATCH RETURNS, VCA, COMPRESSOR SIDE
   CHAIN", title-block date **19-8-80**, revision **G**, with the marginal notes "All Diodes 1S44",
   "\* 0·5 % Tol." and "Last used R58, C29, D11, T8, TR4, A2". The audio VCA (dbx 202C, A1), its
   68.1 kΩ input resistor and 50 kΩ `DISTORTION NULL` trimmer, the sidechain VCA (A2) with its E175
   buffer, the precision full-wave rectifier (T5, T6, T7), the threshold diode D6, the RATIO switch
   network R38–R45 with D7, and the ±12 V subregulators. Sections 3.2, 3.3, 4.1, 5.1, 5.2 and 6.1 are
   read from it. 800 × 555. https://www.gyraf.dk/gy_pd/ssl/ssl_82e26.gif
2. Solid State Logic, **card 82E27**, "CF82E27 COMPRESSOR TIME CONSTANTS, QUAD FADER, AUTOFADE",
   drawing number **82E27-710-7911-0**, with the marginal note "FIT R26 FOR COMPRESSOR POST FADER.
   OMIT R26 FOR COMPRESSOR PRE FADER." The attack ladder R1–R6, the release ladder C3–C6 with R9–R12,
   the Auto network C1/R7 and C2/R8, the LF351N buffer, the MAKE UP and THRESHOLD summing, the
   control-voltage split R21/R26/R27 to the audio and sidechain VCAs, and the quad fader and autofade
   machinery. Sections 3.4, 7.1, 7.2 and 7.4 are read from it. 800 × 538.
   https://www.gyraf.dk/gy_pd/ssl/ssl_82e27.gif

**Solid State Logic documents and product pages**

3. Solid State Logic, *G Series Bus Compressor Module for 500 Series Racks — User Guide*, revision
   V2.0, June 2020. Eight pages. The source for the IN switch's behaviour, the moving knee point, the
   dominant-channel detector, the sidechain HPF and the +4 dBu nominal level; page 5 carries the
   front-panel illustration used in section 2.
   https://solidstatelogic.com/assets/uploads/downloads/SSL_500_Series_G_Comp_Module_User_Guide.pdf
4. Solid State Logic, **Stereo Bus Compressor Module Recall Sheet**. A dimensioned line drawing of the
   module face with every legend and every detent dot, cross-checked against [5] in section 2.2.
   https://www.solidstatelogic.com/assets/uploads/downloads/500-mods/Bus-Compressor_500-Module-Recall-Sheet.pdf
5. Solid State Logic, **500-series G-Comp front-panel render**, 1618 × 2697 PNG. Every colour and every
   proportion in sections 2.2 and 2.3 is measured from this image; saved as
   `ref/ssl-SSL_500_G-Comp._1685.png`.
   https://www.solidstatelogic.com/assets/uploads/images/500-mods/500%20mods%20Facelift/SSL%20500%20G-Comp.%201685.png
6. Solid State Logic, **500-series G-Comp angled photograph**, 2000 × 1996 JPEG; saved as
   `ref/ssl-Bus_Comp_Facelift_500_series_module_angle.jpg`.
   https://www.solidstatelogic.com/assets/uploads/images/Bus_Comp_Facelift_500_series_module_angle.jpg
7. Solid State Logic, **Stereo Bus Compressor Module** product page. The "Glueing mixes together for
   40+ years" copy, the sidechain filter frequencies, and the "Additional compression ratio settings
   1.5 / 3 / 10" bullet that section 1.3 shows cannot be right.
   https://www.solidstatelogic.com/products/stereo-bus-compressor-module
8. Solid State Logic, ***XLogic Multichannel Compressor Owner's Manual***. Fourteen pages, including
   Appendix C (Performance Specification) and Appendix D (Calibration Information). The source for
   everything in 8.2 and 8.3, including the 50 mV/dB control bus and SSL's statement that 10 dB of
   sidechain gain is 10 dB of threshold reduction. https://archive.org/details/manualsbase-id-561341
   (direct: https://archive.org/download/manualsbase-id-561341/561341.pdf)
9. Solid State Logic, *SSL Native V6.5 — User Guide*, 72 pages. Section 3 is the Bus Compressor
   plug-in: its switch values, its −20/+20 dB threshold, its −5/+15 dB make-up, its Auto description,
   and the "2:1, 4:1 and 20:1" ratio list discussed in 6.3.
   https://www.solidstatelogic.com/assets/uploads/downloads/plug-ins/SSL%20Native%20v6.5%20-%20User%20Guide.pdf
10. Solid State Logic, **SSL Native Bus Compressor 2** product page. "an accurate emulation of the
    legendary G-Series Bus Compressor", the 2× and 4× oversampling, the Mix control, and the "audio
    glue" quotation. https://www.solidstatelogic.com/products/ssl-native-bus-compressor-2
11. Solid State Logic, **THE BUS+** product page. The LOW THD, F/B and 4K modes quoted in 1.4, 4.3 and
    5.3, the four stereo modes, the negative ratios and the "Auto 2" release.
    https://www.solidstatelogic.com/products/the-bus-plus
12. Solid State Logic, **support downloads**. A JavaScript search form; a plain fetch returns "Sorry,
    we couldn't find anything!" and the product selector lists no legacy product (1.0).
    https://www.solidstatelogic.com/support/downloads
13. Solid State Logic, *X-Logic Series / X-Rack user manual*, initial release rev. 0A, September 2005.
    Checked for a bus compressor module and found to contain the mic amp, EQ and dynamics modules
    only. https://archive.org/details/manualsbase-id-233017
14. Solid State Logic, *Bus Compressor 2 User Guide*. **Unreachable.** Listed as a download on [10] but
    the page carries no URL for it, and every filename constructed under the plug-in download path
    returned HTTP 403 while [9] at the same prefix returned 200 (1.0).
    https://www.solidstatelogic.com/assets/uploads/downloads/plug-ins/
15. Solid State Logic, *SSL Studio Tools Brochure*, linked from [7]. **Unreachable**, HTTP 404 at every
    filename I constructed. https://solidstatelogic.com/assets/uploads/downloads/

**Clone and descendant documentation**

16. Jakob Erland (Gyraf Audio), **"The SSL Mixbus Compressor Clone"**, dated 10 May 2006. The single
    most-cited document about this compressor, and the source of [1] and [2]. Quoted in 3.1, 3.3, 4.1,
    4.3, 5.2, 7.4 and 8.4, including the tracking dummy VCA, the 0.6 V threshold diode, the
    "acting mostly as a feed-back compressor" reading, the meter's 50 µA/dB, and the Auto-release
    parenthesis that 7.4 shows is the wrong way round. https://www.gyraf.dk/gy_pd/ssl/ssl.htm
17. Jakob Erland, **GSSL components list**. The source for the RATIO switch being a 4-pole 3-position
    type (3.5, 6.1), and for the 6.8 µF and five 0.47 µF tantalums that confirm the timing values of
    7.2. https://www.gyraf.dk/gy_pd/ssl/ssl_complist.htm
18. Jakob Erland, **GSSL clone schematic**, 4715 × 5963 GIF. Erland's own redraw, consulted as a
    cross-check on [1] and [2] and cited nowhere as evidence about SSL's circuit.
    https://www.gyraf.dk/gy_pd/ssl/ssl_sch.gif
19. Jakob Erland, **ratio-adjust modification note**: the 100 kΩ that becomes 127 kΩ when a THAT part
    replaces a dbx 2150, quoted in 4.1. https://www.gyraf.dk/gy_pd/ssl/ratio_2180.jpg
20. Jakob Erland, **unity-gain modification note**: the two 15 kΩ resistors that become 27 kΩ.
    https://www.gyraf.dk/gy_pd/ssl/unity_gain.jpg
21. Jakob Erland, **threshold-desensitising modification note**: a 47 kΩ resistor added ahead of the
    threshold pot for modern levels. https://www.gyraf.dk/gy_pd/ssl/thresh_red.jpg
22. Smart Research Ltd., **products and specifications**. The C1 and C2 specifications of 8.5, the
    ratio, attack and release ladders of 1.3, the 150 Hz −6 dB/octave sidechain cable of 5.6, and the
    "window through which transients can still pass" description of 9.1.
    https://www.smartresearch.co.uk/
23. Smart Research Ltd., **"About Smart Research"** (same page, anchor `#About_Smart_Research`). Alan
    Smart's SSL provenance, quoted in 1.3.
    https://www.smartresearch.co.uk/#About_Smart_Research

**Component data**

24. THAT Corporation, *THAT 2180 Series — Blackmer Pre-Trimmed IC Voltage Controlled Amplifiers*
    datasheet, document 600029 rev. 02, 2008. Twelve pages. The gain-control constant, temperature
    coefficient, linearity, THD tables and noise figures of 4.2 and the constants table of 11.7.
    https://www.thatcorp.com/datashts/THAT_2180-Series_Datasheet.pdf
25. THAT Corporation, *THAT 2181 Series* datasheet, the externally-trimmed sibling of [24].
    https://www.thatcorp.com/datashts/THAT_2181-Series_Datasheet.pdf
26. THAT Corporation, *THAT 2151 / 2150A / 2155* datasheet, marked OBSOLETE. "Based on dbx
    technology"; the symmetry control voltage window and trimmed-THD figures of 4.2, and the note
    about control-path pickup and the "e.g. 61 mV/dB — ten times higher than the VCA re[quires]"
    scaling advice used in 8.3. https://www.thatcorp.com/datashts/THAT_2150-Series_Datasheet.pdf
27. Texas Instruments, *NE5534 / NE5534A* datasheet. The op-amp SSL used for the VCA's
    current-to-voltage converter (3.2), and the part Alan Smart specifically criticises for its slew
    rate in [22]. https://www.ti.com/lit/ds/symlink/ne5534.pdf
28. Vishay, *1N4148 / 1N4448 small signal fast switching diodes* datasheet. Used only as the
    stand-in for the 1S44 parts SSL specify (11.7); the lab already carries these constants in
    `dsp::bridge`. https://www.vishay.com/docs/81857/1n4148.pdf
29. David E. Blackmer, "Multiplier circuits", US 3,714,462. **Unreachable**: Google Patents returned
    HTTP 503 on every attempt (1.0). Nothing is lost that [24] and [26] do not carry, except the
    primary-source pedigree. https://patents.google.com/patent/US3714462A/en

**Modelling literature and the one published measurement**

30. Yicheng Gu, Runsong Zhang, Lauri Juvela and Zhizheng Wu, **"Solid State Bus-Comp: A Large-Scale
    and Diverse Dataset for Dynamic Range Compressor Virtual Analog Modeling"**, *Proc. 28th Int.
    Conf. Digital Audio Effects (DAFx25)*, Ancona, 2–5 September 2025, pages 55–60. Open access, CC BY
    4.0. **The most important document in this file after the two schematics.** 2528 hours recorded
    through a real SSL 500 G-Bus in 220 parameter combinations; the recording rig and level
    calibration of 10.2; Table 4's benchmark of four commercial plug-ins against the hardware; the
    soft-knee and switching-one-pole grey-box findings; and the "grey-box models without explicit
    feedback mechanisms" result that corroborates 5.3.
    https://www.dafx.de/paper-archive/2025/DAFx25_paper_13.pdf
31. Solid State Bus-Comp project page, with demos and links to the dataset and checkpoints. Consulted;
    it does not restate the recording parameters, which are only in [30].
    https://www.yichenggu.com/SolidStateBusComp/
32. DAFx paper archive search. How [30] was found, by searching the archive for the string `SSL` —
    which the archive also matches against "self-supervised learning" (1.0).
    https://www.dafx.de/paper-archive/search.php?q=SSL
33. Coriander V. Pines, "Real-Time Virtual Analog Modelling of Diode-Based VCAs", *Proc. DAFx-25*,
    Ancona, September 2025. Cited here only for the silicon diode constants the lab already uses in
    `dsp::bridge` (11.7); its subject is the diode bridge, not this box.
    https://www.dafx.de/paper-archive/2025/DAFx25_paper_44.pdf
34. Dimitrios Giannoulis, Michael Massberg and Joshua D. Reiss, "Digital Dynamic Range Compressor
    Design — A Tutorial and Analysis", *Journal of the Audio Engineering Society*, vol. 60 no. 6,
    2012. Reference [1] of [30] and the standard statement of the gain-computer-plus-level-detector
    architecture the DAFx grey-box models use. **Abstract read; the full paper is behind the AES
    paywall and was not read.** https://www.aes.org/e-lib/browse.cfm?elib=16354
35. Marco Comunità et al., **NablAFx**, the toolbox [30] used for its benchmark experiments.
    https://github.com/mcomunita/nablafx
36. **ToneTwist AFX dataset**, the comparison dataset and the source of the TVFiLM and TVConcat
    conditioning layers [30] benchmarks. https://github.com/mcomunita/tonetwist-afx-dataset
37. **Reaper**, the DAW [30] drove the hardware from. https://www.reaper.fm/
38. **RME Fireface UFX+**, the interface [30] recorded through.
    https://rme-audio.de/fireface-ufx.html
39. **Cambridge Multitrack Library**, the source of the 175 unmastered songs in [30]. **Unreachable**
    from this machine: HTTP 403. https://www.cambridge-mt.com/ms/mtk/

**Emulations, for benchmarking**

40. Universal Audio, **SSL 4000 G Bus Compressor Collection**. "an expert end-to-end circuit
    emulation", "Fully endorsed by Solid State Logic"; adds a sidechain filter, a Mix control and a
    Headroom control not on the hardware.
    https://www.uaudio.com/uad-plugins/compressors-limiters/ssl-4000-g-bus-compressor-collection.html
41. Waves, **SSL G-Master Buss Compressor**. The page I fetched carries one line of product copy and
    no specification; the survey's description of it as licensed by SSL [55] is not on it.
    https://www.waves.com/plugins/ssl-g-master-buss-compressor
42. Cytomic, **The Glue**. The dual-diode envelope follower, the nodal-analysis claim, the "soft
    maximum" stereo detector, the Range control and the oversampling options quoted in 5.5, 9.1 and
    10.3. `curl` receives HTTP 403 from this host; fetched through WebFetch (1.0).
    https://www.cytomic.com/product/glue/
43. Softube, **Bus Processor**. "a glue compressor with ultramodern algorithms"; **the page never names
    SSL or the G Series**, which matters for how its result in 10.2 should be read.
    https://www.softube.com/bus-processor
44. Overloud, **Comp G**. "The Revolutionary Simulation of the British VCA Master Bus Compressor";
    does not name the hardware and publishes no figures.
    https://www.overloud.com/products/comp-g
45. PSPaudioware, **PSP BussPressor**. "combines the sound of classic VCA compression with wide tuning
    capabilities"; publishes nothing, and is the closest of the four to the hardware in [30]'s
    measurement. https://www.pspaudioware.com/products/psp-busspressor
46. Brainworx, **bx_townhouse Buss Compressor**. Named by the survey [55]; page fetched, no
    measurements. https://www.plugin-alliance.com/en/products/bx_townhouse_buss_compressor.html

**Background and photographs**

47. Wikipedia, **Solid State Logic**. Colin Sanders, 1969, the organ-control origin, the B/E/G
    chronology, the XLogic and X-Rack dates, and the Duende.
    https://en.wikipedia.org/wiki/Solid_State_Logic
48. Wikipedia, **Solid State Logic SL 4000**. The 1976–2002 production span, the E Series as "the
    first console to offer a compressor/gate on every channel as well as a master bus compressor", the
    Listen Mic and gated-reverb story, and the 1996 Billboard figure.
    https://en.wikipedia.org/wiki/Solid_State_Logic_SL_4000
49. Sound On Sound, **"Classic Compressors"**. The comparison with the API 2500, the "mid-range crunch
    when more heavily provoked" description, Tom Lord-Alge on using it as a transient tool, Jason
    Goldstein on the console being "always just shy of distorting", and its description of the Smart
    Research C2 as an emulation of the SSL. Cited for description only; it gives no quantified figure
    for this unit. https://www.soundonsound.com/techniques/classic-compressors
50. Wikimedia Commons, **SSL G+4000**, 4608 × 3072, CC BY-SA 4.0. Fetched and cropped at full
    resolution; **the bus compressor panel is not in frame** (1.0, 2.5).
    https://commons.wikimedia.org/wiki/File:SSL_G%2B4000.JPG
51. Wikimedia Commons, **Solid State Logic SL4064G+**, 4264 × 2832, CC BY-SA 3.0. Fetched and cropped
    at full resolution; **the centre section is obscured by a Total Recall terminal** (1.0, 2.5).
    https://commons.wikimedia.org/wiki/File:Solid_State_Logic_SL4064G%2B.jpg
52. Funky Junk / Pro Audio Europe, **SSL G384** listing, linked from [16] as an alias of the same
    compressor. The page now resolves to a shop front with no G384 content.
    https://www.proaudioeurope.com/london/vintage/compressor/ssl_g384.html
53. Internet Archive **advanced search API**, used to enumerate what SSL documentation exists on
    archive.org; how [8] and [13] were found. https://archive.org/advancedsearch.php
54. Wikimedia Commons **API search**, used to enumerate SSL console photographs and their licences.
    https://commons.wikimedia.org/w/api.php

**In this repository**

55. `research/SURVEY.md`, section 3.3, which ranked this unit third and set the brief this file
    answers. Its judgements are corrected in 1.0 (the schematics are SSL's own, not a clone's), 1.3
    (SSL's "1.5 / 3 / 10" bullet), 5.3 (feedforward versus feedback) and 8.6 (there *is* now a
    published measurement).
56. `research/Distressor.md`, the lab's existing VCA model and the comparison that section 12 is built
    on.
57. `research/Neve-33609.md`, the dossier this file is written to match, and the source of the
    sheet-by-sheet citation convention and the constants-table format.
58. `noob-electrical-components`, README. The components crate's boundary rule, the photocell
    precedent, the diode-bridge crate's description, and the "VCA... waits for a second real user"
    entry that section 12 argues is now satisfied.
    https://github.com/Noob-Audio-Engineering/noob-electrical-components

[1]: https://www.gyraf.dk/gy_pd/ssl/ssl_82e26.gif
[2]: https://www.gyraf.dk/gy_pd/ssl/ssl_82e27.gif
[3]: https://solidstatelogic.com/assets/uploads/downloads/SSL_500_Series_G_Comp_Module_User_Guide.pdf
[4]: https://www.solidstatelogic.com/assets/uploads/downloads/500-mods/Bus-Compressor_500-Module-Recall-Sheet.pdf
[5]: https://www.solidstatelogic.com/assets/uploads/images/500-mods/500%20mods%20Facelift/SSL%20500%20G-Comp.%201685.png
[6]: https://www.solidstatelogic.com/assets/uploads/images/Bus_Comp_Facelift_500_series_module_angle.jpg
[7]: https://www.solidstatelogic.com/products/stereo-bus-compressor-module
[8]: https://archive.org/details/manualsbase-id-561341
[9]: https://www.solidstatelogic.com/assets/uploads/downloads/plug-ins/SSL%20Native%20v6.5%20-%20User%20Guide.pdf
[10]: https://www.solidstatelogic.com/products/ssl-native-bus-compressor-2
[11]: https://www.solidstatelogic.com/products/the-bus-plus
[12]: https://www.solidstatelogic.com/support/downloads
[13]: https://archive.org/details/manualsbase-id-233017
[14]: https://www.solidstatelogic.com/assets/uploads/downloads/plug-ins/
[15]: https://solidstatelogic.com/assets/uploads/downloads/
[16]: https://www.gyraf.dk/gy_pd/ssl/ssl.htm
[17]: https://www.gyraf.dk/gy_pd/ssl/ssl_complist.htm
[18]: https://www.gyraf.dk/gy_pd/ssl/ssl_sch.gif
[19]: https://www.gyraf.dk/gy_pd/ssl/ratio_2180.jpg
[20]: https://www.gyraf.dk/gy_pd/ssl/unity_gain.jpg
[21]: https://www.gyraf.dk/gy_pd/ssl/thresh_red.jpg
[22]: https://www.smartresearch.co.uk/
[23]: https://www.smartresearch.co.uk/#About_Smart_Research
[24]: https://www.thatcorp.com/datashts/THAT_2180-Series_Datasheet.pdf
[25]: https://www.thatcorp.com/datashts/THAT_2181-Series_Datasheet.pdf
[26]: https://www.thatcorp.com/datashts/THAT_2150-Series_Datasheet.pdf
[27]: https://www.ti.com/lit/ds/symlink/ne5534.pdf
[28]: https://www.vishay.com/docs/81857/1n4148.pdf
[29]: https://patents.google.com/patent/US3714462A/en
[30]: https://www.dafx.de/paper-archive/2025/DAFx25_paper_13.pdf
[31]: https://www.yichenggu.com/SolidStateBusComp/
[32]: https://www.dafx.de/paper-archive/search.php?q=SSL
[33]: https://www.dafx.de/paper-archive/2025/DAFx25_paper_44.pdf
[34]: https://www.aes.org/e-lib/browse.cfm?elib=16354
[35]: https://github.com/mcomunita/nablafx
[36]: https://github.com/mcomunita/tonetwist-afx-dataset
[37]: https://www.reaper.fm/
[38]: https://rme-audio.de/fireface-ufx.html
[39]: https://www.cambridge-mt.com/ms/mtk/
[40]: https://www.uaudio.com/uad-plugins/compressors-limiters/ssl-4000-g-bus-compressor-collection.html
[41]: https://www.waves.com/plugins/ssl-g-master-buss-compressor
[42]: https://www.cytomic.com/product/glue/
[43]: https://www.softube.com/bus-processor
[44]: https://www.overloud.com/products/comp-g
[45]: https://www.pspaudioware.com/products/psp-busspressor
[46]: https://www.plugin-alliance.com/en/products/bx_townhouse_buss_compressor.html
[47]: https://en.wikipedia.org/wiki/Solid_State_Logic
[48]: https://en.wikipedia.org/wiki/Solid_State_Logic_SL_4000
[49]: https://www.soundonsound.com/techniques/classic-compressors
[50]: https://commons.wikimedia.org/wiki/File:SSL_G%2B4000.JPG
[51]: https://commons.wikimedia.org/wiki/File:Solid_State_Logic_SL4064G%2B.jpg
[52]: https://www.proaudioeurope.com/london/vintage/compressor/ssl_g384.html
[53]: https://archive.org/advancedsearch.php
[54]: https://commons.wikimedia.org/w/api.php
[58]: https://github.com/Noob-Audio-Engineering/noob-electrical-components

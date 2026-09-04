# The Fairchild variable-mu limiter, 660 and 670: research notes for the variable-mu side of `noob-compressorlab`

Research dossier for the variable-mu model of the `noob-compressorlab` example plug-in of
noob-vst-webgui-framework. The example is a humorous, affectionate spoof of the Fairchild 660 and 670
limiting amplifiers. It is not a product, it is not endorsed by anybody, and it does not use the Fairchild
name as its own name. Fairchild is a registered trademark of Avid Technology, Inc. [38]. Trademarks below
belong to their owners and are used only to identify the devices and the products discussed. This model sits
behind the same per-instance `model` switch that already selects the 1176, LA-2A, LA-3A, CL 1B, Distressor,
6176 and Neve behaviours; see [[1176]], [[610]], [[LA-2A]], [[LA-3A]], [[CL-1B]], [[Distressor]] and
[[Neve-33609]].

Conventions (kept the same as the other dossiers so they all read alike):

- Citations are `[n]`; the numbered list in section 13 gives the URL for every source, and reference-style
  link definitions at the very end make the `[n]` markers clickable.
- Numbers that come from a manufacturer specification, a manual, a schematic, a datasheet or a published
  chart are attributed. Numbers that are my own derivation or assumption are labelled **derived** or
  **estimate**. Nothing labelled a measurement was invented, and where I read a value off a 1959 graph by
  eye I say so and give the uncertainty.
- "GR" is gain reduction. "CV" is the DC control voltage that drives the grids. "IM" is intermodulation
  distortion, which is what Fairchild measured; THD is total harmonic distortion, which is what the
  specification sheet quotes. dBm is used as the sources use it, into 600 Ω.
- Component designators are given exactly as the drawings give them, and the *same designator means
  different things on the two drawings*: R32 is the 220 kΩ time-constant resistor on the 660 sheet and a
  bias-string resistor in the 670's power supply. I have named the drawing every time.
- The 670 is two of the 660 on one chassis plus a matrix. Where a statement is true of both I say
  "the Fairchild"; where it is true of one I name the model.
- This is a spoof target, not a parity goal. I want the *feel*: a limiter whose gain element is also its
  amplifier, so that turning it up and dirtying it up are the same knob; six time constants of which two
  are program-dependent in a way no other box in the lab is; and a stereo mode that is really mid-side and
  was built for cutting records, not for mixing. I am not trying to beat Universal Audio, who modelled a
  named golden unit at Ocean Way [34].

**The one thing to carry away before reading any of it.** Every gain element the lab owns is a *separate
part* that the audio passes through on its way somewhere else: a FET channel to ground, a photocell in a
divider, a Blackmer cell, a ring of diodes between two transformers. In all of them the thing that reduces
gain and the thing that adds colour are different components, so the model can tune them independently.
The Fairchild has no such part. **The gain element is the amplifier.** The audio is amplified by eight
6386 triode sections per channel, and the control voltage reduces gain by walking those same triodes down
their own remote-cutoff curve. There is nothing in the signal path to attenuate. It follows, as an identity
rather than a modelling choice, that gain reduction and distortion cannot be separated: they are the same
curve read at two points. Fairchild published a chart in March 1959 that shows exactly this, IM distortion
against decibels of limiting at seven output levels [10], and section 4.6 turns it into the argument that
the whole engine hangs on. If a model of this box lets you have 15 dB of gain reduction cleanly, the model
is wrong.

**A second thing, because it corrects the survey that sent me here.** The survey said the Fairchild's
documentation is "deep on circuit and thin on ground truth", that we "would be checking a model against a
model", and that the only benchmark with numbers in it is Raffensperger's DAFx paper [52]. That was true of
what the survey found. It is not true any more. The December 1959 instruction manual contains **two factory
measurement charts** that the survey did not open: a static input-versus-output family of five curves with
the control positions that produce each one [9], and the IM distortion chart just mentioned [10]. Those are
measurements of the real hardware, published by the people who built it, and they are better ground truth
than anything the Neve or the CL 1B dossiers had. Sections 7.1 and 7.2 read them; section 12 turns them
into tests. The survey's pessimism is superseded and I have said so where it matters.

**A third thing, which is the largest single piece of new work in this file.** Raffensperger publishes a
table of the six time-constant positions as component values [18] and does not say where he got it. I have
now read the same network off the **original Fairchild 660 factory drawing** [5], where the switch
positions are numbered on the sheet, and it agrees with his table in all six positions and all fourteen
component values. That has two consequences. It confirms his table against a primary source, which nobody
has done in print. And it resolves a capacitor whose value the redrawn 670 sheet marks `???` [4]: it is
2 µF at 200 V, C7 on the 660 and C115/C215 on the 670. Section 5.3 shows the working. Section 5.5 then
derives all six published release times from that network and finds a single constant of proportionality,
2.59, that reproduces five of them to within 13 per cent — which tells us what Fairchild's phrase "release
time from 10 dB of limiting" actually meant.

---

## 1. What these units are, and the family

### 1.0 Where the documents are, and how I found them

Everything primary is in the John Leimseider archive on the Internet Archive [11], which is a collection of
scanned service documentation for old studio equipment. The survey listed the identifiers; I downloaded all
of them and I will say what each actually contains, because the titles are misleading in two places.

| identifier | title on the item | what it really is |
|---|---|---|
| `Fairchild_670_owners_manual` [1] | Fairchild 670 owner's manual | The **December 1959 instruction manual**, 16 pages, complete, including the specification sheet, both factory charts and a redrawn schematic. The best single document. |
| `JL10882` [2] | 670 stereo limiting amplifier instructions | The **same manual, a different scan**, at higher resolution on the chart pages. I read the two charts from this one. |
| `JL10878` [3] | 670 Limiter manual | The **same manual again**, a third scan. Useful only for cross-reading OCR ambiguities, which is exactly what I used it for (1.4). |
| `JL10883` [4] | 670 stereo limiting amplifier schematic | A **modern redraw** of the 670 schematic, 4800 × 4056, clean vector-quality line art. Not a scan of a Fairchild original. Beautifully legible and it has one unreadable value on it. |
| `JL10866` [5] | 660 mono limiting amplifier schematic | The **original Fairchild factory drawing**, title block "MODEL 660 AUTOMATIC GAIN CONTROL AMPLIFIER". A 300 dpi bitonal scan of a blueprint. Harder to read and worth every minute. |
| `JL10865` [6] | Fairchild 660 Drawn | A hand-drawn partial sketch of the 660's control circuit. Fragmentary; I got almost nothing from it. |
| `JL10873` [7] | 663 mono compressor schematic | A **different unit**, the solid-state-ish 663, drawing A-9608, issue 1 dated 9/22/69, with transistors (RCA 34966 or 2N1183). Cited only in 1.2 to say what it is not. |
| `JL10876` [8] | 670 Component Layout | A 600 dpi mechanical drawing, "670 LIMITER COMPONENT LAYOUT (REAR VIEW)", with the Fairchild title block. Its value to me is the **address in that title block** (1.4). |

Two documents outside the archive complete the chain. The **General Electric 6386 datasheet ET-T1113** of
August 1954 [12] is the tube's published law, six pages, and it is filed under volume 142 on
`frank.pocnet.net` where no guessed path finds it. And **Raffensperger's DAFx-12 paper** [18] is the only
published circuit model of the 670.

**What I could not reach, said plainly.** Fairchild's own product datasheet, which *Mix* links to as
`/oldmiximage/online_extras/fairchild-datasheet.pdf`, 404s and has no Wayback snapshot. The AES obituary of
Rein Narma (JAES vol. 67 no. 11, November 2019, p. 931) [28] 404s on `aes.org` and the Wayback copy returned
HTTP 429 on every attempt, so I have the citation but not the text. The AES oral history with Narma exists
only as a video [29] and I cannot transcribe it. Universal Audio's 2003 webzine article on the 670, which
Wikipedia cites, is dead and also blocked at the Wayback Machine. Abbey Road Studios' own website says
nothing about the Fairchild anywhere: I fetched their 52-URL sitemap and hand-checked five pages, and the
word does not appear. Reverb serves no listing data to a plain fetch and refused WebFetch with HTTP 403, so
**I have no sale price from an auction or marketplace record**, only prices quoted in articles. And
`fairchildrecording.com` does not resolve at all — the domain is gone.

**The Raffensperger paper is truncated in the copy DAFx serves.** The PDF at the paper-archive URL is six
pages and the text stops mid-sentence in section 6 at "Quantifying the size of the errors caused by the
fictitious delays". Everything I cite from it is in the six pages that exist; the conclusions section I have
not read.

### 1.1 Rein Narma, Sherman Fairchild, and where the design came from

The company is older than the limiter by nearly thirty years. **Sherman Mills Fairchild** (7 April 1896 –
28 March 1971) founded the **Fairchild Recording Equipment Corporation in 1931 in Whitestone, New York**, to
support his interests in photography and image projection [20] [21]. He founded something over seventy
companies including Fairchild Aviation and Fairchild Camera and Instrument, inherited his father's IBM stock
and was IBM's largest individual shareholder until his death [21]. He did not design the limiter and, as far
as any source I reached says, had nothing to do with its circuit.

The limiter is **Rein Narma's**. Narma was Estonian, fled Soviet-occupied Estonia, worked for the US Army as
a broadcast and recording technician during the Nuremberg trials, emigrated to New York, joined Gotham
Recording and co-founded Gotham Audio Developments [22] [23]. He built consoles for Rudy Van Gelder, Olmsted
Recording and Les Paul; the Les Paul desk was built to go with Paul's eight-track Ampex [22] [25]. Les Paul
then asked him for a limiter. Sherman Fairchild, a friend of Les Paul's, heard about the project, **licensed
the design and hired Narma as Fairchild's chief engineer** [19] [22] [23]. Wikipedia adds, uncited to a
document I could check, that **Narma hand-built the first ten 660s himself**, that the first went to Rudy
Van Gelder for cutting lacquers for Blue Note and Vox, the second to Olmsted Sound Studios and the third to
Mary Ford and Les Paul [19].

Two things about that story are worth flagging rather than smoothing over.

**The date is not as firm as it looks.** Every popular source says 1959 and Radiomuseum's model record gives
1959 [14]. My own primary document, the instruction manual, is dated **December 1959** and says on its cover
"Supersedes all previous issues" [1] — so there were earlier issues, and one of the two factory charts is
dated **3/59** [10] while the other says "December 1959 (supersedes March 1959 issue)" [9]. That is
consistent with a product that was already being measured and documented in March 1959. Sound On Sound
dissents further and puts the development "in the early '50s" [26]. What I can say from documents rather
than from folklore: **the 670 was in production and being characterised by March 1959, and the manual I am
working from is the December 1959 issue and supersedes at least one earlier one.**

**What Narma did afterwards is reported inconsistently.** *Mix* says he "moved to Northern California and
was a vice-president at Ampex" [22]. I found no source calling him Ampex's chief engineer, although
Wikipedia's article on John G. McKnight does call him "chief engineer at Fairchild Recording Equipment",
which is the Fairchild job and not the Ampex one [19]. I have used *Mix*'s wording.

### 1.2 The 660, the 670, and the 663 which is not related

The **660** is the mono unit: one channel, four 6386s, one meter. The **670** is two 660 channels on one
chassis plus a matrix switch that turns them into a lateral-and-vertical pair. Wikipedia says the 670 was
"introduced shortly after the 660" [19]; I have no primary document that dates either one against the
other, and the earliest dated Fairchild document I hold is a 670 chart from March 1959 [10].

The **663** is a different animal and it is easy to trip over because its schematic sits next to the others
in the same archive. Drawing A-9608, "663 COMPRESSOR SCHEMATIC (FIG. 5)", issue 1 dated **22 September
1969**, shows a small circuit built around **transistors — "RCA 34966 OR 2N1183"** — with 6.3 V AC heaters
for a valve or two and a THRESHOLD control [7]. It is a decade later than the 660 and it is not a
variable-mu design. **Do not model it and do not cite its component values for anything.** I mention it only
because the survey listed it among the Fairchild documents [52] and someone will otherwise open it expecting
a 660.

### 1.3 What actually differs between the 660 and the 670

This table is read off the two schematics [4] [5] side by side, plus the manual [1]. It is the most useful
single thing in section 1, because almost every popular account treats the two units as the same circuit
twice and they are not.

| | 660 [5] | 670 [4] |
|---|---|---|
| channels | 1 | 2 |
| 6386 gain valves | 4 (V1–V4), 8 triode sections, 4 per push-pull half | 8 (V101–V104, V201–V204), same 4-per-half arrangement in each channel |
| sidechain small-signal valves | V5 12AX7, V6 12BH7A | V105/V205 12AX7, V106/V206 12BH7, one pair per channel |
| sidechain output valves | **2 × 6V6GT** (V7, V8) | **4 × 6973** (V107, V108, V207, V208) [16] |
| plate/cathode resistors on the gain stage | R4, R5 = **1800 Ω 1 W** | R103, R104 = **680 Ω** |
| balance control | R3-A / R3-B, **500 Ω** each | R105a / R105b, **100 Ω** each |
| stabilising capacitors on the gain stage | C1, C2 = **680 pF** | C101, C102 = **4 µF** |
| AC threshold pot | R10-A / R10-B, **180 kΩ** | R115a / R115b, **100 kΩ linear** |
| DC threshold pot | R12, 100 kΩ | R117, 100 kΩ |
| ZERO control | R41, **250 kΩ** | R142 / R242, **2.5 kΩ** |
| meter sense resistors | R6, R7 = 30 Ω 1% | R107, R108 = 30 Ω |
| **time-constant network** | R32 220 kΩ, C7 2 µF, S2 with R33/R34/R35/R37 and C8/C9/C10/C11 | R137 220 kΩ, C115 2 µF, S102 with R138/R139/R140/R141 and C110/C111/C112/C113 — **identical values** |
| HT regulator | series **6BL7** (V9), error amp **6084** (V10), reference 5651 (V11), rectifier GZ34 | series **EL34** (V302), error amp V303, reference 5651 (V304), rectifier GZ34 |
| matrix | none | **S301, ten decks A–K**, LEFT&RIGHT / LATERAL&VERTICAL |
| meter selector | S1, 3 positions: BALANCE, ZERO, BALANCE | S101/S201, same three positions |
| transformers | T1, T2 audio; T3 sidechain output; T4/T5/T6/T7 | T101–T104 and T201–T204 signal and sidechain; T301–T304 supply |

**Read the two rows in bold together, because they are the interesting ones.** The 670 gives each half of
the gain stage a cathode resistor less than half the 660's and bridges the two halves with a capacitor six
thousand times larger. That is a different operating point and a different low-frequency corner, in the one
stage that does all the work. Anyone who tells you the 660 is "the 670 in mono" is describing the block
diagram, not the circuit. The sidechain output valve differs too, a 6V6GT beam pentode against a 6973, and
the 6973 is the more linear and more modern part.

**And the time-constant network is bit-for-bit identical**, which is the single best piece of news in this
file, because it means the six release positions I derive in section 5 are the same six on both units and
the 660 drawing's numbered switch positions can be used to read the 670's unnumbered ones.

### 1.4 The two addresses, and what the manual's OCR gets wrong

Fairchild's paperwork gives two different addresses for the same product, and I can document both from
primary sources, which is more than any secondary account manages.

- The **December 1959 instruction manual** gives, on its cover, "Fairchild Recording Equipment Corporation,
  10-05 5th Avenue, Long Island City 1, New York" [1].
- The **front panel of the unit itself** is engraved "FAIRCHILD  RECORDING EQPT. CORP.  LONG ISLAND CITY 1,
  N.Y.  MODEL 670", which I read directly off two independent photographs [31] [32].
- The **670 Component Layout drawing's title block** says "FAIRCHILD RECORDING EQUIPMENT CORPORATION,
  154 ST. & 7 AVE. WHITESTONE, L.I. N.Y." [8].

Wikipedia and Radiomuseum both give **Whitestone** only [20] [27], and Radiomuseum is the only secondary
source I reached that has the street address, "154 St. and 7th Ave., Whitestone, New York (~1950's)" [27].
So the drawing office was in Whitestone and the manual and the front panel say Long Island City. Both are in
Queens and about five miles apart. **If you draw the faceplate, engrave Long Island City**, because that is
what is on the metal.

**Three OCR readings I checked against three separate scans**, because they matter and the machine text of
each is wrong in the same place:

- The panel size reads `1)" panel space` in one scan [1], `1h!` in another [2] and `1h" pamel space` in the
  third [3]. The glyph read as `h` or `)` is a **4** — the same sentence renders `depth behind panel 11"`
  correctly in all three, so the machine can read a `1`. The manual says **14 inches of panel space**, and
  Wikipedia's "14-inch rack-space unit" [19] is right. **Sound On Sound, Vintage King and Heritage Audio all
  say 6U** [26] [25] [42], which is 10.5 inches, and they are wrong about the original. 14 inches is 8U.
- The tube complement reads `1-608h5` in the JL10878 scan [3] and `1-608)` in the owner's-manual scan [1].
  Same substitution: it is a **6084**. The 660 factory drawing settles it independently by labelling the
  regulator's error amplifier **V10 6084** in plain draughtsman's lettering [5]. Radiomuseum lists it as
  "6064" [14], which is a typo.
- `Do not adjust R313 (B+ ADJ) ... unless the voltage on Pin 8 of V302 (EL34) deviates from 20 V` [1]. The
  schematic annotates that node **(235 Vdc)** and the rail it feeds **(adj to 240V)** [4]. The manual means
  **240 V**; a digit is lost in the scan.
### 1.5 Why it is famous, and which of that is documented

The Fairchild's fame rests on Abbey Road and the Beatles, and the story is better sourced than most studio
folklore but not as well sourced as it is repeated.

**What is documented.** Geoff Emerick, quoted in Mark Lewisohn's *The Complete Beatles Recording Sessions*
(Hamlyn, 1988, p. 72), on the drum sound: *"we put the [drums'] sound through Fairchild 660 valve limiters
and compressors. It became the sound of Revolver and [Sgt.] Pepper really. Drums had never been heard like
that before."* [19] [24]. MusicTech reports that Abbey Road bought Fairchilds after staff engineer **Peter
Bown** heard one at Capitol Records in America, that the unit became the studio's preferred vocal processor,
and that **in 1966 Emerick began using it on Ringo Starr's kit** [23]. Sound On Sound says vocals went
through the Fairchilds from the *A Hard Day's Night* sessions in **1964**, and mentions a specific trick:
the wobbling backing vocals on "Octopus's Garden" were compressed by a 660 whose sidechain was fed with a
pulsing oscillator [26].

**What is not documented, though it is everywhere.** Wikipedia says Abbey Road bought **twelve** 660s and
still had **eight** as of 2014, and cites the MusicTech article for it [19]. I read that article in full.
It says "several", it has no count, and **it contains no sentence about eight surviving units**. Treat both
numbers as unverified. Sound On Sound and Vintage King also disagree about when the Beatles' vocals started
going through the box, 1964 against *Revolver* in 1966 [26] [25]; I have quoted both rather than picking.

**The best first-party statement I found is a photograph.** Abbey Road Studios' own website is silent, but
a 2009 photograph on Wikimedia Commons shows a 660 on display at the studio with the studio's own printed
placard beside it, headed "Fairchild 660 limiter (1960s)" and reading in part: *"The Fairchild has a special
sound that helped create the legendary drum and vocal tones found on recordings by The Beatles. As such,
they are highly prized and quite expensive. Abbey Road still keep their original 1960s units in operation at
the studios every day."* [33] [30]. That is Abbey Road speaking, in their own display copy, and it is the
closest thing to a primary statement anyone is going to get now that their website carries none.

**How many were made, and what one costs.** The sources conflict and none of them shows its working.
Vintage King says "somewhere between five hundred and a thousand" for the 670 and "approximately 800" for
the 660 [25]; Sound On Sound says "around 1000 stereo units were sold in the early years" [26]; Softube says
"fewer than a thousand 670s were ever made" [39]. For price, *Mix* said about **$30,000** in 2007 [22],
Sound On Sound said **upwards of £20,000** in 2016 [26], MusicTech said **at least £10,000 for a good 660**
in 2014 [23] and, in 2019, **£50,000** for a 670 [24]. Wikipedia's price sentence carries a
citation-needed tag dated July 2025 and should not be repeated [19]. **I obtained no auction or marketplace
record.** For a modern comparison that *is* a live published price, POM Audio Design's 2026 list for
new-built 670 mkII units runs **£6,990 to £8,490** depending on transformer provenance [46].

The 670 was inducted into the TECnology Hall of Fame in 2007 [19] [22].

### 1.6 What this dossier is for

The plug-in is a spoof, not a replacement, and it does not need to be right about everything. It needs to be
right about the three things that make this box a different *kind* of machine from the six the lab already
has, which are:

1. **The gain element is the amplifier** (section 4). No separate attenuator exists, so gain reduction and
   harmonic content move together and cannot be given separate knobs without lying.
2. **Two of the six time constants are program-dependent** through real capacitors that charge on a slower
   clock than the main one (section 5). The Neve has auto-release positions too, but they are two-pole
   networks with fixed poles. The Fairchild's slow capacitors change the *impedance* the fast capacitor
   discharges into, so the fast release is fast only while they are empty. That is a genuinely different
   mechanism and section 5.5 shows it accounts for the manual's otherwise baffling claim that position 6
   has a 0.3 second release *and* a 25 second one.
3. **Its stereo mode is mid-side, and it was for cutting records** (section 6). The lab has no matrix
   anywhere.

Everything else — the exact transformer parasitics, the twenty valves, the regulated 240 V rail — is
scenery. I have documented it because the documents are there and because someone reading this later should
not have to re-fetch them, but the DSP design in section 10 spends its budget on the three things above.

---

## 2. Controls, the front panel, and enough geometry to draw a faceplate

### 2.1 The controls, from the manual

The manual lists them exactly, and the count matters because Universal Audio's 2004 press release describes
the 670 as having "seven dual controls" [36], which is the same inventory counted a different way [1]:

> a) 2 Input Gain Controls — Step attenuator: 1 db per step.
> b) 2 Threshold Controls — Continuously variable.
> c) 2 Time Constant Switches — 6 positions each, so as to provide fixed and variable time constants for
> any type of program material.
> d) 2 Metering Switches — 3 positions each, which permits the measurement of plate current of each set of
> control tubes.
> e) Mode Switch — Left-Right position: 2 independent limiters. Vertical-Lateral position: matrixing input
> and output, left and right in and out, limiting action vertical-lateral.
> f) ON-OFF Switch

The schematic adds the trimmers, and names them [4]:

| designator | control | how it is operated |
|---|---|---|
| AT101 / AT201 | INPUT GAIN | knob, front panel, step attenuator, 1 dB per step |
| R115a+b / R215a+b | AC THRESHOLD | knob, front panel, 100 kΩ linear, ganged pair |
| S102 / S202 | TIME CONSTANT | knob, front panel, 6 positions |
| S101 / S201 | METERING | lever switch, front panel, 3 positions: BALANCE, ZERO, BALANCE |
| S301 | matrix / AGC mode | toggle, front panel, LEFT-RIGHT or LAT-VERT |
| R105a+b / R205a+b | BALANCE | **screwdriver, front panel**, 100 Ω ganged |
| R142 / R242 | ZERO | **screwdriver, front panel**, 2.5 kΩ |
| R117 / R217 | DC THRESHOLD | **screwdriver, inside the chassis** |
| R313 | REG B+ ADJ | screwdriver, inside the chassis, do not touch |

The 660 drawing carries the same legend with its own designators — S2 TIME CONSTANT POS. 1-6; S1 METER
SELECTOR POS. 1 BALANCE, 2 ZERO, 3 BALANCE; AT-1 INPUT GAIN; R3 BALANCE; R10 THRESHOLD; R12 DC THRESHOLD;
R41 ZERO — and a symbol key that tells you where each adjustment lives: a hollow circle is a knob on the
front panel, an open screwdriver slot is a screwdriver adjustment on the front panel, and a filled one is a
screwdriver adjustment at the rear of the chassis [5]. On the 670 the corresponding note reads "SCREWDRIVER
ADJUSTMENT FRONT PANEL" and "SCREWDRIVER ADJUSTMENT BENEATH FRONT PANEL" [4], and the manual says the DC
THRESHOLD "is located inside the chassis as a screwdriver adjustment" [1].

**The DC THRESHOLD being inside the box is the single most consequential fact on this list**, because it is
the ratio and knee control (5.2), it is not on the panel, and every emulation that is any good brings it
out to the front. Overloud say so explicitly: *"A trimmer inside the original unit allowed to change the
compression characteristic from soft-knee to hard-knee by changing the biasing current of the feedback
amplifier. The [Comp670] brings this control on the front panel"* [40]. Sound On Sound independently says
"the curve can be adjusted to some degree by way of an internal trim pot" [26]. And the manual's own
calibration procedure is entirely about setting it (7.2).

### 2.2 The panel, measured

**Overall size, from the manual [1]:** standard 19-inch rack, **14 inches of panel space** (8U), depth
behind the panel **11 inches**, weight **approximately 65 lb**. Sound On Sound gives 30 kg, which is the
same weight [26]; their 6U is wrong (1.4).

I worked from two photographs of real units, both CC BY 2.0 on Wikimedia Commons [31] [32], and I did the
measuring on the second because it is lit more evenly and shows the whole plate square-on enough to
rectify. I found the plate's four corners in the source photograph, fitted a projective homography from
those corners to a 19 × 14 rectangle, and resampled at 100 pixels per inch. The rectified image is saved as
`ref/fairchild-670-panel-rectified.png` and the same image with a one-inch grid drawn on it as
`ref/fairchild-670-panel-inch-grid.png`; every position below was read off that grid.

**How much to trust it.** The rectification is anchored on corners I located to about ±10 pixels in a
2048-pixel-wide photograph, which is about ±0.1 inch on the panel, and the right-hand side of the plate was
the far side of the shot so it is softer and slightly more stretched than the left. Positions below x = 16
inches are good to roughly **±0.15 inch**; the two TIME CONSTANT columns at the right edge are good to
about **±0.3 inch**. One consistency check passed: the two meter bezels come out symmetric about the panel's
horizontal centreline, with the upper bezel's top 0.85 inch from the top edge and the lower bezel's bottom
1.1 inch from the bottom edge. One check was inconclusive: small round features (the fuse cap, the
screwdriver adjustments) do not come out perfectly circular, with width-to-height ratios scattered between
0.70 and 1.07, which is what small dim features in an underexposed photograph do and is not enough evidence
to overturn the manual's 19 × 14.

Origin is the top-left corner of the panel; x runs right, y runs down, both in inches.

| element | centre (x, y) | size | notes |
|---|---|---|---|
| ON toggle | 0.9, 3.3 | bat ≈ 0.5 wide | chrome bushing |
| upper meter bezel | 3.7, 3.4 | 3.95 × 5.05 | black moulded surround, recessed |
| upper meter dial | 3.7, 2.65 | 3.05 × 2.5 | cream face, arc scale |
| round screw | 0.85, 6.7 | ø 0.6 | |
| FUSE cap | 0.95, 10.2 | ø ≈ 0.85 | "FUSE" moulded round the cap, "5A" engraved below at y ≈ 11.2 |
| lower meter bezel | 3.7, 10.35 | 3.85 × 5.1 | |
| LEFT-LAT METERING lever | 7.3, 3.2 | lever ≈ 0.6 × 1.9 | tall black lever, not a round knob |
| ZERO screwdriver (upper) | 6.2, 6.2 | ø 0.35 | label "ZERO" at 6.25, 5.75 |
| BAL screwdriver (upper) | 8.35, 6.25 | ø 0.35 | label "BAL" at 8.4, 5.75 |
| LEFT CHANNEL INPUT GAIN | 10.6, 3.4 | knob ø ≈ 1.5, scale ring ø ≈ 3.1 | |
| LEFT-LAT THRESHOLD | 13.9, 3.5 | knob ø ≈ 1.5, scale ring ø ≈ 2.6 | |
| LEFT-LAT TIME CONSTANT | 17.2, 2.8 | knob ø ≈ 0.9, scale ring ø ≈ 1.7 | sits ~0.6 higher than its row |
| AGC toggle | 17.3, 6.0 | | "AGC" at 17.2, 5.1 |
| RIGHT-VERT METERING lever | 7.5, 9.6 | | |
| ZERO screwdriver (lower) | 6.3, 13.0 | | label at 6.4, 12.55 |
| BAL screwdriver (lower) | 8.5, 13.05 | | label at 8.6, 12.55 |
| RIGHT CHANNEL INPUT GAIN | 10.7, 9.7 | | |
| RIGHT-VERT THRESHOLD | 14.1, 9.7 | | |
| RIGHT-VERT TIME CONSTANT | 17.6, 10.5 | | |
| FAIRCHILD script logo | 10.4–12.9, 12.2–12.9 | | italic script, capital F with a long descender |
| address block | 12.9–14.3, 12.3–12.7 | | two small lines, right of the logo |
| MODEL 670 | 14.4–15.9, 12.5–12.8 | | |

**The layout in one sentence, for whoever draws it.** Two identical channel rows, upper for LEFT-LAT and
lower for RIGHT-VERT, each running left to right as meter → metering lever → input gain → threshold → time
constant; the meters stacked in a recessed panel down the left with the mains toggle above and the fuse
below them; the mode toggle alone in the vertical gap between the two rows at the far right; and the maker's
name across the bottom right. The panel corners are gently rounded.

**Two deliberate asymmetries to keep.** The TIME CONSTANT knobs are noticeably smaller than the INPUT GAIN
and THRESHOLD knobs, and they sit vertically offset from their own rows — the upper one about 0.6 inch high
and the lower one about 0.8 inch low — to open a gap in the middle of the right-hand column for the AGC
toggle. And the two channels' controls are *not* on identical x centres: the lower row sits between 0.1 and
0.4 inch to the right of the upper. On a real hand-drilled 1959 panel that is probably tolerance rather than
intent, but it is what the metal does and reproducing it is free.

### 2.3 The silkscreen, exactly

Read from the two photographs [31] [32], which agree everywhere. All caps, sans-serif, condensed, except
the logo.

Upper row, in order:

```
LEFT-LAT              LEFT CHANNEL          LEFT-LAT              LEFT-LAT
METERING              INPUT GAIN            THRESHOLD             TIME CONSTANT
```

with `BAL   ZERO   BAL` arced above the metering lever, `ZERO` and `BAL` engraved below it beside their
screwdriver slots, the input-gain ring numbered `0 2 4 6 8 10 12 14 16 18 20` with unnumbered dots between
(21 detents, 1 dB each), the threshold ring numbered `0 1 2 3 4 5 6 7 8 9 10`, and the time-constant ring
numbered `1 2 3 4 5 6`.

Lower row, identically laid out:

```
RIGHT-VERT            RIGHT CHANNEL         RIGHT-VERT            RIGHT-VERT
METERING              INPUT GAIN            THRESHOLD             TIME CONSTANT
```

Between the rows at the right:

```
              AGC
LEFT                 LAT
RIGHT                VERT
```

with a small drawn bracket linking each pair of words to its end of the toggle throw. Top left, above the
mains toggle: `ON`. Left, below the upper meter: `FUSE` moulded around the cap and `5A` engraved beneath.
Each meter dial is printed `VU` at both ends of the scale and `FAIRCHILD` across the lower half of the face.

Bottom right, on three parts:

```
FAIRCHILD          RECORDING EQPT. CORP.        MODEL 670
                   LONG ISLAND CITY 1, N.Y.
```

`FAIRCHILD` is the script logo, much larger than everything else, italic, with an oversized `F`. The two
address lines are small and set tight against the logo's right shoulder. `MODEL 670` is the same height as
the address lines but tracked wider.

### 2.4 Colour

Measured from the rectified photograph with a script, sampling means over small patches; the values are in
`ref/fairchild-670-panel-rectified.png` and reproducible from it.

| element | measured | what to actually use |
|---|---|---|
| panel field | `#030401` (darkest 30 % of a clear area) | **near-black**; see the caveat |
| knob bodies | `#010100` | black, slightly glossier than the panel |
| silkscreen | `#D8D0C9` – `#E4DCD6` (brightest 5 % inside a glyph) | **white** |
| meter dial face | `#F3E5D5` | warm cream |
| meter over-zero arc | `#D7A696` | red |
| script logo | `#D5CCC3` | white, same ink as the rest |

**The caveat, which matters if anyone builds a faceplate from these numbers.** The photograph is
underexposed on the panel: the darkest one per cent of the image sits at luminance 0.7 out of 255, so the
black is clipped and I cannot tell you whether the real panel is a true black, a very dark grey or a black
wrinkle finish. The whole image also has a warm cast, which is why the "white" silkscreen measures warm
off-white and the red arc measures a washed salmon. **What is safe to take from this table is the relative
ordering and the meter's cream face**, which is well exposed and clearly warmer than the silkscreen white.
For everything else use a neutral near-black panel with white legends and a red over-zero arc, which is what
a VU meter is by convention, and treat my hex values as evidence rather than as the answer. Both photographs
show a satin rather than gloss panel with visible fine scratching, so a small amount of surface noise on the
faceplate is truer than a flat fill.

### 2.5 The meter, and what it actually reads

Standard VU scale, printed `VU` at both ends, running `-20 -10 -7 -5 -3 -2 -1 0 +1 +2 +3` on the upper arc
and `0 20 40 60 80 100` on the lower, with the arc from 0 to +3 in red [31] [32]. `FAIRCHILD` across the
lower face.

**But it is not a programme meter and it is not a gain-reduction meter.** The METERING switch has three
positions, BALANCE / ZERO / BALANCE [1] [4] [5], and the manual's balancing procedure is: warm up for half
an hour, adjust BALANCE until the same reading is obtained in both BALANCE positions, then set the switch to
ZERO and adjust the ZERO control for a reading of zero, then repeat; and if that does not converge, change
6386s [1]. So the meter is a valve-current bridge across the push-pull output stage. POM Audio Design, who
build new 670s, describe the three positions in more detail than any other source I reached:

> The Fairchild metering allows you to measure the current that passes through both legs of the output
> transformer's primary windings, it is a bit like a tube tester… a) The "Push" side current of the output
> stage by flicking the "BAL" switch to the Left, b) The overall current that is flowing through the centre
> tap of the output transformer in the "ZERO" position, c) The "Pull" side current of the output stage by
> flicking the "BAL" switch to the Right [45].

And they make a point that is easy to miss and important for the model:

> People do not realise that when you adjust the "ZERO" on any FAIRCHILDs, you are actually changing the
> Biasing of the tubes, you are not changing the Vu calibration by design [45].

That is corroborated by the circuit: R142/R242 sits in the return of the time-constant network and its wiper
sets the standing grid bias, which is Raffensperger's `V_bias = −7.2 V` [18] [4]. **ZERO is a bias trim
wearing a meter-calibration label.** In the plug-in it should move the operating point, not the meter.

Once the unit is balanced and zeroed, deflection from zero during operation is the *change* in control-valve
current, which is gain reduction — so in normal use the needle does read gain reduction, but as a
consequence of the bridge going out of balance rather than because anything measures it. Universal Audio
removed the metering positions in their emulation because "the software version of the meters have the
benefit of not requiring recalibrating", replacing them with input and output metering and disabling the
zero screw [36]. That is a defensible choice and I would make the opposite one (10.6).

### 2.6 The rear

From the manual [1], the terminal strip:

| terminal | function |
|---|---|
| 1 | left channel input, tie point, no internal connection |
| 2, 3 | left channel input high, low (600 Ω) |
| 4 | ground (chassis) |
| 5 | right channel input, tie point, no internal connection |
| 6, 7 | right channel input high, low (600 Ω) |
| 8, 9 | left channel remote meter |
| 10, 11 | right channel remote meter |
| 12, 13 | left channel output high, low (600 Ω) |
| 14 | ground (chassis) |
| 15, 16 | right channel output high, low (600 Ω) |
| 17 | ground (chassis) |

With two warnings that tell you something about the design. "If a floating circuit is used, it is necessary
that Terminals 3 and 7 be strapped to Terminal 4 (chassis)". And, in capitals, "CAUTION: Terminals 8, 9, 10
and 11 are 20 V above ground", which the remote-meter drawing on the same page restates as "REMOTE METER
LEADS ARE APPROX 250V ABOVE GROUND" [1]. Both numbers are in the same manual and they do not agree; the
meter bridge sits in the plate circuit at a couple of hundred volts, so the drawing's figure is the
plausible one and the specification page has lost a digit again.

Any standard VU meter works as a remote meter if its internal rectifier is removed, with about 10 kΩ of
variable resistance (an Ohmite CLU-1031) to match its zero to the internal meter's; Fairchild would sell you
one as part A-23294 [1].

**Remote threshold control** is possible without modifying the unit, using ganged 20-step 20 dB bridged-T
faders at the input and output, at a cost of "approximately 10 to 16 db" of level which Fairchild suggests
making up with their Model 722 line amplifier [1]. That is a period detail with no bearing on the model, but
it does tell you the threshold control is really a level control in a feedback loop, which is section 5.2.
### 2.7 The 660's panel

The Abbey Road photograph [33] shows a 660 only partly, below its display placard, but enough is visible to
confirm the family resemblance and the differences: **one meter, not two**; the same `BAL ZERO BAL` metering
lever; INPUT GAIN, THRESHOLD and TIME CONSTANT in the same order; `ON` at the left; and the same script
FAIRCHILD logo at the bottom right. There is no AGC toggle because there is nothing to matrix. The panel is
the same black with white legends. I did not find a photograph square-on enough to measure, so **I have no
660 faceplate geometry** and I would draw it as one row of the 670 on a shorter panel and say so.

### 2.8 What I could not establish about the panel

Two things, and I would rather leave them open than guess.

**Whether the engraved black plate is the whole 19 × 14 panel.** In the higher-resolution photograph [31]
the black plate is clearly inset in a lighter grey surface that carries a warning plaque and another toggle,
and that grey surface has rack ears with screws. In the other photograph [32] the black plate appears to run
to the edges. The two units are in different mountings and I could not tell from either whether the grey is
part of the unit or a cabinet somebody built. The manual gives 19 × 14 for the unit and says nothing about a
sub-panel. If the plate is inset, my inch positions in 2.2 are still correct *relative to the plate*, which
is what a faceplate needs.

**The knob styles.** The INPUT GAIN and THRESHOLD knobs are large black skirted knobs with a white pointer
dot on the skirt, and there is a second, smaller concentric black cap on top of each; whether that is a
two-part knob or a knob plus a retaining cap I cannot tell from a photograph. The METERING control is not a
knob at all but a tall black lever with a white arrow tip, of the kind used on rotary lever switches. The
TIME CONSTANT control is a small skirted knob with a pointer wing.

---

## 3. Signal path and circuit, read from the drawings

### 3.1 The shape of the thing, and that it is a feedback design

One channel is four blocks and a loop:

```
in ─ 600Ω ─ AT101 ─ T101 ─┬─ [ eight 6386 sections, push-pull ] ─ T102 ─┬─ out
       step atten  input   │       the gain element AND the amplifier    │  600Ω
                  xfmr     │                                            │
                           │                                            │
                     grid bias                                    R145..R150 pad
                           │                                            │
                    R142 ZERO ── S102 time constants ── rectifier ── T104 ── 6973 ── 12BH7 ── 12AX7 ── R115 AC THRESHOLD ── T103
```

Read that loop anticlockwise from the output and it says: **the sidechain listens to the output, not the
input.** Raffensperger states it in his first figure caption — *"The Fairchild 670 circuit is a feedback
design as the 'sidechain' amplifier monitors the output voltage to control the gain of the signal
amplifier"* [18] — Sound On Sound says "the side-chain signal is tapped after the gain cell" [26], and POM
Audio Design, who build the things, describe their optional feed-forward blend by reference to it: mix
control all the way up is "equivalent to using a Feed-Back Compressor (The way the FAIRCHILD 670 design
normally is)" [45]. The schematic confirms it directly: the pad that feeds the sidechain input transformer
T103 hangs off T102's secondaries, which are the output [4]. Three independent sources and the drawing.

This matters more here than it does in most feedback compressors, because the loop closes around a
nonlinearity that *is* the amplifier. In the 1176 the feedback loop encloses a FET whose distortion is a
small side effect; here it encloses the whole gain stage, so the loop linearises the stage as well as
controlling it, and the observed distortion is what survives that linearisation. That is one reason the box
is as clean as it is at low gain reduction and gets dirty so decisively when it runs out of loop gain
(4.6, 7.2).

### 3.2 The audio path, component by component

Channel 1 of the 670, reading the redrawn schematic left to right [4]. Channel 2 is identical with 2xx
designators.

**Input.** From the terminal strip through the matrix switch decks S301A/S301B, then **R101 and R102, both
150 Ω 1 %**, into the primary of **T101**. The two 150 Ω resistors are the 600 Ω termination split either
side of the line. T101's secondary is centre-tapped and the centre is grounded through the shield.

**The gain stage.** T101's secondary drives the grids of the two halves differentially. Each half is **four
6386 triode sections in parallel**: the upper half is V101a, V102a, V103a and V104a and the lower half is
V101b, V102b, V103b and V104b, so all four envelopes contribute one section to each side of the push-pull
[4]. Sound On Sound describes exactly this: *"each channel employs no fewer than four 6386 dual-triode
valves. This, in turn, means that each half of the push/pull stage relies on four triode elements wired in
parallel. This amplification circuitry behaves like a single gain stage, albeit one with a very low
impedance"* [26]. The plates go to the two ends of T102's primary, whose centre tap is annotated **(240 V)**;
the plate rail itself is annotated **(230 V)**, and the ten volts between them is the drop through the
primary and the sense resistor. The cathodes return through **R103 and R104, 680 Ω each**, meeting at the
**R105a/R105b 100 Ω BALANCE pot**, and **C101/C102, 4 µF**, bridges the two cathode nodes.

That 4 µF is doing the most under-appreciated job on the sheet. It makes the two halves share an AC cathode
point, which turns what would otherwise be a cathode-coupled pair into a true push-pull stage, and it is
what sets the low-frequency corner. Raffensperger names it `C_1 = 4 µF` and says outright that *"the low
frequency response of the signal amplifier is determined by the bypass capacitor C1 = 4 µF"* [18].

**Metering.** **R106 4.7 kΩ** in series with the meter, **R107 and R108, 30 Ω** as the sense resistors in
the two plate legs, and **S101** selecting between them and the centre tap. Raffensperger models these as
`R_12 = R_22 = 33 Ω` [18], which is the value of R111/R211 on the schematic rather than R107/R108; the
difference is immaterial and I mention it only so nobody thinks one of us misread.

**Output.** **T102**, two secondaries: one to the output terminals at 600 Ω, one to the sidechain pad.

**The sidechain pad, which also fixes Raffensperger's numbers.** Each T102 sidechain secondary feeds a pad
of two 150 Ω series resistors into a 680 Ω shunt: **R145 and R146 (150 Ω) with R149 (680 Ω)**, and
**R147 and R148 (150 Ω) with R150 (680 Ω)**, and the two pads feed the two primary windings of **T103**, the
sidechain input transformer [4]. Raffensperger's sidechain input model uses `R_in = 600 Ω` and
`R_term = 1360 Ω` [18]. Four 150 Ω resistors are 600 Ω and two 680 Ω resistors are 1360 Ω. **Those are the
same components**, and finding that correspondence is how I satisfied myself that his simplified diagrams
are honest reductions of this drawing and not a different unit.

T103 steps up hard: Raffensperger gives `N_p/N_s = 1/17` [18].

### 3.3 The 660's audio path, and where it is genuinely different

Same shape, different numbers, read off the factory drawing [5]:

- Input through **AT-1** (a three-terminal step attenuator drawn as a box marked IN / OUT / COM) into **T1**.
- T1's secondary ends go to the grids through **R1 and P2, both 100 kΩ 1 %**, whose junction is where the
  control voltage enters. **This is the clearest statement anywhere of how the control voltage reaches the
  grids** and it is why I read the 660 sheet at all; the 670 redraw does not label the corresponding pair.
  Raffensperger's `R_L1 = R_L2 = 100 kΩ`, which he describes as modelling "both the resistive component of
  the sidechain and the impedance of the grids" [18], is these two resistors.
- Four 6386s: **V1-A, V2-A, V3-B, V4-B** in the upper half and **V1-B, V2-B, V3-A, V4-A** in the lower, with
  the pin numbers drawn (2/3/4 for one section, 6/7/8 for the other, matching the GE basing [12]). The
  plates all tie to a single top rail.
- **R4 and R5, 1800 Ω 1 W**, as the cathode resistors, with **R3-A / R3-B, 500 Ω** as BALANCE between them,
  and **C1 and C2, 680 pF** in the same region.
- **T2** out, with **R6 and R7, 30 Ω 1 %**, **R57 2500 Ω 1 %** and **R58 33 Ω** in the meter bridge and
  **M1** the meter.
- **R54 330 Ω, R55 330 Ω and R56 1200 Ω** around **T7**, the sidechain input transformer.

So the 660 runs 1800 Ω cathode resistors against the 670's 680 Ω, and has 680 pF where the 670 has 4 µF.
I have not been able to establish from the drawings alone whether C1/C2 on the 660 occupy the same node as
C101/C102 on the 670 — the 660 blueprint is faded in exactly that region — so **I will not claim the 660 has
a six-thousand-times-smaller cathode bypass.** What I will claim, because it is unambiguous on both sheets,
is that **the cathode resistors differ by a factor of 2.6** and therefore the two units do not share an
operating point. If the plug-in offers a 660 model, that is the difference to implement, and section 10.4
records it as the one 660-versus-670 constant I am confident in.

### 3.4 The power supply

Not modelled, documented so that nobody has to open the drawing again [4] [5].

The 670: mains through **S302** and fuse **F301** to **T302**, whose 400 V AC winding feeds **V301, a GZ34**
full-wave rectifier — the redraw labels it **6Z34**, which is not a type; Radiomuseum notes the same error
and adds that the original schematic says 6234, and that a 5V4GA is a stated substitute [14]. Then **C304
4 µF/440 V**, **R305 and R306 100 kΩ 1 W**, choke **L301**, and **C305/C306, 140 µF/350 V**, giving an
unregulated **420 to 450 V**.

That feeds a series regulator: **V302, an EL34**, its cathode annotated **(235 Vdc)** and the rail it
delivers **(adj to 240 V)**, controlled by **V303**, the **6084** error amplifier, referenced to **V304, a
5651** gas reference tube annotated **(86 V)**. The divider is **R310 100 kΩ, R311 270 kΩ, R312 150 kΩ 1 W,
R313 27 kΩ 1 W, R313 150 kΩ** (the designator R313 appears twice on the sheet, once as a fixed resistor and
once as the B+ ADJ pot — a drawing error, not mine), **R314 and R315, 100 kΩ 1 W**, with **C307a and C307b,
60 µF/450 V**.

A separate negative supply: **T301** into a bridge, choke **L302, 1.5 H at 200 mA**, **C302 250 µF/50 V**,
**C303 1000 µF/15 V**, **C308 250 µF/50 V**, **C309 25 µF/25 V**, **R302 1 kΩ 1 W**, **R302 10 kΩ 5 %**
(again a repeated designator) and **R303 27 kΩ 5 %**, annotated **−17 V** at one node and **−13 V to
−14.5 V** at the output. **T304** provides a 6.7 V AC winding with **R307 100 kΩ**. **T303**, a **6.3 V
2.5 A** transformer, runs the 6386 heaters at an annotated **6.6 V AC**, with the small-signal and 6973
heaters on **6.3 V AC** from another winding.

The 660 does the same job with **V12 GZ34 (5V4GA)**, a **6BL7** series regulator (V9) [17], a **6084** error
amplifier (V10) and a **5651** reference (V11), with **C13/C18 140 µF/350 V**, **C12-A 60 µF/450 V**,
**C12-B 60 µF/350 V**, **R51 1500 Ω 5 W**, **R48 220 Ω 1 W**, **R49 6.8 Ω 1 W** and **C17**, which note 3 on
the drawing says is "supplied as part of T5 & has value of .5/660 VAC" [5].

**Why this is worth even a paragraph.** The heater supply is regulated, the HT is regulated to 240 V by a
valve loop with a gas reference, and the manual claims the unit "maintains stability of gain, gain reduction
and balance over a range of line voltage fluctuations from 100 to 127 volts" [1]. So **there is no sag to
model**. Whatever the Fairchild's character is, it is not a wobbling power rail, and a model that adds
supply droop for flavour is adding something Fairchild spent four valves removing.

### 3.5 The metering circuit, and what ZERO really does

Covered in 2.5 from the operator's side; here is the circuit. **R107 and R108, 30 Ω**, sit in the two plate
legs; **R106, 4.7 kΩ**, is the meter's series resistor; **S101** selects one leg, the centre tap, or the
other leg [4]. The 660 adds **R57, 2500 Ω 1 %**, and **R58, 33 Ω**, in the same bridge [5].

**R142 (R242 on channel 2), 2.5 kΩ**, labelled ZERO in the manual's control list [1], sits at the *cold* end
of the time-constant network, between it and the supply return, with **C115** across that node. So the whole
control voltage developed across R137 and the switched network is referred not to ground but to R142's
wiper, and moving that wiper moves the standing grid bias of all eight 6386 sections. Raffensperger models
this as a constant, `V_bias = −7.2 V`, and says so: it is "the bias voltage produced by the 'zero' adjust
potentiometer on the front panel" [18]. POM's blunt version — *"when you adjust the 'ZERO' on any
FAIRCHILDs, you are actually changing the Biasing of the tubes, you are not changing the Vu calibration"*
[45] — is the same statement in service-engineer language.

**Consequence for the model.** If the plug-in exposes ZERO, it must move the operating point on the tube
curve, which changes the standing gain, the available gain reduction *and* the standing distortion together.
That is a genuinely interesting control and it is the honest version of the "Headroom" knob that Universal
Audio added [34] and the "calibration knob to influence the unique compression-dependent saturation" that
Softube added [39]. Both vendors invented a control that the hardware already had, hidden under a
misleading label.

### 3.6 Counting the valves and transformers, and correcting the published counts

**Valves.** The manual's own list [1] [3], with the OCR resolved as in 1.4:

| type | count | where |
|---|---|---|
| 6386 | 8 | the gain stage, four per channel |
| 12AX7 | 2 | sidechain first stage, one per channel |
| 12BH7 | 2 | sidechain second stage, one per channel |
| 6973 | 4 | sidechain output, two per channel |
| EL34 | 1 | HT series regulator |
| 6084 | 1 | HT error amplifier |
| 5651 | 1 | HT voltage reference |
| GZ34 (5V4) | 1 | HT rectifier |
| **total** | **20** | |

Twenty, which agrees with Wikipedia, Vintage King, Sound On Sound and POM [19] [25] [26] [45]. *Mix* says
"20 vacuum tubes (or 21 if you include the 5V4 rectifier)", which double-counts the rectifier the manual has
already listed [22]. Sound On Sound's "20 valves with 30 systems" [26] counts triode and pentode sections:
eight 6386 twin triodes are 16 sections, two 12AX7 and two 12BH7 are 8 more, four 6973 are 4, and the
regulator's four are 4 or 5 depending on how you count the GZ34's two diodes — so 30 is the right order and
the exact figure depends on the convention.

**Transformers.** I counted them on the drawing rather than trusting anybody: **T101, T102, T103, T104,
T201, T202, T203, T204, T301, T302, T303, T304 — twelve** — plus **two chokes, L301 and L302** [4]. Wikipedia
and Vintage King say "11 transformers, and 2 inductors" [19] [25], Sound On Sound says 11 [26] and *Mix* says
14 [22]. My twelve counts the mains transformer, the heater transformer and both bias-supply transformers;
eleven is what you get if you leave out one of those, and I would guess the heater transformer T303. **The
count that matters to the model is four signal-path transformers per channel** — input, output, sidechain
input, sidechain output — and that is not in dispute.

**Heritage Audio's "22 valves and 9 transformers"** [42] describes their own clone, not the original, and
should not be quoted for the Fairchild.
---

## 4. The variable-mu gain element

This is the section the rest of the file exists to support. The lab has never modelled one of these and
nothing it already owns transfers.

### 4.1 What "remote cutoff" means, and why an ordinary triode will not do

An ordinary small-signal triode — a 12AX7, the one in the 610 preamp — has a grid that is a uniform helix.
Every part of the grid is the same distance from the cathode with the same pitch, so every part of it starts
cutting off the electron stream at about the same grid voltage. Push the grid a few volts negative and the
plate current collapses over a narrow range; the tube goes from amplifying to cut off in maybe ten volts,
and its amplification factor barely changes on the way. That is why the mu of a 12AX7 is quoted as a single
number.

A **remote-cutoff** (or "variable-mu", or in the older literature "supercontrol") triode has a grid wound
with **deliberately varying pitch**. The turns are close together over most of the grid's length and spread
apart in the middle. The close-wound sections cut off early, at a few volts negative; the open-wound section
keeps passing electrons until the grid is tens of volts negative. So as the grid goes down, the tube does
not switch off — it *progressively* switches off, one region of the grid at a time, and what falls smoothly
over that whole range is the **transconductance**, and with it the amplification factor. The tube's mu is
not a number; it is a function of bias.

General Electric say it in the first paragraph of the 6386 datasheet:

> The 6386 is a miniature medium-mu twin triode in which each section exhibits a remote-cutoff
> characteristic. It is designed primarily for use as a cascode radio-frequency, intermediate-frequency
> amplifier, or mixer in circuits to which it is desired to apply automatic-gain-control. [12]

That last clause is the whole story. **The 6386 is an automatic-gain-control tube.** It was designed to have
its gain moved by a DC voltage on its grid, for radio receivers, and Narma used it in an audio limiter.
Radiomuseum classifies it as "Double Triode Controlling (mu)", registered on the Electron Tube Registration
List on **2 October 1953**, "computer rated, remote cutoff medium mu twin triode, derived from 2C51, Mu 17"
[13].

**This is why Raffensperger had to build a new tube model.** He says it plainly:

> This unusual property is called 'remote cutoff' because the grid-cathode voltage must take an unusually
> large negative value to get a small anode-cathode current, as shown in Fig. 2. Existing triode models
> [4, 5] were designed for tubes like the 12AX7 which do not have the remote cutoff characteristic of the
> 6386. [18]

So the triode model in the lab's `dsp::pre`, which was fitted for the 610's 12AX7-class valves, **does not
transfer**. Not "needs different parameters" — the functional form is wrong. That is worth being blunt
about because the components-crate README currently lists "tube stage and transformer" as a coming shared
component partly on the grounds that a variable-mu unit would want them [51], and half of that is true (the
transformer) and half is not (12).

### 4.2 The published numbers that bound the model

Everything here is from the General Electric datasheet ET-T1113, dated August 1954, with the characteristic
curves themselves dated 21 August 1953 [12].

**Ratings, each section:**

| quantity | value |
|---|---|
| plate voltage, design centre | 300 V |
| plate dissipation | 1.5 W |
| DC cathode current | 18 mA |
| heater-cathode voltage, either polarity | 90 V |
| heater | 6.3 V ±10 %, 0.35 A |

**Class A1 amplifier, each section, and this block is the one that matters:**

| quantity | value |
|---|---|
| plate voltage | 100 V |
| cathode-bias resistor | 200 Ω |
| **amplification factor** | **17** |
| plate resistance, approximate | 4250 Ω |
| **transconductance** | **4000 µmhos** |
| plate current | 9.6 mA |
| **grid voltage for gm = 100 µmhos** | **−16 V** |

**Read the last two rows together and you have the gain-control range of this tube as a published number.**
Transconductance falls from 4000 µmhos at the operating point to 100 µmhos at −16 V on the grid, both stated
in the same block under the same conditions. That is a factor of 40, which is

```
20 · log10(4000 / 100) = 20 · log10 40 = 32.04 dB
```

**32 dB of gain range, published by General Electric in 1954, across a 16 volt swing on the grid.**
(**Derived** only in the sense that I took the logarithm; both endpoints are printed.) It is the single most
useful number in this file for the gain element, because it says how much authority the control voltage has
and it can be tested (12.2, test 6). The 670 uses at most about 20 dB of that in practice, from the static
curves (7.2), so the tube is working comfortably inside its range and the model should not be running out of
law at the bottom.

**Interelectrode capacitances, each section:** grid-to-plate 1.2 pF, input 2.0 pF, output 1.1 pF,
heater-to-cathode 2.6 pF, grid-to-grid 0.003 pF, plate-to-plate 0.11 pF (without external shield) [12]. With
four sections in parallel per half, the input capacitance per half is about 8 pF and the Miller capacitance
about 4.8 pF before gain multiplication; at the impedances here that is inaudible and I list it only so that
nobody has to go and look.

**One more published distortion figure, from the cascode block:** third harmonic distortion **0.5 %** at
`Esig = 1.0 V` peak [12]. That is a cascode RF condition and not the Fairchild's circuit, so it cannot be
asserted of the plug-in, but it establishes the order of magnitude of the tube's own nonlinearity at a volt
of grid drive.

**Basing** is RETMA 8CJ, nine-pin miniature: pin 1 heater, 2 cathode (section 2), 3 grid (section 2), 4 plate
(section 2), 5 internal shield, 6 plate (section 1), 7 grid (section 1), 8 cathode (section 1), 9 heater;
"it is recommended that Pin 5 be grounded" [12]. The 660 drawing uses exactly these numbers on its tube
symbols, which is how I know Fairchild's "A" section is GE's section 2 and "B" is section 1 [5].

**A modern part exists.** JJ Electronic sell a 6386 LGP and publish typical characteristics of
`Ua = 100 V, Rk = 200 Ω, Ia = 9.6 mA, S = 3 mA/V, Ri = 6 kΩ, μ = 18` [15]. Note their gm is **3000** µmhos
against GE's 4000 and their mu 18 against GE's 17, at the same operating point. Radiomuseum lists the JJ as
"normally replaceable, slightly different" [13]. **If the plug-in ever offers a tube-choice switch, that is
a real, published, sourced difference to implement** and it is a 2.5 dB gain difference, not a flavour.

### 4.3 Raffensperger's fitted law, and my check on it

> **One parameter of this law is refitted in the build, and the reason is that it was never
> constrained.** The published fit is to plate **current**, and its **slope** is not fitted to
> anything. For a variable-mu stage the audio *is* the derivative, so the slope is the quantity the
> model actually uses, and as published it is 42 % low in transconductance at the class-A1 operating
> point and non-monotone, dipping and climbing again in a way the maker's own curve does not.
> Measured against the plate characteristics at 250 V it is 8.6 dB out at −50 V of grid and 35.7 dB
> out at −70, which is where this unit spends its deepest limiting.
>
> The check below is the reason it survived: three points on a plot whose lower decade is squashed
> into the bottom few per cent of the paper. A check made on a plot that cannot resolve the region it
> is checking can hardly fail.
>
> So `p8`, the exponential cut-off rate, moves from 0.2 to 0.131 87 with `p1` renormalised, fitted to
> General Electric's own plate characteristics **across the working range and in the right
> topology** — one published source to another, never to an invention. Least-squares cost falls from
> 20.05 to 0.09; letting three more parameters move buys 0.03 and was declined. Shallower than −30 V
> the term is negligible and the power law carries the curve, which is exactly why three points
> never caught it.
>
> What it cost is in the plug-in's own misses table: three distortion rows that used to pass now
> fail, and they were passing because the law was low precisely where those curves live.

Raffensperger fits a black-box anode-current model to the GE curves by Levenberg-Marquardt least squares
plus hand tuning [18]:

```
                 p1 · Vak^p2
Ia = ───────────────────────────────────────────
     (p3 − p4·Vgk)^p5 · [ p6 + exp(p7·Vak − p8·Vgk) ]
```

with `Ia` in amperes and `Vak`, `Vgk` in volts, and

| | | | |
|---|---|---|---|
| p1 | 3.981 × 10⁻⁸ | p5 | 1.8 |
| p2 | 2.383 | p6 | 0.5 |
| p3 | 0.5 | p7 | −0.03922 |
| p4 | 0.1 | p8 | 0.2 |

He notes that because `Vgk` is negative throughout the Fairchild's operating range, **grid current is assumed
negligible** [18], which is a simplification the model can keep as long as the control voltage never lets the
grid go positive.

**I checked it, because a fitted equation with eight parameters and no residuals quoted deserves checking.**
Evaluating the formula against the datasheet's "average transfer characteristics, each section" curves [12],
which is the family Raffensperger's Fig. 2 reproduces:

| Vak | Vgk | equation gives | GE curve reads |
|---|---|---|---|
| 250 V | −10 V | **19.9 mA** | ≈ 20 mA |
| 250 V | −30 V | **4.15 mA** | ≈ 4 mA |
| 250 V | −50 V | **0.56 mA** | ≈ 0.5–1 mA |

Three points across two decades of current, agreeing to within the width of the printed curve. (**Derived**:
the equation values are my arithmetic; the curve values are my eye on GE's graph, ±20 %.) **That check was
too easy, and the next two paragraphs say why** — it was read off a linear plot in the region where a linear
plot resolves nothing, so it could hardly have failed. The equation is sound in the upper part of its range
and I will use it there. It is also the only published fit of this tube that exists, and the fact that it
reproduces a 1953 GE plot to that accuracy is a better validation than Raffensperger himself gives it — his
paper validates the *circuit* against SPICE, not the tube against the datasheet.

**A second deficiency, at the deep end, which is worse than the slope problem and which I found only after
the 176 research suggested the technique.** Their point is that a fitted law can be badly wrong exactly where
the plot you validated it on cannot resolve anything. GE's per-section *transfer* curves (page 4) are linear
in plate current from 0 to 40 mA, so below about −30 V the whole family is squashed into the bottom few per
cent of the paper — which is where I did my original three-point check, and it is why that check passed too
easily. GE's **plate characteristics** (page 5) plot the same tube as curves of constant grid voltage against
plate voltage, with curves drawn down to **−70 V**, and there the deep end is legible. Reading them at
Va = 250 V and comparing:

| Vg | GE page 5 | eq. 1 | error |
|---|---|---|---|
| −8 V | 24.7 mA | 25.7 mA | +0.3 dB |
| −10 V | 21.5 mA | 19.9 mA | −0.7 dB |
| −20 V | 10.2 mA | 7.9 mA | −2.2 dB |
| −30 V | 5.2 mA | 4.1 mA | −2.0 dB |
| −40 V | 3.1 mA | 2.1 mA | −3.5 dB |
| −50 V | 1.5 mA | 0.56 mA | **−8.6 dB** |
| −70 V | 0.5 mA | 0.01 mA | **−35.7 dB** |

(GE figures read by eye off a calibrated overlay, `ref/fairchild-6386-plate-curves-calibrated.png`; the last
two rows sit within 50 px of the baseline and are soft, ±20 % at −50 V and worse at −70 V. The trend from
−8 V to −40 V is well outside reading error.)

**The equation cuts the tube off far too early.** GE's 6386 is still passing half a milliamp at −70 V — that
is what a remote-cutoff tube *is* — and eq. 1 has it at one hundredth of that. **And this is not an academic
region for the Fairchild.** Raffensperger's own simulation swings the control voltage to about −80 V
(his Fig. 10), and the 670's cathodes sit well above ground on the 680 Ω resistors (3.2), so the model
operates squarely in the range where its own tube law is 8 to 36 dB wrong. **Use eq. 1 inside roughly
0 to −30 V and treat anything below −40 V as unmodelled**; section 10.4 records the range.

**The same fault appears in the 176's tube law, in the same direction, and that is not a coincidence.** Their
fitted 6BC8 law is usable from 0 to −8 V of grid to about ±4 dB and is **16 dB low by −10 V** [56]; mine is
good to −30 V and 8.6 dB low by −50 V. **Both cut the tube off too early**, and both were fitted where
linear-axis plots are readable and then extrapolated into a region those plots could not show. A
remote-cutoff tube's whole purpose is the long shallow tail, and the tail is exactly what fitting to readable
data discards. Two tubes, two functional forms, two people working independently, one fault: that is a
property of the method, not an accident.

**So the rule, which belongs beside any procedure for fitting these tubes.** A tube law must be validated
against a plot that resolves deep cutoff *on its own terms* before it is trusted in the region a limiter
actually works in. Three plots do that and any of them would have caught both fits: a **logarithmic**
transconductance axis (GE page 3, 12.3a), **constant-parameter plate characteristics** (GE page 5, the table
above), and a **plate-resistance** curve, which works because resistance grows without limit as the tube cuts
off and so is legible exactly where current and transconductance are not [56].

**And a floor on how accurate any of this can be, which I cannot compute for the 6386.** The 176 research
observes that the two manufacturers' published gm curves for the 6BC8 differ from each other by 1.3 to
1.5 dB where both are readable, so **a fitted law cannot be more accurate than its sources are consistent**,
and their ±4 dB is bounded below by that rather than by their residual [56]. I cannot apply the same bound
here: probing every volume on `frank.pocnet.net` for `6386`, `6386A` and `6386LGP` returns **exactly one
sheet**, the General Electric ET-T1113 in volume 142 [12]. So there is no second manufacturer's curve for
this tube to cross-check against, and any accuracy I claim for a 6386 law rests on a single source. **That is
an asymmetry between the two units and it should be recorded rather than glossed**: the 176's law has a
measured consistency floor and this one has none.

**A deficiency in it that matters more than the divergence, and that I only found when the shared-component
question forced me to differentiate it (12.3a).** The equation is fitted to plate *current*, and it
reproduces current well — that is what my three-point check above tested. Its *slope* is a different matter,
and slope is what a variable-mu model actually runs on, because gain is transconductance.

| | Raffensperger eq. 1 | General Electric [12] |
|---|---|---|
| Ia at the class-A1 point (Eb 100 V, Vg −1.92 V) | 8.52 mA | 9.6 mA, **11 % low** |
| gm at the same point | 2309 µmho | 4000 µmho, **42 % low** |
| gm at Vg = −16 V | 114 µmho | 100 µmho, good |

Worse, its transconductance is **not monotone in slope**. Differentiating it at Vak = 250 V, the decay rate
falls from 4.09 dB per volt near zero bias to a minimum of **0.15 dB per volt around −40 V** and then rises
again to 1.73 dB per volt by −60 V (**derived**). GE's published curve has no such inflection: it is a
straight line on a logarithmic gm axis, at a constant 0.94 dB per volt, over that whole span (12.3a). The
wobble is an artefact of the interaction between the equation's power-law factor and its exponential factor,
and it is invisible if you only ever look at current.

**So: use eq. 1 for plate current, and do not trust its derivative.** If the model needs transconductance —
for the meter, for a gain-range test, or for anything that reasons about gain directly — take it from GE's
own curve, where it is a clean exponential, rather than from the derivative of this fit. Test 6 in section
11.1 asserts GE's figures for exactly this reason and would fail on eq. 1's derivative.

> **The build follows the first half of this and declines the second, and the reason is worth
> keeping.** Using the equation for current is right, and the refit at the head of this section makes
> its slope right with it, because a law fitted to current across the working range carries that
> range's slope. Taking transconductance from GE's tabulated figures instead cannot be done the
> obvious way: the two class-A1 points sit at 100 V of plate between −1.92 and −16 V of grid, while
> this stage runs at 216 to 230 V of plate and −22 to −70 V of grid, entirely outside that interval.
> Anchoring an exponential on them and extrapolating at 2.28 dB per volt gives about 110 dB of
> control authority where the unit has 20 — not a small error but a model that could not work.
>
> So the build takes gain from the refitted current law and **records test 6 as a miss**: 26.4 dB of
> control range against GE's 32 ± 3. The gap is printed rather than closed, because closing it with
> a two-point extrapolation would have been a constant chosen to make a test pass.

**Where it is silent.** The fit is to a family of static curves between roughly 0 and −55 V of grid and 100
to 300 V of plate. Outside that box it is an extrapolation of an eight-parameter empirical function and it
will do whatever it likes. In particular `(p3 − p4·Vgk)` goes to zero at `Vgk = +5 V`, so the expression
**blows up for grid voltages above +5 V** and should be clamped well below that. Section 10.4 records the
clamp.

### 4.4 The stage: four sections a side, and why that matters

Each push-pull half is four 6386 triode sections in parallel [4] [5] [26]. Paralleling `n` identical triodes
multiplies transconductance by `n` and divides plate resistance by `n`, so the four-section half has

```
gm  ≈ 4 × 4000 µmhos = 16 mA/V        rp ≈ 4250 / 4 = 1063 Ω
```

at the datasheet's class-A1 point (**derived** from [12]). Sound On Sound reaches for the same conclusion in
words: the stage "behaves like a single gain stage, albeit one with a very low impedance" [26].

**Why Narma did it.** A single 6386 section can pass 18 mA and dissipate 1.5 W [12]. A push-pull stage that
has to drive 600 Ω through a transformer to +27 dBm clipping — which is what the specification claims [1] —
needs more current than one section can give, and a low enough plate impedance to look like a source rather
than a current generator into the transformer. Four in parallel gets both. But it also gets a third thing,
and I think it is the real reason: **paralleling averages the tubes.** Eight sections per channel, matched
in production and trimmed by the BALANCE control, means the stage's law is the mean of eight remote-cutoff
curves rather than any one of them. The manual's balancing procedure ends "If above procedure does not
produce reasonable balance, exchange one or more of the 6386 tubes" [1], which is a maintenance instruction
that only makes sense if the design is relying on the average.

**For the model this is a licence to simplify.** Four sections in parallel and two halves in push-pull is
eight instances of the same equation per channel per sample. Raffensperger already takes this step —
*"the current is then multiplied by 2 because Eq. 1 represents a single 6386 triode, while the circuit has
two such tubes. Consequently, the circuit can be modified to include six, eight or even more tubes at no
cost"* [18] — and the right move is to keep one evaluation per half and scale, not eight evaluations. What
you lose is per-tube mismatch, which is precisely what the hardware's BALANCE control exists to remove.
Section 10.3 keeps a single mismatch term as a spoof control rather than eight tubes.

### 4.5 How the control voltage reaches the grids

This is the mechanism, and the 660 drawing states it most clearly [5].

The input transformer's secondary is centre-tapped and its two ends go to the two halves' grids. The
**audio** therefore appears **differentially**: when the upper grid swings positive the lower swings
negative, and the difference is what the output transformer takes. The **control voltage** is injected at
the secondary's centre, through **R1 and P2, 100 kΩ each** on the 660 (Raffensperger's `R_L1 = R_L2 =
100 kΩ` [18]), so it appears at both grids **in common mode**: both grids move down together.

Write it as Raffensperger does [18]:

```
Vg1(t) = V_RL1(t) + Vx(t)
Vg2(t) = −V_RL1(t) + Vx(t)
Vx(t)  = V_sc(t − 1) + V_bias
```

where `V_RL1` is the audio half-swing on the transformer secondary, `V_sc` is the sidechain's rectified
output and `V_bias` is the standing bias from the ZERO pot, which he takes as **−7.2 V** [18].

**Three consequences fall straight out of this and they are the design.**

1. **The control voltage is common-mode, so the push-pull output cancels it.** When the sidechain steps, both
   grids move the same way, both plate currents change the same way, and the output transformer, which takes
   the *difference*, sees nothing. That is the mechanism behind the manual's first boast — *"characterized
   by the complete absence of audible thumps"* [1] — and it is not marketing. It is a structural property of
   injecting the control at the centre tap of a push-pull stage. A model that computes gain as a scalar
   multiplier and applies it to a mono signal path **will thump where the hardware does not**, and will
   therefore need an artificial control-signal smoother that the hardware never had. Modelling the push-pull
   pair honestly removes the need for one.
2. **The audio is differential, so it does not move the operating point — to first order.** The two halves'
   currents move in opposite directions, so the *sum* is constant and the common cathode node stays put.
   Second order, it does not: the remote-cutoff curve is convex, so a positive grid excursion raises current
   more than an equal negative excursion lowers it, the sum rises with signal, and the stage self-biases
   slightly downward on loud material. That is a real, small, level-dependent gain reduction with no
   sidechain involved at all, and it is one of the things people mean when they say the box "does something
   even at zero gain reduction". Slate Digital's copy for FG-MU says exactly that about their algorithm:
   *"just going through the processor without any gain reduction will reveal a beautiful open sound due to
   the modeling of the tube circuit path"* [41].
3. **The 100 kΩ injection resistors and the grid capacitance form a filter on the control path.** With about
   8 pF of input capacitance per half (4.2) the corner is up in the hundreds of kilohertz and irrelevant.
   The real bandwidth limit on the control path is the time-constant network (section 5), not this.

### 4.6 Why gain and distortion are the same curve, and the chart that proves it

Here is the argument the whole engine hangs on, stated as plainly as I can.

The stage's small-signal gain is proportional to its transconductance:

```
A(Vg) ∝ gm(Vg) = ∂Ia/∂Vg  evaluated at the operating point Vg
```

and its second-order distortion is proportional to the *curvature* of the same characteristic:

```
d2(Vg) ∝ ∂²Ia/∂Vg²  at the same point
```

These are the first and second derivatives of **one function**, evaluated at **one point**, and the control
voltage moves that point. You cannot change one without changing the other. In a remote-cutoff tube the
characteristic is close to exponential over its control range, and for an exponential the ratio of curvature
to slope is constant — so the *fractional* distortion per volt of signal stays roughly constant while the
gain falls, which means that **for a fixed output level, distortion rises as gain reduction increases**,
because you need proportionally more grid swing at the input to get the same swing at the output.

In push-pull, second-order terms cancel between the halves and third-order survives, so what actually comes
out of the transformer is dominated by third harmonic, with second appearing only through mismatch between
the halves — which is what the BALANCE control trims out. **A well-balanced Fairchild is a third-harmonic
machine; an unbalanced one adds second.** That is a testable, structural prediction (12.2, test 8) and it is
the single most useful thing the push-pull topology gives the model.

**Fairchild measured this and published it.** Page 8 of the December 1959 manual is a chart headed
**"IM DISTORTION AS A FUNCTION OF OUTPUT LEVEL & AMOUNT OF LIMITING, 60 CYCLES 7KC 4:1"**, dated **3/59**
[10]. The x-axis is decibels of limiting from 0 to 20; the y-axis is per cent IM; and there are seven
curves, one for each output level:

| curve | output level |
|---|---|
| 1 | 0 dBm out |
| 2 | +4 dBm out |
| 3 | +8 dBm out |
| 4 | +12 dBm out |
| 5 | +16 dBm out |
| 6 | +20 dBm out |
| 7 | +24 dBm out |

Every one of the seven rises monotonically with the amount of limiting, and every one of them turns sharply
upward near its right-hand end. Values I read off the chart by eye, which I would trust to about ±0.5
percentage points:

| | 0 dB limiting | where it reaches ≈9 % |
|---|---|---|
| curve 7, +24 dBm out | ≈ 3.8 % | ≈ 7 dB of limiting |
| curve 6, +20 dBm out | ≈ 1.6 % | ≈ 10.5 dB |
| curve 5, +16 dBm out | ≈ 0.5 % | ≈ 14 dB |
| curve 4, +12 dBm out | ≈ 0.2 % | ≈ 16.5 dB |
| curve 3, +8 dBm out | ≈ 0.2 % | ≈ 18 dB |
| curve 2, +4 dBm out | ≈ 0.2 % | ≈ 20 dB |
| curve 1, 0 dBm out | ≈ 0.2 % | beyond the chart |

**Two independent checks that the chart and the specification sheet agree.** The specification claims "less
than 1 % at 10 db limiting and +12 dbm output" [1]; curve 4 at 10 dB of limiting reads about 0.4 % on the
chart, which is under 1 %. And it claims "less than 1 % at any level up to +18 dbm output (no limiting)";
at zero limiting the chart's +16 dBm curve reads about 0.5 % and +20 dBm reads about 1.6 %, so +18 dBm
interpolates to roughly 1 %. The specification is the chart, rounded and stated conservatively. That is a
good sign about both documents.

**And here is the thing the survey did not know.** The survey worried that we would be "checking a model
against a model" [52]. This chart is a measurement of hardware, published by the manufacturer, of exactly
the quantity that this family of compressor is interesting for. It is the ground truth the Fairchild was
supposed not to have. It is IM rather than THD, at 60 Hz and 7 kHz in a 4:1 ratio, which is the SMPTE
condition, so a test against it has to measure SMPTE IM and not harmonic distortion (12.2, test 9).

### 4.7 What none of this shares with anything the lab already owns

Set against the four gain elements in the codebase:

| | how gain falls | where distortion comes from | can they be separated? |
|---|---|---|---|
| FET (1176) | channel resistance falls, shunting to ground | the FET's own square-law, plus the class-A output stage | **yes**, they are different stages |
| photocell (LA-2A, LA-3A, CL 1B) | cell resistance falls in a divider | the photoconductor's own law, plus the amplifier after it | **yes** |
| Blackmer VCA (Distressor) | log-antilog multiplier gain falls | the cell's mistracking, plus the make-up amp | **yes** |
| diode bridge (Neve) | bridge dynamic resistance falls | the bridge itself, and it *is* the attenuator | partly — the bridge's law is fixed, the amplifiers around it are not |
| **remote-cutoff triode (Fairchild)** | **the amplifier's own transconductance falls** | **the same characteristic's curvature** | **no. Structurally, never.** |

The diode bridge is the nearest relative and it is not close, and the [[Neve-33609]] dossier's section 4.4 is
worth reading beside this one for the contrast [53]; the [[CL-1B]] dossier makes the same point from the
opposite end, where the timing lives in an op-amp and not in the part at all [54]. There, the bridge is an attenuator whose law
happens to be nonlinear, sitting between two amplifiers that are separately linear; you can push it harder
or softer with the level into it. Here there is nothing between the transformers *but* the tubes. There is
no "input level to the gain element" independent of the input level to the amplifier, because they are the
same node.

**The practical consequence for the plug-in.** Every other model in the lab can offer a drive control that
trades distortion against gain reduction. This one cannot, honestly. What it can offer is a control that
moves the **operating point** — which is what the hardware's ZERO screw does (3.5) — and that changes
standing gain, available gain reduction and standing distortion **together**, in the direction the tube's
curve dictates. Universal Audio's "Headroom" [34] and Softube's "calibration knob to influence the unique
compression-dependent saturation of the variable-mu architecture" [39] are both, I think, this control.
Getting it right is more interesting than getting a drive knob right, and it is section 10.2's `fc_zero`.
---

## 5. The sidechain, the six time constants, and what each position really does

The TIME CONSTANT switch is the unit's signature and it is the part of the circuit with the most published
numbers attached to it, so this is the longest section and it is where most of the new work is.

### 5.1 The chain, stage by stage

Reading the 670's channel 1 from the output backwards to the grids [4], with the 660's designators in
brackets where they differ [5]:

1. **Tap.** Two secondary windings of the output transformer T102 [T2].
2. **Pad.** R145, R146 and R149 [R54, R55, R56] — 150 Ω, 150 Ω into 680 Ω, twice. Raffensperger's
   `R_in = 600 Ω`, `R_term = 1360 Ω` (3.2).
3. **Sidechain input transformer.** T103 [T7], stepping up by `N_p/N_s = 1/17` [18].
4. **Threshold.** R115a/R115b, 100 kΩ linear ganged [R10-A/R10-B, 180 kΩ], the **AC THRESHOLD** on the front
   panel, with R116 270 kΩ and R117 100 kΩ, the **DC THRESHOLD** screwdriver inside the box.
5. **First stage.** V105, a 12AX7 [V5], with R118/R119 1 MΩ and R120/R121 100 kΩ, coupled out through
   C103/C104, 0.05 µF [C3/C4, 0.05].
6. **Second stage.** V106, a 12BH7 [V6, 12BH7A], with R122/R123 1 MΩ, R124/R125 1 kΩ, C105/C106 20 pF and
   C107/C108 0.1 µF.
7. **Output stage.** V107 and V108, two 6973 [V7 and V8, two 6V6GT], with R126/R127 100 kΩ, R128/R129
   2.7 kΩ, R130/R131 150 Ω, R132/R133 560 Ω and R134/R135 560 Ω. Supply rails on this stage are annotated
   **440 V** by Raffensperger, with **240 V** on the earlier stages, **154 V** on a screen and **−14 V** as a
   bias rail [18].
8. **Sidechain output transformer.** T104 [T3]. Raffensperger reads four windings on it: a positive-going
   primary, a negative-going primary, a secondary and a **tertiary feedback coil**, with turns ratios
   `N_p/N_s = 4` and `N_p/N_t = 9.5` [18]. He says why the feedback winding is there: *"The feedback from
   the tertiary coil reduces the output impedance of the amplifier. A low output impedance is essential for
   driving the capacitive load with enough current to achieve a fast attack time"* [18].
9. **Rectifier.** R136, 3.9 kΩ [R31, 3900 Ω 1 W] across the secondary, into a **full-wave bridge**.
10. **Time constants.** R137 220 kΩ [R32], C109 0.1 µF, C115 2 µF [C7, 2 µF/200 V], and S102 [S2].
11. **Bias.** R142, 2.5 kΩ [R41, 250 kΩ], the ZERO control (3.5), and out to the grid injection resistors.

That is **five gain stages of sidechain to two of audio**, if you count the input transformer's step-up and
the output transformer's step-down as gain. Almost all of this box is the detector.

### 5.2 The AC and DC threshold controls, which together are the ratio

This is the pair of controls that most descriptions of the Fairchild get wrong, so here is what the primary
documents say.

**The manual's calibration procedure** [1], quoted in full because every number in it is testable:

> Before adjusting these controls, the desired output level at 3 db and 10 db of limiting respectively, must
> be chosen. **The 3 db limiting point is controlled by the DC THRESHOLD, the 10 db limiting point by the AC
> THRESHOLD.** … 1. Set the AC THRESHOLD control to zero and adjust the INPUT GAIN control for unity gain.
> 2. Turn both the AC and the DC THRESHOLD controls to their full clockwise rotation. 3. Apply a signal to
> the input 3 db higher than the desired output level and adjust the DC THRESHOLD control to desired output
> level. 4. Increase the input signal to 10 db higher than the desired output level and adjust the AC
> THRESHOLD control to desired output. 5. Repeat Steps 3 and 4 until the desired slope and position are
> obtained.

So **the two controls set two points on the static curve** — one at 3 dB of gain reduction and one at 10 dB
— and everything between and beyond is whatever the circuit does. The manual calls the result "the desired
slope and position", and *slope* is the ratio.

**What the circuit does with them.** The AC THRESHOLD is a straightforward ganged attenuator on the
sidechain signal: turn it down and the detector sees less, so limiting starts later. Turning it "completely
counterclockwise removes the limiting action completely. The unit is now a simple Unity Gain Line
Amplifier" [1]. The DC THRESHOLD is not an attenuator at all. Raffensperger: *"The first stage of the
sidechain amplifier is a Class B amplifier with an amount of intentional crossover distortion set by the DC
threshold potentiometer"* [18], and he models the stage as

```
                   ⎡ 1 + exp( V_pot − φ'_DC) ⎤
V_stage1(t) = ln   ⎢ ───────────────────────── ⎥          (Raffensperger eq. 10)
                   ⎣ 1 + exp(−V_pot − φ'_DC) ⎦
```

with `V_pot = φ_AC · V_Rload / 2` and `φ'_DC = 12.2 (φ_DC + 0.1)` [18]. That is a soft, symmetric dead zone
whose width is set by `φ_DC`: with the DC threshold at one end the transfer is nearly linear through zero,
and at the other there is a wide flat region where small sidechain signals produce no output at all.

**A dead zone in a feedback detector is a knee control.** Small overshoots fall inside it and are not
detected, so the ratio near threshold is low; large ones clear it and are detected fully, so the ratio at
depth is high. That is precisely the "progressive ratio" the clone builders describe. Heritage Audio call
theirs "the progressive ratio, which is a unique curve adjustment combining control of the ratio and the
knee" [42] [44]; Overloud say the original's internal trimmer "allowed to change the compression
characteristic from soft-knee to hard-knee by changing the biasing current of the feedback amplifier" [40];
Sound On Sound says "the curve can be adjusted to some degree by way of an internal trim pot" [26]. Four
sources, one control, and Raffensperger gives it an equation.

**So the Fairchild has no ratio knob and does not need one.** The specification's "compression ratio
variable from 1:1 to 1:20 above a predetermined level" [1] is describing the range that these two controls
between them can reach, and section 7.2 reads the five published curves that show it.

Raffensperger also records that the AC threshold pots are not plain 100 kΩ linear: *"φ_AC is a pair of
100 kΩ linear potentiometers with 24 kΩ resistors connected between ground and a center tap on each
potentiometer. Each AC threshold potentiometer is effectively a 76 kΩ potentiometer with a piecewise linear
taper"* [18]. I could not locate those 24 kΩ resistors on the redrawn 670 sheet, but the **660 factory
drawing shows R8 and R9 at 24 kΩ 5 %** in exactly that part of the threshold network [5], which corroborates
him from a primary document. **A tapped pot with a kink in its law is a real thing to model** and it is why
the threshold knob's numbers are not decibels.

### 5.3 The time-constant network, read off the factory drawing

**This is the part of the file I am most confident is new.** Raffensperger publishes the network as a table
of component values and does not say where it comes from [18]. The 670 redraw shows the network but leaves
one capacitor marked `???` and does not number the switch positions [4]. The **660 factory drawing numbers
every switch position on the sheet** [5], and it turns out to be the same network.

The topology is a current source (the sidechain output stage through the rectifier) driving a parallel RC,
with up to two more series-RC legs hung off the same node — Raffensperger's Fig. 8, with `I_sc` into
`C_T ∥ R_T`, and `R_U + C_U` and `R_V + C_V` as optional legs [18].

**Permanently in circuit, on both units:**

| | 660 [5] | 670 [4] |
|---|---|---|
| R_T fixed | **R32, 220 kΩ** | R137, 220 kΩ |
| C_T fixed | **C7, 2 µF / 200 V** | C115, marked `???` / 200 V |
| RF bypass | — | C109, 0.1 µF |

**Switched by S2 [S102], which is two six-position wafers ganged:**

| deck | position | what it connects |
|---|---|---|
| upper | 1 | **R37, 68 kΩ** to the return |
| upper | 2 | nothing |
| upper | 3, 4, 5 (tied) | **C11, 2 µF / 200 V** |
| upper | 6 | **R35, 100 kΩ** in series with **C10, 20 µF / 150 V** |
| lower | 1 | nothing |
| lower | 2 | **R33, 470 kΩ** to the return |
| lower | 3 | nothing |
| lower | 4 | **C9, 4 µF / 150 V** |
| lower | 5, 6 (tied) | **R34, 100 kΩ** in series with **C8, 8 µF / 150 V** |

Combining the two decks position by position, and folding in the fixed R32 and C7:

| position | R_T | C_T | slow leg U | slow leg V |
|---|---|---|---|---|
| 1 | 220k ∥ 68k = **51.94 kΩ** | 2 µF | — | — |
| 2 | 220k ∥ 470k = **149.86 kΩ** | 2 µF | — | — |
| 3 | 220 kΩ | 2 + 2 = **4 µF** | — | — |
| 4 | 220 kΩ | 2 + 2 + 4 = **8 µF** | — | — |
| 5 | 220 kΩ | 2 + 2 = **4 µF** | 100 kΩ + 8 µF | — |
| 6 | 220 kΩ | **2 µF** | 100 kΩ + 8 µF | 100 kΩ + 20 µF |

**Compare with Raffensperger's Table 3** [18], which gives `C_T` = 2, 2, 4, 8, 4, 2 µF; `R_T` = 51.9, 149.9,
220, 220, 220, 220 kΩ; `C_U` = 8 µF with `R_U` open except at positions 5 and 6 where it is 100 kΩ; `C_V` =
20 µF with `R_V` open except at position 6 where it is 100 kΩ.

**Every one of the fourteen values matches, in every one of the six positions.** Including the two awkward
parallel combinations: 220 ∥ 68 is 51.944 kΩ against his 51.9, and 220 ∥ 470 is 149.855 kΩ against his
149.9. He rounded; the resistors are 68 kΩ and 470 kΩ.

**Two things follow.**

First, **Raffensperger's table is confirmed against a primary source**, which as far as I can tell nobody
has published. His paper is the only circuit model of this unit in the literature and its most-cited table
now has provenance.

Second, **the 670's unreadable capacitor is resolved. C115 and C215 are 2 µF at 200 volts.** The redrawn
670 sheet marks the value `???` but keeps the voltage rating, `200v` [4]; the 660's C7 is `2/200` and its
C11 is also `2/200`, and both are the only 200-volt parts in a network where everything else is 150 volts
[5]. The rating matches, the position matches, the function matches and the arithmetic matches. I am
labelling this **derived**, not stated, because no document I hold prints "C115 = 2 µF" — but it is derived
from three independent agreements and I would build on it.

### 5.4 Deriving the published release times, and what Fairchild's phrase actually meant

The manual publishes six release times "from 10 db of limiting" [1]. The network above has six
configurations. Nobody, as far as I can find, has ever checked one against the other. Here it is.

Take the primary time constant of each position, `τ = R_T · C_T`, ignoring the slow legs:

| position | R_T | C_T | τ = R_T·C_T | published release [1] | ratio |
|---|---|---|---|---|---|
| 1 | 51.94 kΩ | 2 µF | 0.1039 s | 0.3 s | 2.89 |
| 2 | 149.86 kΩ | 2 µF | 0.2997 s | 0.8 s | 2.67 |
| 3 | 220 kΩ | 4 µF | 0.880 s | 2 s | 2.27 |
| 4 | 220 kΩ | 8 µF | 1.760 s | 5 s | 2.84 |
| 5 (individual peaks) | 220 kΩ | 4 µF | 0.880 s | 2 s | 2.27 |

Five positions, one constant: **2.59, with a standard deviation of 0.27** (**derived**). Every published
release time is 2.59 ± 13 % times the RC product of the network in that position.

**And that constant tells us what Fairchild meant by "release time".** If gain reduction in decibels decays
as `GR(t) = GR₀ · exp(−t/τ)` — which it roughly does, because the control voltage decays exponentially and
the remote-cutoff law is roughly exponential in grid volts, so dB of gain reduction is roughly proportional
to control voltage — then after 2.59 τ the remaining gain reduction is

```
10 dB × exp(−2.59) = 0.75 dB
```

So **"release time from 10 dB of limiting" means the time to recover to within about three quarters of a
decibel of unity**, or equivalently to give back about 92.5 % of the reduction. **The assumption that dB of
gain reduction is proportional to control voltage is now supported rather than merely asserted**: section
12.3a reads the 6386's transconductance law off GE's logarithmic plot and finds it at or below a pure
exponential in every condition GE plot, n between 0.59 and 1.00, at roughly 1.2 to 2.2 dB per volt of grid. Close to exponential
is enough for the release-time derivation, which needs proportionality only to the accuracy of a round
number in a 1959 specification. That is a perfectly sensible
1950s definition and it is not one you could have guessed. (**Derived.**)

**Notice which two positions agree with each other.** Positions 3 and 5 have identical `R_T` and `C_T` and
Fairchild publishes identical release times for them, 2 seconds, position 5's being qualified as "for
individual peaks" [1]. That is an internal consistency check on my reading of the switch that costs nothing
and passes.

> **Ruled against by the build, and 5.5 wins.** The derivation above takes position 5's
> individual-peak figure from `R_T·C_T` alone, treating the uncharged slow leg as not yet loading the
> node. Section 5.5 needs the exact opposite, uncharged legs pulling the effective resistance *down*,
> to reach position 6's 0.3 s, and this file admits the two cannot both hold: "no single simple
> reading reproduces all of positions 5 and 6."
>
> Building the network settles it, and the mechanism is real but conditional. It is one inequality
> pointing opposite ways at the two positions. At position 6 the node's own constant is 0.44 s,
> genuinely fast against the legs' 0.8 s and 2.0 s, so while the legs are empty they do pull the
> resistance down. At position 5 the node's constant is 0.88 s, which is **slower than its one leg's
> 0.8 s**; a leg cannot be "not yet charged" relative to a node that moves more slowly than it does,
> so the 8 µF joins immediately whatever the stimulus, and the tail is 220 kΩ into 12 µF from the
> first millisecond.
>
> The consequence is a recorded miss rather than a fix: position 5's individual peak comes out 3.3 s
> against the published 2. Its multiple-peaks figure is met. No component value was touched to chase
> it, and all four fixed positions and all three of position 6's figures fall out of the same
> integration.

### 5.5 The two program-dependent positions, and why position 6 is fast *and* slow

Positions 5 and 6 add series-RC legs, and this is the mechanism that makes the Fairchild's automatic
releases different from every other auto-release in the lab.

The extra legs are **capacitors behind resistors**. `C_U` = 8 µF behind `R_U` = 100 kΩ charges with a time
constant of **0.8 s**; `C_V` = 20 µF behind `R_V` = 100 kΩ charges with **2.0 s** (**derived** from [5]).
Those are slow compared to the main capacitor's discharge. So:

- **On a short peak**, the slow capacitors have not had time to charge. They are still near zero volts, so
  they look like **near short circuits to the return through their series resistors**. The main capacitor
  therefore discharges not into `R_T` alone but into `R_T` in parallel with those resistors, and the release
  is **fast**.
- **On sustained loud material**, the slow capacitors charge up. Once charged they stop sinking current, the
  effective resistance rises to `R_T` alone, and — crucially — the charge they are now holding has to come
  back out through the same resistors. The release becomes **slow**, and it has a long tail.

**Position 6 is the extreme case and it is where the manual's three numbers come from.** Fairchild publish,
for position 6: *"Automatic function of program material: .3 seconds for individual peaks, 10 seconds for
multiple peaks, 25 seconds for consistently high program level"* [1]. Work the network:

| state | effective R | effective C | τ | ×2.59 | published |
|---|---|---|---|---|---|
| slow caps empty (individual peak) | 220k ∥ 100k ∥ 100k = **40.74 kΩ** | 2 µF | 0.0815 s | **0.21 s** | 0.3 s |
| C_U charged (multiple peaks) | 220 kΩ | 2 + 8 = 10 µF | 2.20 s | **5.7 s** | 10 s |
| both charged (consistently high) | 220 kΩ | 2 + 8 + 20 = 30 µF | 6.60 s | **17.1 s** | 25 s |

(**Derived** throughout, using the 2.59 constant from 5.4.)

**Look at the first row.** Position 6 has the *slowest* main resistor on the switch, 220 kΩ, and the
*smallest* capacitor, 2 µF. Taken at face value it should release in `2.59 × 0.44 s` = 1.14 s. Fairchild say
0.3 s, four times faster, and the same as position 1, which has a 68 kΩ resistor bolted across the network
specifically to make it fast. **The only way position 6 gets to 0.3 s is if the two uncharged slow
capacitors are pulling the effective resistance down to 41 kΩ** — which is right next to position 1's
51.9 kΩ, and gives 0.21 s against a published 0.3 s. That is the mechanism, demonstrated by the fact that no
other reading of the network produces the published number.

The two sustained rows are less good, 5.7 against 10 and 17.1 against 25, both about 40 % low. That does not
worry me much: those states are not single-pole decays, the 2.59 constant was calibrated on single-pole
positions, and "10 seconds" and "25 seconds" are round numbers describing a programme-dependent
behaviour rather than a measurement of a step response. What matters is that **the ordering, the ratios and
the mechanism all come out right, and the fast case comes out right to within the roundness of the number.**

Position 5 is the same story with one leg instead of two: fast release from the main RC at 2 s, lengthening
towards 10 s as `C_U` charges [1]. Its "individual peaks: 2 seconds" figure works out at `2.59 × 0.88` =
2.28 s **if you treat the uncharged `C_U` as not yet loading the node** — which is the opposite assumption
to the one that works for position 6, and I want to be honest that **no single simple reading reproduces all
of positions 5 and 6.** The network is genuinely three-pole and its behaviour depends on the whole history
of the signal, which is exactly why Fairchild described these two positions in words instead of numbers.

**Which is the design guidance.** Do not implement positions 5 and 6 as switched exponential release times.
**Implement the actual RC network** — one node, three capacitors, three resistors, values from the table in
5.3 — and let the six behaviours emerge. It is three state variables and about a dozen multiply-adds per
sample, it is cheaper than the branchy program-dependent logic it replaces, and it is the only way to get a
release that depends on how long the last twenty seconds were loud. Section 10.3 writes it out and section
12.3 tests the emergent times against the manual's six published figures.

> **Confirmed by the build.** This section's mechanism is the one that survives; 5.4's arithmetic is
> the one that does not. See the block at the end of 5.4 for the inequality that decides which
> position each applies at.

### 5.6 The attack times, and a correction to the manual

The manual publishes attack as **0.2 ms in positions 1, 2 and 6; 0.4 ms in positions 3, 4 and 5** [1]. Sound
On Sound publish a table giving **0.2, 0.2, 0.4, 0.8, 0.4, 0.2 ms** for positions 1 to 6 [26]. They disagree
about position 4.

The circuit settles it. Attack is not an RC charge here; it is **slew limiting**. Raffensperger models the
sidechain output as a current source with a hard saturation, and gives the limit explicitly as
`I_max = 0.5 A` [18]:

```
I_sc(t) = I_nom(t) − (I_max/10) · ln[ 1 + exp( 10·I_nom(t)/I_max − 10 ) ]     (eq. 14)
```

A current-limited source charging a capacitor to a fixed voltage takes a time proportional to the
capacitance: `t = C · ΔV / I_max`. So the attack time should be **proportional to `C_T`** and to nothing
else. Test it against the two published tables:

| position | C_T (5.3) | manual attack [1] | SOS attack [26] | attack / C_T |
|---|---|---|---|---|
| 1 | 2 µF | 0.2 ms | 0.2 ms | 0.10 ms/µF |
| 2 | 2 µF | 0.2 ms | 0.2 ms | 0.10 ms/µF |
| 3 | 4 µF | 0.4 ms | 0.4 ms | 0.10 ms/µF |
| 4 | 8 µF | **0.4 ms** | **0.8 ms** | 0.10 ms/µF *using SOS* |
| 5 | 4 µF | 0.4 ms | 0.4 ms | 0.10 ms/µF |
| 6 | 2 µF | 0.2 ms | 0.2 ms | 0.10 ms/µF |

**Sound On Sound's table is exactly proportional to the timing capacitance, at 0.10 ms per microfarad, in
all six positions. The manual's is not, in one position.** (**Derived.**) Position 4 has twice position 3's
capacitance and the same charging current; it cannot attack in the same time. I conclude the manual's
grouping — "0.4 milliseconds in positions 3, 4 and 5" — collapses three positions into one line and loses
position 4, and that **the correct attack times are 0.2, 0.2, 0.4, 0.8, 0.4, 0.2 ms**.

That also puts a number on the control voltage. With `I_max = 0.5 A` and 0.10 ms per µF, the swing is

```
ΔV = I_max · t / C = 0.5 A × 0.1 ms / 1 µF = 50 V
```

> **Built as a circuit, not imposed as a constant.** The 0.10 ms per microfarad above is derived by
> holding the sidechain output stage at its current limit for the whole of the attack, which assumes
> the rectifier conducts continuously; a sine drives it on peaks. The model therefore builds the
> circuit, a hard current limit with a softplus knee, and asserts this constant as a **check on** that
> circuit rather than an **input to** it. It holds: all six positions land within 20 % of it and the
> ordering is exactly proportional to `C_T`. Where a derived constant and the circuit it came from
> could have disagreed, what ships is the circuit.

**about fifty volts of control voltage between no limiting and full limiting** (**derived**, and dependent
on Raffensperger's `I_max`). Which is a satisfying number to arrive at, because it is what the manual is
boasting about on its first page — *"a single push-pull stage of audio amplification and an extremely high
control voltage, with the result that the Automatic Gain-Controlled Amplifier never produces any audible or
observable thumps"* [1] — and it is the reason the sidechain needs five gain stages, a 440 volt rail and
four output valves to make a DC voltage.

**One number in the manual I cannot reconcile with anything.** The features page claims the unit "can
produce full limiting effect during the first 10,000ths of a second" [1] [2], which is 100 µs, while the
specification page in the same document says the fastest attack is 200 µs. The circuit says 200 µs. I think
the features page is marketing rounding in the wrong direction and I would not test against it.

### 5.7 Raffensperger's sidechain model, and what to take from it

For completeness, the rest of his black-box chain, all from [18]:

Stages two and three are modelled as a static gain with a hard clip, the gain **estimated from simulation**
at `A_v = 8.4` and the clip at ±100 V:

```
V_stage3(t) = clamp( 8.4 · V_stage1(t), −100 V, +100 V )                      (eq. 11)
```

The rectifier is a diode-plus-resistance model with a soft transition, `λ = 10`, germanium drop
`V_d = 0.3 V`, and output resistance `R_out = 160 Ω` **estimated from simulation**:

```
I_nom(t) = (2·V_d)/(λ·R_out) · ln[ 1 + exp( λ·V_diff(t)/(2·V_d) − λ ) ]        (eq. 12)
V_diff(t) = |V_stage3(t)| − [ V_sc+(t−1) − V_sc−(t−1) ]                        (eq. 13)
```

followed by the saturation of eq. 14 above.

**What to take.** The shapes: a soft dead zone whose width is the DC threshold, a linear stage that clips, a
soft-knee rectifier, a hard current limit, and then the RC network. **What not to take without saying so:**
`A_v = 8.4`, `R_out = 160 Ω`, `I_max = 0.5 A` and `V_d = 0.3 V` are Raffensperger's fits to *his SPICE
simulation*, not measurements, and he is explicit that his validation is against simulation rather than
hardware. Section 10.4 marks all four as **E**.

**Germanium is worth a second look.** He assumes `V_d = 0.3 V`, a germanium drop, and Radiomuseum's parts
list for the 670 names **1N538** among its semiconductors [14] — a silicon rectifier with a drop nearer
0.7 V. The 660 drawing labels its bridge only `CR` with no type [5]. So the diode type in the sidechain
rectifier is **not established**, and since the drop only sets the softness of the knee near zero
detection, I would make it a constant and label it an estimate rather than argue about it.
---

## 6. Lateral and vertical, which is mid-side and is not stereo linking

### 6.1 What the switch does

One switch, **S301**, with **ten wafers** — the schematic labels them S301A, B, C, D, E, F, G, H, J and K,
skipping I in the usual way [4]. Ten wafers to throw one two-position switch, because it has to re-route
both channels' inputs *and* both channels' outputs through matrix networks at once.

The manual describes it exactly [1]:

> Mode Switch. Left-Right position: 2 independent limiters. Vertical-Lateral position: matrixing input and
> output, left and right in and out, limiting action vertical-lateral.

and, in the general description:

> The latter is accomplished by first bringing the two stereo channels through a matrixing network, dividing
> them into their vertical and lateral components, limiting them independently, and recombining them through
> a second matrixing network into left and right channels.

The panel calls it **AGC**, with **LEFT / RIGHT** at one end of the toggle throw and **LAT / VERT** at the
other (2.3). In the vertical-lateral position, **channel 1 (the upper row, labelled LEFT-LAT) processes the
lateral component and channel 2 (RIGHT-VERT) processes the vertical** — the manual says "The upper channel
now acts as a limiter for the lateral component, the lower channel as a limiter for the vertical component"
[1].

**Lateral is the sum, vertical is the difference.** In disc cutting, the lateral component of the groove is
side-to-side motion, which a 45/45 stereo cutter produces from the sum of the two channels; the vertical
component is up-and-down motion, produced by the difference. So lateral is **mid** and vertical is
**side**, in the terms a mixing engineer would use, and the matrix is an ordinary sum-and-difference network
in both directions. The 670 was doing mid-side processing in 1959, with an eleven-position rotary's worth of
switch contacts, for a reason that had nothing to do with mixing.

### 6.2 Why Fairchild built it, in their own words

The manual spends two full pages on this and it is the most interesting prose in the document, because it
explains a feature by explaining a manufacturing problem [1]:

> The cutting of STEREO DISKS has uncovered a number of new problems heretofore unknown. The normally
> modulated STEREO groove requires approximately twice the space of lateral grooves for similar modulation,
> yet the available space is no greater. In addition, the two STEREO channels, depending on their phase
> relationship, may result in either lateral or vertical modulation, or a combination of both. It stands to
> reason that some peaks may result in purely vertical modulation on the disk, others purely lateral. If
> this were allowed to happen continually, the result would be only 15 minutes on the STEREO LP side or else
> it would be necessary to reduce the recorded level radically.

And on tracking:

> Most commercially available STEREO pickups have considerably less vertical compliance and consequently are
> less capable of tracking large vertical modulations. Also, we should not forget that the tip radius on the
> STEREO playback stylus is still 0.7 mil, which necessitates a minimum of 1 mil groove width, or poor
> tracking might result.

And the conclusion, which is the feature:

> The one apparent solution to these problems is to break the STEREO signal down to its respective vertical
> and lateral components. Limit the vertical and lateral components independently corresponding to the
> available groove space and depth, then recombine these components to regain the original STEREO signal.
> This can be done with the FAIRCHILD MODEL 670 LIMITER and many thousands of STEREO masters have been cut
> successfully with the help of this FAIRCHILD unit.

The 670's front page claims it is "the only unit presently in production which can control both components
(vertical and lateral) independently" [1].

**The musical justification is in there too, almost as an aside**, and it is the one that survives into
modern use:

> The limiting of the vertical and lateral components instead of the left and right channels has additional
> merits. Such limiting will retain the spatial distribution of instruments and soloists as originally
> recorded without producing any annoying image drift. Of course some program material of the ping-pong type
> requires independent limiting of each channel, and this is also available in the MODEL 670 LIMITER.

**"Without producing any annoying image drift" is the entire argument for mid-side bus compression**, made
in 1959, in a manual about groove geometry.

### 6.3 How this differs from ordinary stereo linking, and why that matters to the model

The lab has stereo linking already: the Neve's `link`, the Distressor's, the 1176's. Linking means **two
detectors, one control voltage**, usually the maximum of the two, applied to both gain elements. Both
channels then always get the same gain, so the image cannot move.

The Fairchild's lateral-vertical mode is **not that**. It is **two matrices and two entirely independent
limiters** [1] [4]. The two channels do not share a detector, do not share a control voltage and are not
linked in any way. What they share is that they are no longer looking at left and right; they are looking at
mid and side.

The consequences are different and they are worth spelling out because a model that implements "M/S mode" as
a checkbox on a linked stereo compressor gets them all wrong:

- **The image can still move**, and Fairchild say so: "often it is advantageous to reduce the vertical
  component more than the lateral, resulting in some loss of separation" [1]. Compressing side harder than
  mid narrows the image. That is a *feature* here, deliberately used to fit a groove, and it is why the two
  channels have separate thresholds and separate time constants rather than one of each.
- **A centred mono source only drives the lateral channel.** Its vertical channel sees nothing, so the
  vertical limiter sits idle at unity while the lateral one works. In an ordinary linked pair, both would
  be compressing.
- **A hard-panned source drives both equally**, because a signal in one channel only is half mid and half
  side.
- **Out-of-phase content drives the vertical channel alone**, which is exactly the case the whole feature
  was built for.

**And the mode switch changes the separation specification**, which is the next section's problem.

### 6.4 The separation figure, which looks like a typo and is not

The specification sheet says [1]:

| SEPARATION | |
|---|---|
| Left-Right position | 60 db |
| Vertical-Lateral position | **0 db** |

Zero decibels of separation reads like a defect. It is a definition. In the vertical-lateral position the
two channels are *by construction* carrying mid and side, and a signal present in left alone appears in both
of them at equal amplitude — that is what a sum-and-difference matrix does. So the crosstalk between
"channel 1" and "channel 2", measured with a left-right test signal, is 0 dB. Fairchild are stating that
honestly rather than hiding it.

**What it means for the model.** If the two channels are given identical settings and identical gain
reduction, the matrix and the de-matrix cancel exactly and left-right separation is restored perfectly. The
manual says it: "As long as the amount of lateral and vertical component reduction in each channel is
identical, no deterioration of separation will occur" [1]. Any *difference* in the two channels' gain
reduction leaks one input channel into the other output channel, by an amount that is a direct function of
the gain difference. That is a clean, exactly-derivable relationship and it makes a good test (12.4, test
16): with identical settings, a left-only signal must produce nothing measurable at the right output.

The 60 dB figure in the left-right position is a normal channel-separation specification for two amplifiers
sharing a chassis, a supply and a heater string.

---

## 7. Published measurements, with their conditions

### 7.1 The complete 1959 specification

Reproduced from the December 1959 instruction manual, per channel unless otherwise specified [1]. This is
the whole page; I have not selected.

| quantity | published value |
|---|---|
| input impedance | 600 Ω |
| output impedance | 600 Ω |
| output level | +4 or +8 VU (**+27 dBm clipping point**) |
| gain | 7 dB (no limiting) |
| frequency response | **40 c/s to 15 kc ±1 dB** |
| noise level | **70 dB below +4 dBm** |
| limiting noises | "Below audibility." |
| IM or harmonic distortion | **less than 1 % at any level up to +18 dBm output (no limiting); less than 1 % at 10 dB limiting and +12 dBm output** |
| attack time (adjustable) | 0.2 ms in positions 1, 2 and 6; 0.4 ms in positions 3, 4 and 5 (**see 5.6 — position 4 is 0.8 ms**) |
| release, position 1 | 0.3 s |
| release, position 2 | 0.8 s |
| release, position 3 | 2 s |
| release, position 4 | 5 s |
| release, position 5 | automatic function of programme material: 2 s for individual peaks, 10 s for multiple peaks |
| release, position 6 | automatic function of programme material: 0.3 s individual, 10 s multiple, 25 s for consistently high programme level |
| compression ratio | **variable from 1:1 to 1:20 above a predetermined level. Predetermined level factory-adjusted to +2 dBm** |
| separation, left-right | 60 dB |
| separation, vertical-lateral | 0 dB |
| power | 117 V, 50–60 c/s, 3 A |
| stability | gain, gain reduction and balance stable over a line-voltage range of **100 to 127 V** |
| dimensions | standard 19-inch rack, **14-inch** panel space, 11 inches behind the panel |
| weight | approximately 65 lb |
| tube complement | 8 × 6386, 1 × 6084, 1 × 5651, 2 × 12AX7, 2 × 12BH7, 1 × EL34, 4 × 6973, 1 × GZ34 (5V4) |

Note "1:1 to 1:20", which is Fairchild's notation for what everyone now writes as 20:1. Note also that the
gain figure, 7 dB, sits oddly beside the operating instruction to "Set the INPUT GAIN controls to unity gain
(approx. 10 db attenuation)" [1] — 10 dB of attenuation giving unity implies about 10 dB of amplifier gain,
not 7. The published static curves (7.2) put the net gain at about +2.5 dB with whatever attenuator setting
the test used, which does not settle it either. **I would not test against the 7 dB figure.**

**And a contradiction inside the same document.** The features page, two pages before the specification,
says the 670 "can be adjusted to work either as a compressor, with a ratio of 2 to 1 and a threshold of 5 db
below normal program level; or as a peak limiter, with a compression ratio of **30 to 1** and a threshold of
10 db above normal program level" [1]. The specification page says the range tops out at **20:1**. Wikipedia
quotes the 30:1 version [19]. Both numbers are Fairchild's, in one manual, three pages apart. Sound On Sound
independently describes the behaviour as "between 1:1 and 2:1 for smaller peaks, and gradually increases to
a ratio of up to **20:1** on loud input signals" [26]. **I would use 20:1 and note the 30:1.**

### 7.2 The static curves, which are the best thing in the document

Page 13 of the manual is a plot headed **"INPUT VS. OUTPUT CURVES"**, dBm out against dBm in, dated
**December 1959** and annotated "(supersedes March 1959 issue)" [9]. Five curves, each labelled with the
control positions that produce it, transcribed verbatim:

> 1. Straight amplifier, AC THRESHOLD control fully CCW, DC THRESHOLD control position nonimportant.
> 2. AC THRESHOLD control slightly CW from CCW position, DC THRESHOLD control fully CW.
> 3. **Factory-adjusted condition.**
> 4. AC THRESHOLD control fully CW and DC THRESHOLD control slightly CCW from CW position.
> 5. AC THRESHOLD control fully CW, DC THRESHOLD control slightly CW [from] CCW position.

I calibrated the axes against the chart's own heavy gridlines and printed labels and read the curves by eye
against a superimposed grid; the working image is
`ref/fairchild-670-input-output-curves-dec1959.png`. **Values below are ±0.5 dB.**

| dBm in | curve 1 (straight) | curve 3 (factory) | curve 4 (max limiting) | curve 5 (high output) | curve 2 (mild) |
|---|---|---|---|---|---|
| 0 | +2.0 | +2.0 | −4.5 | +2.0 | +2.0 |
| +5 | +7.0 | +4.3 | −2.5 | +7.0 | ≈ +5 |
| +10 | +12.5 | +5.3 | −1.0 | +9.5 | +7.5 |
| +15 | +17.5 | +5.7 | −0.3 | +10.0 | +9.3 |
| +20 | +22.5 | +5.9 | 0.0 | +10.2 | +10.5 |

**Six things this chart establishes that nothing else does.**

1. **Curve 1 is a straight line of unit slope.** With the AC threshold fully counter-clockwise the unit is
   linear over at least 30 dB, confirming the manual's "The unit is now a simple Unity Gain Line Amplifier"
   [1]. That is a test (12.1, test 2).
2. **The factory curve departs from linear at about +1 to +2 dBm in**, which agrees with the specification's
   "Predetermined level factory-adjusted to +2 dBm" [1] from a completely independent direction. Two
   Fairchild documents agreeing is worth more than either alone.
3. **The factory curve's plateau is about +6 dBm out** and it is genuinely flat: 12 dB more input between
   +12 and +24 dBm produces well under a decibel of output change.
4. **The ratio is progressive and the numbers are readable.** From +2 to +20 dBm in, the factory curve gives
   2.6 dB of output change for 18 dB of input change — an average of about **6.9:1**. From +10 to +20 dBm
   in, it gives 0.6 dB for 10 dB — about **17:1** (**derived** from the chart). That is precisely Sound On
   Sound's description of a ratio that starts low and climbs towards 20:1 [26], and it means a fixed-ratio
   compressor cannot imitate this curve at any setting.
5. **Gain reduction at the factory setting**, taking curve 1 as the unlimited reference: about **3.2 dB at
   +5 dBm in, 7.2 dB at +10, and 16.6 dB at +20** (**derived**). So the unit reaches the 10 dB of limiting
   that the distortion specification is quoted at somewhere around +13 dBm in, and never gets near the
   32 dB the tube could give (4.2).
6. **The DC threshold really does change the shape and not just the position.** Curves 4 and 5 have the AC
   threshold in the same place, fully clockwise, and differ only in the DC threshold — and they plateau
   14 dB apart, at 0 dBm and +10 dBm out. That is the internal trimmer doing exactly what Overloud say it
   does [40].

   > **Corrected by the build: 10.2 dB, not 14.** This paragraph's own transcribed table of the same
   > chart gives the two plateaux at 0.0 and +10.2 dBm out. The table is a reading and the prose is
   > an arithmetic slip on two numbers printed three lines above it, so the test asserts **10.2 dB**.
   > Asserting 14 would have meant asserting a figure this document's own data contradicts.

### 7.3 What has never been measured

- **No independent laboratory measurement of a Fairchild exists that I could reach.** Audio Science Review
  does not test studio compressors. This is the same category-wide gap the survey found across every
  candidate [52], and the Fairchild is now better off than most of them only because the *manufacturer*
  published charts.
- **No plug-in vendor publishes a null test or any measurement.** Universal Audio, Waves, IK, Softube,
  Overloud and Slate between them publish not one number about their own models' behaviour (section 9).
- **The two program-dependent releases have never been quantified by anybody**, including Fairchild, who
  described them in words. My derivation in 5.5 is the only quantification I know of and it is a derivation,
  not a measurement.
- **Raffensperger validated against SPICE, not hardware.** He says so, and he says nobody had published a
  model of the 670 before him [18]. So his eight tube parameters, his `A_v = 8.4`, his `R_out = 160 Ω` and
  his `I_max = 0.5 A` are fits to a simulation of a circuit, and the only place his work touches real
  measurement is the GE datasheet curves — which is why I checked his tube equation against them (4.3) and
  why that check matters.
- **Frequency response is published only as a tolerance band**, 40 Hz to 15 kHz ±1 dB [1]. There is no curve
  and therefore no way to know the shape inside the band or the slopes outside it.
- **Noise is published as one number**, 70 dB below +4 dBm [1], with no weighting and no bandwidth. POM
  Audio Design, who build new ones, describe the original's noise as "the poor −70 dBu figure" and claim
  their own is "roughly 20 dB quieter: between −90 dBu and −96 dBu" [45] — a manufacturer's claim about a
  competitor's product, which is worth exactly what that is worth, but it is at least consistent with
  Fairchild's own number.
---

## 8. How it is described as sounding, and what an emulation must get right

### 8.1 The published descriptions

I am quoting rather than paraphrasing, because most writing about this box is adjectives and the few
concrete statements are worth isolating.

**Fairchild themselves**, and note that every claim is about the *absence* of something [1]:

> A radical departure from the classical limiter design in the MODEL 670 is characterized by the complete
> absence of audible thumps, absence of distortion and noise, and it is extremely stable over long periods
> of time.

> Owing to the wide choice of attack and release time, as well as the automatic recovery feature, this unit
> can be used to limit program material severely without producing the audible thumps or pumping so often
> associated with limited program material.

**Fairchild on how to choose a time constant**, which is the most practical paragraph anyone has written
about this box [1]:

> Position 3 is merely a first suggestion for a general purpose timing circuit. With certain popular music
> or speech a much faster time constant, such as positions 1 or 2, might be more desirable. For classical
> music a much slower position, such as 4, is useable. Positions 5 and 6 combine fast release with slow
> release and can be useful if a great amount of automatic level correction is required. These positions
> should also make the limiter action least audible since they will reduce overall program level if
> continual limiting persists.

Sound On Sound's paraphrase — "the two fastest time-constant settings were recommended for pop productions,
and the following two presets for classical music" [26] — reverses the emphasis slightly; the manual
suggests 1 or 2 for pop and 4 for classical, with 3 as the default.

**Geoff Emerick**, on the drums, via Lewisohn [19] [24]: *"It became the sound of Revolver and Pepper
really. Drums had never been heard like that before."*

**Sound On Sound** [26]: the soft knee is *"an inherent feature of the variable-mu compression principle"*.

**Softube** [39], who are describing their own product but making a claim about the hardware: *"the unit is
known for adding sound and harmonic distortion even when you don't do a thing."*

**Waves** [37]: *"Their open, warm sound and fast attack made them ideal for creative transient shaping and
for 'gluing' mixes via stereo bus compression."* And, more usefully, a statement about mechanism:
*"Compression takes place in the audio path of the tube itself, rather than being routed to a separate gain
control circuit like a FET, VCA, or Opto compressor."*

**Universal Audio** [34]: *"Renowned for its aggressiveness, the 660 punches up piano, bass, and guitar
tracks, while the flagship 670 is a full stereo compressor that injects vibe and color."*

**Slate Digital** [41], the most specific description of the *tonal* change anyone offers: *"The midrange
thickens, the lows get tighter and rounder, and the top end opens up with a beautiful sparkle… even better,
those harsh upper mid overtones seem to get tamed too."*

### 8.2 What an emulation must get right, in priority order

My ordering, and I am prepared to defend the order rather than the list.

1. **Distortion must rise with gain reduction and it must not be separable from it.** This is the identity in
   4.6 and Fairchild published the measurement in 4.6's chart [10]. If the plug-in has a drive knob that
   dirties it without compressing, or a mode that compresses hard while staying clean, the model is not a
   variable-mu model, it is a compressor with a saturator bolted on. **This is the one that would make me
   throw the build away.**
2. **The knee must be progressive, and it must be a control.** The static curves show a ratio climbing from
   near 1:1 to about 17:1 over 20 dB [9], set by a trimmer inside the box that every good emulation brings
   out (5.2). A fixed ratio, or a ratio knob with numbers on it, is the wrong shape.
3. **Positions 5 and 6 must be an RC network, not a switch statement.** Position 6 releasing in 0.3 seconds
   after a short peak and 25 seconds after a sustained one is the same three capacitors in two states
   (5.5). Implement the network.
4. **Attack must be slew-limited, and must therefore be slower in position 4.** 0.10 ms per microfarad of
   timing capacitance (5.6). A fixed attack per position that ignores where the capacitor started is wrong
   for the same reason as (3).
5. **Third harmonic, not second, when balanced.** Push-pull cancels even orders (4.6). If the plug-in offers
   a mismatch or "age" control, that is what should bring second harmonic in, and it should be small.
6. **No thump.** The control voltage is common-mode into a push-pull stage and cancels at the output
   transformer (4.5). If the model needs a control-signal smoother to stop clicking, the topology is wrong.
7. **Lateral-vertical is two independent limiters on mid and side**, not a linked stereo pair (6.3).
8. **The low corner is the 4 µF cathode bridge**, and it is a real, published-in-a-schematic component that
   makes the 670 and the 660 different machines (1.3, 3.3).

### 8.3 What I would not bother with

- **The transformers' fine detail.** Raffensperger publishes six parameters for each of three transformers
  [18] and they earn their place in a wave-digital-filter paper. In a spoof they buy a gentle low corner and
  a gentle top-end softening that a second-order shelf will imitate at a hundredth of the cost.
- **The power supply.** It is regulated to 240 V by a valve loop with a gas reference and specified stable
  from 100 to 127 V mains (3.4). There is nothing to model.
- **Twenty tubes.** Eight sections per channel is one equation evaluated twice with a scale factor (4.4).
  The other twelve valves are the sidechain and the supply, and the sidechain is better modelled by
  Raffensperger's five black-box equations than by five more tube models.
- **Hum.** Waves offer "original hum and noise floor (50/60 Hz) for full analog authenticity" [37]. The
  Fairchild's published noise is 70 dB below +4 dBm [1] and it is a DC-heated, regulated design. I would
  offer noise as an off-by-default toy and would not pretend it is authenticity.
- **The remote meter, the remote threshold faders, and the Model 722.** Period detail (2.6).

---

## 9. Existing emulations, and what each claims

Nobody publishes a measurement. What follows is what each vendor says about their own product, which is
useful for two things: knowing what the field has already decided is important, and knowing which controls
everyone adds because the hardware is missing them.

### 9.1 Universal Audio — Fairchild Tube Limiter Collection

The reference implementation, in the sense that everyone else is measured against it. $299 at release,
announced **18 November 2013**, replacing a "Fairchild 670 Legacy" plug-in that UA had shipped in **2004**
[34] [35] [36].

Claims:

> Far beyond other Fairchild emulations, only the UAD Fairchild Collection is based on accurate circuit
> models of "golden-reference" units from legendary Ocean Way Studios [34]

> Today, UA's team of DSP experts have improved the original time constants and gain reduction curves while
> modeling — for the first time ever — the complete tube-powered amplifier and transformer sections of their
> hardware counterparts [35]

> The models follow the variable-mu tube gain control and transformer behavior of the originals, not just
> their broad compression curve [34]

Added controls: **Wet/Dry Mix**, **Headroom** (*"raises the compression threshold and lowers distortion for
clean mastering, or lowers it for grit and drive"*), and **sidechain filters** [34]. The 2004 release notes
are unusually candid about what they changed and why:

> the meter select switch positions, which were used to calibrate the meter on the original hardware, were
> removed since the software version of the meters have the benefit of not requiring recalibrating! The
> positions on the switch were replaced with highly useful input and output level metering views and the
> 'zero' screw-slot control was disabled. [36]

They also note that "The DC bias controls are original controls (on the back of the real hardware units)",
that their sidechain link is "a common mod, which had been done to the Ocean Way 670 that UA modeled", and
that the controls link and output level are plug-in-only additions [36].

**My reading.** Their "Headroom" is the ZERO bias trim under a friendlier name (3.5, 4.7), and disabling the
zero screw while adding Headroom is the same decision made twice. Note also that they say the DC bias
controls are on the **back** of the hardware; my documents say the DC THRESHOLD is a screwdriver adjustment
**inside the chassis** [1] and the 660 drawing's symbol key distinguishes front-panel screwdriver
adjustments from ones at the **rear of chassis** [5]. Different units may have been modified differently.

### 9.2 Waves — PuigChild 660 and 670

Modelled from Jack Joseph Puig's own units [37]. Claims include *"Accurate circuit modeling with total
harmonic distortion"*, *"Variable-mu tube topology reacts to input load for well-balanced sonics"*, and a
mechanism statement quoted in 8.1. Modes: mono, dual mono, stereo, M/S. Adds hum and noise at 50/60 Hz.

**One number on the page is wrong and worth flagging** because it will be copied: *"turn the 6-step Time
Constant to control attack (200-400 ms) and release (from 300 ms to a super-slow 25 seconds)"* [37]. The
attack is **200 to 400 microseconds**, not milliseconds — the release figures in the same sentence are
right. Someone converted the units in the wrong direction.

### 9.3 IK Multimedia — Vintage Tube Compressor/Limiter Model 670

*"based on the 'Holy Grail' of compressor/limiters"*, *"a faithful reproduction of each control"*, *"one
considered a true 'golden reference'"* [38]. The most interesting thing on the page is that they implement
the matrix as the hardware does rather than as a mode:

> As in the hardware unit, our model actually consists of two separate and independent limiters… This is the
> only module in the T-RackS series that doesn't feature the standard "L/R-M/S" selector, as this
> functionality is achieved with the standard controls that were on the original hardware. [38]

That is the right call and it is the one I would copy (6.3). Their page also carries the trademark notice:
*"Fairchild® is a registered trademark property of Avid Technology, Inc."* [38].

IK's **Dyna-Mu** is a different, unnamed American variable-mu unit with a 1.5:1 / 4:1 hard switch [48]. It
is not a Fairchild and should not be cited as one.

### 9.4 Softube — Bus Processor 670

*"Our modern take on the legendary dual-channel variable-mu tube compressor"*, explicitly not a replica:
*"This plug-in is the product of our quest to discover where the magic of the mythological 670 hardware lies
– and distil it"* [39]. Adds tube and transformer saturation knobs, *"a calibration knob to influence the
unique compression-dependent saturation of the variable-mu architecture"*, sidechain filters, tone shift,
mono/stereo linking, separate M/S outputs, and spatialisation (air band, mono maker, width).

**Their calibration knob is the same idea as UA's Headroom and as the hardware's ZERO**, and the phrase
"compression-dependent saturation" is the identity of 4.6 stated as a product feature. Softube are also the
source of the "fewer than a thousand 670s were ever made" figure [39].

### 9.5 Overloud — Gem COMP670

€139, and the source of the single most technically useful vendor claim I found anywhere [40]:

> Compression curve tweaking — A trimmer inside the original unit allowed to change the compression
> characteristic from soft-knee to hard-knee by changing the biasing current of the feedback amplifier. The
> [Comp670] brings this control on the front panel allowing to tweak the compressor response to dynamic.

That is the DC THRESHOLD (5.2), described correctly, by someone who has opened one. They also sample three
different physical units — *"found in London, Los Angeles and Milan studios"* — on the argument that
*"each original unit has been serviced multiple times: it has its own history and component tolerance that
gives it a unique tone"* [40], and expose meter calibration at four sensitivities.

### 9.6 Slate Digital — FG-MU

Part of Virtual Buss Compressors, $149 [41]. **Slate does not claim to model a Fairchild.** Steven Slate
names the Fairchild 670 and the Manley Vari Mu as things he has used and admires, and then says *"we put all
of these wonderful qualities into the algorithm"*. That is an honest statement of inspiration and it is a
different claim from everyone else's on this page; it should not be listed as a Fairchild emulation.

### 9.7 Hardware recreations, which publish more circuit detail than the plug-ins

- **POM Audio Design, "FAIRCHILD 670 mkII"** [45] [46], £6,990–£8,490 in their 2026 list. They claim
  *"The AUDIO path on the FAIRCHILD 670 mkII is identical to the original version: It has the same input
  transformers, the same tubes, and the same output transformers"*, using original transformers "at least 50
  years old", and replace the valve sidechain with *"a 200W Discrete Power Amplifier… The Amp has the same
  topology as the original: it has an input and an output transformer and behaves the same way"*, saving
  twelve valves for a total of eight instead of twenty. They add a sidechain filter section and a
  feed-forward/feedback blend, and they are the source of the metering explanation in 2.5.
- **Heritage Audio HERCHILD 670, GRANDCHILD 670/500, HERCHILD 670N** [42] [43] [44]. Their published
  specifications for their own units: input 2 kΩ, output 600 Ω, +30 dBu clipping, 20 Hz–20 kHz ±1 dB,
  self-noise below −82 dBu, THD under 1 % up to +18 dBu, over 200 W, 6U, 13.3 kg. **These are Heritage's
  numbers for Heritage's product** and must not be quoted as the Fairchild's; note in particular that their
  "22 valves and 9 transformers" contradicts every count for the original (3.6), and the 500-series version
  uses **6BA6 pentodes**, not 6386 triodes.
- **UnderTone Audio's "UnFairchild 670M"** exists but their site was unreachable (HTTP 000) so I have no
  claims from them.

### 9.7a Three products that are not Fairchild emulations and get mistaken for them

Worth naming, because all three come up in searches for this unit and none of them is a model of it.

- **NEOLD V76U73** models the Telefunken U73, which Plugin Alliance describe as *"With the nickname 'The
  German Fairchild,' the U73 compressor/limiter used a variable mu design that made it incredibly popular
  with mastering engineers in Europe until 1980"* [47]. A different variable-mu unit with a nickname.
- **Arturia Comp TUBE-STA** models the **Gates STA-Level**, *"a reimagining of the legendary vacuum-tube
  powered Gates' STA-Level"* [49]. A contemporary of the Fairchild and a different design; the survey
  advised against modelling the Sta-Level because no schematic for the 1956 original is reachable [52].
- **Gyraf G22** is Gyraf's own variable-mu lineage, *"an elaborated and dual'ified version of our venerable
  G10 vari-mu compressor"*, with its own internal mid-side matrixing and a sidechain high-pass at 80 Hz
  [50]. Gyraf make no Fairchild claim, and they publish **no** variable-mu DIY project, which is worth
  recording because their DIY pages are otherwise the first place to look for a schematic.

### 9.8 What none of them does

- **Nobody publishes a measurement of their own model**, let alone a null test against hardware.
- **Nobody models the 660's different operating point.** UA, Waves and IK all ship a "660", and every
  description of it I read treats it as the 670 in mono. The cathode resistors differ by 2.6× (1.3) and
  nobody mentions it.
- **Nobody exposes the attack difference at position 4** (5.6), because nobody appears to have noticed that
  the manual's own attack table contradicts the circuit.
- **Nobody derives the release times from the network.** Raffensperger publishes the network and does not
  compute the releases; every other source repeats the manual's six numbers as given.

That last one is the gap this dossier fills, and it is the reason section 12's dynamics tests can assert the
manufacturer's figures against an emergent behaviour rather than against a coefficient the model was handed.
---

## 10. Recommended DSP design

### 10.1 The shape of the engine, and what is new about it

`dsp::vmu`. Per channel, per sample, at an oversampled rate:

```
        ┌──────────────────────────── feedback ───────────────────────────┐
        │                                                                 │
in ─ atten ─ xfmr_in ─┬─ Vg1 ─┐                                           │
                      │       ├─ push/pull remote-cutoff pair ─ xfmr_out ─┴─ out
                      │  Vg2 ─┘         ▲
                      │                 │ Vx = Vsc + Vbias
                      └── common-mode ──┘
                                        ▲
   Vsc ── RC network (3 states) ── rectifier ── clip ── deadzone ── AC thresh ── pad ── (output)
```

Four things about this shape are new to the codebase and they are the reasons to build it rather than
reskin something:

1. **There is no gain multiplier anywhere.** No `y = x * g`. The output is the difference of two tube
   currents, and the control voltage changes those currents. Everything the model does about compression it
   does by moving a bias.
2. **The detector is in a feedback loop around a nonlinearity** (3.1), so the loop must be closed with a
   one-sample delay and the model must be oversampled enough that the delay is short compared to the
   attack.
3. **The release network is a three-state RC**, not a switched one-pole (5.5).
4. **Two channels with a matrix on both sides**, not a link (6.3).

Reuse from what exists: the 1176's oversampler and feedback-loop structure; the 610's transformer shaping in
`dsp::pre`; the framework's stepped-parameter and metering plumbing. **Not** the 610's triode model (4.1).

### 10.2 Parameter table

All parameters prefixed `fc_`. The hardware's controls are stepped where they are stepped and continuous
where they are continuous, and the two screwdriver adjustments are exposed because they are the interesting
ones (2.1, 3.5, 5.2).

| parameter | type | range / values | default | source of the range |
|---|---|---|---|---|
| `fc_model` | enum | `660`, `670` | `670` | 1.3 |
| `fc_bypass` | bool | — | false | — |
| `fc_input_gain_l` | index 0–20 | 0 to 20 dB attenuation, 1 dB steps | index 10 (10 dB) | AT101, "1 db per step" [1]; default is the manual's unity-gain setting [1] |
| `fc_input_gain_r` | index 0–20 | as above | index 10 | AT201 [1] |
| `fc_threshold_l` | float | 0 to 10, panel units | 10.0 | R115a/b, panel scale [31] [32]; 10 is "fully CW" = the factory curve [9] |
| `fc_threshold_r` | float | 0 to 10 | 10.0 | R215a/b |
| `fc_time_l` | index 1–6 | 1, 2, 3, 4, 5, 6 | 3 | S102; default 3 is the manual's recommendation [1] |
| `fc_time_r` | index 1–6 | 1–6 | 3 | S202 |
| `fc_agc` | enum | `left_right`, `lat_vert` | `left_right` | S301 [1] |
| `fc_dc_threshold_l` | float | 0 to 1 | factory value, 10.4 | R117, the internal trimmer [1] [40] |
| `fc_dc_threshold_r` | float | 0 to 1 | factory value | R217 |
| `fc_zero_l` | float | −12 to −3 V of standing grid bias | −7.2 V | R142; Raffensperger's `V_bias` [18] |
| `fc_zero_r` | float | −12 to −3 V | −7.2 V | R242 |
| `fc_balance_l` | float | ±1, maps to ±50 Ω on the cathode split | 0 | R105a/b, 100 Ω [4] |
| `fc_balance_r` | float | ±1 | 0 | R205a/b |
| `fc_meter_l` | enum | `bal_push`, `zero`, `bal_pull` | `zero` | S101, three positions [1] [45] |
| `fc_meter_r` | enum | as above | `zero` | S201 |
| **spoof extras, not on the hardware** | | | | |
| `fc_mix` | float | 0 to 1, dry/wet | 1 | UA and Softube both add one [34] [39] |
| `fc_sc_hpf` | enum | `off`, `90Hz`, `180Hz` | `off` | POM's filter section names exactly these corners [45] |
| `fc_link` | bool | detector link across the pair | false | UA ship it and say it was a mod on the unit they modelled [36] |
| `fc_tube` | enum | `ge_6386`, `jj_6386lgp` | `ge_6386` | GE gm 4000 µmhos vs JJ 3000 [12] [15] |
| `fc_oversample` | enum | 4×, 8×, 16× | 8× | Raffensperger uses 8× at 44.1 kHz [18] |

**On `fc_threshold`.** The panel scale is 0 to 10 and it is not decibels; the pot is linear but tapped, with
an effective 76 kΩ law and a kink (5.2) [18]. Map the panel number to the pot's fractional position, apply
the piecewise law, and let the threshold in dBm fall out. Do not label the knob in decibels — the hardware
does not and the whole point is that this control and the DC threshold jointly set a curve, not a point.

**On `fc_zero`.** This is the one to be proud of. It is the hardware's meter-calibration screw, it actually
moves the tube bias (2.5, 3.5) [45], and moving it changes standing gain, available gain reduction and
standing distortion together in the direction the tube's curve dictates. It is the honest version of UA's
Headroom [34] and Softube's calibration knob [39], and unlike either of those it should *also* move the
meter, because on the hardware it does.

**On `fc_balance`.** Mismatching the two halves brings in second harmonic (4.6). At zero it should be
inaudible; at the extremes it should be a few per cent of second. This is the substitute for modelling eight
individual tubes (4.4) and it is one parameter instead of eight.

**No ratio control, no attack control, no release control.** All three are consequences of `fc_time`,
`fc_threshold` and `fc_dc_threshold`, and adding them would be adding controls the box does not have to
paper over mechanisms the model got wrong.

### 10.3 The blocks, in equations

Per sample at the oversampled rate `Fs`. Voltages in volts, currents in amps.

**Block A — input attenuator and transformer.**

```
x_att = x_in · 10^(−fc_input_gain/20)
v_sec = N_in · x_att                         N_in = step-up of T101
```

Model T101 as a gain plus a second-order high-pass at `f_lo` and a first-order low-pass at `f_hi`
(10.4). Do not build a wave digital filter; Raffensperger needs one because he is doing a WDF paper, and a
two-pole shelf pair is within a fraction of a decibel of it inside the audio band.

**Block B — grid voltages.** From 4.5:

```
Vx   = Vsc[n−1] + fc_zero                    common mode, one sample delayed (feedback)
Vg1  =  v_sec + Vx
Vg2  = −v_sec + Vx
```

**Block C — the push-pull remote-cutoff pair.** Per half, using Raffensperger eq. 1 [18]:

```
Ia(Vgk, Vak) = p1·Vak^p2 / [ (p3 − p4·Vgk)^p5 · (p6 + exp(p7·Vak − p8·Vgk)) ]

Vgk_clamped = min(Vgk, −0.5 V)               guard: eq. 1 diverges above +5 V (4.3)
```

Each half is `n_par = 4` sections in parallel with a cathode resistor, so per half solve for the cathode
voltage:

```
Vk_h  = R_k_h · n_par · Ia(Vg_h − Vk_h, Vplate − Vk_h)
```

That is one scalar fixed point per half per sample. Two Newton steps from the previous sample's `Vk_h`
converge to well under a millivolt at 8× oversampling because `Vk` moves slowly; a full solve is not needed.
Then:

```
I_push = n_par · Ia(Vg1 − Vk_1, Vplate − Vk_1)
I_pull = n_par · Ia(Vg2 − Vk_2, Vplate − Vk_2)
v_out  = R_L · (I_push − I_pull) · N_out
```

**The difference is the whole compressor.** Common-mode control voltage cancels here (4.5), so no
anti-thump smoothing is needed anywhere. `R_k_1` and `R_k_2` are `R_k ± 50 Ω · fc_balance`, which is the
balance pot.

**Block D — output transformer and the cathode bridge.** The 4 µF between the two cathode nodes (3.2)
sets the low corner. In the lumped model, put a one-pole high-pass at `f_c1` between `Vk_1` and `Vk_2`'s
common behaviour, or more simply apply a first-order high-pass at `f_c1` to the *difference* current before
the output transformer, which is equivalent to first order and is one multiply-add. Then T102's shaping.

**Block E — the sidechain feed.** From 3.2:

```
v_sc_in = v_out · G_pad · N_sc              G_pad = R_term/(R_term + R_in), N_sc = 17
```

**Block F — AC threshold and the dead zone.** From 5.2 and Raffensperger eq. 10 [18]:

```
V_pot     = φ_AC(fc_threshold) · v_sc_in / 2
φ'_DC     = 12.2 · (fc_dc_threshold + 0.1)
V_stage1  = ln[ (1 + exp( V_pot − φ'_DC)) / (1 + exp(−V_pot − φ'_DC)) ]
```

`φ_AC` is the tapped-pot law: a piecewise-linear map from the panel's 0–10 to a fraction, with the kink at
the tap (5.2). Use `log1p(exp(·))` in its numerically stable form, or the equivalent `softplus`, so that
large `|V_pot|` does not overflow.

**Block G — stages two and three.** Raffensperger eq. 11 [18]:

```
V_stage3 = clamp(A_v · V_stage1, −V_clip, +V_clip)
```

**Block H — rectifier with a soft knee, and the current limit.** Raffensperger eqs. 12–14 [18]:

```
V_diff = |V_stage3| − (Vsc_p[n−1] − Vsc_n[n−1])
I_nom  = (2·V_d)/(λ·R_o) · ln[ 1 + exp( λ·V_diff/(2·V_d) − λ ) ]
I_sc   = I_nom − (I_max/10) · ln[ 1 + exp( 10·I_nom/I_max − 10 ) ]
```

The second line is the slew limit that makes attack proportional to `C_T` (5.6). **Do not replace it with a
per-position attack coefficient**, because then position 4 will be wrong and the attack will not depend on
how charged the network already is.

**Block I — the time-constant network, which is the interesting block.** Three states, from the table in
5.3. With `q_T`, `q_U`, `q_V` the charges (or equivalently `v_T`, `v_U`, `v_V` the voltages) on `C_T`, `C_U`
and `C_V`:

```
i_R  = v_T / R_T
i_U  = (v_T − v_U) / R_U          (0 if leg U is out of circuit)
i_V  = (v_T − v_V) / R_V          (0 if leg V is out)

dv_T/dt = ( I_sc − i_R − i_U − i_V ) / C_T
dv_U/dt =   i_U / C_U
dv_V/dt =   i_V / C_V

Vsc = −v_T                        the control voltage, negative-going at the grids
```

Integrate with the trapezoidal rule (bilinear), which at 8× oversampling is exact to well below the
accuracy of the component values. **Switching `fc_time` must not reset the states**, because a real rotary
switch does not discharge the capacitors; the legs that leave the circuit keep their charge and the ones
that join bring theirs. That is a two-line detail that gives the model the right behaviour when someone
turns the knob during a passage, which is exactly the kind of thing this box is famous for surviving.

**Block J — the matrix.** When `fc_agc = lat_vert`:

```
lat = (L + R) / 2                 in
ver = (L − R) / 2
                                  ... two independent channels ...
L   = lat + ver                   out
R   = lat − ver
```

Two independent limiters, no shared detector (6.3). With `fc_agc = left_right`, both matrices are identity.

**Block K — the meter.** From 2.5. In `zero` the reading is the change in `I_push + I_pull` from the
balanced standing value, scaled to the VU face; in the two `bal` positions it is `I_push` or `I_pull`
alone. `fc_zero` moves the standing point and therefore moves the needle, which is what the hardware does.

### 10.4 Constants

**Where each number comes from is the point of this table.** M = the December 1959 manual [1]; S670 = the
670 schematic [4]; S660 = the 660 factory drawing [5]; GE = the 6386 datasheet [12]; R = Raffensperger's
published value [18]; D = my derivation; E = my estimate.

| symbol | value | source |
|---|---|---|
| **Tube law** | | |
| p1 | 3.981 × 10⁻⁸ | **R** [18] |
| p2 | 2.383 | **R** |
| p3 | 0.5 | **R** |
| p4 | 0.1 | **R** |
| p5 | 1.8 | **R** |
| p6 | 0.5 | **R** |
| p7 | −0.03922 | **R** |
| p8 | 0.2 | **R** |
| `Vgk` clamp | −0.5 V | **D**, eq. 1 diverges at +5 V (4.3) |
| eq. 1 usable range | **0 to −30 V of grid**; −8.6 dB at −50 V, −35.7 dB at −70 V | **D** (4.3) |
| checked against GE curves at (250 V, −10/−30/−50 V) | 19.9 / 4.15 / 0.56 mA | **D** (4.3) |
| eq. 1's gm at the class-A1 point | 2309 µmho against GE's 4000 — **do not use its derivative** | **D** (4.3) |
| gm law from GE's logarithmic plot | at or below exponential, **n = 0.59 to 1.00** across GE's four plotted conditions | **D** (12.3a) |
| μ, GE 6386 | 17 | **GE** [12] |
| gm at the class-A1 point | 4000 µmhos | **GE** |
| gm at Vg = −16 V | 100 µmhos | **GE** |
| **published gain-control range** | **32.0 dB** | **D** from the two rows above (4.2) |
| rp | 4250 Ω | **GE** |
| gm, JJ 6386 LGP | 3000 µmhos, μ 18 | **M**, JJ's own page [15] |
| **Gain stage, 670** | | |
| sections per half, `n_par` | 4 | **S670** [4] |
| cathode resistor `R_k` | 680 Ω | **S670** |
| balance pot | 100 Ω, ±50 Ω a side | **S670** |
| Raffensperger's lumped `R_11 = R_21` | 705 Ω | **R** (= 680 + half the pot) |
| cathode bridge `C_1` | 4 µF | **S670**, and **R** names it as the LF-determining part |
| plate rail | 230 V, transformer centre tap 240 V | **S670** annotations |
| meter sense | 30 Ω (R107/R108) | **S670**; **R** uses 33 Ω |
| standing bias `V_bias` | −7.2 V | **R** |
| `V_bal` | −3.1 V | **R** |
| **Gain stage, 660 — the only 660 constant I trust** | | |
| cathode resistor | **1800 Ω** | **S660** [5] |
| balance pot | 500 Ω | **S660** |
| grid injection resistors | 100 kΩ each (R1, P2) | **S660**; **R** as `R_L1 = R_L2` |
| **Sidechain feed** | | |
| pad series `R_in` | 4 × 150 Ω = 600 Ω | **S670** [4] |
| pad shunt `R_term` | 2 × 680 Ω = 1360 Ω | **S670** |
| T103 step-up | `N_p/N_s = 1/17` | **R** |
| **Sidechain amplifier** | | |
| AC threshold pot | 100 kΩ linear, tapped with 24 kΩ, effective 76 kΩ piecewise | **R**; the 24 kΩ parts are R8/R9 on **S660** |
| AC threshold pot, 660 | 180 kΩ | **S660** |
| DC threshold pot | 100 kΩ linear | **S670**, **S660**, **R** |
| dead-zone map `φ'_DC` | 12.2 (φ_DC + 0.1) | **R** |
| stage 2–3 gain `A_v` | 8.4 | **E** (Raffensperger's fit to *his simulation*) |
| clip `V_clip` | 100 V | **E** (same) |
| rectifier `R_o` | 160 Ω | **E** (same) |
| rectifier `V_d` | 0.3 V | **E**; and the diode type is unestablished (5.7) |
| rectifier softness `λ` | 10 | **R** |
| current limit `I_max` | 0.5 A | **E** (Raffensperger's fit) |
| **implied control swing** | **≈ 50 V** | **D** from `I_max` and 0.10 ms/µF (5.6) |
| **Time-constant network — every value from the factory drawing** | | |
| `R_T` fixed | 220 kΩ (R32 / R137) | **S660**, **S670** |
| `C_T` fixed | 2 µF / 200 V (C7 / **C115, marked `???` on S670**) | **S660**; **D** for the 670 (5.3) |
| position 1 shunt R | 68 kΩ (R37 / R141) | **S660**, **S670** |
| position 2 shunt R | 470 kΩ (R33 / R138) | **S660**, **S670** |
| switched C, positions 3–5 | 2 µF (C11 / C113) | **S660**, **S670** |
| switched C, position 4 | +4 µF (C9 / C111) | **S660**, **S670** |
| slow leg U | 100 kΩ + 8 µF (R34/C8, R139/C110) | **S660**, **S670** |
| slow leg V | 100 kΩ + 20 µF (R35/C10, R140/C112) | **S660**, **S670** |
| RF bypass | 0.1 µF (C109) | **S670** |
| `R_T` position 1 | 51.94 kΩ | **D** |
| `R_T` position 2 | 149.86 kΩ | **D** |
| `R_T` position 6, legs empty | 40.74 kΩ | **D** (5.5) |
| leg U charging τ | 0.8 s | **D** |
| leg V charging τ | 2.0 s | **D** |
| **release constant `k`** | **2.59 ± 0.27** | **D** (5.4) — published release = k · R_T·C_T |
| **attack constant** | **0.10 ms per µF of `C_T`** | **D** (5.6), matches SOS's six values exactly |
| **Levels and response** | | |
| threshold, factory | +2 dBm | **M** [1] |
| clipping point | +27 dBm | **M** |
| plateau, factory curve | ≈ +6 dBm out | **D** from the published chart [9] |
| LF corner `f_lo` | ≈ 40 Hz at −1 dB | **M**, the response band [1]; **E** as a corner |
| HF corner `f_hi` | ≈ 15 kHz at −1 dB | **M**; **E** as a corner |
| `f_c1`, the cathode bridge | **not established** | see below |
| noise floor | −70 dB relative to +4 dBm | **M** |

**On `f_c1`.** Raffensperger says the 4 µF cathode bridge determines the low-frequency response [18] and the
manual publishes 40 Hz at −1 dB [1]. Those two facts constrain each other but do not give a corner, because
the impedance the capacitor works into is the two cathode resistors in parallel with the tubes' cathode
impedances, and the latter depends on the operating point and therefore on the gain reduction. **That is
worth saying out loud: the Fairchild's low-frequency corner moves with gain reduction.** Set `f_c1` so the
model meets the published 40 Hz / −1 dB at zero gain reduction and let it move; do not pin it.

### 10.5 Oversampling, rates and hygiene

**Oversample at 8×.** Raffensperger uses 8× at 44.1 kHz and gives three reasons, all of which apply here
[18]: aliasing of the tube nonlinearity's products; reduction of the bilinear transform's frequency warping;
and — the one people forget — shortening the fictitious one-sample delays that break the feedback loop, so
that the delay is small compared to a 200 µs attack. At 44.1 kHz, 8× gives a loop delay of 2.8 µs against a
200 µs attack, which is 1.4 %. At 1× it would be 11 %, which would visibly slow the fastest positions.

Offer 4× for weak machines and 16× for offline, and scale so that 96 kHz sessions use fewer factors for the
same effective rate.

**Hygiene.** Flush denormals; the RC network's slow states decay for tens of seconds and will hit denormal
range. Clamp `Vgk` (10.4). Guard the two `log1p(exp(·))` calls against overflow. Reset the tube fixed point
to its quiescent solution on a rate change, not to zero. Do not reset the RC states when `fc_time` changes
(10.3, block I).

### 10.6 What the page should show

- The **panel**, laid out as 2.2, in the plug-in's own colours and not Fairchild's (the framework rule is
  that every look belongs to the example, not the framework).
- **Two meters**, with the three-position metering switch working, because it is the strangest control on
  the box and because disabling it, as Universal Audio did [36], throws away the one place the hardware
  admits that its meter is a valve tester.
- A **gain-reduction trace** and, next to it, a **live plot of the static curve** the current
  `fc_threshold` / `fc_dc_threshold` pair produces, with the manual's five published curves drawn behind it
  in grey [9]. That is the single most educational thing this plug-in could put on screen: it shows the two
  controls jointly bending one curve, and it shows the model against the manufacturer's own measurement.
- A **distortion-versus-gain-reduction readout**, with Fairchild's 1959 chart behind it [10]. Same argument.
- The **charge state of the three timing capacitors**, as three small bars. Positions 5 and 6 are
  incomprehensible without it and obvious with it.
---

## 11. Test plan

**Two standards this repository enforces, and both apply here.** Every test asserts a **published figure**,
names it, and cites it. Where no real number is reachable, the test says so and asserts a *direction*, an
*ordering* or a *circuit identity* instead of an invented bound. An audit found nine tests across five
plug-ins that compared a model against its own output; those can never fail, and this file is written so
that none of its tests can be written that way.

**The Fairchild is unusually well supplied with figures**, and it is worth saying where they come from
before the list, because the strength of a test is the strength of its anchor:

| anchor | what it gives | strength |
|---|---|---|
| the December 1959 specification page [1] | 20 figures, no tolerances | manufacturer specification |
| the input/output curve chart, Dec 1959 [9] | 5 static curves with their control settings | **manufacturer measurement** |
| the IM distortion chart, Mar 1959 [10] | 7 curves of IM against gain reduction | **manufacturer measurement** |
| the GE 6386 datasheet [12] | the tube's law and its 32 dB control range | component manufacturer data |
| the 660 factory drawing [5] | every timing component value | primary circuit document |
| Sound On Sound's attack table [26] | six attack times, one of which the manual gets wrong | secondary, but confirmed by the circuit (5.6) |

Unless a test says otherwise: 1 kHz sine, `fc_model` = `670`, `fc_agc` = `left_right`, `fc_time` = 3,
`fc_input_gain` = index 10, `fc_threshold` = 10.0 (fully clockwise, which is the factory curve), 8×
oversampling, run at 44.1, 48 and 96 kHz.

### 11.1 Static behaviour and calibration

1. **Bypass is exact.** `fc_bypass` on: output equals input to 1 × 10⁻⁶.
   *Figure:* none needed; this is an identity, and the test says so.

2. **With the AC threshold fully counter-clockwise the unit is a linear amplifier.** `fc_threshold` = 0.0,
   sweep the input from −10 to +24 dBm: the output must be a straight line of slope **1.00 ± 0.02** with no
   more than **0.3 dB** of departure anywhere.
   *Figure:* curve 1 of the published input/output chart, "Straight amplifier, AC THRESHOLD control fully
   CCW", which is a straight line over that range. *Source:* the December 1959 manual, p. 13 [9]. Also
   asserted in prose: "Turning the AC THRESHOLD controls completely counterclockwise removes the limiting
   action completely. The unit is now a simple Unity Gain Line Amplifier" [1].

3. **The factory curve's threshold is +2 dBm.** `fc_threshold` = 10.0, `fc_dc_threshold` at its factory
   value: the input at which the transfer curve departs from linear by 1.0 dB must be **+2 dBm ± 1.5 dB**.
   *Figure:* "Predetermined level factory-adjusted to +2 dBm". *Source:* the specification page [1], and
   independently the departure point of curve 3 on the chart [9], which I read at +1 to +2 dBm.

4. **The factory curve's plateau.** Same settings, inputs of +12, +16, +20 and +24 dBm: every output must
   lie within **±1.0 dB of +6 dBm**, and the total output change across that 12 dB of input must be under
   **1.0 dB**.
   *Figure:* curve 3, "Factory-adjusted condition", read at +6 dBm out and flat. *Source:* [9]. **This is
   the strongest static test in the file** — it asserts a manufacturer measurement of a named control
   setting, and it fails if the knee, the ratio law or the sidechain gain is wrong.

5. **The progressive ratio.** Same settings. Measure the output change for a 10 dB input increase at two
   places:

   | input range | required output change |
   |---|---|
   | +2 → +12 dBm | **3.3 dB ± 1.0 dB** (≈ 3:1) |
   | +10 → +20 dBm | **0.6 dB ± 0.6 dB** (≈ 17:1) |

   *Figure:* curve 3 of the published chart, read at those four input levels [9]. *Corroboration:*
   "Gain reduction starts with a very low ratio, between 1:1 and 2:1 for smaller peaks, and gradually
   increases to a ratio of up to 20:1 on loud input signals", Sound On Sound [26]. **A fixed-ratio
   compressor cannot pass both halves of this test**, which is the point of it.

6. **The tube's gain-control range.** Not a plug-in test but a `dsp::vmu` unit test on the tube block alone.
   Evaluate `gm = ∂Ia/∂Vgk` at the datasheet's class-A1 operating point (plate 100 V, cathode resistor
   200 Ω, plate current 9.6 mA) and again at `Vgk = −16 V` with the same plate voltage. The ratio in decibels
   must be **32.0 dB ± 3 dB**.
   *Figures:* transconductance 4000 µmhos at the operating point, and "Grid Voltage, approximate,
   Gm = 100 Micromhos, −16 Volts". *Source:* General Electric datasheet ET-T1113, "Characteristics and
   Typical Operation, Class A₁ Amplifier, Each Section" [12]. **This is the only test in the file that
   asserts a component manufacturer's data rather than Fairchild's**, and it is the one that would catch a
   wrong tube model before anything else did.

7. **The tube equation reproduces the datasheet curves.** Same unit test. Evaluate Raffensperger's eq. 1 at
   `Vak` = 250 V and `Vgk` = −10, −30 and −50 V; require **19.9, 4.15 and 0.56 mA, each ± 25 %**.
   *Figures:* the plate currents at those points on the "Average Transfer Characteristics, Each Section"
   plot, which I read as ≈20, ≈4 and ≈0.5–1 mA. *Source:* the GE datasheet [12], the same family
   Raffensperger's Fig. 2 reproduces [18]. The tolerance is wide because the anchor is my eye on a 1953
   graph, and the test says so.

8. **Push-pull cancels even harmonics.** `fc_balance` = 0, `fc_threshold` = 0 (linear), 1 kHz at +18 dBm:
   second harmonic must be at least **20 dB below** third harmonic. Then `fc_balance` = 1.0: second harmonic
   must **rise by at least 12 dB**.
   *Figure:* none — **no published figure exists for the Fairchild's harmonic spectrum, and I say so.** What
   is asserted is a *circuit identity*: a balanced push-pull stage cancels even orders and an unbalanced one
   does not, which is why the hardware has a BALANCE control and why the manual's balancing procedure ends
   "exchange one or more of the 6386 tubes" [1]. The test asserts the ordering and the direction of change,
   not a magnitude.

### 11.2 Distortion, which is the family's whole point

9. **IM distortion rises with gain reduction, at Fairchild's own numbers.** SMPTE conditions: 60 Hz and
   7 kHz mixed **4:1**, which is what the chart specifies. Adjust `fc_threshold` and the input to hold the
   output at +12 dBm while producing 0, 5, 10 and 15 dB of gain reduction, and measure IM:

   | gain reduction | required IM |
   |---|---|
   | 0 dB | **below 1.0 %** |
   | 10 dB | **below 1.0 %** |
   | 15 dB | **above the 10 dB figure** |

   *Figures:* "Less than 1% at any level up to +18 dbm output (no limiting)" and "Less than 1% at 10 db
   limiting and +12 dbm output", from the specification page [1]; and curve 4 (+12 dBm out) of the chart
   "IM DISTORTION AS A FUNCTION OF OUTPUT LEVEL & AMOUNT OF LIMITING, 60 CYCLES 7KC 4:1", dated 3/59, which
   reads about 0.4 % at 10 dB of limiting and turns sharply upward beyond about 15 dB [10]. **The
   monotonicity is the assertion that matters** and it is the one that fails if somebody bolts a separate
   saturator on (8.2, item 1).

10. **IM rises with output level at fixed gain reduction.** At 5 dB of gain reduction, measure IM at output
    levels of 0, +8, +16 and +24 dBm. The sequence must be **strictly increasing**, and the +24 dBm figure
    must be at least **10 times** the 0 dBm figure.
    *Figures:* the seven curves of the same chart are ordered by output level throughout, and at zero
    limiting they span roughly 0.2 % (0 dBm) to 3.8 % (+24 dBm), a factor of about 19 [10]. The test asserts
    the ordering, which the chart states unambiguously, and a factor of 10 against a chart-read 19, which
    leaves room for my eye.

11. **There is no way to get clean deep compression.** Over the whole `fc_threshold` × `fc_dc_threshold` ×
    `fc_zero` parameter space, sampled on a grid, **no setting** may produce more than 15 dB of gain
    reduction at +12 dBm out with IM below the 10 dB-of-limiting figure.
    *Figure:* this is the identity of 4.6 turned into a test, anchored on the same chart [10], every one of
    whose seven curves rises monotonically with limiting. **No published figure sets the bound**, so the
    test asserts the *shape* — that distortion is a monotone function of gain reduction at fixed output —
    rather than a number, and it says so.

### 11.3 Dynamics

12. **The six release times, emergent from the network.** For each `fc_time` position: apply a 1 kHz tone
    stepped up to produce exactly 10 dB of gain reduction, hold for 2 seconds so the network settles into
    its short-peak state, remove the excess, and measure the time for gain reduction to fall to **0.75 dB**
    (which is what 5.4 establishes Fairchild's phrase means):

    | `fc_time` | required release | tolerance |
    |---|---|---|
    | 1 | **0.3 s** | ±30 % |
    | 2 | **0.8 s** | ±30 % |
    | 3 | **2 s** | ±30 % |
    | 4 | **5 s** | ±30 % |
    | 5 | **2 s** | ±35 % |
    | 6 | **0.3 s** | ±40 % |

    *Figures:* the six release times on the specification page, "RELEASE TIME (from 10 db of limiting)" [1].
    *Why the tolerances differ:* positions 1 to 4 are single-pole and my derivation reproduces them to ±13 %
    (5.4); positions 5 and 6 are multi-pole and the derivation reproduces position 6's fast case to about
    30 % (5.5). **The model must not be given these numbers.** It is given the component values from the
    factory drawing [5] and the network of 10.3 block I, and these six times must fall out. If they are
    hard-coded the test is worthless and this is the test the whole design exists to pass.

13. **Position 6 is fast and slow.** `fc_time` = 6. (a) A single 50 ms burst producing 10 dB of gain
    reduction must release to 0.75 dB in **0.3 s ± 40 %**. (b) Thirty seconds of continuous
    material holding 10 dB of gain reduction, then silence: release to 0.75 dB must take **at least
    8 seconds**, and the model must still be more than 1 dB down after **6 seconds**.

    > **The 50 ms stimulus in (a) cannot produce the figure it checks.** Fifty milliseconds is already long
    > against the 0.8 s charging constant of the first slow leg, so the network is well into its
    > multiple-peaks state by then and reads about 1.6 s, where the figure under test is 0.3 s. An
    > individual peak in programme material is a few milliseconds. The build uses **2 ms** and gets
    > 0.32 s, and records the 50 ms reading in the test's own comment. The same trap sits in test
    > 12's proposed two-second hold: harmless at positions 1 to 4, where nothing depends on history
    > and that is the point of those positions, but the wrong stimulus at 5 and 6. The build uses one
    > second there.

    *Figures:* "Position 6: Automatic function of program material: .3 seconds for individual peaks, 10
    seconds for multiple peaks, 25 seconds for consistently high program level" [1]. The (b) bound is stated
    as "at least 8 s" against a published 10 s because my derivation lands 40 % low on the sustained cases
    (5.5) and I would rather assert a floor I can defend than a window I cannot. **The test names the 10 s
    and 25 s figures and says which part of them it is and is not checking.**

14. **The two program-dependent positions are the only ones whose release depends on history.** For each of
    positions 1 to 4, the release measured after a 50 ms burst and after 30 s of sustained limiting must
    agree to within **15 %**. For positions 5 and 6 they must differ by a factor of at least **4**.
    *Figure:* the specification lists positions 1–4 as single numbers and positions 5 and 6 as "automatic
    function of program material" [1]. The test asserts that qualitative distinction, which is what
    Fairchild published, rather than inventing numbers for it.

15. **The six attack times, and position 4 is the slow one.** For each `fc_time`, step the input to produce
    10 dB of steady gain reduction and measure the time to reach 9 dB:

    | `fc_time` | required attack | tolerance |
    |---|---|---|
    | 1 | 0.2 ms | ±40 % |
    | 2 | 0.2 ms | ±40 % |
    | 3 | 0.4 ms | ±40 % |
    | 4 | **0.8 ms** | ±40 % |
    | 5 | 0.4 ms | ±40 % |
    | 6 | 0.2 ms | ±40 % |

    *Figures:* Sound On Sound's published attack table [26], which gives exactly these six values. **The
    manual gives 0.4 ms for position 4** [1] and the test deliberately asserts Sound On Sound's 0.8 ms
    instead, because the circuit says attack is proportional to the timing capacitance and position 4 has
    twice position 3's (5.6). The test must carry that comment. Additionally: assert that the measured
    attack times are proportional to `C_T` across all six positions with a fitted constant of
    **0.10 ms/µF ± 20 %**, which is the derived relationship itself.

16. **Attack does not depend on the slow legs' charge.** Position 6, attack measured with the slow
    capacitors empty and again after 30 s of limiting: the two must agree to within **20 %**.
    *Figure:* the specification gives one attack figure for position 6 and three release figures [1]. The
    asymmetry is the assertion: the slow capacitors are on the discharge path, not the charge path, because
    the charge path is current-limited (5.6). This is a circuit identity and the test says so.

### 11.4 Stereo, the matrix, and hygiene

17. **Lateral-vertical is exact when both channels match.** `fc_agc` = `lat_vert`, both channels identically
    configured, a signal in the left input only: the right output must be at least **60 dB** below the left.
    *Figure:* "As long as the amount of lateral and vertical component reduction in each channel is
    identical, no deterioration of separation will occur" [1], and the specification's left-right separation
    of **60 dB** [1] as the bound. The matrix is mathematically exact, so what this really tests is that the
    two channels' gain reduction is identical when their settings are, which is a real bug class.

18. **Lateral-vertical is not stereo linking.** `fc_agc` = `lat_vert`, a centred mono signal loud enough for
    10 dB of gain reduction: **the vertical channel's gain reduction must be below 0.5 dB.** Then a
    hard-left signal at the same level: **both channels' gain reduction must agree to within 1 dB.**
    *Figure:* none published — **and I say so.** What is asserted is the arithmetic of a sum-and-difference
    matrix, which Fairchild describe in prose: "each stereo channel is divided into its respective lateral
    and vertical components. The upper channel now acts as a limiter for the lateral component, the lower
    channel as a limiter for the vertical component" [1]. A model that implements the mode as a linked pair
    fails the first half.

19. **Frequency response.** `fc_threshold` = 0, +4 dBm in: response from **40 Hz to 15 kHz within ±1.0 dB**.
    *Figure:* "FREQUENCY RESPONSE: 40 cycles to 15 kc ± 1 db". *Source:* the specification page [1]. Note
    that this is a band with a tolerance and **no shape**, so the test may not assert anything about the
    slope outside the band or the ripple inside it beyond the ±1 dB.

20. **The low corner moves with gain reduction.** Measure the −1 dB low-frequency point at 0 dB and at 10 dB
    of gain reduction. Assert only that both are **below 40 Hz** and that the test records the difference.
    *Figure:* the 40 Hz figure above [1] is specified without a gain-reduction condition, so it must hold in
    both. **No published figure describes the movement**, which follows from the cathode bridge working into
    a bias-dependent impedance (10.4), so the test asserts the specification twice and *logs* the delta
    rather than bounding it. This is deliberately a weak test and it is written that way because the
    alternative is an invented bound.

21. **Noise.** Output noise with no input must be at least **70 dB below +4 dBm**.
    *Figure:* "NOISE LEVEL: 70 db below +4 dbm" [1]. Note that Fairchild give no weighting and no bandwidth,
    so the test must state the bandwidth it uses (20 Hz–20 kHz, unweighted) and acknowledge that it is not
    necessarily Fairchild's.

22. **No thump.** With a step change in gain reduction from 0 to 12 dB, the output must contain no transient
    below 20 Hz more than **20 dB above** the noise floor.
    *Figure:* "characterized by the complete absence of audible thumps" and "the Automatic Gain-Controlled
    Amplifier never produces any audible or observable thumps" [1]. Fairchild give no number, so the bound
    is chosen and the test says it is chosen; what is being verified is the *mechanism* — that the
    common-mode control voltage cancels in the push-pull difference (4.5) — and if the model needs a control
    smoother to pass this, the topology is wrong and the test has done its job.

23. **Rate independence.** Every dynamic figure above, measured at 44.1, 48 and 96 kHz, must agree to within
    **5 %**.
    *Figure:* none; this is a correctness property of the implementation and the test says so.

24. **Switching the time constant does not discharge the network.** Hold 10 dB of gain reduction in position
    6 for 30 s, switch to position 3, then remove the input: the release must be **slower** than a release
    from position 3 reached without that history. *Figure:* none published — asserted as a circuit identity
    (10.3, block I): the capacitors keep their charge when the switch moves. This is the test that catches
    a model that resets state on a parameter change.

### 11.5 What I will not test, and why

- **Gain.** The specification's 7 dB [1] contradicts the manual's own operating instruction and the
  published curves (7.1). Testing it would be testing my guess about which is right.
- **The 30:1 ratio** on the features page [1], because the specification page in the same manual says 20:1
  and the published curves support the lower figure (7.1).
- **The 100 µs attack** claimed on the features page [1], because the specification and the circuit both say
  200 µs (5.6).
- **Anything against a plug-in.** No vendor publishes a measurement of their model (7.3), so a comparison
  would be against an unmeasured artefact.
- **The 660's release times, ratios or distortion.** The manual I hold is the 670's. The timing network is
  identical (1.3) so positions 1–6 should behave the same, but the cathode resistors and hence the operating
  point differ and **I have no 660 specification sheet**. If the plug-in ships a 660, its dynamics tests
  should assert the 670's timing figures (which the shared network justifies) and assert **nothing** about
  its static curve or distortion until a 660 specification turns up.
---

## 12. Reuse: does a variable-mu element belong in the components crate?

The components crate holds a photocell and a diode bridge, and its README states the rule plainly: *"A
component earns a place **once something real shares it, or is about to**. I am not trying to atomise a
codebase into parts that each have one caller. An abstraction pulled out of a single user is usually the
wrong shape for the second one, and I would rather discover the right shape from two real users than guess
it from one."* It also already names this case: *"**Variable-mu element.** The gain element of a whole
family the plug-ins do not cover yet, so it will arrive with the first unit that needs one."* [51]

So the question is not whether it is wanted but whether it qualifies on the second half of the rule, the way
the diode bridge did.

### 12.1 Specify it, but do not create it yet

**Read 12.3a before this section.** This section was written first and argued that a variable-mu component
should be admitted now, on the same "about to be shared" footing the diode bridge was admitted on. **I no
longer think that, for two reasons that arrived afterwards and are both recorded below.** The diode bridge's
own footing collapsed when its predicted second user, the EMI TG12413, turned out not to contain a bridge at
all, and the repository's rule has since been tightened to require two units *documented* to contain the
part rather than one that does and one that is expected to. And 12.3a measures the two candidate tubes'
control laws and finds their exponents genuinely different, 1.0 against 2.16, which means a second user
cannot inherit the first one's curve and must be fitted independently before anyone knows what the shared
part's parameterisation has to cover. The conclusion I would now defend is the one in 12.3a: **specify the
component, write the plug-in's tube so it is separable, and lift it out when a second real fit exists.**
The reasoning below about *what* would go in it and what would not still stands, and is why the
specification is worth having now.

**What the component would be: a remote-cutoff triode.** Not "the Fairchild's gain stage". The part is one
triode section whose anode current is a function of grid and anode voltage, parameterised by a fitted law.
Its interface is:

```
Ia = tube.anode_current(vgk, vak)
gm = tube.transconductance(vgk, vak)         // for metering and for the gain-range test
```

and its state is nothing at all. It is a pure function with parameters, like the diode bridge's `tanh` law
with its `k`.

**Two real users, one built and one planned.** The Fairchild is the first. The **Universal Audio 176** is
already in the survey's build order at position five, explicitly *"only after the Fairchild"*, and the
survey's reasoning for that placement is precisely this shared part: *"Small if the Fairchild is built
first, because the variable-mu stage, the oversampling and the feedback loop already exist by then. Built
first, it would cost nearly as much as the Fairchild for a less famous box."* [52] The 176 uses a **6BC8**
rather than a 6386 [52], which is a different remote-cutoff twin triode with different parameters — and
that is the *good* case for a component, because it means the second user exercises the parameterisation
rather than reusing the first user's constants.

**That is the diode bridge's situation exactly**, and the README is candid that the bridge was *"the one
case so far admitted on the second half of the rule, *about to* be shared rather than already shared: it has
one user today, and a second, the EMI TG12413, is next but one in the plug-in's build order. That is a
weaker justification than the photocell's and is recorded as such"* [51]. **A variable-mu component would be
the second such case and should be recorded the same way, in the same words.** I would not pretend it is
stronger than the bridge's, and I would not pretend it is weaker.

**There is also a third potential user that strengthens it slightly and is worth naming.** The survey
assessed the **Altec 436C** — the Motown compressor — as a variable-mu unit with a schematic and nothing to
check it against, and concluded it is *"worth revisiting only after a variable-mu engine exists"* [52]. That
is not a commitment, but it means the family has depth beyond two.

### 12.2 What goes in, and what emphatically does not

The crate's own boundary rule is *"Circuitry does not belong here. A component is the part. The resistive
divider it shunts, the sidechain that drives it and the make-up gain after it are the machine"* [51].
Applied here:

**In the component:**

- The anode-current law and its parameters. For the 6386, Raffensperger's eight [18], with a named
  constructor `RemoteCutoffTriode::ge_6386()` and a second `::jj_6386_lgp()` for the modern replacement,
  whose published transconductance differs by 2.5 dB [15].
- Transconductance, as the analytic or numerical derivative, because it is what the gain-range test needs
  (11.1, test 6) and because a metering block wants it.
- The `Vgk` clamp, because the divergence at +5 V is a property of *this fitted law* and not of the
  Fairchild (4.3).
- Interelectrode capacitances as data, because they are the tube's [12].

**Not in the component:**

- **Four-in-parallel.** That is Narma's circuit choice (4.4), and the 176 does not do it. A `n_par` scale
  factor belongs to the caller.
- **Push-pull.** Also the machine. The 176 is a different topology.
- **The cathode resistor and its bypass capacitor.** 680 Ω on the 670, 1800 Ω on the 660 (1.3), something
  else on the 176. Circuitry.
- **The common-mode control injection and its 100 kΩ resistors.** Circuitry.
- **The time-constant network.** Emphatically circuitry, and Fairchild-specific: the six positions are a
  property of that switch on that unit.
- **The rectifier, the dead zone, the current limit.** All sidechain, all the machine.

### 12.3 What a second variable-mu unit would actually share, honestly

Listing this properly is the point of the exercise, because "they are both variable-mu" is not a plan.

| | shared with the 176? | why |
|---|---|---|
| remote-cutoff anode-current law | **yes, as a parameterised part** | different tube, same functional form |
| the fitted parameters | **no** | 6386 vs 6BC8; the 176 needs its own fit against its own datasheet |
| gain-and-distortion-are-one-curve identity | **yes, as an architectural property** | it is what the family means (4.6) |
| four sections per half | **no** | Fairchild's choice |
| push-pull output stage | **probably not** | the 176 is a smaller, single-ended-ish design [52] |
| oversampled feedback loop around a nonlinearity | **yes, as infrastructure** | but the README says infrastructure does not belong in the crate [51], so this lives in the lab |
| transformer shaping | **yes, eventually** | the README already lists "tube stage and transformer" as a candidate; the transformer half of that is still valid |
| the six time constants | **no** | Fairchild's switch |
| the matrix | **no** | the 176 is mono |
| the dead-zone knee | **no** | that is Fairchild's DC threshold |

**Numbers for the two tubes, side by side.** The 176 research supplied the 6BC8 figures from three
manufacturer datasheets (RCA 10-66, Tung-Sol 1961, Sylvania 1955), and they are worth putting next to the
6386's because at first glance they look like two different laws and on inspection they do not.

| | 6386 [12] | 6BC8 (per the 176 research) |
|---|---|---|
| datasheet wording | "**remote-cutoff** characteristic", for circuits "to which it is desired to apply automatic-gain-control" | "**semiremote-cutoff**", for VHF television tuner cascodes |
| operating point quoted | Eb 100 V, Rk 200 Ω, Ip 9.6 mA → Vg ≈ **−1.92 V** | Ep 150 V, Rk 220 Ω, Ip 10 mA → Vg ≈ **−2.20 V** |
| μ | 17 | 33 to 35 |
| rp | 4250 Ω | 5300 Ω |
| gm at that point | 4000 µmho | 6200 µmho |
| grid volts for a small gm | **−16 V for 100 µmho** | **−13 V for 50 µmho** |
| range, as published | **32.0 dB over 14.1 V** | 41.9 dB over 10.8 V |

**Those last two rows are not comparable as printed**, because GE quote their endpoint at 100 µmho and the
6BC8's makers quote theirs at 50, which is 6 dB further down a curve that is still falling. Normalising them
to a common endpoint is the right move. **I did it backwards the first time and this table is the
correction.** I placed the 6BC8's 100 µmho point at −14.6 V, *below* its 50 µmho point at −13 V; since
transconductance falls as the grid goes negative, 100 µmho must sit at a **less** negative voltage, near
−10.3 V. The 176 research caught the sign error and supplied the corrected rows:

| basis | 6386 | 6BC8 | ratio |
|---|---|---|---|
| as printed | 32.0 dB over 14.1 V = **2.28 dB/V** | 41.9 dB over 10.8 V = **3.88 dB/V** | 1.70 |
| both to a 100 µmho endpoint | 2.28 dB/V | 35.9 dB over 8.1 V = **4.43 dB/V** | **1.95** |
| both to a 50 µmho endpoint | 38.1 dB over 17.1 V = **2.23 dB/V** | 3.88 dB/V | 1.74 |

So the corrected taper ratio is **1.7 to 1.95**, not the 1.28 I first reported: normalising the endpoints
widens the gap rather than closing it. (The 6BC8 figures are quoted between its two tabulated points at
−2.2 V and −13 V, a span of 10.8 V, not from zero bias where the tabulated gm does not apply.)

**And a ratio of averaged tapers cannot settle the question anyway**, which is the deeper point and the one
12.3a acts on. An average slope over an interval is a property of the interval as much as of the tube, and
it is blind to *how* the slope varies inside it — which is exactly what distinguishes an exponential law
from a steeper one. Averaging destroys the evidence. The taper rows above are worth keeping only as a
statement that both tubes are the same order of steepness.

**The one thing that could still have sunk it, checked rather than left as a caution.** The 176 research
reported the 6BC8's μ as "comparatively flat" across its control range, which would have been a genuine
shape difference: if μ holds near 34 while gm falls 42 dB, plate resistance has to rise by that whole factor
and the two tubes load their following stages quite differently. **It is not flat.** Sylvania's *Engineering
Data Service* sheet for the 6BC8 and 4BC8, September 1955, plots μ and gm together against grid voltage on
one axis, and I read the μ family off it after calibrating the μ scale against the plot's own gm gridlines
[55]. The calibration checks out: at zero bias the curves land on μ ≈ 35, which is the value Sylvania
tabulate on page 1 of the same document.

| grid volts | μ, Eb = 100 V | μ, Eb = 150 V | μ, Eb = 200 V |
|---|---|---|---|
| 0 | ≈ 34 | ≈ 35 | ≈ 36 |
| −4 | ≈ 22 | ≈ 24 | ≈ 26 |
| −6 | ≈ 14 | ≈ 16 | ≈ 21 |
| −8 | ≈ 11 | ≈ 12 | ≈ 17 |
| −10 | ≈ 8 | ≈ 12 | ≈ 18 |

Read by eye against a calibrated overlay, so ±2 on each figure; the working image is saved as
`ref/fairchild-6bc8-mu-curve-calibrated.png`. **The 6BC8's μ falls by a factor of about four across its
control range**, which is more variation than the 6386 shows, not less.

Putting both tubes on the same footing, as decibels of μ change per volt of grid at Eb = 100 V
(**derived**, and the 6386 figure composes two of GE's plots so it carries two eyeball steps rather than
one):

| | μ at the operating point | μ near the bottom of the control range | dB of μ per volt of grid |
|---|---|---|---|
| 6386 [12] | ≈ 19 at −1.5 V | ≈ 7 at −7.5 V | **1.45** |
| 6BC8 [55] | ≈ 34 at 0 V | ≈ 8 at −10 V | **1.26** |

Within 15 per cent of each other. So a single tabulated μ at one operating point says nothing about bias
dependence, and both tubes collapse their amplification factor at a similar *average* rate.

### 12.3a The exponent, which is where the two tubes actually differ

**Everything above compares average tapers, and average taper is the wrong statistic.** The 176 research
fitted the 6BC8's transconductance to a stretched exponential,

```
gm(w) = gm0 · exp( −(w / V0)^n )          w = volts of grid below zero
```

and obtained **V0 = 4.16 V, n = 2.16**, arguing that a true remote-cutoff tube is designed to give an
exponential law, meaning n = 1, so that an exponent far from 1 would be a difference of *shape* rather than
of constants. I fitted the same form to the 6386 to settle it, and **the 176 research is right about the
difference.**

**Which figure this is, since it was challenged and the challenge was fair.** General Electric ET-T1113,
**page 3, upper figure**, headed "AVERAGE TRANSFER CHARACTERISTICS — CASCODE CONNECTION", with
`TRANSCONDUCTANCE IN MICROMHOS` on a **logarithmic** vertical axis from 10 to 10 000 and
`GRID-NUMBER 1 VOLTAGE IN VOLTS` on a linear horizontal axis from 0 to −60 [12]. Four curves, labelled
`Ecc2 = 300 VOLTS`, `250`, `200` and `150`, with an inset of the measuring circuit. **`Ecc2` is not a screen
voltage** — the 6386 is a twin triode and has no screen. Page 2 names it: in the `CASCODE AMPLIFIER` block it
is the **"Voltage-Divider Supply Voltage"**, the rail feeding the 470 kΩ / 470 kΩ divider that biases the
upper, grounded-grid triode, and both the page 3 inset and the page 6 circuit diagram draw it that way [12].

**And the closure check that this makes possible, which resolves an apparent contradiction.** GE's tabulated
`Gm = 100 Micromhos at −16 Volts` sits in the **`CLASS A₁ AMPLIFIER, EACH SECTION`** block at **plate voltage
100 V**; the logarithmic curve is the **cascode** at Eb = 250 V. Different circuits, so the two need not
agree — but they must be consistent in direction and magnitude, and they are:

| curve | reaches 100 µmho at |
|---|---|
| Ecc2 = 300 V | −27.5 V |
| Ecc2 = 250 V | −23.6 V |
| Ecc2 = 200 V | −18.9 V |
| **Ecc2 = 150 V** | **−14.6 V** |
| GE's tabulated single section, plate 100 V | **−16 V** |

The lowest divider supply puts the lower triode at the lowest plate voltage of the four, so it should cut off
earliest, and it does — landing 1.4 V from the tabulated figure for a section at 100 V plate. **A reading of
this curve that had the wrong axis origin, or that was tracing another tube, could not land there.**

**The exponent, from all four curves.** Traced by the same method and fitted over the same range:

| curve | n | V0 | rms | decade spacing (V) |
|---|---|---|---|---|
| Ecc2 = 300 V | **1.00** | 7.0 | 0.37 dB | 16.0, 16.3, 16.7, 15.6 |
| Ecc2 = 250 V | **0.84** | 4.4 | 0.33 dB | 13.9, 15.1, 16.1, 15.7 |
| Ecc2 = 200 V | **0.71** | 2.6 | 0.31 dB | 11.5, 12.9, 14.8, 15.5 |
| Ecc2 = 150 V | **0.59** | 1.4 | 0.44 dB | 9.1, 11.1, 13.6, 15.1 |

Forcing the 6BC8's 2.16 on the same points costs 2.95, 3.59, 4.43 and 5.30 dB respectively — seven to twelve
times the residual of the free fit.

**And that table is the real finding, which is not the one I set out to report.** The exponent is **not a
property of the tube**. It runs from 0.59 to 1.00 across the four operating conditions GE plot for *one*
tube, monotonically with the divider supply. The 176 research reports the matching instability from their
side: their own 6BC8 figure moves from 2.16 to 1.71 depending on whether it is anchored on interior or
endpoint points, and two published transconductance points cannot fix a three-parameter form at all.

**So the honest conclusion is that the published data cannot settle the shape comparison**, and I withdraw
the clean n = 1.01 I reported earlier. What does survive, and is worth keeping:

- **In every condition GE plot, the 6386's transconductance is at or below a pure exponential** — n from
  0.59 to 1.00, never above 1 — and an exponent of 2.16 is three to five decibels worse than the free fit on
  every one of the four curves.
- **My earlier reasoning was wrong regardless of the number.** Comparing average tapers cannot answer a
  question about curvature, and the taper ratio I first published had a sign error in it besides (12.3).
- **A quantity that moves by a factor of 1.7 across one tube's own operating conditions, and by a factor of
  1.3 across one fitter's choice of anchors, is not a quantity on which two datasheets can be compared.**
  That is the finding, and it is more useful than either "the tubes match" or "the tubes differ".

**One limitation of my curve that I should name rather than let a reader find.** It is a *cascode* curve. In
a cascode the lower triode's plate is not held at the tube's normal operating voltage; it floats at the upper
triode's cathode. So this is the gm law of a 6386 section under cascode loading, not of a grounded-cathode
stage at the 230 V plate the Fairchild actually runs (3.2). GE tabulate cascode transconductance and
Class A₁ transconductance as the same 4000 µmho, which is a point of contact, but one point is not a shape.
**Carrying the exponent over to the Fairchild's own topology is an assumption, and I have not tested it.**

**What that does and does not kill.** It kills the idea that this family has a universal shape, and with it
any plan to fit one tube and assume the other's curve. It does *not* by itself kill a shared component,
because **n is a parameter of the form**, and a part holding `(gm0, V0, n)` per tube type covers both a
6386 at n = 1.0 and a 6BC8 at n = 2.16. What the finding changes is the discipline: every tube type needs
its own fit against its own published curve, and the physical argument "remote cutoff implies exponential"
holds for the 6386 and demonstrably does not hold for the semiremote-cutoff 6BC8. That is a stronger reason
to defer the component than the one in 12.1, not a weaker one.

**And there is still a dividend for this plug-in, weaker than I first claimed.** An exponent at or below 1
means the 6386's transconductance is close to a pure exponential in grid voltage over its working range, so **gain in
decibels is close to linear in control voltage** — about 1.2 dB per volt (**derived**, and see the spread
above). Section 5.4 assumed exactly that when it turned the release network's RC products into the manual's
published release times, and the assumption now has a measurement behind it rather than only an argument
from the phrase "remote cutoff".

**So the honest summary is: one part, one architectural lesson, and one piece of infrastructure.** The part
belongs in the crate. The architectural lesson belongs in the lab's documentation and in the shape of
`dsp::vmu`. The infrastructure belongs where the DSP lives.

### 12.4 One correction to the components README

The README's "coming candidates" list says: *"**Tube stage and transformer.** Both already exist in the 610
preamp of the compressor lab's 6176, and both would be wanted by any variable-mu unit, which is the next
family to model."* [51]

**Half of that is wrong and it is worth fixing before somebody builds on it.** The transformer half is
right. The tube half is not: the 610's triode model was fitted for 12AX7-class valves, and Raffensperger is
explicit that *"Existing triode models were designed for tubes like the 12AX7 which do not have the remote
cutoff characteristic of the 6386"* [18]. The functional form differs, not just the parameters (4.1). A
variable-mu unit does **not** want the 610's tube stage; it wants a different tube stage that happens to
also be a triode. If the crate ever holds both, they should be two components with two names, and the README
should say why.

### 12.5 Effort, honestly

The survey called this "the highest of the top four" and it was right, but the shape of the cost has changed
now that the documents are read.

**Cheaper than the survey expected:** the timing network is fourteen component values off a factory drawing
and three state variables (5.3, 10.3); the sidechain is five published equations rather than five tube
models (5.7); the twenty valves are eight evaluations of one function plus a regulated supply that needs no
model at all (3.4, 4.4); and the ground truth turned out to be two manufacturer measurement charts rather
than nothing (7.2, 4.6).

**More expensive than the survey expected:** the tube model needs a scalar fixed-point solve per half per
sample inside an 8× oversampled loop, which is the real cost and is unlike anything in the lab; the feedback
loop closes around that solve; and the matrix means the stereo path is not two independent mono paths.

**Net:** still the most expensive unit in the lab, still worth it, and now with better anchors than the
Neve had.

---

## 13. References

Everything below was fetched and read while writing this file, except where the entry says otherwise. The
Fairchild manual is cited **by scan**, because three separate scans of the same December 1959 document are
in the archive and they OCR differently in the three places where it mattered (1.4); where I say "the
manual" without qualification I mean [1] and I have checked the passage against [2] and [3].

Entries 52 to 54 are files in this repository and its sibling rather than public URLs, so they carry a
path instead of a link. Entry 55 arrived after the rest and is numbered last rather than renumbering the
others; it is the 6BC8 datasheet that settled the shared-component question in 12.3.

Manufacturer and vendor documents are cited as manufacturer claims. Where two Fairchild documents disagree
— and they do, about the maximum ratio, the fastest attack, the position-4 attack time and the remote-meter
voltage (5.6, 7.1, 2.6) — both are cited and the disagreement is stated rather than resolved.

**Primary Fairchild documents**

1. Fairchild Recording Equipment Corporation, *Instruction Manual, Model 670 Stereo Limiter*, December 1959,
   "Supersedes all previous issues", 10-05 5th Avenue, Long Island City 1, New York. 16 pages: outstanding
   features, general description, limiting in stereo disk recording, the specification sheet, the control
   list, input and output connections, balance and zero adjustments, threshold adjustments, normal
   operation, both factory charts and a redrawn schematic. Document number DS-670-659 appears on the
   features page. https://archive.org/details/Fairchild_670_owners_manual
2. The same manual, second scan, at higher resolution on the chart pages. This is the scan I read [9] and
   [10] from. https://archive.org/details/JL10882
3. The same manual, third scan. Used only to cross-read the OCR of the panel size and the tube complement
   (1.4). https://archive.org/details/JL10878
4. Fairchild 670 stereo limiting amplifier schematic, a **modern redraw**, 4800 × 4056. Complete: both
   channels, the matrix switch S301 A–K, the sidechain, the time-constant switch S102 and the power supply,
   with component values and DC annotations. Carries `C115 ??? 200v` and `C215 ??? 200v` where the redrawer
   could not read the original, and repeats the designators R313 and R302 on two different parts.
   https://archive.org/details/JL10883
5. Fairchild Recording Equipment Corporation, **"MODEL 660 AUTOMATIC GAIN CONTROL AMPLIFIER"**, the original
   factory schematic, 3071 × 2550 bitonal. **The switch positions of S2 are numbered on the sheet**, which is
   what makes section 5.3 possible, and the notes give the resistor and capacitor conventions and the
   symbol key for knob, front-panel screwdriver and rear-of-chassis screwdriver adjustments.
   https://archive.org/details/JL10866
6. "Fairchild 660 Drawn", a fragmentary hand sketch of the control circuit. Little usable content.
   https://archive.org/details/JL10865
7. Fairchild 663 compressor schematic, drawing A-9608, issue 1 dated 22 September 1969. A **different,
   transistor-based unit** ("RCA 34966 OR 2N1183"); cited only to warn against it (1.2).
   https://archive.org/details/JL10873
8. Fairchild Recording Equipment Corporation, "670 LIMITER COMPONENT LAYOUT (REAR VIEW)", 9921 × 7016
   mechanical drawing. Title block gives the address as **154 St. & 7 Ave., Whitestone, L.I., N.Y.**, which
   is not the address on the manual or the panel (1.4). https://archive.org/details/JL10876
9. **"INPUT VS. OUTPUT CURVES"**, dBm out against dBm in, five curves with their control settings named,
   dated **December 1959**, annotated "(supersedes March 1959 issue)". Page 13 of [2]. Read in section 7.2;
   working image saved as `ref/fairchild-670-input-output-curves-dec1959.png`.
10. **"IM DISTORTION AS A FUNCTION OF OUTPUT LEVEL & AMOUNT OF LIMITING, 60 CYCLES 7KC 4:1"**, seven curves
    at 0, +4, +8, +12, +16, +20 and +24 dBm out, IM per cent against decibels of limiting, dated **3/59**.
    Page 8 of [2]. Read in section 4.6; working image saved as
    `ref/fairchild-670-im-distortion-mar1959.png`.
11. The John Leimseider archive on the Internet Archive, the collection holding items [2] to [8].
    https://archive.org/details/john-leimseider-archive

**Component data**

12. General Electric, Electronic Components Division, **6386 Twin Triode, Five-Star Tube**, datasheet
    **ET-T1113**, dated 8-54, characteristic curves dated 21 August 1953. Six pages: description and rating,
    maximum ratings, Class A₁ and cascode characteristics, four pages of curves and a typical cascode
    circuit. The source of every tube figure in section 4.2.
    https://frank.pocnet.net/sheets/142/6/6386.pdf
13. Radiomuseum, 6386 tube entry: "Double Triode Controlling (mu)", first source 2 October 1953 Electron
    Tube Registration List, "computer rated, remote cutoff medium mu twin triode, derived from 2C51, Mu 17".
    https://www.radiomuseum.org/tubes/tube_6386.html
14. Radiomuseum, Fairchild Stereo Limiter 670 model record, ID 120760, year 1959. Lists the valve
    complement, names **1N538** among the semiconductors, and notes that the redraw's "6Z34" and the
    original's "6234" should both be GZ34.
    https://www.radiomuseum.org/r/fairchild_stereo_limiter_670.html
15. JJ Electronic, 6386 LGP. Published typical characteristics Ua = 100 V, Rk = 200 Ω, Ia = 9.6 mA,
    S = 3 mA/V, Ri = 6 kΩ, μ = 18 — a lower transconductance and higher mu than GE's original at the same
    operating point. https://www.jjtubes.eu/jj-6386-lgp-new
16. 6973 beam power pentode datasheet, the 670's sidechain output valve.
    https://frank.pocnet.net/sheets/049/6/6973.pdf
17. 6BL7GTA datasheet, the 660's series regulator. https://frank.pocnet.net/sheets/127/6/6BL7GTA.pdf

**Modelling literature**

18. Peter Raffensperger, **"Toward a Wave Digital Filter Model of the Fairchild 670 Limiter"**, *Proc.
    DAFx-12*, York, 17–21 September 2012. The only published circuit model of this unit. Source of the
    fitted 6386 law (Table 1), the transformer parameters (Table 2), **the time-constant component table
    (Table 3)** and the five sidechain equations (10–14). Validated against SPICE, not hardware, and the
    author says so. **The copy DAFx serves is six pages and its text stops mid-sentence in section 6**
    (1.0). https://www.dafx.de/paper-archive/2012/papers/dafx12_submission_9.pdf

**History and background**

19. Wikipedia, "Fairchild 660". `Fairchild 670` is a redirect to this article. Source of the Narma
    provenance, the first-ten-units story, the 20 valves / 11 transformers / 2 inductors count and the
    Emerick quotation via Lewisohn. Its price sentence carries a citation-needed tag dated July 2025 and its
    "12 purchased / 8 remaining" figures are **not supported by the source it cites** (1.5).
    https://en.wikipedia.org/wiki/Fairchild_660
20. Wikipedia, "Fairchild Recording Equipment Corporation". Founded 1931 in Whitestone, Queens.
    https://en.wikipedia.org/wiki/Fairchild_Recording_Equipment_Corporation
21. Wikipedia, "Sherman Fairchild". 7 April 1896 – 28 March 1971.
    https://en.wikipedia.org/wiki/Sherman_Fairchild
22. *Mix*, "1959 Rein Narma Fairchild 670 Compressor/Limiter", TECnology Hall of Fame, Mix staff, published
    1 September 2007. The fullest reachable account of Narma's career; says he became "a vice-president at
    Ampex". Its link to Fairchild's own datasheet PDF is dead with no Wayback copy.
    https://www.mixonline.com/technology/1959-rein-narma-fairchild-670-compressor-limiter-377967
23. MusicTech, "Studio Icons: Fairchild 660/670", John Pickford, 28 May 2014. The Peter Bown / Capitol
    Records story and the 1966 Emerick drums claim. Does **not** contain the unit counts Wikipedia
    attributes to it. https://musictech.com/reviews/studio-icons-fairchild-660670/
24. MusicTech, "Vintage Rewind: Fairchild 660 and 670", 22 January 2019.
    https://musictech.com/reviews/vintage-rewind-fairchild-660-and-670/
25. Vintage King, "Fairchild 660/670 Compressor Limiter". Production estimates and the Revolver-onward vocal
    claim. https://vintageking.com/fairchild-660-670-compressor-limiter
26. **Sound On Sound, "Fairchild 660 & 670", Hannes Bieger, May 2016.** The most technically substantial
    reachable secondary source: four 6386 per channel with four triode elements paralleled per push-pull
    half, the sidechain tapped after the gain cell, the soft knee and progressive ratio, the internal trim
    pot, the 30 kg / 6U figures (the 6U is wrong, 1.4) and **the six-position attack and release table whose
    position-4 attack the manual contradicts** (5.6).
    https://www.soundonsound.com/reviews/fairchild-660-670
27. Radiomuseum manufacturer record, Fairchild Recording Equipment Corp., "154 St. and 7th Ave., Whitestone,
    New York (~1950's)", founded 1931. The only secondary source I reached with a street address.
    https://www.radiomuseum.org/dsp_hersteller_detail.cfm?company_id=7890
28. Jay McKnight and Vaino Narma, "Obituaries: Rein Narma", *Journal of the Audio Engineering Society*,
    vol. 67 no. 11, November 2019, p. 931. **NOT READ** — 404 on aes.org and the Wayback copy returned
    HTTP 429 on every attempt. Citation given so somebody with access can check it.
    https://www.aes.org/aeshc/jaes.obit/JAES_V67_11_PG931.pdf
29. AES Oral History 048: Rein Narma. Exists as video only; **not transcribed**.
    https://www.youtube.com/watch?v=MzdnXJCtnYw
30. Abbey Road Studios, exhibition placard "Fairchild 660 limiter (1960s)", photographed 25 November 2009,
    legible in [33]. The only first-party Abbey Road statement about the Fairchild I could obtain; their
    current website contains none (1.0).

**Photographs, all CC BY 2.0 on Wikimedia Commons**

31. "Fairchild Model 670 Compressor", mac morrison, 3 February 2011, 4000 × 2248. Source of the silkscreen
    reading and a cross-check on the layout. Saved as `ref/fairchild-670-front-4000px.jpg`.
    https://commons.wikimedia.org/wiki/File:Fairchild_Model_670_Compressor.jpg
32. "Fairchild 670 Compressor at Audio Mix House", Audio Mix House, 28 July 2014, 2048 × 1536. **The
    photograph section 2.2's geometry and 2.4's colours are measured from**, after projective rectification.
    Saved as `ref/fairchild-670-rack-audiomixhouse.jpg`, with the rectified version as
    `ref/fairchild-670-panel-rectified.png` and an inch grid as `ref/fairchild-670-panel-inch-grid.png`.
    https://commons.wikimedia.org/wiki/File:Fairchild_670_Compressor_at_Audio_Mix_House_(2014-07-28_by_Audio_Mix_House).jpg
33. "Fairchild 660 limiter (1960s), EMI Presence Box (1960s), Altec RS124 compressor (1960s), Abbey Road
    Studios", Josephenus P. Riley, 25 November 2009, 2048 × 1536. Shows a 660 with the Abbey Road placard of
    [30]. Saved as `ref/fairchild-660-abbeyroad.jpg`.
    https://commons.wikimedia.org/wiki/File:Fairchild_660_limiter_(1960s),_EMI_Presence_Box_(1960s),_Altec_RS124_compressor_(1960s),_Abbey_Road_Studios.jpg

**Emulations and recreations, for benchmarking**

34. Universal Audio, Fairchild Tube Limiter Collection.
    https://www.uaudio.com/products/fairchild-tube-limiter-collection
35. Universal Audio press release, 18 November 2013, announcing the collection at $299.
    https://www.uaudio.com/blogs/press/fairchild-collection
36. Universal Audio press release, 15 January 2004, announcing the Fairchild 670 Legacy plug-in at $149 and
    describing exactly which hardware controls were changed and why.
    https://www.uaudio.com/blogs/press/0115_fairchild
37. Waves, PuigChild 660 and 670. Contains an attack figure in milliseconds that should be microseconds
    (9.2). https://www.waves.com/plugins/puigchild-compressor
38. IK Multimedia, "Vintage Tube Compressor/Limiter Model 670 — Vari-Mu Compressor/Limiter". Carries the
    trademark notice quoted at the head of this file.
    https://www.ikmultimedia.com/products/trvintubcomplim/
39. Softube, Bus Processor 670. Source of the "fewer than a thousand" production figure.
    https://www.softube.com/plug-ins/bus-processor-670
40. Overloud, Gem COMP670. **The best vendor description of the DC threshold trimmer anywhere** (5.2, 9.5).
    https://www.overloud.com/products/comp670-compressor-limiter
41. Slate Digital, Virtual Buss Compressors, containing FG-MU. Names the Fairchild as an inspiration and
    does **not** claim to model it. https://slatedigital.com/plugin/virtual-buss-compressors/
42. Heritage Audio, HERCHILD Model 670. https://heritageaudio.com/herchild-670/
43. Heritage Audio, GRANDCHILD 670/500, which uses 6BA6 pentodes rather than 6386 triodes.
    https://heritageaudio.com/grandchild-670/
44. Heritage Audio, HERCHILD Next Gen Model 670N. https://heritageaudio.com/herchild-next-gen-model-670n/
45. POM Audio Design, FAIRCHILD 670 mkII, with sub-pages on the feed-forward/feedback blend, the filter
    section and the metering. **The best explanation of what the meter and the ZERO control actually do**
    (2.5, 3.5). https://www.fairchild-recording-equipment.com/
46. POM Audio Design, 2026 price list, £6,990 to £8,490.
    https://www.fairchild-recording-equipment.com/PriceList2026/
47. Plugin Alliance, NEOLD V76U73, describing the Telefunken U73 as "The German Fairchild" and a variable-mu
    design. Cited only as evidence of what the family name is taken to mean.
    https://www.plugin-alliance.com/en/products/v76u73.html
48. IK Multimedia, Dyna-Mu. A variable-mu emulation of an unnamed American unit; **not a Fairchild**.
    https://www.ikmultimedia.com/products/trdynamu/
49. Arturia, Comp TUBE-STA. A Gates STA-Level, not a Fairchild; cited to keep the two apart.
    https://www.arturia.com/products/software-effects/comp-tubesta/overview
50. Gyraf Audio, G22 dual vari-mu compressor. Gyraf's own G10 lineage, **not** a Fairchild derivative, and
    Gyraf publish no variable-mu DIY project.
    https://www.gyraf.dk/g-22-dual-stereo-ms-vari-mu-compressor/

**This repository**

51. `noob-electrical-components`, README. The component-boundary rule, the "coming candidates" list that
    already names the variable-mu element, and the diode bridge's recorded weaker justification.
    https://github.com/Noob-Audio-Engineering/noob-electrical-components
55. Sylvania Electric Products Inc., Radio Tube Division, **Engineering Data Service, types 6BC8 and
    4BC8**, September 1955, five pages, prepared and released by the Technical Publications Section,
    Emporium, Pennsylvania. Page 1 gives the Class A characteristics (plate 150 V, cathode-bias resistor
    220 Ω, plate current 10 mA, transconductance 6200 µmho, amplification factor 35, grid voltage for
    gm = 50 µmho approximately −13 V) and describes the tube as "a miniature, medium mu, **semi-remote
    cutoff** twin triode intended for application as a v-h-f cascode amplifier in television receivers".
    **Page 4 plots μ and gm together against grid voltage**, which is the plot section 12.3 reads. Four
    copies of this sheet are on `frank.pocnet.net` under volumes 049, 106, 127 and 137; volume 137 is the
    highest resolution. Saved as `ref/fairchild-6bc8-sylvania-1955.pdf`.
    https://frank.pocnet.net/sheets/137/6/6BC8.pdf

52. `noob-compressorlab`, `research/SURVEY.md`. The survey that ranked this unit second, listed the
    documents and judged its ground truth thin — a judgement sections 4.6 and 7.2 supersede.
53. `noob-compressorlab`, `research/Neve-33609.md`. The diode-bridge dossier, whose structure and standards
    this file follows.
54. `noob-compressorlab`, `research/CL-1B.md`. The optical dossier, likewise.

56. `noob-compressorlab`, `research/UA-176.md`. The variable-mu sibling dossier. Cited for the 6BC8's
    measured μ behaviour (12.3), its fitted transconductance law and that law's validity range and
    accuracy floor (4.3), and the observation that two manufacturers' curves for one tube differ by 1.3 to
    1.5 dB. Several corrections in this file came from that research and are attributed where they land.

**Unreachable, listed so nobody repeats the search**

- Fairchild's own product datasheet, linked from [22] as `/oldmiximage/online_extras/fairchild-datasheet.pdf`
  — 404, and no Wayback snapshot exists.
- `fairchildrecording.com` — the domain does not resolve.
- `undertoneaudio.com` — connection failed on every attempt; their UnFairchild 670M is undocumented here.
- Universal Audio's webzine article "Compression Obsession: The Fairchild 670", Will Shanks, September 2003,
  cited by [19] — dead at `uaudio.com` and blocked with HTTP 429 at the Wayback Machine.
- Reverb and every auction record — no listing data is served to a plain fetch and WebFetch is refused with
  HTTP 403. **No sourced sale price was obtained** (1.5).
- Abbey Road Studios' website — reached in full, 52-URL sitemap plus five hand-checked pages, and the word
  "Fairchild" appears nowhere on it (1.0).
- Acustica Audio — their site is a JavaScript application that serves no product text, and no page I reached
  names Fairchild, 660, 670 or variable-mu. **The widely repeated claim that Acustica sampled a Fairchild is
  unsourced as far as this search goes.**
- Any open DIY 670 build page with real component values. Gyraf has no variable-mu project [50]; Drip
  Electronics' `/670.html` is a 404. The POM pages [45] and Sound On Sound [26] are the substitutes.

[1]: https://archive.org/details/Fairchild_670_owners_manual
[2]: https://archive.org/details/JL10882
[3]: https://archive.org/details/JL10878
[4]: https://archive.org/details/JL10883
[5]: https://archive.org/details/JL10866
[6]: https://archive.org/details/JL10865
[7]: https://archive.org/details/JL10873
[8]: https://archive.org/details/JL10876
[9]: https://archive.org/details/JL10882
[10]: https://archive.org/details/JL10882
[11]: https://archive.org/details/john-leimseider-archive
[12]: https://frank.pocnet.net/sheets/142/6/6386.pdf
[13]: https://www.radiomuseum.org/tubes/tube_6386.html
[14]: https://www.radiomuseum.org/r/fairchild_stereo_limiter_670.html
[15]: https://www.jjtubes.eu/jj-6386-lgp-new
[16]: https://frank.pocnet.net/sheets/049/6/6973.pdf
[17]: https://frank.pocnet.net/sheets/127/6/6BL7GTA.pdf
[18]: https://www.dafx.de/paper-archive/2012/papers/dafx12_submission_9.pdf
[19]: https://en.wikipedia.org/wiki/Fairchild_660
[20]: https://en.wikipedia.org/wiki/Fairchild_Recording_Equipment_Corporation
[21]: https://en.wikipedia.org/wiki/Sherman_Fairchild
[22]: https://www.mixonline.com/technology/1959-rein-narma-fairchild-670-compressor-limiter-377967
[23]: https://musictech.com/reviews/studio-icons-fairchild-660670/
[24]: https://musictech.com/reviews/vintage-rewind-fairchild-660-and-670/
[25]: https://vintageking.com/fairchild-660-670-compressor-limiter
[26]: https://www.soundonsound.com/reviews/fairchild-660-670
[27]: https://www.radiomuseum.org/dsp_hersteller_detail.cfm?company_id=7890
[28]: https://www.aes.org/aeshc/jaes.obit/JAES_V67_11_PG931.pdf
[29]: https://www.youtube.com/watch?v=MzdnXJCtnYw
[30]: https://commons.wikimedia.org/wiki/File:Fairchild_660_limiter_(1960s),_EMI_Presence_Box_(1960s),_Altec_RS124_compressor_(1960s),_Abbey_Road_Studios.jpg
[31]: https://commons.wikimedia.org/wiki/File:Fairchild_Model_670_Compressor.jpg
[32]: https://commons.wikimedia.org/wiki/File:Fairchild_670_Compressor_at_Audio_Mix_House_(2014-07-28_by_Audio_Mix_House).jpg
[33]: https://commons.wikimedia.org/wiki/File:Fairchild_660_limiter_(1960s),_EMI_Presence_Box_(1960s),_Altec_RS124_compressor_(1960s),_Abbey_Road_Studios.jpg
[34]: https://www.uaudio.com/products/fairchild-tube-limiter-collection
[35]: https://www.uaudio.com/blogs/press/fairchild-collection
[36]: https://www.uaudio.com/blogs/press/0115_fairchild
[37]: https://www.waves.com/plugins/puigchild-compressor
[38]: https://www.ikmultimedia.com/products/trvintubcomplim/
[39]: https://www.softube.com/plug-ins/bus-processor-670
[40]: https://www.overloud.com/products/comp670-compressor-limiter
[41]: https://slatedigital.com/plugin/virtual-buss-compressors/
[42]: https://heritageaudio.com/herchild-670/
[43]: https://heritageaudio.com/grandchild-670/
[44]: https://heritageaudio.com/herchild-next-gen-model-670n/
[45]: https://www.fairchild-recording-equipment.com/
[46]: https://www.fairchild-recording-equipment.com/PriceList2026/
[47]: https://www.plugin-alliance.com/en/products/v76u73.html
[48]: https://www.ikmultimedia.com/products/trdynamu/
[49]: https://www.arturia.com/products/software-effects/comp-tubesta/overview
[50]: https://www.gyraf.dk/g-22-dual-stereo-ms-vari-mu-compressor/
[51]: https://github.com/Noob-Audio-Engineering/noob-electrical-components
[55]: https://frank.pocnet.net/sheets/137/6/6BC8.pdf

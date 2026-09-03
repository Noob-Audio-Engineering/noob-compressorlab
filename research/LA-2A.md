# The LA-2A leveling amplifier: research notes for the LA-2A side of `noob-compressorlab`

Research dossier for the LA-2A model of the `noob-compressorlab` example plug-in of noob-vst-webgui-framework. The
example is an affectionate spoof of the Teletronix / UREI / Universal Audio LA-2A. It is
not a product and does not use the LA-2A, Teletronix or Universal Audio names as its own
name. Trademarks below belong to their owners and are used only to identify the device
and the products discussed.

Conventions:

- Citations are `[n]`; the numbered list in section 9 gives the URL for every source.
  Reference-style link definitions at the very end make the `[n]` markers clickable.
- Numbers that come from a manufacturer specification or a measurement are attributed.
  Numbers that are my own derivation are labelled **estimate** or **derived**.
- Where sources disagree, both figures are given.
- "GR" means gain reduction. "PR" means the Peak Reduction control. Levels given as
  "dBm" in the old manuals mean dBu into 600 ohms.

---

## 1. What the LA-2A is

### 1.1 History

- **Jim Lawrence (James F. Lawrence II)**, born 1924, was a WWII Navy radar operator and
  a USC-trained electrical engineer who developed optical sensors for the Titan missile
  programme at Caltech's Jet Propulsion Laboratory in the early 1950s. Working at radio
  station KMGM in Los Angeles he grew tired of riding gain on air, and combined an
  electroluminescent panel with photoresistors in a tube-sized metal can: the "T4"
  optical attenuator. [1] [7]
- Lawrence founded the **Teletronix Engineering Company** in Pasadena in 1958, making
  broadcast products (transmitter tubes, tone generators, transmitters). His first
  leveling amplifier was the **LA-1** (about one hundred made; Gene Autry used one). The
  **LA-2** followed with a larger VU meter and the "T4A" attenuator; CBS bought a pair
  after an engineer showed one to the Ed Sullivan Show crew, then RCA bought them for New
  York and Nashville. [1] [3]
- In **1962** Lawrence reconfigured the LA-2 into the **LA-2A**: gray faceplate, turret
  boards in a strip along the bottom of the chassis, shorter wiring and a lower noise
  floor than the LA-2. [1] [7]
- **1965**: Teletronix sold to **Babcock Electronics** of Costa Mesa. **1967**: Bill
  Putnam's **Studio Electronics** (renamed **UREI** shortly after) bought Babcock's
  broadcast division including the Teletronix brand, and kept building the LA-2A with a
  **silver** faceplate instead of gray. Three variants were produced under the three
  companies before production stopped **around 1969**, as the solid-state LA-3A, LA-4 and
  LA-5 (all using the patented T4) took over. [1] [2] [7]
- Reissues: UA's history article says one limited edition in the 1970s and one in the
  1980s [1]; Wikipedia gives 1979 (UREI) and 1992 (Harman/JBL) [7]. The sources disagree
  on the second date. **Universal Audio**, refounded by Bill Putnam Jr. in 1999, has
  built the LA-2A continuously since 1999 [1] [6], and shipped the first UAD LA-2A plug-in
  in 2001 [3] (Wikipedia says 2002 [7]). The LA-2A entered the TECnology Hall of Fame in
  2004 [7].
- The reissue T4 required "a lengthy study of the original photocell formula", the
  original manufacturing equipment and re-qualifying the manufacturer; UA says the T4 was
  not fully understood until the DSP modelling research for the 2001 plug-in. [1]

### 1.2 T4, T4A, T4B, T4C

- All variants are a sealed octal-based can holding an electroluminescent (EL) panel and
  cadmium-sulfide (CdS) photoresistors. One photoresistor is the audio attenuator, a
  second, hand-matched one drives the gain-reduction meter. [1] [2]
- **T4A** (LA-2, early LA-2A) and **very early T4B** (up to about 1969) contained three
  photocells: the main Clairex CL-505L pair plus a fast Clairex CL-705 wired in parallel
  with the audio cell, giving a dual time constant that broadcast engineers liked. Later
  T4Bs dropped the third cell; Kenetek's Ken Kantor concluded that the overall response
  "is dominated by the response of the slower photocell". [23] [24] [31]
- **T4B** is the module in the late-1960s silver units and in all reissues. Original
  T4Bs had a 2.2 megohm resistor across the EL panel; UA's current T4B adds a 4.7 nF
  series capacitor and uses a GSI EL panel with Silonex NSL-02-042 photocells; the UREI
  T4B examined by IGS Audio used CL-505L cells, the parallel CL-705HL, a 2M2 resistor and
  a brass wire mesh over the panel. [31] [33]
- **T4C** was fitted only to the UREI BL-40 Modulimiter and was meant to be slower, but
  original T4Cs varied so much that many were faster than a "classic" T4B. [24]
- Photocells are sprayed with photosensitive material and the density varies, so "slower
  photocells are inherently more sensitive (lower threshold and longer memory effect)
  than faster ones"; attack and release are co-dependent (slow release means slow
  attack); EL panels wear "like tires". [23]

### 1.3 Why it is famous

- It was the first electro-optical compressor to use an EL panel instead of a neon or
  incandescent lamp, which gave a usable attack for broadcast, and the CdS cells gave a
  "two-stage" release that was more transparent than contemporaries. [2]
- Two controls, 40 dB of gain (usable as a preamp), and a release that is "slow, gentle,
  and versatile". Bill Putnam Jr.: "It responds especially well to the human voice in a
  way that inspires performance." [1] [6]
- Engineers use it above all on vocals and bass. Jim Scott: "LA-2As warm things up
  ... they EQ all the warmth and low mids and bass. When you put bass and drums in them
  they get fatter and bigger. And unless you hit them way hard and make the tubes sizzle
  they don't really distort." [2] Sound On Sound: an "extremely soft-knee" compressor with
  "fairly slow attack" and programme-dependent release that "preserves the impression of
  performance dynamics despite quite extreme level management", used on Alanis
  Morissette's and Kurt Cobain's vocals among many others. [7] [9]

---

## 2. Controls and their real ranges

| Control | Hardware | Range / behaviour | Sources |
|---|---|---|---|
| Gain | 100 kΩ pot (R1) across the attenuator output, feeding the 12AX7 / 12BH7 output amplifier | 0 to about +40 dB make-up (spec: gain 40 dB ±1 dB). Does not affect compression. Knob 0-100 is arbitrary. | [2] [3] [4] |
| Peak Reduction | 100 kΩ pot (R2) at the sidechain input | Sets sidechain amplifier drive, which sets both threshold and amount of GR; up to 40 dB of GR. Knob 0-100 arbitrary; the scale is not linear and most use is between 30 and 50. | [2] [3] [55] |
| Limit / Compress | Toggle in the sidechain circuit | Compress: sidechain fed from the attenuator output (pure feedback), soft knee, "about 3:1" (UA) or "4:1" (Mix, others). Limit: sidechain fed from a tap that adds a small fraction of the input; higher ratio, harder knee. Differences are subtle below a few dB of GR. See section 3.4. | [2] [3] [8] [17] [29] [30] |
| Meter switch | 3-position rotary | Gain Reduction (needle rests at 0 VU and swings left by the GR in dB); Output +10 (0 VU = +10 dBm); Output +4 (0 VU = +4 dBm). | [2] [3] |
| Meter zero | Screw pot next to the switch | Zeroes the GR meter with no signal; users report it drifting by about 1 dB after an hour, so warm up first. | [2] [29] |
| R37 "Limit Response" / pre-emphasis | 1 MΩ screw pot on the front panel (rear on some units), in the sidechain after the 12AX7 | Clockwise = flat (factory setting). Counter-clockwise raises the pot's resistance and makes compression "increasingly more sensitive to the higher frequencies". Intended for FM, where 75 µs pre-emphasis boosts 15 kHz by 17 dB. Third-party descriptions: attenuates sidechain lows by up to 10 dB, emphasis above about 1 kHz [16]; "shelf filter" [3]; "increases voltage amplifier gain ... above 1 kHz" [55]. | [2] [3] [13] [16] [55] |
| R3 stereo balance | 1 MΩ trimmer | Balances GR between two units linked via terminal 6; calibrate at about 5 dB GR. | [2] |

Additional control facts:

- The hardware has no attack, release, ratio or threshold controls. The UA plug-in
  manual: PR "applies the compression threshold to the incoming signal up to -40 dB". [3]
- Waves' model calibrates the hardware so that -18 dBFS in the DAW equals +4 dBu = 0 VU
  on the unit; the Gain control's unity setting is 32.28 on the 0-100 scale. [55] UA's
  Leveler Collection runs at an internal reference of -12 dBFS. [3]
- The original manual and several reviews say "attack 10 microseconds"; this is a known
  typo for 10 milliseconds, corrected on UA's pages. [5] [8] [29]

---

## 3. Signal path and circuit behaviour

### 3.1 Block diagram (UA manual, figure 3)

Input transformer -> optical attenuator (T4) -> Gain pot -> voltage amplifier (12AX7A)
with negative feedback -> cathode follower (12BH7A) -> output transformer (UTC A-24).
Sidechain: Peak Reduction pot -> voltage amplifier (12AX7A, both halves) -> pre-emphasis
trim (R37) -> driver amplifier (6AQ5A) -> EL panel. Stereo interconnection at the driver.
[2]

Tube complement: two 12AX7A, one 12BH7A, one 6AQ5A. [2] Original units used a UTC A-10
input transformer and UTC A-24 output; the UA reissue uses an HA-100X input and A-24
output. [2] [8] [53] The Klark Teknik clone substitutes an EL84 for the 6AQ5. [71]

### 3.2 The attenuator (UA manual, figure 5)

The input transformer secondary feeds R6 (68 kΩ) in series with R7 (2.7 kΩ) to a node
that is shunted to ground by the T4 audio photocell; the 100 kΩ Gain pot sits across the
same node. The photocell is "the bottom leg in a voltage divider": high resistance in
the dark means no gain reduction, and the resistance "can not go completely to zero and
hence there will always be some signal present". [2] The EL panel is coupled through a
4.7 nF capacitor; the sidechain tap is via R5 (68 kΩ). [2]

Rough numbers for the divider (**derived** from the schematic values): with the cell
dark (more than 1 MΩ) the node sits at about 0.56 of the transformer voltage (-5 dB
insertion loss, absorbed by the gain stage). Unity-relative gain reduction of 6 dB occurs
when the cell's resistance in parallel with the pot equals 68 kΩ, i.e. a cell of about
212 kΩ (Compress tap) or 241 kΩ (Limit tap). [17] With a bright-light cell resistance of
0.5-2 kΩ the divider gives 26-38 dB of GR, which is why the spec's "up to 40 dB" is only
reached with a very low cell resistance. [2] [17] [19]

### 3.3 The T4 cell as a transducer

- The EL panel "is essentially a night-light"; the larger the drive voltage, the
  brighter it glows. EL panels are made for 120 V, 60 Hz mains and audio drive shortens
  their life. [2]
- EL panels are capacitors. A modern panel measured 1.6-1.9 nF; old panels had a
  breakdown of about 100 V, modern Japanese ones a threshold of about 20-30 V; luminance
  rises with frequency and peaked around 7 kHz in one measurement, with the driver's
  10 kΩ output impedance and the panel capacitance giving a corner near 8 kHz. Constant
  current drive was "not usable"; constant voltage drive is required. Colour shifts from
  yellow-green to green-blue as frequency rises. [20]
- Time-averaged EL brightness of ZnS phosphors follows the Alfrey-Taylor relation
  B = B0 exp(-b / sqrt(V)) over a wide frequency range. [39]
- The CdS cells (Clairex CL-505L: 7.5 kΩ at 2 footcandles, spectral peak 550 nm, 60 V
  rating) have resistance that "varies between less than 1 kΩ and more than 1 MΩ". Clairex
  type-5 material rise time goes from 1.1 s at 0.01 fc to 2 ms at 100 fc, decay from
  0.12 s to 5 ms over the same range. [21] [35]
- PerkinElmer's application note: rise time is the time for conductance to reach 63 %
  of final, decay to fall to 1/e; at 1 fc both are "typically 5 msec to 100 msec"; "all
  material types show faster speed at higher light levels and slower speed at lower light
  levels"; dark storage slows response; light history changes resistance by a factor of
  1.55 at 0.01 fc down to 1.10 at 100 fc, and 24 h is needed to reach a steady state.
  Resistance vs. illumination is a straight line on log-log paper with slope gamma,
  defined between 10 lux and 100 lux. [36] Datasheet gammas for common CdS cells are
  0.6-0.9 (0.65 and 0.75 are typical stated values). [38]
- General CdS opto-isolator behaviour: turn-on about ten times faster than turn-off;
  turn-off 2.5 to 1000 ms; resistance rises about 1 % per °C at low light; distortion
  below 100-300 mV across the cell is about 0.01 % and second-harmonic dominated, above
  that a third harmonic appears and grows with the square of the voltage. [37]
- The UA manual's summary: "two-stage decay: after the light is removed from the cell,
  it releases quickly (40-80 milliseconds) to approximately half of its off resistance.
  The remainder of its release can take place over as much as several seconds", and
  "memory: the amount of time it takes for the cell to recover ... depends on how long
  light had been shining on it and how bright the light", so release is slower after long
  or deep compression. [2]
- Note two errors in otherwise useful sources: UA's history page says the photoresistor's
  impedance "increases" with light [1] (it decreases, which is what shunts the signal
  [2]), and the Huddersfield paper describes a "tungsten filament lamp" inside the T4
  [53] (it is an EL panel [1] [2]).

### 3.4 The sidechain and the Limit / Compress switch

- The LA-2A "is a feed-back style compressor": the sidechain is driven by the
  gain-reduced signal. The PR pot sets sidechain drive, a 12AX7 amplifies, R37 shapes,
  then a 6AQ5 drives the EL panel. [2] The sidechain amplifier is a "least parts" class-A
  stage with higher distortion than the negative-feedback output amplifier. [17]
- Circuit reading used by most DIY analysts (KVR user aciddose, GroupDIY users
  Voyager10 and ruffrecords): in **Compress** the switch shorts the sidechain to the
  attenuator output node, so `sidechain = output`. In **Limit** the sidechain is taken
  from between R6 and R7, so it becomes a linear mix of about 1/25 input and 24/25
  output; "in limit mode it is more or less like feedforward" and "you get more
  compression when the switch is opened". [17] [18] [19] With R6 = 68 kΩ and R7 = 2.7 kΩ
  the input fraction is R7/(R6+R7) = 0.038 (**derived**).
- A second reading (ruffrecords, same thread): in Compress R7 stays in the divider and
  limits the maximum GR to about 26 dB; in Limit R7 is shorted so the cell can pull the
  node down to about 680 Ω for the full 40 dB. [19] The UA schematic places the "LIM"
  switch in the sidechain figure [2], which supports the first reading; the two readings
  are not mutually exclusive in effect (both raise loop gain at heavy GR).
- Consequence (**derived**): at light GR the attenuator output is close to the input,
  so the 4 % input term is negligible and the two modes behave almost identically. At
  heavy GR (output pulled 20-30 dB below input) the feed-forward term dominates, the
  sidechain keeps rising with input even though the output is clamped, and the ratio
  climbs steeply. This matches user reports that the switch does "merely nothing" at
  2-6 dB of GR and matters "if you are hitting it hard". [29] [30]
- Sidechain time constants (aciddose, from the schematic): the amplified sidechain is
  high-passed and fed through 47 kΩ into a 10 nF capacitor from 220 kΩ, giving about a
  70 Hz low-pass ("5 ms to 90 %") ahead of the driver; the 6AQ5 plate is fed from about
  275 V through 10 kΩ and pulses into C11 (10 nF) onto the EL capacitance; the panel's
  2.2 MΩ bleed resistor sets a long discharge path. [17]
- Frequency dependence is built in even with R37 flat: GroupDIY user PRR reported the
  ratio changing from "18:8" at 100 Hz to "18:4" at 10 kHz for the same limiting depth,
  i.e. 10 dB GR versus 14 dB GR for an 18 dB input rise (**derived** from his figures).
  [20] UA states plainly that the ratios "are nonlinear and frequency dependent". [3]

### 3.5 Output stage and metering

- Output: 12AX7 voltage amplifier with negative feedback, 12BH7A cathode follower, UTC
  A-24 output transformer for impedance matching and a balanced 600 Ω output. [2]
- Metering: the second photocell, illuminated by the same panel and hand-selected to
  match, drives the meter in GR mode; +4 and +10 positions read output level. [2]
- Stereo: terminals 6 of two units are joined (shielded, under 2 feet) so that the
  sidechains are bridged and both units produce the same GR; R3 trims the balance. [2]

---

## 4. Measured behaviour

### 4.1 Published specifications

| Quantity | Original Teletronix spec [5] [74] | UA reissue manual [2] |
|---|---|---|
| Gain reduction | up to 40 dB | up to 40 dB |
| Attack | 10 ms | "very fast" (product page: 10 ms) |
| Release | 0.06 s for 50 %; 0.5-5 s complete, depends on previous reduction | same |
| Frequency response | +0/-1 dB, 30 Hz-15 kHz | ±0.1 dB, 30 Hz-15 kHz |
| Distortion | < 0.5 % THD (0.25 % typical) at +10 dBm | < 0.35 % at +10 dBm, < 0.75 % at +16 dBm |
| Noise | 70 dB below +10 dBm | 75 dB below +10 dBm |
| Gain | 40 dB ±1 dB | 40 dB ±1 dB |
| Output | +10 dBm nominal, +16 dBm peaks | same |
| Impedances | 50/150/250/600 Ω selectable | 600 Ω balanced in and out |

UA's plug-in copy restates the release as "about 60 milliseconds for 50 % of the release,
and anywhere from 1 to 15 seconds for the rest" and the ratio as "roughly 3:1". [4]
Clones quote similar numbers: Warm Audio WA-2A (Kenetek T4B) attack 10 ms "varies
somewhat with frequency", release 0.06 s / 0.5-5 s, ratio "about 4:1" in Compress and
"closer to 100:1" in Limit [69] [70]; Klark Teknik 2A-KT attack typically 10 ms, release
typically 50 ms for 50 % and up to 5 s, THD < 0.1 % at unity gain. [71]

### 4.2 Attack

- The 10 ms figure is a manufacturer average. A Gearspace user (Canopus) recorded a
  -24 dBFS tone with an instantaneous -3 dBFS peak and saw 63 % of a 6 dB GR reached at
  about 50 ms. [29]
- Moore (University of Huddersfield, 2025) measured six units (three vintage Teletronix,
  three UA reissues) with a 1 kHz burst of -18 / -6 / -18 dBFS: attack-to-stabilisation
  33-81 ms (mean 52.8 ms, SD 18.5). "Stabilisation" was a 1 % derivative criterion, so it
  is a longer measure than a 63 % onset. [53]
- CdS rise time shortens with illumination (section 3.3), and a physical vactrol model
  shows "the higher the input voltage, the shorter the attack and the longer the
  release". [36] [44] In a feedback loop the closed-loop attack is also faster than the
  open-loop cell because the error signal is larger for bigger overshoots.
- Kush Audio's Gregory Scott: a compressor with a nominal 10 ms attack "lets most of a
  transient thru", whereas the LA-2A "just pancakes them". There is no industry standard
  for defining attack time. [29]

### 4.3 Release and memory

- Manufacturer: 40-80 ms to roughly half of the recovery, then 0.5-5 s (or 1-15 s) for
  the rest, longer after longer or deeper compression. [2] [4]
- Canopus, after a sustained peak: 50 % recovery in 0.8 s, 85 % in 1.5 s, 99 % in
  21.65 s. [29]
- Moore: release-to-stabilisation 449-1670 ms (mean 916 ms, SD 427) after the -6 dBFS
  section of the burst; release varied far more between units than attack. [53]
- ProReplicas: the release tail "can extend even to tens of seconds at heavy
  compression". UA's plug-in manual: the T4 "can take a few minutes to fully recover".
  Waves warns that "the same passage [may] sound different during successive playbacks,
  as the Release does not return to the unity position". [3] [32] [55]
- Joe-electro (GroupDIY) selects cells for "approximately 60 ms for 50 % release, then
  gradual release over 1-15 seconds". [25]
- Physics behind the memory: photocurrent decay in CdS has a fast recombination
  component (about 1 ms) and a slow trap-controlled component (about 10 s); at low light
  the slow component dominates; decay time = recombination time + de-trapping time
  scaled by the re-trapping probability. [40] The port-Hamiltonian vactrol model of
  Najnudel et al. reproduces program-dependent attack and release from Shockley-Read-Hall
  recombination with two carrier populations whose rate constants differ by two orders
  of magnitude. [44]

### 4.4 Ratio, knee and threshold versus Peak Reduction

Reported Compress / Limit ratios disagree:

| Source | Compress | Limit |
|---|---|---|
| UA plug-in manual [3], UA tips [4] | about 3:1 | about infinity:1, "nonlinear and frequency dependent" |
| Mix magazine review of the reissue [8], UA WebZine quoted in [30] | soft knee, 4:1 "not really fixed" | infinity:1 |
| mix:analog [16], Requisite L2M designer via [29], note.com [72] | about 3:1 | closer to 10:1 (note.com: 10:1 to 20:1) |
| Waves CLA-2A [55], Warm Audio [70] | about 3:1 / 4:1 | about 100:1 |
| Tim Farrant, Buzz Audio (guess) [29] | roughly 10:1 | roughly 20:1 |
| Gearspace user EvgenyStudio, "real-world measurements" [30] | 3:1 | 4.2:1 |
| Yu and Fazekas, feed-forward fit to SignalTrain data [46] | about 4:1, rising with PR (fitted values span roughly 3 to 7 over PR 40-100) | "slightly higher", same trend |
| aciddose, circuit estimate [17] | maximum depth about 1/20 to 1/34 of level; "there is no such thing as ratio here since the transfer function is a continuous function" | |

Points of agreement: the knee is very soft; there is no fixed threshold; PR moves both
the threshold and the effective ratio ("This knob affects both the compression ratio and
the threshold level simultaneously" [56]); ratio grows with drive; and the difference
between modes is small at low GR. Yu and Fazekas could not fit a feed-forward compressor
below PR 40 because "the compression is less noticeable in this range and is surpassed by
other non-linearities"; their fitted threshold falls monotonically with PR over the
40-100 range, spanning roughly -10 to -40 dB on the SignalTrain scale, and the fitted
attack and release times "vary exponentially with the peak reduction". [46]

### 4.5 Frequency dependence and R37

- Built-in: more compression at high frequencies (section 3.4). [3] [20]
- R37 counter-clockwise: less low-frequency sensitivity, so overall GR drops on
  bass-heavy sources and sibilance is compressed more; Reid Shippen used it as a
  "character changer" and noted that returning it to flat "instantly started attenuating
  more heavily". [13] [14] Mike Shipley used the rear "flat control" to roll off
  frequencies on piercing voices. [2]
- A Drip T4B with a 47 nF series capacitor "practically doesn't work for 1 kHz" and
  gives "no compression on bass sounds", showing how strongly the panel coupling network
  shapes the sidechain response. [31]

### 4.6 Distortion and frequency response

- Nominal THD is below 0.5 %, but THD rises during gain reduction. GroupDIY users
  measured at +15 dB input: 0.54 % at 0 dB GR rising to 4.2 % at 5 dB GR with a 1:4 input
  transformer, and 0.26 % rising to 1.37 % at 10 dB GR with a 1:1 transformer; the
  distortion was mostly third harmonic, attributed to "gain change across a single cycle
  of the waveform" and to the cell's non-linearity: "All LA-2As do it. So does the LA-3.
  It's part of the nature of the unit." [26]
- Moore: at 6 dB GR and +4 dBu, THD at 1 kHz ranged 0.9-4.2 % across six units, with the
  third harmonic 10-20 dB above the second. Frequency response variance within
  30 Hz-15 kHz was 0.5-1.3 dB, the deviations lying below 30 Hz or above 14 kHz
  (up to -5.5 dB at the extremes on one unit), attributed to transformers. [53]
- UA notes "an interesting sidechain distortion ... at the most extreme Peak Reduction
  settings, which primarily affects low frequencies". [3] This is consistent with the EL
  panel emitting on both half cycles, so that for low frequencies the cell resistance
  ripples at twice the signal frequency (**interpretation**).
- The tubes: "unless you hit them way hard ... they don't really distort"; when pushed
  "the tube will get crispier on the top". [2] [66]

### 4.7 Unit-to-unit variation

- UA models three eras with "distinct variations in time constants, compression knee,
  headroom, distortion, program and frequency dependence": late-1960s Silver (T4B, fast
  time constant, more headroom), mid-1960s Gray (medium time constant, "reference"
  starting point) and the LA-2 (slowest, "mellowed" by 50 years of EL panel ageing,
  hard-wired in Limit, "most distinctive clipping character"). [3] [4]
- Moore found attack spread 33-81 ms and release 449-1670 ms across six units, THD
  0.9-4.2 %, one unit unable to exceed 9 dB of GR, and no consistent vintage-versus-
  reissue grouping; trained listeners could distinguish the two most different units only
  in a dense rock mix, and only marginally. [53]
- Waves: a "depleted" T4 gives "up to 80 % less compression", and "up to 90 % of T4
  components in use today have never been replaced". [55] Jim Scott: LA-2As are "more
  inconsistent piece to piece than the 1176s, because of the tubes". [2] Kenetek: wide
  attack/release variation within a batch, threshold tied to speed. [23]

---

## 5. Sound character, and what makes emulations right or wrong

Descriptions from users and reviewers:

- "Big, fat, and warm"; vocals sit "up front" with "edges bleeding into the rest of the
  mix". [66] "Gentler and warmer" than VCA designs; "long and lush and beautiful" notes.
  [9] Low-mid thickness, "silky" tubes and transformers. [3] [15]
- Attack Magazine (Gregory Scott, comparing hardware with UA and Cakewalk plug-ins by
  ear): the hardware has "significantly more punch", a "tighter envelope and faster
  here-and-gone sensation", more 60-80 Hz presence and less sub-40 Hz weight; plug-ins
  are "heavier, slower", extend deeper in the lows and are more neutral; the CA-2A
  behaved like "a fast-ish limiter"; a commenter felt the Silver model attacks faster
  than the hardware. [65]
- Compress "sounds more like a slightly thick liquid; limit sounds more linear"; others
  hear no difference at all until GR is heavy. [30]

What the literature and the measurements say an emulation must get right:

1. **Feedback topology with an emergent knee.** There is no threshold/ratio pair; the
   curve comes from EL threshold, CdS gamma and loop gain. A feed-forward fit needs a
   different threshold, ratio, attack and release for every PR setting and still fails
   below PR 40. [46] [17]
2. **Two-stage, history-dependent release.** A single release time constant "does not
   capture the documented two-stage release behaviour" [46]; neural models needed about
   300 ms of receptive field to reach good accuracy [48], and the best models are those
   with explicit long memory [51]. The release must lengthen with the duration and depth
   of prior compression. [2]
3. **Level-dependent attack** (faster when hit harder) and **frequency-dependent
   sensitivity** (more GR at high frequencies, tunable by R37). [3] [20] [36] [44]
4. **Modest, mostly odd-order distortion that rises with GR** (1-4 % THD at 6 dB GR)
   and low-frequency sidechain ripple at extreme settings; clean tubes at nominal level,
   soft clipping only when the Gain stage is driven. [3] [26] [53]
5. **Transformer band limits** (gentle roll-off below 30 Hz and above 14-15 kHz) rather
   than flat response. [53] [5]
6. **Calibration.** A plug-in needs an explicit reference (Waves -18 dBFS = +4 dBu, UA
   -12 dBFS) because the hardware has a fixed internal threshold. [3] [55] [72]
7. **Variation** is real (three UA models, T4 ageing), so a single "correct" set of time
   constants does not exist; exposing an "age" or "cell" choice is defensible. [3] [55]

Softube's Niklas Odelholm on modelling in general: "a difference in frequency response at
the input transformer will lead to different distortion in the next stage", and a 2 mV
bias difference in a detector circuit produced audible distortion differences. UA's Dave
Berners starts from the schematic, then measures parasitics and calibrates component
values to measurements. [67] [68]

---

## 6. How the LA-2A and optical compressors are simulated

### 6.1 Physical and grey-box models of photoresistors and opto cells

- **Parker and D'Angelo, DAFx 2013** (Buchla lowpass gate, VTL5C3 vactrol): heuristic
  one-pole on the light control with time constants switched by the sign of the
  derivative, about 12 ms rising and 250 ms falling, "modulated further by the current
  output value of the vactrol model, so that it responds quicker when at high values";
  LED-current to resistance law R = A·I^-1.4 + B with A = 3.464 and B = 1136 Ω. [43]
- **Eichas and Zölzer, SPIE 2016** (VTL5C2 in a guitar compressor): measured LDR
  turn-on below 1 kΩ within 5 ms at 10 mA and turn-off above 1 MΩ only after 500 ms
  ("100 times longer"); the digital model is a peak detector, a measured static
  input-level-to-gain lookup table with pre/post gains, a smoothing block of three
  first-order low-passes with separate attack and release coefficients and blending
  weights, plus a measured FIR for the linear response; parameters fitted by
  Levenberg-Marquardt, achieving error-to-signal ratios of 1-3 % and PEAQ grades better
  than -1. [41] The same group generalised the approach to DRC systems (AES 142, 2017)
  and to the 1176LN. [42]
- **Najnudel, Müller, Hélie, Roze, DAFx 2023** (VTL5C3/2): a port-Hamiltonian,
  passive, "entirely physical" model. Free carriers obey q'- = -f_opt - a-(q- - q+)q-
  and q'+ = -f_opt - a+(q + q+ - q-)q+ (Shockley-Read-Hall), the cell resistance is
  R = 1/(u+ q+ + u- q-) bounded between dark and light limits, the LED is a softplus
  law with explicit threshold, and the optical coupling is a dual-slope power law
  f(P) = P0·P^a0 + P1·P^a1. Identified rate constants differ by two orders of magnitude
  (0.977 and 135 C^-1 s^-1), which is what produces fast and slow recovery. Simulating a
  divider compressor showed "the higher the input voltage, the shorter the attack and
  the longer the release" and that ratio and knee are set by the divider resistances.
  [44] The underlying photoconductor transient theory is Iverson's (cited in [43] [44]).
- **Wright and Välimäki, DAFx 2022** (grey-box, LA-2A, SignalTrain data): a standard
  log-domain compressor (Giannoulis et al. [54]) whose threshold, ratio and knee are
  predicted from the PR setting by a small MLP, a level detector that is either a
  one-pole, a switching one-pole (separate attack/release) or an RNN-modulated one-pole,
  and a make-up stage that is either a constant or a 4-unit GRU. Learned attack time
  constants were about 15 ms or less with much longer release; the RNN detector learned to
  make the time constant "very small when the input signal is large" and larger when it
  falls; the best grey-box models reached ESR 0.031-0.032 against 0.023 for a 32-unit GRU
  black box while using under 10 % of its operations. [45]
- **Yu and Fazekas, AES AIMLA 2025**: a five-parameter feed-forward compressor fitted
  to each SignalTrain PR setting by Newton-Raphson; mappings from PR to threshold, ratio,
  attack, release and make-up are published; they note the LA-2A "is technically a
  feedback compressor" and that modelling the two-stage release explicitly should improve
  results. Compared on the same data, the UA plug-in and Waves CLA-2A showed similar
  error patterns (lighter than the unit at PR 40-60, heavier at 60-80), and Cakewalk's
  CA-2A had the highest error, above 40 % ESR at PR 100. [46]

### 6.2 Black-box neural models

- **Hawley, Colburn, Mimilakis 2019 (SignalTrain)**: about 20 hours of LA-2A input/output
  pairs at 44.1 kHz, PR in steps of 5 for both switch positions, 15-minute files; 21 GB on
  Zenodo. [47]
- **Steinmetz and Reiss 2022**: dilated TCNs with FiLM conditioning on PR and
  Compress/Limit; about 300 ms receptive field needed; real time on CPU with 10 minutes
  of training data; listeners still told model and hardware apart. [48]
- **Simionato and Fasciani 2022 / 2023**: feed-forward, LSTM and encoder-decoder models
  of the Tube-Tech CL 1B (DAFx 2022), then a conditioned low-latency model applied to the
  LA-2A (DAFx 2023) which restates the device as "average attack time of 10 ms and a
  multi-stage release", first stage 0.06 s, second stage "controlled by the photocell's
  memory ... 0.5 to 5 seconds". [49] [50] Their 2024/25 JAES work uses selective state
  space (S6) layers, about 1000 parameters and under 200 MFLOPS for stereo at 48 kHz, with
  64-sample latency, and states that "the release time for the LA-2A cannot be known a
  priori, as it is highly dependent on the signal's history". [51] A parallel S4-based
  study reports real-time performance on the same dataset. [52]

### 6.3 What commercial emulations say they model

| Product | Claimed model content | Notes |
|---|---|---|
| Universal Audio Teletronix LA-2A Leveler Collection (Silver, Gray, LA-2, plus Legacy) | Three measured units; time constants, knee, headroom, distortion, program and frequency dependence; transformer and I/O distortion (not in the 2001 Legacy version); R37 exposed on all, "hot-rodded" onto the LA-2; LA-2 hard-wired Limit; -12 dBFS reference; ratio 3:1 / infinity:1 | UA's own description; DSP research "at the quantum physics level" is marketing language [1] [3] [4] |
| Waves CLA-2A | Modelled from Chris Lord-Alge's unit; THD, "variable release times" lasting seconds, 50/60 Hz hum, T4 ageing discussion; HiFreq control (voltage amplifier gain above 1 kHz); ratio 3:1 / about 100:1; -18 dBFS = +4 dBu | [55] |
| Native Instruments / Softube VC 2A | Softube modelling (Rosén, Öberg, Odelholm); Comp = softer curve and lower ratio, Limit = higher; PR moves ratio and threshold; adds sidechain input, detector high-pass, dry blend | [56] |
| IK Multimedia T-RackS White 2A | "no electronic circuitry involved with the compression itself. It's just a tube amp with photo-resistors, lighted by a fluorescent panel driven by the output signal" | [57] |
| Cakewalk CA-2A | Four tubes, EL panel plus photocell as the attack/release source, R37 as a 0-100 % control; "compression ratio varies at different frequencies"; "only reduces gain to a certain point before giving in" | Measured as the least accurate of the tested plug-ins in [46]; "fast-ish limiter" per [65] |
| Black Rooster Audio VLA-2A | "SPICE-type component-based circuit simulation"; T4A cell "modelled as a living component rather than baked into a fixed envelope"; emphasis network matched to a 1968 unit | [59] |
| Niviem OPT4 (free) | "EL panel threshold, CdS photoresistor gamma curve, two-stage release [60 ms then 0.5-5 s], and program-dependent memory effect"; feedback topology; divider R_photo/(R_photo+R_fixed); 12AX7/12BH7 harmonics; UTC transformer B-H saturation; about 3:1 / 10:1+ | Closest published description to the design in section 7 [60] |
| Analog Obsession LALA (free) | "roughly 10 ms attack and a dual-stage release"; sidechain filter and mix; "not a detailed circuit-level emulation" | [61] |
| Neold U2A | Two-phase release ("about 60 ms to recover 50 %, then a tail from 0.5 to several seconds"), an Aging control for unit variation, a Recovery control, decoupled drive and gain, R37 exposed | [62] |
| Softube OPTO Compressor | "an iconic early 1960s T4 Opto Cell tube compressor/limiter" plus a Time control | [64] |
| Arturia Comp TUBE-STA | Not an LA-2A: it recreates the Gates STA-Level, a variable-mu design. Arturia does not sell an LA-2A emulation. | [63] |

Hardware recreations describe the same ingredients: mix:analog's dual unit uses original
UTC transformers, a vintage UREI T4B "slow" cell and a Drip EL "fast" cell, a Feedback
knob for tube-stage distortion, and states 3:1 / about 10:1 and a 10 dB, above-1 kHz
R37 shelf. [16] Warm Audio uses a Kenetek T4B [70]; AudioScape builds its own T4B and
stresses photocell matching [33].

---

## 7. Recommended DSP design for noob-vst-webgui-framework (44.1 to 96 kHz, real time)

The design below is a grey-box physical model: the circuit topology is kept (feedback
compressor, divider attenuator, EL light law, CdS cell with traps), and the constants
are tuned to the published behaviour. Everything runs at the audio rate with one-pole
recursions; no block look-ahead and no latency.

### 7.1 Block diagram in words

1. **Input conditioning**: first-order high-pass at 20 Hz (input transformer). Optional
   mild low-frequency saturation.
2. **Attenuator node**: `y_att = x * A(R_cell)` where `A` is the divider gain.
3. **Make-up**: Gain knob, 12AX7 / 12BH7 stage with soft asymmetric clip, then the
   output transformer band limit (first-order low-pass around 30 kHz for the vintage
   voicing, high-pass already applied at the input).
4. **Sidechain tap**: Compress `s = y_att`; Limit `s = (1-beta) * y_att + beta * x`
   with `beta = 0.038`.
5. **Peak Reduction**: `v = g_pr(PR) * s`.
6. **Sidechain amplifier**: soft saturation `v = V_sat * tanh(v / V_sat)`.
7. **Sidechain shaping**: R37 low-shelf cut, then the fixed EL/driver tilt.
8. **EL light**: full-wave rectify, smooth with a 1 ms one-pole, apply the Alfrey-Taylor
   law to get light `L`.
9. **CdS cell**: carrier / trap state update gives conductance `G`, hence `R_cell` and
   `A(R_cell)` for the next sample (one-sample loop delay, negligible at these time
   constants).
10. **Meter**: GR in dB from the (matched) meter cell, or output level, through VU
    ballistics.
11. **Stereo link**: shared cell state driven by the mean of both sidechains.

### 7.2 Equations

Let `fs` be the sample rate and `T = 1/fs`. A one-pole with time constant `tau` uses
`a = 1 - exp(-T / tau)`, `y += a * (u - y)`.

**Attenuator (voltage divider)**, from the schematic values [2]:

```
R_series = 70.7e3            // R6 + R7
R_pot    = 100e3             // Gain pot
R_p      = (R_cell * R_pot) / (R_cell + R_pot)
A_raw    = R_p / (R_series + R_p)
A        = A_raw / A_dark    // normalised so PR = 0 gives unity
```

With `R_dark = 2 MΩ` and `R_min = 0.5 kΩ` the range is about 38 dB (**derived**),
matching the 40 dB specification. GR in dB is `-20 log10(A)`.

**Sidechain tap** (section 3.4):

```
s = y_att                              // Compress
s = 0.962 * y_att + 0.038 * x          // Limit  (R7 / (R6 + R7) = 0.038)
```

**Sidechain amplifier and shaping**:

```
v  = g_pr(PR) * s
v  = V_sat * tanh(v / V_sat)           // 6AQ5 runs out of swing; caps maximum light
v  = LowShelf(v; f_c = 1 kHz, gain_dB = -10 * (1 - r37))     // r37: 1 = flat (clockwise)
v  = Tilt(v)                           // fixed: about +4 dB per decade from 100 Hz to 6 kHz, flat above
```

The R37 depth (0 to -10 dB) and corner (about 1 kHz) follow [16] [55] and are
**estimates**; the divider formed by the 1 MΩ pot allows up to about -15 dB in principle
(**derived**). The tilt magnitude is an **estimate** chosen so that a 10 kHz sine
receives roughly 4 dB more GR than a 100 Hz sine at the same level for an 18 dB step
above onset, after PRR's observation [20].

**EL light** (Alfrey-Taylor, [39]):

```
u   += a_1ms * (|v| - u)                       // phosphor plus fast cell response; keeps 2f ripple at LF
L    = B0 * exp(-b / sqrt(max(u, 1e-6) / V_ref))    // b ≈ 3 (estimate)
```

`exp(-b/sqrt(V))` is zero-slope near zero (a soft threshold) and saturates at high drive,
which gives both the soft knee and the "gives in at extreme levels" behaviour described
in [17] [58]. `V_ref` is chosen so that the onset (1 dB GR) at PR 30 sits at 0 VU
(section 7.3).

**CdS cell with traps** (recommended). States: free carriers `n_f`, trapped carriers
`n_t` (0 to `N_t`).

```
gen      = k_gen * L^gamma                              // gamma ≈ 0.7  [36] [38]
tau_f    = tau_f0 / (1 + L / L_a)  if gen > n_f   else tau_r1    // attack faster in bright light
capture  = c * n_f * (1 - n_t / N_t)
detrap   = n_t / tau_t,   tau_t = tau_t0 * (1 + k_m * n_t / N_t)  // deeper traps empty slower
n_f     += T * ( (gen - n_f) / tau_f - capture + detrap )
n_t     += T * ( capture - detrap )
G        = 1 / R_dark + k_G * n_f
R_cell   = clamp(1 / G, R_min, R_dark)
```

Behaviour: on a short burst `n_f` rises within a few ms (closed-loop attack about
10 ms for moderate hits, tens of ms to full settling, [2] [29] [53]); when the light
stops, `n_f` decays with `tau_r1` (first stage, half the GR gone in 40-80 ms [2]) down to
the level sustained by `detrap`, which then decays with `tau_t`, 0.5 s when traps are
nearly empty and 5 s or more after long, bright exposure (the "memory" [2] [4]). Trap
filling is the only memory mechanism, which keeps the model interpretable and reproduces
"slower if the unit has either been in compression for a while, or the amount of
compression is large". [2]

Default constants (all **estimates** to be tuned against section 8; anchored values
are cited):

| Constant | Default | Anchor |
|---|---|---|
| `gamma` | 0.7 | CdS gamma 0.6-0.9 [36] [38] |
| `tau_f0` (attack, moderate light) | 12 ms | 10 ms spec [4] [5]; 15 ms learned [45] |
| `L_a` | 1.0 (light giving about 10 dB GR) | attack faster with light [36] [44] |
| `tau_r1` (fast release) | 60 ms | 40-80 ms [2]; 60 ms [4] |
| `tau_t0` (slow release, empty traps) | 0.5 s | 0.5 s minimum [2] |
| `k_m` | 9 | 5 s after heavy use [2]; longer tails reported [29] [32] |
| `c` (trap capture rate) | 1 / 300 ms | chosen so that 50 % of GR is in the slow pool after a 2 s tone (**estimate**) |
| `N_t` | 1.0 (normalised) | saturation gives the "memory" plateau |
| `R_dark`, `R_min` | 2 MΩ, 0.5 kΩ | >1 MΩ dark [21]; 0.68-2 kΩ lit [17] [19] |
| `b` (EL law) | 3 | shape only; Alfrey-Taylor [39] |
| `V_sat` | 8 × onset voltage | caps GR near 35-40 dB (**estimate**) |
| optional deep pool | tau 45 s, weight 0.05 | "few minutes to fully recover" [3]; 21 s to 99 % [29] |

Simpler alternative (if the trap model proves hard to tune): two conductance pools
`G_f` (one-pole, attack `tau_f`, release 60 ms) and `G_s` (fed by a fraction `phi(M)` of
the target, release `tau_s(M)`), with an exposure memory `M` (one-pole on `L`, 3 s up,
10 s down) that scales `phi` from 0.3 to 0.8 and `tau_s` from 0.5 s to 5 s. This is the
"two-stage plus memory" structure that Niviem, Neold and the UA text describe. [2] [60]
[62]

**Photocell distortion** (optional, small): `y_att *= 1 - kappa * (y_att / V_0)^2` with
`kappa` tuned for 1-2 % THD (third harmonic) at 6 dB GR and 0 VU, after [26] [37] [53].
The 2f ripple from the 1 ms smoothing already generates odd harmonics on bass at heavy
settings, so keep `kappa` modest.

**Make-up and tube stage** (Gain knob `p` in 0-1, unity at 0.32 after Waves' calibration
[55]):

```
gain_dB(p) = 40 * (1 + 2.02 * log10(max(p, 1e-4)))       // +40 dB at p = 1, 0 dB at p = 0.32
w  = y_att * 10^(gain_dB / 20)
z  = (tanh(k * (w + bias)) - tanh(k * bias)) / (k * (1 - tanh(k * bias)^2))   // soft, slightly asymmetric
```

Set `k` so that a sine at the +16 dBu equivalent gives about 0.75 % THD and +10 dBu about
0.3 % [2]; `bias` about 0.05 of the clip level adds the small second harmonic of a
single-ended stage. The output transformer is a 20 Hz first-order high-pass and a 30 kHz
first-order low-pass (-1 dB at 15 kHz, [5]); 45 kHz for the "reissue" voicing (±0.1 dB
[2]).

### 7.3 Control mappings and defaults

- **Gain**: 0-100 -> `gain_dB` above. Default 32 (unity).
- **Peak Reduction**: 0-100 -> sidechain gain `g_pr(PR) = 10^((G0 + 0.55 * PR) / 20)`.
  Choose `G0` and `V_ref` so that, with a 1 kHz sine in Compress: PR 0 gives no GR up to
  the +16 dBu equivalent; PR 30 gives 1 dB GR at 0 VU; PR 50 gives about 5 dB GR at 0 VU
  (the "sweet spot" [72], "most common range 30 to 50" [55]); PR 100 gives 1 dB GR about
  40 dB below 0 VU and 30 dB or more GR at 0 VU ("threshold ... up to -40 dB" [3]). The
  0.55 dB per unit slope follows the fitted threshold span in [46]. Default 40.
- **Limit / Compress**: `beta` 0.038 or 0. Default Compress.
- **Meter**: GR / +10 / +4. Default GR. 0 VU = -18 dBFS (+4 dBu) by default, with a
  calibration setting (-12 dBFS as in UA's plug-ins [3]).
- **R37 Emphasis**: 0-1, default 1 (flat).
- **Hidden trims for the tribute**: meter zero (±2 dB, with a slow drift option [29]),
  stereo balance (±3 dB sidechain offset [2]), cell age (scales `k_gen` down to 0.2,
  after Waves' "up to 80 % less compression" [55]), hum (50 / 60 Hz at about -80 dBFS,
  after [55]), and a "cell speed" choice (Silver / Gray / LA-2: scale `tau_f0`, `tau_r1`
  and `tau_t0` by about 0.7 / 1.0 / 1.6, **estimate** after [3] [4]).

### 7.4 Compress versus Limit in the model

Only `beta` changes. At low GR the two modes coincide because `0.038 * x` is small next
to `0.962 * y_att`; as GR grows the feed-forward term dominates and the effective ratio
rises steeply toward limiting. Expected static behaviour (**derived**, to be verified in
tests): within 0.3 dB of each other below 3 dB GR; Limit gives about 1.5 to 2 times the
GR of Compress at 15-20 dB of Compress GR. If the ruffrecords reading is preferred, also
lower `R_min` from 0.7 kΩ to 0.5 kΩ in Limit. [19]

### 7.5 Meter

- **Ballistics**: ANSI C16.5-1942 / IEC 60268-17 VU: 99 % of reading in 300 ms with 1
  to 1.5 % overshoot [75] [76]. A second-order system with natural frequency about
  15.5 rad/s (about 2.5 Hz) and damping ratio about 0.8 meets this (**derived**; verify
  in the unit test).
- **GR mode**: input to the ballistics is the GR in dB from the meter cell (identical
  state; optionally a second cell with ±0.5 dB mismatch). Needle rests at 0 VU and moves
  left. Because the cell is slow, the meter mostly shows the first release stage, then
  creeps back toward zero over seconds (a famous LA-2A observation [1] [3]).
- **Output modes**: average-responding rectifier on the output, 0 VU at +4 or +10 dBu
  equivalent.

### 7.6 Sample rate, oversampling, stereo

- All time constants are in seconds and coefficients are recomputed per sample rate; the
  model is rate-independent to first order because every state is a one-pole or an Euler
  step with `tau >= 1 ms >> T`.
- Oversampling is not needed for the gain loop (its bandwidth is far below 1 kHz). The
  tube stage and the optional cell cubic are low-order and gentle at nominal level, so
  aliasing stays below about -90 dB; offer 2x oversampling of the make-up stage only for
  users who drive it hard.
- **Stereo**: the hardware bridges the two sidechains through the R3 trimmers so both
  panels see a blend of both channels [2]. Implement "Link" as one shared cell driven by
  `v = 0.5 * (v_L + v_R)` before rectification (hardware-like, polarity sensitive) with an
  alternative of summing the two lights (`L = L_L + L_R`) for anti-phase-safe behaviour.
  Unlinked: two independent cells with a per-channel balance trim.
- Denormals: flush-to-zero in the audio thread and clamp `n_f`, `n_t` and `u` at `1e-9`
  when below it, so silence after heavy compression cannot leave denormal states.

---

## 8. Test plan

Each test drives the DSP core offline at 44.1, 48 and 96 kHz and compares against
tolerances; where hardware evidence exists the expected value is cited.

1. **Bypass transparency**: PR 0, Gain 32, hard bypass on -> output equals input to
   1e-6. With the tube stage on and PR 0, THD at 0 VU below 0.3 % (nominal 0.25-0.35 %
   [2] [5]) and response flat within ±1 dB from 30 Hz to 15 kHz (+0/-1 dB [5]).
2. **Steady-state GR versus PR and level**: 1 kHz sines from -40 to +16 dB relative to
   0 VU, PR in {0, 20, 30, 40, 50, 60, 80, 100}; assert monotonic GR in both variables,
   PR 30 onset (1 dB) within ±1 dB of 0 VU, PR 50 about 5 dB GR at 0 VU (±1.5 dB), maximum
   GR between 30 and 40 dB at PR 100 and +16 dB ([2] [3]).
3. **Ratio and knee**: from the curves above, the local slope in the 6-20 dB GR region
   is 2.5:1 to 4.5:1 in Compress (3:1 [3] [4], 4:1 [8] [46]); Limit differs from Compress
   by under 0.3 dB below 3 dB GR and exceeds Compress by at least 4 dB of GR at the
   input level that gives 20 dB in Compress (section 7.4, [29] [30]). Knee is soft: the
   second derivative of the I/O curve never exceeds a set bound (no corner).
4. **Attack**: -24 dB tone stepping to -3 dB (Canopus' test [29]) at PR 50: 63 % of the
   final GR reached in 5 to 60 ms; stabilisation to 1 % of the GR derivative in 20 to
   100 ms (33-81 ms measured [53]); a 6 dB step attacks slower than an 18 dB step (level
   dependence [36] [44]).
5. **Two-stage release**: after a 2 s burst at 10 dB GR, GR falls to 50 % in 40-120 ms
   (40-80 ms [2]) and to 10 % in 0.5-3 s.
6. **Memory**: compare a 100 ms burst with a 20 s burst at 20 dB GR; the time to 90 %
   recovery after the long burst is at least twice that after the short one, and the
   long-burst tail to 99 % is at least 5 s (5 s [2]; 21 s reported [29]). Repeating a
   passage without resetting state changes the GR trace (Waves' warning [55]).
7. **Frequency dependence**: equal-level 100 Hz and 10 kHz sines at PR 50: the 10 kHz
   GR exceeds the 100 Hz GR by 2 to 6 dB for an 18 dB step above onset ([20]); with R37 at
   0 the 100 Hz GR drops by 6 to 12 dB equivalent while 10 kHz changes by under 1 dB
   (10 dB shelf [16]).
8. **Distortion during GR**: 1 kHz at 0 VU with 6 dB GR: THD 0.8 to 4 % with the third
   harmonic above the second ([26] [53]); 60 Hz at PR 90: visible odd harmonics from
   cell ripple ([3]).
9. **Meter ballistics**: step of a steady tone: 99 % of reading at 300 ± 30 ms, overshoot
   1.0-1.5 % [75] [76]; GR meter matches the attenuator within 0.5 dB at steady state.
10. **Stereo link**: identical mono material on both channels gives identical GR;
    linked GR on a hard-panned burst is between the two unlinked values.
11. **Numerical hygiene**: 10 minutes of silence after 30 s of 30 dB GR: no denormals
    (check the FTZ flag and time per block), no NaN or infinity for inputs of ±10.0, DC,
    and digital silence; state values stay within bounds; sample-rate consistency: GR
    envelopes at 44.1, 48 and 96 kHz agree within 0.2 dB.
12. **Performance**: per-sample cost bounded (two `exp`/`pow` calls, a `tanh`, a dozen
    multiply-adds); a 10-minute stereo render at 96 kHz completes in under 5 % of real
    time on the reference machine.

---

## 9. References

1. Universal Audio Support, "A History of the Teletronix LA-2A Leveling Amplifier".
   https://help.uaudio.com/hc/en-us/articles/215779663-A-History-of-the-Teletronix-LA-2A-Leveling-Amplifier
2. Universal Audio, "Model LA-2A Leveling Amplifier, User's Guide" (reissue manual,
   rev. 1.3, 2000; specifications, calibration, theory of operation, schematic figures).
   https://media.uaudio.com/assetlibrary/l/a/la-2a_manual.pdf
3. Universal Audio Support, "Teletronix LA-2A Leveler Collection Manual".
   https://help.uaudio.com/hc/en-us/articles/4419496124180-Teletronix-LA-2A-Leveler-Collection-Manual
4. Universal Audio blog, "Tips & Tricks: Teletronix LA-2A Classic Leveler Plug-In
   Collection". https://www.uaudio.com/blogs/ua/la-2a-collection-tips-tricks
5. Universal Audio, "Teletronix LA-2A Classic Leveling Amplifier" (hardware product page
   with original specifications).
   https://www.uaudio.com/products/teletronix-la-2a-classic-leveling-amplifier
6. Universal Audio blog, "Teletronix LA-2A: Why It's the World's Most Famous Audio
   Compressor".
   https://www.uaudio.com/blogs/ua/why-is-the-teletronix-la-2a-the-worlds-most-famous-audio-compressor
7. Wikipedia, "LA-2A Leveling Amplifier". https://en.wikipedia.org/wiki/LA-2A_Leveling_Amplifier
8. M. Cooper, "Universal Audio LA-2A", Mix, November 2000.
   https://www.mixonline.com/technology/universal-audio-la-2a-370120
9. Sound On Sound, "Classic Compressors". https://www.soundonsound.com/techniques/classic-compressors
10. J. Saunders, "Gear Icons: LA2A", Mixdown, 2024. https://mixdownmag.com.au/features/gear-icons-la2a/
11. MusicRadar, "The producer's guide to the Teletronix LA-2A".
    https://www.musicradar.com/news/producers-guide-la-2a
12. Teletronix, "Model LA-2A Leveling Amplifier" instruction manual, circa 1966 (Library
    of Congress scan; not fetched directly, cited through [53]).
    https://tile.loc.gov/storage-services/master/mbrs/recording_preservation/manuals/Teletronix%20Model%20LA-2A%20Leveling%20Amplifier.pdf
13. Sweetwater inSync, "LA-2A Emphasis Control". https://www.sweetwater.com/insync/la-2a-emphasis-control/
14. Puremix, "F. Reid Shippen: Universal Audio LA-2A emphasis".
    https://www.puremix.com/blog/f-reid-shippen-universal-audio-la-2a-emphasis
15. Puremix, "Fab Dupont: how optical compressors work".
    https://www.puremix.com/blog/fab-dupont-how-optical-compressors-work
16. mix:analog blog, "LA-2A Dual Compressor Tutorial". https://blog.mixanalog.com/dual-la2a-compressor
17. KVR Audio forum, "Help Understanding the LA-2A" (posts by aciddose, ghettosynth).
    https://www.kvraudio.com/forum/viewtopic.php?t=514970
18. GroupDIY forum, "Student looking for help understanding how the LA-2A works".
    https://groupdiy.com/threads/student-looking-for-help-understanding-how-the-la-2a-works.86194/
19. GroupDIY forum, "LA2A feedforward/feedback design?".
    https://groupdiy.com/threads/la2a-feedforward-feedback-design.50129/
20. GroupDIY forum, "LA-2A Theory of Operation - EL Panel Characteristics?".
    https://groupdiy.com/threads/la-2a-theory-of-operation-el-panel-characteristics.37472/
21. GroupDIY forum, "T4B Photocells". https://groupdiy.com/threads/t4b-photocells.64961/
22. GroupDIY forum, "T4 optical attenuator for Teletronix LA-2A ... and how the
    compressor works".
    https://groupdiy.com/threads/t4-optical-attenuator-for-teletronix-la-2a-and-how-the-compressors-works.80210/
23. GroupDIY forum, "New Kenetek T4B Opto-Attenuators for your LA-2A, LA-3A and similar
    builds" (posts by Kenetek).
    https://groupdiy.com/threads/new-kenetek-t4b-opto-attenuators-for-your-la-2a-la-3a-and-similar-builds.72265/
24. GroupDIY forum, "Adding Attack and Release to an LA-2A?".
    https://groupdiy.com/threads/adding-attack-and-release-to-an-la-2a.85440/
25. GroupDIY forum, "DIY T4B, matching EL Panels and Cells".
    https://groupdiy.com/threads/diy-t4b-matching-el-panels-and-cells.44255/
26. GroupDIY forum, "LA-2A THD rise with compression action?".
    https://groupdiy.com/threads/la-2a-thd-rise-with-compression-action.41169/
27. GroupDIY forum, "Choosing Photoresistors for optical compressor".
    https://groupdiy.com/threads/choosing-photoresistors-for-optical-compressor.57521/
28. GroupDIY forum, "LA-2a T4b Comparison". https://groupdiy.com/threads/la-2a-t4b-comparison.40833/
29. Gearspace forum, "LA2A limit mode attack time?" (measurement by Canopus; typo note by
    bewareofdogs). https://gearspace.com/board/high-end/367872-la2a-limit-mode-attack-time.html
30. Gearspace forum, "LA2A Limit vs Compress".
    https://gearspace.com/board/so-much-gear-so-little-time/776746-la2a-limit-vs-compress.html
31. I. Sobczyk, IGS Audio, "T4Bx photocell: learn how 'the sound' is created" (PDF).
    https://igsaudio.com/wp-content/uploads/2024/05/qXh3AjYh.pdf
32. ProReplicas, "T4B Opto-Attenuator". https://www.proreplicas.com/t4b_cell.html
33. AudioScape Engineering, "Why do we make our own T4B Optical Cells?".
    https://www.audio-scape.com/news/t4b
34. DIYRE wiki, "Kenetek T4B Opto-Attenuator Cell".
    https://wiki.diyrecordingequipment.com/projects/kenetek-t4b-opto-attenuator-cell/
35. Clairex Corp., photoconductive cells catalogue (archive.org full text; CL-505L and
    CL-705 data).
    https://archive.org/stream/TNM_Clairex_photoconductive_cells_-_Clairex_Corp_20180514_0017/TNM_Clairex_photoconductive_cells_-_Clairex_Corp_20180514_0017_djvu.txt
36. PerkinElmer Optoelectronics, "Photoconductive Cells" application note (mirror).
    https://cdn-learn.adafruit.com/assets/assets/000/010/129/original/APP_PhotocellIntroduction.pdf
37. Wikipedia, "Resistive opto-isolator". https://en.wikipedia.org/wiki/Resistive_opto-isolator
38. GL5528 CdS photoconductive cell datasheet (gamma definition).
    https://pi.gate.ac.uk/pages/airpi-files/PD0001.pdf
39. "ZnS:Sm and ZnS:Cu,Sm electroluminescent phosphors", Bulletin of Materials Science
    (Alfrey-Taylor brightness relation).
    https://www.ias.ac.in/article/fulltext/boms/005/05/0405-0415
40. "On the relationship between photocurrent decay time and trap distribution in CdS
    and CdSe photoconductors", Solid-State Electronics.
    https://www.sciencedirect.com/science/article/abs/pii/0038110165900055
41. F. Eichas, U. Zölzer, "Modeling of an Optocoupler-Based Audio Dynamic Range Control
    Circuit", Proc. SPIE 9948, 2016.
    https://www.hsu-hh.de/ant/wp-content/uploads/sites/699/2017/10/Eichas-Modeling-of-an-optocoupler-based-audio-dynamic-range-control-circuit-99480W.pdf
42. F. Eichas, E. Gerat, U. Zölzer, "Virtual Analog Modeling of Dynamic Range Compression
    Systems", AES Convention 142, 2017. https://aes.org/publications/elibrary-page/?id=18628
43. J. Parker, S. D'Angelo, "A Digital Model of the Buchla Lowpass-Gate", DAFx-13.
    https://dafx.de/paper-archive/2013/papers/44.dafx2013_submission_56.pdf
44. J. Najnudel, R. Müller, T. Hélie, D. Roze, "Power-Balanced Dynamic Modeling of
    Vactrols: Application to a VTL5C3/2", DAFx23.
    https://www.dafx.de/paper-archive/2023/DAFx23_paper_50.pdf
45. A. Wright, V. Välimäki, "Grey-Box Modelling of Dynamic Range Compression",
    DAFx20in22. https://www.dafx.de/paper-archive/2022/papers/DAFx20in22_paper_35.pdf
    (code: https://github.com/Alec-Wright/GreyBoxDRC)
46. C.-Y. Yu, G. Fazekas, "Sound Matching an Analogue Levelling Amplifier Using the
    Newton-Raphson Method", AES AIMLA 2025. https://arxiv.org/pdf/2509.10706
47. S. H. Hawley, B. Colburn, S. I. Mimilakis, "SignalTrain: Profiling Audio Compressors
    with Deep Neural Networks", AES Convention 147, 2019. https://arxiv.org/abs/1905.11928
    (dataset: https://zenodo.org/records/3348083)
48. C. J. Steinmetz, J. D. Reiss, "Efficient neural networks for real-time modeling of
    analog dynamic range compression", AES Convention 152, 2022.
    https://ar5iv.labs.arxiv.org/html/2102.06200
49. R. Simionato, S. Fasciani, "Deep Learning Conditioned Modeling of Optical
    Compression", DAFx20in22. https://dafx2020.mdw.ac.at/proceedings/papers/DAFx20in22_paper_6.pdf
50. R. Simionato, S. Fasciani, "Fully Conditioned and Low-latency Black-box Modeling of
    Analog Compression", DAFx23. https://www.dafx.de/paper-archive/2023/DAFx23_paper_10.pdf
51. R. Simionato, S. Fasciani, "Modeling Time-Variant Responses of Optical Compressors
    with Selective State Space Models", JAES 73(3), 2025 / arXiv.
    https://arxiv.org/html/2408.12549
52. "Modeling Analog Dynamic Range Compressors using Deep Learning and State-space
    Models", arXiv 2024. https://arxiv.org/html/2403.16331
53. A. Moore, "Objective Analysis and Perceptual Evaluation of LA-2A Compressors and
    Vocal Recordings", University of Huddersfield (accepted manuscript, 2025).
    https://pure.hud.ac.uk/ws/portalfiles/portal/140787498/AAM.pdf
54. D. Giannoulis, M. Massberg, J. D. Reiss, "Digital Dynamic Range Compressor Design:
    A Tutorial and Analysis", JAES 60(6), 2012. https://www.aes.org/e-lib/download.cfm?ID=16354
55. Waves, "CLA-2A User Guide". https://assets.wavescdn.com/pdf/plugins/cla-2a-compressor-limiter.pdf
56. Native Instruments, "Vintage Compressors VC 2A Manual".
    https://files.plugin-alliance.com/products/native-instruments-vc-2a/native-instruments-vc-2a_manual.pdf
57. IK Multimedia, "White 2A Leveling Amplifier". https://www.ikmultimedia.com/products/trwhite2a/
58. Cakewalk, "CA-2A T-Type Leveling Amplifier". https://legacy.cakewalk.com/Products/CA-2A
59. Black Rooster Audio, "VLA-2A". https://blackroosteraudio.com/en/products/vla-2a
60. Niviem, "OPT4 - LA-2A Opto Compressor Plugin". https://niviem.net/products/opt4/
61. Bedroom Producers Blog, "LALA Is A FREE LA-2A Limiting Amplifier VST By Analog
    Obsession". https://bedroomproducersblog.com/2020/05/07/audio-obsession-lala/
62. Midi Audio Expert, "Neold U2A Plugin Review".
    https://midi-audio-expert.com/2025/11/04/neold-u2a-plugin-review/
63. Arturia, "Comp TUBE-STA" overview.
    https://www.arturia.com/products/software-effects/comp-tubesta/overview
64. Softube, "OPTO Compressor" user manual. https://www.softube.com/user-manuals/opto-compressor
65. G. Scott, "Model Behaviour: LA-2A Emulations vs Hardware", Attack Magazine.
    https://www.attackmagazine.com/features/long-read/model-behaviour-uad-la-2a-emulations-hardware/
66. Tape Op, "CLA-2A & CLA-76 Compressor/Limiter Plug-Ins" review.
    https://tapeop.com/reviews/gear/106/cla-2a-cla-76-compressorlimiter-plug-ins
67. Sound On Sound, "Plug-in Modelling: How Industry Experts Do It".
    https://www.soundonsound.com/techniques/plug-in-modelling-how-industry-experts-do-it
68. Tape Op, "Dr. David Berners: Behind The Gear with Universal Audio".
    https://tapeop.com/interviews/69/dr-david-berners/
69. Sound On Sound, "Warm Audio WA-2A" review. https://www.soundonsound.com/reviews/warm-audio-wa-2a
70. Warm Audio, "WA-2A" manual (ManualsLib listing).
    https://www.manualslib.com/manual/3725660/Warm-Audio-Wa-2a.html
71. Klark Teknik, "2A-KT" user manual (manua.ls). https://www.manua.ls/klark-teknik/2a-kt/manual
72. note.com, "Operation Report for the Teletronix LA-2A Tube Compressor".
    https://note.com/miyamoto_2025/n/n839b9acb2f4b?hl=en
73. Sonarworks blog, "Get the Most From Optical Compressors".
    https://www.sonarworks.com/blog/learn/get-the-most-from-optical-compressors
74. Vintage Digital, "The Legendary Teletronix LA-2A Leveling Amplifier from 1962".
    https://www.vintagedigital.com.au/teletronix-la-2a/
75. Prism Sound glossary, "VU Meter" (ANSI C16.5-1942 ballistics).
    http://www.prismsound.com/define.php?term=VU_Meter
76. EDN, "Analog VU Meters & Quick Pointers". https://www.edn.com/analog-vu-meters-quick-pointers/

[1]: https://help.uaudio.com/hc/en-us/articles/215779663-A-History-of-the-Teletronix-LA-2A-Leveling-Amplifier
[2]: https://media.uaudio.com/assetlibrary/l/a/la-2a_manual.pdf
[3]: https://help.uaudio.com/hc/en-us/articles/4419496124180-Teletronix-LA-2A-Leveler-Collection-Manual
[4]: https://www.uaudio.com/blogs/ua/la-2a-collection-tips-tricks
[5]: https://www.uaudio.com/products/teletronix-la-2a-classic-leveling-amplifier
[6]: https://www.uaudio.com/blogs/ua/why-is-the-teletronix-la-2a-the-worlds-most-famous-audio-compressor
[7]: https://en.wikipedia.org/wiki/LA-2A_Leveling_Amplifier
[8]: https://www.mixonline.com/technology/universal-audio-la-2a-370120
[9]: https://www.soundonsound.com/techniques/classic-compressors
[10]: https://mixdownmag.com.au/features/gear-icons-la2a/
[11]: https://www.musicradar.com/news/producers-guide-la-2a
[12]: https://tile.loc.gov/storage-services/master/mbrs/recording_preservation/manuals/Teletronix%20Model%20LA-2A%20Leveling%20Amplifier.pdf
[13]: https://www.sweetwater.com/insync/la-2a-emphasis-control/
[14]: https://www.puremix.com/blog/f-reid-shippen-universal-audio-la-2a-emphasis
[15]: https://www.puremix.com/blog/fab-dupont-how-optical-compressors-work
[16]: https://blog.mixanalog.com/dual-la2a-compressor
[17]: https://www.kvraudio.com/forum/viewtopic.php?t=514970
[18]: https://groupdiy.com/threads/student-looking-for-help-understanding-how-the-la-2a-works.86194/
[19]: https://groupdiy.com/threads/la2a-feedforward-feedback-design.50129/
[20]: https://groupdiy.com/threads/la-2a-theory-of-operation-el-panel-characteristics.37472/
[21]: https://groupdiy.com/threads/t4b-photocells.64961/
[22]: https://groupdiy.com/threads/t4-optical-attenuator-for-teletronix-la-2a-and-how-the-compressors-works.80210/
[23]: https://groupdiy.com/threads/new-kenetek-t4b-opto-attenuators-for-your-la-2a-la-3a-and-similar-builds.72265/
[24]: https://groupdiy.com/threads/adding-attack-and-release-to-an-la-2a.85440/
[25]: https://groupdiy.com/threads/diy-t4b-matching-el-panels-and-cells.44255/
[26]: https://groupdiy.com/threads/la-2a-thd-rise-with-compression-action.41169/
[27]: https://groupdiy.com/threads/choosing-photoresistors-for-optical-compressor.57521/
[28]: https://groupdiy.com/threads/la-2a-t4b-comparison.40833/
[29]: https://gearspace.com/board/high-end/367872-la2a-limit-mode-attack-time.html
[30]: https://gearspace.com/board/so-much-gear-so-little-time/776746-la2a-limit-vs-compress.html
[31]: https://igsaudio.com/wp-content/uploads/2024/05/qXh3AjYh.pdf
[32]: https://www.proreplicas.com/t4b_cell.html
[33]: https://www.audio-scape.com/news/t4b
[34]: https://wiki.diyrecordingequipment.com/projects/kenetek-t4b-opto-attenuator-cell/
[35]: https://archive.org/stream/TNM_Clairex_photoconductive_cells_-_Clairex_Corp_20180514_0017/TNM_Clairex_photoconductive_cells_-_Clairex_Corp_20180514_0017_djvu.txt
[36]: https://cdn-learn.adafruit.com/assets/assets/000/010/129/original/APP_PhotocellIntroduction.pdf
[37]: https://en.wikipedia.org/wiki/Resistive_opto-isolator
[38]: https://pi.gate.ac.uk/pages/airpi-files/PD0001.pdf
[39]: https://www.ias.ac.in/article/fulltext/boms/005/05/0405-0415
[40]: https://www.sciencedirect.com/science/article/abs/pii/0038110165900055
[41]: https://www.hsu-hh.de/ant/wp-content/uploads/sites/699/2017/10/Eichas-Modeling-of-an-optocoupler-based-audio-dynamic-range-control-circuit-99480W.pdf
[42]: https://aes.org/publications/elibrary-page/?id=18628
[43]: https://dafx.de/paper-archive/2013/papers/44.dafx2013_submission_56.pdf
[44]: https://www.dafx.de/paper-archive/2023/DAFx23_paper_50.pdf
[45]: https://www.dafx.de/paper-archive/2022/papers/DAFx20in22_paper_35.pdf
[46]: https://arxiv.org/pdf/2509.10706
[47]: https://arxiv.org/abs/1905.11928
[48]: https://ar5iv.labs.arxiv.org/html/2102.06200
[49]: https://dafx2020.mdw.ac.at/proceedings/papers/DAFx20in22_paper_6.pdf
[50]: https://www.dafx.de/paper-archive/2023/DAFx23_paper_10.pdf
[51]: https://arxiv.org/html/2408.12549
[52]: https://arxiv.org/html/2403.16331
[53]: https://pure.hud.ac.uk/ws/portalfiles/portal/140787498/AAM.pdf
[54]: https://www.aes.org/e-lib/download.cfm?ID=16354
[55]: https://assets.wavescdn.com/pdf/plugins/cla-2a-compressor-limiter.pdf
[56]: https://files.plugin-alliance.com/products/native-instruments-vc-2a/native-instruments-vc-2a_manual.pdf
[57]: https://www.ikmultimedia.com/products/trwhite2a/
[58]: https://legacy.cakewalk.com/Products/CA-2A
[59]: https://blackroosteraudio.com/en/products/vla-2a
[60]: https://niviem.net/products/opt4/
[61]: https://bedroomproducersblog.com/2020/05/07/audio-obsession-lala/
[62]: https://midi-audio-expert.com/2025/11/04/neold-u2a-plugin-review/
[63]: https://www.arturia.com/products/software-effects/comp-tubesta/overview
[64]: https://www.softube.com/user-manuals/opto-compressor
[65]: https://www.attackmagazine.com/features/long-read/model-behaviour-uad-la-2a-emulations-hardware/
[66]: https://tapeop.com/reviews/gear/106/cla-2a-cla-76-compressorlimiter-plug-ins
[67]: https://www.soundonsound.com/techniques/plug-in-modelling-how-industry-experts-do-it
[68]: https://tapeop.com/interviews/69/dr-david-berners/
[69]: https://www.soundonsound.com/reviews/warm-audio-wa-2a
[70]: https://www.manualslib.com/manual/3725660/Warm-Audio-Wa-2a.html
[71]: https://www.manua.ls/klark-teknik/2a-kt/manual
[72]: https://note.com/miyamoto_2025/n/n839b9acb2f4b?hl=en
[73]: https://www.sonarworks.com/blog/learn/get-the-most-from-optical-compressors
[74]: https://www.vintagedigital.com.au/teletronix-la-2a/
[75]: http://www.prismsound.com/define.php?term=VU_Meter
[76]: https://www.edn.com/analog-vu-meters-quick-pointers/

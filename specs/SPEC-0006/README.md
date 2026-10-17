# SPEC-0006: A modular kit beside the sequencer

| Field | Value |
|---|---|
| Spec | SPEC-0006 |
| Revision | r0, draft, 2026-10-17 |
| Stage | **Planning only.** This pull request adds a spec, a function reference and a journal entry. It changes no code, contract, fixture or threshold, and it asks for nothing to be built until the owner answers section 7 and approves a slice in their own words |
| Risk tier | **Low** for this pull request (`specs/`, `journal/`). Slices K0 to K5 are **Medium** (`apps/web/`, Forge). K6 (the patching surface) touches the design system and is **High** and gets its own spec first. None of them touches `contracts/`, the engine or the ABI |
| Owner and approver | `@beaudesign` |
| Task | `WENGE-0035` |
| Extends | SPEC-0004 (the `Voice` registry and the rig; S0 is pull request #73); SPEC-0004 D1 (the page's sound is a feature), whose ADR is ADR-0014 (#70, numbered 0014 because the owner's engineering spec reserves 0013) |
| Companion | `specs/SPEC-0006/modules.md`: what each module does, the numbers, what is not published |
| Sister specs | SPEC-0004 (built-in voices), SPEC-0005 (a chord style). Independent of SPEC-0005. Overlaps SPEC-0004 and decision D8 settles how |
| Base | `main` at `c079f8f` |

## 1. The ask

Recorded here because chat is not a source of truth. On Sat 2026-10-17 at 14:56 London the owner wrote (the dashes are written as hyphens here):

> https://modulargrid.net/e/racks/view/1010952 this is the modular system I want to replicate almost sound for sound and function - specifically the oscillator sound or the verbos, make noise and , Orthogonal Devices ER-101 which is for creating longer 64 patterns - research the functions of all of these then start to put together an implementation and rebuild plan. Think of it as a modular kit which sits aside the sequencer as sources of filters, synthesis and oscillation - then we'll add reverbs and build some pedals as well (original

**The reading.**

- "The modular system" is the rack at that address: 16 entries, of which 13 are modules that do something (section 3). "Sound for sound and function" is read as **function for function everywhere, and sound as close as published information and the owner's ear can take it**. Section 3 says what that does and does not promise.
- "Sits aside the sequencer as sources of filters, synthesis and oscillation" is read as: the kit is a second thing the page can do, beside the Octopus, fed by the Octopus's notes and clock, and making sound. It does not replace the Octopus, change what it sends to MIDI, or turn the page into a MIDI device.
- "Research ... then start to put together an implementation and rebuild plan" is read as: this pull request is the research and the plan. Nothing is built from it until it is approved.
- "(original" ends mid-word. It is read as "(original designs)": the **reverbs and pedals are new designs of our own**, not copies of named products. If the owner meant something else, section 7 asks.
- The "verbos" is read as the one Verbos module in the rack, the **Harmonic Oscillator**. SPEC-0004's S7 "Complex" is the other Verbos module (two oscillators with FM, AM and wave-shaping). Q-S6-1 asks which is wanted; the default builds the Harmonic Oscillator first because it is in the rack.
- "Longer 64 patterns" is read in section 3 (the ER-101 has no 64-step limit; 64 is the Octopus's own chain length). Q-S6-3 asks.

## 2. What the page has today (read from the code, not recalled)

- **The monitor** (`apps/web/src/monitor.ts`) turns the engine's note events into audio, in the AudioWorklet: 24 voices, a sine with two harmonics, three drums, a `tanh` master, mono. It reads the events and changes none. With SPEC-0004's S0 (pull request #73, not yet merged) the tone is the voice `Plain` behind a `Voice` interface, and a rig measures any voice (pitch, aliasing, stability, cost).
- **The clock the engine has.** The engine counts 48 ticks to the quarter note and sends a MIDI clock every 2 ticks (24 per quarter) as realtime events (`TICKS_PER_CLOCK = 2`), with Start, Continue and Stop. It also gives the page its tick position every render. So a clock for the kit already exists and does not need to be invented.
- **Track chains.** The engine already has chained tracks (`ChainBase`, `chain_members`), which is how the Octopus makes 32, 64 or 128 steps from tracks of 16.
- **What is not there:** a signal graph, anything with a control voltage, any effect, any stereo, any patch storage. SPEC-0004's section 9 puts effects, patch storage and stereo out of scope; this spec is where they come in, and decision D1 says so on purpose.

## 3. What is being replicated, and what may be taken

### The rack

The page at that address loads its module list by script, so I took the list from the data the page embeds (saved 2026-10-17; the rack is called "Caterina Barbieri's case (updated)", last modified 2026-04-02):

| Slice | Working name | In the rack | Kind |
|---|---|---|---|
| K0 | Notes | Doepfer A-190-3 | MIDI to CV and gate |
| K1 | Partials | Verbos Harmonic Oscillator | eight sine partials and a mixer |
| K1 | Twin | Make Noise DPO | two oscillators, FM, sync, timbre, fold |
| K2 | Function | Make Noise Maths | function generator and signal combiner |
| K2 | Gate pair | Make Noise Optomix | dual lowpass gate |
| K2 | Quad level | Doepfer A-135-1 | four VCAs and a mixer |
| K2 | Ladder | Doepfer A-105 | four-pole lowpass |
| K3 | Chance | Make Noise Wogglebug | random voltages |
| K3 | Clock bank | ALM Busy Circuits Pamela's NEW Workout | clock generator and modulator |
| K4 | Index | Orthogonal Devices ER-101 | indexed four-track sequencer |
| K5 | String | 2hp Pluck | plucked string |
| K5 | Pitch echo | Make Noise Echophon | pitch-shifting echo |
| K5 | Space | Make Noise Erbe-Verb | reverb |

The rack also holds a CV bus case and two 1 HP blanks. They do nothing to the signal. The "working names" are ours and changeable (D2); `modules.md` has a sheet for each.

### Findings that change the brief

1. **The rack is not the 2018 setup.** The best-known write-up of the artist's rig (2018) lists a different set (Tempi, a Doepfer ADSR and LFO; no Optomix, A-190-3, Pluck or Pamela's). The 2026 rack is followed. The 2018 extras are a parts bin (Q-S6-4).
2. **The Verbos module in the rack is the Harmonic Oscillator**, with eight sine outputs and a mixer, not the Complex Oscillator that SPEC-0004's S7 is about.
3. **The ER-101 has no 64-step limit.** It has up to 100 steps in a pattern, 100 patterns on a track and 2,000 steps in all. The only 64 found is the Octopus's chain builder, and the engine already chains tracks. What the ER-101 gives that the Octopus does not is *indexed* values, per-step durations and ratchets, and tracks of unrelated length that phase against each other.
4. **The Optomix has no mode switch.** Its gate, lowpass and amplifier behaviours come from what is patched.
5. **No schematic was found for any of the Make Noise, Verbos or Orthogonal modules.** The makers publish controls, ranges and some behaviour. The sound of a module comes from circuit detail they do not give.

### How "sound for sound" is handled

It cannot be promised from published documents, and this spec does not pretend to. Three layers, kept apart on purpose:

- **Documented** (a range, a count, an inequality, a table): becomes a test, with the source named. Most of `modules.md` is this.
- **Not published** (a fold curve, a vactrol's speed, a distribution): becomes a **named constant with a status** (`documented`, `by-ear`, or `measured`) in a truth file next to the module, set by listening and measuring, and changed only with a journal line (K-12). This is where a module's character lives, and the plan says so rather than hiding it.
- **The owner's ear** is the last test. As in SPEC-0004, an offline renderer writes WAVs so the owner can listen on a phone before a slice is approved. If the owner has recordings of their **own** hardware, those can be distilled into numbers (a spectrum, an envelope), never loaded into context and never copied (CLAUDE.md rule 7's reasoning for photographs applies to audio).
  Commercial recordings of the artist are not asked for and are not used.

### What may be taken (clean room)

- **From manuals and product pages:** functions, ranges and counts, in our own words. Nothing is copied.
- **From the open literature:** algorithms (Karplus and Strong, Jaffe and Smith, Jot and Chaigne, Dattorro, Parker and D'Angelo, Huovilainen, Zavalishin), each tied to a paper that a reviewer can open.
- **Not taken:** panel art, logos, names, firmware, any recording, any schematic (none are public, and none are looked for). The makers' names appear **in this folder only**; K-10 scans `apps/` and `crates/` for them. The ER-101's file format is not used; patches are our own.
- The legal question of copying a *function* is not a lawyer's answer here. Functions are not what a copyright or a licence of these makers protects, but names, panel art and firmware are, so the plan takes none of those. If the owner wants a legal view before shipping, that is the owner's to get.

## 4. How it is built, and what the slices build

### 4.1 Architecture

**A kit is a patch of modules.** A patch is data: a list of module instances (a kind and its knob values) and a list of cables (an output of one module to an input of another). It is saved as versioned JSON, compiled once off the audio thread, and run in the AudioWorklet in the same place and under the same rules as `monitor.ts` (no allocation per sample, no `Math.random`, nothing that differs between two runs).

**Volts.** Every jack carries a number that means volts, as in the hardware: audio about plus or minus 5 V, control voltages plus or minus 10 V or 0 to 8 V, gates 0 V or 5 to 10 V, pitch at 1 V per octave. Using volts means a test can state a manual's number as it is written (the ER-101's 8.192 V, the Doepfer's 5 V span), and a module's behaviour is read against its manual without a conversion table.
At the output the sum is scaled so that 10 V peak is 1.0 and the existing `tanh` master follows (the scale is set by listening in K0, D4).

**One sample at a time, in order.** Modules run once per sample in a fixed order from a topological sort of the cables. A cable that closes a loop is delayed by exactly one sample, as in other software racks and as an analog loop's own latency would be; this is tested (K-3). Modules that need it oversample inside themselves (the fold, the FM bus, the ladder). Knobs are smoothed so a move never clicks.

**Normalling.** Many modules assume a signal when nothing is plugged in (Function's channels 2 and 3 sit at +10 V and +5 V; Gate pair's Control and Damp at +8 V). An input can name a normal source; plugging a cable uses the cable, and unplugging goes back. Tested (K-2).

**How the sequencer feeds the kit.** Two modules touch the page's events. **Notes** (A-190-3) turns the engine's note events into a pitch voltage, a gate, a velocity voltage and a controller voltage, by the table in `modules.md` and the hardware's four key-assignment modes. **Clock bank** and **Index** take the engine's clock pulses and transport (Start, Stop, Continue, and the tick position) as their clock and reset.
Nothing flows the other way: the kit sends nothing to the engine, so the events, the ABI and MIDI out are byte for byte what they are (H-12 of SPEC-0004 still holds with the kit on).

**The kit is one more voice.** SPEC-0004 S0 makes a `Voice` registry whose entries create one note at a time. A patch is not per-note; it is a persistent graph whose output is one stream. K0 therefore adds a second kind of voice to S0's interface, a **bus voice**, which the monitor feeds with the events and which returns one sample per frame to add to the mix (the drums stay as they are). `Plain` and every note voice are untouched. The kit is the bus voice `Rack`, chosen with the same strip button.

**Polyphony.** The hardware bridge is mono and the Octopus plays chords of up to seven notes. Notes ships as the hardware is (mono, four assignment modes) and adds an **optional poly mode of up to four note outputs**, oldest taken first as the monitor does (D7). Each output is patched to its own oscillator, as in hardware, so the cost of four notes is four oscillators the user chose, not a hidden copy of the whole rack.

**Where it runs.** TypeScript in the worklet (D5), in `apps/web/src/kit/`, sharing its DSP parts with SPEC-0004's voices: one band-limited oscillator core, one ladder, one low-pass gate (D8). No part is written twice. If the cost bound (K-6) cannot be met, the revisit condition in D5 moves the core into the existing wasm build by ADR.

**Effects.** A reverb or a pedal is a module with an audio input and output, in the same format, so it can sit inside a patch (Space and Pitch echo already do) or at the end of a **chain** after the rack's output (a pedalboard). One format, one rig, one set of cost tests.

**Surfaces.** In order: (1) a `Rack` button in the strip that steps through **preset patches** (as SPEC-0004's `Voice` button does for voices), so the owner can hear the kit on the page from K0; (2) WAV renders of each preset for listening on a phone; (3) later, a patching surface with module panels and cables (K6). Panels for the kit are **original designs in the design system's language** (no accent colour, only a LED is saturated), not copies of any maker's panel (D12). The Octopus remains the only pixel-exact panel.

### 4.2 The slices

| Slice | Content | Tier |
|---|---|---|
| **K0** | **The foundation.** ADR for D1 (the page becomes a kit host); the kit core (volts, jacks, patch format, compile, normalling, one-sample loops, knob smoothing); the module test rig (S0's rig, extended); the bus-voice kind added to S0's `Voice`; **Notes**, **Clock** and **Out**; **Quad level** (so there is something to hear with); one preset patch and its WAV; the `Rack` button | Medium |
| **K1** | **The oscillators.** **Partials** and **Twin**, with S1's band-limited parts; two preset patches (a drone from Partials, an FM voice from Twin) | Medium |
| **K2** | **The shapers.** **Function**, **Gate pair**, **Ladder** (S2 and S4's parts); the parts bin if wanted (an envelope, an LFO); presets that play the demo's bass and lead | Medium |
| **K3** | **Chance** and **Clock bank**, and a divider if wanted; a generative preset | Medium |
| **K4** | **Index.** K4a is the engine of the sequencer (tracks, patterns, steps, tables, divide and multiply, ratchet, smoothing, loop points, reset) driven by a patch file or typed pattern; K4b is its editor, a grid for up to 100 steps, which is a UI and waits for K6's rules | Medium (K4a); High (K4b) |
| **K5** | **String**, **Pitch echo**, **Space** | Medium |
| **K6** | **The rack surface**: panels, cables, patch save and load, in the design system's language. Its own spec first (it adds a surface and tokens) | High |
| **K7** | **Reverbs**: small room, plate, hall, shimmer and reverse of section 4.3 as modules and as end-of-chain effects; the spring last, after a patent view (D14) | Medium |
| **K8** | **Pedals**: the original designs of section 4.3, cheapest first (Shake, Drive, Sweep, Squeeze), then Chorus, Tape, Fold, Shift, Ring and Crush, one slice each after the first; the chain that orders them | Medium |
| **K9** | **The listening pack**: a preset for each module and a render of each for the owner | Medium |

K0 alone makes sound on the page and proves the whole path. The order after it follows what was named: the oscillators first (K1), then the Make Noise shapers (K2, K3), then the sequencer (K4), then time and space (K5), then the original reverbs and pedals (K7, K8). The owner can reorder (Q-S6-7).

### 4.3 Reverbs and pedals (original)

Read from the open literature on 2026-10-17. Every algorithm below is tied to a paper a reviewer can open; none is copied from a product, a plugin or a firmware. Cost classes are estimates to be replaced by K-6's first measurement: **cheap** is under about 20 multiply-adds a sample, **medium** is 20 to 200, **heavy** is more, or an FFT, or oversampling by 8 or more.

**Reverbs (K7), original in setting and name, standard in method.** Space (K5) is already a feedback network; these are the others.

| Design | Method and paper | Cost | What a test pins |
|---|---|---|---|
| **Small room** | Parallel feedback combs into series allpasses, with a lowpass in each comb loop (Schroeder 1962; Moorer, "About this reverberation business", CMJ 1979). The Freeverb arrangement (8 combs, 4 allpasses) is a public-domain description of the same idea | cheap to medium | comb gain `10^(-3 d / T)` gives the stated decay time; the high band decays faster than the low when damped; echo density rises with time |
| **Plate** | Dattorro, "Effect design, part 1", JAES 1997: a mono input, pre-delay, four input allpasses, a figure-eight tank with modulated allpasses, damping and decay gains, and several output taps per side. Published for 29,761 Hz, so delays are rescaled to the context's rate. The paper's text and figures belong to the AES, so the build uses the numbers only | medium | tail length is monotone in decay; output stays finite at maximum decay; left and right tails are decorrelated; modulation sidebands sit near 1 Hz |
| **Hall (a network)** | Delay lines through a unitary matrix, per-line gain from the delay and the decay time, a lowpass per line for frequency-dependent decay (Gerzon 1971; Stautner and Puckette, CMJ 1982; Jot and Chaigne 1991) | medium | unity gain with a unitary matrix conserves energy; decay time within 5 percent of the setting; the high-to-low ratio matches |
| **Shimmer** | A pitch shifter in the feedback path of a long reverb (the practice described by the Valhalla notes; Dal Santo et al., "Shimmer reverberation with nonlinear feedback delay networks", DAFx-26, 2026) | set by the shifter | a 440 Hz sine gives 880 and 1760 Hz partials on successive recirculations; loop gain under 1 decays monotonically; no DC or sub-bass builds up |
| **Reverse** | Windowed reverse playback of a rolling buffer before the reverb. No canonical paper: the thresholds come from our own measurement | cheap | the level rises about linearly in decibels over the buffer length then ends; the onset delay equals the buffer length; the seams do not click |
| **Spring** | Two recirculating loops of stretched first-order allpass chains give the chirped echoes (Välimäki, Parker and Abel, JAES 2010; Parker, "Efficient dispersion generation structures for spring reverb emulation", EURASIP JASP 2011; Parker and Bilbao, DAFx-09) | heavy | group delay varies with frequency and peaks at the cutoff; the echo spacing equals the loop delay; the chirps recur with decaying level. **Built last, and only after a patent view (D14)** |

Order: small room first (the light setting for a phone), then plate, hall, shimmer and reverse, spring last.

**Pedals (K8): original designs by function, each with a plain name of its own** (working names, changeable: Drive, Shake, Sweep, Chorus, Tape, Squeeze, Ring, Fold, Shift, Crush). They are not copies of a product; the method is the published one.

| Pedal | Method and paper | Cost | What a test pins |
|---|---|---|---|
| **Shake** (tremolo, vibrato) | Amplitude modulation, or a modulated delay of 0 to 5 ms (Dattorro, "Effect design, part 2"); the optical, harmonic and bias families are described in the amp literature | cheap | sidebands at plus or minus the LFO rate with level m over 2; band gains in antiphase for the harmonic type; vibrato deviation is 2 pi times the rate times the peak excursion |
| **Drive** (overdrive, fuzz) | Conditioning filter, memoryless nonlinearity, equalisation (Yeh, Abel and Smith, DAFx-07; the K-method, IEEE TASLP 2010); first-order antiderivative anti-aliasing (Parker, Zavalishin and Le Bivic, DAFx-16; Vicanek's note) | cheap to medium | a symmetric shaper gives odd harmonics only and bias adds even ones; aliasing of a high sine at full drive is under a bound from the first measurement; no DC |
| **Sweep** (phaser, flanger) | Cascaded first-order allpasses, two stages to a notch (Smith, ICMC 1984; Huovilainen, DAFx-05); a modulated comb of 0 to 10 ms for the flanger, whose feedback does not come from the modulated tap (Dattorro part 2) | cheap, medium if nonlinear | the notch count is half the stages; notches sweep with the LFO; the allpass magnitude is flat; comb spacing is the inverse of the delay |
| **Chorus** and a bucket-brigade **delay** | Delay of N over 2 f_clk driven by a clock, input and output lowpass filters, a 2:1 compander, a cubic shaper (Raffel and Smith, DAFx-10; Huovilainen, DAFx-05) | medium | delay equals N over 2 f_clk; the lowpass tracks the clock; the compander slope is 2:1. The Raffel page states no code licence, so only the paper is used |
| **Tape** | Cheap: a `tanh` saturator, a modulated delay and head-bump and high-frequency-loss filters. Physical (heavy): Jiles-Atherton hysteresis at 16 times oversampling (Chowdhury, DAFx-19; Zavalishin and Parker, DAFx-18) | medium (cheap model) | weighted wow and flutter near 0.02 percent (pro open reel) to 0.08 percent (a good cassette) per the standard measure; odd harmonics grow with drive; a delay-time change glides the pitch |
| **Squeeze** (compressor) | Feed-forward, log-domain detector after the gain computer, a decoupled smooth peak detector, a soft knee (Giannoulis, Massberg and Reiss, JAES 2012) | cheap | the static curve including the knee; steady-state gain reduction; attack and release constants |
| **Ring** (ring modulator) | Four parallel static diode curves (Parker, DAFx-11), about 66 operations a sample; the paper wants 8 to 32 times oversampling for wideband input | cheap at 1 times, heavy as published | a 50 Hz modulator on a 1500 Hz carrier gives sidebands at 1450 and 1550 Hz and a suppressed carrier |
| **Fold** (wavefolder) | The Lockhart and Serge folds with a Lambert W solution and first-order anti-aliasing (Esqueda, Pontynen, Parker and Bilbao, Applied Sciences 2017); the Buchla fold with band-limited ramps (DAFx-17). Also the engine of Twin's fold | cheap to medium | output is bounded; harmonics grow with gain; offset adds even harmonics |
| **Shift** (pitch shifter, granular delay) | Two-tap crossfade (cheap), granular delay (medium) or a phase vocoder (heavy) (Dattorro part 2; Laroche and Dolson, WASPAA 1999; Truax on granular synthesis). Also the engine of Pitch echo | cheap to heavy | output pitch equals the ratio within a few cents; crossfade windows sum to 1; ripple at the window rate |
| **Crush** (bitcrusher) | Quantise to 2^N levels and sample-and-hold decimation. Aliasing is the effect, so no anti-aliasing | cheap | only 2^N output levels appear; full-scale signal to quantisation noise is about 6.02 N plus 1.76 dB; an unfiltered sine folds to the expected alias |

**Oversampling in a browser.** Needed, or replaced by anti-derivative anti-aliasing, for the clipper, the fuzz, the folds, the diode ring modulator, the tape hysteresis and any nonlinearity inside the shimmer loop. Not needed for reverbs, delays, chorus and flanger with good interpolation, the compressor or the bitcrusher. The default is first-order anti-derivative anti-aliasing plus 2 to 4 times polyphase half-band oversampling (which Vicanek and Kahles et al. measured), with the cheaper `x over sqrt(1 plus x squared)` shaper where `tanh` is too dear. All of it runs in preallocated `Float64Array`s with a guard against denormals in every feedback loop.

**Chain, not only modules.** Each effect also works at the end of the rack, in a chain the owner orders (a pedalboard), so a pedal can colour any sound the page makes. Its place in the chain is a value in the patch, tested like any other.

**Licences and patents (D14).** Algorithms and numbers from papers may be reimplemented; the text and figures of a paper may not be copied. No code is taken from any project, but the survey checked which are unsafe to read as a source: Surge XT, VCV Rack, ChowTape (AnalogTapeModel), Dragonfly Reverb and the Mutable Instruments AVR code are GPL-3.0 and are not read for this work; anything under the Hippocratic License is not used (SPEC-0004's reason). Product names (the reverb and pedal names of other makers) are not used. Patents, which are not legal advice: US 8,391,504 (a digital spring emulation using several propagation modes with a dispersive delay per mode) is listed as active to 2031 and may or may not reach a two-loop chirp design; US 10,019,980 (a modal reverb architecture with pitch processing) is listed as active to 2036 and is a different architecture from the shimmer above; US 8,204,240 (a constrained-routing feedback network) was not read and its status is unchecked; US 6,549,884 (phase-vocoder pitch shifting) is listed as expired in 2019.

## 5. Observables (written red first)

"Sounds like the hardware" is a taste judgement, and CLAUDE.md turns each of those into a measurement. Below, the rows that hold for the whole kit, then the smallest rows each module must have (each slice writes its full list red first, and a number marked "from the first measurement" is set by the first green run with a stated margin and only ever tightened).

### The kit

| # | Observable | Test |
|---|---|---|
| K-1 | **Pitch.** Every pitched module tracks 1 V per octave within 2 cents over 8 octaves at 44.1, 48 and 96 kHz (SPEC-0004's rig measurement) | `kit/pitch.test.ts` |
| K-2 | **Cables and normalling.** An output feeds many inputs; two cables never drive one input (the compile refuses); a normalled input uses its source until a cable arrives and goes back when it leaves | `kit/patch.test.ts` |
| K-3 | **Loops.** A feedback cable has exactly one sample of delay and nothing else changes; a three-module loop matches a hand-computed series | `kit/patch.test.ts` |
| K-4 | **Deterministic.** Two compiles of one patch, and two runs of one set of events, are equal sample for sample, including every seeded random stream | `kit/determinism.test.ts` |
| K-5 | **Stable.** 10,000 random patches (random modules, cables and knobs) at 44.1, 48 and 96 kHz: no NaN, no infinity, nothing above 1 after the master, and a patch with every gate low and no free-running source falls below an inaudible bound within 5 s | `kit/stability.test.ts` |
| K-6 | **Cost.** The reference rack (every module in section 3 running, four note outputs) renders 10 s of audio in under a share of real time in Node set from the first measurement (aimed at 40 percent), and no `process` function in `kit/` allocates (the SPEC-0004 H-13 text check, extended) | `kit/cost.test.ts` |
| K-7 | **Notes.** Note to volts is (note minus reference) over 12 with reference 36; a note outside the 5 V span is ignored; the gate is high from Note On to Note Off; the four assignment modes return to a held note on release; the poly mode gives up to four outputs, oldest taken | `kit/notes.test.ts` |
| K-8 | **Clock.** The engine's 24-per-quarter clock gives 24 pulses per quarter; Start, Stop and Continue give run and reset as the engine sends them; a stopped transport makes no pulse | `kit/clock.test.ts` |
| K-9 | **The page.** With `Rack` chosen: the events the worklet sends are byte for byte as with any voice (SPEC-0004 H-12); the drums are unchanged; the output differs from Plain and is audible (the AudioNode tap); the choice made before Play is kept | `kit.browser.test.ts` |
| K-10 | **No borrowed names.** None of the makers' or modules' names appears in `apps/`, `crates/`, the page or its assets. The list lives in the test and is only ever extended | `kit/names.test.ts` |
| K-11 | **Patches round-trip.** A patch saved and loaded is the same patch; an unknown module kind or a newer version is refused with a message and never partly loaded | `kit/patch.test.ts` |
| K-12 | **Provenance.** Every constant a module takes from a source or sets by ear is in that module's truth file with its status and source key; the code reads it from there; a test fails if a constant in code is not in the file, or if a `documented` value changes without the file saying why | `kit/truth.test.ts` |
| K-13 | **The strip keeps its rules.** The `Rack` button uses the existing tones and tokens and adds no colour, gradient, shadow or easing (`app_source_rules.rs`, `app-css.test.ts`) | existing gates |

### The smallest rows for each module

| Module | Rows (a sample; the slice writes the full list) |
|---|---|
| **Partials** | P-1 the eight outputs are at 1 to 8 times one frequency, locked, from one accumulator; P-2 the master is the exact sum of channel levels times partials; P-3 all levels at zero is silence; P-4 tilt at centre changes nothing; P-5 moving the window centre up raises the centroid; P-6 no partial above Nyquist is folded down; P-7 linear FM is linear in volts and exponential FM is octaves per volt |
| **Twin** | T-1 coarse 12 Hz to 6 kHz and fine 1.75 octaves; T-2 sync adds harmonics only when A is above B; T-3 lock pulls a near-integer ratio to the integer and leaves a far one; T-4 follow at minimum lags a pitch step, at maximum does not; T-5 fold is symmetric for a symmetric input and linear in its VCA region; T-6 a strike gate opens the fold and it decays; T-7 aliasing under a bound from the first measurement, aimed 50 dB below the fundamental; T-8 final stays within 10 Vpp |
| **Function** | F-1 SUM is exactly the sum and OR exactly the maximum; F-2 a trigger rises then falls, and retriggers in the fall and not the rise; F-3 a slew limiter never moves faster than its set rate and only slows; F-4 equal times on the linear curve cycle as a triangle within a bound; F-5 time to the midpoint orders log, linear, exponential; F-6 positive Both CV shortens; F-7 times from about 1 ms to about 25 min; F-8 the end-of-rise and end-of-cycle gates; F-9 the manual's example patches behave as it says |
| **Gate pair** | G-1 the rise is quicker than the fall by the ratio of the cited model, within 10 percent; G-2 level and brightness fall together; G-3 more Damp is a longer decay; G-4 a strike with nothing patched is a positive envelope; G-5 Sum out is exactly channel 1 plus channel 2 plus Aux |
| **Quad level, Ladder** | Q-1 gain is linear in the control from 0 to 5 V and zero is silence; Q-2 the output is the exact sum of the four; L-1 24 dB per octave within 2 dB; L-2 resonance rises monotonically and self-oscillates near the top with a bounded level; L-3 1 V per octave |
| **Chance** | C-1 Stepped changes only on a clock edge; C-2 Smooth lies between old and new and approaches the new; C-3 Woggle rings and decays; C-4 Disturb hold freezes every output and press forces a value; C-5 the clock range; C-6 a seeded run repeats |
| **Clock bank** | B-1 x4 gives four edges a beat and /3 one in three beats; B-2 width 50 percent at 120 BPM is a 250 ms high; B-3 a Euclidean output has the standard pattern; B-4 skip 0 passes all; B-5 delay with divisor 2 shifts alternate steps; B-6 a looped random output repeats exactly |
| **Index** | I-1 divide by 3 passes pulses 1, 4, 7; I-2 multiply by 4 gives four evenly spaced pulses in phase; I-3 duration 8, gate 2 with ratchet fires twice; I-4 gate 0 is silent and duration 0 skips; I-5 a reset returns every track to step 1; I-6 loop points loop; I-7 smoothing starts when the gate falls and ends at the step end; I-8 table index 12 of the twelve-tone table is exactly 1 V above index 0; I-9 two tracks of unrelated period phase and realign after their common multiple; I-10 a 100-step pattern plays (more than the 64 asked for) |
| **String** | S-1 the fifth trigger steals the oldest voice; S-2 older notes keep their pitch while the newest follows V/Oct; S-3 pitch within 2 cents over five octaves; S-4 Damp or Decay changed mid-note leave the sounding note unchanged |
| **Pitch echo** | E-1 echo time 7 ms to 1.7 s, exponential; E-2 the tempo ratio table; E-3 freeze makes the output independent of new input; E-4 the two feedback placements differ on repeats; E-5 the 40 kHz, 16 bit character option is off by default and exact when off |
| **Reverbs and pedals** | X-1 every effect's output is finite and bounded for 10,000 random settings; X-2 no DC from any effect at rest; X-3 a nonlinear stage's alias products at full drive are under a bound from the first measurement (aimed 50 dB below the fundamental) or the stage says it is meant to alias (Crush); X-4 a reverb at its top setting holds without growth for ten minutes of audio; X-5 tail length is monotone in the decay setting; the per-design rows are in section 4.3 |
| **Space** | V-1 decay time rises with Size; V-2 at the top of Decay the tail holds without growth for ten minutes of audio; V-3 pre-delay is independent of Size; V-4 Tilt does not change the decay; V-5 Speed does nothing at Depth centre; V-6 stable over 10,000 random settings |

**What cannot be measured:** whether it sounds like the hardware, or pleases the owner. K0's renders and each slice's renders exist so the owner can listen before the next slice is approved, and the first question of each slice is theirs.

## 6. Decisions for the owner (defaults are taken; change any)

| # | Decision | Default and why |
|---|---|---|
| D1 | The page becomes a host for a modular kit and for effects, beside the sequencer | **Yes.** It is what was asked. The Octopus stays as it is; the kit is audio the page makes and never MIDI. Recorded as an ADR in K0 (numbered when it is written), because it widens SPEC-0004's D1 from a sound to a product area |
| D2 | Names and clean room | **Working names** in `modules.md`, plain and descriptive. **No maker's name outside this folder** (K-10). Functions from manuals and papers; no panel art, firmware, schematic or recording |
| D3 | Fidelity | Documented behaviour is tested; unpublished behaviour is a named constant with a status, set by ear and measurement; the owner's ear decides (section 3). "Sound for sound" is the aim and is **not** claimed |
| D4 | Signal model | Volts; one sample at a time in sorted order; one-sample delay on loop-closing cables; oversampling inside the modules that need it; output scale 10 V peak to 1.0, then the existing master (set by listening in K0) |
| D5 | Where it runs | **TypeScript in the worklet**, as SPEC-0004's D12 and D13. **Revisit if** the reference rack cannot meet K-6 on the CI machine with the sharing of D8, or the owner wants the sound identical on a native build; that is an ADR moving the core into the existing wasm build |
| D6 | The ER-101 | **A module in the kit** (a second sequencer clocked by the engine), not a change to the Octopus, the engine or the ABI. Making the Octopus itself play longer or indexed patterns is a different piece of work and is not planned here |
| D7 | Polyphony | Notes is **mono as the hardware is**, with an optional poly mode of up to four outputs, each patched by hand. No automatic copy of the whole rack per note |
| D8 | Relation to SPEC-0004 | **One set of parts.** The oscillator core (S1), ladder (S2) and low-pass gate (S4) are written once and used by voices and modules. `Rack` is one more voice. SPEC-0004's S1 onward still waits for Q-S4-1; the parts are written to serve both, so nothing is built twice. Whether the voices later become presets of the kit is left open |
| D9 | Reverbs | **Algorithmic only** (feedback networks, plates, springs by waveguide models). No reverb built from recordings of real spaces |
| D10 | Pedals | **Original designs** by function, not copies of a product, each with a plain name of its own |
| D11 | The first rack | The current 13 modules. The parts bin (envelope, LFO, divider) only if asked (Q-S6-4) |
| D12 | Panels and the design system | The kit's panels are **original** and follow the design system (no accent colour, a LED is the only saturated colour). They are a new surface and need their own spec, the Panelwright and the Curator (K6). Until then the owner hears the kit through presets and renders |
| D13 | Surfaces in order | `Rack` button with presets (K0), WAVs (every slice), patching surface (K6), the Index editor (K4b). A patch can also be loaded as a file in K0 for the owner's own experiments |
| D14 | Licences and patents for reverbs and pedals | **Papers and numbers only; no code from any project; no GPL or Hippocratic-licensed source is read for this work.** The spring reverb is built last and only after the owner has a view on US 8,391,504 (section 4.3); if the view is no, it is dropped and nothing else changes. This is the survey's reading and not legal advice |

## 7. Questions only the owner can answer

- **Q-S6-1.** The rack has the Verbos **Harmonic Oscillator**; SPEC-0004's S7 is the **Complex Oscillator** (from the engineering spec, now attached as pull request #71). Which did "the verbos" mean? Default: the Harmonic Oscillator now (it is in the rack), and the Complex one stays S7 of SPEC-0004.
- **Q-S6-2.** "(original" ended mid-word. Default: original designs for the reverbs and pedals.
- **Q-S6-3.** "Longer 64 patterns": the ER-101 has up to 100 steps per pattern, and the 64 matches the Octopus's own chain builder, which the engine already has. Did you want the ER-101 module (default), longer or chained patterns on the Octopus itself, or both?
- **Q-S6-4.** Follow the 2026 rack as it stands (default), or the 2018 list too (Tempi, an ADSR, an LFO)?
- **Q-S6-5.** Do you have recordings of your **own** hardware that I can distil into numbers? Not needed; it would help the "by ear" constants. No third-party recordings are asked for or used.
- **Q-S6-6.** The rack names some module versions (Optomix black panel, Pamela's NEW Workout, ER-101 original flavor, Wogglebug 2014) and not others (A-135-1 version, Optomix revision). Default: the behaviour the current manuals describe.
- **Q-S6-7.** Order. Default: K0, then the oscillators (K1), as the oscillator sound was named first.
- **Q-S6-8.** Phone or desktop first for the patching surface (K6)? Default: phone, as the rest of the page.
- **Q-S6-9.** The spring reverb sits near a live patent (section 4.3, D14). Do you want a view on it before it is built, or should it be dropped from the plan? Default: built last, after a view; dropped if the view is no.

## 8. Risks

- **"Sound for sound" cannot be proven from public documents.** The nonlinear parts that make a Make Noise or Verbos module sound like itself (the fold, the vactrols, the "imperfect" sines) are not published. Mitigation: section 3's three layers, truth files with a status per constant (K-12), renders to listen to, and no claim beyond what is measured.
- **Cost on a phone.** A rack of 13 modules, with oversampled nonlinear stages and a reverb, is more work than 24 sine voices. K-6 gates it in Node; a phone is slower. Mitigation: oversampling only where needed, a lighter reverb setting, the bus voice cap, `Plain` as the fallback, and D5's revisit condition.
- **Scope.** Thirteen modules, a sequencer, a patching surface, reverbs and pedals is a long road. Each slice stands alone and ships sound; the order can change and slices can stop at any point.
- **A product widening.** The page has been a sequencer with a plain monitor. D1 changes that on purpose and needs the ADR before code.
- **Trade dress and names.** Mitigated by D2 and K-10, and by building none of the makers' panel art. A legal view is the owner's to get.
- **The patching surface is a large, new UI** and the design system has no cable or knob vocabulary. K6 is its own spec for that reason.
- **Manuals disagree with themselves** in several places (`modules.md` section 14). Each disagreement is a named choice with a test, so a measurement or the owner's ear can overturn it without surgery.
- **Source quality.** Several facts come from retailers or forums and are marked so. None is relied on without saying.
- **Oldest-note stealing and the Octopus's chords** can make a mono bridge sound wrong. The poly mode (D7) exists for that.
- **A patent or a licence on a reverb or pedal** (section 4.3, D14). The spring and a modal-pitch reverb are the two found. The plan takes methods from papers only, avoids the live ones until the owner has a view, and reads no GPL source. Not legal advice.
- **Browser differences.** `Math.sin` and `Math.exp` can differ in the last digit between engines, so equality tests run in Node and browser tests assert measured properties, as in SPEC-0004.

## 9. Not done

Everything in section 4.2 (nothing is built by this pull request). Also not done: a change to the Octopus, the engine, the events, the ABI, MIDI out or any contract; patch storage beyond a file; stereo (the output is mono, as today, until a module needs two channels, and Space's right output is summed); recording or bouncing; any panel art; and any change to SPEC-0004 or SPEC-0005. The shape player and the Complex voice stay where SPEC-0004 and SPEC-0005 put them.

**Numbering.** The owner's engineering spec (#71, section 3.1) expects the shape player's spec to be `specs/SPEC-0006/` "if free". It was not: this is SPEC-0006, opened first (#74), so the shape player's spec takes the next free number when it is written. Its ADR, 0013, stays reserved for it (the sound ADR is 0014).

**The `Voice` interface.** K0 adds a second kind of voice (a bus voice) to S0's `Voice` interface. The engineering spec asks to be consulted before S0's interface is touched (its section 3.5). This spec is that consultation: K0 does not start, and the interface is not changed, until the owner has approved K0 in their own words with that change in view.

## 10. Sources

Read on 2026-10-17. Types: **M** maker (manual or product page), **P** press or review, **R** retailer, **F** forum, **D** database, **A** academic. Tagged because the weight of a fact depends on it.

**The rack and the artist.** ModularGrid rack 1010952 (**D**, `https://modulargrid.net/e/racks/view/1010952`, saved list); the 2018 live-setup article (**P**, `https://synthanatomy.com/2018/03/take-a-closer-look-caterina-barbieris-live-setup.html`); Sound On Sound 2017 interview (**P**, `https://www.soundonsound.com/node/4917144`).

**Verbos Electronics.** Harmonic Oscillator page and tech card (**M**, `https://verboselectronics.com/harmonic-oscillator/`, `https://verboselectronics.com/tech-cards/Harmonic+Oscillator+Tech+Card.pdf`); the designer's 2011 post on harmonic oscillators (**M**, blog, `https://buchlatech.blogspot.com/2011/10/harmonic-oscillators.html`); retailer pages (**R**, Perfect Circuit, Signal Sounds, Clockface, Thomann); VCV Rack and Elektronauts threads (**F**).

**Make Noise.** Manuals for Maths, Wogglebug, DPO, Echophon, Erbe-Verb (**M**, `https://www.makenoise-manuals.com/<module>/<module>-manual.pdf`); the Optomix revision 2 manual (**M**, `https://www.makenoisemusic.com/wp-content/uploads/2024/03/optomixrev2manual.pdf`); product pages (**M**, `https://www.makenoisemusic.com/modules/<module>/`); Tempi manual (**M**); retailer pages (**R**); interviews with Tom Erbe, who wrote the Echophon and Erbe-Verb DSP (**P**, Cycling '74 and Perfect Circuit); an essay on the rotating-head pitch changer (**P**, `https://soundandscience.net/contributor-essays/anton-springer-and-the-time-and-pitch-regulator/`).

**Orthogonal Devices.** ER-101 manual v2.09 and ER-102 manual v2.04 (**M**, `https://docs.orthogonaldevices.com/images/er-101-user-manual-f2.09.pdf`, `.../er-102-manual-v2.04.pdf`); their changelogs (**M**, `https://github.com/odevices/er-101/blob/main/CHANGELOG.md`, `.../er-102/...`); a review (**P**, Synthtopia 2018).

**Doepfer.** A-105, A-135, A-140, A-147 manuals and pages (**M**, `https://doepfer.de/a100_man/`); A-190-3 page and the A-190-2 manual (**M**, `https://doepfer.de/a1903.htm`, `https://doepfer.de/a100_man/A190_2_man.pdf`).

**2hp, ALM Busy Circuits.** Pluck manual and page (**M**, `https://www.twohp.com/modules/pluck`); Pamela's NEW Workout manual and page (**M**, `https://assets.busycircuits.com/docs/alm017-manual.pdf`); a Sound On Sound review (**P**).

**The Octopus.** The genoQs functional manual v0.9 (**M**, third-party host, `https://www.analoguehaven.com/genoqs-machines/octopus/storyboard.pdf`) and the Sound On Sound review (**P**, `https://www.soundonsound.com/reviews/genoqs-octopus`).

**Papers.** Karplus and Strong, "Digital synthesis of plucked-string and drum timbres", Computer Music Journal 7(2), 1983; Jaffe and Smith, "Extensions of the Karplus-Strong plucked-string algorithm", CMJ 7(2), 1983; Parker and D'Angelo, "A digital model of the Buchla lowpass-gate", DAFx-13; Huovilainen, DAFx-04; Zavalishin, "The Art of VA Filter Design"; Välimäki and Huovilainen, IEEE SPM 2007; Jot and Chaigne, "Digital delay networks for designing artificial reverberators", AES 90th Convention, 1991; Dattorro, "Effect design, part 1: reverberator and other filters", JAES 45(9), 1997; a 2023 DAFx paper on vactrol response (**A**, `https://www.dafx.de/paper-archive/2023/DAFx23_paper_50.pdf`). The papers for reverbs and pedals are those cited in section 4.3 (Schroeder 1962; Moorer, CMJ 1979; Gerzon 1971; Stautner and Puckette, CMJ 1982; Välimäki, Parker and Abel, JAES 2010; Parker, EURASIP JASP 2011; Dal Santo et al., DAFx-26; Raffel and Smith, DAFx-10; Huovilainen, DAFx-05; Chowdhury, DAFx-19; Zavalishin and Parker, DAFx-18; Smith, ICMC 1984; Parker, DAFx-11; Yeh, Abel and Smith, DAFx-07 and TASLP 2010; Parker, Zavalishin and Le Bivic, DAFx-16; Esqueda et al., Applied Sciences 2017 and DAFx-17; Giannoulis, Massberg and Reiss, JAES 2012; Laroche and Dolson, WASPAA 1999; Vicanek's note on anti-aliased distortion; Kahles et al., JAES 2019), all free to read or found through the survey of 2026-10-17, and each is pinned again in the slice that builds it. Patent status from Google Patents, a non-legal line.

**This repository.** `apps/web/src/monitor.ts`, `abi.ts`, `worklet.ts`; `docs/03-sequencer-core.md` (track chains); SPEC-0004 and ADR-0014; the engineering spec in pull request #71 (`handoffs/eng-spec-shape-player-and-complex-voice-v2.md`).

## 11. Approval record

Not yet approved. Awaiting `@beaudesign`. Merging is the owner's act; the agent merges nothing. A merge is not an approval: approval is the owner's own words, written here with the date.

Nothing the owner has said so far approves this spec: the 14:56 message above is a request for research and a plan, which this is. Approval would name a slice (for example "build K0"), and the agent will write down the words, the time, the reading and what the reading does not reach, as SPEC-0004's section 11 does.

## 12. Revision history

| Rev | Date | What changed |
|---|---|---|
| r0 | 2026-10-17 | The spec: the ask, the rack (16 entries, 13 modules), the findings that change the brief, architecture, slices K0 to K9, observables K-1 to K-13 and module rows, section 4.3 (reverbs and pedals from the literature, with licence and patent cautions), decisions D1 to D14, questions Q-S6-1 to Q-S6-9, risks, and `modules.md` |

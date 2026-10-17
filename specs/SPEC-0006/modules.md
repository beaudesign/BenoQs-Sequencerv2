# SPEC-0006 reference: what each module does

Companion to `README.md`. Read on 2026-10-17 from manufacturers' manuals and product pages, with reviews and forums used only to flag what is
unconfirmed. This file states **functions and numbers**, in our own words. It copies no manual text, schematic, panel art, firmware or recording
(this plan looks for no schematic: none was found for the Make Noise, Verbos or Orthogonal modules, and one is not needed to state what a module does).

How to read a sheet:

- **Working name** is what our version is called in the page. It is original, descriptive and changeable (decision D2). The maker's name appears
  in this folder and nowhere in `apps/`, `crates/` or anything the page shows.
- **Documented** is what a source states, with the source key from the end of this file. **Testable** is the part a test can pin.
  **Not published** is what no source gives; for those the first green run sets a number (marked "from the first measurement" in the README's
  observables) and the owner's ear decides the rest. A maker not publishing a thing is a finding, not a gap to fill by guessing quietly.
- Where two sources disagree, both are given and the disagreement is listed in section 14 so a measurement can settle it.

## The rack

ModularGrid rack 1010952, "Caterina Barbieri's case (updated)", last modified 2026-04-02 [MG]. Its saved page lists 16 entries: a Make Noise CV bus case,
Maths, Wogglebug (2014), DPO, Echophon, Erbe-Verb, Optomix, two 1 HP blanks; Orthogonal Devices ER-101; Doepfer A-135-1, A-105, A-190-3;
Verbos Harmonic Oscillator; 2hp Pluck; ALM Busy Circuits Pamela's NEW Workout. A 2018 article about her live setup [SA] lists a different set
(it adds Tempi, an A-140 envelope and an A-147 LFO, and has none of Optomix, A-190-3, Pluck or Pamela's NEW Workout); this plan follows the **current** rack and
treats the 2018 extras as a parts bin (section 12). In a 2017 interview she calls the ER-101 the centre of her setup [SOS-I]; whose rack
it is changes nothing the page needs to know.

| # | In the rack | Working name | Kind | Slice |
|---|---|---|---|---|
| 1 | Verbos Harmonic Oscillator | **Partials** | oscillator, eight sine partials and a mixer | K1 |
| 2 | Make Noise DPO | **Twin** | two oscillators, FM, sync, timbre and fold | K1 |
| 3 | Make Noise Maths | **Function** | four-channel function generator and signal combiner | K2 |
| 4 | Make Noise Optomix | **Gate pair** | dual lowpass gate (vactrol model) | K2 |
| 5 | Doepfer A-135-1 | **Quad level** | four VCAs summed to a mixer | K2 |
| 6 | Doepfer A-105 | **Ladder** | four-pole lowpass with resonance | K2 |
| 7 | Make Noise Wogglebug | **Chance** | stepped, smooth and ringing random voltages | K3 |
| 8 | ALM Busy Circuits Pamela's NEW Workout | **Clock bank** | clock generator and modulator | K3 |
| 9 | Orthogonal Devices ER-101 | **Index** | indexed four-track CV and gate sequencer | K4 |
| 10 | Doepfer A-190-3 | **Notes** | MIDI to CV and gate | K0 |
| 11 | 2hp Pluck | **String** | plucked string | K5 |
| 12 | Make Noise Echophon | **Pitch echo** | pitch-shifting delay | K5 |
| 13 | Make Noise Erbe-Verb | **Space** | reverb | K5 |
| - | CV bus case, two 1 HP blanks | none | power and filler | the case is the page's `Rack` itself |

---

## 1. Partials (Verbos Harmonic Oscillator, 32 HP)

**Role.** One pitch, many sines. A single oscillator whose first eight harmonics are separate outputs and are also mixed by a voltage-controlled
mixer that can be swept like a filter. It is the module the owner named first ("the oscillator sound").

**Documented** [VH-M, VH-B, VH-R]
- One discrete, analog, triangle-core oscillator, 3 Hz to 20 kHz. Outputs: triangle, square, saw, and **eight sine outputs for harmonics 1 to 8** of the
  one pitch (retailers describe them as the fundamental plus seven overtones).
- Pitch inputs: a calibrated 1 V/oct; an exponential CV input with a reversing attenuator; a linear FM input with an attenuator.
- The mixer: one slider and one CV input for each of the eight harmonics, summing to a master output. **Width** and **center** move a window across
  the eight harmonics like a band-pass filter, each with a reversing attenuator and a CV input. **Tilt** leans the balance to the high or low harmonics
  (centred is no effect). One LED per channel shows its level.
- A 2011 post by the designer describes an earlier prototype: one core feeding a waveshaper that makes ten sines; he dropped a first version that used
  distorted CMOS op-amps because it sounded poor [VH-B]. It does not say how the production unit makes its sines.

**Testable.** The eight outputs sit at 1 to 8 times one frequency, locked (an integer ratio, not a measured one). All eight sliders at zero is silence.
The master output is the sum of the partials, each scaled by its channel level, so doubling one level doubles that partial. Tilt at centre leaves the mix
unchanged. Moving the window's centre up raises the spectral centroid. Linear FM changes frequency by an amount proportional to the FM voltage; the exponential
input changes it by octaves per volt.

**Not published.** CV ranges and scales; the slider law (linear or exponential); the shape and edges of the window; tracking accuracy and drift;
whether there are coarse and fine knobs (one retailer says so, the manual does not); the **phase** of each partial against the fundamental; and how pure the
sines are. Two forum posts say the sines are "imperfect", from a video of a scope; that is the single most useful clue about the *sound* and it is unconfirmed [VH-F].

**Our approach.** One phase accumulator drives all eight sines, so the ratios are exact; partials that would pass Nyquist are faded out and never folded
(H-3's rule). The "imperfect" character is a small, seeded, parameterised distortion of each partial that starts at zero and is set by ear (decision D3);
it is deterministic. Phases start at zero until a measurement says otherwise.

## 2. Twin (Make Noise DPO, 28 HP)

**Role.** Two triangle-core oscillators with two kinds of FM, weak and hard sync, and a timbre section that shapes and folds the second oscillator.
The owner's second named oscillator.

**Documented** [DPO-M]
- Both oscillators: coarse tune 9.5 octaves (12 Hz to 6 kHz), fine tune 1.75 octaves, 1 V/oct best over plus or minus 5 V.
- Oscillator A: triangle (10 Vpp), saw (9 Vpp, made from the triangle, with a stronger fundamental than a plain saw) and sine (10 Vpp). A button steps its core
  mode: standard; **Lock** (a very weak sync that pulls A toward integer relationships with B); **Sync** (hard sync to B, needs A above B); **LFO**.
- Oscillator B: sine (10 Vpp), an asymmetric square (9 Vpp, hard edged, not AC coupled) and the **Final** output after the timbre section. An external lock input gives a weak,
  phase-reversed lock to an outside hard-edged signal.
- **Follow** sets how closely A tracks B's pitch CV; less than full lags, giving a transient detune.
- FM bus: an internal sine runs both ways between A and B, with an index, per-destination attenuators and both linear and exponential FM for each. Patching a jack cuts
  the internal route. Past about 5 o'clock on index with attenuators over about 80 percent the bus overdrives.
- Timbre on B, fed by A's sine unless an external source is patched: **Shape** morphs sine, spike and a "glitched" triangle, with amplitude modulation and a phase
  reversal around noon; **Angle** tilts the signal into the fold; **Fold** acts as a plain VCA over roughly its first 30 percent, then folds, adding low harmonics. A **Strike**
  gate of 8 to 10 V briefly opens the fold fully. Maker states the gain cells are vactrols (slow, organic response) and that the routing is in the 259 style, with a fold
  like the 259 and Music Easel timbre circuits.

**Testable.** Coarse range and fine range; sync adds harmonics only when A is above B; Lock pulls a near-integer ratio onto the integer and leaves a distant one alone; Follow
at minimum lags B's pitch step and at maximum does not; Fold below its VCA region is linear; folding a symmetric signal gives a symmetric result; a Strike gate gives a transient
fold opening that decays; the final output stays inside 10 Vpp.

**Not published.** The fold's transfer curve; the vactrol time constants (so the speed of Shape, Fold and Strike); the FM overdrive law; tracking accuracy and drift; the
exact weak-lock strength. All of these decide how it *sounds*. They are set by ear and kept as named constants in a `twin.truth.json`-style file with their provenance
(README section 5, K-12).

**Our approach.** Phase-accumulator triangle cores, band-limited steps (the SPEC-0004 S1 parts) for saw, square and sync, and 2 to 4 times oversampling around the fold and the FM
bus. The vactrol is modelled as a one-pole smoother with different attack and release constants (decision D3 says how those are chosen).

## 3. Function (Make Noise Maths, 20 HP)

**Role.** An analog-computer-style module: two function generators (envelope, slew limiter, LFO, clock divider, comparator), two attenuverted offsets, and mixing buses. The workhorse of the rack.

**Documented** [MA-M]
- Channels 1 and 4: DC-coupled signal in (plus or minus 10 V), trigger in, cycle button and cycle gate in (at least 2.5 V), **Rise** and **Fall** with linear CV inputs (plus or minus 8 V; positive
  lengthens), a **Both** CV input that is exponential (plus or minus 8 V; positive shortens the whole function), and **Vari-Response**, a shared curve control running from logarithmic through
  linear (a tick mark) and exponential to hyper-exponential. Outputs: a Unity out and a Variable out behind an attenuverter; plugging the Variable out removes that channel from the buses. An
  end-of-rise gate on channel 1 and end-of-cycle gate on channel 4 (0 or 10 V).
- Channels 2 and 3: DC-coupled inputs normalled to +10 V and +5 V, each through an attenuverter, so each gives an offset or an amplified or inverted signal.
- Buses: SUM (plus or minus 10 V), its inverse, and OR (an analog maximum, 0 to 10 V, which rectifies).
- Time range about 25 minutes to 1 kHz (slower needs a CV offset).
- A trigger gives a 0 to 10 V rise then fall with no sustain; it can retrigger during the fall and not during the rise; if the rise is longer than the clock period it divides the clock. A gate at the
  signal input gives attack, sustain and release. As a slew limiter it only slows a signal, never speeds it: Rise limits upward slew and Fall downward. Linear curve with Rise equal to Fall cycles as a triangle.
  The manual's patch examples (peak detector, envelope follower, ADSR from two channels and the OR bus, bouncing ball, flip-flop, full-wave rectifier, clock divider) are ready-made regression patches.

**Testable.** Every sentence above that has a number or an inequality. The OR bus is exactly the maximum of its inputs. SUM is exactly the sum. A linear, equal-time cycle is a triangle within a small
bound. A slew limiter's output never moves faster than its set rate. The envelope-follower and flip-flop patches behave as the manual says.

**Not published.** The curve law of Vari-Response; the exponential law of the Both CV (volts per octave of time); the exact end-of-cycle gate timing; the circuit and its component values; an inconsistency
in the manual (a triggered function is 0 to 10 V, a cycling Unity out is "0 to 8 V") that a measurement of a real unit would settle. Retailer copy claims a lineage to other makers' function generators; unverified and not used.

**Our approach.** Each channel is a pair of slews with a curve warp; the curve control is a documented shape ordering (log below linear below exponential) with the law set by ear within that ordering.
The buses are exact. Time constants are in seconds, so a test can state them.

## 4. Gate pair (Make Noise Optomix, black panel)

**Role.** Two vactrol lowpass gates and a mixer: each channel is a level control and a filter that open together, with a response that is quick to open and slow to close. It is what makes a pluck sound like a struck object.

**Documented** [OM-M, OM-P]
- Eight HP, four vactrols (two per channel), simultaneous control of level and brightness. Each channel has Signal in, Control CV in, Damp CV in, Strike in and Signal out; a combined pot sits on the Control and Damp inputs. Control and Damp are normalled to +8 V, so with nothing patched the pot is a manual control; Control takes 0 to 8 V; Strike expects an 8 V gate.
  Signal in is itself normalled to a positive offset, so a Strike with no signal patched is an envelope generator. An Aux in and both channels go to a Sum out.
- There is **no mode switch**: the gate, lowpass and VCA behaviours come from what is patched into Strike, Control and Damp (the revision 2 manual describes none). In revision 2 channel 1's output is no longer normalled to channel 2's input, and Damp also accepts audio-rate signals (side-chain use).
- The maker describes the response as "moderate attack, slow decay", a mild non-resonant lowpass, unsuited to clicks. Damp runs from long ringing to short and muted. A higher control voltage raises level and brightness together, with the lows rising first. Signals up to 10 Vpp.
- Generic vactrol figures, **not** the maker's: a common part's datasheet gives about 2.5 ms to turn on and 35 ms to turn off [VT-D]; a 2023 paper gives attack from tens of microseconds to a few milliseconds, release up to hundreds, and attack shortening as the light increases [VT-P]. The maker does not name its part.

**Testable.** An 8 V Strike gives a quick rise then a slow decay; a longer Damp gives a longer decay; level and brightness fall together (the SPEC-0004 H-10 method, from the Parker and D'Angelo model); with nothing at Signal in, Strike gives a positive enveloped output; a closed gate leaks a little and Damp reduces it; Sum out is exactly channel 1 plus channel 2 plus Aux. Shapes and orderings are tested, not exact times.

**Not published.** Attack and decay times, the Damp law, mix gains, the Strike threshold, the vactrol part. A forum thread that might say more was unreadable. The revision 1 manual gives channel 1's input as 18 Vpp and revision 2 as 10 Vpp; the rack's revision is unknown (README Q-S6-6).

**Our approach.** The low-pass gate of SPEC-0004 S4 (a vactrol model in the style of the cited paper), with the Strike, Control and Damp inputs and their normalling as above, shared with the voices.

## 5. Quad level (Doepfer A-135-1, 18 HP)

**Role.** Four voltage-controlled amplifiers summed to a mixer output (a voltage-controlled mixer), the rack's general-purpose level control.

**Documented** [A135-M]
- Four linear VCAs summed to one output; versions 2 and 3 add four single outputs, and version 3 adds input normalling with an all-or-selection jumper. The manual available describes version 1 (a CEM3381 part); the rack's 18 HP module
  is version 2 or 3 (SSM2164, DC coupled) [MG].
- Control range 0 V (no gain) to about +5 V (full gain). The effective control is the Gain knob (about 0 to +5 V) plus the attenuated external control; the response is linear.

**Testable.** Gain is linear in the control voltage between 0 and 5 V; zero control is silence; the output is the exact sum of the four; a DC-coupled input passes DC.

**Not published.** Numeric CV scale for versions 2 and 3; distortion and noise; behaviour above 5 V (treated as clipped to unity gain until measured).

## 6. Ladder (Doepfer A-105, 4-pole lowpass, discontinued)

**Role.** The rack's filter: a four-pole (24 dB per octave) lowpass with voltage-controlled resonance up to self-oscillation, and an input that can be overdriven.

**Documented** [A105-M]
- A 4-pole SSM2044 filter; two frequency CV inputs (one at 1 V/oct, one with an attenuator), a resonance CV with an attenuator, and a level control. The manual is shared with a different-sounding module in the same family, so only the
  functional list is relied on. Successor module: A-105-2.

**Testable.** 24 dB per octave within a tolerance (the SPEC-0004 H-4 method); minus 3 dB near cutoff at low resonance; resonance rises monotonically and self-oscillates near its top (sine at the cutoff, level bounded); 1 V/oct tracking within 2 cents.

**Not published.** Any numeric CV range, the resonance law, how it distorts when overdriven, the gain loss at high resonance.

**Our approach.** The ladder of SPEC-0004 S2 (Huovilainen-style non-linear ladder, zero-delay feedback, 2 times oversampled), shared with the voices. Nothing is recreated twice.

## 7. Chance (Make Noise Wogglebug, 10 HP)

**Role.** A family of related random voltages from one clocked source: stepped, smoothed, ringing, a random gate, and two tiny audio oscillators.

**Documented** [WB-M]
- Outputs: **Stepped** (0 to 10 V, a new value at each clock edge; at an audio-rate clock with an audio signal in, it acts as a bit reducer), **Smooth** (a lagged stepped), **Woggle** (stepped with decaying-sine edges, chasing the
  stepped value; its knob sets how fast), **Burst** (a random gate synced to the clock, influenced by the stepped, smooth and woggle settings), **Smooth VCO** (a "shark-fin" wave, 10 Vpp), **Woggle VCO** (square, 10 Vpp), **Ring-mod** (a pulse wave, 10 Vpp), **Clock out**.
- Controls: **Speed/Chaos** (internal clock from one cycle a minute to about 40 Hz, about 200 Hz with the speed CV attenuator opened; also sets the lag rate), **Ego/Id balance** (how clustered the random values are, or a balance of an outside signal against the internal source),
  **Influence** (0 to 10 V, modulates both oscillators), **Disturb** (a press forces a sample; hold freezes), and an external clock input.
- Lineage stated by the maker: a tribute to a Buchla random source; internal blocks are a clock, sample-and-hold, two lag processors, a burst generator, two oscillators and a ring mod [WB-M]. Whether the lag uses vactrols is only stated in secondary press [WB-S].

**Testable.** Stepped changes only on clock edges; Smooth always lies between old and new stepped values and approaches the new one; Woggle rings and decays; Disturb hold freezes every output and Disturb press forces a new value; the clock range; a seeded run repeats exactly.

**Not published.** The noise source and its distribution, the burst rule, the lag and ring-mod circuits. The panel and manual label an input differently ("Heart" on the product page, "Ego" in the manual).

**Our approach.** A seeded xorshift stream (no `Math.random`, as everywhere), a distribution shaped by Ego/Id that is set by ear, and a lag with the documented rate range.

## 8. Clock bank (ALM Busy Circuits Pamela's NEW Workout)

**Role.** An eight-output clock generator and modulator: from one clock it makes divisions, multiplications, swing, probability and shapes, so a rhythm and a set of slow modulations come from one box.

**Documented** [PW-M] (the manual covers firmware 206; the site offers 207; the module is discontinued and succeeded by a larger one, so which the rack holds is Q-S6-6)
- Eight outputs, 0 to 5 V, 10 bit, updated at most 500 times a second. Tempo 10 to 300 BPM; clock, run, and two CV inputs (0 to 5 V, plus or minus 5 V) to which any parameter can be assigned. External clock at 48 down to 1 pulse per quarter note (default 24); Run can start, stop or reset.
- Per output: divide or multiply from /512 to x48 including non-integers (triplets, dotted), plus constant on, constant off, a pulse at start, a pulse at stop; wave (gate, triangle, sine, envelope, random, smooth random); level, offset, width (the share of a step that is high, so pulse width), phase, delay and delay divisor (swing comes from a delay with divisor 2);
  slop (a percentage chance of timing error); Euclidean steps, hits and rotation; random skip; a loop length (0 re-seeds the random stream; the seed is saved); a quantiser to 13 scales; and a logic combine of two outputs (and, or, exclusive or).

**Testable (all edge counts).** At x4, four rising edges per beat; at /3, one per three beats. Width 50 percent at x1 and 120 BPM is a 250 ms high. A Euclidean output (steps, hits, rotation) has the known edge pattern. Skip 0 passes every step. A delay with divisor 2 shifts every other step. A looped random output repeats exactly.
An AND of x4 and /3 passes the fast edges only while /3 is high. 24 pulses per quarter in gives clock-synced outputs.

**Not published.** The exact divisor list; the length of the utility pulses; jitter; the Euclidean and random algorithms; the slop distribution; the envelope shape. Marketing and the manual disagree on the number of stored banks (200 against 26 banks of 8); not needed.

**Our approach.** A clock module driven by the engine's clock (24 pulses per quarter note, or the finer tick position), with these outputs as pure functions of the pulse count and a seeded random stream. The Euclidean rule is the standard one (Bjorklund's), stated in the test.
Most of its value is in feeding Index and Chance; a smaller first version (K3) does divisions, width, delay, probability and Euclidean; the shapes and logic come after.

## 9. Index (Orthogonal Devices ER-101, 26 HP; the ER-102 controller is optional)

**Role.** The sequencer of the rack. An externally clocked four-track CV and gate sequencer in which each step holds an *index* into a table of voltages, not a voltage. The owner asked for it for "longer patterns".

**Documented** (manual v2.09, 2018-08-20) [ER101-M]
- **Shape.** Four tracks, each with CV-A, CV-B and a GATE output, all four sharing one clock input and one reset input. Each track holds up to 100 patterns of up to 100 steps; the whole snapshot holds up to 400 patterns and 2,000 steps, shared first come, first served. Patterns are blocks only: a track plays its patterns in order and loops.
  There is no pattern chaining; loop points play that part.
- **Per step.** CV-A index and CV-B index (0 to 99 into the track's two tables); DURATION (0 to 99 clock pulses; 0 means the step is skipped); GATE (0 to 99 pulses of high time; 0 is a rest; equal or longer than DURATION holds the gate for the whole step, which is how steps are tied);
  a **ratchet** flag (the gate repeats with equal high and low times of GATE pulses until the step ends); and smoothing, per step, pattern or track, separately for A and B (the output ramps linearly to the next step's voltage; it starts when the gate falls and lasts DURATION minus GATE).
- **Per track.** Clock divide and multiply, 1 to 99 each (together since firmware 2.02); gate or trigger mode (a trigger is GATE times 0.1 ms, 0.1 to 9.9 ms; a ratcheted trigger is fixed at 0.5 ms and GATE sets the count); loop start and loop end steps; note or number display.
- **Clock.** Threshold about 2.5 V (low below 1.5 V, high above 3.5 V). A divider counts raw pulses and passes every Nth (divide by 3 passes pulses 1, 4, 7); a multiplier measures the last period T and emits N pulses at T/N spacing, in phase with the clock and one period late to follow tempo changes. A manual suggests a clock of 4 pulses per quarter note.
  A reset edge rewinds every track to step 1 of pattern 1, ignoring loop points. Pause inhibits the clock and the gates.
- **Tables.** Two per track, 100 entries each, 0.000 to 8.192 V; reference tables for twelve-tone equal temperament, 24-tone, 22-shruti, two scales, and linear and exponential ramps; eight user slots; tables need not rise.
  The outputs are 14 bit, 0 to 8.192 V, refreshed about 3 kHz; gates 0 V and at least 9 V.
- **Maths.** Destructive edits per step, pattern or track: add, multiply or divide, set, random up to N, jitter plus or minus N, quantise to a multiple of N. A result outside 0 to 99 leaves the value unchanged.
- **Editing.** Edit, follow and hold modes; hold edits a shadow copy and commits at once or at the end of the step, pattern or track. 16 snapshots in flash exchanged with a computer as readable XML.
- **Period and phasing.** There is no track-length setting: a track's period is the sum of the DURATIONs it plays. Tracks of different periods drift against each other; the manual names phasing between tracks as a purpose and gives no recipe.
- **ER-102** (optional, 14 HP): up to 99 CV-selectable *parts* per snapshot, each storing a reset step and loop section per track, with a transition setting; up to 16 *groups* of steps (including Euclidean masks) with transforms keyed to three gates and a three-input modulation matrix added after the table lookup;
  recording from CV and gate; MIDI file import (channels mapped to tracks 1 to 4) [ER102-M].

**Testable.** All of the above is pulses and numbers. Divide by 3 passes pulses 1, 4, 7. A multiplier by 4 emits four evenly spaced pulses per period. DURATION 8 with GATE 2 and ratchet fires twice. GATE 0 is silent, DURATION 0 costs no clock time (to be confirmed) and skips. A reset returns to step 1. Loop points loop.
Smoothing starts at the gate fall and ends at the step end. 12-tone table index 12 is exactly one volt above index 0. Output range is 0 to 8.192 V with a step of 8.192/16383 V.

**Not published or inconsistent.** What ratchet does when DURATION is not a multiple of twice GATE; what smoothing does when GATE is at least DURATION; the multiplier's maximum rate; whether a DURATION 0 step costs time. The manual gives CV resolution as 2 mV in one place and 1 mV in another (14 bits over 8.192 V is about 0.5 mV) and latency as 0.333 ms and 0.35 ms. The ER-102 manual is v2.04 while the maker's changelog goes to v2.24: features after the manual are undocumented.

**Where "64" comes from.** The maker's documents contain no 64-step limit. The only 64 found is the Octopus's own chain builder (two chains of four tracks make 64 steps, one chain of eight makes 128) [OCT-M], and the engine already has chained tracks (`crates/octocore`, `ChainBase`). Question Q-S6-3 asks which was meant. What the ER-101 adds beyond longer tracks is the *index* (retune everything by editing a table), per-step durations and ratchets, and tracks of unrelated length that phase.

**Our approach.** A module whose state is the snapshot above (tracks, patterns, steps, tables) and whose clock is the page's engine clock through the Clock bank or directly. It is a second sequencer *inside the kit*: it reads the engine's clock and transport and writes nothing to the engine, so the Octopus is unchanged (decision D6).
The editor is its own slice (K4b) because it needs a grid view of up to 100 steps.

## 10. Notes (Doepfer A-190-3, USB and MIDI to CV and gate)

**Role.** The bridge from the sequencer to the rest of the kit: MIDI notes become a pitch voltage and a gate, and the two controllers become further voltages. In hardware it is how the Octopus would drive her oscillators. Here it is the one place the engine's events enter the kit.

**Documented** [A190-M, A190-2-M] (the A-190-3 page, plus the A-190-2 manual which the maker says shares the learn functions)
- Six HP, USB and MIDI in. Four CV outputs and one gate; **mono only**; no clock, start, stop or continue outputs (those are on a sister module). CV1 is the note at **1 V per octave**, 0 to 5 V, buffered, with a glide control; CV2 is pitch bend; CV3 is velocity (or, per the A-190-2 manual, volume; the two disagree); CV4 is a learnable controller.
- Note to volts: volts = (note minus reference) / 12, reference note 36 (C2) by default, MIDI channel 1 by default; both are learnable. 0 V is the reference note and 5 V is 60 notes above it; notes outside that span are ignored.
- Gate is 0 to +5 V (a jumper gives +12 V): high on a valid Note On, low on the matching Note Off. Retrigger (a gate dip of "a few ms") is off by default. Key assignment (set by program change in learn mode): highest note, last note, reference note, or stack.
- Pitch bend about plus or minus 2.5 V for data 0 to 127 (its width in semitones is set outside the module).

**Testable.** The note to volts table, and the span; a note outside the span is ignored; gate high from Note On to Note Off; last-note priority returns to the held note when the newest is released; the retrigger dip when enabled; pitch bend end points; the CV4 controller.

**Not published.** Gate latency and jitter, the retrigger length, the glide range, bend resolution, aftertouch (none), Note On with velocity 0, and which of velocity or volume CV3 carries.

**Our approach, and one deliberate extension.** The Octopus plays chords of up to seven notes per step, and a mono bridge cannot. The module ships as the hardware does (mono, the four assignment modes, reference 36, channel select) and adds an optional **poly** mode with up to four note outputs, assigned oldest first as the page's monitor already does (decision D7).
It reads the events the monitor already reads. It cannot send MIDI and changes nothing in the engine. Because the page is software, an "extended range" option lifts the 5-volt span; it is off by default so the tests of the table can be exact.
Notes also provides run, stop and clock pulses from the engine's realtime events, which in hardware come from a sister module.

## 11. String (2hp Pluck), Pitch echo (Make Noise Echophon), Space (Make Noise Erbe-Verb)

### 11a. String (2hp Pluck)

**Role.** A plucked string: a trigger and a pitch make a note whose body decays like a real string. It is the rack's only physical-model voice.

**Documented** [PL-M]
- Two HP, **four voices**, an algorithm the maker states as Karplus-Strong. Inputs: Trig (a trigger starts a new note with the knobs as they are; the oldest of the four voices is replaced), V/Oct plus a Pitch knob (the newest note follows pitch live, older notes keep theirs, allowing vibrato and slides), **Damp** (knob plus plus or minus 5 V; sets string "size" independent of pitch; the left end damps the most harmonics) and **Decay** (knob plus plus or minus 5 V; from percussive to near infinite). Damp and Decay affect new notes only. Output 10 Vpp.
- The maker claims accurate 1 V per octave over five octaves.

**Testable.** The fifth trigger steals the oldest voice; older notes keep their pitch while the newest follows V/Oct; pitch within 2 cents over five octaves; changing Damp or Decay mid-note leaves the sounding note unchanged; decay time rises with Decay; a longer string (more Damp) is darker.

**Not published.** The lowest and highest note, tuning error, trigger threshold, chip and firmware, the excitation (a noise burst or otherwise), the decay law and the damping filter. A review says it squeaks at the top of its range (anecdotal).

**Our approach.** The extended Karplus-Strong string of the open literature (Jaffe and Smith): a noise or short-burst excitation from a seeded stream, a delay line with a fractional-delay tuning allpass, and a damping filter in the loop, four voices, oldest stolen. The tuning is held to the same 2-cent bound as every other pitch (K-1), which is tighter than the maker's own claim.

### 11b. Pitch echo (Make Noise Echophon, 20 HP)

**Role.** A pitch-shifting echo: a delay line whose repeats are shifted in pitch, so that feedback spirals up or down.

**Documented** [EP-M, EP-P]
- A digital core: 24 bit converters, 32 bit processing, 16 bit delay storage, 40 kHz. The dry path is analog. The pitch shifter was written by Tom Erbe and is said to be inspired by a rotating-head tape pitch changer (four heads 90 degrees apart reading segments of about 40 ms) [SPR]. The manual does not say the module implements that.
- **Echo time** 7 ms to 1.7 s, exponential, with CV in (0 to +8 V); below about 20 ms it acts as a comb filter. **Tempo in** (at least 1.5 V, at least 6 ms) turns Echo time into a ratio from 1/16 up to 12/1 with 1/1 at noon; Echo clock out is a 4 ms pulse (0 to 5 V).
- **Pitch** is a bipolar knob (noon is no shift) with two CV inputs (plus or minus 4 V with attenuverter, plus or minus 2 V for the full range); **Depth** scales the pitch range from a fraction of a semitone to plus or minus two octaves; pitch shifting needs Depth above about 9 o'clock.
- **Feedback** is bipolar. One side puts the feedback *around* the pitch shifter, so each repeat is shifted again; the other puts it around the echo alone; noon is none (one repeat). An external feedback loop (AC coupled, 10 Vpp) is available.
- **Freeze** (button or a gate of at least 1.5 V; the button wins) holds the buffer; pitch still works, Echo time then changes the sound destructively, and feedback does nothing.
- Input is clean for the first 70 percent of the Level knob (10 Vpp).

**Testable.** Echo time range and its exponential law; the tempo ratio table; Freeze makes the output independent of new input; the two feedback placements differ in pitch behaviour on repeats; the pitch range is plus or minus 2 octaves.

**Not published.** Number of heads, segment length, crossfade and window of the pitch shifter; the law of the vactrol mix. **The manual contradicts itself** on which way Depth and the ratio direction go; section 14.

**Our approach.** A delay line with a rotating-head or granular pitch shifter (from the published idea and from the open DSP literature on delay-line pitch shifting; the crossfade and window set by ear). The 40 kHz and 16 bit character is an option, off by default, tested as a sound choice (E-5).

### 11c. Space (Make Noise Erbe-Verb, 20 HP)

**Role.** A continuously variable reverb with no presets, from a small box to a very large space, with modulation, reverse and a shimmer.

**Documented** [EV-M, EV-P]
- Digital: a 24 bit, 48 kHz codec and 32 bit floating point, mono in, an analog dry path. The algorithm is described only as "modeless" and continuously variable; the structure is not stated anywhere found.
- **Size** 35 to 9.3 million cubic feet, with CV; typical settings by position (coffin, room, plate, hall, ambient, heaven). **Pre-delay** 7 to 500 ms, independent of Size, or with a clock a ratio 1/12 to 12/1. **Reverse** (button or at least 1.5 V) plays the pre-delay buffer backwards.
- **Decay** 0 to 120 percent of reflection gain, infinite at the end of its travel. **Absorb** combines diffusion and damping: fully down is neither, about 10 o'clock is full diffusion and no damping, fully up is both.
- **Depth** is bipolar: noon is none; one side adds cyclic modulation (a multiphase chorus-like cascade), the other adds ergodic modulation (random shifts of the room's dimensions that turn granular at high depth, ending in a shimmer). Speed 0.5 to 256 cycles per second (more with a clock), with no audible effect at Depth noon.
- **Tilt** is applied last, outside the loop: noon is unity, one side cuts highs and boosts lows, the other the reverse. **CV out** follows the reverb's envelope. **Level** is unity at 3 o'clock. Outputs: left (mono) and right.

**Testable.** Maximum decay sustains without growth; decay time rises with Size; pre-delay is independent of Size; Tilt does not change decay; Speed does nothing at Depth noon; the table of Size positions gives monotone decay times.

**Not published.** The topology, diffusion and damping structures, the mapping from Size to delay lengths, the shimmer's pitch method. **The manual contradicts itself** on which end of Decay is infinite.

**Our approach.** An algorithmic network of the kind in the open literature (Jot and Chaigne's feedback delay network; Dattorro's plate), with Size scaling all delay lengths and Absorb mapped to input diffusion and damping. The "120 percent" is not implemented as a gain above unity (that blows up); the top of Decay is a *hold* at exactly unity loop gain with damping off, and a test pins it (V-2).
No recorded spaces are used (decision D9).

---

## 12. Parts bin (not in the current rack, wanted by any kit)

Small utility modules the 2018 setup list or the patches above assume. Each is a single page of function, built in K2 or K3 if the owner wants them (Q-S6-4).

- **Envelope (Doepfer A-140 class).** ADSR with gate and retrigger inputs, three time ranges, exponential segments. Attack about 20 microseconds to 100 ms (low range), 400 microseconds to 2 s (middle), 20 ms to 120 s (high); decay and release three to four times longer. Outputs about 0 to +8 V and an inverted 0 to -8 V [A140-M].
- **LFO (Doepfer A-147 class).** Triangle, sine, square and falling saw; reset by gate; 0.01 to 50 Hz by the manual, about 150 Hz with CV by the product page; levels about plus or minus 5 V, saw 0 to +7 V [A147-M].
- **Divider (Make Noise Tempi class, six channels; absent from the 2026 rack).** Divides, multiplies and phase-shifts a clock six ways, 64 stored states in 4 banks of 16, tap or external clock (locks after two pulses), run and stop, shift-register mode, 50 percent clock or 10 ms trigger outputs [TP-M]. It has no LFO function (a mistake some summaries make).
  Not published: tempo range, fine step size, output level, jitter.

## 13. The case

The CV bus case is the page's `Rack`: the place that holds the modules, powers them (nothing to model), and carries the few signals that a bus case carries in hardware (a gate and a clock that modules are normalled to). Blank panels are layout and cost nothing.

## 14. Where the sources disagree (to be settled by measurement or by the owner)

| Module | Disagreement | Plan |
|---|---|---|
| Function | Triggered function is 0 to 10 V, cycling Unity out is "0 to 8 V" in the same manual | Measure the page's own build against 10 V; keep the documented 10 V; flag |
| Function | End-of-cycle gate described as idle-high, and as going high at the end of the fall | A K2 test pins one reading; the owner confirms by ear on a patch |
| Chance | A product page says "Heart" and the manual "Ego" for the same input | Name it by function |
| Pitch echo | Depth full range is "counter-clockwise" in one place and clockwise in another; ratio direction (multiply or divide) is stated both ways | Choose the reading that matches the panel text of the tempo table; owner confirms by ear |
| Space | Infinite decay is stated at both ends of the Decay control | Infinite at the end the factory table uses; owner confirms |
| Index | CV resolution 2 mV and 1 mV; latency 0.333 and 0.35 ms | Use 14 bits over 8.192 V; latency as one sample |
| Index | Manuals v2.09 and v2.04 are older than the changelog (later firmware undocumented) | Build to the manuals; list later features as not planned |
| Ladder | Manual shared with a different filter chip | Build to the SSM-class description only |
| Quad level | Manual describes version 1; the rack has version 2 or 3 | Build to the DC-coupled, linear behaviour common to all |
| Rack | The 2018 list and the 2026 rack differ | Follow the 2026 rack (Q-S6-4) |

## Source keys (full list with types is in README section 10)

MG ModularGrid rack and module pages (database, lowest trust) | SA 2018 article on the live setup (press) | SOS-I 2017 interview (press) | VH-M, VH-B, VH-R, VH-F Verbos Harmonic Oscillator: maker, designer's blog, retailers, forums |
DPO-M, MA-M, WB-M, WB-S, EP-M, EP-P, EV-M, EV-P, TP-M Make Noise manuals, product pages, press | A135-M, A105-M, A140-M, A147-M Doepfer manuals | ER101-M, ER102-M Orthogonal Devices manuals and changelog |
OCT-M the Octopus manual v0.9 | SPR an essay on the rotating-head pitch changer

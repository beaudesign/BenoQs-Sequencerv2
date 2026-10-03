# Sequencing theory, for an Octopus-style step sequencer

**Status: reference note, Conductor hat, documents only.** It is the second half of the owner's request of 2026-10-02 16:19 Paris ("learn deep music theory for sequencing"); the
first half is `specs/SPEC-0002/ableton-mcp-review.md`. It changes no behaviour and no threshold. Its job is to let an agent or the owner say *what the instrument can
say musically*, and to turn each statement into something a test can assert (section 6), because a taste judgement that is not a measurement does not ratchet.

**Source labels.** **[M p.N]** the Octopus Reference Manual CE v5.30 in `reference/manual`. **[T]** Toussaint, *The Euclidean Algorithm Generates Traditional Musical Rhythms*
(Bridges 2005), as read. **[L]** Roger Linn, as quoted on melodiefabriek.com. **[A]** Ableton's reference manual. **[C]** checked by running `handoffs/evidence/theory-check.py`
(72 checks, output in `theory-check.txt`; a Python model, not the engine). **[G]** general music theory from my own knowledge, **not source-checked in this session**: where a claim
only carries [G], treat it as a well-known convention and not as a finding.

**D0 and every duration here.** The manual makes a step at default length 12/192 of a "note" and "1/16" of it ([M p.15]; `tests/conformance/AMBIGUITIES.md`, "tick resolution"),
so a step is a sixteenth and a 16-step bar one whole note ([M p.67]). The engine today has 192 ticks to the *quarter*, so its step is a 64th note: 31.25 ms at 120 BPM, where the
manual's reading gives 125 ms. That is D0 and it is the owner's. **This note writes positions in steps and ticks, which do not depend on D0, and gives milliseconds at both readings
where they matter.** Nothing here claims a pattern is "in time".

## 1. Time

### 1.1 The grid
- 192 = 2^6 x 3. It holds 16ths (12 ticks), 32nds (6), 64ths (3), 8th-note triplets (16) and 16th-note triplets (8), and **holds no quintuplet or septuplet** (5 and 7 do not divide
  192) [C]. The instrument can say duplet and triplet feels exactly and cannot say 5 or 7 against 4 except by approximation.
- A 16-step track is one bar of 4/4 in sixteenths. Other metres are made with track length, so 12 steps is 3/4 in sixteenths, 20 is 5/4, and 14 is 7/8 [G].

### 1.2 Swing
The MPC defined it as a percentage: it **delays the second 16th note of each pair** (the even-numbered ones), **50 %** is none, **66 %** is the triplet feel (the first note gets
two thirds of the pair), **75 %** is the most it goes, and beyond that it stops sounding like swing [L]. The delay in ticks is `(percent/100 - 1/2) x pair length`; a pair of steps is
24 ticks [C]:

| Swing | Delay of the second step | At 120 BPM, reading A (125 ms step) | At 120 BPM, engine today (31.25 ms step) |
|---|---|---|---|
| 54 % | 1 tick (0.96) | 10 ms | 2.5 ms |
| 58 % | 2 ticks (1.92) | 20 ms | 5 ms |
| 62 % | 3 ticks (2.88) | 30 ms | 7.5 ms |
| 66.7 % | 4 ticks, a third of a step | 41.7 ms | 10.4 ms |
| 70.8 % | 5 ticks | 52 ms | 13 ms |
| 75 % | 6 ticks, half a step | 62.5 ms | 15.6 ms |

**On the Octopus there is no swing control.** Swing is made by pushing the even steps with STA. A step's push is at most **5 ticks** at the neutral track STA [M p.16], so the neutral
track reaches (12 + 5) / 24 = 70.8 %; the track's STA scales the step's value and the chart gives **+6 from track STA 10**, which is 75 % [M p.45; C]. 8th-note swing is the same
arithmetic on the offbeat 8ths, which are steps 3, 7, 11 and 15 of 16, with a pair of 48 ticks. Whether swing is the same at 70 BPM and 160 BPM is a taste fact, not arithmetic: jazz swing
ratios are said to flatten as the tempo rises [G].

### 1.3 Groove is two maps
Ableton's Groove Pool describes a groove by **Base** (the grid it is measured against), **Quantize**, **Timing** (how much of the groove's timing is applied), **Random**, **Velocity**
(-100 to +100) and **Global Amount** (up to 130 %), and a groove can be extracted from any clip [A]. Underneath, a groove is a **timing offset and a velocity offset for each position of
the grid.** The Octopus has both per step (STA and VEL) and a per-track scaling of each (the map factors, 1 to 17 with **9 neutral** [M p.53]). **A groove template is therefore a pair of
16-value maps, and "amount" is the track's factor.** Programming a groove on this instrument means setting STA and VEL maps and turning the factors.

### 1.4 Humanise: deterministic by default
Live's Random gives each voice its own timing wobble [A]. Roger Linn's view is that the best grooves sit when notes are played "at exactly the perfect time slots", and he rejected random
variation as a source of musicality [L]. The two are a design tension, not a contradiction. **Stance for this repo: groove is deterministic (STA, VEL); randomness is seeded**, so a
test can assert it (the engine takes a seed).

### 1.5 Several lengths at once
Tracks have their own lengths and clock multipliers. Two loops of a and b steps realign after lcm(a, b) steps: 16 and 12 after 48, 16 and 15 after 240, 16 and 7 after 112 [C]. That is
how short material gets long-form variation: **phasing** with no extra data. A **hyperstep** is a step that triggers a whole other track, time-compressed into the hyperstep's own length,
with its pitch and velocity offsets; it cannot be nested, and a hyped track has one hyperstep [M p.31]. A step can hold a whole figure of its own, which is how a motif sits inside a larger phrase.

### 1.6 Euclidean rhythms
E(k, n) spreads k onsets over n slots as evenly as possible; Bjorklund's grouping-and-remainder procedure generates it, and Toussaint showed that many traditional rhythms are E(k, n) up to
rotation [T]. These are **checked: my implementation reproduces every one of the ten patterns below exactly as printed, and each is maximally even** [C].

| E(k, n) | Pattern | Tradition, as given by [T] |
|---|---|---|
| E(2,5) | `x.x..` | Persian; "Take Five" |
| E(3,7) | `x.x.x..` | Bulgarian folk dance |
| E(3,8) | `x..x..x.` | Cuban tresillo |
| E(4,7) | `x.x.x.x` | Bulgarian folk dance |
| E(5,8) | `x.xx.xx.` | Cuban cinquillo |
| E(5,12) | `x..x.x..x.x.` | Venda (South Africa) |
| E(5,16) | `x..x..x..x..x...` | Brazilian bossa nova (as a rotation: `x..x..x...x..x..`) [C] |
| E(7,12) | `x.xx.x.xx.x.` | West African bell |
| E(7,16) | `x..x.x.x..x.x.x.` | Brazilian samba |
| E(9,16) | `x.xx.x.x.xx.x.x.` | West African |

**Rotation is a knob.** Rotating a rhythm gives the same necklace with another downbeat, and the Octopus's POS knob shifts a track's steps around [M p.45]. So a family of grooves from
one Euclidean pattern costs one knob. **The 3-2 son clave `x..x..x...x.x...` is not Euclidean** (its gaps are 3 3 4 2 4, which differ by 2), and the 2-3 clave is the same pattern rotated by
half a bar [C]. "Euclidean" is not the same as "traditional".

### 1.7 Idioms worth knowing [G]
Backbeat (snare on 2 and 4: steps 5 and 13), four on the floor (kick on steps 1, 5, 9, 13), offbeat hats (steps 3, 7, 11, 15), the tresillo as 3 + 3 + 2 sixteenths, and **ghost notes**
(low velocity between accents, which is a VEL map, not a pattern).

## 2. Pitch and harmony

### 2.1 Scales are sets
A scale is a set of pitch classes, and the engine stores one as a 12-bit mask (`PitchClassSet`, `crates/octocore/src/scale.rs`). The Octopus lets any notes be toggled into a page's scale
around a root [M p.71], so **every scale in the books is one mask.** Modes are rotations of the major scale's intervals [C]:

| Mode | Intervals from the root |
|---|---|
| Ionian | 0 2 4 5 7 9 11 |
| Dorian | 0 2 3 5 7 9 10 |
| Phrygian | 0 1 3 5 7 8 10 |
| Lydian | 0 2 4 6 7 9 11 |
| Mixolydian | 0 2 4 5 7 9 10 |
| Aeolian (natural minor) | 0 2 3 5 7 8 10 |
| Locrian | 0 1 3 5 6 8 10 |

Also: major pentatonic 0 2 4 7 9, harmonic minor 0 2 3 5 7 8 11, whole-tone 0 2 4 6 8 10 [G].

### 2.2 Chords are stacked thirds
Stacking thirds on each degree of the major scale gives triads **major, minor, minor, major, major, minor, diminished** and sevenths **maj7, m7, m7, maj7, 7, m7, m7b5** [C]. Roman
numerals and function (tonic, subdominant, dominant) are conventions for naming them [G]; the common loops are I V vi IV, I vi IV V, ii V I and (minor) i VII VI V [G].

On the Octopus a step holds a chord (the strum table covers up to seven notes). **Strum** delays the later notes by a table of ticks that, the manual says, grows exponentially with the level: level 1 puts note 7 3
ticks late, level 9 puts it **45** ticks late, 3.75 steps [M p.22]. The engine's table is **cell for cell the manual's** [C], and it never decreases along either axis [C]. A strum on a single note plays it followed by six duplicates, which
is an echo [M p.22]. **Polyphony below the chord size picks notes at random from the chord, and above it picks from the chord plus rests**: with C E G, polyphony 2 plays two of the three, and 5
plays three from {C, E, G, rest, rest} [M p.22]. That is a probabilistic arpeggiator in one parameter.

### 2.3 Forcing to a scale, and what the tie rule decides
Force-to-scale moves every pitch to the nearest scale tone, **after all offsets are summed** (`docs/03` section 3; `scale.rs`). A pitch exactly between two scale tones is a **tie**, and it
happens exactly in the middle of a two-semitone gap of the scale: C major has five (C# D# F# G# A#), the major pentatonic three, the whole-tone scale all six of its out-of-scale
classes [C]. **The engine sends ties down, "matching v1"; the manual does not say what the Octopus does.** It matters, and this is a checked result:

> Take a tonic chord, shift it by the root of each degree (0 2 4 5 7 9 11 semitones, which is what a feeder track's pitch offsets do, section 2.4), and force the result to C major.
> **With ties down, you get the seven diatonic triads and the seven diatonic sevenths exactly.** With ties up, four of seven triads (degrees 2, 3, 6 and 7) come out wrong, and five of seven sevenths [C].

That is why "ties go down" is a musical choice and not a detail: it is what makes parallel chromatic shifting sound diatonic in a major key. **Whether the real instrument does it is an
Octopus question** (Q-M7 in the review). What is checked is the model of the engine's documented rule; the engine had a test for the single case C# to C and none for this property until `crates/octocore/tests/theory.rs` (2026-10-03), which says what the engine does and not what the instrument does.

### 2.4 Functional harmony with the effector
The effector lets a **feeder** track project step offsets (VEL, PIT, LEN, MCC) onto **listener** tracks in the same page, always from higher tracks to lower, and **the offsets add** [M p.59-61]. The manual's
example: feeders at +3, -1, -2 and -2 leave a net of **-2 semitones** on the last listener [M p.60; C]. A feeder with PIT offsets `0 5 7 5` over four steps is a progression I IV V IV applied to every
listener, and a page scale on the listeners snaps the result into key (2.3). **Scale cadence mode** goes the other way: editing the scale remaps the pitches already in the page at once [M p.71], which is re-harmonisation
by editing a set. Two limits [G]: a chromatic offset is not a scale-degree offset (parallel shifting keeps the *shape* and relies on the snap for the key), and a feeder carries roots, not voicings.

### 2.5 Voice leading
Good chord changes move few notes a short way, keep common tones, and avoid parallel perfect fifths and octaves [G]. Measured on triads in C: C to F moves **3 semitones** in total (C stays,
E to F, G to A) against **15** for the same chords in root position; C to G moves 3 (C to B, E to D, G stays); C to Am moves 2 (two common tones) [C]. A chord helper, if the owner ever wants one (Q-M5), has a
checkable objective: minimise total motion. The Octopus stores chords as pitch sets per step and does not choose inversions.

## 3. Melody and development

### 3.1 Material and its transformations [G]
Develop a motif by **transposition**, **inversion** (mirror the pitches), **retrograde** (reverse the order), **augmentation and diminution** (twice as long, half as long), **sequence** (repeat at another
pitch) and **rotation**. On the Octopus: direction 2 is retrograde and 3 (ping-pong) alternates it, the POS knob rotates, a track multiplier below or above 1 is augmentation or diminution, shifting a PIT map is transposition,
and a hyperstep with a PIT offset plays a whole track transposed (statement and answer) [M p.31, p.45, p.53]. I found no inversion control in the manual pages read; it would be a map edit.

### 3.2 Direction is a stochastic process
Directions 1 to 5 are forward, reverse, ping-pong, **Brownian (2/3 forward, 1/3 reverse)** and random order; 6 to 16 are editable [M p.45]. Brownian is a biased random walk with a drift of **+1/3 step
a step** [C]. An editable direction is 16 **slices**, each holding up to nine chase-light triggers; an empty slice picks a position at random, and each slice has a `certainty_next` percentage [M p.56]. That is a
**Markov chain over step positions**, and with a seed it is testable by its statistics [G].

### 3.3 Phrases, mutation, MCC
Three banks of 16 phrases cover delays, rhythmic delays and note intervals, with a time-compression value (8 is neutral) [M p.16-17]: ratchets, echoes and arpeggio figures from one dial. The RND and RMX mutators
re-roll or remix the maps [M p.53]. MCC steps send a controller value, and the track's resolution setting inserts intermediate controller messages on a **linear slope** between steps [M p.53]: that is a stepped LFO
you can draw. A useful form rule [G]: repeat a phrase three times and vary the fourth.

## 4. What the other tools generate, and how to say it with these controls

| Generator in the MCP survey | With the Octopus's own controls |
|---|---|
| Euclidean rhythm | Toggle the steps of E(k, 16) (section 1.6) and rotate with POS |
| Swing, Groove Pool | STA pushes on even steps; STA and VEL maps scaled by the track factors (sections 1.2, 1.3) |
| Humanise | A small deterministic STA and VEL map, or a seeded phrase; not noise |
| Chord progression | A feeder track's PIT offsets over a scale-forced listener page (section 2.4) |
| Voice-led voicings | Step chords entered by hand; the voice-leading rule in section 2.5 is the guide |
| Arpeggio, random picks | Step polyphony below the chord size; a phrase from the note-interval bank |
| Motif transform | Direction, POS, track multiplier, hyperstep with a pitch offset (section 3.1) |
| Walking bass | A feeder with chord roots plus a listener on a scale; passing tones by hand [G] |

## 5. Open to a person with an Octopus
1. The force-to-scale **tie rule** (section 2.3). Test: force C major, play C# at default settings, listen for C or D.
2. D0: at 120 BPM does a 16-step pattern at x1 with default lengths take 2 s or 0.5 s?
3. The contents of the 48 phrases ([M p.16] names three banks; the engine's "forward enriches step" fixture is the only one).
4. Strum direction labels (the manual says clockwise strums up; confirm against the sound).

## 6. From theory to tests

| Property | Oracle | Where it stands |
|---|---|---|
| Euclidean patterns | Bjorklund, ten patterns from [T] | Checked in Python [C]. No generator exists in the engine, so nothing to assert there unless Q-M5 is accepted |
| A swing of s % starts the even steps `(s/100 - 1/2) x 24` ticks late | Arithmetic in section 1.2 | STA scaling is covered by conformance (`AMBIGUITIES.md`, "LEN/STA scaling tables: resolved"); **the whole-bar swing statement is not asserted** |
| Force-to-scale puts every emitted pitch in the scale | Set membership | One engine test (`scale_quantization_pulls_pitch_into_scale`, page scale only); and `crates/octocore/tests/theory.rs` (the nearer tone wins where there is no tie; through a feeder and a page scale) |
| Ties go down | `scale.rs` test `out_of_scale_prefers_downward`, and `theory.rs` (every tie in C major, the pentatonic and the whole-tone scale, two octaves) | Tested in the engine; **not manual-sourced**, so an Octopus question |
| **Feeder offsets plus force-to-scale on C major give the seven diatonic triads and sevenths** | Section 2.3 | **Built** (2026-10-03): `crates/octocore/tests/theory.rs`, in `scale.rs` alone and through a feeder and a page scale; ties up fails three tests (`handoffs/evidence/theory-tests-mutation.txt`, 9 of 9 breakages caught) |
| Effector offsets add (+3 -1 -2 -2 = -2) | [M p.60] | Fixture `tests/conformance/effector/feeder_pit.fixture` |
| Strum offsets match the table | [M p.22] | The engine's table equals the manual's cell for cell [C]; the engine's own tests check three cells (`tables.rs`) |
| Brownian direction drifts forward at +1/3 a step | Seeded run, binomial bound | Direction 4 assignment confirmed (`AMBIGUITIES.md`); **statistics not asserted** |
| Loops of a and b steps realign after lcm(a, b) | Property over lengths | Not asserted |
| The grid holds triplets and not quintuplets | 192 mod n | Changes with D0 only if the tick is redefined |

Of the "proposed" rows, the diatonic one is built (`journal/metronome/requests/2026-10-03-theory-tests.md`); the swing statement, Brownian statistics and lcm are not, and the request says why for each. The Octopus questions need a person with the instrument.

## 7. What this note did not check
No instrument was heard. Section 1.7, the progressions in 2.2, the transformation list in 3.1, the idioms, and every [G] claim are conventions from my own knowledge. The Euclidean table follows
[T] as summarised by the fetch tool; the Groove Pool and MIDI-effect descriptions follow Ableton's manual as summarised. The Python model in `theory-check.py` is not the engine.

Sources: [Toussaint 2005](https://cgm.cs.mcgill.ca/~godfried/publications/banff.pdf); [How MPC swing works (melodiefabriek)](https://melodiefabriek.com/sound-tech/mpc-swing-reason/); [Ableton: Using Grooves](https://www.ableton.com/en/manual/using-grooves/);
[Ableton: Live MIDI Effect Reference](https://www.ableton.com/en/manual/live-midi-effect-reference/); the Octopus Reference Manual CE v5.30 (`reference/manual`).

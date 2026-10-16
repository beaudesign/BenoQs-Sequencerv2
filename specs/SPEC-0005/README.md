# SPEC-0005: A chord style that drifts (chance with a memory)

| Field | Value |
|---|---|
| Spec | SPEC-0005 |
| Revision | r0, draft, 2026-10-16 |
| Stage | **Planning only.** This pull request adds the spec, one request and a journal entry, and changes no code, contract, fixture or threshold |
| Risk tier | **Low** for this pull request (`specs/`, `journal/`). The slices are **Medium** under D3's default (a web-only extension: `crates/octocore`, `apps/web/engine`, `apps/web/`) and **High** if the owner picks the frozen FFI (`crates/octoffi/`, `contracts/`) |
| Owner and approver | `@beaudesign` |
| Task | `WENGE-0031` |
| Extends | SPEC-0003 (the circle: chords, polyphony and strum are the CHORD block's). Needs none of its slices |
| Sister spec | SPEC-0004 (a sound that behaves like hardware), on its own branch. The two do not depend on each other |
| Base | `main` at `6a14539` |

## 1. The ask

Recorded here because chat is not a source of truth. On Fri 2026-10-16 (17:15 London, from a phone) the owner wrote:

> let's explore a sequencer pattern which was unique and unusual as an additional style to add to chords which is also in
> benny - https://github.com/playbenny/benny/ on my phone now so can't show it exactly but let's explore and plan

**The reading.** The owner could not point at a block from a phone, so this spec surveys benny's sequencer blocks
(section 3), picks one, and says why. "A style to add to chords" is read as: **a new way of choosing which notes of a
chord sound, and when**, next to the three the Octopus has (section 2). "Unique and unusual" is read as: not a thing the
Octopus already does. The pick is **Drift**, from benny's `seq.turing`. Two others are close runners-up and are in
section 3. If the owner meant another block, it costs one paragraph here and nothing in the code (Q-S5-1).

## 2. What chords do today (the manual's words, then the engine's)

The Octopus has three chord behaviours, all on the CHORD block (manual p.20 to 23):

1. **The chord.** Up to seven notes on a step: the base pitch and up to six offsets. Polyphony equal to the chord size
   plays them all.
2. **Random picks from the chord pool** (p.22). Polyphony different from the chord size: "the number of played chord
   notes is always the smaller of the two". Smaller polyphony draws that many chord notes; larger pads the pool with
   rests, and a draw can land on a rest. C-E-G with polyphony 5 is "3 elements picked at random from the pool
   {C, E, G, rest, rest}".
3. **The strum** (p.22). A level from minus 9 to 9 delays note 2 to 7 by a fixed table of ticks, up or down.

**How the engine does the second one.** `crates/octocore/src/engine.rs` (the chord branch of the step firing, near line
1384) draws with `self.rng.next_below(..)`, once per pick, without replacement, from the one generator the whole engine
shares. So the draw is **memoryless**: step 5 knows nothing about step 1, and a pattern of chord steps never repeats
itself. That is what the manual describes, and it is right.

**Finding F1, not part of this ask.** The manual's p.23 says the same pool trick works "without a chord ... on single
notes, thus effectively giving the chance of a note or a rest": one note with polyphony 2 is a 50 percent rest, with
polyphony 4 a 25 percent note. The engine's branch for a step with no chord (`else { notes[0] = final_pit; ... }`) never
reads the polyphony, so a single note always plays. I found it while reading; I have not fixed it (not my zone) and have
filed `journal/conductor/requests/2026-10-16-random-step-rest.md` for the Metronome. It matters here only because Drift
would apply to it too (S3).

## 3. benny's sequencer blocks, surveyed

Read on 2026-10-16: the `help_text` of each block in `note_blocks/` in `https://github.com/playbenny/benny/`, and its
README and `LICENSE.md`. benny is built from Max/MSP patches and is under the **Hippocratic License 3.0**, whose notice
and copyleft terms make it unwise to take any of its code before the owner decides on a licence for this repository
(SPEC-0004 section 3 has the detail). This spec takes **ideas only**; the algorithm below is a few lines, is described
here in full, and is written from this description.

| benny block (its own words) | What it is | Fit for chords | Verdict |
|---|---|---|---|
| `seq.turing`: "Implementation of the famous music thing turing machine" | A looping register of random values. A **lock** control sets how likely a value is to be replaced as it comes round. All the way locked it repeats exactly; all the way open it is pure chance; in between it **drifts** | The Octopus's chord draw has chance and no memory. This gives it a memory | **Chosen: Drift** |
| `midi.delay`: "per-note random delays are useful for eg randomly varying the order and spacing of the notes of chords" | A random spread of a chord's notes in order and time | A second kind of strum: today's is a fixed table | **Runner-up: Scatter.** Timing only, so cheap. Shares the hook (S4) |
| `midi.fidget`: "refuses to sit on one value for too long, once it gets bored takes a brownian hop to a nearby value" | A value that stays, then hops to a neighbour | The chosen chord note stays, then moves one chord tone up or down | **Runner-up: Walk** (S4) |
| `seq.rene`: "a circle on a cartesian plane step sequencer" | A pointer moving over a grid of notes | A different instrument; the Octopus's order comes from its directions | No |
| `seq.wonky`: step lengths that "pro-rata" to fit a period | Uneven step times | The Octopus has START offsets, groove and shuffle | No |
| `seq.shape.player`: replays a stored "shape" in any scale, with ornaments | A contour, transposed and decorated | The Octopus's phrases and scales | No |
| `seq.jumps`: steps up the scale by how far the input is from C3 | A relative stepper | The Octopus's custom directions | No |
| `seq.strudel.mini`: a text pattern language, e.g. `[c4 g4 <f3 [f2 f4, c4]>]` | Written patterns | A language, not a style | No |
| `midi.rhythmes.alpes`: "fades / switches between subdivisions of a clock" | Time | Not about chords | No |

## 4. Drift

**One sentence.** A chord step that draws its notes at random draws them from a **loop of remembered draws** instead of
from fresh chance, and each time the loop comes round, some of the draws are replaced.

**The definition.** Per track, Drift holds a **tape** of `L` cells (default 16, 1 to 16), each a 32-bit number, and a
**lock** from 0 to 9 (default 6). A step that has a random pick to make (a chord whose polyphony differs from its size)
takes the next cell:

1. With probability `(9 - lock) / 9` the cell is first replaced by a fresh number. Lock 9 never replaces, lock 0 always
   does.
2. A small generator is started **from that cell** and does the step's draw exactly as the engine does it today (the same
   code path, the same without-replacement rule, the same rests). So the step's notes are a **pure function of its
   cell**.
3. The cursor moves on, and after the last cell goes to the first.

Nothing else in the step changes: velocity, length, strum, timing, the phrase and the rest are as they were. Steps that
make no pick (polyphony equal to size, or a single note) take no cell and sound as they do now.

**What it does to the music.** At lock 9 and `L` 8 the voicings of a track's random-pick steps come round every eight
such steps: chance, then repeated until it feels written. At lock 6 the loop is recognisable but slides. At lock 0 it is
the Octopus's own randomness. It is the Turing Machine's whole trick, applied to the one place the Octopus uses chance
on chords, and it is the Octopus's grammar: it adds nothing to the chord, the polyphony or the strum, it changes where
the randomness comes from.

**Why it is safe.**

- It is off by default, and with it off the code path is the Octopus's own, byte for byte (DR-1). The manual wins on
  musical behaviour; Drift is a WENGE extra, named as one.
- It has its own generator per track, so turning it on or off does not change what the engine's shared generator
  gives shuffle, directions or grooves (DR-6). Two runs with the same seed are equal.
- The manual's rule that the number of notes is "always the smaller of the two" holds for every setting (DR-5).

## 5. Where it lives, and what is touched

The Octopus has no control for this and the panel is the instrument's, so **no panel control is added** and
`contracts/controls.json` is not touched.

| Part | Zone | Content |
|---|---|---|
| The style | `crates/octocore/` (Metronome) | A `ChordStyle` per track: `Manual` (default) or `Drift { length, lock }`; the tape; the hook at the one place the chord branch draws |
| The way in | `apps/web/engine/` (Forge; the crate is `octoweb`) | One additive export, `octoweb_set_chord_style(track, style, length, lock)`. The ABI number stays, as it did for `octoweb_set_track` (ABI.md) |
| The button | `apps/web/` (Forge) | A `Drift` button in the strip that steps Off, Loose (lock 3), Loop (lock 7), Locked (lock 9) for all tracks |

**The frozen FFI.** `TrackAttr` is mirrored by a C enum in `crates/octoffi/octoffi.h`, which is frozen. Adding Drift as
new `TrackAttr` values would change that header and the wire format (a contract change: High, an ADR).
D3's default avoids it by giving the web build its own setter and leaving `Command`, `TrackAttr` and the header alone.
The cost: a host that is not the web page (the Max for Live spike) cannot set Drift. That is no loss today.

## 6. Observables (written red first, in the fixtures' style)

| # | Observable | Test |
|---|---|---|
| DR-1 | **Off is the Octopus.** With every track on `Manual`, the events of every conformance fixture and of the seed-by-seed chord tests are byte for byte what `main` makes. The 569 assertions and the baseline do not move | `conformance`, a new golden test over 200 seeds |
| DR-2 | **Locked, it loops.** At lock 9 and `L` 4, the notes of random-pick step `k` and `k + 4` are equal for 64 consecutive random-pick steps, for every pool size 2 to 7 and polyphony 1 to 7 | `octocore/tests/drift.rs` |
| DR-3 | **Open, it is chance.** At lock 0 each chord tone is chosen with the frequency the Octopus's draw gives, within 4 standard deviations over 4096 steps; no tone is favoured | `drift.rs` |
| DR-4 | **In between, it drifts at the rate the lock says.** The fraction of cells that differ between one pass of the tape and the next is `(9 - lock) / 9` within a bound from the binomial, for each lock 0 to 9, on a fixed seed | `drift.rs` |
| DR-5 | **The manual's count holds.** Whatever the style, the number of notes a step sounds is at most the smaller of chord size and polyphony, and exactly that when polyphony does not exceed the chord size (p.22) | `drift.rs` |
| DR-6 | **Isolated.** A pattern with shuffle, a random direction and a groove makes the same events for those features with Drift on and off, apart from the chord notes | `drift.rs` |
| DR-7 | **Deterministic.** Same seed and same settings give the same events; a different seed gives different ones; the tape starts from the seed again on a Reset and survives a Stop and a Continue | `drift.rs` |
| DR-8 | **No hang, no wrong note.** For every length, lock, pool size and polyphony, and for tape cells of 0, 1 and the largest number, the draw ends, picks distinct tones, and every note is a MIDI note (0 to 127) | `drift.rs` |
| DR-9 | **Settings are bounded.** A length of 0 or 17 and a lock of 10 or minus 1 are clamped, a track that does not exist changes nothing and returns the code the other setters return | `apps/web/engine` tests |
| DR-10 | **The page.** The button reaches the worklet; with a chord track the notes sent to a MIDI output differ between Off and Locked and repeat in Locked; the monitor plays what is sent | `drift.browser.test.ts` |
| DR-11 | **The strip keeps its rules.** The button uses the existing raised and pressed tones and tokens, adds no colour, gradient, shadow or easing | `app_source_rules.rs`, `app-css.test.ts` |
| DR-12 | **Not in the way.** The engine's per-block worst case (the soak and overload scenes) is no slower with Drift off, and the render path allocates nothing | existing soak gates |

The owner cannot hear a table, so S1 also produces a capture: the notes of 32 passes of a C-E-G step at locks 9, 6 and
0, as one picture to look at on a phone, drawn from the events the headless runner (`crates/octorun`) already writes
(as ndjson and as a MIDI file, so the owner can also hear them in any player).

## 7. Slices

| Slice | Content | Tier |
|---|---|---|
| **S0** | Pin the Octopus's draw: the golden test of DR-1 on `main`, before any change. It is green on day one, which is the point | Medium (Metronome) |
| **S1** | `ChordStyle` and the tape in `octocore`, with DR-2 to DR-8 and the capture | Medium (Metronome) |
| **S2** | `octoweb_set_chord_style`, the worklet message and the `Drift` button, with DR-9 to DR-11 | Medium (Forge) |
| **S3** | Drift on the single-note random rest, **after** the Metronome has built p.23 (F1). The same hook | Medium |
| **S4** | Optional: **Scatter** or **Walk** (section 3) as further values of `ChordStyle`, if the owner wants them (Q-S5-1) | Medium |

## 8. Decisions for the owner (defaults are taken; change any)

| # | Decision | Default and why |
|---|---|---|
| D1 | Which benny pattern | **Drift**, from the Turing machine: the one that is unlike anything the Octopus does and needs the least new vocabulary. Scatter and Walk are S4 |
| D2 | An extra that is not in the manual | **Yes, off by default and named as an extra.** The manual wins on musical behaviour; this adds a choice and removes none |
| D3 | How it reaches the engine | **A web-only setter** (section 5), leaving the frozen FFI, `Command`, the wire format and the contracts alone. The alternative, through the FFI, is High tier and an ADR |
| D4 | One cell per random-pick step, not per pick | Per step, so a loop of `L` is `L` such steps and lines up with the pattern. A step with three picks and a step with two take one cell each |
| D5 | The tape's start | From the seed, again on a Reset; kept across Stop and Continue, so a song resumes where it was. Play from the start after a Reset repeats |
| D6 | Length and lock | Default 16 and 6. The strip offers four presets, not two knobs; per-track settings need a track picker the strip does not have |
| D7 | Scope of the button | All tracks. A per-track choice waits for SPEC-0003's Track mode |
| D8 | Saved with the pattern | **No.** Drift is a setting of the page, not of the instrument's memory; a loaded pattern plays by the Octopus's rules until the button says otherwise |
| D9 | The name | "Drift". The strip and docs call it a WENGE extra |
| D10 | Licence | Idea only, described here and written from this text (section 3). The repository has no licence file; deciding one is the owner's |

## 9. Questions only the owner can answer

- **Q-S5-1.** Which benny pattern did you have in mind? If it was not the Turing machine, name it or describe what it did
  and the plan changes at section 3 only. Default: Drift.
- **Q-S5-2.** After the capture: is the loop at lock 6 recognisable as a loop to you, or should the presets be tighter?
  Default: Loose 3, Loop 7, Locked 9.

## 10. Risks

- **Two kinds of chance in one engine.** A reader of the manual sees one; the code has two. The golden test (DR-1) and
  the name "extra" are the guard.
- **A hidden second generator.** Each track's tape generator is seeded from the engine's seed and the track number; it
  must not be fed from the shared generator, or DR-6 fails.
- **The strip grows.** One more button on a strip that is already full on a phone. The strip's own tests (width, tab
  order, the rules) are the gate, and D6's presets keep it to one control.
- **Taste.** The numbers show it loops and drifts at the stated rate; whether it is a good idea musically is Q-S5-2.

## 11. Not done

Any panel control, any change to `contracts/`, the FFI, the wire format or the saved pattern, the single-note rest (F1,
requested), and any other benny block.

## 12. Sources

- benny, `https://github.com/playbenny/benny/`, the `help_text` of `note_blocks/seq.turing.json`, `midi.delay.json`, `midi.fidget.json` and others, read 2026-10-16; its `LICENSE.md` (Hippocratic License 3.0).
- Music Thing Modular, the Turing Machine (a random looping sequencer with a "probability" knob), the idea benny's block implements. Nothing of its firmware or schematic is used.
- Octopus Reference Manual CE v5.30, p.20 to 23 (`reference/manual/pages/p020.txt` to `p023.txt`).
- This repository: `crates/octocore/src/engine.rs` (the chord branch), `rng.rs`, `types.rs`, `crates/octoffi/octoffi.h`, `apps/web/engine/ABI.md`.

## 13. Approval record

Not yet approved. Awaiting `@beaudesign`. Merging is the owner's act; the agent merges nothing.

# SPEC-0005: A chord style that bends time (curved strum, from benny's curved time)

| Field | Value |
|---|---|
| Spec | SPEC-0005 |
| Revision | r1.2, 2026-10-17. r0 (merged 2026-10-16, #67) picked a different benny block, Drift. The owner's reply of 2026-10-17 11:44 named the block, so r1 changes the pick and keeps Drift as a runner-up. r1.1 records the owner's go-ahead (section 14). r1.2 states what an independent review of S0 to S2 found (section 14) |
| Stage | **S0 to S2 approved to build (section 14); S3 and S4 not.** This pull request amends the spec and adds journal entries. It changes no code, contract, fixture or threshold. The slices are their own pull requests (`WENGE-0033` Metronome: S0 and S1; `WENGE-0034` Forge: S2) |
| Risk tier | **Low** for this pull request (`specs/`, `journal/`). The slices are **Medium** under D3's default (a web-only extension: `crates/octocore`, `apps/web/engine`, `apps/web/`) and **High** if the owner picks the frozen FFI (`crates/octoffi/`, `contracts/`) |
| Owner and approver | `@beaudesign` |
| Task | `WENGE-0031` (the spec); `WENGE-0033` (S0 and S1); `WENGE-0034` (S2) |
| Extends | SPEC-0003 (the circle: chords, polyphony and strum are the CHORD block's). Needs none of its slices |
| Sister spec | SPEC-0004 (a sound that behaves like hardware). The two do not depend on each other |
| Base | `main` at `12c7938` |

## 1. The ask

Recorded here because chat is not a source of truth.

- Fri 2026-10-16, 17:15 London, from a phone: "let's explore a sequencer pattern which was unique and unusual as an
  additional style to add to chords which is also in benny" (`https://github.com/playbenny/benny/`).
- Sat 2026-10-17, 11:44, with two screenshots of benny: "It's this one in benny".

**The screenshots, in words** (they are not stored in the repository). The first is benny's **`seq.curved.time`** block:
a plot of a rising curve that restarts each cycle, with evenly spaced thresholds crossed by it, and white ticks where it
crosses them, bunched toward the end of each cycle. Its controls: divisions 4, bar length 4 beats, counter length 4, and
two internal phasors, the first with rate 2 per bar, phase offset 0, **warp -0.30**, gain 1.00 and the second with rate 2,
phase offset 0, **warp -0.11**, gain 0.25. Beside it benny's patch graph: clock, **curved time, shape player**, a basic
voice, a reverb, the output. The second screenshot is **`core.scales.shapes`**: a keyboard with a scale lit, a stepped
contour below it, a "patterns in" menu and a loop switch.

**The reading.** The pattern is **curved time**: notes whose times are not even but follow a curve, so that a run
accelerates or drags. The second block is its partner in benny's chain: `curved.time` is the clock, `shape.player`
plays a stored shape of notes (kept by `scales.shapes`) on it. "A style to add to chords" is read as: **a chord, or a
repeated note, that is played as a roll along a curve**, next to the three chord behaviours the Octopus has (section 2).
The shape player's own ornaments are kept as a later candidate (section 12). If the owner meant something wider
(Q-S5-2 and Q-S5-3), it costs a paragraph here and nothing in the code.

## 2. What chords do today (the manual's words, then the engine's)

The Octopus has three chord behaviours, all on the CHORD block (manual p.20 to 23):

1. **The chord.** Up to seven notes on a step: the base pitch and up to six offsets. Polyphony equal to the chord size
   plays them all.
2. **Random picks from the chord pool** (p.22): polyphony different from the chord size draws that many notes, padding
   with rests when it is larger.
3. **The strum** (p.22). A level from minus 9 to 9, set by holding a chord button and turning the main rotary. Green is up,
   red is down. The notes are delayed by a **fixed table of ticks**, "preset", and "the strum effect increases
   exponentially with the strum level". At level 9 the notes 2 to 7 start at +5, +10, +17, +23, +30 and +45 ticks. The
   manual adds: a strum on a **single note** plays that note "followed by 6 duplicate notes played at the set strum
   timing".

**How the engine does the third one.** In `crates/octocore/src/engine.rs` (the step-firing code, near line 1450) the
level is the step's `strum` clamped to minus 9 to 9; a negative level reverses the order of the notes; each note's start
is `tick_due + on_tick_offset + strum_offset_ticks(level, k)`, where `strum_offset_ticks` in `tables.rs` is the manual's
table, verbatim. A single note is scheduled seven times in the same way. **That table is the only shape the Octopus's roll
has**: for seven notes at level 9 the gaps between notes are 5, 5, 7, 6, 7 and 15 ticks: a roll that always drags toward
its end.

**Finding F1, not part of this ask (unchanged from r0).** The manual's p.23 says the pool trick also works "on single
notes": one note with polyphony 2 is a 50 percent rest. The engine's branch for a step with no chord never reads the
polyphony, so a single note always plays. I have not fixed it (not my zone) and it is filed as
`journal/conductor/requests/2026-10-16-random-step-rest.md` for the Metronome. It does not touch Curve.

## 3. What benny's curved time is, and why it fits

From the block's own description (`audio_blocks/seq.curved.time.json`, read 2026-10-17): "This sequencer is about
bending time. It sums the internal phasors and the audio input into a timing signal. When this crosses the thresholds
(set as even divisions of the 0-1 range) a trigger is emitted. Using interesting timing signals will create interesting
rhythms." Its outputs are a trigger, the division number, a counter and a counter wrap, and "you can use the trigger
events as a clock to drive other sequencers".

Put plainly: a ramp is bent by a **warp** (flat then steep, or steep then flat), and an event fires each time the bent
ramp crosses the next of **D evenly spaced levels**. Straight ramp, even events. Bent ramp, events that bunch together
at one end: an accelerating roll, like a bouncing ball. Two ramps of different rates added together make an irregular
pattern that still repeats. The Octopus has nothing like it: its strum is one fixed table, and its step clock is
even except for groove and shuffle.

**Why it fits a chord.** A chord roll is the same thing in small: n notes, each at a time, between "all together" and
"spread over a span". The Octopus fixes the span by the strum level and the spacing by one table. Curved time asks the
other question: **given the span, how are the notes spaced along it?** That is the whole of the style, and it uses
the Octopus's own vocabulary: the strum level still means "how long the roll lasts".

**The licence, as in SPEC-0004 section 3.** benny is under the Hippocratic License 3.0 (ethical conditions, a notice duty
and copyleft), and this repository has no licence file. So this spec takes **the idea only**. The curve below is a few
lines of arithmetic that is written out here in full and is written from this text; nothing of benny's patches, code or
presets is read into it.

## 4. Curve

**One sentence.** A strummed chord (or a strummed single note) is played with its notes spaced along a **bent curve** from
its first note to its last, instead of along the Octopus's table, with the same span.

**The definition.** `Curve` is a style with a **warp** from minus 1 to 1 (0 is even). For a roll of `n` notes (a chord's
notes, or 7 for a single note) at strum level `L`:

- the span is the table's own end point, `T = strum_offset_ticks(L, n)`. So a level still means what it means.
- the start of note `k` (1 to `n`) is `round(T * W((k - 1) / (n - 1)))` ticks after the first, where
  `W(x) = (e^(a x) - 1) / (e^a - 1)` with `a = 4 * warp`, and `W(x) = x` when the warp is 0. `W` runs from 0 to 1.
  Rounding is half up.
- a negative warp puts the notes close together **late** (accelerating, the bouncing ball); a positive warp puts them
  close together **early** (dragging). Warp `w` and warp `-w` are exact mirror images: `W_-w(x) = 1 - W_w(1 - x)`.
- the engine reads **integers, not `exp`**: the offsets for each warp, level and size are a pinned table (like the
  manual's), generated by a script from this formula and committed, so a native build and the wasm build cannot differ
  by a rounding.

**Worked example.** Seven notes, level 9 (T = 45 ticks, about 470 ms at 120 BPM, 48 ticks to the quarter note):

| Style | Start of notes 1 to 7 (ticks) | Gaps |
|---|---|---|
| The Octopus's table | 0, 5, 10, 17, 23, 30, 45 | 5, 5, 7, 6, 7, 15 |
| Curve, warp -1 ("Bounce") | 0, 22, 34, 40, 43, 44, 45 | 22, 12, 6, 3, 1, 1 |
| Curve, warp -0.5 ("Accel") | 0, 15, 25, 33, 38, 42, 45 | 15, 10, 8, 5, 4, 3 |
| Curve, warp 0.5 ("Rit") | 0, 3, 7, 12, 20, 30, 45 | 3, 4, 5, 8, 10, 15 |

A single note at the same level with warp -1 is a **bouncing-ball ratchet**: seven hits, the first gap about half the
span and each later one shorter. That is the sound of the screenshot's plot: ticks that bunch toward the end of a cycle.

**What it changes, and what it does not.** It changes **when each note of a roll starts**, and nothing else: which notes,
their order (a negative strum level still reverses it), velocity, length, channel, port, the random picks, the phrase
and the rest are as they were. It draws no random number, so every other feature's randomness is untouched.

**Why it is safe.** It is off by default, and with it off the table is used byte for byte (CV-1). The manual wins on
musical behaviour; Curve is a WENGE extra and is named as one. Its offsets are never negative and never above the
table's own largest (45 ticks), so the engine's guarantee that "strum only ever pushes a note later" and its early-note
lookahead are as they were. The curve is **musical timing inside the engine, not interface motion**: CLAUDE.md rule 5
is about the interface and is not touched.

## 5. Where it lives, and what is touched

The Octopus has no control for this and the panel is the instrument's, so **no panel control is added** and
`contracts/controls.json` is not touched.

| Part | Zone | Content |
|---|---|---|
| The style | `crates/octocore/` (Metronome) | A `ChordStyle` per track: `Manual` (default) or `Curve { warp }`; the pinned offset table; the hook at the one call to `strum_offset_ticks` in the step firing, for chords and for the single-note repeat |
| The way in | `apps/web/engine/` (Forge; the crate is `octoweb`) | One additive export, `octoweb_set_chord_style(track, style, warp)`. The ABI number stays, as it did for `octoweb_set_track` (ABI.md) |
| The button | `apps/web/` (Forge) | A `Curve` button in the strip that steps Off, Accel, Bounce, Rit for all tracks |

**The frozen FFI.** `TrackAttr` is mirrored by a C enum in `crates/octoffi/octoffi.h`, which is frozen. Adding the style as
new `TrackAttr` values would change that header and the wire format (a contract change: High, an ADR). D3's default
avoids it by giving the web build its own setter. The cost: a host that is not the web page cannot set it. That is no
loss today.

## 6. Observables (written red first, in the fixtures' style)

| # | Observable | Test |
|---|---|---|
| CV-1 | **Off is the Octopus.** With every track on `Manual`, the events of every conformance fixture, and of a new golden over strum levels minus 9 to 9, chord sizes 1 to 7 and 200 seeds, are byte for byte what `main` makes. The 569 assertions and the baseline do not move | `conformance`, a golden test |
| CV-2 | **The span is kept.** For every level 1 to 9, size 2 to 7 and warp, note 1 starts at 0, note `n` at `strum_offset_ticks(level, n)`, and the starts never decrease | `octocore/tests/curve.rs` |
| CV-3 | **Warp 0 is even.** The gaps between notes differ by at most one tick | `curve.rs` |
| CV-4 | **The sign means what it says, and is symmetric.** Warp below 0 gives gaps that never grow by more than one tick; above 0 gaps that never shrink by more than one tick; the gaps of warp `-w`, read backwards, equal those of `w` within one tick. (Rounding to whole ticks makes 18 of the 270 table cells swap two neighbouring gaps by one tick against the sign, for example Accel at level 6 with seven notes has gaps 8, 6, 4, 2, 3, 1. r1 said "never"; the engine and its test were always "within one tick".) | `curve.rs` |
| CV-5 | **Only the starts move.** For any step, the set of (pitch, velocity, length, channel, port) is identical with Curve and with Manual; a negative strum level still reverses the order; level 0 is no roll in every style | `curve.rs` |
| CV-6 | **The ratchet.** A single note with level above 0 is scheduled 7 times, first at 0, and the gaps follow CV-4 | `curve.rs` |
| CV-7 | **No randomness.** The engine's random stream (shuffle, a random direction, groove, chord picks) is identical with Curve on and off | `curve.rs` |
| CV-8 | **Within the lookahead.** Every offset is at most 45 ticks and never negative, and the engine's timing and early-note tests pass with Curve on at level 9 on all ten tracks (the overload scene) | existing timing gates, `curve.rs` |
| CV-9 | **The table is pinned.** The committed offset table equals what the generator script makes from the formula (half up), so the table cannot drift from this spec and no build computes `exp` | `curve.rs`, a script test |
| CV-10 | **Settings are bounded.** With the style Curve, a warp outside the five stored values, a style that does not exist and a track that does not exist change nothing and return the code the other setters return. With the style Manual the warp is ignored. A number that is not an integer reaches the export as the integer JavaScript makes of it (`NaN` is 0, 2^32 is 0), so only a page that sends the presets is held to this; the page does | `apps/web/engine` tests |
| CV-11 | **The page.** The button reaches the worklet; for a chord track at strum 9 the notes sent to a MIDI output start at the offsets of CV-2 for each preset, and the monitor plays them | `curve.browser.test.ts` |
| CV-12 | **The strip keeps its rules.** The button uses the existing raised and pressed tones and tokens and adds no colour, gradient, shadow or easing | `app_source_rules.rs`, `app-css.test.ts` |
| CV-13 | **Not in the way.** The engine's soak and overload scenes are no slower with Curve off, and the render path allocates nothing | existing soak gates |

The owner cannot hear a table, so S1 also produces a capture: the notes of a C-E-G-B-D chord at level 9 and of a single
note, under each preset, as one picture and as a MIDI file, from the events the headless runner (`crates/octorun`)
already writes, to look at and play on a phone.

## 7. Slices

| Slice | Content | Tier |
|---|---|---|
| **S0** | Pin the Octopus's roll: the golden test of CV-1 on `main`, before any change. It is green on day one, which is the point | Medium (Metronome) |
| **S1** | `ChordStyle`, the pinned table and its generator, the hook, CV-2 to CV-9, and the capture | Medium (Metronome) |
| **S2** | `octoweb_set_chord_style`, the worklet message and the `Curve` button, with CV-10 to CV-12 | Medium (Forge) |
| **S3** | **Two ramps.** A second warped ramp of another rate added to the first, as in benny's block, for rolls that are irregular and still repeat. Needs its own design pass (the sum can run backwards) | Medium |
| **S4** | Optional, from section 12: Shape ornaments, Drift, Scatter or Walk, as further values of `ChordStyle` | Medium |

## 8. Decisions for the owner (defaults are taken; change any)

| # | Decision | Default and why |
|---|---|---|
| D1 | Which benny pattern | **Curved time**, applied to the chord roll and the single-note repeat. It is the block in the owner's screenshots. The rest are section 12 |
| D2 | An extra that is not in the manual | **Yes, off by default and named as an extra.** The manual wins on musical behaviour; this adds a choice and removes none |
| D3 | How it reaches the engine | **A web-only setter** (section 5), leaving the frozen FFI, `Command`, the wire format and the contracts alone. The alternative is High tier and an ADR |
| D4 | The span | **The manual's own end point** for the level and size, so the strum level means what it meant. The alternative, a span of its own, would need a second control |
| D5 | The curve family and the numbers | `W` as in section 4, five stored warps (-1, -0.5, 0, 0.5, 1), as a **pinned integer table**. Continuous warp and a floating-point curve in the engine are not planned |
| D6 | The control | One `Curve` button for all tracks: Off, Accel (-0.5), Bounce (-1), Rit (0.5). Even (0) and Drag (1) are stored and testable and not on the button, to keep it to four states. Per-track waits for a track picker the strip does not have, and for SPEC-0003's Track mode |
| D7 | Saved with the pattern | **No.** The style is a setting of the page, not of the instrument's memory; a loaded pattern plays by the Octopus's rules until the button says otherwise |
| D8 | benny's clock is a bar clock, the Octopus's step clock is not bent | **Not planned.** Bending when every step fires is a change to the timing path (High tier, ADR-0009's clock) and the Octopus already has start offsets (STA), groove and shuffle. If the owner wants "curved steps", it is its own spec (Q-S5-3) |
| D9 | The name | "Curve". The strip and docs call it a WENGE extra |
| D10 | Licence | Idea only, written out here (section 3). The repository has no licence file; deciding one is the owner's |

## 9. Questions only the owner can answer

- **Q-S5-1 (answered 2026-10-17).** Which benny block? "It's this one": curved time. Recorded in section 1.
- **Q-S5-2 (answered 2026-10-18: keep the default).** The second screenshot is `core.scales.shapes`, the store the shape player reads. Did you mean that as well,
  that is, storing the order of the notes you play and replaying them with flam, trill, arp and skip? Default: **not in
  this spec's first slices**; it is the S4 candidate "Shape ornaments" (section 12).
- **Q-S5-3 (answered 2026-10-18: keep the default).** Do you want the bend on the chord roll and the single-note ratchet only (the default), or also on the step
  clock, so that steps through a bar speed up and slow down? The second is a different, larger spec (D8).
- **Q-S5-4 (answered 2026-10-18: Off, Accel, Bounce, Rit).** After the capture: is Bounce the sound you meant, or should it be gentler? Default: Accel (-0.5) first on
  the button, Bounce second.

## 10. Risks

- **A second timing for a roll.** A reader of the manual sees one strum table; the code would have two behaviours. The
  golden (CV-1), the name "extra" and the default Off are the guard.
- **Rounding differences between builds.** Avoided by pinning integer tables (D5, CV-9).
- **Rolls longer than a step.** A step is 12 ticks and the longest roll is 45, so a roll can run past the next steps. The
  table already allows it at high levels, and Curve never exceeds it.
- **The strip grows.** One more button on a strip that is already full on a phone. The strip's own tests are the gate.
- **Taste.** The numbers show the curve is what it says; whether Bounce is musical was Q-S5-4, answered 2026-10-18 with the default order (Off, Accel, Bounce, Rit).
- **The button is silent on the page as it stands.** The panel writes only Active, Skip, Velocity, Pitch, Length and Start; the
  page has no way to enter a chord or a strum, and the demo has none, so Off, Accel, Bounce and Rit give the same 242 events and
  the same hash there. The style is heard on patterns that carry strums (the octorun patterns and the capture). A way to strum
  on the page is a panel change and is not in S0 to S2 (S3 and S4 are not approved); it is the owner's call, made 2026-10-18: wait for chord
  entry. The button stays on the page as S2 built it, silent until chords can be entered there.
- **Steep curves put notes on one tick.** On a whole-tick grid, Bounce and Drag put two notes on the same tick in 30 of the 45
  (level, size 3 to 7) rows, against 9 for Manual; the last notes of a Bounce roll arrive together (level 6, seven notes: starts
  0, 12, 18, 21, 23, 24, 24). That is the curve, not a defect; the capture lets the owner hear it.
- **The queue is a little tighter.** Seven-note chords at strum 9 on every step of all ten tracks at 2 times speed and 120 BPM:
  Manual peaks at 920 of 1024 slots and refuses nothing; Bounce reaches 1024 and refuses whole notes (1,975 in 10 seconds,
  17,245 at 999 BPM), because a curve bunches notes into the lookahead. The refusal is the queue's own whole-note rule. CV-8's
  scenes cover 1 times (650 of 1024) and a 4 times overload; this one is outside them.
- **A chain plays in its head's style.** A chained track is driven by its head, which owns the cursor, so a style set on a
  member does nothing. The page sets all ten tracks alike, so it makes no difference there. Pinned by
  `a_chain_plays_in_its_heads_style`.

## 11. Not done

Any panel control, any change to `contracts/`, the FFI, the wire format or the saved pattern, the single-note rest (F1,
requested), a bent step clock (D8), and any other benny block.

## 12. Runner-ups, kept

- **Shape ornaments** (`core.scales.shapes` and `seq.shape.player`: "stores the 'shape' of a pattern played into it as
  well as the scale", replays it "transposed in-scale, rotated in time" with ornaments: skip, double, flam, swap, trill,
  arp, sub loop, back, forward). The Octopus has phrases and scales for much of this. The ornaments that suit chords are
  flam (a two-note strum), trill and arp (the chord notes in sequence). Needs a way to enter order, which the Octopus's
  sorted chord does not keep.
- **Drift** (r0's pick, from `seq.turing`): chance with a memory. A chord whose notes are drawn at random draws them from
  a looping tape of seeds, a lock setting how many are replaced each pass; lock 9 repeats, lock 0 is the Octopus's own
  randomness. It changes only where the randomness comes from, with its own generator per track, a pure function of the
  tape cell for each step, and observables for it (off is the Octopus; locked it loops; open it is the Octopus's
  frequencies; the manual's note count holds; deterministic; no hang). Its full text is r0: `git show d37eeab:specs/SPEC-0005/README.md`.
- **Scatter** (`midi.delay`: random order and spacing of a chord's notes) and **Walk** (`midi.fidget`: a hop to a
  neighbouring chord tone).

## 13. Sources

- benny, `https://github.com/playbenny/benny/`: `audio_blocks/seq.curved.time.json`, `note_blocks/seq.shape.player.json` and `core.scales.shapes.json` (descriptions and parameter names), and its `LICENSE.md` (Hippocratic License 3.0), read 2026-10-16 and 2026-10-17; the owner's two screenshots of benny, 2026-10-17.
- Octopus Reference Manual CE v5.30, p.20 to 23 (`reference/manual/pages/p020.txt` to `p023.txt`), the strumming page p.22 for the table.
- This repository: `crates/octocore/src/engine.rs` (the step firing), `tables.rs` (`strum_offset_ticks`), `types.rs`, `crates/octoffi/octoffi.h`, `apps/web/engine/ABI.md`.

## 14. Revisions and approval record

- r0, 2026-10-16: Drift. Merged as #67 by the owner. Not an approval.
- r1, 2026-10-17: Curve, after the owner's reply. Pull request #68.
- r1.1, 2026-10-17: the approval record below, and the task ids for the slices. Nothing else changes.
- r1.3, 2026-10-18: the owner's answers to Q-S5-2, Q-S5-3 and Q-S5-4, the silent button and S3, all on the defaults already recorded; closed in section 9 and recorded below. No slice, decision (D1 to D10) or number changes.
- r1.2, 2026-10-17: after an independent review of S0 to S2 (tests, mutation and a re-derivation of the table: no engine defect, Manual byte for byte). CV-4 and CV-10 now say what the code and tests always did, and section 10 records what the review found: the button is silent on the page as it stands, steep curves put notes on one tick, the queue is tighter, and a chain plays in its head's style. Three engine tests were added (`WENGE-0036`: the roll for the notes that sound, a style per track, a chain's head). No approval changes.

**r1.1, 2026-10-17, 14:50 (Europe/London).** While the agent was building SPEC-0004's first slice (the owner's "Okay build"
at 14:46), the owner wrote **"Let's build r1 - Benny's curved time as well"**. The words are the owner's own, they name
this spec and its revision, and they ask for it to be built; that is an approval, and it is written here with the date
and the reading, because chat is not a source of truth. The agent reads "build r1" as the spec's first three slices, which
together make the feature work end to end (the golden pin, the engine, the button). It reads the spec's own words as
limiting the rest: S3 "needs its own design pass" and S4 is "optional". If the reading is wrong, say so and this record is
revised. (The merge of #67, which carried r0, was not an approval. #68 is open and unmerged: the agent builds on its
branch and merges nothing.)

| Item | What is approved | What is not |
|---|---|---|
| This spec as the plan | Yes, r1, with the defaults D1 to D10 and the open questions on their defaults (Q-S5-2: no shape ornaments yet; Q-S5-3: the bend stays on the roll, not the step clock; Q-S5-4: Accel first on the button, Bounce second) | |
| **S0** the golden pin (CV-1) | **Yes, build it** (`WENGE-0033`, Metronome) | |
| **S1** `ChordStyle`, the pinned table and its generator, the hook, CV-2 to CV-9, the capture (`WENGE-0033`, Metronome) | **Yes, build it** | |
| **S2** `octoweb_set_chord_style`, the worklet message, the `Curve` button, CV-10 to CV-12 (`WENGE-0034`, Forge) | **Yes, build it**, after S1's engine API exists | |
| S3 two ramps | | **No.** It needs a design pass first and the owner's ear on S1's capture (Q-S5-4) |
| S4 runner-ups (Shape ornaments, Drift, Scatter, Walk) | | **No.** Each is its own decision. The shape player is also the subject of a separate engineering spec the owner has not yet attached |
| The frozen FFI, `contracts/`, the wire format | | **No.** D3's web-only setter is the default and is what is approved; the High-tier alternative is not |

Merging is the owner's act; the agent merges nothing. A merge is not an approval: approval is the owner's own words,
written here with the date.

**r1.3, 2026-10-18 (Europe/London), after the owner's message "Continue q-s5" at 20:03.** The agent put SPEC-0005's open questions to the owner as
four multiple-choice questions in chat, and the owner chose the answer marked as the default in each. The words are the owner's own selections, written
here with the date and the reading, because chat is not a source of truth.

| Question | The owner chose | Reading |
|---|---|---|
| Q-S5-4, the order of the Curve button | "Off, Accel, Bounce, Rit (Recommended)" | The button order stays as D6 and S2 built it: Accel (-0.5) first, Bounce (-1) second. The question asked after the renders; the agent does not know whether the owner listened and takes the answer as given. Bounce is not made gentler and is not taken off the button |
| What to do about the silent button | "Wait for chord entry (Recommended)" | The button stays on the page as built, silent until chords can be entered there (SPEC-0003). Nothing new is approved: no strummed demo step, no new control, no revert |
| S3 two ramps | "Not yet (Recommended)" | S3 stays **not approved**, as in the record above. No design pass is asked for either; it waits for the owner's ear on the strip with a chord |
| Q-S5-2 and Q-S5-3 | "Keep both defaults (Recommended)" | No shape ornaments in this spec (the shape player is the separate plan in #71, `handoffs/eng-spec-shape-player-and-complex-voice-v2.md`), and the bend stays on the chord roll and the ratchet, not the step clock |

Nothing in this entry approves a slice. S3 and S4 are still not approved, the High-tier setter alternative is still not approved, and a merge is still not
an approval. #68 and #77 (the S2 strip carried onto this branch, because #75 was merged into a branch that is not this one) are open and unmerged.

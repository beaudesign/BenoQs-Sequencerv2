# SPEC-0003: The circle, the part of the Octopus that makes its patterns unique

| Field | Value |
|---|---|
| Spec | SPEC-0003 |
| Revision | r1, approved with all defaults, 2026-10-13 13:41 (r0 was the draft) |
| Stage | **Approved by the owner (see the approval record). This pull request is still planning only: it adds the spec and a journal entry, and changes no code, contract, fixture or threshold. S0 (ADR-0012) is the first pull request of the series; nothing has been built.** |
| Risk tier | **Low** for this pull request (`specs/`, `journal/`). The work it plans is **Medium** for each slice and **High** for S0, which changes `contracts/` |
| Owner and approver | `@beaudesign` |
| Task | `WENGE-0028` (`handoffs/WENGE-0028.ndjson`, written with the first slice) |
| Extends | SPEC-0002: this is the circle half of its wave 2 ("Shape"). The matrix half stays there |
| Base | `main` at `6c789e7` |

## The ask

Recorded here because chat is not a source of truth. On 2026-10-13 (12:56 London) the owner wrote, with the manual
attached (`Octopus_ReferenceManual_v530-2.pdf`):

> The sequence cycles on the right side serve very specific and useful purposes for sequencer and creating midi orders
> in unique patterns - read through the full reference manual, then plan a new spec to include them in the midi
> application -- this should give you a base of how the features should work and the USP of the sequencer

**The reading.** "Cycles" is read as the circles: the outer circle (the numeric quadrant and the ring of number keys and
named keys), the inner circle (the pitch and scale ring), and the blocks round them (CHORD, TRANSPORT, MODE). "Midi
orders in unique patterns" is read as the ways the Octopus orders, times and varies what it plays. If the owner meant
something else, say so: it costs one paragraph here and nothing in the code.

## What this spec proposes

1. **Name the circle** (`product.md` section 1): it is where a pattern is given its meaning, and today, of its 42
   controls, the page binds five.
2. **State the USP** (`product.md` section 2) as five sources of variation, each set from the circle: *order*
   (programmable directions with a probability of going forward or back), *time* (a clock multiplier per track,
   chains, hypersteps), *pitch* (scales, chords, strum), *chance* (polyphony and rests, random slices) and
   *modulation* (step events and the effector). The engine already holds most of it. What is missing is the way to reach
   it.
3. **Account for every control** (`product.md` section 3) and every feature (section 4) with the manual's page, and the
   place the manual is silent.
4. **Audit the repo** against that (`tech.md` section 1, `findings.md` section 2): the circle works nowhere from the
   panel today.
5. **Plan nine slices** (`plan.md`), each a family of fixtures written red first, about 100 in all, plus four tutorial
   walk-throughs from the manual as the end-to-end check that the USP works from the panel alone.
6. **Ask 14 new questions** (`findings.md` section 4, Q43 to Q56) that only a person with an Octopus can answer, each with
   a default that lets the work go on.

## Findings that shape the plan

Evidence is in `findings.md`.

- **F1.** The controller binds five of the circle's 42 controls. The circle has no display anywhere (F2).
- **F3.** The displays are LEDs on keys the plate has, as far as the manual says. So the design needs no widget and no
  token, and none of it touches the look that PRs #56 to #58 set.
- **F4.** Number entry is one stated rule: a single click sets the ones, a double click the tens [p17, p57].
- **F5.** Thirteen keys on the outer ring have no identity the manual states. This is the largest unknown and a person
  with the hardware can close it in minutes.
- **F6.** About half the circle lives in Track and Grid modes, which are not built. The series is cut at a gate.
- **F7.** The engine already does most of the USP. It is an interface problem.
- **F8.** The 2007 tutorials are button-by-button: they are fixtures waiting to be written.

## Decisions for the owner

Defaults apply if you say "approve, all defaults", as in SPEC-0001 and SPEC-0002. **Answered in r1: all defaults apply** (approval record).

| # | Decision | Default | Alternatives |
|---|---|---|---|
| D1 | Approve this spec as the plan for the circle half of wave 2, with the exclusions of `product.md` section 6 (control maps, recording, transpose, SysEx, Grid and performance modes) | Yes | No; or approve only S0 to S2 |
| D2 | Slice order | S1 to S6 (everything reachable from Page and Step mode), then Gate T, then S7 and S8; S9 any time after S1 | USP first: build Track mode now and take S8 (directions) and S7 before S3 to S6 |
| D3 | When Track mode (the matrix half of wave 2) is built, as Gate T | After S6, as its own amendment to SPEC-0002. The Panelwright is one hat in one crate, so it serialises with the slices | Now, ahead of S1 |
| D4 | How the numeric quadrant is read | LEDs, as the hardware, plus a sentence in the accessibility tree. No printed digits on the plate. The strip keeps the readable tempo | A quiet printed readout on the plate in the plate's ink, for people with no hardware beside them |
| D5 | The thirteen unnamed ring keys | Ids by position (`circle.ring.1` to `.13`), flagged; functions keep their own ids and are bound to a key when the owner, or a sharper reading of the ring, says which it is. You are asked, once, to read the ring's names to me | Wait for the answer before S0 |
| D6 | Holds on a mouse | The latch of SPEC-0002 `tech.md` section 5: Shift-click or a long press latches a hold, shown by the held control's LED flashing; keyboard hold keys; real chords on a touch screen. Needed before S4 | Another gesture; or hold-and-click only |
| D7 | Where the manual is silent | A controller may ship a **named provisional choice** (comment, `QUESTIONS.md`, the number of the question) when it is one function or one table, as P5 did. No fixture asserts it | Pending only: the feature waits for the hardware |
| D8 | One state, two views | The strip's Tempo and MIDI Clock select and the panel's Tempo and 200 keys are one value (CA6) | Keep the strip separate |
| D9 | Chance and replay (Q49) | The engine's generator is seeded by the host. The page seeds it from the clock when it starts and keeps the seed in the saved state, so any session replays exactly | A fixed seed always; or no replay |
| D10 | The 2007 tutorials as fixtures | Yes, each citing the tutorial and, where it differs, the v5.30 page that wins | Pages of v5.30 only |

## Series

`plan.md` has each slice's observables. The agent merges nothing, and no pull request is stacked on another.

| Slice | What it gives the user | Tier |
|---|---|---|
| S0 | The ring is in the inventory. Nothing visible | High |
| S0b | The controller is split into modules. Nothing visible | Medium |
| S1 | A knob that sets the tempo, number keys that key it in, bank view | Medium |
| S2 | A pitch ring that reads and enters pitch | Medium |
| S3 | Scales: force, compose, base, lock, cadence | Medium |
| S4 | Chords, octave view, strum, polyphony and chance | Medium |
| S5 | Chains and hypersteps: tracks that play tracks | Medium |
| S6 | Step events and the effector: tracks that change tracks | Medium |
| Gate T | Track mode (SPEC-0002) | |
| S7 | Track clock multipliers, step length multipliers, pause, program and bank, map factors | Medium |
| S8 | Direction maps and `certainty_next`: the order a track plays in | Medium |
| S9 | The 200 key, the overload LEDs | Medium |

## What happens to work in flight

- **PRs #56 to #58** (the Linear look, the corner, Drums) are unaffected. The circle's readouts are LED writes, so no
  token moves; a sentence for the accessibility tree is plain text.
- **SPEC-0002 wave 2** loses its circle half to this spec. On approval a line goes in its Amendments, as ADR-0010 did for
  D3. It is not in this pull request.
- **P7's layout counts** (242 drawn, 5 not) change in S0, by ADR, upward.
- **The pending fixture** `the_numeric_quadrant_and_the_inner_circle_show_velocity_and_pitch` (Q42) is picked up by S2.
- **The Metronome** receives its requests slice by slice (`tech.md` section 5), not now.

## Files

| File | Contents |
|---|---|
| `product.md` | The circle, the USP, every control and feature with its page, the acceptance criteria |
| `tech.md` | What exists, the design (display model, holds, engine surface, controller modules), contracts, risks |
| `plan.md` | The nine slices with their observables and fixtures, the tutorial fixtures, what "done" means |
| `findings.md` | What was read, the circle against the repo, Q43 to Q56 and the open questions this spec leans on |

## Approval record

**r1, 2026-10-13, 13:41 (Europe/London).** The owner had been asked to answer D1 to D10 or to say "approve, all
defaults". The owner replied **"approve, all defaults"**. The agent reads that as the second option, because it answers
that question and names no change to a default. Chat is not a source of truth, so the reading is written down here. If it
is wrong, say so and this record is revised. (The owner's earlier "continue", a minute before, was not taken as an
approval; nothing was started on it.)

| Item | What is approved | What is not |
|---|---|---|
| D1 the plan | This spec as the plan for the circle half of SPEC-0002 wave 2, with the exclusions of `product.md` section 6 | Merging. The agent never merges |
| D2 order | S1 to S6, Gate T, S7 and S8; S9 any time after S1 | Starting a slice before the one before it has merged |
| D3 Track mode | Gate T after S6, as its own amendment to SPEC-0002 | Building Track mode inside this series |
| D4 the quadrant | LEDs as the hardware, plus a sentence for the accessibility tree; no printed digits | A printed readout |
| D5 the ring keys | Ids by position (`circle.ring.1` to `.13`), flagged. **The owner is still asked, once, to read the thirteen printed names**; it is not required for S0 | Binding any function to a ring key before it is answered |
| D6 holds | The latch (Shift-click or a long press), keyboard hold keys, real chords on touch | Starting S4 before S3 has merged: D6 is needed there |
| D7 provisional choices | Named provisional choices where the manual is silent, one function or one table each | Asserting any of them in a fixture |
| D8 one state | The strip's Tempo and clock select and the panel's are one value (CA6) | |
| D9 chance and replay | The engine's generator is seeded by the host; the page seeds it from the clock and keeps the seed in the saved state | Any change to the engine's generator in this series without the Metronome's request |
| D10 tutorials | The 2007 tutorials as fixtures, the v5.30 page winning on a conflict | |
| Series | S0 first, as ADR-0012 (proposed), a pull request of its own; the contract change that follows is a second pull request after the ADR has merged | S1 onward, and any answer to Q43 to Q56: their defaults are provisional until the hardware says |

Anything the owner did not answer with this reply is still not approved. The questions Q43 to Q56 need a person with
an Octopus; an approval does not answer them.

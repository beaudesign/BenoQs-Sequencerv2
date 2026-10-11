# P6b: the first step of a Play sounds at Play (`WENGE-0018`, R0b)

Owner: Metronome (the engine). Tier: **Medium** (engine behaviour; moves recorded goldens). Follows D0 (#46, merged): a step is now 125 ms at 120 BPM, so the old late start is 62.5 ms and audible.

## What is wrong

`StepClock` starts at phase 0 and fires when the phase reaches the step's length, so a run's first step (step 1) sounds only after step 1's own time has gone by: 11 ticks after Play, 5,500 samples, 114.6 ms at 120 BPM and 48 kHz. Every later step is that far late with it, and the metronome's first click comes before the first note. The manual's sequencer plays step 1 when Play is pressed.

## The change

A `started` flag on `StepClock`: the first tick of a run fires step 1 and leaves the phase at 0, so step 2 is a step later. A clock built `at(phase, unit)` (a resume, or a clock carried across a speed change) is already started and is not touched. `Reset` builds a new clock, so it starts again; a Stop then Play resumes exactly, as before (ADR-0009 amendment 3), and does not fire step 1 again.

## What it moves

- Every recorded golden and conformance fixture whose first note sat at tick 11 moves to tick 0, by the same amount for every track. Re-derived with the tool that made them, in their own commit, with the before and after of each.
- **ADR-0009 amendment 1** says "the first note is 11 ticks after Start" and the baseline test `the_first_note_follows_start_by_the_eleven_ticks_the_engine_already_waits` pins it. That sentence stops being true. The test keeps its name (the ratchet) and its body is changed to say what is now true; the removal of the name is asked of the Conductor by request (`journal/metronome/requests/2026-10-11-r0b-amendment-1.md`). The notes now sit **on** the engine's clock pulses, not 11 ticks behind them, which also removes one of the two reasons a pattern was off a receiver's grid.

## Tests (red first, `crates/octocore/tests/first_step.rs`)

1. The first note is at sample 0 or 1 and comes round every 96,000 samples at 120 BPM.
2. A first step that a negative start offset would pull before Play still sounds at Play, and is not lost.
3. Ten tracks with step 1 on all sound at Play, not only the first.

## Not decided here

Nothing the owner must choose. If the owner prefers the old late start it is one revert.

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

## Measured on 2026-10-11: what the change moves (so it is a decision, not a drive-by)

The patch below (`p6b-first-step.patch`, 20 lines in `steps.rs`) makes the three new tests pass, and moves everything else that has an absolute time in it:

- **51 Rust tests fail**: 18 in `octocore`'s own unit tests (`engine::tests` 10, `steps::tests` 7, `fixture` 1), 6 in `golden.rs` (five pinned hashes and one more), 4 each in `o4_steps.rs` and `resume.rs`, 3 in `invariants.rs`, 2 each in `o4_guards.rs` and `program.rs`, and one each in `clock.rs` (the pinned amendment-1 test) and `cli.rs`.
- **The conformance gate fails on about 77 cases** (`cargo xtask verify`: 2 pass, 2 fail; the determinism gate on all five goldens).
- The web layer is not measured; every test that hard-codes a time from the engine's first note (the worklet, follower and plant tests, the browser chase-light test) is affected.

Every note of every track moves earlier by the same 11 ticks at ×1 (the grid shifts, it does not stretch), which a script can prove the way `handoffs/evidence/tick-length/compare.py` did for D0. That is the safe way to do it: the old stream shifted by 11 ticks equals the new one, event for event, at every ×1 track; the tracks at other speeds are checked one by one.

**What it buys:** the first note of a Play sounds 114.6 ms earlier at 120 BPM, and the engine's notes sit on its clock pulses and not 11 ticks behind them (the second reason, with D0, that a pattern was off a receiver's grid). **What it costs:** a day's careful re-derivation of about 130 expectations, an ADR-0009 amendment, and a second move of every golden.

**Parked on purpose (Forge/Metronome, 2026-10-11).** The page now runs and the owner's complaint was that it did not; 115 ms of lead-in is the smallest of the things that still make it not the Octopus. Taken up when the owner says to, or when the Ableton run (S3) shows the 11 ticks matter on a real grid. Nothing here is lost: the red tests are on the branch and the patch is the file beside this one.

## Not decided here

Nothing the owner must choose. If the owner prefers the old late start it is one revert.

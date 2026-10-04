# Request to Conductor: record the metronome click

From: Forge (WENGE-0016, `specs/SPEC-0002/p5d-metronome-click.md`). To: Conductor (`journal/STATE.md`, `specs/SPEC-0002/p5-plan.md`) and Scribe (`docs/10-risks-and-decisions.md`). Tier of the change: **Medium**; nothing here is a contract.

## What happened

The owner wrote "Metronome needs to be built" (Sun 2026-10-04 16:09 London). `p5-plan.md` D-P5-13 had two readings of the owner's earlier "tick should just be metronome on or off - which follows the midi and tempo set": (a) the tick is not a decision, nothing built; (b) an audible click, "not in P5, its own Forge slice (Medium)". This is read as **(b)** and the slice is built on `metronome/click` (stacked on #42).

## What to record, when the pulls merge

1. **D-P5-13** (`p5-plan.md`, and the decision table in `docs/10-risks-and-decisions.md`): reading (b) is built; reading (a) stays true as well, since the click counts the clock's 24 pulses to a quarter and does not need D0. **D0 is still open**; nothing in this slice answers it.
2. **The plan's W4** ("Stop then Play resumes from where it stopped") is met by #41, and the click depends on it: after a Stop it carries on from the beat it had not reached.
3. **`journal/STATE.md`**: the metronome click exists, off by default, on the strip and not the panel; the defaults M-1 to M-7 of `p5d-metronome-click.md` are the owner's to overturn; its latency against external gear is **not measured**.
4. The plan said the click "would count MIDI clock pulses". It does not count them: it reads the engine's tick position, so it keeps no count of its own that could drift from the engine's. The unit is the same (24 pulses of `TICKS_PER_CLOCK` ticks).

## Why a request and not an edit

`specs/SPEC-0002/p5-plan.md` is on #40, which is open, and `docs/` is the Scribe's. Neither is mine to touch from this branch.

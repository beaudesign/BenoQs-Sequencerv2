# Request to Metronome: Stop then Play must carry on from the next note

From: Conductor (P5 plan r3). To: Metronome (`crates/octocore/`, `tests/conformance/`). Task: WENGE-0016 (`specs/SPEC-0002/p5-plan.md`, D-P5-4, W4). Tier: **High (timing)**.

## What the owner asked for

On Sun 2026-10-04 at 11:07 London, answering the P5 plan: "metronome must stop and play at the point of which it's stopped on the sequence." It is a deliberate departure from the manual, which says Stop then Play realigns (p086), and it is recorded in `docs/10-risks-and-decisions.md` section 3.

## What the engine does today

It keeps the position and resumes there, but the position is up to 12 ticks (one step) ahead of what has sounded. `render` steps ticks up to `MAX_EARLY_TICKS` (12) ahead of the audio so a note pulled early by STA is still in the future when it is scheduled. `Stop` clears the queue (`all_notes_off_now`) and `Play` carries on from the tick the engine had reached, so **every note that was scheduled and had not yet sounded is lost**. `tests/conformance/AMBIGUITIES.md`, "lookahead, Stop and commands", records this as "chosen knowingly" and names the alternative ("rewind the position on Stop"). The owner's line reverses that choice.

Measured (`handoffs/evidence/scorecard-2026-10-04/`): through the headless runner, 100% of stop positions lose a note with every step on and 25% with notes four steps apart, at 60, 120 and 240 BPM; through the shipped wasm, 12 of 12. The only test of Stop then Play, `play_after_a_stop_sends_continue_and_the_pulses_stay_on_the_grid_the_pattern_is_on`, checks the clock message and the pulse grid, so it passes.

## What is wanted

After any Stop, the next Play sounds exactly the notes the stream would have sounded had it not stopped, in the same order, none lost and none repeated. Acceptance, red today and green after: `headless.py` section C reports 0 lost notes at every stop position, and a native engine test says the same for a dense and a sparse pattern. The MIDI clock stays `Continue` after a Stop, and the stream still balances.

## Two ways, the Metronome's to choose

1. **Rewind** the tracks by the ticks stepped ahead (the AMBIGUITIES alternative). It has to restore everything a tick advances, including the random and brownian direction state and the groove's random delays, or the notes after the resume differ.
2. **Keep what was dropped**: on Stop, keep the scheduled-but-unsounded events with their distance from the Stop, and put them back at the same distance from the next Play. No state is rewound.

Either way goldens that stop and start may change and are listed first and re-derived in the diff (`examples/golden/`, `mcc_and_transport`), and `just verify` and the determinism gate stay green.

## Not asked

Re-aligning on Stop then Play (p086), a way back to step 1 (plan F-P5-8), or any change to Pause. The page's `transport.play` stopgap goes in P5d, which needs this first only for its Node test.

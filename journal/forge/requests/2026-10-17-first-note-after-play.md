# Request to Metronome: three facts about the first note after Play (R0b), for the owner decision that is already parked

From: Forge (read-only look, task 128). To: Metronome (`crates/octocore/`, `tests/conformance/`). Nothing in the engine was changed.

**What the engine does.** On Play the first note of a pattern sounds on tick index 11, the twelfth tick: 5,500 samples at 120 BPM and
48 kHz. A scratch run (track 0, step 0 only, no strum, then strum 1, 5 and 9) gave the first NoteOn at sample 5,500 in all four. The cause is
`StepClock::new()` in `steps.rs`: `phase: 0`, each tick adds one tick, and a step starts when the phase reaches 12 ticks. It is not the
lookahead (`MAX_EARLY_TICKS`, 12) and not the step start offset (`STA`, 0 at the neutral factor 8). It is already known and pinned: `AMBIGUITIES.md`
"the first note after Play", `o4_guards.rs` (`FIRST_NOTE_TICK`), `clock.rs`, `resume.rs`, ADR-0009 amendment 3, and `p6a-tick-length.md` as R0b
(parked, owner decision). The manual says nothing about it (checked p015, p066, p069, p087, p101).

**Three things that are new, none of them a change to the decision.**

1. **The stated delays are wrong.** `p6a-tick-length.md` section 5 says 62.5 ms at 120 BPM (it was 15.6). ADR-0009 amendment 3 says 28.6 ms.
   With 48 ticks to a quarter note a tick is 10.42 ms at 120 BPM, so 11 ticks are **114.6 ms**, nearly a whole default step (125 ms). 28.6 ms was
   right for 192 to the quarter (11 times 2.60 ms); 15.6 and 62.5 are six ticks, not eleven. The decision should be taken on 114.6 ms.
2. **A measure boundary lands on the last step, not between measures.** `step_all_tracks` applies `measure_deferred` when
   `global_tick % (page_length * 12) == 0`, and `global_tick` is already incremented, so for a 16 step page that is tick index 191. Step 15 fires on
   index 12 * 16 - 1 = 191, the same tick, and the deferred action is applied just before the tracks step. By arithmetic, an On-The-Measure Mute or
   Solo (the only users of `measure_deferred`) takes effect before step 15 sounds, one step before the measure the player hears ends. Untested:
   I did not build a fixture, so treat it as a lead.
3. **Fixing it moves the goldens twice.** `p6a` says R0b moves the goldens a second time. Fix 1 (start the phase at one tick short of a step so the
   first step is on tick 0) would also move item 2 from "one step early" to "one tick early", which is still not the boundary. The boundary itself
   needs its own fixture before either is touched.

**Please.** Correct the two figures in the documents they sit in (the Scribe's and the Conductor's, per `p6a` section 5), and when R0b is decided, add
a red fixture for item 2 first: an On-The-Measure Mute event, the step on which the track goes quiet, against the step the measure is heard to start on.

**Why it matters to me.** Nothing in SPEC-0004 or SPEC-0005 depends on it. The audio renders I sent for review start 115 ms late, which is why `plain.wav`
and the Curve renders begin with a short silence.

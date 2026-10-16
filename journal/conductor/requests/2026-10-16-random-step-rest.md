# Request to Metronome: the single-note random step rest (manual p.23) is not in the engine

From: Conductor (SPEC-0005). To: Metronome (`crates/octocore/`, `tests/conformance/`). Task: `WENGE-0031`.

**What the manual says.** p.23, "Random note picks for random step rest": the pool trick of p.22 "can also be used
without a chord, i.e. on single notes". A step with one note and polyphony 2 is {note, rest}, a 50 percent chance of
the note; polyphony 4 is {note, rest, rest, rest}, 25 percent. The page adds: "Step GRV will not function as expected on
a step set to random step rest."

**What the engine does.** In `crates/octocore/src/engine.rs`, the step-firing code draws only inside
`if step.chord.count > 0 { ... }`. The other branch is `else { notes[0] = final_pit; note_count = 1; }`. Nothing else
in the firing code reads `step.chord.polyphony` (I searched for `polyphony` in the file: the only uses are inside the
chord branch and in the phrase code). So a single-note step always plays, whatever its polyphony.

**What I have not checked.** Whether the panel controller can set the polyphony of a step that has no chord (SPEC-0003
Q55 asks how the gesture works), and whether `tests/conformance/AMBIGUITIES.md` already records this. If it does, this
request only asks that the entry point here.

**Please:**

1. Write the fixtures red first: seeds 0 to 399, one note, polyphony 2 and 4, the fraction that sounds within a bound
   from the binomial around 1/2 and 1/4; polyphony 1 always sounds.
2. Make them pass with the same without-replacement draw as the chord pool (a pool of one note and `polyphony - 1`
   rests, drawing one), so the engine has a single draw for both.
3. Record the GRV caveat of p.23 as chosen or confirmed in `AMBIGUITIES.md`.

**Why now.** SPEC-0005 (a chord style that drifts) would give this draw a memory too (its slice S3), and the draw should
exist and be tested before anything is built on it. Nothing in SPEC-0005 S0 to S2 waits for this.

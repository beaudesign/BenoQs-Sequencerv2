# 09: Roadmap

**Owner:** Conductor.

Phases are gated by **exit criteria**, not by time. A phase ends when its criteria are
met and not before. Each phase also has **kill criteria**: conditions under which we
stop and rescope rather than push through. Naming the kill criteria in advance is what
makes them usable, because in the moment nobody wants to be the one to say it.

---

## Phase 0: The ratchet

Build the verification system before the thing it verifies.

**Scope:** `harness/` end to end against a *deliberately crude* placeholder renderer
(flat-shaded circles from the truth file). Deterministic capture. Every gate
implemented and wired, even where the placeholder fails them. The report page. The
`just` verbs. Worktrees, merge queue, journals, `STATE.md`.

**Exit criteria**
- `just verify` runs in under 4 minutes and reports every gate.
- `verify:determinism` passes: two captures byte-identical.
- The placeholder *fails* `verify:color` and `verify:motion` and *passes*
  `verify:geometry`, proving the gates discriminate.
- The report renders and shows the assertion count as a line.
- Eight worktrees exist and one commit has gone through the merge queue.

**Kill criteria:** if determinism cannot be achieved on the target hardware, stop.
Everything downstream depends on it and there is no version of this project that works
without it.

---

## Phase 1: The vertical slice

The narrowest possible thing that is genuinely finished, at full quality.

**Scope:** one track. Sixteen steps. One MIX encoder and one EDIT encoder. Transport.
One room (`flat`, a neutral studio). Real MIDI out. Loads as VST3 in Live 12 and
plays a synth.

**Full quality means full quality.** Traced chrome, real materials, SDF engraving,
physical motion, correct colour. This phase is not a prototype. It is a small,
complete piece of the finished object.

**Exit criteria**
- All Phase 0 gates green on the slice.
- Geometry p95 ≤ 0.35 mm on the controls present.
- Colour mean ΔE ≤ 2.0 on the material patches present.
- 120 Hz sustained, zero over-budget frames, input-to-photon ≤ 8 ms p95.
- Notes reach Live with σ ≤ 120 µs.
- First blind comparison run: **below 75% accuracy** (a weaker bar than the final
  60%, because a single track gives fewer tells to notice).
- Someone plays it for an hour and wants more of it.

**Kill criteria:** if the blind comparison is above 90%, the material model is
fundamentally wrong and no amount of breadth will fix it. Stop and re-approach the
render technique before building anything else.

---

## Phase 2: The instrument

Widen the slice to the whole panel.

**Scope:** all 160 matrix buttons, all 20 encoders, the full spiral cluster, all four
modes, pages and banks, the complete attribute model, chains, phrases, chords,
directions, the effector, scales, MCC. Push and generic MIDI controller mapping.
AU wrapper alongside VST3. The M4L transport bridge.

**Exit criteria**
- ≥ 250 conformance fixtures, 100% passing.
- Geometry and colour gates green across the full panel, including the spiral.
- Stress scene (all tracks, maximum density) at 120 Hz with zero over-budget frames.
- Worst-case tick ≤ 250 µs.
- 30-minute timing run: zero missed ticks.
- Blind comparison below 70%.
- Assertion count ≥ 280.

**Kill criteria:** if the spiral geometry cannot be recovered to tolerance from
available plates and no better plates can be obtained, relax the tolerance by ADR and
continue. This is a documented degradation, not a kill. The actual kill condition is
frame budget: if the full panel cannot hold 120 Hz on an M-series laptop, the render
technique needs rework before more features land.

---

## Phase 3: The rooms

The mechanic that makes this a new instrument rather than a replica.

**Scope:** `octoroom` complete. Prompt to intent to geometry. Optical and acoustic
bake. Five shipped rooms. The room dropdown with live probe preview. Convolution on
the output bus. The grade.

**Exit criteria**
- `verify:acoustics` green: traced RT60 within 10% of Eyring on all five rooms.
- Cold room bake under 2.5 s; cached room instant.
- Probe preview on dropdown browse at full frame rate with no hitch.
- Room transition: `verify:motion` green on `spring.world`, zero continuity spikes.
- Five rooms that are genuinely different, judged by the blind panel as five
  different spaces both visually and acoustically.
- Assertion count ≥ 340.

**Kill criteria:** if the acoustic tracer cannot hit the Eyring check, the physics is
wrong and the sound will not convince anyone. Fix it before building the shell around
it. If prompt-to-intent proves unreliable, ship authored rooms only and defer
generation to 2.1: **the mechanic works with authored rooms**, and generation is an
amplifier rather than a prerequisite. This is an important pressure valve.

---

## Phase 4: The shell

Fable's phase, though Loom work runs from Phase 3 onward.

**Scope:** the rehearsal rooms space. Settings. The full arrival-to-instrument
transition. Onboarding for the room vocabulary. Persistence and project save.

**Exit criteria**
- All three surfaces complete, `verify:slop` and `verify:tokens` green.
- `verify:a11y` green on all five rooms, including computed contrast.
- Ten iteration cycles logged in `journal/loom/` with hypotheses and outcomes.
- Findings conversion ratio ≥ 40% across those cycles.
- Someone unfamiliar with the product reaches a room they made, and plays, without
  instruction.

**Kill criteria:** none. If the shell is not good enough, iterate. It is the one part
of the system that can be improved indefinitely without structural risk.

---

## Phase 5: Release

**Scope:** signing, notarisation, installer, the manual, the demo Live set, the
launch materials.

**Exit criteria:** every item in [`SPEC.md §7`](../SPEC.md), including the final blind
comparison below 60%.

---

## Sequencing notes

**What must be serial.** Phase 0 before everything. Phase 1 before fan-out. The truth
file's ratification before any geometry gate means anything.

**What can overlap.** Sceneshaper can begin the acoustic tracer during Phase 1, since
it depends only on the room contract. Curator's type identification can run from
Phase 0. Scribe's manual indexing should run in Phase 0 because everyone needs it.
Metronome can port `octocore` during Phase 0 against the null host, because the core
has no rendering dependency at all. This is the payoff of the module boundaries in
[Architecture §3](02-architecture.md): four roles can start on day one.

**The rhythm.** Each phase ends with: a blind comparison, a findings triage, a
threshold review (tighten what has become easy), and a pruning of `STATE.md`. Then
re-fan.

---

## The one thing to protect

Phase 1. There will be pressure to widen it, because a single track feels
unimpressive and because the whole panel is more exciting to build. Resist it
completely.

A finished single track proves the technique. An unfinished full panel proves
nothing and costs ten times as much to diagnose, because when something looks wrong
you will not know which of forty subsystems is responsible.

Narrow and finished. Then wide.

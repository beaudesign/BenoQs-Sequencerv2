# 09: Roadmap

**Owner:** Conductor.

Phases and waves are gated by **exit criteria**, not by time. One ends when its criteria
are met and not before. Each also has **kill criteria**: conditions under which we stop
and rescope rather than push through. Naming the kill criteria in advance is what makes
them usable, because in the moment nobody wants to be the one to say it.

The product is the web sequencer of [ADR-0006](../adr/0006-web-sequencer-supersedes-the-rooms-scope.md)
and `specs/SPEC-0002/`. The old Phases 1 to 5 (a photoreal vertical slice, the whole
panel, the rooms, the shell, release) are replaced by the waves below.

---

## Phase 0: The ratchet

Build the verification system before the thing it verifies.

**Scope:** `harness/` end to end: deterministic replay of the engine, the gates, the
report, the `just` verbs, worktrees, merge queue, journals, `STATE.md`. A gate that
cannot be built yet reports `not_implemented`, never a pass.

**Exit criteria**
- `just verify` runs in under 4 minutes and reports every gate.
- `verify:determinism` passes: two runs byte-identical.
- The report renders and shows the assertion count as a line.
- A worktree exists for each role and one commit has gone through the merge queue.

**Kill criteria:** if determinism cannot be achieved, stop. Everything downstream
depends on it and there is no version of this project that works without it.

---

## Done

- **SPEC-0001, phases F0 and F1** (`specs/SPEC-0001/`; the item list is in `journal/STATE.md`).
- **O4 steps 1 and 4a**: the null host with timing guards G1 to G5 (#14) and integer
  step accumulators (#16).
- **Open: D0**, the tick length (`WENGE-0012`). O4 step 4b waits for it. SPEC-0002 reframes
  4b and 4c for the browser: the host is the audio clock and an incoming MIDI clock, not
  a plugin host. O6 and `octoffi` are frozen, not deleted (SPEC-0002 D5).

---

## The SPEC-0002 series

Each is its own pull request. P2 to P5 wait for P0 and P1 to merge, so none is stacked.

| PR | What | Tier |
|---|---|---|
| P0, P1 | ADR-0006, the `SPEC.md` banner, `STATE.md` (documents only); then the removals and the `verify:scope` gate, red first | High |
| P2 | `contracts/controls.json` and the panel fixture format; the first five workflows red first (spike S5) | Medium |
| P3 | `apps/web`: the WebAssembly engine, the 16 by 10 matrix and the LEDs, one Web MIDI output (spikes S1 and S2) | Medium |
| P4 | MIDI: two ports, clock out and in, input, program change; the Tier 1 Ableton guide and test procedure (spike S3) | High (timing) |
| P5 onward | The waves below | Medium |

---

## Waves 1 to 5

The contents of each wave are in `specs/SPEC-0002/product.md` §5, and the owner may
reorder them. Each workflow in a wave is a fixture cited to its manual page (`tech.md` §4).

**Exit criteria, for every wave**
- Every workflow in the wave's scope has a fixture, and they pass (criterion A1). Where
  the manual is unclear, the fixture is `pending` with the ambiguity written next to it,
  and closes by a hardware check or the owner's ruling, never by a guess.
- A demonstration on real gear: a recorded run on named gear that uses what the wave added.
- `just verify` is green and the assertion count has only grown (A10).
- A threshold marked *set by spike* that the wave depends on was set by the owner before
  the wave started (`product.md` §9).

| Wave | Name | Adds |
|---|---|---|
| 1 | Play | Page mode, Step zoom for VEL PIT LEN STA, the MIX and EDIT encoders, transport and tempo, two MIDI outputs, save and load in the browser |
| 2 | Shape | Track zoom and the attribute maps, chords, phrases, hypersteps, scales, DIR, chains, the effector, step events |
| 3 | Perform | Grid mode and Live, On-The-Measure, Grid-Track, page sets, clusters, follow, PLAY snapshot |
| 4 | Record and sync | MIDI input and recording, keyboard transpose, force-to-scale, controller map learn, the "200" clock states, program change, ALL NOTES OFF |
| 5 | System | Device View (start mode, port priority), the file formats, MIDI file export, factory reset |

Wave 1's demonstration is criterion A6: a pattern made on the panel plays a real synth on
port 1 and a second device on port 2 at once.

**Kill criteria** (proposed; the owner's to change). If spike S1 shows Web MIDI output too
loose for the gear at any lookahead the owner will accept, stop and rescope before
building further (`tech.md` §11). Tier 2 (Max for Live, spike S4) has none: it stands or
falls alone, and Tier 1 does not depend on it.

---

## Sequencing notes

**What must be serial.** Phase 0 before everything. P0 and P1 before P2. Wave 1 before
fan-out. `contracts/controls.json` (P2) before any panel fixture, because it gives
`ControlId` its meaning (`tech.md` §4).

**What can overlap.** Each spike runs before the wave that needs it (`tech.md` §8): S5 in
P2, S1 and S2 in P3, S3 in P4. The Scribe's correction of the manual index (the MIDI,
load and save, and appendix chapters; `specs/SPEC-0002/findings.md` §4) is already
requested and can run in parallel, because every panel fixture cites a page.

**The rhythm.** Each wave ends with: a findings triage, a threshold review (tighten what
has become easy), a demonstration on real gear, and a pruning of `STATE.md`. Then re-fan.

---

## The one thing to protect

Wave 1. There will be pressure to widen it, because Page mode alone feels unimpressive
and because Grid and Perform are more exciting to build. Resist it completely.

A finished Page mode proves the panel controller, the WebAssembly engine and the MIDI
path. An unfinished full panel proves nothing and costs ten times as much to diagnose,
because when something plays wrong you will not know which of forty subsystems is
responsible.

Narrow and finished. Then wide.

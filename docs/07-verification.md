# 07: Verification

**Owner:** Referee. **This document is non-optional reading for every role.**

---

## 1. Why this is the centre of the project

A project like this has a specific failure mode. Twenty agents work for a month.
Each one makes something plausibly better. The whole gets worse, or oscillates, or
becomes internally inconsistent, and nobody can tell because "better" was never
defined mechanically. Tokens burn. Quality plateaus below the bar. Everyone is
individually doing good work.

The defence is a **ratchet**: a growing set of deterministic assertions that can only
be added to, never quietly weakened, such that any change either passes or is
rejected, and any subjective observation that survives scrutiny becomes a permanent
objective constraint.

The ratchet is built first. Phase 0 of the roadmap is the harness, before the web
panel exists. This will feel like a delay. It is the opposite.

---

## 2. Deterministic replay

Every gate below depends on being able to reproduce a run exactly. That requires:

- **No wall-clock time in the engine.** It renders in blocks and stamps each event with
  a sample offset, so event times do not depend on the buffer size.
- **Time is an input to the panel controller.** It will take a millisecond count from
  its caller, so a fixture replays a session exactly (`specs/SPEC-0002/tech.md` §4).
- **Seeded everything.** The engine's RNG is seeded by `Engine::new(seed)`. One seed,
  one run.

`octorun` plays each example pattern headless and writes an event log and a Standard
MIDI File. The SHA-256 of both is compared with `examples/golden/`. The same pattern
gives the same bytes every time. `just run` plays a pattern and `just golden` rewrites
the hashes after an intended change, which needs a reason in the commit.
`verify:determinism` checks exactly this, and it is the first gate to be built
because every other gate is meaningless without it. The WebAssembly build is held to
the same hashes by `just wasm-smoke`.

---

## 3. The gates

Each gate is a `verify:<name>` verb (in the `justfile`, `just verify-<name>`, which runs
`cargo xtask gate <name>`), emits JSON conforming to
`contracts/verification.report.schema.json`, and has a hard pass/fail threshold.
Thresholds may be *tightened* by any role. **Loosening a threshold requires an ADR
with a stated reason and an expiry date.**

Wired today: `conformance`, `regressions`, `determinism` and `scope`. These four are
listed in `harness/required-gates.txt`, so `just verify` fails if one of them does not
pass. Every other gate reports `not_implemented`, which is never a pass. A gate lands
when it is implemented, has a test that shows it failing on a bad input, and is added to
`required-gates.txt` in the same commit (`harness/README.md`).

### `verify:determinism`
Two identical runs produce identical bytes. Fail on any difference. In practice these
are the `golden` tests of `octorun` (§2). **Gate:** at least five of them ran and none
failed.

### Retired: `verify:geometry`, `verify:color`, `verify:frames`, `verify:motion`
These four measured a photoreal render: marker detection against `panel.truth.json`,
colour difference against material patches, GPU frame time and frame-to-frame
continuity, and motion curves against a physical model. They retired with the native
panel (ADR-0006) and had never been implemented. The web panel is not photo-exact and
has almost no motion (an LED flash and a few state changes), so it needs none of them.
Its acceptance is measured by the panel fixtures and by criteria A1 to A10 in
`specs/SPEC-0002/product.md` §9. The documents behind the four gates are archived in
`archive/native-panel/`, and their old definitions are in git history.

### `verify:slop`
The lint from [Design System §4](05-design-system.md). Fast enough for the pre-commit
hook.

### `verify:tokens`
AST scan of the web app's source: no literal value where a token exists.

### `verify:timing`
Per [Sequencer §7](03-sequencer-core.md). Synthetic clock, analytic jitter,
allocation and lock assertions on the audio thread.

### `verify:conformance`
All fixtures in `tests/conformance/`. **Gate: 100%.** No expected failures, no skips.
A fixture that cannot pass is either wrong (fix it) or describes unimplemented
behaviour (move it to `tests/conformance/pending/`, which is tracked and reported but
does not gate). From P2 the gate also runs the panel fixtures (criterion A1): a timed
list of button and encoder events in, an expected LED frame and engine state out.

### `verify:a11y`
Keyboard traversal completeness, in the panel's layout order. Label coverage: every
control has a label from the manual's own names (**gate: 100% of controls**).
Acceptance criterion A8 in `specs/SPEC-0002/product.md` §9. An automated check plus
a written traversal order.

### `verify:arch`
FFI declaration agreement between Rust and `octoffi.h`. No `Math.random` equivalents.
No allocation in audio-thread call graphs (static analysis over the call tree from
`Engine::render`). No hardcoded pixel coordinates in the UI: positions come from
`contracts/controls.json`.

### `verify:persistence`
Round-trips every fixture from every historical version of the saved state. Every
schema change ships a migration and a fixture ([Architecture §4](02-architecture.md)).

### `verify:regressions`
**Assertion count is monotonic non-decreasing across commits on `main`.** This is the
ratchet made literal. The floor is a list of names in `harness/baseline.txt`, not a
count, so deleting one test and adding another is caught. Removing an assertion
requires an ADR (`cargo xtask baseline --remove <kind> <id> --adr ADR-NNNN`).

### `verify:scope`
The live tree must not describe the product this repo used to be aimed at (ADR-0006).
The gate scans every file git tracks or would track, and fails when a phrase from
`harness/scope-banned.txt` appears outside the places listed in
`harness/scope-allow.txt`. Acceptance criterion A9.

- **Matching.** Case-insensitive, whole words: a letter or digit beside a match stops
  it, a slash, colon, hyphen or underscore does not. The last word of a phrase may take
  a plural `s`. Any run of whitespace, a line break included, joins the words of a
  phrase. The path is scanned as well as the content, so an empty file cannot bring a
  name back. It matches phrases, not ideas: a synonym gets through, and review still
  applies. The bare word "room" is ordinary English and is not banned.
- **Allow-list.** History keeps its words: `adr/`, `handoffs/`, `journal/`,
  `specs/SPEC-0001/`, `specs/SPEC-0002/`, `archive/`, `reference/`, and three files in
  `harness/` (the baseline and the two lists). An entry ending in a slash allows a
  directory; any other entry allows exactly that file. A new entry widens the gate and
  needs the Referee and a reason in the pull request.
- **Floor.** The banned list can grow freely. Removing a phrase loosens the gate, so it
  needs an ADR with a reason and an expiry date (`CLAUDE.md` rule 3), and `MIN_BANNED`
  in `xtask/src/scope.rs` is lowered in the same change. The gate also fails when it
  scanned fewer than `MIN_FILES_SCANNED` files, or when a list file is missing.

---

## 4. Findings

A vision model used to critique captures against an anchored rubric. That perceptual
pass judged a photoreal render and is retired with the native panel (ADR-0006),
together with the rubric and its anchors. What stays is the rule behind it: **every
taste judgement becomes a test.**

Any observation about the product is a finding: a reviewer's, one from the
demonstration on real gear that ends each wave (`09-roadmap.md`), or one from someone
using the app.

**Rules:**

1. **Scores never gate.** If a model or a person rates something, the rating is never
   a pass/fail condition. Only a converted test gates.
2. **Findings are the product.** A finding has an id, a severity, an observation and a
   status, and names the workflow or control it is about.
3. **Every finding is triaged before its wave ends** into exactly one of:
   - **Converted.** A new deterministic assertion was written, and the finding closes
     when the assertion passes. This is the desired outcome.
   - **Rejected.** With a written reason. Rejections are kept, so the same false
     positive does not get relitigated every wave.
   - **Deferred.** With a wave. Bounded; a finding may be deferred at most twice.
4. **The conversion is the point.** "The EDIT light looks wrong" is worthless. "After
   one click on EDIT the LED frame shows edit as orange, flashing (manual p068 and
   p069)" is a fixture that will still be true in a year and will fail the instant
   someone breaks it.

Not every finding can be converted, and that is fine. But the ratio matters: if fewer
than 40% of a wave's findings convert, the observations are producing vibes rather
than signal, and the Referee tightens how they are recorded.

---

## 5. The blind comparison

Retired with the native panel (ADR-0006). It was a forced choice between a calibrated
photograph and a render, and the web panel is not a photo-exact render
(`specs/SPEC-0002/product.md` §8). What replaces it is the panel fixtures, criteria A1
to A10, and a demonstration on real gear at the end of every wave. Anything the
demonstration shows enters the findings pipeline in §4.

---

## 6. The report

`cargo xtask verify` writes `harness/report/latest.json`, and `just report` prints it.
It holds

- the status of every gate, with each metric against its threshold,
- the assertion counts, the tests and fixtures added, and any removed by ADR.

It is the shared world model. Every agent reads it before starting work and after
finishing. If a fact is not in the report, it is not known. A rendered page,
`harness/report/index.html`, is not built. It is meant to add the trend of every gate
across the last 50 commits, the findings board grouped by status, and the assertion
count over time (the ratchet, drawn as a line that should only go up).

---

## 7. Guarding against the metrics

Every one of these gates is gameable, and agents optimising against metrics will
eventually game them. Explicit countermeasures:

- **Assertion count** could be gamed by adding trivial assertions. Countermeasure: the
  Referee spot-audits ten assertions per wave for substance, and the findings
  conversion ratio (§4) is looked at separately.
- **Fixtures** could be gamed by writing the expected output from the code instead of
  the manual. Countermeasure: every fixture cites its manual page (N8, the manual is
  the truth), and where the manual is unclear or contradicts itself the fixture is
  `pending` with the ambiguity written next to it, so a guess cannot pass as a fact.
- **Scope** could be gamed by widening the allow-list or dropping a phrase.
  Countermeasure: an allow-list entry needs the Referee and a reason, and dropping a
  phrase needs an ADR and a lower `MIN_BANNED`, which is a loosening.

State this plainly to every agent at the start of a session: **the goal is the
instrument, and the gates exist to serve it.** An agent that finds a way to pass a
gate without improving the instrument has found a bug in the gate, and the correct
response is to report it, not to use it.

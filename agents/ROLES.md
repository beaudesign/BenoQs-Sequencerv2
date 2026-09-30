# Role Briefs

A role is a hat, not a person. One agent may wear several sequentially, never two at
once. The constraint is the value: it is what makes parallel work safe.

Each brief states what you own, what you must never do, what you read first, and how
you know you are succeeding.

---

## Conductor

**Owns:** `contracts/`, `adr/`, `justfile`, `crates/octoffi/`, the roadmap, the merge
queue, `journal/STATE.md`.

**Never:** implements a feature. The moment the Conductor starts building, contract
changes stop being scrutinised and the whole model degrades.

**Reads first:** all open merge requests, `STATE.md`, the phase exit criteria.

**Does:**
- Operates the merge queue serially. Rebase, verify, ownership check, merge.
- Writes and ratifies ADRs. Every contract change is an ADR first, merged separately.
- Publishes the current fan-out: who is working on what, and which contracts are
  frozen.
- Prunes `STATE.md` when it exceeds 200 lines. Pruning, not appending.
- Enforces phase gates. Refuses fan-out until the phase before it has exited (`docs/09`).
- Owns R8 (trademark) and D8 (naming). Address in week one.

**Succeeding when:** the roles are working in parallel with zero merge conflicts,
and every one of them can state what they are doing without asking anyone.

---

## Panelwright

**Owns:** `reference/`, `crates/octoface/` and `contracts/controls.json` (both planned, P2;
the contract is changed by the Conductor, by ADR).

**Never:** draws anything, and never fills a gap in the manual with a guess. The Panelwright
turns the manual into a panel controller and a control inventory; the Forge draws it. Where
the manual is silent or contradicts itself, the fixture is `pending` with the ambiguity
written next to it.

**Reads first:** `specs/SPEC-0002/product.md`, then `specs/SPEC-0002/tech.md` section 4, then
the manual pages for whatever workflow you are on (`just manual <topic>`).

**Does:**
- Builds the control inventory: every button, encoder and LED, with its manual name, kind,
  position in the layout and page.
- Writes the panel fixtures from the manual before the controller: a timed list of button
  and encoder events in, an expected LED frame and engine state out, each cited to a page.
- Builds `octoface` one workflow at a time, red first: hold and double-click gestures, EDIT
  states, the LED language, the displays.
- Keeps the list of questions that only a real Octopus can answer
  (`specs/SPEC-0002/tech.md` section 9) and asks the owner for the answers.

**Succeeding when:** every workflow in a wave's scope has a fixture that passes, and nobody
needs to open the manual to learn what a button does in that wave.

---

## Forge

**Owns:** `apps/web/` (planned, P3).

**Never:** contains sequencing logic, or invents a colour, a position or a curve. Every
position comes from `contracts/controls.json`, every colour is an LED role or the one neutral
palette. The page never reaches into the engine: a UI feature that seems to need an engine
query is a missing field in the snapshot or the LED frame, and a request to its owner.

**Reads first:** `docs/02-architecture.md`, `docs/05-design-system.md`, then
`docs/00-north-star.md` section 2, so you understand what you are correcting.

**Does:**
- Draws the matrix, the encoders and the LEDs in SVG (or Canvas) from the inventory, at a
  fixed aspect that scales and never reflows.
- Loads the WebAssembly engine in an AudioWorklet and passes events across with
  `postMessage`. Runs spikes S1 and S2 and records the numbers.
- Owns the Web MIDI layer: two outputs, timestamped sends with a lookahead, `clear()` and
  ALL NOTES OFF on Stop, inputs and clock.
- Owns settings and saved state (IndexedDB, and export as a file).
- Makes the panel operable by keyboard in the panel's layout order, with a label on every
  control from the manual's own names.

**Succeeding when:** a musician who knows the hardware plays and programs a pattern with the
gestures they already have, the panel fixtures pass in Chrome, and the a11y gate is green.

---

## Metronome

**Owns:** `crates/octocore/`, `tests/conformance/`, `hosts/m4l/` (the Tier 2 Ableton spike
only), and the timing path.

**Never:** takes a timing shortcut. A dropped frame is a bug; a late note is a
catastrophe. And never ports a v1 test alongside v1 code, because that launders the
original author's misreadings into the new codebase.

**Reads first:** `docs/03-sequencer-core.md`, then the manual sections for whatever
you are implementing.

**Does:**
- Ports the Octopus behaviour model with a manual citation on every constant.
- Writes conformance fixtures from the *manual's* description, before porting the
  code, so the fixture can disagree with v1 and win.
- Mines the manual's Appendix for worked examples: it is free, high-quality test data
  written by the people who built the machine.
- Sample-accurate event emission with `at_sample` offsets.
- Keeps the frozen native pieces (the command ring, the triple-buffered snapshot) and
  their tests passing. No new work on them (SPEC-0002 D5).
- Follows an incoming MIDI clock: a small pure function that estimates tempo and phase,
  tested on a simulated jittery clock at several tempi (P4).
- The Max for Live bridge, only if spike S4 shows `jweb` can carry it, and under 400 lines.

**Succeeding when:** the fixtures pass, note-on jitter and clock follow are within the
numbers the owner set from spikes S1 and S3, and the audio thread has provably never
allocated.

---

## Curator

**Owns:** the visual rules in `docs/05-design-system.md`: the LED roles, the one neutral
palette, the layout rule and the slop rules.

**Never:** builds. Separating the producing role from the evaluating role is the
reason the rules mean anything. An agent that makes and grades its own work
converges on whatever it finds easy.

**Reads first:** `docs/05-design-system.md`, `docs/00-north-star.md`.

**Does:**
- Selects the one typeface family from three specimens rendered at real size beside the
  panel, and defends it in an ADR.
- Maintains the slop rules. Rules are added, never removed. Every rule exists because
  it caught a real instance. The Referee builds the gate that runs them.
- Audits the app against the generated-design cluster after every wave.
- Removes one accessory before every wave boundary.

**Succeeding when:** the rules fire on real problems and not on real solutions, and
nobody on the team can name a decision made by default.

---

## Referee

**Owns:** `harness/`, `xtask/`, `tests/golden/`, CI, all thresholds, and the `scope` and
`a11y` gates.

**Never:** lets a threshold loosen without an ADR, and never lets a rating gate anything.

**Reads first:** `docs/07-verification.md` in full.

**Does:**
- Builds and guards the ratchet: the baseline, the required gates, the report. This will
  feel like a delay. It is the opposite.
- Deterministic replay. Byte-identical repeat runs, or nothing else works.
- Every gate, each under its own `just` verb, each emitting schema-conformant JSON.
- Keeps the banned-phrase list and the allow-list of `verify:scope`, and turns any
  return of the old scope into a red gate.
- Keeps `just verify` under four minutes. Hard requirement. A suite that takes twenty
  minutes gets skipped, and a skipped gate is not a gate.
- Triages every finding within one wave into converted, rejected (with a reason, kept
  forever), or deferred (at most twice), and tracks the conversion ratio. Below 40% means
  the observations are producing vibes, and the way they are recorded needs tightening.
- Tightens thresholds that have become easy.
- Spot-audits ten assertions per wave for substance, so the count cannot be gamed.

**Succeeding when:** the assertion line only goes up, and no agent has ever passed a gate
without improving the instrument.

---

## Scribe

**Owns:** `docs/`, `README.md`, `CHANGELOG.md`, `reference/manual/`.

**Never:** lets a doc drift from the code without flagging it.

**Reads first:** every merge request that lands.

**Does:**
- Page-indexes the reference manual in Phase 0 and builds `just manual <topic>`.
  Everyone needs this and it is cheap.
- Keeps `docs/` truthful. When an ADR lands, the affected doc changes in the same
  merge, not later.
- Writes the user-facing manual. It should be as good as the original genoQs one,
  which is a high bar: that manual teaches an instrument rather than listing features.
- Maintains `tests/conformance/AMBIGUITIES.md`: where the manual is unclear, both
  readings and which we chose.

**Succeeding when:** a new agent can start productive work from `CLAUDE.md` in twenty
minutes without asking anyone a question.

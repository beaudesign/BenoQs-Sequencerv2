# ADR 0007: The control inventory is a contract, and the panel is tested by fixtures

## Status

Proposed on 2026-09-30 by the Conductor. It follows from SPEC-0002 r1 (approved by the owner,
"Approve", 2026-09-30 15:06 Paris), which approved P2 and named `contracts/controls.json`
(`specs/SPEC-0002/tech.md` section 4). It takes effect when the owner merges the pull request
that carries it. The files it authorises are built in P2b, the pull request after this one.

## Context

The engine accepts `ButtonDown`, `ButtonUp` and `EncoderTurn` and ignores them. The reason in its
own doc comment is that there is no control inventory, so `ControlId` maps to nothing
(`crates/octocore/src/engine.rs`, the doc comment on `handle_command`; `specs/SPEC-0002/findings.md` F1). The old inventory,
`panel.truth.json`, was a set of millimetre positions and materials measured from photographs for
a native renderer. That renderer is retired (ADR-0006). What a web panel needs is smaller and
different: which controls exist, what the manual calls them, how many there are, and where they
sit relative to each other.

Three contract files are in `contracts/` today:

| File | State |
|---|---|
| `panel.truth.schema.json` | Schema for the millimetre model. Nothing reads it. It names plates, scale uncertainty and aperture in millimetres |
| `motion.registry.schema.json` | Schema for eased motion on the native panel. Nothing reads it. Its examples name the retired shell |
| `verification.report.schema.json` | Read by the report writer and its test. Gate names were fixed by ADR-0006. The `findings` section still describes a photoreal pass |

`CLAUDE.md` rule 2 says a contract changes only by ADR, merged separately. This is that ADR.

## Decision

### 1. `contracts/controls.json` is a contract, with a schema

P2b adds `contracts/controls.json` and `contracts/controls.schema.json` (JSON Schema 2020-12,
`"schema": "controls/1"`). The file lists every control the manual names. A control carries:

| Field | Meaning |
|---|---|
| `n` | The number. An integer from 0 to 511 (the engine's `MAX_CONTROLS` is 512). Unique, assigned once, never reused, never renumbered. See decision 2 |
| `id` | A dotted lower-case name, for example `matrix.r3.c5`, `mode.step`, `edit`. Unique. A rename is an ADR |
| `kind` | `button` (a key, which may have an LED), `encoder` (a rotary, which may have a push), or `display` (a readout the controller writes) |
| `zone` | One of the zones in the file's `zones` list: matrix, selector column, mutator column, MIX, EDIT, circle (outer and inner), chord block, transport, mode, and any the manual adds |
| `name` | The manual's own name for the control, as printed. A control the manual does not name carries the manual's description and is marked `unnamed` |
| `at` | A position in the zone's own grid, as **integers in the manual's numbers** (matrix columns are steps 1 to 16; matrix rows are 0 to 9, row 0 at the bottom). No millimetres, no pixels |
| `led` | Whether the control has an LED |
| `cite` | The manual pages, `["p013", "p014"]`. At least one. A control cited to no page is not in the file |
| `pending` | Present when the manual does not settle the control's existence, count or position. It names the question (`specs/SPEC-0002/findings.md` section 6 or `tech.md` section 9). A pending control is listed without a guessed position |

The file also carries `zones` (each with its grid size, where the manual gives one), and an
optional `relations` list of layout facts the manual states, each cited, for example that the
selector column is left of the matrix. **The file holds what the manual says and nothing else.**
Where the manual gives no arrangement, the web app's own layout file (`apps/web`, the Forge's,
P3) makes a provisional choice and marks it provisional. A design choice is not a manual fact
and does not belong in a contract.

It replaces `panel.truth.json` for this product. There is no provenance block of plate hashes
and no photograph is read to write it (`CLAUDE.md` rule 7).

**Change policy.** Adding or removing a control, renaming an `id`, renumbering, or changing a
cited position is an ADR. Filling a `pending` marker with a hardware check or the owner's ruling
is recorded as a one-line entry in the Amendments list at the end of this ADR, made by the
Conductor, and is reviewed as a contract change. The Panelwright drafts; the Conductor commits.

### 2. `ControlId` means `n`

`ControlId(u32)` in `crates/octocore/src/types.rs` is the `n` of a control in `controls.json`.
The type does not change, and neither does anything on the C interface (`octoffi` stays frozen).
Only the doc comment on `ControlId` changes, to say what it means (comments only, the same
authority ADR-0006 used for `octoffi`). The engine is not told about the inventory and does not
read the file. The panel controller, which is where `ControlId` is interpreted, is built
against it (decision 5).

### 3. Two schemas are retired and one is corrected

- `panel.truth.schema.json` and `motion.registry.schema.json` **move** to
  `archive/native-panel/contracts/`, beside the two documents that moved there in P1. Git keeps
  the history. The motion schema's replacement is not a schema: the LED flash and the few state
  changes the web panel has are in `docs/05-design-system.md` section 3 and are tested there.
- In `verification.report.schema.json`, in the `findings` section only:
  - `dimension` (an enum of seven photoreal rubric terms) is removed;
  - `workflow` (a string, the workflow's id in the digests or its fixture path) and `control` (a
    string, an `id` from `controls.json`) are added, both optional, because `docs/07` section 4
    rule 2 says a finding "names the workflow or control it is about";
  - the `source` enum loses `perceptual_pass` and `blind_comparison` and gains `hardware_check`,
    the demonstration on real gear that ends each wave.
- The report's `perceptual` and `blind_comparison` objects are removed. Nothing writes them
  (`xtask/src/report.rs` does not emit them), both describe judging a photograph, and
  `docs/07` section 4 says the pass they served is retired.
- The schema's version string stays `verification.report/1`. The removed properties were
  optional and never emitted, so no report that exists becomes invalid.

### 4. Panel fixtures sit beside the engine fixtures

The fixture DSL in `crates/octocore/src/fixture.rs` is a script for the engine: transport,
tracks, rendered sample buffers. A panel fixture is a different thing: a timed list of button
and encoder events in, and the LED frame, the displays and the engine commands out. Extending
the first to carry the second would put panel vocabulary in the core, which has no I/O and no
time source. So:

- Panel fixtures are `tests/conformance/panel/**/*.panel`. Their runner is an integration test
  in `crates/octoface` (`tests/panel_fixtures.rs`). `crates/octocore/tests/conformance.rs` walks
  `*.fixture` only, so it does not see them and is not edited.
- A fixture is cited to manual pages in its header and names the workflow it covers.
- A fixture the manual cannot settle lives in `tests/conformance/panel/pending/` and does not
  assert. It must carry a `# pending:` header with the page, the ambiguity and the question, which
  `tech.md` section 9 or `findings.md` section 6 lists. It closes by a hardware check or the
  owner's ruling, never by a guess: the file then moves out of `pending/` with the assertions the
  answer supports.
- `xtask` (the Referee's, a tightening): the `conformance` gate counts `.panel` files as
  fixtures, requires the `panel_fixtures` test binary to have run and to pass, and reports the
  number of pending fixtures as a metric. A pending file with no header fails the gate. The
  ratchet baseline's kinds do not change. A panel fixture that passes is recorded under `fixture`
  by `cargo xtask baseline` like any other, and a pending one is reported, not floored.
  Making pending a baseline kind is a later decision.

### 5. `crates/octoface` has no runtime dependencies

The architecture puts the controller next to the engine in the audio worklet
(`docs/02-architecture.md` section 1), so it is a dependency-free library. The inventory reaches
it as a table of `(n, id)` pairs. The controller reads the role from the `id`. The test binary
loads `contracts/controls.json`, validates it against `controls.schema.json` and builds the table
from it, using `serde_json` and `jsonschema` as **dev-dependencies**, which the lockfile already
holds for `xtask`. A test fails when the controller names a control the file does not hold, so
the two cannot drift.

### 6. Risk tier

`AGENTS.md` puts any change under `contracts/**` in the High tier (both specs, a pull request and
a release plan). The SPEC-0002 README rated P2 Medium before that was read against this table;
the higher tier applies. Both specs are approved. The release plan is the pull request itself:
reverting it restores the two retired schemas and removes the two new contract files, with no
data to migrate, because nothing reads any of them yet.

## Not decided here

- Anything the manual leaves open (`findings.md` section 6). Those are `pending` fixtures and
  `pending` control markers.
- **The engine's read model.** The controller needs to read step state from the engine to light
  the matrix. `Snapshot` carries no step data, and it is a C-layout struct (frozen). P2b reads
  `Engine::grid` through its public fields in the test runner, and a request to the Metronome
  asks for the engine side (`journal/metronome/requests/`). It is not decided here.
- **Transport and PLAY-mode snapshot commands** the engine does not have. The controller emits an
  intent and the fixture asserts the intent. The Metronome's request covers them.
- The web app's provisional layout (Forge, P3) and the design tokens (Curator).
- **D0**, the tick length. Still open.

## Consequences

- `ControlId` has a meaning and the panel controller has something to be tested against.
- Two schemas that described a renderer that is not being built leave the live tree, so a
  reader of `contracts/` meets only contracts the product uses.
- The contract stays small and tied to the manual: if a fact is not on a cited page, it is
  `pending`, not a guess.
- A hardware check that answers a question changes the contract by a one-line amendment
  instead of a full ADR.
- The ratchet floors grow in P2b by the passing panel fixtures and tests. Nothing leaves the
  baseline.

## Amendments

None yet.

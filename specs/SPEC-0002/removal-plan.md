# SPEC-0002 removal plan

Draft for the owner. **Nothing in this file has been done.** It says what would be removed,
retired, edited and kept, in what order, and how the removal is made permanent by a test.
Counts are from `git ls-files` at `fcf1aec` and a search of the tree; they are not a claim of
completeness beyond the terms listed in section 5.

## 1. What counts as world-model content

The WENGE spec described a product that generates a room from a prompt (the interaction model
is borrowed from Genie), lights a photoreal instrument with that room's light, convolves its
sound with that room's acoustics, and navigates by "rehearsal rooms". The README already says
this spec was written for another project and applied here by mistake. World-model content is
anything that exists only to make, browse, light or hear a generated room.

**Not world-model content:** the phrase "shared world model" in `journal/STATE.md`,
`agents/CLAUDE.md`, the `justfile`, `harness/README.md` and `contracts/verification.report.schema.json`
is the factory's name for the shared project state. It stays. And `loom` in
`crates/octocore/loom/` and `crates/octocore/tests/loom_sync.rs` is the concurrency model checker, unrelated to
the Loom role.

## 2. Inventory

Sizes in lines. "Zone" is the owner in `docs/08`.

### 2.1 Remove: whole files

| Path | Lines | Notes |
|---|---|---|
| `crates/octoroom/` (7 files) | 495 | Room intent, geometry synthesis, Eyring RT60. A workspace member in `Cargo.toml`. **Ten ratchet tests** in `harness/baseline.txt` (lines 224 to 233) go with it |
| `docs/06-shell-and-rooms.md` | 307 | The Genie-style shell, the room contract, the pipeline, Fable's evolution loop |
| `apps/OctoShell/` | scaffold | Empty (`.gitkeep`) |
| `justfile` recipe `verify-acoustics` | 2 | |
| Gate `acoustics` in `xtask/src/gates.rs` | 1 | And in the gate-name test near line 400 |
| Gate name `acoustics` in `contracts/verification.report.schema.json` | 1 | A contract: the Conductor, by ADR-0006 |
| `agents/ROLES.md`: Sceneshaper and Loom (Fable) sections | about 57 | Lines 117 to 173 |
| `.github/CODEOWNERS`: the Sceneshaper and Loom groups | 6 | |

### 2.2 Edit: sections of mixed documents

| File | What to cut or rewrite |
|---|---|
| `SPEC.md` | Section 1 (the thesis and "The unifying mechanic" are the room idea); section 3 deliverables D2, D3, D4 and the plugin bullets; section 4 (macOS-only and "generative audio"); section 5 (layout entries for `octoroom` and `OctoShell`); section 7 items 2 to 4, 7 and 8 (photoreal panel, rooms). Keep N5, N7 and N8. Restate N1 to N4 and N6 after D3 |
| `docs/00-north-star.md` | Rewrite as the principles of the web panel. Keep the description of the object (section 1) and the four ways v1 looked generated (section 2), which apply to any web mock. Drop the room paragraphs, the shell type register and the layout that shows a room |
| `docs/02-architecture.md` | New architecture from `specs/SPEC-0002/tech.md`. Drop the room thread, "Room to Render", the `octoroom`, `OctoPanel` and `OctoShell` modules, and the plugin host section |
| `docs/03-sequencer-core.md` | Keep. Add the panel controller and MIDI in and clock (Scribe, after P2 and P4) |
| `docs/05-design-system.md` | Rewrite very short: the LED colours, the layout rule, the one neutral palette. Drop room-derived shell colour, the three shell springs and the shell type ADR |
| `docs/07-verification.md` | Remove `verify:acoustics` and the room scenes in the capture examples; reframe or retire `geometry`, `color`, `frames`, `motion`; add `scope` |
| `docs/08-agent-operating-model.md` | Role table, the ownership map, the merge fan-out (Sceneshaper and Loom lines) |
| `docs/09-roadmap.md` | Phases 3 and 4 (rooms, shell) become the waves in `product.md` section 5; Phase 1's exit criteria are rewritten around the fixtures |
| `docs/10-risks-and-decisions.md` | Drop R2 (chrome), R6 (prompt to room), decisions D1 (shell typeface), D4 to D7; mark the rows of section 3 that are reversed (section 4 below) |
| `AGENTS.md` | The purpose paragraph (line 14) and the crate table (`octoroom` row and the "Sceneshaper, Loom" role list) |
| `agents/CLAUDE.md` | The first paragraph ("lives inside generated rehearsal rooms"), the document table, and the nine things: numbers 4 to 7 (gradients, easing, colour, photography) are photoreal-render rules and are reworded to D3 |
| `README.md` | The paragraph on the "rehearsal rooms" vision and the Status section |
| `journal/STATE.md` | The `octoroom` section; the reference-photography section (photos become optional) |
| `crates/octoffi/src/lib.rs` (line 9), `crates/octoffi/octoffi.h` (lines 7 and 8) | Comments that name `octopanel` and `octoshell` as the future Swift callers. Conductor zone; a comment-only change |

### 2.3 Retire: the native and photoreal panel path

These are not world-model content. They serve a photoreal, native, Metal panel, which a web app
does not need. Retire unless the owner keeps some of it under D3. All are empty scaffolds or
documents.

| Path | Notes |
|---|---|
| `apps/OctoPanel/`, `apps/OctoStandalone/`, `hosts/vst3/`, `hosts/au/` | Empty scaffolds. `hosts/m4l/` stays: it is where Tier 2 would live |
| `harness/capture/`, `harness/lint/`, `harness/measure/` | Empty scaffolds for frame capture, the slop lint and geometry measurement |
| `docs/01-panel-truth.md` (349 lines), `docs/04-render-engine.md` (336 lines) | Photogrammetry and a Metal ray tracer. Kept only as an appendix if photo-exact fidelity is ever wanted |
| `contracts/panel.truth.schema.json`, `contracts/motion.registry.schema.json` | Replaced by `contracts/controls.json` (`tech.md` section 4). The motion schema's examples name `shell.dropdown.reveal` and `room.probe.crossfade` |
| Gates `geometry`, `color`, `frames`, `motion` | All `not_implemented` today. `slop` and `tokens` stay only if D3 keeps their rules |
| `reference/plates/` | Keep the file. It is no longer a blocker |

### 2.4 Keep

| Path | Why |
|---|---|
| `crates/octocore/` | The engine. Nothing in it refers to rooms (the search hits are the word "room" meaning capacity, and `loom`) |
| `crates/octorun/`, `examples/` | The headless runner and the goldens |
| `crates/octoffi/`, `harness/wasm/` | The WebAssembly build the smoke test uses. Frozen otherwise (README D5) |
| `xtask/`, `harness/` (the ratchet, `required-gates.txt`, `baseline.txt`), `.github/workflows/` | The factory |
| `tests/conformance/`, `reference/manual/` | The manual is the truth (N8) |
| `AGENTS.md`, `agents/`, `.claude/skills/`, `handoffs/`, `journal/`, `adr/`, `specs/` | The lifecycle. History is never deleted |

### 2.5 Records: annotate, never delete

`adr/0002-supersede-v1-adopt-wenge.md` gets a status line, "superseded in part by ADR-0006".
`specs/SPEC-0001/` stays as it is. Old journal entries and handoffs that mention rooms are
history and are on the scope gate's allow-list.

## 3. Order that keeps the gates green

Each step is a commit on the P1 branch, and `cargo xtask verify --base origin/main` runs
before each push.

1. **Write the scope gate first, and watch it fail.** A preview scan for this draft, with the
   phrases in section 5 and the allow-list below, finds **131 matching lines in 26 files** in
   today's tree, from `docs/06` (22) and `crates/octoroom` (32 across its files) to one line
   each in `README.md` and `Cargo.toml`. The real red run is saved to `handoffs/evidence/`.
2. **Remove `crates/octoroom`**, its workspace member, and its ten tests from the baseline with
   `cargo xtask baseline --remove test <id> --adr ADR-0006`, one per test. The ratchet refuses
   any other removal.
3. **Remove the `acoustics` gate** (`xtask`, `justfile`) and fix the test that lists the gate
   names. `harness/required-gates.txt` does not name it, so no required gate changes. The
   schema's gate-name list is a contract: it changes in the same PR, by ADR-0006.
4. **Cut the documents and roles**: the tables in section 2.
5. **Turn the scope gate on** in `harness/required-gates.txt` in the same commit that makes it
   pass.

Removing a required gate or a ratchet test without an ADR is exactly what the ratchet forbids,
so ADR-0006 lands before P1 (README series, P0).

## 4. Closed decisions this reverses

From `docs/10-risks-and-decisions.md` section 3, which says these were recorded so that they
were not reopened by agents who were not there. The owner is reopening them, and says so here.

| Decision | Was | Becomes |
|---|---|---|
| Render technique | Analytic ray tracing | SVG, or Canvas (D3) |
| Antialiasing | 3x supersampling | The browser's |
| Tonemap | AgX | Not applicable |
| UI host | Native app, not Max for Live `jweb` | Web app. `jweb` is reopened as Tier 2, on evidence (spike S4) |
| Plugin formats | VST3 and AUv2 | None in v1. Ableton by routing (Tier 1) or Max for Live (Tier 2) |
| IR generation | Ray traced from geometry | Removed |
| Motion | Physical simulation | LED flash and a few state changes only |
| Panel geometry | A measured truth file | A logical control inventory from the manual; measurement optional |
| Core language | Rust | Unchanged |
| v1 code | Archive, transcribe two files | Unchanged |

## 5. The scope gate

The removal is a test, in the project's own terms: every judgement becomes an assertion.
`cargo xtask gate scope` (Referee, in `xtask/`) scans tracked files and fails on a banned
phrase outside the allow-list. It is added to `required-gates.txt` when it passes.

**Banned phrases** (case-insensitive, whole words; D9 is the owner's to change):

`rehearsal room`, `light probe`, `environment probe`, `impulse response`, `convolution`,
`octoroom`, `octoshell`, `sceneshaper`, `genie`, `rt60`, `eyring`, `hdri`, `room contract`,
`room dropdown`, `room schema`, `prompt-to-room`, `probe cross-fade`.

The bare word "room" is not banned: it is ordinary English (`room for`, `Room` in a comment about
capacity). The phrases above are the ones that carry the idea.

**Allow-list** (`harness/scope-allow.txt`, one path prefix per line): `adr/`, `handoffs/`,
`journal/`, `specs/SPEC-0001/`, `specs/SPEC-0002/`, `archive/`, `reference/`,
`harness/baseline.txt` (its `removed` lines will name the ten `octoroom` tests, by design), and
the allow-list file itself. History keeps its words.

**Explicitly not matched:** `shared world model`, and `loom` as a crate or path.

**Its own tests** (they count in the ratchet): a file containing a banned phrase fails; the
same file under an allow-listed prefix passes; `shared world model` passes; `crates/octocore/loom`
passes; a phrase split across a line break is reported once.

## 6. Rollback

The removal is a set of deletions and edits on one branch. Everything is in git history. If the
owner changes course, reverting the P1 merge restores the tree, and the ratchet's
`removed` lines are the record of what was dropped and why.

## 7. What is carried forward from WENGE

Removing the room does not mean dropping the discipline. These stay, and appear in the rewritten
`SPEC.md`:

- **N5, timing beats pixels.** A late note is worse than a dropped frame.
- **N7, every taste judgement becomes a test**, and the ratchet that follows.
- **N8, the manual is the truth**, with each disagreement logged as a fixture.
- **The four ways v1 looked generated** (a gradient for a material, uniform lighting,
  approximated geometry, borrowed chrome), which apply to a web mock as much as to a Metal one.
- **The lifecycle**: roles, risk tiers, handoffs, the independent review, the agent merging
  nothing.
- **"Narrow and finished before wide."** Wave 1 is a playable Page mode.

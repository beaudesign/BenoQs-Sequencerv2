# STATE

Capped at 200 lines. Conductor prunes rather than appends when it grows past
that. Last pruned: 2026-09-06 (first write).

## Phase

**Phase 0 (the ratchet) — scaffolding done, the ratchet itself not built.**
No real gate exists yet except the umbrella structure. Per
`docs/09-roadmap.md`, Phase 0 exit criteria (`just verify` green under 4 min,
`verify:determinism` passing, placeholder renderer discriminating gates,
report rendering, eight worktrees through the merge queue) are **not met**.

## What's scaffolded (2026-09-06)

- Repo layout matches `SPEC.md` §5: `docs/`, `contracts/`, `agents/`,
  `adr/`, `archive/`, `apps/*`, `hosts/*`, `harness/*`, `crates/octoffi`,
  `reference/*`, `tests/conformance`, `tests/golden`, `journal/`.
- `justfile` at root: `verify` (umbrella), `verify-<zone>` per
  `docs/07-verification.md` (all stubs except `octocore`, once it exists),
  `capture`, `report`, `manual`, `journal`.
- `adr/0001-portable-cpp-core-with-adapters.md` (moved from `docs/adr/`) and
  `adr/0002-supersede-v1-adopt-wenge.md` (new — archives both prior efforts).
- `archive/v1-max4live/` and `archive/v1-cpp-juce/` — both prior
  implementations, preserved, explicitly not to be merged. See ADR-0002.

## What's missing / blocking

- **No reference manual, no reference photography.** `reference/manual/` and
  `reference/plates/` are empty. Panelwright cannot build
  `contracts/panel.truth.json` and Metronome cannot cite behavioral constants
  without these. See `reference/NOTES.md`. This blocks real (non-placeholder)
  work in both roles.
- `contracts/panel.truth.json`, `motion.registry.json`, `materials.json`,
  `room.schema.json` — none exist. Deliberately not fabricated (would violate
  N1/N2). The three *schema* files for these already exist in `contracts/`.
- No renderer of any kind exists yet (`apps/OctoPanel` is an empty
  scaffold) — so `harness/capture` has nothing to drive and Phase 0's
  placeholder-renderer exit criterion is not yet reachable.
- Xcode (full, not just Command Line Tools) is not installed in this dev
  environment — `xcodebuild` and the Metal shader compiler are unavailable.
  `octopanel`/`octoshell`/the VST3/AU hosts cannot be compiled here until
  that's resolved.
## `crates/octocore` (2026-09-06, same session, after the bootstrap above)

Real, tested, and committed — not a stub. Rust toolchain installed (`rustup`,
stable 1.98). Ported/implemented from `docs/03-sequencer-core.md` +
`archive/v1-max4live/octopus_{engine,data,scale}.js` (the archived JS is the
only prior working implementation; the archived C++ port has no
chains/effector/hypersteps at all, so it contributed only the domain-type
shape, not behaviour):

- Domain model (`domain.rs`): Grid/Bank/Page/Track/Step, fixed-size arrays
  throughout except `Grid.banks` (heap `Vec<Bank>` — ten `Bank`s is ~1MB,
  too large to construct as a stack value safely; see the comment on
  `Grid` for why. Below `Bank`, everything is a plain `Copy` struct — `Page`
  (~6KB) is copied once per tick as the engine's per-tick snapshot).
- Deterministic seeded RNG (`rng.rs`, splitmix64) — replaces v1's
  `Math.random()`, which made the JS engine non-reproducible. Every
  stochastic decision (GRV shuffle range, direction 4/5, chord polyphony
  pick) draws from it.
- `tables.rs`: the GRV shuffle table and chord strum table, ported from v1,
  flagged PROVISIONAL (not cited against a real manual page — see
  `tests/conformance/AMBIGUITIES.md`).
- `engine.rs`: the tick loop. Processes tracks **descending** (9→0), not
  ascending like v1 — required for the effector (feeders must be computed
  before the listeners reading them, same tick) and for step-event timing.
  Implements: transport, per-track multiplier/accumulator, all 5 fixed
  directions + chains + the effector (feeder/listener) + GRV shuffle +
  scale quantization + chords/strum/polyphony + MCC step CC + sample-accurate
  event scheduling via `render()` (an `f64` absolute-sample timeline, not
  buffer-quantised). Phrases, hyperstep-carry, and step-events exist as real
  data models with working resolution/carry functions, but are not yet wired
  into the note-firing path — see AMBIGUITIES.md, they need the manual (or at
  least a real `panel.truth.json`/input path) to do correctly rather than
  guessed at.
- `fixture.rs` + `crates/octocore/tests/conformance.rs`: the DSL runner from
  §8, walking `tests/conformance/**/*.fixture` (excluding `pending/`).
  `tests/conformance/effector/feeder_pit.fixture` is the doc's own worked
  example, transcribed verbatim, and it passes.
- 18 unit tests + the 1 conformance fixture, all green, in both debug and
  release (`cargo test -p octocore` / `--release`). `just verify-octocore`
  and `just verify-conformance` are both wired for real now (see justfile).
- Six real open ambiguities logged in `tests/conformance/AMBIGUITIES.md`,
  all against the archived priors rather than a manual citation, because
  there is still no manual in this repo.

**Not done:** true `no_std` (crate is plain `std` right now — `Vec` in
`Grid.banks` and in the test-only `fixture.rs`); `verify:timing` (jitter
measurement — needs a null host); `octoffi` (the C ABI boundary — exists as
an empty placeholder, depends on this crate's API settling further);
anything requiring the real manual (constant citations, ≥250 conformance
fixtures, the direction-4/5 and phrase-type-2/3 ambiguities).

## Fan-out

Still mostly premature — Phase 0's harness/renderer/photography/manual
blockers (above) are unchanged. The one thing that *can* now safely proceed
in parallel, per `docs/09-roadmap.md` "Sequencing notes" ("Metronome can port
octocore during Phase 0 against the null host, because the core has no
rendering dependency at all"): more `octocore` work — deepening
phrases/hyperstep/step-events, or `octoffi` once someone wants to start the
Swift side against a `cdylib`. Everything else still depends on the manual,
photography, or Xcode.

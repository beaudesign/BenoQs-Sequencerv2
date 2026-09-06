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
- `crates/octocore` — status as of this writing: in progress in this same
  session (concurrent with the bootstrap work above). Whoever next edits
  this file should replace this line with what's actually true of it rather
  than trust this note.
- Xcode (full, not just Command Line Tools) is not installed in this dev
  environment — `xcodebuild` and the Metal shader compiler are unavailable.
  `octopanel`/`octoshell`/the VST3/AU hosts cannot be compiled here until
  that's resolved.
- No git commits exist yet for any of this — everything above is
  uncommitted at the time of this entry. See this session's commit for the
  first one.

## Fan-out

Not yet meaningful — Phase 0 hasn't produced a real ratchet, so there's
nothing for a multi-agent fan-out to build against safely. Per
`docs/08-agent-operating-model.md` §4: "the first vertical slice is
deliberately narrow and mostly serial. Fan out after it lands, not before."

# ADR 0003: Drop `archive/v1-max4live/` from the live tree

## Status

Accepted. Amends ADR-0002 (does not reverse its reasoning about supersession —
only the "keep both archives in this repo's live tree" mechanism).

## Context

This repo (`BenoQs-Sequencerv2`) was split out from `beaudesign/BenoQs-Sequencer`
after it became clear the WENGE spec this repo implements had been applied to
the wrong project, and `origin/main` over there had continued real, active
development of the Max4Live JS implementation in parallel (PRs #4-#9:
extracting `octopus_scheduler.js`/`octopus_state.js`/`octopus_schema.js`/
`octopus_presets.js`/`octopus_live.js` from the monolith, six ADRs, a 51-test
suite, a CHANGELOG). That original repo is now archived (read-only) on
GitHub, not deleted — its full history, including everything ADR-0002
preserved under `archive/v1-max4live/` here, remains permanently browsable
there in a more complete form than the snapshot this repo carries.

That makes `archive/v1-max4live/`'s copy in *this* repo's live tree doubly
redundant: it was already a superseded snapshot when ADR-0002 archived it,
and the canonical, more complete version of that same lineage now lives in
its own dedicated, permanently-preserved repository.

`archive/v1-cpp-juce/` is different and unaffected by this ADR: that C++/JUCE
prototype was never committed to `beaudesign/BenoQs-Sequencer`'s history at
all (it existed only as uncommitted local files before this project's
session began) — this repo's own history is the *only* place it's preserved.

## Decision

Remove `archive/v1-max4live/` from this repo's live tree. It is not lost:
it remains in this repo's own git history (the commit that moved it here is
unchanged), and in full, more-complete form in
`https://github.com/beaudesign/BenoQs-Sequencer` (archived, permanent).
`archive/v1-cpp-juce/` stays, unaffected.

Code comments citing `archive/v1-max4live/...` as the historical provenance
of a ported-then-manual-verified constant (in `tables.rs`, `scale.rs`,
`engine.rs`, `rng.rs`, `domain.rs`) are left as-is — they document lineage,
not a live dependency, and remain accurate whether or not the file is
present in this tree.

## Consequences

- This repo carries only what's actually relevant to its own project (the
  Rust core and its supporting docs), not a stale duplicate of a different
  project's history.
- Anyone wanting to see the original JS implementation's full history —
  including everything this repo's snapshot ever had, and more — goes to
  `beaudesign/BenoQs-Sequencer` directly, which is authoritative for that.
- `archive/v1-cpp-juce/` remains the one place its contents are preserved at
  all; it is not touched by this decision.

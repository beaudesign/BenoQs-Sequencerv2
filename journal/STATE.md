# STATE

Capped at 200 lines. Conductor prunes rather than appends when it grows past
that. Last pruned: 2026-09-06 (rewritten after the repo split — this is now
`BenoQs-Sequencerv2`, not a branch/PR against the original).

## This repo's origin (read this before anything else)

This project split out from `beaudesign/BenoQs-Sequencer` after the WENGE
spec this repo implements turned out to have been applied to the wrong
project — that repo's `origin/main` had continued real, active JS/Max4Live
development in parallel (PRs #4-#9) while a local session here archived that
same code and built a from-scratch Rust core on top of a spec meant for
somewhere else. The original repo is now archived (read-only) on GitHub, not
deleted — full history preserved there permanently. See `README.md` and
`adr/0003-drop-archived-js-from-live-tree.md` for the full account.
`archive/v1-max4live/` (a stale, doubly-redundant snapshot of that JS code)
has since been removed from this repo's live tree per ADR-0003.
`archive/v1-cpp-juce/` stays — it's the *only* place that earlier C++
prototype is preserved at all.

**Process fix from this history:** `.claude/skills/commit-and-push/SKILL.md`
now exists because commits sat local-only for a full prior session and were
never pushed until asked — push after every commit or small batch, not just
commit. Referenced in `agents/CLAUDE.md`'s "nine things" list.

## Phase

**Phase 0 (the ratchet) — scaffolding done, the ratchet itself not built.**
No real gate exists yet except the umbrella structure and `octocore`'s own
`verify-octocore`/`verify-conformance`. Per `docs/09-roadmap.md`, Phase 0
exit criteria (`just verify` green under 4 min, `verify:determinism`
passing, placeholder renderer discriminating gates, report rendering, eight
worktrees through the merge queue) are **not met**. Per "Sequencing notes,"
work with no rendering dependency (`octocore`, now `octoroom`'s pure-Rust
half) is explicitly allowed to proceed in parallel during Phase 0.

## Reference material: manual present, photography still one insufficient plate

- `reference/manual/` has the real CE v5.30 manual, page-indexed
  (`reference/manual/INDEX.md`, `just manual <topic>`), including a bundled
  2007 tutorial series that resolved a design question (custom direction
  multi-trigger timing) the main chapters didn't cover alone.
- `reference/plates/web-frontal-01.jpg` — one uncalibrated frontal photo,
  topology/count sanity-check use only (see `reference/plates/NOTES.md`).
  Does **not** unblock `contracts/panel.truth.json` at any tolerance — still
  need ≥3 calibrated plates per `docs/01-panel-truth.md` §3.1.
- `contracts/panel.truth.json`, `motion.registry.json`, `materials.json`,
  `room.schema.json` — still none exist. Deliberately not fabricated (N1/N2).
- No renderer exists (`apps/OctoPanel` empty scaffold); Xcode (full) still
  not installed here — Metal, signing, VST3/AU/octopanel/octoshell builds
  remain out of reach in this dev environment.

## `crates/octocore`: real, tested, manual-corrected, still growing

23 commits against the real manual, plus the 2026-09-09 phrase/rotate pass.
`tests/conformance/AMBIGUITIES.md` has the full per-topic detail; headlines:

- Several early bugs were **outright wrong**, not just unconfirmed: default
  track pitches, the effector (missing MCC, wrong timing, no listener gate),
  chord polyphony's random-pick, FLT (was a bool attribute — it's a
  multi-track merge op), phrase types, hypersteps (was hold-to-apply — it's
  a persistent link), custom directions (real mechanism: 16 slices × up to 9
  ordered triggers + a certainty_next probability).
- LEN/STA track scaling: was a linear 0..2x formula, confirmed wrong;
  replaced with the manual's real non-linear lookup tables (p.44-45).
- **Phrases now play**, including factory charts and POS remapping.
  `Engine::new` loads the 48 factory phrases (p.24-26). `scale_phrase_sta`
  applies the p.30 speed table (POS 8 = identity).
- **Track Rotate / Skip Rotate now play.** Hop over skip, hyperstep,
  AMT=-127. AMT is direction/distance, applied to the event's own track.
- Brownian (dir 4) has a 400-seed statistical fixture for the 2/3 forward
  split.
- 66 unit tests + 3 conformance fixtures, all green.
- **Still open, logged with citations:** generic VEL/PIT-style scaling
  table (p.53-55); attribute-map-factor step events (p.34-37); genuine
  same-tick step-event application; hyperstep LEN-scaling curve; MCC
  sub-step CC interpolation. Sparse Red/Orange factory cells should be
  spot-checked against hardware.

## `crates/octoffi`: thin C ABI wrapper + Grid-mutation surface

new/free/handle_command/render/is_running over an opaque `*mut Engine`,
plus `octocore_track_{set,get}_i32` / `octocore_step_{set,get}_i32` (landed
on `main` as `c68959a`). A pattern programmed entirely through FFI plays.
Hand-maintained `octoffi.h` (no `cbindgen` here). Phrase / phrase-note
programming is not on this surface yet — step GRV can store an index, but
the 48-slot phrase pool is still Rust-only.

## `crates/octoroom`: new, pure-Rust parts of D3

`RoomIntent` (hand-authored, no LLM available here), deterministic seeded
geometry synthesis matching `docs/06-shell-and-rooms.md` §3's room contract
shape, and the Eyring RT60 prediction §3 names as `verify:acoustics`'s
self-check (cross-checked against a hand-computed value). 10 tests. The
optical bake, acoustic ray-traced bake, and grade derivation all need Metal
compute and aren't attempted — see `crates/octoroom/README.md`.

## Fan-out

What can proceed without Xcode or real photography: attribute-map-factor
step events (p.34-37); `octoffi` phrase-pool accessors; deepening
`octoroom` (more materials, real prompt-parsing once/if an LLM call
becomes available). Everything touching the panel itself still depends
on real calibrated photography.

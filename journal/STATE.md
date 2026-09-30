# STATE

Capped at 200 lines. Conductor prunes rather than appends when it grows past
that. Last pruned: 2026-09-29 (Phase and SPEC-0001 sections rewritten, again after O6 and the O4 plan; before that 2026-09-06, rewritten after the repo split — this is now
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

**Phase 0 (the ratchet) — the ratchet exists, most gates do not.**
`cargo xtask verify` runs 14 gates. Three are real and required: `conformance`,
`regressions` (the name-based ratchet over `harness/baseline.txt`) and `determinism`
(golden event streams from `octorun`). The other 11 report `not_implemented`, which is
never a pass. Per `docs/09-roadmap.md`, Phase 0 exit criteria (`just verify` green under
4 min, placeholder renderer discriminating gates, report rendering, eight worktrees through
the merge queue) are **not met**: no renderer exists. The `verify` workflow (its `verify` and
`wasm-smoke` jobs) has run on GitHub for every branch tip of the SPEC-0001 series and passed,
but it is not yet a required check. Work with no rendering dependency (`octocore`, `octorun`, `octoroom`'s pure-Rust half)
proceeds in parallel, as the roadmap's sequencing notes allow. See `AGENTS.md` for the task
lifecycle and `specs/SPEC-0001/` for the current plan.

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
- The tests and fixtures that must keep passing are listed in `harness/baseline.txt`
  (258 tests and 11 fixtures on 2026-09-29, at the top of the PR stack). `cargo xtask verify` refuses a run that lost any.
- **Still open, logged with citations:** generic VEL/PIT-style scaling
  table (p.53-55); attribute-map-factor step events (p.34-37); genuine
  same-tick step-event application; hyperstep LEN-scaling curve; MCC
  sub-step CC interpolation. Sparse Red/Orange factory cells should be
  spot-checked against hardware.

## `crates/octoffi`: thin C ABI wrapper + Grid-mutation surface

new/free/handle_command/render/is_running over an opaque `*mut Engine`,
plus `octocore_track_{set,get}_i32` / `octocore_step_{set,get}_i32` (landed
on `main` as `c68959a`). A pattern programmed entirely through FFI plays.
Since SPEC-0001: `octocore_engine_diagnostics` (counters for refused, deferred and late
events), `render` respects `out_capacity`, and `Event` has PitchBend and ChannelPressure.
Hand-maintained `octoffi.h` (no `cbindgen` here); a test now parses it. Phrase / phrase-note
programming is not on this surface yet — step GRV can store an index, but
the 48-slot phrase pool is still Rust-only.

## `crates/octoroom`: new, pure-Rust parts of D3

`RoomIntent` (hand-authored, no LLM available here), deterministic seeded
geometry synthesis matching `docs/06-shell-and-rooms.md` §3's room contract
shape, and the Eyring RT60 prediction §3 names as `verify:acoustics`'s
self-check (cross-checked against a hand-computed value). 10 tests. The
optical bake, acoustic ray-traced bake, and grade derivation all need Metal
compute and aren't attempted — see `crates/octoroom/README.md`.

## SPEC-0001 (2026-09-29): make the sequencer run, make the factory enforce it

Owner approved phases F0 and F1 ("Okay build the spec"). Full record in
`specs/SPEC-0001/README.md`; handoffs in `handoffs/`; red runs in `handoffs/evidence/`.

- **Done, in review:** O1, O2 (Stop flushes notes, Play does not replay a backlog), O3 (event
  times do not depend on buffer size), O5 (nothing dropped silently, counters), O7 (`octorun`
  headless runner, golden hashes, WASM smoke test), O8 (real gates, ratchet, CI, CODEOWNERS),
  O10 (ordering, velocity 0, bend and pressure), O9 (fixture DSL v3, invariants), plus
  WENGE-0011 (queue headroom, overload warning) from the independent review.
- **O6 (command ring and snapshot) approved in r2 and built:** PR #11, merged into a stacked branch, not yet on `main`. Wait-free ring in,
  snapshot out, `SetTrack` and `SetStep`, additive C interface (ABI 2), no `unsafe` in
  `octocore`. Loom models plus mutation scripts; two `AcqRel` orderings are reasoned, not tested
  (needs an ARM soak).
- **O4 (integer tick clock and host lock) plan approved with all defaults in r3 (2026-09-30):**
  build as PRs 4a, 4b, 4c in order. **Built and open for review:** step 1 (#14: the null host and
  guards G1 to G5, no engine change; worst G1 deviation 0.995 samples; ten breakages of the clock each
  caught) and **PR 4a (#15: integer step accumulators, medium tier)**: 49 of 159 multipliers fired a step
  one tick late, now none, at any tick count; goldens byte-identical. Latent: no host can set a
  multiplier yet. **4b and 4c are not started:** 4b waits for the owner's answer on D0, both for the D3
  ADR (the Conductor's). **D0 is a finding: the engine counts 192 ticks per quarter note and the manual
  192 per whole note, so steps play four times too fast.** Engine tick untouched in O4; `WENGE-0012` is
  the triage record.
- **Not approved, not started:** the design-system and DSP track (Q1, unanswered).
- **Merge state on 2026-09-30:** `main` holds PRs 1 to 8 (#13, the landing PR, was merged by the
  owner). The stacked branches (`conductor/octorun`, `metronome/queue-headroom`,
  `metronome/command-ring`, `conductor/o4-release-plan`, `conductor/land-stack`,
  `conductor/factory-layer`, `referee/ratchet`, `metronome/transport-safety`,
  `metronome/timing-and-emission`) can be deleted. #14 and #15 are both based on `main`: no stack.
- **Owner-only:** branch protection and required Code Owner review (make the `verify` check
  required); the `jsonschema` dev-dependency (about 100 lockfile entries).
- **Known limits:** the 256-events-per-call cap binds above about 4,096 samples on dense
  patterns; the queue (1,024) overflows on the dense stress scene, by design and counted; the
  lookahead costs up to 12 ticks of command latency and assumes a constant tempo, so a tempo
  ramp leaves a permanent offset (24 ms after a 60 to 180 BPM ramp; O4 plan); **the tick is four
  times too short against the manual (D0, `WENGE-0012`)**; legato (manual p.16) and live `midir`
  output are not done.

**Next three steps:** (1) the owner reviews #14 and #15 and answers D0; (2) Scribe, Referee and Conductor work through `journal/metronome/requests/` (docs 02 and
03, the loom gate and an ARM soak, the `AGENTS.md` crate table, the `verify:timing` gate calling
`g1_*`); (3) 4b does not start before D0, and if D0 says fix the tick, `WENGE-0012` gets a spec first;
the D3 ADR before 4b or 4c merges.

## Fan-out

What can proceed without Xcode or real photography: attribute-map-factor
step events (p.34-37); `octoffi` phrase-pool accessors; deepening
`octoroom` (more materials, real prompt-parsing once/if an LLM call
becomes available). Everything touching the panel itself still depends
on real calibrated photography.

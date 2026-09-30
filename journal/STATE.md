# STATE

Capped at 200 lines. Conductor prunes rather than appends when it grows past
that. Last pruned: 2026-09-30 (P1: the rooms scope and the native panel path removed; Phase, reference, octoroom and
fan-out sections rewritten). Before that 2026-09-29, and 2026-09-06 after the repo split — this is now
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

**Phase 0 (the ratchet) — the ratchet exists, six of ten gates do not.**
`cargo xtask verify` runs 10 gates. Four are real and required: `conformance`, `regressions`
(the name-based ratchet over `harness/baseline.txt`), `determinism` (golden event streams from
`octorun`) and `scope` (P1: the live tree may not describe the old rooms product; banned phrases in
`harness/scope-banned.txt`, allow-list in `harness/scope-allow.txt`). The other six (`timing`,
`a11y`, `arch`, `slop`, `tokens`, `persistence`) report `not_implemented`, which is never a pass.
`geometry`, `color`, `frames`, `motion` and `acoustics` were removed with the photoreal path. The
`verify` workflow has passed on every branch tip of the SPEC-0001 series but is not yet a required
check (owner only). The product is a web sequencer (ADR-0006, `specs/SPEC-0002/`); phases and
waves are in `docs/09-roadmap.md`, the task lifecycle in `AGENTS.md`.

## Reference material: the manual is the source; photography is optional

- `reference/manual/` has the real CE v5.30 manual, page-indexed (`reference/manual/INDEX.md`,
  `just manual <topic>`), including a bundled 2007 tutorial series. The manual's MIDI and
  appendix page numbers in the index are wrong (request to the Scribe filed).
- `reference/plates/web-frontal-01.jpg` is one uncalibrated frontal photo. It no longer blocks
  anything: the web panel takes its layout from a control inventory built from the manual
  (`contracts/controls.json`, planned, P2), not from photographs. Request to the Panelwright to
  reword `reference/NOTES.md` is filed.
- `contracts/panel.truth.schema.json` and `motion.registry.schema.json` remain until P2, when
  ADR authority to retire them is needed (ADR-0006 authorises no other contract change).
  `apps/` has no scaffolds left; `apps/web` arrives with P3.

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

## `crates/octoffi`: thin C ABI wrapper + Grid-mutation surface (frozen, SPEC-0002 D5)

new/free/handle_command/render/is_running over an opaque `*mut Engine`,
plus `octocore_track_{set,get}_i32` / `octocore_step_{set,get}_i32` (landed
on `main` as `c68959a`). A pattern programmed entirely through FFI plays.
Since SPEC-0001: `octocore_engine_diagnostics` (counters for refused, deferred and late
events), `render` respects `out_capacity`, and `Event` has PitchBend and ChannelPressure.
Hand-maintained `octoffi.h` (no `cbindgen` here); a test now parses it. Phrase / phrase-note
programming is not on this surface yet — step GRV can store an index, but
the 48-slot phrase pool is still Rust-only.

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
  build as PRs 4a, 4b, 4c in order. **Built and merged (2026-09-30):** step 1 (#14: the null host and
  guards G1 to G5, no engine change; worst G1 deviation 0.995 samples; eleven breakages of the clock
  each caught, G1 fencing the mean as well) and **PR 4a (#16: integer step accumulators, medium tier)**: 49 of 159 multipliers fired
  a step one tick late, now none, at any tick count and through changes of multiplier; goldens
  byte-identical. Latent: no host can set a multiplier yet. An independent review of #14 and #16 found
  that the first 4a version drifted late on chained tracks (0.26 %); fixed by carrying the phase
  exactly. **4b and 4c are not started:** 4b waits for the owner's answer on D0, and both wait for the
  D3 ADR (the Conductor's). **D0 is a finding: the engine counts 192 ticks per quarter note and the
  manual 192 per whole note, so steps play four times too fast.** Engine tick untouched in O4;
  `WENGE-0012` is the triage record.
- **Not approved, not started:** the design-system and DSP track (Q1, unanswered).
- **Merge state on 2026-09-30:** `main` holds PRs 1 to 8 (#13, the landing PR, was merged by the
  owner) and #15, the owner's prune of two unused public helpers in `scale.rs` (no overlap with #14 or
  #16; the evidence checks "no engine change" against the branch point for that reason). The stacked branches (`conductor/octorun`, `metronome/queue-headroom`,
  `metronome/command-ring`, `conductor/o4-release-plan`, `conductor/land-stack`,
  `conductor/factory-layer`, `referee/ratchet`, `metronome/transport-safety`,
  `metronome/timing-and-emission`) can be deleted. #14, #16, #17 (the SPEC-0002 draft), #18 (the babysit skill) and #19 (the owner's prune of
  the triple-buffer accessors) have since merged into `main`.
- **Owner-only:** branch protection and required Code Owner review (make the `verify` check
  required); the `jsonschema` dev-dependency (about 100 lockfile entries).
- **Known limits:** the 256-events-per-call cap binds above about 4,096 samples on dense
  patterns; the queue (1,024) overflows on the dense stress scene, by design and counted; the
  lookahead costs up to 12 ticks of command latency and assumes a constant tempo, so a tempo
  ramp leaves a permanent offset (24 ms after a 60 to 180 BPM ramp; O4 plan); **the tick is four
  times too short against the manual (D0, `WENGE-0012`)**; legato (manual p.16) and live `midir`
  output are not done.

**Next three steps:** (1) the owner answers D0; (2) Scribe, Referee and Conductor work through `journal/metronome/requests/` (docs 02 and
03, the loom gate and an ARM soak, the `AGENTS.md` crate table, the `verify:timing` gate calling
`g1_*`); (3) 4b does not start before D0, and if D0 says fix the tick, `WENGE-0012` gets a spec first;
the D3 ADR before 4b or 4c merges.

## SPEC-0002 (2026-09-30, approved r1): a web sequencer, and the rooms scope removed

The owner named the v5.30 release notes, reference manual and the "basic navigation" blog post as
the feature and UI direction, said a "world model" (Genie-style rooms) project had slipped in, and
asked to remove it and keep the product a web app with live MIDI, external instruments and
Ableton. The draft merged as #17. **The owner replied "Approve" at 15:06 and all defaults apply**
(record in `specs/SPEC-0002/README.md`; ADR-0006). D0, the tick length, is still open.

- **Finding:** the engine has no front panel. Button and encoder commands are ignored and no LED
  is written, so the manual's roughly 140 workflows exist nowhere. MIDI is output only. The WASM
  build already exists (`just wasm-smoke`).
- **Series:** P0 (ADR-0006, the `SPEC.md` banner, the approval record) and P1 (the `scope` gate
  red first, then the removals) are two pull requests. **P2 to P5 (`octoface`, `controls.json`,
  the web app, MIDI) do not start until P0 and P1 have merged.**
- **P0 is #20 (draft) and P1 is the branch `conductor/remove-rooms-scope`.** P1 contains P0's
  commit, so merge #20 first. P1 removes `octoroom`, `apps/OctoShell`, the scaffolds of the native
  path, `docs/06` (`docs/01` and `docs/04` moved to `archive/native-panel/`), the gates `acoustics`,
  `geometry`, `color`, `frames`, `motion`; adds the `scope` gate; rewrites `SPEC.md`,
  `agents/CLAUDE.md` (rules 4 to 7), the roles and the docs. Ten `octoroom` tests left the baseline
  by `--remove ... --adr ADR-0006`; assertions 313 -> 303 -> 304 (293 tests, 11 fixtures).
- **Open finding:** `BANK_COUNT` is 10 in `octocore`, while the manual reads as 9 banks of 16
  pages (`specs/SPEC-0002/product.md`). Not changed here; a Metronome question for P2.
- **"World model" here means two things.** The factory's shared state (this file, `just report`)
  keeps its name; the rooms project is what is removed.
- **Request:** the Scribe fixes `reference/manual/INDEX.md` (MIDI is pages 93 and 94, not 109 to
  112); `journal/conductor/requests/2026-09-30-fix-manual-index.md`.
- **Hardware questions** (`tech.md` section 9) need someone with an Octopus before Wave 1 ends.

**Next three steps:** (1) the owner reviews and merges P0, then P1; (2) P2: `contracts/controls.json`
and the panel fixture format, the first five workflows red first; (3) P3: the static web app loads
the WASM engine and plays to one Web MIDI output, with spikes S1 and S2 recorded.

## Fan-out

Nothing is live except P1 in review. After P0 and P1 merge: P2 (`contracts/controls.json` by ADR,
the panel fixture format, `crates/octoface`, the first five workflows red first). What can proceed
in the engine meanwhile: attribute-map-factor step events (p.34-37). D0 (the tick) blocks O4
step 4b, and everything touching the panel waits for `controls.json`. `octoffi` is frozen.

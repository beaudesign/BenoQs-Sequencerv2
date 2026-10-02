# STATE

Capped at 200 lines. Conductor prunes rather than appends when it grows past
that. Last pruned: 2026-10-02 (P3 merged, its paragraphs shortened; the P4 plan added; P4 status updated by the forge). Before that 2026-10-01 (P2 merged; P3 started). Before that 2026-09-29, and 2026-09-06 after the repo split — this is now
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
  (`contracts/controls.json`, P2b), not from photographs. Request to the Panelwright to
  reword `reference/NOTES.md` is filed.
- P2b added `contracts/controls.json` and its schema and moved the `panel.truth` and `motion.registry` schemas to
  `archive/native-panel/contracts/` (ADR-0007). `apps/web` is P3 (merged).

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
  (342 tests and 48 fixtures after P2b: 390 assertions; 11 fixtures are the engine's, 37 are panel fixtures).
  `cargo xtask verify` refuses a run that lost any.
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
- **Series:** P0 (ADR-0006, the `SPEC.md` banner, the approval record: #20) and P1 (the `scope` gate
  red first, then the removals: #21) are **merged** by the owner. P1 removed `octoroom`,
  `apps/OctoShell`, the native-path scaffolds, `docs/06` (`docs/01` and `docs/04` moved to
  `archive/native-panel/`), the gates `acoustics`, `geometry`, `color`, `frames`, `motion`; added
  the `scope` gate; rewrote `SPEC.md`, `agents/CLAUDE.md` (rules 4 to 7), the roles and the docs.
  Ten `octoroom` tests left the baseline by `--remove ... --adr ADR-0006`; assertions now 310
  (299 tests, 11 fixtures).
- **P2 is merged (2026-10-01 12:01 Paris, #22 then #23).** P2a is ADR-0007: `controls.json` is a contract and
  `ControlId` is its `n`; panel fixtures are `.panel` files beside the engine's; `octoface` has no runtime
  dependencies. P2b added `contracts/controls.json` (247 controls, each cited to manual pages; what the manual
  leaves open is `pending` or in `open`, never guessed), `crates/octoface` (the panel controller: `input` gives
  engine commands and intents, `leds` a 512-slot LED frame), 37 asserting panel fixtures for the five workflows
  (Page mode step toggle, Step zoom, ESC, the EDIT cycle, the PLAY LED) and 15 pending ones, and the
  `conformance` gate change (counts `.panel` files, reports pending ones, fails a malformed or stray one).
  Red first; 27 of 27 controller mutants and 15 of 15 gate mutants caught; an independent review's twenty
  findings checked against the manual (`journal/panelwright/2026-09-30.md`). P2 was High tier (`contracts/**`).
- **Questions for someone with an Octopus** (before Wave 1 ends): Q21 to Q38 in `tests/conformance/panel/QUESTIONS.md`,
  the twenty in `specs/SPEC-0002/findings.md` section 6 and `tech.md` section 9. Each has a pending fixture or marker.
- **Open finding:** `BANK_COUNT` is 10 in `octocore`, while the manual reads as 9 banks of 16
  pages (`specs/SPEC-0002/product.md`). Not changed here; a Metronome question for P2.
- **"World model" here means two things.** The factory's shared state (this file, `just report`)
  keeps its name; the rooms project is what is removed.
- **Requests from P2b, open:** the Metronome, for an engine read model, Audition and PLAY-snapshot commands
  and the boot mode (`journal/metronome/requests/2026-09-30-engine-read-model-and-panel-intents.md`);
  the Scribe, for the "planned, P2" wording (`journal/conductor/requests/`).
- **Request:** the Scribe fixes `reference/manual/INDEX.md` (MIDI is pages 93 and 94, not 109 to
  112); `journal/conductor/requests/2026-09-30-fix-manual-index.md`.

## SPEC-0002 P3 (merged 2026-10-01): the engine in the browser

Plan: `specs/SPEC-0002/p3-plan.md` (§9 is "P3c as built"); decisions: ADR-0008 and its two amendments (task `WENGE-0014`). **P3a** (#24, 12:43 Paris), **P3b** (#25) and **P3c** (#26, both 17:39 Paris) are on `main`.
**What exists:** `octoweb` (15 exports, `apps/web/engine/ABI.md`; the page calls Rust through C exports because the worklet has no `TextDecoder`, `fetch` or `performance`), the worklet host, a two-port-capable Web MIDI scheduler (the page wires port 1 only), and an SVG panel
(160 matrix keys and 11 controls in a provisional layout, one Start button) drawn from `contracts/design.tokens.json` (`status: candidate`) and `apps/web/layout/panel.layout.json`. 114 Node and 26 Chromium tests, mutation 72/72, baseline 407 tests.
**Measured:** S2 passed (five golden hashes and two pressed programs equal native in an AudioWorklet, `render` allocates nothing, about 2 us a block of 2667). S1 here: no late events, stamp margin median about 65 ms, min 59.6 ms at a 30 ms lookahead. **Finding:** the page never calls `MIDIOutput.clear()` and never stamps an output earlier than its last send (amendment 1).
**Open for the owner:** S1's real-port half (a Mac with a loopback port, `apps/web/spikes/s1/page.html`; the kill criterion in `docs/09` waits for it); the tokens are ratified by the merge and the Conductor's flip to `ratified` is filed (`journal/conductor/requests/2026-10-01-docs-after-p3.md`); 160 Tab stops before the first non-matrix control; E2 asserted as median and never-late.
**Not done:** the chase-light, a tempo control, `prefers-contrast`, the other 76 controls.

## SPEC-0002 P4 (task `WENGE-0015`): two ports, the clock, MIDI in

Plan `specs/SPEC-0002/p4-plan.md` ("Accept p4", 2026-10-02 10:54 Paris); decisions ADR-0009 and its two amendments. **Merged on `main`:** the plan, ADR-0009 (P4a), two ports and the MIDI input path (P4b, #30), the engine's optional MIDI clock and ABI 2 (P4c, #31).
**P4d (in review, `forge/p4d-follower`, task `WENGE-0015`):** the clock follower. An estimator reads an incoming clock, a first-order loop steers the engine's tempo and transport from outside, the echo passes the clock on at once, and the clock state (Off, Master, Slave, Slave with echo) is in the strip with its sentences. 277 Node and 50 Chromium tests, mutation runs in `handoffs/evidence/p4d-mutation-run.txt`. Measured with the real engine in the loop: under half a tick steady from 2 s; 3.51 on a ramp (bound 4); not held at 160 BPM with 1.5 ms of output noise.
**Everything is measured against a made-up sender.** A real port's `event.timeStamp` clock is assumed. Found: Chromium's `currentFrame` jumps 1024 frames at context start (explains a flake in a P4c test, now waited out); the engine's notes sit 11 ticks after its pulses (ADR-0009 amendment 1). **Owner decides:** D0 (M3 stays `pending`); ADR-0009 amendment 2 (six departures, two defaults); S1 then S3 on a real port. P4e (ABLETON.md and the S3 harness) not started.
**D0 as arithmetic:** at 120 BPM a step is 1.5 clock pulses, not 6. F-P4-2: a track reaches port 2 by `MidiChannel` 17 to 32, which only Track zoom (Wave 2) sets; P4b's `?route=` is the stand-in.

**Next three steps:** (1) the owner merges P4d, answers D0 and the amendments; (2) P4e: `ABLETON.md` and the S3 harness; (3) the owner runs S1, then S3 on Live, and the follower's numbers are re-read against a real stream.

## Fan-out

Live: P4d in review. Not started: P4e and later. Engine work meanwhile: attribute-map-factor step events (p.34-37, #3) and the Metronome's requests from P2b. D0 (the tick) blocks O4 step 4b and P4c's acceptance.
The panel's next workflows (Track zoom, the direction map, the chase-light) wait on owner answers to Q07, the mutator questions and the chase-light question. `octoffi` is frozen.

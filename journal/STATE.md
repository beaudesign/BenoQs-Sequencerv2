# STATE

Capped at 200 lines. The Conductor prunes rather than appends when it grows past that. **Last rewritten: 2026-10-19**, from `main` at `d5ed9a0`,
with the owner's merges of 17 and 18 Oct. Before this the file had not been updated since about 2026-10-06 and still said D0 (the tick length) was
open; it was decided on 2026-10-07 (#46). The old per-slice narratives (SPEC-0001, SPEC-0002 P2 to P4) are in git: `git show d5ed9a0:journal/STATE.md`.
A line marked *(not re-checked)* was carried over from that file and may be stale. **Updated later the same day**, after #68 and #70 merged: `main` is at
`9977ec0`, and the spec table, the open pull requests and the next steps below reflect that.

## Origin and process (read before anything else)

- This repo split from `beaudesign/BenoQs-Sequencer` (now archived, read-only) after the WENGE spec was applied to the wrong project. See `README.md` and
  `adr/0003-drop-archived-js-from-live-tree.md`. `archive/v1-cpp-juce/` is the only preserved copy of the earlier C++ prototype.
- **The owner merges; the agent never merges, approves, marks a draft ready or enables auto-merge.** The one exception so far: on 2026-10-19 the owner said
  in chat "Merge #68 and #70 yourself", naming the pull requests, and the agent marked them ready and merged them. The rule stands for every other pull
  request. A merge is not an approval of a spec. Approval is
  the owner's own words, written into the spec's approval record with the date (`.claude/skills/babysit/SKILL.md`). Push after every commit
  (`.claude/skills/commit-and-push/SKILL.md`).
- **Traps that have cost work.** (1) A PR whose base is another PR's branch merges into that branch, not `main`: #6 to #8, #10 to #12 and #75 did, and
  #75's Curve button had to be carried by #77. Check `base.ref` before saying a slice landed. (2) Never share a `CARGO_TARGET_DIR` between worktrees or
  agents: a stale mutated binary showed a phantom test failure; `touch` the sources after switching branches. (3) In the cloud container `git fetch` maps
  only `main`; read other branches with `gh api` or make the ref with `git update-ref`. (4) Task ids: `git grep` before taking one. `WENGE-0036` went to two
  tasks (the Curve tests, and SPEC-0007 r1); #79 moved the spec to `WENGE-0037`, and the squash commit's title for #76 on `main` still says 0036.

## The product on `main`

A web sequencer for the genoQs Octopus (ADR-0006, `specs/SPEC-0002/`). Crates: `octocore` (engine), `octoffi` (frozen C ABI), `octoface` (panel
controller), `octorun` (headless runner). `apps/web` is the page, the worklet and `octoweb` (the engine as WASM, 22 exports, ABI 2: `apps/web/engine/ABI.md`).
What the page does today (P3 to P11, all merged):

- Opens with a demo pattern (five tracks, 23 steps); one Play/Stop button, a tempo field, the chase-light on the step each track plays (P6).
- A built-in monitor sound, on by default, with Sound and Drums buttons (P6, P10) and a metronome click (#43). SPEC-0004 S0 (#73) put the sound behind a
  `Voice` interface: `Plain` is the old tone, bit for bit, and a rig of checks (the H-observables) and an offline renderer stand ready for the next voices.
- Stop then Play carries on from where the sound stopped (ADR-0009 amendment 3, #41 and #42).
- Two Web MIDI output ports, MIDI in, the engine's optional MIDI clock master, and a clock follower with echo (P4, ADR-0009). `apps/web/ABLETON.md` is
  **not checked against Live** *(not re-checked)*.
- The panel is laid out like the instrument (242 of 247 inventory controls at measured places, P7), drawn in the hardware's materials, flat (ADR-0010,
  P8), with the strip and page in a midnight ground with hairlines (ADR-0011, P9). `contracts/controls.json` has 257 controls: the 247 and the ten
  number keys (ADR-0012, #61). The controller handles five workflows (Page mode step toggle, Step zoom, ESC, the EDIT cycle, the PLAY LED) and the
  VEL, PIT, LEN and STA knobs in Step zoom (P5b, #40).
- A Pages workflow publishes the page at an address (P11, #65).

**On the page since #68:** a Curve button (Off, Accel, Bounce, Rit) for how a track spaces a chord's roll (SPEC-0005 S0 to S2).

**Not on the page:** a way to enter a chord or a strum, scales, chains, step events, Track mode, the display, most encoder workflows (SPEC-0003's series).
So the Curve button is silent: nothing there can make a roll for it to bend.

## `octocore` and `octoffi`

- Manual-corrected against CE v5.30; per-topic detail and the open ones are in `tests/conformance/AMBIGUITIES.md`. Early bugs that were outright wrong
  and are fixed: default track pitches, the effector, chord polyphony's random pick, FLT, phrase types, hypersteps, custom directions, LEN and STA scaling
  (now the manual's lookup tables). Phrases (48 factory phrases, POS speed table) and Track Rotate and Skip Rotate play.
- **The tick is 48 to the quarter note** (D0, decided by default 2026-10-07, #46): a default step is 12 ticks, 125 ms at 120 BPM. The MIDI clock is 24
  pulses to the quarter (2 ticks). `MAX_EARLY_TICKS` is 12: the engine steps one step ahead of the audio.
- **The first note after Play sounds on tick 11: 114.6 ms at 120 BPM.** The step clock's phase starts at 0 (`steps.rs`). Known, pinned
  (`o4_guards.rs`) and parked as R0b for the owner. The figures in `specs/SPEC-0002/p6a-tick-length.md` section 5 (62.5 ms) and ADR-0009 amendment 3
  (28.6 ms) are wrong. Lead, untested: the measure boundary (`global_tick % 192`) is tick index 191, the tick step 15 fires on, so an On-The-Measure Mute
  or Solo may act one step early. `journal/forge/requests/2026-10-17-first-note-after-play.md`.
- **Still open, logged with citations:** the generic VEL/PIT-style scaling table (p.53-55); attribute-map-factor step events (p.34-37); same-tick step
  events; the hyperstep LEN curve; MCC sub-step interpolation; **the single-note random step rest (p.23, not in the engine:
  `journal/conductor/requests/2026-10-16-random-step-rest.md`)**. `BANK_COUNT` is 10 where the manual reads as 9 banks of 16 pages.
- **Known limits:** 256 events per `render` call (binds above about 4,096 samples on dense patterns); the 1,024-event queue overflows on the dense stress
  scene, by design and counted; the lookahead means up to 125 ms of command latency at 120 BPM and assumes a constant tempo; legato (p.16) and native
  `midir` output are not done.
- `octoffi` is frozen (SPEC-0002 D5): new, free, handle_command, render, is_running, the track and step getters and setters, diagnostics. The page's new
  exports go through `octoweb`, not the FFI. Phrase programming is not on the FFI.
- **The Curve chord style (SPEC-0005 S0 to S2) is on `main` since #68** (`7c7d237`): the engine's chord style and bent-strum table
  (`crates/octocore/src/curve.rs`) and the web-only `octoweb_set_chord_style`. The frozen `octoffi` does not carry it.

## Gates

`cargo xtask verify` runs ten gates. Four are real and required: `conformance`, `regressions` (the name-based ratchet over `harness/baseline.txt`),
`determinism` (golden event streams from `octorun`) and `scope` (the live tree may not describe the old rooms product). Six report `not_implemented`
(`timing`, `a11y`, `arch`, `slop`, `tokens`, `persistence`), which is never a pass. On `main` at `9977ec0` (run 2026-10-19 on a docs-only branch off it): 4 pass, 0 fail, 6 not_implemented, assertions 600 (baseline 600). CI jobs: `verify`, `wasm-smoke`, `web`
(Node tests: 432 on 2026-10-17, not re-run since #68). `verify` is not yet a required check on GitHub (owner only) *(not re-checked)*. `just report` renders the gate table.

## Specs: where each stands (2026-10-19)

| Spec | What | Stands |
|---|---|---|
| SPEC-0001 | Make the sequencer run, make the factory enforce it | Approved; O1 to O10 built except O4's 4b and 4c, whose state I did not re-check. D0 is decided |
| SPEC-0002 | A web sequencer; the rooms scope removed | Approved r1. Work for P0 to P11 is on `main`. Its circle half moved to SPEC-0003 |
| SPEC-0003 | The circle (right-hand rings) | Approved r1, "approve, all defaults", 2026-10-13. S0 merged (ADR-0012, #60; contract, #61). The record approves S0 first and says **S1 onward is not approved**; S0b is not started. Q43 to Q56 need a person with an Octopus |
| SPEC-0004 | A built-in sound that behaves like hardware | r1.2. The owner's "Okay build" (2026-10-17 14:46) approved **S0 only**; the record is on `main` since #70. S0 merged (#73). S1 to S7 not approved. The Verbos-model voice is slice S7 (D12) |
| SPEC-0005 | Curve: a bent strum for chord rolls, from benny's curved time | r1.3, on `main` since #68. S0 to S2 approved ("Let's build r1", 2026-10-17 14:50) and built (#72, #77). S3 and S4 not approved. Every question answered (2026-10-18) |
| SPEC-0006 | A modular kit beside the sequencer | r0.1 merged (#74), planning only. K0 not approved. Q-S6-1 to Q-S6-9 open |
| SPEC-0007 | A pedalboard below the sequencer, Ghosts as pedal 1 | r1 on `main` (#76), planning only. r2 on `main` (#79, merged 2026-10-19) proposes the information flow (`specs/SPEC-0007/information-flow.md`), a two-lane fork and Mixer, and slices P0a and P2. **Nothing approved**: the merge is not an approval. Q-7-1 to Q-7-21 open |
| handoff v2 | Shape player and S7 Complex (`handoffs/eng-spec-shape-player-and-complex-voice-v2.md`, #71) | Planning only. 14 open questions (QG1 to QG3 and others). It says the shape player starts with `specs/SPEC-0006/`, which is the modular kit now; the next free spec number is SPEC-0008 |
| SPEC-0008 | Playing hardware synthesisers and live MIDI | r0, planning only, written 2026-10-24 on the owner's ask (`specs/SPEC-0008/`, `bench.md`). **Nothing approved.** Thirteen findings (nothing persists; a swapped or lost device hangs notes; one throwing send ends a batch; no page-side panic; no per-device timing; no Song Position out; the first note is 11 ticks late), slices H0 to H7 (H0, H1a and H1b need no decision), Q-8-1 to Q-8-11. It takes the number the handoff row above calls free: **the next free spec number is SPEC-0009** |

ADRs on `main`: 0001 to 0012 and 0014 (0006 web sequencer, 0007 controls contract, 0008 C exports and tokens, 0009 the clock, 0010 materials, 0011 midnight
ground, 0012 number keys, 0014 the sound is a feature, #70). **0013 is reserved** for the shape player (the handoff, QG3). 0015 is reserved by SPEC-0007.

## Open pull requests

- Mine: #83 (SPEC-0007 r2.1, the owner's answers to Q-7-16 and Q-7-19; marked ready by the owner 2026-10-23, CI green, not merged) and the SPEC-0008 pull request.
- Older, reviewed 2026-10-23 against `main` at `d0d1992` (read-only; I closed nothing). **#52, #56 and #57:** 98.9%, 99.7% and 100% of their added lines are already in `main`
  (ADR-0010 arrived in #54, ADR-0011 and P9 in #58), so they look superseded; the owner decides whether to close them. ADR-0010 on `main` still reads "Proposed... takes effect when the owner
  merges the pull request that carries it" (that is #52): the owner says whether merging #54 counted, and the status line then changes. **#64** (the README for a newcomer) is still needed, since `main`'s
  README still says there is no web app, but it conflicts on `README.md` with the owner's 17 Oct edit; the Scribe rebases it. **#3** (attribute-map-factor step events) is not superseded, but git finds no common
  ancestor with `main` (it forked from a Sept 9 history) and it cannot merge; it is a reference for a future Metronome task.

## Questions only the owner can answer

- SPEC-0004: **Q-S4-1** (which voice character first; default Saw ladder), Q-S4-2, Q-S4-3 and **Q-S4-5** (bit-exact sound between a native build and the
  browser is given up, D13) on their defaults. Q-S4-4 is answered by #71. A hole left for the Conductor: whether a note's sound may depend on what its
  voice slot played before (the rig does not forbid it; S1's drift might need it).
- SPEC-0005: all answered 2026-10-18 (r1.3). The Curve button waits for chord entry on the page.
- SPEC-0006: Q-S6-1 to Q-S6-9. SPEC-0007: Q-7-1 to Q-7-21 (Q-7-16, what "performance" means, and Q-7-19, step-locked CC now or later, decide what is built
  first). The handoff: QG1 (which way the crate dependency runs), QG2, QG3 and the rest.
- R0b: the first note 114.6 ms after Play, and the tempo-change latency. Whether to fix it moves every golden. SPEC-0008 makes it matter more: a drum machine slaved to our Start plays its first step 11 ticks before our first note (Q-8-4).
- SPEC-0008: Q-8-1 to Q-8-11 (Q-8-1, the owner's gear, decides what the bench can use). H0 is a run by the owner with any one device; H1a and H1b need no decision.
- SPEC-0003: someone with an Octopus reads the thirteen ring names (D5); Q43 to Q56. SPEC-0002: Q21 to Q38, and a real-port run of S1 and S3
  against Live *(not re-checked)*. Owner-only: branch protection and making `verify` required; the `jsonschema` dev-dependency *(not re-checked)*.

## Requests open (the zone owner picks them up; nobody edits across zones)

`journal/metronome/requests/` (18 files; some are answered by merged work and I did not re-read them), `journal/conductor/requests/` (including the random step
rest and `2026-10-12-enforce-the-tinted-roles-expiry.md`), `journal/forge/requests/` (including the R0b figures), `journal/panelwright/requests/`. Carried
over, *(not re-checked)*: the Conductor removes the old clock test name by ADR (`journal/metronome/requests/2026-10-07-d0-decided-by-default.md`); docs
that said 192 ticks to the quarter (`docs/02`, `docs/03`, `AGENTS.md`, `SPEC.md`, `tech.md`).

## Next three steps

1. The owner answers Q-S4-1 so SPEC-0004's S1 has a character to build (without S1 nothing new is heard), and reads `specs/SPEC-0007/information-flow.md` and answers
   Q-7-16 to Q-7-21 (or says "defaults"). Q-7-16 (what "performance" means) and Q-7-19 (step-locked CC now or later) decide what is built first.
2. The owner decides R0b, and which of SPEC-0003 comes next (S0b, then S1: the tempo knob and number keys; chords and strum are S4).
3. Nothing of SPEC-0004 S1 or later, SPEC-0005 S3 or S4, SPEC-0006 K0, SPEC-0003 S1 onward or SPEC-0008 H1a onward starts without the owner's own words in the approval record.

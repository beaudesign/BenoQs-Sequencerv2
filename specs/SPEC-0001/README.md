# SPEC-0001: Make the sequencer run, and make the factory enforce it

| Field | Value |
|---|---|
| Spec | SPEC-0001 |
| Revision | r2 (r0 as published on 2026-09-29; r1 adds the open-question resolutions; r2 records the second approval, below) |
| Audited commit | `6ef921f` |
| Stage | Product spec and tech spec approved. F0 and F1 implemented and in review. O6 approved for implementation (r2). O4 approved for a release plan only (r2). |
| Risk tier | Medium overall. O4 is high: its release plan is written for sign-off, and no O4 code exists or may be written until the owner approves that plan. |
| Owner and approver | `@beaudesign` |
| Original write-up | Private artifact published 2026-09-29, not a source of truth. This directory is. |

## Files

| File | Contents |
|---|---|
| `findings.md` | The audit: what was run, what it showed, the ten opportunities with evidence. |
| `product.md` | Problem, users, scope, acceptance examples A1 to A7, invariants, failure behaviour. |
| `tech.md` | Current design, proposed change, files touched, risks, alternatives, plan. |
| `o6-test-plan.md` | The tests for `WENGE-0006`, written before the ring code (r2). |
| `o4-release-plan.md` | Release plan for `WENGE-0004`, for the owner to approve or reject (r2). Not yet written at this commit. |

## Approval record

**r1, 2026-09-29, 13:28.** The repo owner reviewed r0 and replied "Okay build the spec".
That is recorded here because chat is not a source of truth. The reply approved the product
spec and the tech spec for the work in phases F0 and F1. It did not approve O4, O6, phase
F2 or F3, or the design system and DSP track.

**r2, 2026-09-29, 15:11.** The owner replied "Let's continue - I approve". That message
does not say what it approves, so the agent asked, and the owner picked exactly two of the
options offered. The record of what r2 covers:

| Item | What is approved | What is not |
|---|---|---|
| O6, `WENGE-0006` command ring and snapshot | Build it, as its own pull request, using the Q6 default (hand-rolled ring, tested with loom). The loom test plan is written and committed before any ring code. | Merging it. The agent never merges. |
| O4, `WENGE-0004` integer tick clock and host lock | Write a release plan and re-read both specs, for the owner to sign off. | Any O4 code. High tier: needs the owner's approval of the plan, then a pull request. Nothing for O4 is built until then. |
| Design system track (Q1) | Nothing. | Everything. It needs its own spec and its own approval. Q1 is still unanswered. |
| DSP track (Q1) | Nothing. | Everything. |
| Merging #9 and #10 | Nothing. | Stays with the owner. |

Anything the owner did not pick in r2 is still not approved, however close it sits to
something that was.

## Open questions and how they stand

The owner did not answer Q1 to Q6 one by one. Where the spec named a default, the
default applies until the owner says otherwise. Any answer overrides it and gets a
revision.

| # | Question | Resolution |
|---|---|---|
| Q1 | "Designs dns system": design system, DSP or both? | Default applies: design system first, DSP second. Both are parallel tracks and out of scope for this PR series. |
| Q2 | What should "runs" mean first? | Default applies: headless runner and WASM demo (O7), not a plugin host. |
| Q3 | Who approves medium and high tier work? | Default applies: `@beaudesign` approves all tiers. |
| Q4 | May the agent open pull requests, and what is the spend cap? | Answered by the instruction to build: branches are pushed and pull requests are opened. The agent does not merge. Retry cap is 3. Spend cap is unset. |
| Q5 | Retrigger of a sounding pitch, and stop of a sounding note? | Default applies: implicit NoteOff first, stop flushes all. Must be checked against the manual before the O1 and O10 fixtures lock it in. If the manual disagrees, the manual wins and this row is revised. |
| Q6 | Command ring: hand-rolled or crossbeam? | Default applies: hand-rolled with loom. Confirmed for O6 by the r2 approval. See `o6-test-plan.md` for what "hand-rolled" turned out to mean (no `unsafe`, which `docs/02` §3 reserves for `octoffi`). |

## Task IDs

Task number equals opportunity number, so `WENGE-0007` is O7. `WENGE-0000` is the
factory layer that carries this spec.

| Task | Opportunity | Tier | Phase | Status |
|---|---|---|---|---|
| WENGE-0000 | Factory layer: `AGENTS.md`, ADR-0004, this spec, CODEOWNERS, handoffs | Medium | F0 | Merged to `main` (PR 1) |
| WENGE-0001 | O1 Flush sounding notes on stop and reset (pilot) | Medium | F0 | Merged into a parent branch, not yet on `main` (PR 3) |
| WENGE-0002 | O2 Stop the backlog burst on Play after idle | Medium | F0 | Merged into a parent branch, not yet on `main` (PR 3) |
| WENGE-0003 | O3 Timing independent of buffer size | Medium | F1 | Merged into a parent branch, not yet on `main` (PR 4) |
| WENGE-0004 | O4 Integer tick clock and host lock | High | F2 | Release plan for sign-off (`o4-release-plan.md`). No code, and none until the plan is approved. |
| WENGE-0005 | O5 Count dropped events, never drop a NoteOff | Medium | F1 | Merged into a parent branch, not yet on `main` (PR 4) |
| WENGE-0006 | O6 Command ring and snapshot | Medium | F2 | Approved in r2. Built as PR 7, in review. Test plan first (`o6-test-plan.md`, section 10 records where the build differed). |
| WENGE-0007 | O7 Headless runner, golden streams, WASM demo | Medium | F1 | In review (PR 5). Live `midir` mode not done (Q6). |
| WENGE-0008 | O8 Real gates: tri-state verify, CI, CODEOWNERS, ratchet | Medium | F0 | Merged into a parent branch, not yet on `main` (PR 2) |
| WENGE-0009 | O9 Fixture DSL v2 and metamorphic invariants | Low | F0 and ongoing | Started; merged into a parent branch, not yet on `main` (PR 3: DSL v2, transport invariants. PR 4: DSL v3 `vel`, `mcc`, `bend`, `pressure`, and buffer-size, order and velocity invariants) |
| WENGE-0010 | O10 Emission details and doc drift | Medium | F1 | Merged into a parent branch, not yet on `main` (PR 4). Doc drift was fixed in PR 1. |
| WENGE-0011 | Review findings: queue headroom (1,024), overload counters in `octorun`, `play N step` guard | Medium | F1 | In review (PR 6). Task added by the independent review. |

## Pull request series

A linear stack: each branch is based on the one above it, so each pull request shows
only its own change and the ratchet from the earlier PR protects the later ones. The
owner merges, in order. Nothing merges on the strength of an agent's own status message.

| PR | GitHub | Branch | Tasks | Base |
|---|---|---|---|---|
| 1 | #5, merged into `main` | `conductor/factory-layer` | WENGE-0000 | `main` |
| 2 | #6, merged into `conductor/factory-layer` | `referee/ratchet` | WENGE-0008 | PR 1 |
| 3 | #7, merged into `referee/ratchet` | `metronome/transport-safety` | WENGE-0001, 0002, 0009 | PR 2 |
| 4 | #8, merged into `metronome/transport-safety` | `metronome/timing-and-emission` | WENGE-0003, 0005, 0010 | PR 3 |
| 5 | #9, open, retargeted to `main` | `conductor/octorun` | WENGE-0007 | PRs 1 to 4 |
| 6 | #10, open | `metronome/queue-headroom` | WENGE-0011 | PR 5 |
| 7 | opened after this commit | `metronome/command-ring` | WENGE-0006 | PR 6 |

**What happened to the stack.** The owner merged #5 to #8 in order, but the branches were not
deleted after each merge, so GitHub did not retarget the next pull request to `main`. #6, #7
and #8 were merged into their parent branches and `main` holds only PR 1. Nothing was lost:
PR 5's branch contains all of it, so #9 now targets `main` and merging it brings PRs 2 to 5
in. For a stack, delete each branch when it merges, or merge the top of the stack.

`journal/STATE.md` is updated once at the end of the series, so the branches do not
conflict on it.

## Evidence rule

Each new test must fail on `6ef921f` (or on the parent commit of the fix) and pass
after the fix. The handoff record for the task holds both results.

# Review guide: the SPEC-0001 pull requests, for review on 2026-09-30

Written on 2026-09-29 evening. Four pull requests are open and none is merged. **The agent
merges nothing.** CI (`verify` and `wasm-smoke`) is green on the tip of every one of them.

> **Update, 2026-09-30.** All four pull requests were merged, but only #9 into `main`. #10, #11
> and #12 went into their stacked base branches (the branches were not deleted), so `main` holds
> PRs 1 to 5. Nothing is lost: `metronome/command-ring` has all of PRs 6 to 8, and the pull
> request from `conductor/land-stack` into `main` lands them. **Merging #12 was not approval to
> build O4: D0 to D6 are still unanswered.** The rest of this guide is as written on 2026-09-29.

## 1. The four pull requests

| PR | Branch → base | What it is | Size | Tier | Needs from you |
|---|---|---|---|---|---|
| **#9** PR 5 | `conductor/octorun` → `main` | The headless runner `octorun`, golden event streams, the `determinism` gate, the WASM smoke test. Because #6 to #8 were merged into their parent branches and not into `main`, this diff is **PRs 2 to 5 together**: the ratchet, transport safety (O1, O2), timing and emission (O3, O5, O10) and the runner (O7) | 85 files, +7,407 −166, 11 commits | Medium | A review, then merge |
| **#10** PR 6 | `metronome/queue-headroom` → `conductor/octorun` | Queue of 1,024 events, overload warnings in `octorun`, the `play N step` guard (WENGE-0011, from the first independent review) | 28 files, +467 −50, 4 commits | Medium | A review, then merge. **One decision** (§3) |
| **#11** PR 7 | `metronome/command-ring` → `metronome/queue-headroom` | O6: a wait-free command ring in, a snapshot out, `SetTrack` and `SetStep`, additive C interface (ABI 2). No `unsafe` in `octocore` | 42 files, +4,489 −178, 8 commits | Medium | A review, then merge |
| **#12** PR 8 | `conductor/o4-release-plan` → `metronome/command-ring` | The O4 release plan (revision 2). **No code.** Plus measurements, the `WENGE-0012` triage, two `AMBIGUITIES.md` entries | 15 files, +1,116 −4, 3 commits | High (plan) | **Decisions D0 to D6.** Merging it is not approval to build |

Each branch is based on the one above it. Nothing in #12 touches `crates/`.

## 2. What to read first

Merge order and reading order differ, on purpose.

1. **#12, section 0 of `specs/SPEC-0001/o4-release-plan.md`.** It is short, it is the only one
   that needs *your answers*, and D0 is a finding you should know about before you read
   anything else: **the engine's tick is four times too short** (the code has 192 ticks per
   quarter note, the manual 192 per whole note; steps play four times too fast). Nothing
   about it is fixed, on purpose.
2. **#9.** The largest diff and the foundation: the gates, the ratchet, the runner. It has
   already been through one independent review (fixes are in its last commit).
3. **#10.** Small. The decision in §3 is here.
4. **#11.** The most careful piece of engineering: read `crates/octocore/src/ring.rs` and
   `triple.rs`, then the evidence in its description. Its known limits are listed in §4.

## 3. Decisions waiting for you

| Where | Decision | My default | Blocks |
|---|---|---|---|
| #12, D0 | The tick length: fix the engine to the manual, or record why it is right. **A real Octopus settles it: at 120 BPM does a 16-step pattern at ×1 take 2 s or 0.5 s?** | Engine untouched; decide before O4's second PR | Every golden and timing figure |
| #12, D1 | Approve, change or reject the O4 plan | Approve | All of O4 (nothing is built before your answer) |
| #12, D2 | Drop the PLL from `octocore` (a plug-in host counts samples exactly) | Drop | 4c |
| #12, D3 | Accept a weaker ramp criterion than A4 (a **loosening**: needs an ADR with reason and expiry), or pay one buffer of latency | Accept, with the ADR | 4b, 4c |
| #12, D4 | Who owns the transport: host flag, commands, or last change wins | Last change wins | 4c |
| #12, D5 | Locate, loop wrap, frozen position: release notes, keep step positions | Release notes | 4c |
| #12, D6 | Three O4 pull requests instead of one | Three | Order of work |
| #10 | When a real pattern overloads the event queue: refuse and count (today), thin the pattern, or tell the player | Refuse and count | Nothing; today's behaviour stands |
| Settings | Make `verify` a required check; require Code Owner review; delete each branch on merge | n/a | Only you can do these |
| Settings | Allow the `jsonschema` dev-dependency (about 100 lockfile entries) | n/a | The handoff-schema check |
| SPEC-0001 Q1 | "Designs dns system": design system, DSP or both? Still unanswered. **Not started, not approved** | Design system first | That whole track |

To answer: reply on #12 (or in the session) with D0 and D1 and whichever of D2 to D6 you
want to change. I record it in the spec README as r3 before anything is written.

## 4. Known limits, so nothing surprises you

- **O6, the ARM gap.** Two `AcqRel` orderings in `triple.rs` are reasoned, not tested. Loom
  does not model that reordering, and the soaks ran on x86-64. The product ships on Apple
  Silicon. An ARM soak is the fix (Referee request, item 6).
- **O6, the snapshot leads the sound by about 13 ticks.** A display driven by it would light
  early. Documented; publishing the audible tick instead is your design choice.
- **O6, the C header test does not link.** A signature that disagrees between the header and
  the Rust would not be caught. Follow-up.
- **Loom is not in any required gate.** Request to the Referee. Until then the loom result is
  what #11 shows, not what CI enforces.
- **The O4 plan's "after" numbers are a model** (`handoffs/evidence/o4-model.txt`), not
  engine code. Its tests are the arbiter. No host and no Live 12 here, so a loopback test on
  your machine is a release condition for O4.
- **The independent reviews were fresh subagent sessions**, not a human or another team.
  They are evidence, not a substitute for your review.
- **Not done:** live `midir` output, legato (manual p.16), FFI phrase and MCC setters, the
  design-system and DSP track.

## 5. What to run

```
cargo xtask verify --base <the PR's base>     # conformance, regressions, determinism: PASS, nothing lost from the ratchet
just run examples/hello                        # a pattern file in, an event log and a .mid file out (#9)
just wasm-smoke                                # the engine compiled to WebAssembly writes the identical log (#9)

# #11 only
RUSTFLAGS="--cfg loom" CARGO_TARGET_DIR=target/loom cargo test -p octocore --test loom_sync --release
crates/octocore/loom/mutants.py                # breaks the ring and buffer on purpose: 7 caught, 1 equivalent survivor (M8)
crates/octocore/loom/link_mutants.py           # 12 of 12 caught by cargo test
cargo test --release -p octocore --test soak -- --include-ignored    # two-thread soaks, 100 million commands and publishes

# #12 only (reproduce the plan's numbers; see handoffs/evidence/o4-baseline/README.md)
python3 handoffs/evidence/o4-baseline/model_tick_timeline.py
python3 handoffs/evidence/o4-baseline/golden_floor_flips.py
```

Expected: `cargo xtask verify` on #12's tip says `3 pass, 0 fail, 11 not_implemented`,
assertions 269, none lost. The eleven `not_implemented` gates are never a pass; they wait
for a renderer.

## 6. Merging

1. Merge **#9** with "delete branch". GitHub then retargets #10 to `main`.
2. Merge **#10** with "delete branch", then **#11**, the same way.
3. **#12** last, or leave it open until you have answered D0 to D6. **Merging #12 does not
   approve building O4.** The approval is your reply.

If a branch is *not* deleted, the next PR is not retargeted and merges into the dead branch
instead of `main`. That is what happened to #6 to #8 (`main` holds only PR 1 until #9 lands).
If that happens again nothing is lost: #9's branch already contains everything below it.

## 7. Per-PR review checklist

`AGENTS.md` has the full one and #9 and #10 carry it. The short form:

- [ ] Inside the task's zone, or a cross-zone request exists (`journal/metronome/requests/`).
- [ ] Each new test was shown failing before its fix (`handoffs/evidence/*-red.txt`).
- [ ] No threshold loosened, no contract edited, no test deleted (the ratchet says so).
- [ ] Nothing on the audio path allocates, locks or blocks (a counting-allocator test says so).
- [ ] Docs the change makes false are fixed, or a Scribe request is filed.
- [ ] Section 4 of this guide: you accept the known limits, or say which to fix first.

## 8. Where the record is

`specs/SPEC-0001/README.md` (approvals, task table, PR series), `handoffs/WENGE-*.ndjson`
(one line per stage), `journal/metronome/2026-09-29.md` (what surprised me),
`journal/STATE.md` (the shared world model), `tests/conformance/AMBIGUITIES.md` (every place
the manual is unclear or the engine disagrees with it).

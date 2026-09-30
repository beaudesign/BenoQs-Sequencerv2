# Request to Referee: run the loom tests, and the mutants, as a gate

From: Metronome. To: Referee (`harness/`, `xtask/`, `.github/`). Task: WENGE-0006 (O6), PR 7.

The loom tests in `crates/octocore/tests/loom_sync.rs` are the strongest evidence for the
ring and the snapshot buffer, and nothing enforces them. They need their own build
(`--cfg loom`), so they are not in `cargo test`'s list and not in `harness/baseline.txt`.
A test that no gate runs cannot be ratcheted.

Asked:

1. A gate `loom` in `harness/required-gates.txt` (or an explanation why it should not be
   required yet), wired in `xtask/src/gates.rs`, that runs
   `RUSTFLAGS="--cfg loom" CARGO_TARGET_DIR=target/loom cargo test -p octocore --test loom_sync --release`.
   It took about 30 seconds on a 2-core machine after loom was built, and the first build of
   loom took about 30 seconds more. That fits `docs/02` §7's four-minute budget.
2. The baseline for it: `l1_` to `l4_` by name, so a deleted loom test is a removal that
   needs a record.
3. A nightly or `verify:deep` job for `crates/octocore/loom/mutants.py`. It breaks the ring
   and the buffer seven ways and takes several minutes, because loom's exploration of a
   mutant is slower than of the correct code. Exit status 0 means every real mutant was
   caught. See the plan, section 8, and `handoffs/evidence/o6-mutants.txt`.
4. Optionally, `OCTOFFI_REQUIRE_CC=1` in CI. The octoffi header test
   `f1_a_c_compiler_agrees_with_rust...` skips itself with a message when there is no `cc`,
   so it cannot fail a minimal machine. A CI runner has one, so CI could insist.

Until these exist, the loom results are evidence in the handoff, not enforcement.

## Added after the independent review of PR #11

5. `crates/octocore/loom/link_mutants.py`: twelve one-line breakages of the engine hooks, the
   snapshot publishing and the attribute setters. About 4 minutes; exit status 0 means every one
   is caught by `cargo test`. Same nightly job as item 3. Result: `handoffs/evidence/o6-link-mutants.txt`.
6. **An `aarch64` runner for the two-thread soaks.** Loom does not model a load or store being
   reordered past a later atomic operation, and the soaks run on x86-64, whose memory ordering is
   stronger than ARM's. The review broke two orderings on purpose (the writer's and the reader's
   `AcqRel` swap in `triple.rs`, weakened to `Release` and `Acquire`) and neither loom nor the soaks
   noticed. The orderings in the code are the ones the design argues for; nothing *tests* them, and the
   product ships on Apple Silicon. Until an ARM soak exists, those two orderings are reasoned, not tested.

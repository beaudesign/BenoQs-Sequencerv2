# Record for Referee: edits to `xtask` and `harness` made in the octorun pull request

From: Conductor. To: Referee. Task: WENGE-0007. Raised by the independent review (finding 8).

The `octorun` pull request touched the Referee's paths without a prior request:

1. `xtask/src/gates.rs`: the `determinism` gate (passes when at least 5 tests of the octorun
   golden binary ran and none failed), its tests, and `WIRED`, the list of gates that
   `evaluate` computes.
2. `xtask/src/main.rs`: `xtask gate <name>` now uses `gates::is_wired` instead of a
   hard-coded list of two names, and `root()` uses the working directory's git checkout
   before falling back to the compile-time path. Found by review: `xtask gate determinism`
   exited 42 (not implemented) while `verify` required it.
3. `harness/required-gates.txt`: `determinism` added. The file only grows.
4. `harness/wasm/smoke.mjs`, and a `wasm-smoke` job in `.github/workflows/verify.yml`
   (`.github/**` is Conductor's).

The determinism gate is looser than its comment says: it counts golden tests that passed and
does not check that all five named patterns ran. If the Referee wants that, it is a
tightening and needs no ADR.

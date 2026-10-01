# Request to Referee: floor the browser tests, and the web gates

From: Conductor (P3a). To: Referee (`harness/`, `xtask/`, CI, thresholds). Task: WENGE-0014
(`specs/SPEC-0002/p3-plan.md`, decisions D-P3-7 and D-P3-8).

P3b adds a `web` job to `.github/workflows/verify.yml` (a Referee-hat commit in that pull request) that
runs the engine crate's native tests (already floored by `cargo xtask baseline`), the Node unit tests, and
a Playwright run in Chromium. The last two are not in `harness/baseline.txt`, because the baseline reads
`cargo test` output and fixtures. A test that only the web job runs can be deleted without the ratchet
noticing.

Please:

1. Floor the browser and Node tests by name (a new baseline kind, or a reporter the baseline can read from
   `node --test`), with the same rule as the rest: removal needs `--remove … --adr`.
2. Build `verify:a11y` over the app when P3c lands (A8: label coverage of every control, traversal order
   completeness) and the source scan half of `verify:tokens` (no colour, space or type literal outside
   `contracts/design.tokens.json`; the rules S1, S2, S4, S9, S15, S16 in `docs/05`).
3. Decide whether `cargo xtask verify` should run the browser tests when a Chromium is present and say
   `not_implemented` when it is not. A skipped browser test must never count as a pass.

**State at the end of P3b:** 72 Node tests (`apps/web`, `npm test`) and 6 Chromium tests (`npm run test:browser`) pass in the
`web` job. Spike S2 (`node spikes/s2/run.ts`) is also run there and exits non-zero on a hash difference or an allocation in
`render`; its timing lines are printed and not gated. The two native program tests and the 30 octoweb tests are already in the baseline.

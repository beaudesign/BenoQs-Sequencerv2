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

## Added by P3c

**What P3c puts in the repository for these gates to start from** (all in `apps/web/`):

- `engine/tests/tokens.rs` (19 tests) asserts the rules of ADR-0008 decision 6 over `contracts/design.tokens.json` and its schema, and
  shows each rule failing on a document made to break it. This is the whole of `verify:tokens`' contract half.
- `engine/tests/app_source_rules.rs` (14 tests) asserts S1, S2, S4, S5, S7, S9, S11, S12, S15 and S16 over the app's files
  (`src`, `pages`, `layout`), the "no colour, length or type size outside the tokens" rule, and that every `var(--x)` names a token
  property. It is the source-scan half of `verify:tokens` and most of `verify:slop` for the app. **Both are Rust tests, so the baseline
  floors them by name already.** If the gates are built as separate programs, either keep these as the engine of the gate or port them
  and delete these in the same pull request with `--remove … --adr`, not before.
- `test/browser/app.browser.test.ts` (20 Chromium tests): E1, E2, E7, E8. For `verify:a11y`: every drawn control is a `button` whose
  `aria-label` is the manual's name (171 of 171, found again through the browser's own accessibility tree by `getByRole`), the Tab order
  is the layout's order, and the focus ring is drawn from the focus tokens and not left behind by a pointer press.
- `test/readme.test.ts` keeps the written traversal order in `apps/web/README.md` equal to the layout's.

**Not done, for the Referee or the owner:** `prefers-contrast` (a mode that raises the contrast of labels and `led.off`; `docs/05`
section 5), which needs a design decision about what it raises to; the screen-reader pass with a real reader (A8 says "an automated
a11y check plus a written traversal order", and that is what exists); an axe-style scan, which would add a dependency.

**State at the end of P3c:** the `web` job runs 114 Node tests and 26 Chromium tests (up from 72 and 6); the `verify` job floors
`tokens.rs` and `app_source_rules.rs`. Nothing in `harness/` was loosened.

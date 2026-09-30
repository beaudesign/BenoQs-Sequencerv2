# Request to the Scribe: "planned, P2" is no longer true once the P2b pull request merges

From: Panelwright (P2b). To: Scribe (`docs/`, `README`). The Conductor owns the two other files listed.
Nothing has been changed. Do it after the owner merges the P2b pull request, not before, because until
then the text is correct for `main`.

`contracts/controls.json`, `contracts/controls.schema.json` and `crates/octoface` exist after the merge.
These lines still call them planned:

- `docs/02-architecture.md` lines 33, 105 and 127 (the `octoface` section starts at 105): the crate is
  built, has no runtime dependencies, and takes `(n, id)` pairs from the inventory.
- `docs/03-sequencer-core.md` line 182: `ControlId` is now defined as the `n` of a control.
- `docs/00-north-star.md` lines 33 and 154; `docs/07-verification.md` line 104 (the panel fixtures are
  `tests/conformance/panel/**/*.panel`, run by `crates/octoface/tests/panel_fixtures.rs`);
  `docs/08-agent-operating-model.md` lines 40 and 72; `docs/09-roadmap.md` lines 53 and 95 (P2 status).
- Conductor's: `SPEC.md` lines 53, 82, 132, 179 and 182; `agents/ROLES.md` 37; `agents/CLAUDE.md` 58;
  `crates/octoffi/README.md` line 18.

Facts to use, all in `tests/conformance/panel/QUESTIONS.md` and `adr/0007-…`: the numbering scheme
(Q01 to Q20 are `findings.md` section 6, Q21 onward are in `QUESTIONS.md`), the fixture format, the gate's
new metrics (`conformance.panel_runner_passed`, `conformance.pending_fixtures`,
`conformance.pending_fixtures_malformed`), and the baseline (385 assertions after P2b).

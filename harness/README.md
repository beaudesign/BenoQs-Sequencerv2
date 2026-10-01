# harness/

The verification harness. Owned by the Referee (`agents/ROLES.md`). Design doc:
`docs/07-verification.md`. Decision record: `adr/0005-ratchet-baseline-and-required-gates.md`.

## Commands

```
just verify                        # every gate, writes harness/report/latest.json
just verify --base origin/main     # plus: nothing in the baseline may vanish vs main
just verify-<zone>                 # one gate; exit 42 means "not implemented"
just baseline                      # record new tests as the new floor (never drops any)
```

`just` is a thin wrapper. `cargo xtask verify` does the same thing and is what CI runs.

## What a run does

1. Runs `cargo test --workspace --no-fail-fast` once and parses the result by test name.
2. Evaluates every gate in `contracts/verification.report.schema.json`. Four gates exist
   today (`conformance`, `regressions`, `determinism`, `scope`). The other six report `not_implemented`.
   `conformance` also covers the panel fixtures (`tests/conformance/panel/**/*.panel`, ADR-0007): when
   any exist, the octoface `panel_fixtures` binary and its `all_panel_fixtures_pass` test must have run
   and passed. Fixtures in a `pending/` directory assert nothing; the gate reports how many there are
   (`conformance.pending_fixtures`, never floored) and fails one that lacks its `# pending:`, `# manual:`
   or `# question:` header or holds script lines. A `.panel` file outside `tests/conformance/panel/` is
   not counted (the runner does not read it) and fails the gate (`conformance.panel_files_malformed`).
3. Writes `harness/report/latest.json` (schema `verification.report/1`) and
   `harness/report/cargo-test.log`. Both are git-ignored. CI uploads them.
4. Prints a table, then exits **0 only if** no gate failed **and** every gate listed in
   `required-gates.txt` reported `pass`.

A gate that is not built is `not_implemented` and is never printed or recorded as a pass.
That was the old behaviour: `just verify` used to exit 0 with 12 of 13 gates unbuilt.

## Files

| File | What it is | How it changes |
|---|---|---|
| `required-gates.txt` | Gates that must pass for the run to succeed. | Grows when a gate lands, in the same commit. Removing a line needs an ADR. |
| `baseline.txt` | The ratchet floor: every test and conformance fixture that must keep passing. | Only `cargo xtask baseline` writes it. It adds. It drops nothing without `--remove <kind> <id> --adr ADR-NNNN`. |
| `report/latest.json` | The last run, as the shared world model. | Generated. Not committed. |

## The ratchet (`verify:regressions`)

`docs/07`: "assertion count is monotonic non-decreasing". A count alone can be gamed by
deleting one test and adding another, so the floor is a list of names.

- The run must still pass every `test` and `fixture` in `baseline.txt`. A deleted,
  renamed, ignored or failing test is a violation even if new tests were added.
- With `--base <ref>` (CI always passes it) no entry may leave `baseline.txt` relative to
  that ref unless a `removed` line names an ADR that exists in `adr/`. Editing the baseline
  to hide a deletion is therefore caught, and the change needs the ADR and the owner.
- Adding tests is free. Run `just baseline` and commit the new lines.

**Unit of assertion.** Today one assertion is one passing test case or one conformance
fixture. That is a proxy: per-assertion counting does not exist yet. The Referee still
spot-audits ten assertions per phase for substance (`docs/07` section 7), because names
in a list do not prove a test is meaningful.

## Adding a gate

1. Implement it in `xtask/src/gates.rs` with a test that shows it failing on a bad input.
2. Add its name to `required-gates.txt` in the same commit.
3. Add any new tests to the baseline with `just baseline`.

## Files that do not exist yet

Gates `slop`, `tokens`, `timing`, `a11y`, `arch` and `persistence` are `not_implemented`. The
directories that were scaffolds for the photoreal panel (`harness/capture/`, `harness/lint/`,
`harness/measure/`) are gone (ADR-0006); a gate that needs a directory creates it when it
lands.

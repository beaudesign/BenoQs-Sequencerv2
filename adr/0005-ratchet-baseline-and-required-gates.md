# ADR 0005: The ratchet is a test-name baseline, and `verify` is tri-state

## Status

Accepted by the repo owner on 2026-09-29 through SPEC-0001 (WENGE-0008).

## Context

`just verify` exited 0 in 2.9 s while 12 of 13 gates printed "not yet implemented". The
runner treated exit code 42 as a soft pass and wrote no report, although
`contracts/verification.report.schema.json` already has a `not_implemented` status and a
required `assertions` field. `verify:regressions`, the assertion ratchet behind rule N7,
was a stub. octoffi and octoroom tests never ran under `just verify`, and conformance ran
twice. No CI existed.

## Decision

1. `cargo xtask verify` (crate `xtask`) replaces the shell loop. `just verify` and the
   `verify-<zone>` recipes delegate to it. Runtime dependencies: none.
2. Status is tri-state in practice: `pass`, `fail`, `not_implemented` (and `skipped`,
   unused). The run exits non-zero if any gate fails or if any gate listed in
   `harness/required-gates.txt` is not `pass`. Unbuilt, unrequired gates are reported as
   such and never as a pass.
3. The report is written to `harness/report/latest.json` and must validate against the
   existing schema. A test in `xtask` proves the writer's output validates. The schema is
   unchanged.
4. `verify:regressions` is implemented as a name-based baseline (`harness/baseline.txt`):
   every listed test and fixture must keep passing. Against a base ref, an entry may only
   leave the file with a `removed` line that cites an ADR that exists. One assertion is
   one passing test case or one conformance fixture, until per-assertion counting exists.
5. The full workspace test run (octocore, octoffi, octoroom, xtask) happens once per
   verify, and feeds both `conformance` and `regressions`.
6. `.github/workflows/verify.yml` runs it on every pull request and on pushes to `main`.
7. `required-gates.txt` starts as `conformance` and `regressions`. It only grows.

## Consequences

- The schema has no "unit tests" gate, so unit test results are reported under
  `regressions`. If a dedicated gate is wanted later, adding an enum value is a contract
  change and needs its own ADR.
- The baseline must be updated in the same pull request that adds tests. A stale baseline
  is harmless (new tests are only "added"). A missing entry is a failure.
- Renaming a test is a removal plus an addition, so it needs an ADR. That friction is
  deliberate for now; if it becomes noise, add a `--rename` path that records both names.
- Branch protection with "Require status checks" naming the `verify` job, and "Require
  review from Code Owners", must be switched on by the repo owner. Until then CI is
  advisory.
- `xtask` has dev-dependencies (`jsonschema`, `serde_json`) used only by its own test.
  They add entries to `Cargo.lock` and about 30 s to a cold `cargo test --workspace`.
  They are not in the audio path and not in any shipped artifact.

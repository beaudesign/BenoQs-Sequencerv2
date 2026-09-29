# Note to Conductor and Referee: files in PR 7 outside Metronome's zone

From: Metronome. Task: WENGE-0006 (O6), PR 7. Same arrangement as
`2026-09-29-cross-zone-edits-in-pr3-and-pr4.md`: the edits are made in the PR so that it is
one reviewable change, and the owner's Code Owner review of the PR is the check. Nothing here
is a contract.

| File | Zone | What changed, and why it could not be done elsewhere |
|---|---|---|
| `crates/octoffi/src/lib.rs`, `octoffi.h`, `README.md` | Conductor | The additive link surface (open, push, stats, claim, free, version), the two new commands and `OctoSnapshot` in the header, and the header tests. The FFI setters now call `Grid::set_track_attr` and friends in `octocore` instead of holding the clamping tables themselves. No existing signature, struct or tag number changed (tests f4 and the C compiler test). |
| `Cargo.lock` | shared | `loom` and its dependencies, 174 lines, 18 packages. Compiled only under `--cfg loom`. |
| `harness/baseline.txt` | Referee | Grown with `cargo xtask baseline`, the sanctioned command: 47 tests added, none removed. |
| `handoffs/`, `specs/SPEC-0001/` | Conductor | The task record and the plan. |

Not touched: `contracts/`, `harness/required-gates.txt`, `xtask/`, `.github/`, `docs/`.

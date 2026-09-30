# Note to Conductor and Referee: files in the O4 guards commit outside Metronome's zone

From: Metronome. Task: WENGE-0004 (O4), step 1 of the release plan (the guards, no engine
change). Same arrangement as `2026-09-29-cross-zone-edits-in-pr7.md`: the edits are made in the
PR so that it is one reviewable change, and the owner's Code Owner review is the check. Nothing
here is a contract.

| File | Zone | What changed, and why it could not be done elsewhere |
|---|---|---|
| `crates/octoffi/src/lib.rs` | Conductor | One test added, `g4_render_without_a_link_allocates_nothing`, next to `e4_render_push_and_claim_allocate_nothing` and using its counting allocator. No code outside `mod tests` changed. The guard needs the C entry point without a link, and the counting allocator is private to that test module. |
| `harness/baseline.txt` | Referee | Grown with `cargo xtask baseline`, the sanctioned command: 7 tests added (6 in `o4_guards.rs`, 1 in `octoffi`), none removed. |
| `handoffs/`, `specs/SPEC-0001/` | Conductor | The task record, the guard evidence and the mutation script. |

## Request to the Referee

`crates/octocore/tests/o4_guards.rs` `g1_*` is the content the `verify:timing` gate has been
missing (`docs/07`, listed `not_implemented`). It runs in about a second in debug. If you agree,
the gate can call it directly; I have not touched `harness/` or the gate list.

Not touched: `contracts/`, `harness/required-gates.txt`, `xtask/`, `.github/`, `docs/`,
`crates/octocore/src/`.

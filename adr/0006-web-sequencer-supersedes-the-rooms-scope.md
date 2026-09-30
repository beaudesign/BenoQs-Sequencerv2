# ADR 0006: The product is a web sequencer, and the rooms scope is removed

## Status

Accepted by the repo owner on 2026-09-30 through SPEC-0002 r1 (`WENGE-0013`; the reply was
"Approve", recorded in `specs/SPEC-0002/README.md`). It takes effect for each change when the
owner merges the pull request that carries it. Supersedes `SPEC.md` in part and ADR-0002 in
part. Reverses the closed decisions listed below.

## Context

`SPEC.md` describes a product that generates a room from a prompt, lights a photoreal panel
with that room's light and convolves the sound with its acoustics. That spec was written for
another project and applied here by mistake (`README.md`). What is left of it in the tree is a
crate (`octoroom`), a 307-line document (`docs/06`), two roles, a gate (`acoustics`) and
prose in ten other files. None of it plays a note.

The owner named the v5.30 release notes, the v5.30 reference manual and a blog post on basic
navigation as the feature and UI direction, and asked for a focused web app that runs the
sequencer with live MIDI and external instruments and is available on Ableton. The audit
(`specs/SPEC-0002/findings.md`) found that the engine has no front panel, that MIDI is
output only, and that the engine already compiles to WebAssembly.

## Decision

1. **The product** is the web sequencer in `specs/SPEC-0002/product.md` and `tech.md`. Where
   `SPEC.md` disagrees, SPEC-0002 wins.
2. **Removed** (SPEC-0002 `removal-plan.md` sections 2.1 and 2.2): the `octoroom` crate,
   `docs/06-shell-and-rooms.md`, `apps/OctoShell`, the Sceneshaper and Loom roles, the
   `acoustics` gate and the `verify-acoustics` recipe, and the room, probe, impulse-response
   and shell text in the other documents. Retired (section 2.3): the native and photoreal panel
   path.
3. **The ratchet.** This ADR authorises `cargo xtask baseline --remove test <id> --adr
   ADR-0006` for exactly these ten tests, which go with the crate, and for no other test:

   `octoroom::acoustics::tests::eyring_matches_hand_computed_value`,
   `octoroom::acoustics::tests::higher_absorption_means_shorter_decay`,
   `octoroom::acoustics::tests::stairwell_3am_rt60_is_in_a_plausible_range`,
   `octoroom::acoustics::tests::zero_absorption_is_infinite`,
   `octoroom::geometry::tests::different_seeds_can_differ`,
   `octoroom::geometry::tests::mean_absorption_matches_uniform_material_input`,
   `octoroom::geometry::tests::same_seed_gives_identical_geometry`,
   `octoroom::geometry::tests::stairwell_3am_matches_the_contracts_own_numbers`,
   `octoroom::rng::tests::range_f32_stays_in_bounds`,
   `octoroom::rng::tests::same_seed_same_sequence`.

   After the removal the assertion count only grows again.
4. **Contracts.** `contracts/verification.report.schema.json` changes in the pull request that
   removes the gates it names: the gate-name list loses `acoustics` and the gates retired
   under section 2.3 of the removal plan, and gains `scope`. This ADR is the authority for
   that edit (`CLAUDE.md` rule 2). No other contract changes.
5. **A new gate, `scope`**, is added to `xtask`, written red first, and added to
   `harness/required-gates.txt` in the commit that makes it pass. It is a tightening.
6. **Closed decisions reversed** (`docs/10-risks-and-decisions.md` section 3, which said they
   were recorded so that they would not be reopened by agents who were not there; the owner
   reopens them here):

   | Decision | Was | Becomes |
   |---|---|---|
   | Render technique | Analytic ray tracing | SVG, or Canvas |
   | Antialiasing | 3x supersampling | The browser's |
   | Tonemap | AgX | Not applicable |
   | UI host | Native app, not Max for Live `jweb` | Web app. `jweb` is reopened as Tier 2, on evidence (spike S4) |
   | Plugin formats | VST3 and AUv2 | None in v1. Ableton by routing, or Max for Live |
   | IR generation | Ray traced from geometry | Removed |
   | Motion | Physical simulation | LED flash and a few state changes only |
   | Panel geometry | A measured truth file | A logical control inventory from the manual |

   Unchanged: the core language (Rust), and v1 code stays archived.
7. **Kept, and restated in the rewritten documents:** N5 (timing beats pixels), N7 (every
   taste judgement becomes a test, and the ratchet), N8 (the manual is the truth), the four
   ways v1 looked generated, the lifecycle in `AGENTS.md`, and the rule that the agent merges
   nothing.

## Not decided here

- **D0**, the tick length (`WENGE-0012`). Still open. O4 step 4b waits for it.
- **`octoffi` and O6** are frozen (SPEC-0002 D5), not deleted.
- The design of `crates/octoface`, `contracts/controls.json` and the web app: SPEC-0002
  `tech.md`, built in P2 onward.

## Consequences

- The repo describes one product. A new agent reading `AGENTS.md` and `SPEC.md` no longer meets
  a room, a probe or a shell.
- The scope gate turns the removal into a test, so the rooms scope cannot return without an
  ADR that changes the banned list.
- Sunk work goes: the 495 lines of `octoroom` (Rust, its README and manifest), the 307-line
  `docs/06`, and sections of about ten other files. All of it stays in git history. Reverting
  the removal pull request restores it.
- Tests that pass today leave the baseline, by name, in the ratchet's own `removed` records.

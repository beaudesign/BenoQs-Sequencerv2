# Request to Metronome: comments that name `panel.truth.json` or Swift

From: Conductor. To: Metronome (`crates/octocore/`, `tests/conformance/`). Task: WENGE-0013 (P1, ADR-0006).

ADR-0006 retired the native panel path. `contracts/controls.json` (P2) replaces `panel.truth.json`.
These comments still describe the old plan. They are in your zone, so I did not touch them:

- `crates/octocore/src/domain.rs` line 299, `src/engine.rs` lines 447 and 519, `src/types.rs` lines
  3 to 8: "no `panel.truth.json`", "the Swift side".
- `tests/conformance/AMBIGUITIES.md` line 271: "no `panel.truth.json`".

Not urgent, and nothing fails. The clean moment is P2, when `ControlId` gets its meaning from
`controls.json`: change the comments then, in the same pull request, so they say what is true.
The same wording sits in `crates/octoffi/` comments and `README.md`, which are mine, and I will
change those in P2 as well.

**Status, 2026-09-30 (P2b, Panelwright).** The `ControlId` doc comment in `types.rs` now says what the
number means (ADR-0007 decision 2). The rest is still yours and still harmless: `engine.rs` 447 and 519,
`domain.rs` 299, `types.rs` line 3 ("the Swift side") and `tests/conformance/AMBIGUITIES.md` 271. The
inventory now exists, so "there is no control map" is no longer true for the panel. What is still true is
that the engine does not act on `ButtonDown`, `ButtonUp` and `EncoderTurn`: the controller, not the engine,
reads `ControlId` (`crates/octoface`). A sentence to that effect is the whole change.

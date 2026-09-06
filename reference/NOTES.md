# Reference material — status

This repo currently has **no reference photography** and **no copy of the
Octopus CE OS v5.30 reference manual**.

Both are required before real work can start in two roles:

- **Panelwright** needs calibrated plates (`reference/plates/`) to build
  `contracts/panel.truth.json` per
  [`docs/01-panel-truth.md`](../docs/01-panel-truth.md). Without them there is
  no measured geometry, and N1 ("the panel is measured, not drawn") cannot be
  satisfied — inventing coordinates would violate the non-negotiable it exists
  to enforce.
- **Metronome** needs the manual (`reference/manual/`, page-indexed) to cite
  every behavioral constant per
  [`docs/03-sequencer-core.md`](../docs/03-sequencer-core.md) §1. Without it
  there is no source of truth to write conformance fixtures against, and
  `verify:conformance`'s citation lint has nothing to check against.

This is a genuine open blocker, tracked in `journal/STATE.md`, not a
placeholder to fill with invented numbers or guessed constants.

When plates and the manual arrive:
- Plates go in `reference/plates/`, calibrated, with a note per plate on what
  it's good for (per `docs/08-agent-operating-model.md` §8, the note is the
  interface — nobody should need to load the image itself into context).
- The manual goes in `reference/manual/`, split and page-indexed, so
  `just manual <topic>` can grep it and return only the relevant pages.

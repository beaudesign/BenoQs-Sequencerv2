# Reference material — status

## Manual: present, page-indexed

`reference/manual/CE-v5.30-reference-manual.pdf` (the real genoQs Octopus CE OS
v5.30 Reference Manual, 124pp, Stuttgart 2009) is in the repo, split into
per-page plain text at `reference/manual/pages/`, and topic-indexed at
`reference/manual/INDEX.md`. `just manual <topic>` greps the index and returns
the matched pages — see that file's header for the exact convention and its
page-numbering note (there's a 2-page offset between the PDF's own page count
and the manual's printed footer numbers; already accounted for).

Metronome can now cite real behavioral constants per
[`docs/03-sequencer-core.md`](../docs/03-sequencer-core.md) §1 instead of
porting guesses from the archived v1 implementations. See
`tests/conformance/AMBIGUITIES.md` for what's been confirmed, what's newly
corrected (several archived-v1-derived guesses turned out to be outright wrong,
not just unconfirmed), and what still needs a follow-up read (a handful of large
non-linear lookup tables that are confirmed to exist but not yet transcribed to
full precision).

## Reference photography: one plate, topology-check only

`reference/plates/web-frontal-01.jpg` — a single uncalibrated frontal product
photo. See `reference/plates/NOTES.md` for exactly what it can and can't be used
for. **It does not unblock `contracts/panel.truth.json`.** Panelwright still
needs ≥3 calibrated plates (known lens/calibration set, in-plane scale anchor,
ColorChecker, raw/16-bit capture) per `docs/01-panel-truth.md` §3.1 before any
geometry work — inventing coordinates from what's available now would violate
N1, the non-negotiable this requirement exists to enforce. This remains a
genuine open blocker, tracked in `journal/STATE.md`.

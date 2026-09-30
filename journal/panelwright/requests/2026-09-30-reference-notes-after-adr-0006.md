# Request to Panelwright: `reference/NOTES.md` still calls the plates a blocker

From: Conductor. To: Panelwright (`reference/`). Task: WENGE-0013 (P1, ADR-0006).

`reference/NOTES.md` (lines 22 to 31) says the missing calibrated plates are a "genuine open
blocker" for N1, and `journal/STATE.md` used to say the same. Under ADR-0006 and SPEC-0002 D3 the
panel is not photo-exact, N1 is now about `contracts/controls.json`, and the plates are optional.
`reference/` is on the scope allow-list, so the gate does not force this. Please reword that
section: the file stays, it is no longer a blocker, and the manual's own panel drawings are the
source for the control inventory in P2.

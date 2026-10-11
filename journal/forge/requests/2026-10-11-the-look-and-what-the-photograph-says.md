# Request to Conductor (and Panelwright): the look (R3), and what the photograph says that the inventory does not

From: Forge (`WENGE-0019`, `specs/SPEC-0002/p7-faithful-panel.md`). To: Conductor (`contracts/`, ADRs, `journal/STATE.md`) and Panelwright (`contracts/controls.json`, Q-list). Tier of P7: **Medium**; P7 touches no contract. This follows
`2026-10-07-p6-state-and-the-look.md`, which asked whether the owner's "It should at least resemble something like https://roland50.studio" overturns D3 for the panel; the owner has since said "Continue" and has not answered it.

## 1. R3, the look: needs an ADR, the Conductor's

P7 draws the panel at the Octopus's own geometry in the **current tokens** (surface #2e2e2e, ink #ececec, flat fills, hairlines, no gradient, no accent). It is recognisably the Octopus's layout and it is still not the instrument's *look*.
A throwaway preview (not committed; `shot-look` in the Forge's scratch, shown to the owner with the report) changes only colours and shows how far a palette alone gets, with no gradient, no shadow and no easing:

| Part | Measured on the photograph (numbers only, CLAUDE.md item 7) |
|---|---|
| Faceplate | #faf1ea to #fff7f1 across the plate |
| Frame (wood) | #38271e, and #4b3e38 in the lighter grain |
| Printed ink | #040104 |
| Keys, chrome | **not measured**; the preview uses flat greys, not a ramp (E1) |

What the ADR must settle (nothing in it is mine to loosen, CLAUDE.md items 3, 4 and 6):

1. **S8 conflicts with the faithful plate.** The slop lint flags any colour within ΔE 3 of #F4F1EA. The measured faceplate is at ΔE76 2 to 3.1 from it. Either the plate is a token the lint knows is measured (an allowance with an expiry date),
   or the plate is drawn a little off the measured value; I will not choose by changing the number. This is the finding; the owner's "faithful layout, restrained materials" (D3) is the tension.
2. **A frame colour needs a token** (`colour.frame`), a schema change in `contracts/design.tokens.json` (status `candidate`). A new colour is an introduced colour (item 6): it is a neutral, not an accent, and needs the ADR.
3. **Chrome is a mirror, not a ramp** (item 4). Flat fills and a hairline are what P7 draws. If the owner wants more depth than that, it is a decision about D3 and the no-gradient rule for the *panel only*, with an expiry date, and the new rule is
   written as a test first.
4. **Contrast.** Computed (WCAG): ink #040104 on #faf1ea is 18.6:1 and #4b3e38 on it 9.2:1; the preview's LED-off dot (#8a8480) is only 3.3:1 on it, so the off-LED needs its own measured value. Re-check against the a11y thresholds once chosen (tighten, never loosen).

Proposed token patch for the Conductor to take or change (I do not apply it): `colour.plate` #faf1ea, `colour.frame` #38271e, `colour.ink` #040104, `colour.ink-quiet` #4b3e38, with the page ground `colour.frame` and the strip in the same ink on that ground.

## 2. For the Panelwright: what the photograph says that `controls.json` does not

1. **Q04, the EDIT column's order:** top to bottom, VEL PIT LEN STA POS DIR AMT GRV MCC MCH (photograph; the manual's own text does not give it). The column **bows** by about 1.5 pitches.
2. **Q22, a row of eleven MIX TARGET keys is under the matrix** (ATR VOL PAN MOD EXP MAP 1 to 5) and is in no inventory. P7 draws them as inert plate keys (`hw.target.*`).
3. **Q23, the MODE block has MAP**, and the EDIT key sits under the EDIT knobs. `mode.edit` is placed by that position (`basis: position`).
4. **Q26, the transport keys have LEDs**: the photograph shows a dot beside Play, Stop and Pause. P7 draws them as dead dots (the engine does not light them). If they are never lit, the dots are drawn for nothing; if they are, a transport LED row is a new slice.
5. **Q33, the note ring: 13 keys against 14, and digits against Roman numerals.** The photograph's number keys read as digits 1 to 10; P7 draws digits. The 13 outer-ring keys with no legible name are inert and unnamed.
6. **The mutator column has ten keys and a MUT key under it; the inventory has twelve mutators.** P7 places them top to bottom as tgl sol clr rnd flt zom rmx cpy pst eff (`basis: order`; only some of the printed names are legible) and `mutator.mute` as the key at the foot of the column; `mutator.aln` has no place (unplaced, with a reason).
7. **Five controls have no place on the plate** and are not drawn: `circle.low_c`, `circle.chr`, `circle.follow`, `mutator.aln`, `circle.numeric`. Their reasons are in `apps/web/layout/panel.layout.json`. The list may shrink and not grow.
8. **18 identities are a stated order within a group, not read from the plate** (`basis: order`): the ten mutators, the seven chords b1 to b7 top to bottom, and `circle.clock200`. Please confirm or correct each; a correction is one line in the layout file and a test that names it.
9. **The matrix row pitch is 1.495 column pitches.** `controls.json`'s zone note should say so; the matrix is not square.
10. **Marks on the plate with no inventory entry:** five small black-key marks above the matrix, and the ten number keys around the ring. P7 draws the number keys as inert keys and does not draw the marks (D-P7-7).

## 3. `journal/STATE.md` (at its 200-line cap, so a request and not an edit)

Proposed paragraph for the SPEC-0002 section when P7 merges:

> **P7 (`WENGE-0019`): the panel is laid out like the Octopus.** 242 of the 247 controls are drawn at the place the owner's photograph measures (matrix 1.5 times taller than wide, the knob columns, the mutator column, the circle and its blocks), each a button by the manual's own name; 37 plate keys that are in no inventory are drawn and do nothing; 21 encoders turn and the turn reaches the engine, which ignores it. LEDs are dots beside the keys. The layout file is `provisional` (18 identities by order). Node 377, Chromium 79, 542 assertions. **Not done:** the look (R3, an ADR), knob behaviour, the display, 5 controls with no place.

## Why a request

`contracts/` and `journal/STATE.md` are the Conductor's; `controls.json` is the Panelwright's.

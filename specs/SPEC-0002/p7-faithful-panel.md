# SPEC-0002 P7: the panel is laid out like the Octopus

| Field | Value |
|---|---|
| Task | `WENGE-0019` (the owner's review of 2026-10-07: "this was the original spec to vuild this" with the genoQs Octopus article, and "It should at least resemble something like this: https://roland50.studio"; "Continue", 2026-10-11) |
| Spec | SPEC-0002 r1, D3 ("a faithful layout"), `docs/00-north-star.md` section 1 ("the layout: the matrix, the encoder columns, the selector and mutator columns, the circle, the chord block, the transport and the spiral stay where the hardware has them") |
| Plan revision | r0, 2026-10-11 |
| Tier | **Medium**, Forge: `apps/web/` and `handoffs/evidence/`. `contracts/` is not touched (the look is R3, a request to the Conductor); the thresholds and the golden files are not touched |

## 1. What the owner saw, and what is true of `main` (d1bbd78)

The page draws 160 matrix keys in a square grid and eleven other controls in rows below, as plain labelled circles. The Octopus has a matrix that is 1.5 times taller than it is wide, a column of ten big knobs
down its left, a SEL column, a mutator column and a bowed column of ten EDIT knobs on its right, a row of keys under it, and, to the right, a spiral: number keys, a note ring, the MODE block, scale keys, transport, a chord arm, and
one big knob. 247 controls are in `contracts/controls.json`; the page draws 171 of them.

`reference/plates/web-frontal-01.jpg` (the photograph on the owner's article) is the only source that says *where* anything is: the manual does not (`controls.json`, zone notes, Q04, Q22, Q23, Q33), and `journal/STATE.md` says the layout would
not come from photographs. The owner has now named the photograph's page as the original spec, so the geometry comes from it, as **numbers** (CLAUDE.md item 7): the photograph is measured by a script, and only the measurements are
committed. Nothing reads a photograph at run time, and no photograph is in any context a test reads.

## 2. What was measured (`handoffs/evidence/panel-layout/`)

- `extract.py` finds every key and knob by template matching and fits the 160 matrix keys to a lattice: **265 keys, 21 knobs**, the matrix fits with a worst residual of **0.84 px** (mean 0.42) on a 16.4 px pitch.
  The matrix rows are **1.4953** column pitches apart. The photograph's roll is 0.2 degrees, removed by the fit, so the panel is drawn level.
- Each key's LED is found beside it: **above** each matrix key, **right** of each SEL key, **left** of each mutator key, mostly above in the circle.
- `measured.json` is the output (rectified, in matrix column pitches). `assign.py` says what each measured thing is, and writes `apps/web/layout/panel.layout.json` (`panel.layout/2`).
- Not measured: the printed names (about 4 px high in the photograph, unreadable except a few: MODE, SCALE, TRANSPORT, CHORD, PAGE, STEP, PLAY, GRID, TRACK, SEL, MOD, CAD, ESC, Tempo, Program, Select, the EDIT names, the MIX TARGET row, and the transport symbols).

## 3. What changes, with the observable for each (tests in `apps/web/test/layout.test.ts` and `apps/web/test/browser/panel.browser.test.ts`, written first)

| # | Change | Observable |
|---|---|---|
| P7-1 | Every control in the inventory is drawn once or named as not drawn, with a reason: 242 drawn, 5 not (`circle.low_c`, `circle.chr`, `circle.follow`, `mutator.aln`, `circle.numeric`); the list may shrink and not grow | `layout.test.ts` |
| P7-2 | Each position is within 0.1 pitch of one measured point, and no point serves two keys | `layout.test.ts` (reads `measured.json`) |
| P7-3 | The blocks are where the hardware has them: MIX knobs, SEL column, matrix, mutator column, EDIT knobs, circle, in that order from the left; the row of keys under the matrix in order; the note ring chromatic and clockwise from the bottom-left C; scale keys under it; MODE inside it; transport round its bottom; chord arm to its right, top to bottom; main knob top right; the EDIT column bows by about 1.5 pitches | `layout.test.ts` |
| P7-4 | A key is the key token, a knob the knob token, an LED dot the dot token, a pitch the key plus the gap. No pixel in the layout or the code | `layout.test.ts`, `app_source_rules.rs` (S12, unchanged) |
| P7-5 | Nothing overlaps; every printed label clears every key, dot, knob, legend and label | `layout.test.ts` |
| P7-6 | The LED is a dot beside the key (as on the hardware), and not the key itself | `panel.browser.test.ts`, `app.browser.test.ts` (E1, selectors updated) |
| P7-7 | The 21 encoders are drawn as hex-nut knobs, and turn: wheel and arrow keys send `octoweb_input` kind 2 (the engine ignores a turn until a knob workflow exists, `ABI.md`) | `panel.browser.test.ts` |
| P7-8 | The plate's printed legends: block names, MIX TARGET, row numbers 0 to 9, both clefs | `layout.test.ts` |
| P7-9 | Reading order (the Tab order) is the plate's: bands of one matrix row from the top, left to right in each | `layout.test.ts`, `app.browser.test.ts` E8 |

## 4. Decisions taken by default (each is the owner's to overturn)

- **D-P7-1: geometry from the photograph, identity from the photograph where it can be read, and from a group's order where it cannot, flagged.** 63 controls are `photo`, 18 are `order` (the ten mutators, the seven chords, `circle.clock200`), one is `position` (`mode.edit` is the key under the EDIT knobs; the MODE block's sixth key reads MAP, Q23).
- **D-P7-2: the plate's keys that are in no inventory are drawn and do nothing** (37: ten number keys, thirteen unnamed outer-ring keys, the eleven MIX TARGET keys, SEL, MAP, a left transport key). They are not buttons and not tab stops. Pressing nothing is honest; leaving them out would not look like the instrument.
- **D-P7-3: the five controls with no place on the plate are not drawn.** Their reasons are in the layout file and are findings for the Panelwright.
- **D-P7-4: sizes from the tokens as they are**: key `space.4`, gap `space.3` (a pitch of 32), dot `space.2`, knob `space.5`. No token changes. The look (R3) is a request to the Conductor.
- **D-P7-5: encoders have no visible value.** The engine has no knob state yet, so a pointer that moved would be a lie. A knob turns, and the turn goes to the engine, which ignores it for now.
- **D-P7-6: no brand marks.** The plate's "genoQs Machines" and "octopus" are not drawn.
- **D-P7-7: the black-key marks above the matrix are not drawn** (five small boxes; they need a meaning before they need a place).

## 5. Not in this slice

The look (cream plate, wenge frame, chrome as mirror): an ADR, the Conductor's; a throwaway preview of it goes with the report and is not committed. Behaviour for any new control. The numeric field (no display on the plate). Labels
for the thirteen unnamed ring keys. A measurement of the photograph's perspective (the plate is level to 0.2 degrees; its edges are not square to within a pitch).

## 6. Risks

- 18 identities are a guess in a stated order; if the hardware disagrees, a key is labelled wrongly until a person with an Octopus says so. They are flagged in the file and in `journal/forge/requests/`.
- The clefs are text glyphs (U+1D11E, U+1D122); a machine with no font for them shows a box. Chromium here has one; macOS and Windows ship one.
- 247 controls at a pitch of 32 make a plate about 1,600 wide; it scales as one object, so labels are 12 px at 1:1 and smaller on a narrower window.

# SPEC-0002 P9: the strip and the page are a midnight ground with hairlines

| Field | Value |
|---|---|
| Task | `WENGE-0024` (the owner, Tue 2026-10-13 10:58: "Let's give the top section and background a … feel", with the Linear "midnight precision instrument" style reference; 11:15: "Also add in some design embellishment from Eleven Labs", with a screenshot of three line-drawing cards; 12:52, with a screenshot of the page: "Let's remove the top shapes above they offer nothing", which took the drawings out again) |
| Spec | SPEC-0002 r1 D3 as amended by `adr/0010-...` (merged) and `adr/0011-the-strip-and-the-page-are-a-midnight-ground-with-hairlines.md` (proposed in #56) |
| Plan revision | r0, 2026-10-13 |
| Tier | **Medium**, Forge: `apps/web/`, `specs/` and `handoffs/evidence/`. `contracts/` is not touched (PR A did that); no threshold, golden file or engine code is touched |
| Stacked on | PR A (#56, the ADR and the tokens). This pull request carries its commits and merges after it |

## 1. What the owner saw, and what is true after P8b

The strip was a grey bar of white-outlined square buttons in Helvetica, and below it the frame's dark brown filled the window. PR A put eighteen colour roles, one corner, a vendored face and two weights in the tokens contract. Nothing drew them. This does.

## 2. What changes, with the observable for each (tests written first, 11 of 14 red on `7a801b5`)

| # | Change | Observable |
|---|---|---|
| P9-1 | The page is `surface` (the root too, so no other colour shows past it); the strip has no ground of its own and a hairline of `edge` under it | P9-1 |
| P9-2 | A control of the strip is a hairline border of `edge.control` on the ground with `radius.control`; Play is the one control filled at rest (`ink`, text `surface`); a toggle that is on is `surface.raised` with an `ink` border, one that is off has no fill | P9-2, C8 |
| P9-3 | A control's border is at least 3:1 on the ground; every text of the strip is at least 7:1 on what it sits on, measured on the colours the browser computed | P9-3 |
| P9-4 | The face is loaded from the file the tokens name (`document.fonts.check`, one `@font-face`, status `loaded`), with its features on, the body at `size.body`, and every weight in the strip one of the two | P9-4 |
| P9-5 | Nothing in the strip or the page has a shadow, gradient, transition, tracking, capitals or a filter | P9-5 |
| P9-6 | Keyboard focus on a strip control is the `focus` ring at `stroke.focus`, 3:1 on the ground | P9-6 |
| P9-7 | At five window sizes (1280 by 900, 1920 by 1080, 1024 by 768, 600 by 800, 1600 by 480) the case is the SVG, its frame rect fills it and the plate is inside it with a margin; the element that holds it has no colour; the case is under the strip and inside the stage; the plate's aspect is kept; the pixels at the case's corner, the plate's corner and the stage's corner are `frame`, `plate`, `surface`; the page does not scroll | P9-7 (x5), P8-1, P8-3, P8-7 |
| P9-8 | There is nothing in the ground: no `#ground`, the only SVG on the page is the panel's, the stage holds the case and nothing else, and no stroke is outside the panel (the owner, 12:52: "remove the top shapes above they offer nothing"; ADR-0011 decision 10, withdrawn). Rust: `the_ground_has_no_drawings` reads every source file and fails on a module, a call, an element or a rule that draws in it | P9-8, `app_source_rules.rs` (two tests) |
| P9-9 | The case sits in the middle of the room under the strip: at five window sizes the gap above it equals the gap below it, to a pixel | P9-9 |
| P9-10 | Retired with the drawings (it said they were still). The number is not reused | none |
| P9-11 | The up and down block of a number field is the page's own: the `ink` with the `surface` for its arrows, the owner's 8 px on all four corners (the same as `radius.control`, and rendered: the four corner pixels are the field's ground and the middle of each edge is the block), the browser's own spin button gone (no pixel left of the block but the ground, with the field hovered), the number never under the block, hidden from the accessibility tree, nothing in it focusable | P9-11 |
| P9-12 | A press steps the number as the browser's did: one press is one step with an `input` and a `change`, held it repeats and stops on release, at a limit nothing happens and no event is made up, a press leaves the field focused, and the arrow keys still step | P9-12 |
| R-1 | The strip's tones never appear in a `#panel` rule, and the panel's never in the strip's; `src/panel.ts`, `pages/app.ts` and `src/strip.ts` write no number beyond 0, 1 and 2 (S12) | `app_source_rules.rs` |

## 3. Decisions the owner can change, with the default taken

| # | Decision | Default and why |
|---|---|---|
| D-P9-1 | The case hugs the plate (the frame is the first rect of the panel's SVG) and the page shows round it | Yes. It overturns D-P8-1 (the frame fills the room): the owner's "background" is the page, and a case that fills the window hides it |
| D-P9-2 | (Withdrawn, 12:52.) The ground's drawings were in `edge.control`, the dots too | The drawings are gone, and with them the three tests of their tone and the S12 entry for `src/ground.ts`; the two that were in the baseline left it through `cargo xtask baseline --remove ... --adr ADR-0011` |
| D-P9-3 | The case sits in the middle of the room, not at the top of it | With nothing above it, the top of the room would leave all the empty ground below. One line (`justify-content` on `#stage`) puts it at the top if the owner would rather |
| D-P9-4 | (Withdrawn, 12:52.) Ellipses were drawn as two arcs | The S15 finding (the rule reads `rx` and `ry` and also catches `<ellipse>`) stands in the journal; no ellipse is drawn now, so nothing here works round it |
| D-P9-5 | The status lines are at the label size (12 px), in `ink.quiet` (7.34:1) | Compact, and over the floor |
| D-P9-6 | Fields are a name over a control, the name quiet | Matches the reference's compact look; the tab order and the ids are unchanged |
| D-P9-7 | The corner is 8 px for every control and for the stepper's block (the owner, Tue 12:00: "round the corners to an 8px for the beige background of it"); the contract moved from 6 to 8 in PR A | One corner is one corner (ADR-0011 decision 6). The owner named the block; applying 8 to it alone would give a block rounder than the box that holds it |
| D-P9-8 | The block is drawn by the page, in the `ink` (light, as the owner saw it) | The browser's takes no corner and chooses its own tone (dark in headless Chromium, light in the owner's view). If the owner wants it dark, `surface.raised` for the block is one line, and Play is then the one fill again |
| D-P9-9 | Hold to repeat: 400 ms, then every 70 ms | The browser's repeats; the numbers are the page's own, close to what it did. No other change to how a number field behaves |

## 4. What is not done

- **A different look for the stepper** than the owner saw: only its corners changed (and the browser's own tone became the `ink`).
- **Acid lime and every other accent**, cards, pills, pastel orbs, the soft shadow, tracking, a serif display face (ADR-0011 decision 2).
- **Other pages**: there are none.
- **roland50.studio**: never read.

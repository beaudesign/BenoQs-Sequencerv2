# SPEC-0002 P9: the strip and the page are a midnight ground with hairlines

| Field | Value |
|---|---|
| Task | `WENGE-0024` (the owner, Tue 2026-10-13 10:58: "Let's give the top section and background a … feel", with the Linear "midnight precision instrument" style reference; 11:15: "Also add in some design embellishment from Eleven Labs", with a screenshot of three line-drawing cards) |
| Spec | SPEC-0002 r1 D3 as amended by `adr/0010-...` (merged) and `adr/0011-the-strip-and-the-page-are-a-midnight-ground-with-hairlines.md` (proposed in #56) |
| Plan revision | r0, 2026-10-13 |
| Tier | **Medium**, Forge: `apps/web/` and `handoffs/evidence/`. `contracts/` is not touched (PR A did that); no threshold, golden file or engine code is touched |
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
| P9-8 | The ground is `aria-hidden`, not focusable, takes no pointer, has three drawings, and every stroke is a hairline of `edge.control` with no fill and `non-scaling-stroke`; the dots are a zero-length dash every `space.1` with a round cap | P9-8 |
| P9-9 | The ground is under the strip and above the case, never behind either, every stroke inside it; 60 px or more at 1280 by 900 and 1920 by 1080, none at 1600 by 480 | P9-9 |
| P9-10 | The ground is the same picture 400 ms later and has no animation | P9-10 |
| G-1 to G-5 | The drawing is a function of the space scale (doubling the scale doubles every length; the same tokens give the same bytes), every coordinate is a multiple of half the smallest step, strokes only, inside the view box | `test/ground.test.ts` |
| R-1 | `#ground` rules use only `--colour-edge-control`, no fill and no tint; the strip's tones never appear in a `#panel` rule; `src/ground.ts` writes no number beyond 0, 1 and 2 (S12) | `app_source_rules.rs` (three tests) |

## 3. Decisions the owner can change, with the default taken

| # | Decision | Default and why |
|---|---|---|
| D-P9-1 | The case hugs the plate (the frame is the first rect of the panel's SVG) and the page shows round it | Yes. It overturns D-P8-1 (the frame fills the room): the owner's "background" is the page, and a case that fills the window hides it |
| D-P9-2 | Ground drawings are in `edge.control` for the dots too | The hairline tone `edge` is 1.28:1 on the ground; dots in it were not visible in a rendered check. ADR-0011 decision 10 says so |
| D-P9-3 | The drawings are above the case and shrink to the room; none when there is none | The alternative (behind the case, or in the corners) would be cut off by the case at some window sizes |
| D-P9-4 | Ellipses are drawn as two arcs | S15 reads the attribute names `rx` and `ry` for a rounded rectangle and also catches `<ellipse>`. An ellipse is not a rounded rectangle; the rule is a finding (see the journal), not changed here |
| D-P9-5 | The status lines are at the label size (12 px), in `ink.quiet` (7.34:1) | Compact, and over the floor |
| D-P9-6 | Fields are a name over a control, the name quiet | Matches the reference's compact look; the tab order and the ids are unchanged |

## 4. What is not done

- **Acid lime and every other accent**, cards, pills, pastel orbs, the soft shadow, tracking, a serif display face (ADR-0011 decision 2).
- **Other pages**: there are none.
- **roland50.studio**: never read.

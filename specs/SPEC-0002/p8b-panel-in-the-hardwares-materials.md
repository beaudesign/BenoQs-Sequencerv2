# SPEC-0002 P8b: the panel is drawn in the hardware's own materials, flat

| Field | Value |
|---|---|
| Task | `WENGE-0022` (the owner, Wed 2026-10-07: "It should at least resemble something like this: https://roland50.studio"; Mon 2026-10-12 21:59: "Yes they are correct and the panel should look like the instrument") |
| Spec | SPEC-0002 r1 D3 as amended by `adr/0010-the-panel-is-drawn-in-the-hardwares-materials-flat.md` (proposed in #52); the six roles are P8a (#53) |
| Plan revision | r0, 2026-10-12 |
| Tier | **Medium**, Forge: `apps/web/` and `handoffs/evidence/`. `contracts/` is not touched (P8a did that); no threshold, golden file or engine code is touched |
| Stacked on | P8a (#53), which is stacked on ADR-0010 (#52). This pull request carries their commits and merges after both |

## 1. What the owner saw, and what is true after P7

P7 gave the panel the Octopus's layout and none of its look: light hairline circles on a dark grey ground. ADR-0010 lets the panel copy three things of the hardware's material, flat: the faceplate's colour, the frame's, and the
keys' metal, with the printed names in a dark ink. P8a put six colour roles in the tokens contract (`plate`, `frame`, `plate.ink`, `plate.ink.quiet`, `key`, `key.edge`). Nothing drew them. This does.

## 2. What changes, with the observable for each (tests written first, red on `48818d0`)

| # | Change | Observable |
|---|---|---|
| P8-1 | The plate is `plate` with no outline; the frame is `frame` with a margin of `space.4` round the plate; a key, a nut and a ball are `key` with a `key.edge` hairline; the strip and the page keep `surface` and `ink` | `panel-colours.browser.test.ts` P8-1, on computed styles |
| P8-2 | Every printed name (105: 21 legends and 84 key labels) is `plate.ink`, or `plate.ink.quiet` for captions, row numbers and clefs, and is at least 7:1 on the plate, measured on the colours the browser computed | P8-2 |
| P8-3 | Rendered, a pixel of the plate, of the frame, of the middle of a key and of the strip is the token's colour exactly | P8-3 (a screenshot read back through the browser's own decoder) |
| P8-4 | A lit LED has a rim of `plate.ink`, a hairline, whatever its colour (a lit green is 1.6:1 on the plate and a lit orange 2.2:1, so colour does not mark it); shining has the heavier rim (`stroke.focus`); flashing keeps the rim while the fill flashes; a dark LED and a dead dot have the rim too | P8-4 |
| P8-5 | A held key's edge goes to `plate.ink`: the face does not change (a key of the hardware does not), so the edge says it | P8-5 |
| P8-6 | The focus ring shows on the plate and on the key: the outer stroke is the dark tone (at least 3:1 on the plate) and the inner the light one; one stroke is at least 3:1 on the key. This is ADR-0010 decision 5: white on the plate is 1.1:1 | P8-6; `app.browser.test.ts` E8 now asserts the swapped tones |
| P8-7 | The whole plate is still on screen, inside its frame, at 1280 by 900 and 900 by 700 | P8-7, P7-9 and P7-10 unchanged |
| S17 | The six colours are used only by rules that start at `#panel` or `#panel-frame`, and by no script or page; the panel's parts (`.plate`, `.face`, `.led` and the rest) are styled only under `#panel`; each of the six is used by a panel rule; the panel borrows none of the strip's roles (`surface`, `ink`, `ink.quiet`, `flash.shine`) | `app_source_rules.rs`, seven new tests, each rule shown to fire on a snippet made to break it |

The 7 new Rust test names are in `harness/baseline.txt` (550 to 557). Twelve mutations of the stylesheet (the plate in the strip's ground, a part unscoped, the ring unswapped, no LED rim, a panel colour in the strip, a thin shine rim, captions in the key tone,
no margin, a pressed key unchanged, an outlined plate, a panel rule in the strip's ink, the frame in the key tone) were each caught by the test named for the rule.

## 3. Decisions taken by default (each is the owner's to overturn)

- **D-P8-1: the frame is everything under the strip.** One flat tone fills the room the panel is given; the plate sits in it with a margin of `space.4`, centred. In a wide window there is a lot of frame above and below the plate (the plate is as wide as the window allows).
  The alternative, a frame that hugs the plate, needs the strip to sit elsewhere, and the page behind the strip is still the strip's dark on purpose (ADR-0010 "Not decided": a second ground for the strip).
- **D-P8-2: the plate has no outline.** The frame is its edge.
- **D-P8-3: a knob is the key tone.** The nut and the ball are `key`, told apart by their edges. The photograph's nut and ball differ in lightness (L\* 67 and 51); a second metal tone is a seventh role, which ADR-0010 refuses.
- **D-P8-4: the shine rim is heavier, in the ink.** `flash.shine` names `ink`, which on the plate is the strip's light tone and invisible (1.1:1). The panel does not use it; a request to the Conductor says so (`journal/forge/requests/2026-10-12-flash-shine-on-the-plate.md`).
- **D-P8-5: the colours are the candidates of ADR-0010.** The owner ratifies them by merging this pull request with the screenshots in `handoffs/evidence/p8b/` (before: `p7-panel-running.png`; after: `p8b-panel-idle.png`, `p8b-panel-running.png`, `p8b-panel-detail.png`).
- **D-P8-6: `npm run screenshot` takes an optional prefix** and its stale selector (`#midi-output`, which the strip lost when it grew two outputs) is fixed; it had not run since.

## 4. Not in this slice

The typeface (the Curator's). A frame that hugs the plate, or a case-coloured page. A second metal tone for knobs. Reading roland50.studio (never opened: the fetch permission was left unanswered). Photograph colour calibration (the photograph has no colour chart: a better one moves the
values and not the rules). Enforcing the 2027-01-31 expiry of the two tinted roles (a request to the Referee, in the P8a pull request).

## 5. Risks

- A flat grey key does not read as chrome. ADR-0010 says so up front: whether it is enough is for the owner's eye on the screenshots; more depth is a later decision about D3 and photoreal drawing.
- The values are read from an uncalibrated retailer photograph. If the cream looks pink on the owner's screen, that is the value, not the rule, and moves with P8a's tokens.

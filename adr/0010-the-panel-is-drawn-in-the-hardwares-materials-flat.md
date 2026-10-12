# ADR 0010: The panel is drawn in the hardware's own materials, flat

## Status

Proposed on 2026-10-12 by the Conductor. It takes effect when the owner merges the pull request that carries it, as ADR-0008 did. If the owner closes it, the panel stays as P7 left it (`6fa1338`).

It answers the owner's own words. On Wed 2026-10-07, 18:03 London: "this was the original spec to vuild this" (the genoQs Octopus article); at 18:04: "It should at least resemble something like this: https://roland50.studio". The owner has since merged P7 (#51), which drew the layout, and said "Continue" on 2026-10-11 and 2026-10-12. The Forge's request of 2026-10-07 asked whether those words overturn D3 for the panel and got no answer in words; this is the proposal for the owner to answer by merging or closing.

**The owner's answer.** Mon 2026-10-12, 21:59 London, in chat, after reading the report on this pull request: "Yes they are correct and the panel should look like the instrument". That is the owner's word on the direction: the panel is to look like the instrument, so D3 is to be amended as proposed. It does not by itself take this ADR into effect (merging does, as with ADR-0008), and it does not choose the values, the expiry date or the role names, which stay as written below unless the owner changes them when merging. The follow-on pull requests, P8a and P8b, were started on the strength of it and carry this ADR's commits until it merges.

It amends part of SPEC-0002 D3 ("a faithful layout, restrained materials, LEDs as the only colour"), the second sentence of `SPEC.md` N2, `docs/00-north-star.md` section 1, `docs/05-design-system.md` sections 1, 2.1 and 4, and ADR-0008 decision 6 (the tokens contract, rule 7 and the list of nine roles). It changes no contract file: that is P8a, in a pull request of its own.

**What this ADR could not read.** https://roland50.studio was never opened: the fetch permission was left unanswered in this session, and it was not worked around. "Resemble" is read from the photograph the owner called the original spec (`reference/plates/web-frontal-01.jpg`). If the owner means something the photograph does not say, the screenshot in P8b is where it is caught, before any value is ratified.

## Context

**What P7 left.** The page has the Octopus's layout (242 of 247 controls at measured places) and none of its look: light hairline circles on a dark grey ground, the same in every part. The owner's complaint was that it does not resemble the instrument. The layout was the half of that which the rules allowed.

**What the rules say today, and what each would stop.**

| Rule | Says | Stops |
|---|---|---|
| SPEC-0002 D3 | "restrained materials, LEDs as the only colour" | a faceplate or a frame in any colour |
| `SPEC.md` N2 | The only *saturated* colour is an LED's. "Everything else is one neutral palette" | a tinted faceplate: the first sentence allows it, the second does not |
| `docs/00-north-star.md` s.1 | "The web panel does not copy the wood, the coat or the chrome" | a frame, a plate tone, a metal tone |
| ADR-0008 decision 6, rule 7 | "The only keys with a hue are the three `led.*` roles"; nine roles and no tenth | any tinted token, and any new role |
| `tokens.rs` T7 and the schema | the same, as a test: `ink`, `surface` and the rest must have equal red, green and blue | the same |

**What the photograph says**, as numbers (`handoffs/evidence/panel-colours/measure.py`, which reads colour at the places `panel-layout/measured.json` says things are). It is a retailer's product shot with no colour chart and an unknown white balance (`reference/plates/NOTES.md`), so these are what the photograph shows and not what the instrument is:

| Part | Median | Lightness L\* | Chroma C\*ab | Note |
|---|---|---|---|---|
| Faceplate | `#fcf3ec` | 96.5 | 4.5 | 237,549 pixels clear of every key; p10 `#f1e7e2`, p90 `#ffffff` (the lighting) |
| Frame (the wenge cheek) | `#32241e` | 15.7 | 8.5 | 13,494 pixels; p10 `#261b16`, p90 `#483426` |
| Key face (the chrome) | `#a0958c` | 62.4 | 6.3 | a mirror: p10 `#25191d`, p90 `#fefaf7`, so no one flat value is "the" colour |
| Knob nut, knob ball | `#aaa297`, `#827772` | 67.1, 51.0 | 7.1, 5.0 | |
| Printed ink | darkest 1 percent `#1e0e0a` | 5.5 | 8.9 | names are about 4 px high, so this is the ink blurred into the plate; the ink is darker |

Every tinted part has a chroma of 9 or less. None of it is saturated: the one saturated thing in the photograph is an LED.

**S8 does not forbid the faceplate.** The faceplate's median is ΔE2000 3.01 from `#F4F1EA`, on the line S8 draws. `docs/05` section 4 says what S8 is for and what to do about it: the point of the rule "is not that the colour is forbidden; it is that arriving there deliberately must be distinguishable from arriving there by default", and the response is a suppression that states the reason. `tokens.rs` T4 already implements that exit (`"suppress": [{"rule": "S8", "why": …}]`). No threshold moves. The Forge's request of 2026-10-11 called this a conflict; it is not one.

**Why the review in `docs/00` section 3.5 does not apply.** It rejects "warm cream background with high-contrast serif and terracotta accent". The reasons are the serif, the high contrast and the accent: the generated-design cluster. The Octopus's faceplate is a warm off-white (`docs/00` section 1 says so in its own first paragraph). The panel keeps a sans face, no serif and no accent.

## Decision

### 1. The panel copies three things of the hardware's material, flat

The colour of the **faceplate**, the colour of the **frame** round it, and the **metal of the keys** as one tone each, with the printed names in a dark **ink**. It still does not copy grain, the coat's texture, gloss, a highlight, a reflection, a bevel or a shadow. So S1 (no gradient), S4 (no shadow), S9, S15 and S16 stand as written, and "chrome is a mirror, not a ramp" (`CLAUDE.md` rule 4) stands: a flat grey key is not a mirror and does not pretend to be one. Whether a flat key reads as metal is for the owner's eye on the P8b screenshot; the way to more depth than that is a later, separate decision about D3 and photoreal drawing (ADR-0006 left that "a later option, not a scope").

### 2. `SPEC.md` N2 is reworded; its first sentence is unchanged

> **N2. The only saturated colour is an LED's.** Red, green and orange, steady or flashing, and only as roles: the colourways in the manual change the hue, not the role. Everything else is flat and low in chroma: one neutral palette, and the two materials of the hardware, the faceplate and the frame (ADR-0010). No gradient stands in for a material, no drop shadow fakes depth, no accent colour.

`CLAUDE.md` rule 6 ("There is no accent colour in this product. The only saturated colour is emitted by an LED") is true as written and is not touched.

### 3. Six colour roles are added to the nine, and no seventh

| Role | What it is | Rule (a test in P8a, red first) |
|---|---|---|
| `plate` | the faceplate | tinted; chroma C\*ab at most 12; carries the S8 suppression with its reason |
| `frame` | the case round the plate | tinted; chroma at most 12 |
| `plate.ink` | printed names, on the plate | pure grey (equal red, green and blue); at least 7:1 against `plate` |
| `plate.ink.quiet` | captions, row numbers, clefs | pure grey; at least 7:1 against `plate` |
| `key` | the flat tone of a key or a knob | pure grey |
| `key.edge` | the key's outline | pure grey; at least 3:1 against `plate` (WCAG 2.1 non-text) |

The two tinted roles, `plate` and `frame`, are the only ones that may have a hue other than a lit LED's. Everything new that can be pure grey is pure grey, so the exemption is as small as the look allows. The nine existing roles are unchanged, and so is the strip: it keeps `surface`, `ink` and `ink.quiet`, and the rule in `docs/05` section 1 stands ("the panel's look may never be quoted in the strip").

Candidate values, for the Curator to choose from and the owner to ratify in P8b (they are not decided here): `plate` `#fcf3ec`, `frame` `#32241e`, `plate.ink` `#1e1e1e` (15.2:1 on the plate; ΔE2000 3.8 from `#111111`, so no S8 waiver), `plate.ink.quiet` `#4d4d4d` (7.7:1), `key` `#9a9a9a`, `key.edge` `#5c5c5c` (6.1:1). A rendering of them over P7's panel is in the pull request.

### 4. The six roles are used only inside the panel

A new rule, **S17**: `plate`, `frame`, `plate.ink`, `plate.ink.quiet`, `key` and `key.edge` appear only in the stylesheet rules of `#panel` and `#panel-frame`. Rules are added and never removed (`docs/05` section 4); this one keeps the strip plain.

### 5. Two rules from ADR-0008 are amended; one expires

- Rule 7 becomes: "The only keys with a hue are the three `led.*` roles, `plate` and `frame`; the last two have a chroma of at most 12; every other role is pure grey." **This loosens a rule, so it has an expiry date: 2027-01-31.** On that date the exemption lapses and `plate` and `frame` must be pure grey again unless the owner has renewed it by merging an amendment to this ADR. The date is a default; the owner may set another. How the lapse is enforced is the Referee's: a test that reads the clock is not deterministic, but the date of the checked-out commit is, and `determinism` stays intact.
- The "nine roles and no tenth" test becomes fifteen, named.
- Rule 5 (the focus ring visible wherever it falls) gains the plate: at least one of `focus` and `focus.inner` reaches 3:1 against `plate` as well. This is satisfied today by `focus.inner` (15:1); the outer stroke is white and would vanish on the plate (1.1:1), so P8b swaps the two strokes on the plate, and a Chromium test measures the computed colours.

### 6. The series

| PR | What | Tier | Why |
|---|---|---|---|
| P8a | `contracts/design.tokens.json` and its schema gain the six roles; `tokens.rs` gains the amended rule 7, the new rules and S17, each shown to fail on a bad document; status back to `candidate` | High | A contract file |
| P8b | The panel drawn in the new roles; a lit LED gets a rim of `plate.ink` (a lit green is 1.6:1 against the plate, a lit orange 2.2:1, so colour alone would not mark it); the focus ring on the plate; screenshots for the owner | Medium | `apps/web` only |

P8b carries P8a's commits if P8a has not merged when it is opened. The agent merges neither. The owner ratifies the values by merging P8b, as ADR-0008 asked for P3c.

## Alternatives considered

- **Leave it as P7 left it.** Honest to D3 and to the north star; does not answer the owner's message. The cost is the owner's trust, and the owner has said so.
- **A cream plate and nothing else.** One tinted role, no frame, no key tone. Smaller, and it would look like a dark-ink spreadsheet on cream: the keys would still be dark circles and the plate would have no edge.
- **A photoreal panel (WebGL, a reflection model).** Ruled out by ADR-0006 and D3. It is where "chrome is a mirror" would be answered properly, and it stays a later option.
- **The photograph as the background.** It is a retailer's, watermarked, and `docs/00` and `CLAUDE.md` rule 7 keep photography out of the product and out of context.
- **Pure grey everywhere, with a lighter grey plate.** Keeps T7 untouched, no expiry. It would look like the plate in the photograph with the warmth removed, and it is what the product has been doing. The tinted roles are what the owner is asking for; this is the version without them, and the owner may prefer it.

## Not decided here

- **The typeface.** Still the Curator's, by ADR, from three specimens beside the panel (`docs/00` section 3.2). roland50.studio was not read, so no claim is made about type.
- **The values.** Candidates only; P8b is where the owner sees them on the panel.
- **Colour accuracy.** The photograph is uncalibrated. A better one moves the values and not the rules, so it needs no new ADR.
- **How the expiry is enforced** (the Referee's, above).
- **A second ground for the strip.** The strip stays dark and plain. If the owner wants the page to look like the instrument's case too, that is the rule in `docs/05` section 1, and it is a bigger decision than this one.

## Consequences

- This pull request changes documents only: `SPEC.md` N2, `docs/00-north-star.md` sections 1, 3.1 and 3.5, `docs/05-design-system.md`, a note in `specs/SPEC-0002/README.md`, a row in `docs/10-risks-and-decisions.md`, and the colour evidence in `handoffs/evidence/panel-colours/`. No contract, threshold, test or code. Tier **High**, because it authorises a contract (ADR-0007).
- After it merges, nothing in the product changes until P8a and P8b. The panel stays dark and flat until then.
- It is reversible: closing the pull request leaves things as they are; letting the expiry lapse returns the two tinted roles to grey.
- The assertion set grows (the new token rules, S17, the focus and rim tests); none leaves.

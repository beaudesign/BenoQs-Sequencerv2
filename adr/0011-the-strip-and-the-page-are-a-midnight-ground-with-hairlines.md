# ADR 0011: The strip and the page are a midnight ground with hairlines

## Status

Proposed on 2026-10-13 by the Conductor. It takes effect when the owner merges the pull request that carries it, as ADR-0008 and ADR-0010 did. If the owner closes it, the strip and the page stay as P8b left them (`6c789e7`).

It answers the owner's own words. Tue 2026-10-13, 10:58 London, in chat, after a screenshot of the strip (a grey `#2e2e2e` bar of white-outlined square buttons and inputs, the Sound toggle on, and the frame's dark brown filling everything below): "Let's give the top section and background a" and then, pasted in full, a style reference headed "Linear — Style Reference > midnight precision instrument" (a near-black canvas, hairline borders, a compact scale, Inter Variable, a single acid-lime action colour), ending "feel". That is the owner's word on the direction: the top section (the strip) and the background (the page behind the panel) are to take that look. It does not by itself take this ADR into effect (merging does), and it does not choose the values, which are written below as defaults the owner can change when merging.

**The second message.** Tue 2026-10-13, 11:15 London, mid-work: "Also add in some design embellishment from Eleven Labs - https://elevenlabs.io - https://www.designmd.co/d/elevenlabs", with a screenshot of three cards of black line drawings on a warm off-white (a pair of cones and ellipses on dotted axes, a grid cube with dotted diagonals, tangent concentric circles with a dotted outer ring), each with a one-word title and a grey sentence under it. This ADR reads "embellishment" as that screenshot's drawing language, and decision 10 says what is taken from it.

**The third message.** Tue 2026-10-13, 12:00 London, with a screenshot of the Clock offset field (a hairline box holding `0`, and at its right a light block with an up and a down arrow, which the owner calls the beige background): "Let's round the corners to an 8px for the beige background of it". The light block is the browser's own number-field spin button. This ADR reads the word as the corner of the strip: 8 px, not the reference's 6, and it reaches the light block only because the page draws that block itself (decisions 6 and 7), the browser's own not taking a radius. The same message asks for a button that silences the drums; that is a feature (specs/SPEC-0002/p10-silence-the-drums.md) and not this ADR's.

It amends part of `docs/00-north-star.md` sections 3.1, 3.2 and 3.5 and E4, `docs/05-design-system.md` sections 1, 2.1, 2.3, 2.4 and 4 (rules S2, S11, S15, S17), SPEC-0002 D3 as ADR-0010 amended it, and ADR-0008 decision 6 (the tokens contract: eighteen roles, one radius, a face). `SPEC.md` N2 and `CLAUDE.md` rule 6 are **not** touched. It changes the contract in the same pull request as its tests, and the strip's stylesheet in a second one.

**What this ADR could not read.** https://roland50.studio was never opened: the fetch permission was left unanswered and was not worked around. https://www.designmd.co/d/elevenlabs could not be fetched (an error on two tries), and https://elevenlabs.io returned page text with no styles in it. The ElevenLabs values below are from a third-party transcription of the same kind of design file (https://www.shadcn.io/design/elevenlabs/raw), which is data and not the owner's word; the screenshot the owner attached is the primary source for "embellishment".

## Context

**What the strip and the page are now** (P8b). The page's ground is `surface` `#2e2e2e`, a mid-dark grey. The strip is a row of square, white-outlined buttons and inputs on it, set in Helvetica at 16 px. Below it, `#panel-frame` fills the rest of the window with the dark brown of the case, and the plate sits inside with a 20 px margin of it. The owner's screenshot is this.

**What the reference asks for**, as far as it bears on a sequencer's strip:

| The reference | Value |
|---|---|
| Canvas | Void `#08090a`, with Carbon `#0f1011` and Obsidian `#161718` for raised surfaces |
| Hairlines | Graphite `#23252a`, with Smoke `#383b3f` and Ash `#62666d` above it |
| Text | Mist `#d0d6e0`, with Fog `#8a8f98` for muted text |
| One action colour | Acid Lime `#e4f222`, for a single primary action per view |
| Face | Inter Variable with `cv01`, `ss03`, `zero`; weights 300, 400, 510, 590 |
| Corners | 6 px for buttons and inputs, 12 px for cards, 9999 px for pills |
| Depth | Hairline borders, no shadows; no gradient except a hero floor |
| Spacing | 4 px base, compact |

**What the rules say today, and what each would stop.**

| Rule | Says | Stops |
|---|---|---|
| `SPEC.md` N2, `CLAUDE.md` rule 6 | The only saturated colour is an LED's; no accent colour | Acid lime, anywhere |
| `tokens.rs` T7 | Every role but the three lit LEDs, `plate` and `frame` is pure grey | The reference's blue-tinted greys |
| `tokens.rs` T4, rule S8 | Nothing within ΔE 3 of `#0b0b0b`, `#111111`, `#F4F1EA`, `#D97757`, unless the token says why | Void `#08090a`, which is inside the cluster, unless it says why. S8 has this exit written in (`docs/05` section 4) |
| `tokens.rs` T2 | `ink` and `ink.quiet` at least 7:1 against `surface` | Fog `#8a8f98`, which is 6.1:1 against Void |
| ADR-0008 decision 6, `docs/05` section 2.3, S15, `tokens.rs` T10 | No radius group; no `border-radius` but `50%` | One corner, 8 px (the reference's is 6; the owner's word of 12:00 is 8) |
| S11, `tokens.rs` T10, `docs/00` section 3.2 | Not Inter | The reference's face |
| `docs/05` section 5 (WCAG 2.1) | A control's boundary is 3:1 against the ground | Graphite `#23252a` as a control's border (1.3:1) |
| S1, S4, S5, S16 | No gradient, shadow, letter-spaced or all-caps label, transition | Nothing the reference needs; its "no shadows" is ours |

**What the ElevenLabs reference asks for** (the transcription and the screenshot):

| The reference | Value |
|---|---|
| Illustration | Black hairline line drawings of geometric solids on a card, with dotted construction lines; pastel radial gradient "orbs" as atmosphere |
| Cards | White or `#f0efed` fills, 1 px `#e7e5e4` hairline, radii 12, 16 and 24 px |
| Buttons | An ink pill (`#292524`, white text), an outline pill; radii up to 9999 px |
| Face | Waldenburg 300 (licensed, a serif-like display face) for headlines, Inter for body, +0.15 to +0.18 px tracking, uppercase 12 px badges |
| Depth | One soft hover shadow, `0 4px 16px rgba(0, 0, 0, 0.04)` |
| Don'ts | No saturated brand or action colour; no bold display copy |

Each of these has a route written into the rules (an ADR, a stated reason, a narrower rule in place of a wider one). None of them is met by changing a threshold.

## Decision

### 1. The look reaches the strip and the page, and stops there

The strip (`#strip`), the page's ground (`html`, `body`), the case's edge and the line drawings in the ground (decision 10) change. The panel's drawing does not: the plate, the frame, the keys, the printed ink and the LEDs keep ADR-0010's roles and values. S17 now also runs the other way (decision 8).

### 2. What is taken from the reference, and what is not

| Taken | Not taken, and why |
|---|---|
| A near-black ground; a hairline between regions; controls as a border on the ground; one small corner; a compact scale; Inter Variable; a calm, flat, contrast-led hierarchy | **Acid lime.** N2 and rule 6 say the only saturated colour is an LED's, and a lime action would read as a lit green LED. The one filled control is `ink` on `surface` (Play); a pressed toggle takes `surface.raised` |
| | **The blue tint in the greys.** T7. The ground is pure grey |
| | **Fog for muted text.** 6.1:1 is under the 7:1 floor; `ink.quiet` is lighter (7.34:1) |
| | **Graphite for a control's border.** 1.3:1. A control's border is `edge.control`, 3.26:1; the hairline between regions is `edge`, which only has to be seen |
| | **Tracking, shadows, cards, pills, the hero floor, Berkeley Mono, a light theme.** S5, S4, §4 of the north star, S1, S7. The strip has none of these to apply them to |
| | **Four weights.** Two: regular 400 and strong 510 |
| ElevenLabs: hairline line drawings with dotted construction lines (decision 10) | **Cards** (12, 16 and 24 px radii, a warm off-white fill) and **pills**: the north star's "no cards", and one corner is not a scale. The off-white `#f5f5f5` to `#f0efed` family also lives next to S8's `#F4F1EA`, on a ground that is now dark |
| ElevenLabs: an ink primary button | The shape. Play is already `ink` on `surface`; it keeps the 8 px corner, not a pill |
| | **Gradient orbs, pastel tints** (S1, N2, rule 6), **the soft hover shadow** (S4), **tracking and uppercase badges** (S5), **Waldenburg** (licensed, and a serif-like display face that the north star rejects) |

### 3. Three colour roles are added to the fifteen, and three values are re-set

The eighteen: the nine of ADR-0008, the six of ADR-0010, and three the strip owns.

| Role | What it is | Value | Rule (a test in PR A, red first) |
|---|---|---|---|
| `surface` | The page's and the strip's ground | `#090909` (was `#2e2e2e`) | pure grey; carries the S8 suppression and its reason (`#090909` is well inside S8's ΔE 3 of `#0b0b0b`: arrived at on purpose, from the owner's reference) |
| `ink` | Text, and the one filled control | `#d4d4d4` (was `#ececec`) | pure grey; at least 7:1 against `surface` (13.43:1) and against `surface.raised` (11.5:1, T16) |
| `ink.quiet` | Captions, status lines, labels | `#9d9d9d` (was `#bdbdbd`) | pure grey; at least 7:1 against `surface` (7.34:1) |
| `surface.raised` | The fill a pressed or selected control takes | `#1c1c1c` | pure grey; the tone above `surface` (T15) |
| `edge` | A hairline between regions | `#242424` | pure grey; at least 1.2:1 against `surface` (1.28:1, T15) |
| `edge.control` | A control's border | `#626262` | pure grey; at least 3:1 against `surface` (3.26:1, T14) |

The six tones from the ground up run `surface`, `surface.raised`, `edge`, `edge.control`, `ink.quiet`, `ink`, each lighter than the last (T15). `focus` and `focus.inner` do not move: the white ring is 19:1 against the new ground. The panel's roles do not move.

The values are defaults, taken from the reference with the three corrections in decision 2 (pure grey, a lighter muted text, a visible control border). The owner ratifies them by merging PR B with its screenshot; a value can change in that pull request without a new ADR if it keeps the rules.

### 4. The case hugs the plate, and the page shows round it

P8b drew the frame as `#panel-frame`'s background, filling the room under the strip. That is the screenshot's dark brown, and it hides the page. From PR B the frame is a rectangle inside the panel's own SVG, the size of its view box (the plate and the margin that is already there), so the case is an object on the page: the page's ground shows round it, and the panel still scales as one object and never rearranges. This overturns P8b's decision D-P8-1 (the frame fills the room); the roles and values are unchanged.

### 5. The face is Inter Variable, as a file in the repository

- The tokens gain `type.face`: the name `Inter Variable`, the variable font file (`pages/fonts/inter-latin-wght-normal.woff2`, the Latin subset of the open-source Inter, 48 KB), its licence beside it (`pages/fonts/inter-LICENSE.txt`, the SIL Open Font License 1.1), and the features `cv01`, `ss03`, `zero`. `type.family` leads with it, then `Helvetica Neue`, `Helvetica`, `Arial`, `sans-serif`.
- **T17 (new):** a face the contract names is a `.woff2` file under `pages/fonts/` with a `.txt` licence beside it, its name leads `type.family`, and its features are four-letter tags. Naming a face is therefore not a promise about what the visitor has installed, and nothing is fetched from another host.
- The generated `tokens.css` writes the `@font-face` (the whole weight axis) from the contract; a stylesheet writes no face, weight, feature or variation of its own (S2 now says so for weight, feature and variation).
- **Weights (T18, new):** `type.weight.regular` 400 and `type.weight.strong` 510. Emphasis is made with weight, and not with bold: the strong weight is under 700.
- `type.size.body` goes from 16 to 14 (the strip is compact); `type.size.label` stays 12.
- **S11** is narrowed, not dropped: `Inter`, `Google Sans`, `SF Pro` and the system faces are still refused anywhere in the app's source, and a bare `Inter` in the tokens is still refused (T10). The one face is the file the contract names, reached only through `var(--type-family)`. "Not Inter" in `docs/00` section 3.2 and E4 changes to say so.
- There is one family, so the panel's printed names are drawn in Inter too. They are Helvetica-like in the instrument; the owner's screenshot in PR B is where a difference is caught. The Curator's specimen decision (`type.family` stays `pending`) is not closed by this.

### 6. One corner

`radius.control` is 8 px. The reference's corner for buttons and inputs is 6, and 6 was this ADR's first value; the owner's word of 12:00 ("round the corners to an 8px") replaced it, and one corner stays one corner, so every control of the strip and the number field's light block take 8 and none takes another. It is one token, not a scale: `tokens.rs` T10 keeps the `radius` group to the one key `control` between 1 and 12, and refuses a `radius.card` or a second value. **S15** becomes: no `border-radius` but `50%` and `0`, and `var(--radius-control)` in a rule whose selectors all begin at `#strip`. The panel's controls are still the hardware's shapes, and nothing else has a corner.

### 7. How the strip is drawn (PR B)

Flat, with no fill but the ground: a hairline of `edge` under the strip; a control is a border of `edge.control` and the ink on the ground with an 8 px corner; Play is the one filled control (`ink` fill, `surface` text, strong weight), and the one other fill at rest is the **stepper** of a number field, the small block with an up and a down arrow that the owner sees as beige: the browser draws its own in a light tone of its choosing and cannot give it a corner, so the page draws it, `ink` for the block and `surface` for the arrows, with the same 8 px corner, redundant with the arrow keys and so out of the tab order; a toggle that is on takes `surface.raised` and the ink as its border; the focus ring is the same two strokes; the status lines are `ink.quiet`; sentence case, no tracking. No shadow, gradient, card, pill, transition or accent. The tone and the border change together when a toggle is on, so state is not carried by a shade alone, and `aria-pressed` is unchanged.

### 8. S17 runs both ways

The six panel roles stay out of the strip (ADR-0010). From now the strip's tones and ink (`surface`, `surface.raised`, `edge`, `edge.control`, `ink`, `ink.quiet`, and the shine that names `ink`) may not appear in a `#panel` rule. A test with synthetic stylesheets shows it fires.

### 9. The series

| PR | What | Tier | Why |
|---|---|---|---|
| A | This ADR and its doc edits; `contracts/design.tokens.json` and its schema (three roles, three values, `radius`, `type.face`, `type.weight`); `tokens.rs` (T10 amended, T14 to T18, the ground's S8 exit, eighteen roles, each rule shown to fail); `app_source_rules.rs` (S2, S15, S17 both ways); the generator, `tokens.ts`, the font and its licence, the MIME type; baseline | High | A contract file |
| B | `pages/app.css`, `pages/app.html` (the stage and the ground), `src/panel.ts` (the frame inside the SVG), `src/ground.ts` (the drawings), the browser tests for the new look, the tests P8b wrote that assumed the old one (updated, none removed), screenshots | Medium | `apps/web` only |

PR B carries PR A's commits until it merges. The agent merges neither.

### 10. Line drawings in the ground above the case

The page ground behind the panel carries three small drawings, in the language of the owner's screenshot and in none of its three compositions: hairline strokes, no fill, dotted construction lines, geometric solids. They are this product's own and are drawn from the tokens: a **detent** (rings whose radii are the space scale, round a dotted crosshair), a **stack** (four oblique planes, one per track, joined by dotted verticals) and an **orbit** (an ellipse on dotted axes, with a small circle on it). They are generated by `src/ground.ts` from the space tokens, so no length is written in the code (S12 now covers it), and they are the ground's only decoration.

- They sit in the ground, above the case, in the room the case leaves: the case is sized to the window by the plate's aspect and takes what it needs; the drawings scale to what is left and are gone when none is. They are never behind the case, never under the strip, and never in the panel (no overlay on the panel, `docs/05` section 1).
- Their strokes are `edge.control`, solid and dotted alike, a hairline wide and unscaled. The hairline tone `edge` was tried for the construction lines and is not seen at 1.28:1 on the ground (the dots vanish). A rule in `app_source_rules.rs` keeps every `#ground` rule to that one role and to `fill: none`, and a Chromium test measures the computed strokes. They are `aria-hidden`, take no pointer events and no focus, and do not move (P3, S14).
- No new role, and no new token: dots are `stroke-dasharray: 0 var(--space-1)` with a round cap.
- If the owner would rather have the ground empty, deleting `#ground` and its element is one edit; the tests that name it go with it.

## Alternatives considered

- **Take the reference whole, acid lime included.** The most literal reading of the owner's message. It needs N2 and rule 6 amended and an accent colour next to the LEDs it would be mistaken for. Not decided here; if the owner wants it, that is a separate ADR with its own tests, and it would be the first accent colour in the product.
- **Take the reference's own greys, tint and all.** Eleven more roles that fail T7, or T7 loosened for a taste. The pure greys are within a few points of lightness of the reference and differ only in the faint blue.
- **Restyle the strip and leave the page brown.** Smaller, and it would leave the case filling the window under a black bar, which is the owner's complaint about "background".
- **Keep Helvetica and borrow only the colours.** The reference is as much its face as its colours, and the owner pasted the face. It costs a file and a licence.
- **Two families, Inter for the strip and Helvetica for the panel.** More faithful to the hardware's printing, and a second family to choose, test and ship. The screenshot decides whether it is needed.
- **Cards.** Both references have them. The north star's "SaaS card kit: impossible by construction" stands.
- **Copy the three ElevenLabs drawings.** The nearest reading of the screenshot, and a specific artwork of someone else's. These drawings are made for this product and share the language only.
- **Put the drawings on cards, as in the screenshot.** Cards are refused above. The drawings stand on the ground.
- **Give the browser's spin button a corner.** It is the first thing to try, and it does not work: the browser paints the control itself, takes no `border-radius` on it, and chooses its tone by its own rule (dark in one browser, light in another), so the owner's 8 px could not be promised in any browser. Checked in Chromium on the built page.
- **Hide the spin button.** Keeps one corner and loses the click to step that the owner has in front of them, and the owner asked for its corners, not its removal.
- **Make the stepper dark (`surface.raised`) and so keep Play the one fill.** It fits the rest of the strip better, and it changes a colour the owner did not ask to change. It is one token in one rule if the owner wants it, and the first thing to try if the light block reads as loud.

## Not decided here

- **Acid lime or any accent** (above). **A hero floor, Berkeley Mono, a light theme, pills, badges, pastel orbs, a display face.** None has anything to apply to.
- **More drawings, or drawings elsewhere** (a settings page, an empty state). Three, above the case.
- **The Curator's specimens** for the panel's printed names.
- **Other pages.** The strip and the page behind the panel are the whole app today. A settings page, when there is one, takes this look by default and is its own decision.
- **How the case should be framed at very wide windows** (a maximum width, centring). The panel scales to the room it is given; the page ground shows on the sides.
- **roland50.studio**, never read.

## Consequences

- PR A changes the contract and its tests; the stylesheet, until PR B, still draws the strip as before, now on the new ground and in Inter. The page is usable and a little odd for one pull request. PR B is where it is right.
- It is reversible: closing the pull request leaves things as they are; a value can change in PR B without a new ADR as long as T2 and T14 to T16 hold.
- The assertion set grows (T14 to T18, the ground's exit, the radius and strip-tone tests, S2 and S15 and S17 both ways, the generator's font test); none leaves, and the tests P8b wrote that named the old strip's colours are updated to name the roles, not removed.
- **CLAUDE.md rule 3** asks for an expiry date on a loosening. This ADR loosens no number: 7:1, 3:1, ΔE 3 and chroma 12 are as they were. S11, S15 and the "no radius group" line of T10 each change from a wider rule to a narrower one that stays in force (T17 and the `#strip`-only corner), so nothing lapses and no date is set. If the owner reads them as loosenings, they set a date when merging and the Referee enforces it.

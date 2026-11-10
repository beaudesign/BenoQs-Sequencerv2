# ADR 0016: The shell is a paper ground with one curve

## Status

**Proposed** on 2026-11-10 by the Conductor. It takes effect when the owner merges the pull request that carries slice R1 of SPEC-0009 (the contract and its tests), as ADR-0008, ADR-0010 and ADR-0011 did. Merging the pull request that carries this file alone approves nothing: it puts the proposal on `main` to be read. If the owner closes it, the strip and the page stay as ADR-0011 left them.

It answers the owner's own words of Tue 2026-11-10, quoted in full in `specs/SPEC-0009/README.md` section 1: "think 16px smooth curves of the shell", four references, "Here;s the design palette and font you should use" with a Figma link, and, asked what happens to the midnight look of 13 October, the choice "Paper replaces midnight".

A second message the same day, with a screenshot of the prototype's panel: "This panel shoudl have 16px corners and feel more built in". Decision 5a is its answer.

It would supersede ADR-0011 decisions 1 to 3 and 5 to 7 (the midnight values, the Inter face, the one 8 px corner and how the strip is drawn), and the part of decision 4 that makes the frame a rectangle inside the panel's drawing. ADR-0011 decisions 8 (S17 runs both ways) and 10 (the ground is empty) stand. `SPEC.md` N2 and `CLAUDE.md` rule 6 are not touched.

## Context

**What the owner gave.** The Figma frame "Shell" (file `Pw2mc2YkHW3xIrblIS7xVJ`, node 73:15030), read through the Figma connector on 2026-11-10. Its variables:

| Variable | Value |
|---|---|
| `surface/background` | `#f8f5f0` |
| `surface/container` | `#ffffff` |
| `surface/inverse` | `#252525` |
| `surface/inset` | `#66491f` at 5% |
| `content/primary` | `#252525` |
| `content/secondary`, `content/tertiary` | `#252525` at 60% and at 35% |
| `border/minimal` | `#e4e1db` |
| `Shadow/md` | a drop shadow, 0 2 4, `#2b2925` at 8% |
| Face | IBM Plex Sans: Regular, Medium and SemiBold, at 12 and 14 px |
| Corners in the frame | 6, 8, 12 and 36 px |

**What the rules say today, and what each would stop.**

| Rule | Says | Stops |
|---|---|---|
| ADR-0011 decision 3, `docs/05` section 1 | The ground is a near-black pure grey | A light ground |
| `tokens.rs` T7 | Every role but the lit LEDs and the two materials is pure grey | The paper tones, whose chroma is 2.7 to 3.3 |
| `tokens.rs` T1 | Every colour is `#rrggbb` | The Figma inks and inset, which are a colour and an opacity |
| `tokens.rs` T2 | Text is at least 7:1 against the ground | The 60% ink (4.1:1) and the 35% ink (2.1:1) |
| `tokens.rs` T4, rule S8 | Nothing within ΔE 3 of `#F4F1EA`, unless the token says why | `#f8f5f0`, about 1.8 from it. S8 has the exit written in |
| `tokens.rs` T10, rule S15 | One `radius` key, `control`, from 1 to 12 | 16 px, and a second key |
| Rule S11, T17 | The one face is the file the tokens name | IBM Plex Sans, until it is a file with its licence |
| Rule S4 | No `box-shadow` | `Shadow/md` |
| `docs/00`, ADR-0011 | No cards; no drawing anywhere without a new ADR | A raised container; the routing figure |
| `docs/05` section 1 | The strip never sits on the panel | A sheet that opens over the case |

Each has a route written into the rules (an ADR, a stated reason, a narrower rule in place of a wider one). None is met by changing a threshold.

## Decision

### 1. The look reaches the shell and stops there

The top row, the notice, the sheet and the page's ground change, and so does the shape of the case (decision 5a). What is drawn on the plate does not: the plate, the frame, the keys, the printed ink and the LEDs keep ADR-0010's roles and values, and S17 keeps the two apart in both directions.

### 2. The colour roles

The roles stay eighteen. Six values are re-set. Every value is a solid `#rrggbb` (T1): where the Figma file gives a colour and an opacity, the value is that colour laid over the ground.

| Role | What it is | Value | Rule it must keep |
|---|---|---|---|
| `surface` | The page's ground | `#f8f5f0` (was `#090909`) | carries the S8 suppression and its reason: the owner's own Figma variable, about ΔE 1.8 from `#F4F1EA` |
| `surface.raised` | A container on the ground, and the fill of a control that is on | `#ffffff` (was `#1c1c1c`) | lighter than `surface` |
| `ink` | Text, and the one filled control | `#252525` (was `#d4d4d4`) | at least 7:1 on `surface` (14.1) and `surface.raised` (15.3) |
| `ink.quiet` | Captions, hints, a field's name | `#4d4d4c` (was `#9d9d9d`) | at least 7:1 on both (7.8, 8.5) |
| `edge` | A hairline round a container | `#e4e1db` (was `#242424`) | at least 1.2:1 on `surface` (1.20) |
| `edge.control` | A control's border | `#888784` (was `#626262`) | at least 3:1 on both (3.3, 3.6) |

`focus` becomes `ink` with `surface.raised` as its inner stroke, so the ring shows on both grounds. The LED roles and the six panel roles do not move.

**The Figma file's `content/secondary` and `content/tertiary` are not taken as text colours.** At 60% and 35% they are 4.1:1 and 2.1:1 on the ground, under the 7:1 floor. `ink.quiet` is the nearest tone that keeps the floor. Lowering the floor would be a loosened threshold (rule 3), and this ADR does not do it (SPEC-0009 Q-9-1).

**T7 is narrowed, not dropped.** A neutral may carry the paper's tint up to a chroma of 4; the measured values are 0 to 3.3. Above 4 a role is a hue, and only the lit LEDs and the two materials may be one. The test is shown to fail on a neutral at chroma 5.

### 3. One curve, and one derived from it

- `radius.shell` is **16**. It is the corner of a container: the transport bar, the notice, the case and the sheet.
- `radius.control` stays **8**. It is the corner of a control, and it is not a second choice: a control sits 8 from the edge of its container, so its arc and the container's are concentric, and 16 less 8 is 8.
- **The curve is continuous where the browser can draw one** (`corner-shape: squircle`; Chromium 156 has it, checked on 2026-11-10, and Chromium is the page's browser), with the radius scaled so that it reads as the same size, and a plain arc elsewhere.
- No pill, no third value. T10 allows the `radius` group exactly these two keys, with `shell` twice `control`. S15 becomes: no `border-radius` but `50%`, `0` and the two tokens.

The Figma frame's own corners (6, 8, 12 and 36) are not taken: the owner's words are "16px smooth curves", and four unrelated corners in one frame is what the one-corner rule is for.

### 4. The face is IBM Plex Sans, as a file in the repository

`type.face` names IBM Plex Sans, a `.woff2` under `pages/fonts/` with its licence (the SIL Open Font License 1.1) beside it, as T17 asks. `type.weight.regular` is 400 and `type.weight.strong` is 500 (T18: under 700). Sizes stay 14 and 12. The Inter file and its licence leave in the same pull request. There is one family, so the panel's printed names are drawn in it too (SPEC-0009 Q-9-4).

### 5. Three containers, by name

The north star's "no cards" is narrowed to: no card grid, and no container that is not one of these.

| Container | Tone | Why it is one |
|---|---|---|
| The transport bar (and the notice, which is its kind) | `surface.raised`, a hairline of `edge` (the notice takes `ink`) | The three controls used all session are one object, apart from the two used once |
| The case | `frame`, which is its own tone | The instrument is part of the shell's layout, not an object set down in it (decision 5a) |
| The sheet | `surface.raised`, a hairline of `edge` | What is set once a session, out of the way until it is asked for |

A sheet docks beside the case and never covers the panel. The case narrows and the panel scales as one object.

### 5a. The case is built in

The owner's word: "This panel shoudl have 16px corners and feel more built in".

- The case's corner is `radius.shell`, the same continuous curve as the bar's and the sheet's.
- Its edge is `space.2` (8) wide at every size. It does not scale with the panel, so the plate's corner is the concentric `radius.control`, by the same rule that puts a control 8 from its container's edge.
- The case starts flush under the top row and is as wide as the row, or as the room a sheet leaves. It is at the top of its room and not the middle, which changes P9-9. There is no well round it.
- So the frame stops being a rectangle in the panel's own drawing (ADR-0011 decision 4) and becomes the element that holds the drawing. The panel still scales as one object and never rearranges. `frame` and `plate` keep their values, and S17 still keeps them to `#panel` and `#panel-frame`.
- S15 allows the two radius tokens on `#panel-frame` and `#panel` as well as on the shell's own rules.

### 6. One drawing, which is a readout

The Connections sheet opens with a routing figure: the input, the sequencer and the two outputs as nodes, a solid line with an arrowhead where notes or a clock run, a dotted line where nothing is chosen, and one sentence under it that says the same in words. It is drawn from the state the fields beside it read, so it cannot disagree with them, and a test says so.

This is the new ADR that ADR-0011 asks for before any drawing returns. It is not the kind the owner took out on 13 October ("they offer nothing"): those were still, and said nothing. This one changes when a device is chosen, and each line in it is a fact about where the music goes. The ground round the case stays empty, and `the_ground_has_no_drawings` stands.

### 7. What is not taken

| Not taken | Why |
|---|---|
| `Shadow/md` | S4. Nothing floats over anything, so nothing needs to be lifted |
| An accent, or a hue for a state | N2, rule 6. On is a doubled `ink` border with the raised fill and the strong weight; a fault is an `ink` border and a mark with a shape |
| Movement | S16. A sheet opens at once (SPEC-0009 Q-9-3) |
| Pills, a rail, a search field, a user chip | The frame is a project list. The shell has three controls used all session and nine used once |
| SemiBold | Two weights (T18) |

## Alternatives considered

- **Keep midnight and round the corners to 16.** The smallest reading of "16px smooth curves". The owner was asked and chose "Paper replaces midnight".
- **Paper by default with midnight as a dark theme.** Offered, not chosen. It doubles the token values and every contrast test.
- **Take the Figma inks as they are.** The most literal reading of "the design palette". It puts every caption under the floor the product has kept since ADR-0008.
- **A rail down the left, as the Figma frame has.** Offered as direction B. It keeps setup on show all session, which is finding 2 of SPEC-0009.
- **The figure always on show beside the case** (direction C). It takes width from the instrument for something read once a session.
- **A sheet that floats over the case with a shadow.** Nearer the references, and against `docs/05` section 1 and S4 at once.
- **Full pills for the bar and its buttons.** Both of the first two references have them. A pill is a third corner.

## Not decided here

- **The 7:1 floor** (Q-9-1), **movement** (Q-9-3), **a second family for the panel's printed names** (Q-9-4).
- **Where SPEC-0006's kit and SPEC-0007's pedalboard would sit** in this shell. Neither is approved. The stage is the obvious place for a second object; that is their specs' to say.
- **A dark theme.**

## Consequences

- `contracts/design.tokens.json` changes (High tier), with `tokens.rs` first: T7 narrowed, T10 amended, six values re-set, the face replaced. Each changed rule is shown to fail before it passes.
- `docs/05-design-system.md` sections 1, 2.1, 2.3 and 2.4 and rules S8, S11 and S15 are rewritten in the same pull request; `docs/00` where it says "no cards".
- The browser tests that assume the midnight look are updated, none removed.
- ADR-0011's status line gains "superseded in part by ADR-0016".

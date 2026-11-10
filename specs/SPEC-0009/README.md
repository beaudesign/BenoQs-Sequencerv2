# SPEC-0009: The shell: a paper ground, a transport bar and a Connections sheet

| Field | Value |
|---|---|
| Spec | SPEC-0009 |
| Revision | r0, draft, 2026-11-10 |
| Stage | **Planning only.** It adds a prototype page under `specs/SPEC-0009/prototype/` and no code, contract, fixture, threshold or baseline entry. Every decision and slice below is a proposal, not an approval |
| Risk tier | **Low** for the spec itself (`specs/`, `adr/`, `journal/`). Slice R1 is **High** (`contracts/design.tokens.json`). Slices R2 to R4 are **Medium** (`apps/web/`, Forge) |
| Owner and approver | `@beaudesign` |
| Task | **`WENGE-0043`** (the next free id: `main` and the open pull requests end at `WENGE-0042`) |
| Supersedes in part | ADR-0011 (the midnight ground), if ADR-0016 is merged. `docs/05-design-system.md` sections 1, 2.1, 2.3 and 2.4 and rules S8, S11 and S15 would change with it, in slice R1 and not here |
| ADR | **ADR-0016, proposed** (`adr/0016-the-shell-is-a-paper-ground-with-one-curve.md`). 0013 and 0015 are reserved |
| Base | `main` at `af6e404`. What is said about the page below was read from `apps/web/pages/app.html`, `apps/web/pages/app.css`, `contracts/design.tokens.json` and the screenshot in `README.md` on that commit. **The page was not run**: this machine has no `wasm32-unknown-unknown` target, and the published address returned 404 on 2026-11-10 |
| Companion | **`prototype/shell.html`**: the proposal as a page you can click. `prototype/embed_panel.py` writes the panel's measured layout into it |

## 1. The ask

Recorded here because chat is not a source of truth. The owner wrote on Tue 2026-11-10:

> Evaluate the UI, UX for this - specifically the shell - think 16px smooth curves of the shell - https://www.udo-audio.com use https://cursor.com design references, https://vmragona.com/visai/, https://bakkenbaeck.com/case/deepmind - study them closely, plan some prototypes for the reshell, then let me choose which direction for you to build. Here;s the design palette and font you should use: https://www.figma.com/design/Pw2mc2YkHW3xIrblIS7xVJ/Agent-Town?node-id=73-15030&t=tYjR8hcxAMcschA3-1. Let's 1000x the designs

Four directions were put to the owner the same day (a floating bar; a workbench with a rail; an annotated plate; the floating bar with a routing diagram in its Connections sheet). The owner chose, through the session's question form:

- **Direction:** "D. Bar + routing diagram".
- **Theme:** "Paper replaces midnight".
- **Medium:** "Clickable HTML".
- **Rules:** "Draft ADR-0004", which is read as "draft the ADR": the number 0004 came from a stale checkout, and the ADR is 0016.

These choose what to prototype and which ADR to draft. They approve no slice and ratify no token value. A first round of directions that day was drawn for the product ADR-0006 removed, from a checkout 207 commits behind `main`; it is discarded and nothing from it is in this spec.

## 2. The shell today, and what is wrong with it

"The shell" is read as everything on the page that is not the panel: the strip and the ground round the case.

| Finding | Where it shows |
|---|---|
| **One level.** Twelve controls and seven status lines sit in one wrapping row. Play has the same weight as the lookahead, which wraps onto a line of its own at 1440 px | `app.html`, the README screenshot |
| **Setup is always on show.** Two MIDI outputs, the input, the clock state, the clock offset and the lookahead are chosen once a session and hold the top of the page for all of it | `app.html` |
| **A fault is whispered.** "MIDI access was refused. Allow it for this site, then press Play again." is the quiet ink at the label size, in a run of sentences beside "Started." | `app.css` (`#strip p`), the README screenshot |
| **Nothing shows where the notes go.** Which device is on which port, and which way the clock runs, is four select boxes to read one by one | `app.html` |
| **The instrument is boxed in.** The case sits in the middle of what the strip leaves, with empty ground above and below it | the README screenshot |
| **Names carry their units.** "Tempo in beats per minute", "Clock offset in milliseconds", "Lookahead in milliseconds" are the visible labels | `app.html` |

## 3. What the references do

Read in a browser on 2026-11-10. Values are the computed styles of each page.

| Reference | Ground and ink | Corners | What is taken |
|---|---|---|---|
| udo-audio.com | white, black | 11.5 px tiles, full pills | The instrument is the only content. It sits in a large rounded tile; the navigation is one small floating bar |
| cursor.com | `#f7f7f4`, `#26251e`, ink at 60% and 40% | 4 and 8 px, full pills | A paper ground with stacked tones; hierarchy by the weight of one ink, with no hue |
| vmragona.com/visai | light grey, deep indigo | 4 px plates | A system drawn as a captioned figure: nodes, lines and dotted construction |
| bakkenbaeck.com/case/deepmind | white, `#121212` | 2 px | A two-column sheet: a quiet name on the left, its value on the right |

Not taken: UDO's and Cursor's pills (one corner stays one corner, section 5), UDO's numbered eyebrows and the figure numbers of the third reference (S6), Cursor's monospace (S7) and its amber accent (N2).

## 4. The design

`prototype/shell.html` is the proposal. In words:

1. **The top row** has three parts. On the left, the name and one word of state ("Not started", "Playing", "Stopped"). In the middle, the **transport bar**: Play, the tempo with a minus and a plus, and Metronome, in one raised container. On the right, two buttons that open a sheet: **Sound** and **Connections**. The Connections button carries a mark that is a ring when no device is chosen, a filled disc when one is, and a square with an exclamation mark when something needs attention. The mark is shape, not colour.
2. **A notice** appears under the top row only when something stops the music from getting out, with one sentence and a button that opens the sheet that fixes it. It covers five cases: access refused, a master clock with no output, an echo with no output, a clock to follow with no input, and the built-in sound off with no output chosen. The last is new: today the page says nothing when it has been made silent.
3. **The stage** is a rounded well that the case fills, in the middle, as P9-9 has it.
4. **A sheet** docks beside the stage. It never covers the panel (`docs/05` section 1): the stage narrows and the case scales, as one object.
   - **Connections** opens with a **routing figure**: the input, the sequencer and the two outputs as nodes. A line is solid with an arrowhead where notes or a clock run and dotted where nothing is chosen, and a sentence under it says the same in words. Below it are the three device fields, with "Channels 1 to 16" and "Channels 17 to 32" under the outputs (the port rule of SPEC-0008 section 2.1), then the clock state in the manual's four names, the clock offset and the lookahead, each with its unit after the number.
   - **Sound** holds the built-in sound, the drums and the Curve of a chord roll.
5. **Nothing moves** but the LEDs. A sheet opens at once. S16 stands.

## 5. Decisions proposed (each is ADR-0016's to make)

| # | Decision | Default |
|---|---|---|
| D1 | The ground is the owner's paper, and midnight goes | `surface` `#f8f5f0`, `surface.raised` `#ffffff`, a new `surface.well` `#f1ece6` |
| D2 | The inks are held to the 7:1 floor | `ink` `#252525` (14.1:1 on the ground). `ink.quiet` `#4d4d4c` (7.2:1 on the well). **The Figma file's 60% and 35% inks are 4.1:1 and 2.1:1 and are not used for text** |
| D3 | Edges | `edge` `#e4e1db` (the Figma border, 1.2:1 on the ground). `edge.control` `#888784` (3.06:1 on the well) |
| D4 | One curve, and one derived from it | `radius.shell` 16 for a container; `radius.control` 8 for a control, which sits 8 from its container's edge so the two arcs are concentric. Drawn as a continuous curve (`corner-shape: squircle`) where the browser has it, a plain arc where it has not. No pill |
| D5 | The face | IBM Plex Sans, as a file in the repository with its licence (T17). Regular 400 and strong 500 |
| D6 | Three containers, by name | The transport bar, the stage and the sheet (and the notice, which is the bar's kind). No card grid |
| D7 | One drawing, which is a readout | The routing figure. It is redrawn from state, and every line in it means something. ADR-0011's "any drawing, anywhere, is a new ADR" is answered by ADR-0016 |
| D8 | No shadow | The Figma file's `Shadow/md` is not taken. Nothing floats over anything, so nothing needs one. S4 stands |
| D9 | No accent, no hue for state | N2 and rule 6 stand. On is a doubled ink border with the raised fill and the strong weight |

## 6. Slices (none approved)

| Slice | What | Tier |
|---|---|---|
| R1 | ADR-0016's doc edits; `contracts/design.tokens.json` and its schema (the values of D1 to D3, `surface.well`, `radius.shell`, the face and its file); `tokens.rs` (T7 gains a chroma ceiling for the shell's neutrals, T10 allows the second corner, the S8 exit for the ground), each rule shown to fail first | High |
| R2 | `pages/app.html` and `pages/app.css`: the top row, the transport bar and the stage. The strip's controls move; none is removed | Medium |
| R3 | The sheet: Connections with its fields, Sound, the notice and its five cases, with a browser test for each case | Medium |
| R4 | The routing figure, drawn from the same state the fields read, with a node test that it never disagrees with them | Medium |

## 7. Acceptance examples (for the slices, when they are approved)

- **A1.** At 1440 by 900 with the sheet closed, Play, the tempo and Metronome are on one line and nothing in the top row wraps.
- **A2.** With MIDI refused and Play pressed, the notice shows the sentence the page has today, and its button opens Connections.
- **A3.** With the built-in sound off and no output chosen, the notice says nothing can be heard, and its button opens Sound.
- **A4.** Opening a sheet never covers a pixel of the panel; the case's width over its height is unchanged.
- **A5.** Every sentence, name and control is at least 7:1 against what is behind it, and every control's border at least 3:1 (computed, per container).
- **A6.** With Out 1 chosen and Out 2 not, the figure has one solid line and one dotted, and the sentence under it names the device.
- **A7.** No `box-shadow`, gradient, `transition` or easing keyword in the page's styles; no `border-radius` but `50%`, `0` and the two tokens.

## 8. Checked in the prototype (2026-11-10, Chromium 156)

| Check | Result |
|---|---|
| The script runs with no console error | Yes |
| 1440 by 900, sheet open: no page scroll | `scrollWidth` 1440, `scrollHeight` 900 |
| 390 by 844: no sideways scroll | `scrollWidth` 390 |
| Play starts the chase; the state word and the button change | "Playing", "Stop", five red LEDs |
| Access refused, after Play: the notice | The sentence of `midi-access.ts`, and the button |
| `corner-shape: squircle` is supported; IBM Plex Sans loads | Yes; yes |
| `box-shadow`, `transition`, gradient or easing in the file | None (`grep -c`: 0) |

The panel in the prototype is a simplified drawing made from `apps/web/layout/panel.layout.json` (119 controls, 21 legends and the 160 matrix keys). It is not the page's own panel, and its keys do nothing.

## 9. Questions for the owner

- **Q-9-1.** The Figma inks at 60% and 35% fail the 7:1 floor (D2). Keep the floor and use `#4d4d4c`, or lower the floor for the shell? The default keeps it; lowering it is a loosened threshold and needs its own ADR with an expiry (rule 3).
- **Q-9-2.** The plate (`#fcf3ec`) and the ground (`#f8f5f0`) are close: 1.007:1. The dark case is the only edge between the instrument and the page, and the well darkens the ground behind it a little. Is that enough, or should the well be deeper?
- **Q-9-3.** A sheet opens at once. A short movement would need S16 changed. The default is no movement.
- **Q-9-4.** One family: the panel's printed names would be drawn in IBM Plex Sans too, as they are in Inter today. The default is one family.
- **Q-9-5.** On a narrow window the sheet goes under the stage. Is the page meant for a phone at all? The default is that it only has to hold together there.

## 10. Approval record

Nothing in this spec is approved. An approval is the owner's own words, written here with the date.

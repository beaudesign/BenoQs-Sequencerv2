# SPEC-0002 P5 plan: Wave 1, "Play", the controller takes over from the page

| Field | Value |
|---|---|
| Task | `WENGE-0016` (`handoffs/WENGE-0016.ndjson`) |
| Spec | SPEC-0002 r1, approved 2026-09-30 15:06 Paris. This plan fills in the P5 row ("the workflow waves in `product.md` section 5") for **Wave 1 only**, and **narrows it in two places the owner must accept or reject** (D-P5-1, D-P5-2) |
| Plan revision | r0, written 2026-10-03 after the owner's "Go" (Sat 20:26 London). "Go" did not say which of five offered items; this is the one that needs no decision of the owner's to *start*, and the plan is the stage the lifecycle requires before Medium-tier work (`AGENTS.md`, "Tech spec: yes, for medium and high") |
| Stage | Tech spec, awaiting the owner. Nothing in P5 is built. No code, contract, threshold or existing doc is changed by this pull request |
| Tier | P5 as a whole is **Medium** (`SPEC-0002/README.md`). Slice tiers are in section 4; the Stop and Play change (P5d) is **High (timing)** |
| Hats | P5a Conductor. P5b Panelwright. P5c Forge. P5d Metronome (request first), Panelwright, Forge. P5e and P5f Panelwright and Forge |

## 1. What P5 is

`product.md` section 5, Wave 1: *Page mode: toggle steps, the chase-light, tempo, Play, Stop and Pause. Step zoom for VEL, PIT, LEN and STA. The MIX and EDIT encoders. Two MIDI outputs. Save and load in the browser.* A1 requires a fixture for every workflow in a wave's scope, and each wave ends "with its fixtures passing and a demonstration on real gear".

Wave 1 as it stands on `main` (8e4d7af), each line checked in the code:

| Wave 1 item | State | Where it is |
|---|---|---|
| Page mode step toggle | **Built** (P2b) | `crates/octoface/src/panel.rs`; fixtures in `tests/conformance/panel/page_view/` |
| Step zoom table, TGL, Main Mute skip, leaving | **Built** (P2b) | 16 fixtures in `step_zoom/`, 4 in `leaving_step_zoom/` |
| Two MIDI outputs | **Built** (P4b) | `apps/web/src/midi-out.ts`, the settings view |
| Step zoom: VEL, PIT, LEN, STA by their knobs | **Not built.** The controller accepts `Input::Turn` and ignores it (`panel.rs` line 38), the page draws no encoder, and `StepView` carries no attribute values | W10 in the digest, p015 to p017 |
| EDIT encoders | **Not built**; ten controls exist in `contracts/controls.json` (n 192 to 201), eight with no open question, `amt` and `mcc` pending Q04 | `edit.enc.*` |
| MIX encoders | **Not built.** In the manual they edit *other things*: unselected steps of tracks that hold a selection (W15 to W17, p018) and the per-track MIX map (W14, p072 to p074), both of which need machinery that is Wave 2's | `mix.enc.r0` to `r9` |
| Transport Play | **The page stands in.** `apps/web/pages/app.ts` lines 49 to 61 send the engine's `Play` itself for `transport.play` (ADR-0008 decision 4); the controller handles `transport.stop` only, and the MODE block's PLAY key (`mode.play`) is a different key | Q21 |
| Transport Pause | **Not built** | Q26 for its LED; see F-P5-3 and F-P5-4 |
| Tempo | **The page stands in.** `octoweb_set_tempo`; no control on the panel; the engine plays at 120 until something sets it | F-P5-2 |
| The chase-light | **Not built** (D-P3-5); the Panelwright has the request (`journal/panelwright/requests/2026-10-01-chase-light-and-transport-play.md`) | `octoweb_playheads` exists |
| Save and load in the browser | **Not built**; `tech.md` section 7 sketches `octopus-state/1` | High tier by `AGENTS.md` ("persistence format") |

Nothing in this table is blocked by D0. D0 changes what a tick *lasts*, not the numbers a knob edits or a bar shows (F-P5-5).

### Findings that change the plan

- **F-P5-1. The encoder path does not exist end to end.** The controller ignores turns, the page draws no encoder, and the fixture runner already has a `turn` directive but only `expect engine step <t> <s> active|skip = <bool>`. But no engine change is needed: `Command::SetStep` already writes `VelocityOffset`, `PitchOffset`, `LengthTicks`, `LengthMultiplier` and `StartOffset` (`crates/octocore/src/types.rs`), and `PageView::from_page` reads the `Page` directly, so the values are in reach. P5b and P5c are controller and page work only.
- **F-P5-2. "Tempo" is three things in the manual, and `product.md` cites the wrong page for the first.**
  1. The **global tempo**: the numeric field shows it in Page mode and, in Track mode, shows the program number instead (p050); the "main knob / tempo knob" turns it (p077, p086), and it is the only live knob under interface lock (p086).
  2. The **track tempo multiplier**: Track mode, the one-, two- and four-triangle play buttons, or hold circle TEMPO (XII) and use the MCH row (p049, p050).
  3. The **step LEN multiplier**: the same buttons in Step mode (p016, W13).
  `product.md` section 4 and the Wave 1 row cite p049 for the tempo encoder; p049 is (2). (1) rests on p050, p077 and p086, and the blog post `[B]` for the bar graph, which this repo holds only as a digest. `controls.json` has one `main.enc`, pending Q05 ("one knob or two"); p050, p077 and p086 together read as one knob with a role per mode, which is not stated. So P5e builds (1) only; (2) and (3) stay with Track zoom and the triangle keys (Q21).
- **F-P5-3. Pause is two things.** With a track selector held, Pause sets that track's condition to paused (p050, p049) and, with the sequencer stopped and zoomed into a track, toggles it (p050). Without one, "Pause and Resume (pressing pause again)" is the global pause (p086). The Panelwright's request assumed only the second. P5d builds the global one and leaves the track one for Wave 2, where track selection is built.
- **F-P5-4. The engine cannot tell Stop-then-Play from Pause-then-Resume, and the manual does.** `Command::Play` and `Command::Continue` share one arm (`crates/octocore/src/engine.rs` line 677), and `Stop` keeps every track's position. So on the engine today a Stop followed by Play *resumes*. The manual says "the chase-light will also be realigned whenever you Stop and then play a sequence. Pause and Resume ... will not realign the chase-light" (p086). The repo already knows Stop keeps the position (`tests/conformance/AMBIGUITIES.md`, the "stop with sounding notes" entry names "rewind the position on Stop" as the alternative). It matters for the clock too: the engine sends `Start` when the transport starts at tick 0 and `Continue` anywhere else (`engine.rs` line 613), so a master clock sends `Continue` after every Stop, although the manual's Stop then Play re-aligns the tracks (p086). This is a **Metronome change with a conformance test**, so it is a request first (P5d), and it is High (timing).
- **F-P5-5. The LEN and STA bars are in ticks, not milliseconds.** "Each Green increment corresponds to 1/192 of a note and each Red value corresponds to 12/192 = 1/16 of a note" (p015); a step is 12 ticks in the engine. The fixtures assert ticks. D0 stays open without touching them.
- **F-P5-7 (added at r1, found by P5b). The manual does not say which matrix row shows VEL, PIT, LEN or STA** (`findings.md` section 6, item 6, Q06; hardware). p015 and p016 say how a value looks in its row (tens Red and a Green ones LED, minus as three Green LEDs in columns 14 to 16, the LEN dot and the STA bars) and p068 shows velocity in the numeric quadrant and pitch in the inner circle as well (Q42). So P5b asserts the knob-to-value half and leaves the drawing pending; a person sees the value change when the page shows it (P5c) or when the rows are settled.
- **F-P5-6. The tempo knob and the clock follower must not fight.** In Slave (P4d) the follower steers the engine's tempo from the incoming clock, sending a new value whenever the loop's tempo has moved by more than a small threshold (`apps/web/src/follower.ts`). A knob that also writes it would be overwritten as soon as the estimate moved. Section 3, D-P5-6.

**How I will know it worked.** Each line is a command with an expected result, written red first. Section 4 says which slice owns each.

| # | Observable | How it is checked |
|---|---|---|
| W1 | In Step zoom, turning the VEL, PIT, LEN or STA knob by *n* detents changes the selected step's attribute by one unit a detent within the engine's limits, and nothing else changes. **Not asserted: the row that shows the value** (F-P5-7, Q06): p015 and p016 say how a row looks, not which row it is, so the drawing is a pending fixture | `.panel` fixtures with `turn`, cited to p015 to p017, written red first. The runner gains `given step ... = <int>` and `expect engine step ... <attr> = <int>` |
| W2 | Every EDIT encoder has an element on the page with the manual's own name, reachable by keyboard in layout order, turned by drag, wheel and the arrow keys, and each turn reaches `octoweb_input` kind 2 | Node tests over the page's input module; a Chromium test; the a11y check (A8) |
| W3 | Play, Stop and Pause-Resume run through the controller, and the page's `transport.play` mapping is gone | Panel fixtures; a source-rule test that `pages/app.ts` does not name `transport.play`; the Chromium test still starts and stops the engine |
| W4 | After Stop, Play re-aligns every track to the master clock (p086) and, with the clock on, sends `Start`; after Pause, Resume continues from where it was and sends `Continue` | A native engine test and a conformance fixture; any golden or fixture that stops and starts is listed first and re-derived in the diff |
| W5 | The main knob changes the global tempo by a stated amount per detent, the numeric field shows it, and in Slave it does nothing | Panel fixtures; a Node test with the follower in the loop |
| W6 | The chase-light is drawn from `octoweb_playheads`, delayed by the lookahead plus the snapshot's lead of about 13 ticks (`tech.md` section 5), and a skipped step is passed over | Fixtures asserted where the manual speaks, `pending` where it does not (the three questions in the Panelwright's request); a Chromium test that the light and the note agree |
| W7 | `cargo xtask verify --base origin/main` passes and the baseline only grew | CI `verify` |

## 2. What was measured before planning

All from `main` at `8e4d7af`; nothing here is a browser measurement.

- **Controller.** 775 lines in five files. Built workflows: page view toggle, Step zoom, leaving Step zoom, the EDIT cycle, PLAY mode. `Input::Turn` is "accepted and ignored until a workflow uses it". `Role::Stop` emits `Command::Stop`; there is no role for `transport.play` or `transport.pause`.
- **Fixtures.** 37 asserting `.panel` files in `tests/conformance/panel/` and 15 pending. The runner (`crates/octoface/tests/panel_fixtures.rs`) understands `press`, `release`, `click`, `turn`, `expect led`, `expect command`, `expect intent`, `expect engine mode` and `expect engine step ... active|skip`.
- **Page.** 11 controls beyond the matrix, none an encoder (`apps/web/layout/panel.layout.json`, `status: provisional`). `octoweb-abi/2` already has `octoweb_input(now_ms, kind, control, detents)` with kind 2 an encoder turn, `octoweb_playheads` and `octoweb_status` (bit 0 running). `octoweb_dropped_intents` counts Audition and PLAY-snapshot intents the engine cannot yet act on (the 300 to 500 per run found in Friday's random-input probe).
- **Engine.** `MIN_BPM` 1 and `MAX_BPM` 999 (`engine.rs` lines 276 and 277). `Play` and `Continue` are one arm (line 677). `Stop` keeps position (`set_running`, line 528). The engine has no tempo command; the tempo comes with each render.
- **Not read for this plan:** the blog post `[B]` itself (only the digests are in the repo), so the tempo bar graph and "jump to fixed tempos" are not specified here; how the chase-light shares a key with a lit step (p014, p017 and p052 do not say; the Panelwright's request lists it as open); Live.

What those facts do to the plan:

- P5b and P5c need **nothing** from the engine, the contracts or D0.
- P5d needs the Metronome first, and is the only slice with a timing risk.
- P5e needs a numeric display, which the controller does not output (`tech.md` section 4 lists "the displays"; only the LED frame exists). That is a new output and a new read in the ABI (D-P5-5).
- P5f needs the playhead and the lookahead, both of which the page already has.

## 3. Decisions

Each is stated so a reviewer can reject it.

| # | Decision | Default | Alternative |
|---|---|---|---|
| D-P5-1 | What P5 contains | **The controller and page work of Wave 1 that the manual specifies:** the four step attributes by their knobs (P5b, P5c), the transport keys (P5d), the global tempo (P5e), the chase-light (P5f). **Save and load is not in P5**: it is High tier (the format is a contract) and gets its own plan after P5c. **The MIX encoders move to Wave 2** (the MIX row of the table in section 1) | Keep MIX in Wave 1 as a bare "turn writes a track attribute", which the manual does not describe |
| D-P5-2 | Order | **b, c, d, e, f**: a person can make a pattern (toggle, then shape) before the transport and tempo move off the page | Transport first, because the page's stopgap is the oldest debt |
| D-P5-3 | Where the controls go | The EDIT knobs in the order of `controls.json` in a row under the matrix, `status: provisional`, Q04 named in the layout file (as P3c did for the others). `amt` and `mcc` are drawn but do nothing until Q04 and their pages are fixtured | Draw only the four built |
| D-P5-4 | Stop and Play | **Ask the Metronome** (request filed first) for: `Play` after `Stop` re-aligns every track to the master clock (p086, the behaviour of the ALN button), `Continue` resumes where it was. The controller sends `Play` for Play and for Stop-then-Play, and `Continue` for Pause's second press | Leave the engine alone and let Pause equal Stop in the panel, with a documented gap against p086 |
| D-P5-5 | The numeric field | A **display output** beside the LED frame: a small struct (`numeric: Option<i32>` and what it is showing), read by the page through one new export, `octoweb_display()`. Adding an export does not bump the ABI (`apps/web/engine/ABI.md`: "A breaking change bumps the number"). The inner circle's pitch display is not in P5 | Show the tempo in the status strip only, and defer the display to Wave 2 |
| D-P5-6 | Tempo knob and the follower | In **Slave and Slave with echo** the main knob does nothing and the field shows the tempo the follower is applying. In **Off** and **Master** one detent is 1 BPM within the engine's accepted range (1 to 999); the manual states neither. The first tempo is the engine's 120 as today | A fixed list of tempos; or the knob writes through to the follower's lead |
| D-P5-7 | What Play does while playing, and Play while paused | **Nothing** in both cases, as `pending` fixtures (question numbers assigned when P5d lands; Q39 to Q42 were taken by P5b). The manual does not say | Restart; or Resume |
| D-P5-8 | Transport LEDs | None drawn (Q26 stays open). The strip's sentence already says what the transport is doing | Draw Play green while running as a provisional choice |
| D-P5-9 | The chase-light | Drawn from the playhead, delayed by the lookahead plus the snapshot's lead of about 13 ticks (`tech.md` section 5), **replacing** the step's colour on its key (Red, or Orange with an MCC) and passing over skipped steps (p014, p017, p052). How it overlays a lit key is Q-listed in the Panelwright's request, so the fixtures for that part are `pending` | Share the key with a ring; leave it out until the owner rules |
| D-P5-10 | Encoder input on a mouse | As `tech.md` section 5: drag and wheel become detents, no rotational physics. Added: arrow keys, one detent each. The pixels per detent and the wheel step are one constant each in the page | Wheel only |
| D-P5-11 | The ratchet | The panel fixtures and engine tests are floored in `harness/baseline.txt` as before; the Node and Chromium tests run in the `web` job. Each slice's commit states its fixture and test counts and its mutation run, in `handoffs/evidence/` | |
| D-P5-12 | Contracts | **None changed.** The ten EDIT controls and `main.enc` are already in `controls.json`. If P5b or P5e finds a missing control (the TEMPO key's role, the numeric field), the Panelwright files the request and the Conductor decides by ADR | |

## 4. The series

Each is its own pull request off `main`, none stacked, with tier and gate numbers in the commit. **P5b does not start until this plan is merged** (the lifecycle's tech-spec gate), and P5d's engine change does not start until the owner has answered D-P5-4.

| PR | What | Tier | Red first |
|---|---|---|---|
| **P5a** | This plan, the task record, the journal. Documents only | Medium | n/a |
| **P5b** | `octoface`: `StepView` carries the four attributes; `Turn` on `edit.enc.vel`, `pit`, `len`, `sta` in Step zoom writes `SetStep` and the row draws the value (W1). The fixture runner gains `given step ... = <int>` and `expect engine step ... <attr> = <int>`. Pending fixtures for what the manual leaves open: the value rows (Q06), acceleration (Q39), legato (Q40), a grabbed step in Page view (Q41) and where velocity and pitch show (Q42). GRV, POS, AMT, MCC and the quick keys of W11 get their own later slice | Medium | Fixtures first, failing on `main` |
| **P5c** | `apps/web`: the encoder element (drag, wheel, arrows), the layout cells, labels, the a11y traversal, the strip's sentence per turn (W2) | Medium | Node tests, then Chromium |
| **P5d** | The Metronome request; then, in order: `octocore` (Stop then Play re-aligns, Continue resumes; W4), `octoface` (`transport.play`, `transport.pause` roles; W3), `apps/web` (drop the page's `transport.play` mapping, the source rule). Pending fixtures for D-P5-7 | **High (timing)** | Engine test and conformance fixture first; any golden or fixture that stops and starts is listed first and re-derived in the diff |
| **P5e** | The display output and `octoweb_display()` (D-P5-5), the main knob in Page mode (W5), the follower interplay (D-P5-6) | Medium | Fixtures, then a Node test with the follower |
| **P5f** | The chase-light (W6) | Medium | Fixtures first; the pending ones carry their question |

**What stays out of P5:** save and load (its own plan), the MIX encoders, Track zoom and the track tempo multiplier and Pause, the triangle keys, the step LEN multiplier (W13), the quick keys of W11, GRV, POS, AMT, MCC, Step Shift (the DIR encoder), the pitch display, the "200" button, and any change to the tick (D0).

## 5. What can and cannot be shown here

Everything in W1 to W6 can be proved against the engine and the controller in this environment. Two things cannot:

- **A demonstration on real gear** (Wave 1's own exit, `product.md` section 5). It needs the owner's run, with S1 first, as P4 did.
- **Whether a mouse drag feels like a knob.** The constants in D-P5-10 are mine, set so a fixture can assert them, and the owner's to change.

## 6. Out of scope

Everything under "What stays out of P5", and D0.

## 7. Questions for the owner

1. **D-P5-1 and D-P5-2.** Is P5 the controller side of Wave 1, in the order b to f, with save and load planned on its own and the MIX encoders in Wave 2? If you want MIX in Wave 1 as the manual's selection edit (W15 to W17), I replan, and it needs the SEL stores first.
2. **D-P5-4 and F-P5-4.** May the Metronome change `Stop` then `Play` to re-align every track while `Continue` resumes? It is the manual's own wording (p086), it changes MIDI behaviour and the clock's `Start` against `Continue`, and it is the one High slice.
3. **D-P5-6.** Is "the knob does nothing while following a clock, one BPM a detent otherwise" acceptable until someone with an Octopus gives the tempo range and the bar graph (`tech.md` section 9, item 6)?
4. **D-P5-5.** Is a display output and one new export acceptable for the numeric field, or should the tempo be shown in the strip only until Wave 2?
5. **Still open, and not a blocker:** D0; Q21 (my default is two keys, because the page and `controls.json` already treat `mode.play` and `transport.play` as two controls); Q26; Q04; Q05 (my default is one knob with a role per mode); and the owner's runs of S1 and S3.

## 8. Risks

| Risk | Consequence | Mitigation |
|---|---|---|
| The Stop and Play change alters goldens or the clock stream | A determinism gate fails, or a follower test changes | The change is its own slice, any golden or fixture that stops and starts is listed first and re-derived in the diff, and the follower's Start and Continue tests are re-run |
| The knob fights the follower | Tempo jumps every pulse | D-P5-6, and a Node test with the follower in the loop that fails if the knob writes while following |
| A fixture asserts a guess | The ratchet keeps a wrong answer | Every fixture cites its page; where the manual is silent the fixture is `pending` with a question, never asserted |
| The display output grows into the pitch circle and the numeric entry (p052) | P5e never ends | D-P5-5 names one field; the inner circle is out of scope |
| Encoder turns flood the input path | The panel or worklet lags | `octoweb_input` takes detents, so a drag can be coalesced to one call per animation frame; the Chromium test measures the rate |
| A second `Role` for the transport keys changes the layout contract | A `controls.json` edit by accident | D-P5-12: no contract changes; a missing control is a request |

## 9. Follow-ups, not in P5

- The Panelwright numbers the transport questions (Play while playing, Play while paused, Pause with a track held) after Q42 when P5d lands.
- The Scribe corrects `product.md` section 4's tempo row, which cites p049 for the global tempo (F-P5-2).
- The plan for save and load, with `octopus-state/1` as a contract by ADR.
- The Referee floors the browser tests by name (still open from P3 and P4).

## 10. Approval record

**r1, 2026-10-03 (Europe/London).** The owner merged this plan (#39) at 19:52Z, 26 minutes after "Go" (Sat 20:26 London) and its last chat message "Check". **Nothing was written in answer to section 7.** The agent reads the merge as passing the tech-spec gate for **P5b only**, because the plan says P5b waits for the merge and needs nothing from the engine, the contracts or D0. It is not read as an answer to D-P5-1 to D-P5-12: P5c to P5f, and above all P5d's engine change, wait for the owner's reply to section 7. If the merge was meant as more or less than that, say so and this record is revised.

| Item | Read as | Not read as |
|---|---|---|
| The merge | The tech-spec gate for P5b: the four knobs in Step zoom, controller and fixtures only (#40 or its successor) | An answer to section 7, or approval of P5c to P5f |
| D-P5-1 and D-P5-2 | The defaults stand while P5b is built | Settled |
| D-P5-4 | Open | The Metronome change |

r1 also adds F-P5-7 and corrects W1 (P5b found that the matrix row for a value is not given by the manual).

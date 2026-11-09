# SPEC-0008 plan: the slices

Each slice is its own pull request, one hat, one tier, red first (`CLAUDE.md`: every taste judgement becomes a test). A slice starts when the one it needs has merged, and **no pull request is stacked on another** (the trap in
`.claude/skills/babysit/SKILL.md`: a base that is not `main`). Slices with no dependency on each other may run side by side. The agent merges nothing. **Nothing here starts until the owner has written their own words in
`README.md` section 12**: a merge is not an approval, and "continue" does not name a slice. `tech.md` is the how; this file is the order, the files, the tests and the end of each slice.

**How a slice runs.** The Forge writes the failing test first and watches it fail, then the code. The stand-in's additions (`tech.md` section 11) are written first, in the slice that needs them, each with the belief it encodes named in a comment. Where a slice
changes a decision of an ADR the Conductor writes the amendment first, before any code. The Referee reviews every **High** slice. Gate numbers go in every commit message: `cargo xtask verify --base origin/main` (today 4 pass, 0 fail, 6 not_implemented,
assertions 600 against a baseline of 600), `npm run typecheck`, and the number of tests `npm test` runs before and after. A browser test (`npm run test:browser`) is part of a slice that adds a control. A threshold stated in `README.md` section 5 is a proposal: the
first green run sets the number with a stated margin, and it is only tightened.

**Sizes** are rough estimates of lines changed (code plus tests), floors for planning and not limits, and I have not built any of it.

## The series at a glance

| Slice | Name | Rows | Hat | Tier | Needs | Rough size |
|---|---|---|---|---|---|---|
| H0 | The owner's measurement | B1, B2 | the owner | none | a synthesiser (a loopback if there is one) | no code |
| H0b | The follow report reads the real step | HW-28 | Forge | Medium | none; the owner's answer to Q-8-13 settles one line | 100 |
| H1a | The output stage's record and guard | HW-11, 13, 14 | Forge | Medium | none | 500 |
| H1b | Panic and the watchdogs | HW-12, 15, 16, 17 | Forge | Medium | H1a | 600 |
| H2 | The rig | HW-1 to 5, 26 | Forge | Medium | H1a | 900 |
| H3 | Timing | HW-6 to 10 | Forge, Conductor (ADR) | **High** | H2 | 800 |
| H4 | Clock to gear | HW-18, 19, 21 | Forge, Conductor (ADR) | **High** | H2; P0a if approved | 600 |
| H5 | The built-in sound steps aside | HW-27 | Forge | Medium | H2 | 250 |
| H6 | The input stage | HW-22 to 25 | Forge | Medium | H2 | 600 |
| H7 | The gear run | B3 to B8 | the owner | none | H3, H4 | no code |

**Order.** H0, H0b and H1a first, in parallel (no one needs another and none needs a decision, a contract or the gear list). Then H1b. Then H2. Then H3 and H4, which may run side by side after H2 (they touch the same file, so the second to start merges `main` first). Then H5 and H6 in either order, then H7.
**The smallest useful increment is H1a and H1b**: the page can no longer hang a synthesiser silently, and has a way out when something does. If the owner approves only one thing, that is the thing.

## H0. The owner's measurement

Hat: the owner. No code, no file but evidence. The procedure for the default setup is `bench.md` section H0 (what to plug in, what to open, what to write down, in order). It answers the first half of the roadmap's question, how loose a real port is, and gives criterion A3 a number.
**Stop condition.** If B1 shows the port too loose for the gear at any lookahead the owner will accept, stop and rescope (`docs/09-roadmap.md`, the kill criterion). Nothing after H1 is worth building on a port that cannot keep time.

## H0b. The follow report reads the real step

Hat: **Forge** (`apps/web/spikes/s3/`, `apps/web/src/abi.ts`, tests). Tier **Medium**. Detail: `tech.md` section 12. It is the answer to Q-8-13 made code, and it is independent of every other slice.

| # | Observable | Test |
|---|---|---|
| HW-28 | **The follow report reads the real step.** `STEP_TICKS` equals `DEFAULT_STEP_TICKS` in `crates/octocore/src/domain.rs`; the analysis's step at B BPM is `15000 / B` ms (125 at 120); its bound is printed as a fraction of a step **and** in ms at the run's tempo (25.00 ms at 120 BPM for a fifth); the share of samples within 6.25 ms is printed as information; no figure in the report is a literal that came from a step length | `test/s3-analyse.test.ts`, `test/s3-report.test.ts`, `test/abi.test.ts` |

Red first: (1) add the `STEP_TICKS` test against `domain.rs` (fails: no export); (2) correct the two pinned assertions of `s3-analyse.test.ts` (6.25 becomes 25) and their one-step comment, and add the "within 6.25 ms" share assertion (fail against `STEP_MS_AT_1_BPM = 3750`); (3) change the code.
**The two corrected assertions are corrections of arithmetic and not a loosened threshold**: the criterion was never a gate and the label "a fifth of a step" was wrong about the step. The pull request lists each old and new value, and the Referee checks that nothing under `harness/` moved.
Done when `npm test` and `typecheck` pass, `s3.browser.test.ts` passes with its comment corrected (it compares the median against `fifthOfStepMs`, a variable, so it needs no number change), and the report of a simulated run states the fifth in both forms.

## H1a. The output stage's record and guard

Hat: **Forge** (`apps/web/src/`, `apps/web/test/`). Tier **Medium**. Detail: `tech.md` sections 4 and 11. No rig, no persistence, no new control.

Files: `src/output-stage.ts` (new), `src/release.ts` (new, `releaseMessages` only), `src/midi-out.ts` (stages in place of bare outputs; `setOutput(port, output, id?)`; `try` around `onSend`), `pages/app.ts` (pass `device.id`), `test/support/receiver.ts` (`heldEventually`, CC 123 and CC 120, `throwOnSend`), `test/output-stage.test.ts` (new).

| # | Observable | Test |
|---|---|---|
| HW-11 | **The table is the truth.** After every step of a generated run (events, Stop, Start, a swap), each device's table equals `heldEventually()`, with `clear()` present and absent, with a retriggered pitch and a pitch bend and a sustain in the mix | property over `Session` |
| HW-13 | **A swap releases first.** Choosing another device for a port while notes sound sends the old device its Note Offs before it is dropped; a device lost with notes owed is released on return, before anything else reaches it | `output-stage.test.ts` |
| HW-14 | **A dead device is contained.** A stand-in whose `send` throws on every call: every other device receives every record of the batch, the thrower is marked failed once with one sentence, no exception escapes, the engine keeps running | `output-stage.test.ts` |

Red first, in this order: the stand-in's `heldEventually`; HW-11 against the current scheduler (it has no table: fails); HW-14 against the current scheduler (a throw aborts the batch: fails); HW-13 (no release on a swap: fails). Then the code.
**Done when** every existing test in `midi-out.test.ts`, `wire.test.ts`, `stop-property.test.ts` and `worklet.test.ts` passes **unchanged**, the three rows pass, and `stats` keeps every existing field and meaning.

## H1b. Panic and the watchdogs

Hat: **Forge**. Tier **Medium**. Needs H1a. Detail: `tech.md` section 5. The Conductor does not need an ADR: no decision of ADR-0008 changes (`clear()` is still never called; a second, stamped send is how the queue is outlasted).

Files: `src/release.ts` (`cutMessages`), `src/stall.ts` (new), `src/midi-out.ts` (`panic`, `releaseAll`, `testNote`), `pages/app.ts` (the Panic control, the test note control, `pagehide` and `freeze`, the watchdog's timer), the strip and CSS (text controls, keyboard reachable, a key that does not collide with the panel's: checked against `src/layout.ts`, D-H6),
`test/panic.test.ts` (new), a browser test for the controls.

| # | Observable | Test |
|---|---|---|
| HW-12 | **Panic is complete.** After any generated run and a Panic (Stop if the engine answers, then Release, then Cut), the stand-in holds nothing on any channel, including notes whose Note On is still queued, with `clear()` present and absent, and with a worklet stand-in that never answers | property over `Session` |
| HW-15 | **Closing is attempted.** `pagehide` and `freeze` handlers are installed and each calls `releaseAll`. That a message reaches a synthesiser is B5 | `panic.test.ts` with a stand-in `EventTarget`; bench |
| HW-16 | **A stalled engine is noticed.** A worklet stand-in that stops posting while running: within 250 ms the devices are released, `transport(false)` is posted, the sentence appears once; no automatic resume; a page timer that was itself asleep does not raise it | `panic.test.ts`, `StallWatchdog` with an injected clock |
| HW-17 | **The test note ends.** One Note On and its Note Off at the chosen velocity (default 64), the Note Off stamped by the scheduler and not by a page timer; never sent without a click | `panic.test.ts` |

Red first: HW-12 with the stand-in whose queue holds a Note On when the panic is pressed (fails against an unstamped single cut, which is the case F-H6 describes); then HW-16 and HW-15; then the controls' browser test.
**Done when** the rows pass, a Cut is shown to be at most 144 bytes beyond the Release in a test (the number the owner is told, `tech.md` section 5), the Panic control is reachable by keyboard in the browser test, and no colour, gradient or easing was added (the CSS and token tests, `app-css.test.ts` and `tokens-css.test.ts`, stay green, and the `slop` gate catches the rest once it is built).

## H2. The rig

Hat: **Forge**. Tier **Medium**. Needs H1a (the swap release). Detail: `tech.md` section 6. The rig file format stays out of `contracts/` (D-H3).

Files: `src/rig.ts`, `src/rig-bind.ts`, `src/rig-store.ts`, `src/router.ts` (new), `src/midi-out.ts` (resolve through the router), `src/midi-access.ts` (optional `manufacturer`), `pages/app.ts` and the strip (the rig sheet: devices, their routes, missing and ambiguous markers, export and import, the read-only track-to-device view with shared addresses), `test/support/receiver.ts` (`FakeMidiAccess`), tests `rig.test.ts`, `rig-bind.test.ts`, `router.test.ts`, a browser test for the sheet.

| # | Observable | Test |
|---|---|---|
| HW-1 | **The default rig is today.** Every existing scheduler test passes unchanged, and a generated stream through the router with the default rig equals the current scheduler's sends, bytes and timestamps | `midi-out.test.ts` unchanged; `router.test.ts` |
| HW-2 | **A split route delivers each event once.** Port 1 channels 1 to 8 to A and 9 to 16 to B with a rewrite: every event to exactly one device with the right channel nibble; overlapping routes refused with a sentence; a device with no routes accepted | `router.test.ts`, generated |
| HW-3 | **The rig round-trips.** `serialise(parse(text)) = text`; an unknown version refused with a sentence; an unknown field kept on export | `rig.test.ts` |
| HW-4 | **Binding is cautious.** By id; else by name and manufacturer if exactly one matches; two matches are `ambiguous` and never guessed; a missing device keeps its routes, receives nothing (counted, not thrown), and is bound again when it returns | `rig-bind.test.ts` with `FakeMidiAccess` |
| HW-5 | **Nothing goes out on load.** A rig read from storage or a file, a bind, and a re-bind send zero bytes until Play or a test button | `rig.test.ts`, `rig-bind.test.ts` |
| HW-26 | **Shared addresses are listed.** Tracks that share a port and channel are reported by a pure function | `rig.test.ts` |

Red first: HW-1 written as the equality, then the router. **Done when** the six rows pass, the two selects still work with no stored rig, and a rig can be exported from one run and imported in another with the same routes.

## H3. Timing (High)

Hat: **Forge**; the **Conductor** first writes the amendment to ADR-0008 (the next number: a per-device term beside the lookahead, and the effective lookahead) and the owner accepts it before the code. Tier **High**: `docs/09-roadmap.md` treats timing as such, as P4 was. The Referee reviews. Needs H2. Detail: `tech.md` section 7.

Files: `src/trim.ts`, `src/align.ts` (new), `src/midi-out.ts` (per-stage trim, `orderSlackMs`, effective lookahead), `src/protocol.ts` and `src/worklet.ts` (`alignClick`), `src/settings.ts` (`TRIM_MS = { default: 0, min: -50, max: 100 }`, `GUARD_MS`), `pages/app.ts` and the strip (the trim control, the align tool, the wire indicator), tests `trim.test.ts`, `align.test.ts`, a worklet test for `alignClick`.

| # | Observable | Test |
|---|---|---|
| HW-6 | **A trim moves its device and no other.** Trims {A: 0, B: +12}: B's stamps are A's minus 12 ms (within 0.001), its clock and Start move with its notes, rule 1 holds per device | `trim.test.ts`, generated |
| HW-7 | **A trim change never hangs a note.** Changing a trim between a Note On and its Note Off, at any point, in either direction and by up to 150 ms, leaves the Note Off after its Note On and the stand-in holding nothing | property |
| HW-8 | **The lookahead covers the trims.** For any trims the effective lookahead is at least the largest plus the 10 ms guard, the person is told when it was raised, and a generated run has no event late because of a trim | `trim.test.ts` |
| HW-9 | **Align is arithmetic.** Two stand-ins with sound delays of 7 and 19 ms, trimmed by the difference, sound within 0.5 ms of each other; the tool's click and note are for one instant | `align.test.ts`, worklet test |
| HW-10 | **Wire load is a number.** For a device the worst burst in a window is `bytes x 0.32` ms; a seven-note chord on one tick reads 6.72; shown, never gated | `trim.test.ts` |

Red first: HW-7 as a property before any trim exists (it fails because the setting does not), then HW-6, HW-8, HW-9, HW-10. **Done when** the five rows pass and the default rig's output is unchanged (HW-1 still passes). Nothing about the hearing of a real unit is claimed: B3 does that.

## H4. Clock to gear (High)

Hat: **Forge**; the **Conductor** first writes the amendment to ADR-0009 (the next number): decision 3 there says the page sends the real-time messages to every device it has chosen, and per-device flags change it. Tier **High** (the clock, as P4 was). The Referee reviews. Needs H2, and SPEC-0007's P0a if the owner approves that first. Detail: `tech.md` section 8.

Files: `src/song-position.ts`, `src/echo-guard.ts` (new), `src/protocol.ts`, `src/host.ts`, `src/wire.ts`, `src/worklet.ts` (the `songPosition` field), `src/midi-out.ts` (flags, the Song Position before a Continue, the echo filter), `pages/app.ts` and the strip (the flags in the rig sheet, the refusal sentence), `test/support/receiver.ts` (the position register), tests `song-position.test.ts`, `transport-flags.test.ts`, `echo-guard.test.ts`.

| # | Observable | Test |
|---|---|---|
| HW-18 | **Flags are honoured.** A device with `clock` off receives no `0xF8`; with `transport` off no Start, Continue or Stop; its notes are unaffected; the defaults are on | `transport-flags.test.ts` |
| HW-19 | **Continue carries the place.** A Continue away from zero is preceded, on each device with `transport` on, by a Song Position of `floor(tick / 12)` (low byte first); Start is not preceded; a stand-in with a position register is at the engine's place after Stop and Continue. **Red today** | `song-position.test.ts` over `Session` |
| HW-21 | **The echo guard.** An echo destination whose name equals the clock input's name (lower case, spaces removed) is refused with a sentence | `echo-guard.test.ts` |
| HW-20 | **Start alignment. Held** until Q-8-4 is answered; until then it is bench run B7 | bench |

Red first: HW-19 (a Continue carries nothing today), then HW-18, then HW-21. **Done when** the three rows pass, `wire.test.ts`, `worklet.test.ts` and `clock-path.test.ts` pass unchanged before the field is added, and the default behaviour with every flag on is today's.

## H5. The built-in sound steps aside

Hat: **Forge**. Tier **Medium**. Needs H2. Detail: `tech.md` section 10. It touches `monitor.ts`'s inputs and not its sound: the voices and drums are unchanged.

Files: `src/monitor.ts` (`NoteEvent.port`, `ports` mask, release on change), `src/protocol.ts`, `src/worklet.ts` (`monitorPorts`), `src/host.ts` (a setter), `pages/app.ts` and the strip ("also play here"), `test/monitor-ports.test.ts`.

| # | Observable | Test |
|---|---|---|
| HW-27 | **The sound steps aside.** With a device for port 1 and none for port 2 the monitor renders no sample for a port 1 note and renders one for port 2; with "also play here" both; the events are byte for byte the same either way; a mask change in the middle of a held note leaves no voice running | fake worklet test |

Red first: the rendered-sample assertions. **Done when** `monitor.test.ts` and `monitor-voice.test.ts` pass unchanged.

## H6. The input stage

Hat: **Forge**. Tier **Medium**. Needs H2. Detail: `tech.md` section 9. Record, transpose, force-to-scale and control maps are **not** here (D-H12): they are a later spec (`SPEC-0009`), which this slice gives a consumer list and `gridTickAt` to attach to.

Files: `src/midi-in.ts` (`addConsumer`, `consumerErrors`), `src/grid-clock.ts`, `src/thru.ts` (new), `src/midi-out.ts` (a send-now path through `deliver`), `pages/app.ts` (follower and echo moved to `addConsumer`), the rig sheet (the thru and offset settings), tests `input-consumers.test.ts`, `thru.test.ts`, `grid-clock.test.ts`.

| # | Observable | Test |
|---|---|---|
| HW-22 | **Fan-out is lossless and ordered.** Every decoded message reaches every consumer once, in order, with its timestamp; a throwing consumer does not stop the others; `midi-in.test.ts` passes unchanged | `input-consumers.test.ts` |
| HW-23 | **Thru is exact, and ends.** A note on the input channel arrives as the right Note On and Note Off on the chosen device and channel; if the input goes away or Panic is pressed the table releases it; the two stated limits are documented | `thru.test.ts` |
| HW-24 | **No thru into the same device.** An input routed to an output of the same name is refused unless confirmed | `thru.test.ts` |
| HW-25 | **`gridTickAt` is right.** For a played timestamp equal to the page time of tick `k` leaving the output, it returns `k` within 0.5 tick at several tempi and with a tempo change inside one panel interval; null when the transport was not running | `grid-clock.test.ts` with a stand-in clock map |

Red first: HW-22 (one slot today), HW-25. **Done when** the four rows pass.

## H7. The gear run

Hat: the owner. No code. `bench.md` B3 to B8 with the gear named, the "desk" and "rig" sessions, and B6(c), the follow run against Live, with the S3 report from H0b. Its result is the owner's statement, in their own words, of whether the page can be played on stage and of what it still cannot do.

## Done for the whole series

- Every row of `README.md` section 5 either passes in CI or is a bench run with a record in `handoffs/evidence/` that names the gear.
- The default rig's behaviour is still byte for byte today's (HW-1 never turned off).
- No colour, gradient or easing curve was added anywhere; the CSS and token tests are green.
- The engine, `engine/ABI.md`, the layout and `contracts/` are untouched, except for the ADR amendments of H3 and H4, which are the Conductor's.
- `harness/baseline.txt` only grew, and no threshold was loosened.
- `journal/STATE.md` names the series' state, and each slice has a journal entry that says what it changed and how it knew.

## How the owner approves, and what does not count

`README.md` section 12 is the record. An approval is the owner's own words, written there with the date, **naming SPEC-0008 and the slices**. For example, and only as a form: "Approve SPEC-0008 H1a and H1b" starts the two slices that need no decision; "Approve SPEC-0008 H0b" starts the report fix; "Approve SPEC-0008 H2 with D-H2 as written" starts the rig. I will not fill the record in on the owner's behalf from a message that does not name the spec.
Not approvals: a merge, a ready-for-review mark, "continue", "start", "keep pushing", a thumbs-up, or silence. "Defaults" on its own answers the questions with their defaults (`README.md` section 8) and starts no slice. Each of H3 and H4 additionally needs the owner to accept its ADR amendment before the code.

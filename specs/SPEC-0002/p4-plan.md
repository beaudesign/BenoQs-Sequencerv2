# SPEC-0002 P4 plan: two ports, the clock, and MIDI in

| Field | Value |
|---|---|
| Task | `WENGE-0015` (`handoffs/WENGE-0015.ndjson`) |
| Spec | SPEC-0002 r1, approved 2026-09-30 15:06 Paris. This plan fills in P4, and **narrows it in one place that the owner must accept or reject** (D-P4-1) |
| Plan revision | r0, written 2026-10-02 after the owner's "Yes" (10:00 Paris), on the morning after P3 merged (#24, #25, #26) |
| Stage | Tech spec, awaiting the owner. Nothing in P4 is built. No code, contract or threshold is changed by this pull request |
| Tier | P4 as a whole is **High (timing)** (`docs/09-roadmap.md`). Slice tiers are in section 4 |
| Hats | P4a Conductor. P4b Forge. P4c Metronome hat for the engine event, Forge for the page. P4d Forge. P4e Forge and Scribe |

## 1. What P4 is

`docs/09-roadmap.md` and the SPEC-0002 series table: *MIDI: two ports, clock out and in, input, program
change; the Tier 1 Ableton guide and test procedure (spike S3).* It is the step that turns the P3 page, which
plays one pattern to one output, into something that can sit in a rig: two outputs, a clock it can send and a
clock it can follow, and the way back in.

Two things in the approved text do not fit together, and a plan that ignored them would build the wrong thing.

- **F-P4-1. The roadmap lists MIDI work twice.** The P4 row has "input, program change". Wave 4 in the same
  document has "MIDI input and recording, keyboard transpose, force-to-scale, controller map learn, the '200'
  clock states, program change, ALL NOTES OFF". Wave 1 has "two MIDI outputs". The consumers of input are not in
  the engine (README F1: no record, no keyboard transpose, no force-to-scale, no map learn), and program change
  selects a page set (channel 10) or toggles a page in a bank (channels 1 to 9) [p094], and page sets and Grid
  mode are Wave 3. A P4 that built those would be four waves of engine work wearing one name.
- **F-P4-2. Wave 1's demonstration cannot be made as written.** A6 is "a pattern plays a real synth on port 1
  and a second device on port 2 at once", and `docs/09` makes it Wave 1's demonstration. A track reaches port 2
  by its `MidiChannel` being 17 to 32 (`resolve_port_channel`, `engine.rs`), and every track starts at 1.
  Nothing on the panel sets it: the manual puts the port in the track's attributes [p046], which is Track zoom,
  which is Wave 2. The page has no export that writes a track either (`apps/web/engine/ABI.md`, 15 exports,
  none is `SetTrack`). So today port 2 is reachable from a test and from nowhere a person can touch.

**How I will know it worked.** Each line is a command with an expected result, written red first where code is
involved. Section 4 says which slice owns each.

| # | Observable | How it is checked |
|---|---|---|
| M1 | Events for port 1 go to the output chosen for port 1, and events for port 2 to the output chosen for port 2, each with its own ordering rule; an event for a port with no output is counted and not sent | A Node test over the real scheduler with two recording outputs, then a Chromium test in the built app with two fake outputs |
| M2 | With the clock set to master and the transport running, each output chosen receives `0xF8` 24 times per quarter note, `0xFA` at Play, `0xFC` at Stop and `0xFB` at Continue, and the pulses are stamped like the notes (the same lookahead, never earlier than the one before) | A native test of the engine's clock events at several tempos and sample rates (exact positions), then the Node and Chromium tests over the scheduler and the page |
| M3 | With the clock set to master, the pulse and the step grid agree: the first pulse after Play is at the same sample as the first step, and a step is a whole number of pulses | A native test. **Passes only when D0 is answered** (section 2): today a step is 12 ticks and a pulse is 8, so a step is 1.5 pulses where a sixteenth note is 6, and the test is written now and marked `pending` with the reason, which is how the Panelwright treats an unanswered manual question |
| M4 | The follower turns a simulated clock into a tempo and a phase: steady 120 BPM with up to 3 ms of jitter in, a tempo within 0.1 BPM and a phase error within one tick out, after 2 seconds; a tempo ramp is followed; a missing pulse and a duplicated pulse do not make it run away | A pure function with its own fixtures, in the style of the O4 null-host fixtures. The numbers are mine and are the owner's to change |
| M5 | `0xFA` starts the transport at the pulse, `0xFC` stops it, `0xFB` continues it; the page does not start on a pulse alone; with no clock for 500 ms the app says so and keeps playing at the last tempo | The same Node tests, with a fake `MIDIInput` that delivers bytes with timestamps |
| M6 | Input messages from the chosen port reach the page's input path with their timestamps and are decoded (note on, note off, controller, bend, pressure, program change, realtime); nothing consumes them yet, and they are counted | A Node test with a fake `MIDIInput`; a Chromium test of the settings view |
| M7 | The settings view lets the owner choose the output for port 1, the output for port 2, the input, and the clock state (off, master, slave, slave with echo), says what each is doing in a sentence, and is usable by keyboard with the manual's own words as labels (A7, A8) | Chromium tests, red first |
| M8 | In a browser without Web MIDI, or with an input refused, the app runs and says so (A7), and a clock state of master or slave is refused with a sentence rather than shown as working | A Chromium test with `requestMIDIAccess` removed and with it refusing |
| M9 | Live, set up as `ABLETON.md` says, follows the app's clock and the app follows Live's, and the run is recorded with the numbers | **Spike S3, run by the owner** (section 5). The harness and the written procedure are mine; the run needs Live and a virtual port |
| M10 | `cargo xtask verify --base origin/main` passes and the baseline only grew | CI `verify` |

## 2. What was measured before planning

All from the code on `main` at `8d7ee25`; nothing here is a browser measurement.

- **The scheduler already routes two ports.** `MidiScheduler` keeps a map from port to output
  (`apps/web/src/midi-out.ts`, `setOutput(port, output)`), rule 1 (timestamps on one output never go backwards) is per
  output, and the engine's events carry `port` 1 or 2. The page wires port 1 only (`pages/app.ts`). The second
  output is a settings-and-wiring job, not a scheduling job.
- **There is no clock output.** `octocore::Event` has five variants: note on, note off, controller, bend, pressure
  (`crates/octocore/src/types.rs`). `Command::HostTransport { ppqn_pos, bpm, playing }` exists and the engine acts
  on `playing` only (`engine.rs` line 546); the position and tempo in it are not read. The engine's `Continue`
  command exists. `Command::Stop` flushes (a Note Off for each sounding note, then CC 123 on each channel that
  owes one), and Stop pressed while already stopped sends CC 123 on all 32 channels **unconditionally**: the manual says
  "when defined as MIDI master or slave" [p094], and the engine has no such setting, so it treats the condition as
  always true (`engine.rs` line 524, `tests/conformance/AMBIGUITIES.md`, "stop with sounding notes").
- **There is no input path.** `octoweb`'s exports take panel inputs (press, release, encoder), a transport call and a
  tempo. MIDI in appears nowhere in `crates/` or `apps/web/` (README F1, and a search).
- **The tick and D0.** `TICKS_PER_QUARTER` is 192 (`domain.rs` line 33), a step is 12 ticks, and the engine counts
  `sample_rate * 60 / (bpm * 192)` samples a tick (`engine.rs` line 274). At 120 BPM that is a step every 31.25 ms; a sixteenth
  note at 120 BPM is 125 ms. The manual reads as 192 per whole note, which would make the step 125 ms
  (D0, `WENGE-0012`, still open). MIDI clock is 24 pulses to the quarter note, which is a pulse every 20.83 ms at
  120 BPM, so **today a step is 1.5 pulses (12 ticks at 8 to a pulse), not the 6 that a sixteenth note is**, and a
  clock that is correct as MIDI will sit under a pattern that runs four times too fast for it. This does not stop the work. It
  stops the *claim*: no run against Live or a drum machine can be judged "in time" until D0 is answered, because
  the thing being compared with Live's grid is wrong by a factor the owner has not yet chosen.
- **The second port is reachable only from tests** (F-P4-2 above).
- **Not read for this plan:** the W3C Web MIDI text on what the `timeStamp` of an incoming message is and
  whether real-time messages reach a page without the SysEx permission. `tech.md` cites [W3C] for output
  timestamps only. P4b's first act is to read it and to measure it on a real port, and the plan does not rely on either.

What those facts do to the plan:

- The second port, the settings view and the input path (P4b) need nothing from the engine and nothing from D0.
- Clock output needs the engine to say where the pulses are (D-P4-3), and its proof (M3) waits on D0.
- Clock input is a pure estimator plus a small amount of page logic (D-P4-4) and can be proved against simulated
  clocks without a browser; what it achieves against Live is what S3 measures.
- The Ableton guide can be written now and its numbers filled by the owner's run.

## 3. Decisions

Each is stated so a reviewer can reject it.

| # | Decision | Default | Alternative |
|---|---|---|---|
| D-P4-1 | What P4 contains | **Two ports, the clock out and in, start/stop/continue, the input path with no consumer, the settings view, the Ableton guide, spike S3.** Input *consumers* (recording, keyboard transpose, force-to-scale, map learn) stay in Wave 4 where `docs/09` puts them; **program change moves to follow page sets (Wave 3)** and is decoded and counted in P4 and not acted on | P4 exactly as the series row reads: it would carry the engine work for record, page sets and Grid mode, and is a different size of job. Say so and I replan |
| D-P4-2 | How a track reaches port 2 before Track zoom exists | A **stand-in in the page**, not a product control: a `?route=` query string read once at start (`?route=3:17,4:18` is track 3 to port 2 channel 1, track 4 to port 2 channel 2), written through one new export, `octoweb_set_track`. It is documented as a test and demonstration hook in `apps/web/README.md`, is not drawn on the panel, and is removed when Track zoom exists | Move A6 to the end of Wave 2 and ship no hook; or a settings row "track to port" (a control the hardware does not have, and the Panelwright does not invent controls) |
| D-P4-3 | Where the clock pulses come from | **The engine emits them**, as new `Event` variants (`Clock`, `Start`, `Continue`, `Stop`) with a sample offset, at every `TICKS_PER_QUARTER / 24` ticks while the clock state is master. One constant, `TICKS_PER_CLOCK`, so D0 changes one line, and a native test pins it (M3). This is a change to `crates/octocore`, so it is its own pull request under the Metronome hat, High tier, with a request filed to the Metronome first | `octoweb` derives pulses from the tempo and the sample clock, with no change to `octocore`. Smaller, and the pulse can disagree with the step grid when the tempo changes mid-bar, which is the one thing a drum machine hears. A page timer is rejected: it is what S1 exists to avoid |
| D-P4-4 | How the app follows a clock | **Tempo and phase follow, on the page.** A pure estimator turns the timestamps of incoming `0xF8` into a tempo and a phase; the page sets the engine's tempo (`octoweb_set_tempo`) and starts and stops the transport on `0xFA`, `0xFB`, `0xFC`; the engine's own sample clock still times every tick. The engine runs a phase *lead* equal to the lookahead `L` plus a user offset (default 0 ms), so the notes it sends land on Live's grid and not `L` after it | **The engine ticks on pulses** (host lock, the old O4 step 4b): exact, and a larger engine change that needs D0 first. Revisit it if S3 says the first design cannot hold the phase |
| D-P4-5 | The clock states in the app | A **settings control with four states** (off, master, slave, slave with echo) and one sentence of status. The "200" button workflow stays with the Panelwright: the manual's prose, its diagram and the release notes disagree about it (`tech.md` section 9, item 3) and the plan does not guess | Draw the "200" button now with the prose's reading and mark it provisional |
| D-P4-6 | Port priority and the overload LEDs | Not in P4. Port priority is a Device View item (Wave 5); the pipe-full LEDs (CHORD1, CHORD2 red [p102]) need the Octopus's own pipe model, which a browser does not have | |
| D-P4-7 | Stop while stopped | **Unchanged.** The engine already sends ALL NOTES OFF on 32 channels whatever the clock state, which is the safe side of the manual's "when defined as master or slave" [p094]. P4c adds the clock state and leaves this alone; making it conditional would change a tested behaviour for no gain a rig can hear, and the divergence stays on record in `tests/conformance/AMBIGUITIES.md` until the "200" workflow (Wave 4) | Make it conditional in P4c, and update the tests that assume it |
| D-P4-8 | Echo | Slave with echo passes incoming clock, start, stop and continue to the outputs chosen, unmodified and stamped with the same lookahead; the pass-through is separate from the engine's own events (an output never gets both a pass-through pulse and an engine pulse, because master and slave are exclusive) | Do not offer echo until the follower is proved |
| D-P4-9 | Input and the SysEx permission | The page asks for Web MIDI without SysEx, as P3 does. Realtime messages are not SysEx, and what a browser delivers without the permission is read and measured in P4b before anything relies on it. If a browser needs the permission for clock, that is a finding and the plan stops there | Ask for SysEx always (a second prompt for a feature v1 does not have; `product.md` section 6 says SysEx is out of v1) |
| D-P4-10 | Where the clock-follow numbers come from | M4's thresholds are mine and proposed. A4 is *set by spike S3* (`product.md` A4) and the owner sets it from S3's numbers | |
| D-P4-11 | The browser tests and the ratchet | As D-P3-8: the engine's native tests (clock events, the constant, the exports) are floored in `harness/baseline.txt`; the Node and Chromium tests run in the `web` job. The Referee request from P3 is still open and is extended with P4's counts | |
| D-P4-12 | ABI | `octoweb-abi/2`: adds `octoweb_set_track`, `octoweb_set_clock` (the state) and the clock event kinds 5 to 8 in the event record. Existing exports and layouts do not change, and the number is bumped because the event kinds are new (ADR-0008, "a breaking change bumps the number"). A short **ADR-0009** records D-P4-3, D-P4-4 and D-P4-12, written in P4a *after* the owner answers; this plan does not write it first | A minor note in ABI.md without a number bump (the number is for breaking changes, and new kinds a page does not know would be ignored) |

## 4. The series

Each is its own pull request off `main`, none stacked, with tier and gate numbers in the commit. **P4a does not
start until the owner has answered D-P4-1 to D-P4-5, and P4c's acceptance (M3) waits for D0.**

| PR | What | Tier | Red first |
|---|---|---|---|
| **P4a** | ADR-0009 (D-P4-3, D-P4-4, D-P4-12), the Metronome request, `ABLETON.md` skeleton. Documents only | High | n/a |
| **P4b** | Two ports in the page and the settings view (M1, M6, M7, M8); the input path to the page, decoded and counted, no consumer; `octoweb_set_track` and the `?route=` stand-in (D-P4-2). No change to `octocore` | Medium | Node: scheduler with two outputs, input decoder, settings state. Chromium: the settings view. Native: `set_track` writes the track |
| **P4c** | `octocore`: `Event::{Clock, Start, Continue, Stop}`, the clock state, `TICKS_PER_CLOCK`, Stop-while-stopped panic (M2, M3, D-P4-7). `octoweb` encodes the new kinds; the scheduler sends them. `octoffi` is frozen, so any match on `Event` there is touched only to keep it compiling, and the PR says so | High | Native: exact pulse positions at 40, 120, 240 BPM and 44.1, 48 kHz; Start at the first pulse; the goldens unchanged with the clock off; M3 `pending` until D0 |
| **P4d** | The clock follower (M4, M5, D-P4-4, D-P4-8): the estimator, the page's start/stop/continue handling, echo, the lead and offset | High | Node fixtures first: jittery, ramping, dropped and doubled pulses, a stalled clock; then the page tests with a fake input |
| **P4e** | `ABLETON.md` (routing, the two preferences, the test procedure), the S3 harness in `apps/web/spikes/s3/` (a page that follows a clock and logs the phase, and one that sends a clock and logs what returns), and the worked numbers from here | Medium | The harness against a simulated clock in CI |

**What stays out of P4:** recording, keyboard transpose, force-to-scale, map learn, program change's effect,
SysEx, port priority, the pipe-full LEDs, the "200" button workflow, Max for Live (S4), Link, a tempo control (the
panel's tempo encoder is Wave 1) and any change to the tick (D0).

## 5. Spike S3, and what can and cannot be run here

S3 (`tech.md` section 8): *Live master at 120 BPM over IAC for 30 minutes; estimate tempo and phase.* It needs Live and
a virtual port, so **it is the owner's run**, as S1's real-port half is. What I can do here, as in P3:

- The page that follows a clock and logs `(pulse timestamp, estimated phase error)` and the page that sends a clock
  and logs what comes back at a loopback input, both runnable by the owner with a click.
- The estimator against simulated clocks with jitter shapes I choose, which shows the arithmetic is right and
  says nothing about what Chrome and the OS do to a real stream. Every record says so on its first line, as the S1
  records do.

**The kill criterion for this slice** (proposed): if S3 shows that the follower cannot hold a phase within a
fifth of a step (about 6 ms at 120 BPM) over 30 minutes with the tab visible, the plan reopens D-P4-4 and
replans around host lock. The owner can change the number.

## 6. Out of scope

Everything under "What stays out of P4" in section 4, and D0.

## 7. Questions for the owner

1. **D0, the tick.** Answer it before P4c is accepted. Until then the plan builds the clock output and proves
   everything except that the pulse and the step agree (M3 stays `pending`). It also stops P4e's "in time" claim.
2. **D-P4-1.** Is P4 the transport layer (ports, clock, input path), with recording and program change
   waiting for the waves that own their engine work? If you want the full P4 as the series row reads, I replan, and
   it is no longer one wave of the work.
3. **D-P4-2 and F-P4-2.** A6 cannot be shown until a track can be put on port 2. Is a query-string stand-in
   acceptable until Track zoom, or should A6 move to the end of Wave 2?
4. **Run S1, then S3.** S1's real-port half is still yours and the kill criterion in `docs/09` is still untested.
   A clock that follows Live is only worth building well if Web MIDI's own jitter is small enough, so S1
   comes first.
5. **D-P4-3.** Are you content for P4c to change `crates/octocore` (a new event variant and the clock constant), a
   frozen-layout enum? It is the Metronome's zone: I would file the request first and wear the hat in its own pull request.

## 8. Risks

| Risk | Consequence | Mitigation |
|---|---|---|
| D0 is answered after P4c is built | The pulse and the step disagree by a constant | `TICKS_PER_CLOCK` is one line; M3 is written now and `pending` with the reason; nothing in P4 claims "in time" before it passes |
| Web MIDI input timestamps are coarse or batched on some platforms | The estimator sees jitter that is not the sender's | The estimator is built to tolerate it (M4 uses 3 ms of simulated jitter); S3 measures the real stream; D-P4-4 names the fallback |
| Chrome needs the SysEx permission for clock on some platform | A second permission prompt for a v1 non-feature | D-P4-9: read and measure before relying; stop and report if so |
| A new `Event` variant breaks a `match` in a frozen crate | A build fails far from the change | P4c's first test builds the whole workspace; `octoffi` is touched only to compile and the PR says so |
| The `?route=` stand-in becomes a feature by habit | A control the hardware does not have | It is not drawn, it is documented as a hook, and the Referee's source rules get a test that it is read only in `app.ts` |
| Live's own clock and Link settings differ from `tech.md` section 6 (that article was not read in full) | The guide is wrong on a menu name | The guide is a written procedure the owner follows once, and S3's run corrects it; the first line says which version of Live it was written against, or that none was available |
| The follower and the lookahead interact | A constant offset against Live's grid | D-P4-4's lead equals `L` plus a user offset; the offset's purpose is exactly this and it is shown in the settings view |
| The scope grows into recording | P4 never ends | D-P4-1, and `docs/09`'s "the one thing to protect": narrow and finished, then wide |

## 9. Follow-ups, not in P4

- The Scribe corrects `docs/03` (MIDI in and the clock, "after P2 and P4" per `removal-plan.md`) and `docs/07` once P4c and P4d land.
- The Panelwright builds the "200" workflow when the owner or a person with an Octopus settles `tech.md` section 9, item 3.
- The Referee floors the browser tests by name (the P3 request, extended with P4's counts).

## 10. Approval record

**r1, 2026-10-02, 10:54 (Europe/Paris).** The owner had been asked the five questions in section 7 (pull request #27, opened
at about 10:14 Paris). The owner replied **"Accept p4"**. The agent reads that as *accept this plan with its defaults*: it
names no change to any decision, and it is the same shape as "Approve O4 plan, all defaults" and "Approve" earlier in
this series. Chat is not a source of truth, so the reading is written down here. If it is wrong, say so and this record is
revised. The pull request was **not merged** when this was written.

| Item | What is approved | What is not |
|---|---|---|
| D-P4-1 scope | P4 is the transport layer: two ports, the clock out and in, start/stop/continue, the input path with no consumer, the settings view, the Ableton guide and spike S3. Recording, keyboard transpose, force-to-scale, map learn and the effect of program change stay in their waves | Any of those consumers in P4 |
| D-P4-2 port 2 | The `?route=` stand-in, read once at start, documented as a hook, not drawn, removed when Track zoom exists (F-P4-2) | A settings control that routes tracks; moving A6 |
| D-P4-3 the clock events | The engine emits them (`Event` variants and `TICKS_PER_CLOCK` in `crates/octocore`), as its own pull request under the Metronome hat with a request filed first | A page timer; derivation in `octoweb` |
| D-P4-4, D-P4-8 the follower | Tempo and phase follow on the page, a phase lead equal to the lookahead `L` plus a user offset (default 0 ms), echo as a separate pass-through | Host lock (the engine ticking on pulses) unless S3 forces it |
| D-P4-5 | Four clock states in the settings view; the "200" button workflow stays with the Panelwright | Drawing the "200" button |
| D-P4-6, D-P4-7 | Port priority and the pipe-full LEDs not in P4; Stop while stopped unchanged (unconditional ALL NOTES OFF) | Making it conditional |
| D-P4-9 | Web MIDI without SysEx; what a browser delivers without it is measured before anything relies on it | Asking for SysEx |
| D-P4-10 to D-P4-12 | M4's thresholds are proposed and the owner's to change; A4 is set from S3; the baseline floors the native tests; ABI 2 and ADR-0009 | |
| Series | P4a to P4e as in section 4. Because each slice builds on the one before (the ADR, then the page, then the engine event that the page sends, then the follower, then the guide), the slices are **stacked**, each pull request carries its predecessors' commits, and each says "merge in order" the way P3c did. Section 4's "none stacked" is revised by this record | A slice merged out of order |

**Not answered by this reply, and still open.** **D0, the tick length** (question 1): P4c is built with M3 `pending`, and nothing in P4
claims "in time" before D0 is answered. **Question 4**: the owner's runs of S1 and then S3 on real ports. **Merging**: the agent
merges nothing.

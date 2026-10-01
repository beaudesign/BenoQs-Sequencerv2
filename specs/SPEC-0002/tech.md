# SPEC-0002 tech spec: the web sequencer

Draft for the owner. Nothing here is built. Claims about browsers, Ableton and Max carry their
source in section 10; anything not confirmed by a source or a run is marked **spike** and has a
pass criterion in section 8.

## 1. What exists and what is missing

| Piece | State |
|---|---|
| Sequencing engine `octocore` | Real, conformance-tested against the manual, deterministic, no allocation after init on the render path, compiles to WebAssembly |
| WebAssembly build | `just wasm-smoke` builds `octoffi` for `wasm32-unknown-unknown` and plays the five golden patterns in Node, byte-identical to native (CI job `wasm-smoke`) |
| Engine interface | `Command` in, `Event { NoteOn, NoteOff, Cc, PitchBend, ChannelPressure }` out with a sample offset, `Snapshot` (LEDs, encoders, playheads, transport, mode) |
| Headless runner `octorun` | Pattern in, event log and Standard MIDI File out |
| Ratchet, gates, CI | Real (`cargo xtask verify`) |
| **Front panel** | **Missing.** `ButtonDown`, `ButtonUp`, `EncoderTurn` are no-ops; nothing writes LEDs |
| **MIDI in, clock, program change, control maps** | **Missing** (code search) |
| **Web app, Web MIDI layer** | **Missing** |

## 2. Architecture

```
 Browser tab (HTTPS, or localhost)
 +---------------------------------------------------------------+
 |  UI  (TypeScript, SVG)                                        |
 |    pointer, wheel, keyboard --> input events                  |
 |    <-- LED frames, displays (about 60 per second)             |
 |                                                               |
 |  MIDI layer (main thread)                                     |
 |    out: MIDIOutput.send(bytes, timestamp)   port 1, port 2   |
 |    in : MIDIInput messages --> input events, clock            |
 |         ^ events with audio-clock times     | MIDI in         |
 |         |                                   v                 |
 |  AudioWorklet  (the clock; silent output)                     |
 |    octoface (panel controller, Rust/WASM)                     |
 |        --> octocore (engine, Rust/WASM)                       |
 |    process(): every 128 frames, render events                 |
 +---------------------------------------------------------------+
        |  Web MIDI                       ^  MIDI clock, notes
        v                                 |
   external instruments             Ableton Live (virtual port)
```

Decisions inside this picture, each stated so a reviewer can reject it:

- **One thread for the engine, no shared memory.** The engine and the panel controller run in
  one AudioWorklet and talk to the page with `postMessage`. That avoids `SharedArrayBuffer`
  and the cross-origin isolation headers it needs, which static hosts often cannot send. It
  is why the lock-free ring of O6 is not needed here (README D5).
- **The audio clock is the clock.** Timers on the main thread are throttled in background
  tabs and jittery; the audio thread is neither (**spike S1 and S2 verify this in a hidden
  tab for 30 minutes**). The worklet's output is silent, but the node must be connected to
  the destination or the browser may not run it (**S2**).
- **TypeScript, strict, no UI framework.** A few hundred SVG elements and one frame per
  update do not need one. Simple, explicit code is what a smaller model can extend safely.
- **Static files.** No backend, no account. The app can be a PWA and work offline.
- **How the page calls Rust** (the `octoffi`-style C exports the smoke test uses today, or
  `wasm-bindgen`) is decided in P3; it does not change the design.

## 3. Timing and the clock

The engine already renders in blocks and stamps each event with a sample offset. The browser
side turns that into a MIDI timestamp:

1. Each `process()` call renders 128 frames and returns events with audio-clock times.
2. The page converts audio time to page time with `AudioContext.getOutputTimestamp()`, which
   gives a pair of the audio context time and `performance.now()` time.
3. It sends each event with `MIDIOutput.send(data, timestamp)`, at least a lookahead `L` early,
   so the timestamp is in the future. A timestamp of zero or in the past means "now", and
   several sends with one timestamp keep their order [W3C].
4. On Stop it sends the engine's flush (a Note Off for each sounding note, and ALL NOTES OFF) [p094],
   stamped after anything already queued. It does not call `MIDIOutput.clear()`: Chromium has none, and
   where it exists it drops Note Offs that were already counted as sent. Changed by ADR-0008
   amendment 1 (2026-10-01); the first approved text called `clear()` here.

`L` is a tunable, default 30 ms, set by spike S1. It costs command latency, as the engine's
existing 12-tick lookahead does; the two add and must be reported together.

**Tempo and the tick.** The engine counts 192 ticks per quarter note, and the manual reads as
192 per whole note (D0, `WENGE-0012`, still open). MIDI clock is 24 pulses per quarter note, so
the ratio of engine ticks to clock pulses is 8 today and 2 if the tick is fixed. The slave clock
and the master clock cannot be designed finally until D0 is answered; the master output can be
built first because it divides a known tick.

**Clock follow.** In slave mode the app estimates tempo and phase from incoming `0xF8` pulses,
starts on `0xFA` and stops on `0xFC`. The estimator is a small pure function with its own
fixtures (a simulated jittery clock in, a steady tempo out), in the null-host style of O4
step 1. Live sends MIDI Clock only, and can add Song Position [Ableton].

**What carries over from O4.** The null-host guards and the integer step clock (#14, #16) apply
unchanged. Steps 4b and 4c were a plugin host lock and tempo ramps; the web equivalent is the
clock follow above, and they are re-planned after D0.

## 4. The panel controller (`crates/octoface`)

A pure state machine. It is the missing product (README F1).

**Inputs.** Button down and up, encoder turns, and time. Time is a millisecond count passed in
by the caller, so a fixture can replay a session exactly. Engine snapshots come back as input.

**Outputs.** Engine commands, an LED frame, and the displays (the numeric circle and the pitch
circle).

**LED frame.** Each LED is `{ colour: Off | Red | Green | Orange, phase: Steady | Flash }`. The
UI maps colours to hues through a colourway, so the alternate colourways in the manual [p004]
are one table. "Shine", which the manual uses and never defines [p014, p021, p033], is one of
the open questions in section 9.

**Control inventory.** A new contract, `contracts/controls.json`, written by the Conductor by
ADR from the manual and the blog: every control with an id, a kind (button with LED, encoder,
display), a zone (matrix, selector column, mutator column, MIX, EDIT, circle, chord block,
transport, mode, spiral), the manual's own name, and a **logical** grid position for layout.
It carries no millimetres. It replaces `panel.truth.json` for this product and gives
`ControlId` its meaning.

**Fixtures.** Every workflow in the digests becomes one or more fixtures, cited to its page. A
fixture is a timed list of inputs and the expected outputs, in the style of the existing
fixture DSL (`crates/octocore/src/fixture.rs`); whether it extends that DSL or sits beside it
is decided in P2. Sketch:

```
fixture "page mode: a click toggles a step"          # p013, p014
  page 1 bank 1, stopped
  at   0 ms  press   step(track 3, step 5)
  at  40 ms  release step(track 3, step 5)
  expect led step(3, 5) = green steady
  expect engine track 3 step 5 active = true

fixture "EDIT cycle: one click is preview, two is perform"   # p068, p069
  at 0 ms click edit
  expect led edit = orange flash
  at 400 ms click edit
  expect led edit = green flash
```

Where the manual is unclear or contradicts itself (the digests list about 40 places), the
fixture is added as `pending` with the ambiguity written next to it, as
`tests/conformance/AMBIGUITIES.md` already does for the engine. A pending fixture counts in the
ratchet as a known gap, and it is closed by a hardware check or the owner's ruling, never by
guessing.

**Why this suits smaller models.** A task is one workflow: read its pages, write the fixture
so it fails, make it pass, run the gates. The fixture is the specification, and the ratchet
stops the next task from breaking the last.

## 5. The web UI

- **Layout.** Fixed aspect, scales like an object and never reflows, as the old north star
  said and as still holds. Positions come from `controls.json`.
- **Drawing.** SVG. LEDs are the only saturated colour on the page; everything else is
  neutral, and emphasis uses contrast, weight and space. This is decision D3's default.
- **Input.** Pointer Events with pointer capture. An encoder turns by drag or wheel into
  detents (no rotational physics: that belonged to the photoreal plan).
- **Chords on a mouse.** The manual is built on hold-and-press. A mouse presses one thing at a
  time. Proposal: Shift-click, or a long press, **latches** a hold, shown by the held
  control's LED flashing; a second click releases it. Touch screens and multiple pointers get
  real chords. Keyboard shortcuts give hold keys. This is an open design question (section 9).
- **The chase-light and the snapshot.** The engine's snapshot leads the sound by about 13
  ticks (found in the O6 review). The UI delays the chase-light by the same amount plus the
  MIDI lookahead so the light and the sound agree.
- **Accessibility.** Every control is a labelled element with the manual's name, reachable in
  layout order; focus is visible. A blind user is served by the same names.
- **Settings.** One plain view: which Web MIDI output is port 1 and which is port 2, which input
  listens, clock state, lookahead, colourway, start mode and port priority (the Device View
  items [p099]).

## 6. Ableton

Tier 1 needs no code on Live's side. Everything below is from the sources in section 10.

**Tier 1, routing.** macOS: turn on the IAC Driver in Audio MIDI Setup. Windows: install
loopMIDI. Live then lists the port and can use it for notes, CC and sync [Ableton bus]. In
Live's Link, Tempo and MIDI preferences, turn on Track for the *input* port so Live accepts
notes from the app (Live's MIDI settings article, [Ableton MIDI], was not read in full for
this draft), turn on Sync on the *output* port to send Live's clock to the app, and press
Ext on Live's transport only to make Live a *slave*. Live sends MIDI Clock, as Song or Pattern
mode, and can correct a fixed delay with its MIDI Clock Sync Delay setting [Ableton sync].
The web page needs Chrome, Edge or Opera, a secure context and one permission prompt.

**Tier 2, inside Live.** `jweb` is a Chromium Embedded Framework view in Max, available in Max
for Live from Max 8.1 and Live 10.1.2. Frozen devices cannot load HTML files from inside
themselves, so the page must be served from a URL or injected as script at load
[h1data]. `jweb` and Max exchange messages with `window.max.outlet` and `bindInlet`, and Max
handles the outlet messages asynchronously on its low-priority queue [Max]. Max's documentation
does not say whether `jweb` exposes Web MIDI. The design therefore does not rely on it: the
page sends timestamped events to Max, and the device schedules them and writes MIDI to the
track. Whether that is tight enough is spike S4. The original WENGE plan rejected this route
because v1 tried to render the whole instrument there; here the instrument is a plain SVG page
and the engine is WebAssembly, which is a different bet and needs its own evidence.

**Tier 3.** Ableton Link has no browser API, so it needs a local helper that speaks Link and
the page over a socket. A VST3 or AU with an embedded web view is possible. Neither is
scoped.

## 7. Saving

- A versioned JSON state, `octopus-state/1`, with a schema in `contracts/`, holding the grid
  (9 banks of 16 pages), the global settings and the clock state.
- Autosave to IndexedDB; export and import as a file.
- Standard MIDI File export of what a page plays, reusing the writer in `octorun`.
- Octopus SysEx dumps are not supported in v1: the manual states sizes and procedure and not
  the byte format [p096, p097].

## 8. Spikes

Each spike is a small piece of code and a recorded run in `handoffs/evidence/`, before the
wave that needs it. Pass criteria marked *proposed* are mine and are the owner's to change.

| # | Question | Method | Pass |
|---|---|---|---|
| S1 | How steady is Web MIDI output, from a worklet clock, to a virtual port and to Live? | Sixteenth notes at 120 BPM for 30 minutes. Measure arrival at a loopback input in the same browser, and in Live by recording the MIDI into a clip. Compare a worklet clock with a worker timer. Repeat with the tab hidden | Numbers recorded (sigma, max, dropouts, hidden-tab behaviour). The owner sets A3 from them |
| S2 | Does the engine run in an `AudioWorklet`, allocation-free and byte-identical? | Load the module in a worklet, play the five golden patterns, hash the events; time `process()` on a dense pattern | Hashes equal native. *Proposed:* p99 block time under 25% of the 2.67 ms block |
| S3 | How well can the app follow Live's clock? | Live master at 120 BPM over IAC, 30 minutes; estimate tempo and phase | Numbers recorded. The owner sets A4 |
| S4 | Is a Max for Live `jweb` route good enough? | A minimal device shows a page, sends timestamped events, writes MIDI. Check for `navigator.requestMIDIAccess` inside `jweb` | A go or no-go for Tier 2 with the jitter measured |
| S5 | Can the panel controller be built from fixtures? | Five workflows, red first: Page mode toggle, Step zoom, ESC, EDIT cycle, the Play LED | Each fixture fails before and passes after; the LED frames match the tables on p014 and p068 to p070 |

## 9. Open questions that need hardware or the owner

The manual does not answer these. A real Octopus, or the owner, must.

1. The double-click interval and the hold threshold.
2. The flash rate and duty cycle, and what "shine" looks like [p014, p021, p033].
3. The "200" button: the prose, the diagram and the release notes disagree [p093, RN].
4. Whether the MIDI control maps are global or per page [p074 against T07, T08].
5. How to leave Track zoom with ESC, and On-The-Measure [p042, p067].
6. The tempo range, and how the tempo encoder's bar graph maps to BPM [B].
7. The chord-on-a-mouse design in section 5.

The digests list 36 conflicts and open items, 20 of them tabled in `findings.md` section 6.

## 10. Sources

- [W3C] Web MIDI API, `MIDIOutput.send` and `clear`: <https://www.w3.org/TR/webmidi/>
- [MDN] Secure context and permissions: <https://developer.mozilla.org/en-US/docs/Web/API/Web_MIDI_API>
- [caniuse] Support: <https://caniuse.com/midi> (Chrome 43+, Edge 79+, Opera 30+, Firefox 108+, Safari and iOS none)
- [Firefox 108] MIDI needs a site-permission add-on: <https://developer.mozilla.org/en-US/docs/Mozilla/Firefox/Releases/108>
- [Ableton sync] <https://help.ableton.com/hc/en-us/articles/209071149-Synchronizing-Live-via-MIDI>
- [Ableton bus] <https://help.ableton.com/hc/en-us/articles/209774225-Setting-up-a-virtual-MIDI-bus>
- [Ableton MIDI] <https://help.ableton.com/hc/en-us/articles/209774205-Live-s-MIDI-Settings> (listed by search; not read)
- [Max] jweb: <https://docs.cycling74.com/userguide/web_browser/>
- [h1data] <https://github.com/h1data/M4L-jweb-injection>
- [web.dev] Scheduling with two clocks: <https://web.dev/articles/audio-scheduling>

A third-party article says Firefox has no Web MIDI; Mozilla's own release note says it has, behind
an add-on. The Mozilla note is used. These pages were read through a summarising fetch tool,
not downloaded, so a decision that rests on an exact line should be checked at the source.

## 11. Risks

| Risk | Consequence | Mitigation |
|---|---|---|
| Browser MIDI timing is looser than the gear needs | Audible flams on tight drum machines, and the same jitter reaches Live | S1 first; a lookahead setting; a documented limit; the owner sets A3 from the measured numbers |
| Hidden tabs throttle timers | Timing collapses when the window is not visible | The worklet clock (S1, S2), tested hidden |
| About 140 workflows is a great deal | The work never finishes | Waves, fixtures, and the ratchet; each wave ships |
| The manual is silent on gesture timing and LED animation | A UI that feels wrong | Section 9, and a hardware check before Wave 1 ends |
| Hold-and-press on a mouse | The interface is unusable on a laptop | The latch design, tested with someone who knows the hardware |
| Reversing closed decisions | Sunk work in `octoroom` and the docs | Small in code (495 lines of Rust and prose in the `octoroom` crate) and larger in prose (one 307-line document and sections of nine others, `removal-plan.md`). All of it stays in git history |
| Tier 2 does not work | "In Ableton" means routing only | Tier 1 stands alone; S4 says so early |

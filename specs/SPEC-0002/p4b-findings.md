# P4b findings: what Web MIDI gives a page, and what could be measured here

P4b's first act (`p4-plan.md` D-P4-9): read what the Web MIDI text says about real-time messages without the SysEx
permission and about the timestamp of an incoming message, and measure it on a real port. **No real port could be
reached from here**, so the first half is read and the second half is not measured. Both are said plainly below, and
nothing in P4c to P4e relies on either until the owner's run.

## 1. What was read

Source: the W3C Web MIDI API text, https://www.w3.org/TR/webmidi/, fetched on 2026-10-02 through the fetch tool, which
returns a summary written by a small model, not the page. The quotations below are as that tool returned them, and the
section numbers are the tool's. **They were not checked against the page by eye.**

| Question | What the text said |
|---|---|
| Are 0xF8, 0xFA, 0xFB, 0xFC filtered without the SysEx permission? | The rule for an incoming message is "If the MIDIAccess did not enable System Exclusive access, and the message is a System Exclusive message, abort this process" (5.4.1). Real-time messages are not System Exclusive, so on this text they are delivered |
| What is `MIDIMessageEvent.timeStamp`? | A `DOMHighResTimeStamp` that "represents the high-resolution time of when the event was received or is to be sent" (5.5). **The text does not name the clock.** That it is on the `performance.now()` timebase is the expectation, and it is unmeasured |
| What does `send` do with a timestamp of zero or in the past? | "the data is to be sent as soon as possible" (5.4.2). P3's scheduler already relies on this and says so |

What this means for D-P4-9: the plan said "if a browser needs the permission for clock, that is a finding and P4d stops".
The text gives no sign that it does. That is a reading and not a measurement, so it stays open until a real port has
delivered a clock to this page. The page still asks for Web MIDI without SysEx.

## 2. What was measured here

Chromium 141.0.7390.37 (the one the tests run), headless, in this Linux container, page on a `localhost` origin:

| Page and permission | `requestMIDIAccess({ sysex: false })` |
|---|---|
| `about:blank` (not a secure context) | The function does not exist |
| `localhost`, nothing granted | Rejects with `NotAllowedError`, "Permission to use Web MIDI API was not granted." |
| `localhost`, `midi` granted by Playwright | Still `NotAllowedError`. Why is not known |
| `localhost`, `midi` and `midi-sysex` granted | Rejects with `InvalidStateError`, "Platform dependent initialization failed." The container has no MIDI backend |

So Web MIDI cannot be started in this environment, with or without a permission. Three consequences:

- **Not measured:** the timestamp an incoming message carries, the clock it is on, whether 0xF8 reaches a page on a real
  port without SysEx, jitter on arrival, any difference between macOS, Windows and Linux. Spike S1's page already measures
  output arrival; P4e's S3 page adds `event.timeStamp - performance.now()` at the moment of receipt for an input, so the
  owner's run on a Mac with the IAC Driver gives the first real number.
- **Used as evidence in the tests:** the refusal is real, so the Chromium test for M8 runs against the browser's own
  `NotAllowedError` with no stand-in (the other cases use a stand-in and say so).
- **A finding that changed the page:** `InvalidStateError` is not a refusal. The page said "MIDI access was refused. Allow it
  for this site", which would send a person to the wrong fix. It now says "Web MIDI could not start on this system" and
  repeats the browser's own reason, and a refusal still says how to allow it (`src/midi-access.ts`, tests in
  `test/midi-access.test.ts` and `M8` in the Chromium suite).

## 3. Smaller findings from building P4b

- **`MidiScheduler.setOutput` cleared every device's ordering memory on any change.** With one port that was harmless. With
  two it let a change on port 2 forget what port 1's device had been promised, and the page calls it for both ports whenever
  either list changes. It now forgets only a device that no port uses any more (`test/midi-out.test.ts`, "choosing a device
  for the other port does not unseat…"). A device shared by both ports is one ordering, which is what the device sees.
- **The browser's `MIDIInput` handler takes the whole event.** A narrower handler type fails the type check against the real
  interface, so `MidiInputLike` states the handler as a method signature, which TypeScript compares bivariantly.
- **`octoweb_set_track` added without a version bump.** It is an additive export, a page built for ABI 1 never calls it, and
  `ABI.md` lists the attribute numbers in the engine's own order, pinned by a Rust test, a Node test that reads `ABI.md`, and
  `TRACK_ATTR`.

## 4. P4b against the plan's observables

| Observable | Status |
|---|---|
| M1, two ports each to its own device | Met. Node (scheduler) and Chromium (two fake outputs, the route stand-in, one device for both, one port without a device) |
| M6, input decoded and counted, no consumer | Met against a fake `MIDIInput`. Not against a real port (section 2) |
| M7, the settings view | **Partly.** Out 1, Out 2 and In are chosen from the keyboard with the manual's names. **The clock state (off, master, slave, slave with echo) is not there:** master needs P4c and slave needs P4d, and a control that does nothing must not be shown. It arrives with them |
| M8, no Web MIDI or refusal | **Partly.** No Web MIDI, a real refusal and a platform failure are each said and the app runs. **Refusing a clock state with a sentence** arrives with the clock state |

# apps/web

The WENGE web sequencer: the engine and the panel controller in an AudioWorklet, one Web MIDI output, and the first
panel. SPEC-0002, task WENGE-0014, ADR-0008. Owner: the Forge.

## What is here

| Path | What |
|---|---|
| `engine/` | The Rust crate `octoweb`: the engine and `octoface` behind plain C exports, no `wasm-bindgen`. The export list and buffer layouts are `engine/ABI.md`; a test keeps it in step with the code |
| `src/abi.ts` | A typed wrapper over the exports. Runs in the worklet and in Node |
| `src/worklet.ts` | The `AudioWorkletProcessor`: renders 128 frames, posts events and, every 8 blocks, the LED frame and playheads |
| `src/host.ts` | The page's side: compiles the module, starts the worklet, routes its messages, sends panel input |
| `src/midi-out.ts`, `src/wire.ts` | Turns events into `MIDIOutput.send(data, timestamp)` calls with a lookahead, one device for each port, and joins them to the host |
| `src/midi-in.ts`, `src/route.ts` | The input path (decode and count; the clock is the one consumer) and the `?route=` stand-in for port 2 (parses text only) |
| `src/clock-estimator.ts`, `src/follower.ts`, `src/clock-state.ts` | The clock as a slave: a Kalman filter on the pulses' times, the loop that steers the engine's tempo and transport from outside, and the four clock states with their sentences. See **The MIDI clock** |
| `pages/app.*` | The app: `app.html` (the strip is written here), `app.css` (names tokens, writes none), `app.ts` (joins the panel, the engine and Web MIDI) |
| `src/panel.ts`, `src/layout.ts`, `src/press.ts`, `src/strip.ts`, `src/midi-access.ts` | The SVG panel, where each control goes, pointer and key presses, the strip, and Web MIDI as the page needs it |
| `src/tokens.ts`, `src/tokens-css.ts` | `contracts/design.tokens.json` read and written out as the stylesheet `dist/tokens.css` |
| `layout/panel.layout.json` | Where the first panel puts what it draws: cells and token names, no length |
| `pages/host.*` | The page the Chromium tests of the engine drive (no UI) |
| `spikes/s2/` | Spike S2: the module in an AudioWorklet, hashes against native, and the cost of `render` |
| `spikes/s1/` | Spike S1: Web MIDI timing. `run.ts` for here, `page.html` for a person with a loopback port |
| `spikes/s3/` | Spike S3: the MIDI clock with Ableton Live, followed and sent. `page.html` for a person with a real port, `report.ts` to read the saved file, `run.ts` for a run here against a simulated clock. `ABLETON.md` is the guide |
| `test/` | Node tests, and `test/browser/` for Chromium |

## Commands

```
just web-app            # build, then serve the app: open http://localhost:8080/pages/app.html in Chrome or Edge
just web-wasm           # build the module into apps/web/dist (the shipped build, and one with the spike exports)
just web-test           # type-check and the Node tests
just web-browser-test   # the real host and worklet in Chromium
just spike-s2 --write   # S2, and save the record to handoffs/evidence/
just spike-s1 --seconds 60 --write        # S1's stand-in run; add --hidden under xvfb for the background-tab run
just spike-s3 --seconds 30 --write        # S3's run against a SIMULATED clock; the real one is spikes/s3/page.html
```

Chromium comes from `npx playwright-core install chromium`, or from `PLAYWRIGHT_BROWSERS_PATH`.

## The panel

`pages/app.html` is a strip and a panel. The strip is plain: **Start**, the device for **MIDI Out 1** and for **MIDI Out 2**,
the device for **MIDI In**, the **MIDI Clock** state, the **clock offset** in milliseconds, the **lookahead** in milliseconds
(30 by default), and sentences that say what the app and the browser's MIDI and clock are doing. It never sits on the panel and holds no colour. The panel is one SVG that scales as a whole and
never rearranges.

- **Start** is a button because a browser keeps audio silent until the page has been used. It starts the engine in its
  worklet and asks the browser for MIDI. Until then a press on the panel only says to press Start.
- **No Web MIDI** (Safari, Firefox): the page loads and runs, and the strip says the browser has no Web MIDI. A refusal of
  access says that instead. Either way the keys and the LEDs work; nothing is sent.
- **Two outputs**: MIDI Out 1 is the engine's port 1 and MIDI Out 2 is port 2 (a track's MIDI channel 1 to 16 is port 1 and
  17 to 32 is port 2, as the manual has it). Nothing is sent to a port until a device is chosen for it, and a port with no
  device sends nothing while the other still plays. One device may be chosen for both: it hears both, and the strip says it
  is one device. Each device keeps its own timestamp ordering (`src/midi-out.ts`), so a device shared by both ports is one
  ordering. A track gets onto port 2 from Track zoom, which does not exist yet; until then see **The route hook**.
- **One input**: choose a device for MIDI In and its messages are decoded (`src/midi-in.ts`: Note On and Off, controller,
  pitch bend, channel pressure, program change and the four real-time messages 0xF8, 0xFA, 0xFB, 0xFC) and counted, and the
  strip shows the count. **Only the clock uses them** (see **The MIDI clock**): recording, transpose, force-to-scale and map
  learn attach at `InputPath.onMessage` in the waves that own them. The page asks for Web MIDI without the system exclusive
  permission, and system exclusive is not delivered without it.
- **A browser that has Web MIDI and cannot start it** (no MIDI backend on the system) says so and shows the browser's own
  reason, apart from a refusal, which says how to allow it.
- **What is drawn**: the 160 matrix keys, and the ten controls the controller acts on (`mode.page`, `mode.step`,
  `mode.play`, `mode.edit`, `keys.esc`, `keys.program`, `mutator.tgl`, `mutator.zom`, `mutator.mute`, `transport.stop`)
  and `transport.play`. The other 76 of the 247 controls arrive with their waves. Their arrangement is provisional
  (`layout/panel.layout.json` says why).
- **The Play key** (`transport.play`) is the page's own command to the engine, a stopgap until the transport workflow is
  built from the manual (ADR-0008 decision 4). **The tempo** is the engine's 120; its encoder is Wave 1.
- **The chase-light** is not drawn (the manual does not say how it overlays a lit step; request to the Panelwright).
- **Keys**: a pointer press, or Space or Enter on a focused control, presses it; letting go releases it. Holding a key down
  does not press it twice. A click that no pointer made (a screen reader's Activate) is one press and one release.
- **LEDs** are drawn from the engine's LED frame: dark, or lit in the role's colour. A flashing LED flashes at the token's
  period and all flashing LEDs flash together; a shine LED is ringed in ink. The state is also said in words
  (`aria-description`), because colour is not the only carrier.
- **No colour, length or type size is written in the app.** `dist/tokens.css` is generated from the tokens file; the
  stylesheet only names it; the panel's geometry is computed from the tokens. `engine/tests/app_source_rules.rs` fails the
  build on a hex colour, a length, a gradient, a shadow, an easing keyword or a second animation in the app's files.

## The route hook

`?route=0:17,1:3` in the address sets the MIDI channel of tracks for the session: track 0 to channel 17, which is port 2
channel 1, and track 1 to channel 3, which is port 1 channel 3. The track is the row of the matrix, 0 to 9, as the control
ids give it (`matrix.r3.c1` is track 3). The channel is 1 to 32. The page reads the address once, when it starts, applies the
routes after the engine is up, and the strip says what it applied and what it left out and why.

It is a test and demonstration hook, **not a control**: nothing draws it, nothing saves it, and it exists because a track can
only reach port 2 from Track zoom, which is not built (`specs/SPEC-0002/p4-plan.md` D-P4-2 and F-P4-2). It is removed when
Track zoom lands. Only `pages/app.ts` reads the address (`engine/tests/app_source_rules.rs` fails if another file under
`src/` reads `location`); `src/route.ts` only parses text.

## The MIDI clock

The strip's **MIDI Clock** chooses one of four states, with the manual's own words (p. 93): **Off**, which is the default
(nothing is sent and nothing is followed), **Master Clock**, **Slave Clock**, and **Slave Clock with MIDI Clock echo**. One
sentence says what the state is doing, and it changes only when the state does (it is a live region); the numbers under it
(tempo, pulses heard, jitter, how far the engine is from the sender's beat) are redrawn four times a second and are not one.
The state is not saved: Save and Load do not exist yet.

- **Master** (ADR-0009). The engine sends Start or Continue when the transport starts and a Clock every 8 ticks, which is 24 to
  the quarter note, as kind 5 records in the same events as the notes, in sample order. The scheduler sends each one to
  **every chosen output, once for each device**: a device chosen for both ports hears one clock. With no output chosen it is
  accepted and says that nothing is sent yet.
- **Slave**. The clock on the chosen **MIDI In** steers the engine from outside. The engine still ticks on its own sample clock:
  the page reads the pulses' times (`src/clock-estimator.ts`, a Kalman filter that copes with jitter, a ramp, a step, a missing
  or doubled pulse and a stall), sets the engine's tempo (the sender's, trimmed by up to 8% to keep the beat in step) and
  starts and stops the transport (`src/follower.ts`). Start and Continue take effect at the next pulse, Stop at once; a pulse
  alone never starts the transport. With no pulse for 500 ms the sentence says the clock stopped and the engine plays on at the
  last tempo. Choosing another input starts the follower again. With no input chosen the state is accepted and says to choose one.
- **Echo** is slave, and passes the clock, Start, Continue and Stop on to the chosen outputs as they arrive: at once, with no
  timestamp and no lookahead (the sender timed them). **Do not send the echo to the port the clock comes from.** A virtual
  port that both receives and sends feeds the clock back to itself, and the tempo runs away; nothing in the page can see that.
- **Clock offset in milliseconds** is extra lead for the notes while following: positive makes them land earlier. The engine
  runs ahead of the sender's grid by the lookahead plus this offset, because the scheduler adds the lookahead to everything it
  plays. It is the constant that Live's "MIDI Clock Sync Delay" corrects, with the opposite sign. Zero is the lookahead alone.
- **A state that cannot work is refused, in words, and the control goes back to Off**: no Web MIDI, MIDI access refused, or Web
  MIDI that could not start. Before Start (and while the browser's MIDI prompt is open) the choice is kept and says it waits.

**Not done, and said here so that nobody finds it by surprise.** Song position (0xF2) is read and ignored, so a Continue from the
middle of a song resumes where this sequencer stopped and not where the sender is; the engine does not rewind on Stop, so Start
and Continue are the same to it. The Play key on the panel stays the page's own command whatever the clock state, so Play while
slaved starts the engine on its own and the sender's next Start finds it running. Turning the engine's tempo by hand while slaved
is overwritten by the follower within a few milliseconds. A step of the pattern is 12 ticks, which is 1.5 pulses; whether that is
in time with a drum machine is decision D0 (`WENGE-0012`) and not something this page claims.

**Not measured.** The follower has been run only against a simulated sender and a simulated engine, and with the real engine
and worklet in `test/follower-engine.test.ts`. That Web MIDI's `event.timeStamp` is on `performance.now()`'s clock is an
assumption, and so is what a real sender's jitter looks like; spike S3 (`spikes/s3/`, `ABLETON.md`) is the measurement.

## Tab order

Acceptance criterion A8. The order of the elements in the page is the tab order (nothing has a `tabindex` above 0), and it
follows the panel from the top left to the bottom right. This section is checked against `layout/panel.layout.json` by
`test/readme.test.ts`, so it cannot drift.

1. The strip: `start`, `midi-out-1`, `midi-out-2`, `midi-in`, `clock-state`, `clock-offset`, `lookahead`.
2. The 160 matrix keys in reading order: row 9 (the top row) from column 1 to column 16, then row 8, and so on down to
   row 0 (the bottom row).
3. The other controls, in the order of the layout file (left to right, row by row):
   1. `mode.page`
   2. `mode.step`
   3. `mode.play`
   4. `mode.edit`
   5. `keys.esc`
   6. `keys.program`
   7. `mutator.tgl`
   8. `mutator.zom`
   9. `mutator.mute`
   10. `transport.play`
   11. `transport.stop`

Every control is a `button` whose name is the manual's own name for it (`contracts/controls.json`, `name`). Focus is a
ring of two strokes, a light one outside the key and a dark one inside its edge, so that it shows on the ground and on a
lit LED; it is drawn from the focus tokens and appears on keyboard focus, not on a pointer press.

**Known cost.** There are 160 Tab stops before the first control that is not a matrix key. A single stop with arrow keys
(a roving focus) is the usual cure, and it would change what E8 checks, so it is the owner's to choose; it is not done.

## Two rules in the MIDI scheduler

Both came from the Stop property test (`test/stop-property.test.ts`) failing, and both are in ADR-0008 amendment 1.

1. **`MIDIOutput.clear()` is never called.** Chromium does not have it, and where it exists it drops Note Offs the
   engine has already counted as sent, so notes stay on.
2. **Timestamps on one output never go backwards.** The clock map is re-read for every batch and moves by a
   millisecond or two. A flush Note Off one sample after a Note On could otherwise be stamped before it.

The cost of the first: what was queued when Stop is pressed plays out, up to the lookahead (30 ms by default).

## Running spike S3 on your machine

S3 asks how well the app follows Ableton Live's clock, and how a port treats the clock the app sends. `ABLETON.md` has the whole
procedure; in short:

1. Build as for S1 (steps 1 and 2 above) and open `/spikes/s3/page.html` in Chrome or Edge.
2. **Follow a clock**: choose the port Live's clock arrives on, press Start following, then Play in Live, and leave the tab in
   front for 30 minutes. **Send a clock**: a loopback port, output and input, no Live needed.
3. Save the result (a JSON file of the raw records) and read it with `node spikes/s3/report.ts <file>`; add `--settle <ms>` for a
   different settling and `--nominal <bpm>` for the tempo Live was set to.
4. Put the file and the report in `handoffs/evidence/`.

`just spike-s3` runs both against a simulated sender and port, which shows the harness works and nothing else: its records say on
their first line that they are not Live, not a MIDI port and not Web MIDI's timing.

## Running spike S1 on your machine

S1 has two halves. The half that runs in a container uses a timer in place of a MIDI port, so it only measures how
early events reach the scheduler. The other half needs a real port, and until it has been run the kill criterion in
`docs/09` cannot be evaluated.

1. `just web-wasm`, then in `apps/web`: `npm ci && npm run build`, then serve the folder (any static server that
   serves `/dist`, `/spikes` and `/contracts` from the repository; `node -e "import('./scripts/serve.ts').then(m=>m.serve(8080)).then(s=>console.log(s.url))"` does).
2. Open `/spikes/s1/page.html` in Chrome or Edge. macOS: enable the IAC Driver in Audio MIDI Setup first. Windows: loopMIDI.
3. Allow MIDI, choose the loopback's output and input, set the minutes (30 is the usual), Start.
4. Run it twice: once with the tab in front, once switching to another tab for the whole run.
5. Save the result and put it in `handoffs/evidence/`.

A run sends about 16 messages a second to the output you chose. Use a loopback and not a synth.

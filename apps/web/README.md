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
| `src/midi-in.ts`, `src/route.ts` | The input path (decode and count; no consumer yet) and the `?route=` stand-in for port 2 (parses text only) |
| `pages/app.*` | The app: `app.html` (the strip is written here), `app.css` (names tokens, writes none), `app.ts` (joins the panel, the engine and Web MIDI) |
| `src/panel.ts`, `src/layout.ts`, `src/press.ts`, `src/strip.ts`, `src/midi-access.ts` | The SVG panel, where each control goes, pointer and key presses, the strip, and Web MIDI as the page needs it |
| `src/tokens.ts`, `src/tokens-css.ts` | `contracts/design.tokens.json` read and written out as the stylesheet `dist/tokens.css` |
| `layout/panel.layout.json` | Where the first panel puts what it draws: cells and token names, no length |
| `pages/host.*` | The page the Chromium tests of the engine drive (no UI) |
| `spikes/s2/` | Spike S2: the module in an AudioWorklet, hashes against native, and the cost of `render` |
| `spikes/s1/` | Spike S1: Web MIDI timing. `run.ts` for here, `page.html` for a person with a loopback port |
| `test/` | Node tests, and `test/browser/` for Chromium |

## Commands

```
just web-app            # build, then serve the app: open http://localhost:8080/pages/app.html in Chrome or Edge
just web-wasm           # build the module into apps/web/dist (the shipped build, and one with the spike exports)
just web-test           # type-check and the Node tests
just web-browser-test   # the real host and worklet in Chromium
just spike-s2 --write   # S2, and save the record to handoffs/evidence/
just spike-s1 --seconds 60 --write        # S1's stand-in run; add --hidden under xvfb for the background-tab run
```

Chromium comes from `npx playwright-core install chromium`, or from `PLAYWRIGHT_BROWSERS_PATH`.

## The panel

`pages/app.html` is a strip and a panel. The strip is plain: **Start**, the device for **MIDI Out 1** and for **MIDI Out 2**,
the device for **MIDI In**, the **lookahead** in milliseconds (30 by default), and sentences that say what the app and the
browser's MIDI are doing. It never sits on the panel and holds no colour. The panel is one SVG that scales as a whole and
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
  strip shows the count. **Nothing uses them yet.** Recording, transpose, force-to-scale, map learn and the clock follower
  attach at `InputPath.onMessage` in the waves that own them. The page asks for Web MIDI without the system exclusive
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

## Tab order

Acceptance criterion A8. The order of the elements in the page is the tab order (nothing has a `tabindex` above 0), and it
follows the panel from the top left to the bottom right. This section is checked against `layout/panel.layout.json` by
`test/readme.test.ts`, so it cannot drift.

1. The strip: `start`, `midi-out-1`, `midi-out-2`, `midi-in`, `lookahead`.
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

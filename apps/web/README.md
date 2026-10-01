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
| `src/midi-out.ts`, `src/wire.ts` | Turns events into `MIDIOutput.send(data, timestamp)` calls with a lookahead, and joins them to the host |
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

`pages/app.html` is a strip and a panel. The strip is plain: **Start**, the **MIDI output** to send to, the
**lookahead** in milliseconds (30 by default), and two sentences that say what the app and the browser's MIDI are doing.
It never sits on the panel and holds no colour. The panel is one SVG that scales as a whole and never rearranges.

- **Start** is a button because a browser keeps audio silent until the page has been used. It starts the engine in its
  worklet and asks the browser for MIDI. Until then a press on the panel only says to press Start.
- **No Web MIDI** (Safari, Firefox): the page loads and runs, and the strip says the browser has no Web MIDI. A refusal of
  access says that instead. Either way the keys and the LEDs work; nothing is sent.
- **One output**: it is port 1. Nothing is sent until one is chosen. The second port comes with P4.
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

## Tab order

Acceptance criterion A8. The order of the elements in the page is the tab order (nothing has a `tabindex` above 0), and it
follows the panel from the top left to the bottom right. This section is checked against `layout/panel.layout.json` by
`test/readme.test.ts`, so it cannot drift.

1. The strip: `start`, `midi-output`, `lookahead`.
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

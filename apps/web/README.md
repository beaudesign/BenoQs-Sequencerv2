# apps/web

The WENGE web sequencer: the engine and the panel controller in an AudioWorklet, one Web MIDI output.
SPEC-0002, task WENGE-0014, ADR-0008. Owner: the Forge. The first panel (P3c) is not here yet; this is the
engine in the browser, with the two spikes that decide whether the plan stands.

## What is here

| Path | What |
|---|---|
| `engine/` | The Rust crate `octoweb`: the engine and `octoface` behind plain C exports, no `wasm-bindgen`. The export list and buffer layouts are `engine/ABI.md`; a test keeps it in step with the code |
| `src/abi.ts` | A typed wrapper over the exports. Runs in the worklet and in Node |
| `src/worklet.ts` | The `AudioWorkletProcessor`: renders 128 frames, posts events and, every 8 blocks, the LED frame and playheads |
| `src/host.ts` | The page's side: compiles the module, starts the worklet, routes its messages, sends panel input |
| `src/midi-out.ts`, `src/wire.ts` | Turns events into `MIDIOutput.send(data, timestamp)` calls with a lookahead, and joins them to the host |
| `pages/host.*` | The page the Chromium tests drive (no UI) |
| `spikes/s2/` | Spike S2: the module in an AudioWorklet, hashes against native, and the cost of `render` |
| `spikes/s1/` | Spike S1: Web MIDI timing. `run.ts` for here, `page.html` for a person with a loopback port |
| `test/` | Node tests, and `test/browser/` for Chromium |

## Commands

```
just web-wasm           # build the module into apps/web/dist (the shipped build, and one with the spike exports)
just web-test           # type-check and the Node tests
just web-browser-test   # the real host and worklet in Chromium
just spike-s2 --write   # S2, and save the record to handoffs/evidence/
just spike-s1 --seconds 60 --write        # S1's stand-in run; add --hidden under xvfb for the background-tab run
```

Chromium comes from `npx playwright-core install chromium`, or from `PLAYWRIGHT_BROWSERS_PATH`.

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

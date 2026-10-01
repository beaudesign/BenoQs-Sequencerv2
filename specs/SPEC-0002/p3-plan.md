# SPEC-0002 P3 plan: the engine in the browser, and the first panel

| Field | Value |
|---|---|
| Task | `WENGE-0014` (`handoffs/WENGE-0014.ndjson`) |
| Spec | SPEC-0002 r1, approved 2026-09-30 15:06 Paris. This plan is a revision of nothing: it fills in P3, which `tech.md` leaves open in two places (ADR-0008) |
| Plan revision | r0, written 2026-10-01 after the owner's "Start p3" (12:00 Paris) |
| Stage | Tech spec, awaiting the owner. Implementation of P3b and P3c starts from this plan; the owner can reject any numbered decision |
| Decision record | `adr/0008-the-page-calls-rust-through-c-exports-and-tokens-are-a-contract.md` |
| Tier | P3a High (it authorises a contract), P3b Medium, P3c High (it adds a contract file) |

## 1. What P3 is

`docs/09-roadmap.md`: *`apps/web`: the WebAssembly engine, the 16 by 10 matrix and the LEDs, one Web
MIDI output (spikes S1 and S2).* The first browser build of the instrument, and the two spikes
that can end the plan early (the roadmap's kill criterion is S1).

**How I will know it worked.** Each line is a command with an expected result, written red first
where code is involved:

| # | Observable | How it is checked |
|---|---|---|
| E1 | A matrix press in a real browser turns the engine's step on and lights the key green; a second press turns it off | A Playwright test against the built app, reading the LED state from the page. Fails before P3c |
| E2 | Pressing Play makes a fake Web MIDI output receive a Note On with the right bytes, a Note Off after it, and timestamps no earlier than `now + L` | The same test, with a fake `MIDIOutput` that records `send(data, timestamp)` |
| E3 | Pressing Stop makes the receiver hold no note that is on, and `clear()` was called before the flush | A test of the scheduler with a fake receiver; a property over generated patterns |
| E4 | The five golden patterns give the same SHA-256 inside an `AudioWorklet` as natively | Spike S2, a recorded browser run. Hashes equal or the spike fails |
| E5 | `render` allocates nothing, natively and in the worklet | A native test and the S2 run, both with the counting allocator (`measure`) |
| E6 | The Web MIDI path's scheduling margin and arrival jitter are recorded, with the tab visible and hidden | Spike S1: the scheduling side here, the arrival side by the owner (section 5) |
| E7 | In a browser without Web MIDI the app loads, runs and says it has no MIDI (A7) | A Playwright test that removes `navigator.requestMIDIAccess` |
| E8 | Every control the page draws has the manual's own name, and tab order follows the layout (A8) | A Playwright test over `controls.json` names; the order is written down |
| E9 | `cargo xtask verify --base origin/main` passes and the baseline only grew | CI `verify` |

## 2. What was measured before planning

Evidence in `handoffs/evidence/p3-browser-probe/` (run again with `node run.mjs`):

- An `AudioWorkletGlobalScope` in Chromium 141 has no `TextDecoder`, `TextEncoder`, `fetch`, `crypto`,
  `performance`, `setTimeout` or `SharedArrayBuffer`. It has `Date`, `Atomics` and `WebAssembly`.
- A compiled `WebAssembly.Module` sent in `processorOptions` arrives as a module and instantiates
  synchronously.
- `AudioContext.getOutputTimestamp()` gives both clocks (`contextTime`, `performanceTime`), and the
  context ran at 48 kHz with a base latency of 10.02 ms.
- `navigator.requestMIDIAccess()` is refused in headless Chromium even with the `midi` permission
  granted. This container has no `/dev/snd`, so Chromium has no ALSA MIDI device to open, and no real Web MIDI
  traffic can be produced or received here.

What those facts do to the plan:

- The engine module is called through C exports (ADR-0008 decision 1).
- High-resolution timing of `process()` cannot be taken inside the worklet. S2 times the same module on the
  main thread with `performance.now()` and in the worklet in batches with `Date`, and says which is which.
- Spike S1 splits in two: what the page controls (when it sends relative to when the event is due)
  runs here against a recording output; what the browser, the operating system and the other end add
  needs a real port and the owner's machine.

## 3. Decisions

Each is stated so a reviewer can reject it. D-P3-1 and D-P3-2 are in ADR-0008.

| # | Decision | Default | Alternative |
|---|---|---|---|
| D-P3-1 | How the page calls Rust | C exports, no `wasm-bindgen` (ADR-0008 1) | `wasm-bindgen` with a polyfilled worklet |
| D-P3-2 | Where the module lives | `apps/web/engine/`, package `octoweb`, `octoffi` stays frozen (ADR-0008 2) | A crate under `crates/` with a role-brief and `CODEOWNERS` change |
| D-P3-3 | Build tooling | TypeScript (strict) compiled with `tsc`, ES modules served as they are, no bundler, no UI framework. Node 22 runs the unit tests with its built-in runner and type stripping. Dev dependencies: `typescript` and `playwright`, pinned by a lockfile | A bundler |
| D-P3-4 | What the first panel draws | The 160 matrix keys, and the controls `octoface` acts on today (the ten in its roles table, plus `transport.play`), in one provisional layout file. The other 76 of the 247 controls arrive with their waves | Draw every control now, with a provisional arrangement for the zones the manual leaves open |
| D-P3-5 | The chase-light | Not drawn in P3. The manual gives its colour but not how it overlays a lit step (p014, p017, p052). A request to the Panelwright asks for it as a workflow with a pending question | Draw a Red marker now and mark it provisional |
| D-P3-6 | The Play key | The page sends the engine's `Play` for `transport.play`, as a stopgap until the Panelwright builds the transport workflow (ADR-0008 4) | Wear the Panelwright hat in P3c and add it to `octoface` with a fixture |
| D-P3-7 | CI | A `web` job beside `verify` and `wasm-smoke`: builds the module, type-checks, runs the Node tests and the Playwright run, with `npx playwright install --with-deps chromium` | Add the browser run to `cargo xtask verify` |
| D-P3-8 | The ratchet for browser tests | Not in the baseline in P3. A request to the Referee asks to floor them by name (the baseline reads `cargo test` output today) | Floor them now with a small parser in `xtask` |
| D-P3-9 | Lookahead `L` | 30 ms until S1 says otherwise (`tech.md` section 3) | |

D-P3-8 is a known gap: a test that only the web job runs can be deleted without the ratchet noticing. The
native tests of the crate (ABI, determinism, allocation) are in the baseline.

## 4. The series

### P3a: this pull request (Conductor hat, documents only)

ADR-0008, this plan, `journal/`, `handoffs/WENGE-0014.ndjson`, `STATE.md` and the probe evidence. No
code and no contract change.

### P3b: the engine in the browser (Forge hat, with Referee and Conductor hats for CI and `justfile`)

Medium. Branch `forge/p3-engine-in-the-browser`. Red first, in this order:

1. **The crate and its ABI**, tested natively. `apps/web/engine/tests/`:
   - the module reports its ABI number and rejects a layout that lacks a control the controller needs;
   - a press on `matrix.r0.c1`, through `input`, lights that key green (the LED byte), and a second press clears it;
   - after programming a step by pressing, `play` then `render` emits the engine's Note On at the
     expected note, channel and port, byte for byte what `Engine` gives when it is driven directly;
   - events after Stop contain a Note Off for every note on, and the engine's `CC 123` flush;
   - `render` makes zero allocations (the `measure` feature, thread-local counter, as `octoffi`'s test does);
   - the pointers the module reports stay valid between calls (the buffers are not reallocated).
2. **The worklet host** (`apps/web/src/worklet.ts`, `host.ts`): the page compiles the module, passes it in
   `processorOptions`, forwards panel input by `postMessage`, receives events with audio-clock times and a
   LED frame about 60 times a second. Tested in Chromium (Playwright).
3. **The MIDI scheduler** (`apps/web/src/midi-out.ts`): a pure class with an injected clock and an
   injected `MIDIOutput`-like. Node tests: timestamp is `perf(audio time) + L`; an event that is already late
   is sent at once and counted, never dropped; Stop calls `clear()` and then sends the flush; no Note Off is
   sent for a note that was never on; velocity is at least 1.
4. **Spike S2**, then **spike S1**, each a script in `apps/web/spikes/` and a recorded run in
   `handoffs/evidence/` (section 5).
5. **CI**: the `web` job; `just` verbs `web-build`, `web-test`, `spike-s2`.

Exit: E3, E4, E5 and E6 (the part that runs here) recorded; `verify` green; baseline grew.

### P3c: the first panel (Forge hat, with Curator and Conductor hats for the tokens)

High (a contract file). Branch `forge/p3-web-app`, built on P3b. Red first:

1. `contracts/design.tokens.json` and its schema, with the contract test of ADR-0008 decision 6
   written before the values: it fails, then the Curator hat proposes values that pass.
2. The panel: SVG from `controls.json` and one provisional layout file (no pixel literal in the
   panel code). The matrix and the controls the controller acts on. LEDs from the LED frame with the
   steady, flashing and shine states drawn from tokens only.
3. The settings strip: the one MIDI output to use, a message when the browser has no Web MIDI, the
   lookahead. Plain, no colour, never on top of the panel (`docs/05` section 1).
4. Keyboard and labels: each control a labelled element with the manual's name, tab order in layout order,
   visible focus.
5. Playwright tests for E1, E2, E7 and E8. The traversal order is written in `apps/web/README.md`.

Exit: E1, E2, E7, E8, E9 and the tokens contract test pass; a screenshot of the panel is in the PR for the
owner to ratify the tokens.

### Follow-ups, not in P3

- The chase-light as an `octoface` workflow (Panelwright; request filed).
- The Referee floors the browser tests by name; builds `verify:a11y` and `verify:tokens` over the app
  (requests filed).
- The Scribe corrects `docs/02` (`unsafe`, the crate list, "planned, P3") and `docs/05` (status).

## 5. The spikes

### S2: the engine in an `AudioWorklet`

Pass (`tech.md` section 8): hashes equal native; *proposed* p99 block time under 25 % of the 2.67 ms block.

Method, run in this container with Chromium 141:

1. Build `octoweb` for `wasm32-unknown-unknown` with the `measure` and `spike` features. `spike` adds
   one export that runs a pattern through `octorun` and returns the SHA-256 of its event log.
2. The page compiles the module and starts a worklet that instantiates it.
3. For each of the five patterns in `examples/`, the worklet runs the pattern and the page compares the hash
   with `examples/golden/NAME.sha256`. Then the same patterns with the render buffer rewritten to 128
   frames (the worklet's block) against a native run of the same text. Phrase and MCC patterns run too;
   the Node smoke test skips them for lack of FFI setters.
4. The dense pattern (ten tracks, chords, every step on) runs for a stated number of blocks:
   - allocation count before and after, with the counting allocator;
   - the same blocks timed on the main thread with `performance.now()` (per-block p50, p99, max);
   - inside the worklet, in batches timed with `Date` (a mean, since `performance` is absent there).

Recorded: `handoffs/evidence/p3-s2-*.txt` with the Chromium version, the commit, the hashes and the numbers.
Fail: any hash differs, or any allocation in `render`. A timing over the proposed budget is a finding for the
owner, not a gate change.

### S1: how steady is Web MIDI output

Pass (`tech.md` section 8): numbers recorded (sigma, max, dropouts, hidden-tab behaviour); the owner sets A3.

What can be measured **here**, with a recording output standing in for the port:

- Margin: for each event the page sends, `timestamp - performance.now()` at the moment of the call. A
  negative margin is an event that was already late. Distribution, count of late events.
- Worklet clock against a worker timer, both feeding the same scheduler.
- Visible against hidden: Playwright can hide a tab only in a headed browser. If a virtual display is
  available the run is repeated with the tab in the background, and the record says whether it was.

What needs **the owner's machine** (a real port):

- Arrival jitter at a loopback input in the same browser, and in Live by recording into a clip.
  The page for it is `apps/web/spikes/s1/`. It lists the browser's outputs and inputs, sends sixteenth
  notes at 120 BPM for a chosen time (30 minutes by default), measures arrival with each
  `MIDIMessageEvent`'s `timeStamp`, prints sigma, p99, max, missing and extra events, and offers the
  result as a JSON file to save into `handoffs/evidence/`. macOS: the IAC Driver; Windows: loopMIDI.

The record says which half it is. **The kill criterion (`docs/09`) cannot be evaluated until the owner has
run the second half.** P3c does not depend on it, because the panel does not change if `L` does.

## 6. Out of scope

MIDI input, clock in and out, program change, the second port (P4). Save and load. Track zoom, the
encoders and every workflow past the five `octoface` has (Waves 1 to 5). The tempo encoder (the page holds a
tempo number until then). Ableton routing documents (P4). Any change to `octocore`, `octoffi` or `octorun`.
D0.

## 7. Questions for the owner

1. **Run S1 on your machine** when you have a spare hour (a loopback is enough; Live adds half an hour).
   Until then A3 and the lookahead are unset and the kill criterion is untested.
2. **Tokens.** P3c proposes the colour values and the flash rate with a screenshot. The typeface stays
   provisional until the Curator's specimens.
3. **D-P3-4.** Is a first panel of the matrix plus the ten controls the controller acts on the right size, or do you want
   every control drawn now?

## 8. Risks

| Risk | Consequence | Mitigation |
|---|---|---|
| S1 cannot be run here | The riskiest number is unmeasured when P3 merges | Said plainly in every record; the harness is ready; P3c does not depend on it |
| The headless browser differs from the owner's Chrome | S2 passes here and fails there | The S2 page is committed and runs in CI on the runner's Chromium; the owner can run it |
| Hidden tabs cannot be reproduced headless | The hidden-tab claim is untested | The record says so; a headed run under a virtual display if one exists |
| A browser-only test can be deleted unnoticed | The ratchet leaks | D-P3-8 request; the crate's native tests are floored |
| The `unsafe_code` lint is allowed in two modules | A reviewer sees it as precedent | The export shims hold the attribute and no `unsafe` keyword; the allocator is behind a feature the shipped module lacks; a test pins both (ADR-0008 3) |
| The scope grows to the whole panel | P3 never ends | D-P3-4, and `docs/09`'s "the one thing to protect" |

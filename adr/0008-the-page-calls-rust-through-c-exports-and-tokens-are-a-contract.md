# ADR 0008: The page calls Rust through C exports, in a crate of its own, and the design tokens become a contract

## Status

Proposed on 2026-10-01 by the Conductor. It follows from SPEC-0002 r1 (approved by the owner,
"Approve", 2026-09-30 15:06 Paris) and from the owner's "Start p3" of 2026-10-01 12:00 Paris.
SPEC-0002 approved P3 and left two things for it to decide: how the page calls Rust
(`specs/SPEC-0002/tech.md` section 2) and the tokens contract, which `docs/05-design-system.md`
section 2 says is "planned with the app, P3, by ADR". It takes effect when the owner merges the
pull request that carries it. The code and the contract file it authorises arrive in P3b and
P3c. The plan is `specs/SPEC-0002/p3-plan.md`.

## Context

The engine already compiles to WebAssembly. `just wasm-smoke` loads `octoffi` in Node. The panel
controller (`octoface`, merged in #23) is a Rust library. The architecture puts both in one
AudioWorklet (`tech.md` section 2), so the page needs a WebAssembly module that holds the engine
*and* the controller, and a rule for how JavaScript talks to it.

Three facts bear on that, each measured in this session. The run is in
`handoffs/evidence/p3-browser-probe/` (headless Chromium 141.0.7390.37, a localhost page):

| In an `AudioWorkletGlobalScope` | Result |
|---|---|
| `TextDecoder`, `TextEncoder` | Absent |
| `fetch`, `crypto`, `performance`, `setTimeout`, `SharedArrayBuffer` | Absent. `Date`, `Atomics` and `WebAssembly` are present |
| A compiled `WebAssembly.Module` passed in `processorOptions` | Arrives as a `WebAssembly.Module` and instantiates synchronously, with no imports |
| Web MIDI, with the `midi` permission granted by the test driver | `NotAllowedError`: not exercisable in headless Chromium |

Two more things are already in the repository:

- `octoffi` holds the engine and not the controller, is frozen (SPEC-0002 D5) and is the Conductor's.
  Its header is hand-maintained and checked against the Rust layout by a C compiler in a test.
  Adding the controller to it would unfreeze it.
- `docs/02-architecture.md` says `octoffi` is "the only place `unsafe` appears". The
  allocation test in `octoffi` already carries one `unsafe impl GlobalAlloc` for counting.

## Decision

### 1. The page calls Rust through plain C exports, and not through `wasm-bindgen`

The module has no imports and exports functions that take and return integers and floats, plus
the addresses of a few fixed buffers the page reads from the module's memory. The page fetches
and compiles the module on the main thread, hands the compiled `WebAssembly.Module` to the
worklet in `processorOptions`, and the worklet instantiates it synchronously.

The reason is the table above. The worklet has no way to fetch bytes and no `TextDecoder`, which
the usual `wasm-bindgen` glue relies on for strings (not tested here; the claim is about the
worklet, which was). C exports need no glue in the worklet at all, and the smoke test already
uses this style (`harness/wasm/smoke.mjs`). Strings cross once, at start, as bytes the page
writes into a scratch buffer the module exposes.

The export surface is named `octoweb-abi/1`. It is not a contract file: its only consumers are
the page and the module, both in the Forge's zone, and a native test and a browser test pin it.
A breaking change bumps the number the module reports and the tests that read it.

### 2. The module is a new crate, `octoweb`, in `apps/web/engine/`

It depends on `octocore` and `octoface` and contains no sequencing logic and no panel logic. It
joins the two: it builds the controller's page view from `Engine::grid`, applies the commands
the controller returns to the engine, renders blocks, and exposes events and the LED frame as
bytes. It sits inside `apps/web/` so that it is in the Forge's zone (`CODEOWNERS` already
lists `/apps/web/**`) and no role brief, ownership map or `CODEOWNERS` line has to change.
`octoffi` stays frozen. Adding a crate is Medium tier (`AGENTS.md`).

### 3. Memory, and where `unsafe` is allowed

The crate is `#![deny(unsafe_code)]`, and it holds no `unsafe` block or function in the shipped module.
It cannot be `forbid`: an exported C function needs `#[no_mangle]`, and rustc's `unsafe_code` lint counts
that attribute as unsafe code, in edition 2021 and in 2024 with `#[unsafe(no_mangle)]` (checked with rustc
1.95 for the wasm32 target; `forbid` refuses the file, `deny` with an `allow` on the item accepts it). So
two modules, and only two, may allow the lint:

- `exports.rs`: the export shims. Each is one line that calls safe code in another module. It holds the
  attribute and no `unsafe` keyword, and a test fails if the keyword appears in it or in any other file
  of the shipped module.
- `measure.rs`, behind the cargo feature `measure`: a counting global allocator (`unsafe impl GlobalAlloc`,
  forwarding to `System`, as `octoffi`'s test does) and an export that reports the count. It exists for
  spike S2 and for a native test that asserts `render` allocates nothing. It is not in the shipped module.

The state lives in a thread-local that holds a `Box`ed host. The buffers the page reads are fields of it,
so reading their addresses needs no `unsafe`. `docs/02`'s sentence that `octoffi` is "the only place `unsafe`
appears" stays true of `unsafe` blocks in the shipped build; the Scribe adjusts it to say where the lint is
allowed (a request is filed).

### 4. The page is the host: transport and tempo are its commands

The engine takes `Play`, `Stop` and the tempo as host inputs (`RenderContext.bpm` and
`.playing`). `octoface` handles the Stop key (p049) and not yet the Play key. Until the
Panelwright builds the manual's transport workflow, the page maps a press of `transport.play`
to the engine's `Play` command through an export of its own, and says so in a comment and in
the plan. This is a stopgap, not a behaviour the manual describes. The tempo is a number the
page keeps and passes with each render. The tempo encoder's workflow is Wave 1.

### 5. The engine's output reaches Web MIDI by the scheme in `tech.md` section 3

Unchanged: the worklet renders blocks of 128 frames, stamps each event with an audio-clock
time, the page converts with `getOutputTimestamp()` and sends with
`MIDIOutput.send(data, timestamp)` at least a lookahead `L` early. `L` defaults to 30 ms, and
spike S1 sets it. Because the worklet has no `performance` clock, the worklet cannot time
itself with a high-resolution clock. Spike S2 times the same module on the main thread, where
`performance.now()` exists, and in the worklet in batches with `Date`. Its record says that.

### 6. `contracts/design.tokens.json` is a contract

P3c adds `contracts/design.tokens.json` and `contracts/design.tokens.schema.json` (JSON Schema
2020-12, `"schema": "design.tokens/1"`). The app's stylesheet is generated from the file, and no
colour, space or type value is written anywhere else in `apps/web/`.

| Group | Holds |
|---|---|
| `colour` | `led.red`, `led.green`, `led.orange`, `led.off`, `ink`, `ink.quiet`, `surface`, `focus`, `focus.inner`, each `#rrggbb`. The three LED roles are roles and not hues: the manual's other colourways (p004) are a later table |
| `space` | `0` to `7`, the eight values in `docs/05` section 2.2. No ninth |
| `type` | `family`, a small set of sizes, and `measure` (the 68-character line length) |
| `flash` | `period_ms`, the LED flash period |

There is no `radius`, `shadow`, `gradient` or `easing` group, so the rules S1, S4, S9, S15 and S16
have nothing to be satisfied by.

A token whose value depends on something the manual does not say carries `pending`, the same
shape as in `controls.json`: `{ "question": "Qnn or tech.md 9.n", "note": "…" }`. Three begin
that way: `flash.period_ms` (the flash rate is `tech.md` section 9 item 2), the `shine` state's
drawing (Q03; it is drawn from `ink`, not from a new colour) and `type.family` (the Curator's
specimens, `docs/05` section 2.4). A provisional type family may not outlive the first Wave 1
demonstration.

The contract test, in P3c, asserts rules instead of values, so a change of hue passes and a
change that breaks a rule does not:

1. Every colour is `#rrggbb`. Every value in `space` is one of the eight.
2. `ink` and `ink.quiet` against `surface`: contrast ratio at least 7:1 (`docs/05` section 5).
3. `led.off` differs from each lit LED by more than hue: contrast ratio at least 3:1 (the WCAG 2.1
   non-text figure) between `led.off` and each lit role, so lit-versus-off survives a
   colour-vision difference.
4. No colour is within ΔE 3 of the AI-design cluster in rule S8, or carries a suppression
   comment that says why.
5. `focus` and `focus.inner` are neutral (equal red, green and blue). At least one of them has
   contrast of at least 3:1 against `surface`, and at least one against each lit LED role, so the
   ring is visible wherever it falls (`docs/05` section 5).
6. `flash.period_ms` gives fewer than three flashes a second (WCAG 2.3.1), or the file says the
   rate is a finding.
7. The only keys with a hue are the three `led.*` roles. `ink`, `ink.quiet`, `surface` and `focus*`
   are neutral.

The values are the Curator's choice and the owner's to ratify, by merging P3c. They are
proposed there with the UI sketch, as `docs/05` says.

### 7. Risk tiers and the series

| PR | What | Tier | Why |
|---|---|---|---|
| P3a (this one) | ADR-0008 and `specs/SPEC-0002/p3-plan.md`. Documents only | High | It authorises a contract (`contracts/**`). The tier follows ADR-0007 |
| P3b | `apps/web/engine` (the crate), the worklet host, the Web MIDI scheduler, spikes S2 and S1, a CI job | Medium | A new crate, CI and `justfile` edits. No contract changes |
| P3c | `apps/web` UI, `contracts/design.tokens.json` and its schema | High | A contract file. It needs P3b's crate |

P3b carries P3a's commits if P3a has not merged when it is opened, and P3c carries P3b's, as P2b
carried P2a's. Merge in order. The agent merges none of them.

## Not decided here

- **The typeface.** The Curator chooses one family by ADR after three specimens are rendered at
  real size beside the panel (`docs/00-north-star.md` section 3.2). Until then `type.family` is
  provisional.
- **The chase-light.** The manual describes a Red chase-light in the matrix, Orange when a track
  sends an MCC (p017, p052), and says a skipped step is ignored (p014). It does not say how the
  chase-light is drawn over a lit step in Page view. That is a Panelwright workflow and a
  question for the owner (a request is filed). P3c draws no chase-light.
- **The lookahead `L` and the jitter budget (A3).** Spike S1 measures them. The owner sets A3.
  Only the scheduling side can be measured in a container (the plan says which parts need the
  owner's machine).
- **D0**, the tick length. Still open.
- MIDI input, clock in and out, the second port and saving (P4 and Wave 1).

## Consequences

- The worklet needs no JavaScript glue and no polyfill. The cost is a hand-written export list
  and a native test that keeps it honest. A header-style listing in the plan is kept in step by
  that test.
- The shipped module has no `unsafe` block. The lint is allowed in the export shims and, behind a
  feature, in a counting allocator. Tests keep it that way. The Scribe adjusts `docs/02`.
- The tokens are a contract before any colour is chosen, so the first colour the page uses is
  already under a rule that can fail.
- If spike S1 shows Web MIDI too loose for the gear at any lookahead the owner will accept,
  `docs/09` section "Kill criteria" applies: stop and rescope before P3c.

## Amendments

### Amendment 1 (2026-10-01, Conductor, found in P3b): Stop does not call `MIDIOutput.clear()`

Decision 5 kept the scheme in `specs/SPEC-0002/tech.md` section 3, whose step 4 says that on Stop the
page calls `MIDIOutput.clear()` and then sends ALL NOTES OFF. P3b's property test for observable E3
(after Stop the receiver holds no note) failed that scheme in 6 of 8 generated cases, for two reasons:

1. **`clear()` leaves notes on.** It drops everything queued and not yet sent, including Note Offs the
   engine has already counted as sent. The engine's flush then covers only the notes it still thinks
   are sounding, so those notes stay on at a receiver that does not honour CC 123 (ALL NOTES OFF).
   The trace is in `handoffs/evidence/p3b-ts-red-run.txt` (seed 1148: two Note Offs queued 17 ms
   ahead are cleared, two notes stay on).
2. **Chromium does not implement it.** In Chromium 141 `MIDIOutput.prototype` has `send` and nothing
   else (`handoffs/evidence/p3-browser-probe/midioutput-probe.txt`). The step could not run in the
   browser the product targets first.

What replaces it, in `apps/web/src/midi-out.ts`:

- `clear()` is never called.
- Timestamps given to one output never go backwards (within a 50 ms window, beyond which the clocks
  are taken to have jumped and the new time is believed). The page re-reads the audio-to-page clock
  map for each batch and it moves by a millisecond or two; an output delivers by timestamp, so a flush
  Note Off one sample after a Note On could otherwise be stamped before it and the note would stay on.

Stop then works like this: what the page has already queued plays out, at most one lookahead `L` plus
the clock jitter (30 ms by default), and the engine's flush (a Note Off for every sounding note, then CC 123,
ALL NOTES OFF) is stamped after it. **The cost is up to `L` of notes after the Stop key. The Stop
command itself is not delayed.** E3 now reads "after Stop the receiver holds no note, with or without
`clear()` in the browser, and the scheduler does not call it".

Not done: a page-side ledger of notes sent, which would let the page call `clear()` where it exists and
re-send the Note Offs it dropped. No browser the product targets has `clear()`, and the path could not
be tested against a real one. If one does, and the tail matters, that is the way to tighten it.

The owner approved `tech.md` section 3 as part of SPEC-0002 r1. This amendment departs from its step 4
on evidence, and the departure is visible here and in the P3 pull requests. `tech.md` step 4 carries a
pointer to this amendment.

### Amendment 2 (2026-10-01, Conductor, found in P3c): what the tokens file holds, and where its rules are asserted

Decision 6 listed the groups. Writing the file, and the panel that reads it, showed four places where the list is
short or the placement can be better. Each is small; each is written down because `contracts/` changes only by ADR.

1. **A `stroke` group**: `hairline` (1), `focus` (2) and `focus_inner` (1). `docs/05` section 5 sets the focus ring at
   2 px with a 1 px contrasting inner stroke, and the strip and the key outlines are hairlines. None of 1 and 2 is on
   the space scale (base 4, smallest step 4), and "no value written outside the tokens file" would otherwise force
   them into the stylesheet as literals. Widths are not colour, space or type, so rule 2 of CLAUDE.md is not touched.
2. **`flash.duty_percent`** (50, `pending`, `tech.md` 9.2). The keyframes of the one flashing LED class need the point
   at which the light goes off, and it is a number the manual does not give.
3. **`flash.shine`** is the name of a neutral colour token (`ink` or `ink.quiet`) that the shine state is ringed in
   (decision 6 said the shine drawing is "drawn from `ink`"; this is where that is recorded). It carries `pending` Q03.
4. **The rules are asserted by a Rust test**, `apps/web/engine/tests/tokens.rs`, and not by a Node test. The ratchet
   (`harness/baseline.txt`) is read from `cargo test` output, so a Rust test is floored by name and a Node test is not
   (D-P3-8). The test also runs each rule against a document made to break it, so no rule can sit there unable to
   fail, and it checks its own ΔE (CIEDE2000, the stricter of the usual measures) against four pairs from the
   published test data. Three rules were added to the seven of decision 6, and each only tightens: the three lit roles
   are at least ΔE 15 apart and none is grey (T9); the type face is not one of the defaults rule S11 refuses (T10);
   the measure is 68 (T11).

The schema closes each group (`additionalProperties: false`, the colour group lists its nine keys, the space group its
eight values), so a tenth colour or a ninth space step is a change to the schema and therefore to this ADR.

`status` is `candidate`. The owner ratifies the values by merging P3c (decision 6); the line is changed to `ratified`
in the next documents pull request.

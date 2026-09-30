# 02: Architecture

**Owner:** Conductor. **Gate:** `verify:arch`, `verify:timing`.

The product is a web sequencer (`adr/0006-web-sequencer-supersedes-the-rooms-scope.md`,
`specs/SPEC-0002/`). **Planned** marks a piece that does not exist yet; its P-number is the
phase that builds it, in the series in `specs/SPEC-0002/README.md`.

---

## 1. The decision that shapes everything

**The instrument is a static web app. Ableton Live is a client of it over MIDI, not its host.**
The engine is Rust compiled to WebAssembly; the page draws the panel; Web MIDI carries notes and
clock to external instruments and to Live. No backend, no account. v1 rendered the whole
instrument in a Max for Live `jweb` view driven by ES5 JavaScript; v2 reopens `jweb` only as one
route into Live, on evidence (Tier 2 below, spike S4).

```
 Browser tab (HTTPS, or localhost)
 +---------------------------------------------------------+
 | UI (TypeScript, SVG)  input events in, LED frames out   |
 | MIDI layer (main)     MIDIOutput.send(bytes, time);     |
 |                       MIDIInput messages and clock in   |
 | AudioWorklet (clock)  octoface --> octocore, per 128    |
 |                       frames, silent output             |
 +---------------------------------------------------------+
     | Web MIDI, ports 1 and 2       ^ clock, notes
 external instruments          Ableton Live (virtual port)
```

**Built:** `octocore`, `octorun`, `octoffi` (frozen), the WebAssembly smoke test, the gates and
the ratchet. **Planned:** `octoface` and `contracts/controls.json` (P2), the web app and Web
MIDI output (P3), MIDI in, clock and the Ableton guide (P4), the workflow waves (P5 onward).

**Why Rust:** no garbage collector, so no unbounded pause on the audio thread, and one
implementation. The same source builds to WebAssembly for the page and to native code for the
conformance suite and `octorun`; `just wasm-smoke` plays the golden patterns through the
WebAssembly build and checks the event log against the native hashes, byte for byte. Today it
plays three of the five: `phrases` and `mcc_and_transport` need more than the frozen C
interface offers, and close with the `wasm-bindgen` route in P3.

**Browsers.** Chrome, Edge and Opera; Firefox best effort; Safari and iOS run the UI without
MIDI, which they lack (SPEC-0002 D4).

**Ableton.** Tier 1 (planned, P4): the page sends and receives MIDI through a virtual port (the
IAC Driver on macOS, loopMIDI on Windows) and can follow Live's clock; no code on Live's side.
Tier 2 (spike S4 only): a Max for Live device in `hosts/m4l` shows the page in `jweb` and writes
MIDI to the track; whether `jweb` exposes Web MIDI is not documented, so the design does not rely
on it. Tier 3 (Link needs a local helper) is not scoped.

---

## 2. Threading model

Two threads and no shared memory. Their contract is more important than their contents.

| Thread | Owns | Rule |
|---|---|---|
| **AudioWorklet** (planned, P3) | `octoface`, `octocore`, event rendering with audio-clock times | The audio clock is the clock. The engine does not allocate on its render path after init (spike S2 checks it in a worklet) |
| **Page** (main) | UI, Web MIDI, settings, saving | Does not keep time: main-thread timers are throttled in background tabs and jittery |

They talk with `postMessage`: input events go in; events, LED frames and displays come out. That
avoids `SharedArrayBuffer` and the cross-origin isolation headers it needs, which static hosts
often cannot send. **Rule:** the page never reaches into the engine. A UI feature that seems to
need an engine query is a missing field in the snapshot or the LED frame. The worklet's output
is silent, but the node must be connected to the destination or the browser may not run it.
Spikes S1 and S2 check this, and hidden-tab behaviour, for 30 minutes.

**The native ring and snapshot (frozen).** For a native host with separate audio and render
threads, the engine has a wait-free command ring and a triple-buffered snapshot (SPEC-0001 O6:
`octocore::ring`, `triple`, `link`). A single-threaded WebAssembly engine does not need them, so
they are frozen (SPEC-0002 D5): no new work, and the tests stay. The ring holds 1024 commands;
when it is full the oldest command is dropped and a counter increments (dropping input is bad,
blocking the audio thread is worse). `Engine::render` applies at most 256 per call and publishes a
`#[repr(C)]` snapshot of 8,408 bytes at the end, through three slots and one atomic swap on each
side, so neither waits and the reader never sees a torn state.

---

## 3. Module boundaries

Ownership is enforced by the build, not by convention. Each crate and each app target has
exactly one owning role, and cross-boundary changes require the owning role's sign-off in the
merge queue.

### `crates/octocore` (Metronome)

Pure sequencing. **Has no I/O, no time source, and no dependencies on the platform.** Its
interface, in outline:

```rust
impl Engine {
    pub fn new(seed: u64) -> Self;
    pub fn handle_command(&mut self, cmd: Command);
    pub fn render(&mut self, ctx: &RenderContext, out: &mut EventBuffer); // ctx.buffer_len samples
    pub fn snapshot(&self, into: &mut Snapshot);
}
```

It is told how many samples to advance, at what tempo and sample rate, and appends the events
due in that window, each with a sample offset. It never asks what time it is, so it is
deterministic and can be driven by a synthetic clock.

### `crates/octoface` (planned, P2)

The panel controller, the layer the product is missing: the engine accepts `ButtonDown`,
`ButtonUp` and `EncoderTurn` today and ignores them. A pure state machine. In: button down and
up, encoder turns, and time as a millisecond count the caller passes in, so a fixture can replay
a session exactly. Out: engine commands, an LED frame (each LED is `{ Off | Red | Green | Orange,
Steady | Flash }`) and the displays. Each manual workflow becomes fixtures cited to its page.
Design: `specs/SPEC-0002/tech.md` section 4.

### `crates/octoffi` (Conductor, frozen)

The C ABI surface over `octocore`, hand-written and tiny, and **the only place `unsafe`
appears** (`octocore` is `#![forbid(unsafe_code)]`). Frozen (SPEC-0002 D5): no new work, and the
tests stay, including the one that checks `octoffi.h` against the Rust sizes and offsets with a C
compiler. The WebAssembly smoke test loads it today. How the page calls Rust, these exports or
`wasm-bindgen`, is decided in P3.

### The rest

- **`crates/octorun`** (Conductor): the headless runner. A pattern in, an event log and a
  Standard MIDI File out, byte for byte the same every run.
- **`apps/web`** (planned, P3): TypeScript (strict), SVG, no UI framework, static files. Loads
  the engine in the worklet, draws the matrix and LEDs from `contracts/controls.json` (planned,
  P2, by ADR), owns the Web MIDI layer and settings. No sequencing logic.
- **`hosts/m4l`** (Metronome): Tier 2 only. An empty scaffold today.
- **`xtask/`, `harness/`** (Referee): the gates, the ratchet, the report. A gate that is not
  built reports `not_implemented`, never a pass.

---

## 4. State model

One source of truth: `octocore::Grid`. Everything else is a projection.

```
Grid
├── banks: [Bank; 10]                       (the manual reads as nine; open finding, see STATE.md)
│   └── Bank { pages: [Page; 16] }
│       └── Page { tracks: [Track; 10], scale, pitch_offset, velocity_factor, mute_pattern[10], … }
│           └── Track { steps: [Step; 16], vel pit len sta pos dir amt grv mcc mch, chain, mute, solo, … }
│               └── Step { on, skip, vel pit len sta grv mcc, phrase, chord, … }
├── phrases: [Phrase; 48]
├── page_sets: [Vec<PageRef>; 16]        (planned: not in the code yet)
├── active page, mode, tempo, …
└── (in `Engine`, not `Grid`) runtime per track: { step_phase, pos, ping_dir, chain_state }
```

This mirrors the v1 schema, which was one of the few things v1 got right, and the hardware's own
model. `runtime` is kept apart from the persisted structure so that save and load is a clean
serialisation of the rest.

**Persistence (planned):** `Command::LoadState` is a no-op today and nothing serialises the
grid. The design is a versioned JSON state, `octopus-state/1`, with a schema in `contracts/`,
autosaved to IndexedDB and exportable as a file (`specs/SPEC-0002/tech.md` section 7). Every
schema change ships a migration and a fixture, and `verify:persistence` (`not_implemented`
today) round-trips every fixture from every historical version.

---

## 5. Determinism

The engine is deterministic given `(seed, command sequence, render calls)`. This is load-bearing
for verification, because a non-deterministic sequencer cannot have golden tests. The panel
controller is deterministic given its inputs and the millisecond clock the caller passes in.

- All randomness goes through the engine's seeded splitmix64 (`crates/octocore/src/rng.rs`).
  `Math.random()` equivalents are banned; `verify:arch` is meant to grep for them and is not
  built yet.
- Random draws happen in a fixed order per tick, track 9 first and track 0 last. The Octopus
  processes tracks top-down (the manual's note on effector feed order) and we preserve that
  because it is observable in the effector's behaviour.
- No floating-point accumulation for musical position. The tick counter is a `u64` and the step
  phase is an integer. The sample clock that times each tick is still an `f64` (`sample_clock`,
  `next_tick_due`), so for it the rule is a target, not yet a fact.
- The tick is 192 per quarter note today; the manual reads as 192 per whole note. This is D0
  (`WENGE-0012`), still open. The ratio of engine ticks to MIDI clock pulses waits on it.

---

## 6. Timing and latency

N5: timing beats pixels. The engine stamps each event with a sample offset; the page turns that
into a MIDI timestamp (`specs/SPEC-0002/tech.md` section 3). Each `process()` call renders 128
frames and returns events with audio-clock times. The page converts audio time to page time with
`AudioContext.getOutputTimestamp()` and sends each event with `MIDIOutput.send(data, timestamp)`
at least a lookahead `L` early; `L` is a tunable, default 30 ms, set by spike S1. On Stop it
calls `MIDIOutput.clear()` for anything queued and unsent (whether Chrome implements it is
checked in spike S1), then sends ALL NOTES OFF.

`L` and the engine's own lookahead add, and are reported together. The engine steps up to 12
ticks ahead of the audio (`MAX_EARLY_TICKS`) and applies an edit at the start of the next
`render`, so an edit to a step less than 12 ticks from firing is heard on the following pass:
about 31 ms at 120 BPM at the current tick length. The snapshot is published after that
stepping, so `transport.tick` and the playheads run about 13 ticks ahead of what is heard. A
display that must line up with the sound, the chase-light, delays itself by that amount plus `L`.

**Budgets are not set yet.** Browser-to-instrument jitter (A3) is set by spike S1 and clock
follow (A4) by spike S3, by the owner, before the wave that needs them
(`specs/SPEC-0002/product.md` section 9). The 120 microsecond figure in
[Sequencer Core §7](03-sequencer-core.md) was for a plugin inside Live and does not apply.

**Clock (planned, P4).** MIDI clock is 24 pulses per quarter note, so engine ticks per pulse are
8 today and 2 if D0 goes the other way. The master output can be built first because it divides
a known tick. In slave mode the app estimates tempo and phase from incoming `0xF8` pulses with a
small pure function, tested on a simulated jittery clock.

---

## 7. Build and tooling

One entry point: `just`. If a task cannot be run by an agent with a single `just` verb, it does
not exist as far as the operating model is concerned.

```
just verify         # every gate; this is what CI runs (just gate <name> for one)
just run <pattern>  # headless run (octorun): event log and .mid
just wasm-smoke     # build octoffi for wasm32, play the goldens in Node
```

`just verify` must complete in under **four minutes**. This is a hard requirement, not an
aspiration. A verification suite that takes twenty minutes gets skipped, and a skipped gate is
not a gate. If the suite slows down, the fix is to make it faster, not to run it less.

---

## 8. Where v1's code goes

`beaudesign/BenoQs-Sequencer` is archived, not merged. Two things are salvaged, both by
transcription rather than import:

1. **`octopus_scheduler.js`.** It contains real Octopus semantics that someone clearly derived
   from the manual with care: the GRV shuffle delay table, the chord strum timing table,
   ping-pong direction handling, chain member traversal. These are ported to Rust as
   `octocore::tables` **with a manual page citation on every constant**. Any table entry that
   cannot be cited is marked `unverified` and gets a conformance fixture written against the
   manual before it is trusted.

2. **`octopus_schema.js` defaults.** The default grid, default PIT values per track, and
   structural shape. Ported to `Grid::default_grid` in `octocore::domain`, again with citations.

Everything else (the Max patcher, `jweb` bridge, HTML mock, the adapter, the CSS) is deleted.
Not refactored. Deleted. It encodes the wrong architecture and keeping it around guarantees
someone reaches for it.

The tests in v1 (`tests/*.test.js`) are read for *ideas about what to test*, then rewritten.
Their homegrown runner is replaced by `cargo test` plus the harness.

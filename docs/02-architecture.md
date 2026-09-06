# 02: Architecture

**Owner:** Conductor. **Gate:** `verify:arch`, `verify:timing`.

---

## 1. The decision that shapes everything

v1 tried to render an instrument in a WebKit view (`jweb`) inside Max for Live, driven
by ES5 JavaScript in Max's interpreter, with state in a Max `Dict`. That stack has a
hard ceiling well below what this project needs: no GPU control, no colour management,
unpredictable compositing, no real-time thread, and a language that cannot express a
sample-accurate scheduler safely.

v2 inverts the relationship. **The instrument is a native application. Live is a
client of it, not a host for it.**

```
                 ┌──────────────────────────────────────────┐
                 │            OctoStandalone.app            │
                 │   (or the same binary, hosted as a       │
                 │    plugin view by VST3 / AUv2)           │
                 │                                          │
   ┌─────────┐   │  ┌────────────┐        ┌──────────────┐  │
   │ Ableton │◄──┼─►│  hosts/    │◄──────►│  octocore    │  │
   │ Live 12 │   │  │  vst3, au  │  ABI   │  (Rust)      │  │
   └─────────┘   │  └────────────┘        └──────┬───────┘  │
        ▲        │                               │ snapshot │
        │        │  ┌────────────┐        ┌──────▼───────┐  │
        │  MIDI  │  │ OctoShell  │◄──────►│  OctoPanel   │  │
        └────────┼──│  (Swift)   │  probe │  (Metal)     │  │
        + Link   │  └─────┬──────┘        └──────▲───────┘  │
                 │        │                      │          │
                 │        │  room model    probe │ + IR     │
                 │        └────►┌──────────────┐─┘          │
                 │              │  octoroom    │            │
                 │              │  (Rust+MSL)  │            │
                 │              └──────────────┘            │
                 └──────────────────────────────────────────┘
```

### Host formats

Live 12 on macOS loads AU (v2 and v3, since Live 11.3) and VST3. It does not load
CLAP. We therefore ship **VST3 and AUv2** from one wrapper codebase, plus a
standalone with Ableton Link. AUv3 is not worth the sandbox cost for a desktop-only
product.

`hosts/m4l/Octopus.amxd` remains, reduced to what Max is actually good at: transport
and clock bridging, MIDI routing, and Live API access for tempo and time signature.
It contains no UI and no sequencing logic. Roughly 200 lines. If it grows past 400,
something has been put in the wrong place.

### Why Rust for the core

Three properties the core needs that are hard to get elsewhere: no garbage collector
(so no unbounded pause on the audio thread), a type system that can encode "this
buffer is owned by the audio thread right now", and a single implementation that
compiles to a native staticlib for the app, to WASM for the browser-based harness and
documentation, and to a test binary for the conformance suite. One implementation,
three consumers, no divergence.

---

## 2. Threading model

Four threads. Their contract is more important than their contents.

| Thread | Priority | May allocate | May block | Owns |
|---|---|---|---|---|
| **Audio** | real-time | never | never | `octocore` tick, MIDI emission, convolution |
| **Render** | display-link | only at init | never | Metal command encoding, panel state read |
| **Room** | background | yes | yes | ray tracing, probe baking, IR computation |
| **Main** | normal | yes | yes | AppKit, input events, file I/O, shell |

### Audio to Render: triple-buffered snapshot

The renderer must never see a torn sequencer state and must never make the audio
thread wait. Use a three-slot snapshot buffer with an atomic index:

- Audio thread writes into the slot that is neither `published` nor `reading`, then
  releases it by a single `store(Release)` on `published`.
- Render thread does `exchange` on `published` to claim it as `reading`.
- No locks, no allocation, single writer, single reader, wait-free on both sides.

The snapshot is a flat POD struct of about 6 KB: LED states for 160+ controls,
encoder angles, playhead positions per track, transport state, and a generation
counter. It is `#[repr(C)]` and shared with Swift by pointer, never copied across
the FFI per frame.

**Rule:** the render thread never calls into `octocore` for anything. It reads the
snapshot and nothing else. Any renderer feature that seems to need a core query is
a missing field on the snapshot.

### Main to Audio: command queue

Input events become commands on a bounded lock-free SPSC ring (`crossbeam`
`ArrayQueue`, capacity 1024). Commands are POD and self-contained. If the ring is
full, the *oldest* command is dropped and a counter increments, which is surfaced in
the debug overlay. Dropping input is bad; blocking the audio thread is worse.

### Room to Render: double-buffered probe

Room computation takes hundreds of milliseconds and must never stall a frame. The
room thread writes into an offscreen probe texture and IR buffer, then atomically
swaps. The renderer cross-fades between the outgoing and incoming probe over the
transition duration declared in the motion registry. See
[Shell §5](06-shell-and-rooms.md) for the transition choreography.

---

## 3. Module boundaries

Ownership is enforced by the build, not by convention. Each crate and each app target
has exactly one owning role, and cross-boundary changes require the owning role's
sign-off in the merge queue.

### `crates/octocore` (Metronome)

Pure sequencing. **Has no I/O, no time source, and no dependencies on the platform.**
Its entire interface is:

```rust
pub struct Core { /* … */ }

impl Core {
    pub fn new(seed: u64) -> Self;
    pub fn apply(&mut self, cmd: Command);              // from the command ring
    pub fn tick(&mut self, ppqn_pos: u64, out: &mut EventSink);
    pub fn snapshot(&self, into: &mut Snapshot);
}
```

That is it. It is given a position and asked what should happen. It never asks what
time it is. This makes it trivially testable, trivially deterministic, and means the
conformance suite can run a thousand simulated hours in seconds.

### `crates/octoroom` (Sceneshaper)

Room model to (probe, IR, grade). Also has no I/O beyond reading a room description
struct. Contains the acoustic ray tracer and the probe baker. Its Metal compute
kernels live here as `.metal` sources compiled into a library that the app loads.

### `crates/octoffi` (Conductor)

The C ABI surface. Hand-written, tiny, and **the only place `unsafe` appears** outside
the renderer. Every function here has a corresponding assertion in `verify:arch`
that its Swift declaration and its Rust declaration agree, generated by parsing both.
FFI drift is the most common source of memory corruption in mixed-language projects
and it is fully preventable by a build-time check.

### `apps/OctoPanel` (Forge)

Metal renderer. Reads the compiled truth file, the snapshot, and the probe. Emits
frames. Contains no application logic and no knowledge of what a "track" means. It
knows about *controls*, *materials*, and *state values*.

### `apps/OctoShell` (Loom, assigned to Fable)

Room dropdown, rehearsal rooms, settings. Owns no rendering of the panel and no
sequencing. Communicates with the panel only by requesting a room change and reading
the current room. Deliberately thin: see [Shell](06-shell-and-rooms.md).

### `hosts/` (Metronome)

Plugin wrappers. Parameter exposure, state save/restore, transport sync. Contains
zero business logic; it translates host callbacks into `Command`s and reads
`EventSink` back out.

---

## 4. State model

One source of truth: `octocore::State`. Everything else is a projection.

```
State
├── grid: [Bank; 10]
│   └── Bank { pages: [Page; 16] }
│       └── Page {
│             tracks: [Track; 10],
│             scale, pit_offset, vel_factor, mute_pattern[10], …
│           }
│           └── Track {
│                 steps: [Step; 16],
│                 vel pit len sta pos dir amt grv mcc mch,
│                 chain, mute, solo, paused, …
│               }
│               └── Step { on, skip, vel pit len sta grv mcc, phrase, chord, … }
├── page_sets: [Vec<PageRef>; 16]
├── active: { bank, page, track, mode, … }
└── runtime: per-track { accum, pos, ping_dir, chain_state }
```

This mirrors the v1 schema, which was one of the few things v1 got right, and it
mirrors the hardware's own model (grid of banks of pages of tracks of steps).
`runtime` is separated from the persisted structure so that save/load is a clean
serialisation of everything above it.

**Persistence:** a versioned binary format with a JSON debug encoder. Every schema
change ships a migration and a fixture. `verify:persistence` round-trips every
fixture from every historical version.

---

## 5. Determinism

The core is deterministic given `(seed, command sequence, tick sequence)`. This is
load-bearing for verification, because a non-deterministic sequencer cannot have
golden tests and a non-deterministic renderer cannot have pixel tests.

Requirements:

- All randomness goes through `Core::rng`, a seeded PCG64. `Math.random()` equivalents
  are banned; `verify:arch` greps for them.
- Random draws happen in a fixed order per tick, defined by track index ascending.
  The Octopus itself processes tracks top-down (track 9 first, track 0 last, per the
  manual's note on effector feed order), and we preserve that because it is
  observable in the effector's behaviour.
- No floating-point accumulation for musical position. Position is `u64` ticks at
  192 PPQN. Float appears only when converting to audio-sample offsets, once, at the
  boundary.
- The renderer is deterministic given `(snapshot, probe, camera, frame_index)`.
  No `time()` calls in shaders. Animation is driven by an explicit `t` passed in,
  which is what makes offline frame-stepped capture possible.

---

## 6. Latency budget

Input-to-photon, measured with a high-speed camera against a photodiode rig in the
lab and with `CAMetalLayer` presentation timestamps in CI.

| Segment | Budget | Notes |
|---|---|---|
| HID event to main thread | 2.0 ms | macOS event tap |
| Main to command ring | 0.1 ms | wait-free push |
| Core apply, next tick | 1.3 ms | worst case at 192 PPQN, 300 BPM |
| Snapshot publish to render claim | 0.1 ms | atomic exchange |
| Render encode | 2.5 ms | at 120 Hz, budget is 8.33 ms; we target 30% headroom |
| Compositor to photon | 1.5 ms | ProMotion, direct-to-display |
| **Total** | **7.5 ms** | gate: ≤ 8.0 ms p95, ≤ 12 ms max |

MIDI timing is budgeted separately and more strictly: see
[Sequencer Core §7](03-sequencer-core.md).

---

## 7. Build and tooling

One entry point: `just`. If a task cannot be run by an agent with a single `just`
verb, it does not exist as far as the operating model is concerned.

```
just setup            # toolchains, git-lfs, hooks
just build            # everything, debug
just build --release
just run              # standalone
just test             # unit + conformance, no GPU
just capture <scene>  # deterministic offline frame capture
just verify           # every gate; this is what CI runs
just verify:geometry  # …and each gate individually
just report           # open the HTML verification report
just truth            # re-run geometry extraction (Panelwright only)
just journal <role>   # open today's journal entry
```

`just verify` must complete in under **four minutes** on an M-series laptop. This is
a hard requirement, not an aspiration. A verification suite that takes twenty minutes
gets skipped, and a skipped gate is not a gate. If the suite slows down, the fix is
to make it faster, not to run it less.

Slow, exhaustive checks (the 10 000-frame soak, the 30-minute timing run, the full
conformance sweep) live in `just verify:deep` and run nightly plus on release
candidates.

### Pre-commit

Installed by `just setup`. Runs `verify:slop`, `verify:arch`, and `cargo fmt --check`.
Under two seconds. Anything slower goes in CI, not in the hook.

---

## 8. Where v1's code goes

`beaudesign/BenoQs-Sequencer` is archived, not merged. Two things are salvaged, both
by transcription rather than import:

1. **`octopus_scheduler.js`.** It contains real Octopus semantics that someone
   clearly derived from the manual with care: the GRV shuffle delay table, the chord
   strum timing table, ping-pong direction handling, chain member traversal. These
   are ported to Rust as `octocore::tables` **with a manual page citation on every
   constant**. Any table entry that cannot be cited is marked `unverified` and gets a
   conformance fixture written against the manual before it is trusted.

2. **`octopus_schema.js` defaults.** The default grid, default PIT values per track,
   and structural shape. Ported to `octocore::defaults`, again with citations.

Everything else (the Max patcher, `jweb` bridge, HTML mock, the adapter, the CSS)
is deleted. Not refactored. Deleted. It encodes the wrong architecture and keeping it
around guarantees someone reaches for it.

The tests in v1 (`tests/*.test.js`) are read for *ideas about what to test*, then
rewritten. Their homegrown runner is replaced by `cargo test` plus the harness.

# SPEC-0001 findings

Audit of `beaudesign/benoqs-sequencerv2` at commit `6ef921f`. Every number below comes
from a command run on a fresh shallow clone. The probe programs lived outside the repo
and become tests under WENGE-0009. The machine was an x86-64 Linux container, not an
M-series laptop: treat absolute times as indicative and ratios as solid.

"Reproduced" means a probe failed on `6ef921f`. "Measured" means the repo's own tooling
was run. "Read" means confirmed in the source, but no failing case was run. Line
numbers refer to `crates/octocore/src/engine.rs` at `6ef921f` unless stated.

## What was run

| Check | Result |
|---|---|
| `cargo test --release --workspace` | 82 tests pass: octocore 66 unit plus 1 runner over 3 fixtures, octoffi 5, octoroom 10. 9.4 s cold. |
| `just verify` | 2.9 s, exit 0. 12 of 13 gates print "not yet implemented". Only octocore tests run. octoffi and octoroom tests are not part of it. |
| WASM | `cargo build -p octocore --release --target wasm32-unknown-unknown --lib` succeeds in 1.5 s. |
| Allocations on the audio path | 0 heap allocations over 30 s of dense rendering. `Engine::new` makes 1 allocation of about 1 MB. A strength to keep and gate. |
| Struct sizes | Page 6,727 B, Track 473 B, Step 27 B, Engine 10,656 B. |
| Stress benchmark | 10 tracks x 16 steps, 4x speed, 7-note chords, strum 5, a phrase on every step, 240 BPM, 64-sample buffers, 60 s: 945,478 events, 0 buffers hit the 256-event cap. |

Cost of one 64-sample render call against its 1,333 us budget:

| Median | p99 | p99.9 | Max |
|---|---|---|---|
| 14.0 us (1.1%) | 39.3 us (3.0%) | 57.7 us (4.3%) | 321.8 us (24.1%) |

The core is not slow: the median is about 1% of budget and the p99.9 is under 8%.
The problem is correctness under a host and missing enforcement.

**Correction (2026-09-29, WENGE-0001).** The 321.8 us maximum above came from one lucky
run and must not be quoted as "24% of budget". Seven later runs in the same
container (three on `6ef921f`, four after the O1 and O2 fix) gave: median 12.4 to 14.0 us, p99 35.4 to
44.2 us, p99.9 57.1 to 106.0 us (4.3% to 8.0% of budget), and a maximum between 0.9 ms and
3.8 ms. The maximum is scheduler pre-emption in a shared container, not the engine: it
moves by 4x between identical runs and is the same before and after the fix. Read the
p99.9, and repeat on real hardware before quoting any worst case.

## Ten opportunities

| ID | Opportunity | Evidence | Tier | Effort |
|---|---|---|---|---|
| O1 | Flush sounding notes on stop and reset | Reproduced | Medium | S |
| O2 | Stop the backlog burst on Play after idle | Reproduced | Medium | S |
| O3 | Make timing independent of buffer size | Reproduced | Medium | M |
| O4 | Lock to the host clock with an integer tick clock | Read | High | L |
| O5 | Count every dropped event, never drop a NoteOff | Reproduced | Medium | S |
| O6 | Build the command ring and snapshot the spec promises | Read | Medium | M |
| O7 | Headless runner, golden streams, WASM demo | Read | Medium | M |
| O8 | Make the gates mean something | Measured | Medium | S to M |
| O9 | Fixture DSL v2 and metamorphic invariants | Read | Low | M |
| O10 | Emission details: velocity 0, overlaps, bend and pressure, doc drift | Read | Medium | S to M |

### O1 Flush sounding notes on stop and reset

- Evidence: `all_notes_off_now` (357 to 360) only clears the queue. Its comment says it
  matches CC 123, but nothing is emitted. `reset` (334 to 344) does the same.
- Probe: one track with long notes (LEN 192 x 4), 40 buffers of 256 samples at 120 BPM
  gave 6 NoteOns and 0 NoteOffs. Then 2,000 more buffers with transport stopped gave 0
  NoteOffs and 0 CCs. Six notes stay on in every receiver.
- Change: a sounding-note table (2 ports x 16 channels x 128 notes, one byte each, 4 KB,
  no allocation). On stop or reset emit a NoteOff for every sounding note at sample 0 of
  the next render, send CC 123 on each used channel, then clear the queue.
- Assertion: for any pattern and any stop point, after the next render call NoteOns minus
  NoteOffs is 0 for every (port, channel, note).

### O2 Stop the backlog burst on Play after idle

- Evidence: `render` (365 to 384) advances `sample_clock` while stopped but leaves
  `next_tick_due` where it was. On Play the loop replays every idle tick.
- Probe: `render` for 10 s with `playing=false`, then `playing=true` with one 512-sample
  buffer returned 256 events (the cap) in 1.6 ms. The right answer is 0 or 1. About
  3,840 ticks ran in one call and the overflow was dropped silently (see O5).
- Change: on the false-to-true transition set the tick clock to the current sample
  position (later: the host position from O4).
- Assertion: after idle time N, the first play buffer contains only events due inside
  it. Run for N = 0 s, 1 s, 10 s and 1 h.

### O3 Make timing independent of buffer size

- Evidence: lines 400 to 401 clamp any event due before the buffer start to sample 0.
  Negative STA pulls a note earlier by up to 12 ticks (`STA_TABLE` row 16 at offset -5,
  `tables.rs` 181 to 199), so an early note that lands at a buffer boundary is emitted
  late.
- Probe: 8 s at 133 BPM, 10 tracks, STA offsets -2 to +2, same seed. Against the 512
  sample run, buffers of 32, 64, 128 and 1024 change 763, 710, 579 and 136 of 3,743
  events. Worst shift 226 samples (4.7 ms at 48 kHz). With STA at 0, every buffer size
  gives identical output (0 differences), which isolates the cause. The 512 run has the
  same clamp, so this is a comparison, not ground truth.
- Change: schedule ahead. Step the tick clock `MAX_EARLY_TICKS` (12, asserted against
  the tables by a test) past the buffer end and emit only events due inside the window.
  Cost: edits made through commands land up to 12 ticks later (31 ms at 120 BPM and
  48 kHz, under one step).
- Assertion: the absolute-time event stream is identical for buffer sizes 1, 32, 64, 128,
  256, 512, 1024 and 4096, and for a randomised variable-size sequence.

### O4 Lock to the host clock with an integer tick clock (not approved)

- Evidence: `Command::HostTransport { ppqn_pos, bpm, playing }` uses only `playing`
  (line 329). Tempo arrives per buffer in `RenderContext`. Tick times accumulate as
  `next_tick_due += samples_per_tick` in f64 (line 382), and each track keeps an f32 step
  accumulator (57 to 58, 507 to 513). `docs/02` section 5 says position is u64 ticks with
  no float accumulation. `docs/10` R4 asks for a PLL. `Grid.tempo_bpm` (`domain.rs` 811)
  is never read. Locate, loop and tempo ramps have no defined behaviour.
- Change: u64 tick counter, host PPQN mapped to sample time per tempo segment, due time
  computed from origin plus tick x samples-per-tick at the boundary only, a PLL for hosts
  with variable buffers, integer per-track step accumulators.
- Assertion: tempo ramp 60 to 180 BPM over 8 bars drifts at most 1 sample against the
  analytic answer. A locate to any PPQN puts the next tick within 1 sample. On the
  synthetic null host, jitter sigma at most 120 us and max at most 500 us at every buffer
  size (`docs/03` section 7).
- High tier because the constitution calls a late note a catastrophe (N5).

### O5 Count every dropped event, never drop a NoteOff

- Evidence: `EventBuffer::push` (146 to 151) and `schedule` (346 to 353) drop silently
  when full (256 per call, 512 queued) and keep no counter. `octocore_engine_render` in
  octoffi truncates to the caller's capacity the same way. The O2 burst hit the cap. The
  steady-state stress run never did, so this is an overload guard.
- Change: a `Diagnostics` struct (dropped events, dropped scheduled, late events, max
  lateness in samples, forced NoteOffs) behind a new FFI getter and in the snapshot.
  Reserve queue space so a NoteOn and its NoteOff are allocated together, and force an
  off from the sounding table if a pair cannot fit.
- Assertion: saturate the engine, then stop. Counters are non-zero and NoteOns minus
  NoteOffs is 0.

### O6 Command ring and snapshot (not approved)

- Evidence: `docs/02` section 2 specifies a wait-free SPSC command ring and a
  triple-buffered snapshot. `types.rs` defines `Snapshot`, but no code builds one. Button,
  encoder and `LoadState` commands are no-ops (line 330). octocore has no dependencies.
  Today the only way to drive the engine is direct `&mut` access.
- Change: hand-rolled SPSC ring of plain-data commands (capacity 1024, drop-oldest with a
  counter), triple-buffer publish at the end of render, logical commands (`SetStep`,
  `SetTrack`) mirroring the FFI setters.
- Assertion: two-thread stress with no torn snapshot across 107 publishes. Overflow
  increments the drop counter. Miri or loom over the ring.

### O7 Headless runner, golden streams, WASM demo

- Evidence: the workspace holds three library crates. No binary, example or run recipe.
  `docs/02` section 7 lists `just run`, the justfile has no such recipe, and
  `just capture` exits 1. Xcode is absent from the dev environment.
- Change: new crate `octorun`. Input is a pattern file that reuses the fixture DSL.
  Output is an NDJSON event log and a Standard MIDI File, both byte-identical for a given
  seed. An optional live feature sends real MIDI through midir. The WASM build already
  compiles and can carry a WebMIDI page.
- Assertion: golden hashes of event streams for 5 patterns. `just run examples/hello`
  gives the same hash twice.

### O8 Make the gates mean something

- Evidence: `just verify` ended in 2.9 s with exit 0 while 12 of 13 gates printed "not yet
  implemented". The runner treats exit code 42 as a soft pass and writes no JSON, although
  `contracts/verification.report.schema.json` already defines `not_implemented` and a full
  report shape. octoffi (5 tests) and octoroom (10) never run under `just verify`, and
  conformance runs twice. There is no `.github` directory, CI workflow or CODEOWNERS file,
  although `docs/08` section 3 says ownership is enforced in the merge queue.
  `verify:regressions`, the assertion-count ratchet that rule N7 rests on, is a stub.
- Change: tri-state verify, `harness/report/latest.json` against the existing schema,
  non-zero exit when a required gate is `not_implemented`, `cargo test --workspace`, a CI
  workflow, CODEOWNERS, and the regressions gate against a committed baseline (82 tests,
  3 fixtures).
- Assertion: a pull request that deletes a test fails CI. A pull request that leaves a
  required gate unbuilt fails CI.

### O9 Fixture DSL v2 and metamorphic invariants

- Evidence: 3 fixtures (43 lines) against a target of 250 (`docs/03` section 8). The
  fixture DSL (`fixture.rs`) can assert only "a note fired on track N with pitch P". It
  cannot express timing, velocity, length, count, absence, transport or buffer size, so
  O1 to O3 are invisible to it.
- Change: `expect at= vel= len= count= absent`, plus `stop`, `play` and `render buffer=`.
  A new `tests/invariants.rs` for buffer-size invariance, stop/play idempotence, note
  balance, seed replay and tempo-scale invariance.
- Assertion: the invariants suite fails on `6ef921f` for O1, O2 and O3 and passes after.

### O10 Emission details and doc drift

- Evidence: lines 663 to 665 can emit a NoteOn with velocity 0, which most receivers read
  as a NoteOff. `docs/03` section 2 says a step value of 0 suppresses the note. MCC Bend
  and Pressure are never emitted because `Event` has no such variants (820 to 822).
  Same-pitch overlap: an earlier NoteOff can cut a retriggered note, and the queue's
  swap-remove (396 to 398) does not keep insertion order for equal due times. The default
  legato case (LEN equals step length) was tested for 20 s: 640 notes, no overlaps. So this
  one is unproven, not broken.
- Doc drift: `docs/02` says seeded PCG64, the code uses splitmix64 (`rng.rs`). `docs/08`
  says role briefs live in `agents/roles/`, the file is `agents/ROLES.md`. `CLAUDE.md` says
  `just verify:<zone>`, the recipe is `verify-<zone>`.
- Change: velocity floor of 1, with "0 suppresses" implemented as no note. Append
  `PitchBend` and `ChannelPressure` event variants. Stable queue order by sample, then off
  before on, then insertion sequence. Record the retrigger decision in `AMBIGUITIES.md`.
- Assertion: every NoteOn has velocity at least 1. An equal-time ordering test. A
  retrigger fixture once decided.

## Deliberately not proposed

Micro-optimising the tick loop. The per-tick copy of a 6.7 KB `Page` and the linear scan
of the event queue look wasteful, but the measured p99.9 is 4.3% to 8.0% of budget and
the median about 1% (the 24% maximum was one noisy run, see the correction above). Revisit
only if the stress benchmark's p99.9 crosses 50% of budget on target hardware.

## Known limits of this audit

- Timings come from a cloud container. Repeat the stress benchmark on an M-series machine
  before quoting numbers.
- The 512-sample run in the O3 comparison has the same clamp, so it is a baseline, not
  ground truth.
- The audit did not run any host, plugin or UI. None exists.

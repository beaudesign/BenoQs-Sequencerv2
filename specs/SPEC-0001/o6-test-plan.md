# WENGE-0006 (O6): test plan for the command ring and the snapshot

| Field | Value |
|---|---|
| Task | `WENGE-0006`, opportunity O6 of SPEC-0001 |
| Spec revision | r2 (approved by the owner on 2026-09-29, 15:11; Q6 default applies) |
| Tier | Medium |
| Status | Plan written and committed before any ring code (`842aeb9`). Built on `metronome/command-ring`, PR 7. Section 10 records where the build differed. |
| Decides | Nothing the owner has not already approved. Section 2 lists the design choices this plan makes inside that approval, and section 9 lists what needs the owner's eyes. |

The rule in `CLAUDE.md` is that every judgement becomes a test. This file is the list of
tests, written first. Each has an ID, what it asserts, and where it lives. During the
build, every test in sections 4 to 7 is shown failing on a stub before it is shown passing
(evidence in `handoffs/evidence/o6-*`), and the ID is in the test's name so the trail from
this plan to `harness/baseline.txt` is greppable.

## 1. What exists today, measured on `9e4860b`

| Fact | Value | Where it comes from |
|---|---|---|
| Ways to change the engine from outside | Direct `&mut Engine`, or `octocore_track_set_i32` and `octocore_step_set_i32`, which write the grid in place | `crates/octoffi/src/lib.rs` lines 184 to 271 |
| Safety of those setters while `render` runs on another thread | None. A data race by construction. `octoffi` says the caller owns the single-writer rule | module comment in `crates/octoffi/src/lib.rs` |
| `Command` values that do something | Play, Stop, Continue, Reset, SetActivePage, SetMode, HostTransport (only `playing`) | `Engine::handle_command` |
| `Command` values that are no-ops | ButtonDown, ButtonUp, EncoderTurn, LoadState | same. They wait for `panel.truth.json`, which does not exist |
| Code that builds a `Snapshot` | None. The type exists in `types.rs` | grep |
| `size_of::<Command>()` | 24 bytes, align 8. Passed by value across the C ABI today | measured |
| `size_of::<Snapshot>()` | **8,408 bytes**, align 8. `docs/02` §2 says "about 6 KB". 512 LEDs at 16 bytes is 8,192 of it | measured |
| `unsafe` in `octocore` | None. `docs/02` §3 reserves `unsafe` for `octoffi` and the renderer | grep |
| Allocation counter test in the repo | None | grep |

## 2. Design choices made inside the approval

**D1. No `unsafe` in `octocore`.** A lock-free ring and a triple buffer are usually written
with `UnsafeCell`. That would put `unsafe` in `octocore` and contradict `docs/02` §3. So
the slots are arrays of atomic words (`AtomicU64` for commands, `AtomicU32` for the
snapshot) and the types are encoded to words and decoded back. The cost is an encode step
and about 2,100 relaxed word stores per snapshot publish. The benefit is that the rule
holds, and that a wrong protocol shows up as a wrong value rather than as undefined
behaviour. "Hand-rolled" in Q6 means this.

**D2. The command ring is an overwrite ring with a per-slot seqlock stamp.**
The producer owns a private `head` counter and never reads anything the consumer writes,
so a full ring cannot make it wait or fail. It writes slot `head % capacity`, then
publishes `head` with `Release`. The consumer owns a private `tail`. If `head - tail` is
more than the capacity it was lapped: it jumps forward and counts the skipped commands as
dropped. That is exactly "the oldest command is dropped and a counter increments"
(`docs/02` §2), and neither side loops on the other. A stamp on each slot, written odd
before the payload and even after, lets the consumer notice a slot that was overwritten
while it read it, and count that one as dropped too.
Payload words use `Release` stores and `Acquire` loads, so the stamp check needs no fence
(loom's fence support is partial).

**D3. The snapshot buffer is the swap-based triple buffer, not the `store(Release)`
version in `docs/02` §2.** As written there, the audio thread "writes into the slot that is
neither `published` nor `reading`", but `reading` is private to the render thread, so the
writer cannot know it. The standard fix is one atomic word holding the middle slot's
index plus a dirty bit. Both sides `swap` it: the writer to publish and learn its next
free slot, the reader to claim only when the dirty bit is set. Both are one RMW, so both
sides are still wait-free. This is a defect in the doc, not in the intent, and goes to the
Scribe as a request.

**D4. Commands cross the ring as three `u64` words.** `Command::to_words` and
`Command::from_words` are the wire format. `from_words` returns `None` for an unknown tag
instead of panicking, so a torn or foreign word can never be misread as a command.

**D5. Two logical commands, `SetTrack` and `SetStep`.** They mirror the FFI setters
(track, optional step, attribute, `i32` value) and are appended after `LoadState`, so
every existing tag keeps its number. The attribute enums and the clamping table move from
`octoffi` into `octocore`, and the FFI setters call the same function. Without this the
ring could carry Play and Stop but not an edit, and edits while playing are the reason for
having a ring.

**D6. Commands apply at the start of `render`, in push order, at most
`MAX_COMMANDS_PER_RENDER` (256) per call.** Before any tick of that call. The bound keeps
a flood from starving the render; what is left is applied in the next call.

**D7. The snapshot is written at the end of every `render` call.** It carries what the
engine knows today: generation, transport (`playing`, `tick`), `mode`, `active`, and the
playhead of each track. `leds` and `encoders` stay zero. There is no control map without
`panel.truth.json`, and inventing one would be exactly the guess `CLAUDE.md` rule 7 warns
about. This is a known gap, recorded in the journal and in section 9, and deliberately
**not** turned into a test that asserts zeros, because that test would have to be deleted
when the truth file lands.

**D8. The link is optional.** `Engine::open_link()` returns the two ends once. An engine
that never opens a link behaves byte for byte as today, so every golden stream, the
determinism gate and the conformance suite are unchanged by construction (test E3).

**D9. FFI is additive.** No existing function signature or struct changes. `OctoDiagnostics`
keeps its five fields, because a caller built against the old header would have the new
code write past the end of its struct. Link counters get their own struct. See section 6.

## 3. Files touched, and whose they are

| File | Zone | Change |
|---|---|---|
| `crates/octocore/src/sync.rs`, `ring.rs`, `triple.rs`, `wire.rs` (new) | Metronome | The primitives. `sync.rs` picks `std` or `loom` atomics |
| `crates/octocore/src/types.rs` | Metronome | `SetTrack`, `SetStep`, the attribute enums, `Snapshot::zeroed`, `PartialEq` derive |
| `crates/octocore/src/domain.rs` | Metronome | `#[repr(u8)]` on `Mode` so `Snapshot` has a defined layout for C |
| `crates/octocore/src/engine.rs` | Metronome | link, drain, publish, `apply_track_attr`, `apply_step_attr` |
| `crates/octocore/Cargo.toml` | Metronome | `loom` under `[target.'cfg(loom)'.dependencies]` only |
| `crates/octocore/tests/`, `crates/octocore/loom/` (new) | Metronome | tests in sections 4 to 7, mutation script in section 8 |
| `crates/octoffi/src/lib.rs`, `octoffi.h`, `README.md` | Conductor | additive surface, header, allocation test |
| `Cargo.lock` | shared | loom and its dependencies, only built under `--cfg loom` |
| `harness/`, `.github/`, `xtask/` | Referee | **untouched.** A request asks for the loom gate (section 8) |
| `docs/02`, `docs/03` | Scribe | **untouched.** A request lists the doc fixes (section 9) |

The `octoffi` files belong to the Conductor. This PR touches them, as PRs 3 and 4 did, and
the request file says so. The owner's Code Owner review on the PR is the check.

## 4. Unit tests: ring, wire, triple buffer

Single-threaded, deterministic, in the default `cargo test`, so they are in the ratchet
from the day they merge.

| ID | Asserts | Lives in |
|---|---|---|
| W1 | `from_words(to_words(c)) == c` for every `Command` variant at boundary values: `u64::MAX`, `i16::MIN`, `-0.0`, NaN with a payload (compared as bits), every `Mode`, every attribute | `wire.rs` |
| W2 | `from_words` on an unknown tag, and on a known tag with an out-of-range enum field, is `None`. It never panics | `wire.rs` |
| W3 | The list of all variants used by W1 comes from an exhaustive `match`, so adding a `Command` variant without wire coverage does not compile | `wire.rs` |
| W4 | `size_of::<Command>() == 24` and `align_of == 8`. This is the by-value ABI. The new variants must not grow it | `wire.rs` |
| R1 | Push up to capacity, drain: the same commands in the same order | `ring.rs` |
| R2 | Draining an empty ring returns nothing and changes no counter | `ring.rs` |
| R3 | Push `capacity + k` with no drain: the drain returns exactly the newest `capacity`, in order, and `dropped == k` | `ring.rs` |
| R4 | Wrap: 100,000 push-then-drain cycles on a ring of 4 give the pushed sequence back with zero drops | `ring.rs` |
| R5 | A capacity that is not a power of two, or is below 2, is refused by the constructor | `ring.rs` |
| R6 | `drain(max)` returns at most `max` and leaves the rest for the next call | `ring.rs` |
| R7 | Property: over 1,000 seeded random push and drain sequences (`octocore::rng`), `pushed == received + dropped + pending` at every step | `ring.rs` |
| T1 | Publish A then B: the reader claims B. A second claim reports nothing new and returns the same slot | `triple.rs` |
| T2 | Claim before any publish returns the initial zeroed snapshot, generation 0, without blocking | `triple.rs` |
| T3 | 1,000 publishes with no reader never block or fail; the reader then sees only the last | `triple.rs` |
| T4 | The generation a reader sees never decreases across claims (random interleaving of publish and claim, seeded) | `triple.rs` |
| T5 | `Snapshot` to words and back is bit-identical, including NaN payloads | `triple.rs` |
| T6 | `size_of::<Snapshot>() == 8408` and the offset of each field are pinned. The header text is checked against the same numbers | `octoffi` header test |

## 5. Engine and logical-command tests

| ID | Asserts | Failing-first shape |
|---|---|---|
| C1 | For every `TrackAttr` and `StepAttr`, the values `i32::MIN, -1, 0, 1, 127, 255, i32::MAX` clamp to a literal table. The table is generated from the parent commit's FFI setters, so the move into `octocore` cannot change behaviour | characterisation; green on both sides, and the reason it exists is that the code moves |
| C2 | `SetTrack` and `SetStep` with a track of 10 or a step of 16 change nothing and do not panic | fails on the stub: the variants do not exist |
| C3 | A pattern built with direct setters and the same pattern built with commands through the ring give identical event streams over 32 steps | fails on the stub |
| C4 | Order: `SetStep{Active}` then `Play` in one drain fires the step at tick 0 of that same render call. Commands apply before ticks | fails on the stub |
| C5 | `10 * MAX_COMMANDS_PER_RENDER` commands: the first render applies exactly `MAX_COMMANDS_PER_RENDER`, the next continues, and the end state equals all of them applied in order | fails on the stub |
| C6 | `Stop` through the ring flushes sounding notes the same way `Stop` through `handle_command` does (the O1 invariant, through the new path) | fails on the stub |
| E1 | After N renders with a link, the reader's generation is N | fails on the stub |
| E2 | After a known pattern has run k ticks, `transport.tick`, `transport.playing`, `mode`, `active` and each `playheads[i].step_index` equal the engine's own values. `SetMode` and `SetActivePage` through the ring show up in the next snapshot | fails on the stub |
| E3 | An engine with a link and an engine without one, given the same edits, produce byte-identical event streams. **This is the regression guard for D8**, alongside the unchanged golden streams and the determinism gate | fails on the stub |
| E4 | Allocation: `render` with a link, `push` and `claim` allocate nothing. Counting allocator, per-thread counter, measured across 1,000 renders | fails on the stub if the stub allocates; the real value is that it fails on any future change that does |

`handle_command` on `SetTrack` and `SetStep` from the direct path is covered by C1 and C2
too, so the direct setters and the ring cannot drift apart.

## 6. FFI tests

The FFI surface added, so the plan and the code can be compared:

```c
#define OCTOFFI_ABI_VERSION 2                       // 1 is the header as of 9e4860b
uint32_t octocore_abi_version(void);

typedef struct OctoSender OctoSender;               // main thread, or any one producer
typedef struct OctoReader OctoReader;               // render thread

typedef struct { uint64_t pushed; uint64_t dropped; uint64_t applied; uint64_t published; } OctoLinkStats;

// Once per engine. 0 on success, -1 null engine, -2 null out pointer, -3 already opened.
int32_t octocore_engine_open_link(OctoEngine *engine, OctoSender **sender, OctoReader **reader);
bool    octocore_sender_push(OctoSender *sender, OctoCommand cmd);   // false only for null
int32_t octocore_sender_stats(const OctoSender *sender, OctoLinkStats *out);
const OctoSnapshot *octocore_reader_claim(OctoReader *reader);       // valid until the next claim
void    octocore_sender_free(OctoSender *sender);
void    octocore_reader_free(OctoReader *reader);
```

| ID | Asserts |
|---|---|
| F1 | Every new declaration and both new structs are present in `octoffi.h`, and `OctoSnapshot`'s fields match `Snapshot` in order and type |
| F2 | `open_link` twice returns -3. Null arguments return -1 and -2. `push`, `claim` and `free` on null do nothing |
| F3 | Through the C functions only: push `SetStep`, `SetTrack`, `Play`, render, and events come out. `claim` returns a snapshot whose transport says playing |
| F4 | Every declaration that the header had at `9e4860b` is still there, byte for byte. The old text is embedded in the test. This is the "additive only" guarantee |
| F5 | `OCTOFFI_ABI_VERSION` in the header, the Rust constant and `octocore_abi_version()` agree |
| F6 | Free the engine, then push and claim through the surviving ends. No crash and no error. The ends share the buffers, they do not point into the engine |

Known and not fixed here: `octocore_engine_handle_command` and the new `octocore_sender_push`
take `Command` by value from C. A tag outside the enum is undefined behaviour in Rust. The
Swift import of `OctoCommandTag` cannot build one, so nothing breaks today, but a validated
raw entry point is the right shape, and it is listed in section 9 as a follow-up.

## 7. Concurrency tests

### Soak (default `cargo test`)

| ID | Asserts |
|---|---|
| S1 | Producer thread pushes 2,000,000 sequenced commands as fast as it can, so the ring overflows many times. Consumer drains in a loop. The received sequence numbers strictly increase, every command's redundant fields agree (no tear), and at the end `received + dropped == pushed` |
| S2 | Writer thread publishes 10,000,000 snapshots on a 16-word slot, each word derived from its generation. Reader claims in a loop. Every claimed snapshot is internally consistent, generations never go backwards, and the reader saw at least one new generation |
| S3 | The same as S2 on the real `Snapshot` (8,408 bytes) for 200,000 publishes. The count is smaller because each publish writes about 2,100 words |

Counts are results-based (no timing assertion), so the tests do not flake on a slow
runner. A deeper run of the same tests at 100,000,000 is `#[ignore]`d and is run by hand
for the evidence file, not by the gates.

### Loom

`RUSTFLAGS="--cfg loom" cargo test -p octocore --test loom_sync --release`. Loom explores
every interleaving of the atomic operations, so it can prove the protocol correct on a
small model where the soak can only fail to find a bug.

| ID | Model | Asserts in every interleaving |
|---|---|---|
| L1 | Ring of 2, producer pushes 4, consumer drains once at an arbitrary point and once after the producer joins | Received values are strictly increasing, each equals what was pushed with that number (no tear), and `pushed == received + dropped` |
| L2 | Same ring, consumer drains twice while the producer runs | No command is received twice |
| L3 | Triple buffer with a 2-word slot holding `(g, !g)`, writer publishes 3, reader claims 3 times | The reader never sees a pair that is not `(g, !g)`, generations never decrease, and a claim after the writer joins returns the last published |
| L4 | Triple buffer, one publish and one claim | The writer and the reader never hold the same slot (each slot carries an owner flag that loom checks) |

## 8. Testing the tests

A passing loom test proves little if loom never reaches the bad interleaving. So the
plan includes deliberate breakage, run by `crates/octocore/loom/mutants.sh`, which copies
the crate, applies one edit at a time with `sed`, runs the loom tests, and expects a
**failure**:

| Mutant | Edit | Which test must fail |
|---|---|---|
| M1 | Ring: the final stamp store becomes `Relaxed` | L1 |
| M2 | Ring: the consumer skips the second stamp check | L1 |
| M3 | Ring: the producer publishes `head` before the payload words | L1 |
| M4 | Triple buffer: the writer does `store` where it should `swap` | L3 or L4 |
| M5 | Triple buffer: the reader claims without checking the dirty bit | L3 |

If a mutant survives, the loom model is too weak and the model is changed, not the
mutant. The result of each run goes into `handoffs/evidence/o6-mutants.txt`.

**Loom is not in a required gate.** `--cfg loom` needs its own build, and the required
gates run plain `cargo test`. The loom tests are behind `#![cfg(loom)]`, so they are not
in `harness/baseline.txt` and the ratchet does not protect them. Until the Referee wires a
`verify:loom` gate, they are evidence, not enforcement. A request to the Referee asks for
that gate. The unit tests and the soak tests in sections 4, 5 and 7 are in the ratchet.

Timing of the new work is measured and reported, not asserted: a timing assertion on a
shared CI runner is a flaky gate. The measurement (per-render cost of the drain and the
snapshot publish, release build, over 100,000 renders) goes into the journal.

## 9. What needs the owner

1. **Dependency.** `loom`, under `[target.'cfg(loom)'.dependencies]` in `octocore`, so it
   is not built for anything except loom runs. It adds 174 lines (18 packages) to
   `Cargo.lock`. `docs/02` §3 says `octocore` has no platform dependencies; loom is a test
   tool, and this is the smallest way to use it.
2. **The `docs/02` text is wrong in two places** (the triple-buffer protocol, the "6 KB"
   figure) **and says `crossbeam`**, which Q6 replaced. `docs/03` needs the `Command`
   additions. These go to the Scribe as a request. The code and this plan are the source
   until the Scribe has changed them.
3. **The snapshot's LEDs and encoders are empty** until `panel.truth.json` exists. The
   ring and the buffer are ready for them. Filling them is a separate task.
4. **Follow-ups, not done here:** a validated raw command entry point for C (section 6);
   `verify:loom` gate (section 8); `ButtonDown`, `ButtonUp`, `EncoderTurn` and `LoadState`
   are still no-ops.
5. **Rollback.** The link is optional and additive, so reverting PR 7 restores the current
   behaviour. Nothing else depends on it.
6. **Kill criterion.** If a loom mutant survives and the model cannot be strengthened, or
   S1 finds a tear that loom did not, the ring is not merged. The plan is then revised
   with the finding, not the test loosened.

## 10. Amendments made during the build

The plan above is kept as it was committed. Where the build disagreed with it, this section
says so, so that nothing in the plan reads as true when it is not.

| Plan says | What was built | Why |
|---|---|---|
| Section 2 D2: the stamp store is `Release` and needs no fence | Same, plus a note that the *final* stamp store could be `Relaxed` without harm | The head counter's own `Release` and `Acquire` already order the payload. Mutant M1 in the plan (final stamp made `Relaxed`) is therefore an equivalent mutant that no test can catch. M1 became "the odd stamp is never written", which is a real defect |
| Section 4: `Receiver` has `drain` and counters | Also `reject()`, which moves one command from "received" to "dropped" | The engine needs it for words that do not decode. Test R8 covers it. Without it the counters would double-count |
| Section 5 C4: "`SetStep{Active}` then `Play` in one drain fires the step at tick 0 of that render" | "A command in the first drain lands on exactly the sample a direct edit does" | The first attempt failed for a reason in the test, not the code: the engine decides its first note in the first render, but it sounds about 1,375 samples later (buffer 5 at 256 samples). The test now renders 8 buffers and compares with a direct edit. See `o6-slice4-red.txt` |
| Section 5: tests C1 to C6, E1 to E4 | Also C7 (undecodable words are counted and never applied), and `the_link_can_be_opened_once` | Both were behaviour the code had that the plan did not name |
| Section 4 T5, T6 | Also a test that a bad mode word is refused, and one that a zeroed snapshot is all zero words | Same reason |
| Section 6: `OctoLinkStats` has `pushed`, `dropped`, `applied`, `published` | Three fields; `published` is the snapshot's `generation` | One less thing to keep in step |
| Section 6: `octocore_reader_claim` "valid until the next claim" | The pointer is the same on every call and valid until `octocore_reader_free`; its contents change only inside `claim` | This is what it does, and it is easier for the caller to hold |
| Section 6 F1: a text check of the header | Also a real C compiler test with `_Static_assert` on 29 sizes, offsets and tags | The text check alone cannot see a wrong size |
| Section 7 S2: 10,000,000 publishes; S3: 200,000 | Those counts in a release build; 2,000,000 and 30,000 in a debug build. A debug S3 at 200,000 took about 17 seconds | The default `cargo test` stays quick. The release run and the ignored deep runs do the full counts |
| Section 7 L4: an owner flag on each slot | The reader reads its slot twice with a scheduling point between and asserts the two reads agree | The slots are private to `triple.rs`. The check finds the same defect: a writer that writes the reader's slot |
| Section 8: five mutants, `mutants.sh` | Seven mutants that must be caught, plus M8, an equivalent mutant that must survive, in `mutants.py` | M4 (payload loads made `Relaxed`) and M7 (writer swap made `Relaxed`) test the memory ordering, which the first five do not. M8 is the plan's original M1 (final stamp store made `Relaxed`), kept on purpose: it is harmless today, and the day it stops being harmless the run shows a failure to explain. A shell script could not hold replacement text that contains `|` |
| Section 9.1: "roughly 30 lines" added to `Cargo.lock` | 174 lines, 18 packages | Miscounted |
| `octocore/Cargo.toml` change: loom under `cfg(loom)` | Also `[lints.rust] unexpected_cfgs`, so `cfg(loom)` is a declared cfg | Otherwise every build warns |

**Loom result for the mutants** is in `handoffs/evidence/o6-mutants.txt`: M1 to M7 caught, M8 survives as expected. M6 is caught by an abort of the whole `l3_` test process rather than by a named assertion, which the runner now says instead of printing a bare "a loom test".

**Measured, from the build.** The snapshot is 8,408 bytes. Publishing it costs about 0.7
microseconds per render and applying 256 commands about 7 microseconds (release build,
`handoffs/evidence/o6-cost.txt`). `Engine` grew from 47,552 to 47,560 bytes.

**Found on the way, and recorded rather than fixed here:**

- `RenderContext::playing` overrides `Command::Play` and `Command::Stop` at the top of the
  render, whether the command came from the ring or from `handle_command`. Who owns the
  transport is O4's question.
- An edit made while playing is heard no sooner than the render after it is pushed, and a
  step less than 12 ticks from firing has already been decided (`MAX_EARLY_TICKS`). That is
  about 31 ms at 120 BPM. `docs/02` §6 budgets 1.3 ms for "core apply, next tick".
- `rustfmt --check` reports 396 diffs at the parent commit, so `docs/02` §7's pre-commit
  check cannot be enforced yet. This PR did not reformat.

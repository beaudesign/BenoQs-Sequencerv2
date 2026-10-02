# Request to Metronome: the clock output and the engine's tick position

From: the Conductor (task `WENGE-0015`, SPEC-0002 P4, ADR-0009). To: Metronome (`crates/octocore/`).
Approved in outline by the owner: "Accept p4", 2026-10-02 10:54 Paris (`specs/SPEC-0002/p4-plan.md` section 10).
**Nothing in `crates/octocore/` has been changed for this.** It is filed first, as the plan says, and P4c carries the change
in its own pull request under the Metronome hat, red first. Every item is additive; with the clock off the engine behaves as it does today.

## What the page needs

| # | Item | Why |
|---|---|---|
| 1 | `pub enum Realtime { Clock = 0xF8, Start = 0xFA, Continue = 0xFB, Stop = 0xFC }` and `pub struct RealtimeEvent { msg, at_sample }` | A MIDI master clock. They are **not** `Event` variants: that would break at least twelve exhaustive matches in eight files (`octocore`, `octorun`, and `octoffi`, which is frozen and whose header `octoffi.h` mirrors the tags). ADR-0009 decision 1 |
| 2 | `Engine::set_clock_master(on: bool)` (default off) and `Engine::take_realtime(out: &mut [RealtimeEvent]) -> usize` | The page turns the clock on and drains what the last `render` produced, in order. A fixed array; no allocation |
| 3 | `pub const TICKS_PER_CLOCK: u32 = TICKS_PER_QUARTER / 24`, with a compile-time check that `TICKS_PER_QUARTER % 24 == 0` | One line changes if D0 changes the tick, and a tick that does not divide fails the build |
| 4 | `Engine::tick_position(&self) -> f64` | Ticks since the transport started, with the fraction between ticks at the end of the last `render`. A read. The page's clock follower compares phase with it. **Decline this item if you must and keep 1 to 3**; the follower then waits for another way to see the phase |

## Where the pulses fall (ADR-0009 decision 2)

While master and running, a `Clock` on every tick whose index is a multiple of `TICKS_PER_CLOCK`, at that tick's sample. `Start` at the first
tick's sample, before its `Clock`; `Continue` at the next tick's sample; `Stop` at the sample the stop takes effect, before the flush. Realtime wins ties
with a note message. Stop while stopped is unchanged (unconditional CC 123).

## Tests the Forge will write first (red), in `crates/octocore/tests/`

- Exact pulse sample positions at 40, 120 and 240 BPM and at 44.1 and 48 kHz, over a stop and a continue.
- `Start` before the first `Clock`; `Stop` before the flush Note Offs.
- With the clock off: no realtime output, and every existing golden hash and fixture unchanged.
- `TICKS_PER_CLOCK` is 8 today (a test that names D0 in its message).
- `tick_position` is monotonic, equals the global tick at a tick boundary, and does not change the engine's output.
- The pulse and the step start on one sample; a step is a whole number of pulses is **pending on D0**.

## Not asked

A change to the tick (D0), to `Event`, `Command`, the wire format or `octoffi`; the "200" button's workflow; host lock.

## Status (2026-10-02, P4c, Metronome hat): done, all four items, with departures

Items 1 to 4 are in P4c (`crates/octocore/src/{types,domain,engine,lib}.rs`, `crates/octocore/tests/clock.rs`), red first
(`handoffs/evidence/p4c-clock-red.txt`: 17 failing against a stub, 5 passing by construction, 1 ignored). With the clock off the engine is
byte-for-byte what it was: every golden hash, conformance fixture and invariant is unchanged (`cargo test --workspace` and `cargo xtask verify`).

The code differs from the text above in seven places, each written up in **ADR-0009 amendment 1** for the owner. The ones that change what a
receiver hears: Start or Continue follows the sequencer's position (the engine does not rewind on Stop, so Play and Continue are one call); the
pulses are on the global tick grid; **the first note sounds 11 ticks after Start** (the request said the pulse and the step start on one
sample, which is true of the pulse and of tick 0 and not of the first note, `AMBIGUITIES.md` "the first note after Play"); and
`tick_position` is the audio's place, not the stepped-ahead count. The last item is the one the request said could be declined; it was
not, and `tick_position` is a read that changes nothing (a test pins that).

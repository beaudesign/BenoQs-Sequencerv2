# ADR 0009: The engine emits the MIDI clock beside its events, the page follows one, and the ABI grows to 2

## Status

Proposed on 2026-10-02 by the Conductor. It follows from SPEC-0002 r1 (approved by the owner,
"Approve", 2026-09-30 15:06 Paris) and from the owner's **"Accept p4"** of 2026-10-02 10:54 Paris, recorded in
`specs/SPEC-0002/p4-plan.md` section 10. It takes effect when the owner merges the pull request that
carries it. The code arrives in P4b to P4e. The plan is `specs/SPEC-0002/p4-plan.md`; this ADR records
decisions D-P4-3, D-P4-4, D-P4-8 and D-P4-12 and **refines two of them where the code showed a smaller way**
(decisions 1 and 7, flagged there).

## Context

Everything below is from the code on `main` at `8d7ee25`; nothing in it is a browser measurement.

| Fact | Where |
|---|---|
| `octocore::Event` has five variants (note on, note off, controller, bend, pressure), is `#[repr(C)]`, and carries a sample offset | `crates/octocore/src/types.rs` |
| There is no clock output, and no `Event` for one | same |
| The engine counts 192 ticks to the quarter note and `sample_rate * 60 / (bpm * 192)` samples a tick; a step is 12 ticks | `domain.rs` line 33, `engine.rs` line 274 |
| Exhaustive matches on `Event`: at least twelve, in eight files across `octocore` (source and tests), `octorun` (the NDJSON log and the Standard MIDI File writer) and `octoffi`, whose hand-maintained header `octoffi.h` mirrors the tags | a search for the `ChannelPressure` arms |
| `Command::HostTransport { ppqn_pos, bpm, playing }` is acted on for `playing` only | `engine.rs` line 546 |
| Stop while stopped already sends CC 123 on all 32 channels, unconditionally | `engine.rs` line 524 |
| The page's scheduler already keeps one output per port and orders timestamps per output | `apps/web/src/midi-out.ts` |
| `octoweb-abi/1` is 15 exports and an event record of 12 bytes with kinds 0 to 4 | `apps/web/engine/ABI.md` |

D0, the length of the tick (`WENGE-0012`), is still open. At 120 BPM the engine's step is 31.25 ms and a MIDI
clock pulse is 20.83 ms, so a step is 1.5 pulses where a sixteenth note is 6. This ADR does not change the tick.

## Decision

### 1. The engine emits the clock, beside its events and not as one of them

*Refines D-P4-3, which said "new `Event` variants".* The intent stands: the pulses come from the engine, at the
sample where the tick falls, so a drum machine hears them in step with the notes. They do not travel as `Event`
variants, because that is the wider change for the same result: it breaks twelve exhaustive matches in eight
files, two of them in a crate that is frozen (SPEC-0002 D5), and makes the hand-maintained `octoffi.h` disagree with
`Event` unless it is edited too. A separate channel touches none of those.

`crates/octocore` gains, all additive:

- `pub enum Realtime { Clock = 0xF8, Start = 0xFA, Continue = 0xFB, Stop = 0xFC }` (`#[repr(u8)]`, the MIDI status bytes)
  and `pub struct RealtimeEvent { pub msg: Realtime, pub at_sample: u32 }`.
- `Engine::set_clock_master(&mut self, on: bool)`, off by default, and
  `Engine::take_realtime(&mut self, out: &mut [RealtimeEvent]) -> usize`, which hands over what the last
  `render` produced, in order, and clears it. A fixed array holds them and none is allocated.
- `Engine::tick_position(&self) -> f64` (decision 4).
- `pub const TICKS_PER_CLOCK: u32 = TICKS_PER_QUARTER / 24`, with a compile-time assertion that
  `TICKS_PER_QUARTER % 24 == 0`, so a tick that does not divide into pulses fails the build and does not drift.

`Event`, `Command`, the wire format, `octoffi`, `octorun` and every golden hash are untouched. With the clock off
(the default) the engine's behaviour is byte-for-byte what it was.

### 2. When the pulses fall

While the clock is master and the transport runs, a `Clock` falls on every tick whose index is a multiple of
`TICKS_PER_CLOCK`, at that tick's sample, which is where the step grid also falls: the first pulse and the first step
are the same sample, and the two cannot drift apart, because both are counted in ticks. **What D0 decides is how many
pulses a step is**: 1.5 with today's tick (12 ticks at 8 to a pulse), 6 if the tick is 48 to the quarter note. So whether
the pattern is in time with the clock a human hears is D0's, and plan observable M3 (a step is a whole number of
pulses) is written now and `pending` until D0 is answered.

- **Play** from stopped: `Start` at the first tick's sample, ordered before the `Clock` on that sample.
- **Continue**: `Continue` at the next tick's sample; the pulse count continues from the global tick.
- **Stop**: `Stop` at the sample the stop takes effect, ordered before the flush Note Offs and CC 123 at that sample.
  Stop while stopped sends nothing new on the clock and keeps its CC 123 flush (D-P4-7: unchanged).
- Ties between a realtime message and a note message at one sample go to the realtime message.

### 3. The page puts the two streams in one buffer

`octoweb` calls `take_realtime` after each `render` and merges the two lists by sample offset into the one events
buffer, realtime first on ties, so the page sees one ordered list and its scheduler's rule (timestamps on an output
never go backwards) holds for the clock as for the notes. A realtime message has no port. **The scheduler sends it
to every output that has a device chosen, once per device**: if one device is chosen for both ports it hears one
clock, not two.

### 4. The engine says where its tick is

`Engine::tick_position()` returns the ticks since the transport started, as a fraction: the global tick count plus how far
the engine is between ticks at the end of the last `render`. It is a read; it changes nothing. The page needs it because
the follower (decision 5) compares the engine's phase with the pulses it receives, and the engine's tick falls
inside a render block, where the page cannot see.

### 5. The page follows a clock: an estimator, a comparator, an actuator and a lead

*D-P4-4, with the detail the design needs.* In slave mode the engine does not emit a clock (it is not master) and does
not tick on pulses. The page, which already owns the tempo (`octoweb_set_tempo`) and the transport (`octoweb_transport`):

- **Estimator.** A pure function takes the page-clock timestamp of each incoming `0xF8` and gives a period, a tempo
  (`60000 / (period * 24)` BPM) and a predicted time for the next pulse, with a lock state. It lives in
  `apps/web/src/` with its own fixtures (a simulated clock with jitter, ramps, a dropped pulse and a doubled one),
  and its thresholds are plan observable M4, which are the owner's to change.
- **Comparator.** The phase error is the difference between where the engine's pulse falls and where the estimator
  says the incoming pulse falls, both in page time. The engine's pulse time is `tick_position` at a render block
  mapped through the page's audio-to-page clock map, which P3 measured to move by a millisecond or two; that error is
  inside the loop's noise budget and S3 measures the whole.
- **Actuator.** The tempo given to the engine is the estimator's tempo with a small trim proportional to the phase
  error (a second-order loop). The engine's own sample clock still times every tick.
- **Lead.** The notes the engine sends reach the output `L` (the lookahead) after the engine made them, so the engine
  runs ahead of the incoming pulses by `L` plus a user offset (default 0 ms): the target phase is the incoming grid
  advanced by that amount, and the notes land on the sender's grid instead of `L` after it. The offset is shown in the
  settings view; its purpose is the constant delay Live's "MIDI Clock Sync Delay" also corrects [Ableton sync].
- **Start, continue and stop** follow `0xFA`, `0xFB` and `0xFC`; a pulse alone never starts the transport; with no pulse
  for 500 ms the page says so and keeps playing at the last tempo (plan M5).

If spike S3 shows this cannot hold a phase within the kill criterion (plan section 5), the engine ticking on pulses (host
lock, the old O4 step 4b) is the next design, and it waits for D0.

### 6. Echo, and master and slave are exclusive

Slave with echo passes incoming `0xF8`, `0xFA`, `0xFB` and `0xFC` to the chosen outputs, stamped like the notes and with
the same ordering rule, from the page and not through the engine (D-P4-8). The four clock states are off, master, slave
and slave with echo, so an output never receives both an engine pulse and a passed one.

### 7. The ABI becomes 2, in P4c; two of the additions come earlier

*Refines D-P4-12, which listed four event kinds.* One kind is enough, and it is in the event record's existing 12 bytes:

| Change | Slice | Detail |
|---|---|---|
| `octoweb_set_track(track, attr, value)` | P4b | Writes one attribute of one track through the engine's existing `SetTrack`. The `?route=` stand-in for port 2 (D-P4-2) uses it. Additive, so the number stays 1 |
| Event record kind 5, "realtime" | P4c | `d1` is the status byte (`0xF8`, `0xFA`, `0xFB` or `0xFC`); `port` and `channel` are 0; `d2` is 0 |
| `octoweb_set_clock(master)` | P4c | Calls `Engine::set_clock_master` |
| `octoweb_tick_position() -> f64` | P4c | Calls `Engine::tick_position` |
| `octoweb_abi()` returns 2 | P4c | The number is bumped once, where the event record gains a kind that a page built for 1 would count as invalid |

`ABI.md` lists every export; the test that fails if it and `exports.rs` disagree is extended, and a test pins the new
record.

## Not decided here

- **D0**, the tick length. Still open. Nothing in P4 claims the pattern is "in time" with a clock before it is answered.
- **Host lock**, the engine ticking on pulses. Revisited only if S3 forces it.
- **What a browser delivers to a page without the SysEx permission**, and what an incoming message's `timeStamp` is on
  each platform. Read and measured first in P4b (plan D-P4-9); if realtime messages need the permission, that is a finding
  and P4d stops.
- **The "200" button's workflow** (Panelwright; `tech.md` section 9 item 3) and **persisting the clock state**.
- **Program change, recording and the other input consumers** (D-P4-1): the waves that own their engine work.

## Consequences

- The Metronome's zone grows by four additive items and a constant. The request is
  `journal/metronome/requests/2026-10-02-clock-output-and-tick-position.md`, filed with this ADR; P4c is built under
  the Metronome hat in its own pull request, and the change is red first.
- No golden hash, `Event` consumer, `octoffi` header or `octorun` writer changes. If the owner prefers the plan's
  first reading (`Event` variants), that is a larger P4c and this ADR is amended before it starts.
- The follower needs the engine's tick position, which is a read accessor and not a behaviour change; it is in the
  Metronome request as its own item so it can be declined without the clock.
- The ABI number changes once in P4. The page and the module ship together, so the number is a consistency check and not
  a compatibility promise.
- A realtime message is sent to every chosen device once. A rig that routes one device to both ports hears one clock.

## Amendments

**Owner's acceptance.** Amendments 1 and 2 were put to the owner for acceptance or change in the reports on #31, #32 and #33. The owner replied **"Accept all changes"**
(2026-10-02 16:08 Paris), which the agent reads as accepting both as built, with their defaults and the thresholds they name as proposals; the reading, and what it does
not cover (D0, the real-port runs, any claim that a pattern is in time with a clock), is written down in `specs/SPEC-0002/p4-plan.md` section 10, r2. The text of the
amendments is unchanged by this note.

### Amendment 1 (2026-10-02, Conductor, found in P4c): what the engine's clock does, as built

Decisions 1 to 4 and 7 were written from a reading of the code. P4c built them red first (`crates/octocore/tests/clock.rs`, 22 tests,
17 failing against a stub) and the code showed six places where the ADR said less than the engine does, and one where it said something
the engine does not. Each is a departure from the text above, flagged for the owner, and none touches a contract or an existing
golden hash.

1. **Start or Continue is decided by where the sequencer is, not by which button was pressed.** The engine does not rewind on Stop (the
   manual, p.40: POS "does not restore to the start POS(ition) when stopping the sequencer"), and `Command::Play` and `Command::Continue`
   are the same call (`set_running(true)`). So the engine cannot tell a Play from a Continue, and it sends `Start` when it is at tick 0
   (new, or after `Reset`) and `Continue` everywhere else. A `Start` sent from the middle of a pattern would tell a drum machine the
   sequencer is at its beginning when it is not. Decision 2's "Play from stopped: `Start`" is amended to this.
2. **The pulses are on the grid of the global tick, for every run.** A `Clock` falls on every tick whose global index is a multiple of
   `TICKS_PER_CLOCK`, counted from the last `Reset` and not from the last Play. After a Stop at tick 20 and a Play, `Continue` is on the
   first tick's sample and the first `Clock` is four ticks later, on tick 24. That is MIDI-correct (a receiver advances on the next
   pulse) and it keeps the clock and the pattern on one grid. Decision 2 said this only for Continue; it holds for all.
3. **The first note sounds 11 ticks after `Start`, not on its sample.** Decision 2 says "the first pulse and the first step are the same
   sample". The pulse is (`Start` and the first `Clock` share a sample), the step is not: a step fires when its tick counter reaches the
   step length, so the first note after Play is on tick 11 (`tests/conformance/AMBIGUITIES.md`, "the first note after Play"). At 120 BPM
   that is 28.6 ms, or 1.4 pulses, after `Start`. **This is not a clock fault and the clock does not hide it**: `clock.rs` pins it by name.
   It is a second reason, beside D0, that a pattern on this clock is not on the receiver's grid, and it belongs to the same owner
   decision. P4d's user offset can correct a constant; it does not remove it.
4. **Turning the clock on while the transport runs says where the sequencer is; turning it off says nothing.** Decision 2 did not
   cover it. On: `Start` at tick 0, otherwise `Continue`, at the next tick's sample, then the pulses. Off: the pulses waiting are
   dropped and no `Stop` is sent, because the sequencer has not stopped, and a `Stop` would stop the receivers while it plays on.
5. **`tick_position()` is the audio's place on the grid and is counted from the last `Reset`.** Decision 4 said "the global tick count plus how far the
   engine is between ticks". The engine steps up to `MAX_EARLY_TICKS` (12) ahead of the audio, so the global count is up to twelve ticks ahead of
   what the speakers play, which is what a phase comparison must not see. The position is found from a ring of the last 32 ticks'
   due samples, and it is the plain tick count while stopped. It is a read.
6. **A real-time message that finds no room is dropped, newest first, and counted.** `Engine::realtime_dropped()` reports it; it is not an
   export. `REALTIME_PER_RENDER` is 256 and the longest block the module renders (4,096 frames at 8 kHz and 999 BPM) holds about 205
   pulses, so the page cannot lose one; a test pins that.
7. **`Reset` while running sends `Stop`, and `Play` after it sends `Start`.** Decision 2 said nothing of `Reset`. A `Stop` while stopped
   still sends nothing on the clock and keeps its CC 123 flush (D-P4-7), as decision 2 says.

The ABI (decision 7) is as built: kind 5 with the status byte in `d1`, `octoweb_set_clock`, `octoweb_tick_position`, `octoweb_abi()` 2. Two
details it did not say: the events buffer is **512 records** (256 for notes and 256 for the clock) and was 256, and a real-time send is counted
in the scheduler's `realtime` statistic and not in `sent`.

**The ratchet.** One baseline entry is removed under this ADR: `abi::the_module_reports_abi_version_one`, whose subject (the number `octoweb_abi`
returns) decision 7 changes. `abi::the_module_reports_abi_version_two` replaces it and asserts the new number, so the floor does not loosen:
it rises by the clock's tests (`harness/baseline.txt`, `removed test ... ADR-0009`).

**M3 is still `pending` on D0** (`a_step_is_a_whole_number_of_pulses` is an ignored test that names it). Nothing here claims a pattern is
in time with a clock; with item 3 it is now two reasons and not one.


### Amendment 2 (2026-10-02, Conductor, found in P4d): what the follower, the echo and the clock state do, as built

Decisions 5 and 6 were written before the follower existed. P4d built it red first (the estimator's fixtures, the follower's fixtures, the
clock state's and the echo's, each failing against a stub before the code), measured it, and ran mutants against it
(`handoffs/evidence/p4d-*.txt`). Six places differ from the text above, and the owner is asked to accept or change each. None touches a
contract, a golden hash or a ratchet entry.

1. **The actuator is a first-order loop, not a second-order one.** Decision 5 said "a small trim proportional to the phase error (a
   second-order loop)". It is built as the sender's tempo times `1 + trim`, with `trim = clamp(-phase / 250 ms, ±8%)`: the trim is
   proportional to the error and there is no integral term, so it is first order. A second-order loop was not needed to meet M4 (below), and an integral
   term would add a wind-up to a loop that already has a limit; if S3 shows a constant error that a first-order loop cannot take out, it is the next step.
   The numbers (250 ms, 8%) are the author's and the owner's to change.
2. **The estimator is not a pure function; it keeps state.** Decision 5 said "a pure function takes the timestamp of each incoming pulse". It is a
   class (`apps/web/src/clock-estimator.ts`): a two-state Kalman filter (phase and period) with process noise that adapts when the errors lean one way,
   a seeding of four pulses and a locking of 24 in which the count is not allowed to skip, a gate on the raw spacing of each pulse, a record of the
   anomalies (pulses turned away, gaps) that decides when to start again, and four states (`idle`, `locking`, `locked`, `lost`). It is pure in the
   sense that matters, deterministic and without a clock of its own; and it is exercised on simulated senders only. The reason it is more than a window of
   averages is in `p4d-estimator-measured.txt`: through a step in tempo the pulse count went wrong in 42 of 100 runs until the anomaly record was added.
3. **The lead and its sign.** A positive offset makes the notes land earlier: the engine is that much further ahead of the sender's grid than the lookahead
   alone asks. That is the opposite sign to Live's "MIDI Clock Sync Delay" (which delays what Live sends), and ABLETON.md (P4e) says so beside the
   two preferences. The follower's lookahead must be the scheduler's: the page keeps them equal (`applyLookahead`), and a test shows what happens when they are not.
4. **Start, Continue and Stop, and what the follower does not follow.** Start and Continue act at the next pulse; Stop acts at once; a pulse alone never starts
   the transport; with no pulse for 500 ms the status says so and nothing more is sent. At the first pulse after a Start the engine's whole-pulse position is
   taken as the sender's pulse count, so a downbeat is on the right pulse and not a whole pulse off; the engine starts about a lookahead late (more, in a
   browser: about 53 ms at the first position report) and runs up to 8% fast for about a second to catch up, so **the first second after a Start is not on the
   beat and the first bar is slightly compressed**. Song Position (`0xF2`) is decoded and **ignored**: a Continue from the middle of a song resumes where
   the sequencer stopped, not where the sender is. This is a limit of the design, not of the tests, and ABLETON.md says it.
5. **The echo does not wait for the lookahead.** Decision 6 said the echo is "stamped like the notes and with the same ordering rule". It is sent at once, with no timestamp,
   to every distinct chosen device (`MidiScheduler.passThrough`), counted with the other real-time sends and not reported to `onSend`. A pulse that has
   arrived from the sender is already as late as the page will ever be; stamping it ahead by a lookahead would delay the echo by that much against the
   sender's grid, which is the opposite of what an echo is for. The ordering rule is not applied to it, so a Stop passed on at once can reach a device
   before notes the engine had already stamped a few milliseconds ahead: a few notes after a Stop is the cost, and the engine's own flush follows. Echo into a
   loop (to the port the clock comes from) is **the user's to avoid**: no code can see it through a virtual port, and the sentence says so every time it is shown.
6. **The clock state and what refuses it.** Four states, off first and the default (the manual's), with the manual's own words as labels. Master and slave are
   exclusive. A choice that cannot work is refused with a sentence and the control goes back to Off (M8); a choice made before Start, or while MIDI access is
   awaited, is kept, says it waits, and takes effect when MIDI has been asked for (M7). The status sentence is a live region that changes only when the state
   does; the numbers (tempo, pulses, jitter, how far ahead or behind the engine is) are in a second line that is not live.

**Defaults taken, for the owner to overturn.** (a) While slaved, the local Play key still sends the engine's Play and is not gated, so a person can start the
engine against a stopped sender; the follower then believes the transport is stopped and a Stop from the sender still stops it. (b) The tempo knob is
overwritten as soon as the sender's tempo arrives. Neither is hidden; both are things a manual-faithful workflow, which the Panelwright owns, may want otherwise.

**Thresholds, restated.** M4 says "a tempo within 0.1 BPM and a phase error within one tick out, after 2 seconds" at 120 BPM with up to 3 ms of jitter. Met, with the real
engine, worklet and scheduler and with the output's own noise off: the worst heard error on seeds the tests do not use is 0.38 ticks from 2 s (0.53 at 160 BPM),
0.24 after a step in tempo, 0.34 after a ramp, and **3.51 on a ramp of 3.75 BPM a second against a bound of 4**: the loop does not have much room there.
**With the output's own noise on (the page's idea of the audio clock wrong by up to 1.5 ms) a tick is not reliably held at 160 BPM** (worst 1.11 of 1.9 ms), and the
test for that is not written, because its bound would not hold. All of these bounds are the author's and the owner's to change; none was loosened (the first
drafts were tightened). `p4d-follower-measured.txt` holds the tables.

**Not established, and the owner should not read it as established.**
- Everything above was measured against a **made-up sender** and, in a browser, a timer-driven one. Whether a real port's `event.timeStamp` is on `performance.now()`'s
  clock is **assumed** (S3 measures it). Whether the audio-to-page map's error is the same for the position the follower reads and the events the scheduler
  stamps (they share one map, so it may largely cancel) is **unknown**; the two noises are drawn independently in the simulation.
- **A finding about Chromium that touches every clock number here:** at the start of an AudioContext, `currentFrame` jumps 896 or 1024 frames between the first and the
  second `process()` call (10 of 12 fresh contexts, headless; `p4d-startup-frame-jump.txt`), and the engine renders only the calls it gets. A Play within about 20 ms of that
  has a hole in its first pulse. The app does not play that early, and the P4c test that measures pulse spacing was flaky for exactly this reason until it waited. What a
  real device and a hidden tab do is for S1 and S3; and if `process()` can skip mid-session the engine falls behind the audio timeline for good, which master mode would
  not correct and slave mode would. That is a question for the Metronome and the owner, not decided here.
- The 11-tick lag of the engine's notes behind its pulses (amendment 1, item 3) is **unchanged**, and the follower lands the *pulses* on the sender's grid: a pattern's notes
  sit 11 ticks after it, at any tempo. With D0 (the tick length) still open, **M3 stays `pending`** and nothing here claims a pattern is in time with a clock.

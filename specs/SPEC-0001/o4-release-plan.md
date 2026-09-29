# WENGE-0004 (O4): release plan for the integer tick clock and host lock

| Field | Value |
|---|---|
| Task | `WENGE-0004`, opportunity O4 of SPEC-0001 |
| Spec revision | r2 (the owner approved a release plan and a re-read of the specs, and nothing else) |
| Tier | **High** (`AGENTS.md`: clock and host-sync rewrite). Needs both specs, a pull request and this release plan |
| Status | **Plan for sign-off. No O4 code exists, and none will be written until the owner approves this plan.** Nothing in `crates/` changes in this pull request |
| Written by | Metronome, from measurements of the engine at `c1e04c1` (the last engine change in PR #11). Programs and outputs are in `handoffs/evidence/o4-baseline/` and `handoffs/evidence/o4-*.txt` |
| Asks of the owner | Section 0. Six decisions, each with a default |

## 0. What I am asking you to decide

Nothing gets built until you answer D1. D2 to D6 each have a default that I will follow if
you approve without comment, so you only need to speak up where you disagree.

| # | Decision | My default | If you say no |
|---|---|---|---|
| D1 | Approve, approve with changes, or reject this plan | Approve, so the three pull requests in section 5 can start | Tell me what to change. I revise the plan, not the code |
| D2 | **Drop the PLL from `octocore`.** `tech.md` F2 and `docs/10` R4 say to build one | Drop it. Section 2.6 says why: a plug-in host counts samples exactly, so there is nothing to lock to. A PLL belongs in a caller driven by a wall clock, which is `octorun --live` (not approved, Q6) | I add a PLL to O4b and the plan gets a fourth risk: a PLL on an exact sample clock can only add jitter |
| D3 | **State the ramp accuracy by buffer size** instead of one number. Section 3 has the table. On the reference ramp a steady ramp is exact at every buffer size, and the buffers where a ramp *starts* are within 1 sample up to 512-sample buffers and within 0.45 ms (21 samples) at 4,096 | Accept the table. The current engine is 1,160 to 1,830 samples out on the same ramp | Say which number you want held at 1,024 and above. It cannot be met from a host that reports the tempo only at the start of a buffer, so O4 would then need a host that reports the tempo at the end too |
| D4 | **Who owns the transport**: the host's `playing` flag or the `Play` and `Stop` commands. Today the flag wins every render, so a `Stop` from the ring is undone one render later | Edge-triggered: a change of the host's flag acts, and so does a command, and the last change wins. Section 2.5 | Level-triggered as today (commands can never stop a host that says "playing"), or commands only (breaks every fixture that passes `playing` alone) |
| D5 | **What a locate or loop wrap does.** The manual is silent (pp.93 and 94 are the only pages on external clock) | Release every sounding note at the jump, and leave each track's step position where it is. Both readings go in `AMBIGUITIES.md`. Section 2.4 | "Chase": step silently from the start to the new position so patterns land where a linear play would put them. Deterministic, but random and effector state depend on history, so it cannot be exact. A follow-up task, not part of O4 |
| D6 | **Three pull requests, not one.** 4a exact step accumulators, 4b tick timeline, 4c host position | Three, in that order, each revertable alone | One large pull request. I advise against it: the review cost is the whole clock at once |

## 1. What the clock does today, measured

Every row is reproducible with `handoffs/evidence/o4-baseline/README.md`. The programs drive
the real `Engine::render` on a synthetic host and compare with the analytic answer. A
"buffer" is the number of samples the host asks for per call. One sample is 20.8 µs at
48 kHz. One tick (1/192 of a quarter note) is 125 samples, or 2.6 ms, at 120 BPM.

| # | Measurement | Result | Reading |
|---|---|---|---|
| A | Constant tempo (120 BPM at 48 kHz, 133 at 44.1 kHz, 97.3 at 48 kHz, 200 at 96 kHz), buffers 64 to 4,096, 20 s each: how far each note-on lands from the ideal grid | Worst 1.02 samples (at most 21.8 µs), σ 0.29 samples (6.5 µs). Identical at every buffer size. Exactly 0 at 120 BPM and 200 BPM | **The `docs/03` §7 jitter target (σ ≤ 120 µs, max ≤ 500 µs) is already met at constant tempo, about 20 times under.** The 1 sample is just rounding a fractional sample down |
| B | 60 to 180 BPM ramp over 8 bars (16 s), host reports the tempo at the start of each buffer | The last note is **24.05 ms late at 32-sample buffers, 25.6 ms at 256, 30.9 ms at 1,024, 38.1 ms at 2,048** (1,154 to 1,827 samples) | **This is the real defect.** 24 ms is about 50 times the 500 µs maximum in `docs/03` §7 |
| C | 60 to 120 BPM ramp over 4 s, then constant 120 BPM until 16 s | Lateness climbs to 23.92 ms by the end of the ramp and is **still exactly 23.92 ms 12 seconds later** | The offset never recovers. After any tempo automation the sequencer sits permanently behind the host's beat grid until it is stopped and started |
| S | The cause, tested: the same ramp with the host reporting the tempo `shift` ms *later* than the buffer start | Lateness at shift 0, 10, 20, 25, 30, 40 ms: 25.6, 18.9, 12.3, 8.9, 5.6, -1.1 ms | Lateness falls in a straight line as the reported tempo is moved earlier, and reaches zero near 38 ms, which is inside the engine's 21 to 63 ms lookahead window. So the lateness is the staleness of the tempo, as the code below says |
| F | The per-track `f32` step accumulator against the exact tick, for every reduced multiplier num/den up to 16 (159 of them) | 30 minutes at 120 BPM: 49 multipliers fire some step 1 tick late or early. 7 hours: 80, still at most 1 tick. 72 hours: 87, worst error **9 ticks (23 ms)** | Small in a session, but it grows with time (1 tick at 7 hours, 9 at 72): the `f32` step length is a slightly wrong number, so the error builds up |
| R | Who can set a track's speed multiplier today | Nothing but a test writing the field. It is not in `TrackAttr`, the FFI, or `octorun` | The accumulator defect is **latent**: no host can hear it yet. It gets audible when the panel or phrases can change a track's speed |

**Why B and C happen, from `engine.rs` `render_core`.** The engine fixes each tick's length
when it *steps* the tick: `next_tick_due += spt`, where `spt` (samples per tick) comes from
the tempo in the `render` call that stepped it. Ticks are stepped `MAX_EARLY_TICKS` (12) plus
part of a chunk ahead of the moment they sound (that lookahead is O3's fix and stays). During
a ramp every tick is therefore measured with the tempo as it was up to 12 ticks earlier (21 to
63 ms across this ramp), too slow, and the total of those small errors is the offset. Nothing
ever corrects it, because the engine keeps no idea of where the host thinks the beat is.

**What this changes about the case for O4.** `findings.md` presents O4 as jitter, PLL and
drift. The measurements say: jitter at constant tempo is fine, a PLL would not help
(section 2.6), and the defect is a permanent offset after a tempo ramp (a tempo jump has the same cause; R5 will measure it) plus a
locate/loop that does nothing. That is still worth a high-tier change, but it is a smaller
and better-defined change than the spec suggests.

## 2. The change

### 2.1 Two layers, as `docs/02` §3 already says

`docs/02` §3 gives the core `tick(ppqn_pos: u64)`: it is told a position and never asks the
time. The engine today is the opposite, and O4 moves it toward the document:

1. **The tick layer** (sequencing). Steps integer ticks. Every event it schedules carries an
   integer tick position: the tick it fired on plus integer offsets (start offset, strum,
   length, all of which are already whole ticks in the code). No `f64` anywhere in it.
2. **The clock layer** (new file `clock.rs`, still pure, no I/O, no time source). Knows the
   sample rate, the tempo and the position at the start of the buffer. Answers two questions:
   *which ticks fall in this buffer*, and *at which sample offset does each one land*. This is
   the only place a tick becomes a sample.

The event queue is ordered by `(tick, rank, sequence)` instead of `(whole sample, rank,
sequence)`. The two orders are the same whenever a tick is at least one sample long. The
engine's limits guarantee that (8 kHz and 999 BPM give 2.5 samples per tick), and a test
asserts it.

### 2.2 Converting a tick to a sample

At the start of each buffer the clock layer holds an anchor: the position `p0` in ticks and
the tempo `b0`. A tick `T` inside the buffer lands at

`sample = (T - p0) * spt(b0)`, floored, measured from the buffer start.

That is one multiplication from an integer and an anchor, not a running sum, so at a constant
tempo nothing accumulates. Notes are stepped 12 ticks ahead as now, but they wait in the queue
as tick positions and are converted only when the buffer that holds them is rendered, with
the tempo of that buffer. That removes the cause in section 1.

For tempo that is changing, the buffer is modelled as a straight-line tempo change: `b0` at
the start and a slope taken from the previous buffer's change. **The slope is used only when
the last two changes agree within 5%.** Otherwise the buffer is modelled at constant tempo.
This is the guard that stops a sudden tempo jump (say 120 to 140 in one buffer) from being
turned into a huge fake slope for the next buffer. The guard is a test (R5), not an
assumption.

### 2.3 Host position and free-running

| Mode | When | What anchors each buffer |
|---|---|---|
| Host position | The host gives the position in ticks (fractional) at the start of every buffer | The host's position. No error can build up: each buffer restarts from the host's answer |
| Free-running | Only `bpm` and `playing`, as every caller passes today | The engine's own position, advanced by the same model. Error can build up, but far less than today (section 3) |

Both modes use the same clock layer. `RenderContext` is not changed. That would break the
18 places that build it. A new method `Engine::render_at(ctx, position, out)` takes the
position, and `render(ctx, out)` calls it with none. The `HostTransport` command keeps its
`u64` position and now means "locate to this tick at the start of the next render", for
hosts that push transport changes through the ring. When `render_at` is given a position,
that position wins over a queued `HostTransport`.

The engine's own position (free-running mode) is held as whole ticks (`u64`) plus a fraction below one tick, so its precision does not fall as a session runs on. The sample clock becomes a `u64` count of samples, not an `f64`. Both are exact up to 2^53,
so nothing changes in a normal session; it makes the "no float accumulation" statement in
`docs/02` §5 true for the sample side too.

### 2.4 Locate, loop and discontinuity (host position mode)

- The engine expects each buffer's position to equal where the last buffer's model ended,
  within 1 tick, or within the model's own error bound for that buffer if that is larger (at
  999 BPM a tick is only 2.5 samples). A host that reports a slightly different position each buffer (rounding in
  the host) is followed silently, and that is test R9.
- A difference of more than 1 tick is a **discontinuity**: a locate, a loop wrap, or a
  tempo-map change. The engine releases every sounding note at sample 0 of that buffer (a
  `NoteOff` for each, no CC 123, so a loop does not spray controller messages), drops the
  events still queued for the old position, and re-anchors on the new one. Step positions
  are left alone (D5). The next tick lands within 1 sample of the analytic position.
- A locate while stopped moves the anchor. The next Play starts from there.
- A host that wraps a loop *inside* a buffer without splitting the buffer is seen one buffer
  late. I cannot fix that from the engine. It is in the risks.
- New counters (discontinuities, positions corrected) go in a **new** struct with its own
  getter. `OctoDiagnostics` gets no new field, because a new field in an existing `repr(C)`
  struct is an ABI break and that is high tier by itself.

### 2.5 Transport ownership (D4)

The engine remembers the host flag it saw last render. When the flag differs, the engine
acts on it. A `Play`, `Stop` or `Continue` command acts immediately. So the last change wins,
from either side, and a `Stop` from the ring is no longer undone one render later. A caller
that always passes `playing: true` and never sends `Stop` behaves exactly as today. The
change is visible only to callers that use both, and today those callers are buggy.

### 2.6 Why no PLL in `octocore` (D2)

A plug-in host hands the engine a buffer of N samples at a time, and every sample is
accounted for: sample 0 of the next buffer is N samples after sample 0 of this one, exactly.
There is no second clock in the plug-in to lock to, so a PLL has nothing to correct. With
the host's position (2.3) the engine also knows where the beat is. Variable buffer sizes are
covered already: the event stream is identical at every buffer size from 1 to 4,096 and for
random sequences (O3, tested).

A PLL is the right tool where the only clock is wall time and callbacks arrive irregularly:
a standalone app, or `octorun --live` with `midir`. That is where the `docs/10` R4 trigger
("loopback jitter above 200 µs in Live") could actually be caused by a wall clock. It
belongs in that caller, which turns wall time into a sample timeline and hands `octocore`
the exact buffers this plan is written for.

### 2.7 Integer step accumulators (PR 4a)

Today: `accum += 1.0; if accum + 1e-6 < step_ticks { return } accum -= step_ticks`, with
`step_ticks = 12 / (num/den)` in `f32`. The `1e-6` is a patch for the `f32` error, and (F)
shows it does not hold.

New: the step is exactly `12*den/num` ticks, so keep the phase as a whole number of
`1/num`-tick units. Each tick adds `num`, and the step fires when the phase reaches
`12*den`, then subtracts `12*den`. A step shorter than one tick fires every tick (as the
`max(1.0)` does today). A hyperstep-linked track keeps its 192-tick step. When a track's
multiplier changes, the phase in ticks is carried over and rounded down to a whole unit of
the new step, which loses less than one tick, and R1 states it. Integer arithmetic on `u8` numerator and
denominator, so it is exact for every value the fields can hold, not only up to 16.

## 3. Acceptance: F2 and A4, criterion by criterion

"On the null host" means the synthetic host in section 4. "Before rounding" means the time
the clock layer computes, before it is floored to a whole sample, because the floor alone
costs up to 1 sample at any tempo and is not a timing error.

| Criterion | Where it comes from | Today (measured) | After O4 | PR |
|---|---|---|---|---|
| Constant-tempo jitter σ ≤ 120 µs, max ≤ 500 µs, buffers 1 to 4,096 and random sequences | `docs/03` §7, F2 | **Already met**: max 21.8 µs, σ 6.5 µs (A) | Still met (a **guard**, G1). Not "fixed" by O4 | all |
| Ramp, host position mode: notes within 1 sample of analytic | A4, F2 | 1,160 to 1,830 samples out (B) | Steady ramp: **0.000 samples at every buffer size** (model). In the two buffers where the ramp starts and its slope is not yet known: 0.33 samples at 512, 1.3 at 1,024, 5.2 at 2,048, 21.3 (0.44 ms) at 4,096. See D3 | 4c |
| Ramp: no lasting offset once the tempo is constant | A4 ("ramp drift") | 23.92 ms, permanent (C) | Host position mode: 0 ticks, by construction, because every buffer restarts from the host. Free-running: 0.23 samples at 512-sample buffers, 0.9 at 1,024, 3.6 at 2,048, 14.6 at 4,096 (model, the last note of the ramp) | 4b, 4c |
| Ramp, free-running (no host position): the case every caller is in today | not in the spec | 1,160 to 1,830 samples (B) | Worst over the ramp: 0.68 samples at 512, 2.7 at 1,024, 10.8 at 2,048, 42.8 at 4,096 (model). Within 1 sample only up to 512-sample buffers | 4b |
| Locate to any tick within 1 sample; loop wrap | A4 | Ignored: `ppqn_pos` is never read | Next tick within 1 sample of analytic. Sounding notes released at the jump; NoteOn/NoteOff balance holds across it | 4c |
| Step timing exact for every multiplier | O4 finding ("integer per-track step accumulators") | 49 of 159 within 30 min at least 1 tick out; 9 ticks in 72 h (F) | Exactly `ceil(k * step)` for every k, every `u8` num/den | 4a |
| A thousand simulated hours without missed ticks | `docs/03` §7 "missed ticks in 30 min: 0" | Not measured | Tick position after N ticks is exactly N, on the tick layer alone (integers, no `f64`) | 4b |
| Nothing else moves | invariant | n/a | G2 to G6 below | all |

**D3, in plain words.** The spec asked for "ramp drift at most 1 sample" with no buffer
size. The model says: once the engine has seen two buffers of a ramp it knows the slope and
a straight ramp is exact. In the first two buffers it does not know a ramp has started, and
the error there grows with the square of the buffer size: under 1 sample up to 512, 1.3 at
1,024, 21 (0.44 ms, still inside the 500 µs max-jitter target of `docs/03` §7) at 4,096.
Those are the numbers for the reference ramp (60 to 180 BPM over 8 bars). A steeper ramp
is worse, so R2 and R3 do not check a table: they check each note against a bound computed
from the ramp rate and the buffer size (half the rate of tempo change times the buffer time
squared, in beats, plus 1 sample for rounding), and they separately assert the 512-sample
row above. If you want 1 sample held at larger buffers, only a host that reports the tempo at
the end of the buffer as well can do it.

## 4. Verification plan

Written before any code, as for O6. Each test has an ID that appears in its name, so the
trail from this plan to `harness/baseline.txt` can be searched. Tests marked *red* are shown
failing on the parent commit and passing after, with the output in
`handoffs/evidence/o4-*-red.txt`. Tests marked *guard* pass on the parent commit and must
still pass. A guard is proved to protect something by a mutant: I break the new code on
purpose and show the guard fail (the way `o6-mutants.txt` does for the loom tests).

**The null host** (a test helper, no engine change): produces buffers of a given size or a
seeded random size sequence, with the position and tempo taken from an analytic tempo curve
(constant, linear ramp, jump, ramp then hold, loop). It records what the engine emits and
compares each note with the analytic time. The `(buffer, position, tempo, playing)` list it
feeds is also written as NDJSON, so any failure is a replayable "host clock trace" (`product.md` invariants).

| ID | Kind | Asserts | PR |
|---|---|---|---|
| G1 | guard | Constant tempo, 4 tempo and rate pairs, buffers 1 to 4,096 and 20 random sequences: σ ≤ 120 µs and max ≤ 500 µs. This is the first content for the `verify:timing` gate | first commit |
| G2 | guard | The five `octorun` golden hashes. Four are at constant tempo (`hello`, `chords_and_strums`, `effector`, `phrases`) and must be **byte-identical** in every PR. The fifth, `mcc_and_transport`, goes from 100 to 140 BPM mid-run and includes stop and play cycles. It is byte-identical in 4a and 4c. In 4b it is **expected to change**, only for events stepped before the tempo change and sounding after it. The PR shows that diff event by event, explains each, and regenerates the golden (`just golden`) with the reason in the commit | first commit, then all |
| G3 | guard | The O3 buffer-size invariance tests (sizes 1 to 4,096 and random sequences) | first commit, then all |
| G4 | guard | Zero allocation in `render` with and without a link (the counting-allocator test from O6) | first commit, then all |
| G5 | guard | Conformance fixtures 100% (`verify:conformance`) | first commit, then all |
| G6 | guard | Worst full-density tick time, measured before and after (`docs/03` §7 says at most 250 µs). Reported, not asserted on a shared runner, as in `o6-cost.txt` | 4b |
| R1 | red | For every `u8` num and den, the k-th step fires on tick `ceil(k*12*den/num)`, over 10^7 ticks by simulation and to 10^12 by closed form. Changing the multiplier mid-run loses less than one unit of phase | 4a |
| R2 | red | Free-running ramp 60 to 180 BPM over 8 bars, and a steeper one (30 to 300 BPM in 2 bars): every note within the computed bound of D3, and within 1 sample at buffers up to 512, for buffers 32 to 4,096 | 4b |
| R3 | red | Same ramps, host position mode: within the bound, and 0 outside the first two buffers | 4c |
| R4 | red | 60 to 120 ramp then hold: offset vs the host grid is 0 ticks (host) or at most 1 sample up to 512-sample buffers (free-running) at 12 s after the ramp | 4b, 4c |
| R5 | red | Tempo **jump** 120 to 140 in one buffer: the buffers after it are exact (the slope guard), and the jump does not produce more error than a constant-tempo run | 4b |
| R6 | red | Locate forward, backward, and 100 loop wraps: next tick within 1 sample; every sounding note gets one NoteOff; NoteOn minus NoteOff is 0 per (port, channel, note) | 4c |
| R7 | red | Transport: `Stop` from the ring stays stopped while the host flag stays "playing"; a host flag change acts; a caller that only passes `playing` is unchanged | 4c |
| R8 | red | 1,000 simulated hours on the tick layer alone: tick N is exactly N, and a 10^8-buffer free-running constant-tempo soak stays within 1e-6 tick (about 0.0001 sample) of the analytic position | 4b |
| R9 | red | A host whose reported position is off by up to 1 tick from buffer to buffer is followed with no flush and no counter change | 4c |
| R10 | red | Locate while stopped, then Play: the first tick is at the located position | 4c |
| R11 | red | An edit made by a command while playing takes effect no later than today (no new latency) | 4b |

**Mutants for the new code** (each must make at least one test fail): the anchor uses the
previous buffer's tempo; the slope guard is removed; the sample offset is rounded instead of
floored; `T - p0` off by one tick; the discontinuity threshold is 0 or 10 ticks; the
transport flag is level-triggered again; the accumulator adds `num` but subtracts `12*den+1`.

**Independent review.** Before each pull request is marked ready, a fresh session fills in the
`AGENTS.md` checklist. The clock layer gets a second reading of the arithmetic in section 2.2
by a session that has not seen this plan's numbers.

## 5. Pull requests, files and order

Stacked on `metronome/command-ring` (PR #11), because O6 changed `engine.rs`. Each one is
independently revertable.

| PR | Task | Contains | Tier | Emitted MIDI changes? |
|---|---|---|---|---|
| 4a | WENGE-0004 | Integer step accumulators (`steps.rs`, `TrackRuntime` field, `step_one_track`). G1, R1. Smallest and safest, so it goes first | Medium | Yes, for speed multipliers other than ×1 that the `f32` version gets wrong (1 tick, 2.6 ms at 120 BPM). Not reachable from a host today (R in section 1) |
| 4b | WENGE-0004 | `clock.rs`, tick-keyed queue, drain converts tick to sample, `u64` sample clock, free-running mode with the slope model. G2 to G6, R2, R4, R5, R8, R11 | **High** | Yes: notes during a tempo change sit on the grid. Constant tempo is byte-identical (G2); the one golden with a tempo change is regenerated |
| 4c | WENGE-0004 | `Engine::render_at`, discontinuity handling, transport edge rule, `HostTransport` semantics, additive FFI: `octocore_engine_render_at`, `OctoHostPosition`, a new counters struct. `OCTOFFI_ABI_VERSION` 2 to 3. R3, R6, R7, R9, R10 | **High** | Yes: a locate releases notes. Callers that never give a position see no change |

Files: `crates/octocore/src/engine.rs`, new `clock.rs` and `steps.rs`, `types.rs`
(no layout change), `crates/octocore/tests/` (null host, R and G tests), `crates/octoffi/`
(additive), `harness/baseline.txt`. Files owned by others that will need requests: `docs/02`
§5 (lines 216 and 220; the second says "no floating-point accumulation" and describes the
target, not the code) and `docs/03` §5 and §7 (Scribe); the `verify:timing` gate (Referee).

Nothing in the current `Command` or `Event` layout changes. If a `repr(C)` layout change turns
out to be needed, that is a stop and a new question to you, not a decision I make.

## 6. Risks

| Risk | Consequence | Mitigation |
|---|---|---|
| The clock layer has a bug that only shows on a real host | A late or doubled note on a real machine, which `N5` calls a catastrophe | No host is available here (no Xcode, no Live). The null host proves the arithmetic, not the integration. **Before this ships, a loopback test in Live 12 on your machine** (`docs/03` §7's "lab" number) is a release condition. I say so plainly: this plan cannot promise it |
| Existing golden or conformance output changes | The ratchet blocks, or worse, is "fixed" | G2 and G5 must be byte-identical at constant tempo. `mcc_and_transport` changes tempo mid-run, so its golden is *expected* to change in 4b and is regenerated with each difference explained. Any other difference that is not a rounding-boundary flip stops the work (kill criteria) |
| The slope guess is wrong for non-smooth automation | An error for one or two buffers, and in free-running mode that error stays as an offset | R5 tests the jump. The guard falls back to constant tempo, and costs one extra buffer before the slope is used (which is why free-running is 0.68 and not 0.34 samples at 512). Host position mode forgets the error at the next buffer |
| A host loop wrap inside a buffer | The wrap is seen one buffer late | Documented limit. Most hosts split buffers at a loop end. Verified only in the loopback test above |
| A host reports positions that wobble | A false discontinuity and a flush of sounding notes | The 1-tick tolerance and R9. If a real host needs more, the threshold is a named constant that can go up (loosening it is the owner's call, with an ADR) |
| Edit latency (up to about 31 ms at 120 BPM, from the 12-tick lookahead) | Not changed by this plan | R11 asserts it does not get worse. `docs/02` §6 budgets 1.3 ms; that mismatch is already with the Scribe |
| Three pull requests take longer than one | Slower to review | Each is small and each fixes something on its own. 4a and 4b are useful even if 4c is never approved |
| The slope model is over-engineered | More arithmetic than needed | It is about 10 lines. Without it a steady ramp is off by up to 0.33 samples at 512 and 21 at 4,096 in host mode, and free-running mode drifts by 170 samples (3.6 ms) at 512 by the end of the ramp (`o4-model.txt`) |

## 7. Rollback, kill criteria, migration

- **Rollback:** each pull request is one `git revert`. Nothing persists (no saved-state format
  exists), no host consumes the engine yet, and the FFI change in 4c is a new function, so
  reverting it removes a function nobody calls. The parent tip is the rollback point.
- **Stop and report** (not "try harder") if: G2 or G5 differ at constant tempo and it is not a
  rounding-boundary flip; a red test is not green after 3 attempts (`AGENTS.md` limit); R2 cannot
  meet its row at 512-sample buffers; the full-density tick goes over 250 µs; or a `repr(C)`
  change looks necessary.
- **Migration:** none. `RenderContext`, `Command`, `Event`, `Snapshot` and `OctoDiagnostics` keep their
  layouts. The C header gains one function, one struct and the version number.
- **Health signals for later:** `late_events` stays 0 on every null-host run; the new
  discontinuity counter is 0 on a straight play-through and equals the number of loop wraps on a loop.

## 8. The re-read of `product.md` and `tech.md`

What I found reading them again for this plan. None of it is applied until you approve; the
right-hand column is the revision r3 text I would propose.

| Where | What it says | Problem | Proposed r3 text |
|---|---|---|---|
| `product.md` A4 | "Ramp drift is at most 1 sample" | No buffer size, and "drift" is not defined | "Before rounding: exact on a steady ramp; in the two buffers where a ramp starts, within 1 sample up to 512-sample buffers and within 0.5 ms up to 4,096 (reference ramp). No lasting offset after the ramp in host position mode" |
| `tech.md` F2 row | "Analytic jitter σ ≤ 120 µs, max ≤ 500 µs at every buffer size" | Already true today; reads as a task | Keep as a guard (G1), and say it holds at baseline |
| `tech.md` F2 kill | "If the PLL cannot meet σ under variable buffers, ship fixed-latency mode" | Presumes a PLL (D2) | "If the tick timeline cannot meet the ramp table at 512-sample buffers, stop and report" |
| `tech.md` diagram | "Tempo map and PLL" | Same | "Tempo model" |
| `tech.md` Clock line | "Sample time is derived per tempo segment, never accumulated" | Matches 2.2 for constant tempo. During a ramp each buffer is its own segment, so the origin is rebased each buffer | Add: "During a ramp, per buffer, from the host position when there is one" |
| `tech.md` Known risks | "Lookahead delays command effects by up to 31 ms" | Right, and unchanged by O4. `docs/02` §6 says 1.3 ms | No change here. Already with the Scribe |
| `findings.md` O4 evidence | "each track keeps an f32 step accumulator" | True, but no host can set a multiplier yet, so it is latent | Add the (R) row from section 1 |
| `findings.md` O4 assertion | "A locate to any PPQN puts the next tick within 1 sample" | The manual is silent on locate (pp.93 and 94) | Add the AMBIGUITIES entry and D5 |
| `product.md` invariants | "deterministic given seed, command log and host clock trace" | The trace format is not defined | The null host writes it as NDJSON (section 4) |
| `tech.md` "Test plan" | "30 minutes at 120 BPM is 691,200 ticks" | Right | Used for R8 |

## 9. What this plan does not cover

The design system and DSP (Q1). Live MIDI in `octorun`. Legato, the panel, or any
renderer work. A real host. Chase on locate (D5). Anything that needs a Live 12 machine.

## 10. After you approve: the next three steps

1. Write the guard tests G1 to G5 on the parent commit and show them passing (they must pass
   before any O4 code, or they prove nothing), plus the null host. Commit that alone.
2. PR 4a, red-first (R1), then 4b, then 4c, each pushed and opened as its own pull request
   with the gate numbers in the commit message.
3. File the Scribe and Referee requests when each PR opens, not at the end.

If you reject it or ask for changes, the next step is a revision of this file, and nothing else.

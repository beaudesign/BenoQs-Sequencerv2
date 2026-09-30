# WENGE-0004 (O4): release plan for the integer tick clock and host lock

| Field | Value |
|---|---|
| Task | `WENGE-0004`, opportunity O4 of SPEC-0001 |
| Spec revision | r2 (the owner approved a release plan and a re-read of the specs, and nothing else) |
| Plan revision | 2. Revision 1 was read by a session that had not written it; section 11 lists what that review found and where each finding went |
| Tier | **High** (`AGENTS.md`: clock and host-sync rewrite). Needs both specs, a pull request and this release plan |
| Status | **Plan for sign-off. No O4 code exists, and none will be written until the owner approves this plan.** Nothing in `crates/` changes in this pull request |
| Written by | Metronome, from measurements of the engine at `c1e04c1` (the last engine change in PR #11). Programs and outputs are in `handoffs/evidence/o4-baseline/` and `handoffs/evidence/o4-*.txt` |
| Asks of the owner | Section 0. Seven decisions, D0 to D6, each with a default |

## 0. What I am asking you to decide

**D0 is a finding, not a build item, and you should read it first.** It changes what every
millisecond figure in this plan means. Nothing gets built until you answer D1. D2 to D6 each
have a default that I will follow if you approve without comment, so you only need to speak
up where you disagree.

| # | Decision | My default | If you say no |
|---|---|---|---|
| D0 | **The engine's tick is four times too short.** The code counts 192 ticks per *quarter* note (`domain.rs:33`, `TICKS_PER_QUARTER`; `docs/02` §5 line 221 says "192 PPQN"). The manual counts 192 ticks per *whole* note: p.15 "Each Green increment corresponds to 1/192 of a note and each Red value corresponds to 12/192 = 1/16 of a note", p.16 "one full note - 192/192", p.67 "A measure is 16 steps at x1 speed". So a default step (12 ticks) is a 1/16 note, 125 ms at 120 BPM, and the engine plays it in 31.25 ms. Every step plays four times too fast at every tempo, and 16 steps last 0.5 s at 120 BPM where a bar of 4/4 lasts 2 s. `docs/03` line 43 already says "PPQN 192 … one step at default length = 12 ticks = 1/16", which is only true if 192 is a whole note, so the docs disagree with themselves as well as with the code. Section 1.1 has the evidence | **Do not change the engine in O4.** File it as `WENGE-0012` (triage only, high tier, not started; the record is in this pull request). 4a does not depend on it, because it counts in ticks. I would ask you to decide it *before 4b*, so that the goldens change once, not twice. If you agree the manual is right, `WENGE-0012` becomes the task that fixes it, with its own spec | Tell me why the code is right (for example, a real Octopus at 120 BPM plays 16 steps at ×1 in 0.5 s, which would mean BPM counts something other than quarter notes). I record that in `AMBIGUITIES.md`, and ask the Scribe to correct `docs/03` line 43. One thing you can check on hardware: **at 120 BPM, does a 16-step pattern at ×1 with default step lengths take 2 s or 0.5 s?** The manual never says "PPQN" or "quarter note", so the manual alone cannot say what unit BPM counts; it only fixes 12/192 as a 1/16 |
| D1 | Approve, approve with changes, or reject this plan | Approve, so the three pull requests in section 5 can start | Tell me what to change. I revise the plan, not the code |
| D2 | **Drop the PLL from `octocore`.** `tech.md` F2 and `docs/10` R4 say to build one | Drop it. Section 2.6 says why, and what host position mode does instead (which is smoothing, not a PLL). A PLL belongs in a caller driven by a wall clock, which is `octorun --live` (not approved, Q6) | I add a PLL to 4c and the plan gets a fourth risk: a PLL on a sample-exact clock can only add jitter |
| D3 | **Accept a weaker ramp criterion than A4 states.** `product.md` A4 says "ramp drift is at most 1 sample" with no buffer size. Section 3 has what can be met: on the reference ramp a steady ramp is exact at every buffer size, and in the two buffers where a ramp *starts* the error is within 1 sample up to 512-sample buffers and 21 samples (0.44 ms) at 4,096. **This is a loosening**, so under `CLAUDE.md` rule 3 it needs an ADR with a reason and an expiry, written by the Conductor. Approving D3 is the request for that ADR; 4b and 4c do not merge without it. Proposed reason: a host that reports the tempo only at the start of a buffer gives the engine no way to know a ramp has begun. Proposed expiry: the first host adapter that reports the tempo at both ends of a buffer, or 2027-06-30, whichever comes first | Accept the table and the ADR. The current engine is 1,160 to 1,830 samples out on the same ramp | Two other options. (a) **Fixed latency:** the engine delays its output by one buffer (10.7 ms at 512 samples and 48 kHz), so it knows the tempo at both ends of every buffer, and A4 can be met at every buffer size with no loosening. It costs latency on a live instrument and I have not modelled it, so I have no accuracy numbers to quote. `tech.md` F2 already names it as the fallback. (b) Hold 1 sample at every buffer size without added latency: I do not believe that is possible from start-of-buffer tempo alone, because the engine cannot know a ramp has begun until it has seen the tempo change |
| D4 | **Who owns the transport**: the host's `playing` flag or the `Play` and `Stop` commands. Today the flag wins: a `Stop` from the ring is undone **in the same render call** that applies it (the ring is drained just before `render_core` compares the flag). See the `AMBIGUITIES.md` entry "who owns the transport" | Edge-triggered: a change of the host's flag acts, and so does a command, and the last change wins. Section 2.5 | Level-triggered as today (commands can never stop a host that says "playing"), or commands only (breaks every fixture that passes `playing` alone) |
| D5 | **What a locate, a loop wrap or a frozen position does.** The manual is silent (pp.93 and 94 are the only pages on external clock) | Release every sounding note at the jump, and leave each track's step position where it is. A position that stops advancing while the flag says "playing" flushes once and then plays nothing until it moves (section 2.8, item 7). Both readings go in `AMBIGUITIES.md` | "Chase": step silently from the start to the new position so patterns land where a linear play would put them. Deterministic, but random and effector state depend on history, so it cannot be exact. A follow-up task, not part of O4 |
| D6 | **Three pull requests, not one.** 4a exact step accumulators, 4b tick timeline, 4c host position | Three, in that order, each revertable alone | One large pull request. I advise against it: the review cost is the whole clock at once |

## 1. What the clock does today, measured

Every row is reproducible with `handoffs/evidence/o4-baseline/README.md`. The programs drive
the real `Engine::render` on a synthetic host and compare with the analytic answer. A
"buffer" is the number of samples the host asks for per call. One sample is 20.8 µs at
48 kHz. One tick is 125 samples, or 2.6 ms, at 120 BPM **at the engine's tick length** (D0:
if a tick becomes four times longer, every time figure in this section that depends on the
tick length, which means B, C, S and F, changes with it; A does not).

| # | Measurement | Result | Reading |
|---|---|---|---|
| A | Constant tempo (120 BPM at 48 kHz, 133 at 44.1 kHz, 97.3 at 48 kHz, 200 at 96 kHz), buffers 64 to 4,096, 20 s each: how far each note-on lands from the ideal grid | Worst 1.02 samples (at most 21.8 µs), σ 0.29 samples (6.5 µs). Identical at every buffer size. Exactly 0 at 120 BPM and 200 BPM | **The `docs/03` §7 jitter target (σ ≤ 120 µs, max ≤ 500 µs) is already met at constant tempo, about 20 times under.** The 1 sample is just rounding a fractional sample down |
| B | 60 to 180 BPM ramp over 8 bars (16 s), host reports the tempo at the start of each buffer | The last note is **24.05 ms late at 32-sample buffers, 25.6 ms at 256, 30.9 ms at 1,024, 38.1 ms at 2,048** (1,154 to 1,827 samples) | **This is the real defect.** 24 ms is about 50 times the 500 µs maximum in `docs/03` §7 |
| C | 60 to 120 BPM ramp over 4 s, then constant 120 BPM until 16 s | Lateness climbs to 23.92 ms by the end of the ramp and is **still exactly 23.92 ms 12 seconds later** | The offset never recovers. After any tempo ramp the sequencer sits permanently behind the host's beat grid until it is stopped and started |
| S | The cause, tested: the same ramp with the host reporting the tempo `shift` ms *later* than the buffer start | Lateness at shift 0, 10, 20, 25, 30, 40 ms: 25.6, 18.9, 12.3, 8.9, 5.6, -1.1 ms | Lateness falls in a straight line as the reported tempo is moved earlier, and reaches zero near 38 ms, which is inside the engine's 21 to 63 ms lookahead window. So the lateness is the staleness of the tempo, as the code below says |
| F | The per-track `f32` step accumulator against the exact tick, for every reduced multiplier num/den up to 16 (159 of them) | 30 minutes at 120 BPM: 49 multipliers fire some step 1 tick late or early. 7 hours: 80, still at most 1 tick. 72 hours: 87, worst error **9 ticks (23 ms)** | Small in a session, but it grows with time (1 tick at 7 hours, 9 at 72): the `f32` step length is a slightly wrong number, so the error builds up |
| R | Who can set a track's speed multiplier today | Nothing but a test writing the field. It is not in `TrackAttr`, the FFI, or `octorun` | The accumulator defect is **latent**: no host can hear it yet. It gets audible when the panel or phrases can change a track's speed |
| Q | How many ticks the running sum `next_tick_due += spt` puts on a different whole sample than `floor(k * spt + 1e-9)` (`golden_floor_flips.py`, output `o4-golden-floor.txt`), over the first 7 s of each golden pattern | 96, 100, 120 and 128 BPM: **0**. 110 BPM (`phrases`): **57 of 2,464 ticks**, the sum landing 1 sample early. 140 BPM (the second half of `mcc_and_transport`): 153 of 3,136 | A whole-sample tick time is sometimes rounded down to one sample before it. It happens only where samples per tick is not a binary fraction (136.3636… at 110 BPM, 107.1428… at 140). It changes what G2 can promise (section 4) |

**Why B and C happen, from `engine.rs` `render_core`.** The engine fixes each tick's length
when it *steps* the tick: `next_tick_due += spt`, where `spt` (samples per tick) comes from
the tempo in the `render` call that stepped it. Ticks are stepped `MAX_EARLY_TICKS` (12) plus
part of the host buffer ahead of the moment they sound (that lookahead is O3's fix and
stays). During a ramp every tick is therefore measured with the tempo as it was up to 12
ticks earlier (21 to 63 ms across this ramp), too slow, and the total of those small errors
is the offset. Nothing ever corrects it, because the engine keeps no idea of where the host
thinks the beat is.

**What this changes about the case for O4.** `findings.md` presents O4 as jitter, PLL and
drift. The measurements say: jitter at constant tempo is fine, a PLL would not help
(section 2.6), and the defect is a permanent offset after a tempo ramp (a tempo jump has the
same cause; R5 will measure it) plus a locate/loop that does nothing. That is still worth a
high-tier change, but it is a smaller and better-defined change than the spec suggests.

### 1.1 Evidence for D0

| Source | Says |
|---|---|
| `reference/manual/pages/p015.txt`, line 47 | "Each Green increment corresponds to 1/192 of a note and each Red value corresponds to 12/192 = 1/16 of a note." |
| `p016.txt`, line 4 | "The natural maximum length of a step is one full note - 192/192." |
| `p067.txt`, line 5 | "A measure is 16 steps at x1 speed" |
| `crates/octocore/src/domain.rs:33-34` | `TICKS_PER_QUARTER: u32 = 192`, and `DEFAULT_STEP_TICKS: u32 = 12; // 1/16 note at 192 PPQN` |
| `crates/octocore/src/engine.rs:272` | `sample_rate * 60 / (bpm * TICKS_PER_QUARTER)`: 125 samples per tick at 120 BPM and 48 kHz (asserted in two engine tests' comments) |
| `docs/03-sequencer-core.md:43`, `docs/02-architecture.md:221`, `AGENTS.md:20`, `SPEC.md:271`, `specs/SPEC-0001/tech.md:9` | All say "192 PPQN". Only `docs/03:43` also says 12 ticks is a 1/16 |

The arithmetic: 12 ticks is 1/16 of 192, so 192 ticks is the note the manual calls "a note",
a whole note. At 120 BPM a quarter note is 0.5 s, so a 1/16 note is 125 ms; the engine's 12
ticks last 12 × 125 samples = 31.25 ms. What I cannot settle from the manual is the unit BPM
counts (quarter notes is the usual and only sensible one, and is why I call the engine wrong,
but it is a reading). That is why D0 is your decision and the engine is untouched.

## 2. The change

### 2.1 Two layers, as `docs/02` describes them

`docs/02` §3 (line 129) gives the core `tick(ppqn_pos: u64)`, told a position and never
asking the time, and §5 (line 220) says position is `u64` ticks and float appears "only when
converting to audio-sample offsets, once, at the boundary". The engine today does neither: it
steps ticks off a running `f64` sum of samples. O4 moves it toward the document:

1. **The tick layer** (sequencing). Steps integer ticks. Every event it schedules carries an
   integer tick position: the tick it fired on plus integer offsets (start offset, strum,
   length, all of which are already whole ticks in the code). No `f64` anywhere in it. Tick
   keys are signed `i64`, so a host position before bar 1 (a count-in) is legal.
2. **The clock layer** (new file `clock.rs`, still pure, no I/O, no time source). Knows the
   sample rate, the tempo and the position at the start of the buffer. Answers two questions:
   *which ticks fall in this buffer*, and *at which sample offset does each one land*. This is
   the only place a tick becomes a sample.

The event queue is ordered by `(tick, rank, sequence)` instead of `(whole sample, rank,
sequence)`. The two orders are the same whenever a tick is at least one sample long. The
engine's limits guarantee that (8 kHz and 999 BPM give 2.5 samples per tick, at the current
tick length; a longer tick, D0, only makes it safer), and a test asserts it.
`step_once_for_test` and every conformance fixture work on the tick layer and do not change.

### 2.2 Converting a tick to a sample

At the start of each buffer the clock layer holds an anchor: the position `p0` (whole ticks
plus a fraction, the number of the next tick to be stepped) and the tempo `b0`. A tick `T`
inside the buffer lands at

`sample = floor((T - p0) * spt(b0) + 1e-9)`, measured from the buffer start.

That is one multiplication from an integer and an anchor, not a running sum, so at a constant
tempo nothing accumulates. **The `1e-9` is deliberate** and section 1 row Q is why: a tick
whose exact time is a whole number of samples must land on that sample, and today, at 110 BPM,
57 of 2,464 ticks land one sample early. `(T - p0)` is small (a buffer holds far fewer than
2^20 ticks), so the `f64` product is accurate to about 1e-12 samples, far inside the `1e-9`
band. Test R7 compares the engine with exact rational arithmetic (the `f32` bpm and rate are
exact binary fractions, so `i128` arithmetic can do it) on a million random cases, and
asserts that any disagreement lies inside that band and errs upward by less than `1e-9`.

Notes are stepped 12 ticks ahead as now, but they wait in the queue as tick positions and
are converted only when the buffer that holds them is rendered, with the tempo of that
buffer. That removes the cause in section 1.

For tempo that is changing, the buffer is modelled as a straight-line tempo change: `b0` at
the start and a slope `a` (BPM per sample) taken from the previous buffer's change. **The
slope is used only when the last two changes agree within 5%.** Otherwise the buffer is
modelled at constant tempo. This is the guard that stops a sudden tempo jump (say 120 to 140
in one buffer) from being turned into a huge fake slope for the next buffer. The tick `d`
ticks after the anchor lands at `t = (sqrt(b0² + 2·a·k·d) − b0) / a` samples, with
`k = 60·sample_rate / ticks_per_quarter`, and at `t = d·k / b0` when `a` is 0. **Two limits
make it safe**: if the modelled tempo would leave `MIN_BPM..=MAX_BPM` anywhere in the buffer,
or the square root's argument is negative (the tempo would reach zero inside the buffer), the
buffer is modelled at constant tempo. Both are tests (R5, R6), not assumptions.

`RenderContext::bpm` is an `f32`. The references in every test are computed from the `f32`
the engine sees (97.3 arrives as 97.30000305…), not from the decimal the test author typed.

### 2.3 Host position and free-running

| Mode | When | What anchors each buffer |
|---|---|---|
| Host position | The host gives the position in ticks (fractional) at the start of every buffer | The host's position. No error can build up: each buffer restarts from the host's answer |
| Free-running | Only `bpm` and `playing`, as every caller passes today | The engine's own position, advanced by the same model. Error can build up, but far less than today (section 3) |

Both modes use the same clock layer. `RenderContext` is not changed: it is built by struct
literal in 17 places across five files, so a new field would touch all of them. A new method
`Engine::render_at(ctx, position, out)` takes the position, and `render(ctx, out)` calls it
with none.

**Two tick frames, kept apart.** `global_tick` is the sequencer's own count of ticks stepped
since `Reset` (it survives Stop and Play, and it drives measure boundaries); it does not
change. The queue keys and the anchor are in the *timeline* frame. In free-running mode the
timeline tick equals `global_tick`. In host position mode it is `global_tick + origin`, where
`origin` is set when the transport starts and is recomputed on a discontinuity (section 2.4),
so a locate moves the timeline and leaves the step positions alone (D5).

**Free-running position across Play, Stop and Reset.** Position advances only while running.
Stop keeps it, as it keeps `global_tick`. Play anchors on sample 0 of the buffer in which the
flag flipped (O2's rule today: `next_tick_due = sample_clock`), with the position where Stop
left it. Reset sets it to 0.

The free-running position is held as whole ticks (`u64`) plus a fraction below one tick
(`f64`). **That fraction is a float accumulation**, updated once per buffer, and I do not
claim otherwise. Its error is at most about 1.1e-16 tick per buffer, so 10^8 buffers is at
most about 1e-8 tick, and S1 checks it. `docs/02` §5's rule ("no floating-point accumulation
for musical position") holds for the musical position, which is whole ticks; in host
position mode nothing accumulates at all. The absolute sample clock is no longer used for
scheduling. It stays as a `u64` counter for the report and the tests. That is hygiene and not
a fix: `f64` is exact to 2^53 samples, about 5,900 years at 48 kHz.

### 2.4 Locate, loop and discontinuity (host position mode)

- The engine expects each buffer's position to equal where the last buffer's model ended,
  within a **tolerance of the largest of**: 1 tick, the model's own error bound for that
  buffer, and 0.5 ms expressed in ticks at that buffer's tempo (at 999 BPM a tick is only
  2.5 samples, 52 µs, smaller than the rounding a host may do). A host that reports a
  slightly different position each buffer is followed silently (R10).
- A difference beyond the tolerance is a **discontinuity**: a locate, a loop wrap, or a
  tempo-map change. The engine releases every sounding note at sample 0 of that buffer (a
  `NoteOff` for each, no CC 123, so a loop does not spray controller messages), drops the
  events still queued for the old position, and re-anchors on the new one. Step positions
  are left alone (D5). The next tick lands within 1 sample of the analytic position.
- A locate while stopped moves the anchor. The next Play starts from there (R11).
- **A locate is given only by `render_at`'s position.** `Command::HostTransport { ppqn_pos }`
  stays as it is today: its position is never read, and callers may already be sending any
  value in it. Reading it now would turn those into locates. A host that drives the engine
  through the ring only, and wants to locate, needs a new command, which is an ABI question
  and not part of O4.
- A host that wraps a loop *inside* a buffer without splitting the buffer is seen one buffer
  late. I cannot fix that from the engine. It is in the risks.
- New counters (discontinuities, positions corrected) go in a **new** struct with its own
  getter. `OctoDiagnostics` gets no new field, because a new field in an existing `repr(C)`
  struct is an ABI break and that is high tier by itself.

### 2.5 Transport ownership (D4)

The engine remembers the host flag it saw last render. When the flag differs, the engine
acts on it. A `Play`, `Stop` or `Continue` command acts immediately. So the last change wins,
from either side, and a `Stop` from the ring is no longer undone in the render call that
applied it. A caller that only changes `playing` (every fixture, `octorun`) behaves exactly as
today, and so does one that always passes `true` and never sends `Stop` (R9). The change is
visible only to callers that use both, and today those callers are buggy.

Two consequences, both tested in R9. A host flag that never changes (constant `false` with a
`Play` command, or constant `true` with a `Stop` command) leaves the commands in charge until
the flag changes. And after a command overrides the flag, the engine keeps to it until the
host's flag *changes*, so a host that holds "playing" while the user pressed Stop gets a
stopped engine until it stops and starts.

### 2.6 Why no PLL in `octocore` (D2)

A plug-in host hands the engine a buffer of N samples at a time, and every sample is
accounted for: sample 0 of the next buffer is N samples after sample 0 of this one, exactly.
There is no second clock in the plug-in to lock to, so a PLL has nothing to correct. With
the host's position (2.3) the engine also knows where the beat is. Variable buffer sizes are
covered already: the O3 tests show the same event stream for the 11 buffer sizes they use
(1 to 4,096; the comparison against the 512 reference uses 8 of them) on 24 seeds, and for
seeded random size sequences. The sizes stop at 4,096 on purpose (see the test's comment).

**What host position mode does instead, and what it is not.** A host that resamples or
varispeeds its transport, which `docs/10` R4 names, makes the position it reports drift from
the sample count. Host position mode restarts every buffer from the reported position: a
difference inside the tolerance is followed, a difference beyond it is a discontinuity. That
is **smoothing, not a PLL**. It filters nothing and predicts nothing, so a host whose position
wobbles inside the tolerance gives a sequencer that wobbles one for one. That is right for a
first version, and R10 and R13 test it. If a loopback test shows σ above 200 µs (the `docs/10`
R4 trigger), a PLL is the response, and it goes in the host adapter or in `octorun --live`,
where the only clock is wall time and callbacks arrive irregularly, and hands `octocore` the
exact buffers this plan is written for.

### 2.7 Integer step accumulators (PR 4a)

Today: `accum += 1.0; if accum + 1e-6 < step_ticks { return } accum -= step_ticks`, with
`step_ticks = 12 / (num/den)` in `f32`. The `1e-6` is a patch for the `f32` error, and (F)
shows it does not hold.

New: the step is exactly `12*den/num` ticks, so keep the phase as a whole number of
`1/num`-tick units. Each tick adds `num`, and the step fires when the phase reaches
`12*den`, then subtracts `12*den`. A step shorter than one tick fires every tick (as the
`max(1.0)` does today). A hyperstep-linked track keeps its 192-tick step. When a track's
multiplier changes, the phase in ticks is carried over and rounded down to a whole unit of
the new step, which loses less than one tick, and R1 states it. Integer arithmetic on `u8`
numerator and denominator, so it is exact for every value the fields can hold, not only up
to 16. This is in ticks, so it does not depend on D0.

### 2.8 Gaps the review found, and the rule for each

1. **Signed keys.** Tick keys and anchors are `i64` (section 2.1). `global_tick` stays `u64`.
2. **`late_events` keeps its meaning.** An event that goes out at sample 0 of a buffer because
   its time was before the buffer started, counted once per event. With tick keys that is an
   event whose tick is below the buffer's first tick. Events deferred by a full output buffer
   are counted the same way, as today (the engine test at line ~2552 expects it). Events
   dropped by a discontinuity flush are not late. G1 and the null host assert 0 on every
   run.
3. **Two tick frames** (section 2.3): the timeline tick and `global_tick` are different
   numbers, and only the timeline is in the queue keys.
4. **Free-running position on Play, Stop and Reset** (section 2.3).
5. **An unusable tempo keeps the last usable one for queued notes.** Today a render with tempo
   0 or NaN steps no tick, but the notes already queued still run to their `NoteOff`s, because
   their times are in samples. With tick keys, time must keep moving or those `NoteOff`s never
   come and the notes stick. So while the host reports an unusable tempo, the clock layer
   keeps advancing the position at the last usable tempo (slope 0) and steps no new tick.
   With no usable tempo yet there is nothing queued. G9 pins it and passes on the parent.
6. **Slope limits** (section 2.2): clamp to the tempo limits, and fall back to constant tempo
   on a negative square-root argument.
7. **A frozen host.** A position that does not advance while `playing` is true is one
   discontinuity (flush, re-anchor) and then silence, not a flush every buffer and not the
   same tick fired again and again. When the position moves again it is a discontinuity
   again. This is my choice and D5 covers it; the alternative is to treat a frozen position
   as free-running. R12 pins it. A host *flag* that never changes is section 2.5.
8. **A locate is `render_at` only** (section 2.4). `HostTransport.ppqn_pos` stays ignored.
9. **One white-box test reads private fields.** `play_resyncs_the_tick_clock_to_the_current_sample`
   (`engine.rs`, line ~2460) asserts on `sample_clock` and `next_tick_due`, which this change
   replaces. It is **ported to the new state, not deleted**: renaming or removing a test
   needs `cargo xtask baseline --remove … --adr`, and the intent (Play does not replay idle
   ticks) is worth keeping. The port keeps its name.
10. **Amend the `AMBIGUITIES.md` sentence "a constant tempo … is exact"** (entry "lookahead,
    Stop and commands"). It is exact to one sample; after a ramp or a jump the offset stays
    (rows B and C). This plan's pull request amends it, since the file is in Metronome's zone.

## 3. Acceptance: F2 and A4, criterion by criterion

"On the null host" means the synthetic host in section 4. "Before rounding" means the time
the clock layer computes, before it is floored to a whole sample, because the floor alone
costs up to 1 sample at any tempo and is not a timing error. The "after" figures come from
the model in `o4-model.txt`, a program that does the proposed arithmetic and nothing else.
They are predictions until the tests in section 4 confirm them; if the built code disagrees,
the tests win and this plan is revised. **They depend on the tempo curve and the buffer, not
on the tick length, so D0 does not change them.** The "today" figures do change with D0.

| Criterion | Where it comes from | Today (measured) | After O4 | PR |
|---|---|---|---|---|
| Constant-tempo jitter σ ≤ 120 µs, max ≤ 500 µs, buffers 1 to 4,096 and random sequences | `docs/03` §7, F2 | **Already met**: max 21.8 µs, σ 6.5 µs (A) | Still met (a **guard**, G1), and G1 also holds it to 1.05 samples and σ 0.3 samples, which today's engine meets (1.02, 0.29): a tightening, so the 500 µs target, which is 24 samples and could hide a regression, is not the only fence. Not "fixed" by O4 | all |
| Ramp, host position mode, reference ramp (60 to 180 BPM over 8 bars): notes within 1 sample of analytic | A4, F2 | 1,160 to 1,830 samples out (B) | Steady ramp: **0.000 samples at every buffer size** (model). In the two buffers where the ramp starts and its slope is not yet known: 0.33 samples at 512, 1.3 at 1,024, 5.2 at 2,048, 21.3 (0.44 ms) at 4,096. See D3 | 4c |
| Ramp: no lasting offset once the tempo is constant | A4 ("ramp drift") | 23.92 ms, permanent (C) | Host position mode: 0 ticks, by construction, because every buffer restarts from the host. Free-running: 0.23 samples at 512-sample buffers, 0.9 at 1,024, 3.6 at 2,048, 14.6 at 4,096 (model, the last note of the reference ramp) | 4b, 4c |
| Ramp, free-running (no host position), reference ramp: the case every caller is in today | not in the spec | 1,160 to 1,830 samples (B) | Worst over the ramp: 0.68 samples at 512, 2.7 at 1,024, 10.8 at 2,048, 42.8 at 4,096 (model). Within 1 sample only up to 512-sample buffers | 4b |
| A steeper ramp (30 to 300 BPM in 2 bars) | not in the spec | not measured | **Only the computed bound** (D3): I do not assert 1 sample at any buffer size for a steeper ramp, because the error in the first two buffers grows with the ramp rate | 4b, 4c |
| Locate to any tick within 1 sample; loop wrap | A4 | Ignored: `ppqn_pos` is never read | Next tick within 1 sample of analytic. Sounding notes released at the jump; NoteOn/NoteOff balance holds across it | 4c |
| Step timing exact for every multiplier | O4 finding ("integer per-track step accumulators") | 49 of 159 within 30 min at least 1 tick out; 9 ticks in 72 h (F) | Exactly `ceil(k * step)` for every k, every `u8` num/den | 4a |
| Whole-sample tick times land on that sample | new (Q) | 57 of 2,464 ticks 1 sample early at 110 BPM | 0, exact rational arithmetic (R7) | 4b |
| A thousand simulated hours without missed ticks | `docs/03` §7 "missed ticks in 30 min: 0" | True today: the tick count is an integer | Still true (a **guard**, G7). Not "fixed" by O4 | all |
| Nothing else moves | invariant | n/a | G2 to G9 below | all |

**D3, in plain words.** The spec asked for "ramp drift at most 1 sample" with no buffer
size. The model says: once the engine has seen two buffers of a ramp it knows the slope and
a straight ramp is exact. In the first two buffers it does not know a ramp has started, and
the error there grows with the square of the buffer size: under 1 sample up to 512, 1.3 at
1,024, 21 (0.44 ms, inside the 500 µs max-jitter target of `docs/03` §7) at 4,096. Those are
the numbers for the reference ramp. **A steeper ramp is worse, so R2 and R3 check each note
against a bound computed from the ramp rate and the buffer size** (half the rate of tempo
change times the buffer time squared, in beats, plus 1 sample for rounding), and only for
the reference ramp do they also assert the 512-sample row above. If you want 1 sample held at
larger buffers, the answer is fixed latency (D3 option a), not more arithmetic.

## 4. Verification plan

Written before any code, as for O6. Each test has an ID that appears in its name, so the
trail from this plan to `harness/baseline.txt` can be searched. Tests marked *red* are shown
failing on the parent commit and passing after, with the output in
`handoffs/evidence/o4-*-red.txt`. **A red test that needs `render_at` is shown failing on a
stub `render_at` that ignores the position and calls `render`**, so that it fails for the
behaviour it tests and not because the method does not exist. Tests marked *guard* pass on
the parent commit and must still pass. A guard is proved to protect something by a mutant: I
break the new code on purpose and show the guard fail (the way `o6-mutants.txt` does for the
loom tests).

**The null host** (a test helper, no engine change): produces buffers of a given size or a
seeded random size sequence, with the position and tempo taken from an analytic tempo curve
(constant, linear ramp, jump, ramp then hold, loop, varispeed, frozen). It records what the
engine emits and compares each note with the analytic time, using the `f32` tempo the
engine sees. The `(buffer, position, tempo, playing)` list it feeds is also written as
NDJSON, so any failure is a replayable "host clock trace" (`product.md` invariants).

| ID | Kind | Asserts | PR |
|---|---|---|---|
| G1 | guard | Constant tempo, 4 tempo and rate pairs, buffers 1 to 4,096 and 20 random sequences: σ ≤ 120 µs and max ≤ 500 µs (the `docs/03` §7 target), **and** max ≤ 1.05 samples and σ ≤ 0.3 samples (today: 1.02 and 0.29). `late_events` is 0. This is the first content for the `verify:timing` gate | first commit |
| G2 | guard | The five `octorun` golden hashes. **Three are byte-identical in every PR**: `hello` (120 BPM), `chords_and_strums` (96) and `effector` (128), whose samples per tick are binary fractions so no whole-sample tick can be rounded down (Q). **`phrases` (110 BPM) is expected to change in 4b**, only where a tick lands on a whole sample and today's sum puts it one sample early (57 of 2,464 ticks in the first 7 s): each affected event moves **1 sample later, never earlier**. **`mcc_and_transport` (100 to 140 BPM at 2 s, with two stop and play cycles) is expected to change in 4b for every event from the tempo change to the end of the run**: events stepped just before the change move earlier by up to about 500 samples (10 ms; the arithmetic is 12 ticks × (150 − 107.1) samples, and the PR will show the real figure) because they are now timed with the new tempo, and later events by 1 sample where 140 BPM lands on a whole sample. All five are byte-identical in 4a and 4c. The 4b PR shows the diff event by event, explains each class of change, and regenerates the goldens (`just golden`) with the reason in the commit | first commit, then all |
| G3 | guard | The O3 buffer-size invariance tests (11 sizes, 1 to 4,096, 24 seeds, and random sequences) | first commit, then all |
| G4 | guard | Zero allocation in `render` with and without a link (the counting-allocator test from O6) | first commit, then all |
| G5 | guard | Conformance fixtures 100% (`verify:conformance`) | first commit, then all |
| G6 | guard | Worst full-density tick time, measured before and after (`docs/03` §7 says at most 250 µs). Reported, not asserted on a shared runner, as in `o6-cost.txt` | 4b |
| G7 | guard | Tick N is exactly N after 1,000 simulated hours, on the tick layer alone (integers). True on the parent, so it is a guard: it protects the property that O4 must not break | 4b |
| G8 | guard | An edit made by a command while playing takes effect no later than today (no new latency). True on the parent by construction | 4b |
| G9 | guard | With the host reporting tempo 0 or NaN for a few buffers, notes already queued still get their `NoteOff` at the expected time and nothing sticks (section 2.8, item 5); the white-box test `play_resyncs_the_tick_clock_to_the_current_sample` is ported and keeps its name | 4b |
| R1 | red | For every `u8` num ≥ 1 and den ≥ 1 (65,025 pairs), the k-th step fires on tick `ceil(k*12*den/num)`, to 10^12 ticks by closed form. Separately, a simulation of 10^7 ticks for the 159 reduced pairs up to 16 (the set measured in F). Changing the multiplier mid-run loses less than one unit of phase | 4a |
| R2 | red | Free-running ramp 60 to 180 BPM over 8 bars, and a steeper one (30 to 300 BPM in 2 bars): every note within the computed bound of D3, for buffers 32 to 4,096. On the reference ramp only, also within 1 sample at buffers up to 512 | 4b |
| R3 | red on a stub | Same ramps, host position mode: within the bound, and 0 outside the first two buffers | 4c |
| R4 | red | 60 to 120 ramp then hold: offset vs the host grid is 0 ticks (host) or at most 1 sample up to 512-sample buffers (free-running) at 12 s after the ramp | 4b, 4c |
| R5 | red | Tempo **jump** 120 to 140 in one buffer: the buffers after it are exact (the slope guard), and the jump does not produce more error than a constant-tempo run | 4b |
| R6 | red | Slope limits: a ramp that would take the tempo below `MIN_BPM` or above `MAX_BPM` inside a buffer, and one whose square root would be negative, give no panic, no NaN, and constant-tempo timing for that buffer | 4b |
| R7 | red | Exact sample arithmetic: on the null host at 110 and 140 BPM every note is at the exact-rational floor (today 57 of 2,464 ticks are not); and one million random `(bpm, rate, tick)` cases agree with `i128` rational arithmetic except inside the `1e-9` band, where the engine errs upward | 4b |
| R8 | red on a stub | Locate forward, backward, and 100 loop wraps: next tick within 1 sample; every sounding note gets one NoteOff; NoteOn minus NoteOff is 0 per (port, channel, note). `HostTransport` with any `ppqn_pos` moves nothing | 4c |
| R9 | red | Transport: `Stop` from the ring stays stopped while the host flag stays "playing"; a host flag change acts; a constant `false` flag with a `Play` command runs; a caller that only passes `playing` is unchanged | 4c |
| R10 | red on a stub | A host whose reported position is off by up to the tolerance from buffer to buffer is followed with no flush and no counter change | 4c |
| R11 | red on a stub | Locate while stopped, then Play: the first tick is at the located position | 4c |
| R12 | red on a stub | A position that stops advancing while `playing` is true flushes once, then plays nothing and does not spray; when it moves again the engine re-anchors (section 2.8, item 7) | 4c |
| R13 | red on a stub | Varispeed: a host whose position runs 0.5% faster than its sample count is followed with no flush, each note within the model's bound of the host grid | 4c |
| S1 | soak, reported | 10^8 buffers of free-running constant tempo end within 1e-6 tick (about 0.0001 sample) of the analytic position. Run by hand and not in the gate (it takes minutes); the number goes in the PR | 4b |

**Mutants for the new code** (each must make at least one test fail): the anchor uses the
previous buffer's tempo; the slope guard is removed; the tempo-limit clamp is removed; the
sample offset is rounded instead of floored; the `1e-9` is dropped (R7); `T - p0` off by one
tick; the discontinuity threshold is 0 or 10 ticks; the transport flag is level-triggered
again; `HostTransport` starts honouring its position (R8); the accumulator adds `num` but
subtracts `12*den+1`.

**Independent review.** Before each pull request is marked ready, a fresh session fills in the
`AGENTS.md` checklist. The clock layer gets a second reading of the arithmetic in section 2.2
by a session that has not seen this plan's numbers.

## 5. Pull requests, files and order

Stacked on `metronome/command-ring` (PR #11), because O6 changed `engine.rs`. Each one is
independently revertable. If you decide D0 by fixing the tick, that task goes between 4a and
4b.

| PR | Task | Contains | Tier | Emitted MIDI changes? |
|---|---|---|---|---|
| 4a | WENGE-0004 | Integer step accumulators (`steps.rs`, `TrackRuntime` field, `step_one_track`). G1 to G5 first, then R1. Smallest and safest, so it goes first | Medium: the only behaviour change is for speed multipliers other than ×1, which no host can set (R in section 1), and nothing else in the clock changes | Yes, for speed multipliers other than ×1 that the `f32` version gets wrong (1 tick, 2.6 ms at the current tick length). Not reachable from a host today |
| 4b | WENGE-0004 | `clock.rs`, tick-keyed queue, drain converts tick to sample, free-running mode with the slope model and its limits, exact sample arithmetic. G6 to G9, R2, R4, R5, R6, R7, S1 | **High** | Yes: notes during a tempo change sit on the grid, and whole-sample ticks land on their sample. Three goldens are byte-identical (G2); `phrases` and `mcc_and_transport` are regenerated, each change explained |
| 4c | WENGE-0004 | `Engine::render_at`, discontinuity handling, transport edge rule, additive FFI: `octocore_engine_render_at`, `OctoHostPosition`, a new counters struct. `OCTOFFI_ABI_VERSION` 2 to 3. R3, R8 to R13 | **High** | Yes: a locate releases notes. Callers that never give a position see no change |

Files: `crates/octocore/src/engine.rs`, new `clock.rs` and `steps.rs`, `types.rs`
(no layout change), `crates/octocore/tests/` (null host, R and G tests), `crates/octoffi/`
(additive), `harness/baseline.txt`, `tests/conformance/AMBIGUITIES.md`. Files owned by others
that will need requests: `docs/02` §5 (lines 220 and 221) and `docs/03` §5, §7 and line 43
(Scribe); the `verify:timing` gate (Referee); the D3 ADR (Conductor).

Nothing in the current `Command` or `Event` layout changes. If a `repr(C)` layout change turns
out to be needed, that is a stop and a new question to you, not a decision I make.

## 6. Risks

| Risk | Consequence | Mitigation |
|---|---|---|
| The clock layer has a bug that only shows on a real host | A late or doubled note on a real machine, which `N5` calls a catastrophe | No host is available here (no Xcode, no Live). The null host proves the arithmetic, not the integration. **Before this ships, a loopback test in Live 12 on your machine** (`docs/03` §7's "lab" number) is a release condition. I say so plainly: this plan cannot promise it |
| D0 is decided after 4b, and the tick length changes | Every golden and every millisecond figure changes a second time, and the lookahead (12 ticks) would last four times longer in time | Decide D0 before 4b (section 0). 4a is unaffected |
| Existing golden or conformance output changes | The ratchet blocks, or worse, is "fixed" | G2 and G5 are byte-identical except for the changes G2 names, and a change it does not name stops the work (section 7) |
| The slope guess is wrong for non-smooth automation | An error for one or two buffers, and in free-running mode that error stays as an offset | R5 tests the jump and R6 the limits. The guard falls back to constant tempo, and costs one extra buffer before the slope is used (which is why free-running is 0.68 and not 0.34 samples at 512). Host position mode forgets the error at the next buffer |
| A host loop wrap inside a buffer | The wrap is seen one buffer late | Documented limit. Most hosts split buffers at a loop end. Verified only in the loopback test above |
| A host reports positions that wobble by more than the tolerance | A false discontinuity and a flush of sounding notes | The tolerance (largest of 1 tick, the model bound, 0.5 ms) and R10. If a real host needs more, the threshold is a named constant that can go up (loosening it is the owner's call, with an ADR) |
| Edit latency (up to about 31 ms at 120 BPM at the current tick length, from the 12-tick lookahead) | Not changed by this plan | G8 asserts it does not get worse. `docs/02` §6 budgets 1.3 ms; that mismatch is already with the Scribe |
| Three pull requests take longer than one | Slower to review | Each is small and each fixes something on its own. 4a and 4b are useful even if 4c is never approved |
| The slope model is over-engineered | More arithmetic than needed | It is about 10 lines. Without it a steady ramp is off by up to 0.33 samples at 512 and 21 at 4,096 in host mode, and free-running mode drifts by 170 samples (3.6 ms) at 512 by the end of the ramp (`o4-model.txt`) |
| The "after" numbers are a model, not the engine | The built code could miss them | Section 3 says so, and the tests are the arbiter |

## 7. Rollback, kill criteria, migration

- **Rollback:** each pull request is one `git revert`. Nothing persists (no saved-state format
  exists), no host consumes the engine yet, and the FFI change in 4c is a new function, so
  reverting it removes a function nobody calls. The parent tip is the rollback point.
- **Stop and report** (not "try harder") if: G2 or G5 differ in a way G2 does not describe; a
  golden event moves *earlier* at constant tempo; a red test is not green after 3 attempts
  (`AGENTS.md` limit); R2 cannot meet its row at 512-sample buffers on the reference ramp; the
  full-density tick goes over 250 µs; or a `repr(C)` change looks necessary.
- **Migration:** none. `RenderContext`, `Command`, `Event`, `Snapshot` and `OctoDiagnostics` keep their
  layouts. The C header gains one function, one struct and the version number.
- **Health signals for later:** `late_events` stays 0 on every null-host run; the new
  discontinuity counter is 0 on a straight play-through and equals the number of loop wraps on a loop.

## 8. The re-read of `product.md` and `tech.md`

What I found reading them again for this plan. None of it is applied until you approve; the
right-hand column is the revision r3 text I would propose.

| Where | What it says | Problem | Proposed r3 text |
|---|---|---|---|
| `product.md` A4 | "Ramp drift is at most 1 sample" | No buffer size, and "drift" is not defined. Making it achievable is a loosening (D3, ADR) | "Before rounding: exact on a steady ramp; in the two buffers where a ramp starts, within 1 sample up to 512-sample buffers and within 0.5 ms up to 4,096 (reference ramp; a steeper ramp within the computed bound). No lasting offset after the ramp in host position mode" |
| `tech.md` F2 row | "Analytic jitter σ ≤ 120 µs, max ≤ 500 µs at every buffer size" | Already true today; reads as a task | Keep as a guard (G1), and say it holds at baseline |
| `tech.md` F2 kill | "If the PLL cannot meet σ under variable buffers, ship fixed-latency mode" | Presumes a PLL (D2) | "If the tick timeline cannot meet the ramp table at 512-sample buffers, stop and report" |
| `tech.md` diagram | "Tempo map and PLL" | Same | "Tempo model" |
| `tech.md` Clock line | "Sample time is derived per tempo segment, never accumulated" | Matches 2.2 for constant tempo. During a ramp each buffer is its own segment, so the origin is rebased each buffer | Add: "During a ramp, per buffer, from the host position when there is one" |
| `tech.md` line 9 and the "192 PPQN" statements (`docs/02` §5, `docs/03` line 18, 43 and 220, `AGENTS.md`, `SPEC.md`) | 192 ticks per quarter note | The manual gives 192 per whole note (D0) | No change until you decide D0. Then either the manual's reading or a recorded reason for the engine's |
| `tech.md` Known risks | "Lookahead delays command effects by up to 31 ms" | Right at the current tick length, and unchanged by O4. `docs/02` §6 says 1.3 ms | No change here. Already with the Scribe. D0 would make it about 125 ms unless the lookahead constant is revisited |
| `findings.md` O4 evidence | "each track keeps an f32 step accumulator" | True, but no host can set a multiplier yet, so it is latent | Add the (R) row from section 1 |
| `findings.md` O4 assertion | "A locate to any PPQN puts the next tick within 1 sample" | The manual is silent on locate (pp.93 and 94) | Add the `AMBIGUITIES.md` entry and D5 |
| `product.md` invariants | "deterministic given seed, command log and host clock trace" | The trace format is not defined | The null host writes it as NDJSON (section 4) |
| `tech.md` "Test plan" | "30 minutes at 120 BPM is 691,200 ticks" | Right at 192 per quarter (384 ticks per second); it would be a quarter of that under D0 | Used for S1 and G7. Restated when D0 is decided |

## 9. What this plan does not cover

The design system and DSP (Q1). Live MIDI in `octorun`. Legato, the panel, or any
renderer work. A real host. Chase on locate (D5). Fixing the tick length (D0, `WENGE-0012`).
A locate command for hosts that use only the ring (section 2.4). Anything that needs a
Live 12 machine.

## 10. After you approve: the next three steps

1. Write the guard tests G1 to G5 on the parent commit and show them passing (they must pass
   before any O4 code, or they prove nothing), plus the null host. Commit that alone.
2. PR 4a, red-first (R1), then 4b, then 4c, each pushed and opened as its own pull request
   with the gate numbers in the commit message. Ask the Conductor for the D3 ADR when 4b
   opens.
3. File the Scribe and Referee requests when each PR opens, not at the end.

If you reject it or ask for changes, the next step is a revision of this file, and nothing else.

## 11. What changed since revision 1

Revision 1 was read by a session that had not written it. Its findings, and where each went:

| Finding | Where |
|---|---|
| Not in revision 1: the tick is 4× short (found while checking an unrelated claim) | D0, section 1.1, `WENGE-0012` |
| "Constant tempo is byte-identical" was false for `phrases`; `mcc_and_transport` changes for every event after the tempo change, not only those stepped before it | Section 1 row Q, G2 |
| Sample arithmetic was not exact | Section 2.2, R7 |
| D3 was presented as an option, and is a loosening; fixed latency was not offered; "cannot be met" was overclaimed | D3, section 3 |
| Steeper ramps were held to the reference ramp's table | Section 3, R2 |
| Tick keys unsigned; `late_events` undefined; two tick frames; free-running position across Play, Stop and Reset; unusable tempo would stick queued notes; slope not limited; frozen host; locate through `HostTransport`; a white-box test | Section 2.8 |
| Discontinuity tolerance was 1 tick, too small at high tempo | Section 2.4 |
| D4 said "one render later"; it is the same render call; the "constant tempo is exact" entry | D4, section 2.8 item 10 |
| Sample clock and "no float accumulation" wording | Section 2.3 |
| O3 test coverage overstated ("every size from 1 to 4,096"); `docs/02` attribution | Section 2.1, 2.6 |
| The plan admitted no smoothing, and did not cover a resampled transport | Section 2.6, R13 |
| Some "red" tests could not be red; tick-layer soak already true; R1 covered only 159 pairs; no tightening of G1 | Section 4 |
| 4a's tier reason; 0.44 ms said two ways; "part of a chunk"; f32 tempo reference | Sections 1, 2.2, 3, 5 |

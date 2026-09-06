# 03: Sequencer Core (`octocore`)

**Owner:** Metronome. **Gate:** `verify:conformance`, `verify:timing`, `verify:determinism`.
**Source of truth for behaviour:** Octopus CE OS v5.30 Reference Manual.

---

## 1. Principle: the manual is the specification

Every behavioural decision in this crate must be traceable to a page of the reference
manual. Where the manual is ambiguous, the ambiguity is recorded in
`tests/conformance/AMBIGUITIES.md` with the competing readings and the one we chose,
so that it can be revisited when someone with real hardware can arbitrate.

Practically this means **every non-obvious constant carries a citation**:

```rust
/// Track shuffle delay in ticks at 192 PPQN.
/// Even settings are randomised within a range.
/// Ref: CE v5.30 §3 Track Mode, "GRV attribute", p.48
const GRV_DELAY: [GrvDelay; 17] = [ /* … */ ];
```

`verify:conformance` includes a lint that fails on any `const` in `octocore::tables`
without a `Ref:` line. This sounds pedantic. It is the difference between a
reimplementation and a guess that happens to compile.

---

## 2. The model

### Dimensions (fixed, from the hardware)

| Entity | Count | Notes |
|---|---|---|
| Banks | 10 | the grid |
| Pages per bank | 16 | 160 pages total |
| Tracks per page | 10 | panel rows, numbered 9 (top) to 0 (bottom) |
| Steps per track | 16 | one matrix row |
| Page sets | 16 | ordered lists of page references |
| Phrases | 16 | each with 8 notes |
| MIDI ports | 2 | channels 1–16 on port 1, 17–32 on port 2 |
| PPQN | 192 | one step at default length = 12 ticks = 1/16 |

Track indices in code run 0..=9 and correspond to panel row labels 0..=9, where 9 is
the **top** row. Do not silently flip this. The effector's feeder/listener direction
depends on it (higher indices may modulate lower ones, and track 0 may modulate
nothing), so an off-by-inversion here produces subtly wrong music rather than an
obvious crash.

### Attributes

Ten attributes exist at track level and a subset at step level, matching the ten
EDIT encoders on the panel.

| Code | Name | Track | Step | Meaning |
|---|---|---|---|---|
| VEL | Velocity | ✓ | ✓ | MIDI velocity; step value 0 suppresses the note entirely |
| PIT | Pitch | ✓ | ✓ | note number; middle C = 60 = C5 in Octopus naming |
| LEN | Length | ✓ | ✓ | gate length, with a track-level multiplier |
| STA | Start | ✓ | ✓ | intra-step start offset |
| POS | Position | ✓ | ✗ | track rotation / phase |
| DIR | Direction | ✓ | ✗ | playback direction, 1–5 fixed, 6+ user-editable |
| AMT | Amount | ✓ | ✗ | scales the track's own offsets; neutral value recommended |
| GRV | Groove / phrase | ✓ | ✓ | track: shuffle table; step: phrase index 1–16 |
| MCC | MIDI CC | ✓ | ✓ | track: which CC; step: the value |
| MCH | MIDI channel | ✓ | ✗ | 1–32 across two ports |
| FLT | Flat | ✓ | ✗ | page-level flattening |

Step values are **offsets** applied to the track value, except where the manual
states an attribute is absolute. DIR, POS, and MCH are absolute when set as step
events. This asymmetry is real and must be preserved.

### Modes

Four operating modes, selected by the spiral's mode buttons: **Step**, **Track**,
**Page**, **Grid**. Mode determines what the matrix displays and what the encoders
edit. The core owns mode state because mode changes are musically observable
(for example, some values only take effect on transport stop).

---

## 3. Behaviours that must be ported exactly

These are the features that make an Octopus an Octopus. Each gets a dedicated
conformance fixture set. Listed roughly in order of implementation risk.

**Directions.** Directions 1–5 are fixed and read-only: forward, reverse, ping-pong,
random, brownian (confirm exact assignment against the manual during port).
Directions 6 and above are user-programmed step sequences: the user selects an
ordering by lighting cells in rows above row 0, and the trigger order is given by the
rows. Empty rows are skipped. If all rows are empty, the direction is random. This is
one of the deepest features on the machine and the fixture set should be large.

**Track chains.** Tracks can be chained so that several rows form one longer
sequence. A chained track has a head and members; the head owns the runtime cursor
and the members contribute their steps as segments. Mute, solo, and pause interact
with chains in specific ways (a member may be paused independently; mute and solo
behaviour depends on the chain's configuration).

**Hypersteps.** A step can be "hyped" so that pressing a step button in another row
while the hyped step is held creates a cross-track relationship, carrying PIT and VEL.

**Phrases.** Sixteen phrases, eight notes each, with per-note VEL, PIT, LEN, STA
offsets and a type. Type 4 randomises the programmed note attributes, which is the
basis of several of the manual's suggested techniques. Polyphony setting on a phrase
determines how many of the pool actually fire, which is how the machine produces
probabilistic rests (a one-note step with polyphony 2 plays half the time).

**Chords.** Per-step chord pool with a polyphony count, a random-pick mode, and a
strum with a timing table indexed by strum level and note ordinal.

**The effector.** Higher-indexed tracks (feeders) inject their current step's VEL,
PIT, and LEN offsets into lower-indexed tracks (listeners). Multiple feeders
accumulate. Track 0 can never be a feeder. Muting interacts with the effector in a
documented and non-obvious way.

**Step events and track toggles.** A step can carry an event that changes another
track's POS, DIR, or MCH, or toggles another track's Mute, Solo, or Record. Events
applied to *higher-indexed* tracks execute on the following step, because of the
top-down processing order. Range and value semantics wrap around the track set.
Track toggles are subordinate to on-the-measure mode.

**Scales.** Two levels: grid scale and page scale, each independently on or off, with
four combinations producing four documented behaviours (page scale, chromatic, locked
to grid, and so on). Quantisation happens after all pitch offsets are summed.

**MCC.** A track with MCC set to anything other than "none" displays an orange chase
light rather than red. This is a good early integration test because it couples the
core's state to the panel's LED colour through the snapshot, exercising the whole
pipeline.

---

## 4. Port plan from v1

`octopus_scheduler.js` is the only substantial asset. Port procedure, per function:

1. Read the JS.
2. Find the manual page that justifies it. Record the citation.
3. Write the Rust with the citation.
4. Write a conformance fixture from the *manual's* description, not from the JS.
5. Run it. If the JS and the fixture disagree, the fixture wins and the JS was wrong.

Step 4 is the important one. Porting tests along with code just launders the original
author's misreadings into the new codebase. Two tables are known to need
re-derivation this way:

- `_grvDelayTicks` (the shuffle table): the JS randomises even settings over a
  three-tick window. Verify the window against the manual's table.
- `_strumOffsetTicks` (chord strum): the JS references a "Chord Strum Timings in
  Ticks" table for strum 1–9 and notes 2–7. Verify all 54 entries.

---

## 5. Command and event interfaces

```rust
#[repr(C)]
pub enum Command {
    // transport
    Play, Stop, Continue, Reset,
    // panel actuation, expressed physically
    ButtonDown { control: ControlId, velocity_mm_s: f32 },
    ButtonUp   { control: ControlId },
    EncoderTurn{ control: ControlId, detents: i16, angular_velocity: f32 },
    // structural
    SetActivePage { bank: u8, page: u8 },
    SetMode { mode: Mode },
    // host
    HostTransport { ppqn_pos: u64, bpm: f32, playing: bool },
    LoadState { handle: StateHandle },
}
```

Note `ButtonDown` carries an actuation velocity and `EncoderTurn` carries angular
velocity. The core mostly ignores them, but the *renderer* needs them for physical
motion (see [Render §6](04-render-engine.md)), and putting them on the command means
input is described once, physically, rather than twice in two vocabularies.

`ControlId` is the same identifier used in `panel.truth.json`. One namespace for the
whole system.

```rust
#[repr(C)]
pub enum Event {
    NoteOn  { port: u8, ch: u8, note: u8, vel: u8, at_sample: u32 },
    NoteOff { port: u8, ch: u8, note: u8,           at_sample: u32 },
    Cc      { port: u8, ch: u8, cc: u8, val: u8,    at_sample: u32 },
}
```

`at_sample` is an offset within the current audio buffer. The core computes it from
tick position and the buffer's sample rate. This is what makes timing
sample-accurate rather than buffer-quantised, and it is the single most important
difference between this and a Max for Live implementation.

---

## 6. Snapshot

```rust
#[repr(C)]
pub struct Snapshot {
    pub generation: u64,
    pub leds: [Led; MAX_CONTROLS],     // color + intensity 0..1, pre-decay
    pub encoders: [EncoderState; 20],  // absolute angle in radians, detent index
    pub playheads: [PlayheadState; 10],
    pub transport: TransportState,
    pub mode: Mode,
    pub active: ActiveRefs,
}

#[repr(C)]
pub struct Led { pub color: LedColor, pub target: f32 }
```

Note `target`, not `value`. The core publishes the LED's *commanded* state. The
renderer runs the rise/decay simulation itself at frame rate, because LED physics is
a display concern and running it at 192 PPQN would alias badly against a 120 Hz
display. This split matters: the core says "on", the renderer says how "on" looks
4.2 ms later.

---

## 7. Timing requirements

Stricter than the visual budget, because a musician forgives a dropped frame and does
not forgive a late note.

| Metric | Target | Gate |
|---|---|---|
| Note-on jitter, σ | ≤ 120 µs | `verify:timing` |
| Note-on jitter, max | ≤ 500 µs | `verify:timing` |
| Missed ticks in 30 min | 0 | `verify:timing:deep` |
| Audio-thread allocations | 0 | `verify:timing` (custom allocator asserts) |
| Audio-thread locks | 0 | `verify:timing` (instrumented mutex shim in test builds) |
| Worst-case tick duration | ≤ 250 µs at full density | `verify:timing` |

"Full density" means all ten tracks active, all sixteen steps on, chords at maximum
polyphony, all phrases firing, and the effector fully wired. Measure the pathological
case, not the typical one.

**Measurement method.** In CI, a null host drives the core with an exact synthetic
clock and records requested versus emitted sample offsets; jitter is measured
analytically with no OS involvement. In the lab, a loopback MIDI capture in Live 12
records against the host clock. Both numbers go in the report. The synthetic number
proves the core is right; the loopback number proves the integration is.

---

## 8. Conformance fixtures

Format: a small text DSL so fixtures are readable and writable by hand, which matters
because most of them are transcribed from manual prose.

```
# tests/conformance/effector/feeder_pit.fixture
# Ref: CE v5.30 §3, "Effector - Feeders/Listener", p.60
seed 42
page.tracks 9,8,4,0 enabled
track 9 role feeder
track 8 role feeder
track 4 role listener
track 0 role listener
track 9 step 0 pit +3
track 4 step 0 pit 0
play 1 step
expect note track=4 pit=+3
expect note track=0 pit=+3
```

The manual's own worked examples (the Appendix "Octopus Techniques" section is full
of them, and the direction-editing walkthroughs give exact expected chase-light
behaviour) convert almost directly into fixtures. **Mine the appendix first**: it is
free, high-quality test data written by the people who built the machine.

Target: ≥ 250 conformance fixtures by the end of Phase 3, with at least one per
manual section.

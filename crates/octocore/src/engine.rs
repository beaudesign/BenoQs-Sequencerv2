//! The tick loop. Ported from `archive/v1-max4live/octopus_engine.js` `tick()`
//! where that function's behaviour is trustworthy, extended with the effector,
//! phrases, step events and sample-accurate scheduling that v1 never implemented.
//!
//! Processing order is descending track index (9 downto 0), not ascending as in
//! v1. Confirmed against the real manual: Ref: CE v5.30 §3 Track Mode, "The EFF
//! mechanism", p.59-60 — "The modulation is always happening 'top-down', i.e.
//! upper tracks may modulate lower tracks, but not vice-versa... track 9 may
//! modulate all other tracks but track 0 cannot modulate any other track." This
//! only holds in code if a feeder/source track is actually computed before the
//! lower-indexed track that reads it, in the *same* tick — v1 looped ascending
//! and had no effector at all, so this was a deliberate correction, not a port,
//! made before the manual confirmed it was the right call.
//!
//! Each tick reads from a single `Page` snapshot taken at tick start (`Page` is
//! `Copy`), so every track sees the same page state regardless of processing
//! order — this is also why step events (Ref: CE v5.30 p.34-40) apply to live
//! state at the *start of the following tick* for every target, not just
//! higher-indexed ones as the manual actually specifies (see
//! `Engine::deferred_actions`'s doc comment for the full reasoning and
//! AMBIGUITIES.md "step events: same-tick timing").

use crate::domain::*;
use crate::rng::Rng;
use crate::scale;
use crate::tables;
use crate::types::{Event, MAX_EVENTS_PER_TICK};

/// A step event already resolved to one target track and one primitive
/// change — `StepEventKind::TrackToggle`'s AMT/Range addressing is resolved
/// into zero or more of these (one per affected track) at the moment the
/// event fires, so the deferred-application queue never needs to re-resolve
/// addressing later.
#[derive(Clone, Copy, Debug)]
enum DeferredAction {
    AddDir(i8),
    AddPos(i8),
    AddMch(i8),
    ToggleOn(ToggleKind),
    ToggleOff(ToggleKind),
    /// Ref: CE v5.30 p.38-39. Applied to the event's own track (`source`),
    /// not via AMT-as-target — AMT's sign/magnitude is the rotate direction
    /// and distance, not a track index.
    Rotate(i8),
    SkipRotate(i8),
}

#[derive(Clone, Copy, Debug, Default)]
struct ChainRuntime {
    member_idx: u8,
    seg_pos: u8,
    ping_dir: i8,
}

#[derive(Clone, Copy, Debug)]
struct TrackRuntime {
    accum: f32,
    pos: u8,
    ping_dir: i8,
    chain: ChainRuntime,
    /// The offsets of this track's current step, exposed for listeners if this
    /// track is a feeder. Updated at this track's own step boundary regardless of
    /// role, so a track can be freely rewired feeder/listener without stale data.
    feed_pitch: i8,
    feed_velocity: i8,
    feed_length_ticks: i32,
    /// Ref: CE v5.30 p.40, "DIR will default to the Track attribute amount
    /// when the sequencer stops" — `Some` while a Step Event has live-overridden
    /// this track's DIR; `None` means "use the persisted `Track::direction_raw`".
    /// Cleared on `Engine::set_running(false)`.
    live_direction_raw: Option<u8>,
    /// Ref p.40, "MCH will also default to the Track attribute amount when
    /// the sequencer stops" — same shape as `live_direction_raw`.
    live_midi_channel: Option<u8>,
    /// Ref: CE v5.30 p.56: "a slice is not restricted to holding only one
    /// trigger, but may hold up to nine triggers. These nine triggers will be
    /// played in sequence every time the respective slice is being played."
    /// Which trigger within the *current* slice (`pos`/`chain.seg_pos`) fires
    /// next, for a `Direction::UserProgrammed` track. Reset to 0 whenever the
    /// slice itself advances.
    custom_dir_cursor: u8,
}

impl Default for TrackRuntime {
    fn default() -> Self {
        TrackRuntime {
            accum: 0.0,
            pos: 0,
            ping_dir: 1,
            chain: ChainRuntime { member_idx: 0, seg_pos: 0, ping_dir: 1 },
            feed_pitch: 0,
            feed_velocity: 0,
            feed_length_ticks: 0,
            live_direction_raw: None,
            live_midi_channel: None,
            custom_dir_cursor: 0,
        }
    }
}

/// A scheduled MIDI event, queued by absolute sample time (an `f64` monotonic
/// timeline so multi-hour sessions at high sample rates don't lose the precision
/// an accumulated `f32` would).
#[derive(Clone, Copy, Debug)]
struct Scheduled {
    due_sample: f64,
    event: RawEvent,
    /// The queue's sort key: (whole sample the event lands in, `RawEvent::rank`, creation
    /// number). See `Engine::queue`.
    order: (i64, u8, u64),
}

#[derive(Clone, Copy, Debug)]
enum RawEvent {
    NoteOn { port: u8, ch: u8, note: u8, vel: u8 },
    NoteOff { port: u8, ch: u8, note: u8 },
    Cc { port: u8, ch: u8, cc: u8, val: u8 },
    PitchBend { port: u8, ch: u8, value: u16 },
    ChannelPressure { port: u8, ch: u8, value: u8 },
}

impl RawEvent {
    /// Order of events that land in the same sample: releases first, then controllers, then
    /// new notes. A note that ends where the same pitch starts again must release before it
    /// strikes, and a controller change should reach the receiver before the note it shapes.
    fn rank(&self) -> u8 {
        match self {
            RawEvent::NoteOff { .. } => 0,
            RawEvent::Cc { .. } | RawEvent::PitchBend { .. } | RawEvent::ChannelPressure { .. } => 1,
            RawEvent::NoteOn { .. } => 2,
        }
    }
}

/// Events the engine can hold between scheduling and emission. A heavy but ordinary pattern
/// (every step a strummed 6-note chord at 240 BPM) peaks near 430; the overload scene fills
/// it. Public so a test or a host can compare `Diagnostics::queue_high_water` with it.
pub const QUEUE_CAP: usize = 1024;

/// The furthest a note can start ahead of the tick that fires it, in ticks. It is the most
/// negative entry of `tables::STA_TABLE` (Track STA 16, Step STA -5: "-12", CE v5.30
/// p.45); shuffle, strum and phrase offsets only ever push a note later. `render` steps
/// every tick this far before the tick's time, so a note pulled early is still in the
/// future when it is scheduled. A unit test checks this value against the tables, so
/// editing a table without editing this constant fails the build.
pub const MAX_EARLY_TICKS: u32 = 12;

/// How many NoteOns each note has had without a matching NoteOff, as the receiver sees it:
/// `[port - 1][channel - 1][note]`. 2 x 16 x 128 bytes is 4 KB with no allocation.
/// Updated only when an event is actually handed to the caller, never when it is merely
/// scheduled, so it is exactly what a synth on the other end would be holding.
type SoundingTable = [[[u8; 128]; 16]; 2];

/// The `SoundingTable` index for a note, or `None` for a port, channel or note outside the
/// MIDI ranges (ports are 1 and 2, channels 1 to 16, notes 0 to 127). Such an event is
/// still emitted; it just is not tracked, so a bad value can never index out of bounds.
fn sounding_slot(port: u8, ch: u8, note: u8) -> Option<(usize, usize, usize)> {
    if (1..=2).contains(&port) && (1..=16).contains(&ch) && note < 128 {
        Some(((port - 1) as usize, (ch - 1) as usize, note as usize))
    } else {
        None
    }
}

pub struct EventBuffer {
    events: [Event; MAX_EVENTS_PER_TICK],
    count: usize,
    /// How many events this buffer accepts: `MAX_EVENTS_PER_TICK`, or less when the caller
    /// has less room (see `with_limit`).
    limit: usize,
}

impl Default for EventBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBuffer {
    pub fn new() -> Self {
        EventBuffer {
            events: [Event::Cc { port: 0, ch: 0, cc: 0, val: 0, at_sample: 0 }; MAX_EVENTS_PER_TICK],
            count: 0,
            limit: MAX_EVENTS_PER_TICK,
        }
    }

    /// A buffer that takes at most `limit` events (and never more than
    /// `MAX_EVENTS_PER_TICK`). The engine treats a full buffer as "not now": whatever does not
    /// fit is emitted by the next `render`, so a host with a small buffer of its own delays
    /// events and loses none.
    pub fn with_limit(limit: usize) -> Self {
        EventBuffer { limit: limit.min(MAX_EVENTS_PER_TICK), ..Self::new() }
    }

    pub fn as_slice(&self) -> &[Event] {
        &self.events[..self.count]
    }

    pub fn clear(&mut self) {
        self.count = 0;
    }

    /// Room left in this buffer.
    pub fn remaining(&self) -> usize {
        self.limit.saturating_sub(self.count)
    }

    /// Appends `e`. Returns `false`, and does not add it, when the buffer is full. Callers
    /// keep the event and offer it again on the next render.
    fn push(&mut self, e: Event) -> bool {
        if self.count < self.limit {
            self.events[self.count] = e;
            self.count += 1;
            true
        } else {
            false
        }
    }
}

pub struct RenderContext {
    pub sample_rate: f32,
    pub buffer_len: u32,
    pub bpm: f32,
    pub playing: bool,
}

/// Counters for things the engine used to do silently. All are cumulative since
/// `Engine::new`, read with `Engine::diagnostics`. A host that shows a health light watches
/// for them to change. None of them is an error the engine cannot continue from.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Diagnostics {
    /// Notes (or CCs) refused because the event queue was full. A note is refused whole,
    /// never as a lone NoteOn.
    pub queue_overflows: u32,
    /// The most events the queue has held at once. `QUEUE_CAP` is 1,024.
    pub queue_high_water: u32,
    /// Times an event that was due had to wait for a later render because the caller's
    /// buffer was full. It is counted once per internal chunk of up to `RENDER_CHUNK`
    /// samples that the event waits through, so one large render can count the same waiting
    /// event several times. Read it as "nonzero means the caller's buffer was too small",
    /// not as a number of events.
    pub deferred_events: u32,
    /// Events emitted after their time, at sample 0 of the buffer, because they were
    /// deferred or scheduled in the past.
    pub late_events: u32,
    /// Renders with the transport running but a tempo or sample rate the engine cannot use
    /// (zero, negative, NaN, infinite, or outside `MIN_BPM..=MAX_BPM` / `MIN_SAMPLE_RATE..=MAX_SAMPLE_RATE`). No tick runs then.
    pub unusable_tempo_renders: u32,
}

/// The tempos the engine follows. Hosts report 0 before playback starts, and a stray NaN or
/// infinity from a broken plugin must not stop the clock or hang the audio thread (an
/// infinite tempo means zero samples per tick, which is an endless loop).
pub const MIN_BPM: f32 = 1.0;
pub const MAX_BPM: f32 = 999.0;
/// The sample rates the engine follows. Together with the tempo range this bounds the ticks
/// one buffer can contain (about 1,700 for 4,096 samples at the extremes).
pub const MIN_SAMPLE_RATE: f32 = 8_000.0;
pub const MAX_SAMPLE_RATE: f32 = 768_000.0;

/// Longest stretch `render` handles in one go. The queue then only ever holds one chunk's
/// worth of notes plus the lookahead, however big the host's buffer is.
const RENDER_CHUNK: u32 = 1024;

fn samples_per_tick(sample_rate: f32, bpm: f32) -> Option<f64> {
    let rate_ok = sample_rate.is_finite() && (MIN_SAMPLE_RATE..=MAX_SAMPLE_RATE).contains(&sample_rate);
    let tempo_ok = bpm.is_finite() && (MIN_BPM..=MAX_BPM).contains(&bpm);
    if rate_ok && tempo_ok {
        Some(sample_rate as f64 * 60.0 / (bpm as f64 * TICKS_PER_QUARTER as f64))
    } else {
        None
    }
}

/// Everything about a fired note that still needs `&mut self` (rng draws for
/// chord/strum, and scheduling) but requires no further borrow of `self.grid` —
/// computed from a `Page` snapshot copied by value, never a live reference, so it
/// can be passed alongside `&mut self` without a borrow conflict.
struct FireInputs {
    ti: TrackIndex,
    page_pitch_offset: i8,
    page_velocity_factor: u8,
    effective_scale: Option<scale::PitchClassSet>,
    base_track: Track,
    step: Step,
    shuffle_delay: u32,
    feed_pit: i32,
    feed_vel: i32,
    feed_len: i32,
    routing_mode: RoutingMode,
    fixed_routing: FixedRouting,
}

/// A note as it actually fired this tick, tagged by track — the MIDI event queue
/// (port/channel/note) can't be unambiguously mapped back to a track, so this is
/// the debug/test seam for "what did track N just play". Also the natural start of
/// the "debug overlays" ROLES.md asks the Forge to build early.
#[derive(Clone, Copy, Debug)]
pub struct NoteFire {
    pub track: TrackIndex,
    pub pitch: u8,
    pub velocity: u8,
}

const FIRE_LOG_CAP: usize = 32;

pub struct FireLog {
    entries: [NoteFire; FIRE_LOG_CAP],
    count: usize,
}

impl Default for FireLog {
    fn default() -> Self {
        FireLog { entries: [NoteFire { track: 0, pitch: 0, velocity: 0 }; FIRE_LOG_CAP], count: 0 }
    }
}

impl FireLog {
    fn clear(&mut self) {
        self.count = 0;
    }

    fn push(&mut self, nf: NoteFire) {
        if self.count < FIRE_LOG_CAP {
            self.entries[self.count] = nf;
            self.count += 1;
        }
    }

    pub fn as_slice(&self) -> &[NoteFire] {
        &self.entries[..self.count]
    }
}

pub struct Engine {
    pub grid: Grid,
    rng: Rng,
    running: bool,
    global_tick: u64,
    /// Absolute sample time (since `Engine::new`) of the next tick boundary.
    next_tick_due: f64,
    /// Absolute sample time elapsed so far.
    sample_clock: f64,
    track_rt: [TrackRuntime; TRACK_COUNT],
    /// Scheduled events as a binary min-heap on `Scheduled::order`: the earliest sample
    /// first, then NoteOffs before controllers before NoteOns, then first scheduled first.
    /// That key is total, so the order events leave in never depends on the host's buffer
    /// size or on how many events came out before (SPEC-0001 O10).
    queue: [Option<Scheduled>; QUEUE_CAP],
    queue_len: usize,
    /// Creation number of the next scheduled event, the last part of the sort key.
    next_seq: u64,
    /// Notes currently on in the receivers. See `SoundingTable`.
    sounding: SoundingTable,
    /// Set by Stop and Reset. The next `render` turns off everything in `sounding` before it
    /// does anything else. Stays set across renders if the output buffer fills up, so a
    /// flush of more notes than one buffer holds still completes.
    flush_pending: bool,
    /// Channels that still owe a CC 123 (bit `c` of entry `p` is port `p + 1`, channel
    /// `c + 1`), sent once the NoteOffs are out.
    flush_cc: [u16; 2],
    /// Step events, already resolved to a single target and a primitive
    /// action, applied to live state at the start of the *next* tick.
    ///
    /// Ref: CE v5.30 p.40: "All tracks are processed... top-down... when a
    /// [step event] is applied to tracks higher in the matrix it will be
    /// executed on the following step" — for tracks *lower* in the matrix
    /// (not yet processed this tick) the manual has same-tick application
    /// take effect before that track's own step plays. This engine applies
    /// every step event at the next tick uniformly instead, regardless of
    /// target index: same-tick application would require a target track to
    /// see a live mutation made *during* the same tick's processing, but
    /// every track instead reads from one `Page` snapshot taken once at tick
    /// start (see module docs) specifically so tracks don't see each other's
    /// mid-tick mutations — that's what makes the effector's ordering
    /// reasoning sound. Supporting genuine same-tick step events would need
    /// reworking that snapshot architecture; deferred rather than done
    /// halfway. See AMBIGUITIES.md "step events: same-tick timing".
    deferred_actions: [Option<DeferredAction>; TRACK_COUNT],
    /// Ref: CE v5.30 p.40, Track Toggle Consideration 5: "Track Toggle Events
    /// of Mute & Solo are subordinate to the On-The-Measure mode condition of
    /// the Page." Held here instead of `deferred_actions` when
    /// `Page::on_the_measure` is set at the moment a Mute/Solo toggle fires;
    /// applied at the next measure boundary rather than the next tick — see
    /// `queue_step_event` and `step_all_tracks`.
    measure_deferred: [Option<DeferredAction>; TRACK_COUNT],
    /// What fired *this tick*, cleared and refilled every `step_all_tracks` call.
    pub last_tick_fires: FireLog,
    diag: Diagnostics,
}

impl Engine {
    pub fn new(seed: u64) -> Self {
        let mut grid = Grid::default_grid();
        // Ref: CE v5.30 p.16 / p.23-26 — a fresh machine ships the three
        // factory phrase banks, not an empty pool.
        grid.phrases = crate::phrases::factory_phrases();
        Engine {
            grid,
            rng: Rng::new(seed),
            running: false,
            global_tick: 0,
            next_tick_due: 0.0,
            sample_clock: 0.0,
            track_rt: [TrackRuntime::default(); TRACK_COUNT],
            queue: [None; QUEUE_CAP],
            queue_len: 0,
            next_seq: 0,
            sounding: [[[0; 128]; 16]; 2],
            flush_pending: false,
            flush_cc: [0; 2],
            deferred_actions: [None; TRACK_COUNT],
            measure_deferred: [None; TRACK_COUNT],
            last_tick_fires: FireLog::default(),
            diag: Diagnostics::default(),
        }
    }

    pub fn diagnostics(&self) -> Diagnostics {
        self.diag
    }

    /// Test/fixture seam: run exactly one 192-PPQN tick, ignoring real-time
    /// sample scheduling, and leave `last_tick_fires` populated with whatever
    /// fired. `tests/conformance` fixtures are written in ticks, not samples.
    pub fn step_once_for_test(&mut self) {
        self.step_all_tracks(0.0, 1.0);
    }

    pub fn set_running(&mut self, running: bool) {
        if !self.running && running {
            // Play starts from now. While stopped, `render` advances `sample_clock` but no
            // ticks run, so `next_tick_due` still points at the moment the transport
            // stopped. Left alone, Play would replay every idle tick in one call.
            self.next_tick_due = self.sample_clock;
        }
        if self.running && !running {
            self.all_notes_off_now();
            // Ref: CE v5.30 p.40: "DIR will default to the Track attribute
            // amount when the sequencer stops" / "MCH will also default to
            // the Track attribute amount when the sequencer stops" — POS is
            // deliberately not cleared here ("POS does not restore to the
            // start POS(ition) when stopping the sequencer").
            for rt in self.track_rt.iter_mut() {
                rt.live_direction_raw = None;
                rt.live_midi_channel = None;
            }
        }
        self.running = running;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Handles the non-timing commands from `docs/03-sequencer-core.md` §5.
    /// `ButtonDown`/`ButtonUp`/`EncoderTurn` are accepted but currently no-ops:
    /// there is no `panel.truth.json` yet (reference/NOTES.md), so `ControlId`
    /// has no real mapping to a track/step/encoder to act on. `LoadState` is a
    /// no-op for the same reason state serialization doesn't exist yet.
    pub fn handle_command(&mut self, cmd: crate::types::Command) {
        use crate::types::Command;
        match cmd {
            Command::Play | Command::Continue => self.set_running(true),
            Command::Stop => {
                // Ref: CE v5.30 p.94, "ALL NOTES OFF message": Stop pressed while the
                // sequencer is not running sends CC 123 on each of the 32 MIDI channels.
                // (The manual says "when defined as MIDI master or slave"; this engine has
                // no such setting, so the condition is treated as always true. See
                // AMBIGUITIES.md "stop with sounding notes".)
                if !self.running {
                    self.flush_pending = true;
                    self.flush_cc = [u16::MAX; 2];
                }
                self.set_running(false);
            }
            Command::Reset => self.reset(),
            Command::SetActivePage { bank, page } => {
                self.grid.active_page = ActivePage {
                    bank: bank.min(BANK_COUNT as u8 - 1),
                    page: page.min(PAGE_COUNT as u8 - 1),
                };
            }
            Command::SetMode { mode } => self.grid.mode = mode,
            Command::HostTransport { playing, .. } => self.set_running(playing),
            Command::ButtonDown { .. } | Command::ButtonUp { .. } | Command::EncoderTurn { .. } | Command::LoadState { .. } => {}
        }
    }

    pub fn reset(&mut self) {
        self.running = false;
        // The queue below is cleared, so the NoteOffs it held are gone. The sounding table is
        // kept on purpose: the next render turns those notes off.
        self.flush_pending = true;
        self.global_tick = 0;
        self.next_tick_due = 0.0;
        self.sample_clock = 0.0;
        self.track_rt = [TrackRuntime::default(); TRACK_COUNT];
        self.queue = [None; QUEUE_CAP];
        self.queue_len = 0;
        self.deferred_actions = [None; TRACK_COUNT];
        self.measure_deferred = [None; TRACK_COUNT];
    }

    fn schedule(&mut self, due_sample: f64, event: RawEvent) {
        if self.queue_len < QUEUE_CAP {
            let order = (due_sample.floor() as i64, event.rank(), self.next_seq);
            self.next_seq += 1;
            self.heap_push(Scheduled { due_sample, event, order });
            self.diag.queue_high_water = self.diag.queue_high_water.max(self.queue_len as u32);
        } else {
            // Never reallocates on the audio path. QUEUE_CAP is sized well above "full
            // density" (docs §7), and this counter says when it was not.
            self.diag.queue_overflows = self.diag.queue_overflows.saturating_add(1);
        }
    }

    fn order_at(&self, i: usize) -> (i64, u8, u64) {
        self.queue[i].map_or((i64::MAX, u8::MAX, u64::MAX), |s| s.order)
    }

    fn heap_push(&mut self, s: Scheduled) {
        let mut i = self.queue_len;
        self.queue[i] = Some(s);
        self.queue_len += 1;
        while i > 0 {
            let parent = (i - 1) / 2;
            if self.order_at(i) < self.order_at(parent) {
                self.queue.swap(i, parent);
                i = parent;
            } else {
                break;
            }
        }
    }

    /// Removes and returns the earliest event.
    fn heap_pop(&mut self) -> Option<Scheduled> {
        if self.queue_len == 0 {
            return None;
        }
        let top = self.queue[0].take();
        self.queue_len -= 1;
        if self.queue_len > 0 {
            self.queue[0] = self.queue[self.queue_len].take();
            let mut i = 0;
            loop {
                let (l, r) = (2 * i + 1, 2 * i + 2);
                let mut smallest = i;
                if l < self.queue_len && self.order_at(l) < self.order_at(smallest) {
                    smallest = l;
                }
                if r < self.queue_len && self.order_at(r) < self.order_at(smallest) {
                    smallest = r;
                }
                if smallest == i {
                    break;
                }
                self.queue.swap(i, smallest);
                i = smallest;
            }
        }
        top
    }

    /// Queues a note: its NoteOn at `on` and its NoteOff at `off`, both or neither. Room for
    /// the NoteOn alone would leave a note sounding until the next Stop, so a queue with one
    /// free slot refuses the whole note (and counts it).
    fn schedule_note(&mut self, on: f64, off: f64, port: u8, ch: u8, note: u8, vel: u8) {
        if self.queue_len + 2 > QUEUE_CAP {
            self.diag.queue_overflows = self.diag.queue_overflows.saturating_add(1);
            return;
        }
        self.schedule(on, RawEvent::NoteOn { port, ch, note, vel });
        self.schedule(off, RawEvent::NoteOff { port, ch, note });
    }

    /// Drops everything still scheduled and asks the next `render` to turn off every note
    /// that is sounding (a NoteOff per NoteOn still owed, then CC 123 on each channel that
    /// had one). Until SPEC-0001 O1 this only cleared the queue, so a note whose NoteOff was
    /// still queued stayed on in the receiver forever.
    fn all_notes_off_now(&mut self) {
        self.queue = [None; QUEUE_CAP];
        self.queue_len = 0;
        self.flush_pending = true;
    }

    /// Number of distinct (port, channel, note) the receivers are holding on right now.
    /// A diagnostic for tests and the future snapshot; it scans 4 KB, so keep it off the
    /// per-sample path.
    pub fn sounding_count(&self) -> usize {
        self.sounding.iter().flatten().flatten().filter(|n| **n > 0).count()
    }

    fn track_emitted(&mut self, e: RawEvent) {
        match e {
            RawEvent::NoteOn { port, ch, note, .. } => {
                if let Some((p, c, n)) = sounding_slot(port, ch, note) {
                    self.sounding[p][c][n] = self.sounding[p][c][n].saturating_add(1);
                }
            }
            RawEvent::NoteOff { port, ch, note } => {
                if let Some((p, c, n)) = sounding_slot(port, ch, note) {
                    self.sounding[p][c][n] = self.sounding[p][c][n].saturating_sub(1);
                }
            }
            RawEvent::Cc { .. } | RawEvent::PitchBend { .. } | RawEvent::ChannelPressure { .. } => {}
        }
    }

    /// Emits the flush requested by `all_notes_off_now`, `reset` or a Stop while stopped, at
    /// sample 0 of this buffer. If the buffer fills, it returns with `flush_pending` still
    /// set and the table still holding what is left, so the next render carries on. (If the
    /// transport restarts before then, notes the new run has started are flushed too. That
    /// needs more than one buffer's worth of sounding notes, and it can only cut a note
    /// short, never leave one on.)
    fn emit_pending_flush(&mut self, out: &mut EventBuffer) {
        if !self.flush_pending {
            return;
        }
        for p in 0..2 {
            for c in 0..16 {
                for n in 0..128 {
                    // One NoteOff per NoteOn still owed. A pitch retriggered while it was
                    // sounding is held twice by a receiver that stacks voices, and a single
                    // NoteOff would release only one of them.
                    while self.sounding[p][c][n] > 0 {
                        let off = Event::NoteOff { port: p as u8 + 1, ch: c as u8 + 1, note: n as u8, at_sample: 0 };
                        if !out.push(off) {
                            return;
                        }
                        self.sounding[p][c][n] -= 1;
                        self.flush_cc[p] |= 1 << c;
                    }
                }
            }
        }
        for p in 0..2 {
            for c in 0..16 {
                if self.flush_cc[p] & (1 << c) == 0 {
                    continue;
                }
                let cc = Event::Cc { port: p as u8 + 1, ch: c as u8 + 1, cc: 123, val: 0, at_sample: 0 };
                if !out.push(cc) {
                    return;
                }
                self.flush_cc[p] &= !(1 << c);
            }
        }
        self.flush_pending = false;
    }

    /// Advance the engine by exactly `ctx.buffer_len` samples, appending any events
    /// due in that window to `out`. `out` is not cleared by this call — callers
    /// rendering buffer-by-buffer should clear it between calls. Events that do not fit in
    /// `out` stay queued and go out first in the next call; nothing is dropped for lack of
    /// room in `out`.
    pub fn render(&mut self, ctx: &RenderContext, out: &mut EventBuffer) {
        if ctx.playing != self.running {
            self.set_running(ctx.playing);
        }
        self.emit_pending_flush(out);

        let buffer_start = self.sample_clock;
        let spt = samples_per_tick(ctx.sample_rate, ctx.bpm);
        if self.running && spt.is_none() {
            self.diag.unusable_tempo_renders = self.diag.unusable_tempo_renders.saturating_add(1);
        }

        // Work through the buffer in chunks, stepping ticks and then draining events for
        // each. A big host buffer then never piles a whole buffer's notes into the queue.
        let mut done = 0u32;
        while done < ctx.buffer_len {
            done += (ctx.buffer_len - done).min(RENDER_CHUNK);
            let chunk_end = buffer_start + done as f64;

            if self.running {
                match spt {
                    Some(spt) => {
                        // Step ticks up to `MAX_EARLY_TICKS` past the chunk's end. A tick that
                        // is stepped only once its time has come can schedule a note before
                        // the start of the chunk being rendered, and such a note used to be
                        // clamped to the buffer's first sample: late by an amount that
                        // depended on where the host's buffer boundary fell (SPEC-0001 O3).
                        // Stepped this far ahead, every note is still in the future when it
                        // is scheduled, so its time depends only on the pattern and the tempo.
                        let horizon = chunk_end + MAX_EARLY_TICKS as f64 * spt;
                        while self.next_tick_due < horizon {
                            let at = self.next_tick_due;
                            self.step_all_tracks(at, spt);
                            self.next_tick_due += spt;
                        }
                    }
                    None => {
                        // No usable tempo: no ticks, and no backlog to replay once one comes
                        // back. Notes already queued still run to their NoteOffs.
                        if self.next_tick_due < chunk_end {
                            self.next_tick_due = chunk_end;
                        }
                    }
                }
            }

            self.drain_due(chunk_end, buffer_start, ctx.buffer_len, out);
        }

        self.sample_clock = buffer_start + ctx.buffer_len as f64;
    }

    /// Moves every queued event due before `until` into `out`, earliest first. When `out` is
    /// full the rest stay queued for the next render and are counted in `Diagnostics`.
    fn drain_due(&mut self, until: f64, buffer_start: f64, buffer_len: u32, out: &mut EventBuffer) {
        while self.queue_len > 0 {
            let Some(top) = self.queue[0] else { break };
            if top.due_sample >= until {
                break;
            }
            if out.remaining() == 0 {
                let waiting = self.queue[..self.queue_len].iter().flatten().filter(|s| s.due_sample < until).count();
                self.diag.deferred_events = self.diag.deferred_events.saturating_add(waiting as u32);
                break;
            }
            self.heap_pop();

            if top.due_sample < buffer_start {
                self.diag.late_events = self.diag.late_events.saturating_add(1);
            }
            let at_sample = (top.due_sample - buffer_start).max(0.0) as u32;
            let at_sample = at_sample.min(buffer_len.saturating_sub(1));
            let emitted = match top.event {
                RawEvent::NoteOn { port, ch, note, vel } => Event::NoteOn { port, ch, note, vel, at_sample },
                RawEvent::NoteOff { port, ch, note } => Event::NoteOff { port, ch, note, at_sample },
                RawEvent::Cc { port, ch, cc, val } => Event::Cc { port, ch, cc, val, at_sample },
                RawEvent::PitchBend { port, ch, value } => Event::PitchBend { port, ch, value, at_sample },
                RawEvent::ChannelPressure { port, ch, value } => Event::ChannelPressure { port, ch, value, at_sample },
            };
            if out.push(emitted) {
                self.track_emitted(top.event);
            }
        }
    }

    /// One 192-PPQN tick across every track, descending index order (see module
    /// docs). `tick_due_sample` is this tick's absolute sample time, the base for
    /// any note this tick schedules.
    fn step_all_tracks(&mut self, tick_due_sample: f64, samples_per_tick: f64) {
        self.global_tick += 1;
        self.last_tick_fires.clear();

        for ti in 0..TRACK_COUNT {
            if let Some(action) = self.deferred_actions[ti].take() {
                self.apply_deferred_action(ti as TrackIndex, action);
            }
        }

        // Ref: CE v5.30 p.67: "A measure is 16 steps at x1 speed, or if a page
        // has a Page Length of less than 16 then the Length of Measure will be
        // equal to the Page Length." Applied here, once per measure, rather
        // than every tick like `deferred_actions` above.
        let measure_len = self.grid.active_page().length.clamp(1, STEP_COUNT as u8) as u64;
        let measure_ticks = measure_len * DEFAULT_STEP_TICKS as u64;
        if self.global_tick % measure_ticks.max(1) == 0 {
            for ti in 0..TRACK_COUNT {
                if let Some(action) = self.measure_deferred[ti].take() {
                    self.apply_deferred_action(ti as TrackIndex, action);
                }
            }
        }

        // One snapshot for the whole tick: every track reads the same page state
        // regardless of processing order, and nothing borrowed from `self.grid`
        // needs to stay alive once this line finishes.
        let page: Page = *self.grid.active_page();
        let page_len = page.length.clamp(1, STEP_COUNT as u8);
        let routing_mode = self.grid.routing_mode;
        let fixed_routing = self.grid.fixed_routing;
        let effective_scale = self.effective_scale(&page);

        for ti_desc in 0..TRACK_COUNT {
            let ti = (TRACK_COUNT - 1 - ti_desc) as TrackIndex; // 9 downto 0
            self.step_one_track(ti, &page, page_len, effective_scale, routing_mode, fixed_routing, tick_due_sample, samples_per_tick);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn step_one_track(
        &mut self,
        ti: TrackIndex,
        page: &Page,
        page_len: u8,
        effective_scale: Option<scale::PitchClassSet>,
        routing_mode: RoutingMode,
        fixed_routing: FixedRouting,
        tick_due_sample: f64,
        samples_per_tick: f64,
    ) {
        let track = page.tracks[ti as usize];
        if track.paused || track.muted || track.chain_head.is_some() {
            return; // chain members are driven by their head, below
        }

        let is_chain_head = track.chain_member_count > 0;
        let mut base_track = if is_chain_head && matches!(track.chain_base, ChainBase::Individual) {
            let order = chain_play_order(ti, &track);
            let member_idx = (self.track_rt[ti as usize].chain.member_idx as usize) % TRACK_COUNT.max(1);
            page.tracks[order[member_idx.min(order.len() - 1)] as usize]
        } else {
            track
        };
        // Ref: CE v5.30 p.40, "DIR/MCH will default to the Track attribute
        // amount when the sequencer stops" — a live Step Event override, if
        // any, takes precedence over the persisted value while playing.
        // Keyed by `ti` (the track owning this runtime), not whichever track
        // `base_track` resolved to via chain-base-switching above — a chosen
        // simplification for the rare case of both combined at once.
        if let Some(live_dir) = self.track_rt[ti as usize].live_direction_raw {
            base_track.direction_raw = live_dir;
        }
        if let Some(live_mch) = self.track_rt[ti as usize].live_midi_channel {
            base_track.midi_channel = live_mch;
        }

        let hyperstep_link = page.hyperstep_links[ti as usize];
        // Ref: CE v5.30 p.31: "the track is being triggered to play at a speed
        // corresponding to the step absolute length (in 1/192)" — a hyped
        // track's own tick length becomes 192 (full note) rather than the
        // normal per-multiplier length while linked. The source track's own
        // Track LEN scaling this further (p.31-32) is a confirmed-but-not-yet-
        // transcribed refinement — see AMBIGUITIES.md.
        let step_ticks = if hyperstep_link.is_some() {
            192.0
        } else {
            let mult = base_track.multiplier().max(1e-6);
            (DEFAULT_STEP_TICKS as f32 / mult).max(1.0)
        };

        {
            let rt = &mut self.track_rt[ti as usize];
            rt.accum += 1.0;
            if rt.accum + 1e-6 < step_ticks {
                return;
            }
            rt.accum -= step_ticks;
        }

        let raw_index = if is_chain_head { self.track_rt[ti as usize].chain.seg_pos } else { self.track_rt[ti as usize].pos };

        // Ref: CE v5.30 p.56, "Triggers and slices": for a custom direction,
        // `raw_index` is which of the 16 *slices* we're in, not the step
        // itself — the slice's trigger(s) say which physical step(s) actually
        // fire. Slices are always the fixed 16 (Ref p.58's chart), independent
        // of the page's own length.
        let step_index = if let Direction::UserProgrammed(dir_idx) = track.direction() {
            let slice = &page.user_directions[dir_idx as usize].slices[raw_index as usize % STEP_COUNT];
            if slice.trigger_count == 0 {
                // Ref p.56: "every time Octopus gets an empty slice, it will
                // pick a trigger for you at random, and play it normally."
                self.rng.next_below(STEP_COUNT as u32) as u8
            } else {
                let cursor = (self.track_rt[ti as usize].custom_dir_cursor as usize).min(slice.trigger_count as usize - 1);
                slice.triggers[cursor].saturating_sub(1).min(STEP_COUNT as u8 - 1)
            }
        } else {
            ((raw_index as u16 + track.rotation as u16) % page_len as u16) as u8
        };
        let mut step = base_track.steps[(step_index % STEP_COUNT as u8) as usize];

        // Ref: CE v5.30 p.31, "Hyperstep PIT and VEL": "the hypedtrack will
        // assume both the pitch and the velocity values of the hyperstep...
        // in real-time... at the position of the chase light" — read live from
        // the source step every time, not copied once at link creation.
        if let Some(link) = hyperstep_link {
            let source = page.tracks[link.source_track as usize].steps[link.source_step as usize];
            step.pitch_offset = source.pitch_offset;
            step.velocity_offset = source.velocity_offset;
        }

        // Ref: CE v5.30 p.34: "an event is a programmed change of the
        // attributes of a track and is attached to a step" — fires whenever
        // this step is visited by the chase light, independent of whether it
        // also produces a note (Considerations p.40 #1 treats "no note
        // transmitted" and "event fires" as separate concerns).
        if let Some(ev) = step.event {
            self.queue_step_event(ti, ev);
        }

        let shuffle_delay = if step_index % 2 == 1 {
            tables::grv_delay_ticks(base_track.groove, &mut self.rng)
        } else {
            0
        };

        // Feeder contribution: sum offsets currently published by every
        // higher-indexed feeder track. Ref: CE v5.30 p.59-60, "the effect of the
        // feeder tracks is additive down the track indexes" — confirmed by the
        // manual's own worked dry-run example (track 9 PIT +3, track 6 PIT -1,
        // tracks 3+2 PIT -2 each -> track 0 sees +3-1-2-2 = -2). Because we
        // process 9 downto 0, every feeder above `ti` has already run this tick.
        let mut feed_pit = 0i32;
        let mut feed_vel = 0i32;
        let mut feed_len = 0i32;
        for j in (ti as usize + 1)..TRACK_COUNT {
            if page.tracks[j].is_feeder {
                let f = &self.track_rt[j];
                feed_pit += f.feed_pitch as i32;
                feed_vel += f.feed_velocity as i32;
                feed_len += f.feed_length_ticks;
            }
        }

        if step.active && !step.skip {
            let inputs = FireInputs {
                ti,
                page_pitch_offset: page.pitch_offset,
                page_velocity_factor: page.velocity_factor,
                effective_scale,
                base_track,
                step,
                shuffle_delay,
                feed_pit,
                feed_vel,
                feed_len,
                routing_mode,
                fixed_routing,
            };
            self.fire_step(inputs, tick_due_sample, samples_per_tick);
        }

        // Ref: CE v5.30 p.59-60, "Feeders, Listeners and Listening Feeders": "A
        // track may also be both a listener and a feeder... if that track itself
        // is playing notes, then the attributes of those notes are modulated,
        // while the resulting values will modulate the corresponding listeners
        // below it" — a listening feeder forwards its *post-modulation* offset,
        // not its raw step value. For a feeder-only track (not a listener),
        // `feed_pit`/`feed_vel`/`feed_len` are always 0 here (nothing was
        // received), so this is a no-op difference for the common case.
        let (own_pit, own_vel, own_len) = (
            step.pitch_offset as i32,
            step.velocity_offset as i32,
            step.length_ticks as i32 * step.length_multiplier as i32,
        );
        let rt = &mut self.track_rt[ti as usize];
        if track.is_listener {
            rt.feed_pitch = (own_pit + feed_pit).clamp(i8::MIN as i32, i8::MAX as i32) as i8;
            rt.feed_velocity = (own_vel + feed_vel).clamp(i8::MIN as i32, i8::MAX as i32) as i8;
            rt.feed_length_ticks = own_len + feed_len;
        } else {
            rt.feed_pitch = step.pitch_offset;
            rt.feed_velocity = step.velocity_offset;
            rt.feed_length_ticks = own_len;
        }

        // Live DIR shadow (if any) applies to direction-driven advancement
        // too — same `ti`-keyed override as above, via `base_track` (which
        // already has it patched in; equals `track` outside chain-base-switch).
        self.advance_position(ti, base_track.direction(), is_chain_head, page, page_len, page.tracks[ti as usize].chain_member_count);
    }

    fn fire_step(&mut self, inp: FireInputs, tick_due_sample: f64, samples_per_tick: f64) {
        let FireInputs {
            ti,
            page_pitch_offset,
            page_velocity_factor,
            effective_scale,
            base_track,
            step,
            shuffle_delay,
            feed_pit,
            feed_vel,
            feed_len,
            routing_mode,
            fixed_routing,
        } = inp;

        // Only an actual listener is modulated by the effector — a track that
        // is neither a feeder nor a listener sits outside the effector entirely.
        // Not a direct manual quote: inferred from the existence of the Listener
        // Step Mask below, which would have nothing to gate if every track
        // already received modulation unconditionally regardless of role.
        // Flagged as a chosen reading in AMBIGUITIES.md.
        //
        // Ref: CE v5.30 p.63, "Effector Listener Step Mask": "If a Listener (or
        // Listening/Feeder) step has a Step AMT attribute value of -127 then
        // that Listener step will not be influenced by the Feeder."
        let masked = step.amount == -127;
        let (feed_pit, feed_vel, feed_len) = if base_track.is_listener && !masked { (feed_pit, feed_vel, feed_len) } else { (0, 0, 0) };

        let mut final_pit = page_pitch_offset as i32 + base_track.pitch as i32 + step.pitch_offset as i32 + feed_pit;
        if let Some(pcs) = effective_scale {
            final_pit = scale::quantize_to_scale(clamp_midi(final_pit), pcs) as i32;
        }
        let final_pit = clamp_midi(final_pit);

        let vel_raw = clamp_midi(base_track.velocity as i32 + step.velocity_offset as i32 + feed_vel) as i32;
        let vf = page_velocity_factor.max(1) as i32;
        let final_vel = clamp_midi(vel_raw * vf / 8);

        self.last_tick_fires.push(NoteFire { track: ti, pitch: final_pit, velocity: final_vel });

        // Ref: CE v5.30 p.44-45 — real non-linear lookup tables, not a linear
        // 0..2x formula (see tables.rs).
        let sta_ticks = tables::scale_sta_ticks(base_track.start_factor, step.start_offset as i32);

        let base_len = (step.length_ticks as i32).clamp(1, 192);
        let len_mult = (step.length_multiplier as i32).clamp(1, 8);
        let raw_len = (base_len * len_mult + feed_len).min(192);
        let final_len_ticks = tables::scale_len_ticks(base_track.length_factor, raw_len);

        let (port, ch) = resolve_port_channel(base_track.midi_channel, routing_mode, fixed_routing, ti);
        let on_tick_offset = shuffle_delay as i32 + sta_ticks;

        let mut notes: [u8; CHORD_POOL_MAX + 1] = [0; CHORD_POOL_MAX + 1];
        let note_count: usize;
        if step.chord.count > 0 {
            // Ref: CE v5.30 §2 Step Mode, "Random note picks from chord pool", p.22:
            // "the number of played chord notes is always the smaller of the two
            // values (i.e. chord size or polyphony)... When chord size is greater
            // than polyphony, the note pool is made up by all notes that make up
            // the chord[, draw polyphony of them]. When polyphony is greater than
            // the chord size, the note pool is made up by the chord notes plus a
            // number of rests... the difference between the polyphony and the
            // chord size[, draw chord-size of them]." Worked example: chord C-E-G
            // (chord_size=3) — polyphony 3 plays C-E-G every time; polyphony 2
            // draws 2 of {C,E,G} at random; polyphony 5 draws 3 elements at random
            // from {C,E,G,rest,rest} — so even with generous polyphony, a draw can
            // land on a rest and sound fewer than chord_size real notes. The
            // `polyphony == chord_size` case ("consistently play the chord") falls
            // out of this same algorithm for free: a zero-rest pool drawn down to
            // its own size always yields every real note, just in a randomised
            // draw order that `played.sort_unstable()` below erases anyway.
            let chord_size = step.chord.count as usize + 1; // + implicit base pitch
            let poly = (step.chord.polyphony as usize).clamp(1, CHORD_POLYPHONY_MAX);
            let draw_count = poly.min(chord_size);
            let mut pool_offsets = [0i8; CHORD_POOL_MAX + 1];
            for k in 0..step.chord.count as usize {
                pool_offsets[k + 1] = step.chord.offsets[k];
            }

            if poly < chord_size {
                let mut used = [false; CHORD_POOL_MAX + 1];
                let mut n = 0;
                while n < draw_count {
                    let pick = self.rng.next_below(chord_size as u32) as usize;
                    if used[pick] {
                        continue;
                    }
                    used[pick] = true;
                    notes[n] = clamp_midi(final_pit as i32 + pool_offsets[pick] as i32);
                    n += 1;
                }
                note_count = n;
            } else {
                // Padded pool: chord_size real notes + (poly - chord_size) rests,
                // pool size = poly. Draw `draw_count` (== chord_size) without
                // replacement; a draw index >= chord_size is a rest (no note).
                let padded_pool_size = poly;
                let mut used = [false; CHORD_POLYPHONY_MAX];
                let mut drawn = 0;
                let mut sounded = 0;
                while drawn < draw_count {
                    let pick = self.rng.next_below(padded_pool_size as u32) as usize;
                    if used[pick] {
                        continue;
                    }
                    used[pick] = true;
                    drawn += 1;
                    if pick < chord_size {
                        notes[sounded] = clamp_midi(final_pit as i32 + pool_offsets[pick] as i32);
                        sounded += 1;
                    }
                }
                note_count = sounded;
            }
        } else {
            notes[0] = final_pit;
            note_count = 1;
        }

        let played = &mut notes[..note_count];
        played.sort_unstable();
        let strum = step.strum.clamp(-9, 9);
        if strum < 0 {
            played.reverse();
        }
        let level = strum.unsigned_abs();

        // Ref: CE v5.30 p.40, Step Event Consideration 1: "If the Velocity is 0 then the note
        // is not transmitted." So a note that scales down to velocity 0 sends neither a
        // NoteOn (which every receiver reads as a NoteOff) nor a NoteOff.
        if final_vel == 0 {
            // The step still fires: controllers and the effector below are unaffected.
        } else if level > 0 && note_count == 1 {
            let pitch = played[0];
            for k in 1..=7u8 {
                let off = tables::strum_offset_ticks(level, k) as i32;
                let due = tick_due_sample + (on_tick_offset + off) as f64 * samples_per_tick;
                self.schedule_note(due, due + final_len_ticks as f64 * samples_per_tick, port, ch, pitch, final_vel);
            }
        } else {
            for (pi, &pitch) in played.iter().enumerate() {
                let off = tables::strum_offset_ticks(level, (pi + 1) as u8) as i32;
                let due = tick_due_sample + (on_tick_offset + off) as f64 * samples_per_tick;
                self.schedule_note(due, due + final_len_ticks as f64 * samples_per_tick, port, ch, pitch, final_vel);
            }
        }

        // Ref: CE v5.30 p.16, "Step phrasing (GRV)": "A Step may be enriched
        // at playtime by a certain amount of notes determined by phrases."
        // p.27: phrase #0 "has no effect, i.e. plays the step as is" — so the
        // step's own note (already scheduled above) always fires, and a
        // selected phrase adds extras. Phrase PIT/VEL/LEN/STA are offsets on
        // top of the step's already-resolved values. Phrase POS compression
        // (p.17 / p.30) is not applied yet — extras use raw `start_ticks`.
        // Chord + phrase together: extras are scheduled from `final_pit`
        // only, not per chord tone (chosen; p.23 warns GRV "will not function
        // as expected" on random-rest steps, and says nothing about chords).
        if let Some(phrase_idx_1based) = step.phrase {
            if phrase_idx_1based > 0 {
                let idx = phrase_idx_1based.saturating_sub(1);
                let (phrase_notes, poly) = self.resolve_phrase(idx);
                let mut fired = 0usize;
                for note in phrase_notes {
                    if !note.enabled {
                        continue;
                    }
                    if fired >= poly {
                        break;
                    }
                    fired += 1;
                    let extra_pit = clamp_midi(final_pit as i32 + note.pitch_offset as i32);
                    let extra_pit = if let Some(pcs) = effective_scale {
                        scale::quantize_to_scale(extra_pit, pcs)
                    } else {
                        extra_pit
                    };
                    let extra_vel = clamp_midi(final_vel as i32 + note.velocity_offset as i32);
                    let extra_len = (final_len_ticks as i32 + note.length_ticks as i32).clamp(1, 192 * 8);
                    // Ref: CE v5.30 p.17 / p.30 — phrase POS time-compresses
                    // the extras' STA. Neutral 8 is identity.
                    let extra_start = on_tick_offset + tables::scale_phrase_sta(note.start_ticks, step.phrase_pos) as i32;
                    self.last_tick_fires.push(NoteFire { track: ti, pitch: extra_pit, velocity: extra_vel });
                    let due = tick_due_sample + extra_start as f64 * samples_per_tick;
                    if extra_vel > 0 {
                        self.schedule_note(due, due + extra_len as f64 * samples_per_tick, port, ch, extra_pit, extra_vel);
                    }
                }
            }
        }

        if let Some(step_mcc) = step.mcc_value {
            let due = tick_due_sample + on_tick_offset as f64 * samples_per_tick;
            let v = clamp_midi(step_mcc as i32);
            // Ref: CE v5.30 p.46 and p.91. The bender and channel pressure flags make the
            // track send those messages "according to the MCC values stored in that track's
            // steps". The editor holds only the top 7 bits of a bend value, the rest clear.
            match base_track.mcc {
                Mcc::Cc(cc) => self.schedule(due, RawEvent::Cc { port, ch, cc, val: v }),
                Mcc::Bend => self.schedule(due, RawEvent::PitchBend { port, ch, value: (v as u16) << 7 }),
                Mcc::Pressure => self.schedule(due, RawEvent::ChannelPressure { port, ch, value: v }),
                Mcc::None => {}
            }
        }
    }

    fn advance_position(&mut self, ti: TrackIndex, dir: Direction, is_chain_head: bool, page: &Page, page_len: u8, chain_member_count: u8) {
        if let Direction::UserProgrammed(dir_idx) = dir {
            self.advance_custom_direction(ti, dir_idx, is_chain_head, page, chain_member_count);
            return;
        }

        let rt = &mut self.track_rt[ti as usize];
        let (pos, ping) = if is_chain_head {
            (&mut rt.chain.seg_pos, &mut rt.chain.ping_dir)
        } else {
            (&mut rt.pos, &mut rt.ping_dir)
        };
        let len = page_len.max(1);

        match dir {
            Direction::Forward => *pos = (*pos + 1) % len,
            Direction::Reverse => *pos = (*pos + len - 1) % len,
            Direction::PingPong => {
                if *pos == 0 {
                    *ping = 1;
                } else if *pos >= len - 1 {
                    *ping = -1;
                }
                *pos = (*pos as i16 + *ping as i16).clamp(0, len as i16 - 1) as u8;
            }
            Direction::Brownian => {
                let forward = self.rng.next_f32() < 0.6667;
                *pos = if forward { (*pos + 1) % len } else { (*pos + len - 1) % len };
            }
            Direction::Random => *pos = self.rng.next_below(len as u32) as u8,
            Direction::UserProgrammed(_) => unreachable!("handled above"),
        }

        if is_chain_head && rt.chain.seg_pos == 0 {
            let chain_len = 1 + chain_member_count as usize;
            rt.chain.member_idx = ((rt.chain.member_idx as usize + 1) % chain_len.max(1)) as u8;
        }
    }

    /// Ref: CE v5.30 p.56, "Certainty_next": "A setting of 100% means that the
    /// next slice will be the one following naturally... A setting of 0% will
    /// specify that the next slice will be the naturally previous one...
    /// [values in between produce] a 50/50 chance." And "Full and empty
    /// slices": a slice's triggers "will be played in sequence every time the
    /// respective slice is being played" — confirmed by
    /// `reference/manual/pages/tutorial-03.txt`'s "Step double-play" worked
    /// example (two triggers in one slice = that step plays twice, and the
    /// track ends up "4 steps behind" after 4 doubled steps — only consistent
    /// with every extra trigger consuming its own tick-boundary visit, not
    /// firing simultaneously). So: fire the current trigger, advance the
    /// cursor; only once every trigger in the slice has fired does the slice
    /// itself move on, via the certainty_next coin flip.
    fn advance_custom_direction(&mut self, ti: TrackIndex, dir_idx: u8, is_chain_head: bool, page: &Page, chain_member_count: u8) {
        let slice_len = STEP_COUNT as u8;
        let slice = page.user_directions[dir_idx as usize].slices[if is_chain_head {
            self.track_rt[ti as usize].chain.seg_pos
        } else {
            self.track_rt[ti as usize].pos
        } as usize
            % STEP_COUNT];
        // An empty slice behaves as a single (random) trigger for this visit
        // — see the step-lookup side in `step_one_track` — so it also
        // completes in one visit here.
        let effective_trigger_count = slice.trigger_count.max(1);

        let more_triggers_remain = {
            let rt = &mut self.track_rt[ti as usize];
            rt.custom_dir_cursor += 1;
            rt.custom_dir_cursor < effective_trigger_count
        };
        if more_triggers_remain {
            return; // same slice, next trigger, no slice-advance this visit
        }

        let rt = &mut self.track_rt[ti as usize];
        rt.custom_dir_cursor = 0;
        let forward = self.rng.next_below(100) < slice.certainty_next as u32;
        let pos = if is_chain_head { &mut rt.chain.seg_pos } else { &mut rt.pos };
        *pos = if forward { (*pos + 1) % slice_len } else { (*pos + slice_len - 1) % slice_len };

        if is_chain_head && rt.chain.seg_pos == 0 {
            let chain_len = 1 + chain_member_count as usize;
            rt.chain.member_idx = ((rt.chain.member_idx as usize + 1) % chain_len.max(1)) as u8;
        }
    }

    /// Ref: CE v5.30 p.72/p.85, "Exempting pages from the grid scale": "the
    /// GRID scale is overruled by any other scales active in particular
    /// pages." Page scale, if enabled, always wins; see AMBIGUITIES.md "scale
    /// grid/page combination" for the full derivation.
    fn effective_scale(&self, page: &Page) -> Option<scale::PitchClassSet> {
        if page.scale.enabled {
            Some(scale::build_scale_pitch_classes(page.scale.root as i32, &to_i32(page.scale.intervals())))
        } else if self.grid.global_scale.enabled {
            Some(scale::build_scale_pitch_classes(
                self.grid.global_scale.root as i32,
                &to_i32(self.grid.global_scale.intervals()),
            ))
        } else {
            None
        }
    }

    fn apply_deferred_action(&mut self, target: TrackIndex, action: DeferredAction) {
        match action {
            DeferredAction::AddDir(delta) => {
                let persisted = self.grid.active_page().tracks[target as usize].direction_raw;
                let base = self.track_rt[target as usize].live_direction_raw.unwrap_or(persisted);
                self.track_rt[target as usize].live_direction_raw = Some(wrap_1_based(base, delta, 16));
            }
            DeferredAction::AddMch(delta) => {
                let persisted = self.grid.active_page().tracks[target as usize].midi_channel;
                let base = self.track_rt[target as usize].live_midi_channel.unwrap_or(persisted);
                self.track_rt[target as usize].live_midi_channel = Some(wrap_1_based(base, delta, 32));
            }
            DeferredAction::AddPos(delta) => {
                // Ref p.40: "POS does not restore to the start POS(ition) when
                // stopping the sequencer" — a direct, persistent mutation, no
                // live-shadow/revert needed (unlike DIR/MCH). No wrap maximum
                // is given for POS in the manual; plain wrapping u8 arithmetic
                // is harmless since this only ever feeds a `% page_len`
                // downstream — see AMBIGUITIES.md.
                let track = &mut self.grid.active_page_mut().tracks[target as usize];
                track.rotation = track.rotation.wrapping_add_signed(delta);
            }
            DeferredAction::ToggleOn(kind) | DeferredAction::ToggleOff(kind) => {
                let on = matches!(action, DeferredAction::ToggleOn(_));
                let track = &mut self.grid.active_page_mut().tracks[target as usize];
                match kind {
                    ToggleKind::Mute => track.muted = on,
                    ToggleKind::Solo => track.soloed = on,
                    ToggleKind::Record => track.record_armed = on,
                    ToggleKind::Pause => track.paused = on,
                }
                // On-The-Measure gating for Mute/Solo happens earlier, in
                // queue_step_event, by routing into measure_deferred instead
                // of deferred_actions — by the time this runs, that decision
                // has already been made.
            }
            DeferredAction::Rotate(amt) => {
                self.grid.active_page_mut().tracks[target as usize].rotate_steps(amt);
            }
            DeferredAction::SkipRotate(amt) => {
                self.grid.active_page_mut().tracks[target as usize].rotate_skips(amt);
            }
        }
    }

    /// Resolves a step event fired this tick (by `source` track) into zero or
    /// more `DeferredAction`s and queues each for application at the start of
    /// the next tick. `Ref: CE v5.30 p.34-40` — see `deferred_actions`' doc
    /// comment for why every target defers uniformly rather than the manual's
    /// same-tick/next-tick split.
    fn queue_step_event(&mut self, source: TrackIndex, ev: StepEvent) {
        match ev.kind {
            StepEventKind::SetDir(delta) => self.deferred_actions[ev.target_track as usize] = Some(DeferredAction::AddDir(delta)),
            StepEventKind::SetPos(delta) => self.deferred_actions[ev.target_track as usize] = Some(DeferredAction::AddPos(delta)),
            StepEventKind::SetMch(delta) => self.deferred_actions[ev.target_track as usize] = Some(DeferredAction::AddMch(delta)),
            StepEventKind::TrackToggle { kind, amt, range } => {
                // Ref p.39: |amt| selects the target track (10 means track 0,
                // the one value that can't otherwise be reached since 0 itself
                // means "no target"); sign selects On(+)/Off(-); range (max
                // 10) selects how many consecutive tracks are affected,
                // descending from the target and wrapping from 0 back to 9.
                if amt == 0 {
                    return;
                }
                let target0 = if amt.unsigned_abs() == 10 { 0 } else { (amt.unsigned_abs() as usize) % 10 };
                let on = amt > 0;
                let range = range.clamp(1, TRACK_COUNT as u8);
                // Ref p.40, Track Toggle Consideration 5: "Track Toggle Events
                // of Mute & Solo are subordinate to the On-The-Measure mode
                // condition of the Page." Record/Pause aren't mentioned and
                // stay on the normal next-tick queue.
                let on_the_measure = matches!(kind, ToggleKind::Mute | ToggleKind::Solo) && self.grid.active_page().on_the_measure;
                let action = if on { DeferredAction::ToggleOn(kind) } else { DeferredAction::ToggleOff(kind) };
                for i in 0..range as i32 {
                    let t = (target0 as i32 - i).rem_euclid(TRACK_COUNT as i32) as usize;
                    if on_the_measure {
                        self.measure_deferred[t] = Some(action);
                    } else {
                        self.deferred_actions[t] = Some(action);
                    }
                }
            }
            // Ref p.38-39: applies to the event's own track. AMT is
            // direction/distance (p.39: positive = forward, negative =
            // backward), not a target-track index like Mute/Solo.
            StepEventKind::TrackRotate { amt } => {
                if amt != 0 {
                    self.deferred_actions[source as usize] = Some(DeferredAction::Rotate(amt));
                }
            }
            StepEventKind::TrackSkipRotate { amt } => {
                if amt != 0 {
                    self.deferred_actions[source as usize] = Some(DeferredAction::SkipRotate(amt));
                }
            }
        }
    }

    /// Ref: CE v5.30 p.31: "hold a track selector down, and at the same time
    /// designate the hyperstep in the matrix, in a row other than the track's."
    /// Creates or replaces the hyperstep link on `hyped_track` (each hyped
    /// track holds at most one, per the manual — assigning a new one replaces
    /// any existing link on that slot). `source_step`'s own step LEN is set to
    /// 192/192 ("On making a step a hyperstep its LEN is automatically set to
    /// 192/192") and its `hyperstep` flag is raised. Real creation is still
    /// gated on a `ControlId` -> (track, step) mapping this crate doesn't have
    /// yet (see reference/NOTES.md) — this is the seam a future input layer
    /// calls once that mapping exists.
    pub fn hyperstep_link(&mut self, source_track: TrackIndex, source_step: u8, hyped_track: TrackIndex) {
        let page = self.grid.active_page_mut();
        let src = &mut page.tracks[source_track as usize].steps[source_step as usize];
        src.hyperstep = true;
        src.length_ticks = 192;
        src.length_multiplier = 1;
        page.hyperstep_links[hyped_track as usize] = Some(HyperstepLink { source_track, source_step });
    }

    /// Ref: CE v5.30 p.31, "Destroying hypersteps": "hold the respective track
    /// selector of the hyped track pressed and then press any step button in
    /// the matrix row of the hyped track. The association... will be removed."
    /// The freed source step's LEN resets "to the default value of 12/192".
    pub fn hyperstep_unlink(&mut self, hyped_track: TrackIndex) {
        let page = self.grid.active_page_mut();
        if let Some(link) = page.hyperstep_links[hyped_track as usize].take() {
            let src = &mut page.tracks[link.source_track as usize].steps[link.source_step as usize];
            src.hyperstep = false;
            src.length_ticks = DEFAULT_STEP_TICKS as u8;
            src.length_multiplier = 1;
        }
    }

    /// Ref: CE v5.30 p.23, "The available phrase types are as follows: Type 1:
    /// Forward: notes are played in the order 1,2,3.. Type 2: Reverse: notes
    /// are played in the order 8,7,6.. Type 3: Random pitch[:] programmed
    /// notes pitches are played in random order, determined at playtime. Type
    /// 4: Random all: programmed note attributes played in random
    /// combinations, determined at playtime."
    ///
    /// `RandomPitch`/`RandomAll` shuffle the *programmed* note data — they
    /// never invent new offset values, only recombine what's already there.
    pub fn resolve_phrase(&mut self, phrase_index: u8) -> ([PhraseNote; PHRASE_NOTE_COUNT], usize) {
        let mut phrase = self.grid.phrases[(phrase_index as usize).min(PHRASE_COUNT - 1)];
        match phrase.phrase_type {
            PhraseType::Forward => {}
            PhraseType::Reverse => phrase.notes.reverse(),
            PhraseType::RandomPitch => {
                let mut pitches: [i8; PHRASE_NOTE_COUNT] = phrase.notes.map(|n| n.pitch_offset);
                self.shuffle_i8(&mut pitches);
                for (note, &p) in phrase.notes.iter_mut().zip(pitches.iter()) {
                    note.pitch_offset = p;
                }
            }
            PhraseType::RandomAll => {
                let originals = phrase.notes;
                let mut order: [usize; PHRASE_NOTE_COUNT] = std::array::from_fn(|i| i);
                self.shuffle_usize(&mut order);
                for (slot, &src) in phrase.notes.iter_mut().zip(order.iter()) {
                    *slot = originals[src];
                }
            }
        }
        let enabled_count = phrase.notes.iter().filter(|n| n.enabled).count().max(1);
        let poly = (phrase.polyphony as usize).clamp(1, enabled_count);
        (phrase.notes, poly)
    }

    fn shuffle_i8(&mut self, arr: &mut [i8; PHRASE_NOTE_COUNT]) {
        for i in (1..arr.len()).rev() {
            let j = self.rng.next_below((i + 1) as u32) as usize;
            arr.swap(i, j);
        }
    }

    fn shuffle_usize(&mut self, arr: &mut [usize; PHRASE_NOTE_COUNT]) {
        for i in (1..arr.len()).rev() {
            let j = self.rng.next_below((i + 1) as u32) as usize;
            arr.swap(i, j);
        }
    }
}

fn to_i32(v: &[i8]) -> [i32; MAX_SCALE_INTERVALS] {
    let mut out = [0i32; MAX_SCALE_INTERVALS];
    for (i, &x) in v.iter().enumerate().take(MAX_SCALE_INTERVALS) {
        out[i] = x as i32;
    }
    out
}

/// Adds `delta` to a 1-based value in `1..=max`, wrapping around. E.g. for
/// `max=16`: 16 + 2 -> 2 (not 18 or 0), matching "if DIR is 6 a Step Event of
/// '+2' will change it to 8" style arithmetic without ever landing on 0.
fn wrap_1_based(value: u8, delta: i8, max: u8) -> u8 {
    let zero_based = (value as i32 - 1 + delta as i32).rem_euclid(max as i32);
    (zero_based + 1) as u8
}

fn resolve_port_channel(mch: u8, mode: RoutingMode, fixed: FixedRouting, track_index: TrackIndex) -> (u8, u8) {
    if mode == RoutingMode::Fixed {
        let ch = ((fixed.base_channel_port1 as u16 - 1 + track_index as u16) % 16 + 1) as u8;
        return (1, ch);
    }
    let v = mch.max(1);
    if v <= 16 {
        (1, v.clamp(1, 16))
    } else {
        (2, (v - 16).clamp(1, 16))
    }
}

/// Hardware default play order for a chain is top-to-bottom (row 9 -> row 0).
fn chain_play_order(ti: TrackIndex, track: &Track) -> [TrackIndex; TRACK_COUNT] {
    let mut order = [ti; TRACK_COUNT];
    let mut n = 1usize;
    for m in 0..track.chain_member_count as usize {
        if let Some(idx) = track.chain_members[m] {
            if !order[..n].contains(&idx) {
                order[n] = idx;
                n += 1;
            }
        }
    }
    order[..n].sort_unstable_by(|a, b| b.cmp(a));
    let fill = order[n - 1];
    for slot in order.iter_mut().skip(n) {
        *slot = fill;
    }
    order
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Event;

    fn note_ons(events: &[Event]) -> Vec<(u8, u8, u8, u8)> {
        events
            .iter()
            .filter_map(|e| match *e {
                Event::NoteOn { port, ch, note, vel, .. } => Some((port, ch, note, vel)),
                _ => None,
            })
            .collect()
    }

    /// A basic 120bpm/48kHz render over one full bar (16 steps) should fire
    /// track 0's default-active-nothing grid... so first enable one step, then
    /// confirm `render()` (the real sample-accurate path, not the fixture's
    /// tick-only bypass) actually emits it with a sane `at_sample`.
    #[test]
    fn render_emits_note_with_at_sample_in_range() {
        let mut engine = Engine::new(1);
        engine.grid.active_page_mut().tracks[0].steps[0].active = true;

        let ctx = RenderContext { sample_rate: 48_000.0, buffer_len: 512, bpm: 120.0, playing: true };
        let mut all = Vec::new();
        // 120bpm/192ppqn/48kHz => ~125 samples/tick => a step (12 ticks) fires
        // roughly every 1500 samples, so a few dozen buffers must be rendered to
        // be sure to cross a step boundary.
        for _ in 0..40 {
            let mut out = EventBuffer::new();
            engine.render(&ctx, &mut out);
            for e in out.as_slice() {
                assert!(matches!(e, Event::NoteOn{at_sample,..}|Event::NoteOff{at_sample,..}|Event::Cc{at_sample,..} if *at_sample < ctx.buffer_len));
            }
            all.extend_from_slice(out.as_slice());
        }
        // Track::DEFAULT_PITCH[0] = 69 (A5) per CE v5.30 p.43 — 60 (C5) is track
        // index 4's default.
        assert!(note_ons(&all).iter().any(|&(_, _, note, _)| note == 69), "expected track 0's default pitch (69) to fire; got {:?}", note_ons(&all));
    }

    #[test]
    fn forward_direction_advances_one_step() {
        let mut engine = Engine::new(1);
        engine.grid.active_page_mut().tracks[0].direction_raw = 1; // forward
        for _ in 0..DEFAULT_STEP_TICKS {
            engine.step_once_for_test();
        }
        assert_eq!(engine.track_rt[0].pos, 1);
    }

    #[test]
    fn reverse_direction_wraps_backward() {
        let mut engine = Engine::new(1);
        engine.grid.active_page_mut().tracks[0].direction_raw = 2; // reverse
        engine.grid.active_page_mut().length = 16;
        for _ in 0..DEFAULT_STEP_TICKS {
            engine.step_once_for_test();
        }
        assert_eq!(engine.track_rt[0].pos, 15);
    }

    #[test]
    fn ping_pong_bounces_at_upper_edge() {
        let mut engine = Engine::new(1);
        let page = engine.grid.active_page_mut();
        page.length = 4;
        page.tracks[0].direction_raw = 3; // ping-pong
        engine.track_rt[0].pos = 3; // sit at the top edge before the boundary
        for _ in 0..DEFAULT_STEP_TICKS {
            engine.step_once_for_test();
        }
        assert_eq!(engine.track_rt[0].pos, 2, "should have turned around at the upper edge");
    }

    #[test]
    fn chain_play_order_is_descending_head_first() {
        let mut track = Track::default_for_index(9);
        track.chain_members[0] = Some(5);
        track.chain_members[1] = Some(3);
        track.chain_member_count = 2;
        let order = chain_play_order(9, &track);
        assert_eq!(&order[..3], &[9, 5, 3]);
    }

    #[test]
    fn chain_member_never_plays_independently() {
        // A track with `chain_head` set is a member, not a head; docs §3: "a
        // chained track has a head and members... the head owns the runtime
        // cursor." A member's own steps must never fire on their own.
        let mut engine = Engine::new(1);
        let page = engine.grid.active_page_mut();
        page.tracks[5].chain_head = Some(9);
        page.tracks[5].steps[0].active = true;
        page.tracks[5].pitch = 100; // distinctive, would be unmistakable if it fired

        let ctx = RenderContext { sample_rate: 48_000.0, buffer_len: 2048, bpm: 120.0, playing: true };
        let mut all = Vec::new();
        for _ in 0..4 {
            let mut out = EventBuffer::new();
            engine.render(&ctx, &mut out);
            all.extend_from_slice(out.as_slice());
        }
        assert!(note_ons(&all).iter().all(|&(_, _, note, _)| note != 100), "a chain member fired on its own: {:?}", note_ons(&all));
    }

    #[test]
    fn chord_full_polyphony_plays_every_note() {
        let mut engine = Engine::new(7);
        let page = engine.grid.active_page_mut();
        page.tracks[0].steps[0].active = true;
        page.tracks[0].steps[0].chord.offsets[0] = 4;
        page.tracks[0].steps[0].chord.offsets[1] = 7;
        page.tracks[0].steps[0].chord.count = 2;
        page.tracks[0].steps[0].chord.polyphony = 3; // base + 2 offsets, all of them

        let ctx = RenderContext { sample_rate: 48_000.0, buffer_len: 2048, bpm: 120.0, playing: true };
        let mut all = Vec::new();
        for _ in 0..4 {
            let mut out = EventBuffer::new();
            engine.render(&ctx, &mut out);
            all.extend_from_slice(out.as_slice());
        }
        // Track 0's default pitch is 69 (see DEFAULT_PITCH, CE v5.30 p.43), so
        // the triad is 69 / 69+4 / 69+7.
        let pitches: std::collections::BTreeSet<_> = note_ons(&all).into_iter().map(|(_, _, n, _)| n).collect();
        assert!(pitches.contains(&69) && pitches.contains(&73) && pitches.contains(&76), "expected a full 69/73/76 triad, got {:?}", pitches);
    }

    /// Ref: CE v5.30 p.22: "A polyphony of 5 will play 3 elements picked at
    /// random from the pool {C, E, G, rest, rest}" (chord_size=3 there; here
    /// chord_size=2, polyphony=5, so the pool is {base, offset, rest, rest, rest}
    /// and 2 elements are drawn). Proves draws can land on rests (note_count < 2
    /// on some seeds) while never exceeding chord_size (note_count never > 2) —
    /// the bug this replaced always played both real notes deterministically.
    #[test]
    fn chord_polyphony_over_chord_size_can_draw_rests() {
        let mut saw_full = false;
        let mut saw_partial = false;
        for seed in 0..200u64 {
            let mut engine = Engine::new(seed);
            let page = engine.grid.active_page_mut();
            page.tracks[0].steps[0].active = true;
            page.tracks[0].steps[0].chord.offsets[0] = 7;
            page.tracks[0].steps[0].chord.count = 1; // chord_size = 2 (base + 1 offset)
            page.tracks[0].steps[0].chord.polyphony = 5; // 3 rest placeholders

            let ctx = RenderContext { sample_rate: 48_000.0, buffer_len: 1600, bpm: 120.0, playing: true };
            let mut out = EventBuffer::new();
            engine.render(&ctx, &mut out);
            let count = note_ons(out.as_slice()).len();
            assert!(count <= 2, "seed {seed}: drew more notes than chord_size allows: {count}");
            if count == 2 {
                saw_full = true;
            } else {
                saw_partial = true;
            }
        }
        assert!(saw_full, "expected at least one seed to draw both real notes");
        assert!(saw_partial, "expected at least one seed to draw a rest and sound fewer than chord_size notes");
    }

    #[test]
    fn scale_quantization_pulls_pitch_into_scale() {
        let mut engine = Engine::new(3);
        let page = engine.grid.active_page_mut();
        page.scale.enabled = true;
        page.scale.root = 60;
        page.scale.intervals = {
            let mut a = [0i8; MAX_SCALE_INTERVALS];
            a[..7].copy_from_slice(&[0, 2, 4, 5, 7, 9, 11]);
            a
        };
        page.scale.interval_count = 7;
        page.tracks[0].pitch = 61; // C#, not in C major
        page.tracks[0].steps[0].active = true;

        let ctx = RenderContext { sample_rate: 48_000.0, buffer_len: 2048, bpm: 120.0, playing: true };
        let mut all = Vec::new();
        for _ in 0..4 {
            let mut out = EventBuffer::new();
            engine.render(&ctx, &mut out);
            all.extend_from_slice(out.as_slice());
        }
        let pitches: Vec<_> = note_ons(&all).into_iter().map(|(_, _, n, _)| n).collect();
        assert!(!pitches.is_empty());
        assert!(pitches.iter().all(|&p| p != 61), "61 is out of C major and should have been quantized away: {:?}", pitches);
        assert!(pitches.iter().all(|&p| p == 60), "expected quantization down to 60 (downward tie-break): {:?}", pitches);
    }

    fn test_phrase() -> Phrase {
        let mut phrase = Phrase::default();
        for (i, note) in phrase.notes.iter_mut().enumerate() {
            note.pitch_offset = i as i8;
            note.velocity_offset = (i * 2) as i8;
            note.enabled = true;
        }
        phrase
    }

    #[test]
    fn phrase_forward_plays_programmed_order() {
        let mut engine = Engine::new(1);
        engine.grid.phrases[0] = test_phrase();
        engine.grid.phrases[0].phrase_type = PhraseType::Forward;
        let (notes, _) = engine.resolve_phrase(0);
        for (i, note) in notes.iter().enumerate() {
            assert_eq!(note.pitch_offset, i as i8);
        }
    }

    #[test]
    fn phrase_reverse_plays_8_to_1() {
        let mut engine = Engine::new(1);
        engine.grid.phrases[0] = test_phrase();
        engine.grid.phrases[0].phrase_type = PhraseType::Reverse;
        let (notes, _) = engine.resolve_phrase(0);
        for (i, note) in notes.iter().enumerate() {
            assert_eq!(note.pitch_offset, (PHRASE_NOTE_COUNT - 1 - i) as i8);
        }
    }

    #[test]
    fn phrase_random_pitch_permutes_pitch_only() {
        let mut engine = Engine::new(3);
        engine.grid.phrases[0] = test_phrase();
        engine.grid.phrases[0].phrase_type = PhraseType::RandomPitch;
        let (notes, _) = engine.resolve_phrase(0);

        let mut pitches: Vec<i8> = notes.iter().map(|n| n.pitch_offset).collect();
        pitches.sort_unstable();
        assert_eq!(pitches, (0..PHRASE_NOTE_COUNT as i8).collect::<Vec<_>>(), "pitch set must be a permutation of the originals, not new values");
        // Velocity must stay with its original slot (only pitch moves).
        for (i, note) in notes.iter().enumerate() {
            assert_eq!(note.velocity_offset, (i * 2) as i8);
        }
    }

    #[test]
    fn phrase_random_all_permutes_whole_notes() {
        let mut engine = Engine::new(5);
        engine.grid.phrases[0] = test_phrase();
        engine.grid.phrases[0].phrase_type = PhraseType::RandomAll;
        let (notes, _) = engine.resolve_phrase(0);

        // Each (pitch, vel) pair must still be one of the originals, and the
        // pair must move together (vel = pitch*2 in the fixture data).
        for note in notes.iter() {
            assert_eq!(note.velocity_offset, note.pitch_offset * 2, "attributes must move together, not be freshly randomised");
        }
        let mut pitches: Vec<i8> = notes.iter().map(|n| n.pitch_offset).collect();
        pitches.sort_unstable();
        assert_eq!(pitches, (0..PHRASE_NOTE_COUNT as i8).collect::<Vec<_>>());
    }

    #[test]
    fn phrase_count_covers_48_across_three_banks() {
        let engine = Engine::new(1);
        assert_eq!(engine.grid.phrases.len(), 48);
    }

    /// Ref: CE v5.30 p.31: "the track is being triggered to play at a speed
    /// corresponding to the step absolute length (in 1/192)" — a linked hyped
    /// track fires once per 192 ticks, not once per the normal 12.
    #[test]
    fn hyperstep_linked_track_fires_every_192_ticks_not_12() {
        let mut engine = Engine::new(1);
        engine.grid.active_page_mut().tracks[9].steps[0].active = true;
        engine.grid.active_page_mut().tracks[5].steps[0].active = true;
        engine.hyperstep_link(9, 0, 5);

        let mut fires_in_first_12 = 0;
        for _ in 0..12 {
            engine.step_once_for_test();
            fires_in_first_12 += engine.last_tick_fires.as_slice().iter().filter(|f| f.track == 5).count();
        }
        assert_eq!(fires_in_first_12, 0, "should not fire at the normal 12-tick boundary while hyperstep-linked");

        for _ in 0..180 {
            engine.step_once_for_test();
        }
        let fired_at_192 = engine.last_tick_fires.as_slice().iter().any(|f| f.track == 5);
        assert!(fired_at_192, "should fire at tick 192 (12 + 180)");
    }

    /// Ref: CE v5.30 p.31: "the hypedtrack will assume both the pitch and the
    /// velocity values of the hyperstep... Changes to the hyperstep PIT and
    /// VEL will influence the hypedtrack in real-time" — read live from the
    /// source step, not copied once at link creation.
    #[test]
    fn hyperstep_pit_vel_are_read_live_from_source() {
        let mut engine = Engine::new(1);
        engine.grid.active_page_mut().tracks[9].steps[0].active = true;
        engine.grid.active_page_mut().tracks[9].steps[0].pitch_offset = 5;
        engine.grid.active_page_mut().tracks[5].pitch = 60;
        // Every step active (not just [0]) since the hyped track's own chase
        // light keeps advancing across the two 192-tick windows below.
        for step in engine.grid.active_page_mut().tracks[5].steps.iter_mut() {
            step.active = true;
            step.pitch_offset = 99; // must be ignored regardless of which step fires
        }
        engine.hyperstep_link(9, 0, 5);

        for _ in 0..192 {
            engine.step_once_for_test();
        }
        let first = engine.last_tick_fires.as_slice().iter().find(|f| f.track == 5).map(|f| f.pitch);
        assert_eq!(first, Some(65), "expected track 5's base (60) + source's live pitch offset (5)");

        // Live edit to the source step, no re-linking.
        engine.grid.active_page_mut().tracks[9].steps[0].pitch_offset = 8;
        for _ in 0..192 {
            engine.step_once_for_test();
        }
        let second = engine.last_tick_fires.as_slice().iter().find(|f| f.track == 5).map(|f| f.pitch);
        assert_eq!(second, Some(68), "expected the updated live offset (8), proving no value was cached at link time");
    }

    #[test]
    fn hyperstep_unlink_restores_default_step_length() {
        let mut engine = Engine::new(1);
        engine.hyperstep_link(9, 0, 5);
        assert_eq!(engine.grid.active_page().tracks[9].steps[0].length_ticks, 192);
        engine.hyperstep_unlink(5);
        assert_eq!(engine.grid.active_page().tracks[9].steps[0].length_ticks, DEFAULT_STEP_TICKS as u8);
        assert!(engine.grid.active_page().hyperstep_links[5].is_none());
    }

    fn fire_step_event(engine: &mut Engine, source_track: TrackIndex, kind: StepEventKind, target_track: TrackIndex) {
        engine.grid.active_page_mut().tracks[source_track as usize].steps[0].active = true;
        engine.grid.active_page_mut().tracks[source_track as usize].steps[0].event = Some(StepEvent { target_track, kind });
        for _ in 0..DEFAULT_STEP_TICKS {
            engine.step_once_for_test();
        }
    }

    /// Ref: CE v5.30 p.34: "if DIR is 6 a Step Event of '+2' will change it to
    /// 8" — additive, not an overwrite, applied via a live shadow (the
    /// *persisted* `Track::direction_raw` never changes — see p.40, "DIR will
    /// default to the Track attribute amount when the sequencer stops", which
    /// only makes sense if there's a separate live value to fall away from).
    /// Applied the tick *after* the event fires (this engine's uniform
    /// next-tick simplification — see AMBIGUITIES.md).
    #[test]
    fn step_event_set_dir_is_additive_and_wraps() {
        let mut engine = Engine::new(1);
        engine.grid.active_page_mut().tracks[3].direction_raw = 6;
        fire_step_event(&mut engine, 9, StepEventKind::SetDir(2), 3);
        // Queued but not yet applied within the tick it fired.
        assert_eq!(engine.track_rt[3].live_direction_raw, None);
        engine.step_once_for_test();
        assert_eq!(engine.track_rt[3].live_direction_raw, Some(8));
        assert_eq!(engine.grid.active_page().tracks[3].direction_raw, 6, "persisted value is never touched by a step event");

        // Wrap: 16 + 2 -> 2, never 18 or 0. Fresh engine to avoid the first
        // half's queued/applied timing interacting with this one.
        let mut engine = Engine::new(1);
        engine.grid.active_page_mut().tracks[3].direction_raw = 16;
        fire_step_event(&mut engine, 9, StepEventKind::SetDir(2), 3);
        engine.step_once_for_test();
        assert_eq!(engine.track_rt[3].live_direction_raw, Some(2));
    }

    /// Ref: CE v5.30 p.40: "DIR will default to the Track attribute amount
    /// when the sequencer stops."
    #[test]
    fn step_event_set_dir_reverts_on_stop() {
        let mut engine = Engine::new(1);
        engine.set_running(true); // set_running(false) below is only a real transition if this was true first
        engine.grid.active_page_mut().tracks[3].direction_raw = 6;
        fire_step_event(&mut engine, 9, StepEventKind::SetDir(2), 3);
        engine.step_once_for_test();
        assert_eq!(engine.track_rt[3].live_direction_raw, Some(8));
        engine.set_running(false);
        assert!(engine.track_rt[3].live_direction_raw.is_none(), "stopping must clear the live override");
        assert_eq!(engine.grid.active_page().tracks[3].direction_raw, 6, "persisted value was never touched, so there's nothing to restore");
    }

    /// Ref: CE v5.30 p.39: manual's own worked example: "a Track Toggle Mute
    /// Event has an AMT Value of +1 & Range value of 4, then Tracks 1 & 0 and
    /// Tracks 9 & 8 will be Muted."
    #[test]
    fn track_toggle_range_wraps_manual_worked_example() {
        let mut engine = Engine::new(1);
        fire_step_event(&mut engine, 5, StepEventKind::TrackToggle { kind: ToggleKind::Mute, amt: 1, range: 4 }, 0 /* unused for toggles */);
        engine.step_once_for_test();
        let muted: Vec<u8> = (0..TRACK_COUNT as u8).filter(|&t| engine.grid.active_page().tracks[t as usize].muted).collect();
        assert_eq!(muted, vec![0, 1, 8, 9]);
    }

    /// Ref: CE v5.30 p.39: "An AMT value of zero is an 'off' value therefore
    /// to apply a Track Toggle Event specifically to... Track 0, use AMT
    /// value = '10'."
    #[test]
    fn track_toggle_amt_10_targets_track_0() {
        let mut engine = Engine::new(1);
        fire_step_event(&mut engine, 5, StepEventKind::TrackToggle { kind: ToggleKind::Mute, amt: 10, range: 1 }, 0);
        engine.step_once_for_test();
        assert!(engine.grid.active_page().tracks[0].muted);
    }

    #[test]
    fn track_toggle_negative_amt_is_off() {
        let mut engine = Engine::new(1);
        engine.grid.active_page_mut().tracks[4].muted = true;
        fire_step_event(&mut engine, 5, StepEventKind::TrackToggle { kind: ToggleKind::Mute, amt: -4, range: 1 }, 0);
        engine.step_once_for_test();
        assert!(!engine.grid.active_page().tracks[4].muted);
    }

    /// Ref: CE v5.30 p.40 (Track Toggle Consideration 5) + p.67 (On-the-Measure
    /// Mode, "a measure is 16 steps at x1, or... equal to the Page Length").
    #[test]
    fn track_toggle_mute_defers_to_measure_boundary_when_otm_set() {
        let mut engine = Engine::new(1);
        engine.grid.active_page_mut().length = 2; // measure = 2*12 = 24 ticks
        engine.grid.active_page_mut().on_the_measure = true;

        // Fires at tick 12 (track 5's normal step boundary).
        fire_step_event(&mut engine, 5, StepEventKind::TrackToggle { kind: ToggleKind::Mute, amt: 4, range: 1 }, 0);
        assert!(!engine.grid.active_page().tracks[4].muted, "must not apply at the next tick under OTM");

        for _ in 0..11 {
            engine.step_once_for_test(); // ticks 13..23
        }
        assert!(!engine.grid.active_page().tracks[4].muted, "still not due — measure boundary is tick 24");

        engine.step_once_for_test(); // tick 24: measure boundary
        assert!(engine.grid.active_page().tracks[4].muted, "must apply exactly at the measure boundary");
    }

    #[test]
    fn track_toggle_mute_applies_next_tick_when_otm_off() {
        let mut engine = Engine::new(1);
        engine.grid.active_page_mut().length = 2;
        assert!(!engine.grid.active_page().on_the_measure);
        fire_step_event(&mut engine, 5, StepEventKind::TrackToggle { kind: ToggleKind::Mute, amt: 4, range: 1 }, 0);
        engine.step_once_for_test();
        assert!(engine.grid.active_page().tracks[4].muted, "without OTM, the normal next-tick rule applies");
    }

    /// Ref: CE v5.30 p.57: "you may use CLR to restore the forward direction
    /// in slots 6-16" only makes sense as a *restore* if a fresh custom
    /// direction already behaves like Forward.
    #[test]
    fn custom_direction_default_behaves_like_forward() {
        let mut engine = Engine::new(1);
        engine.grid.active_page_mut().tracks[0].direction_raw = 6; // UserProgrammed(0), untouched default
        for expected_pos in [1u8, 2, 3] {
            for _ in 0..DEFAULT_STEP_TICKS {
                engine.step_once_for_test();
            }
            assert_eq!(engine.track_rt[0].pos, expected_pos);
        }
    }

    /// Ref: CE v5.30 p.56 + `reference/manual/pages/tutorial-03.txt`'s "Step
    /// double-play" worked example: two triggers in slice 1, both targeting
    /// step 1, means step 1 fires twice (two full tick-boundary visits)
    /// before the slice advances.
    #[test]
    fn custom_direction_multi_trigger_slice_fires_step_twice() {
        let mut engine = Engine::new(1);
        engine.grid.active_page_mut().tracks[0].direction_raw = 6;
        engine.grid.active_page_mut().tracks[0].steps[0].active = true;
        engine.grid.active_page_mut().tracks[0].steps[1].active = true;
        {
            let slice0 = &mut engine.grid.active_page_mut().user_directions[0].slices[0];
            slice0.triggers = [1, 1, 0, 0, 0, 0, 0, 0, 0];
            slice0.trigger_count = 2;
        }

        // Visit 1: fires step 0 (trigger 1 -> 0-based step 0), stays on slice 0.
        for _ in 0..DEFAULT_STEP_TICKS {
            engine.step_once_for_test();
        }
        assert_eq!(engine.track_rt[0].pos, 0, "still on slice 0 after only one of its two triggers");
        assert!(engine.last_tick_fires.as_slice().iter().any(|f| f.track == 0));

        // Visit 2: fires the second trigger (also step 0), *then* advances to slice 1.
        for _ in 0..DEFAULT_STEP_TICKS {
            engine.step_once_for_test();
        }
        assert_eq!(engine.track_rt[0].pos, 1, "slice advances only once both triggers have played");
    }

    /// Ref p.56: "every time Octopus gets an empty slice, it will pick a
    /// trigger for you at random, and play it normally" — one trigger per
    /// visit (like a 1-trigger slice), not a special multi-visit case.
    #[test]
    fn custom_direction_empty_slice_fires_once_then_advances() {
        let mut engine = Engine::new(1);
        engine.grid.active_page_mut().tracks[0].direction_raw = 6;
        for step in engine.grid.active_page_mut().tracks[0].steps.iter_mut() {
            step.active = true;
        }
        // Slice 0 already defaults to a single trigger (step 1); explicitly
        // clear it to empty for this test.
        engine.grid.active_page_mut().user_directions[0].slices[0] = DirectionSlice::default();

        for _ in 0..DEFAULT_STEP_TICKS {
            engine.step_once_for_test();
        }
        assert_eq!(engine.track_rt[0].pos, 1, "one visit to an empty slice is enough to advance");
        assert!(engine.last_tick_fires.as_slice().iter().any(|f| f.track == 0), "an empty slice must still fire something");
    }

    /// Ref p.56: "A setting of 100% means that the next slice will be the one
    /// following naturally... A setting of 0% will specify that the next
    /// slice will be the naturally previous one."
    #[test]
    fn custom_direction_certainty_next_boundaries_are_deterministic() {
        let mut engine = Engine::new(1);
        engine.grid.active_page_mut().tracks[0].direction_raw = 6;
        engine.grid.active_page_mut().user_directions[0].slices[3].certainty_next = 100;
        engine.track_rt[0].pos = 3;
        for _ in 0..DEFAULT_STEP_TICKS {
            engine.step_once_for_test();
        }
        assert_eq!(engine.track_rt[0].pos, 4, "100% certainty must always advance forward");

        let mut engine = Engine::new(1);
        engine.grid.active_page_mut().tracks[0].direction_raw = 6;
        engine.grid.active_page_mut().user_directions[0].slices[3].certainty_next = 0;
        engine.track_rt[0].pos = 3;
        for _ in 0..DEFAULT_STEP_TICKS {
            engine.step_once_for_test();
        }
        assert_eq!(engine.track_rt[0].pos, 2, "0% certainty must always go backward");
    }

    /// Ref: CE v5.30 p.16 / p.27. Phrase #0 (None) plays the step as-is; a
    /// selected phrase *enriches* it — base note plus programmed extras.
    #[test]
    fn phrase_on_step_fires_base_plus_enabled_extras() {
        let mut engine = Engine::new(1);
        engine.grid.active_page_mut().tracks[0].pitch = 60;
        engine.grid.active_page_mut().tracks[0].steps[0].active = true;
        engine.grid.active_page_mut().tracks[0].steps[0].phrase = Some(1);
        let mut phrase = Phrase::default();
        phrase.notes[0].enabled = true;
        phrase.notes[0].pitch_offset = 7;
        phrase.notes[0].start_ticks = 0;
        phrase.notes[1].enabled = true;
        phrase.notes[1].pitch_offset = 12;
        phrase.notes[1].start_ticks = 24;
        phrase.notes[2].enabled = false;
        phrase.notes[2].pitch_offset = 99;
        phrase.polyphony = 8;
        engine.grid.phrases[0] = phrase;

        for _ in 0..DEFAULT_STEP_TICKS {
            engine.step_once_for_test();
        }
        let mut pitches: Vec<u8> = engine.last_tick_fires.as_slice().iter().filter(|f| f.track == 0).map(|f| f.pitch).collect();
        pitches.sort_unstable();
        assert_eq!(pitches, vec![60, 67, 72], "base 60 plus extras +7 and +12; disabled +99 must not fire");
    }

    /// Ref: CE v5.30 p.23. Phrase polyphony limits how many of the enabled
    /// extras actually fire (on top of the base note).
    #[test]
    fn phrase_polyphony_limits_extra_notes() {
        let mut engine = Engine::new(1);
        engine.grid.active_page_mut().tracks[0].pitch = 60;
        engine.grid.active_page_mut().tracks[0].steps[0].active = true;
        engine.grid.active_page_mut().tracks[0].steps[0].phrase = Some(1);
        let mut phrase = Phrase::default();
        for (i, note) in phrase.notes.iter_mut().enumerate() {
            note.enabled = true;
            note.pitch_offset = (i as i8 + 1) * 2;
        }
        phrase.polyphony = 1;
        engine.grid.phrases[0] = phrase;

        for _ in 0..DEFAULT_STEP_TICKS {
            engine.step_once_for_test();
        }
        let extras: Vec<u8> = engine
            .last_tick_fires
            .as_slice()
            .iter()
            .filter(|f| f.track == 0 && f.pitch != 60)
            .map(|f| f.pitch)
            .collect();
        assert_eq!(extras.len(), 1, "polyphony 1 must fire exactly one extra on top of the base; got {:?}", extras);
        assert_eq!(extras[0], 62, "forward type + poly 1 takes the first enabled extra (+2)");
    }

    /// Phrase extras are real scheduled MIDI, not just FireLog entries — STA
    /// delay must show up as a later `at_sample` than the base note.
    #[test]
    fn phrase_sta_delays_the_extra_note() {
        let mut engine = Engine::new(1);
        engine.grid.active_page_mut().tracks[0].pitch = 60;
        engine.grid.active_page_mut().tracks[0].steps[0].active = true;
        engine.grid.active_page_mut().tracks[0].steps[0].phrase = Some(1);
        let mut phrase = Phrase::default();
        phrase.notes[0].enabled = true;
        phrase.notes[0].pitch_offset = 4;
        phrase.notes[0].start_ticks = 24;
        engine.grid.phrases[0] = phrase;

        let ctx = RenderContext { sample_rate: 48_000.0, buffer_len: 512, bpm: 120.0, playing: true };
        let mut base_abs = None;
        let mut extra_abs = None;
        for buf_i in 0..40u32 {
            let mut out = EventBuffer::new();
            engine.render(&ctx, &mut out);
            let origin = buf_i * ctx.buffer_len;
            for e in out.as_slice() {
                if let Event::NoteOn { note: 60, at_sample, .. } = *e {
                    base_abs = Some(origin + at_sample);
                }
                if let Event::NoteOn { note: 64, at_sample, .. } = *e {
                    extra_abs = Some(origin + at_sample);
                }
            }
            if base_abs.is_some() && extra_abs.is_some() {
                break;
            }
        }
        let base_abs = base_abs.expect("base note 60 should have fired");
        let extra_abs = extra_abs.expect("phrase extra 64 should have fired");
        // 120 bpm / 192 PPQN / 48 kHz ≈ 125 samples/tick; 24 ticks ≈ 3000 samples.
        let delta = extra_abs as i32 - base_abs as i32;
        assert!(
            (2800..=3200).contains(&delta),
            "expected ~3000-sample STA delay, got extra={extra_abs} base={base_abs} delta={delta}"
        );
    }

    /// Ref: CE v5.30 p.30. POS 5 is double-speed: a 24-tick extra becomes 12.
    #[test]
    fn phrase_pos_double_speed_halves_sta_delay() {
        let mut engine = Engine::new(1);
        engine.grid.active_page_mut().tracks[0].pitch = 60;
        engine.grid.active_page_mut().tracks[0].steps[0].active = true;
        engine.grid.active_page_mut().tracks[0].steps[0].phrase = Some(1);
        engine.grid.active_page_mut().tracks[0].steps[0].phrase_pos = 5;
        let mut phrase = Phrase::default();
        phrase.notes[0].enabled = true;
        phrase.notes[0].pitch_offset = 4;
        phrase.notes[0].start_ticks = 24;
        engine.grid.phrases[0] = phrase;

        let ctx = RenderContext { sample_rate: 48_000.0, buffer_len: 512, bpm: 120.0, playing: true };
        let mut base_abs = None;
        let mut extra_abs = None;
        for buf_i in 0..40u32 {
            let mut out = EventBuffer::new();
            engine.render(&ctx, &mut out);
            let origin = buf_i * ctx.buffer_len;
            for e in out.as_slice() {
                if let Event::NoteOn { note: 60, at_sample, .. } = *e {
                    base_abs = Some(origin + at_sample);
                }
                if let Event::NoteOn { note: 64, at_sample, .. } = *e {
                    extra_abs = Some(origin + at_sample);
                }
            }
            if base_abs.is_some() && extra_abs.is_some() {
                break;
            }
        }
        let delta = extra_abs.expect("extra") as i32 - base_abs.expect("base") as i32;
        // 12 ticks ≈ 1500 samples at 120 bpm / 48 kHz.
        assert!(
            (1400..=1600).contains(&delta),
            "POS 5 should halve a 24-tick STA to ~1500 samples, got {delta}"
        );
    }

    /// Selecting factory Green phrase 1 (index 1) enriches with a -10 vel,
    /// +0 pit echo. The base pitch is unchanged.
    #[test]
    fn factory_green_phrase_1_enriches_a_step() {
        let mut engine = Engine::new(1);
        engine.grid.active_page_mut().tracks[0].pitch = 60;
        engine.grid.active_page_mut().tracks[0].steps[0].active = true;
        engine.grid.active_page_mut().tracks[0].steps[0].phrase = Some(1);

        for _ in 0..DEFAULT_STEP_TICKS {
            engine.step_once_for_test();
        }
        let pitches: Vec<u8> = engine.last_tick_fires.as_slice().iter().filter(|f| f.track == 0).map(|f| f.pitch).collect();
        assert_eq!(pitches.iter().filter(|&&p| p == 60).count(), 2, "base + same-pitch echo: {:?}", pitches);
        let vels: Vec<u8> = engine.last_tick_fires.as_slice().iter().filter(|f| f.track == 0).map(|f| f.velocity).collect();
        assert!(vels.contains(&100), "base vel: {:?}", vels);
        assert!(vels.contains(&90), "echo vel 100-10: {:?}", vels);
    }

    /// Ref: CE v5.30 p.38-39. A Track Rotate event on the source track moves
    /// that track's step data (next-tick, same as every other step event).
    #[test]
    fn track_rotate_event_moves_step_data_next_tick() {
        let mut engine = Engine::new(1);
        engine.grid.active_page_mut().tracks[5].steps[0].active = true;
        engine.grid.active_page_mut().tracks[5].steps[0].pitch_offset = 3;
        engine.grid.active_page_mut().tracks[5].steps[0].event = Some(StepEvent {
            target_track: 5,
            kind: StepEventKind::TrackRotate { amt: 1 },
        });
        engine.grid.active_page_mut().tracks[5].steps[1].active = true;
        engine.grid.active_page_mut().tracks[5].steps[1].pitch_offset = 7;
        // p.38 rotates *all* steps except skip/hyperstep/mask — skip the
        // unused tail so the rotatable set is exactly {0, 1}.
        for step in engine.grid.active_page_mut().tracks[5].steps[2..].iter_mut() {
            step.skip = true;
        }

        for _ in 0..DEFAULT_STEP_TICKS {
            engine.step_once_for_test();
        }
        assert_eq!(engine.grid.active_page().tracks[5].steps[0].pitch_offset, 3, "rotate is deferred to the next tick");
        engine.step_once_for_test();
        assert_eq!(engine.grid.active_page().tracks[5].steps[0].pitch_offset, 7);
        assert_eq!(engine.grid.active_page().tracks[5].steps[1].pitch_offset, 3);
    }

    /// Ref: CE v5.30 p.45. Direction 4 is Brownian: "2/3 probability forward,
    /// 1/3 probability reverse play."
    #[test]
    fn brownian_is_biased_two_thirds_forward() {
        let mut forward = 0u32;
        let mut backward = 0u32;
        for seed in 0..400u64 {
            let mut engine = Engine::new(seed);
            engine.grid.active_page_mut().tracks[0].direction_raw = 4;
            engine.grid.active_page_mut().length = 16;
            engine.track_rt[0].pos = 8;
            for _ in 0..DEFAULT_STEP_TICKS {
                engine.step_once_for_test();
            }
            match engine.track_rt[0].pos {
                9 => forward += 1,
                7 => backward += 1,
                other => panic!("brownian from 8 must land on 7 or 9, got {other}"),
            }
        }
        // 400 trials, p=2/3: expected ~267/133, σ ≈ 9.4. 230/80 is several
        // sigma inside the 2/3 region and several sigma away from 1/2 (200).
        assert!(
            forward > 230 && backward > 80,
            "expected ~2/3 forward over 400 seeds, got forward={forward} backward={backward}"
        );
    }

    // ---- SPEC-0001 O1 / O2: transport safety (WENGE-0001, WENGE-0002) ----

    fn ctx(playing: bool, buffer_len: u32) -> RenderContext {
        RenderContext { sample_rate: 48_000.0, buffer_len, bpm: 120.0, playing }
    }

    /// Track 0, one long step (LEN 100 ticks = 12,500 samples at 120 BPM), so its NoteOff is
    /// still queued after the NoteOn goes out. Shorter than the 24,000-sample loop, so the
    /// note ends before the pattern starts it again.
    fn engine_with_one_long_note() -> Engine {
        let mut e = Engine::new(1);
        let step = &mut e.grid.active_page_mut().tracks[0].steps[0];
        step.active = true;
        step.length_ticks = 100;
        e
    }

    #[test]
    fn sounding_table_follows_emitted_events_not_scheduled_ones() {
        let mut e = engine_with_one_long_note();
        let mut out = EventBuffer::new();
        // Nothing has been handed to the caller yet: scheduled is not sounding.
        assert_eq!(e.sounding_count(), 0);
        for _ in 0..4 {
            out.clear();
            e.render(&ctx(true, 512), &mut out);
        }
        assert_eq!(e.sounding_count(), 1, "the NoteOn went out, the NoteOff is still queued");
        for _ in 0..36 {
            out.clear();
            e.render(&ctx(true, 512), &mut out);
        }
        assert_eq!(e.sounding_count(), 0, "the NoteOff went out too");
    }

    #[test]
    fn retriggered_pitch_gets_one_noteoff_per_noteon_on_stop() {
        let mut e = Engine::new(1);
        let t = &mut e.grid.active_page_mut().tracks[0];
        for s in 0..STEP_COUNT {
            t.steps[s].active = true;
            t.steps[s].length_ticks = 192; // each step starts the same pitch again while the last still rings
        }
        let tally = |out: &EventBuffer| {
            let on = out.as_slice().iter().filter(|ev| matches!(ev, Event::NoteOn { .. })).count();
            let off = out.as_slice().iter().filter(|ev| matches!(ev, Event::NoteOff { .. })).count();
            (on, off)
        };
        let mut out = EventBuffer::new();
        let (mut on, mut off) = (0, 0);
        for _ in 0..40 {
            out.clear();
            e.render(&ctx(true, 512), &mut out);
            let (a, b) = tally(&out);
            on += a;
            off += b;
        }
        assert!(on - off >= 2, "the pitch must be sounding more than once for this test to mean anything");
        out.clear();
        e.render(&ctx(false, 512), &mut out);
        let (a, b) = tally(&out);
        on += a;
        off += b;
        assert_eq!(on, off);
        assert_eq!(e.sounding_count(), 0);
    }

    #[test]
    fn a_flush_larger_than_one_buffer_finishes_over_the_next_renders() {
        let mut e = Engine::new(1);
        // 320 distinct notes on 16 channels of port 1: more than the 256 one buffer holds.
        for c in 0..16 {
            for n in 0..20 {
                e.sounding[0][c][n] = 1;
            }
        }
        e.flush_pending = true;
        let mut out = EventBuffer::new();
        e.render(&ctx(false, 64), &mut out);
        assert_eq!(out.as_slice().len(), MAX_EVENTS_PER_TICK);
        assert!(out.as_slice().iter().all(|ev| matches!(ev, Event::NoteOff { at_sample: 0, .. })));
        assert!(e.flush_pending, "more to do");

        out.clear();
        e.render(&ctx(false, 64), &mut out);
        let offs = out.as_slice().iter().filter(|ev| matches!(ev, Event::NoteOff { .. })).count();
        let ccs = out.as_slice().iter().filter(|ev| matches!(ev, Event::Cc { cc: 123, .. })).count();
        assert_eq!((offs, ccs), (320 - MAX_EVENTS_PER_TICK, 16));
        assert_eq!(e.sounding_count(), 0);
        assert!(!e.flush_pending);

        out.clear();
        e.render(&ctx(false, 64), &mut out);
        assert!(out.as_slice().is_empty(), "the flush is done, a stopped engine is silent");
    }

    #[test]
    fn flush_waits_for_room_when_the_caller_passes_a_nearly_full_buffer() {
        let mut e = Engine::new(1);
        e.sounding[1][3][60] = 1;
        e.flush_pending = true;
        let mut out = EventBuffer::new();
        while out.remaining() > 0 {
            out.push(Event::Cc { port: 1, ch: 1, cc: 1, val: 0, at_sample: 0 });
        }
        e.render(&ctx(false, 64), &mut out);
        assert_eq!(out.as_slice().len(), MAX_EVENTS_PER_TICK, "nothing was added or dropped");
        assert_eq!(e.sounding_count(), 1, "the note is still owed a NoteOff");
        out.clear();
        e.render(&ctx(false, 64), &mut out);
        assert_eq!(out.as_slice().len(), 2, "NoteOff then CC 123");
        assert_eq!(e.sounding_count(), 0);
    }

    #[test]
    fn out_of_range_port_channel_or_note_is_never_tracked_and_never_panics() {
        assert_eq!(sounding_slot(1, 1, 0), Some((0, 0, 0)));
        assert_eq!(sounding_slot(2, 16, 127), Some((1, 15, 127)));
        for (p, c, n) in [(0, 1, 60), (3, 1, 60), (1, 0, 60), (1, 17, 60), (1, 1, 128), (255, 255, 255)] {
            assert_eq!(sounding_slot(p, c, n), None, "({p}, {c}, {n})");
        }
        let mut e = Engine::new(1);
        e.track_emitted(RawEvent::NoteOn { port: 9, ch: 99, note: 200, vel: 1 });
        e.track_emitted(RawEvent::NoteOff { port: 9, ch: 99, note: 200 });
        assert_eq!(e.sounding_count(), 0);
    }

    #[test]
    fn noteoff_without_a_noteon_never_underflows_the_table() {
        let mut e = Engine::new(1);
        e.track_emitted(RawEvent::NoteOff { port: 1, ch: 1, note: 60 });
        assert_eq!(e.sounding_count(), 0);
        for _ in 0..300 {
            e.track_emitted(RawEvent::NoteOn { port: 1, ch: 1, note: 60, vel: 1 });
        }
        assert_eq!(e.sounding[0][0][60], 255, "saturates instead of wrapping");
    }

    #[test]
    fn play_resyncs_the_tick_clock_to_the_current_sample() {
        let mut e = Engine::new(1);
        let mut out = EventBuffer::new();
        for _ in 0..1000 {
            e.render(&ctx(false, 512), &mut out);
        }
        assert_eq!(e.sample_clock, 512_000.0);
        e.set_running(true);
        assert_eq!(e.next_tick_due, e.sample_clock);
        // Already running: a second set_running(true) must not move the clock again.
        e.next_tick_due += 7.0;
        e.set_running(true);
        assert_eq!(e.next_tick_due, e.sample_clock + 7.0);
    }

    #[test]
    fn stop_while_stopped_asks_for_cc_123_on_all_32_channels_but_a_running_stop_does_not() {
        let mut e = Engine::new(1);
        e.handle_command(crate::types::Command::Stop);
        assert!(e.flush_pending);
        assert_eq!(e.flush_cc, [u16::MAX; 2]);

        let mut running = Engine::new(1);
        running.handle_command(crate::types::Command::Play);
        running.handle_command(crate::types::Command::Stop);
        assert_eq!(running.flush_cc, [0; 2], "a first Stop sends CC 123 only where a note was sounding");
    }

    #[test]
    fn host_transport_stop_while_stopped_is_not_a_button_press() {
        let mut e = Engine::new(1);
        e.handle_command(crate::types::Command::HostTransport { ppqn_pos: 0, bpm: 120.0, playing: false });
        assert!(!e.flush_pending, "a host repeating 'not playing' every buffer must not panic the receivers");
    }
    /// `MAX_EARLY_TICKS` must cover the earliest any note can start, or `render` schedules
    /// it in the past again. Every input that moves a note earlier is enumerated here, so a
    /// table edit that lengthens the reach fails this test instead of quietly reintroducing
    /// SPEC-0001 O3.
    #[test]
    fn max_early_ticks_covers_every_table_and_is_tight() {
        let mut earliest = 0i32;
        for track_sta in 0..=16u8 {
            for step_offset in -5..=5i32 {
                earliest = earliest.min(tables::scale_sta_ticks(track_sta, step_offset));
            }
        }
        assert_eq!(earliest, -(MAX_EARLY_TICKS as i32), "the constant is exactly the STA table's reach");

        // Everything else is unsigned by type, so it can only push a note later. The
        // signatures are the guard: if one turns signed this stops compiling.
        let _: u32 = tables::strum_offset_ticks(9, 8);
        let _: u16 = tables::scale_phrase_sta(u8::MAX, 1);
        let _: fn(u8, &mut Rng) -> u32 = tables::grv_delay_ticks;
    }

    // ------------------------------------------------------------------ WENGE-0005

    fn count_kinds(events: &[Event]) -> (usize, usize) {
        let ons = events.iter().filter(|e| matches!(e, Event::NoteOn { .. })).count();
        let offs = events.iter().filter(|e| matches!(e, Event::NoteOff { .. })).count();
        (ons, offs)
    }

    /// SPEC-0001 O5. When more events fall due than the caller's buffer holds, the rest
    /// wait for the next render. Before, they were dropped: a dropped NoteOff is a note
    /// that never ends.
    #[test]
    fn a_full_output_buffer_defers_events_instead_of_dropping_them() {
        let mut e = Engine::new(1);
        for ch in 1..=2u8 {
            for note in 0..100u8 {
                e.schedule(10.0, RawEvent::NoteOn { port: 1, ch, note, vel: 100 });
                e.schedule(20.0, RawEvent::NoteOff { port: 1, ch, note });
            }
        }
        let mut first = EventBuffer::new();
        e.render(&ctx(false, 64), &mut first);
        assert_eq!(first.as_slice().len(), MAX_EVENTS_PER_TICK, "the buffer is full");

        let mut second = EventBuffer::new();
        e.render(&ctx(false, 64), &mut second);
        assert!(
            second.as_slice().iter().all(|ev| matches!(ev, Event::NoteOff { at_sample: 0, .. })),
            "deferred events come out first, as late as possible-early: sample 0"
        );
        let mut all = first.as_slice().to_vec();
        all.extend_from_slice(second.as_slice());
        assert_eq!(count_kinds(&all), (200, 200), "400 events went in, 400 came out");
        assert_eq!(e.sounding_count(), 0);
        let d = e.diagnostics();
        assert_eq!(d.deferred_events, 400 - MAX_EVENTS_PER_TICK as u32, "the health counter saw it");
        assert_eq!(d.late_events, 400 - MAX_EVENTS_PER_TICK as u32, "and they went out late");
        assert_eq!(d.queue_overflows, 0);
    }

    /// A note is one NoteOn and one NoteOff. A queue with room for only one of them takes
    /// neither. Before, the NoteOn went in and the NoteOff was dropped: a stuck note.
    #[test]
    fn a_full_queue_refuses_a_note_as_a_pair_never_half() {
        let mut e = Engine::new(1);
        for i in 0..(QUEUE_CAP - 1) {
            e.schedule(1.0e9, RawEvent::Cc { port: 1, ch: 1, cc: 1, val: (i % 128) as u8 });
        }
        assert_eq!(e.queue_len, QUEUE_CAP - 1);
        e.grid.active_page_mut().tracks[0].steps[0].active = true;
        for _ in 0..DEFAULT_STEP_TICKS {
            e.step_once_for_test(); // the step fires on the twelfth tick
        }

        let queued: Vec<RawEvent> = e.queue[..e.queue_len].iter().flatten().map(|s| s.event).collect();
        let ons = queued.iter().filter(|r| matches!(r, RawEvent::NoteOn { .. })).count();
        let offs = queued.iter().filter(|r| matches!(r, RawEvent::NoteOff { .. })).count();
        assert_eq!(ons, offs, "a NoteOn was queued without its NoteOff");
        assert_eq!(e.diagnostics().queue_overflows, 1, "the refused note is counted");
        assert_eq!(e.diagnostics().queue_high_water, (QUEUE_CAP - 1) as u32);
    }

    /// A host that reports tempo 0 (several do before playback starts), NaN or a negative
    /// number must not break the clock. Before, one such buffer set the next tick time to
    /// infinity, so the engine never ticked again, and queued events with a NaN time.
    #[test]
    fn an_unusable_tempo_does_not_wedge_the_clock() {
        for bad in [0.0f32, f32::NAN, -5.0] {
            let mut e = Engine::new(1);
            e.grid.active_page_mut().tracks[0].steps[0].active = true;
            let mut out = EventBuffer::new();
            for _ in 0..4 {
                out.clear();
                let c = RenderContext { sample_rate: 48_000.0, buffer_len: 512, bpm: bad, playing: true };
                e.render(&c, &mut out);
            }
            assert!(e.queue[..e.queue_len].iter().flatten().all(|s| s.due_sample.is_finite()), "bpm {bad}: a NaN time is queued");
            let mut all = Vec::new();
            for _ in 0..40 {
                out.clear();
                e.render(&ctx(true, 512), &mut out);
                all.extend_from_slice(out.as_slice());
            }
            assert!(count_kinds(&all).0 > 0, "bpm {bad}: the engine never played again once the tempo was sane");
            assert_eq!(e.diagnostics().unusable_tempo_renders, 4, "bpm {bad}: only the four unusable renders are counted");
        }
    }

    #[test]
    fn a_stopped_transport_with_a_bad_tempo_is_not_a_problem() {
        let mut e = Engine::new(1);
        let mut out = EventBuffer::new();
        let c = RenderContext { sample_rate: 48_000.0, buffer_len: 512, bpm: 0.0, playing: false };
        for _ in 0..4 {
            e.render(&c, &mut out);
        }
        assert_eq!(e.diagnostics(), Diagnostics::default(), "nothing is running, so nothing is wrong");
    }

    #[test]
    fn a_render_longer_than_a_chunk_is_split_without_changing_the_result() {
        // The same 3 seconds as one call of 144,000 samples and as calls of 480.
        let run = |len: u32| {
            let mut e = Engine::new(4);
            for t in 0..3 {
                for st in [0usize, 4, 8, 12] {
                    e.grid.active_page_mut().tracks[t].steps[st].active = true;
                }
            }
            let mut all = Vec::new();
            let mut done = 0u32;
            while done < 144_000 {
                let mut out = EventBuffer::with_limit(MAX_EVENTS_PER_TICK);
                e.render(&RenderContext { sample_rate: 48_000.0, buffer_len: len, bpm: 100.0, playing: true }, &mut out);
                for ev in out.as_slice() {
                    let (at, rest) = match *ev {
                        Event::NoteOn { port, ch, note, vel, at_sample } => (at_sample, (1, port, ch, note, vel)),
                        Event::NoteOff { port, ch, note, at_sample } => (at_sample, (2, port, ch, note, 0)),
                        Event::Cc { port, ch, cc, val, at_sample } => (at_sample, (3, port, ch, cc, val)),
                        Event::PitchBend { port, ch, value, at_sample } => (at_sample, (4, port, ch, 0, (value >> 7) as u8)),
                        Event::ChannelPressure { port, ch, value, at_sample } => (at_sample, (5, port, ch, 0, value)),
                    };
                    all.push((done + at, rest));
                }
                done += len;
            }
            all.sort();
            (all, e.diagnostics().queue_high_water)
        };
        let (small, small_hw) = run(480);
        let (huge, huge_hw) = run(144_000);
        assert!(small.len() > 20);
        // The huge buffer holds 100+ events here, under the 256 a call takes, so it is the same.
        assert_eq!(huge, small);
        assert!(huge_hw < 60, "chunking keeps the queue small even for a 3 s buffer: {huge_hw}, small {small_hw}");
    }

    // ------------------------------------------------------------------ WENGE-0010

    fn render_all(e: &mut Engine, len: u32) -> Vec<Event> {
        let mut out = EventBuffer::new();
        e.render(&ctx(false, len), &mut out);
        out.as_slice().to_vec()
    }

    /// Events that share a sample go out in a fixed order: NoteOffs, then controllers, then
    /// NoteOns. A note that ends where the same pitch begins again must release before it
    /// strikes, or the receiver ends up silent. The NoteOn here is due a fraction of a
    /// sample before the NoteOff, so ordering by the raw time gets it wrong.
    #[test]
    fn events_in_one_sample_go_off_then_controller_then_on() {
        let mut e = Engine::new(1);
        e.schedule(100.3, RawEvent::NoteOn { port: 1, ch: 1, note: 60, vel: 90 });
        e.schedule(100.5, RawEvent::Cc { port: 1, ch: 1, cc: 74, val: 5 });
        e.schedule(100.7, RawEvent::NoteOff { port: 1, ch: 1, note: 60 });
        let got = render_all(&mut e, 512);
        assert_eq!(
            got,
            vec![
                Event::NoteOff { port: 1, ch: 1, note: 60, at_sample: 100 },
                Event::Cc { port: 1, ch: 1, cc: 74, val: 5, at_sample: 100 },
                Event::NoteOn { port: 1, ch: 1, note: 60, vel: 90, at_sample: 100 },
            ]
        );
    }

    /// Ties are broken by the order events were scheduled in, always. Before, removal from
    /// the queue swapped the last entry into the gap, so the order of equal-time events
    /// depended on how many had been emitted before them.
    #[test]
    fn equal_events_keep_the_order_they_were_scheduled_in() {
        let mut e = Engine::new(1);
        for val in 0..6u8 {
            e.schedule(200.0, RawEvent::Cc { port: 1, ch: 1, cc: 1, val });
        }
        let vals: Vec<u8> = render_all(&mut e, 512)
            .iter()
            .map(|ev| match ev {
                Event::Cc { val, .. } => *val,
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(vals, vec![0, 1, 2, 3, 4, 5]);
    }

    /// CE v5.30 p.40, Step Event Consideration 1: "If the Velocity is 0 then the note is not
    /// transmitted." A NoteOn with velocity 0 is a NoteOff to every MIDI receiver, so the
    /// note must not be sent at all, and its NoteOff must not be either.
    #[test]
    fn a_note_with_velocity_zero_is_not_transmitted() {
        let mut e = Engine::new(1);
        let t = &mut e.grid.active_page_mut().tracks[0];
        t.velocity = 0;
        t.steps[0].active = true;
        let mut all = Vec::new();
        for _ in 0..60 {
            let mut out = EventBuffer::new();
            e.render(&ctx(true, 512), &mut out);
            all.extend_from_slice(out.as_slice());
        }
        assert_eq!(count_kinds(&all), (0, 0), "{all:?}");
    }

    /// The page's velocity factor scales the note (`vel * factor / 8`), so a quiet track at
    /// the lowest factor rounds to 0 and is not transmitted, where it used to send NoteOn
    /// velocity 0.
    #[test]
    fn velocity_scaled_down_to_zero_is_not_transmitted_and_1_is_the_quietest_note() {
        for (base, factor, transmitted) in [(7u8, 1u8, false), (8, 1, true), (1, 8, true), (0, 8, false)] {
            let mut e = Engine::new(1);
            e.grid.active_page_mut().velocity_factor = factor;
            let t = &mut e.grid.active_page_mut().tracks[0];
            t.velocity = base;
            t.steps[0].active = true;
            let mut all = Vec::new();
            for _ in 0..60 {
                let mut out = EventBuffer::new();
                e.render(&ctx(true, 512), &mut out);
                all.extend_from_slice(out.as_slice());
            }
            let ons: Vec<u8> = all
                .iter()
                .filter_map(|ev| if let Event::NoteOn { vel, .. } = ev { Some(*vel) } else { None })
                .collect();
            assert_eq!(!ons.is_empty(), transmitted, "velocity {base} x {factor}/8: {ons:?}");
            assert!(ons.iter().all(|v| *v >= 1));
        }
    }

    fn play_events(e: &mut Engine, buffers: usize) -> Vec<Event> {
        let mut all = Vec::new();
        for _ in 0..buffers {
            let mut out = EventBuffer::new();
            e.render(&ctx(true, 512), &mut out);
            all.extend_from_slice(out.as_slice());
        }
        all
    }

    /// CE v5.30 p.46 and p.91: a bender track sends pitch bend from its steps' MCC values,
    /// which the editor holds as the top 7 bits of the 14-bit value.
    #[test]
    fn a_bender_track_sends_pitch_bend_with_the_step_value_in_the_top_7_bits() {
        for (mcc, value) in [(0u8, 0u16), (1, 128), (64, 8192), (127, 16256)] {
            let mut e = Engine::new(1);
            let t = &mut e.grid.active_page_mut().tracks[0];
            t.mcc = Mcc::Bend;
            t.steps[0].active = true;
            t.steps[0].mcc_value = Some(mcc);
            let bends: Vec<u16> = play_events(&mut e, 40)
                .iter()
                .filter_map(|ev| if let Event::PitchBend { port: 1, ch: 1, value, .. } = ev { Some(*value) } else { None })
                .collect();
            assert_eq!(bends, vec![value], "MCC {mcc}");
        }
    }

    #[test]
    fn a_pressure_track_sends_channel_pressure_and_a_none_track_sends_nothing_extra() {
        let mut e = Engine::new(1);
        let t = &mut e.grid.active_page_mut().tracks[0];
        t.mcc = Mcc::Pressure;
        t.steps[0].active = true;
        t.steps[0].mcc_value = Some(100);
        let got = play_events(&mut e, 40);
        assert!(got.iter().any(|ev| matches!(ev, Event::ChannelPressure { port: 1, ch: 1, value: 100, .. })), "{got:?}");

        let mut e = Engine::new(1);
        let t = &mut e.grid.active_page_mut().tracks[0];
        t.mcc = Mcc::None;
        t.steps[0].active = true;
        t.steps[0].mcc_value = Some(100);
        let got = play_events(&mut e, 40);
        assert!(got.iter().all(|ev| matches!(ev, Event::NoteOn { .. } | Event::NoteOff { .. })), "{got:?}");
    }

    /// The bend a step sends reaches the receiver before the note it shapes, even though
    /// both are due at the same instant.
    #[test]
    fn a_controller_is_sent_before_the_note_it_belongs_to() {
        let mut e = Engine::new(1);
        let t = &mut e.grid.active_page_mut().tracks[0];
        t.mcc = Mcc::Bend;
        t.steps[0].active = true;
        t.steps[0].mcc_value = Some(64);
        let got = play_events(&mut e, 40);
        let bend = got.iter().position(|ev| matches!(ev, Event::PitchBend { .. })).expect("a bend");
        let on = got.iter().position(|ev| matches!(ev, Event::NoteOn { .. })).expect("a note");
        assert!(bend < on, "{got:?}");
    }

    /// A step's phrase extras follow the same rule as the step's own note.
    #[test]
    fn a_phrase_extra_that_reaches_velocity_zero_is_not_transmitted() {
        let mut e = Engine::new(1);
        e.grid.phrases = crate::phrases::factory_phrases();
        let t = &mut e.grid.active_page_mut().tracks[0];
        t.velocity = 3;
        t.steps[0].active = true;
        t.steps[0].phrase = Some(1);
        let mut all = play_events(&mut e, 60);
        all.retain(|ev| matches!(ev, Event::NoteOn { .. } | Event::NoteOff { .. }));
        let (ons, offs) = count_kinds(&all);
        assert_eq!(ons, offs, "every note sent has its NoteOff: {all:?}");
        assert!(all.iter().all(|ev| !matches!(ev, Event::NoteOn { vel: 0, .. })), "{all:?}");
    }
}

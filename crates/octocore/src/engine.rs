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
}

#[derive(Clone, Copy, Debug)]
enum RawEvent {
    NoteOn { port: u8, ch: u8, note: u8, vel: u8 },
    NoteOff { port: u8, ch: u8, note: u8 },
    Cc { port: u8, ch: u8, cc: u8, val: u8 },
}

const QUEUE_CAP: usize = 512;

pub struct EventBuffer {
    events: [Event; MAX_EVENTS_PER_TICK],
    count: usize,
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
        }
    }

    pub fn as_slice(&self) -> &[Event] {
        &self.events[..self.count]
    }

    pub fn clear(&mut self) {
        self.count = 0;
    }

    fn push(&mut self, e: Event) {
        if self.count < MAX_EVENTS_PER_TICK {
            self.events[self.count] = e;
            self.count += 1;
        }
    }
}

pub struct RenderContext {
    pub sample_rate: f32,
    pub buffer_len: u32,
    pub bpm: f32,
    pub playing: bool,
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
    queue: [Option<Scheduled>; QUEUE_CAP],
    queue_len: usize,
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
}

impl Engine {
    pub fn new(seed: u64) -> Self {
        Engine {
            grid: Grid::default_grid(),
            rng: Rng::new(seed),
            running: false,
            global_tick: 0,
            next_tick_due: 0.0,
            sample_clock: 0.0,
            track_rt: [TrackRuntime::default(); TRACK_COUNT],
            queue: [None; QUEUE_CAP],
            queue_len: 0,
            deferred_actions: [None; TRACK_COUNT],
            measure_deferred: [None; TRACK_COUNT],
            last_tick_fires: FireLog::default(),
        }
    }

    /// Test/fixture seam: run exactly one 192-PPQN tick, ignoring real-time
    /// sample scheduling, and leave `last_tick_fires` populated with whatever
    /// fired. `tests/conformance` fixtures are written in ticks, not samples.
    pub fn step_once_for_test(&mut self) {
        self.step_all_tracks(0.0, 1.0);
    }

    pub fn set_running(&mut self, running: bool) {
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
            Command::Stop => self.set_running(false),
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
            self.queue[self.queue_len] = Some(Scheduled { due_sample, event });
            self.queue_len += 1;
        }
        // A full queue silently drops the event rather than reallocating —
        // QUEUE_CAP is sized well above "full density" (docs §7).
    }

    /// CC 123 (all notes off) — matches `_allNotesOff` in the archived v1 engine,
    /// minus its dead `held`-map bookkeeping (never populated anywhere in v1).
    fn all_notes_off_now(&mut self) {
        self.queue = [None; QUEUE_CAP];
        self.queue_len = 0;
    }

    /// Advance the engine by exactly `ctx.buffer_len` samples, appending any events
    /// due in that window to `out`. `out` is not cleared by this call — callers
    /// rendering buffer-by-buffer should clear it between calls.
    pub fn render(&mut self, ctx: &RenderContext, out: &mut EventBuffer) {
        if ctx.playing != self.running {
            self.set_running(ctx.playing);
        }

        let buffer_start = self.sample_clock;
        let buffer_end = buffer_start + ctx.buffer_len as f64;
        let samples_per_tick = if ctx.bpm > 0.0 {
            (ctx.sample_rate as f64) * 60.0 / (ctx.bpm as f64 * TICKS_PER_QUARTER as f64)
        } else {
            f64::INFINITY
        };

        if self.running {
            while self.next_tick_due < buffer_end {
                let at = self.next_tick_due;
                self.step_all_tracks(at, samples_per_tick);
                self.next_tick_due += samples_per_tick;
            }
        }

        loop {
            let mut earliest: Option<(usize, f64)> = None;
            for i in 0..self.queue_len {
                if let Some(s) = self.queue[i] {
                    if s.due_sample < buffer_end && earliest.map_or(true, |(_, t)| s.due_sample < t) {
                        earliest = Some((i, s.due_sample));
                    }
                }
            }
            let Some((idx, _)) = earliest else { break };
            let scheduled = self.queue[idx].take().unwrap();
            self.queue_len -= 1;
            self.queue[idx] = self.queue[self.queue_len].take();

            let at_sample = (scheduled.due_sample - buffer_start).max(0.0) as u32;
            let at_sample = at_sample.min(ctx.buffer_len.saturating_sub(1));
            out.push(match scheduled.event {
                RawEvent::NoteOn { port, ch, note, vel } => Event::NoteOn { port, ch, note, vel, at_sample },
                RawEvent::NoteOff { port, ch, note } => Event::NoteOff { port, ch, note, at_sample },
                RawEvent::Cc { port, ch, cc, val } => Event::Cc { port, ch, cc, val, at_sample },
            });
        }

        self.sample_clock = buffer_end;
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

        if level > 0 && note_count == 1 {
            let pitch = played[0];
            for k in 1..=7u8 {
                let off = tables::strum_offset_ticks(level, k) as i32;
                let due = tick_due_sample + (on_tick_offset + off) as f64 * samples_per_tick;
                self.schedule(due, RawEvent::NoteOn { port, ch, note: pitch, vel: final_vel });
                self.schedule(due + final_len_ticks as f64 * samples_per_tick, RawEvent::NoteOff { port, ch, note: pitch });
            }
        } else {
            for (pi, &pitch) in played.iter().enumerate() {
                let off = tables::strum_offset_ticks(level, (pi + 1) as u8) as i32;
                let due = tick_due_sample + (on_tick_offset + off) as f64 * samples_per_tick;
                self.schedule(due, RawEvent::NoteOn { port, ch, note: pitch, vel: final_vel });
                self.schedule(due + final_len_ticks as f64 * samples_per_tick, RawEvent::NoteOff { port, ch, note: pitch });
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
                    let extra_start = on_tick_offset + note.start_ticks as i32;
                    self.last_tick_fires.push(NoteFire { track: ti, pitch: extra_pit, velocity: extra_vel });
                    let due = tick_due_sample + extra_start as f64 * samples_per_tick;
                    self.schedule(due, RawEvent::NoteOn { port, ch, note: extra_pit, vel: extra_vel });
                    self.schedule(due + extra_len as f64 * samples_per_tick, RawEvent::NoteOff { port, ch, note: extra_pit });
                }
            }
        }

        if let Some(step_mcc) = step.mcc_value {
            let due = tick_due_sample + on_tick_offset as f64 * samples_per_tick;
            if let Mcc::Cc(cc) = base_track.mcc {
                self.schedule(due, RawEvent::Cc { port, ch, cc, val: clamp_midi(step_mcc as i32) });
            }
            // Bend/pressure need a dedicated MIDI message type; `Event` has no
            // pitch-bend/poly-pressure variant yet, so those are left unemitted
            // rather than mis-encoded as a bogus CC number.
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
}

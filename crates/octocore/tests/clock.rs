//! The engine's MIDI clock output and its tick position (SPEC-0002 P4c, ADR-0009 and its amendment 1, the Metronome request
//! `journal/metronome/requests/2026-10-02-clock-output-and-tick-position.md`).
//!
//! Written before the code. The clock is a second list beside the events (`Realtime`, `RealtimeEvent`), taken after each
//! `render` with `take_realtime`. With the clock off, which is the default, the engine does what it did before; the goldens,
//! the conformance fixtures and `tests/invariants.rs` are the proof of that, and the first tests here repeat it for the clock.
//!
//! **The tick is 192 to the quarter note (D0, `WENGE-0012`, open),** so a pulse is 8 ticks and a step is 12. Nothing here
//! says a step is a whole number of pulses; the test that would is `pending` at the bottom and names D0.

use octocore::domain::{DEFAULT_STEP_TICKS, TICKS_PER_CLOCK, TICKS_PER_QUARTER};
use octocore::engine::REALTIME_PER_RENDER;
use octocore::types::{Realtime, RealtimeEvent, StepAttr};
use octocore::{Command, Engine, Event, EventBuffer, RenderContext};

/// Pairs whose samples per tick are exact binary fractions, so an exact sample position is a fact and not a rounding. 40 BPM at
/// 44.1 kHz is 344.53125 samples a tick; 120 at 48 kHz is 125; 240 at 48 kHz is 62.5; 120 at 44.1 kHz is 114.84375.
const PAIRS: [(f32, f32); 5] = [(40.0, 44_100.0), (120.0, 48_000.0), (240.0, 48_000.0), (120.0, 44_100.0), (240.0, 44_100.0)];

fn spt(bpm: f32, sample_rate: f32) -> f64 {
    sample_rate as f64 * 60.0 / (bpm as f64 * TICKS_PER_QUARTER as f64)
}

/// One note every step on track 0, so the engine has notes to flush.
fn engine_with_notes() -> Engine {
    let mut e = Engine::new(1);
    for s in 0..16u8 {
        e.grid.set_step_attr(0, s, StepAttr::Active, 1);
    }
    e
}

/// A host that renders blocks and keeps both lists with absolute sample positions.
struct Rig {
    e: Engine,
    sample_rate: f32,
    bpm: f32,
    playing: bool,
    /// Samples rendered so far: the start of the next block.
    at: u64,
    events: Vec<(u64, Event)>,
    realtime: Vec<(u64, Realtime)>,
}

impl Rig {
    fn new(e: Engine, bpm: f32, sample_rate: f32) -> Rig {
        Rig { e, sample_rate, bpm, playing: false, at: 0, events: vec![], realtime: vec![] }
    }

    /// Renders `len` samples. Returns what this block produced, as (absolute sample, message) and the events.
    fn render(&mut self, len: u32) -> (Vec<(u64, Realtime)>, Vec<(u64, Event)>) {
        let ctx = RenderContext { sample_rate: self.sample_rate, buffer_len: len, bpm: self.bpm, playing: self.playing };
        let mut buf = EventBuffer::new();
        self.e.render(&ctx, &mut buf);
        let mut rt = [RealtimeEvent { msg: Realtime::Clock, at_sample: 0 }; REALTIME_PER_RENDER];
        let n = self.e.take_realtime(&mut rt);
        let r: Vec<(u64, Realtime)> = rt[..n].iter().map(|x| (self.at + x.at_sample as u64, x.msg)).collect();
        let ev: Vec<(u64, Event)> = buf.as_slice().iter().map(|x| (self.at + at_sample(x) as u64, *x)).collect();
        self.at += len as u64;
        self.realtime.extend(r.iter().copied());
        self.events.extend(ev.iter().copied());
        (r, ev)
    }

    fn run(&mut self, samples: u64, block: u32) {
        let mut left = samples;
        while left > 0 {
            let n = left.min(block as u64) as u32;
            self.render(n);
            left -= n as u64;
        }
    }

    fn clocks(&self) -> Vec<u64> {
        self.realtime.iter().filter(|(_, m)| *m == Realtime::Clock).map(|(t, _)| *t).collect()
    }
}

fn at_sample(e: &Event) -> u32 {
    match *e {
        Event::NoteOn { at_sample, .. } | Event::NoteOff { at_sample, .. } | Event::Cc { at_sample, .. } | Event::PitchBend { at_sample, .. } | Event::ChannelPressure { at_sample, .. } => at_sample,
    }
}

// ------------------------------------------------------------------------------------------------ the constants

#[test]
fn a_clock_pulse_is_ticks_per_quarter_over_24_ticks_which_is_8_until_d0_is_answered() {
    // D0 (`WENGE-0012`): the tick is 192 to the quarter note, which is four times short of the manual's own. If the owner changes it,
    // this changes to match and the compile-time check in `domain.rs` says whether the new tick divides into pulses.
    assert_eq!(TICKS_PER_QUARTER, 192);
    assert_eq!(TICKS_PER_CLOCK, 8);
    assert_eq!(TICKS_PER_CLOCK * 24, TICKS_PER_QUARTER);
}

#[test]
fn the_wire_bytes_of_the_four_messages_are_midi_status_bytes() {
    assert_eq!(Realtime::Clock as u8, 0xF8);
    assert_eq!(Realtime::Start as u8, 0xFA);
    assert_eq!(Realtime::Continue as u8, 0xFB);
    assert_eq!(Realtime::Stop as u8, 0xFC);
}

// ------------------------------------------------------------------------------------------------ the clock is off by default

#[test]
fn with_the_clock_off_nothing_is_produced_and_the_notes_are_as_they_were() {
    let mut off = Rig::new(engine_with_notes(), 120.0, 48_000.0);
    off.playing = true;
    off.run(2 * 48_000, 128);
    assert!(off.realtime.is_empty(), "the clock is off by default");

    // And turning it on changes no note, no sample position and no order: the clock is beside the events, not among them.
    let mut on = Rig::new(engine_with_notes(), 120.0, 48_000.0);
    on.e.set_clock_master(true);
    on.playing = true;
    on.run(2 * 48_000, 128);
    assert!(!on.realtime.is_empty());
    assert!(off.events.len() > 20, "the test should hear notes ({})", off.events.len());
    assert_eq!(on.events, off.events, "the same events at the same samples with the clock on");
}

#[test]
fn take_realtime_hands_over_what_the_last_render_made_and_clears_it() {
    let mut e = engine_with_notes();
    e.set_clock_master(true);
    let ctx = RenderContext { sample_rate: 48_000.0, buffer_len: 1000, bpm: 120.0, playing: true };
    let mut buf = EventBuffer::new();
    e.render(&ctx, &mut buf);
    let mut out = [RealtimeEvent { msg: Realtime::Clock, at_sample: 0 }; REALTIME_PER_RENDER];
    let n = e.take_realtime(&mut out);
    assert!(n >= 2, "Start and at least one Clock in 1000 samples at 120 BPM ({n})");
    assert_eq!(e.take_realtime(&mut out), 0, "taken once");
}

#[test]
fn a_render_that_is_not_taken_does_not_pile_up_into_the_next() {
    // Offsets are relative to the render that made them, so a message left behind would be stamped a block late.
    let mut e = engine_with_notes();
    e.set_clock_master(true);
    let ctx = RenderContext { sample_rate: 48_000.0, buffer_len: 1000, bpm: 120.0, playing: true };
    let mut buf = EventBuffer::new();
    e.render(&ctx, &mut buf);
    buf.clear();
    e.render(&ctx, &mut buf);
    let mut out = [RealtimeEvent { msg: Realtime::Clock, at_sample: 0 }; REALTIME_PER_RENDER];
    let n = e.take_realtime(&mut out);
    assert!(out[..n].iter().all(|x| x.at_sample < 1000));
    assert!(out[..n].iter().all(|x| x.msg == Realtime::Clock), "the first render's Start was dropped with it, not carried over");
}

// ------------------------------------------------------------------------------------------------ where the pulses fall

#[test]
fn pulses_fall_on_the_samples_of_every_eighth_tick_at_each_tempo_and_rate_and_block_size() {
    for (bpm, sr) in PAIRS {
        for block in [128u32, 64, 480, 1000, 1] {
            let mut r = Rig::new(Engine::new(1), bpm, sr);
            r.e.set_clock_master(true);
            r.playing = true;
            // 40 BPM is 16 pulses a second, so one second of one-sample blocks is not the 20 pulses the check below wants.
            let secs = if block == 1 { 2 } else { 4 };
            r.run(secs * sr as u64, block);
            let clocks = r.clocks();
            assert!(clocks.len() >= 20, "{bpm} BPM at {sr}: {} pulses", clocks.len());
            let step = spt(bpm, sr);
            for (m, got) in clocks.iter().enumerate() {
                let want = (m as f64 * TICKS_PER_CLOCK as f64 * step).floor() as u64;
                assert_eq!(*got, want, "pulse {m} at {bpm} BPM, {sr} Hz, blocks of {block}");
            }
        }
    }
}

#[test]
fn a_pulse_is_a_24th_of_a_quarter_note_apart() {
    let (bpm, sr) = (120.0f32, 48_000.0f32);
    let mut r = Rig::new(Engine::new(1), bpm, sr);
    r.e.set_clock_master(true);
    r.playing = true;
    r.run(sr as u64, 128);
    let c = r.clocks();
    // 120 BPM is 500 ms a quarter note, so 24 pulses are 24,000 samples at 48 kHz.
    assert_eq!(c[24] - c[0], (sr as u64) * 60 / bpm as u64, "24 pulses are one quarter note");
}

#[test]
fn start_comes_before_the_first_clock_and_both_are_on_the_first_ticks_sample() {
    let mut r = Rig::new(Engine::new(1), 120.0, 48_000.0);
    r.e.set_clock_master(true);
    r.playing = true;
    let (rt, _) = r.render(128);
    assert_eq!(rt[0], (0, Realtime::Start));
    assert_eq!(rt[1], (0, Realtime::Clock));
}

#[test]
fn the_first_note_follows_start_by_the_eleven_ticks_the_engine_already_waits() {
    // `tests/conformance/AMBIGUITIES.md`, "the first note after Play": a step fires when its tick counter reaches the step length,
    // so the first note sounds 11 ticks after Play. The clock does not hide that; a drum machine on this clock plays its first step
    // on Start and this engine's first note is 11 ticks (a pulse and three eighths) behind it. A fact for the owner, not a bug here.
    let mut r = Rig::new(engine_with_notes(), 120.0, 48_000.0);
    r.e.set_clock_master(true);
    r.playing = true;
    r.run(48_000, 128);
    let first_note = r.events.iter().find(|(_, e)| matches!(e, Event::NoteOn { .. })).expect("a note").0;
    let first_clock = r.clocks()[0];
    assert_eq!(first_note - first_clock, (spt(120.0, 48_000.0) * (DEFAULT_STEP_TICKS as f64 - 1.0)) as u64);
}

// ------------------------------------------------------------------------------------------------ stop, continue, reset

#[test]
fn stop_sends_stop_at_the_sample_it_takes_effect_and_no_pulse_follows_even_one_stepped_ahead() {
    let mut r = Rig::new(engine_with_notes(), 120.0, 48_000.0);
    r.e.set_clock_master(true);
    r.playing = true;
    r.run(24_000, 128);
    let before = r.realtime.len();
    // Stop between renders: it takes effect at the start of the next one.
    r.e.handle_command(Command::Stop);
    r.playing = false;
    let stop_at = r.at;
    let (rt, ev) = r.render(128);
    assert_eq!(rt, vec![(stop_at, Realtime::Stop)], "one Stop, on the first sample of the block");
    assert!(ev.iter().any(|(t, e)| *t == stop_at && matches!(e, Event::NoteOff { .. })), "the flush Note Offs are on the same sample as the Stop; the page puts the Stop first");
    r.run(2 * 48_000, 128);
    assert_eq!(r.realtime.len(), before + 1, "nothing but the Stop after the run: the pulses the engine had stepped ahead are dropped");
}

#[test]
fn stop_while_stopped_sends_no_realtime_and_keeps_its_all_notes_off() {
    // ADR-0009 D-P4-7: unchanged. The engine already sends CC 123 on all 32 channels and does not ask whether a clock is on.
    let mut r = Rig::new(Engine::new(1), 120.0, 48_000.0);
    r.e.set_clock_master(true);
    r.e.handle_command(Command::Stop);
    let (rt, ev) = r.render(128);
    assert!(rt.is_empty());
    assert_eq!(ev.iter().filter(|(_, e)| matches!(e, Event::Cc { cc: 123, .. })).count(), 32);
}

#[test]
fn play_after_a_stop_sends_continue_and_the_pulses_carry_on_from_where_the_sound_stopped() {
    // Play after a Stop carries on from the point the sound stopped (ADR-0009 amendment 3; before it, from the tick the engine had stepped
    // to, up to a step further on). So the message is Continue, and the pulses stay on every eighth tick of the engine's own count, not of
    // the run, with the first one as far from the Play as it was from the Stop.
    let (bpm, sr) = (120.0f32, 48_000.0f32);
    let mut r = Rig::new(engine_with_notes(), bpm, sr);
    r.e.set_clock_master(true);
    r.playing = true;
    r.run(10_000 + 3 * 125 + 60, 128); // tick 83 and a half
    r.e.handle_command(Command::Stop);
    r.playing = false;
    r.run(2_000, 128);
    let p = r.e.tick_position(); // where the audio stopped, in ticks
    assert!((p - (83.0 + 60.0 / 125.0)).abs() < 1e-9, "the position the sound stopped at, not the engine's own count ({})", r.e.tick());
    r.e.handle_command(Command::Play);
    r.playing = true;
    let play_at = r.at;
    let (rt, _) = r.render(1_000); // a pulse is at most 7 ticks, 875 samples, from the first tick
    assert_eq!(rt[0], (play_at, Realtime::Continue), "not Start: the engine is not at its beginning");
    let next_pulse_tick = (p / TICKS_PER_CLOCK as f64).ceil() * TICKS_PER_CLOCK as f64;
    let first_clock = rt.iter().find(|(_, m)| *m == Realtime::Clock).expect("a clock in the first block").0;
    assert_eq!(first_clock, play_at + ((next_pulse_tick - p) * spt(bpm, sr)).round() as u64, "the first pulse is on the engine's own grid (tick {next_pulse_tick}), {} ticks after the position it stopped at", next_pulse_tick - p);
}

#[test]
fn reset_then_play_sends_start_again_and_a_reset_while_running_sends_stop() {
    let mut r = Rig::new(engine_with_notes(), 120.0, 48_000.0);
    r.e.set_clock_master(true);
    r.playing = true;
    r.run(10_000, 128);
    r.e.handle_command(Command::Reset);
    r.playing = false;
    let (rt, _) = r.render(128);
    assert_eq!(rt.iter().filter(|(_, m)| *m == Realtime::Stop).count(), 1, "a reset while running is a stop");
    assert!(rt.iter().all(|(_, m)| *m != Realtime::Clock));
    r.e.handle_command(Command::Play);
    r.playing = true;
    let (rt, _) = r.render(128);
    assert_eq!(rt[0].1, Realtime::Start, "at tick 0 again");
    assert_eq!(rt[1].1, Realtime::Clock);
}

#[test]
fn a_stop_and_a_play_before_the_next_render_come_out_in_that_order_on_one_sample() {
    let mut r = Rig::new(engine_with_notes(), 120.0, 48_000.0);
    r.e.set_clock_master(true);
    r.playing = true;
    r.run(10_000, 128);
    r.e.handle_command(Command::Stop);
    r.e.handle_command(Command::Play);
    let at = r.at;
    let (rt, _) = r.render(128);
    assert_eq!(rt[0], (at, Realtime::Stop));
    assert_eq!(rt[1], (at, Realtime::Continue));
}

#[test]
fn turning_the_clock_on_while_running_says_where_the_sequencer_is_and_then_pulses() {
    let (bpm, sr) = (120.0f32, 48_000.0f32);
    let mut r = Rig::new(engine_with_notes(), bpm, sr);
    r.playing = true;
    r.run(10_000, 128);
    assert!(r.realtime.is_empty());
    r.e.set_clock_master(true);
    let on_at = r.at;
    let (rt, _) = r.render(128);
    assert_eq!(rt[0], (on_at, Realtime::Continue), "running and not at tick 0");
    r.run(4_000, 128);
    assert!(!r.clocks().is_empty(), "pulses follow");
    // Turning it off drops the pulses already stepped ahead and sends nothing.
    r.e.set_clock_master(false);
    let n = r.realtime.len();
    r.run(48_000, 128);
    assert_eq!(r.realtime.len(), n);
}

// ------------------------------------------------------------------------------------------------ tempo changes and odd hosts

#[test]
fn pulses_follow_a_tempo_change_and_never_bunch_or_repeat() {
    let mut r = Rig::new(Engine::new(1), 120.0, 48_000.0);
    r.e.set_clock_master(true);
    r.playing = true;
    for i in 0..400 {
        r.bpm = 100.0 + (i % 40) as f32 * 3.0; // a new tempo every block
        r.render(128);
    }
    let c = r.clocks();
    assert!(c.len() > 30);
    assert!(c.windows(2).all(|w| w[1] > w[0]), "strictly increasing");
}

#[test]
fn an_unusable_tempo_makes_no_pulse_and_leaves_none_to_replay() {
    let mut r = Rig::new(Engine::new(1), 120.0, 48_000.0);
    r.e.set_clock_master(true);
    r.playing = true;
    r.run(10_000, 128);
    let n = r.clocks().len();
    r.bpm = 0.0;
    r.run(10_000, 128);
    assert!(r.clocks().len() <= n + 2, "no tick, no new pulse: only the one or two already stepped ahead of the audio come out ({} then {})", n, r.clocks().len());
    r.bpm = 120.0;
    r.run(1_000, 128);
    let c = r.clocks();
    assert!(c.windows(2).all(|w| w[1] - w[0] >= 900), "no backlog replayed in a burst: {:?}", &c[n.saturating_sub(1)..]);
}

#[test]
fn realtime_that_does_not_fit_the_callers_array_is_dropped_newest_first_and_counted() {
    let mut e = Engine::new(1);
    e.set_clock_master(true);
    // A long block at a fast tempo has more pulses than a one-slot array holds.
    let ctx = RenderContext { sample_rate: 48_000.0, buffer_len: 4_096, bpm: 300.0, playing: true };
    let mut buf = EventBuffer::new();
    e.render(&ctx, &mut buf);
    let mut one = [RealtimeEvent { msg: Realtime::Clock, at_sample: 0 }; 1];
    assert_eq!(e.take_realtime(&mut one), 1);
    assert_eq!(one[0].msg, Realtime::Start, "the oldest is kept");
    assert!(e.realtime_dropped() > 0);
}

#[test]
fn one_render_that_makes_more_messages_than_the_engine_keeps_loses_the_newest_and_counts_them() {
    // 20,000 frames at 8 kHz and 999 BPM is a pulse every 20 samples: about a thousand pulses in one render, where the engine keeps
    // `REALTIME_PER_RENDER`. The same time in blocks of 1,000 makes the whole list with nothing lost, to compare with.
    let (bpm, sr) = (999.0f32, 8_000.0f32);
    let mut whole = Rig::new(Engine::new(1), bpm, sr);
    whole.e.set_clock_master(true);
    whole.playing = true;
    whole.run(20_000, 1_000);
    assert_eq!(whole.e.realtime_dropped(), 0, "blocks of 1,000 lose nothing");
    assert!(whole.realtime.len() > REALTIME_PER_RENDER * 2, "the test needs a render with more than the engine keeps: {} messages", whole.realtime.len());

    let mut one = Rig::new(Engine::new(1), bpm, sr);
    one.e.set_clock_master(true);
    one.playing = true;
    one.render(20_000);
    assert_eq!(one.realtime.len(), REALTIME_PER_RENDER, "the engine keeps its capacity and no more");
    assert_eq!(one.realtime[..], whole.realtime[..REALTIME_PER_RENDER], "and keeps the earliest, in order");
    assert_eq!(one.e.realtime_dropped() as usize, whole.realtime.len() - REALTIME_PER_RENDER, "every other message is counted");
}

// ------------------------------------------------------------------------------------------------ the tick position

#[test]
fn tick_position_is_the_audio_position_at_the_end_of_the_render_and_is_whole_on_a_tick() {
    // 120 BPM at 48 kHz is 125 samples a tick. The engine steps ticks ahead of the audio, so its tick count is more than this.
    let mut r = Rig::new(Engine::new(1), 120.0, 48_000.0);
    r.playing = true;
    for k in 1..=40u32 {
        r.render(125);
        let p = r.e.tick_position();
        assert!((p - k as f64).abs() < 1e-9, "after {k} ticks of audio the position is {p}");
    }
    assert!(r.e.tick() as f64 > r.e.tick_position(), "the engine is stepped ahead of what has played");
}

#[test]
fn tick_position_moves_by_a_fraction_between_ticks() {
    let mut r = Rig::new(Engine::new(1), 120.0, 48_000.0);
    r.playing = true;
    r.render(125 * 4);
    r.render(60);
    assert!((r.e.tick_position() - (4.0 + 60.0 / 125.0)).abs() < 1e-9);
}

#[test]
fn tick_position_never_goes_back_over_blocks_and_tempo_changes() {
    let mut r = Rig::new(Engine::new(1), 120.0, 48_000.0);
    r.playing = true;
    let mut last = 0.0;
    for i in 0..600 {
        r.bpm = if i % 50 < 25 { 60.0 } else { 240.0 }; // jumps by a factor of four
        r.render(128);
        let p = r.e.tick_position();
        assert!(p >= last, "block {i}: {p} after {last}");
        last = p;
    }
    assert!(last > 100.0);
}

#[test]
fn tick_position_while_stopped_is_where_the_audio_stopped_and_reading_it_changes_nothing() {
    let mut a = Rig::new(engine_with_notes(), 120.0, 48_000.0);
    let mut b = Rig::new(engine_with_notes(), 120.0, 48_000.0);
    a.playing = true;
    b.playing = true;
    for _ in 0..300 {
        a.render(128);
        let _ = b.e.tick_position();
        b.render(128);
    }
    assert_eq!(a.events, b.events, "a read is a read");
    let playing = a.e.tick_position();
    a.playing = false;
    a.render(128);
    let stopped = a.e.tick_position();
    a.render(128);
    assert_eq!(a.e.tick_position(), stopped, "frozen while stopped");
    assert_eq!(stopped, playing, "where the audio was when it stopped, not the engine's own count, which is stepped ahead of it");
    assert!((a.e.tick() as f64) > stopped);

    // And Play carries on from there: no jump forward or back, however long it was stopped.
    a.run(5 * 128, 128);
    a.playing = true;
    a.render(125);
    assert!((a.e.tick_position() - (stopped + 1.0)).abs() < 1e-9, "one tick of audio after the Play, one tick on from where it stopped ({} from {stopped})", a.e.tick_position());

    // After a Reset it is the engine's own count again.
    a.e.handle_command(Command::Reset);
    a.playing = false;
    a.render(128);
    assert_eq!(a.e.tick_position(), 0.0);
}

// ------------------------------------------------------------------------------------------------ pending on D0

#[test]
#[ignore = "pending on D0 (WENGE-0012): with a 192-tick quarter a step is 12 ticks, which is 1.5 pulses; a step is a whole number of pulses only if the tick is 48 to the quarter note. Plan observable M3."]
fn a_step_is_a_whole_number_of_pulses() {
    assert_eq!(DEFAULT_STEP_TICKS % TICKS_PER_CLOCK, 0, "a step must be a whole number of pulses");
}

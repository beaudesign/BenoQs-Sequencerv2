//! Tests for the engine's link: commands in through the ring, snapshots out through the
//! buffer (SPEC-0001 O6, WENGE-0006, tests C3 to C7, E1 to E3). One thread; the two-thread
//! behaviour is in `soak.rs` and `loom_sync.rs`.

use octocore::domain::{Mode, STEP_COUNT, TRACK_COUNT};
use octocore::link::{COMMAND_RING_CAPACITY, MAX_COMMANDS_PER_RENDER};
use octocore::rng::Rng;
use octocore::snapshot::{write_words, SNAPSHOT_WORDS};
use octocore::types::{Snapshot, StepAttr, TrackAttr};
use octocore::{Command, Engine, Event, EventBuffer, RenderContext};

const SAMPLE_RATE: f32 = 48_000.0;
const BUFFER: u32 = 256;

/// One edit of the pattern, applied either directly or as a command.
#[derive(Clone, Copy, Debug)]
enum Edit {
    Track(u8, TrackAttr, i32),
    Step(u8, u8, StepAttr, i32),
}

impl Edit {
    fn command(self) -> Command {
        match self {
            Edit::Track(track, attr, value) => Command::SetTrack { track, attr, value },
            Edit::Step(track, step, attr, value) => Command::SetStep { track, step, attr, value },
        }
    }

    fn apply_directly(self, e: &mut Engine) {
        match self {
            Edit::Track(track, attr, value) => assert!(e.grid.set_track_attr(track, attr, value)),
            Edit::Step(track, step, attr, value) => assert!(e.grid.set_step_attr(track, step, attr, value)),
        }
    }
}

/// A random pattern: some steps on, and a few track and step attributes changed.
fn random_edits(seed: u64) -> Vec<Edit> {
    let mut rng = Rng::new(seed ^ 0xC0FFEE);
    let mut edits = Vec::new();
    for track in 0..TRACK_COUNT as u8 {
        edits.push(Edit::Track(track, TrackAttr::Pitch, 36 + rng.next_below(48) as i32));
        edits.push(Edit::Track(track, TrackAttr::MidiChannel, 1 + rng.next_below(16) as i32));
        for step in 0..STEP_COUNT as u8 {
            if rng.next_below(3) == 0 {
                edits.push(Edit::Step(track, step, StepAttr::Active, 1));
                if rng.next_below(2) == 0 {
                    edits.push(Edit::Step(track, step, StepAttr::LengthTicks, 1 + rng.next_below(192) as i32));
                }
                if rng.next_below(3) == 0 {
                    edits.push(Edit::Step(track, step, StepAttr::PitchOffset, rng.next_below(25) as i32 - 12));
                }
            }
        }
    }
    edits
}

/// Drives an engine like a host does, recording every event with its absolute sample.
struct Host {
    clock: u64,
    events: Vec<(u64, Event)>,
}

impl Host {
    fn new() -> Host {
        Host { clock: 0, events: Vec::new() }
    }

    fn render(&mut self, e: &mut Engine, buffers: usize, playing: bool) {
        let ctx = RenderContext { sample_rate: SAMPLE_RATE, buffer_len: BUFFER, bpm: 120.0, playing };
        let mut out = EventBuffer::new();
        for _ in 0..buffers {
            out.clear();
            e.render(&ctx, &mut out);
            for ev in out.as_slice() {
                self.events.push((self.clock + at_sample(ev) as u64, *ev));
            }
            self.clock += BUFFER as u64;
        }
    }
}

fn at_sample(e: &Event) -> u32 {
    match *e {
        Event::NoteOn { at_sample, .. }
        | Event::NoteOff { at_sample, .. }
        | Event::Cc { at_sample, .. }
        | Event::PitchBend { at_sample, .. }
        | Event::ChannelPressure { at_sample, .. } => at_sample,
    }
}

fn note_ons(events: &[(u64, Event)]) -> usize {
    events.iter().filter(|(_, e)| matches!(e, Event::NoteOn { .. })).count()
}

#[test]
fn the_link_can_be_opened_once() {
    let mut e = Engine::new(1);
    assert!(e.open_link().is_some());
    assert!(e.open_link().is_none());
}

#[test]
fn c3_edits_through_the_ring_sound_exactly_like_the_same_edits_made_directly() {
    let mut with_notes = 0;
    for seed in 0..12u64 {
        let edits = random_edits(seed);
        assert!(edits.len() <= COMMAND_RING_CAPACITY, "the ring must hold the whole pattern");

        let mut direct = Engine::new(seed);
        edits.iter().for_each(|ed| ed.apply_directly(&mut direct));
        let mut a = Host::new();
        a.render(&mut direct, 96, true);

        let mut via_ring = Engine::new(seed);
        let (mut tx, _rx) = via_ring.open_link().unwrap();
        edits.iter().for_each(|ed| tx.push(ed.command()));
        let mut b = Host::new();
        b.render(&mut via_ring, 96, true);

        assert_eq!(a.events, b.events, "seed {seed}");
        assert_eq!(tx.stats().dropped, 0);
        if note_ons(&a.events) > 0 {
            with_notes += 1;
        }
    }
    assert!(with_notes >= 10, "the patterns must actually sound, or this test proves nothing");
}

#[test]
fn c4_commands_in_the_first_drain_take_effect_before_the_first_tick_is_stepped() {
    // The engine steps 12 ticks ahead of the audio (`MAX_EARLY_TICKS`), so the first note of
    // a fresh engine is decided in the very first render, though it sounds about 1,400
    // samples later. A command that arrives in that first drain must be in time for it.
    let mut quiet = Engine::new(3);
    let mut h = Host::new();
    h.render(&mut quiet, 8, true);
    assert_eq!(note_ons(&h.events), 0, "control: an untouched engine plays nothing in eight buffers");

    let mut direct = Engine::new(3);
    direct.grid.set_step_attr(0, 0, StepAttr::Active, 1);
    let mut want = Host::new();
    want.render(&mut direct, 8, true);
    assert!(note_ons(&want.events) >= 1, "control: the same edit made directly sounds");

    let mut e = Engine::new(3);
    let (mut tx, _rx) = e.open_link().unwrap();
    tx.push(Command::SetStep { track: 0, step: 0, attr: StepAttr::Active, value: 1 });
    tx.push(Command::Play);
    let mut h = Host::new();
    h.render(&mut e, 8, true);
    assert_eq!(h.events, want.events, "the ring edit lands on exactly the sample the direct edit does");
}

fn pitch_after(k: usize) -> i32 {
    1 + (k % 100) as i32
}

#[test]
fn c5_a_render_applies_at_most_the_limit_and_the_next_render_carries_on() {
    let mut e = Engine::new(1);
    let (mut tx, _rx) = e.open_link().unwrap();
    for k in 1..=COMMAND_RING_CAPACITY {
        tx.push(Command::SetTrack { track: 0, attr: TrackAttr::Pitch, value: pitch_after(k) });
    }
    let mut h = Host::new();
    let mut applied = 0;
    while applied < COMMAND_RING_CAPACITY {
        h.render(&mut e, 1, false);
        applied += MAX_COMMANDS_PER_RENDER;
        assert_eq!(tx.stats().received as usize, applied.min(COMMAND_RING_CAPACITY));
        assert_eq!(e.grid.track_attr(0, TrackAttr::Pitch), pitch_after(applied.min(COMMAND_RING_CAPACITY)), "after {applied} commands");
    }
    assert_eq!(tx.stats().dropped, 0);
    assert_eq!(MAX_COMMANDS_PER_RENDER * 4, COMMAND_RING_CAPACITY, "the arithmetic above assumes four renders");
}

#[test]
fn c5_a_flood_beyond_the_ring_keeps_the_newest_commands_and_counts_the_rest() {
    let mut e = Engine::new(1);
    let (mut tx, _rx) = e.open_link().unwrap();
    let total = 10 * MAX_COMMANDS_PER_RENDER;
    for k in 1..=total {
        tx.push(Command::SetTrack { track: 0, attr: TrackAttr::Pitch, value: pitch_after(k) });
    }
    let mut h = Host::new();
    for _ in 0..6 {
        h.render(&mut e, 1, false);
    }
    let s = tx.stats();
    assert_eq!(s.pushed as usize, total);
    assert_eq!(s.dropped as usize, total - COMMAND_RING_CAPACITY);
    assert_eq!(s.received as usize, COMMAND_RING_CAPACITY);
    assert_eq!(e.grid.track_attr(0, TrackAttr::Pitch), pitch_after(total), "the newest command is the one that stands");
}

#[test]
fn c6_stop_through_the_ring_flushes_sounding_notes_like_stop_through_handle_command() {
    fn scene() -> Engine {
        let mut e = Engine::new(9);
        for track in 0..3u8 {
            e.grid.set_track_attr(track, TrackAttr::Pitch, 40 + track as i32);
            e.grid.set_step_attr(track, 0, StepAttr::Active, 1);
            e.grid.set_step_attr(track, 0, StepAttr::LengthTicks, 192);
            e.grid.set_step_attr(track, 0, StepAttr::LengthMultiplier, 8);
        }
        e
    }

    let mut direct = scene();
    let mut a = Host::new();
    a.render(&mut direct, 8, true);
    let before = a.events.len();
    direct.handle_command(Command::Stop);
    // The host still says "playing", so the only thing that can flush the notes is the Stop
    // command itself (a host that said "stopped" would flush them without any command).
    a.render(&mut direct, 4, true);

    let mut linked = scene();
    let (mut tx, _rx) = linked.open_link().unwrap();
    let mut b = Host::new();
    b.render(&mut linked, 8, true);
    tx.push(Command::Stop);
    b.render(&mut linked, 4, true);

    assert_eq!(a.events, b.events);
    let flush_a = a.events[before..].iter().filter(|(_, e)| matches!(e, Event::NoteOff { .. })).count();
    let flush_b = b.events[before..].iter().filter(|(_, e)| matches!(e, Event::NoteOff { .. })).count();
    assert!(flush_a > 0, "the scene must have notes sounding when Stop arrives, or this proves nothing");
    assert_eq!(flush_a, flush_b, "the Stop that came through the ring flushed the same notes");
}

#[test]
fn c7_words_that_are_not_a_command_are_counted_as_dropped_and_never_applied() {
    let mut e = Engine::new(1);
    let (mut tx, _rx) = e.open_link().unwrap();
    tx.push_words([99, 1, 2]);
    tx.push(Command::SetTrack { track: 0, attr: TrackAttr::Pitch, value: 77 });
    tx.push_words([8, 4, 0]); // SetMode with a mode that does not exist
    let mut h = Host::new();
    h.render(&mut e, 1, false);
    assert_eq!(e.grid.track_attr(0, TrackAttr::Pitch), 77);
    assert_eq!(e.grid.mode, Mode::Grid);
    let s = tx.stats();
    assert_eq!((s.pushed, s.received, s.dropped), (3, 1, 2));
}

#[test]
fn e1_the_generation_is_the_number_of_renders() {
    let mut e = Engine::new(1);
    let (_tx, mut rx) = e.open_link().unwrap();
    assert!(!rx.claim(), "nothing is published before the first render");
    assert_eq!(rx.snapshot().generation, 0);
    let mut h = Host::new();
    for n in 1..=5u64 {
        h.render(&mut e, 1, true);
        assert!(rx.claim());
        assert_eq!(rx.snapshot().generation, n);
        assert!(!rx.claim(), "one publish per render");
    }
    h.render(&mut e, 3, true);
    assert!(rx.claim());
    assert_eq!(rx.snapshot().generation, 8, "a reader that skips renders sees the newest");
}

fn words(s: &Snapshot) -> Vec<u32> {
    let mut w = [0u32; SNAPSHOT_WORDS];
    write_words(s, &mut w);
    w.to_vec()
}

#[test]
fn e2_the_published_snapshot_is_what_the_engine_says_about_itself() {
    let mut e = Engine::new(5);
    let (mut tx, mut rx) = e.open_link().unwrap();
    for track in 0..4u8 {
        for step in 0..STEP_COUNT as u8 {
            tx.push(Command::SetStep { track, step, attr: StepAttr::Active, value: 1 });
        }
    }
    tx.push(Command::SetMode { mode: Mode::Track });
    tx.push(Command::SetActivePage { bank: 2, page: 3 });
    let mut h = Host::new();
    h.render(&mut e, 10, true);

    assert!(rx.claim());
    let s = *rx.snapshot();
    assert_eq!(s.generation, 10);
    assert_eq!(s.mode, Mode::Track);
    assert_eq!((s.active.bank, s.active.page), (2, 3));
    assert!(s.transport.playing);
    assert_eq!(s.transport.tick, e.tick());
    assert!(s.transport.tick > 0);
    for (i, p) in s.playheads.iter().enumerate() {
        assert_eq!(p.track_index as usize, i);
    }
    assert!(s.playheads.iter().any(|p| p.step_index != 0), "ten buffers is more than a step: the playheads have moved");

    let mut direct = Snapshot::zeroed();
    e.snapshot(&mut direct);
    direct.generation = s.generation;
    assert_eq!(words(&direct), words(&s), "the published snapshot is the engine's own, bit for bit");

    h.render(&mut e, 1, false);
    assert!(rx.claim());
    assert!(!rx.snapshot().transport.playing);
    assert_eq!(rx.snapshot().generation, 11);
}

#[test]
fn e3_an_engine_with_a_link_and_one_without_produce_the_same_events() {
    for seed in 0..12u64 {
        let edits = random_edits(seed);
        let mut plain = Engine::new(seed);
        edits.iter().for_each(|ed| ed.apply_directly(&mut plain));
        let mut a = Host::new();
        a.render(&mut plain, 96, true);
        a.render(&mut plain, 8, false);

        let mut linked = Engine::new(seed);
        edits.iter().for_each(|ed| ed.apply_directly(&mut linked));
        let (_tx, mut rx) = linked.open_link().unwrap();
        let mut b = Host::new();
        b.render(&mut linked, 96, true);
        b.render(&mut linked, 8, false);
        rx.claim();

        assert_eq!(a.events, b.events, "seed {seed}");
        assert_eq!(plain.diagnostics(), linked.diagnostics(), "seed {seed}");
    }
}

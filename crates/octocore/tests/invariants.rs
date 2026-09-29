//! Metamorphic and property tests for the engine's output stream (SPEC-0001 O9,
//! WENGE-0009). Each property holds for every pattern and every host behaviour, so one
//! test covers thousands of scenarios where a hand-written fixture covers one.
//!
//! No external dependencies: patterns come from octocore's own seeded `Rng`, so a failure
//! prints the seed and can be replayed exactly. Bump `SEEDS` to search harder; the default
//! keeps `just verify` fast.

use std::collections::BTreeMap;

use octocore::domain::{STEP_COUNT, TRACK_COUNT};
use octocore::rng::Rng;
use octocore::types::Command;
use octocore::{Engine, Event, EventBuffer, RenderContext};

const SAMPLE_RATE: f32 = 48_000.0;
const SEEDS: u64 = 96;

/// Drives an engine like a host would and records every event with its absolute sample.
struct Host {
    engine: Engine,
    bpm: f32,
    clock: u64,
    events: Vec<(u64, Event)>,
}

impl Host {
    fn new(engine: Engine, bpm: f32) -> Host {
        Host { engine, bpm, clock: 0, events: Vec::new() }
    }

    /// One render call. Returns the events of just this call.
    fn render(&mut self, buffer_len: u32, playing: bool) -> Vec<(u64, Event)> {
        let mut out = EventBuffer::new();
        let ctx = RenderContext { sample_rate: SAMPLE_RATE, buffer_len, bpm: self.bpm, playing };
        self.engine.render(&ctx, &mut out);
        let got: Vec<(u64, Event)> = out.as_slice().iter().map(|e| (self.clock + at_sample(e) as u64, *e)).collect();
        self.events.extend(got.iter().copied());
        self.clock += buffer_len as u64;
        got
    }

    fn render_seconds(&mut self, seconds: f64, buffer_len: u32, playing: bool) {
        let n = (seconds * SAMPLE_RATE as f64 / buffer_len as f64).ceil() as u64;
        for _ in 0..n {
            self.render(buffer_len, playing);
        }
    }
}

fn at_sample(e: &Event) -> u32 {
    match *e {
        Event::NoteOn { at_sample, .. } | Event::NoteOff { at_sample, .. } | Event::Cc { at_sample, .. } => at_sample,
    }
}

/// NoteOns minus NoteOffs per (port, channel, note). Empty when everything balances.
fn imbalance(events: &[(u64, Event)]) -> BTreeMap<(u8, u8, u8), i64> {
    let mut net: BTreeMap<(u8, u8, u8), i64> = BTreeMap::new();
    for (_, e) in events {
        match *e {
            Event::NoteOn { port, ch, note, .. } => *net.entry((port, ch, note)).or_default() += 1,
            Event::NoteOff { port, ch, note, .. } => *net.entry((port, ch, note)).or_default() -= 1,
            Event::Cc { .. } => {}
        }
    }
    net.retain(|_, n| *n != 0);
    net
}

/// A dense random pattern from a seed: several tracks, random active steps, random note
/// lengths (long ones matter most: they are still sounding when Stop arrives), strums,
/// start offsets and channels on both ports.
fn random_engine(seed: u64) -> Engine {
    let mut rng = Rng::new(seed ^ 0x5EED);
    let mut engine = Engine::new(seed);
    let page = engine.grid.active_page_mut();
    for t in 0..TRACK_COUNT {
        if rng.next_below(4) == 0 {
            continue; // leave some tracks silent
        }
        let track = &mut page.tracks[t];
        track.midi_channel = 1 + rng.next_below(32) as u8;
        for s in 0..STEP_COUNT {
            let step = &mut track.steps[s];
            step.active = rng.next_below(3) != 0;
            step.length_ticks = 1 + rng.next_below(192) as u8;
            step.length_multiplier = 1 + rng.next_below(4) as u8;
            step.pitch_offset = rng.next_below(25) as i8 - 12;
            step.velocity_offset = rng.next_below(21) as i8 - 10;
            step.start_offset = rng.next_below(5) as i8 - 2;
            step.strum = if rng.next_below(4) == 0 { rng.next_below(19) as i8 - 9 } else { 0 };
        }
    }
    engine
}

fn random_bpm(rng: &mut Rng) -> f32 {
    60.0 + rng.next_below(181) as f32 // 60..=240
}

fn random_buffer(rng: &mut Rng) -> u32 {
    [1u32, 7, 32, 64, 100, 128, 256, 480, 512, 1024, 4096][rng.next_below(11) as usize]
}

// ---------------------------------------------------------------------- WENGE-0001

/// SPEC-0001 A1. After Stop, the next render call turns off every sounding note, so the
/// whole stream balances. Fails on 6ef921f: stop clears the queue and emits nothing.
#[test]
fn stop_at_any_point_leaves_no_note_sounding() {
    let mut failing = Vec::new();
    for seed in 0..SEEDS {
        let mut rng = Rng::new(seed);
        let mut host = Host::new(random_engine(seed), random_bpm(&mut rng));
        let buffers = 1 + rng.next_below(400);
        for _ in 0..buffers {
            let len = random_buffer(&mut rng);
            host.render(len, true);
        }
        host.render(random_buffer(&mut rng), false); // the host stops the transport
        let net = imbalance(&host.events);
        if !net.is_empty() {
            failing.push(format!("seed {seed}: {} notes still sounding, first {:?}", net.len(), net.iter().next().unwrap()));
        }
    }
    assert!(failing.is_empty(), "notes left sounding after Stop:\n{}", failing.join("\n"));
}

/// Same contract when the Stop arrives as a command between render calls.
#[test]
fn stop_command_between_renders_flushes_on_the_next_render() {
    let mut failing = Vec::new();
    for seed in 0..SEEDS {
        let mut rng = Rng::new(seed + 1000);
        let mut host = Host::new(random_engine(seed), random_bpm(&mut rng));
        for _ in 0..(1 + rng.next_below(300)) {
            host.render(random_buffer(&mut rng), true);
        }
        host.engine.handle_command(Command::Stop);
        host.render(random_buffer(&mut rng), false);
        if !imbalance(&host.events).is_empty() {
            failing.push(seed);
        }
    }
    assert!(failing.is_empty(), "seeds with notes left sounding: {failing:?}");
}

#[test]
fn reset_flushes_sounding_notes() {
    let mut failing = Vec::new();
    for seed in 0..SEEDS {
        let mut rng = Rng::new(seed + 2000);
        let mut host = Host::new(random_engine(seed), random_bpm(&mut rng));
        for _ in 0..(1 + rng.next_below(300)) {
            host.render(random_buffer(&mut rng), true);
        }
        host.engine.handle_command(Command::Reset);
        host.render(random_buffer(&mut rng), false);
        if !imbalance(&host.events).is_empty() {
            failing.push(seed);
        }
    }
    assert!(failing.is_empty(), "seeds with notes left sounding after Reset: {failing:?}");
}

/// Stop, play, stop, play ... with random gaps. The stream balances at every stop, not
/// only the last one, and a stopped engine is silent once its flush has gone out.
#[test]
fn stop_play_cycles_never_leak_notes_and_stopped_means_silent() {
    for seed in 0..SEEDS {
        let mut rng = Rng::new(seed + 3000);
        let mut host = Host::new(random_engine(seed), random_bpm(&mut rng));
        for cycle in 0..6 {
            for _ in 0..(1 + rng.next_below(120)) {
                host.render(random_buffer(&mut rng), true);
            }
            host.render(random_buffer(&mut rng), false); // stop lands here, flush goes out
            let net = imbalance(&host.events);
            assert!(net.is_empty(), "seed {seed} cycle {cycle}: still sounding after stop: {net:?}");
            for _ in 0..(1 + rng.next_below(30)) {
                let got = host.render(random_buffer(&mut rng), false);
                assert!(got.is_empty(), "seed {seed} cycle {cycle}: a stopped engine emitted {got:?}");
            }
        }
    }
}

// ---------------------------------------------------------------------- WENGE-0002

/// SPEC-0001 A2. However long the host renders with the transport stopped, the first
/// buffer after Play holds exactly what a fresh engine's first buffer holds. Fails on
/// 6ef921f for any idle time: the idle ticks are replayed in one call.
#[test]
fn first_buffer_after_idle_equals_a_fresh_engines_first_buffer() {
    for seed in [1u64, 2, 3, 11] {
        for buffer in [64u32, 512] {
            let mut fresh = Host::new(random_engine(seed), 133.0);
            let expected: Vec<Event> = fresh.render(buffer, true).into_iter().map(|(_, e)| e).collect();

            for idle_seconds in [0.0f64, 1.0, 10.0, 3600.0] {
                let mut host = Host::new(random_engine(seed), 133.0);
                // Idle in large steps: while stopped the buffer size is irrelevant to the result.
                host.render_seconds(idle_seconds, 48_000, false);
                let got = host.render(buffer, true);
                // Positions are absolute, so compare the buffer-relative events.
                let got: Vec<Event> = got.into_iter().map(|(_, e)| e).collect();
                assert_eq!(got, expected, "seed {seed}, buffer {buffer}, idle {idle_seconds} s");
            }
        }
    }
}

/// The same property through the command path: Play arrives as a command.
#[test]
fn play_command_after_idle_does_not_replay_the_backlog() {
    let mut host = Host::new(random_engine(5), 120.0);
    host.render_seconds(30.0, 512, false);
    host.engine.handle_command(Command::Play);
    let got = host.render(512, true);
    // 512 samples at 120 BPM is 4 ticks, so at most a handful of events can start.
    assert!(got.len() <= 16, "{} events in the first buffer after a 30 s idle", got.len());
}

// ---------------------------------------------------------------------- determinism

/// Seed replay: same seed, same pattern, same host behaviour gives the same stream.
#[test]
fn same_seed_replays_the_same_stream() {
    for seed in 0..16u64 {
        let run = || {
            let mut rng = Rng::new(seed);
            let mut host = Host::new(random_engine(seed), random_bpm(&mut rng));
            for i in 0..200 {
                host.render(random_buffer(&mut rng), i % 50 != 49);
            }
            host.events
        };
        assert_eq!(run(), run(), "seed {seed}");
    }
}

/// A different seed changes the pattern, so the property tests above are not all
/// running the same scenario.
#[test]
fn different_seeds_make_different_scenarios() {
    let stream = |seed| {
        let mut host = Host::new(random_engine(seed), 120.0);
        host.render_seconds(4.0, 512, true);
        host.events
    };
    assert_ne!(stream(1), stream(2));
}

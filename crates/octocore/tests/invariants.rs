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

/// The event with its buffer-relative position zeroed, so streams from hosts with different
/// buffer sizes compare by absolute sample only.
fn without_position(e: Event) -> Event {
    match e {
        Event::NoteOn { port, ch, note, vel, .. } => Event::NoteOn { port, ch, note, vel, at_sample: 0 },
        Event::NoteOff { port, ch, note, .. } => Event::NoteOff { port, ch, note, at_sample: 0 },
        Event::Cc { port, ch, cc, val, .. } => Event::Cc { port, ch, cc, val, at_sample: 0 },
        Event::PitchBend { port, ch, value, .. } => Event::PitchBend { port, ch, value, at_sample: 0 },
        Event::ChannelPressure { port, ch, value, .. } => Event::ChannelPressure { port, ch, value, at_sample: 0 },
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

/// NoteOns minus NoteOffs per (port, channel, note). Empty when everything balances.
fn imbalance(events: &[(u64, Event)]) -> BTreeMap<(u8, u8, u8), i64> {
    let mut net: BTreeMap<(u8, u8, u8), i64> = BTreeMap::new();
    for (_, e) in events {
        match *e {
            Event::NoteOn { port, ch, note, .. } => *net.entry((port, ch, note)).or_default() += 1,
            Event::NoteOff { port, ch, note, .. } => *net.entry((port, ch, note)).or_default() -= 1,
            Event::Cc { .. } | Event::PitchBend { .. } | Event::ChannelPressure { .. } => {}
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

// ---------------------------------------------------------------------- WENGE-0003

/// The events of `seconds` of playback, as (absolute sample, event), rendered in buffers of
/// `buffer` samples, in a canonical order. Only events before the `seconds` mark are kept,
/// so hosts with different buffer sizes (which end at different samples) compare equal.
fn stream_with_buffer(seed: u64, bpm: f32, buffer: u32, seconds: f64) -> Vec<(u64, Event)> {
    let horizon = (seconds * SAMPLE_RATE as f64) as u64;
    let mut host = Host::new(random_engine(seed), bpm);
    while host.clock < horizon + 4096 {
        host.render(buffer, true);
    }
    let mut got: Vec<(u64, Event)> =
        host.events.into_iter().filter(|(t, _)| *t < horizon).map(|(t, e)| (t, without_position(e))).collect();
    got.sort_by_key(|(t, e)| (*t, format!("{e:?}")));
    got
}

/// SPEC-0001 A3. Where an event lands in time is a property of the pattern and the tempo,
/// not of the host's buffer size. Fails on the parent commit: a step whose start offset
/// pulls a note before the tick that fired it is scheduled in the past when the buffer is
/// small, and the engine clamps it to the buffer's first sample.
///
/// The sizes stop at 4,096 samples. Above that, a pattern this dense can put more than the
/// 256 events one call holds into a single buffer, and the extras wait for the next call,
/// late but not lost (see `a_full_output_buffer_defers_events_instead_of_dropping_them`).
#[test]
fn event_times_do_not_depend_on_the_hosts_buffer_size() {
    let mut failing = Vec::new();
    for seed in 0..24u64 {
        let mut rng = Rng::new(seed + 4000);
        let bpm = random_bpm(&mut rng);
        let reference = stream_with_buffer(seed, bpm, 512, 2.0);
        assert!(reference.len() > 20, "seed {seed}: a scenario with no events tests nothing");
        for buffer in [7u32, 32, 64, 100, 128, 480, 1024, 4096] {
            let got = stream_with_buffer(seed, bpm, buffer, 2.0);
            if got != reference {
                // Events in `got` with no equal in `reference` (each one matched at most once).
                let mut unmatched: Vec<&(u64, Event)> = reference.iter().collect();
                let differing = got
                    .iter()
                    .filter(|e| match unmatched.iter().position(|r| *r == *e) {
                        Some(i) => {
                            unmatched.swap_remove(i);
                            false
                        }
                        None => true,
                    })
                    .count();
                failing.push(format!("seed {seed} bpm {bpm}: buffer {buffer} differs from 512 in {differing} of {} events", reference.len()));
            }
        }
    }
    assert!(failing.is_empty(), "the output depends on the buffer size:\n{}", failing.join("\n"));
}

// ---------------------------------------------------------------------- WENGE-0005

/// SPEC-0001 O5. `render` returns for any tempo and sample rate a host can hand it. On the
/// parent commit an infinite tempo makes zero samples per tick, and the tick loop never ends.
#[test]
fn absurd_tempos_and_sample_rates_cannot_hang_render() {
    for bpm in [f32::INFINITY, f32::NEG_INFINITY, f32::NAN, 1.0e30, 1.0e-30, 0.0, -1.0, 5000.0, 0.5] {
        for rate in [48_000.0f32, 0.0, f32::NAN, f32::INFINITY, 1.0, 1.0e9] {
            let mut engine = random_engine(3);
            let mut out = EventBuffer::new();
            for _ in 0..3 {
                out.clear();
                let ctx = RenderContext { sample_rate: rate, buffer_len: 4096, bpm, playing: true };
                engine.render(&ctx, &mut out);
            }
        }
    }
}

/// A buffer or two of unusable tempo, then a sane one: the engine plays exactly what a
/// fresh engine would play from that moment. On the parent commit it never plays again.
#[test]
fn a_bad_tempo_costs_nothing_once_the_tempo_is_sane() {
    let mut fresh = Host::new(random_engine(9), 120.0);
    fresh.render_seconds(2.0, 512, true);

    let mut host = Host::new(random_engine(9), 0.0);
    for _ in 0..3 {
        host.render(512, true);
    }
    assert!(host.events.is_empty());
    let resumed_at = host.clock;
    host.bpm = 120.0;
    host.render_seconds(2.0, 512, true);

    let relative: Vec<(u64, Event)> = host.events.iter().map(|(t, e)| (t - resumed_at, *e)).collect();
    assert!(!relative.is_empty(), "the engine never played again");
    assert_eq!(relative, fresh.events);
}

// ---------------------------------------------------------------------- WENGE-0010

/// CE v5.30 p.40: a note with velocity 0 is not transmitted, and a NoteOn with velocity 0 is
/// a NoteOff to every receiver. So every NoteOn on the wire has velocity 1..=127, and every
/// NoteOff has a NoteOn (the stream balances). Quiet tracks, low velocity factors and
/// negative velocity offsets are what reach 0, so the random patterns are made quiet.
#[test]
fn every_note_on_has_a_velocity_of_at_least_1_however_quiet_the_pattern() {
    let mut zero = Vec::new();
    for seed in 0..SEEDS {
        let mut rng = Rng::new(seed + 5000);
        let mut engine = random_engine(seed);
        engine.grid.active_page_mut().velocity_factor = 1 + rng.next_below(8) as u8;
        for t in 0..TRACK_COUNT {
            let track = &mut engine.grid.active_page_mut().tracks[t];
            track.velocity = rng.next_below(40) as u8;
            for step in track.steps.iter_mut() {
                step.velocity_offset = rng.next_below(21) as i8 - 20;
            }
        }
        let mut host = Host::new(engine, random_bpm(&mut rng));
        host.render_seconds(3.0, 256, true);
        host.render(256, false);
        for (t, e) in &host.events {
            if let Event::NoteOn { vel: 0, .. } = e {
                zero.push(format!("seed {seed} sample {t}"));
                break;
            }
        }
        let net = imbalance(&host.events);
        assert!(net.is_empty(), "seed {seed}: unbalanced {net:?}");
    }
    assert!(zero.is_empty(), "NoteOn with velocity 0 in {} of {SEEDS} seeds, first: {:?}", zero.len(), zero.first());
}

/// The order of events in one sample does not depend on the host's buffer size either.
/// Compared exactly, not sorted: the stream a synth receives is the stream that is tested.
#[test]
fn event_order_does_not_depend_on_the_hosts_buffer_size() {
    let exact = |seed: u64, buffer: u32| {
        let horizon = 2 * SAMPLE_RATE as u64;
        let mut host = Host::new(random_engine(seed), 141.0);
        while host.clock < horizon + 4096 {
            host.render(buffer, true);
        }
        host.events.into_iter().filter(|(t, _)| *t < horizon).map(|(t, e)| (t, without_position(e))).collect::<Vec<_>>()
    };
    for seed in 0..12u64 {
        let reference = exact(seed, 512);
        for buffer in [7u32, 100, 1024, 4096] {
            assert_eq!(exact(seed, buffer), reference, "seed {seed}, buffer {buffer}");
        }
    }
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

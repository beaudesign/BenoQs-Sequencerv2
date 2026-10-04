//! Play after a Stop sounds exactly what the stream would have sounded had it not stopped (SPEC-0002 P5 plan W4, the owner's ruling of
//! 2026-10-04: "metronome must stop and play at the point of which it's stopped on the sequence"; the Metronome request
//! `journal/metronome/requests/2026-10-04-exact-resume-after-stop.md`).
//!
//! Written before the code, and red on the parent commit. The engine steps its tracks up to `MAX_EARLY_TICKS` (12 ticks, about one step)
//! ahead of the audio, so at a Stop the next step's note is already in the queue. Stop used to drop the queue, and the note was lost: at
//! 60, 120 and 240 BPM, Play after a Stop skipped a note on every step boundary the Stop fell before (`handoffs/evidence/scorecard-
//! 2026-10-04/headless.py`, section C).
//!
//! **The property.** Take a run that never stops, the *uninterrupted* stream. Take the same pattern stopped and played again, any number
//! of times, at any sample, with any gap and any buffer sizes. The stopped-and-played stream is the uninterrupted one with the gaps taken
//! out, plus two things that are not in it: at each Stop, a Note Off for every note the receiver is holding and a CC 123 on each channel
//! that had one, at the sample the Stop took effect (ADR-0004's flush, unchanged); and without the Note Off the flush already sent, because
//! the note it ends is over. Nothing else differs. `oracle` writes that down; every test below is that sentence on a different pattern.
//!
//! The pattern gives each track its own channel and one note length, so that on any one channel and pitch the Note Offs leave in the order
//! the Note Ons did and the oracle can tell which Note Off belongs to which note. `two_notes_of_one_pitch_that_overlap_...` is the case
//! where they do not, built by hand.

use std::collections::BTreeMap;

use octocore::domain::{Mcc, STEP_COUNT, TRACK_COUNT};
use octocore::engine::REALTIME_PER_RENDER;
use octocore::rng::Rng;
use octocore::types::{Command, Realtime, RealtimeEvent, StepAttr};
use octocore::{Engine, Event, EventBuffer, RenderContext};

const SR: f32 = 48_000.0;
/// Tempos at which the samples in a tick are an exact binary fraction at 48 kHz (250, 125, 62.5), so a sample position is a fact and not a
/// rounding, and "shifted by the gap" is exact.
const BPMS: [f32; 3] = [60.0, 120.0, 240.0];
const SEEDS: u64 = 60;
const BUFFERS: [u32; 11] = [1, 7, 32, 64, 100, 128, 256, 480, 512, 1024, 4096];

// ------------------------------------------------------------------------------------------------ a host

/// Renders blocks and records everything with absolute sample positions. The position inside the buffer is zeroed so two hosts with
/// different buffers compare by absolute sample alone.
struct Host {
    e: Engine,
    bpm: f32,
    /// Start of the next block, in samples since the host began, gaps included.
    at: u64,
    events: Vec<(u64, Event)>,
    realtime: Vec<(u64, Realtime)>,
}

impl Host {
    fn new(e: Engine, bpm: f32, clock: bool) -> Host {
        let mut h = Host { e, bpm, at: 0, events: vec![], realtime: vec![] };
        h.e.set_clock_master(clock);
        h
    }

    fn render(&mut self, len: u32, playing: bool) {
        let ctx = RenderContext { sample_rate: SR, buffer_len: len, bpm: self.bpm, playing };
        let mut buf = EventBuffer::new();
        self.e.render(&ctx, &mut buf);
        let mut rt = [RealtimeEvent { msg: Realtime::Clock, at_sample: 0 }; REALTIME_PER_RENDER];
        let n = self.e.take_realtime(&mut rt);
        self.realtime.extend(rt[..n].iter().map(|x| (self.at + x.at_sample as u64, x.msg)));
        self.events.extend(buf.as_slice().iter().map(|x| (self.at + at_sample(x) as u64, at_zero(*x))));
        self.at += len as u64;
    }

    /// Renders `samples` in blocks of `block`.
    fn run(&mut self, samples: u64, block: u32, playing: bool) {
        let mut left = samples;
        while left > 0 {
            let n = left.min(block as u64) as u32;
            self.render(n, playing);
            left -= n as u64;
        }
    }

    /// Nothing the engine reports about itself may say an event was late or waited: a late event is emitted at sample 0 of its buffer,
    /// which depends on the buffer, and this file compares absolute samples.
    fn assert_on_time(&self, what: &str) {
        let d = self.e.diagnostics();
        assert_eq!((d.late_events, d.deferred_events, d.queue_overflows), (0, 0, 0), "{what}: events were late, deferred or refused, so the streams are not comparable");
        assert_eq!(self.e.realtime_dropped(), 0, "{what}: real-time messages were dropped");
    }
}

fn at_sample(e: &Event) -> u32 {
    match *e {
        Event::NoteOn { at_sample, .. } | Event::NoteOff { at_sample, .. } | Event::Cc { at_sample, .. } | Event::PitchBend { at_sample, .. } | Event::ChannelPressure { at_sample, .. } => at_sample,
    }
}

fn at_zero(e: Event) -> Event {
    match e {
        Event::NoteOn { port, ch, note, vel, .. } => Event::NoteOn { port, ch, note, vel, at_sample: 0 },
        Event::NoteOff { port, ch, note, .. } => Event::NoteOff { port, ch, note, at_sample: 0 },
        Event::Cc { port, ch, cc, val, .. } => Event::Cc { port, ch, cc, val, at_sample: 0 },
        Event::PitchBend { port, ch, value, .. } => Event::PitchBend { port, ch, value, at_sample: 0 },
        Event::ChannelPressure { port, ch, value, .. } => Event::ChannelPressure { port, ch, value, at_sample: 0 },
    }
}

// ------------------------------------------------------------------------------------------------ the patterns

/// A random pattern. Each track has its own channel (the odd ones on port 2) and one note length, from one tick to the longest, so a Stop
/// finds a note sounding on most tracks. About one step in eight is strummed and about one in five sends a controller.
fn pattern(seed: u64) -> Engine {
    let mut rng = Rng::new(seed ^ 0xC0FFEE);
    let mut e = Engine::new(seed);
    let page = e.grid.active_page_mut();
    for t in 0..TRACK_COUNT {
        if rng.next_below(4) == 0 {
            continue;
        }
        let track = &mut page.tracks[t];
        track.midi_channel = if t % 2 == 0 { t as u8 + 1 } else { 16 + t as u8 + 1 };
        if t % 3 == 0 {
            track.mcc = Mcc::Cc(74);
        }
        let length = 1 + rng.next_below(192) as u8;
        let multiplier = 1 + rng.next_below(3) as u8;
        for s in 0..STEP_COUNT {
            let step = &mut track.steps[s];
            step.active = rng.next_below(3) != 0;
            step.length_ticks = length;
            step.length_multiplier = multiplier;
            step.pitch_offset = rng.next_below(25) as i8 - 12;
            step.velocity_offset = rng.next_below(21) as i8 - 10;
            step.start_offset = rng.next_below(5) as i8 - 2;
            step.strum = if rng.next_below(8) == 0 { rng.next_below(19) as i8 - 9 } else { 0 };
            if t % 3 == 0 && rng.next_below(5) == 0 {
                step.mcc_value = Some(rng.next_below(128) as u8);
            }
        }
    }
    e
}

// ------------------------------------------------------------------------------------------------ the oracle

/// A Stop and the Play after it, in the uninterrupted run's own time: `at` is how many samples the run had played when it stopped, `gap`
/// how long it stayed stopped. `double` is a second Stop while stopped, which asks for CC 123 on every channel.
#[derive(Clone, Copy, Debug)]
struct Stop {
    at: u64,
    gap: u64,
    double: bool,
}

type Slot = (u8, u8, u8);

/// What the stopped-and-played run must emit, written from the uninterrupted run alone (the property in the module comment). Returns
/// the events and the real-time messages, in the order they leave. `end` is how many samples the run has played in all.
fn oracle(uninterrupted_events: &[(u64, Event)], uninterrupted_realtime: &[(u64, Realtime)], stops: &[Stop], end: u64, clock: bool) -> (Vec<(u64, Event)>, Vec<(u64, Realtime)>) {
    let mut events: Vec<(u64, Event)> = vec![];
    let mut realtime: Vec<(u64, Realtime)> = vec![];
    // What the receiver holds, as the stopped run's own output has left it. And the Note Offs of notes already ended by a flush, still to
    // come in the uninterrupted stream and not to be sent: first made, first to end, because one channel has one note length.
    let mut holding: BTreeMap<Slot, i64> = BTreeMap::new();
    let mut skip: BTreeMap<Slot, i64> = BTreeMap::new();
    let mut shift = 0u64;
    let mut next_pulse = 0usize;
    let rt = uninterrupted_realtime;

    let emit_until = |limit: u64, shift: u64, events: &mut Vec<(u64, Event)>, holding: &mut BTreeMap<Slot, i64>, skip: &mut BTreeMap<Slot, i64>, from: &mut usize| {
        while *from < uninterrupted_events.len() {
            let (s, ev) = uninterrupted_events[*from];
            if s >= limit {
                break;
            }
            *from += 1;
            match ev {
                Event::NoteOff { port, ch, note, .. } => {
                    let k = skip.entry((port, ch, note)).or_default();
                    if *k > 0 {
                        *k -= 1;
                        continue;
                    }
                    *holding.entry((port, ch, note)).or_default() -= 1;
                }
                Event::NoteOn { port, ch, note, .. } => *holding.entry((port, ch, note)).or_default() += 1,
                _ => {}
            }
            events.push((s + shift, ev));
        }
    };

    let mut from = 0usize;
    for stop in stops {
        emit_until(stop.at, shift, &mut events, &mut holding, &mut skip, &mut from);
        while next_pulse < rt.len() && rt[next_pulse].0 < stop.at {
            realtime.push((rt[next_pulse].0 + shift, rt[next_pulse].1));
            next_pulse += 1;
        }
        // The flush, at the sample the Stop took effect: a Note Off for each note held, then CC 123 where one went out, or everywhere after
        // a Stop while stopped.
        let when = stop.at + shift;
        let mut channels: Vec<(u8, u8)> = vec![];
        for (&(port, ch, note), n) in holding.iter() {
            for _ in 0..*n {
                events.push((when, Event::NoteOff { port, ch, note, at_sample: 0 }));
            }
            if *n > 0 && !channels.contains(&(port, ch)) {
                channels.push((port, ch));
            }
            *skip.entry((port, ch, note)).or_default() += *n;
        }
        if stop.double {
            channels = (1..=2u8).flat_map(|p| (1..=16u8).map(move |c| (p, c))).collect();
        }
        channels.sort_unstable();
        for (port, ch) in channels {
            events.push((when, Event::Cc { port, ch, cc: 123, val: 0, at_sample: 0 }));
        }
        holding.clear();
        if clock {
            realtime.push((when, Realtime::Stop));
            realtime.push((when + stop.gap, Realtime::Continue));
        }
        shift += stop.gap;
    }
    emit_until(end, shift, &mut events, &mut holding, &mut skip, &mut from);
    while next_pulse < rt.len() && rt[next_pulse].0 < end {
        realtime.push((rt[next_pulse].0 + shift, rt[next_pulse].1));
        next_pulse += 1;
    }
    (events, realtime)
}

// ------------------------------------------------------------------------------------------------ the scenarios

struct Scenario {
    seed: u64,
    bpm: f32,
    clock: bool,
    stops: Vec<Stop>,
    /// The Stop and the Play go in as commands between renders (which allows a gap of nothing), not as the host's transport flag.
    via_commands: bool,
    /// How many samples each stretch of play lasts, one more than there are stops.
    plays: Vec<u64>,
}

fn scenario(seed: u64) -> Scenario {
    let mut rng = Rng::new(seed + 7000);
    let via_commands = seed % 2 == 0;
    let clock = seed % 3 != 0;
    let n = 1 + rng.next_below(4) as usize;
    let mut stops = vec![];
    let mut plays = vec![];
    let mut played = 0u64;
    for _ in 0..n {
        let play = 1 + rng.next_below(60_000) as u64; // up to a second and a quarter: from before the first step to a few
        plays.push(play);
        played += play;
        let gap = if via_commands && rng.next_below(4) == 0 { 0 } else { 1 + rng.next_below(40_000) as u64 };
        stops.push(Stop { at: played, gap, double: via_commands && rng.next_below(5) == 0 });
    }
    plays.push(1 + rng.next_below(60_000) as u64);
    Scenario { seed, bpm: BPMS[(seed / 2 % 3) as usize], clock, stops, via_commands, plays }
}

/// Drives the stopped-and-played run: each stretch of play and each gap in blocks picked from `sizes`, the last block of each cut short so
/// the stretch is exactly as long as the oracle was told.
fn drive(sc: &Scenario, buffers: &mut Rng, sizes: &[u32]) -> Host {
    let mut b = Host::new(pattern(sc.seed), sc.bpm, sc.clock);
    for k in 0..=sc.stops.len() {
        // Play, in random blocks, for exactly `plays[k]` samples.
        let mut left = sc.plays[k];
        while left > 0 {
            let len = sizes[buffers.next_below(sizes.len() as u32) as usize].min(left as u32);
            b.render(len, true);
            left -= len as u64;
        }
        if k == sc.stops.len() {
            break;
        }
        let gap = sc.stops[k].gap;
        if sc.via_commands {
            b.e.handle_command(Command::Stop);
            if sc.stops[k].double {
                b.e.handle_command(Command::Stop);
            }
        }
        let mut left = gap;
        while left > 0 {
            let len = sizes[buffers.next_below(sizes.len() as u32) as usize].min(left as u32);
            b.render(len, false);
            left -= len as u64;
        }
        if sc.via_commands {
            b.e.handle_command(Command::Play);
        }
    }
    b
}

fn first_difference<T: std::fmt::Debug + PartialEq>(got: &[T], want: &[T]) -> String {
    for (i, (g, w)) in got.iter().zip(want.iter()).enumerate() {
        if g != w {
            return format!("first difference at {i}: got {g:?}, want {w:?} (got {} entries, want {})", got.len(), want.len());
        }
    }
    format!("one is a prefix of the other: got {} entries, want {}", got.len(), want.len())
}

#[test]
fn play_after_a_stop_is_the_stream_that_stopped_with_the_gap_taken_out() {
    let mut worst = String::new();
    let mut failed = 0;
    let mut stops_checked = 0;
    for seed in 0..SEEDS {
        let sc = scenario(seed);
        let mut buffers = Rng::new(seed + 9000);
        let b = drive(&sc, &mut buffers, &BUFFERS);
        let end: u64 = sc.plays.iter().sum();

        // The uninterrupted run, in blocks of its own, to the same length of play.
        let mut a = Host::new(pattern(seed), sc.bpm, sc.clock);
        a.run(end, 128, true);
        a.assert_on_time(&format!("seed {seed}: the uninterrupted run"));

        let (want_events, want_realtime) = oracle(&a.events, &a.realtime, &sc.stops, end, sc.clock);
        stops_checked += sc.stops.len();
        if b.events != want_events || b.realtime != want_realtime {
            failed += 1;
            if worst.is_empty() {
                worst = format!(
                    "seed {seed} ({} BPM, clock {}, {} stops via {}): events: {}; real-time: {}",
                    sc.bpm,
                    sc.clock,
                    sc.stops.len(),
                    if sc.via_commands { "commands" } else { "the host flag" },
                    first_difference(&b.events, &want_events),
                    first_difference(&b.realtime, &want_realtime)
                );
            }
        }
        // Said last, so a failure above is reported as the stream it is: the old engine started the first tick after a Play at the Play, and
        // a note pulled early by a negative start offset was then due before the block, so it came out late.
        if failed == 0 {
            b.assert_on_time(&format!("seed {seed}: the stopped run"));
        }
    }
    assert_eq!(failed, 0, "{failed} of {SEEDS} scenarios ({stops_checked} stops) differ from the uninterrupted stream with the gaps taken out. First: {worst}");
}

// ------------------------------------------------------------------------------------------------ the cases the sweep cannot build

/// Track 0 on channel 1 with the given active steps, each (step, length in ticks), all the same pitch; everything else off.
fn one_track(steps: &[(usize, u8)]) -> Engine {
    let mut e = Engine::new(1);
    let page = e.grid.active_page_mut();
    page.tracks[0].midi_channel = 1;
    for s in 0..STEP_COUNT {
        page.tracks[0].steps[s].active = false;
    }
    for &(s, len) in steps {
        page.tracks[0].steps[s].active = true;
        page.tracks[0].steps[s].length_ticks = len;
        page.tracks[0].steps[s].length_multiplier = 1;
    }
    e
}

#[test]
fn two_notes_of_one_pitch_that_overlap_are_told_apart_when_a_stop_falls_between_them() {
    // 120 BPM at 48 kHz is 125 samples a tick, and a step's note comes on the twelfth tick of its step: step 0 at tick 11 and step 1 at
    // tick 23. Step 0 is a long note and step 1 a short one of the same pitch: the first ends at tick 111 and the second at tick 43. A Stop
    // at tick 15 falls after the first note has started and before the second has, with both Note Offs already in the queue. Only the
    // first Note Off is the stop's to send, as the flush; the second goes on with its note. Dropping the wrong one leaves the second note
    // sounding to tick 111.
    let spt = 125u64;
    let mut h = Host::new(one_track(&[(0, 100), (1, 20)]), 120.0, false);
    h.run(15 * spt, 128, true);
    assert_eq!(h.events.len(), 1, "the first note has started: {:?}", h.events);
    let (note, vel) = match h.events[0].1 {
        Event::NoteOn { note, vel, .. } => (note, vel),
        other => panic!("{other:?}"),
    };
    h.e.handle_command(Command::Stop);
    h.run(1000, 128, false);
    let stopped_at = 15 * spt;
    let played_at = h.at;
    h.e.handle_command(Command::Play);
    h.run(14_000, 128, true);

    let got: Vec<(u64, Event)> = h.events[1..].to_vec();
    let gap = played_at - stopped_at;
    let want = vec![
        (stopped_at, Event::NoteOff { port: 1, ch: 1, note, at_sample: 0 }),
        (stopped_at, Event::Cc { port: 1, ch: 1, cc: 123, val: 0, at_sample: 0 }),
        (23 * spt + gap, Event::NoteOn { port: 1, ch: 1, note, vel, at_sample: 0 }),
        (43 * spt + gap, Event::NoteOff { port: 1, ch: 1, note, at_sample: 0 }),
    ];
    assert_eq!(got, want, "{}", first_difference(&got, &want));
}

#[test]
fn a_reset_after_a_stop_forgets_the_notes_held_for_the_play() {
    // Reset is "back to the start". What Stop held for the Play is part of the position, so Reset drops it, and the next Play is a new run.
    let spt = 125u64;
    let steps: Vec<(usize, u8)> = (0..16).map(|s| (s, 6)).collect();
    let mut h = Host::new(one_track(&steps), 120.0, false);
    h.run(10 * spt, 128, true); // step 0's note, at tick 11, is stepped and not yet sounded: Stop holds it
    h.e.handle_command(Command::Stop);
    h.run(500, 128, false);
    h.e.handle_command(Command::Reset);
    h.run(500, 128, false);
    let played_at = h.at;
    h.e.handle_command(Command::Play);
    h.run(30 * spt, 128, true);

    let mut fresh = Host::new(one_track(&steps), 120.0, false);
    fresh.run(30 * spt, 128, true);
    // Nothing was sounding at tick 10, so there is no flush to account for: what leaves after the Play is a fresh engine's stream.
    let got: Vec<(u64, Event)> = h.events.iter().filter(|(s, _)| *s >= played_at).copied().collect();
    let want: Vec<(u64, Event)> = fresh.events.iter().map(|(s, e)| (s + played_at, *e)).collect();
    assert_eq!(got, want, "{}", first_difference(&got, &want));
}

#[test]
fn an_edit_made_while_stopped_is_heard_from_the_first_tick_the_engine_had_not_yet_stepped() {
    // The price of resuming exactly: a tick that was stepped before the Stop has already decided what it plays. Stopped at tick 18, five
    // ticks before step 1's note (tick 23, which is stepped, up to 12 ticks ahead, when the Stop comes), that note still sounds on Play
    // though the player has made a rest of it while stopped. The steps after it are the new ones. A design that rewinds the tracks on Stop
    // would hear the edit at once; this test changes when that design is chosen, and says so.
    let spt = 125u64;
    let steps: Vec<(usize, u8)> = (0..16).map(|s| (s, 6)).collect();
    let mut h = Host::new(one_track(&steps), 120.0, false);
    h.run(18 * spt, 128, true); // step 0 has sounded and ended (tick 11 to tick 17)
    h.e.handle_command(Command::Stop);
    h.run(2000, 128, false);
    let (note, vel) = match h.events[0].1 {
        Event::NoteOn { note, vel, .. } => (note, vel),
        other => panic!("{other:?}"),
    };
    for s in 0..16 {
        h.e.grid.set_step_attr(0, s, StepAttr::Active, 0);
    }
    let played_at = h.at;
    h.e.handle_command(Command::Play);
    h.run(40 * spt, 128, true);

    let after: Vec<(u64, Event)> = h.events.iter().filter(|(s, _)| *s >= played_at).copied().collect();
    let on = |e: &(u64, Event)| matches!(e.1, Event::NoteOn { .. });
    assert_eq!(after.iter().filter(|e| on(e)).count(), 1, "only the step that was already stepped sounds: {after:?}");
    assert_eq!(after[0], (played_at + 5 * spt, Event::NoteOn { port: 1, ch: 1, note, vel, at_sample: 0 }), "five ticks after Play, as it was five ticks after the Stop");
}

#[test]
fn a_hundred_stops_and_plays_do_not_pile_events_up() {
    // Held events are the ones the run would have had in its queue anyway, so a run of many Stops queues no more than a run of none.
    let mut sc = scenario(4); // via commands, clock on, 240 BPM
    sc.stops.clear();
    sc.plays.clear();
    let mut played = 0u64;
    for _ in 0..100 {
        sc.plays.push(3_000);
        played += 3_000;
        sc.stops.push(Stop { at: played, gap: 700, double: false });
    }
    sc.plays.push(3_000);
    let end: u64 = sc.plays.iter().sum();
    let mut buffers = Rng::new(1);
    let mut b = drive(&sc, &mut buffers, &[128]);
    let mut a = Host::new(pattern(sc.seed), sc.bpm, sc.clock);
    a.run(end, 128, true);
    assert!(b.e.diagnostics().queue_high_water <= a.e.diagnostics().queue_high_water, "the queue grew across Stops: {} against {}", b.e.diagnostics().queue_high_water, a.e.diagnostics().queue_high_water);
    assert_eq!(b.e.diagnostics().queue_overflows, 0);
    b.assert_on_time("a hundred stops");
    b.e.handle_command(Command::Stop);
    b.render(1, false);
    assert_eq!(b.e.sounding_count(), 0, "a note is still sounding after the last Stop");
}

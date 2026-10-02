//! The MIDI clock through the module's exported surface (ADR-0009 decisions 1 to 3, `specs/SPEC-0002/p4-plan.md` P4c): event kind 5,
//! `octoweb_set_clock` and `octoweb_tick_position`. The pulse positions themselves are pinned in `crates/octocore/tests/clock.rs`;
//! what is pinned here is that the page receives them, where in the stream, and in the bytes `ABI.md` says. Each test runs on its own
//! thread and the module's state is per thread, so they do not meet.

mod common;

use common::*;
use octocore::{Command, Engine, EventBuffer, RealtimeEvent, RenderContext};
use octoweb::exports::*;

const REALTIME: u8 = 5;
const CLOCK: u8 = 0xF8;
const START: u8 = 0xFA;
const CONTINUE: u8 = 0xFB;
const STOP: u8 = 0xFC;

const SEED: u64 = 7;
/// A line on track 0 and an off-beat on track 3: the pattern the byte-for-byte test in `abi.rs` plays.
const STEPS: [(usize, usize); 6] = [(0, 0), (0, 4), (0, 8), (0, 12), (3, 2), (3, 10)];

type Record = [u8; EVENT_BYTES];

fn at(e: &Record) -> u32 {
    u32::from_le_bytes([e[8], e[9], e[10], e[11]])
}

fn is_realtime(e: &Record) -> bool {
    e[0] == REALTIME
}

/// A started module and how many samples it has rendered, so a record's place is the sample it falls on since the start.
struct Session {
    abi: Abi,
    rendered: u64,
}

impl Session {
    fn start(sample_rate: f32) -> Session {
        Session { abi: Abi::start_with(sample_rate, SEED), rendered: 0 }
    }

    fn program(&self) {
        for (i, (row, step)) in STEPS.iter().enumerate() {
            self.abi.click(&self.abi.matrix(*row, *step), i as f64 * 100.0);
        }
    }

    /// `blocks` renders of `frames` frames: each record with its sample counted from the first render's start.
    fn run(&mut self, blocks: usize, frames: u32) -> Vec<(u64, Record)> {
        let mut out = Vec::new();
        for _ in 0..blocks {
            for e in self.abi.render(frames) {
                out.push((self.rendered + u64::from(at(&e)), e));
            }
            self.rendered += u64::from(frames);
        }
        out
    }

    /// Whole seconds of 128-frame blocks, as the worklet renders them.
    fn seconds(&mut self, secs: u32) -> Vec<(u64, Record)> {
        let blocks = (secs as f32 * self.abi.sample_rate / 128.0) as usize;
        self.run(blocks, 128)
    }
}

fn realtime_only(records: &[(u64, Record)]) -> Vec<(u64, Record)> {
    records.iter().filter(|(_, e)| is_realtime(e)).copied().collect()
}

fn status_bytes(records: &[(u64, Record)]) -> Vec<u8> {
    records.iter().filter(|(_, e)| is_realtime(e)).map(|(_, e)| e[3]).collect()
}

// ---------------------------------------------------------------------------------------------------------------- off by default

#[test]
fn the_clock_is_off_until_the_page_asks_for_it() {
    let mut s = Session::start(48_000.0);
    s.program();
    assert_eq!(octoweb_set_tempo(240.0), 0);
    assert_eq!(octoweb_transport(1), 0);
    let all = s.seconds(2);
    assert!(all.iter().any(|(_, e)| e[0] == 0), "the test should hear notes");
    assert!(realtime_only(&all).is_empty(), "no kind 5 record unless the clock was turned on");
}

#[test]
fn set_clock_takes_any_nonzero_as_on_and_zero_as_off() {
    for on in [1u32, 2, u32::MAX] {
        let mut s = Session::start(48_000.0);
        assert_eq!(octoweb_set_clock(on), 0);
        assert_eq!(octoweb_transport(1), 0);
        assert!(!realtime_only(&s.seconds(1)).is_empty(), "{on} is on");
    }
    let mut s = Session::start(48_000.0);
    assert_eq!(octoweb_set_clock(1), 0);
    assert_eq!(octoweb_set_clock(0), 0);
    assert_eq!(octoweb_transport(1), 0);
    assert!(realtime_only(&s.seconds(1)).is_empty(), "turned on and off again before Play: nothing");
}

#[test]
fn before_init_set_clock_says_not_initialised_and_the_position_is_zero() {
    octoweb_reset();
    assert_eq!(octoweb_set_clock(1), 1);
    assert_eq!(octoweb_tick_position(), 0.0);
}

// ---------------------------------------------------------------------------------------------------------------- the record

#[test]
fn play_with_the_clock_on_sends_start_and_then_a_clock_on_the_same_sample() {
    let mut s = Session::start(48_000.0);
    assert_eq!(octoweb_set_clock(1), 0);
    assert_eq!(octoweb_transport(1), 0);
    let rt = realtime_only(&s.run(8, 128));
    assert_eq!(rt[0], (0, [REALTIME, 0, 0, START, 0, 0, 0, 0, 0, 0, 0, 0]), "Start: kind 5, port 0, channel 0, the status byte in d1");
    assert_eq!(rt[1], (0, [REALTIME, 0, 0, CLOCK, 0, 0, 0, 0, 0, 0, 0, 0]), "the first Clock follows on the same sample");
    for (_, e) in &rt {
        assert_eq!((e[1], e[2], e[4], e[5], e[6], e[7]), (0, 0, 0, 0, 0, 0), "port, channel, d2 and the reserved bytes of a real-time record are zero");
        assert!([CLOCK, START, CONTINUE, STOP].contains(&e[3]), "d1 is a MIDI real-time status byte: {:#x}", e[3]);
    }
}

#[test]
fn a_pulse_is_a_24th_of_a_quarter_note_apart_at_the_pages_tempo_and_rate() {
    // 120 BPM at 48 kHz is 125 samples a tick, and a pulse is every 8 ticks: exactly 1,000 samples, so 24 of them are 500 ms.
    let mut s = Session::start(48_000.0);
    assert_eq!(octoweb_set_tempo(120.0), 0);
    assert_eq!(octoweb_set_clock(1), 0);
    assert_eq!(octoweb_transport(1), 0);
    let clocks: Vec<u64> = s.seconds(2).iter().filter(|(_, e)| e[3] == CLOCK && is_realtime(e)).map(|(p, _)| *p).collect();
    assert!(clocks.len() >= 90, "{} pulses in two seconds", clocks.len());
    for pair in clocks.windows(2) {
        assert_eq!(pair[1] - pair[0], 1_000, "{pair:?}");
    }
    assert_eq!(clocks[24] - clocks[0], 24_000, "24 pulses are a quarter note, 500 ms at 120 BPM");
}

#[test]
fn the_notes_are_the_same_with_the_clock_on_and_off() {
    let notes = |clock: u32| {
        let mut s = Session::start(48_000.0);
        s.program();
        assert_eq!(octoweb_set_tempo(133.0), 0);
        assert_eq!(octoweb_set_clock(clock), 0);
        assert_eq!(octoweb_transport(1), 0);
        let all = s.seconds(4);
        let notes: Vec<(u64, Record)> = all.iter().filter(|(_, e)| !is_realtime(e)).copied().collect();
        (notes, realtime_only(&all).len())
    };
    let (off, off_rt) = notes(0);
    let (on, on_rt) = notes(1);
    assert!(off.len() >= 8, "the test should hear something ({} events)", off.len());
    assert_eq!(off_rt, 0);
    assert!(on_rt > 100, "and the clock should be running ({on_rt} records)");
    assert_eq!(on, off, "the clock adds records and changes none of the notes, their order or their samples");
}

#[test]
fn the_real_time_records_are_the_engines_own_byte_for_byte() {
    let (bpm, sr) = (133.0f32, 48_000.0f32);
    let s = Session::start(sr);
    s.program();
    assert_eq!(octoweb_set_tempo(bpm), 0);
    assert_eq!(octoweb_set_clock(1), 0);
    assert_eq!(octoweb_transport(1), 0);

    let mut engine = Engine::new(SEED);
    for (row, step) in STEPS {
        engine.grid.active_page_mut().tracks[row].steps[step].active = true;
    }
    engine.set_clock_master(true);
    engine.handle_command(Command::Play);
    let (mut buf, mut out) = (EventBuffer::new(), [RealtimeEvent { msg: octocore::Realtime::Clock, at_sample: 0 }; octocore::engine::REALTIME_PER_RENDER]);
    let mut seen = 0;
    for block in 0..(4 * 48_000 / 128) {
        buf.clear();
        engine.render(&RenderContext { sample_rate: sr, buffer_len: 128, bpm, playing: engine.is_running() }, &mut buf);
        let n = engine.take_realtime(&mut out);
        let want: Vec<Record> = out[..n].iter().map(wire_realtime).collect();
        let got: Vec<Record> = s.abi.render(128).into_iter().filter(is_realtime).collect();
        assert_eq!(got, want, "block {block}");
        seen += n;
    }
    assert!(seen > 100, "{seen} real-time messages in four seconds");
}

#[test]
fn a_block_is_in_sample_order_and_a_pulse_comes_before_a_note_on_its_sample() {
    // Every row, every step, at the page's fastest sensible tempo: as many notes and pulses as the module is likely to make at once.
    let s = Session::start(48_000.0);
    for row in 0..10 {
        for step in 0..16 {
            s.abi.click(&s.abi.matrix(row, step), 0.0);
        }
    }
    assert_eq!(octoweb_set_tempo(240.0), 0);
    assert_eq!(octoweb_set_clock(1), 0);
    assert_eq!(octoweb_transport(1), 0);
    for block in 0..(3 * 48_000 / 128) {
        let records = s.abi.render(128);
        for pair in records.windows(2) {
            assert!(at(&pair[0]) <= at(&pair[1]), "block {block}: records out of sample order");
            if at(&pair[0]) == at(&pair[1]) {
                assert!(is_realtime(&pair[0]) || !is_realtime(&pair[1]), "block {block}: a note before a pulse on one sample");
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------- the transport

#[test]
fn stop_and_continue_come_through_in_order() {
    let mut s = Session::start(48_000.0);
    assert_eq!(octoweb_set_clock(1), 0);
    assert_eq!(octoweb_transport(1), 0);
    s.run(20, 128);
    assert_eq!(octoweb_transport(0), 0);
    let stopped_at = s.rendered;
    let after_stop = realtime_only(&s.run(40, 128));
    assert_eq!(after_stop.len(), 1, "Stop, and no pulse after it: {:?}", status_bytes(&after_stop));
    assert_eq!((after_stop[0].0, after_stop[0].1[3]), (stopped_at, STOP), "Stop is on the first sample after the call");

    assert_eq!(octoweb_transport(1), 0);
    let played_at = s.rendered;
    let resumed = realtime_only(&s.run(20, 128));
    assert_eq!((resumed[0].0, resumed[0].1[3]), (played_at, CONTINUE), "the sequencer was not rewound, so Play carries on and says Continue");
    // Start at tick 0 has a pulse on its own sample. A resume is wherever the sequencer stopped, which is between pulses, so the next
    // Clock is less than one pulse (1,000 samples at 120 BPM and 48 kHz) after Continue, on the grid of pulses the sequencer is on.
    assert_eq!(resumed[1].1[3], CLOCK);
    assert!(resumed[1].0 > played_at && resumed[1].0 - played_at < 1_000, "the first Clock after Continue is at {}, Continue at {played_at}", resumed[1].0);
}

#[test]
fn turning_the_clock_off_while_it_runs_stops_the_pulses_and_sends_nothing() {
    let mut s = Session::start(48_000.0);
    assert_eq!(octoweb_set_clock(1), 0);
    assert_eq!(octoweb_transport(1), 0);
    assert!(!realtime_only(&s.run(30, 128)).is_empty());
    assert_eq!(octoweb_set_clock(0), 0);
    let later = s.run(100, 128);
    assert!(realtime_only(&later).is_empty(), "no Stop and no pulse: {:?}", status_bytes(&later));
}

// ---------------------------------------------------------------------------------------------------------------- position

#[test]
fn the_tick_position_is_the_audio_position_and_equals_the_engines_own() {
    let (bpm, sr) = (97.0f32, 44_100.0f32);
    let s = Session::start(sr);
    assert_eq!(octoweb_tick_position(), 0.0, "nothing has played");
    assert_eq!(octoweb_set_tempo(bpm), 0);
    assert_eq!(octoweb_transport(1), 0);
    let mut engine = Engine::new(SEED);
    engine.handle_command(Command::Play);
    let mut buf = EventBuffer::new();
    let mut last = 0.0f64;
    let mut fractional = false;
    for block in 0..(3 * 44_100 / 128) {
        s.abi.render(128);
        buf.clear();
        engine.render(&RenderContext { sample_rate: sr, buffer_len: 128, bpm, playing: engine.is_running() }, &mut buf);
        let got = octoweb_tick_position();
        assert_eq!(got, engine.tick_position(), "block {block}");
        assert_eq!(octoweb_tick_position(), got, "reading the position changes nothing");
        assert!(got >= last, "block {block}: the position went back, {last} then {got}");
        fractional |= got.fract() != 0.0;
        last = got;
    }
    assert!(last > 100.0, "three seconds at 97 BPM is about 930 ticks: {last}");
    assert!(fractional, "the position is between ticks most of the time");
}

#[test]
fn the_position_stands_still_while_stopped() {
    let mut s = Session::start(48_000.0);
    assert_eq!(octoweb_transport(1), 0);
    s.run(40, 128);
    assert_eq!(octoweb_transport(0), 0);
    s.run(2, 128);
    let held = octoweb_tick_position();
    assert!(held > 0.0);
    s.run(40, 128);
    assert_eq!(octoweb_tick_position(), held);
}

// ---------------------------------------------------------------------------------------------------------------- the largest block

#[test]
fn the_longest_block_at_the_lowest_rate_and_fastest_tempo_loses_no_pulse_and_fits_the_buffer() {
    // 4,096 frames at 8 kHz and 999 BPM is about 1,640 ticks, so about 205 pulses: more than any real block, fewer than the 256 the
    // buffer reserves, and with a busy pattern the notes share the buffer.
    let (bpm, sr) = (999.0f32, 8_000.0f32);
    let s = Session::start(sr);
    for row in 0..10 {
        for step in 0..16 {
            s.abi.click(&s.abi.matrix(row, step), 0.0);
        }
    }
    assert_eq!(octoweb_set_tempo(bpm), 0);
    assert_eq!(octoweb_set_clock(1), 0);
    assert_eq!(octoweb_transport(1), 0);

    let mut engine = Engine::new(SEED);
    for row in 0..10 {
        for step in 0..16 {
            engine.grid.active_page_mut().tracks[row].steps[step].active = true;
        }
    }
    engine.set_clock_master(true);
    engine.handle_command(Command::Play);
    let (mut buf, mut out) = (EventBuffer::new(), [RealtimeEvent { msg: octocore::Realtime::Clock, at_sample: 0 }; octocore::engine::REALTIME_PER_RENDER]);
    for block in 0..6 {
        buf.clear();
        engine.render(&RenderContext { sample_rate: sr, buffer_len: 4096, bpm, playing: engine.is_running() }, &mut buf);
        let n = engine.take_realtime(&mut out);
        let records = s.abi.render(4096);
        let got: Vec<Record> = records.iter().filter(|e| is_realtime(e)).copied().collect();
        let want: Vec<Record> = out[..n].iter().map(wire_realtime).collect();
        assert_eq!(got, want, "block {block}");
        assert!(got.iter().filter(|e| e[3] == CLOCK).count() >= 200, "block {block}: {} pulses", got.len());
        assert!(records.len() <= 512, "block {block}: {} records", records.len());
        for pair in records.windows(2) {
            assert!(at(&pair[0]) <= at(&pair[1]), "block {block}: out of sample order");
        }
    }
    assert_eq!(engine.realtime_dropped(), 0, "no pulse was dropped for want of room");
}

#[test]
fn a_request_for_more_frames_than_the_limit_is_cut_down_with_the_clock_on() {
    let _s = Session::start(48_000.0);
    assert_eq!(octoweb_set_clock(1), 0);
    assert_eq!(octoweb_transport(1), 0);
    let n = octoweb_render(u32::MAX);
    assert!((2..=512).contains(&n), "Start and the pulses of 4,096 frames, never more than the buffer holds: {n}");
}

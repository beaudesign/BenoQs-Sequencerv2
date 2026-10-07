//! How long a step lasts, in time (`WENGE-0012`, D0, `specs/SPEC-0002/p6a-tick-length.md`).
//!
//! The manual's numbers, written here as numbers and not read back from the engine, so that a change to the engine's constants
//! cannot move them: a default step is a 1/16 note (p.15: "12/192 = 1/16 of a note"), and a measure is 16 steps at x1 speed (p.67), so
//! at 120 BPM, counting quarter notes, a step lasts 125 ms and a bar of 16 steps lasts 2 s. At 48 kHz that is 6000 samples and 96000.
//!
//! Before the change these failed: the engine counted 192 ticks to the quarter note and played a 12-tick step in 31.25 ms, four times
//! too fast at every tempo (`handoffs/evidence/` for the measurement of 2026-10-07: 125 ms between two steps four apart, where
//! the manual says 500 ms).

use octocore::types::{Realtime, RealtimeEvent, StepAttr};
use octocore::engine::REALTIME_PER_RENDER;
use octocore::{Command, Engine, Event, EventBuffer, RenderContext};

const RATE: f32 = 48_000.0;

fn at_sample(e: &Event) -> u32 {
    match *e {
        Event::NoteOn { at_sample, .. } | Event::NoteOff { at_sample, .. } | Event::Cc { at_sample, .. } | Event::PitchBend { at_sample, .. } | Event::ChannelPressure { at_sample, .. } => at_sample,
    }
}

/// What a run made: the absolute sample of every Note On and Note Off on track 0, and of every clock pulse.
struct Run {
    on: Vec<u64>,
    off: Vec<u64>,
    clocks: Vec<u64>,
}

/// Track 0 with every step on, played for `seconds` at `bpm` from the first sample, in blocks of 128.
fn run(bpm: f32, seconds: f32, clock: bool) -> Run {
    let mut e = Engine::new(1);
    for s in 0..16u8 {
        e.grid.set_step_attr(0, s, StepAttr::Active, 1);
    }
    e.set_clock_master(clock);
    e.handle_command(Command::Play);
    let mut out = Run { on: vec![], off: vec![], clocks: vec![] };
    let total = (seconds * RATE) as u64;
    let mut at = 0u64;
    while at < total {
        let ctx = RenderContext { sample_rate: RATE, buffer_len: 128, bpm, playing: true };
        let mut buf = EventBuffer::new();
        e.render(&ctx, &mut buf);
        for ev in buf.as_slice() {
            match ev {
                Event::NoteOn { .. } => out.on.push(at + at_sample(ev) as u64),
                Event::NoteOff { .. } => out.off.push(at + at_sample(ev) as u64),
                _ => {}
            }
        }
        let mut rt = [RealtimeEvent { msg: Realtime::Clock, at_sample: 0 }; REALTIME_PER_RENDER];
        let n = e.take_realtime(&mut rt);
        for x in &rt[..n] {
            if x.msg == Realtime::Clock {
                out.clocks.push(at + x.at_sample as u64);
            }
        }
        at += 128;
    }
    out
}

fn gaps(v: &[u64]) -> Vec<i64> {
    v.windows(2).map(|w| w[1] as i64 - w[0] as i64).collect()
}

fn within(g: &[i64], want: i64, what: &str) {
    assert!(g.len() >= 8, "{what}: too few events ({}) for a test", g.len());
    for (i, x) in g.iter().enumerate() {
        assert!((x - want).abs() <= 1, "{what}: gap {i} is {x} samples, the manual's is {want}");
    }
}

#[test]
fn at_120_bpm_a_default_step_lasts_125_ms_a_sixteenth_note() {
    let r = run(120.0, 4.0, false);
    within(&gaps(&r.on), 6000, "120 BPM, 48 kHz, every step on");
}

#[test]
fn at_120_bpm_a_bar_of_sixteen_steps_lasts_two_seconds() {
    let r = run(120.0, 9.0, false);
    // The same step one loop apart: every 16th Note On.
    let first_of_each_loop: Vec<u64> = r.on.iter().step_by(16).copied().collect();
    within(&gaps(&first_of_each_loop), 96_000, "120 BPM, one loop of 16 steps");
}

#[test]
fn the_step_follows_the_tempo_it_is_halved_at_60_and_doubled_at_240() {
    within(&gaps(&run(60.0, 8.0, false).on), 12_000, "60 BPM");
    within(&gaps(&run(240.0, 3.0, false).on), 3000, "240 BPM");
}

#[test]
fn a_default_note_is_one_step_long() {
    let r = run(120.0, 4.0, false);
    assert!(r.on.len() >= 8 && r.off.len() >= 8, "notes sounded");
    // The gate is not longer than a step (6000 samples) and not shorter than half of one: a 12-tick LEN is a sixteenth.
    let held: Vec<i64> = r.on.iter().zip(r.off.iter()).map(|(a, b)| *b as i64 - *a as i64).collect();
    assert!(held.iter().all(|h| *h > 3000 && *h <= 6001), "held {held:?}");
}

#[test]
fn a_step_is_six_clock_pulses_the_manual_and_the_clock_agree() {
    // 24 pulses to a quarter note, a step a sixteenth: 6 pulses. The clock was always right; the steps were not.
    let r = run(120.0, 4.0, true);
    let pulses = gaps(&r.clocks);
    within(&pulses, 500 * 2, "a pulse at 120 BPM is 1/24 of 0.5 s = 1000 samples");
    for pair in r.on.windows(2).take(12) {
        let between = r.clocks.iter().filter(|c| **c >= pair[0] && **c < pair[1]).count();
        assert_eq!(between, 6, "pulses between the Note Ons at {} and {}", pair[0], pair[1]);
    }
}

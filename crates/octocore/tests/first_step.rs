//! The first step of a Play sounds when Play is pressed (`WENGE-0018`, R0b, `specs/SPEC-0002/p6b-first-step.md`).
//!
//! The manual's numbers, written as numbers (not read back from the engine): at 120 BPM and 48 kHz a 16-step bar is 96,000 samples (p.67),
//! and the first step of a run is step 1, which sounds when the run starts. Before this change every track began its count a whole step
//! early and played step 1 only when that step had run out: 11 ticks after Play, 5,500 samples, 114.6 ms at 120 BPM and 48 kHz.

use octocore::types::StepAttr;
use octocore::{Command, Engine, Event, EventBuffer, RenderContext};

const RATE: f32 = 48_000.0;

fn at_sample(e: &Event) -> u32 {
    match *e {
        Event::NoteOn { at_sample, .. } | Event::NoteOff { at_sample, .. } | Event::Cc { at_sample, .. } | Event::PitchBend { at_sample, .. } | Event::ChannelPressure { at_sample, .. } => at_sample,
    }
}

/// Step 0 only, played from the first sample.
fn first_step_only(bpm: f32, seconds: f32, set: impl Fn(&mut Engine)) -> Vec<u64> {
    let mut e = Engine::new(1);
    e.grid.set_step_attr(0, 0, StepAttr::Active, 1);
    set(&mut e);
    e.handle_command(Command::Play);
    let mut on = vec![];
    let total = (seconds * RATE) as u64;
    let mut at = 0u64;
    while at < total {
        let ctx = RenderContext { sample_rate: RATE, buffer_len: 128, bpm, playing: true };
        let mut buf = EventBuffer::new();
        e.render(&ctx, &mut buf);
        for ev in buf.as_slice() {
            if let Event::NoteOn { .. } = ev {
                on.push(at + at_sample(ev) as u64);
            }
        }
        at += 128;
    }
    on
}

#[test]
fn the_first_step_sounds_the_instant_play_is_pressed_and_comes_round_every_two_seconds() {
    // The engine used to start every track a whole step early in its count and play step 1 only when that step had run out: at 120 BPM
    // the first note came 11 ticks after Play, and the metronome's first click came before it.
    let on = first_step_only(120.0, 9.0, |_| {});
    assert!(on.len() >= 4, "{on:?}");
    assert!(on[0] <= 1, "the first note is at sample {}, Play was at 0", on[0]);
    for (i, w) in on.windows(2).enumerate() {
        assert!((w[1] as i64 - w[0] as i64 - 96_000).abs() <= 1, "loop {i}: {:?}", w);
    }
}

#[test]
fn a_first_step_that_a_negative_start_offset_would_pull_before_play_still_sounds_at_play() {
    // Track STA 16 with a Step STA of -5 is 12 ticks early (the table's reach). Before Play there is nothing to be early for: the
    // note sounds at the first sample and is not lost.
    let on = first_step_only(120.0, 3.0, |e| {
        e.grid.set_track_attr(0, octocore::types::TrackAttr::StartFactor, 16);
        e.grid.set_step_attr(0, 0, StepAttr::StartOffset, -5);
    });
    assert!(!on.is_empty(), "the first note was dropped");
    assert!(on[0] <= 1, "the first note is at sample {}", on[0]);
}

#[test]
fn every_track_with_step_one_on_sounds_it_at_play_not_only_the_first() {
    let mut e = Engine::new(1);
    for t in 0..10u8 {
        e.grid.set_track_attr(t, octocore::types::TrackAttr::MidiChannel, 1);
        e.grid.set_step_attr(t, 0, StepAttr::Active, 1);
    }
    e.handle_command(Command::Play);
    let mut firsts = vec![];
    let mut at = 0u64;
    while at < 4_800 {
        let ctx = RenderContext { sample_rate: RATE, buffer_len: 128, bpm: 120.0, playing: true };
        let mut buf = EventBuffer::new();
        e.render(&ctx, &mut buf);
        for ev in buf.as_slice() {
            if let Event::NoteOn { .. } = ev {
                firsts.push(at + at_sample(ev) as u64);
            }
        }
        at += 128;
    }
    // Ten tracks, ten notes, all inside the first 4,800 samples (100 ms): before this change none sounded before sample 5,500.
    assert_eq!(firsts.len(), 10, "{firsts:?}");
    assert!(firsts.iter().all(|s| *s <= 1), "{firsts:?}");
}

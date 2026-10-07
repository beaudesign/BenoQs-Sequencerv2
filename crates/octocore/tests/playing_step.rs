//! What a snapshot's playhead is (`WENGE-0017`, `specs/SPEC-0002/p6-runs-when-opened.md` R-3): **the step the track is playing**, the one it fired last,
//! in any direction, and 255 before it has fired one.
//!
//! The playhead was the track's position counter, which is advanced as soon as a step fires: it was the step the track would play *next*. In the
//! forward direction that is one more than the step playing, and in Reverse, Ping-pong, Brownian, Random and a custom direction it cannot be
//! worked out from the counter at all. A chase-light drawn from it was a step ahead of the sound, and wrong in every direction but one.
//! The tests are written in the notes a step makes (its pitch is 60 plus its number), so they say what is heard.

use octocore::types::{Command, Snapshot, StepAttr, TrackAttr};
use octocore::Engine;

const NOT_PLAYING: u8 = 255;
const FORWARD: i32 = 1;
const REVERSE: i32 = 2;
const PING_PONG: i32 = 3;

/// Track 0: every step on, step `s` playing note 60 + `s`.
fn engine(direction: i32) -> Engine {
    let mut e = Engine::new(1);
    e.handle_command(Command::SetTrack { track: 0, attr: TrackAttr::Pitch, value: 60 });
    e.handle_command(Command::SetTrack { track: 0, attr: TrackAttr::DirectionRaw, value: direction });
    for s in 0..16u8 {
        e.handle_command(Command::SetStep { track: 0, step: s, attr: StepAttr::Active, value: 1 });
        e.handle_command(Command::SetStep { track: 0, step: s, attr: StepAttr::PitchOffset, value: s as i32 });
    }
    e
}

fn playheads(e: &Engine) -> [u8; 10] {
    let mut s = Snapshot::zeroed();
    e.snapshot(&mut s);
    let mut out = [0u8; 10];
    for (o, p) in out.iter_mut().zip(s.playheads.iter()) {
        *o = p.step_index;
    }
    out
}

/// Runs `ticks` ticks of a playing engine. At every tick, a note on track 0 means the playhead is the step that made it, and no note means the
/// playhead did not move. Returns the steps that played, in order.
fn run_and_check(e: &mut Engine, ticks: u32) -> Vec<u8> {
    e.handle_command(Command::Play);
    let mut played = Vec::new();
    let mut shown = playheads(e)[0];
    for tick in 0..ticks {
        e.step_once_for_test();
        let now = playheads(e)[0];
        match e.last_tick_fires.as_slice().iter().find(|f| f.track == 0) {
            Some(f) => {
                let step = f.pitch - 60;
                assert_eq!(now, step, "tick {tick}: step {step} sounded and the playhead says {now}");
                played.push(step);
            }
            None => assert_eq!(now, shown, "tick {tick}: nothing sounded on track 0 and the playhead moved from {shown} to {now}"),
        }
        shown = now;
    }
    played
}

#[test]
fn before_a_track_has_played_a_step_it_is_playing_none() {
    let mut e = engine(FORWARD);
    assert_eq!(playheads(&e), [NOT_PLAYING; 10], "a new engine");
    e.handle_command(Command::Play);
    assert_eq!(playheads(&e), [NOT_PLAYING; 10], "played, before the first tick");
}

#[test]
fn forward_the_playhead_is_the_step_that_just_sounded_not_the_next() {
    let mut e = engine(FORWARD);
    let played = run_and_check(&mut e, 12 * 40);
    assert!(played.len() >= 36, "{} notes", played.len());
    for (i, s) in played.iter().enumerate() {
        assert_eq!(*s as usize, i % 16, "note {i}");
    }
}

#[test]
fn reverse_it_is_still_the_step_that_sounded() {
    let mut e = engine(REVERSE);
    let played = run_and_check(&mut e, 12 * 40);
    assert_eq!(&played[..4], &[0, 15, 14, 13], "the first step, then back from the last");
}

#[test]
fn ping_pong_it_follows_the_notes_to_each_end_and_back() {
    let mut e = engine(PING_PONG);
    let played = run_and_check(&mut e, 12 * 80);
    assert!(played.contains(&15) && played.windows(2).any(|w| w[0] == 15 && w[1] == 14), "it turned round at the top: {played:?}");
}

#[test]
fn every_track_that_ticks_has_one_even_over_steps_that_are_off() {
    let mut e = engine(FORWARD);
    e.handle_command(Command::Play);
    for _ in 0..40 {
        e.step_once_for_test();
    }
    let heads = playheads(&e);
    for (t, h) in heads.iter().enumerate() {
        assert!(*h < 16, "track {t} is on step {h}: an empty track's light still runs along its steps");
    }
}

#[test]
fn a_reset_puts_it_back_to_none() {
    let mut e = engine(FORWARD);
    run_and_check(&mut e, 60);
    assert!(playheads(&e)[0] < 16);
    e.reset();
    assert_eq!(playheads(&e), [NOT_PLAYING; 10]);
}

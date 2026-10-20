//! SPEC-0005 r1 S0 (WENGE-0033): the Octopus's roll, pinned before anything changes.
//!
//! CV-1 says that with every track on the manual's strum (the default), the engine makes what
//! it made before the `Curve` chord style existed, byte for byte. This test is the pin. It
//! plays one strummed step for every strum level from -9 to 9, every chord size from 1 (a single
//! note, which the Octopus plays seven times) to 7, and 200 seeds, and hashes every event with
//! its absolute sample. The polyphony is varied with the seed (1 to 7), so for a chord larger
//! than its polyphony the engine draws its notes at random and the random stream is part of
//! what is pinned (CV-7 compares it with and without the style).
//!
//! If an intended change to the engine's output moves the numbers below, say why in the commit
//! (as `just golden` does for the example patterns); a change that moves them by accident is the
//! bug this test exists to catch.

use octocore::domain::CHORD_POOL_MAX;
use octocore::{Engine, Event, EventBuffer, RenderContext};

const SAMPLE_RATE: f32 = 48_000.0;
const BUFFER: u32 = 4096;
/// 16 buffers are 65,536 samples, 131 ticks at 120 BPM: past the last note off of a roll (the
/// longest strum starts a note 45 ticks late and a note lasts 12).
const BUFFERS: u32 = 16;
const SEEDS: u64 = 200;

/// FNV-1a, 64 bits. A pin needs a hash that changes when any byte does, not one that resists an
/// adversary; this one needs no dependency.
struct Fnv(u64);

impl Fnv {
    fn new() -> Fnv {
        Fnv(0xcbf2_9ce4_8422_2325)
    }
    fn byte(&mut self, b: u8) {
        self.0 ^= b as u64;
        self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
    }
    fn word(&mut self, v: u64) {
        for b in v.to_le_bytes() {
            self.byte(b);
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

fn feed(h: &mut Fnv, at: u64, e: &Event) {
    h.word(at);
    match *e {
        Event::NoteOn { port, ch, note, vel, .. } => {
            for b in [1u8, port, ch, note, vel] {
                h.byte(b);
            }
        }
        Event::NoteOff { port, ch, note, .. } => {
            for b in [2u8, port, ch, note] {
                h.byte(b);
            }
        }
        Event::Cc { port, ch, cc, val, .. } => {
            for b in [3u8, port, ch, cc, val] {
                h.byte(b);
            }
        }
        Event::PitchBend { port, ch, value, .. } => {
            for b in [4u8, port, ch, (value & 0xff) as u8, (value >> 8) as u8] {
                h.byte(b);
            }
        }
        Event::ChannelPressure { port, ch, value, .. } => {
            for b in [5u8, port, ch, value] {
                h.byte(b);
            }
        }
    }
}

/// One cell: a track with a single active step carrying a chord of `size` notes (1 is a single
/// note) at strum `level`, played by an engine seeded with `seed`. Returns the events, with
/// their absolute samples.
fn play_cell(level: i8, size: usize, seed: u64) -> Vec<(u64, Event)> {
    let mut engine = Engine::new(seed);
    {
        let track = &mut engine.grid.active_page_mut().tracks[0];
        let step = &mut track.steps[0];
        step.active = true;
        step.strum = level;
        if size > 1 {
            let offsets: [i8; CHORD_POOL_MAX] = [3, 4, 7, 10, 12, 16];
            step.chord.count = (size - 1) as u8;
            for k in 0..size - 1 {
                step.chord.offsets[k] = offsets[k];
            }
            step.chord.polyphony = 1 + (seed % 7) as u8;
        }
    }
    let mut out = Vec::new();
    let mut clock = 0u64;
    for _ in 0..BUFFERS {
        let mut buf = EventBuffer::new();
        let ctx = RenderContext { sample_rate: SAMPLE_RATE, buffer_len: BUFFER, bpm: 120.0, playing: true };
        engine.render(&ctx, &mut buf);
        out.extend(buf.as_slice().iter().map(|e| (clock + at_sample(e) as u64, *e)));
        clock += BUFFER as u64;
    }
    out
}

/// What `main` made before S1: the hash of every event of every cell, and how many there were.
const GOLDEN_HASH: u64 = 0x7616_5095_8863_c711;
const GOLDEN_EVENTS: u64 = 237_776;

fn all_cells() -> (u64, u64) {
    let mut h = Fnv::new();
    let mut events = 0u64;
    for level in -9i8..=9 {
        for size in 1..=7usize {
            for seed in 0..SEEDS {
                h.word(level as i64 as u64);
                h.word(size as u64);
                h.word(seed);
                for (at, e) in play_cell(level, size, seed) {
                    feed(&mut h, at, &e);
                    events += 1;
                }
            }
        }
    }
    (h.0, events)
}

#[test]
fn the_manuals_roll_is_what_main_made_for_every_level_size_and_seed() {
    let (hash, events) = all_cells();
    println!("hash {hash:#018x} events {events}");
    assert_eq!(events, GOLDEN_EVENTS, "the number of events moved (hash {hash:#018x})");
    assert_eq!(hash, GOLDEN_HASH, "an event moved: the manual's strum is no longer byte for byte what it was");
}

/// The pin is only worth something if it can fail: one different seed, one different level and
/// one different size each change what is hashed.
#[test]
fn the_pin_can_fail() {
    let digest = |level: i8, size: usize, seed: u64| {
        let mut h = Fnv::new();
        for (at, e) in play_cell(level, size, seed) {
            feed(&mut h, at, &e);
        }
        h.0
    };
    let base = digest(9, 7, 11);
    assert_ne!(base, digest(8, 7, 11), "a level");
    assert_ne!(base, digest(9, 6, 11), "a size");
    assert_ne!(base, digest(9, 7, 12), "a seed");
    assert_eq!(base, digest(9, 7, 11), "and the same cell twice is the same hash");
}

/// Readable anchors under the hash: the manual's worked case (p.22), seven notes at level 9
/// start 0, 5, 10, 17, 23, 30 and 45 ticks after the first, and a strummed single note is seven
/// notes at the same offsets.
#[test]
fn the_manuals_numbers_are_on_the_events() {
    let ticks = |level: i8, size: usize, seed: u64| -> Vec<u64> {
        // 500 samples to the tick at 120 BPM and 48 kHz.
        let mut starts: Vec<u64> = play_cell(level, size, seed)
            .into_iter()
            .filter_map(|(at, e)| matches!(e, Event::NoteOn { .. }).then_some(at))
            .collect();
        starts.sort_unstable();
        let first = starts[0];
        starts.iter().map(|s| (s - first) / 500).collect()
    };
    // Polyphony 1 draws one of seven at random for seed 0; seed 6 has polyphony 7 and plays all seven.
    assert_eq!(ticks(9, 7, 6), vec![0, 5, 10, 17, 23, 30, 45]);
    assert_eq!(ticks(-9, 7, 6), vec![0, 5, 10, 17, 23, 30, 45], "down is the same offsets on the reversed notes");
    assert_eq!(ticks(9, 1, 0), vec![0, 5, 10, 17, 23, 30, 45], "a single note is struck seven times at the same offsets");
    assert_eq!(ticks(0, 1, 0), vec![0], "level 0 is no roll");
}

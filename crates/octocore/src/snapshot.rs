//! `Snapshot` to and from a flat run of 32-bit words, the form the snapshot buffer carries
//! (SPEC-0001 O6, decision D1: no `unsafe`, so the slots are atomic words).
//!
//! Layout, in words: the generation (2), then four per LED (`r`, `g`, `b`, `target`), then
//! two per encoder (`angle_radians`, `detent_index`), then one per playhead (`step_index` and
//! `track_index << 8`), then the transport (`playing`, then `tick` as 2), the mode, and the
//! active bank and page (`bank | page << 8`). Floats travel as their exact bits.

use crate::domain::{Mode, TRACK_COUNT};
use crate::types::{
    ActiveRefs, EncoderState, Led, LedColor, PlayheadState, Snapshot, TransportState, MAX_CONTROLS,
};

const ENCODERS: usize = 20;

pub const SNAPSHOT_WORDS: usize = 2 + 4 * MAX_CONTROLS + 2 * ENCODERS + TRACK_COUNT + 3 + 1 + 1;

impl Snapshot {
    /// An all-zero snapshot: generation 0, every LED off, every encoder at 0, the transport
    /// stopped at tick 0, `Mode::Grid`, page 0 of bank 0.
    pub fn zeroed() -> Snapshot {
        Snapshot {
            generation: 0,
            leds: [Led { color: LedColor { r: 0.0, g: 0.0, b: 0.0 }, target: 0.0 }; MAX_CONTROLS],
            encoders: [EncoderState { angle_radians: 0.0, detent_index: 0 }; ENCODERS],
            playheads: [PlayheadState { step_index: 0, track_index: 0 }; TRACK_COUNT],
            transport: TransportState { playing: false, tick: 0 },
            mode: Mode::Grid,
            active: ActiveRefs { bank: 0, page: 0 },
        }
    }
}

fn put_u64(out: &mut [u32], at: usize, v: u64) {
    out[at] = v as u32;
    out[at + 1] = (v >> 32) as u32;
}

fn get_u64(words: &[u32], at: usize) -> u64 {
    words[at] as u64 | (words[at + 1] as u64) << 32
}

/// Encodes `s` into `out`.
pub fn write_words(s: &Snapshot, out: &mut [u32; SNAPSHOT_WORDS]) {
    put_u64(out, 0, s.generation);
    let mut at = 2;
    for led in &s.leds {
        out[at] = led.color.r.to_bits();
        out[at + 1] = led.color.g.to_bits();
        out[at + 2] = led.color.b.to_bits();
        out[at + 3] = led.target.to_bits();
        at += 4;
    }
    for e in &s.encoders {
        out[at] = e.angle_radians.to_bits();
        out[at + 1] = e.detent_index as u32;
        at += 2;
    }
    for p in &s.playheads {
        out[at] = p.step_index as u32 | (p.track_index as u32) << 8;
        at += 1;
    }
    out[at] = s.transport.playing as u32;
    put_u64(out, at + 1, s.transport.tick);
    out[at + 3] = s.mode as u8 as u32;
    out[at + 4] = s.active.bank as u32 | (s.active.page as u32) << 8;
    debug_assert_eq!(at + 5, SNAPSHOT_WORDS);
}

/// Rebuilds a snapshot. `false`, with `out` untouched, if a field holds a value no
/// `write_words` produces.
pub fn read_words(words: &[u32; SNAPSHOT_WORDS], out: &mut Snapshot) -> bool {
    let tail = 2 + 4 * MAX_CONTROLS + 2 * ENCODERS + TRACK_COUNT;
    let playing = match words[tail] {
        0 => false,
        1 => true,
        _ => return false,
    };
    let Some(mode) = u8::try_from(words[tail + 3]).ok().and_then(Mode::from_u8) else { return false };
    if words[tail + 4] >> 16 != 0 {
        return false;
    }

    out.generation = get_u64(words, 0);
    let mut at = 2;
    for led in out.leds.iter_mut() {
        led.color.r = f32::from_bits(words[at]);
        led.color.g = f32::from_bits(words[at + 1]);
        led.color.b = f32::from_bits(words[at + 2]);
        led.target = f32::from_bits(words[at + 3]);
        at += 4;
    }
    for e in out.encoders.iter_mut() {
        e.angle_radians = f32::from_bits(words[at]);
        e.detent_index = words[at + 1] as i32;
        at += 2;
    }
    for p in out.playheads.iter_mut() {
        p.step_index = words[at] as u8;
        p.track_index = (words[at] >> 8) as u8;
        at += 1;
    }
    out.transport = TransportState { playing, tick: get_u64(words, tail + 1) };
    out.mode = mode;
    out.active = ActiveRefs { bank: words[tail + 4] as u8, page: (words[tail + 4] >> 8) as u8 };
    true
}

#[cfg(all(test, not(loom)))]
mod tests {
    use super::*;
    use crate::domain::{Mode, TRACK_COUNT};
    use crate::rng::Rng;
    use crate::types::MAX_CONTROLS;
    use std::mem::{align_of, offset_of, size_of};

    /// Every field filled with random bits, floats included, so NaN payloads and negative
    /// zeros are in the mix.
    pub(crate) fn random_snapshot(rng: &mut Rng) -> Snapshot {
        let mut s = Snapshot::zeroed();
        let f = |rng: &mut Rng| f32::from_bits(rng.next_u64() as u32);
        s.generation = rng.next_u64();
        for led in s.leds.iter_mut() {
            led.color.r = f(rng);
            led.color.g = f(rng);
            led.color.b = f(rng);
            led.target = f(rng);
        }
        for e in s.encoders.iter_mut() {
            e.angle_radians = f(rng);
            e.detent_index = rng.next_u64() as i32;
        }
        for (i, p) in s.playheads.iter_mut().enumerate() {
            p.step_index = rng.next_u64() as u8;
            p.track_index = i as u8;
        }
        s.transport.playing = rng.next_below(2) == 1;
        s.transport.tick = rng.next_u64();
        s.mode = Mode::ALL[rng.next_below(4) as usize];
        s.active.bank = rng.next_u64() as u8;
        s.active.page = rng.next_u64() as u8;
        s
    }

    #[test]
    fn t5_snapshot_round_trips_bit_for_bit_including_nan_payloads() {
        let mut rng = Rng::new(7);
        for _ in 0..50 {
            let s = random_snapshot(&mut rng);
            let mut words = [0u32; SNAPSHOT_WORDS];
            write_words(&s, &mut words);
            let mut back = Snapshot::zeroed();
            assert!(read_words(&words, &mut back));
            let mut again = [0u32; SNAPSHOT_WORDS];
            write_words(&back, &mut again);
            assert_eq!(words[..], again[..], "the rebuilt snapshot encodes to different words");
            assert_eq!(back.generation, s.generation);
            assert_eq!(back.transport.tick, s.transport.tick);
            assert_eq!(back.mode, s.mode);
        }
    }

    #[test]
    fn t5_a_word_that_is_not_a_mode_is_refused_and_leaves_the_output_alone() {
        let s = Snapshot::zeroed();
        let mut words = [0u32; SNAPSHOT_WORDS];
        write_words(&s, &mut words);
        words[2103] = 9; // the mode word
        let mut out = Snapshot::zeroed();
        out.generation = 77;
        assert!(!read_words(&words, &mut out));
        assert_eq!(out.generation, 77);
    }

    #[test]
    fn t6_snapshot_layout_is_pinned_for_the_c_header() {
        assert_eq!(size_of::<Snapshot>(), 8408);
        assert_eq!(align_of::<Snapshot>(), 8);
        assert_eq!(offset_of!(Snapshot, generation), 0);
        assert_eq!(offset_of!(Snapshot, leds), 8);
        assert_eq!(offset_of!(Snapshot, encoders), 8200);
        assert_eq!(offset_of!(Snapshot, playheads), 8360);
        assert_eq!(offset_of!(Snapshot, transport), 8384);
        assert_eq!(offset_of!(Snapshot, mode), 8400);
        assert_eq!(offset_of!(Snapshot, active), 8401);
        assert_eq!(size_of::<Mode>(), 1);
        assert_eq!(MAX_CONTROLS, 512);
        assert_eq!(TRACK_COUNT, 10);
        // The words carry exactly the fields: 2 + 4 per LED + 2 per encoder + 1 per playhead + 3 + 1 + 1.
        assert_eq!(SNAPSHOT_WORDS, 2 + 4 * MAX_CONTROLS + 2 * 20 + TRACK_COUNT + 3 + 1 + 1);
    }

    #[test]
    fn t6_zeroed_snapshot_is_all_zero_words() {
        let mut words = [1u32; SNAPSHOT_WORDS];
        write_words(&Snapshot::zeroed(), &mut words);
        assert!(words.iter().all(|&w| w == 0));
    }
}

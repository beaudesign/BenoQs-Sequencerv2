//! PR 4a of O4 (SPEC-0001, WENGE-0004): a track's steps fire on the exact tick, for every speed
//! multiplier. Section 2.7 and test R1 of `specs/SPEC-0001/o4-release-plan.md`.
//!
//! The rule being tested is arithmetic, not taste. A track at multiplier `num/den` has a step of
//! `12 * den / num` ticks. The k-th step fires on tick `ceil(k * 12 * den / num)`, counting ticks
//! from 1 after Play, and a step shorter than one tick fires on every tick. Until PR 4a the
//! engine kept the phase in an `f32`, which is a slightly wrong number for most multipliers, so
//! the error builds up: 49 of the 159 reduced multipliers up to 16 fire a step one tick out
//! within 30 minutes, 87 of them by up to 9 ticks in 72 hours (`handoffs/evidence/o4-baseline-c-s-f.txt`).
//! Nothing can set a multiplier from a host today, which is why this is latent, not audible.
//!
//! These tests go through the engine, not through the accumulator alone, so they fail for the
//! reason a player would care about: the note sounds on the wrong tick. The accumulator's own
//! properties (every `u8` pair, huge tick counts, the phase carried over a change) are tested
//! in `src/steps.rs`.

mod common;

use common::*;
use octocore::types::StepAttr;
use octocore::Engine;

/// 120 BPM at 48 kHz is exactly 125 samples per tick, so a note's sample is a whole multiple of
/// 125 and a tick that is one out is 125 samples out, not a rounding question.
const BPM: f32 = 120.0;
const RATE: f32 = 48_000.0;
const SAMPLES_PER_TICK: u64 = 125;

/// The first step fires on tick 12 at ×1, which is the sample of tick index 11 (see
/// `FIRST_NOTE_TICK` in `o4_guards.rs`): the phase starts at 0, so the first step arrives
/// after a whole step of ticks, the 12th call of the tick loop.
fn expected_fire_ticks(num: u64, den: u64, horizon: u64) -> Vec<u64> {
    if 12 * den <= num {
        return (1..=horizon).collect();
    }
    let mut out = Vec::new();
    let mut k = 1u64;
    loop {
        // ceil(k * 12 * den / num), in integers.
        let tick = (k * 12 * den).div_ceil(num);
        if tick > horizon {
            return out;
        }
        out.push(tick);
        k += 1;
    }
}

/// One track, every step on, at the given multiplier.
fn one_track_at(num: u8, den: u8) -> Engine {
    let mut e = Engine::new(1);
    {
        let t = &mut e.grid.active_page_mut().tracks[0];
        t.multiplier_num = num;
        t.multiplier_den = den;
    }
    for s in 0..16u8 {
        e.grid.set_step_attr(0, s, StepAttr::Active, 1);
    }
    e
}

/// The tick (counted from 1) on which each NoteOn sounds, for `ticks` ticks of play.
fn fire_ticks_from_engine(num: u8, den: u8, ticks: u64) -> Vec<u64> {
    let mut host = NullHost::new(one_track_at(num, den), RATE);
    // A few ticks more than asked for, so the last note has been emitted by the lookahead.
    let seconds = ((ticks + 16) * SAMPLES_PER_TICK) as f64 / RATE as f64;
    host.play(&Curve::Constant { bpm: BPM }, seconds, Sizes::Fixed(4096));
    let d = host.diagnostics();
    assert_eq!((d.late_events, d.queue_overflows, d.deferred_events), (0, 0, 0), "x{num}/{den}: {d:?}");
    host.note_on_samples()
        .into_iter()
        .map(|s| {
            assert_eq!(s % SAMPLES_PER_TICK, 0, "x{num}/{den}: a note at sample {s} is not on a tick of {SAMPLES_PER_TICK} samples");
            s / SAMPLES_PER_TICK + 1
        })
        .filter(|&t| t <= ticks)
        .collect()
}

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

/// R1, engine level, the 159 reduced multipliers up to 16 that the baseline measured (F). Each
/// runs 20,000 ticks, about 52 seconds of play at 120 BPM: enough for the `f32` phase to be
/// wrong for many of them, though not for all 49 that are wrong in 30 minutes.
#[test]
fn r1_every_step_fires_on_the_exact_tick_for_the_159_reduced_multipliers_up_to_16() {
    const TICKS: u64 = 20_000;
    let mut wrong = Vec::new();
    let mut tested = 0;
    for den in 1..=16u8 {
        for num in 1..=16u8 {
            if gcd(num as u32, den as u32) != 1 {
                continue;
            }
            tested += 1;
            let got = fire_ticks_from_engine(num, den, TICKS);
            let want = expected_fire_ticks(num as u64, den as u64, TICKS);
            if got != want {
                let first = got.iter().zip(&want).position(|(a, b)| a != b).unwrap_or(got.len().min(want.len()));
                wrong.push(format!(
                    "x{num}/{den}: {} steps fired, {} expected; first difference at step {} (tick {:?} instead of {:?})",
                    got.len(),
                    want.len(),
                    first + 1,
                    got.get(first),
                    want.get(first)
                ));
            }
        }
    }
    assert_eq!(tested, 159, "the set the baseline measured");
    assert!(wrong.is_empty(), "{} of {tested} multipliers fire a step on the wrong tick:\n{}", wrong.len(), wrong.join("\n"));
}

/// R1, engine level, multipliers the baseline did not cover: the extremes of the two `u8`
/// fields and denominators above 16. The accumulator has to be exact for every value the
/// fields can hold, not only the ones a panel will offer.
#[test]
fn r1_every_step_fires_on_the_exact_tick_for_the_extremes_of_the_u8_fields() {
    const TICKS: u64 = 6_000;
    let values = [1u8, 2, 3, 7, 12, 13, 16, 17, 100, 127, 200, 254, 255];
    let mut wrong = Vec::new();
    let mut tested = 0;
    for &den in &values {
        for &num in &values {
            tested += 1;
            let got = fire_ticks_from_engine(num, den, TICKS);
            let want = expected_fire_ticks(num as u64, den as u64, TICKS);
            if got != want {
                wrong.push(format!("x{num}/{den}: {} steps fired, {} expected", got.len(), want.len()));
            }
        }
    }
    assert!(wrong.is_empty(), "{} of {tested} multipliers fire a step on the wrong tick:\n{}", wrong.len(), wrong.join("\n"));
}

/// The rule on the tick itself, on a case anyone can check by hand. At ×9/1 a step is 4/3 ticks
/// long, so step k fires on tick `ceil(k * 4/3)`: 2, 3, 4, 6, 7, 8, 10, 11, 12 for k = 1 to 9.
/// Nine steps in twelve ticks is the multiplier itself.
#[test]
fn r1_the_rule_by_hand_at_nine_times() {
    let got = fire_ticks_from_engine(9, 1, 12);
    assert_eq!(got, vec![2, 3, 4, 6, 7, 8, 10, 11, 12]);
    assert_eq!(expected_fire_ticks(9, 1, 12), got, "the helper the other tests use says the same");
}

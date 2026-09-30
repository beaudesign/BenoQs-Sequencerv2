//! PR 4a of O4 (SPEC-0001, WENGE-0004): a track's steps fire on the exact tick, for every speed
//! multiplier, and through every change of multiplier. Section 2.7 and test R1 of
//! `specs/SPEC-0001/o4-release-plan.md`.
//!
//! The rule being tested is arithmetic, not taste. A track at multiplier `num/den` has a step of
//! `DEFAULT_STEP_TICKS * den / num` ticks (12 today). The k-th step fires on tick
//! `ceil(k * 12 * den / num)`, counting ticks from 1 from a fresh or reset engine (Stop keeps the
//! phase, as before), and a step shorter than one tick fires on every tick. Until PR 4a the engine
//! kept the phase in an `f32`, which is a slightly wrong number for most multipliers, so the
//! error builds up: 49 of the 159 reduced multipliers up to 16 fire a step one tick late (all 49
//! are wrong within the 20,000 ticks tested here; the baseline measured the same 49 over 30
//! minutes, and 87 with errors up to 9 ticks over 72 hours, `handoffs/evidence/o4-baseline-c-s-f.txt`).
//! Nothing can set a multiplier from a host today, which is why this is latent, not audible.
//!
//! These tests go through the engine, not through the accumulator alone, so they fail for the
//! reason a player would care about: the note sounds on the wrong tick. The accumulator's own
//! properties (every `u8` pair, huge tick counts, the phase carried over a change, an exact
//! reference for changing multipliers) are tested in `src/steps.rs`.
//!
//! The tick length is read from the engine's constants, as in the null host, so a decision on D0
//! moves these tests with it: `DEFAULT_STEP_TICKS` for the step and `TICKS_PER_QUARTER` for the
//! samples in a tick. The one hand-worked case says so.

mod common;

use common::*;
use octocore::domain::{DEFAULT_STEP_TICKS, TICKS_PER_QUARTER};
use octocore::types::StepAttr;
use octocore::Engine;

/// A tempo and rate for which a tick is a whole number of samples, so a note's sample is a whole
/// multiple of it and a tick that is one out is a whole tick of samples out, not a rounding
/// question: 120 BPM at 48 kHz is 24,000 / `TICKS_PER_QUARTER` samples a tick (125 at 192).
const BPM: f32 = 120.0;
const RATE: f32 = 48_000.0;
const SAMPLES_PER_TICK: u64 = 24_000 / TICKS_PER_QUARTER as u64;
const STEP: u64 = DEFAULT_STEP_TICKS as u64;

/// The tick (counted from 1) on which the k-th step of a clock that has run at `num/den` from the
/// start fires, for k = 1, 2, ... up to `horizon`.
fn expected_fire_ticks(num: u64, den: u64, horizon: u64) -> Vec<u64> {
    if STEP * den <= num {
        return (1..=horizon).collect();
    }
    let mut out = Vec::new();
    let mut k = 1u64;
    loop {
        let tick = (k * STEP * den).div_ceil(num);
        if tick > horizon {
            return out;
        }
        out.push(tick);
        k += 1;
    }
}

/// An exact reference for a clock whose rate changes: the phase in ticks as a reduced fraction.
/// It shares no code with the engine.
struct Exact {
    n: u128,
    d: u128,
}

fn gcd(a: u128, b: u128) -> u128 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

impl Exact {
    fn new() -> Exact {
        Exact { n: 0, d: 1 }
    }

    /// One tick at a step of `den / num` ticks (a step of a tick or less fires every tick).
    fn tick(&mut self, num: u128, den: u128) -> bool {
        if num >= den {
            return true;
        }
        let (top, bottom) = (self.n + self.d, self.d);
        let fired = top * num >= den * bottom;
        let (n, d) = if fired { (top * num - den * bottom, bottom * num) } else { (top, bottom) };
        let r = gcd(n, d).max(1);
        (self.n, self.d) = (n / r, d / r);
        fired
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
fn fire_ticks(engine: Engine, ticks: u64, label: &str) -> Vec<u64> {
    let mut host = NullHost::new(engine, RATE);
    // A few ticks more than asked for, so the last note has been emitted by the lookahead.
    let seconds = ((ticks + 16) * SAMPLES_PER_TICK) as f64 / RATE as f64;
    host.play(&Curve::Constant { bpm: BPM }, seconds, Sizes::Fixed(4096));
    let d = host.diagnostics();
    assert_eq!((d.late_events, d.queue_overflows, d.deferred_events), (0, 0, 0), "{label}: {d:?}");
    host.note_on_samples()
        .into_iter()
        .map(|s| {
            assert_eq!(s % SAMPLES_PER_TICK, 0, "{label}: a note at sample {s} is not on a tick of {SAMPLES_PER_TICK} samples");
            s / SAMPLES_PER_TICK + 1
        })
        .filter(|&t| t <= ticks)
        .collect()
}

fn fire_ticks_from_engine(num: u8, den: u8, ticks: u64) -> Vec<u64> {
    fire_ticks(one_track_at(num, den), ticks, &format!("x{num}/{den}"))
}

fn gcd32(a: u32, b: u32) -> u32 {
    gcd(a as u128, b as u128) as u32
}

fn first_difference(got: &[u64], want: &[u64]) -> String {
    let first = got.iter().zip(want).position(|(a, b)| a != b).unwrap_or(got.len().min(want.len()));
    format!("{} steps fired, {} expected; first difference at step {} (tick {:?} instead of {:?})", got.len(), want.len(), first + 1, got.get(first), want.get(first))
}

/// R1, engine level, the 159 reduced multipliers up to 16 that the baseline measured (F). Each
/// runs 20,000 ticks, about 52 seconds of play at 120 BPM.
#[test]
fn r1_every_step_fires_on_the_exact_tick_for_the_159_reduced_multipliers_up_to_16() {
    const TICKS: u64 = 20_000;
    let mut wrong = Vec::new();
    let mut tested = 0;
    for den in 1..=16u8 {
        for num in 1..=16u8 {
            if gcd32(num as u32, den as u32) != 1 {
                continue;
            }
            tested += 1;
            let got = fire_ticks_from_engine(num, den, TICKS);
            let want = expected_fire_ticks(num as u64, den as u64, TICKS);
            if got != want {
                wrong.push(format!("x{num}/{den}: {}", first_difference(&got, &want)));
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
                wrong.push(format!("x{num}/{den}: {}", first_difference(&got, &want)));
            }
        }
    }
    assert!(wrong.is_empty(), "{} of {tested} multipliers fire a step on the wrong tick:\n{}", wrong.len(), wrong.join("\n"));
}

/// The rule on the tick itself, on a case anyone can check by hand **at 12 ticks a step**. At x9/1
/// a step is 4/3 ticks long, so step k fires on tick `ceil(k * 4/3)`: 2, 3, 4, 6, 7, 8, 10, 11, 12
/// for k = 1 to 9. Nine steps in twelve ticks is the multiplier itself. If D0 changes
/// `DEFAULT_STEP_TICKS` this hand case is the one to redo, and the assertion below says so.
#[test]
fn r1_the_rule_by_hand_at_nine_times() {
    assert_eq!(DEFAULT_STEP_TICKS, 12, "the hand-worked list below is for a 12-tick step");
    let got = fire_ticks_from_engine(9, 1, 12);
    assert_eq!(got, vec![2, 3, 4, 6, 7, 8, 10, 11, 12]);
    assert_eq!(expected_fire_ticks(9, 1, 12), got, "the helper the other tests use says the same");
}

/// R1, engine level, **a multiplier that changes while the track plays**. A chain (head track 9,
/// member track 8) plays its head's 16 steps, then its member's 16, and so on, and each takes its
/// own multiplier: the rate changes every 16 steps, which is the one route by which a rate can
/// change today. An independent review found that a phase rounded down at each change lost up to a
/// tick each time and ran 0.26 % short (27,708 steps against 27,781 in 200,000 ticks at x5 then
/// x1). The reference is exact and shares no code with the engine.
#[test]
fn r1_a_chain_that_switches_multiplier_every_16_steps_stays_exact() {
    const TICKS: u64 = 100_000;
    for (a, b) in [((5u8, 1u8), (1u8, 1u8)), ((7, 3), (1, 1)), ((9, 4), (5, 7)), ((1, 3), (255, 254))] {
        let mut e = Engine::new(1);
        {
            let page = e.grid.active_page_mut();
            page.tracks[9].multiplier_num = a.0;
            page.tracks[9].multiplier_den = a.1;
            page.tracks[8].multiplier_num = b.0;
            page.tracks[8].multiplier_den = b.1;
            page.tracks[9].chain_members[0] = Some(8);
            page.tracks[9].chain_member_count = 1;
            page.tracks[8].chain_head = Some(9);
        }
        for t in [9u8, 8] {
            for s in 0..16u8 {
                e.grid.set_step_attr(t, s, StepAttr::Active, 1);
            }
        }
        let got = fire_ticks(e, TICKS, &format!("chain x{}/{} then x{}/{}", a.0, a.1, b.0, b.1));
        // The exact reference: 16 steps at `a`, 16 at `b`, and so on.
        let mut exact = Exact::new();
        let (mut want, mut fires, mut rate) = (Vec::new(), 0u64, a);
        for tick in 1..=TICKS {
            if exact.tick(rate.0 as u128, STEP as u128 * rate.1 as u128) {
                want.push(tick);
                fires += 1;
                if fires % 16 == 0 {
                    rate = if rate == a { b } else { a };
                }
            }
        }
        assert!(want.len() > 1000, "x{}/{} then x{}/{}: {} steps is too few to test anything", a.0, a.1, b.0, b.1, want.len());
        assert_eq!(got, want, "x{}/{} then x{}/{}: {}", a.0, a.1, b.0, b.1, first_difference(&got, &want));
    }
}

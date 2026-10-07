//! Exact per-track step timing (SPEC-0001 O4, PR 4a; WENGE-0004; release plan section 2.7).
//!
//! A track at speed multiplier `num/den` has a step `12 * den / num` ticks long. Until this module
//! existed the engine kept the phase as an `f32` count of ticks and compared it with an `f32` step
//! length, patched with `+ 1e-6`. Most step lengths are not exactly representable, so the phase is
//! a little wrong after every step and the error builds up: 49 of the 159 reduced multipliers up
//! to 16 fired a step one tick late within 20,000 ticks (`handoffs/evidence/o4-4a-red.txt`).
//!
//! Here the phase is a **whole number of `1/unit`-tick units**, and `unit` is always a multiple of
//! the multiplier's numerator. Every tick adds `unit` (one tick) and a step starts when the phase
//! reaches `12 * den * unit / num`, which is then subtracted. Everything is an integer, so for a
//! constant multiplier the k-th step starts on tick `ceil(k * 12 * den / num)` for every `u8` pair
//! at any tick count, and the phase never leaves `0 .. 12 * den + num` units of `1/num` tick, so
//! nothing grows with time.
//!
//! **A change of multiplier carries the phase exactly.** The point in the step, a fraction of a
//! tick, is kept as a reduced fraction; the new `unit` is the smallest multiple of the new
//! numerator that holds it. Chained tracks change multiplier every 16 steps, and a version that
//! rounded the phase down at each change lost up to a tick each time, always late (an independent
//! review measured 73 steps short in 200,000 ticks, 0.26 %, on a chain of x5 and x1). Only if the
//! `unit` would pass 2^80, which takes a long series of unrelated multipliers with nothing
//! reducing the fraction between them, is the phase rounded down to the new numerator's unit,
//! losing under one unit (`1/num` tick) once.
//!
//! The arithmetic is in ticks. It does not depend on what a tick is worth in samples, so it does
//! not depend on the tick-length question (D0, `WENGE-0012`).

/// How long a track's step is: `den / num` ticks. The phase advances by `num` each tick and a
/// step starts when it reaches `den`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepRate {
    num: u32,
    den: u32,
}

impl StepRate {
    /// The step of a track with speed multiplier `mult_num / mult_den`, as the two `u8` fields
    /// hold it, when a step at ×1 is `default_step_ticks` long. A zero denominator reads as ×1, as
    /// `Track::multiplier` always did. A zero numerator is a speed of nothing: the track never
    /// starts a step. (The `f32` code clamped the multiplier to `1e-6`, a step of 12 million
    /// ticks, about 9 hours at 120 BPM; never is what that meant.)
    pub const fn from_multiplier(mult_num: u8, mult_den: u8, default_step_ticks: u32) -> StepRate {
        if mult_den == 0 {
            return StepRate { num: 1, den: default_step_ticks };
        }
        StepRate { num: mult_num as u32, den: default_step_ticks * mult_den as u32 }
    }

    /// A step of exactly `ticks` ticks: a hyperstep-linked track is triggered at the step's
    /// absolute length, 192 (CE v5.30 p.31).
    pub const fn fixed(ticks: u32) -> StepRate {
        StepRate { num: 1, den: ticks }
    }

    /// A step of a tick or less starts on every tick, as `max(1.0)` did.
    const fn every_tick(&self) -> bool {
        self.num >= self.den
    }
}

/// The largest `unit` a clock keeps. Past it a change of multiplier rounds instead of carrying
/// the phase exactly. 2^80 leaves `phase` (at most a few thousand ticks of it) far inside `u128`.
const MAX_UNIT: u128 = 1 << 80;

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

/// One track's position within its current step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepClock {
    /// Ticks into the current step, in units of `1/unit` tick.
    phase: u128,
    unit: u128,
    /// The numerator of the rate the clock last ran at. It always divides `unit`.
    num: u32,
}

impl Default for StepClock {
    fn default() -> Self {
        StepClock::new()
    }
}

impl StepClock {
    pub const fn new() -> StepClock {
        StepClock { phase: 0, unit: 1, num: 1 }
    }

    /// A clock that is `phase` units of `1/unit` tick into its step, running at a multiplier whose
    /// numerator is `unit` (a `unit` of 0 is read as 1). Used by tests to start a track as if it
    /// had already played for a very long time.
    pub const fn at(phase: u32, unit: u32) -> StepClock {
        let unit = if unit == 0 { 1 } else { unit };
        StepClock { phase: phase as u128, unit: unit as u128, num: unit }
    }

    /// The phase and its unit: `phase` units of `1/unit` tick. For tests and diagnostics.
    pub const fn phase(&self) -> (u128, u128) {
        (self.phase, self.unit)
    }

    /// Moves to the multiplier numerator `num`, keeping the same point in the step exactly:
    /// `phase / unit` ticks, as a reduced fraction `p/d`, becomes `phase' / unit'` with `unit'` the
    /// smallest multiple of `num` that is also a multiple of `d`. Only if that passes `MAX_UNIT` is
    /// the phase rounded down to a unit of `1/num` tick, which loses less than one unit.
    fn retune(&mut self, num: u32) {
        let g = gcd(self.phase, self.unit);
        let d = self.unit / g;
        let n = num as u128;
        let unit = d / gcd(d, n) * n;
        if unit <= MAX_UNIT {
            self.phase = self.phase / g * (unit / d);
            self.unit = unit;
        } else {
            self.phase = self.phase * n / self.unit;
            self.unit = n;
        }
        self.num = num;
    }

    /// One tick. Returns true if a step starts on it.
    pub fn tick(&mut self, rate: StepRate) -> bool {
        if rate.num == 0 {
            // A speed of nothing: no progress, and the phase is kept for when it comes back. (The
            // `f32` code went on counting and fired a burst of steps on the way out.)
            return false;
        }
        if rate.num != self.num {
            self.retune(rate.num);
        }
        if rate.every_tick() {
            // A step of a tick or less. The phase is carried through untouched, as the `f32`
            // accumulator's `+ 1 - 1` did, so leaving this state resumes where it left off.
            return true;
        }
        // One tick is `unit` units and a step is `den / num` ticks, `den * (unit / num)` units.
        self.phase += self.unit;
        let step = rate.den as u128 * (self.unit / rate.num as u128);
        if self.phase >= step {
            self.phase -= step;
            true
        } else {
            false
        }
    }
}

/// An exact reference for the tests: the phase as a reduced fraction of a tick, in `u128`, with no
/// cleverness. Kept apart from `StepClock` on purpose: it shares no code with it, so agreeing with
/// it means something. The phase is in **ticks**, as the `f32` accumulator's was, so it carries
/// across a change of multiplier as a point in time and not as a fraction of a step.
#[cfg(test)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct ExactClock {
    n: u128,
    d: u128,
}

#[cfg(test)]
impl ExactClock {
    pub(crate) fn new() -> ExactClock {
        ExactClock { n: 0, d: 1 }
    }

    /// One tick at a rate whose step is `den / num` ticks: the phase gains a tick and a step
    /// starts when it reaches `den / num`, which is then taken off.
    pub(crate) fn tick(&mut self, rate: StepRate) -> bool {
        if rate.num == 0 {
            return false;
        }
        if rate.num >= rate.den {
            return true;
        }
        let (num, den) = (rate.num as u128, rate.den as u128);
        // (n + d) / d ticks against den / num ticks: (n + d) * num against den * d.
        let (top, bottom) = (self.n + self.d, self.d);
        let fired = top * num >= den * bottom;
        let (n, d) = if fired { (top * num - den * bottom, bottom * num) } else { (top, bottom) };
        let r = gcd(n, d).max(1);
        (self.n, self.d) = (n / r, d / r);
        fired
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STEP: u32 = 12;

    fn rate(num: u8, den: u8) -> StepRate {
        StepRate::from_multiplier(num, den, STEP)
    }

    fn gcd32(a: u32, b: u32) -> u32 {
        gcd(a as u128, b as u128) as u32
    }

    /// The exact tick (counted from 1) of the k-th step at multiplier `num/den`, for a clock that
    /// has run at that multiplier from the start.
    fn exact_tick(num: u64, den: u64, k: u64) -> u64 {
        if STEP as u64 * den <= num {
            k
        } else {
            (k * STEP as u64 * den).div_ceil(num)
        }
    }

    // ---- R1: exact for every u8 pair, at any tick count

    /// R1a. For every `num` and `den` from 1 to 255 (65,025 pairs) the k-th step starts on tick
    /// `ceil(k * 12 * den / num)`. After n ticks the phase is `n * num mod (12 * den)`, so the
    /// whole behaviour repeats every `12 * den / gcd` ticks: two full periods prove it for every
    /// tick count that follows. R1b checks the invariant that makes that true, and R1c checks it
    /// far from tick 0.
    #[test]
    fn r1a_every_u8_pair_fires_on_the_exact_tick() {
        for den in 1..=255u32 {
            for num in 1..=255u32 {
                let r = rate(num as u8, den as u8);
                let period = STEP * den / gcd32(STEP * den, num);
                let ticks = 2 * period as u64 + 10;
                let mut clock = StepClock::new();
                let mut k = 0u64;
                let mut next = exact_tick(num as u64, den as u64, 1);
                for tick in 1..=ticks {
                    let fired = clock.tick(r);
                    assert_eq!(fired, tick == next, "x{num}/{den}: tick {tick}, step {} is due on tick {next}", k + 1);
                    if fired {
                        k += 1;
                        next = exact_tick(num as u64, den as u64, k + 1);
                    }
                }
            }
        }
    }

    /// R1b. The invariant that makes the arithmetic exact at any tick count: after n ticks at a
    /// constant multiplier the phase is `n * num mod (12 * den)` units of `1/num` tick, and the
    /// unit stays `num`. Nothing grows with time, so nothing can drift. Checked over three periods
    /// for every pair whose step is longer than a tick.
    #[test]
    fn r1b_the_phase_is_bounded_and_equals_n_times_num_mod_the_step() {
        for den in 1..=255u32 {
            for num in (1..=255u32).filter(|&n| n < STEP * den) {
                let r = rate(num as u8, den as u8);
                let step = STEP * den;
                let mut clock = StepClock::new();
                for n in 1..=3 * (step / gcd32(step, num)) as u64 {
                    clock.tick(r);
                    let (phase, unit) = clock.phase();
                    assert_eq!(unit, num as u128);
                    assert_eq!(phase, n as u128 * num as u128 % step as u128, "x{num}/{den} after {n} ticks");
                }
            }
        }
    }

    /// R1c. A clock started at the phase it would have after N ticks steps exactly like the closed
    /// form from there, for N up to 10^12 and 2^40. It does **not** run 10^12 ticks: it starts at
    /// the phase of tick N, which is `N * num mod (12 * den)`. That is enough because R1b shows the
    /// phase is a bounded function of N and nothing else, so a clock that has played for a thousand
    /// years is a clock at some phase, and this checks a clock at those phases.
    #[test]
    fn r1c_a_clock_started_at_the_phase_of_a_huge_tick_steps_like_the_closed_form() {
        for &n0 in &[1_000_000u64, 999_999_999, 1_000_000_000_000, 1u64 << 40] {
            for &(num, den) in &[(1u32, 1u32), (9, 1), (9, 4), (5, 12), (7, 16), (255, 254), (254, 255), (13, 255), (100, 3), (255, 1)] {
                let r = rate(num as u8, den as u8);
                let step = STEP as u64 * den as u64;
                let mut clock = StepClock::at((n0 as u128 * num as u128 % step as u128) as u32, num);
                for tick in n0 + 1..=n0 + 5_000 {
                    let fired = clock.tick(r);
                    // A step starts on tick n exactly when a whole step's worth of phase has come
                    // in: floor(n * num / step) exceeds floor((n - 1) * num / step). A step of a
                    // tick or less starts on every tick.
                    let want = step <= num as u64 || tick as u128 * num as u128 / step as u128 > (tick as u128 - 1) * num as u128 / step as u128;
                    assert_eq!(fired, want, "x{num}/{den} at tick {tick}");
                }
            }
        }
    }

    /// R1d. The 159 reduced multipliers up to 16 that the baseline measured (F), for 10^6 ticks
    /// (2.9 hours at 120 BPM now: 96 ticks a second since D0, `p6a-tick-length.md`; it was 43 minutes at the old 384 a second, and the baseline's 691,200 ticks
    /// were its 30 minutes):
    /// every step on the exact tick. The 10^7-tick run the plan names is the ignored soak below.
    #[test]
    fn r1d_the_159_reduced_multipliers_up_to_16_stay_exact_for_a_million_ticks() {
        run_reduced_pairs(1_000_000);
    }

    /// The 10^7-tick soak the plan names: `cargo test --release -p octocore --lib -- --ignored r1d`.
    #[test]
    #[ignore = "10^7 ticks for 159 multipliers: seconds in release, minutes in debug"]
    fn r1d_soak_the_159_reduced_multipliers_up_to_16_stay_exact_for_ten_million_ticks() {
        run_reduced_pairs(10_000_000);
    }

    fn run_reduced_pairs(horizon: u64) {
        let mut tested = 0;
        for den in 1..=16u32 {
            for num in 1..=16u32 {
                if gcd32(num, den) != 1 {
                    continue;
                }
                tested += 1;
                let r = rate(num as u8, den as u8);
                let mut clock = StepClock::new();
                let mut k = 0u64;
                let mut next = exact_tick(num as u64, den as u64, 1);
                for tick in 1..=horizon {
                    let fired = clock.tick(r);
                    assert_eq!(fired, tick == next, "x{num}/{den}: tick {tick}, step {} is due on tick {next}", k + 1);
                    if fired {
                        k += 1;
                        next = exact_tick(num as u64, den as u64, k + 1);
                    }
                }
            }
        }
        assert_eq!(tested, 159);
    }

    // ---- R1e, R1f: a change of multiplier carries the phase exactly

    const RATES: [(u8, u8); 14] = [(1, 1), (1, 2), (2, 1), (3, 2), (2, 3), (5, 7), (7, 5), (16, 1), (1, 16), (13, 16), (255, 254), (254, 255), (255, 1), (1, 255)];

    /// R1e. Changing the multiplier keeps the point in the step **exactly**: `phase / unit` ticks
    /// is the same fraction before and after, and the new unit is a multiple of the new
    /// numerator. Every pair of rates from a set that includes the extremes, at every phase of the
    /// old step.
    #[test]
    fn r1e_a_multiplier_change_keeps_the_point_in_the_step_exactly() {
        for (n0, d0) in RATES {
            for (n1, _) in RATES {
                for phase in 0..STEP * d0 as u32 {
                    let mut clock = StepClock::at(phase, n0 as u32);
                    let (p0, u0) = clock.phase();
                    clock.retune(n1 as u32);
                    let (p1, u1) = clock.phase();
                    assert_eq!(u1 % n1 as u128, 0, "x{n0}/{d0} to x{n1}: the unit {u1} is not a multiple of the numerator");
                    assert_eq!(p0 * u1, p1 * u0, "x{n0}/{d0} to x{n1}, phase {phase}/{n0}: {p1}/{u1} is not the same point");
                }
            }
        }
    }

    /// R1f. The reviewer's case, in general: rates that change while the clock runs, at random
    /// times and at step boundaries, agree with an exact reference (`ExactClock`, a reduced
    /// fraction that shares no code with `StepClock`) on every tick, for 300,000 ticks each. The
    /// round-down version of this module was 0.26 % short on x5 and x1 alternating every 16 steps
    /// and failed this; the f32 code passed it to within rounding.
    #[test]
    fn r1f_a_clock_whose_multiplier_keeps_changing_agrees_with_the_exact_reference() {
        let mut rng = crate::rng::Rng::new(4242);
        for run in 0..12u32 {
            let mut clock = StepClock::new();
            let mut exact = ExactClock::new();
            let mut r = RATES[rng.next_below(RATES.len() as u32) as usize];
            let (mut fires, mut changes) = (0u64, 0u64);
            for tick in 1..=300_000u64 {
                // Change the rate at a fire (as a chain does) or at a random tick (as an edit does).
                let (a, b) = (clock.tick(rate(r.0, r.1)), exact.tick(rate(r.0, r.1)));
                assert_eq!(a, b, "run {run}, tick {tick}, x{}/{} after {changes} changes and {fires} steps", r.0, r.1);
                if a {
                    fires += 1;
                }
                if (a && fires % 16 == 0) || rng.next_below(200) == 0 {
                    r = RATES[rng.next_below(RATES.len() as u32) as usize];
                    changes += 1;
                }
            }
            assert!(changes > 500 && fires > 500, "run {run}: {changes} changes and {fires} steps is too few to test anything");
        }
    }

    /// The reviewer's own case: x5 for 16 steps, then x1 for 16 steps, and so on. The exact count
    /// of steps in 200,000 ticks is 27,781 (the number the f32 code and the reference give); the
    /// round-down version gave 27,708.
    #[test]
    fn a_chain_of_x5_and_x1_fires_27781_steps_in_200000_ticks() {
        let mut clock = StepClock::new();
        let (mut fires, mut r) = (0u64, (5u8, 1u8));
        for _ in 0..200_000u32 {
            if clock.tick(rate(r.0, r.1)) {
                fires += 1;
                if fires % 16 == 0 {
                    r = if r.0 == 5 { (1, 1) } else { (5, 1) };
                }
            }
        }
        assert_eq!(fires, 27_781);
    }

    /// A long run of changes through unrelated multipliers, with steps starting in between (each
    /// one leaves a remainder with the new numerator in its denominator), stays exact while the
    /// unit is within 2^80, and past that rounds the phase down once instead of overflowing. The
    /// fallback keeps `unit <= MAX_UNIT`, keeps the unit a multiple of the numerator, and loses
    /// less than one unit.
    #[test]
    fn a_long_run_of_unrelated_multipliers_never_overflows_and_falls_back_to_rounding() {
        let primes = [251u32, 241, 239, 233, 229, 227, 223, 211, 199, 197, 193, 191, 181, 179, 173, 167];
        let mut clock = StepClock::new();
        let (mut fell_back, mut exact_changes) = (0, 0);
        for &p in primes.iter().cycle().take(400) {
            // Change to the multiplier, then run until a step starts: at x p/255 a step is 12 * 255 / p
            // ticks, about 12, and its remainder has p in the denominator.
            let before = clock.phase();
            let r = rate(p as u8, 255);
            for _ in 0..20 {
                if clock.tick(r) {
                    break;
                }
            }
            let (phase, unit) = clock.phase();
            assert!(unit <= MAX_UNIT, "unit {unit} passed the bound");
            assert_eq!(unit % p as u128, 0);
            // What the change alone would have needed.
            let d = before.1 / gcd(before.0, before.1);
            if d / gcd(d, p as u128) * p as u128 > MAX_UNIT {
                fell_back += 1;
                assert!(phase < 40 * unit, "phase {phase} of unit {unit}: a few ticks at most");
            } else {
                exact_changes += 1;
            }
        }
        assert!(fell_back > 0, "the fallback never ran, so it is untested");
        assert!(exact_changes > 0);
    }

    /// The fallback alone: it rounds down, never up, and loses under one unit of the new rate.
    #[test]
    fn the_fallback_rounds_down_by_less_than_one_unit() {
        // A phase whose reduced denominator is 2^81 forces the fallback for any numerator.
        let mut clock = StepClock { phase: 12_345_678_901_234_567_891, unit: (1u128 << 81) + 1, num: 1 };
        let (p0, u0) = clock.phase();
        clock.retune(7);
        let (p1, u1) = clock.phase();
        assert_eq!(u1, 7);
        // p1 / 7 <= p0 / u0 < (p1 + 1) / 7
        assert!(p1 * u0 <= p0 * 7 && p0 * 7 < (p1 + 1) * u0);
    }

    /// The unit is the smallest that holds the phase, so it does not creep up over a long session:
    /// a phase that is a whole number of ticks needs only the new numerator's unit, and a phase of
    /// one third of a tick needs a multiple of 3 as well. (A clock that kept every unit it had ever
    /// needed would reach the 2^80 bound, and start rounding, sooner than it has to.)
    #[test]
    fn a_change_of_multiplier_uses_the_smallest_unit_that_holds_the_phase() {
        let mut whole = StepClock::at(0, 7);
        whole.tick(rate(5, 200));
        assert_eq!(whole.phase().1, 5, "a phase of 0 ticks: just the new numerator's unit");
        // 1/3 of a tick, in units of 1/21: 7 units. Moving to a multiplier numerator of 5 needs 15.
        let mut third = StepClock::at(7, 21);
        third.retune(5);
        let (p, u) = third.phase();
        assert_eq!((p, u), (5, 15), "1/3 of a tick is 5/15");
    }

    /// The hand case behind R1e, and the behaviour of the `f32` code it replaces: five ticks into
    /// a x1 step (12 ticks), the multiplier becomes x3 (4 ticks a step). Five ticks of phase is
    /// more than a whole new step, so the next tick starts one, with 2 ticks carried, and so on.
    #[test]
    fn a_change_from_times_one_to_times_three_after_five_ticks() {
        let mut clock = StepClock::new();
        for _ in 0..5 {
            assert!(!clock.tick(rate(1, 1)));
        }
        let fired: Vec<bool> = (0..6).map(|_| clock.tick(rate(3, 1))).collect();
        // Phase 5 ticks, then +1 each tick against a step of 4: 6 (fire, 2 left), 3, 4 (fire, 0), 1, 2, 3.
        assert_eq!(fired, vec![true, false, true, false, false, false]);
    }

    // ---- the edges of the rules

    /// A step of a tick or less starts on every tick and does not disturb the phase.
    #[test]
    fn a_step_of_a_tick_or_less_fires_on_every_tick_and_keeps_the_phase() {
        for (num, den) in [(12u8, 1u8), (13, 1), (255, 1), (255, 21), (24, 2)] {
            let mut clock = StepClock::new();
            for _ in 0..1000 {
                assert!(clock.tick(rate(num, den)), "x{num}/{den}");
            }
        }
        // Carried through: 3 ticks into a x1 step, a while at x255, then x1 again. The step has 9
        // ticks left, so the first step to start is on the 9th tick back.
        let mut clock = StepClock::new();
        for _ in 0..3 {
            assert!(!clock.tick(rate(1, 1)));
        }
        for _ in 0..100 {
            assert!(clock.tick(rate(255, 1)));
        }
        let mut ticks_until_step = 0;
        loop {
            ticks_until_step += 1;
            if clock.tick(rate(1, 1)) {
                break;
            }
        }
        assert_eq!(ticks_until_step, 9);
    }

    /// A zero denominator reads as x1 (as `Track::multiplier` did). A zero numerator never fires and
    /// **freezes** the phase: 5 ticks into a step, a thousand ticks at x0, then x1 again fires 7 ticks
    /// later, once. The f32 code went on counting during the x0 ticks and fired a burst of steps, one
    /// per tick, when the multiplier came back (measured by an independent review: 91 in a row).
    #[test]
    fn zero_denominator_is_times_one_and_zero_numerator_freezes_the_phase() {
        let mut a = StepClock::new();
        let mut b = StepClock::new();
        for _ in 0..1000 {
            assert_eq!(a.tick(rate(7, 0)), b.tick(rate(1, 1)));
        }
        let mut z = StepClock::new();
        for _ in 0..5 {
            assert!(!z.tick(rate(1, 1)));
        }
        assert!((0..1000).all(|_| !z.tick(rate(0, 5))));
        let fired: Vec<bool> = (0..8).map(|_| z.tick(rate(1, 1))).collect();
        assert_eq!(fired, vec![false, false, false, false, false, false, true, false]);
    }

    /// A hyperstep-linked track has a fixed 192-tick step, whatever its multiplier.
    #[test]
    fn a_fixed_step_fires_every_that_many_ticks() {
        let mut clock = StepClock::new();
        let mut at = Vec::new();
        for t in 1..=600u64 {
            if clock.tick(StepRate::fixed(192)) {
                at.push(t);
            }
        }
        assert_eq!(at, vec![192, 384, 576]);
    }

    /// `StepClock::at` with a unit of 0 (a caller's slip) is read as 1, not divided by.
    #[test]
    fn a_clock_started_with_a_zero_unit_does_not_divide_by_zero() {
        let mut clock = StepClock::at(0, 0);
        assert!(!clock.tick(rate(1, 1)));
    }
}

//! Exact per-track step timing (SPEC-0001 O4, PR 4a; WENGE-0004; release plan section 2.7).
//!
//! A track at speed multiplier `num/den` has a step `12 * den / num` ticks long. Until this module
//! existed the engine kept the phase as an `f32` count of ticks and compared it with an `f32` step
//! length, patched with `+ 1e-6`. Most step lengths are not exactly representable, so the phase is
//! a little wrong after every step and the error builds up: 49 of the 159 reduced multipliers up
//! to 16 fired a step one tick late within 20,000 ticks (`handoffs/evidence/o4-4a-red.txt`).
//!
//! Here the phase is a **whole number of `1/num`-tick units**. Every tick adds `num` and a step
//! starts when the phase reaches `12 * den`, which is then subtracted. Both are integers, so the
//! k-th step starts on tick `ceil(k * 12 * den / num)` for every `u8` pair, at any tick count, and
//! the phase never leaves `0 .. 12 * den + num`, so nothing grows with time.
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

/// One track's position within its current step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepClock {
    /// Ticks into the current step, in units of `1/unit` tick.
    phase: u32,
    unit: u32,
}

impl Default for StepClock {
    fn default() -> Self {
        StepClock::new()
    }
}

impl StepClock {
    pub const fn new() -> StepClock {
        StepClock { phase: 0, unit: 1 }
    }

    /// A clock that is `phase` units of `1/unit` tick into its step. Used by tests to start a
    /// track as if it had already played for a very long time.
    pub const fn at(phase: u32, unit: u32) -> StepClock {
        StepClock { phase, unit }
    }

    /// The phase and its unit: `phase` units of `1/unit` tick. For tests and diagnostics.
    pub const fn phase(&self) -> (u32, u32) {
        (self.phase, self.unit)
    }

    /// Keeps the same point in the step, in ticks, rounded down to a whole unit of `1/num` tick.
    /// That loses less than one unit, and never gains. 64-bit: the phase is under `den + num`
    /// and `num` is at most 255 for a multiplier, so a `u32` would do, but a product that cannot
    /// overflow is easier to trust than one that does not.
    fn rescale(&mut self, num: u32) {
        self.phase = (self.phase as u64 * num as u64 / self.unit as u64) as u32;
        self.unit = num;
    }

    /// One tick. Returns true if a step starts on it.
    pub fn tick(&mut self, rate: StepRate) -> bool {
        if rate.num == 0 {
            return false;
        }
        if rate.num != self.unit {
            self.rescale(rate.num);
        }
        if rate.every_tick() {
            // A step of a tick or less. The phase is carried through untouched, as the `f32`
            // accumulator's `+ 1 - 1` did, so leaving this state resumes where it left off.
            return true;
        }
        self.phase += rate.num;
        if self.phase >= rate.den {
            self.phase -= rate.den;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STEP: u32 = 12;

    fn rate(num: u8, den: u8) -> StepRate {
        StepRate::from_multiplier(num, den, STEP)
    }

    fn gcd(a: u32, b: u32) -> u32 {
        if b == 0 {
            a
        } else {
            gcd(b, a % b)
        }
    }

    /// The exact tick (counted from 1) of the k-th step at multiplier `num/den`.
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
                let period = STEP * den / gcd(STEP * den, num);
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

    /// R1b. The invariant that makes the arithmetic exact at any tick count: after n ticks the
    /// phase is `n * num mod (12 * den)`, and it stays under `12 * den`. Nothing grows with time,
    /// so nothing can drift. Checked over three periods for every pair whose step is longer than
    /// a tick.
    #[test]
    fn r1b_the_phase_is_bounded_and_equals_n_times_num_mod_the_step() {
        for den in 1..=255u32 {
            for num in (1..=255u32).filter(|&n| n < STEP * den) {
                let r = rate(num as u8, den as u8);
                let step = STEP * den;
                let mut clock = StepClock::new();
                for n in 1..=3 * (step / gcd(step, num)) as u64 {
                    clock.tick(r);
                    let (phase, unit) = clock.phase();
                    assert_eq!(unit, num);
                    assert_eq!(phase as u64, n * num as u64 % step as u64, "x{num}/{den} after {n} ticks");
                }
            }
        }
    }

    /// R1c. Far from tick 0. A clock started at the phase it would have after N ticks steps
    /// exactly like the closed form from there, for N up to 10^12 and 2^40. This is what "to
    /// 10^12 ticks" means: the state is a bounded integer, so a clock that has played for a
    /// thousand years is no different from a fresh one at the same phase.
    #[test]
    fn r1c_a_clock_far_from_tick_zero_still_fires_on_the_exact_tick() {
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
    /// (43 minutes at 120 BPM: 384 ticks a second, and the baseline's 30 minutes is 691,200):
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
                if gcd(num, den) != 1 {
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

    // ---- R1e: a multiplier change carries the phase

    /// R1e. Changing the multiplier keeps the point in the step, in ticks, rounded down to a
    /// whole unit of the new rate: it loses less than one unit (`1/new_num` of a tick), and never
    /// gains. Checked for every pair of rates from a set that includes the extremes, at every
    /// phase of the old step.
    #[test]
    fn r1e_a_multiplier_change_loses_less_than_one_unit_of_phase() {
        let rates: [(u8, u8); 14] = [(1, 1), (1, 2), (2, 1), (3, 2), (2, 3), (5, 7), (7, 5), (16, 1), (1, 16), (13, 16), (255, 254), (254, 255), (255, 1), (1, 255)];
        for (n0, d0) in rates {
            for (n1, _) in rates {
                for phase in 0..STEP * d0 as u32 {
                    let mut clock = StepClock::at(phase, n0 as u32);
                    clock.rescale(n1 as u32);
                    let (new_phase, unit) = clock.phase();
                    assert_eq!(unit, n1 as u32);
                    // phase/n0 ticks, in units of 1/n1: new_phase must be that, rounded down.
                    let exact_units = phase as f64 * n1 as f64 / n0 as f64;
                    let lost = exact_units - new_phase as f64;
                    assert!((0.0..1.0).contains(&lost), "x{n0}/{d0} to x{n1}, phase {phase}: {exact_units} became {new_phase}");
                }
            }
        }
    }

    /// The hand case behind R1e, and the behaviour of the `f32` code it replaces: five ticks into
    /// a ×1 step (12 ticks), the multiplier becomes ×3 (4 ticks a step). Five ticks of phase is
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

    /// A zero denominator reads as ×1 (as `Track::multiplier` did), a zero numerator never fires.
    #[test]
    fn zero_denominator_is_times_one_and_zero_numerator_never_fires() {
        let mut a = StepClock::new();
        let mut b = StepClock::new();
        for _ in 0..1000 {
            assert_eq!(a.tick(rate(7, 0)), b.tick(rate(1, 1)));
        }
        let mut z = StepClock::new();
        assert!((0..100_000).all(|_| !z.tick(rate(0, 5))));
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
}

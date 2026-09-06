//! Deterministic PRNG for everything stochastic in the core (shuffle range picks,
//! directions 4/5, chord polyphony sampling).
//!
//! v1 (`archive/v1-max4live/octopus_engine.js`) used `Math.random()`, which made the
//! Max for Live engine non-deterministic run to run. SPEC.md D1 requires the core be
//! "deterministic under a seed", so every stochastic decision in this crate draws
//! from this RNG rather than any host source of randomness. splitmix64 is used for
//! its simplicity and quality, not for cryptographic properties.

#[derive(Clone, Copy, Debug)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        // Avoid the all-zero fixed point.
        Self { state: seed ^ 0x9E3779B97F4A7C15 }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    /// Uniform integer in `[0, bound)`. `bound` must be > 0.
    pub fn next_below(&mut self, bound: u32) -> u32 {
        debug_assert!(bound > 0);
        (self.next_u64() % (bound as u64)) as u32
    }

    /// A float in `[0, 1)`.
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..64 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seed_diverges() {
        let mut a = Rng::new(1);
        let mut b = Rng::new(2);
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn next_below_stays_in_range() {
        let mut r = Rng::new(7);
        for _ in 0..1000 {
            let v = r.next_below(3);
            assert!(v < 3);
        }
    }
}

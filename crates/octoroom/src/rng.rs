//! Deterministic PRNG, splitmix64. Duplicated from `crates/octocore/src/rng.rs`
//! rather than depended on — `docs/02-architecture.md`'s crate diagram has no
//! edge between `octoroom` and `octocore` (they share a repo, not a dependency:
//! `octoroom` takes a room description in, and produces a probe/IR/grade out; it
//! has no notion of tracks, steps, or sequencing). Geometry synthesis needs
//! "same intent plus same seed gives the same room, always"
//! (docs/06-shell-and-rooms.md §4) — this is that seed source.

#[derive(Clone, Copy, Debug)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed ^ 0x9E3779B97F4A7C15 }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    /// Uniform float in `[0, 1)`.
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    /// Uniform float in `[lo, hi)`.
    pub fn range_f32(&mut self, lo: f32, hi: f32) -> f32 {
        lo + self.next_f32() * (hi - lo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..32 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn range_f32_stays_in_bounds() {
        let mut r = Rng::new(7);
        for _ in 0..1000 {
            let v = r.range_f32(2.0, 5.0);
            assert!((2.0..5.0).contains(&v));
        }
    }
}

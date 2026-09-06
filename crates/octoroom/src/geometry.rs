//! Ref: docs/06-shell-and-rooms.md §3 (the room contract) and §4 pipeline step 2
//! ("geometry synthesis... procedural and seeded. Same intent plus same seed
//! gives the same room, always."). This module is the pure-CPU half of that
//! step: a real, deterministic geometry generator. It does not attempt the
//! optical bake (path-traced cubemap) or the acoustic bake (stochastic ray
//! tracing) — those are Metal compute (§4, steps 3a/3b) and this dev
//! environment has no Metal toolchain. See the crate README for the full list
//! of what's out of scope.

use crate::intent::{MaterialHint, RoomIntent, RoomScale};
use crate::rng::Rng;

/// Ref docs/06 §3: "Absorption coefficients are per octave band... Six bands
/// is enough for convincing reverb and cheap enough to trace." The contract's
/// own example keys are `125/250/500/1k/2k/4k`; named fields here rather than
/// a bare `[f32; 6]` so a reader doesn't have to remember index-to-band order.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Bands {
    pub hz125: f32,
    pub hz250: f32,
    pub hz500: f32,
    pub hz1k: f32,
    pub hz2k: f32,
    pub hz4k: f32,
}

impl Bands {
    pub fn new(hz125: f32, hz250: f32, hz500: f32, hz1k: f32, hz2k: f32, hz4k: f32) -> Self {
        Bands { hz125, hz250, hz500, hz1k, hz2k, hz4k }
    }

    pub fn as_array(&self) -> [f32; 6] {
        [self.hz125, self.hz250, self.hz500, self.hz1k, self.hz2k, self.hz4k]
    }
}

/// One surface in the room, matching docs/06 §3's `surfaces[]` entries
/// exactly: `material`, `area_m2`, per-band `absorption`, `scattering`,
/// `albedo`, `roughness`.
#[derive(Clone, Copy, Debug)]
pub struct Surface {
    pub material: MaterialHint,
    pub area_m2: f32,
    pub absorption: Bands,
    /// Ref docs/06 §3: "`scattering` is separate from `roughness`... measured
    /// at wavelengths four orders of magnitude apart... do not tie them."
    pub scattering: f32,
    pub albedo: [f32; 3],
    pub roughness: f32,
}

#[derive(Clone, Debug)]
pub struct RoomGeometry {
    pub bounds_m: [f32; 3],
    pub volume_m3: f32,
    pub surfaces: Vec<Surface>,
}

impl RoomGeometry {
    pub fn total_surface_area_m2(&self) -> f32 {
        self.surfaces.iter().map(|s| s.area_m2).sum()
    }

    /// Area-weighted mean absorption per band, over every surface — the input
    /// the Eyring/Sabine RT60 prediction (`crate::acoustics`) needs.
    pub fn mean_absorption(&self) -> Bands {
        let total_area = self.total_surface_area_m2();
        if total_area <= 0.0 {
            return Bands::default();
        }
        let mut acc = [0f32; 6];
        for s in &self.surfaces {
            for (i, a) in s.absorption.as_array().iter().enumerate() {
                acc[i] += a * s.area_m2;
            }
        }
        for v in acc.iter_mut() {
            *v /= total_area;
        }
        Bands::new(acc[0], acc[1], acc[2], acc[3], acc[4], acc[5])
    }
}

/// Ref docs/06 §3's material table entry for `concrete_bare` (verbatim):
/// `absorption: {125:0.02, 250:0.03, 500:0.03, 1k:0.04, 2k:0.05, 4k:0.07}`,
/// `scattering: 0.12`, `albedo: [0.34,0.33,0.31]`, `roughness: 0.82`. The
/// other materials below are typical published architectural-acoustics
/// absorption coefficients (approximate, not from a specific cited source —
/// unlike `crates/octocore`'s manual citations, there is no single reference
/// document for these; they're standard textbook orders of magnitude) rather
/// than measurements, since there is no calibrated material data in this
/// project yet (see `reference/NOTES.md`).
fn material_properties(material: MaterialHint) -> (Bands, f32, [f32; 3], f32) {
    match material {
        MaterialHint::ConcreteBare => (Bands::new(0.02, 0.03, 0.03, 0.04, 0.05, 0.07), 0.12, [0.34, 0.33, 0.31], 0.82),
        MaterialHint::Stone => (Bands::new(0.02, 0.02, 0.03, 0.04, 0.05, 0.05), 0.10, [0.4, 0.38, 0.35], 0.75),
        MaterialHint::WoodDry => (Bands::new(0.15, 0.11, 0.10, 0.07, 0.06, 0.07), 0.08, [0.45, 0.32, 0.2], 0.35),
        MaterialHint::CarpetedStudio => (Bands::new(0.08, 0.24, 0.57, 0.69, 0.71, 0.73), 0.35, [0.2, 0.18, 0.16], 0.9),
        MaterialHint::Glass => (Bands::new(0.18, 0.06, 0.04, 0.03, 0.02, 0.02), 0.02, [0.15, 0.18, 0.2], 0.05),
        MaterialHint::Cardboard => (Bands::new(0.15, 0.2, 0.3, 0.35, 0.4, 0.4), 0.4, [0.55, 0.45, 0.35], 0.6),
    }
}

fn scale_dimension_range_m(scale: RoomScale) -> (f32, f32) {
    match scale {
        RoomScale::Intimate => (1.5, 2.5),
        RoomScale::Small => (2.5, 4.0),
        RoomScale::Medium => (4.0, 7.0),
        RoomScale::Large => (7.0, 14.0),
        RoomScale::Cavernous => (14.0, 30.0),
    }
}

/// Ref docs/06 §4: "geometry synthesis... procedural and seeded. Same intent
/// plus same seed gives the same room, always." Any axis given explicitly in
/// `intent.dimensions_m` is used as-is (deterministic without even touching
/// the RNG); axes left `None` are drawn from `intent.scale`'s range using
/// `seed`, so two different seeds can still give two different rooms for the
/// same coarse-scale intent, while the same seed always reproduces the same
/// one. Surfaces are a simplified floor/ceiling/walls decomposition — real
/// wall segmentation (windows, the handrail in the stairwell prompt, etc.) is
/// out of scope here; see the crate README.
pub fn synthesize_geometry(intent: &RoomIntent, seed: u64) -> RoomGeometry {
    let mut rng = Rng::new(seed);
    let (lo, hi) = scale_dimension_range_m(intent.scale);
    let bounds_m = [
        intent.dimensions_m[0].unwrap_or_else(|| rng.range_f32(lo, hi)),
        intent.dimensions_m[1].unwrap_or_else(|| rng.range_f32(lo, hi)),
        intent.dimensions_m[2].unwrap_or_else(|| rng.range_f32(lo, hi)),
    ];
    let [l, h, d] = bounds_m;
    let volume_m3 = l * h * d;

    let (absorption, scattering, albedo, roughness) = material_properties(intent.primary_material);
    let floor_ceiling_area = l * d; // one of each
    let wall_area = 2.0 * (l * h) + 2.0 * (d * h); // four walls combined

    let surfaces = vec![
        Surface { material: intent.primary_material, area_m2: wall_area, absorption, scattering, albedo, roughness },
        Surface { material: intent.primary_material, area_m2: floor_ceiling_area, absorption, scattering, albedo, roughness },
        Surface { material: intent.primary_material, area_m2: floor_ceiling_area, absorption, scattering, albedo, roughness },
    ];

    RoomGeometry { bounds_m, volume_m3, surfaces }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stairwell_3am_matches_the_contracts_own_numbers() {
        let intent = RoomIntent::stairwell_3am();
        let geo = synthesize_geometry(&intent, 1);
        // docs/06 §3: bounds_m [2.1, 11.4, 3.0], volume_m3 71.8. All three
        // axes are explicit in the hand-authored intent, so this doesn't even
        // touch the RNG — it's a pure arithmetic check.
        assert_eq!(geo.bounds_m, [2.1, 11.4, 3.0]);
        assert!((geo.volume_m3 - 71.8).abs() < 0.05, "expected ~71.8 m^3, got {}", geo.volume_m3);
    }

    #[test]
    fn same_seed_gives_identical_geometry() {
        let mut intent = RoomIntent::stairwell_3am();
        intent.dimensions_m = [None, None, None]; // force the RNG path
        let a = synthesize_geometry(&intent, 42);
        let b = synthesize_geometry(&intent, 42);
        assert_eq!(a.bounds_m, b.bounds_m);
        assert_eq!(a.volume_m3, b.volume_m3);
    }

    #[test]
    fn different_seeds_can_differ() {
        let mut intent = RoomIntent::stairwell_3am();
        intent.dimensions_m = [None, None, None];
        let a = synthesize_geometry(&intent, 1);
        let b = synthesize_geometry(&intent, 2);
        assert_ne!(a.bounds_m, b.bounds_m, "different seeds landing on identical dimensions would be a suspicious coincidence");
    }

    #[test]
    fn mean_absorption_matches_uniform_material_input() {
        // Every surface here shares one material, so the area-weighted mean
        // must equal that material's own per-band coefficients exactly.
        let intent = RoomIntent::stairwell_3am();
        let geo = synthesize_geometry(&intent, 1);
        let (expected, _, _, _) = material_properties(MaterialHint::ConcreteBare);
        let mean = geo.mean_absorption();
        for (got, want) in mean.as_array().iter().zip(expected.as_array().iter()) {
            assert!((got - want).abs() < 1e-4, "got {got}, want {want}");
        }
    }
}

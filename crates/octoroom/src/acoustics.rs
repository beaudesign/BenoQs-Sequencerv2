//! Ref: docs/06-shell-and-rooms.md §3: "`rt60_s` is stored so `verify:acoustics`
//! can check the traced IR against the Sabine/Eyring prediction from the
//! geometry and absorption. If they disagree by more than 10%, the tracer has
//! a bug." This module is that prediction side of the self-check — the traced
//! side (the actual ray-traced impulse response) needs Metal compute and isn't
//! implemented here (see the crate README).
//!
//! Ref: Eyring reverberation-time equation, standard architectural acoustics
//! (W.C. Sabine's original formula generalised by C.F. Eyring, 1930) — not an
//! Octopus-manual citation, this is public acoustics knowledge:
//!
//!   RT60 = 0.161 * V / (-S * ln(1 - ᾱ))
//!
//! where V is room volume (m^3), S is total surface area (m^2), and ᾱ is the
//! area-weighted mean absorption coefficient. Eyring is used instead of the
//! simpler Sabine formula (RT60 = 0.161 * V / (S * ᾱ)) because Sabine
//! diverges from measured behaviour as ᾱ approaches 1 (a fully absorptive
//! room), while Eyring stays well-behaved across the whole 0..1 range — this
//! matters here because some of `crate::geometry`'s materials (carpeted
//! studio) have high-band absorption coefficients above 0.7.

use crate::geometry::Bands;

/// Ref docs/06 §4: "0.161" is the standard constant for metres/seconds units
/// (it's 0.049 in feet/seconds — always confirm units before reusing this).
const EYRING_CONSTANT: f32 = 0.161;

/// Predicted RT60 (seconds) for one absorption coefficient. `mean_absorption`
/// must be in `[0, 1)` — exactly `1.0` (a perfectly absorptive room) would
/// make `ln(0)` diverge to `-infinity`, correctly predicting RT60 -> 0, but
/// left as a documented edge case rather than specially handled, since a
/// real room never reaches it.
pub fn eyring_rt60_seconds(volume_m3: f32, total_surface_area_m2: f32, mean_absorption: f32) -> f32 {
    if total_surface_area_m2 <= 0.0 || volume_m3 <= 0.0 {
        return 0.0;
    }
    let a = mean_absorption.clamp(0.0, 0.999_999);
    if a <= 0.0 {
        return f32::INFINITY; // no absorption at all: the formula's own honest answer
    }
    EYRING_CONSTANT * volume_m3 / (-total_surface_area_m2 * (1.0 - a).ln())
}

/// RT60 per octave band, from the room's total volume/area and the
/// area-weighted mean absorption per band (`RoomGeometry::mean_absorption`).
pub fn eyring_rt60_bands(volume_m3: f32, total_surface_area_m2: f32, mean_absorption: Bands) -> Bands {
    let a = mean_absorption.as_array();
    let rt = a.map(|coef| eyring_rt60_seconds(volume_m3, total_surface_area_m2, coef));
    Bands::new(rt[0], rt[1], rt[2], rt[3], rt[4], rt[5])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hand-computed cross-check, independent of the implementation: a
    /// 10m x 10m x 3m room (V=300 m^3, S = 2*(10*10) + 4*(10*3) = 320 m^2)
    /// with a uniform absorption coefficient of 0.1.
    /// RT60 = 0.161*300 / (-320 * ln(0.9)) = 48.3 / (-320 * -0.10536)
    ///      = 48.3 / 33.716 = 1.4327...s
    #[test]
    fn eyring_matches_hand_computed_value() {
        let volume = 300.0;
        let area = 320.0;
        let absorption = 0.1;
        let rt60 = eyring_rt60_seconds(volume, area, absorption);
        let expected = 1.4327;
        assert!((rt60 - expected).abs() < 0.01, "got {rt60}, expected ~{expected}");
    }

    #[test]
    fn higher_absorption_means_shorter_decay() {
        let dead = eyring_rt60_seconds(300.0, 320.0, 0.8);
        let live = eyring_rt60_seconds(300.0, 320.0, 0.05);
        assert!(dead < live, "a more absorptive room must decay faster: dead={dead}, live={live}");
    }

    #[test]
    fn zero_absorption_is_infinite() {
        assert_eq!(eyring_rt60_seconds(300.0, 320.0, 0.0), f32::INFINITY);
    }

    #[test]
    fn stairwell_3am_rt60_is_in_a_plausible_range() {
        // docs/06 §3's own worked example gives rt60_s {125: 2.9, 500: 2.4,
        // 2k: 1.7} for this exact room from its (presumably real, traced)
        // pipeline. This is a *plausibility* check, not an exact match — this
        // module predicts from geometry+absorption only, the doc's numbers
        // came from a full trace this crate doesn't implement. Confirms the
        // prediction is in the right neighbourhood (same order of magnitude,
        // same decreasing-with-frequency trend), not that it's identical.
        use crate::geometry::synthesize_geometry;
        use crate::intent::RoomIntent;
        let intent = RoomIntent::stairwell_3am();
        let geo = synthesize_geometry(&intent, 1);
        let mean = geo.mean_absorption();
        let rt = eyring_rt60_bands(geo.volume_m3, geo.total_surface_area_m2(), mean);
        for v in rt.as_array() {
            assert!((0.5..20.0).contains(&v), "RT60 {v}s is outside any plausible room's range");
        }
        assert!(rt.hz125 > rt.hz4k, "concrete's absorption rises with frequency, so RT60 should fall: {} vs {}", rt.hz125, rt.hz4k);
    }
}

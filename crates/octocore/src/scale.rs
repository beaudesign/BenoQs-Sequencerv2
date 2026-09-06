//! Scale-force quantisation. Ported from `archive/v1-max4live/octopus_scale.js`.
//!
//! Quantisation happens after all pitch offsets are summed (docs/03-sequencer-core.md
//! §3, "Scales"), so callers must sum track + page + step pitch before calling
//! `quantize_to_scale`.

/// Bitmask over the 12 pitch classes (bit `n` set means pitch class `n` is in the
/// scale).
pub type PitchClassSet = u16;

pub fn normalize_pitch_class(pc: i32) -> u8 {
    (((pc % 12) + 12) % 12) as u8
}

pub fn build_scale_pitch_classes(root: i32, intervals: &[i32]) -> PitchClassSet {
    let root_pc = normalize_pitch_class(root);
    let mut set: PitchClassSet = 1 << root_pc;
    for &interval in intervals {
        set |= 1 << normalize_pitch_class(root_pc as i32 + interval);
    }
    set
}

/// The seven-note major-scale intervals, used as the default when a scale has no
/// explicit interval list (matches `octopus_data.js` `defaultGlobalScale`).
pub const DEFAULT_INTERVALS: [i32; 7] = [0, 2, 4, 5, 7, 9, 11];

/// Quantise a MIDI pitch to the nearest pitch class in `scale`. Ties prefer downward
/// motion, matching v1's tie-break (chosen there for stability in live use).
pub fn quantize_to_scale(pitch: u8, scale: PitchClassSet) -> u8 {
    let pc = pitch % 12;
    if scale & (1 << pc) != 0 {
        return pitch;
    }
    for d in 1..=11i32 {
        let down = pitch as i32 - d;
        if down >= 0 && scale & (1 << normalize_pitch_class(down)) != 0 {
            return down as u8;
        }
        let up = pitch as i32 + d;
        if up <= 127 && scale & (1 << normalize_pitch_class(up)) != 0 {
            return up as u8;
        }
    }
    pitch
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_scale_pitch_is_unchanged() {
        let maj_c = build_scale_pitch_classes(60, &DEFAULT_INTERVALS);
        assert_eq!(quantize_to_scale(64, maj_c), 64); // E, in C major
    }

    #[test]
    fn out_of_scale_prefers_downward() {
        let maj_c = build_scale_pitch_classes(60, &DEFAULT_INTERVALS);
        // C# (61) is not in C major; nearest neighbours are C (60, down 1) and D (62,
        // up 1) — equal distance, downward must win.
        assert_eq!(quantize_to_scale(61, maj_c), 60);
    }

    #[test]
    fn chromatic_scale_never_moves_pitch() {
        let chromatic = build_scale_pitch_classes(0, &(0..12).collect::<Vec<_>>());
        for p in 0..128u8 {
            assert_eq!(quantize_to_scale(p, chromatic), p);
        }
    }
}

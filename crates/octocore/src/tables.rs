//! Manual-derived timing tables.
//!
//! `docs/03-sequencer-core.md` §1 requires every non-obvious constant to carry a
//! `Ref:` citation to a page of the CE v5.30 reference manual, and §4 names exactly
//! these two tables as "known to need re-derivation": the JS randomises even GRV
//! settings over a window whose exact bounds were eyeballed, and the strum table's
//! 54 entries were transcribed once and never cross-checked.
//!
//! This repo does not yet contain the manual (see `reference/NOTES.md`), so these
//! tables are ported verbatim from `archive/v1-max4live/octopus_engine.js`
//! (`_grvDelayTicks`, `_strumOffsetTicks`) rather than cited against a page number.
//! **They are provisional.** Treat every value here as an open item in
//! `tests/conformance/AMBIGUITIES.md` until someone with the manual (or the
//! hardware) confirms it. Do not add a new constant to this file without either a
//! real `Ref:` citation or the same provisional flag.

use crate::rng::Rng;

/// Track shuffle delay in ticks, applied to even-numbered step positions (2, 4, ...,
/// 16; zero-based indices 1, 3, ..., 15). `grv` is the track's GRV attribute, 0..=16.
///
/// PROVISIONAL — ported from `_grvDelayTicks` in
/// `archive/v1-max4live/octopus_engine.js`, not yet verified against the manual.
/// Odd settings are a fixed delay; even settings are randomised within a
/// three-tick window centered progressively later. `rng` is drawn from only when
/// `grv` is one of the randomised (even) settings, so odd settings stay exactly
/// reproducible even without a seed being threaded through.
pub fn grv_delay_ticks(grv: u8, rng: &mut Rng) -> u32 {
    let g = grv.min(16) as u32;
    match g {
        0 => 0,
        1 => 1,
        2 => rng.next_below(3),           // 0..=2
        3 => 2,
        4 => 1 + rng.next_below(3),        // 1..=3
        5 => 3,
        6 => 2 + rng.next_below(3),        // 2..=4
        7 => 4,
        8 => 3 + rng.next_below(3),        // 3..=5
        9 => 5,
        10 => 4 + rng.next_below(3),       // 4..=6
        11 => 6,
        12 => 5 + rng.next_below(3),       // 5..=7
        13 => 7,
        14 => 6 + rng.next_below(3),       // 6..=8
        15 => 8,
        16 => 7 + rng.next_below(3),       // 7..=9
        _ => 0,
    }
}

/// Chord strum timing, in ticks, for the note at 1-based ordinal `note_number`
/// (1 = the first note struck, never delayed) within a chord struck at strum
/// level `level` (0..=9, magnitude only — direction is handled by the caller
/// reversing the note order before calling this).
///
/// PROVISIONAL — ported from `_strumOffsetTicks` in
/// `archive/v1-max4live/octopus_engine.js` ("Chord Strum Timings in Ticks" table,
/// strum 1..=9 x notes 2..=7 = 54 entries), not yet verified against the manual.
pub fn strum_offset_ticks(level: u8, note_number: u8) -> u32 {
    if note_number <= 1 {
        return 0;
    }
    let lvl = level.min(9);
    if lvl == 0 {
        return 0;
    }
    let row: &[u32; 9] = match note_number {
        2 => &[0, 1, 1, 2, 2, 3, 3, 4, 5],
        3 => &[1, 2, 3, 4, 5, 6, 7, 8, 10],
        4 => &[1, 2, 4, 6, 8, 9, 10, 13, 17],
        5 => &[2, 3, 5, 9, 11, 13, 15, 19, 23],
        6 => &[2, 3, 6, 12, 15, 18, 21, 27, 30],
        7 => &[3, 6, 9, 16, 19, 24, 29, 36, 45],
        _ => return 0,
    };
    row[(lvl - 1) as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn odd_grv_settings_are_fixed() {
        let mut rng = Rng::new(1);
        assert_eq!(grv_delay_ticks(1, &mut rng), 1);
        assert_eq!(grv_delay_ticks(3, &mut rng), 2);
        assert_eq!(grv_delay_ticks(15, &mut rng), 8);
    }

    #[test]
    fn even_grv_settings_stay_in_window() {
        let mut rng = Rng::new(99);
        for _ in 0..200 {
            let v = grv_delay_ticks(10, &mut rng);
            assert!((4..=6).contains(&v));
        }
    }

    #[test]
    fn strum_table_first_note_never_delayed() {
        assert_eq!(strum_offset_ticks(9, 1), 0);
    }

    #[test]
    fn strum_table_known_entries() {
        // Level 9, note 7 -> last entry of the note-7 row.
        assert_eq!(strum_offset_ticks(9, 7), 45);
        // Level 1, note 2 -> first entry of the note-2 row.
        assert_eq!(strum_offset_ticks(1, 2), 0);
    }
}

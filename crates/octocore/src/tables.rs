//! Manual-derived timing tables.
//!
//! `docs/03-sequencer-core.md` §1 requires every non-obvious constant to carry a
//! `Ref:` citation to a page of the CE v5.30 reference manual, and §4 flagged these
//! two tables as "known to need re-derivation" back when they were only ported from
//! `archive/v1-max4live/octopus_engine.js` sight-unseen against the real manual.
//!
//! The manual has since landed (`reference/manual/`, `just manual <topic>`) and
//! both tables check out **cell-for-cell identical** to what v1 already had — see
//! `Ref:` citations on each function below. Do not add a new constant to this file
//! without either a real `Ref:` citation or an explicit PROVISIONAL flag plus a
//! matching entry in `tests/conformance/AMBIGUITIES.md`.

use crate::rng::Rng;

/// Track shuffle delay in ticks, applied to even-numbered step positions (2, 4, ...,
/// 16; zero-based indices 1, 3, ..., 15). `grv` is the track's GRV attribute, 0..=16.
///
/// Ref: CE v5.30 §3 Track Mode, "Track groove (GRV)", p.46:
/// "Shuffle means that the steps with an even index in the track (i.e. 2, 4, 6
/// .... 16) will be played with a delay... the odd GRV values will produce steady
/// shuffle delays, while the even GRV values will produce delays that are variable
/// within one 1/192 and which are determined at runtime":
/// ```text
/// Setting  Delay(1/192)   Setting  Delay(1/192)
///    1          1            9          5
///    2         0-2           10        4-6
///    3          2            11         6
///    4         1-3           12        5-7
///    5          3            13         7
///    6         2-4           14        6-8
///    7          4            15         8
///    8         3-5           16        7-9
/// ```
/// Cell-for-cell identical to what `archive/v1-max4live/octopus_engine.js`'s
/// `_grvDelayTicks` already had (setting 0, absent from the printed table, is
/// treated as no shuffle, which is the only reading consistent with "GRV
/// determines *how much* shuffle is applied" starting from a range of 0-16).
/// `rng` is drawn from only when `grv` is one of the randomised (even) settings,
/// so odd settings stay exactly reproducible even without a seed being threaded
/// through.
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
/// Ref: CE v5.30 §2 Step Mode, "Strumming chords", p.22, table "Chord Strum
/// Timings in Ticks" (strum 1..=9 x notes 2..=7, 54 entries) — cell-for-cell
/// identical to what `archive/v1-max4live/octopus_engine.js`'s
/// `_strumOffsetTicks` already had. Also confirms strum level is really -9..=9
/// (sign = up/down direction, magnitude indexes this table — already how the
/// engine treats it) and that strumming a single-note step plays that note plus
/// 6 duplicates at the strum timing (already how `engine.rs::fire_step` treats
/// `note_count == 1`).
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

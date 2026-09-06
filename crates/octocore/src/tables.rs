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

/// Ref: CE v5.30 p.44, "Track LEN Reference Chart". Rows are the Track LEN
/// factor 0..=16 (8 = neutral, unchanged — "Track LEN 8 Neutral"); columns
/// are the chart's 12 sampled baseline ("Step LEN", i.e. factor-8) tick
/// values. Transcribed directly from `reference/manual/pages/p044.txt`, not
/// derived from a formula: rows 8/14/16 cleanly match "floor(baseline ×
/// 1.0/1.5/2.0), clamped to 192" (confirmed by hand), but row 9 does not
/// extrapolate cleanly from that pattern, so the chart's own printed numbers
/// are used verbatim rather than a formula that fits some rows and not
/// others.
const LEN_BREAKPOINTS: [i32; 12] = [3, 6, 9, 12, 24, 48, 72, 96, 120, 144, 168, 192];
#[rustfmt::skip]
const LEN_TABLE: [[i32; 12]; 17] = [
    /* 0  */ [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    /* 1  */ [1, 1, 1, 1, 1, 4, 4, 6, 7, 9, 10, 12],
    /* 2  */ [1, 1, 2, 3, 6, 12, 18, 24, 30, 36, 42, 48],
    /* 3  */ [1, 3, 4, 6, 12, 24, 36, 48, 60, 72, 84, 96],
    /* 4  */ [1, 3, 5, 7, 15, 30, 45, 60, 76, 90, 104, 120],
    /* 5  */ [2, 4, 6, 9, 18, 36, 54, 72, 90, 108, 126, 144],
    /* 6  */ [2, 5, 7, 10, 21, 42, 63, 84, 105, 126, 148, 168],
    /* 7  */ [2, 5, 8, 11, 22, 46, 68, 90, 112, 134, 156, 180],
    /* 8  */ [3, 6, 9, 12, 24, 48, 72, 96, 120, 144, 168, 192], // neutral
    /* 9  */ [3, 6, 9, 12, 25, 52, 76, 102, 126, 152, 178, 192],
    /* 10 */ [3, 6, 10, 13, 27, 54, 82, 108, 136, 162, 188, 192],
    /* 11 */ [3, 7, 10, 14, 28, 57, 85, 114, 142, 171, 192, 192],
    /* 12 */ [3, 7, 11, 15, 30, 60, 90, 120, 150, 180, 192, 192],
    /* 13 */ [3, 8, 12, 16, 33, 66, 100, 132, 166, 192, 192, 192],
    /* 14 */ [4, 9, 13, 18, 36, 72, 108, 144, 180, 192, 192, 192],
    /* 15 */ [5, 10, 15, 21, 42, 84, 126, 168, 192, 192, 192, 192],
    /* 16 */ [6, 12, 18, 24, 48, 96, 144, 192, 192, 192, 192, 192],
];

/// Ref: CE v5.30 p.44 scales a step's raw length (in ticks, 1..=192 — not
/// limited to the chart's 12 sampled breakpoints, since it comes from
/// `base_len × len_multiplier`) by the track's LEN factor. Piecewise-linear
/// interpolation between the chart's breakpoints, flat-clamped beyond the
/// first/last column: **chosen**, not manual-specified — the manual only
/// samples 12 points per row. See `tests/conformance/AMBIGUITIES.md`.
pub fn scale_len_ticks(track_len_factor: u8, raw_ticks: i32) -> i32 {
    let row = &LEN_TABLE[(track_len_factor as usize).min(16)];
    interpolate(&LEN_BREAKPOINTS, row, raw_ticks).clamp(1, 192)
}

/// Ref: CE v5.30 p.45, "Track STA Reference Chart". Rows are the Track STA
/// factor 0..=16 (8 = neutral); columns are Step STA offsets -5..=+5 — the
/// step's own max push/pull ("the maximum push is 5/192", p.16), so unlike
/// LEN this needs no interpolation: every real offset value is already one
/// of the 11 listed columns. Transcribed directly from
/// `reference/manual/pages/p045.txt`.
#[rustfmt::skip]
const STA_TABLE: [[i32; 11]; 17] = [
    /* 0  */ [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    /* 1  */ [-1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    /* 2  */ [-2, -1, -1, 0, 0, 0, 0, 0, 1, 1, 2],
    /* 3  */ [-2, -2, -1, -1, 0, 0, 0, 1, 1, 2, 2],
    /* 4  */ [-3, -2, -1, -1, 0, 0, 0, 1, 1, 2, 3],
    /* 5  */ [-3, -2, -2, -1, 0, 0, 0, 1, 2, 2, 3],
    /* 6  */ [-4, -3, -2, -1, 0, 0, 0, 1, 2, 3, 4],
    /* 7  */ [-4, -3, -2, -1, 0, 0, 0, 1, 2, 3, 4],
    /* 8  */ [-5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5], // neutral
    /* 9  */ [-5, -4, -3, -2, -1, 0, 1, 1, 3, 4, 5],
    /* 10 */ [-6, -4, -3, -2, -1, 0, 1, 2, 3, 4, 6],
    /* 11 */ [-7, -5, -4, -2, -1, 0, 1, 2, 4, 5, 7],
    /* 12 */ [-8, -6, -4, -3, -1, 0, 1, 3, 4, 6, 8],
    /* 13 */ [-9, -7, -5, -3, -1, 0, 1, 3, 5, 7, 9],
    /* 14 */ [-10, -8, -6, -4, -2, 0, 2, 4, 6, 8, 10], // "if Track STA 14 then Step STA multiplied by 2"
    /* 15 */ [-11, -9, -6, -4, -2, 0, 2, 4, 6, 9, 11],
    /* 16 */ [-12, -10, -7, -5, -2, 0, 2, 5, 7, 10, 12],
];

/// Exact lookup (no interpolation needed — see `STA_TABLE`'s doc comment).
/// `step_offset` outside -5..=5 is clamped to that range first, since no
/// larger offset is reachable through `Step::start_offset`'s own -5..=5
/// bound in practice, but this keeps the table index always in range.
pub fn scale_sta_ticks(track_sta_factor: u8, step_offset: i32) -> i32 {
    let row = &STA_TABLE[(track_sta_factor as usize).min(16)];
    let clamped = step_offset.clamp(-5, 5);
    row[(clamped + 5) as usize]
}

fn interpolate(breakpoints: &[i32], values: &[i32; 12], x: i32) -> i32 {
    if x <= breakpoints[0] {
        return values[0];
    }
    let last = breakpoints.len() - 1;
    if x >= breakpoints[last] {
        return values[last];
    }
    for i in 0..last {
        let (x0, x1) = (breakpoints[i], breakpoints[i + 1]);
        if x >= x0 && x <= x1 {
            let (y0, y1) = (values[i], values[i + 1]);
            if x1 == x0 {
                return y0;
            }
            return y0 + (y1 - y0) * (x - x0) / (x1 - x0);
        }
    }
    values[last]
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

    #[test]
    fn len_scaling_neutral_is_identity_at_breakpoints() {
        for &b in &LEN_BREAKPOINTS {
            assert_eq!(scale_len_ticks(8, b), b);
        }
    }

    #[test]
    fn len_scaling_matches_manual_chart_at_breakpoints() {
        // Ref p.44, row 16 ("x2"): exactly double the neutral row, clamped.
        assert_eq!(scale_len_ticks(16, 3), 6);
        assert_eq!(scale_len_ticks(16, 96), 192);
        assert_eq!(scale_len_ticks(16, 120), 192); // 240 clamped
        // Row 0: everything collapses to 1 tick.
        assert_eq!(scale_len_ticks(0, 192), 1);
        // Row 9 (does not extrapolate cleanly from a formula — this is why
        // the chart is transcribed verbatim rather than computed).
        assert_eq!(scale_len_ticks(9, 12), 12);
        assert_eq!(scale_len_ticks(9, 24), 25);
    }

    #[test]
    fn len_scaling_interpolates_between_breakpoints() {
        // Neutral row 8, between breakpoints 12 and 24: the manual doesn't
        // sample this value, so this is this crate's own chosen linear
        // interpolation, not a manual-cited number.
        let v = scale_len_ticks(8, 18); // halfway between 12 and 24
        assert_eq!(v, 18); // neutral row happens to be identity everywhere
        let below = scale_len_ticks(8, 1); // below the first breakpoint (3)
        assert_eq!(below, 3, "must clamp flat to the first breakpoint's value");
        let above = scale_len_ticks(8, 200); // above the last breakpoint (192)
        assert_eq!(above, 192, "must clamp flat to the last breakpoint's value");
    }

    #[test]
    fn sta_scaling_neutral_is_identity() {
        for offset in -5..=5 {
            assert_eq!(scale_sta_ticks(8, offset), offset);
        }
    }

    #[test]
    fn sta_scaling_matches_manual_note_row_14_doubles() {
        // Ref p.45: "If Track STA 14 then Step STA multiplied by 2."
        for offset in -5..=5 {
            assert_eq!(scale_sta_ticks(14, offset), offset * 2);
        }
    }

    #[test]
    fn sta_scaling_row_0_is_always_zero() {
        for offset in -5..=5 {
            assert_eq!(scale_sta_ticks(0, offset), 0);
        }
    }
}

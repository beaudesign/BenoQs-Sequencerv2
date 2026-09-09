//! Factory step-phrase pool. Ref: CE v5.30 p.16 (three banks of 16 = 48),
//! p.23 (types / default categories), p.24-26 (Green / Red / Orange charts).
//!
//! Transcribed from the printed charts via `pdftoppm` of
//! `reference/manual/CE-v5.30-reference-manual.pdf` (manual pp.24-26). Empty
//! cells are omitted; a note is enabled iff any of VEL/PIT/LEN/STA is
//! non-zero. Poly `x` on the Green/Red charts means full polyphony (8).
//!
//! Phrase POS remapping lives in `tables::scale_phrase_sta` (p.30), not here.

use crate::domain::{
    Phrase, PhraseNote, PhraseType, PHRASE_COUNT, PHRASE_NOTE_COUNT,
};

/// One programmed cell: (note_index 0=base .. 7=note 8, vel, pit, len, sta).
type Cell = (usize, i8, i8, u8, u8);

fn typ(t: u8) -> PhraseType {
    match t {
        2 => PhraseType::Reverse,
        3 => PhraseType::RandomPitch,
        4 => PhraseType::RandomAll,
        _ => PhraseType::Forward,
    }
}

fn assemble(type_raw: u8, poly: u8, cells: &[Cell]) -> Phrase {
    let mut phrase = Phrase {
        notes: [PhraseNote::default(); PHRASE_NOTE_COUNT],
        phrase_type: typ(type_raw),
        polyphony: poly,
    };
    for &(idx, vel, pit, len, sta) in cells {
        if idx >= PHRASE_NOTE_COUNT {
            continue;
        }
        phrase.notes[idx] = PhraseNote {
            velocity_offset: vel,
            pitch_offset: pit,
            length_ticks: len,
            start_ticks: sta,
            enabled: vel != 0 || pit != 0 || len != 0 || sta != 0,
        };
    }
    phrase
}

/// Ref: CE v5.30 p.24, "Step GRV Phrase Reference - Default (Green)".
/// Types: 1,3,1×14. Poly all `x` (full). Delays / rhythmic delays.
fn green_bank() -> [Phrase; 16] {
    [
        // 1: 1/8 echo
        assemble(1, 8, &[(1, -10, 0, 0, 24)]),
        // 2: type 3 (random pitch)
        assemble(3, 8, &[(1, -10, 0, 0, 24), (2, -20, 0, 0, 48)]),
        // 3
        assemble(1, 8, &[(1, -10, 0, 0, 24), (2, -20, 0, 0, 48), (3, -30, 0, 0, 72)]),
        // 4
        assemble(
            1,
            8,
            &[
                (1, -10, 0, 0, 12),
                (2, -20, 0, 0, 26),
                (3, -30, 0, 0, 42),
                (4, -40, 0, 0, 60),
                (5, 0, 0, 0, 80),
                (6, 0, 0, 0, 102),
            ],
        ),
        // 5
        assemble(
            1,
            8,
            &[
                (1, -10, 0, 0, 12),
                (2, -20, 0, 0, 24),
                (3, -30, 0, 0, 36),
                (4, -40, 0, 0, 48),
                (5, -50, 0, 0, 60),
                (6, 0, 0, 0, 72),
                (7, 0, 0, 0, 80),
            ],
        ),
        // 6: base LEN 26
        assemble(
            1,
            8,
            &[
                (0, 0, 0, 26, 0),
                (1, -39, 0, 0, 12),
                (2, -30, 0, 14, 36),
                (3, -20, 0, 0, 48),
            ],
        ),
        // 7
        assemble(
            1,
            8,
            &[
                (0, 0, 0, 26, 0),
                (1, -39, 0, 0, 16),
                (2, -30, 0, 14, 48),
                (3, -20, 0, 0, 72),
            ],
        ),
        // 8
        assemble(
            1,
            8,
            &[
                (0, 0, 0, 22, 0),
                (1, -39, 0, 0, 12),
                (2, -30, 0, 14, 36),
                (3, -17, 0, 0, 60),
                (4, -20, 0, 0, 80),
                (5, -40, 0, 0, 104),
            ],
        ),
        // 9: base VEL -30
        assemble(
            1,
            8,
            &[
                (0, -30, 0, 0, 0),
                (1, 0, 0, 0, 15),
                (2, -40, 0, 0, 72),
                (3, 0, 0, 0, 36),
            ],
        ),
        // 10
        assemble(
            1,
            8,
            &[
                (1, -10, 0, 0, 24),
                (2, -20, 0, 0, 36),
                (3, -30, 0, 0, 72),
                (4, -40, 0, 0, 84),
                (5, -50, 0, 0, 120),
            ],
        ),
        // 11
        assemble(
            1,
            8,
            &[
                (1, -10, 0, 0, 12),
                (2, -20, 0, 0, 48),
                (3, -30, 0, 0, 60),
                (4, -40, 0, 0, 96),
                (5, -50, 0, 0, 120),
            ],
        ),
        // 12
        assemble(
            1,
            8,
            &[
                (1, -10, 0, 0, 32),
                (2, -20, 0, 0, 48),
                (3, -30, 0, 0, 80),
                (4, -40, 0, 0, 96),
                (5, -50, 0, 0, 128),
            ],
        ),
        // 13
        assemble(
            1,
            8,
            &[
                (1, -10, 0, 0, 12),
                (2, -20, 0, 0, 24),
                (3, -30, 0, 0, 72),
                (4, -40, 0, 0, 84),
                (5, -50, 0, 0, 108),
            ],
        ),
        // 14
        assemble(
            1,
            8,
            &[
                (1, -10, 0, 0, 12),
                (2, -20, 0, 0, 24),
                (3, -30, 0, 0, 96),
                (4, -34, 0, 0, 108),
                (5, -40, 0, 0, 120),
            ],
        ),
        // 15
        assemble(
            1,
            8,
            &[
                (1, -10, 0, 0, 24),
                (2, -20, 0, 0, 60),
                (3, -30, 0, 0, 72),
                (4, -40, 0, 0, 96),
                (5, -50, 0, 0, 120),
            ],
        ),
        // 16
        assemble(
            1,
            8,
            &[
                (1, -10, 0, 0, 12),
                (2, -20, 0, 0, 24),
                (3, -30, 0, 0, 36),
                (4, -40, 0, 0, 84),
                (5, -50, 0, 0, 120),
            ],
        ),
    ]
}

/// Ref: CE v5.30 p.25, "Step GRV Phrase Reference - Default (Red)".
/// Pitched delays / arpeggios. Types 1,1,3,3,3,1,2,2,3,3,3,2,1,1,1,1.
fn red_bank() -> [Phrase; 16] {
    [
        assemble(
            1,
            8,
            &[
                (0, 0, 12, 0, 0),
                (1, -20, 0, 0, 24),
                (2, -40, 7, 0, 48),
                (3, -50, 12, 0, 72),
            ],
        ),
        assemble(
            1,
            8,
            &[
                (1, -20, 5, 0, 24),
                (2, -30, 7, 0, 48),
                (3, -40, 12, 0, 72),
                (4, -37, 7, 0, 96),
            ],
        ),
        assemble(
            3,
            8,
            &[
                (1, -20, 9, 0, 24),
                (2, -30, 12, 0, 96),
                (3, -40, 7, 0, 120),
            ],
        ),
        assemble(
            3,
            8,
            &[
                (1, -20, 2, 0, 12),
                (2, -30, 5, 0, 48),
                (3, -40, 7, 0, 60),
                (4, -36, 12, 0, 96),
                (5, -39, 0, 0, 108),
            ],
        ),
        assemble(
            3,
            8,
            &[
                (0, 0, 3, 0, 0),
                (1, -20, -2, 0, 24),
                (2, -30, 5, 0, 48),
                (3, -40, 7, 0, 72),
                (4, -27, 8, 0, 16),
            ],
        ),
        assemble(
            1,
            8,
            &[
                (1, -20, 12, 0, 36),
                (2, -40, 0, 0, 72),
                (3, -46, -12, 0, 108),
                (4, -27, 0, 0, 21),
                (5, -43, 24, 0, 75),
            ],
        ),
        assemble(
            2,
            8,
            &[
                (0, 0, 7, 0, 0),
                (1, -20, -7, 0, 12),
                (2, -40, 5, 0, 24),
                (3, -50, 12, 0, 36),
                (4, -27, 0, 0, 6),
                (5, -43, 0, 0, 24),
                (6, -49, 0, 0, 60),
            ],
        ),
        assemble(
            2,
            8,
            &[
                (0, 0, -7, 0, 0),
                (1, -30, 4, 0, 36),
                (2, -21, 12, 0, 37),
                (3, 0, 5, 0, 64),
                (4, -40, 0, 0, 80),
            ],
        ),
        assemble(
            3,
            8,
            &[
                (0, 0, 2, 0, 0),
                (1, -10, 5, 0, 24),
                (2, -36, 5, 0, 48),
                (3, -30, 7, 0, 72),
                (4, -40, 12, 0, 96),
            ],
        ),
        assemble(
            3,
            8,
            &[
                (0, 0, 12, 0, 0),
                (1, -20, 12, 0, 36),
                (2, -40, 3, 0, 72),
                (3, -36, 12, 0, 108),
                (4, -33, 12, 0, 48),
            ],
        ),
        assemble(
            3,
            8,
            &[
                (0, 0, 7, 0, 0),
                (1, -20, -7, 0, 24),
                (2, -40, 5, 0, 48),
                (3, -50, 12, 0, 72),
                (4, -27, 0, 0, 12),
                (5, -43, 0, 0, 48),
                (6, -49, 0, 0, 120),
            ],
        ),
        assemble(
            2,
            8,
            &[
                (0, 0, -7, 0, 0),
                (1, -30, 4, 0, 64),
                (2, -21, 12, 0, 78),
                (3, 0, 5, 0, 128),
                (4, -40, 5, 0, 160),
                (5, -32, 3, 0, 144),
            ],
        ),
        assemble(
            1,
            8,
            &[
                (1, -20, 12, 0, 12),
                (2, 0, 28, 0, 24),
                (3, 0, 12, 0, 36),
            ],
        ),
        assemble(
            1,
            8,
            &[
                (1, -20, 12, 0, 12),
                (2, 0, 24, 0, 24),
                (3, 0, 12, 0, 36),
                (4, 0, 19, 0, 0),
            ],
        ),
        assemble(
            1,
            8,
            &[
                (1, -20, 12, 0, 12),
                (2, 0, 24, 0, 24),
                (3, 0, 12, 0, 36),
                (4, 0, 19, 0, 48),
            ],
        ),
        assemble(
            1,
            8,
            &[
                (1, -20, 0, 0, 12),
                (2, 0, 24, 0, 24),
                (3, 0, 12, 0, 36),
                (4, 0, 20, 0, 48),
            ],
        ),
    ]
}

/// Ref: CE v5.30 p.26, "Step GRV Phrase Reference - Default (Orange)".
/// All type 4 (random all), polyphony 1. STA / VEL / combined wobbles.
fn orange_bank() -> [Phrase; 16] {
    [
        assemble(4, 1, &[(1, 0, 0, 0, 1)]),
        assemble(4, 1, &[(1, 0, 0, 0, 1), (2, 0, 0, 0, 2)]),
        assemble(4, 1, &[(1, 0, 0, 0, 1), (2, 0, 0, 0, 2), (3, 0, 0, 0, 3)]),
        assemble(
            4,
            1,
            &[
                (1, 0, 0, 0, 1),
                (2, 0, 0, 0, 2),
                (3, 0, 0, 0, 3),
                (4, 0, 0, 0, 4),
                (5, 0, 0, 0, 5),
                (6, 0, 0, 0, 7),
                (7, 0, 0, 0, 9),
            ],
        ),
        assemble(4, 1, &[(1, 10, 0, 0, 0), (2, -10, 0, 0, 0)]),
        assemble(
            4,
            1,
            &[(1, 10, 0, 0, 0), (2, -10, 0, 0, 0), (3, 20, 0, 0, 0), (4, -20, 0, 0, 0)],
        ),
        assemble(
            4,
            1,
            &[
                (1, 10, 0, 0, 0),
                (2, -10, 0, 0, 0),
                (3, 20, 0, 0, 0),
                (4, -20, 0, 0, 0),
                (5, 35, 0, 0, 0),
                (6, -35, 0, 0, 0),
            ],
        ),
        assemble(
            4,
            1,
            &[
                (1, 20, 0, 0, 0),
                (2, -20, 0, 0, 0),
                (3, 35, 0, 0, 0),
                (4, -35, 0, 0, 0),
                (5, 50, 0, 0, 0),
                (6, -50, 0, 0, 0),
            ],
        ),
        assemble(4, 1, &[(1, 10, 0, 0, 1), (2, -10, 0, 0, 0)]),
        assemble(
            4,
            1,
            &[(1, 10, 0, 0, 1), (2, -10, 0, 0, 2), (3, 20, 0, 0, 0), (4, -20, 0, 0, 0)],
        ),
        assemble(
            4,
            1,
            &[
                (1, 10, 0, 0, 1),
                (2, -10, 0, 0, 2),
                (3, 20, 0, 0, 3),
                (4, -20, 0, 0, 0),
            ],
        ),
        assemble(
            4,
            1,
            &[
                (1, 20, 0, 0, 1),
                (2, -20, 0, 0, 2),
                (3, 35, 0, 0, 3),
                (4, -35, 0, 0, 4),
                (5, 35, 0, 0, 5),
                (6, -35, 0, 0, 7),
                (7, 0, 0, 0, 9),
            ],
        ),
        assemble(
            4,
            1,
            &[
                (1, 10, 12, 0, 1),
                (2, -10, 24, 0, 0),
                (3, 0, 12, 0, 0),
                (4, 0, 19, 0, 0),
                (5, 0, 17, 0, 0),
                (6, 0, 12, 0, 0),
                (7, 0, 19, 0, 0),
            ],
        ),
        assemble(
            4,
            1,
            &[
                (1, 10, 12, 0, 1),
                (2, -10, 24, 0, 2),
                (3, 20, 12, 0, 0),
                (4, -20, 19, 0, 0),
                (5, 0, 17, 0, 0),
                (6, 0, 12, 0, 0),
                (7, 0, 19, 0, 0),
            ],
        ),
        assemble(
            4,
            1,
            &[
                (1, 10, 12, 0, 1),
                (2, -10, 24, 0, 2),
                (3, 20, 12, 0, 3),
                (4, -20, 19, 0, 0),
                (5, 0, 17, 0, 0),
                (6, 0, 12, 0, 0),
                (7, 0, 19, 0, 0),
            ],
        ),
        assemble(
            4,
            1,
            &[
                (1, 20, 12, 0, 1),
                (2, -20, 24, 0, 2),
                (3, 35, 12, 0, 3),
                (4, -35, 19, 0, 4),
                (5, 50, 17, 0, 5),
                (6, -50, 12, 0, 7),
                (7, 0, 19, 0, 9),
            ],
        ),
    ]
}

/// The 48 factory phrases in encoder order: Green 1-16, Red 1-16, Orange 1-16.
/// Ref: CE v5.30 p.16-17, p.23.
pub fn factory_phrases() -> [Phrase; PHRASE_COUNT] {
    let green = green_bank();
    let red = red_bank();
    let orange = orange_bank();
    let mut out = [Phrase::default(); PHRASE_COUNT];
    out[..16].copy_from_slice(&green);
    out[16..32].copy_from_slice(&red);
    out[32..48].copy_from_slice(&orange);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_pool_is_48() {
        assert_eq!(factory_phrases().len(), 48);
    }

    /// Ref p.24: Green phrase 1 is a single 1/8-note echo (STA 24, VEL -10).
    #[test]
    fn green_phrase_1_is_eighth_echo() {
        let p = factory_phrases()[0];
        assert_eq!(p.phrase_type, PhraseType::Forward);
        assert!(p.notes[1].enabled);
        assert_eq!(p.notes[1].start_ticks, 24);
        assert_eq!(p.notes[1].velocity_offset, -10);
        assert!(!p.notes[2].enabled, "phrase 1 has no note 3");
    }

    /// Ref p.24: Green phrase 2 is type 3 (random pitch).
    #[test]
    fn green_phrase_2_is_random_pitch() {
        assert_eq!(factory_phrases()[1].phrase_type, PhraseType::RandomPitch);
        assert_eq!(factory_phrases()[1].notes[2].start_ticks, 48);
    }

    /// Ref p.25: Red phrase 1 base note is PIT +12 (a pitched delay).
    #[test]
    fn red_phrase_1_base_is_plus_octave() {
        let p = factory_phrases()[16];
        assert_eq!(p.notes[0].pitch_offset, 12);
        assert!(p.notes[0].enabled);
        assert_eq!(p.phrase_type, PhraseType::Forward);
    }

    /// Ref p.26: every Orange phrase is type 4 / poly 1.
    #[test]
    fn orange_bank_is_random_all_poly_1() {
        for p in &factory_phrases()[32..48] {
            assert_eq!(p.phrase_type, PhraseType::RandomAll);
            assert_eq!(p.polyphony, 1);
        }
    }

    /// Ref p.26: Orange phrase 1 is a 1-tick STA wobble on note 2.
    #[test]
    fn orange_phrase_1_is_one_tick_flam() {
        let p = factory_phrases()[32];
        assert!(p.notes[1].enabled);
        assert_eq!(p.notes[1].start_ticks, 1);
        assert!(!p.notes[2].enabled);
    }
}

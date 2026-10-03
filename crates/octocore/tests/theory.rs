//! The theory note's checked results, as engine tests (`specs/SPEC-0002/sequencing-theory.md`, section 2.3 and the table in
//! section 6; the request is `journal/metronome/requests/2026-10-03-theory-tests.md`).
//!
//! WHERE THE EXPECTATIONS COME FROM. They are written out by hand from music theory and not computed by any code in this repository:
//! the seven diatonic triads and sevenths of C major are spelled by letter below (C E G, D F A, ...), as pitch classes (C 0, D 2, E 4,
//! F 5, G 7, A 9, B 11). Nothing here was copied from `scale.rs` or from a v1 test. What the tests pin is the engine's tie rule, a pitch
//! exactly halfway between two scale tones going DOWN ("matching v1": `scale.rs`), through the property that rule gives the player: shifting
//! a tonic chord by the root of each degree and forcing it to C major gives the diatonic chord of that degree.
//!
//! WHAT THEY DO NOT SAY. The manual does not say what the Octopus does at a tie, so these tests say what THIS ENGINE does, and why it matters
//! musically, not what the instrument does. If the Octopus breaks ties the other way, these tests are what changes with the engine (Q-M7).

use octocore::domain::{Page, MAX_SCALE_INTERVALS};
use octocore::scale::{build_scale_pitch_classes, quantize_to_scale};
use octocore::{Engine, Event, EventBuffer, RenderContext};

/// The roots of the seven degrees of the major scale, in semitones above the tonic.
const DEGREE_ROOTS: [i32; 7] = [0, 2, 4, 5, 7, 9, 11];
const MAJOR: [i32; 7] = [0, 2, 4, 5, 7, 9, 11];

/// The tonic triad and tonic seventh chord on C (C E G, C E G B), as MIDI pitches.
const TONIC_TRIAD: [i32; 3] = [60, 64, 67];
const TONIC_SEVENTH: [i32; 4] = [60, 64, 67, 71];

/// The diatonic triads of C major by degree, spelled by letter: I C E G, ii D F A, iii E G B, IV F A C, V G B D, vi A C E, vii-dim B D F.
const DIATONIC_TRIADS: [[u8; 3]; 7] = [[0, 4, 7], [2, 5, 9], [4, 7, 11], [5, 9, 0], [7, 11, 2], [9, 0, 4], [11, 2, 5]];
/// The diatonic seventh chords of C major: Cmaj7, Dm7, Em7, Fmaj7, G7, Am7, Bm7b5.
const DIATONIC_SEVENTHS: [[u8; 4]; 7] = [[0, 4, 7, 11], [2, 5, 9, 0], [4, 7, 11, 2], [5, 9, 0, 4], [7, 11, 2, 5], [9, 0, 4, 7], [11, 2, 5, 9]];

fn sorted_classes(pitches: impl IntoIterator<Item = u8>) -> Vec<u8> {
    let mut v: Vec<u8> = pitches.into_iter().map(|p| p % 12).collect();
    v.sort_unstable();
    v
}

fn spelled(chord: &[u8]) -> Vec<u8> {
    sorted_classes(chord.iter().copied())
}

#[test]
fn shifting_a_tonic_chord_by_each_degree_and_forcing_it_to_c_major_gives_the_diatonic_chord_of_that_degree() {
    let c_major = build_scale_pitch_classes(60, &MAJOR);
    for (degree, &root) in DEGREE_ROOTS.iter().enumerate() {
        let triad = TONIC_TRIAD.iter().map(|&p| quantize_to_scale((p + root) as u8, c_major));
        assert_eq!(sorted_classes(triad), spelled(&DIATONIC_TRIADS[degree]), "triad on degree {} (shift {root})", degree + 1);
        let seventh = TONIC_SEVENTH.iter().map(|&p| quantize_to_scale((p + root) as u8, c_major));
        assert_eq!(sorted_classes(seventh), spelled(&DIATONIC_SEVENTHS[degree]), "seventh chord on degree {} (shift {root})", degree + 1);
    }
}

#[test]
fn every_pitch_class_halfway_between_two_scale_tones_goes_down() {
    // A tie is a pitch class with a scale tone one semitone below and one above. C major has five: C# D# F# G# A#; the major pentatonic three;
    // the whole-tone scale six. In each, the engine goes to the tone below.
    let cases: [(&str, i32, &[i32], &[(u8, u8)]); 3] = [
        ("C major", 60, &MAJOR, &[(61, 60), (63, 62), (66, 65), (68, 67), (70, 69)]),
        // C D E G A: the gaps of two semitones are C-D, D-E and G-A, so ties at C#, D# and G#; F, F#, A#, B have a nearer tone.
        ("C major pentatonic", 60, &[0, 2, 4, 7, 9], &[(61, 60), (63, 62), (68, 67)]),
        // C D E F# G# A#: every one of the six other pitch classes is a tie.
        ("C whole-tone", 60, &[0, 2, 4, 6, 8, 10], &[(61, 60), (63, 62), (65, 64), (67, 66), (69, 68), (71, 70)]),
    ];
    for (name, root, intervals, ties) in cases {
        let scale = build_scale_pitch_classes(root, intervals);
        for &(pitch, expected) in ties {
            assert_eq!(quantize_to_scale(pitch, scale), expected, "{name}: {pitch} is halfway between two scale tones, and goes down");
            // The same in another octave, so a rule that only holds near the root is not enough.
            assert_eq!(quantize_to_scale(pitch + 24, scale), expected + 24, "{name}: {} two octaves up", pitch + 24);
        }
    }
}

#[test]
fn a_pitch_nearer_to_a_scale_tone_above_goes_up_and_a_pitch_nearer_to_one_below_goes_down() {
    // The tie rule only decides the halfway cases; everywhere else the nearer tone wins, whichever side it is on. In C D E G A: F (65) is one
    // semitone over E and two under G, so it goes down to E; F# (66) is one under G and two over E, so it goes up to G; A# (70) is one over A
    // and two under C, so it goes down to A; B (71) is one under C and two over A, so it goes up to C.
    let pentatonic = build_scale_pitch_classes(60, &[0, 2, 4, 7, 9]);
    for (pitch, expected) in [(65u8, 64u8), (66, 67), (70, 69), (71, 72)] {
        assert_eq!(quantize_to_scale(pitch, pentatonic), expected, "{pitch} in C pentatonic");
    }
    // A tone in the scale is left alone, in any octave.
    for pitch in [60u8, 62, 64, 67, 69, 72, 36, 105] {
        assert_eq!(quantize_to_scale(pitch, pentatonic), pitch, "{pitch} is in C pentatonic");
    }
}

/// An engine with C major on and track `feeder` feeding `shift` semitones to listeners 1.. holding `chord` (one note per track), the first step
/// of each active. Returns the pitch classes that sounded on the listeners' channels.
fn played_through_the_effector(shift: i8, chord: &[i32]) -> Vec<u8> {
    let mut engine = Engine::new(7);
    let page: &mut Page = engine.grid.active_page_mut();
    page.scale.enabled = true;
    page.scale.root = 60;
    let mut intervals = [0i8; MAX_SCALE_INTERVALS];
    for (slot, &i) in intervals.iter_mut().zip(MAJOR.iter()) {
        *slot = i as i8;
    }
    page.scale.intervals = intervals;
    page.scale.interval_count = MAJOR.len() as u8;
    // Listeners on tracks 1.., channels 1.., each with one chord note; the feeder is track 9 on channel 10, above them, so it runs first.
    for (i, &pitch) in chord.iter().enumerate() {
        let track = &mut page.tracks[i + 1];
        track.pitch = pitch as u8;
        track.midi_channel = i as u8 + 1;
        track.is_listener = true;
        track.steps[0].active = true;
    }
    let feeder = &mut page.tracks[9];
    feeder.midi_channel = 10;
    feeder.is_feeder = true;
    feeder.steps[0].active = true;
    feeder.steps[0].pitch_offset = shift;

    let ctx = RenderContext { sample_rate: 48_000.0, buffer_len: 2048, bpm: 120.0, playing: true };
    let mut notes = Vec::new();
    for _ in 0..4 {
        let mut out = EventBuffer::new();
        engine.render(&ctx, &mut out);
        for e in out.as_slice() {
            if let Event::NoteOn { port: 1, ch, note, .. } = *e {
                if ch as usize <= chord.len() {
                    notes.push(note);
                }
            }
        }
    }
    assert_eq!(notes.len(), chord.len(), "each listener sounded once in the first step: {notes:?}");
    sorted_classes(notes)
}

#[test]
fn a_feeder_track_shifting_a_chord_under_a_c_major_page_scale_plays_the_diatonic_chord() {
    // The same property through the engine: the offsets are summed and THEN forced to the scale (`engine.rs`, `final_pit`), so the feeder's
    // chromatic shift lands on the scale. A quantiser applied before the sum, or ties going up, fail here and not in the pure test above.
    for (degree, &root) in DEGREE_ROOTS.iter().enumerate() {
        assert_eq!(played_through_the_effector(root as i8, &TONIC_TRIAD), spelled(&DIATONIC_TRIADS[degree]), "triad on degree {}", degree + 1);
        assert_eq!(played_through_the_effector(root as i8, &TONIC_SEVENTH), spelled(&DIATONIC_SEVENTHS[degree]), "seventh chord on degree {}", degree + 1);
    }
}

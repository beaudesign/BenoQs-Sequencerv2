//! Tests for the attribute setters and the logical commands `SetTrack` and `SetStep`
//! (SPEC-0001 O6, WENGE-0006, tests C1 and C2). The clamping table below was produced by
//! running the FFI setters as they were at `9e4860b`, before the code moved from `octoffi`
//! into `octocore`, so this file is the proof that the move changed nothing.

use octocore::domain::{STEP_COUNT, TRACK_COUNT};
use octocore::types::{Command, StepAttr, TrackAttr};
use octocore::Engine;

const TRACK: u8 = 3;
const STEP: u8 = 5;

// generated from the FFI setters at 9e4860b: (attribute index, value written, value read back)
pub const TRACK_GOLDEN: &[(u32, i32, i32)] = &[
    (0, -2147483648, 0),
    (0, -1, 0),
    (0, 0, 0),
    (0, 1, 1),
    (0, 127, 127),
    (0, 255, 127),
    (0, 2147483647, 127),
    (1, -2147483648, 0),
    (1, -1, 0),
    (1, 0, 0),
    (1, 1, 1),
    (1, 127, 127),
    (1, 255, 127),
    (1, 2147483647, 127),
    (2, -2147483648, 0),
    (2, -1, 0),
    (2, 0, 0),
    (2, 1, 1),
    (2, 127, 16),
    (2, 255, 16),
    (2, 2147483647, 16),
    (3, -2147483648, 0),
    (3, -1, 0),
    (3, 0, 0),
    (3, 1, 1),
    (3, 127, 16),
    (3, 255, 16),
    (3, 2147483647, 16),
    (4, -2147483648, 1),
    (4, -1, 1),
    (4, 0, 1),
    (4, 1, 1),
    (4, 127, 16),
    (4, 255, 16),
    (4, 2147483647, 16),
    (5, -2147483648, 0),
    (5, -1, 0),
    (5, 0, 0),
    (5, 1, 1),
    (5, 127, 127),
    (5, 255, 255),
    (5, 2147483647, 255),
    (6, -2147483648, -128),
    (6, -1, -1),
    (6, 0, 0),
    (6, 1, 1),
    (6, 127, 127),
    (6, 255, 127),
    (6, 2147483647, 127),
    (7, -2147483648, 0),
    (7, -1, 0),
    (7, 0, 0),
    (7, 1, 1),
    (7, 127, 16),
    (7, 255, 16),
    (7, 2147483647, 16),
    (8, -2147483648, 1),
    (8, -1, 1),
    (8, 0, 1),
    (8, 1, 1),
    (8, 127, 32),
    (8, 255, 32),
    (8, 2147483647, 32),
    (9, -2147483648, 1),
    (9, -1, 1),
    (9, 0, 0),
    (9, 1, 1),
    (9, 127, 1),
    (9, 255, 1),
    (9, 2147483647, 1),
    (10, -2147483648, 1),
    (10, -1, 1),
    (10, 0, 0),
    (10, 1, 1),
    (10, 127, 1),
    (10, 255, 1),
    (10, 2147483647, 1),
    (11, -2147483648, 1),
    (11, -1, 1),
    (11, 0, 0),
    (11, 1, 1),
    (11, 127, 1),
    (11, 255, 1),
    (11, 2147483647, 1),
    (12, -2147483648, 1),
    (12, -1, 1),
    (12, 0, 0),
    (12, 1, 1),
    (12, 127, 1),
    (12, 255, 1),
    (12, 2147483647, 1),
    (13, -2147483648, 1),
    (13, -1, 1),
    (13, 0, 0),
    (13, 1, 1),
    (13, 127, 1),
    (13, 255, 1),
    (13, 2147483647, 1),
    (14, -2147483648, 1),
    (14, -1, 1),
    (14, 0, 0),
    (14, 1, 1),
    (14, 127, 1),
    (14, 255, 1),
    (14, 2147483647, 1),
];
pub const STEP_GOLDEN: &[(u32, i32, i32)] = &[
    (0, -2147483648, 1),
    (0, -1, 1),
    (0, 0, 0),
    (0, 1, 1),
    (0, 127, 1),
    (0, 255, 1),
    (0, 2147483647, 1),
    (1, -2147483648, 1),
    (1, -1, 1),
    (1, 0, 0),
    (1, 1, 1),
    (1, 127, 1),
    (1, 255, 1),
    (1, 2147483647, 1),
    (2, -2147483648, -128),
    (2, -1, -1),
    (2, 0, 0),
    (2, 1, 1),
    (2, 127, 127),
    (2, 255, 127),
    (2, 2147483647, 127),
    (3, -2147483648, -128),
    (3, -1, -1),
    (3, 0, 0),
    (3, 1, 1),
    (3, 127, 127),
    (3, 255, 127),
    (3, 2147483647, 127),
    (4, -2147483648, 1),
    (4, -1, 1),
    (4, 0, 1),
    (4, 1, 1),
    (4, 127, 127),
    (4, 255, 192),
    (4, 2147483647, 192),
    (5, -2147483648, 1),
    (5, -1, 1),
    (5, 0, 1),
    (5, 1, 1),
    (5, 127, 8),
    (5, 255, 8),
    (5, 2147483647, 8),
    (6, -2147483648, -5),
    (6, -1, -1),
    (6, 0, 0),
    (6, 1, 1),
    (6, 127, 5),
    (6, 255, 5),
    (6, 2147483647, 5),
    (7, -2147483648, -128),
    (7, -1, -1),
    (7, 0, 0),
    (7, 1, 1),
    (7, 127, 127),
    (7, 255, 127),
    (7, 2147483647, 127),
    (8, -2147483648, -9),
    (8, -1, -1),
    (8, 0, 0),
    (8, 1, 1),
    (8, 127, 9),
    (8, 255, 9),
    (8, 2147483647, 9),
    (9, -2147483648, 1),
    (9, -1, 1),
    (9, 0, 0),
    (9, 1, 1),
    (9, 127, 1),
    (9, 255, 1),
    (9, 2147483647, 1),
    (10, -2147483648, 0),
    (10, -1, 0),
    (10, 0, 0),
    (10, 1, 1),
    (10, 127, 48),
    (10, 255, 48),
    (10, 2147483647, 48),
    (11, -2147483648, 1),
    (11, -1, 1),
    (11, 0, 1),
    (11, 1, 1),
    (11, 127, 16),
    (11, 255, 16),
    (11, 2147483647, 16),
];

#[test]
fn c1_track_attribute_clamping_matches_the_ffi_table_from_9e4860b() {
    for &(attr, written, expected) in TRACK_GOLDEN {
        let attr = TrackAttr::from_u32(attr).expect("attribute index");
        let mut e = Engine::new(1);
        assert!(e.grid.set_track_attr(TRACK, attr, written));
        assert_eq!(e.grid.track_attr(TRACK, attr), expected, "{attr:?} written {written}");
    }
}

#[test]
fn c1_step_attribute_clamping_matches_the_ffi_table_from_9e4860b() {
    for &(attr, written, expected) in STEP_GOLDEN {
        let attr = StepAttr::from_u32(attr).expect("attribute index");
        let mut e = Engine::new(1);
        assert!(e.grid.set_step_attr(TRACK, STEP, attr, written));
        assert_eq!(e.grid.step_attr(TRACK, STEP, attr), expected, "{attr:?} written {written}");
    }
}

#[test]
fn c1_commands_and_setters_agree_for_every_attribute_and_value() {
    for &(attr, written, expected) in TRACK_GOLDEN {
        let attr = TrackAttr::from_u32(attr).unwrap();
        let mut e = Engine::new(1);
        e.handle_command(Command::SetTrack { track: TRACK, attr, value: written });
        assert_eq!(e.grid.track_attr(TRACK, attr), expected);
    }
    for &(attr, written, expected) in STEP_GOLDEN {
        let attr = StepAttr::from_u32(attr).unwrap();
        let mut e = Engine::new(1);
        e.handle_command(Command::SetStep { track: TRACK, step: STEP, attr, value: written });
        assert_eq!(e.grid.step_attr(TRACK, STEP, attr), expected);
    }
}

#[test]
fn c2_out_of_range_track_or_step_changes_nothing_and_does_not_panic() {
    let untouched = Engine::new(1);
    for track in [TRACK_COUNT as u8, 11, 200, 255] {
        let mut e = Engine::new(1);
        assert!(!e.grid.set_track_attr(track, TrackAttr::Pitch, 99));
        assert_eq!(e.grid.track_attr(track, TrackAttr::Pitch), 0);
        e.handle_command(Command::SetTrack { track, attr: TrackAttr::Pitch, value: 99 });
        e.handle_command(Command::SetStep { track, step: 0, attr: StepAttr::Active, value: 1 });
        assert_eq!(e.grid.active_page().tracks[0].pitch, untouched.grid.active_page().tracks[0].pitch);
    }
    for step in [STEP_COUNT as u8, 17, 200, 255] {
        let mut e = Engine::new(1);
        assert!(!e.grid.set_step_attr(0, step, StepAttr::Active, 1));
        assert_eq!(e.grid.step_attr(0, step, StepAttr::Active), 0);
        e.handle_command(Command::SetStep { track: 0, step, attr: StepAttr::Active, value: 1 });
    }
    // A write that is in range still lands after the bad ones.
    let mut e = Engine::new(1);
    e.handle_command(Command::SetTrack { track: 200, attr: TrackAttr::Pitch, value: 99 });
    e.handle_command(Command::SetTrack { track: 0, attr: TrackAttr::Pitch, value: 99 });
    assert_eq!(e.grid.track_attr(0, TrackAttr::Pitch), 99);
}

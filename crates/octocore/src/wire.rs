//! The wire format of a `Command`: three 64-bit words. This is what crosses the command
//! ring (SPEC-0001 O6, decision D4). Word 0 is the tag, words 1 and 2 are the payload.
//! `from_words` checks everything it reads and returns `None` for a word set that no
//! `Command` could have produced, so a corrupt slot is never mistaken for a command.

use crate::domain::Mode;
use crate::types::{Command, ControlId, StepAttr, TrackAttr};

pub type Words = [u64; 3];

const PLAY: u64 = 0;
const STOP: u64 = 1;
const CONTINUE: u64 = 2;
const RESET: u64 = 3;
const BUTTON_DOWN: u64 = 4;
const BUTTON_UP: u64 = 5;
const ENCODER_TURN: u64 = 6;
const SET_ACTIVE_PAGE: u64 = 7;
const SET_MODE: u64 = 8;
const HOST_TRANSPORT: u64 = 9;
const LOAD_STATE: u64 = 10;
const SET_TRACK: u64 = 11;
const SET_STEP: u64 = 12;

impl Command {
    /// Encodes the command as three words. Floats travel as their exact bits.
    pub fn to_words(&self) -> Words {
        match *self {
            Command::Play => [PLAY, 0, 0],
            Command::Stop => [STOP, 0, 0],
            Command::Continue => [CONTINUE, 0, 0],
            Command::Reset => [RESET, 0, 0],
            Command::ButtonDown { control, velocity_mm_s } => [BUTTON_DOWN, control.0 as u64, velocity_mm_s.to_bits() as u64],
            Command::ButtonUp { control } => [BUTTON_UP, control.0 as u64, 0],
            Command::EncoderTurn { control, detents, angular_velocity } => {
                [ENCODER_TURN, control.0 as u64 | (detents as u16 as u64) << 32, angular_velocity.to_bits() as u64]
            }
            Command::SetActivePage { bank, page } => [SET_ACTIVE_PAGE, bank as u64 | (page as u64) << 8, 0],
            Command::SetMode { mode } => [SET_MODE, mode as u8 as u64, 0],
            Command::HostTransport { ppqn_pos, bpm, playing } => [HOST_TRANSPORT, ppqn_pos, bpm.to_bits() as u64 | (playing as u64) << 32],
            Command::LoadState { handle } => [LOAD_STATE, handle, 0],
            Command::SetTrack { track, attr, value } => [SET_TRACK, track as u64 | (attr as u32 as u64) << 8, value as u32 as u64],
            Command::SetStep { track, step, attr, value } => {
                [SET_STEP, track as u64 | (step as u64) << 8 | (attr as u32 as u64) << 16, value as u32 as u64]
            }
        }
    }

    /// Decodes three words. `None` if they are not what `to_words` produces: an unknown
    /// tag, a field out of range, or a bit no encoder sets.
    pub fn from_words(w: Words) -> Option<Command> {
        let [tag, a, b] = w;
        let none_if = |bad: bool| if bad { None } else { Some(()) };
        match tag {
            PLAY | STOP | CONTINUE | RESET => {
                none_if(a != 0 || b != 0)?;
                Some(match tag {
                    PLAY => Command::Play,
                    STOP => Command::Stop,
                    CONTINUE => Command::Continue,
                    _ => Command::Reset,
                })
            }
            BUTTON_DOWN => {
                none_if(a >> 32 != 0 || b >> 32 != 0)?;
                Some(Command::ButtonDown { control: ControlId(a as u32), velocity_mm_s: f32::from_bits(b as u32) })
            }
            BUTTON_UP => {
                none_if(a >> 32 != 0 || b != 0)?;
                Some(Command::ButtonUp { control: ControlId(a as u32) })
            }
            ENCODER_TURN => {
                none_if(a >> 48 != 0 || b >> 32 != 0)?;
                Some(Command::EncoderTurn {
                    control: ControlId(a as u32),
                    detents: (a >> 32) as u16 as i16,
                    angular_velocity: f32::from_bits(b as u32),
                })
            }
            SET_ACTIVE_PAGE => {
                none_if(a >> 16 != 0 || b != 0)?;
                Some(Command::SetActivePage { bank: a as u8, page: (a >> 8) as u8 })
            }
            SET_MODE => {
                none_if(a > u8::MAX as u64 || b != 0)?;
                Some(Command::SetMode { mode: Mode::from_u8(a as u8)? })
            }
            HOST_TRANSPORT => {
                let playing = match b >> 32 {
                    0 => false,
                    1 => true,
                    _ => return None,
                };
                Some(Command::HostTransport { ppqn_pos: a, bpm: f32::from_bits(b as u32), playing })
            }
            LOAD_STATE => {
                none_if(b != 0)?;
                Some(Command::LoadState { handle: a })
            }
            SET_TRACK => {
                none_if(a >> 40 != 0 || b >> 32 != 0)?;
                Some(Command::SetTrack { track: a as u8, attr: TrackAttr::from_u32((a >> 8) as u32)?, value: b as u32 as i32 })
            }
            SET_STEP => {
                none_if(a >> 48 != 0 || b >> 32 != 0)?;
                Some(Command::SetStep {
                    track: a as u8,
                    step: (a >> 8) as u8,
                    attr: StepAttr::from_u32((a >> 16) as u32)?,
                    value: b as u32 as i32,
                })
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Mode;
    use crate::types::{ControlId, StepAttr, TrackAttr};

    /// Every variant of `Command`, at boundary values. The `match` at the end has no
    /// wildcard arm, so a new `Command` variant that is not added here fails to compile
    /// (test W3): the wire format cannot silently miss a command.
    fn all_commands() -> Vec<Command> {
        let mut v = vec![
            Command::Play,
            Command::Stop,
            Command::Continue,
            Command::Reset,
            Command::ButtonDown { control: ControlId(u32::MAX), velocity_mm_s: f32::from_bits(0x7FC0_1234) },
            Command::ButtonDown { control: ControlId(0), velocity_mm_s: -0.0 },
            Command::ButtonUp { control: ControlId(511) },
            Command::EncoderTurn { control: ControlId(u32::MAX), detents: i16::MIN, angular_velocity: f32::INFINITY },
            Command::EncoderTurn { control: ControlId(7), detents: i16::MAX, angular_velocity: -1.5 },
            Command::SetActivePage { bank: 0, page: 0 },
            Command::SetActivePage { bank: u8::MAX, page: u8::MAX },
            Command::HostTransport { ppqn_pos: u64::MAX, bpm: 999.0, playing: true },
            Command::HostTransport { ppqn_pos: 0, bpm: f32::NAN, playing: false },
            Command::LoadState { handle: u64::MAX },
            Command::LoadState { handle: 0 },
        ];
        for mode in Mode::ALL {
            v.push(Command::SetMode { mode });
        }
        for attr in TrackAttr::ALL {
            for value in [i32::MIN, -1, 0, 1, i32::MAX] {
                v.push(Command::SetTrack { track: 9, attr, value });
            }
        }
        v.push(Command::SetTrack { track: u8::MAX, attr: TrackAttr::Pitch, value: 0 });
        for attr in StepAttr::ALL {
            for value in [i32::MIN, -1, 0, 1, i32::MAX] {
                v.push(Command::SetStep { track: 9, step: 15, attr, value });
            }
        }
        v.push(Command::SetStep { track: u8::MAX, step: u8::MAX, attr: StepAttr::Active, value: 0 });
        v
    }

    /// The compile-time guard for W3. Adding a `Command` variant breaks this `match`.
    #[allow(dead_code)]
    fn every_variant_is_listed(c: &Command) -> &'static str {
        match c {
            Command::Play => "Play",
            Command::Stop => "Stop",
            Command::Continue => "Continue",
            Command::Reset => "Reset",
            Command::ButtonDown { .. } => "ButtonDown",
            Command::ButtonUp { .. } => "ButtonUp",
            Command::EncoderTurn { .. } => "EncoderTurn",
            Command::SetActivePage { .. } => "SetActivePage",
            Command::SetMode { .. } => "SetMode",
            Command::HostTransport { .. } => "HostTransport",
            Command::LoadState { .. } => "LoadState",
            Command::SetTrack { .. } => "SetTrack",
            Command::SetStep { .. } => "SetStep",
        }
    }

    #[test]
    fn w1_round_trip_is_exact_for_every_variant_at_boundary_values() {
        let all = all_commands();
        assert!(all.len() > 150);
        for c in all {
            let back = Command::from_words(c.to_words()).unwrap_or_else(|| panic!("{c:?} did not decode"));
            // `PartialEq` on floats says NaN != NaN, so compare the encoded form as well:
            // bit-identical words mean bit-identical floats.
            assert_eq!(back.to_words(), c.to_words(), "{c:?}");
            if !format!("{c:?}").contains("NaN") {
                assert_eq!(back, c);
            }
        }
    }

    #[test]
    fn w2_words_no_command_could_produce_decode_to_none_and_never_panic() {
        // Unknown tags.
        for tag in [13u64, 14, 100, u32::MAX as u64, u64::MAX] {
            assert_eq!(Command::from_words([tag, 0, 0]), None, "tag {tag}");
        }
        // Known tags with a field out of range.
        assert_eq!(Command::from_words([8, 4, 0]), None, "SetMode with mode 4");
        assert_eq!(Command::from_words([11, 0 | (15 << 8), 0]), None, "SetTrack with attr 15");
        assert_eq!(Command::from_words([12, 0 | (0 << 8) | (12 << 16), 0]), None, "SetStep with attr 12");
        assert_eq!(Command::from_words([9, 0, 2 << 32]), None, "HostTransport with playing = 2");
        // Bits that no encoder sets.
        assert_eq!(Command::from_words([0, 1, 0]), None, "Play with a payload");
        assert_eq!(Command::from_words([7, 1 << 16, 0]), None, "SetActivePage with a stray bit");
        assert_eq!(Command::from_words([11, 0, 1 << 40]), None, "SetTrack with a stray high value bit");
        // A sweep: whatever the words, decoding neither panics nor produces a command that
        // encodes differently.
        let mut x = 0x9E37_79B9_7F4A_7C15u64;
        for _ in 0..20_000 {
            let mut next = || {
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                x
            };
            let w = [next() % 16, next(), next()];
            if let Some(c) = Command::from_words(w) {
                assert_eq!(c.to_words(), w, "{w:x?} decoded to {c:?}, which encodes differently");
            }
        }
    }

    #[test]
    fn w3_every_variant_is_covered_by_the_round_trip_list() {
        let mut seen = std::collections::BTreeSet::new();
        for c in all_commands() {
            seen.insert(every_variant_is_listed(&c));
        }
        assert_eq!(seen.len(), 13);
    }

    #[test]
    fn w4_command_size_is_the_by_value_abi() {
        // `Command` crosses the C ABI by value. New variants must not grow it, or a header
        // built before them would pass a struct that is too small.
        assert_eq!(std::mem::size_of::<Command>(), 24);
        assert_eq!(std::mem::align_of::<Command>(), 8);
    }
}

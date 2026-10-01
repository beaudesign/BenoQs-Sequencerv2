//! The wire forms the page reads: an event as 12 bytes, an LED as one byte. Both are listed in
//! `ABI.md`, and `tests/abi.rs` rebuilds them independently from that description.

use octocore::Event;
use octoface::{Colour, Led, Phase};

pub const EVENT_BYTES: usize = 12;

/// `[kind, port, channel, d1, d2 low, d2 high, 0, 0, at_sample as u32 little endian]`.
/// Kind: 0 Note On (d1 note, d2 velocity), 1 Note Off (d1 note), 2 CC (d1 controller, d2 value),
/// 3 pitch bend (d2 the 14-bit value), 4 channel pressure (d1 the value).
pub fn write_event(out: &mut [u8], e: &Event) {
    let (kind, port, ch, d1, d2, at) = match *e {
        Event::NoteOn { port, ch, note, vel, at_sample } => (0u8, port, ch, note, u16::from(vel), at_sample),
        Event::NoteOff { port, ch, note, at_sample } => (1, port, ch, note, 0, at_sample),
        Event::Cc { port, ch, cc, val, at_sample } => (2, port, ch, cc, u16::from(val), at_sample),
        Event::PitchBend { port, ch, value, at_sample } => (3, port, ch, 0, value, at_sample),
        Event::ChannelPressure { port, ch, value, at_sample } => (4, port, ch, value, 0, at_sample),
    };
    out[0] = kind;
    out[1] = port;
    out[2] = ch;
    out[3] = d1;
    out[4..6].copy_from_slice(&d2.to_le_bytes());
    out[6] = 0;
    out[7] = 0;
    out[8..12].copy_from_slice(&at.to_le_bytes());
}

/// The colour in the low two bits (off 0, red 1, green 2, orange 3) and the phase in the next two
/// (steady 0, flash 1, shine 2).
pub fn led_byte(l: Led) -> u8 {
    let colour = match l.colour {
        Colour::Off => 0,
        Colour::Red => 1,
        Colour::Green => 2,
        Colour::Orange => 3,
    };
    let phase = match l.phase {
        Phase::Steady => 0,
        Phase::Flash => 1,
        Phase::Shine => 2,
    };
    colour | (phase << 2)
}

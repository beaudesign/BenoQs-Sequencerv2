//! The wire forms the page reads: an event as 12 bytes, an LED as one byte. Both are listed in
//! `ABI.md`, and `tests/abi.rs` rebuilds them independently from that description.

use octocore::{Event, RealtimeEvent};
use octoface::{Colour, Led, Phase};

pub const EVENT_BYTES: usize = 12;
/// The kind of a record that is a MIDI real-time message (ABI 2, ADR-0009).
pub const KIND_REALTIME: u8 = 5;

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

/// `[5, 0, 0, status, 0, 0, 0, 0, at_sample as u32 little endian]`: kind 5, no port and no channel, the MIDI status byte in `d1`
/// (0xF8 Clock, 0xFA Start, 0xFB Continue, 0xFC Stop), nothing in `d2`.
pub fn write_realtime(out: &mut [u8], e: &RealtimeEvent) {
    out[0] = KIND_REALTIME;
    out[1] = 0;
    out[2] = 0;
    out[3] = e.msg as u8;
    out[4..8].fill(0);
    out[8..12].copy_from_slice(&e.at_sample.to_le_bytes());
}

fn event_at(e: &Event) -> u32 {
    match *e {
        Event::NoteOn { at_sample, .. }
        | Event::NoteOff { at_sample, .. }
        | Event::Cc { at_sample, .. }
        | Event::PitchBend { at_sample, .. }
        | Event::ChannelPressure { at_sample, .. } => at_sample,
    }
}

/// One render's notes and real-time messages as one list of records in `out`, in sample order, and the number of records written.
/// Each list keeps its own order. Where a real-time message and a note fall on one sample the real-time message comes first, so a
/// clock pulse is never held behind a note it is on the same tick as. Stops when `out` is full; the page's buffer is sized for the
/// most either list can hold (`host::EVENT_CAPACITY`), so that does not happen.
pub fn write_merged(out: &mut [u8], events: &[Event], realtime: &[RealtimeEvent]) -> usize {
    let room = out.len() / EVENT_BYTES;
    let (mut i, mut j, mut n) = (0, 0, 0);
    while n < room && (i < events.len() || j < realtime.len()) {
        let realtime_next = match (events.get(i), realtime.get(j)) {
            (Some(e), Some(r)) => r.at_sample <= event_at(e),
            (None, Some(_)) => true,
            _ => false,
        };
        let slot = &mut out[n * EVENT_BYTES..(n + 1) * EVENT_BYTES];
        if realtime_next {
            write_realtime(slot, &realtime[j]);
            j += 1;
        } else {
            write_event(slot, &events[i]);
            i += 1;
        }
        n += 1;
    }
    n
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

#[cfg(test)]
mod tests {
    use super::*;
    use octocore::Realtime;

    const NOTE: fn(u32) -> Event = |at| Event::NoteOn { port: 1, ch: 1, note: 60, vel: 100, at_sample: at };
    const RT: fn(Realtime, u32) -> RealtimeEvent = |msg, at_sample| RealtimeEvent { msg, at_sample };

    fn kinds_and_places(out: &[u8], n: usize) -> Vec<(u8, u32)> {
        out.chunks(EVENT_BYTES).take(n).map(|c| (c[0], u32::from_le_bytes([c[8], c[9], c[10], c[11]]))).collect()
    }

    #[test]
    fn a_real_time_record_is_kind_5_with_the_status_byte_in_d1_and_nothing_else() {
        let mut out = [0xEE; EVENT_BYTES];
        write_realtime(&mut out, &RT(Realtime::Continue, 0x0102_0304));
        assert_eq!(out, [5, 0, 0, 0xFB, 0, 0, 0, 0, 0x04, 0x03, 0x02, 0x01], "every byte is written, including the ones that are zero");
        for (msg, status) in [(Realtime::Clock, 0xF8), (Realtime::Start, 0xFA), (Realtime::Continue, 0xFB), (Realtime::Stop, 0xFC)] {
            write_realtime(&mut out, &RT(msg, 0));
            assert_eq!(out[3], status);
        }
    }

    #[test]
    fn the_two_lists_are_merged_by_sample_and_a_real_time_message_goes_first_on_a_tie() {
        let events = [NOTE(0), NOTE(5), NOTE(5), NOTE(9)];
        let realtime = [RT(Realtime::Start, 0), RT(Realtime::Clock, 0), RT(Realtime::Clock, 5), RT(Realtime::Clock, 7)];
        let mut out = [0u8; 8 * EVENT_BYTES];
        let n = write_merged(&mut out, &events, &realtime);
        assert_eq!(n, 8);
        assert_eq!(
            kinds_and_places(&out, n),
            vec![(5, 0), (5, 0), (0, 0), (5, 5), (0, 5), (0, 5), (5, 7), (0, 9)],
            "on sample 0 both messages come before the note, on sample 5 the pulse comes before both notes, and each list keeps its order"
        );
        assert_eq!(out[3], 0xFA, "Start before the Clock it was scheduled ahead of");
        assert_eq!(out[EVENT_BYTES + 3], 0xF8);
    }

    #[test]
    fn with_one_list_empty_the_other_is_written_as_it_is() {
        let mut out = [0u8; 4 * EVENT_BYTES];
        assert_eq!(write_merged(&mut out, &[], &[]), 0);
        assert_eq!(write_merged(&mut out, &[NOTE(3), NOTE(4)], &[]), 2);
        let mut want = [0u8; EVENT_BYTES];
        write_event(&mut want, &NOTE(4));
        assert_eq!(&out[EVENT_BYTES..2 * EVENT_BYTES], &want);
        assert_eq!(write_merged(&mut out, &[], &[RT(Realtime::Clock, 9), RT(Realtime::Stop, 9)]), 2);
        assert_eq!((out[3], out[EVENT_BYTES + 3]), (0xF8, 0xFC));
    }

    #[test]
    fn a_full_buffer_stops_the_merge_and_never_writes_past_the_end() {
        let events = [NOTE(1), NOTE(2), NOTE(3)];
        let realtime = [RT(Realtime::Clock, 0), RT(Realtime::Clock, 4)];
        let mut out = [0u8; 3 * EVENT_BYTES];
        assert_eq!(write_merged(&mut out, &events, &realtime), 3);
        assert_eq!(kinds_and_places(&out, 3), vec![(5, 0), (0, 1), (0, 2)]);
        assert_eq!(write_merged(&mut [0u8; 0], &events, &realtime), 0);
        assert_eq!(write_merged(&mut [0u8; EVENT_BYTES - 1], &events, &realtime), 0, "a part of a record is no room");
    }
}

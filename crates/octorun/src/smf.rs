//! Standard MIDI File output (format 1, 960 ticks per quarter note).
//!
//! Track 0 carries the tempo map. Then one track per port that has events, each opened with
//! a MIDI port prefix (`FF 21 01 pp`, pp 0 for port 1 and 1 for port 2) so the two ports
//! survive the trip. The engine's sample positions are converted to ticks through the
//! script's tempo map, with plain `f64` arithmetic (multiply, divide, round), which is
//! bit-for-bit the same on every platform, so the bytes are too.

use octocore::fixture::Played;
use octocore::Event;

pub const DIVISION: u16 = 960;

/// The most a Set Tempo meta event can hold: 3 bytes of microseconds per quarter note.
const MAX_TEMPO_META: u32 = 0x00FF_FFFF;

fn micros_per_quarter(bpm: f32) -> u32 {
    // Saturating float to int cast: a tiny tempo gives a huge value, never a wrapped one.
    (60_000_000.0f64 / bpm as f64).round() as u32
}

fn varlen(mut v: u32, out: &mut Vec<u8>) {
    let mut bytes = [0u8; 5];
    let mut n = 0;
    loop {
        bytes[n] = (v & 0x7F) as u8;
        n += 1;
        v >>= 7;
        if v == 0 {
            break;
        }
    }
    for i in (0..n).rev() {
        out.push(bytes[i] | if i > 0 { 0x80 } else { 0 });
    }
}

/// Ticks at sample `s`, integrating the tempo map. `tempo` is `(from sample, bpm)`, sorted.
fn ticks_at(tempo: &[(u64, f32)], sample_rate: f64, s: u64) -> f64 {
    let mut ticks = 0.0f64;
    for (i, &(from, bpm)) in tempo.iter().enumerate() {
        let to = tempo.get(i + 1).map_or(u64::MAX, |n| n.0);
        let per_sample = bpm as f64 * DIVISION as f64 / (60.0 * sample_rate);
        if s <= from {
            break;
        }
        ticks += (s.min(to) - from) as f64 * per_sample;
        if s <= to {
            break;
        }
    }
    ticks
}

fn track(events: Vec<u8>) -> Vec<u8> {
    let mut chunk = Vec::with_capacity(events.len() + 12);
    chunk.extend_from_slice(b"MTrk");
    chunk.extend_from_slice(&((events.len() + 4) as u32).to_be_bytes()); // + end of track
    chunk.extend_from_slice(&events);
    chunk.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]);
    chunk
}

fn meta_text(kind: u8, text: &str, out: &mut Vec<u8>) {
    out.extend_from_slice(&[0x00, 0xFF, kind]);
    varlen(text.len() as u32, out);
    out.extend_from_slice(text.as_bytes());
}

pub fn render(played: &Played) -> Result<Vec<u8>, String> {
    if !played.sample_rate_constant {
        return Err("the script changes the sample rate after rendering began, so its sample positions cannot become ticks".into());
    }
    let rate = played.sample_rate as f64;
    if !(rate.is_finite() && rate > 0.0) {
        return Err(format!("bad sample rate {}", played.sample_rate));
    }
    for &(_, bpm) in &played.tempo {
        if !(bpm.is_finite() && bpm > 0.0) {
            return Err(format!("bad tempo {bpm}"));
        }
        if micros_per_quarter(bpm) > MAX_TEMPO_META {
            return Err(format!(
                "tempo {bpm} BPM is too slow for a Standard MIDI File: a tempo event holds 3 bytes, so the slowest is 3.58 BPM"
            ));
        }
    }

    // Track 0: name, then a tempo event for each entry of the tempo map.
    let mut conductor = Vec::new();
    meta_text(0x03, "octorun", &mut conductor);
    let mut last_tick = 0u64;
    for &(at, bpm) in &played.tempo {
        let tick = ticks_at(&played.tempo, rate, at).round() as u64;
        varlen((tick - last_tick) as u32, &mut conductor);
        last_tick = tick;
        let micros = micros_per_quarter(bpm);
        conductor.extend_from_slice(&[0xFF, 0x51, 0x03, (micros >> 16) as u8, (micros >> 8) as u8, micros as u8]);
    }

    let mut ports: [Vec<u8>; 2] = [Vec::new(), Vec::new()];
    let mut last: [u64; 2] = [0, 0];
    for (t, e) in &played.events {
        let (port, ch, bytes): (u8, u8, [u8; 3]) = match *e {
            Event::NoteOn { port, ch, note, vel, .. } => (port, ch, [0x90, note, vel]),
            Event::NoteOff { port, ch, note, .. } => (port, ch, [0x80, note, 0]),
            Event::Cc { port, ch, cc, val, .. } => (port, ch, [0xB0, cc, val]),
            Event::PitchBend { port, ch, value, .. } => (port, ch, [0xE0, (value & 0x7F) as u8, (value >> 7) as u8]),
            Event::ChannelPressure { port, ch, value, .. } => (port, ch, [0xD0, value, 0]),
        };
        if !(1..=2).contains(&port) || !(1..=16).contains(&ch) || bytes[1] > 127 || bytes[2] > 127 {
            return Err(format!("event at sample {t} is outside MIDI ranges: {e:?}"));
        }
        let p = (port - 1) as usize;
        let tick = ticks_at(&played.tempo, rate, *t).round() as u64;
        varlen((tick - last[p]) as u32, &mut ports[p]);
        last[p] = tick;
        ports[p].push(bytes[0] | (ch - 1));
        // Channel pressure has one data byte, everything else has two.
        let n = if bytes[0] == 0xD0 { 1 } else { 2 };
        ports[p].extend_from_slice(&bytes[1..1 + n]);
    }

    let used: Vec<usize> = (0..2).filter(|&p| !ports[p].is_empty()).collect();
    let mut file = Vec::new();
    file.extend_from_slice(b"MThd");
    file.extend_from_slice(&6u32.to_be_bytes());
    file.extend_from_slice(&1u16.to_be_bytes());
    file.extend_from_slice(&((1 + used.len()) as u16).to_be_bytes());
    file.extend_from_slice(&DIVISION.to_be_bytes());
    file.extend_from_slice(&track(conductor));
    for p in used {
        let mut head = Vec::new();
        meta_text(0x03, &format!("port {}", p + 1), &mut head);
        head.extend_from_slice(&[0x00, 0xFF, 0x21, 0x01, p as u8]);
        head.extend_from_slice(&ports[p]);
        file.extend_from_slice(&track(head));
    }
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn played(events: Vec<(u64, Event)>, tempo: Vec<(u64, f32)>) -> Played {
        Played { events, tempo, sample_rate: 48_000.0, samples: 96_000, sample_rate_constant: true, diagnostics: Default::default() }
    }

    fn var(v: u32) -> Vec<u8> {
        let mut o = Vec::new();
        varlen(v, &mut o);
        o
    }

    #[test]
    fn variable_length_quantities_match_the_spec_examples() {
        // From the Standard MIDI File spec, section 1 "Variable length quantity".
        assert_eq!(var(0x00), [0x00]);
        assert_eq!(var(0x40), [0x40]);
        assert_eq!(var(0x7F), [0x7F]);
        assert_eq!(var(0x80), [0x81, 0x00]);
        assert_eq!(var(0x2000), [0xC0, 0x00]);
        assert_eq!(var(0x3FFF), [0xFF, 0x7F]);
        assert_eq!(var(0x4000), [0x81, 0x80, 0x00]);
        assert_eq!(var(0x0FFF_FFFF), [0xFF, 0xFF, 0xFF, 0x7F]);
    }

    #[test]
    fn one_beat_at_120_bpm_is_960_ticks_and_a_tempo_change_bends_the_map() {
        let flat = [(0u64, 120.0f32)];
        assert_eq!(ticks_at(&flat, 48_000.0, 24_000), 960.0);
        assert_eq!(ticks_at(&flat, 48_000.0, 0), 0.0);
        // 24,000 samples at 120 BPM (960 ticks), then 24,000 at 240 BPM (1,920 ticks).
        let two = [(0u64, 120.0f32), (24_000, 240.0)];
        assert_eq!(ticks_at(&two, 48_000.0, 24_000), 960.0);
        assert_eq!(ticks_at(&two, 48_000.0, 48_000), 960.0 + 1920.0);
    }

    #[test]
    fn header_tempo_and_one_note_are_laid_out_as_the_spec_says() {
        let p = played(
            vec![
                (24_000, Event::NoteOn { port: 1, ch: 3, note: 60, vel: 100, at_sample: 0 }),
                (48_000, Event::NoteOff { port: 1, ch: 3, note: 60, at_sample: 0 }),
            ],
            vec![(0, 120.0)],
        );
        let f = render(&p).unwrap();
        assert_eq!(&f[..14], b"MThd\0\0\0\x06\0\x01\0\x02\x03\xC0", "format 1, 2 tracks, 960 per quarter");
        // Track 0: name "octorun", tempo 500000 us = 0x07A120, end of track.
        let want0: Vec<u8> = [&b"MTrk"[..], &[0, 0, 0, 22], &[0, 0xFF, 3, 7], b"octorun", &[0, 0xFF, 0x51, 3, 0x07, 0xA1, 0x20], &[0, 0xFF, 0x2F, 0]]
            .concat();
        assert_eq!(&f[14..14 + want0.len()], &want0[..]);
        // Track 1: name, port prefix 0, NoteOn ch 3 after 960 ticks, NoteOff after 960 more.
        let t1 = &f[14 + want0.len()..];
        assert_eq!(&t1[..4], b"MTrk");
        let body = &t1[8..];
        let want = [
            &[0u8, 0xFF, 3, 6][..],
            b"port 1",
            &[0, 0xFF, 0x21, 1, 0],
            &[0x87, 0x40, 0x92, 60, 100], // 960 = 0x87 0x40
            &[0x87, 0x40, 0x82, 60, 0],
            &[0, 0xFF, 0x2F, 0],
        ]
        .concat();
        assert_eq!(body, &want[..]);
        assert_eq!(u32::from_be_bytes([t1[4], t1[5], t1[6], t1[7]]) as usize, want.len());
    }

    #[test]
    fn port_2_gets_its_own_track_with_port_prefix_1_and_bend_and_pressure_are_encoded() {
        let p = played(
            vec![
                (0, Event::PitchBend { port: 2, ch: 16, value: 0x1234, at_sample: 0 }),
                (0, Event::ChannelPressure { port: 2, ch: 16, value: 99, at_sample: 0 }),
            ],
            vec![(0, 120.0)],
        );
        let f = render(&p).unwrap();
        assert_eq!(u16::from_be_bytes([f[10], f[11]]), 2, "one conductor track and one for port 2");
        assert!(f.windows(5).any(|w| w == [0, 0xFF, 0x21, 1, 1]), "port prefix for port 2");
        assert!(f.windows(4).any(|w| w == [0x00, 0xEF, 0x34, 0x24]), "bend 0x1234 = lsb 0x34, msb 0x24, channel 16");
        assert!(f.windows(3).any(|w| w == [0x00, 0xDF, 99]), "pressure has one data byte");
    }

    #[test]
    fn a_stream_with_no_events_is_a_valid_file_with_only_the_tempo_track() {
        let f = render(&played(vec![], vec![(0, 100.0)])).unwrap();
        assert_eq!(u16::from_be_bytes([f[10], f[11]]), 1);
    }

    #[test]
    fn a_tempo_too_slow_for_a_three_byte_tempo_meta_is_refused_not_truncated() {
        // A Set Tempo meta holds microseconds per quarter note in 3 bytes: at most 16,777,215,
        // which is 3.5763 BPM. The engine follows tempos down to 1 BPM. Truncating 3 BPM
        // (20,000,000 us) wrote 3,222,784 us, a file that plays at about 18.6 BPM.
        for bpm in [1.0f32, 3.0, 3.5] {
            let e = render(&played(vec![], vec![(0, bpm)])).unwrap_err();
            assert!(e.contains("3.58") && e.contains("tempo"), "bpm {bpm}: {e}");
        }
        // The slowest tempo that fits still round-trips exactly.
        let f = render(&played(vec![], vec![(0, 3.58)])).unwrap();
        let at = f.windows(3).position(|w| w == [0xFF, 0x51, 0x03]).unwrap() + 3;
        let micros = u32::from_be_bytes([0, f[at], f[at + 1], f[at + 2]]);
        assert_eq!(micros, (60_000_000.0f64 / 3.58f32 as f64).round() as u32);
    }

    #[test]
    fn refuses_what_it_cannot_represent() {
        let bad_ch = played(vec![(0, Event::NoteOn { port: 1, ch: 17, note: 60, vel: 1, at_sample: 0 })], vec![(0, 120.0)]);
        assert!(render(&bad_ch).unwrap_err().contains("outside MIDI ranges"));
        let mut rate = played(vec![], vec![(0, 120.0)]);
        rate.sample_rate_constant = false;
        assert!(render(&rate).unwrap_err().contains("sample rate"));
        let tempo = played(vec![], vec![(0, 0.0)]);
        assert!(render(&tempo).unwrap_err().contains("tempo"));
    }
}

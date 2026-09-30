//! The event log: one JSON object per line, written by hand so the bytes are ours to keep
//! stable. The first line describes the run; every other line is one event.
//!
//! ```text
//! {"octorun":"events/1","sample_rate":48000,"samples":96000,"tempo":[[0,120]]}
//! {"t":1375,"kind":"on","port":1,"ch":1,"note":69,"vel":100}
//! {"t":2875,"kind":"off","port":1,"ch":1,"note":69}
//! {"t":1375,"kind":"cc","port":1,"ch":1,"cc":74,"val":33}
//! {"t":1375,"kind":"bend","port":1,"ch":1,"value":8192}
//! {"t":1375,"kind":"pressure","port":1,"ch":1,"value":100}
//! ```
//!
//! `t` is the absolute sample since the first `render`. Events appear in the order the engine
//! emitted them, which is the order a synth would receive them. Floats are printed with
//! Rust's shortest round-trip form, which is the same on every platform.

use octocore::fixture::Played;
use octocore::Event;

pub const FORMAT: &str = "events/1";

pub fn render(played: &Played) -> String {
    let mut out = String::with_capacity(64 + played.events.len() * 56);
    out.push_str(&format!(
        "{{\"octorun\":\"{FORMAT}\",\"sample_rate\":{},\"samples\":{},\"tempo\":[",
        played.sample_rate, played.samples
    ));
    for (i, (at, bpm)) in played.tempo.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!("[{at},{bpm}]"));
    }
    out.push_str("]}\n");

    for (t, e) in &played.events {
        out.push_str(&match *e {
            Event::NoteOn { port, ch, note, vel, .. } => {
                format!("{{\"t\":{t},\"kind\":\"on\",\"port\":{port},\"ch\":{ch},\"note\":{note},\"vel\":{vel}}}\n")
            }
            Event::NoteOff { port, ch, note, .. } => {
                format!("{{\"t\":{t},\"kind\":\"off\",\"port\":{port},\"ch\":{ch},\"note\":{note}}}\n")
            }
            Event::Cc { port, ch, cc, val, .. } => {
                format!("{{\"t\":{t},\"kind\":\"cc\",\"port\":{port},\"ch\":{ch},\"cc\":{cc},\"val\":{val}}}\n")
            }
            Event::PitchBend { port, ch, value, .. } => {
                format!("{{\"t\":{t},\"kind\":\"bend\",\"port\":{port},\"ch\":{ch},\"value\":{value}}}\n")
            }
            Event::ChannelPressure { port, ch, value, .. } => {
                format!("{{\"t\":{t},\"kind\":\"pressure\",\"port\":{port},\"ch\":{ch},\"value\":{value}}}\n")
            }
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn played(events: Vec<(u64, Event)>) -> Played {
        Played { events, tempo: vec![(0, 120.0), (48_000, 133.5)], sample_rate: 48_000.0, samples: 96_000, sample_rate_constant: true, diagnostics: Default::default() }
    }

    #[test]
    fn writes_a_header_then_one_line_per_event_in_a_fixed_shape() {
        let p = played(vec![
            (1375, Event::NoteOn { port: 1, ch: 1, note: 69, vel: 100, at_sample: 351 }),
            (2875, Event::NoteOff { port: 1, ch: 1, note: 69, at_sample: 315 }),
            (2875, Event::Cc { port: 2, ch: 16, cc: 74, val: 33, at_sample: 0 }),
            (2876, Event::PitchBend { port: 1, ch: 3, value: 16256, at_sample: 0 }),
            (2877, Event::ChannelPressure { port: 1, ch: 4, value: 100, at_sample: 0 }),
        ]);
        let text = render(&p);
        let want = concat!(
            "{\"octorun\":\"events/1\",\"sample_rate\":48000,\"samples\":96000,\"tempo\":[[0,120],[48000,133.5]]}\n",
            "{\"t\":1375,\"kind\":\"on\",\"port\":1,\"ch\":1,\"note\":69,\"vel\":100}\n",
            "{\"t\":2875,\"kind\":\"off\",\"port\":1,\"ch\":1,\"note\":69}\n",
            "{\"t\":2875,\"kind\":\"cc\",\"port\":2,\"ch\":16,\"cc\":74,\"val\":33}\n",
            "{\"t\":2876,\"kind\":\"bend\",\"port\":1,\"ch\":3,\"value\":16256}\n",
            "{\"t\":2877,\"kind\":\"pressure\",\"port\":1,\"ch\":4,\"value\":100}\n",
        );
        assert_eq!(text, want);
    }

    #[test]
    fn buffer_relative_positions_are_not_in_the_log() {
        let a = played(vec![(10, Event::NoteOff { port: 1, ch: 1, note: 60, at_sample: 10 })]);
        let b = played(vec![(10, Event::NoteOff { port: 1, ch: 1, note: 60, at_sample: 999 })]);
        assert_eq!(render(&a), render(&b));
    }
}

//! `octorun`: run a pattern file through the sequencer core with no host, no audio device and
//! no clock, and write what it played as a byte-reproducible event log and a Standard MIDI
//! File. The same pattern and seed give the same bytes on every machine, so a SHA-256 of the
//! output is a regression test for the whole engine (see `examples/golden/`).
//!
//! A pattern file is the conformance fixture language (`octocore::fixture`, DSL v1 to v3):
//! `seed`, `bpm`, `track T step S attr V`, `play`, `render 4 s buffer=64`, and so on. Its
//! `expect` lines still run, so a pattern can check itself.

pub mod ndjson;
pub mod sha256;
pub mod smf;

use octocore::fixture::{run_script, Limits};
use octocore::{Diagnostics, QUEUE_CAP};

/// A pattern may render at most an hour at 48 kHz and produce at most 5 million events, so a
/// typo is an error message and not an out-of-memory.
pub const LIMITS: Limits = Limits { max_samples: 48_000 * 3600, max_events: 5_000_000 };

pub struct Output {
    pub ndjson: String,
    pub smf: Vec<u8>,
    pub events: usize,
    pub samples: u64,
    /// What the engine refused, deferred or delayed while playing the pattern.
    pub diagnostics: Diagnostics,
}

impl Output {
    pub fn ndjson_sha256(&self) -> String {
        sha256::sha256_hex(self.ndjson.as_bytes())
    }

    pub fn smf_sha256(&self) -> String {
        sha256::sha256_hex(&self.smf)
    }

    /// Lines to show a person when the engine could not play the pattern in full. Empty for a
    /// clean run. The output is still written: it is what the engine did, and that is what a
    /// golden hash records, but a stream with missing notes must not pass for a good one.
    pub fn warnings(&self) -> Vec<String> {
        let d = &self.diagnostics;
        let mut w = Vec::new();
        if d.queue_overflows > 0 {
            w.push(format!(
                "the engine refused {} notes because its event queue (capacity {QUEUE_CAP}) was full; the output is missing them",
                d.queue_overflows
            ));
        }
        if d.deferred_events > 0 {
            w.push(format!("{} events waited for a later render because a buffer was full", d.deferred_events));
        }
        if d.late_events > 0 {
            w.push(format!("{} events were sent after their time", d.late_events));
        }
        if d.unusable_tempo_renders > 0 {
            w.push(format!("{} renders had a tempo or sample rate the engine cannot follow and played nothing", d.unusable_tempo_renders));
        }
        w
    }

    /// The two-line form stored in `examples/golden/<name>.sha256`.
    pub fn golden_text(&self) -> String {
        format!("events {}\nsmf {}\n", self.ndjson_sha256(), self.smf_sha256())
    }
}

pub fn run_pattern(source: &str) -> Result<Output, String> {
    let played = run_script(source, LIMITS)?;
    let smf = smf::render(&played)?;
    Ok(Output { ndjson: ndjson::render(&played), smf, events: played.events.len(), samples: played.samples, diagnostics: played.diagnostics })
}

//! Spike S2: run a pattern through the headless runner inside the module, so the same code that
//! made `examples/golden/*.sha256` natively runs in an AudioWorklet and its hash can be compared.

use crate::host::with_scratch;

/// Reads `len` bytes of pattern text from scratch, runs it, and leaves the result in scratch: the
/// SHA-256 of the event log as 64 hex digits, or an error message. Returns the result's length,
/// with bit 31 set when it is an error message.
pub fn run(len: usize) -> u32 {
    with_scratch(|s| {
        let result = match s.get(..len).map(std::str::from_utf8) {
            Some(Ok(source)) => octorun::run_pattern(source).map(|o| o.ndjson_sha256()),
            Some(Err(e)) => Err(format!("the pattern is not UTF-8: {e}")),
            None => Err(format!("pattern length {len} is more than the scratch memory")),
        };
        let (text, flag) = match result {
            Ok(hash) => (hash, 0),
            Err(message) => (message, 1u32 << 31),
        };
        s.clear();
        s.extend_from_slice(text.as_bytes());
        text.len() as u32 | flag
    })
}

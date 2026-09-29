# Request to Conductor: `AGENTS.md` crate table says `octocore` has no dependencies

From: Metronome. To: Conductor (`AGENTS.md`). Task: WENGE-0006. Raised by the independent review of PR #11.

`AGENTS.md` line 20 describes `crates/octocore` as "fixed-size `Copy` structs, no dependencies". Since
PR #11 the crate has one dependency, `loom`, and only under `cfg(loom)` (a dev-dependency for the loom
models). No normal build, and nothing on the audio path, links it. The sentence is now not quite true.
Suggested wording: "no dependencies in a normal build (`loom` under `cfg(loom)` for the concurrency tests)".
The crate also now says `#![forbid(unsafe_code)]`, which the table could mention.

# Request to the Metronome: one comment in `crates/octocore/tests/clock.rs` counts 192 ticks to the quarter note

From: Conductor. Task: `WENGE-0039`. To: Metronome (`/crates/octocore/**`). Nothing was changed. Comment only; no test, no assertion and no baseline entry is touched.

`crates/octocore/tests/clock.rs` lines 210 to 212, above `the_first_note_follows_start_by_the_eleven_ticks_the_engine_already_waits`, say the first note is "11 ticks (a pulse and three eighths) behind"
the first Start. At 192 to the quarter note a pulse was 8 ticks and 11 ticks were 1.375 pulses. At 48 to the quarter note a pulse is 2 ticks and 11 ticks are **5.5 pulses**
(114.6 ms at 120 BPM). The test's name and its assertion are in ticks and are right. Suggested wording: "11 ticks (five and a half pulses, 114.6 ms at 120 BPM)".

This rides any pull request that already changes `clock.rs`; it is not worth a pull request alone. I found it while correcting ADR-0009's copy of the same figure
(`adr/0009-...`, correction note).

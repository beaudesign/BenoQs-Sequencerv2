# Request to Scribe: bring `docs/03` and `docs/02` in line with the engine

From: Metronome. To: Scribe (`docs/**`). Tasks: WENGE-0003, WENGE-0005, WENGE-0010.
Raised by the independent review of the stack (finding 6). This adds to the earlier request
`2026-09-29-docs-03-transport-behaviour.md`, which is about Stop, Reset and Play.

The engine changed and these statements are now false or incomplete:

1. `docs/03-sequencer-core.md` lines 186 to 190 show `Event` with three variants. It has five.
   `PitchBend { port, ch, value: u16, at_sample }` (14-bit) and
   `ChannelPressure { port, ch, value: u8, at_sample }` were appended after `Cc`, so the
   existing tags 0 to 2 are unchanged. `Event` is still 12 bytes.
2. The same section should say how events are ordered and timed:
   - Events are ordered by whole sample, then NoteOff before controller before NoteOn, then
     by the order they were scheduled in.
   - The engine steps ticks `MAX_EARLY_TICKS` (12) ahead, so a note pulled early by STA is
     still in time. Consequence: a command (Play, Stop, a parameter change) can take effect
     up to 12 ticks later than it is sent, and a Stop can skip up to one step.
   - Event times do not depend on the host's buffer size. This is invariant-tested at
     buffer sizes 7 to 4,096.
   - A note is queued as a NoteOn and NoteOff pair, or refused whole. Events that do not fit
     the caller's buffer wait for the next call. `Diagnostics` counts refused, deferred and
     late events.
3. A note with velocity 0 is not transmitted (manual p.40). A bender track sends a real
   Pitch Bend whose 14-bit value is the step's MCC value shifted left 7 (64 is centre 8192),
   and a channel-pressure track sends Channel Pressure (p.46, p.91). Source of truth for the
   choices: `tests/conformance/AMBIGUITIES.md`.
4. `docs/02-architecture.md` lines 216 and 220 were already out of date before this work
   (review finding 17). Line 216 says random draws happen in "track index ascending" order,
   and the engine steps tracks 9 down to 0, as the manual's effector note says. Line 220 says
   there is no floating-point accumulation for musical position, and the engine still
   accumulates a float clock. The second is what O4 (`WENGE-0004`, not approved) would build,
   so mark it as target design. The first is a plain correction.

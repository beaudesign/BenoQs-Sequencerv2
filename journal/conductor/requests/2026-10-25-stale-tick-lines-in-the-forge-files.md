# Request to the Forge: four lines in `apps/web/` that still count 192 ticks to the quarter note

From: Conductor. Task: `WENGE-0039`. To: Forge (`/apps/web/**`). Nothing in `apps/web/` was changed. Not urgent; none of these changes behaviour. They matter because
SPEC-0008 (the hardware and live MIDI plan, draft #84) quotes this folder, and a reader who believes "1.5 pulses" will compute every timing in it wrongly.

**The fact.** `TICKS_PER_QUARTER` is 48 (D0, decided by default 2026-10-07, #46), so `TICKS_PER_CLOCK` is 2 (`crates/octocore/src/domain.rs`, `apps/web/src/abi.ts`). A step is 12 ticks, which is
**6 pulses**, a sixteenth note, and **125 ms at 120 BPM**. A Clock is on every second tick. The numbers below were recomputed on `main` at `d0d1992`.

| File and line | It says | It should say |
|---|---|---|
| `ABLETON.md` 82 to 83 | a step is 12 ticks, 1.5 pulses, so 31.25 ms at 120 BPM and a fifth is 6.25 ms | a step is 12 ticks, 6 pulses, 125 ms at 120 BPM; a fifth is 25 ms (but see the question below) |
| `ABLETON.md` 133 to 134 | "D0, the length of a tick, is open"; a step "is 1.5 pulses" | D0 is decided (48 to the quarter, 2026-10-07); a step is 6 pulses, a sixteenth. The sentence that nothing here claims a pattern is in time with a drum machine can stay until a bench run (SPEC-0008 B6, B7) has measured it |
| `README.md` 133 | "a Clock every 8 ticks, which is 24 to the quarter note" | a Clock every 2 ticks, which is 24 to the quarter note |
| `README.md` 156 to 157 | a step "is 12 ticks, which is 1.5 pulses"; whether it is in time with a drum machine "is decision D0" | 6 pulses; D0 is decided. Whether the pattern is on the machine's grid also depends on the first note, which sounds 11 ticks after Start (R0b, undecided) |

**One of these is not a typo, and the Forge must not pick the answer.** `ABLETON.md` section 7 states a *proposed* follow criterion, "within a fifth of a step", and then converts it to
milliseconds. The fraction survives the new tick; the millisecond figure is derived and is now four times larger (6.25 ms became 25 ms). Whether the owner meant the fifth of a step or the
6.25 ms is the owner's to say, and a threshold is only loosened by an ADR with an expiry (CLAUDE.md, rule 3). Change the arithmetic if you must, but keep the sentence "the owner's to
set, and not a gate", and write both figures with the question beside them. The Conductor puts the question to the owner in the pull request that carries this request; SPEC-0008 does not ask it (its bench proposes no threshold).

**Also found, not asked for here.** `README.md` line 12 says the worklet posts the LED frame "every 8 blocks"; `protocol.ts` has `PANEL_EVERY_BLOCKS` = 8, so that one is current.
The `reference/manual/INDEX.md` entry `midi` (pages 109 to 112; the chapter is pages 93 and 94) was requested on 2026-09-30 (`2026-09-30-fix-manual-index.md`, addressed to the Scribe).
`docs/08` section 3 gives `/reference/**` to the Panelwright, so that request may need re-addressing. It is still open and I did not write a second one.

**When.** Any time before SPEC-0008 H3 or H4 (the first slices that quote these numbers). A single small Forge pull request, with no test change, is enough.

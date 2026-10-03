# Request to Metronome: the theory note's engine properties as tests

From: the Conductor (no task number; the owner's "Continue", Sat 2026-10-03 18:23 London, after the Ableton MCP review and its theory note were merged, #34).
To: Metronome (`crates/octocore/`). Source: `specs/SPEC-0002/sequencing-theory.md`, section 2.3 and the table in section 6, whose "proposed" rows say each needs
a request to the Metronome. **Nothing in `crates/octocore/src/` is to change.** These are tests of what the engine already does, written from theory.

## What is asked

| # | Test | Oracle (written by hand, not computed by any code here) | Why it matters |
|---|---|---|---|
| 1 | Shift the tonic triad (C E G) and the tonic seventh chord (C E G B) by 0 2 4 5 7 9 11 semitones and force each to C major: the result is the diatonic triad and seventh chord of that degree (C E G, D F A, E G B, F A C, G B D, A C E, B D F; Cmaj7, Dm7, Em7, Fmaj7, G7, Am7, Bm7b5) | The seven diatonic chords of C major, spelled by letter | This is what makes parallel chromatic shifting sound diatonic in a major key. It rests on ties going down (below) |
| 2 | The same through the engine: a feeder track's pitch offset, summed with a listener's pitch and then forced to a C major page scale, plays that chord on the listeners | The same table | Fails if the scale is applied before the offsets are summed, if a listener ignores its feeder, if the feeder's offset is not added, or if the page scale is not applied |
| 3 | Every pitch class halfway between two scale tones goes to the tone below: C major's five (C# D# F# G# A#), the major pentatonic's three, the whole-tone scale's six; in two octaves | Arithmetic: a tie is a pitch with a scale tone one semitone under it and one over it | The engine says "matching v1"; **the manual does not say what the Octopus does** (Q-M7). The test says what this engine does and why it matters musically |
| 4 | Where there is no tie the nearer tone wins, whichever side it is on, and a pitch in the scale is not moved | The distances in C D E G A | Pins the rest of the rule, so that "ties go down" is not satisfied by a quantiser that only ever looks down |

## Not asked, and why

- **Euclidean patterns:** the engine has no generator (Q-M5 is at its default, no).
- **The whole-bar swing statement** (a swing of s % starts the even steps (s/100 - 1/2) x 24 ticks late): needs the STA tables read against the claim, and D0 changes what the tick is. Left in the note's table as not asserted.
- **Brownian drift** (+1/3 of a step a step): a seeded run with a binomial bound; needs the direction code read first. Not asserted.
- **Loops of a and b steps realign after lcm(a, b):** follows from `crates/octocore/tests/o4_steps.rs` (the k-th step of any multiplier fires on tick ceil(k x 12 x den / num)); no separate test.
- **The chord path** (`extra_pit` in `engine.rs`, which forces chord notes through the same function): not covered by test 2, which uses one note per track.

## Status (2026-10-03): done, tests only

Rows 1 to 4 are `crates/octocore/tests/theory.rs` (four tests). They pass on the engine as it is, as tests of existing behaviour do, so they were not red first; each was
shown to fail on a deliberate breakage instead (`handoffs/evidence/theory-tests-mutation.txt`: 9 of 9 caught). The assertion baseline grows by those four tests (503 to 507).

# SPEC-0002 P6a: the tick length (D0 decided by default)

| Field | Value |
|---|---|
| Task | `WENGE-0012` (D0, filed in O4, never decided), raised to blocking by the owner on Wed 2026-10-07 18:03 London: "Not a sequencer that doesn't even run" |
| Spec | `SPEC-0001/o4-release-plan.md` D0 (the evidence) and `SPEC-0002/p6-runs-when-opened.md` (what this unblocks; written with the next slice, not in this pull request) |
| Plan revision | r0, 2026-10-07 |
| Tier | **High (timing)**, Metronome hat. One constant changes; every timing test and golden moves with it. Nothing in `contracts/` is touched |
| Decision | **Default, not asked:** the manual is right, 192 ticks to the *whole* note. The owner's merge of this pull request is the approval; closing it unmerged is the refusal |

## 1. What was wrong

The engine counted 192 ticks to the **quarter** note (`TICKS_PER_QUARTER = 192`). A default step is 12 ticks. The manual says a step is 12/192 of a note, 1/16 (p.15), a whole note is 192/192 (p.16)
and a measure is 16 steps at x1 speed (p.67). So the engine played a step in 31.25 ms at 120 BPM where the manual's is 125 ms, and a 16-step bar in 0.5 s where it is 2 s: **four times too fast at
every tempo**, which is the first thing anyone hears. It was found while planning O4 (D0) and left open for the owner; nobody answered, and the page that was shown on 2026-10-07 played its pattern as a blur.

## 2. The change

`TICKS_PER_QUARTER` is 48 (`crates/octocore/src/domain.rs`). `TICKS_PER_CLOCK` is `TICKS_PER_QUARTER / 24`, so 2; `DEFAULT_STEP_TICKS` stays 12 and is now six clock pulses. Samples a tick at 120 BPM and 48 kHz
are 500 (were 125). Nothing else in the engine changes: every other length is a count of ticks and keeps its meaning (a hyperstep is 192 ticks, one whole note, as the manual says).

## 3. How I know it worked

1. `crates/octocore/tests/tick_length.rs`, red first (commit `6418a56`: five tests fail with 31 ms where the manual says 125), states the manual's numbers as numbers, not read back from the constants:
   a step is 6000 samples at 120 BPM and 48 kHz, 12,000 at 60, 3000 at 240; 16 steps are 96,000 samples; a note is no longer than a step; six clock pulses lie between two steps.
2. **The change is a pure rescale.** `handoffs/evidence/tick-length/compare.py` runs the old and new `octorun` on the five example patterns: 1,697 events, the same notes in the same order, every time
   four times the old to within 3 samples (`tick-scale-proof.txt`). The examples' `render` lengths are four times longer so each covers the same music, and their goldens are re-derived.
3. `cargo test --workspace`, `npm test` (339), `npm run test:browser` (60), `just verify` all pass; the assertion baseline grows by 7 and no entry leaves it.

## 4. Tests that had to move, and how

Every test that wrote a time in samples or ticks for a 12-tick step moved by a factor of four; no tolerance was widened and no assertion removed.
Where a test counted on the clock (24 pulses to a quarter) it did not move. The names stay. **One baseline test name says "8 until D0 is answered"** (`clock::a_clock_pulse_is_ticks_per_quarter_over_24_ticks_which_is_8_until_d0_is_answered`):
the ratchet does not allow a rename without an ADR, so the name is kept, its body keeps the part that is still true, and `a_clock_pulse_is_two_ticks_since_d0_...` is added beside it. The request
`journal/metronome/requests/2026-10-07-d0-decided-by-default.md` asks the Conductor to remove the old name by ADR.

## 5. Consequences, stated so nothing is a surprise

- **The engine now steps up to `MAX_EARLY_TICKS` = 12 ticks ahead of the audio, which is 6000 samples: 125 ms at 120 BPM** (it was 31 ms). That is one step. A command that lands while a note is already queued is
  heard up to 125 ms later at 120 BPM (250 ms at 60), the price recorded in ADR-0009 amendment 3 scaled by four. It is not a bug to fix here; shortening the lookahead is its own decision.
- The MIDI clock is unchanged in time (24 pulses to a quarter at the tempo); a pulse is 2 ticks and a step is now 6 pulses, which is why an external drum machine and this sequencer agree about a sixteenth.
- **The first step of a Play still sounds 11 ticks after the Play** (the step clock starts at phase 0), now 62.5 ms at 120 BPM instead of 15.6 ms. That is R0b, a separate change that moves the goldens a second
  time, so it is parked on a patch kept outside this pull request and not in this pull request.
- The follower (M4/M5 simulations) is unit-free in ticks except where it names `TICKS_PER_CLOCK`; its tests moved with the constant (41 of 41 pass).
- `docs/02` §5 ("192 PPQN"), `docs/03` lines 18, 43 and 220, `AGENTS.md`, `SPEC.md` and `tech.md` still say 192 to the quarter. They are the Scribe's and the Conductor's; the request lists them.

## 6. What the owner decides

Merge it, or tell me why the engine was right (for example that a real Octopus at 120 BPM plays 16 steps at x1 in 0.5 s). The one hardware check that settles it: at 120 BPM, does a 16-step pattern
at x1 with default step lengths take 2 s or 0.5 s?

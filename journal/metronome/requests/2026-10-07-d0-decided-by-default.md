# Request to Conductor: D0 is decided by default (the manual's 192 to the whole note)

From: Metronome (`WENGE-0012`, `specs/SPEC-0002/p6a-tick-length.md`). To: Conductor (`journal/STATE.md`, `docs/10-risks-and-decisions.md`, `harness/baseline.txt`) and Scribe (`docs/02`, `docs/03`, `AGENTS.md`, `SPEC.md`).
Tier of the change: **High (timing)**; no contract is changed.

## What happened

D0 was filed during O4 and never answered. On Wed 2026-10-07 the owner reviewed the running page and wrote that the project "doesn't even run". The measured cause of the blur was D0: every step played four times too fast. The
default taken is **the manual is right** (192 ticks to the whole note, p.15, p.16, p.67), built on `metronome/d0-tick-length`. It is a draft pull request; the owner's merge is the approval, and closing it is the refusal.

## What to record, when it merges

1. **`WENGE-0012`**: decided by default, built. `specs/SPEC-0001/README.md` line 98 ("Not approved and not specified. The engine is untouched") is no longer true.
2. **`journal/STATE.md`**: a step is 125 ms at 120 BPM; the engine steps 12 ticks (125 ms) ahead of the audio, so a command can be heard up to one step later; the first note of a Play is 11 ticks (62.5 ms) after the Play
   (R0b, not yet built).
3. **"192 PPQN" in the docs**, which is 192 to the *quarter*: `docs/02-architecture.md` line 179 (it states D0 as open), `docs/03-sequencer-core.md` lines 18 and 43 (the row says "PPQN 192" and
   "12 ticks = 1/16" in one line, which only fit if 192 is a whole note, so the second half was always right), `AGENTS.md` line 22, `specs/SPEC-0001/tech.md` line 9, and `specs/SPEC-0002/tech.md` line 79.
   "30 minutes at 120 BPM is 691,200 ticks" is 172,800 now. The engine's own comments and `tests/conformance/AMBIGUITIES.md` are corrected in the pull request (they are the Metronome's).
4. **A baseline removal by ADR**: `test clock::a_clock_pulse_is_ticks_per_quarter_over_24_ticks_which_is_8_until_d0_is_answered` keeps its name so the gate stays green (the ratchet forbids a rename without an ADR); its name is now
   false. `a_clock_pulse_is_two_ticks_since_d0_the_manual_counts_192_to_the_whole_note` is its successor and is in the baseline.
5. **R3 (a hardware-grade look)** needs an ADR overturning SPEC-0002 D3 and the no-gradient / no-colour rules, which is yours. I will draft it and attach it; the owner pointed at `roland50.studio` as the standard.

## Why a request and not an edit

`docs/`, `journal/STATE.md`, `AGENTS.md`, `SPEC.md` and `harness/baseline.txt` removals are not mine (`docs/08` §3).

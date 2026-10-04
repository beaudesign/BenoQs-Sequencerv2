# Scorecard, Sun 2026-10-04

Asked by the owner at 11:59 London: "Test them and review out of 10". "Them" is read as the four things the owner
asked for at 11:07 (plan `specs/SPEC-0002/p5-plan.md`, section 10) and the one thing built this week, the P5b knobs.
What was tested is the head of #40 (main at 31d9c02, plus P5b and the plan). Every number below was produced by
running something, and the output is in this folder.

| What | Score | Why |
|---|---|---|
| Save and load of sequences and MIDI data (line 1) | **0/10** | Nothing stores a sequence. `apps/web` uses no browser storage (`localStorage`, IndexedDB: none). The only writer of a Standard MIDI File is the headless runner. Planned on its own, not built |
| Stop then Play carries on from where it stopped (line 2) | **5/10** | **Good:** no notes while stopped (0), none repeated, `Continue` and not `Start` goes out, the stream balances. **Bad:** every Stop loses the note due in the next 12 ticks, and Play resumes after it. 12 of 12 stop positions through the wasm, 100% of positions with every step on, 25% (10 of 40) with notes four steps apart, at 60, 120 and 240 BPM and at three buffer sizes |
| The tempo knob sets the tempo and the MIDI speed follows (line 3) | **3/10** | **Engine, 9/10:** at 60, 120 and 240 BPM the gap between steps is 3000, 1500 and 750 samples, a clean halving to the sample. **Nothing to turn, 0/10:** no tempo control exists on the page; the tempo is 120 unless a followed clock sets it (P5e). **And** a step at 120 BPM lasts 31.25 ms, not the 125 ms of a 16th note (D0) |
| A metronome click, on or off (line 4, reading b) | **0/10** | Nothing. No oscillator or buffer source exists in `apps/web`; `AudioContext` is used only to host the worklet. Reading (a), that the tick is not a decision, has nothing to test |
| The P5b knobs, VEL PIT LEN STA in Step zoom | **8/10** | Eight of eight cases right end to end through the shipped wasm (`p5b-knobs-wasm-output.txt`): PIT +3 gives a note 3 higher, VEL +10 gives velocity 110, LEN +12 ticks doubles the length, STA +3 delays the note by 3 ticks and +9 stops at 5. Ten fixtures, 20 of 20 mutants caught. Docked: no knob on the page (P5c) and no value display (Q06), so a person cannot use them yet |
| The test suite | **8/10** | 521 native assertions (463 tests and 58 fixtures), 313 Node tests and 55 Chromium tests, all green (`chromium-summary.txt`). Docked: it did not catch the Stop loss, because the one test around Stop then Play checks the clock message and the pulse grid and not the notes; and 6 of the 10 gates are placeholders (timing, a11y, arch, slop, tokens, persistence) |
| **Overall, as an instrument a person can play today** | **5/10** | A sound engine and a controller built on fixtures, but a person cannot yet set a tempo, turn a knob, see a value, save a pattern, or stop without losing a note |

## Opportunities, in the order they pay

1. **Exact resume after Stop** (Metronome, High timing; request filed: `journal/metronome/requests/2026-10-04-exact-resume-after-stop.md`). The owner's ruling needs it, and the repo already names the cause and two fixes (`tests/conformance/AMBIGUITIES.md`, "lookahead, Stop and commands"). The acceptance test is `headless.py` section C: 0 lost notes at every stop position. It is red today.
2. **Decide D0.** At 120 BPM the engine plays 16 steps in 0.5 s where a DAW's 16ths take 2 s. Everything the owner will hear when the tempo knob exists depends on it, and the fourth line of the 11:07 answer may bear on it.
3. **Merge #40, then P5c (the knobs on the page) and P5e (a tempo control).** Until then the four things the owner asked for have nothing a person can touch.
4. **Make the sweep a gated test** (it becomes W4's test in P5d, red first), and give the timing gate a first real test.
5. **The save and load plan**, from the owner's requirement: the sequencer's own types, with the MIDI data and the sequences. `octocore` has no serde today.
6. **The click**, only if reading (b) was meant.

## Reproduce

```
cargo build -p octorun
OCTORUN=target/debug/octorun python3 handoffs/evidence/scorecard-2026-10-04/headless.py   # headless-output.txt
just web-wasm
node handoffs/evidence/scorecard-2026-10-04/p5b-knobs-wasm.ts                              # p5b-knobs-wasm-output.txt
node handoffs/evidence/scorecard-2026-10-04/stop-resume-wasm.ts                            # stop-resume-wasm-output.txt
cd apps/web && npm run test:browser                                                         # chromium-summary.txt
```

These are probes, not gated tests: they are not in `harness/baseline.txt` and nothing runs them in CI.

# Evidence: Play after a Stop carries on from the point the sound stopped (WENGE-0016, ADR-0009 amendment 3)

Probes, not gated tests. The gated test is `crates/octocore/tests/resume.rs`. What these add is the same property seen from outside the crate: through the headless
runner and through the shipped wasm module, the way the page drives it.

| File | What it is | Before the change | After |
|---|---|---|---|
| `headless.py`, `headless-output.txt` | `octorun` patterns. Section C sweeps where a Stop lands, for a dense and a sparse pattern at 60, 120 and 240 BPM | 100% of stop positions lost a note with every step on, 25 to 28% with notes four steps apart (`scorecard-2026-10-04/headless-output.txt` in #40) | **0 of 243 positions lose a note** |
| `stop-resume-wasm.ts`, `stop-resume-wasm-output.txt` | The shipped `octoweb.wasm`, 128-frame blocks, `transport(false)` then `transport(true)`, 24 stop positions per tempo, a gap of 40 blocks. The Note Ons after the Play must be the uninterrupted run's from the Stop on, moved by the gap | `stop-resume-wasm-output-before.txt`: 92 of 94 stop positions differ (built from the engine before the change) | **0 of 94** |

Run: `cargo build -p octorun && OCTORUN=target/debug/octorun python3 handoffs/evidence/exact-resume/headless.py`, and after `just web-wasm`,
`node handoffs/evidence/exact-resume/stop-resume-wasm.ts` (`OCTOWEB_WASM=<path>` points it at another build).

The mutation run for the change is `../exact-resume-mutate.py` and `../exact-resume-mutation-run.txt`.

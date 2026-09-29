# O4 baseline measurements (WENGE-0004), taken on the clock as it is on `metronome/command-ring`

Nothing here is O4 code. These are throwaway programs that drive the *current* `Engine::render`
and measure how far its note times land from the analytic answer, plus one pure-arithmetic model
of the clock O4 proposes. They are kept so that every number in `specs/SPEC-0001/o4-release-plan.md`
can be reproduced, and so that the guard tests O4 adds have a written baseline to be compared with.

| File | What it measures | Output |
|---|---|---|
| `baseline_ab_d.rs` | (A) constant tempo, note onsets against the ideal grid. (B) a 60 to 180 BPM ramp over 8 bars. (D) first tick where the `f32` step accumulator differs from exact arithmetic | `../o4-baseline-ab-d.txt` |
| `baseline_more.rs` | (C) lateness after a ramp ends. (S) whether reporting the tempo earlier removes the lateness (tests the cause). (F) when the `f32` accumulator fires a step, against the exact tick, at three horizons | `../o4-baseline-c-s-f.txt` |
| `model_tick_timeline.py` | The proposed arithmetic (integer tick positions, turned into samples only when the buffer holding them is rendered) on the same ramp, with and without a host position | `../o4-model.txt` |
| `golden_floor_flips.py` | (Q) How many tick times of each golden pattern land on a different whole sample under `floor(k * spt + 1e-9)` than under the engine's running sum. Pure arithmetic in `f64`, no engine code | `../o4-golden-floor.txt` |

## How to run

The Rust files are not part of the workspace. To rerun them:

```
cargo new --bin o4base && cd o4base
# Cargo.toml: octocore = { path = "<repo>/crates/octocore" }, and an empty [workspace] table
cp <repo>/handoffs/evidence/o4-baseline/baseline_ab_d.rs src/main.rs
mkdir -p src/bin && cp <repo>/handoffs/evidence/o4-baseline/baseline_more.rs src/bin/more.rs
cargo run --release                 # A, B, D
cargo run --release --bin more C    # or S, or F <ticks>
python3 <repo>/handoffs/evidence/o4-baseline/model_tick_timeline.py
python3 <repo>/handoffs/evidence/o4-baseline/golden_floor_flips.py
```

Part F over 10^8 ticks takes a few minutes. The results are deterministic: no wall clock is read.

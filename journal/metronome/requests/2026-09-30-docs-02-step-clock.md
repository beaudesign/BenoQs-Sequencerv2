# Request to Scribe: the per-track step phase is an integer now

From: Metronome. To: Scribe (`docs/02-architecture.md`). Task: WENGE-0004, PR 4a
(`crates/octocore/src/steps.rs`).

1. Line 192, the runtime block: `per-track { accum, pos, ping_dir, chain_state }`. `accum` is
   now `step: StepClock`, an integer phase in `1/num`-tick units (`num` is the speed
   multiplier's numerator). Suggest `per-track { step_phase, pos, ping_dir, chain_state }`.
2. Lines 220 to 222, "No floating-point accumulation for musical position." It was not true of
   the step phase before 4a (it was an `f32`); it is true of it now. The tick counter itself was
   already a `u64`. After 4b the queue is keyed by tick too, and the sentence becomes true of the
   whole engine; until then say "the step phase and the tick counter".
3. A sentence for section 5 or `docs/03` section 4: a track at multiplier `num/den` starts its
   k-th step on tick `ceil(k * 12 * den / num)` (ticks counted from 1 after Play), for every
   `u8` pair, at any tick count. Source: `specs/SPEC-0001/o4-release-plan.md` section 2.7, test
   R1 in `crates/octocore/src/steps.rs` and `crates/octocore/tests/o4_steps.rs`.

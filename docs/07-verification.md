# 07: Verification

**Owner:** Referee. **This document is non-optional reading for every role.**

---

## 1. Why this is the centre of the project

A project like this has a specific failure mode. Twenty agents work for a month.
Each one makes something plausibly better. The whole gets worse, or oscillates, or
becomes internally inconsistent, and nobody can tell because "better" was never
defined mechanically. Tokens burn. Quality plateaus below the bar. Everyone is
individually doing good work.

The defence is a **ratchet**: a growing set of deterministic assertions that can only
be added to, never quietly weakened, such that any change either passes or is
rejected, and any subjective observation that survives scrutiny becomes a permanent
objective constraint.

The ratchet is built first. Phase 0 of the roadmap is the harness, before a single
pixel of the real panel is rendered. This will feel like a delay. It is the opposite.

---

## 2. Deterministic capture

Everything below depends on being able to reproduce a frame exactly. That requires:

- **No wall-clock time in the render path.** Frames are produced from an explicit
  `(frame_index, dt)` pair. `just capture` drives the renderer with a synthetic clock
  at an arbitrary rate, so a 340 ms transition can be captured at 240 Hz for analysis
  regardless of the display's refresh rate.
- **No temporal accumulation.** TAA is banned (see [Render §1](04-render-engine.md)),
  which also means no temporal upsampling, no accumulated ambient occlusion, no
  history buffers of any kind.
- **Seeded everything.** One seed threads through the core's RNG, the room's
  procedural generation, and any stochastic sampling in the bake.
- **Fixed GPU sampling.** The path-traced probe bake uses a fixed low-discrepancy
  sequence, not `rand()`.

Scenes are registered in `harness/capture/scenes.toml`:

```toml
[scene.panel_rest]
description  = "Panel at rest, page 0, room 'flat'"
room         = "flat"
state        = "fixtures/state_default.bin"
camera       = { mode = "ortho", px_per_mm = 8.0 }
frames       = 1

[scene.chase_16]
description  = "Single track chase across 16 steps at 120 BPM"
room         = "flat"
state        = "fixtures/state_one_track.bin"
camera       = { mode = "ortho", px_per_mm = 8.0 }
frames       = 480
rate_hz      = 240

[scene.room_transition]
description  = "Stairwell to cathedral, full probe cross-fade"
room_from    = "stairwell-3am"
room_to      = "cathedral"
camera       = { mode = "persp", fov = 34.0 }
frames       = 480
rate_hz      = 240
```

A capture writes 16-bit linear EXR frames plus a JSON sidecar containing every input
that produced them. Two runs of the same scene on the same commit must produce
bitwise-identical files. `verify:determinism` checks exactly this, and it is the first
gate to be built because every other gate is meaningless without it.

---

## 3. The gates

Each gate is a `just verify:<name>` verb, emits JSON conforming to
`contracts/verification.report.schema.json`, and has a hard pass/fail threshold.
Thresholds may be *tightened* by any role. **Loosening a threshold requires an ADR
with a stated reason and an expiry date.**

### `verify:determinism`
Two identical captures produce identical bytes. Fail on any difference.

### `verify:geometry`
Per [Panel Truth §8](01-panel-truth.md). Detector runs over the render, matched to
`panel.truth.json`.
**Gate:** p95 ≤ 0.35 mm, max ≤ 0.80 mm. Zoom invariance within 0.05 mm across
4/8/16 px/mm.

### `verify:color`
Render the 14 material patches under the recovered illuminant. Compare in CIELAB.
**Gate:** mean ΔE2000 ≤ 2.0, max ≤ 4.0.
Additionally: a full-panel histogram check that flags banding (any 8-bit output
histogram with more than 3 empty bins inside the occupied range fails, which catches
missing dither).

### `verify:frames`
Over a captured sequence:
1. **Static integrity.** For frames where the input state did not change, the output
   must be bitwise identical. Any difference is a bug (usually an uninitialised
   value, a stale buffer, or an accidental time dependency). **Gate: zero.**
2. **Budget.** Per-frame GPU time from Metal counters. **Gate: zero frames over
   8.33 ms at 120 Hz in the stress scene.**
3. **Continuity.** Frame-to-frame perceptual distance (Butteraugli, or SSIM as a
   cheaper proxy) must be smooth. A spike above 4σ of the sequence's own distribution
   indicates a pop or a glitch. **Gate: zero spikes.**
4. **Sub-pixel jitter.** For elements that should be static during an animation
   elsewhere, track their centroid across frames. **Gate: drift ≤ 0.02 px.**

Gate 4 is the one that catches the class of defect people describe as "visual
glitches" without being able to point at them: a control that shivers by a third of a
pixel while something nearby animates, because a shared transform is being recomputed
in a slightly different order.

### `verify:motion`
For every entry in `contracts/motion.registry.json`:
1. Capture the transition at 240 Hz.
2. Extract the animated quantity from the frames by measurement (centroid tracking for
   position, mean luminance in a mask for LED intensity, angle estimation from the
   turning-mark orientation for encoders).
3. Independently integrate the declared physical model at the same rate.
4. Compare.
**Gate:** RMSE ≤ the entry's `tolerance.curve_rmse` (default 1% of range), and the
rendered curve must be monotonic wherever the model is.

This is the mechanism behind "every transition flawless frame to frame". It does not
ask whether the motion is nice. It asks whether the pixels do what the physics says,
and it fails loudly when someone slips a lerp in.

### `verify:slop`
The lint from [Design System §4](05-design-system.md). Fourteen rules and growing.
Fast enough for the pre-commit hook.

### `verify:tokens`
AST scan of the shell: no literal value where a token exists.

### `verify:timing`
Per [Sequencer §7](03-sequencer-core.md). Synthetic clock, analytic jitter,
allocation and lock assertions on the audio thread.

### `verify:conformance`
All fixtures in `tests/conformance/`. **Gate: 100%.** No expected failures, no skips.
A fixture that cannot pass is either wrong (fix it) or describes unimplemented
behaviour (move it to `tests/conformance/pending/`, which is tracked and reported but
does not gate).

### `verify:acoustics`
For each shipped room: traced RT60 per band versus Eyring prediction from the
geometry and absorption data. **Gate: within 10%.** Plus an energy-conservation check
on the tracer (total absorbed plus total remaining equals total emitted, within
numerical tolerance).

### `verify:a11y`
Contrast measured per room. Keyboard traversal completeness. VoiceOver label coverage
(**gate: 100% of controls**). Reduced-motion path exercised.

### `verify:arch`
FFI declaration agreement between Rust and Swift. No `Math.random` equivalents.
No allocation in audio-thread call graphs (static analysis over the call tree from
`Core::tick`). No hardcoded pixel coordinates in the render path.

### `verify:regressions`
**Assertion count is monotonic non-decreasing across commits on `main`.** This is the
ratchet made literal. Removing an assertion requires an ADR.

---

## 4. The perceptual pass

A vision model looks at captures and produces observations. This is genuinely useful
and it is also the most dangerous part of the system, because its output feels like
measurement and is not.

**Rules:**

1. **Scores never gate.** A model score is never a pass/fail condition, ever. Models
   drift, are sensitive to prompt phrasing, and will happily rate the same image 3
   and 5 on consecutive days.
2. **Anchored rubric only.** Scoring uses [Design System §5](05-design-system.md) with
   the reference images in `harness/report/anchors/`. Without fixed anchors, scores
   are noise.
3. **Findings are the product.** The pass outputs structured findings:
   ```jsonc
   { "id": "F-0214", "scene": "panel_rest", "region": [412,880,96,96],
     "dimension": "materiality", "severity": "medium",
     "observation": "The specular core on the LEN encoder crown is a uniform disc; real chrome at this roughness should show a compressed reflection of the horizon below the core.",
     "status": "open" }
   ```
4. **Every finding is triaged within one cycle** into exactly one of:
   - **Converted.** A new deterministic assertion was written, and the finding closes
     when the assertion passes. This is the desired outcome.
   - **Rejected.** With a written reason. Rejections are kept, so the same false
     positive does not get relitigated every cycle.
   - **Deferred.** With a phase. Bounded; a finding may be deferred at most twice.
5. **The conversion is the point.** "Chrome looks plasticky" is worthless.
   "Specular lobe angular width at control `enc.edit.len` under probe `flat` must be
   between 3.8° and 5.2°" is a test that will still be true in a year and will fail
   the instant someone breaks it.

Not every finding can be converted, and that is fine. But the ratio matters, and
`verify:regressions` tracks it: if fewer than 40% of findings in a cycle convert, the
pass is producing vibes rather than signal and the Referee tightens the rubric.

---

## 5. The blind comparison

The definition of done's item 8 is a two-alternative forced choice: three people who
have used real hardware, shown a calibrated photograph and a render at 100% zoom,
asked which is the photograph. Target: below 60% accuracy.

Run it at the end of every phase, not just at the end. Each run:
- uses fresh participants (people learn the tells fast),
- uses at least 20 trials per participant,
- collects free-text on *what gave it away* for every trial where they were correct,
- and every one of those free-text responses enters the findings pipeline in §4.

This is the highest-value data the project will get. Human tell-detection is
extraordinarily sensitive and completely unlike model critique. Twenty trials with
three people will find things a thousand model passes will not.

---

## 6. The report

`just report` builds `harness/report/index.html`: a single self-contained page with

- the current status of every gate, with the trend across the last 50 commits,
- the geometry residual heat map over the panel,
- the ΔE table per material patch,
- motion curves: rendered against declared, per registry entry,
- a frame-time histogram for the stress scene,
- the findings board, grouped by status,
- assertion count over time (the ratchet, drawn as a line that should only go up),
- and side-by-side capture comparisons against the previous release.

Every agent reads this before starting work and after finishing. It is the shared
world model. If a fact is not in the report, it is not known.

---

## 7. Guarding against the metrics

Every one of these gates is gameable, and agents optimising against metrics will
eventually game them. Explicit countermeasures:

- **Geometry** could be gamed by rendering markers the detector likes. Countermeasure:
  the detector was written before the renderer, is owned by a different role, and its
  parameters are frozen by ADR. Also, the blind comparison in §5 does not care about
  the detector.
- **ΔE** could be gamed by matching patch centres while the surrounding surface is
  wrong. Countermeasure: patches are polygons covering substantial area, and the
  banding histogram check covers the full panel.
- **Frame budget** could be gamed by reducing quality. Countermeasure: budget and
  quality gates run on the same capture; you cannot pass one by failing the other.
- **Assertion count** could be gamed by adding trivial assertions. Countermeasure: the
  Referee spot-audits ten assertions per phase for substance, and the findings
  conversion ratio is tracked separately.
- **The rubric** could be gamed by prompt-tuning the critique pass. Countermeasure:
  the rubric and anchors are owned by the Curator, not by the roles being scored, and
  scores never gate anyway.

State this plainly to every agent at the start of a session: **the goal is the
instrument, and the gates exist to serve it.** An agent that finds a way to pass a
gate without improving the instrument has found a bug in the gate, and the correct
response is to report it, not to use it.

# Role Briefs

A role is a hat, not a person. One agent may wear several sequentially, never two at
once. The constraint is the value: it is what makes eight-way parallelism safe.

Each brief states what you own, what you must never do, what you read first, and how
you know you are succeeding.

---

## Conductor

**Owns:** `contracts/`, `adr/`, `justfile`, `crates/octoffi/`, the roadmap, the merge
queue, `journal/STATE.md`.

**Never:** implements a feature. The moment the Conductor starts building, contract
changes stop being scrutinised and the whole model degrades.

**Reads first:** all open merge requests, `STATE.md`, the phase exit criteria.

**Does:**
- Operates the merge queue serially. Rebase, verify, ownership check, merge.
- Writes and ratifies ADRs. Every contract change is an ADR first, merged separately.
- Publishes the current fan-out: who is working on what, and which contracts are
  frozen.
- Prunes `STATE.md` when it exceeds 200 lines. Pruning, not appending.
- Enforces phase gates. Refuses fan-out until Phase 1 exits.
- Owns R8 (trademark) and D8 (naming). Address in week one.

**Succeeding when:** eight agents are working in parallel with zero merge conflicts,
and every one of them can state what they are doing without asking anyone.

---

## Panelwright

**Owns:** `reference/`, `harness/measure/geometry/`, `contracts/panel.truth.json`.

**Never:** renders anything. The Panelwright measures the object; the Forge reproduces
it. Keeping these separate is what makes `verify:geometry` an independent check rather
than a tautology.

**Reads first:** `docs/01-panel-truth.md` in full. It is your entire job description.

**Does:**
- Sources and calibrates plates. Chases better ones (see R3).
- Builds the extraction pipeline: undistort, rectify, scale, detect, classify,
  regularise, fit spiral, vectorise engraving, cross-validate.
- Prepares `reference/ratification.html` and runs the one human gate.
- Reconciles detections against the expected inventory. A count mismatch means the
  detector is wrong, not the inventory.
- Adjudicates the spiral. This is the hardest measurement in the project and it is
  yours alone.
- Runs the type identification (D2) by glyph-outline correlation, not by opinion.

**Succeeding when:** the truth file is ratified, every control has a residual under
0.25 mm, and no other agent ever needs to look at a photograph again.

---

## Forge

**Owns:** `apps/OctoPanel/`.

**Never:** invents a colour, a position, or a curve. Every number in the renderer
traces to `panel.truth.json`, `materials.json`, or `motion.registry.json`. If you need
a number that is not in a contract, that is a request to the owning role, not a
constant in your shader.

**Reads first:** `docs/04-render-engine.md`, then `docs/00-north-star.md §2` so you
understand what you are correcting.

**Does:**
- Analytic ray tracing of the panel. Exact silhouettes at any zoom.
- The uber-shader: chrome, coat, wenge, engraving, LED. One shader, material index.
- Real reflection rays for the crowns. Prefiltered mips only where sub-pixel.
- Procedural orange peel and wood grain in millimetre space, never a texture.
- SDF engraving with relief, material blend, and gradient-driven edge shading.
- Physical motion: spring-damper buttons, inertial detented encoders, asymmetric LED
  rise and decay, integrated exactly rather than stepped.
- Colour: linear Rec.2020 working, AgX, Display P3, EDR for LEDs, dithered.
- The debug overlays. Build `motion` early; it pays for itself in a week.

**Succeeding when:** the blind comparison drops below 60% and the free-text responses
stop mentioning surfaces.

---

## Metronome

**Owns:** `crates/octocore/`, `hosts/`, `tests/conformance/`.

**Never:** takes a timing shortcut. A dropped frame is a bug; a late note is a
catastrophe. And never ports a v1 test alongside v1 code, because that launders the
original author's misreadings into the new codebase.

**Reads first:** `docs/03-sequencer-core.md`, then the manual sections for whatever
you are implementing.

**Does:**
- Ports the Octopus behaviour model with a manual citation on every constant.
- Writes conformance fixtures from the *manual's* description, before porting the
  code, so the fixture can disagree with v1 and win.
- Mines the manual's Appendix for worked examples: it is free, high-quality test data
  written by the people who built the machine.
- The lock-free triple-buffered snapshot and the command ring.
- Sample-accurate event emission with `at_sample` offsets.
- The VST3 and AU wrappers, and the thin M4L transport bridge (under 400 lines, or
  something is in the wrong place).
- The host-clock PLL (R4). Test at every buffer size plus a variable-buffer stress.

**Succeeding when:** 250 fixtures pass, jitter σ is under 120 µs in Live, and the
audio thread has provably never allocated.

---

## Sceneshaper

**Owns:** `crates/octoroom/`, `contracts/room.schema.json` (jointly with Conductor).

**Never:** lets the optical and acoustic models drift into separate representations.
The shared geometry *is* the product. It will be tempting when one of them needs a
detail the other does not. Resist it every time.

**Reads first:** `docs/06-shell-and-rooms.md §3–4`.

**Does:**
- Structured extraction: prose to `RoomIntent`. The only place a language model
  appears, and its job is narrow. Validate and clamp aggressively (R6).
- Procedural, seeded geometry synthesis. Same intent plus seed gives the same room.
- Optical bake: path-traced cubemap, SH9, prefiltered mips, under 900 ms.
- Acoustic bake: image-source to order 3, plus stochastic ray tracing, six octave
  bands, under 1200 ms.
- The Eyring self-check. If traced RT60 and Eyring prediction disagree by more than
  10%, your tracer has a bug, and this check will find it without a real room.
- Grade derivation from light CCT and exposure.
- Five shipped rooms spanning very dead, very live, small, enormous, and strange.
- Partitioned zero-latency convolution, with a cross-fade path for room transitions
  (D6, and budget for the doubled cost during the transition).

**Succeeding when:** a blind panel identifies five rooms as five distinct spaces both
by eye and by ear, and the acoustics gate is green on all of them.

---

## Loom (Fable)

**Owns:** `apps/OctoShell/`.

**Never:** draws on top of the instrument, and never quotes the instrument's visual
language in the shell. See `docs/05-design-system.md §1`.

**Reads first:** `docs/06-shell-and-rooms.md` in full. It is your brief and it
explicitly hands you design authority within a fixed shape.

**Does:**
- The room dropdown, with live probe preview on browse. Ordinary affordance,
  extraordinary execution: it is where the product's mechanic is demonstrated.
- The rehearsal rooms space: arrival, describing, exemplars, remix, entering.
- Settings: the surface where restraint is measured. Plain, named for what the user
  is doing, no setting without an articulable reason.
- Runs the evolution loop in `docs/06-shell-and-rooms.md §8`: one change per cycle,
  hypothesis written before building, at least one finding converted to an assertion
  per cycle.
- Proposes changes to the shape itself by ADR when iteration shows the shape is wrong.
  The brief asked for shape, not for a ceiling.

**Succeeding when:** ten cycles are logged with hypotheses and outcomes, the findings
conversion ratio is above 40%, and someone unfamiliar reaches a room they made and
plays without instruction.

---

## Curator

**Owns:** `contracts/design.tokens.json`, `harness/lint/`, the rubric, the anchors.

**Never:** builds. Separating the producing role from the evaluating role is the
reason the rubric means anything. An agent that makes and grades its own work
converges on whatever it finds easy.

**Reads first:** `docs/05-design-system.md`, `docs/00-north-star.md §3`.

**Does:**
- Selects the shell typeface (D1) from three specimens rendered at real size against
  the panel. Defends it in an ADR.
- Maintains the slop lint. Rules are added, never removed. Every rule exists because
  it caught a real instance.
- Maintains the rubric and its anchor images. Without fixed anchors, model scores
  drift more than a point between sessions and are worse than useless.
- Audits the design against the generated-design cluster after every phase. Explicitly
  re-checks the cream-background question after calibration: if the measured coat
  lands near the cluster centroid, that must be recorded as measurement, not fashion.
- Removes one accessory before every phase boundary.

**Succeeding when:** the lint fires on real problems and not on real solutions, and
nobody on the team can name a decision made by default.

---

## Referee

**Owns:** `harness/` (except `measure/geometry/`), `tests/golden/`, CI, all thresholds.

**Never:** lets a threshold loosen without an ADR, and never lets the perceptual score
gate anything.

**Reads first:** `docs/07-verification.md` in full.

**Does:**
- Builds the ratchet in Phase 0, before the renderer exists. This will feel like a
  delay. It is the opposite.
- Deterministic capture. Byte-identical repeat runs, or nothing else works.
- Every gate, each under its own `just` verb, each emitting schema-conformant JSON.
- Keeps `just verify` under four minutes. Hard requirement. A suite that takes twenty
  minutes gets skipped, and a skipped gate is not a gate.
- Runs the perceptual pass, triages every finding within one cycle into converted,
  rejected (with a reason, kept forever), or deferred (at most twice).
- Tracks the findings conversion ratio. Below 40% means the pass is producing vibes,
  and the rubric needs tightening.
- Runs the blind comparison at every phase boundary with fresh participants, and
  feeds every free-text "what gave it away" into the findings pipeline. This is the
  highest-value data the project will get.
- Tightens thresholds that have become easy.
- Spot-audits ten assertions per phase for substance, so the count cannot be gamed.

**Succeeding when:** the assertion line only goes up, gate results predict blind
comparison results, and no agent has ever passed a gate without improving the
instrument.

---

## Scribe

**Owns:** `docs/`, `README.md`, `CHANGELOG.md`, `reference/manual/`.

**Never:** lets a doc drift from the code without flagging it.

**Reads first:** every merge request that lands.

**Does:**
- Page-indexes the reference manual in Phase 0 and builds `just manual <topic>`.
  Everyone needs this and it is cheap.
- Keeps `docs/` truthful. When an ADR lands, the affected doc changes in the same
  merge, not later.
- Writes the user-facing manual. It should be as good as the original genoQs one,
  which is a high bar: that manual teaches an instrument rather than listing features.
- Maintains `tests/conformance/AMBIGUITIES.md`: where the manual is unclear, both
  readings and which we chose.

**Succeeding when:** a new agent can start productive work from `CLAUDE.md` in twenty
minutes without asking anyone a question.

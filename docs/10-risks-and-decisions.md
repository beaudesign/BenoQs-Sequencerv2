# 10: Risks and Open Decisions

**Owner:** Conductor.

---

## 1. Risk register

Ordered by expected cost, which is probability times damage, not by probability.
Every risk has a named owner, a trigger that tells you it is happening, and a
response decided in advance.

### R1. Scope death
**Probability: high. Damage: total.**
The spec describes a renderer, a sequencer, an acoustic simulator, a world generator,
and a shell. Any one is a project. The failure is not that we run out of time; it is
that we build 80% of five things.

*Trigger:* Phase 1 has not exited after twice its expected duration, or work begins
in a phase whose predecessor has not exited.
*Response:* the roadmap's phase gates are hard. The Conductor refuses fan-out until
Phase 1 exits. If Phase 1 is genuinely stuck, cut room generation to authored rooms
only (see R6) and ship a replica with five hand-built spaces, which is still a
complete and desirable product.
*Owner:* Conductor.

### R2. The uncanny valley of chrome
**Probability: medium. Damage: high.**
Traced reflections could still read as wrong, because getting metal right depends on
the *environment* being right, and a procedurally generated room may not contain the
kind of structured detail (window frames, edges, gradients of occlusion) that makes a
reflection legible as a reflection.

*Trigger:* blind comparison above 80% after Phase 1, with free-text responses citing
the chrome.
*Response:* ship the `flat` room as a *photographic* HDRI of a real studio rather than
a generated space, and use generated rooms only where their character is the point.
The mechanic survives: a generated room still drives the acoustics and the grade, and
we blend a structured base probe with the generated one.
*Owner:* Forge, with Sceneshaper.

### R3. Reference plates unobtainable
**Probability: medium. Damage: medium.**
The Octopus is rare (a few hundred were made) and calibrated photography may not be
obtainable. Web imagery is retouched, uncalibrated, and low resolution.

*Trigger:* Phase 0 ends without a plate meeting the requirements in
[Panel Truth §3.1](01-panel-truth.md).
*Response:* relax the geometry tolerance to 0.60 mm p95 by ADR, derive materials from
first principles plus published data for powder coat, chrome, and wenge, and mark
`materials.json` entries `modelled` rather than `measured`. Simultaneously, ask the
genoQs community: the CE firmware exists because that community is active, and
somebody owns a machine and a camera.
*Owner:* Panelwright.

### R4. Timing under a host
**Probability: medium. Damage: high.**
A sequencer that is sample-accurate in isolation can still drift or jitter inside a
host with a variable buffer size, or when the host's transport is itself being
resampled.

*Trigger:* loopback jitter σ above 200 µs in Live at any buffer size.
*Response:* this is a solved problem but not a trivial one. Implement the standard
approach (map host PPQN to a monotonic sample timeline with a PLL, run the core on
the sample timeline, never on the callback boundary) and test at 64, 128, 256, 512,
and 1024 samples plus a deliberately variable-buffer stress test.
*Owner:* Metronome.

### R5. Frame budget on the full panel
**Probability: medium. Damage: medium.**
Tracing reflection rays for 200 primitives at 9× supersampling at 120 Hz is a
substantial pixel load at high zoom levels.

*Trigger:* any over-budget frame in the stress scene at Phase 2.
*Response:* in order of preference, and only in this order: (a) reduce supersampling
to 2× where the SDF already handles edges analytically, (b) restrict reflection rays
to crowns only above a zoom threshold, (c) drop the reflection bounce count for domes.
Never: reduce geometric precision, reintroduce TAA, or bake lighting.
*Owner:* Forge.

### R6. Prompt-to-room unreliability
**Probability: medium. Damage: low, if handled.**
Natural language to a typed room struct will sometimes produce nonsense dimensions or
implausible material combinations.

*Trigger:* more than 15% of prompts in an evaluation set produce a room failing the
Eyring check or with implausible geometry.
*Response:* constrain hard. Validate the struct against physical plausibility bounds
before baking, clamp, and surface what was clamped. Keep authored rooms as the
guaranteed path. Generation is an amplifier, never a dependency.
*Owner:* Sceneshaper.

### R7. Agents gaming the gates
**Probability: medium. Damage: medium.**
Discussed at length in [Verification §7](07-verification.md).

*Trigger:* gates green while the blind comparison degrades.
*Response:* the blind comparison is the outer loop and it is not gameable. If gates
and blind results diverge, the gates are wrong, and fixing them takes priority over
all feature work.
*Owner:* Referee.

### R8. Trademark and likeness
**Probability: low. Damage: medium.**
"Octopus" and "genoQs Machines" are someone else's marks, and this is a faithful
reproduction of a commercial product's industrial design.

*Trigger:* any public release.
*Response:* resolve before Phase 5, not after. The obvious paths are to contact
genoQs (the CE firmware suggests an open-minded community, and an authorised
reproduction of a discontinued instrument is a good story for everyone), or to ship
the engine with an original faceplate design and keep the Octopus panel as a personal
project. Decide early: the answer affects the wordmark, the name, and the launch, and
it is far cheaper to know in Phase 1 than in Phase 5.
*Owner:* Conductor. **This is the only risk with a legal component and it should be
addressed in week one.**

### R9. The manual is wrong or ambiguous
**Probability: high. Damage: low.**
Community-maintained documentation for a fifteen-year-old instrument will contain
errors.

*Trigger:* a conformance fixture that cannot be satisfied without contradicting
another.
*Response:* record in `tests/conformance/AMBIGUITIES.md` with both readings, pick one,
mark it, move on. Revisit if hardware access appears.
*Owner:* Metronome.

---

## 2. Open decisions

Each becomes an ADR. Listed with the deadline phase, because an open decision past its
deadline is a blocker wearing a disguise.

| # | Decision | Deadline | Owner |
|---|---|---|---|
| **D1** | Shell typeface, from three specimens rendered at real size | Phase 1 | Curator |
| **D2** | Panel typeface identification, or ship outlines as vector | Phase 1 | Panelwright |
| **D3** | Plugin wrapper: JUCE, or hand-rolled VST3 + AU | Phase 1 | Metronome |
| **D4** | Whether `flat` is a photographic HDRI or a generated room (see R2) | Phase 2 | Forge |
| **D5** | Room geometry synthesis: parametric primitives, or a small procedural grammar | Phase 2 | Sceneshaper |
| **D6** | IR length and convolution strategy (partitioned, zero-latency vs. buffered) | Phase 3 | Sceneshaper |
| **D7** | Whether rehearsal rooms is the app's home or a space you visit | Phase 4 | Loom |
| **D8** | Naming and trademark posture (see R8) | Phase 1 | Conductor |
| **D9** | Whether to ship Push integration in 2.0 or 2.1 | Phase 2 | Metronome |
| **D10** | Persistence format: bespoke binary, or a widely-readable container | Phase 2 | Metronome |

**D3 note.** JUCE gets you both formats, host compatibility, and a decade of edge
cases handled, at the cost of a large dependency and its licence terms, plus the
awkwardness of hosting a `CAMetalLayer` inside a JUCE component. Hand-rolling is
cleaner architecturally and is roughly two weeks of unglamorous work plus a long tail
of host quirks. Recommendation: JUCE for the wrapper only, with the renderer and core
entirely outside it, so the dependency stays at the edge and can be replaced.

**D6 note.** Zero-latency partitioned convolution is the right answer for a live
instrument and is not hard, but it interacts with the room-transition design: swapping
an IR mid-performance needs a cross-fade between two convolvers, which doubles the
cost during the transition. Budget for it in Phase 3 rather than discovering it.

---

## 3. Decisions already made, recorded here so they are not relitigated

These closed early. They are listed because in a long multi-agent project, closed
decisions get reopened by agents who were not there.

| Decision | Chosen | Rejected | Why |
|---|---|---|---|
| Render technique | analytic ray tracing | rasterised meshes | exact silhouettes at any zoom; true inter-reflection |
| Antialiasing | 3× SSAA | TAA, FXAA, MSAA | determinism; bitwise-static requirement |
| Tonemap | AgX | ACES RRT, Reinhard | hue stability through highlight roll-off, critical for LEDs |
| Core language | Rust | C++, Swift, TypeScript | no GC, one implementation for three targets |
| UI host | native app | Max for Live `jweb` | v1's ceiling; no GPU or thread control in Max |
| Plugin formats | VST3 + AUv2 | CLAP, AUv3 | Live 12 does not load CLAP; AUv3 sandboxing is not worth it here |
| IR generation | ray-traced from geometry | generative audio model | determinism, and it keeps sound and vision as one model |
| Motion | physical simulation | authored easing curves | verifiable frame by frame; produces detail nobody would author |
| Panel geometry | measured truth file | hand-placed coordinates | v1's specific failure; makes verification possible at all |
| v1 code | archive, transcribe two files | refactor | wrong architecture; keeping it guarantees reversion to it |

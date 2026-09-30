# 10: Risks and Open Decisions

**Owner:** Conductor.

---

## 1. Risk register

Ordered by expected cost, which is probability times damage, not by probability.
Every risk has a named owner, a trigger that tells you it is happening, and a
response decided in advance.

R2 and R6 were dropped by ADR-0006 and their numbers are not reused.

### R1. Scope death
**Probability: high. Damage: total.**
The work is a front-panel controller for about 140 manual workflows (SPEC-0002 F1),
MIDI input and clock, and a web app. Any one is a project. The failure is not that we
run out of time; it is that we build 80% of several things.

*Trigger:* wave 1 has not exited after twice its expected duration, or work begins
in a wave whose predecessor has not exited.
*Response:* the roadmap's exit criteria are hard. The Conductor refuses fan-out until
wave 1 exits. If wave 1 is genuinely stuck, cut it to Page mode and one MIDI output
(SPEC-0002 `product.md` §5) and ship that.
*Owner:* Conductor.

### R3. Reference plates unobtainable
Closed by ADR-0006 (2026-09-30). The web panel needs a control inventory from the
manual, not calibrated photography (SPEC-0002 F2).

### R4. Timing under a host
**Probability: medium. Damage: high.**
*(check after ADR-0006: the host is now a browser, see SPEC-0002 `tech.md` §3 and spikes S1 and S3)*
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
Closed by ADR-0006 (2026-09-30). It concerned ray-traced frames at 120 Hz on a native
panel, which is retired.

### R7. Agents gaming the gates
**Probability: medium. Damage: medium.**
Discussed at length in [Verification §7](07-verification.md).

*Trigger:* gates green while a wave's demonstration on real gear fails (SPEC-0002
`product.md` §5).
*Response:* the demonstration on real gear is the outer loop. If gates and the
demonstration diverge, the gates are wrong, and fixing them takes priority over all
feature work.
*Owner:* Referee.

### R8. Trademark and likeness
**Probability: low. Damage: medium.**
"Octopus" and "genoQs Machines" are someone else's marks, and this is a faithful
reproduction of a commercial product's industrial design.

*Trigger:* any public release.
*Response:* resolve before the web app is public, not after. The obvious paths are to contact
genoQs (the CE firmware suggests an open-minded community, and an authorised
reproduction of a discontinued instrument is a good story for everyone), or to ship
the engine with an original faceplate design and keep the Octopus panel as a personal
project. Decide early: the answer affects the wordmark, the name, and the launch, and
it is far cheaper to know before the first public deployment than after.
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

D1 and D4 to D7 were dropped by ADR-0006 and their numbers are not reused. D2 changed meaning
(below).

| # | Decision | Deadline | Owner |
|---|---|---|---|
| **D2** | The app's one typeface family, chosen from three specimens rendered beside the panel (was: identifying the hardware's own typeface from photographs) | Wave 1 (P3) | Curator |
| **D3** | Plugin wrapper: JUCE, or hand-rolled VST3 + AU | Closed by ADR-0006: no plugin formats in v1 | Metronome |
| **D8** | Naming and trademark posture (see R8) | Before the first public deployment | Conductor |
| **D9** | Whether to ship Push integration in 2.0 or 2.1 *(check after ADR-0006: not in SPEC-0002; reopens only by a new spec)* | Not scheduled | Metronome |
| **D10** | Persistence format: bespoke binary, or a widely-readable container *(check after ADR-0006: SPEC-0002 `tech.md` §7 proposes versioned JSON)* | Before the first save format ships (P3) | Metronome |

---

## 3. Decisions already made, recorded here so they are not relitigated

These closed early. They are listed because in a long multi-agent project, closed
decisions get reopened by agents who were not there.

| Decision | Chosen | Rejected | Why | Status after ADR-0006 |
|---|---|---|---|---|
| Render technique | analytic ray tracing | rasterised meshes | exact silhouettes at any zoom; true inter-reflection | Reversed by ADR-0006 (2026-09-30): SVG, or Canvas |
| Antialiasing | 3× SSAA | TAA, FXAA, MSAA | determinism; bitwise-static requirement | Reversed by ADR-0006 (2026-09-30): the browser's |
| Tonemap | AgX | ACES RRT, Reinhard | hue stability through highlight roll-off, critical for LEDs | Reversed by ADR-0006 (2026-09-30): not applicable |
| Core language | Rust | C++, Swift, TypeScript | no GC, one implementation for three targets | Unchanged |
| UI host | native app | Max for Live `jweb` | v1's ceiling; no GPU or thread control in Max | Reversed by ADR-0006 (2026-09-30): web app. `jweb` is reopened as Tier 2, on evidence (spike S4) |
| Plugin formats | VST3 + AUv2 | CLAP, AUv3 | Live 12 does not load CLAP; AUv3 sandboxing is not worth it here | Reversed by ADR-0006 (2026-09-30): none in v1. Ableton by routing, or Max for Live |
| IR generation | ray-traced from geometry | generative audio model | determinism, and it keeps sound and vision as one model | Reversed by ADR-0006 (2026-09-30): removed |
| Motion | physical simulation | authored easing curves | verifiable frame by frame; produces detail nobody would author | Reversed by ADR-0006 (2026-09-30): LED flash and a few state changes only |
| Panel geometry | measured truth file | hand-placed coordinates | v1's specific failure; makes verification possible at all | Reversed by ADR-0006 (2026-09-30): a logical control inventory from the manual |
| v1 code | archive, transcribe two files | refactor | wrong architecture; keeping it guarantees reversion to it | Unchanged |

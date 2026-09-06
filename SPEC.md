# OCTOPUS v2: Master Specification

**Codename:** `WENGE`
**Supersedes:** `beaudesign/BenoQs-Sequencer` (archive, do not merge)
**Status:** Ratified. Section changes require an ADR.
**Reference hardware:** genoQs Machines Octopus, CE OS v5.30, Stuttgart 2009.

---

## 0. How to read this document

This is the root. It states the thesis, the non-negotiables, and the map. Everything
operational lives in `docs/`. Everything an agent needs to start work lives in
`agents/`. Everything that must never drift lives in `contracts/`.

Read in this order:

| # | Document | Owner role | Read if you are |
|---|---|---|---|
| 00 | [North Star](docs/00-north-star.md) | Conductor | Everyone. Non-optional. |
| 01 | [Panel Truth](docs/01-panel-truth.md) | Panelwright | Building or verifying the hardware surface |
| 02 | [Architecture](docs/02-architecture.md) | Conductor | Writing any code at all |
| 03 | [Sequencer Core](docs/03-sequencer-core.md) | Metronome | Touching timing, MIDI, or state |
| 04 | [Render Engine](docs/04-render-engine.md) | Forge | Touching pixels |
| 05 | [Design System](docs/05-design-system.md) | Curator | Making any visual decision |
| 06 | [Shell and Rehearsal Rooms](docs/06-shell-and-rooms.md) | Loom | Building the shell (assigned: Fable) |
| 07 | [Verification](docs/07-verification.md) | Referee | Everyone. Non-optional. |
| 08 | [Agent Operating Model](docs/08-agent-operating-model.md) | Conductor | Everyone. Non-optional. |
| 09 | [Roadmap](docs/09-roadmap.md) | Conductor | Planning or reporting |
| 10 | [Risks and Decisions](docs/10-risks-and-decisions.md) | Conductor | Blocked, or about to make a big call |

If a statement in `docs/` contradicts this file, this file wins, and the contradiction
is a bug: file an ADR.

---

## 1. The thesis

The Octopus is not a user interface. It is an object made of wenge, powder-coated
aluminium, chrome, and light, sitting in a room, under a lamp.

Every previous attempt to reproduce it (including v1 in this repo) failed for one
reason: it drew a **picture of** the object instead of **simulating** the object.
Chrome was `linear-gradient(165deg, #e4e0d8 0%, #c8c4bc 48%, #b0aca4 100%)`. But
chrome has no colour of its own. Chrome is a mirror. What you see in a chrome
ball-top encoder is the ceiling, the window, the person leaning over it. A gradient
can never look like chrome, at any level of tuning, because a gradient is not
what chrome is. That single category error is what people are reacting to when
they say something looks like AI slop. It is not the gradient's fault. It is the
fault of using a gradient where a reflection belongs.

So: **v2 does not style the panel. It renders the panel.** A real scene, a real
light probe, real bidirectional reflectance, real colour management, real sub-pixel
geometry derived from measurement rather than from eyeballing a photograph.

And once you accept that the chrome must reflect a room, one question follows
immediately, and it is the question that turns a faithful replica into a new
instrument:

> **Which room?**

That question is the product.

### The unifying mechanic

**The rehearsal room you are in is the environment map on the instrument's chrome,
and the impulse response on its output bus, and both come from the same generated
geometry.**

You describe a space. A concrete stairwell at three in the morning. A dead carpeted
studio in a Berlin basement. A cathedral. A cardboard box. That description becomes
one room model: geometry, materials, absorption coefficients, light. From that single
model we derive, deterministically:

- an **HDR light probe** that lights the panel and is mirrored in every chrome
  encoder and every button dome,
- an **impulse response**, computed by acoustic ray tracing over the same geometry
  and the same material set, convolved onto the instrument's output,
- an **ambient light temperature and level** that shifts the cream faceplate's
  perceived colour and the LED bloom.

Turn the room from a stairwell to a cathedral and you watch the reflections in the
chrome lengthen and cool at the same moment you hear the decay open up. The visual
and the acoustic are not two features that were coordinated by a designer. They are
two projections of one physical model. Nobody has shipped that. It is buildable with
established techniques (image-source method plus stochastic ray tracing for the
acoustics, split-sum IBL for the render). We are going to build it.

That mechanic is why the shell exists, why the dropdown exists, and why "rehearsal
rooms" is the navigational spine rather than a page in a menu.

### What this makes the product

A 2009 German sequencer, reproduced to the micron, standing inside spaces you speak
into existence, where the space is simultaneously how it looks and how it sounds.

---

## 2. Non-negotiables

These are the things that, if violated, mean we have failed regardless of what else
shipped. Each maps to an automated gate in [Verification](docs/07-verification.md).

**N1. The panel is measured, not drawn.**
All control positions come from `contracts/panel.truth.json`, a frozen vector model
in millimetres, human-ratified once. No hardcoded pixel coordinates anywhere in the
render path. Gate: `verify:geometry`.

**N2. No material is a gradient.**
Chrome, powder coat, wood, LED lens, and engraved fill are BRDF materials evaluated
against a light probe. The token `linear-gradient` is banned in the panel render path
and the lint fails the build on it. Gate: `verify:slop`.

**N3. Motion is simulated, not eased.**
Every moving thing has a physical model: encoders have rotational inertia and detent
torque, buttons have travel in millimetres and a return spring, LEDs have rise and
decay time constants. No animation in the instrument surface may be authored as a
bezier keyword. Gate: `verify:motion`.

**N4. Frame integrity is absolute.**
In an offline deterministic capture, static regions are bitwise identical between
frames when nothing is animating, and no frame exceeds budget. Not "rarely exceeds".
Zero. Gate: `verify:frames`.

**N5. Timing beats pixels.**
If the renderer and the sequencer contend, the sequencer wins. Audio and MIDI threads
never allocate, never lock, never wait on the GPU. A dropped frame is a bug. A late
note is a catastrophe. Gate: `verify:timing`.

**N6. Colour is managed end to end.**
Linear working space, Display P3 output, one tonemap, one place. Any code that writes
an sRGB hex literal into a shader fails review. Gate: `verify:color`.

**N7. Every taste judgement becomes a test.**
When a critique pass finds something ugly, the fix is not "make it nicer". The fix is
a new deterministic assertion in the harness plus the change that satisfies it. The
harness only grows. Gate: `verify:regressions` (assertion count is monotonic).

**N8. The hardware's behaviour is the manual's behaviour.**
Where v2 and the CE v5.30 reference manual disagree about what a control does, the
manual wins, and the discrepancy is logged as a fixture in `tests/conformance/`.
Gate: `verify:conformance`.

---

## 3. What we are building, concretely

Four deliverables, in dependency order.

**D1. `octocore`** (Rust, no_std-friendly core)
The sequencer. Ten tracks, sixteen steps, ten banks of sixteen pages, the full
attribute model (VEL PIT LEN STA POS DIR AMT GRV MCC MCH), chains, hypersteps,
phrases, the effector, directions, scales. Sample-accurate. Deterministic under a
seed. Compiles to a native staticlib and to WASM. Zero allocation after init.

**D2. `octopanel`** (Swift + Metal)
The instrument surface. Renders `panel.truth.json` as a lit physical object at
120 Hz with an IBL probe supplied by the current room. Handles input as physical
actuation (press depth, encoder torque) rather than as click events.

**D3. `octoroom`** (Rust + Metal compute)
The room system. Prompt to room model to (light probe, impulse response, ambient
grade). Owns the acoustic ray tracer and the probe baker.

**D4. `octoshell`** (Swift, assigned to Fable)
The navigational layer: the room dropdown, the rehearsal rooms space, settings.
Genie-influenced. The shell is thin by design and its job is to make the instrument
and the room feel like one instrument, not two apps.

Plus two host wrappers:

- **`octopus.vst3` / `octopus.component`** for Ableton Live 12 on macOS. Live supports
  AU (v2 and v3 since 11.3) and VST3, and does not support CLAP. We ship VST3 and
  AUv2 from one codebase.
- **`Octopus.amxd`**, a thin Max for Live device that does transport sync, Link, and
  MIDI routing only. It does not host the UI. v1's mistake was trying to render an
  instrument inside a WebKit view inside Max. We are not repeating it.

---

## 4. What we are deliberately not doing

Scope discipline is a design decision. Explicitly out for v2.0:

- Windows and Linux. macOS/Apple Silicon only. The renderer is Metal.
- Sysex interchange with real Octopus or Nemo hardware. Designed for, shipped in 2.1.
- Multiplayer or shared rooms.
- Any generative audio model. The room system generates *geometry and materials*,
  and physics generates the sound. This is a feature, not a limitation: it is
  deterministic, auditable, and about four orders of magnitude cheaper.
- Skinning, theming, or user-supplied panel layouts. There is one Octopus.

---

## 5. Repo layout

```
wenge/
├── SPEC.md                     ← you are here
├── CLAUDE.md                   → symlink to agents/CLAUDE.md
├── justfile                    ← every verb an agent needs
├── contracts/                  ← frozen. Architect-only via ADR.
│   ├── panel.truth.json        ← the hardware, in millimetres
│   ├── panel.truth.schema.json
│   ├── motion.registry.json    ← every animated quantity, declared
│   ├── motion.registry.schema.json
│   ├── materials.json          ← BRDF parameters per surface
│   └── verification.report.schema.json
├── crates/
│   ├── octocore/               ← D1  (owner: Metronome)
│   ├── octoroom/               ← D3  (owner: Sceneshaper)
│   └── octoffi/                ← C ABI boundary
├── apps/
│   ├── OctoPanel/              ← D2  (owner: Forge)
│   ├── OctoShell/              ← D4  (owner: Loom / Fable)
│   └── OctoStandalone/         ← host app that composes the above
├── hosts/
│   ├── vst3/  au/              ← plugin wrappers (owner: Metronome)
│   └── m4l/                    ← Octopus.amxd bridge
├── harness/                    ← D-∞  (owner: Referee)
│   ├── capture/                ← deterministic frame capture
│   ├── measure/                ← geometry, photometry, motion, frames
│   ├── lint/                   ← the slop detector
│   └── report/                 ← HTML + JSON reports
├── reference/
│   ├── manual/                 ← CE v5.30, page-indexed, text-extracted
│   ├── plates/                 ← calibrated reference photography (git-lfs)
│   └── NOTES.md                ← what each plate is good for
├── tests/
│   ├── conformance/            ← manual-derived behavioural fixtures
│   └── golden/                 ← frozen render goldens (git-lfs)
├── docs/                       ← this spec
├── adr/                        ← architecture decision records
└── journal/                    ← per-agent working logs (append-only)
```

---

## 6. The single most important paragraph in this document

Most ambitious specs fail at the same place: they describe a beautiful destination and
say nothing about how you would know, mechanically, at 3am, with no human awake,
whether the last commit made things better or worse. That is the difference between a
project that converges and one that oscillates forever while burning tokens.

So the centre of gravity of this specification is not the render engine and not the
room generator. It is [Verification](docs/07-verification.md), and specifically the
rule in **N7**: *every taste judgement becomes a test*. A model looking at a screenshot
and saying "the chrome looks a bit plasticky" is worth nothing on its own, because it
will say something different tomorrow. That same observation, converted into
"specular lobe width at the 12mm encoder crown must fall between X and Y when lit by
probe P", is worth everything, because it is true tomorrow and it fails loudly when
someone breaks it.

Build the ratchet before you build the thing. The ratchet is what makes a thousand
agent-hours add up instead of cancel out.

---

## 7. Definition of done for v2.0

The build ships when all of the following are simultaneously true on `main`:

1. `just verify` is green, with ≥ 400 assertions in the harness.
2. Geometry: p95 control-centroid error ≤ 0.35 mm, max ≤ 0.80 mm against
   `panel.truth.json`.
3. Photometry: mean ΔE2000 ≤ 2.0, max ≤ 4.0 across the 14 named material patches.
4. Frames: 10 000 consecutive frames at 120 Hz under the standard stress scene with
   zero over-budget frames and zero static-region deltas.
5. Timing: MIDI note-on jitter σ ≤ 120 µs and max ≤ 500 µs over a 30 minute run at
   192 PPQN, measured against the host clock in Live 12.
6. Conformance: 100% of the manual-derived fixtures pass.
7. Rooms: five shipped rooms, each with a baked probe and a measured RT60 within
   10% of the acoustic model's prediction.
8. A blind panel of three people who have used the real hardware cannot pick the
   render from a calibrated photograph at 100% zoom in a 2-alternative forced choice
   above 60% accuracy.

Item 8 is the real one. Items 1 through 7 exist to make item 8 reachable.

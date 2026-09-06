# 05: Design System

**Owner:** Curator. **Gate:** `verify:slop`, `verify:tokens`, `verify:a11y`.

---

## 1. Two worlds, one law

This product contains two visual worlds that must never blend:

**The instrument.** A 2009 object. Its visual properties are *measured*, not designed.
The Curator has no authority here. If it is not in `panel.truth.json` or
`materials.json`, it does not exist. The entire creative act on the instrument side is
the choice to be faithful.

**The shell.** A 2026 interface. Fully designed. The Curator has full authority, under
the constraints below.

The law that governs the boundary:

> **The shell may never place a graphic element on top of the instrument, and the
> instrument's visual language may never be quoted in the shell.**

No overlay panels floating over the panel. No skeuomorphic chrome buttons in the
settings page. No cream backgrounds in the shell borrowed from the faceplate. The two
worlds meet only in the room: the instrument sits in the room, the shell is drawn in
the room's air, and neither pretends to be the other.

---

## 2. Tokens

Tokens live in `contracts/design.tokens.json` and are compiled into Swift constants.
Nothing in the shell may use a literal value where a token exists. `verify:tokens`
enforces this by AST scan.

### 2.1 Space

A single modular scale, base 4 pt, ratio derived from the panel's own geometry rather
than an arbitrary number. The matrix's step pitch, in points at the default zoom,
seeds the scale. This means the shell's rhythm is harmonically related to the
instrument's, which nobody will consciously see and everyone will feel.

```
space.0  = 0
space.1  = 4
space.2  = 8
space.3  = 12
space.4  = 20
space.5  = 32
space.6  = 52
space.7  = 84
```

Fibonacci-adjacent by construction. Eight values. If you need a ninth, you are
designing a ninth thing and should stop.

### 2.2 Radius

**There are five radii in this product and all five are physical measurements.**

```
radius.led      ← LED lens radius, from truth file
radius.dome     ← matrix button dome, from truth file
radius.crown    ← encoder crown, from truth file
radius.cheek    ← case end-cheek fillet, from truth file
radius.aperture ← faceplate aperture corner, from truth file
```

The shell may use `radius.cheek` and `radius.aperture` and nothing else. There is no
`radius.sm`, no `radius.md`, no `radius.lg`. The single most reliable signature of
generated interface design is a set of invented radii applied uniformly to unrelated
things, and removing the ability to do it removes the temptation.

### 2.3 Colour

The shell's palette is derived from the current room, not fixed. This is unusual and
it is the right call: the shell exists inside the room, so it should be lit by it.

```
shell.ink        = f(room.luminance)   high-contrast against the room, ≥ 7:1
shell.ink.quiet  = shell.ink @ 0.55
shell.scrim      = room average colour, heavily desaturated, low alpha
shell.focus      = derived from room hue, rotated to maximum separation
```

There is **no brand colour and no accent colour**. The only saturated colour in the
product is emitted by an LED on the instrument. When a shell element needs emphasis it
uses contrast, weight, or space, never hue. This is the hardest rule in the document
to keep and the most valuable.

`shell.focus` exists only for keyboard focus rings, where an accessibility
requirement overrides an aesthetic preference. That is the correct order of
priority.

### 2.4 Type

One family, defined in `contracts/design.tokens.json` after the Curator's ADR (see
[North Star §3.2](00-north-star.md)). Four roles:

```
type.room     54 pt / 1.02   the room name, the only large type in the product
type.body     15 pt / 1.55   settings, descriptions
type.control  13 pt / 1.35   labels on shell controls
type.value    13 pt / 1.35   numeric readouts, tabular figures on
```

Sentence case throughout. No all-caps. No letter-spaced small labels. Line length
capped at 68 characters.

`type.room` is the one place the design is allowed to be loud, per
[North Star P4](00-north-star.md). A room name set at 54 pt, alone, over a generated
world, is the whole shell's personality.

---

## 3. Motion grammar for the shell

The instrument's motion is physics (see [Render §6](04-render-engine.md)). The shell's
motion is also physics, but a different, simpler system: **one spring, three
presets**.

```
spring.tap      ω=28.0  ζ=0.86   ~180 ms   toggles, small state changes
spring.reveal   ω=16.0  ζ=0.80   ~340 ms   dropdown, sheets, page changes
spring.world    ω= 5.2  ζ=1.00   ~1200 ms  room transitions, probe cross-fade
```

Rules:

- Every shell animation uses one of these three. Adding a fourth requires an ADR.
- Springs are integrated analytically, not stepped, so they are frame-rate
  independent and identical in a live session and an offline capture.
- Nothing animates that the user did not cause, except the room's own ambient life.
- `prefers-reduced-motion` collapses `spring.tap` and `spring.reveal` to instant and
  reduces `spring.world` to a 200 ms cross-fade with no camera movement.

**Why `spring.world` is critically damped at ζ=1.0:** a room transition that
overshoots would mean the reflections in the chrome overshoot too, and the instrument
would appear to wobble. Overshoot is charming on a menu and alarming on a physical
object. This is the kind of consequence you only notice once the reflection is real.

---

## 4. The slop lint

`harness/lint/slop.py`, run by `verify:slop` and in the pre-commit hook. Each rule
exists because it caught a real instance of the failure mode. Rules are added, never
removed.

| # | Rule | Rationale |
|---|---|---|
| S1 | No `linear-gradient`, `radialGradient`, or gradient shader helper in the panel render path | N2. Gradients are not materials. |
| S2 | No colour literal outside `materials.json` / `design.tokens.json` | Colour has one source. |
| S3 | No radius value not drawn from the five physical radii | §2.2 |
| S4 | No `box-shadow`-equivalent with `rgba(0,0,0,0.0x)` | Shadows are traced, not painted. |
| S5 | No all-caps or letter-spaced label styles in the shell | E4 |
| S6 | No `→`, `·`-joined meta strings, or `01 / 02 / 03` numbered eyebrows | template chrome |
| S7 | No monospace face used for non-code content | template chrome |
| S8 | No `#0B0B0B`, `#111111`, `#F4F1EA`, `#D97757` or values within ΔE 3 of them | the AI-design cluster |
| S9 | No easing keyword (`ease-in-out`, `cubic-bezier(…)`) anywhere | §3, N3 |
| S10 | No animation not registered in `motion.registry.json` | N3 |
| S11 | No `Google Sans`, `Inter`, `SF Pro` as a shell face | E4, defaults |
| S12 | No hardcoded pixel coordinate in the render path | N1 |
| S13 | Magic-number scan in shaders: any float literal outside [0,1] and not a named constant | traceability |
| S14 | No idle/looping animation on the instrument surface | P3 |

S8 deserves a note. `#F4F1EA` is flagged because it is the centroid of generated
warm-cream backgrounds. If the measured faceplate coat lands within ΔE 3 of it, the
lint will fire, and the correct response is a suppression comment citing the
calibration report. The point of the rule is not that the colour is forbidden; it is
that arriving there **by measurement** must be distinguishable from arriving there
**by default**, and a suppression with a citation makes that distinction permanent
and auditable.

---

## 5. Critique rubric

Used by the perceptual critique pass in [Verification §5](07-verification.md).
Scored 1–5 against fixed anchors. Scores are **advisory only**; their real output is
findings, and findings become assertions.

| Dimension | 1 | 3 | 5 |
|---|---|---|---|
| **Materiality** | reads as flat shapes | reads as a rendering | indistinguishable from a photograph of metal |
| **Light coherence** | elements lit inconsistently | consistent but flat | one convincing light source, correct falloff, correct inter-reflection |
| **Geometric conviction** | visibly approximate | close | no visible deviation at 400% zoom |
| **Typographic authority** | generic | competent | letterforms feel cut into the surface |
| **Motion honesty** | eased | plausible | reads as mass responding to force |
| **Restraint** | decorated | tidy | nothing present that does not earn its place |
| **Coherence of worlds** | shell and instrument fight | coexist | the instrument appears to be *in* the room |

Anchors for each score are stored as reference images in `harness/report/anchors/`
so that scoring is calibrated against fixed exemplars rather than against the model's
mood on the day. Without anchors, VLM scoring drifts by more than a point between
sessions and is worse than useless.

---

## 6. Accessibility floor

Non-optional, and not in tension with the aesthetic.

- Keyboard operable end to end. Every control on the panel is reachable by keyboard
  with a documented traversal order following the physical layout (matrix in reading
  order, then encoders, then spiral by increasing θ).
- Visible focus. `shell.focus` ring, 2 pt, with a 1 pt dark inner stroke so it is
  visible against both bright and dark rooms.
- `prefers-reduced-motion` honoured as in §3.
- `prefers-reduced-transparency` disables the scrim and uses a solid derived from
  the room's average.
- Contrast ≥ 7:1 for all shell text against its computed backdrop. Because the room
  is generated and its luminance varies, this must be **computed per room**, not
  assumed. `verify:a11y` renders each shipped room and measures.
- VoiceOver labels for every control, generated from the truth file's ids and the
  manual's own terminology. A blind musician should be able to operate the sequencer.
  The Octopus's own design (a stave, ten tracks, sixteen steps) is unusually amenable
  to this.
- A high-contrast panel mode that raises LED luminance and increases engraving
  contrast, without changing geometry. Off by default, and it does not count as
  "theming" because it changes exposure rather than design.

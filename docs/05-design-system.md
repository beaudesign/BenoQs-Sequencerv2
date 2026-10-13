# 05: Design System

**Owner:** Curator. **Gate:** `verify:slop`, `verify:tokens`, `verify:a11y`, none built yet.

This is deliberately short. The panel is a control surface whose look is decided by the
hardware's layout and the manual's names, and the only saturated colour in it is a lit
LED (SPEC-0002 D3 as amended by ADR-0010, `SPEC.md` N2 and N6). The Curator's authority is over what may *not* be added.

**Status: proposed.** D3 approved "a faithful layout, restrained materials, LEDs as the only
colour". Everything below that goes further (the strip around the panel, the radius rule, the
focus ring, the contrast mode, the motion note, the slop rules S15 and S16) is the Conductor's
reading of D3, written in P1 so that the old rules could be removed without leaving a hole. The
Curator and the owner ratify or change it with the UI sketch that comes with P3.

---

## 1. The panel and the page around it

The screen holds two things: the panel, and a plain strip of browser furniture (MIDI ports,
clock source, saving, a "this browser has no MIDI" message). They must never blend:

> **The strip may never sit on top of the panel, and the panel's look may never be quoted in
> the strip.**

No overlays on the panel. No round, lit or engraved-looking controls in the strip. No colour
in the strip. Each is one plain thing, and neither pretends to be the other.

The panel has a ground of its own, the plate, inside a frame (ADR-0010). The strip and the page
keep `surface`.

**The strip and the page are a midnight ground with hairlines (ADR-0011).** At the owner's
direction of 2026-10-13 they take their look from a near-black "precision instrument" reference:
a pure-grey ground (`surface`), a hairline between regions (`edge`), controls drawn as a border
(`edge.control`, at least 3:1 against the ground) with one small corner (`radius.control`), the
ink on the ground, and `surface.raised` as the fill a pressed control takes. The instrument sits
on that ground as an object: the frame is the case, it hugs the plate, and the page shows
round it. What the reference has and this product does not: an accent colour (the one filled
control, Play, is `ink` on `surface`), hued greys (the greys here are pure), shadows, gradients,
cards and tracking.

**The ground carries three small line drawings (ADR-0011 decision 10)**, above the case, in the room the
case leaves: hairline strokes in `edge.control`, solid and dotted, no fill, dotted construction lines, drawn
from the space tokens, `aria-hidden`, still. They are the ground's only decoration; none touches the
panel or the strip, and a rule keeps every `#ground` rule to that role and to `fill: none`.

---

## 2. Tokens

Tokens will live in `contracts/design.tokens.json` (planned with the app, P3, by ADR) and are
generated into the app's source. Nothing in the app may use a literal value where a token
exists. `verify:tokens` will enforce this by scanning the source.

### 2.1 Colour

Three LED roles, one neutral palette (the strip's tones, the panel's and the ink), and the panel's two materials. Eighteen roles in all, and no nineteenth.

| Token | Role | Notes |
|---|---|---|
| `led.red`, `led.green`, `led.orange` | The LED language | Roles, not hues. The manual's colourways (red as blue, green as yellow, orange as purple) are a table that maps a role to a hue. Steady or flashing is part of the meaning |
| `led.off` | An unlit LED | Neutral. Distinguishable from every lit state by more than hue |
| `ink`, `ink.quiet` | Text | Contrast at least 7:1 against `surface`; `ink` also 7:1 against `surface.raised` |
| `surface` | The page's and the strip's ground | One value. Near-black pure grey; carries the S8 suppression and its reason (ADR-0011) |
| `surface.raised` | The fill a pressed or selected control takes | Pure grey; the next tone up from `surface` |
| `edge` | A hairline between regions of the strip and the page | Pure grey; at least 1.2:1 against `surface` |
| `edge.control` | The border of a control | Pure grey; at least 3:1 against `surface` (WCAG 2.1 non-text). The tones run `surface`, `surface.raised`, `edge`, `edge.control`, `ink.quiet`, `ink`, each lighter than the last |
| `focus` | Keyboard focus ring | Neutral (ink plus a contrasting inner stroke), not a hue |
| `plate` | The faceplate (ADR-0010) | Tinted, chroma at most 12. Carries the S8 suppression and its reason |
| `frame` | The case round the plate (ADR-0010) | Tinted, chroma at most 12 |
| `plate.ink`, `plate.ink.quiet` | Printed names; captions, row numbers and clefs, on the plate | Pure grey. At least 7:1 against `plate` |
| `key` | A key's or a knob's flat tone | Pure grey |
| `key.edge` | Its outline | Pure grey. At least 3:1 against `plate` |

There is **no brand colour and no accent colour**. When something needs emphasis it uses
contrast, weight or space, never hue. This is the hardest rule in the document to keep and
the most valuable. The values are chosen when the app is built, by the Curator, and recorded
in the tokens file.

### 2.2 Space

One scale, base 4, eight values. If you need a ninth, you are designing a ninth thing and
should stop.

```
space.0 = 0    space.1 = 4    space.2 = 8    space.3 = 12
space.4 = 20   space.5 = 32   space.6 = 52   space.7 = 84
```

### 2.3 Radius

There is no radius scale. The panel's controls are the shapes the hardware has (LEDs, buttons
and encoders are circles). The strip's controls have one corner, `radius.control` (6 px, ADR-0011),
and nothing else has a radius: the token is one value between 1 and 12, only a rule that
starts at `#strip` may use it (S15), and a second one is a scale. There is no `radius.sm`, no
`radius.md`, no `radius.lg`, no card radius. The most reliable signature of generated interface
design is a set of invented radii applied uniformly to unrelated things, and holding it to one
value for one kind of control removes the temptation.

### 2.4 Type

One family. It is Inter Variable, at the owner's direction of 2026-10-13 (ADR-0011), as a file in
the repository with its licence beside it (`type.face`), so that naming it is not a promise
about what the visitor has installed (T17); the Curator may still bring specimens (see
[North Star §3.2](00-north-star.md)). The page turns on the features `cv01`, `ss03` and `zero`.
Weights are two: regular (400) and strong (510). Emphasis is made with weight, and not with
bold: the strong weight is under 700 (T18). Panel labels use the manual's names as the manual
spells them. The strip is set in sentence case, with no all-caps and no letter-spaced small
labels, and a line length of at most 68 characters. Numeric readouts use tabular figures.
Sizes are set with the family.

---

## 3. Motion

There is one kind of motion: **state**. An LED is off, on or flashing at the rate the manual
gives. A button is up or down. An encoder steps by detents; it has no inertia. Nothing else
moves, and nothing moves on its own.

- No easing keyword and no `cubic-bezier`, anywhere. No CSS `transition` or `animation`,
  except the one class that flashes an LED.
- An LED's flash is information, not decoration, so `prefers-reduced-motion` does not change
  it. If the manual's flash rate ever exceeds three flashes a second, that is a finding
  (WCAG 2.3.1), not a reason to slow the language down silently.

---

## 4. The slop rules

Run by `verify:slop` and in the pre-commit hook once the gate is built (the gate is
`not_implemented` today). Each rule exists because it caught a real instance of the failure
mode. Rules are added, never removed.

| # | Rule | Rationale |
|---|---|---|
| S1 | No `linear-gradient`, `radial-gradient`, `conic-gradient`, or SVG gradient element in the app | N2. A gradient is not a material. |
| S2 | No colour literal outside `design.tokens.json`; no font weight, feature or variation literal in a stylesheet | N6. Colour has one source, and so has the face (ADR-0011). |
| S4 | No `box-shadow` and no `drop-shadow` filter | Depth is not painted. |
| S5 | No all-caps or letter-spaced label styles in the strip | E4 |
| S6 | No `→`, `·`-joined meta strings, or `01 / 02 / 03` numbered eyebrows | template chrome |
| S7 | No monospace face used for non-code content | template chrome |
| S8 | No `#0B0B0B`, `#111111`, `#F4F1EA`, `#D97757` or values within ΔE 3 of them | the AI-design cluster |
| S9 | No easing keyword (`ease-in-out`, `cubic-bezier(…)`) anywhere | §3, N3 |
| S11 | No `Google Sans`, `Inter`, `SF Pro` named in the app's source. The one face is the file the tokens name (`type.face`, T17), reached through `var(--type-family)`; a bare `Inter` in the tokens is still refused (T10) | E4, defaults; ADR-0011 |
| S12 | No hardcoded pixel coordinate in the panel code | N1 |
| S14 | No idle or looping animation on the panel | P3 |
| S15 | No `border-radius` other than `50%` on the round controls, and `var(--radius-control)` in a rule that starts at `#strip` | §2.3, ADR-0011 |
| S16 | No CSS `transition`, `animation` or `@keyframes` other than the LED flash | §3, N3 |
| S17 | The six panel roles (`plate`, `frame`, `plate.ink`, `plate.ink.quiet`, `key`, `key.edge`) appear only in the rules of `#panel` and `#panel-frame`; and the strip's (`surface`, `surface.raised`, `edge`, `edge.control`, `ink`, `ink.quiet`, and the shine that names `ink`) appear in none of them | §1, ADR-0010, ADR-0011 |

S8 needs a note. `#F4F1EA` is flagged because it is the centroid of generated warm-cream
backgrounds. If the neutral palette lands within ΔE 3 of it, the lint fires, and the correct
response is a suppression comment that states the reason. The point of the rule is not that
the colour is forbidden; it is that arriving there deliberately must be distinguishable from
arriving there by default. `plate` is such a case (ADR-0010): it carries the suppression and its
reason.

**Retired with the photoreal panel (ADR-0006): S3** (radii limited to the five measured ones),
**S10** (every animation registered in the motion registry) and **S13** (magic numbers in
shaders). Their numbers are not reused; the web panel's own rules are S15 and S16.

---

## 5. Accessibility floor

Non-optional, and not in tension with the look. Acceptance criterion A8
(`specs/SPEC-0002/product.md` §9), checked by `verify:a11y`.

- **Keyboard end to end.** Every control on the panel is reachable by keyboard in a
  documented order that follows the physical layout: the matrix in reading order, then the
  encoders, then the spiral in order. Hold-and-press gestures need a keyboard equivalent; its design is open
  (`specs/SPEC-0002/tech.md` §5).
- **Visible focus.** The `focus` ring, 2 px, with a 1 px contrasting inner stroke so it is
  visible on the surface, on the plate and on a lit LED.
- **Labels.** Every control has an accessible name that is the manual's own name for it
  (**gate: 100% of controls**). The displays are exposed as text, not only drawn. A blind
  musician should be able to operate the sequencer; the Octopus's own design (a stave, ten
  tracks, sixteen steps) is unusually amenable to it.
- **Colour is not the only carrier.** Red, green and orange are hard to tell apart for many
  people, so steady versus flashing carries meaning as well, and the manual's colourways serve
  as a colour-vision option.
- **Contrast.** At least 7:1 for text in the strip against `surface`.
- **`prefers-contrast`.** A high-contrast mode raises the contrast of labels and of `led.off`
  against `surface`, without changing the layout. It does not count as theming.

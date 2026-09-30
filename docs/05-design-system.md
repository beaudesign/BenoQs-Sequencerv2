# 05: Design System

**Owner:** Curator. **Gate:** `verify:slop`, `verify:tokens`, `verify:a11y`, none built yet.

This is deliberately short. The panel is a control surface whose look is decided by the
hardware's layout and the manual's names, and the only colour in it is a lit LED
(SPEC-0002 D3, `SPEC.md` N2 and N6). The Curator's authority is over what may *not* be added.

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

---

## 2. Tokens

Tokens will live in `contracts/design.tokens.json` (planned with the app, P3, by ADR) and are
generated into the app's source. Nothing in the app may use a literal value where a token
exists. `verify:tokens` will enforce this by scanning the source.

### 2.1 Colour

Three LED roles and one neutral palette. That is all of it.

| Token | Role | Notes |
|---|---|---|
| `led.red`, `led.green`, `led.orange` | The LED language | Roles, not hues. The manual's colourways (red as blue, green as yellow, orange as purple) are a table that maps a role to a hue. Steady or flashing is part of the meaning |
| `led.off` | An unlit LED | Neutral. Distinguishable from every lit state by more than hue |
| `ink`, `ink.quiet` | Text and hairlines | Contrast at least 7:1 against `surface` for text |
| `surface` | Panel and page ground | One value |
| `focus` | Keyboard focus ring | Neutral (ink plus a contrasting inner stroke), not a hue |

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
and encoders are circles) and the strip has square corners. There is no `radius.sm`, no
`radius.md`, no `radius.lg`. The most reliable signature of generated interface design is a
set of invented radii applied uniformly to unrelated things, and removing the ability to do
it removes the temptation.

### 2.4 Type

One family, chosen by ADR (see [North Star §3.2](00-north-star.md)). Panel labels use the
manual's names as the manual spells them. The strip is set in sentence case, with no all-caps
and no letter-spaced small labels, and a line length of at most 68 characters. Numeric
readouts use tabular figures. Sizes are set with the family.

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
| S2 | No colour literal outside `design.tokens.json` | N6. Colour has one source. |
| S4 | No `box-shadow` and no `drop-shadow` filter | Depth is not painted. |
| S5 | No all-caps or letter-spaced label styles in the strip | E4 |
| S6 | No `→`, `·`-joined meta strings, or `01 / 02 / 03` numbered eyebrows | template chrome |
| S7 | No monospace face used for non-code content | template chrome |
| S8 | No `#0B0B0B`, `#111111`, `#F4F1EA`, `#D97757` or values within ΔE 3 of them | the AI-design cluster |
| S9 | No easing keyword (`ease-in-out`, `cubic-bezier(…)`) anywhere | §3, N3 |
| S11 | No `Google Sans`, `Inter`, `SF Pro` as the app's face | E4, defaults |
| S12 | No hardcoded pixel coordinate in the panel code | N1 |
| S14 | No idle or looping animation on the panel | P3 |
| S15 | No `border-radius` other than `50%` on the round controls | §2.3 |
| S16 | No CSS `transition`, `animation` or `@keyframes` other than the LED flash | §3, N3 |

S8 needs a note. `#F4F1EA` is flagged because it is the centroid of generated warm-cream
backgrounds. If the neutral palette lands within ΔE 3 of it, the lint fires, and the correct
response is a suppression comment that states the reason. The point of the rule is not that
the colour is forbidden; it is that arriving there deliberately must be distinguishable from
arriving there by default.

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
  visible on the surface and on a lit LED.
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

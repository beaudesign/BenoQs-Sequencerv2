# 00: North Star

**Owner:** Conductor. **Ratifier:** Curator. **Change process:** ADR.

The product is a web sequencer (`adr/0006-web-sequencer-supersedes-the-rooms-scope.md`,
`specs/SPEC-0002/`). This document says what the panel must be, and why v1 failed at it.

---

## 1. The object we are working from

Everyone working on this needs the same picture of what the Octopus physically *is*, because
the layout of the web panel follows from it.

It is a desk instrument roughly 700 mm wide, built by two people in Stuttgart in 2009: a dark
hardwood case, a flat aluminium faceplate in a warm off-white, graphics engraved through the
coat and filled dark. There are 160 buttons in the main matrix (ten rows of sixteen), a set of
control buttons arranged in a spiral on the right half, and two columns of ten rotary encoders.
Beside most buttons sits a small LED, and **the LEDs are the display**: one LED means different
things in different modes, in three colours, steady or flashing.

The web panel does not copy the wood, the coat or the chrome (SPEC-0002 D3). It keeps what the
hand and the eye use:

**The layout.** The matrix, the encoder columns, the selector and mutator columns, the circle,
the chord block, the transport and the spiral stay where the hardware has them, relative to
each other. A player who knows the Octopus finds every control without looking. Positions are
logical, in `contracts/controls.json` (planned, P2), not millimetres.

**The names.** Every control is labelled with the manual's own name (VEL, PIT, LEN, STA, POS,
DIR, AMT, GRV, MCC, MCH). Nothing is renamed.

**The clefs.** At the far left, a treble clef sits at the top of the track column and a bass
clef at the bottom. They tell you the matrix is a stave: the whole panel argues that this is a
musical instrument and not a machine. They are kept.

**The spiral.** The mode, scale, transport and chord controls on the right are laid out on a
spiral rather than a grid. It is the thing that makes an Octopus recognisable from ten metres.
It is in the inventory as a zone, with its own order, and it is not snapped to a grid.

**The LED language.** Red, green and orange, steady or flashing. The manual's alternate
colourways (red as blue, green as yellow, orange as purple) mean the colours are roles, not
hues. This is where the product is allowed to have colour, and it is the only place.

---

## 2. Why v1 read as generated, precisely

Not a vague critique. Four specific, diagnosable errors, all present in
`ui-mock/index.html` in the archived v1 repo. They apply to any web mock of this panel, so
they stay as the checklist for this one.

**E1. Reflection replaced by gradient.**
`--ball-lg: linear-gradient(165deg, #e4e0d8 0%, #c8c4bc 48%, #b0aca4 100%)`. A gradient is a
monotonic ramp along an axis, and it was standing in for chrome, which is not a ramp. A
two-stop ramp reads as plastic because plastic is exactly what it describes. The web panel
does not pretend to be metal: it uses flat fills and hairlines, and it does not fake a
material with a gradient or a shadow.

**E2. Uniform lighting.**
Every ball used the same highlight regardless of position on the panel. A field of 160
identically-lit domes is the visual signature of a stylesheet, and the eye detects the
repetition instantly. In the web panel no control carries a painted highlight at all.

**E3. Approximated geometry.**
`--step-size: clamp(12px, 1.9vw, 22px)` with `--step-gap: 3px`. The layout was derived from
what looked about right in a browser, so it drifted at every viewport. The spiral was
hand-placed in SVG. The web panel draws from the inventory and scales as one object.

**E4. Borrowed chrome.**
Google Sans, a 28px top bar, 7px uppercase letter-spaced status text, a hairline treatment
copied from a dashboard. None of these are on the hardware. They are the vocabulary of a 2024
web dashboard applied to a 2009 German instrument, and every one of them is a tell.

The corrective is stated as N1 to N9 in `SPEC.md`. This document exists so that the corrective
is understood rather than merely obeyed.

---

## 3. The plan for the panel

### 3.1 Colour

There is one neutral palette and three LED roles, and nothing else
([Design System §2](05-design-system.md)). There is **no accent colour and no brand colour**,
because the hardware has none. The only saturated colour anywhere in the product is emitted by
an LED. This is the strictest and most valuable constraint in the design system, and it will
be tempting to break it in the settings page. Do not. Emphasis is made with contrast, weight
and space.

### 3.2 Type

One family, chosen by the Curator by ADR after three specimens are rendered beside the panel
at real size. Not Inter and not Google Sans (E4). Labels on the panel use the manual's names
as the manual spells them. The page around the panel is set in sentence case, with no all-caps
and no letter-spaced small labels.

### 3.3 Layout

The panel is fixed-aspect and centred. It does not reflow, because objects do not reflow. On a
smaller window it gets smaller; it never rearranges. This is the opposite of standard
responsive practice and it is correct here.

Around the panel there is one plain strip for what a browser needs and the hardware does not:
the MIDI ports, the clock source, saving and loading, and a message when the browser has no
MIDI. It is the only page furniture, it never sits on top of the panel, and it borrows nothing
from the panel's look. There is no sidebar, no card, no toast stack. Where the strip sits and
what it holds is settled with the owner in the UI sketch that comes with P3.

### 3.4 Principles

**P1. Behaviour first.** The panel is worth what the engine and the fixtures behind it are
worth. A control that draws well and does not do what the manual says is a defect.

**P2. The manual is the truth.** Where the panel and the manual disagree, the manual wins and
the disagreement is a fixture (N8).

**P3. The panel is inert until touched.** Nothing on it animates on its own except LEDs driven
by the sequencer. No idle pulses, no breathing glows, no attract loops. The restraint is the
point.

**P4. Spend nothing on decoration.** The panel is disciplined to the point of severity. What
makes it good is that a musician's hands already know it.

**P5. Latency is a material.** A control that answers in 8 ms feels like a machine; the same
control at 40 ms feels like a web page. Input-to-sound and input-to-LED are design constraints
of equal standing with layout, and timing beats pixels (N5).

### 3.5 Review of the plan against the defaults

Checked against the known cluster of generated-design tells:

- *Warm cream background with high-contrast serif and terracotta accent:* not used. The
  palette is neutral, and there is no accent at all. The lint flags values near the cluster
  ([Design System §4](05-design-system.md), S8).
- *Near-black with one acid accent:* not used.
- *Broadsheet hairlines and zero radius:* rejected. Controls are the round things the
  hardware has.
- *SaaS card kit:* impossible by construction. There are no cards.
- *Template chrome (all-caps eyebrows, middle-dot meta strings, arrows in buttons, monospace
  data labels):* all banned by the slop rules.

---

## 4. What "top 1%" means operationally

It does not mean more polish. It means these things, which are measurable:

1. **Nothing is approximated.** Every position traces to `controls.json`, every colour to a
   token, every behaviour to a manual page. Grep for magic numbers; there should be none in
   the panel code.
2. **The hard cases are handled.** Forty LEDs changing on one tick. Two MIDI ports at once.
   A clock that jitters. A tab that goes into the background. A browser that has no MIDI.
3. **It plays.** The pattern the manual describes plays the notes the manual describes, on the
   gear on the desk and in Live, and stays in time (A3 to A6 in
   `specs/SPEC-0002/product.md`).
4. **It is fast.** The sequencer never misses. Slowness is a defect.
5. **It knows what it is not.** No feature exists because it was easy. The list in
   `SPEC.md §4` is as important as anything we build.

---

## 5. A warning about the failure mode of this project

The likeliest way this project dies is not technical. It is that it produces a beautiful
panel on top of a sequencer that does not do what the manual says, and then nobody wants to
use it. The mitigation is in the roadmap: **the vertical slice comes first**. One workflow,
end to end (a click on a step toggles it, the LED shows it, the note plays on real gear),
before a second workflow is added. If the slice is not good enough to make someone want the
rest, no amount of the rest will save it.

Build narrow and finished before wide and rough. Every time.

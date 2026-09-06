# 00: North Star

**Owner:** Conductor. **Ratifier:** Curator. **Change process:** ADR.

---

## 1. The object we are reproducing

Before any pixel is drawn, everyone working on this needs the same mental model of
what the Octopus physically *is*, because every rendering decision follows from it.

It is a desk instrument roughly 700 mm wide, built by two people in Stuttgart in
2009. The case is solid dark hardwood with heavily radiused end cheeks, oiled rather
than lacquered, so it has a soft sheen with visible grain rather than a mirror
finish. The faceplate is a flat sheet of aluminium in a warm off-white powder coat,
somewhere between bone and oyster, with a fine orange-peel texture at the micron
scale that scatters light and prevents it ever looking like paper. The graphics are
not printed. They are engraved through the coat and filled dark, which means the
letterforms have a physical depth of a few hundredths of a millimetre and catch a
shadow on their leading edge.

There are 160 buttons in the main matrix (ten rows of sixteen), a further set of
control buttons arranged in a logarithmic spiral on the right half, and two columns
of ten rotary encoders with polished chrome ball-tops. Every one of those chrome
domes is a hemispherical mirror. Beside most buttons sits a 2 mm LED behind a
slightly diffused lens.

Two details matter enormously and are almost always missed:

**The clefs.** At the far left of the faceplate, a treble clef sits at the top of the
track column and a bass clef at the bottom. They are not decoration and they are not
a logo. They tell you the matrix is a stave. The whole panel is arguing that this is
a musical instrument, not a machine, and that argument is made typographically. Get
those two glyphs wrong and the entire personality collapses.

**The spiral.** The mode, scale, transport, and chord controls on the right half are
laid out on a spiral rather than a grid. In 2009 this was a genuinely eccentric
choice and it is the thing that makes an Octopus recognisable from ten metres. It
also means there is **no grid to snap to**. Every one of those control positions has
to come from measurement. This is the single hardest geometry problem in the project
and it is why [Panel Truth](01-panel-truth.md) exists as its own discipline.

---

## 2. Why v1 read as generated, precisely

Not a vague critique. Four specific, diagnosable errors, all present in
`ui-mock/index.html`:

**E1. Reflection replaced by gradient.**
`--ball-lg: linear-gradient(165deg, #e4e0d8 0%, #c8c4bc 48%, #b0aca4 100%)`.
A gradient is a monotonic ramp along an axis. A chrome hemisphere under a real light
produces a hard specular caustic near the pole, a compressed reflection of the
horizon around the equator, a dark band where it reflects the panel it sits on, and
a bright fresnel rim at the silhouette. Four features, none of them monotonic. No
amount of tuning stops a two-stop ramp from reading as plastic, because plastic is
exactly what a two-stop ramp describes.

**E2. Uniform lighting.**
Every ball used the same gradient regardless of position on the panel. Real specular
highlights *migrate* across a field of spheres as you move away from the light. A
field of 160 identically-lit domes is the visual signature of a stylesheet, and the
human eye detects the repetition instantly even when it cannot name why.

**E3. Approximated geometry.**
`--step-size: clamp(12px, 1.9vw, 22px)` with `--step-gap: 3px`. The layout was
derived from what looked about right in a browser, so it drifts at every viewport
and never matches the object at any of them. The spiral was hand-placed in SVG.

**E4. Borrowed chrome.**
Google Sans, a 28px top bar, 7px uppercase letter-spaced status text, a "Pivot /
Tonality-inspired hairline" treatment. None of these are on the hardware. They are
the visual vocabulary of a 2024 web dashboard applied to a 2009 German instrument.
Every one of them is a tell.

The corrective is stated as N1 to N8 in `SPEC.md`. This document exists so that the
corrective is understood rather than merely obeyed.

---

## 3. Design plan

Per the two-pass method: plan, review the plan against the brief for defaults, then
build.

### 3.1 Colour

The palette is not chosen. It is **measured** from calibrated photography and stored
in `contracts/materials.json` as spectral-ish reflectance plus BRDF parameters. What
follows are the working names and their approximate linear-sRGB albedo, for
orientation only. The authoritative values come from the calibration pass.

| Token | Role | Approx. albedo | Notes |
|---|---|---|---|
| `panel.coat` | faceplate powder coat | 0.82, 0.80, 0.75 | warm off-white, µ-roughness 0.34, orange-peel normal map |
| `case.wenge` | case hardwood | 0.09, 0.07, 0.06 | anisotropic grain, oiled clearcoat lobe |
| `chrome.crown` | encoder ball-tops | F0 0.95 broadband | metal, roughness 0.045, no diffuse term |
| `chrome.dome` | matrix button domes | F0 0.93 | roughness 0.075, slightly duller than crowns |
| `engrave.fill` | engraved letterform fill | 0.045 | plus 0.03 mm depth in the height field |
| `led.red` | 620 nm | emissive | rise 4 ms, decay 38 ms |
| `led.green` | 525 nm | emissive | rise 4 ms, decay 42 ms |
| `led.amber` | red+green mix | emissive | mix ratio drives hue, do not fake with a third colour |
| `led.off` | unlit lens | 0.31 grey | a dark LED is still a plastic lens and still catches the room |

Note what is *absent*: there is no accent colour, no brand colour, no
`#D97757`-adjacent warm clay, no acid green on near-black. The instrument has no
accent colour because the hardware has no accent colour. The only saturated colour
anywhere in the product is emitted by an LED. This is the strictest and most
valuable constraint in the whole design system, and it will be tempting to break it
in the shell. Do not.

### 3.2 Type

Two registers, deliberately unrelated, because they describe two different eras.

**Panel register.** Must match the hardware. This is an identification problem, not
a taste problem, and it is specified as a measurable task in
[Panel Truth §5](01-panel-truth.md). The wordmark is a serif italic with an
old-style feel. The labels are a tight grotesque set at very small optical sizes.
Candidates to test by glyph-outline correlation against the plates, in order of
prior likelihood: for the wordmark, a Bookman or Century Schoolbook italic; for
labels, a Helvetica or Univers derivative. The gate is shape correlation, not
opinion.

**Shell register.** Must *not* match the hardware, and must not be a default.
Google Sans is banned (it is the Genie house face, and borrowing it is the same
error as E4). Inter is banned (it is the current AI-generated-page default).
The shell uses **one** family with real optical sizing and a genuine italic, set
mostly at large sizes with a lot of air. Direction: a contemporary grotesque with
slightly narrow proportions and unusually low contrast, so it sits beside the
engraved panel type without competing. The Curator selects and defends the choice
in an ADR, with three specimens rendered against the panel at real size before the
decision.

**Rule:** panel type never appears in the shell, and shell type never appears on
the panel. If you find yourself needing shell type on the panel, you are adding
something the hardware does not have, and the answer is to not add it.

### 3.3 Layout

The instrument is fixed-aspect and centred. It does not reflow, because objects do
not reflow. On a smaller window it gets smaller; it never rearranges. This is the
opposite of standard responsive practice and it is correct here.

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │  room, rendered
│    ╭──────────────╮                                         │  behind and around
│    │ Stairwell, 3am│ ▾                                      │  the instrument
│    ╰──────────────╯                                         │
│                                                             │
│                                                             │
│      ┌───────────────────────────────────────────────┐      │
│      │▓▓  ╔═══════════════════════════════════╗  ▓▓  │      │  the object,
│      │▓▓  ║ ○ ····························  ◉ ║  ▓▓  │      │  fixed aspect
│      │▓▓  ║ ○ ····························  ◉ ║  ▓▓  │      │  ~2.35:1
│      │▓▓  ║      matrix        encoders  spiral ║ ▓▓  │      │
│      │▓▓  ╚═══════════════════════════════════╝  ▓▓  │      │
│      └───────────────────────────────────────────────┘      │
│                                                             │
│                                                       ⚙     │
└─────────────────────────────────────────────────────────────┘
```

The instrument sits low and slightly forward of centre, the way a desk instrument
sits when you are standing over it. The room occupies everything else and is
genuinely three-dimensional: the panel is lit *by* what you can see behind it.
There is no top bar, no sidebar, no chrome, no card. Three affordances exist in the
whole shell, and two of them are hidden until you reach for them.

### 3.4 Principles

**P1. Simulate, do not stylise.** If a real-world mechanism explains the behaviour,
implement the mechanism. It is usually less code than the fake and always looks
better.

**P2. One light, one room, one truth.** A single probe lights everything. There are
no local fill lights, no per-element shadows, no ambient occlusion hacks tuned per
control. Consistency of illumination is what sells physicality.

**P3. The instrument is inert until touched.** Nothing on the panel animates on its
own except LEDs driven by the sequencer, and specular highlights driven by camera or
room change. No idle pulses, no breathing glows, no attract loops. The restraint is
the point.

**P4. Spend all boldness on the room.** The instrument is disciplined to the point
of severity. The room is where the product is allowed to be spectacular. One
memorable element, everything around it quiet.

**P5. Latency is a material.** A control that responds in 8 ms feels like metal.
The same control at 40 ms feels like software. Input-to-photon budget is a design
constraint of equal standing with colour.

### 3.5 Review of the plan against the defaults

Checked against the known cluster of generated-design tells:

- *Warm cream background with high-contrast serif and terracotta accent:* the
  faceplate is warm off-white, which is superficially adjacent. It survives because
  the value is measured from the physical object rather than chosen, and because
  there is no accent colour at all. Flagged for the Curator to re-check after the
  calibration pass: if the measured coat lands near `#F4F1EA` we state explicitly in
  the ADR that this is measurement, not fashion.
- *Near-black with one acid accent:* not used.
- *Broadsheet hairlines and zero radius:* explicitly rejected. Radii in this product
  come from the physical fillets of the hardware and nowhere else.
- *SaaS card kit:* impossible by construction. There are no cards.
- *Template chrome (all-caps eyebrows, middle-dot meta strings, arrows in buttons,
  monospace data labels):* all banned by the slop lint in
  [Verification §6](07-verification.md).

One default survived the first pass and was changed: the initial sketch had a
persistent bottom status strip showing tempo, room, and MIDI activity, in small
letter-spaced caps. That is E4 all over again. It was cut. Transport state is shown
by the hardware's own transport LEDs, which is where a musician looks anyway, and
the room name is only visible while the dropdown is open.

---

## 4. What "top 1%" means operationally

It does not mean more polish. It means these five things, which are measurable:

1. **Nothing is approximated.** Every number in the render traces to a measurement or
   a physical constant. Grep for magic numbers; there should be none in the render
   path.
2. **The hard cases are handled.** Sub-pixel geometry at every zoom level. Correct
   behaviour when 40 LEDs change state on one tick. Correct colour on a P3 display
   and on an sRGB one. Correct rendering at 200% and at 60%.
3. **The transitions are the product.** Anyone can make a static screenshot look
   good. The room cross-fade, the dropdown descent, the encoder detent, the LED
   decay: these are where quality actually lives, and they are the hardest to fake,
   which is why [Verification §4](07-verification.md) audits motion frame by frame
   against declared physical curves.
4. **It is fast.** 120 Hz sustained, 8 ms input-to-photon, and the sequencer never
   misses. Slowness is a visual defect.
5. **It knows what it is not.** No feature exists because it was easy. The list in
   `SPEC.md §4` is as important as anything we build.

---

## 5. A warning about the failure mode of this project

The likeliest way this project dies is not technical. It is that it produces four
thousand lines of beautiful renderer, an exquisite room system, and a sequencer that
drops notes, and then nobody wants to use it.

The mitigation is in the roadmap: **the vertical slice comes first**. Phase 1 ships
one track, sixteen steps, one room, one encoder, playing actual MIDI into actual
Ableton, at full quality, before phase 2 adds a second track. If the slice is not
good enough to make someone want the rest, no amount of the rest will save it.

Build narrow and finished before wide and rough. Every time.

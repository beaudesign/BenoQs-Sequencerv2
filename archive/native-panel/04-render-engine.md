# 04: Render Engine (`OctoPanel`)

**Owner:** Forge. **Gate:** `verify:geometry`, `verify:color`, `verify:frames`, `verify:motion`.

---

## 1. The central technique: trace the panel, do not rasterise it

The panel is a plane, about 200 spherical caps, about 200 small cylindrical LED
lenses, and an engraved height field. That is a trivially small scene. This is the
key observation, because it means we can afford the *exact* technique rather than the
approximate one.

**We ray trace it analytically, on the GPU, every frame.**

Rasterising spheres means tessellating them. Tessellation gives you a silhouette
error that is invisible at 100% zoom and glaring at 400%, which is exactly the
"looks cheap when you zoom in" failure. An analytic ray-sphere intersection has no
silhouette error at any zoom, ever, because there is no mesh. At 200 primitives on
Apple Silicon this costs less than the tessellation would have.

What this buys us, beyond exact silhouettes:

- **True inter-reflection.** Every chrome dome reflects the cream panel beneath it,
  the domes beside it, and the wooden case at the edges. This is the single biggest
  contributor to "that looks like metal" and it is essentially free once you are
  tracing: one bounce, 200 primitives.
- **Real contact shadows.** A shadow ray per light-important direction, giving the
  dark ring under each dome that a box-shadow can only imitate.
- **Correct specular migration.** Highlights move across the field of domes because
  each dome genuinely reflects a different part of the probe. This kills failure mode
  E2 from [North Star §2](00-north-star.md) by construction rather than by tuning.

### Pipeline

```
  compiled truth  ──►  primitive buffer (spheres, caps, cylinders, plane)
  materials.json  ──►  material buffer
  room probe      ──►  cubemap (radiance) + SH9 (irradiance) + prefiltered mips
  snapshot        ──►  per-control state (led target, encoder angle, press depth)
                          │
                          ▼
  ┌────────────────────────────────────────────────────────────┐
  │  1. animate      (compute)  physical models → control xform│
  │  2. build BVH    (compute)  refit only; topology is static │
  │  3. primary      (compute)  camera rays, analytic isect    │
  │  4. shade        (compute)  BRDF + reflection ray + shadow │
  │  5. engrave      (compute)  SDF relief, applied in shade   │
  │  6. emissive     (compute)  LED lens + physically-derived  │
  │                             bloom                          │
  │  7. resolve      (compute)  SSAA downsample, tonemap,      │
  │                             OETF, dither                   │
  └────────────────────────────────────────────────────────────┘
                          ▼
                    CAMetalLayer (EDR, Display P3)
```

The BVH topology never changes (controls do not move relative to each other) so it is
built once and only refit when press depth changes, which is negligible.

### Antialiasing: supersampling, never temporal

**TAA is banned.** Two reasons, both decisive:

1. It makes frames depend on previous frames, which destroys determinism and makes
   offline capture non-reproducible. Verification depends on determinism.
2. Non-negotiable N4 requires that static regions be *bitwise identical* between
   frames when nothing animates. TAA cannot satisfy that, because its jitter sequence
   guarantees they are not.

Instead: render at 3× linear supersampling (9× samples) and resolve with a
Mitchell-Netravali filter (B=1/3, C=1/3). At the scene's complexity this is
affordable, and it produces cleaner small-specular edges than TAA does anyway. Tiny
bright highlights on dark surroundings are the worst case for TAA and the best case
for SSAA.

Where SSAA is not enough (the very thin engraved letterforms at small zoom), the
engraving is antialiased analytically from its signed distance field, which is exact.

---

## 2. Materials

All materials are defined in `contracts/materials.json` and evaluated by one uber-shader
with a material index. There is no per-control shader code, ever.

### `chrome.crown` and `chrome.dome`

Metal. No diffuse term. GGX/Trowbridge-Reitz with Smith height-correlated visibility
and a Schlick fresnel over the measured `F0`. Roughness from Pass B.

Crucially, the specular integral is **not** taken from a prefiltered environment mip
for the crowns. Roughness is around 0.045, which is close enough to a mirror that
prefiltering visibly softens it into plastic. Instead we shoot a real reflection ray
per pixel into the scene, falling back to the probe cubemap on miss. Prefiltered mips
are used only for the duller `chrome.dome` at roughness 0.075, and only above a zoom
threshold where the difference is sub-pixel.

Anisotropy: the crowns are turned parts and retain a faint circumferential turning
mark. Model it as a tangent-space anisotropic roughness with the tangent field
following the lathe axis. This detail is small and it is exactly the sort of thing
that separates "very good" from "indistinguishable".

### `panel.coat`

Dielectric. Lambertian base plus a GGX clearcoat lobe at roughness 0.34. The
orange-peel micro-texture is **procedural**, generated from a band-limited noise with
a tuned power spectrum, evaluated in millimetre space so it is correct at every zoom
level and never tiles. A texture would either alias at high zoom or reveal its tile
at low zoom.

Peel amplitude is around 3 µm at a spatial period of roughly 0.3 mm. These values are
recovered in Pass B from the width of the specular sheen at grazing angles.

### `case.wenge`

Anisotropic dielectric with an oiled clearcoat. Grain direction runs along the long
axis of the case; the end cheeks have cross-grain. Grain is a procedural 1D noise
warped along the growth direction, driving both a subtle albedo variation and a
tangent-space roughness anisotropy. The clearcoat lobe is separate and much sharper
than the wood beneath, which is what makes oiled wood read as oiled rather than matte.

### `engrave.fill`

Not a texture and not a decal. A signed distance field at 64 px/mm, from the truth
file's vectorised outlines. Used three ways in one shader:

1. **Depth.** The SDF drives a 0.03 mm displacement in the height field, applied as
   relief mapping so it self-shadows and shifts correctly with view angle.
2. **Material blend.** Inside the glyph, swap to the dark fill material.
3. **Edge.** The SDF *gradient* gives the wall normal of the engraved channel, which
   is what produces the bright leading edge and dark trailing edge that makes real
   engraving legible. This is the detail that makes engraved type look engraved
   rather than printed, and it is one line of shader code.

### `led.red` / `led.green` / `led.amber` / `led.off`

An LED indicator is a plastic lens over a die. Model it as:

- an emissive term whose radiance is `target` after the temporal filter in §6,
- a dielectric lens on top, which still reflects the room even when the LED is off
  (this is why unlit LEDs are not just dark grey holes, and getting it wrong makes a
  panel look like a screenshot),
- a shallow subsurface term so the lens glows across its whole face rather than as a
  point.

**Amber is a mix of the red and green emitters**, at the ratio in `materials.json`,
because that is physically what the hardware does with a bi-colour LED. Do not
introduce a third emissive colour. The mixed result behaves differently under
different exposures than a synthesised orange would, and that difference is visible.

Bloom is derived from a measured point spread function, not from a gaussian blur
chain. A gaussian bloom is the second most common tell of a fake render after the
gradient. Use a PSF with the correct long tail (a sum of two exponentials fits ocular
scatter well) applied by FFT convolution at quarter resolution.

---

## 3. Lighting

**One probe. No lights.**

The panel is lit entirely by the current room's environment. There is no key light,
no fill, no rim, no per-control shadow tweak. This is what
[North Star P2](00-north-star.md) means and it is the discipline that makes the whole
thing hold together.

The probe arrives from `octoroom` as:
- a **radiance cubemap** at 256², RGBA16F, for reflection ray misses,
- **SH9 irradiance** for the diffuse term,
- a **prefiltered mip chain** for rough specular.

During a room transition, two probes are resident and the shader lerps between them.
The lerp is on radiance in linear space, weighted by the transition curve from the
motion registry. Because the reflection is genuinely traced, a cross-fade between two
probes reads as the room *changing around the instrument*, which is the entire point
of the product.

---

## 4. Colour management

One path, no exceptions.

```
material albedo (linear Rec.2020)
  → shading (linear Rec.2020, scene-referred, unbounded)
  → exposure (from room grade)
  → tonemap: AgX, custom-fit look
  → display transform → Display P3
  → OETF
  → triangular-PDF dither at 10-bit
  → CAMetalLayer (EDR headroom aware)
```

Notes:

- **AgX** rather than ACES RRT or Reinhard. ACES's RRT has a strong look that fights
  a neutral off-white panel and pushes the coat toward yellow. Reinhard desaturates
  highlights, which destroys the LED colours at the exact moment they matter. AgX
  keeps hue stable through the highlight roll-off, which matters enormously here
  because a saturated red LED clipping to white is the difference between an
  indicator and a blob.
- **EDR.** On a display with headroom, LEDs are rendered above diffuse white. A real
  red indicator LED next to a white panel *is* brighter than the panel. This is one of
  the highest-value-per-line features in the whole renderer and it is nearly free on
  Apple hardware.
- No `sRGB` hex literal ever reaches a shader. `verify:color` greps for the pattern
  and fails. Colours enter only through `materials.json`.
- Dither is mandatory. Undithered gradients on a large flat cream field produce
  visible banding on any 8-bit display, and a banded faceplate is instantly cheap.

---

## 5. Zoom, framing, and the "no reflow" rule

The instrument has a fixed aspect ratio derived from `panel.truth.json`. The camera
is orthographic by default with a slight optional perspective for the room shot.

Scale is continuous. There is no snapping to integer pixel ratios, and there is no
layout breakpoint. When the window changes size the object changes size, exactly like
an object.

Because the geometry is analytic and the engraving is an SDF, quality is scale
invariant by construction. `verify:geometry` asserts this at 4, 8, and 16 px/mm.

---

## 6. Motion is physics

**No animation in the panel may be authored as a bezier.** Every moving quantity has
a physical model with parameters in `contracts/motion.registry.json`.

### Buttons

Mass-spring-damper with a hard travel stop.

```
m ẍ + c ẋ + k x = F_finger(t)
x ∈ [0, travel]           travel ≈ 0.6 mm
```

`F_finger` comes from `ButtonDown.velocity_mm_s`. Release is the spring alone.
Critically damped is wrong; real tactile switches are slightly underdamped and have a
faint overshoot on release. Set ζ ≈ 0.7 and let the overshoot happen. Nobody will
consciously notice it. Everybody will notice its absence.

### Encoders

Rotational inertia with detent torque and viscous damping.

```
I θ̈ + b θ̇ + τ_detent(θ) = τ_hand(t)
τ_detent(θ) = -A sin(N θ)          N = detents per revolution
```

This gives, for free and without any special-casing: the encoder settling into a
detent when released mid-way, the slight resistance felt as a visual hesitation when
crossing a detent, and momentum carry when spun fast. A lerp-to-target cannot produce
any of these.

### LEDs

First-order with asymmetric time constants, per [Sequencer §6](03-sequencer-core.md):

```
τ_rise ≈ 4 ms      τ_decay ≈ 38–42 ms depending on emitter
```

Integrated at render rate with an exact exponential step (not an Euler step), so the
result is frame-rate independent and identical in a 120 Hz live session and a 240 Hz
offline capture. This is required for `verify:motion` to mean anything.

The asymmetry is the whole effect: LEDs snap on and fade off. A symmetric fade looks
like software; the asymmetry is what makes a chase light read as a chase light.

### The registry

Every animated quantity is declared:

```jsonc
{
  "id": "button.press",
  "model": "mass_spring_damper",
  "params": { "m": 0.8, "k": 2400, "zeta": 0.7, "travel_mm": 0.6 },
  "settle_ms": 42,
  "tolerance": { "position_mm": 0.005, "curve_rmse": 0.01 }
}
```

`verify:motion` renders the transition offline, extracts the actual position over
time from the frames, integrates the declared model independently, and fails if the
RMSE exceeds `tolerance.curve_rmse`. **This is what "flawless frame to frame"
means mechanically**: not that someone looked at it and liked it, but that the pixels
provably follow the declared physics to within 1%.

---

## 7. Input as actuation

Input events are not clicks. They carry the physical quantities the motion models
need.

- Pointer press → `ButtonDown { velocity_mm_s }`, derived from pointer speed at
  contact, clamped to a plausible range.
- Pointer drag on an encoder → `EncoderTurn { detents, angular_velocity }`, where the
  drag maps to torque, not to angle. Dragging an encoder should feel like pushing it,
  including being able to spin it and let go.
- Trackpad scroll on an encoder → torque impulses, honouring momentum scrolling so
  that a flick genuinely spins the encoder.
- Keyboard and MIDI controller input bypass the physical model for the *core* but
  still drive it for the *renderer*, with a synthetic actuation velocity, so a
  hardware-controlled button still visibly depresses.

Hit testing uses the same ray as the renderer, against the same primitives, so what
you can click is exactly what you can see. No invisible rectangular hit boxes over
round buttons.

---

## 8. Debug surfaces

Behind a flag, and never shipped:

| Overlay | Shows |
|---|---|
| `truth` | truth-file positions as crosshairs over the render |
| `residual` | per-control geometry error, colour-mapped |
| `budget` | per-pass GPU timings against budget |
| `probe` | the current environment probe as a mirror ball in the corner |
| `motion` | live plot of a chosen control's position against its declared model |
| `queue` | command ring depth, dropped-command counter, snapshot generation lag |

The `motion` overlay in particular pays for itself in the first week. Being able to
watch the actual curve against the intended curve turns "that feels a bit off" into a
number in about four seconds.

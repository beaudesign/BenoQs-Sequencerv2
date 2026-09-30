# 01: Panel Truth

**Owner:** Panelwright. **Gate:** `verify:geometry`, `verify:color`, `verify:type`.
**Artifact:** `contracts/panel.truth.json` (frozen after ratification).

---

## 1. The methodological problem, and the fix

The instinct for "pixel perfect replica" is to render a frame, diff it against the
reference photograph, and minimise the error. This does not work, and understanding
why is the foundation of everything below.

A photograph of the Octopus contains, mixed together and inseparable by inspection:

- the true geometry of the object,
- the lens's radial and tangential distortion,
- the perspective of wherever the photographer stood,
- the specific lighting of that studio on that day,
- the camera's colour science and white balance,
- JPEG artefacts,
- and, in a product shot, retouching.

Optimising a render to match that image drives the renderer toward reproducing the
*photograph* rather than the *object*: you end up baking one studio's key light into
your material model, and then everything falls apart the moment the room changes,
which in this product is constantly.

So we split the problem in two, permanently:

> **The photograph is used exactly twice: once to derive geometry, once to calibrate
> materials. After that it is never compared against a render again.**

**Pass A, geometry.** From calibrated plates, recover a metric vector model of the
faceplate in millimetres. Human-ratify it once. Freeze it as
`contracts/panel.truth.json`. From then on, geometry verification is
render-versus-truth-file, which is deterministic, fast, lighting-independent, and
runnable ten thousand times a day.

**Pass B, photometry.** From colour-managed plates with a known chart in frame,
recover per-material BRDF parameters. Freeze into `contracts/materials.json`.
From then on, colour verification is render-under-known-probe versus predicted
radiance, which is again deterministic.

This is the difference between a project that can iterate and one that cannot.

---

## 2. Coordinate system

Right-handed, millimetres, origin at the **top-left corner of the visible faceplate
aperture** (the inside corner of the wooden bezel, not the outer edge of the case).

- `+x` right, `+y` down, `+z` out of the panel toward the viewer.
- The faceplate surface is `z = 0`. Engraving is negative `z`. Domes and crowns are
  positive `z`.
- All angles in degrees, counter-clockwise positive when viewed from `+z`.
- The full panel extent is recorded as `panel.aperture = {w, h}` and every other
  coordinate is absolute, never relative. **No normalised coordinates anywhere.**
  Normalisation is how drift enters.

Rationale for the aperture rather than the case: the aperture edge is a crisp,
machine-cut, high-contrast feature that a corner detector locks onto reliably across
plates. The wooden case edge is radiused and its apparent position shifts with
lighting.

---

## 3. Pass A: geometry recovery

### 3.1 Plate requirements

Existing web photography is sufficient for a first ratification but not for final.
The Panelwright commissions or sources plates meeting:

| Property | Requirement | Why |
|---|---|---|
| Framing | Full faceplate, ≥ 4000 px on the long edge | 0.15 mm/px at 700 mm width |
| View | Orthographic-ish, camera axis within 3° of panel normal | limits homography extrapolation error |
| Lens | Known model, or a checkerboard calibration set from the same body | for undistortion |
| Scale anchor | A ruler or gauge block in-plane, or a stated case width | metric scale |
| Colour | X-Rite ColorChecker in frame, same illuminant | Pass B |
| Format | Raw or 16-bit TIFF, no sharpening, no retouch | sharpening moves edges |
| Count | ≥ 3 plates from different angles | cross-validation |

If final plates cannot be obtained, the fallback is documented in
[Risks R3](10-risks-and-decisions.md), and the geometry tolerance in the definition
of done relaxes from 0.35 mm to 0.60 mm p95, with the relaxation recorded in the
ADR rather than quietly assumed.

### 3.2 Pipeline

Implemented as `harness/measure/geometry/extract.py`. Runs once per plate, outputs a
candidate truth file plus a residual report.

**Step 1. Undistort.** Apply the lens model. If no calibration set exists, estimate
radial distortion by enforcing straightness on the four aperture edges and the two
long engraved rules, then report the estimated `k1`, `k2` for review.

**Step 2. Rectify.** Detect the four aperture corners subpixel (Harris, then
saddle-point refinement). Solve the homography to a fronto-parallel plane. Resample
with Lanczos-3 at a fixed 8 px/mm working resolution.

**Step 3. Scale.** Anchor to the scale reference. If using stated case dimensions,
record the source and its uncertainty in `truth.provenance`.

**Step 4. Detect controls.** Chrome domes on a light coat are low-contrast overall
but have a strong, reliable signature: a bright specular core surrounded by a dark
contact shadow ring. Detect by:
1. band-pass filter at the expected dome radius,
2. Hough circle transform with tight radius priors per control class,
3. per-candidate refinement by fitting an ellipse to the contact-shadow annulus
   (the shadow is a better centre estimate than the specular core, which migrates
   with the light),
4. reject candidates whose fitted eccentricity exceeds 0.15 after rectification,
   since a true circle should rectify to a circle.

**Step 5. Classify.** Cluster fitted radii. Expect four distinct populations:
matrix button domes, MIX-target buttons, spiral buttons, encoder crowns. Assign
class by radius, then verify by expected count (see §4).

**Step 6. Regularise the matrix.** The 160 matrix buttons are on a true rectangular
lattice. Fit a 2-parameter lattice (pitch-x, pitch-y) plus origin and a small shear
term by total least squares over all 160 detections, then **replace** the individual
detections with the lattice prediction. This is important: the lattice is more
accurate than any individual detection, because it averages 160 measurements into
4 parameters. Record per-button residuals; any residual above 0.4 mm indicates a
misdetection to review, not a real irregularity.

**Step 7. Fit the spiral.** The right-hand cluster is not a lattice. Fit a
logarithmic spiral `r(θ) = a·e^(bθ)` in the least-squares sense over the detected
control centres, then record each control as `(θ, r)` **plus** a residual offset
`(dx, dy)`. Do not force controls onto the fitted curve. The fit exists to expose
the design intent and to make interpolation sane; the residuals are real and are
part of the object's character. A perfect spiral would look wrong.

**Step 8. Vectorise the engraving.** Threshold the engraved fill, extract contours,
simplify with Visvalingam at a 0.02 mm area threshold, and emit closed paths. These
become the height-field source for the engraving material. Text is *not* re-typeset
from a font at this stage; it is stored as outlines, so the engraving is
pixel-faithful even before the type identification in §5 completes.

**Step 9. Cross-validate.** Run steps 1 to 8 independently per plate. Compare the
resulting truth files pairwise. Report per-control disagreement. Any control whose
inter-plate standard deviation exceeds 0.25 mm is flagged for manual adjudication.
Final values are the inverse-variance-weighted mean.

### 3.3 Ratification

The candidate truth file is not usable until ratified. Ratification is a **human
gate**, once, and it is the only human gate in the whole verification system.

The Panelwright produces `reference/ratification.html`: the rectified plate with the
truth file overlaid as vector annotations, at 8 px/mm, scrollable and zoomable, with
every flagged control listed. A human confirms or corrects. On confirmation the file
is written with `"status": "ratified"`, a SHA-256 of the plates it derived from, and
a date. From that moment `contracts/panel.truth.json` is **immutable except by ADR**.

Why a human gate here and nowhere else: everything downstream is measured against
this file, so an error here is invisible forever and poisons every subsequent
metric. It is worth exactly one careful hour.

---

## 4. Expected inventory

The Panelwright reconciles detections against this inventory. A mismatch means the
detector is wrong, not the inventory. Counts marked *(derived)* come from the CE
v5.30 manual; counts marked *(to measure)* must be established in Pass A and this
table updated by ADR.

| Class | Count | Source |
|---|---|---|
| Matrix step buttons | 160 (10 rows × 16 cols) | *(derived)* rows numbered 9 at top to 0 at bottom |
| Matrix step LEDs | 160 | one per step button |
| Track MIX encoders | 10 | *(derived)* one per track, left column |
| Attribute EDIT encoders | 10 | *(derived)* VEL PIT LEN STA POS DIR AMT GRV MCC MCH |
| MIX TARGET buttons | 16 | *(derived)* row below the matrix |
| Row index engravings | 10 | `9` … `0`, top to bottom |
| Column index engravings | 16 | `1` … `16`, above the matrix |
| Clef engravings | 2 | treble (top left), bass (lower left) |
| Spiral control buttons | *(to measure)* | mode, scale, transport, chord, edit, and system controls |
| Main rotary encoder | 1 | centre of the spiral cluster |
| Wordmark engraving | 1 | `octopus`, serif italic, upper right |
| Maker engraving | 1 | `genoQs Machines`, upper left, in a rule box |

**Named spiral controls to locate and label** (from manual frequency analysis;
the Panelwright maps each to a measured position and records unmatched detections
as `unknown_N` for adjudication rather than guessing):
`Step Mode`, `Track Mode`, `Page Mode`, `Grid Mode`, `Scale`, `Chord`, `Play`,
`Stop`, `Record`, `Clear`, `Copy`, `Paste`, `Program`, `Snapshot`, `Shift`,
`Follow`, `Rotate`, `Solo`, `Mute`, `Bank`, `Song`, `Pattern`, `Merge`, `Store`,
`Reset`, `Preset`, `Esc`.

---

## 5. Type identification

The panel's letterforms must be identified, not chosen. Procedure:

1. Extract engraved glyph outlines for a target string with good discriminating
   characters. Use `genoQs Machines` (has `Q`, `g`, `M`, `a`) and `octopus`
   (italic `o`, `c`, `t`, `p`, `s`).
2. For each candidate face, typeset the same string, normalise both to the same cap
   height, and align by centroid.
3. Score by **IoU of filled outlines** plus **Hausdorff distance of contours**, per
   glyph and aggregated.
4. Gate: the selected face must score aggregate IoU ≥ 0.94 with no single glyph
   below 0.90. If no candidate clears it, the fallback is to ship the extracted
   outlines directly as vector art and mark the face `unidentified`, which is an
   acceptable outcome and costs nothing visually.

Candidate list, ordered by prior likelihood for a 2009 German instrument:
wordmark: ITC Bookman Light Italic, Century Schoolbook Italic, ITC Garamond Italic,
Palatino Italic. Labels: Helvetica, Univers 55, Frutiger, Akzidenz-Grotesk.

Record the result in an ADR with the score table. This takes an afternoon and
removes a permanent source of "something is off" that nobody can name.

---

## 6. Pass B: photometry

Colour must be recovered under a *known* illuminant so it can be re-rendered under
an *arbitrary* one. That is the whole reason we do BRDF fitting rather than eyedropper
sampling.

### 6.1 Procedure

1. From the plate with the ColorChecker, compute the camera-to-XYZ matrix and the
   scene illuminant. Convert the plate to linear XYZ, then to linear Rec.2020 as
   the working space.
2. Define 14 **material patches**: regions of the rectified plate that are
   unambiguously one material and one geometry class. Store their polygons in
   `contracts/materials.json` so the measurement is reproducible.
3. For diffuse-dominant materials (`panel.coat`, `case.wenge`, `engrave.fill`,
   `led.off`), solve albedo directly given the estimated illuminant and the surface
   normal from the truth file.
4. For metals (`chrome.crown`, `chrome.dome`), albedo is meaningless. Instead fit
   `(F0, roughness)` by inverse rendering: given the recovered environment (estimated
   from the specular reflections in the largest crowns, which act as light probes,
   this is standard mirror-ball probe recovery run in reverse), render a candidate
   dome and minimise residual against the plate. Roughness is well-constrained by
   the *angular width* of the specular lobe, which is robust to exposure error.
5. For LEDs, plates of the powered device under controlled exposure are required.
   If unavailable, use published emission spectra for standard 620 nm / 525 nm
   indicator LEDs, convert to the working space, and mark the values `modelled`
   rather than `measured` in the file. Be honest in the metadata; downstream
   tolerance depends on it.

### 6.2 Gate

`verify:color` renders each of the 14 patches under the *recovered* illuminant and
compares to the plate in CIELAB. Mean ΔE2000 ≤ 2.0, max ≤ 4.0. This is a genuinely
tight tolerance (ΔE 2.0 is around the threshold where a trained observer notices a
difference in a side-by-side) and it is achievable because we are comparing under
the same lighting the measurement came from.

---

## 7. The truth file

Schema at `contracts/panel.truth.schema.json`. Shape:

```jsonc
{
  "schema": "panel.truth/1",
  "status": "ratified",
  "provenance": {
    "plates": ["sha256:…", "sha256:…"],
    "scale_source": "case width 700 mm, genoQs product sheet",
    "scale_uncertainty_mm": 1.5,
    "ratified_by": "…", "ratified_at": "2026-…"
  },
  "units": "mm",
  "aperture": { "w": 0.0, "h": 0.0 },
  "lattices": {
    "matrix": {
      "origin": [0,0], "pitch": [0,0], "shear": 0.0,
      "rows": 10, "cols": 16,
      "row_labels": ["9","8","7","6","5","4","3","2","1","0"],
      "col_group": 4
    }
  },
  "spiral": { "a": 0.0, "b": 0.0, "center": [0,0], "theta0": 0.0 },
  "controls": [
    {
      "id": "matrix.r9.c0",
      "class": "matrix_button",
      "center": [0,0], "radius": 0.0, "z": 0.0,
      "dome": { "profile": "spherical_cap", "height": 0.0 },
      "led": { "center": [0,0], "radius": 0.0, "colors": ["red","green","amber"] },
      "residual_mm": 0.0
    }
  ],
  "engravings": [
    { "id": "wordmark", "paths": [[[0,0],[0,0]]], "depth_mm": 0.03,
      "typeface": "unidentified", "outline_source": "plate" }
  ]
}
```

Rules:

- Every renderable feature has a stable `id`. IDs are the join key between the truth
  file, the motion registry, the state model, and the verification report. Renaming
  an id is an ADR-level change.
- `residual_mm` travels with the data so that downstream code can weight or flag
  low-confidence controls.
- The file is checked in as formatted JSON with sorted keys, so diffs are readable.

---

## 8. Geometry verification

`verify:geometry` runs on every commit that touches the render path:

1. Render the panel head-on, orthographic, at exactly 8 px/mm, flat-lit with a
   synthetic probe designed for detectability rather than beauty.
2. Run the **same detector** from §3.2 steps 4 to 6 over the render.
3. Match detections to truth by nearest-id, and compute per-control centroid error.
4. Report p50 / p95 / max in millimetres, plus a per-class breakdown and a heat map.

Gate: p95 ≤ 0.35 mm, max ≤ 0.80 mm.

Reusing the detector on both sides is deliberate: any bias in the detector cancels,
so the metric measures the renderer rather than the detector.

Additionally, a **zoom-invariance** check: render at 4, 8, and 16 px/mm and confirm
that recovered geometry agrees within 0.05 mm across scales. This catches the entire
class of bugs where something is snapped to the pixel grid at one zoom and drifts at
another, which is the specific defect that makes a UI feel cheap when you resize it.

---

## 9. Handing the truth file to the renderer

The renderer never parses JSON at runtime. A build step compiles
`panel.truth.json` into:

- a **GPU instance buffer** of control transforms, one entry per control,
- a **signed distance field atlas** for the engraving, at 64 px/mm, so letterforms
  stay crisp at any zoom without re-tessellation,
- a **material index** per control referencing `materials.json`,
- a **height field** for the panel's micro-relief (orange peel plus engraving depth).

The compiler asserts that every id in the motion registry and the state model exists
in the truth file, and fails the build otherwise. This is how the four contracts stay
in sync without anybody remembering to keep them in sync.

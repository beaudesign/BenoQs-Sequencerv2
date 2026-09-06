# Reference plates — what each one is good for

## `web-frontal-01.jpg`

Frontal product photo of a real Octopus, sourced from a retailer/gear-review site
(watermarked "greatsynthesizers.com"), not shot for this project.

**What it can be used for:** a visual cross-check of control *topology and
counts* against `docs/01-panel-truth.md` §4's expected inventory — confirms the
10×16 matrix, the left-side MIX encoder + SEL button column, the 10 EDIT
encoders, the spiral cluster with its MODE/SCALE/TRANSPORT/CHORD regions, and the
wenge side cheeks with the top cable channel. This is a relative/counting check,
not a metric one.

**What it cannot be used for, and why:** any input to `contracts/panel.truth.json`,
at either the normal (p95 ≤ 0.35mm) or the ADR-relaxed (p95 ≤ 0.60mm, per R3)
tolerance. Per `docs/01-panel-truth.md` §3.1, a usable plate needs: a known lens
model or its own checkerboard calibration set, an in-plane scale anchor with
stated uncertainty, an X-Rite ColorChecker in-frame under the same illuminant, and
raw/16-bit capture with no sharpening. This photo has none of those. It is also
a single angle, when even the relaxed fallback path's own §3.2 Step 9
(cross-validation) needs ≥2 plates to check inter-plate agreement.

The geometry extraction pipeline (`harness/measure/geometry/extract.py`, not yet
built) must not be run against this plate — doing so would produce a candidate
truth file with no honest path to ratification, and ratification (§3.3) is a
one-time human gate that shouldn't be spent on an input known in advance to fail.

**Still needed:** ≥3 calibrated plates from different angles, meeting the full
§3.1 table, before `contracts/panel.truth.json` can be produced at all (relaxed
tolerance) or ratified for real (full tolerance). Tracked in `journal/STATE.md`.

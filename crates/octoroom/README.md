# octoroom

Room system (D3). Owner: Sceneshaper. See `docs/06-shell-and-rooms.md` §3-4
for the room contract and pipeline this crate implements.

## What's here

- `intent`: `RoomIntent`, the typed struct pipeline step 1 ("structured
  extraction") would produce from a room prompt, plus a hand-authored
  instance of docs/06 §3's own worked example (`RoomIntent::stairwell_3am`).
- `geometry`: pipeline step 2 ("geometry synthesis") — real, deterministic,
  seeded. Produces `bounds_m`/`volume_m3`/a `surfaces` list matching the room
  contract's shape, with per-band absorption from a small material lookup
  table.
- `acoustics`: the Eyring reverberation-time prediction docs/06 §3 names as
  the self-check for `verify:acoustics` ("if they disagree by more than 10%,
  the tracer has a bug").
- `rng`: a small seeded PRNG, duplicated from `crates/octocore/src/rng.rs`
  rather than depended on — see that module's doc comment for why.

## What's deliberately not here

- **Structured extraction from real prose.** No LLM call exists in this dev
  environment (no network access). `RoomIntent` instances are hand-authored.
- **The optical bake** (path-traced cubemap probe, SH9, prefiltered mips) —
  Metal compute, and this environment has no Xcode/Metal toolchain.
- **The acoustic bake** (image-source method + stochastic ray tracing to
  produce a real impulse response) — also Metal compute. `acoustics`'s Eyring
  prediction is the *check* a real tracer's output would be validated
  against, not a substitute for the trace itself.
- **Grade derivation** (light CCT/exposure -> lift/gamma/gain) — not attempted
  since it depends on the optical bake's actual output.
- **Real wall segmentation.** `geometry::synthesize_geometry` produces a
  simplified floor/ceiling/walls decomposition, not window cutouts, the
  handrail in the stairwell prompt, etc.
- Material absorption/scattering/albedo/roughness values beyond
  `concrete_bare` (docs/06 §3's own cited numbers) are typical published
  architectural-acoustics orders of magnitude, not measured or cited to a
  specific source — flagged in `geometry.rs`'s `material_properties` doc
  comment.

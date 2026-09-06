# ADR 0002: Supersede v1, Adopt WENGE

## Status

Accepted. Supersedes ADR-0001.

## Context

Two prior efforts exist in this repo:

1. The original Max4Live JS implementation (`octopus_*.js`, `Octopus.amxd`,
   `octopus_ui_main.maxpat`, `ui-mock/`). Its panel was styled with CSS, including
   `linear-gradient` chrome, and its layout was hand-eyeballed against reference
   photography rather than measured.
2. ADR-0001's "portable C++ core with adapters" refactor (`src/core/`,
   `src/juce/`, `CMakeLists.txt`, `tests/core_tests.cpp`). This moved sequencing
   logic out of Max into a standalone C++ core with JUCE as an adapter, but kept
   the JS/CSS panel rendering unchanged and did not address panel geometry or
   material fidelity at all.

`SPEC.md` (Codename WENGE) defines a ground-up replacement: `octocore` (Rust,
no_std-friendly, WASM-compilable), `octopanel` (Swift + Metal, physically
rendered), `octoroom` (Rust + Metal compute), and `octoshell` (Swift). It states
plainly: "Supersedes: `beaudesign/BenoQs-Sequencer` (archive, do not merge)."

Neither prior effort can satisfy SPEC.md's non-negotiables:

- **N1** (panel measured, not drawn) — both efforts hand-placed panel geometry
  from photography/eyeballing; neither has a `panel.truth.json` derived from
  calibrated measurement.
- **N2** (no material is a gradient) — the JS/CSS mock's chrome is a literal
  `linear-gradient`, the exact category error SPEC.md §1 identifies as the root
  cause of the "AI slop" look.
- **N3** (motion is simulated, not eased) — neither implementation models
  rotational inertia, detent torque, or spring-damper button travel; both use
  eased/keyframed animation.
- **D1** (the core is Rust, no_std-friendly, WASM-compilable) — ADR-0001's core
  is C++, not Rust, and was never built against a WASM target.

## Decision

WENGE supersedes both prior efforts. Neither is merged into the new
architecture. Both are preserved under `archive/`, not deleted:

- `archive/v1-max4live/` — the original Max4Live JS implementation.
- `archive/v1-cpp-juce/` — ADR-0001's C++ core and JUCE adapter, including its
  own ADR at `adr/0001-portable-cpp-core-with-adapters.md`.

Both may be consulted as a *behavioral* reference when porting Octopus
behavior into the new `octocore`, per
[`docs/03-sequencer-core.md` §4](../docs/03-sequencer-core.md). That section is
explicit that porting a v1 test alongside v1 code launders the original
author's misreadings into the new codebase — so any numeric constant carried
over from either archive (it names `_grvDelayTicks` and `_strumOffsetTicks` as
known risks) must still be re-derived and cited against the real CE v5.30
reference manual before a conformance fixture is allowed to depend on it. The
archived code is not a source of truth; the manual is.

## Consequences

- Repo root is clean of v1 artifacts; SPEC.md §5's target tree can be built
  without pre-existing files in the way.
- No `git blame` history is lost — everything moved via the filesystem and is
  re-added under its new path in this commit.
- Panelwright and Metronome still have zero real reference material to work
  from (no manual, no calibrated photography) — that gap is unaffected by this
  ADR and is tracked separately in `reference/NOTES.md` and `journal/STATE.md`.
- Any future agent tempted to "just port the JS" or "just port the C++ core"
  now has to cross an explicit ADR to do it uncritically.

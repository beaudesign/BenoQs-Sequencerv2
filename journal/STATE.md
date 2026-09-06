# STATE

Capped at 200 lines. Conductor prunes rather than appends when it grows past
that. Last pruned: 2026-09-06 (rewritten after the manual landed — most of
the previous version described a "no manual yet" world that no longer exists).

## Phase

**Phase 0 (the ratchet) — scaffolding done, the ratchet itself not built.**
No real gate exists yet except the umbrella structure and `octocore`'s own
`verify-octocore`/`verify-conformance`. Per `docs/09-roadmap.md`, Phase 0 exit
criteria (`just verify` green under 4 min, `verify:determinism` passing,
placeholder renderer discriminating gates, report rendering, eight worktrees
through the merge queue) are **not met**. Per "Sequencing notes", `octocore`
work is explicitly allowed to proceed in parallel during Phase 0 since it has
no rendering dependency — that's exactly what's landed.

## Reference material: manual present, photography still one insufficient plate

- `reference/manual/` has the real CE v5.30 manual, page-indexed
  (`reference/manual/INDEX.md`, `just manual <topic>`), including a bundled
  2007 tutorial series (`pages/tutorial-01..10.txt`) that resolved a design
  question the main chapters didn't cover on their own (custom direction
  multi-trigger timing).
- `reference/plates/web-frontal-01.jpg` — one uncalibrated frontal photo.
  Confirmed useful for a topology/count sanity check only (see
  `reference/plates/NOTES.md`); does **not** unblock `contracts/panel.truth.json`
  at any tolerance. Still need ≥3 calibrated plates per `docs/01-panel-truth.md`
  §3.1 before any real geometry work.
- `contracts/panel.truth.json`, `motion.registry.json`, `materials.json`,
  `room.schema.json` — still none exist. Deliberately not fabricated (N1/N2).
- No renderer exists (`apps/OctoPanel` is an empty scaffold) — `harness/capture`
  has nothing to drive yet.
- Xcode (full) is still not installed here — Metal, signing, VST3/AU/
  octopanel/octoshell builds remain out of reach in this dev environment.

## `crates/octocore`: real, tested, now manual-corrected

Built in one session (no manual), then corrected against the real manual in a
second pass — 17 commits, each with a real `Ref:` citation, `just verify`
green throughout. Full detail lives in the commit messages
(`git log --oneline` from `22230c0` through `6cc99e0`) and
`tests/conformance/AMBIGUITIES.md`, not repeated here. Headline points:

- Several bugs the pre-manual pass got **outright wrong**, not just
  unconfirmed: default track pitches (octave + direction both flipped), the
  effector (missing MCC, wrong same-tick timing model, no listener gating),
  chord polyphony's random-pick algorithm, FLT (modeled as a bool attribute —
  it's actually a multi-track merge operation), phrase types (Reverse did
  nothing, RandomAll invented fake values), hypersteps (modeled as
  hold-to-apply — it's a persistent link), custom directions (fell back to
  uniform random — real mechanism is 16 slices × up to 9 ordered triggers +
  a certainty_next probability, confirmed via the manual's own bundled
  tutorial).
- Two tables ported from the archived v1 JS turned out **cell-for-cell
  correct** (GRV shuffle, chord strum) — citation-only commits, no logic
  change.
- 43 unit tests + 2 conformance fixtures, all green, debug and release.
- Deliberately **not** attempted: the attribute-map-factor step-event
  sub-system (VEL/PIT/LEN/STA/AMT/GRV/MCC step events — real, confirmed, too
  underspecified to implement safely without more transcription work); the
  LEN/STA/generic attribute scaling lookup tables (confirmed real and
  non-linear, large enough that hard-coding them now risked a wrong digit in
  a *cited* constant); genuine same-tick step-event application (the
  manual's real rule is asymmetric same-tick/next-tick by track index; this
  engine applies everything uniformly next-tick because same-tick would need
  reworking the tick loop's per-tick snapshot architecture); Track
  Rotate/Skip Rotate's actual step-shifting behaviour (data model only);
  hyperstep's fine-grained LEN-scaling curve (only the binary 12↔192 case is
  implemented); MCC sub-step CC interpolation.
- All of the above are logged with page citations in
  `tests/conformance/AMBIGUITIES.md` for whoever picks this up next — most
  entries there now name a specific manual page to re-read, not "no manual".

## `crates/octoffi`: thin C ABI wrapper, verified with real linked C

new/free/handle_command/render/is_running over an opaque `*mut Engine`.
Verified with an actual compiled-and-linked C program against the built
`.dylib`, not just Rust tests. No Grid-mutation surface over FFI yet (needs
`panel.truth.json`'s `ControlId` scheme first). Hand-maintained `octoffi.h`
(no `cbindgen` here) — will drift if `Command`/`Event` change shape without a
matching header edit.

## Fan-out

Still mostly premature — the harness/renderer/photography/Xcode blockers
above are unchanged. What can safely proceed in parallel right now:
`octoffi`'s Grid-mutation surface (once someone wants to start the Swift
side against a `cdylib`); a follow-up manual-reading pass to lift the
deferred items above (scaling tables, map-factor step events, Rotate); or
starting `octoroom`'s pure-Rust parts (RoomIntent, Eyring RT60 self-check —
no Metal needed). Everything touching the panel itself still depends on real
calibrated photography.

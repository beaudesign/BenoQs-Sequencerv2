# SPEC-0002: Refocus on a web sequencer, and remove the world-model scope

| Field | Value |
|---|---|
| Spec | SPEC-0002 |
| Revision | r0, draft, 2026-09-30 |
| Stage | **Draft for the owner. Not approved. Nothing has been removed, changed or built.** |
| Risk tier | High. It reverses closed decisions in `docs/10-risks-and-decisions.md` section 3 (native app, analytic ray tracing, VST3/AU) and supersedes most of `SPEC.md`. |
| Owner and approver | `@beaudesign` |
| Audited commit | `fcf1aec` (`main`) |
| Task | `WENGE-0013` (`handoffs/WENGE-0013.ndjson`) |

## The ask

Recorded here because chat is not a source of truth. On 2026-09-30 (12:58 Paris) the owner
gave three sources as "the main feature and UI design directions": the Octopus v5.30 release
notes, the Octopus v5.30 reference manual, and a blog post on basic navigation
(`djjondent.blogspot.com`, "genoQ Octopus basic navigation"). The owner said a "world model"
project had slipped into this repo, asked for a plan or spec to remove its attributes, and
asked that the product stay "really focused" as **a web app that runs the sequencer with live
actual MIDI data and external instruments, also available on Ableton**.

## What this spec proposes

1. **The product** is a web sequencer that looks and behaves like the Octopus (CE OS v5.30):
   the same modes, gestures and LED language, playing real MIDI to external instruments and
   working with Ableton Live. `product.md`.
2. **Remove the world-model scope**: the rehearsal rooms, the Genie-style shell, the light
   probe, the impulse response and the acoustic bake, with the `octoroom` crate, `docs/06`,
   the Sceneshaper and Loom roles, the `acoustics` gate and their ratchet tests.
   `removal-plan.md`.
3. **Retire the native and photoreal panel path** (Metal, Swift, VST3 and AU wrappers,
   ray-traced chrome, the photogrammetry truth file) unless the owner keeps some of it
   (decision D3).
4. **Keep the engine and the factory**: `octocore` (it already runs as WebAssembly), `octorun`,
   the conformance suite, the ratchet, `reference/manual`, and the roles and lifecycle in
   `AGENTS.md`.
5. **Build what is missing** (findings below): a front-panel controller that implements the
   manual's interaction model, Web MIDI in and out with clock, and the routes into Ableton.
   `tech.md`.
6. **Make the removal a test**: a `verify:scope` gate that fails when world-model terms
   appear in the live tree, so the confusion cannot return.

## Findings that shape the plan

Details and evidence are in `findings.md`.

- **F1. The engine has no front panel.** `Command::ButtonDown`, `ButtonUp` and `EncoderTurn`
  are accepted and ignored (`crates/octocore/src/engine.rs`, `handle_command`), and nothing
  writes the LED snapshot. The manual's modes, hold and double-click gestures, EDIT states
  and LED colours (about 140 workflows in the two digests) exist nowhere in code. **That
  layer is the product**, and the old plan never budgeted it: it spent its detail on
  rendering and rooms.
- **F2. The photography blocker does not apply to a functional web UI.** No
  `panel.truth.json` means no `ControlId` mapping, which is why those commands do nothing. A
  web panel needs a control inventory (names, counts, roles), which the manual gives in full.
  Millimetre positions are only needed for a photo-exact render.
- **F3. The engine already compiles to WebAssembly.** The `wasm-smoke` job builds `octoffi`
  for `wasm32-unknown-unknown` and plays the golden patterns in Node, byte-identical to
  native. That is a real head start.
- **F4. MIDI is output-only today.** A code search finds nothing for MIDI clock in or out,
  MIDI input and recording, program change, control maps, page sets, Grid-Track, keyboard
  transpose or anti-echo. The two-port, 32-channel model exists on output only.
- **F5. The repo's manual index is wrong for the MIDI, load/save and appendix chapters.**
  `reference/manual/INDEX.md` says MIDI is pages 109 to 112; it is pages 93 and 94. A request
  for the Scribe is in `journal/conductor/requests/`, and `findings.md` section 4 has the table.
- **F6. "World model" means two things here.** In `STATE.md`, `just report` and the
  verification schema it means the shared project state, which is the factory's own term and
  stays. In the owner's message it means the Genie-style rooms project. This spec only
  removes the second.

## Decisions for the owner

Defaults apply if you say "approve, all defaults", as in SPEC-0001.

| # | Decision | Default | Alternatives |
|---|---|---|---|
| D1 | Approve the refocus: web app first, rooms removed, `SPEC.md` superseded by ADR-0006 | Yes | No, keep WENGE as is; or approve only the removals |
| D2 | What "available on Ableton" means first | **Tier 1**: the web app sends and receives MIDI over a virtual MIDI port, and can follow Live's clock. Tier 2 (a Max for Live device hosting the app inside Live) starts as a timing spike, S4 | Tier 2 first; or both from the start |
| D3 | Panel visuals | A faithful layout, restrained materials, LEDs as the only colour, drawn in SVG or Canvas. No probe, no room. Photoreal WebGL stays a later option, not a scope | Photoreal WebGL panel with one fixed studio light, from the start |
| D4 | Browsers | Chrome, Edge and Opera. Firefox best effort (it needs a Mozilla site-permission add-on for MIDI). Safari and iOS run the UI without MIDI, because they lack Web MIDI | Also require Safari, which means a native helper |
| D5 | The native C interface and the lock-free ring (`octoffi`, O6) | Freeze: no new work, keep the tests | Delete now; or keep as the path to a later plugin |
| D6 | The name of the panel-controller crate | `crates/octoface` | Fold into `octocore` as a module |
| D7 | The tick length (D0 from SPEC-0001) | Unchanged: still open, and now it also sets the ratio of engine ticks to MIDI clock pulses (2 or 8 ticks per pulse) | |
| D8 | Rewrite the constitution (`agents/CLAUDE.md`, root `AGENTS.md` crate table and the nine things) after approval | Yes, by the Conductor in the removal PR | Leave until the first web release |
| D9 | The `verify:scope` gate: which words are banned in the live tree | The list in `removal-plan.md` section 5 | Narrower or wider |

## Series after approval

Each is its own pull request, tier and gate numbers stated in the commit. Nothing starts before
the approval record below is filled in.

| PR | What | Tier |
|---|---|---|
| P0 | ADR-0006 (supersede WENGE scope), `SPEC.md` banner, `STATE.md`. Documents only. | High |
| P1 | Removals and the `verify:scope` gate, red first (the gate fails on today's tree, then passes). Baseline shrinks by the ten `octoroom` tests, with `--remove ... --adr ADR-0006`. | High |
| P2 | `contracts/controls.json` and the panel fixture format; the first five workflows red first (Page mode, step toggle, Step zoom, ESC). | Medium |
| P3 | `apps/web`: the static app loads the WASM engine, draws the 16 by 10 matrix and the LEDs, plays to one Web MIDI output. Spikes S1 and S2 recorded. | Medium |
| P4 | MIDI: two ports, clock out and in, input, program change. Tier 1 Ableton guide and test procedure (spike S3). | High (timing) |
| P5 onward | The workflow waves in `product.md` section 5, each with its fixtures. | Medium |

## What happens to work in flight

- **#14 (guards) and #16 (step accumulators)**: still right. Exact step timing matters in a
  browser at least as much as anywhere. They can merge on their own merits.
- **O4 step 4b and 4c (host lock, ramps)**: reframed. In the browser the host is the audio
  clock and an incoming MIDI clock, not a plugin host with tempo ramps. 4b still waits for D0.
  The D3 ADR request filed in `journal/metronome/requests/` is on hold and probably lapses.
- **O6 (ring, snapshot, loom tests)**: on `main`, tested, and not needed by a single-threaded
  WebAssembly engine. Frozen (D5).

## Files

| File | Contents |
|---|---|
| `product.md` | The product, the UI features and tenets taken from the sources, MIDI and Ableton requirements, the waves, acceptance criteria. |
| `tech.md` | The architecture, timing and clock design, the panel controller, the Ableton routes, the spikes S1 to S5 with pass criteria, risks. |
| `removal-plan.md` | What is removed, retired, edited and kept, file by file; the scope gate; the order that keeps the gates green. |
| `findings.md` | The audit behind F1 to F6, the sources and their limits, the conflicts in the manual, engine coverage against the release notes. |
| `sources/` | Two digests of the manual, written by subagent sessions, each fact cited to a page. Machine-made and not yet human-checked; three claims were spot-checked against the pages (`p093`, `p094`, `p102`) and matched. |

## Approval record

None yet. When the owner answers, record the date, the exact words, and what they approve and
do not approve, as SPEC-0001 does. The agent merges nothing.

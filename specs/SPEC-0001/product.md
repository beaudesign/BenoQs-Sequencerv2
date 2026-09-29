# SPEC-0001 product spec

Revision r1. Approved by `@beaudesign` on 2026-09-29 for phases F0 and F1 (see
`README.md`). O4 and O6 are not approved for implementation.

## Problem and users

A sequencer that is faithful to the manual on paper still fails a musician if it hangs
notes, fires bursts, or drifts against the host. It also fails the people and agents who
maintain it if they cannot run it or trust a green build.

- A musician or producer using the instrument inside a DAW (target host: Live 12).
- Agents and engineers working in the repo, who need a runnable, verifiable core.
- The repo owner, who approves specs and merges.

## In scope

- Core reliability: O1 to O6 and O10 (O4 and O6 only once separately approved).
- A headless runner and WASM demo (O7).
- Honest gates, CI, ownership and the assertion ratchet (O8).
- Fixture DSL v2 and invariant tests (O9).

## Out of scope

- Renderer, panel truth file, plugin wrappers, shell (blocked on Xcode and calibrated
  photography).
- Everything in `SPEC.md` section 4 (Windows, sysex, multiplayer, generative audio,
  skinning).
- Any change to manual-cited musical behaviour without a fixture.
- The design system and DSP tracks (Q1). They run in parallel and need their own spec.

## Acceptance examples

| # | Given | When | Then | Task |
|---|---|---|---|---|
| A1 | Notes are sounding on several channels | Transport stops or the engine resets | Every sounding note gets a NoteOff in the next render call, CC 123 goes out on each used channel, and NoteOns minus NoteOffs is 0. | WENGE-0001 |
| A2 | The host called render for 10 s with transport stopped | Play is pressed | The first buffer holds only events due inside it. Nothing is dropped. | WENGE-0002 |
| A3 | One seed, pattern and tempo | Rendered with any buffer size from 1 to 4096, or a variable sequence | The absolute-time event streams are identical. | WENGE-0003 |
| A4 | The host reports a locate, a loop, or a tempo ramp | The next buffer renders | The next tick lands within 1 sample of the analytic position. Ramp drift is at most 1 sample. | WENGE-0004 (not approved) |
| A5 | More events than the buffer or queue can hold | Rendering continues | Drop counters rise, and no sounding note loses its NoteOff. | WENGE-0005 |
| A6 | A pattern file | `just run` executes it | An NDJSON log and a MIDI file appear, byte-identical on a second run. | WENGE-0007 |
| A7 | A pull request on `main` | CI runs `just verify` | The report JSON validates against the schema. A required gate that is `not_implemented`, failing, or a lower assertion count blocks the merge. | WENGE-0008 |

## Invariants

- The manual wins over any other source (N8). Behaviour changes need a fixture.
- Output is deterministic given seed, command log and host clock trace.
- Zero heap allocation on the audio path (measured 0 today; becomes a gate).
- NoteOn and NoteOff balance per (port, channel, note).
- Every NoteOn has velocity at least 1.
- Tightening a threshold is free. Loosening needs an ADR with an expiry date.

## Failure behaviour and permissions

- The audio thread never blocks, allocates or waits. Overload drops with a count and
  never blocks.
- FFI calls with bad indices or null pointers are no-ops or return an error code, never a
  crash (existing test).
- Agents may not merge, deploy, loosen thresholds or change contracts without the tiered
  approval in `AGENTS.md`.

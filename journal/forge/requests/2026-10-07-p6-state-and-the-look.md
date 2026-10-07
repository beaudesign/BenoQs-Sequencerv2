# Request to Conductor: record P6, and decide the look (R3)

From: Forge (`WENGE-0017`, `specs/SPEC-0002/p6-runs-when-opened.md`). To: Conductor (`journal/STATE.md`, `contracts/`, ADRs). Tier of the change: **Medium**; no contract is touched by P6.

## 1. `journal/STATE.md` (at its 200-line cap, so a request and not an edit)

Proposed paragraph for the SPEC-0002 section, when the stack #46, #47, P6 merges:

> **P6 (`WENGE-0017`): it runs when it is opened.** One press of Play starts the engine and a demo pattern (C minor pentatonic, 4 tracks) plays with a built-in monitor sound (on by default; a switch on the strip), the manual's red chase-light on the step you hear, one Play, a tempo field. The tick is the manual's (D0 decided by default: 48 ticks to a quarter note, 192 to a whole note, a step is 1/16 = 125 ms at 120 BPM; #46). The engine's playhead is the step last played (#47). Chromium 69, Node 367. **Not measured:** sound on a real speaker, Web MIDI in the owner's browser. **Not done:** most of the 247 controls, the display and the encoders. 

## 2. The look (R3): needs an ADR, the Conductor's

The owner's 2026-10-07 18:04 London message: "It should at least resemble something like https://roland50.studio". What the page draws today is what SPEC-0002 **D3** allows: flat, no gradient, no accent colour (CLAUDE.md items 4 and 6). A hardware-grade look (a lit, material panel, real depth) is not reachable inside those rules, and I will not loosen the slop lint or `app_source_rules.rs` to reach it.

What I am asking, not doing:
- Does the owner's message overturn **D3** and the no-gradient rule for the *panel only*? If yes, an ADR with an expiry date (CLAUDE.md item 3) and the new rule written as a test first.
- Per CLAUDE.md item 7 I will not load reference photography. What the Forge needs from the reference is **numbers** (materials, spacing, type, shadow behaviour) put into `contracts/` by the Conductor.
- Until that, R2 (drawing the other controls) is best kept to geometry, which does not depend on the look.

## Why a request

`journal/STATE.md` is capped and `contracts/` is the Conductor's.

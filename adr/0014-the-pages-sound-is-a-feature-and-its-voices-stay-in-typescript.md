# ADR 0014: The page's sound is a feature, and its voices stay in TypeScript

## Status

Proposed on 2026-10-17 by the Conductor. It takes effect when the owner merges the pull request that carries it, as ADR-0010 and ADR-0012 did. If the owner closes it, the monitor stays as it is today: a plain tone that is "not the instrument's voice", and S0 of SPEC-0004 is withdrawn.

It records decisions D1, D12, D13 and D14 of SPEC-0004 r1 (`WENGE-0030`). The owner answered that spec on Sat 2026-10-17 at 14:46 London in chat: "Okay build". How that was read, and how far it reaches, is in the spec's approval record. This ADR changes no file other than itself and the spec's approval record; the code is the Forge's, in its own pull request (`WENGE-0032`).

## Number

This ADR was first drafted as 0013. The owner's engineering spec for the shape player (pull request #71, `handoffs/eng-spec-shape-player-and-complex-voice-v2.md`, section 3.2 and its question QG3) reserves **0013** for "non-manual pattern styles live in `octostyle`", and says the sound ADR takes the next free number even if it is written first. So this is **0014**. Nothing in the decision changes. If the owner answers QG3 the other way, the two numbers swap and nothing else moves. (The frozen copy of the monitor in pull request #73 still says 0013 in its header comment, because that file is pinned byte for byte.)

## Context

**What the page plays today.** `apps/web/src/monitor.ts` plays the engine's own notes in the worklet's output: a pitched tone for a note (a sine and two harmonics) and three drums on MIDI channel 10. Its header says it "is deliberately plain ... and is not the instrument's voice". That was right for SPEC-0002 P6, whose question was whether a page opened with nothing plugged in can be heard to run.

**What the owner asked for.** On Fri 2026-10-16 at 17:15 London: "new oscillator types that are played through the sequencer - benny has got a great example of real sounding oscillators and filters modelled on physical hardware. Let's explore how we can bring that in instead of piano." SPEC-0004 plans it from published papers (benny is under a licence this repository has not decided to take on, so only its ideas are used).

**An overlap, settled.** A second engineering spec (a shape player and a Verbos-model voice) planned a Rust voice crate, a Rust offline renderer and an ADR of its own for the same decision. The owner took the recommendation to fold that voice into SPEC-0004 as a later voice (Sat 2026-10-17, 14:13).

**The rules this has to fit.**

| Rule | Says | Bears on |
|---|---|---|
| SPEC-0002 P6 and P10 | The monitor is audio only: it reads the engine's events, changes none, keeps nothing the engine can see | Any new voice |
| ADR-0008 | The page calls Rust through C exports; the worklet runs the engine and the monitor | Where a voice may live |
| `CLAUDE.md` rule 6 | No colour is introduced; the only saturated colour is an LED | The `Voice` button, at S3 |
| `CLAUDE.md` rule 5 | All motion is a physical simulation; no authored easing | Envelopes (S4) are one-pole charges, as a capacitor's |

## Decision

### 1. The page's sound is a feature

The monitor's "deliberately plain ... not the instrument's voice" stance is withdrawn. The page's sound is something the owner chose to improve, and it is held to measurements like the rest of the instrument (SPEC-0004 section 5). It stays **audio and never MIDI**: the Octopus makes no sound of its own, the events the page sends to its devices are byte for byte the same with any voice, and the drums are not voices (SPEC-0004 D9). The S0 pull request rewrites the monitor's header to say so.

### 2. Voices are a registry, in `apps/web/src/voices/`, in TypeScript

A **voice** is a sound the page can play a pitched note with. It has an id, a name, a per-voice cap on notes, a declared trait or two, and makes one synth per sounding note (`VoiceNote`) from preallocated state. The registry is the list of voices; a voice that is not in it cannot be chosen. One test walks the registry and holds every entry to the voice-wide observables (SPEC-0004 H-16), so a voice added later is tested with no edit to the test.

### 3. Plain stays, sample for sample

The present tone becomes the voice `Plain`. With `Plain` selected the monitor's output equals today's, sample for sample, for any notes and any block sizes. The test is a frozen copy of the monitor at `c079f8f` (in the test tree, never imported by the page) and a pinned hash of its output on the demo. Plain is the fallback on a slow phone and the baseline every older test stands on; whether it is ever removed is the owner's (SPEC-0004 Q-S4-2).

### 4. No Rust voice core (D13)

The voices are written in TypeScript and run in the AudioWorklet beside the monitor they replace. **What is given up:** a native build and the wasm build agreeing bit for bit, which the engineering spec asked about (its QB2). Equality tests run in Node; the browser tests assert measured properties, never a hash. **What is kept:** one language for the page's sound, no second build step for it, and no new crate. **Revisit if** the owner wants a sound identical on a desktop build and in the browser, or a bounce that must match the page. That is a new ADR that moves the DSP of every voice into a crate.

### 5. Caps (D14)

Each voice declares the most notes it plays at once (`maxNotes`). When it is reached the oldest of its notes is taken. The monitor's pool of 24 slots, shared with the drums, and its rule that the oldest slot is taken when none is free, are unchanged, so for `Plain` (cap 24) nothing differs from today.

### 6. What this ADR does not decide

Which voice is the default (SPEC-0004 D2, D3, Q-S4-1: Saw ladder at S3, after the owner has listened), the `Voice` button and its message (S3), any filter or oscillator (S1, S2), the Complex voice (S7, waiting for its engineering spec), the shape player (a separate piece, not planned here), and any licence for the repository.

## Consequences

- The monitor gains a `voice` and delegates the pitched tone to it. The engine, the events, the ABI, the contracts, the worklet protocol and Web MIDI are untouched. The page has no new control at S0.
- The octoweb module and the Rust workspace do not change.
- `apps/web/src/voices/` is compiled into `dist/src/` and copied by the site assembler with the rest of `dist/src`, so the published page needs no new rule.
- A reviewer can check the claim "nothing the owner hears changes at S0" without listening: it is a bitwise equality.

## Alternatives considered

| Alternative | Why not |
|---|---|
| Keep the monitor plain and make a synth beside it | Two sounds, two buttons, and the tone that was always meant to be replaced stays in the code path |
| A Rust crate for the voices (the engineering spec's plan) | Gives bit-exact native and browser sound, which nothing in the product asks for today; costs a second build step and a crate, and splits the DSP from the monitor |
| Delete `Plain` | Loses the sample-exact baseline of every existing sound test and the fallback for a slow phone |
| Put voices in the engine | The engine is the Octopus: it makes events, not sound (SPEC-0002 P6) |

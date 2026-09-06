# ADR 0001: Portable C++ Core With Adapters

## Status

Accepted

## Context

The original implementation mixes sequencing rules, Max Dict schema, Max message routing, JSUI drawing, and browser mock behavior. That makes the instrument hard to test and hard to reuse outside Max for Live.

JUCE 8 is the current JUCE line referenced by the project direction. JUCE can host a C++ application/plugin UI, but the sequencing rules should not depend on JUCE APIs.

## Decision

Create a standalone `benoqs_core` C++ library for the grid/page/track/step model, scale quantization, groove/strum timing, runtime playheads, and MIDI event scheduling.

Keep JUCE, Max, browser, and Ableton integration as adapters around that core.

## Consequences

- Core sequencing behavior can be tested with ordinary C++ tests.
- A JUCE app/plugin can be built without rewriting timing logic.
- Existing Max/JS files can be migrated incrementally instead of replaced in one risky pass.
- UI work has a real model to bind to instead of demo-only browser state.

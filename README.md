# 🐙 BenoQs Sequencer v2 (WENGE)

> *A ground-up reimplementation of the genoQs Octopus (2009) sequencer core, in Rust, measured and cited against the real reference manual.*

## What this is

This repo holds `crates/octocore` — a from-scratch Rust reimplementation of the genoQs Octopus's sequencing engine: transport, all five fixed play directions plus user-programmable custom directions, track chains, the cross-track effector, chord polyphony and strumming, scale quantization, phrases, hypersteps, step events, and sample-accurate MIDI scheduling — plus `crates/octoffi`, a thin C ABI boundary over it, verified against a real compiled C program.

Every non-trivial behavioral constant in `octocore` is cited to a page of the real *Octopus Reference Manual, CE OS v5.30* (genoQs Machines, Stuttgart 2009), which ships in this repo at `reference/manual/`. Where the manual is genuinely ambiguous or a piece of behavior hasn't been implemented yet, that's recorded honestly in `tests/conformance/AMBIGUITIES.md` rather than guessed at silently. 66 unit tests and 3 conformance fixtures, all green. A fresh engine loads the 48 factory phrases (Green / Red / Orange) and applies phrase POS time-compression.

The wenge design vision the crate was built against — a hardware-accurate panel renderer with physically-simulated materials and motion, and a "rehearsal rooms" concept tying acoustic and visual environments together — lives in `SPEC.md` and `docs/`. None of the rendering/room half exists yet; only the sequencer core and its C ABI boundary do. Phrase playback and Track Rotate / Skip Rotate are now wired into the tick loop (CE v5.30 p.16, p.38-39).

## How this repo came to exist

[BenoQs-Sequencer](https://github.com/beaudesign/BenoQs-Sequencer) is the original project: a Max for Live device and JS engine recreating the Octopus in Ableton Live, under active development there (see its own history and PRs).

A separate agent session was given a specification for a from-scratch Rust/Swift/Metal rewrite of the Octopus, intended for a different project, that ended up applied to this one by mistake. Before that was caught, real work had already been built on top of it — most importantly `crates/octocore`, corrected against the actual reference manual and genuinely well-tested. Rather than discard that work or force it back into the original repo (where it would conflict with the real, actively-developed JS implementation), it's split out here as its own project, clearly separated from the original's history and direction.

The original repo is unaffected and continues under its own steam.

## Status

Sequencer core only. No renderer, no shell, no plugin hosts, no room system. See `journal/STATE.md` for the detailed, current state of every piece, and `tests/conformance/AMBIGUITIES.md` for exactly what's confirmed against the manual versus still open.

## Credits

**Gabriel Seher** and **Marcel Achim** at genoQs Machines, Stuttgart, designed the original Octopus. This project is an independent reimplementation, not affiliated with or endorsed by genoQs Machines.

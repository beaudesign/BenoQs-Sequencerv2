# 🐙 BenoQs Sequencer v2 (WENGE)

> The genoQs Octopus step sequencer, rebuilt for the browser. Open a page, press Play, hear a pattern, and send MIDI to your synths and to Ableton Live.

The Octopus is a hardware MIDI sequencer from 2009. It has ten tracks of sixteen steps on a matrix of 160 keys, 21 knobs, and unusual ways of shaping a pattern: play directions, chains, phrases and an effector. This project reproduces it as a web app. The sequencing engine is written in Rust and runs in the browser as WebAssembly. The panel is drawn to match the real one, and everything the engine does is checked against the original reference manual.

**It is not finished.** The engine is far ahead of the panel: it can do a great deal more than the panel lets you reach yet. [What works today](#what-works-today) says exactly where the line is.

## Try it

You need [Rust](https://rustup.rs) with the WebAssembly target (`rustup target add wasm32-unknown-unknown`), [Node](https://nodejs.org) 22.18 or newer, and [just](https://github.com/casey/just). Then:

```
git clone https://github.com/beaudesign/BenoQs-Sequencerv2.git
cd BenoQs-Sequencerv2
just web-app
```

Open <http://localhost:8080/pages/app.html> in **Chrome or Edge** and press **Play**.

The first press starts the audio, loads a small demo pattern (kick, hats, clap, bass and lead, at 120 BPM) and plays it through the page's built-in sound, so you hear something with no MIDI gear at all. The browser may ask to use MIDI: allow it if you want to play real instruments. Safari and Firefox load the page too, and the keys and LEDs work, but they have no Web MIDI, so nothing is sent out of them.

## What works today

**On the page**

- The front panel, drawn flat in the hardware's own materials: the 160 matrix keys, the MIX and EDIT knobs, the mode keys, the note ring and the scale keys. Each control has the manual's own name and can be reached from the keyboard.
- **Play** and **Stop**, a **tempo** field, and a **Metronome** click.
- **Sound** (the built-in monitor) and **Drums**, a button that takes the drums out of that sound and leaves the synth. It changes only what you hear: the drum notes still go out over MIDI.
- **MIDI out** to two devices at once. The Octopus has two ports and so does this: tracks on MIDI channels 1 to 16 go to port 1, and 17 to 32 go to port 2. (A track can only be moved to port 2 through a setting in the page address for now, until Track zoom exists.)
- **MIDI clock**: send it (Master), follow one (Slave), or follow and pass it on (Slave with echo).
- **MIDI in** is decoded and counted, and the clock uses it.
- A few real panel workflows: turning steps on and off from the matrix, zooming into a step, setting its velocity, pitch, length and start with the EDIT knobs, skipping a step, and the EDIT and PLAY mode keys. The list is `tests/conformance/panel/`: one small file for each, each cited to a page of the manual.
- The LEDs light as the engine says, and a red chase-light follows each track.

**In the engine** (`crates/octocore`, reached through the page as it grows)

Transport, the five fixed play directions and user-programmable ones, track chains, the cross-track effector, chord polyphony and strumming, scale quantisation, phrases (a fresh engine loads the 48 factory ones), hypersteps, step events, and sample-accurate MIDI scheduling.

**Not there yet**

- Most of the panel's 257 controls do nothing yet. The circles on the right of the panel are the next slice of work: see [`specs/SPEC-0003/`](specs/SPEC-0003/README.md).
- Track zoom, save and load, recording from MIDI in, and the tempo knob (the tempo field in the strip does the job meanwhile).
- The Ableton guide ([`apps/web/ABLETON.md`](apps/web/ABLETON.md)) has not been checked against a real copy of Live, and the clock has been measured only against a simulated sender. If you try it, the guide says what to look for.

## Playing into your instruments

Pick a device for **MIDI Out 1** (and **MIDI Out 2** if you want it) in the strip above the panel. Nothing is sent to a port until you choose a device for it. For Ableton Live, create a virtual MIDI port (the IAC Driver on macOS, loopMIDI on Windows), choose it as the output, and switch it on as a track input in Live. [`apps/web/ABLETON.md`](apps/web/ABLETON.md) has the full steps, including how to keep the two clocks from feeding each other.

## Run the engine without a browser

The engine also runs headless. This plays a pattern file with no audio device and no clock, and writes the MIDI it produced:

```
just run examples/hello
```

You get an event log and a standard MIDI file in `target/octorun/`. The same pattern gives the same bytes every time, and that is checked. [`crates/octorun/README.md`](crates/octorun/README.md) has the details, and `examples/` has five patterns to start from.

## How it is checked

The Octopus manual ships in [`reference/manual/`](reference/manual/) (the PDF, and its pages as text). The engine's behaviour cites the manual page it came from. Where the manual is unclear, both readings and the one chosen are written down in [`tests/conformance/AMBIGUITIES.md`](tests/conformance/AMBIGUITIES.md) instead of guessed at. To read what the manual says about a topic: `just manual <topic>`.

```
just verify
```

runs every check, in about a minute. The tests and fixtures that must keep passing are listed in [`harness/baseline.txt`](harness/baseline.txt), and the run fails if any of them is lost.

## What is where

| Path | What |
|---|---|
| `crates/octocore` | The sequencing engine |
| `crates/octoface` | The panel controller: what each key and knob does, and which LEDs light |
| `crates/octoffi` | A C interface to the engine, kept and tested but no longer growing |
| `crates/octorun` | The headless runner |
| `apps/web` | The web app: panel, audio, Web MIDI ([its README](apps/web/README.md)) |
| `contracts/` | The control inventory and the design tokens, the facts the rest builds on |
| `reference/` | The Octopus reference manual |
| `tests/conformance/` | Engine and panel fixtures, and the list of ambiguities |
| `specs/`, `adr/` | What is being built and why, and the decisions taken |
| `docs/` | Architecture, the engine, the design rules, the roadmap |
| `journal/` | The working notes and the current state ([`STATE.md`](journal/STATE.md)) |

## Contributing

Most of the work here is done by AI agents working to a written process: [`CLAUDE.md`](CLAUDE.md) is where each of them starts, and [`AGENTS.md`](AGENTS.md) is the task lifecycle. A person starting out can read the same two files. In short: a change starts with a test that fails, nothing is merged without the owner, and a check is loosened only by a written decision with an expiry date.

## Where this repo came from

[BenoQs-Sequencer](https://github.com/beaudesign/BenoQs-Sequencer) was the first attempt: a Max for Live device in JavaScript. A specification for a from-scratch Rust rewrite, written for a different project, was applied to it by mistake, and real work was built on top before anyone noticed, most of all the engine, which was then corrected against the manual and heavily tested. Rather than throw that away or force it into a repository with its own JavaScript history, it was split out here as its own project. The original is archived, with its history intact.

## Credits

**Gabriel Seher** and **Marcel Achim** at genoQs Machines, Stuttgart, designed the original Octopus. This project is an independent reimplementation, not affiliated with or endorsed by genoQs Machines.

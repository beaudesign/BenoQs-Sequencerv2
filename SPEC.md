# OCTOPUS v2: Master Specification

**Codename:** `WENGE`
**Supersedes:** `beaudesign/BenoQs-Sequencer` (archive, do not merge)
**Status:** Ratified. Section changes require an ADR. Rewritten under
`adr/0006-web-sequencer-supersedes-the-rooms-scope.md` and `specs/SPEC-0002/` (2026-09-30): the
product is a web sequencer. The text it replaced is in git history.
**Reference hardware:** genoQs Machines Octopus, CE OS v5.30, Stuttgart 2009.

---

## 0. How to read this document

This is the root. It states the thesis, the non-negotiables, and the map. Everything
operational lives in `docs/`. Everything an agent needs to start work lives in `agents/`.
Everything that must never drift lives in `contracts/`. The product itself is specified in
`specs/SPEC-0002/`, which wins over this file where the two disagree.

Read in this order:

| # | Document | Owner role | Read if you are |
|---|---|---|---|
| 00 | [North Star](docs/00-north-star.md) | Conductor | Everyone. Non-optional. |
| 02 | [Architecture](docs/02-architecture.md) | Conductor | Writing any code at all |
| 03 | [Sequencer Core](docs/03-sequencer-core.md) | Metronome | Touching timing, MIDI, or state |
| 05 | [Design System](docs/05-design-system.md) | Curator | Making any visual decision |
| 07 | [Verification](docs/07-verification.md) | Referee | Everyone. Non-optional. |
| 08 | [Agent Operating Model](docs/08-agent-operating-model.md) | Conductor | Everyone. Non-optional. |
| 09 | [Roadmap](docs/09-roadmap.md) | Conductor | Planning or reporting |
| 10 | [Risks and Decisions](docs/10-risks-and-decisions.md) | Conductor | Blocked, or about to make a big call |

The numbers 01, 04 and 06 are not reused. The panel-measurement and render-engine documents
(01 and 04) are kept for reference in `archive/native-panel/`; the shell document (06) is gone
and is in git history.

If a statement in `docs/` contradicts this file, this file wins, and the contradiction
is a bug: file an ADR.

---

## 1. The thesis

The Octopus is an interface, not an object to be admired. Its whole front is a matrix of
buttons, two columns of encoders and a lot of LEDs, and one LED means different things in
different modes. Someone who has learnt it does not want a different interface. What they
want is the same one where they are: in a browser, playing real MIDI into real
instruments.

So the product is a **web sequencer**:

- **The engine** is `octocore`, a Rust sequencer that behaves as the CE v5.30 reference
  manual says, compiled to WebAssembly for the page and to native code for the tests.
- **The panel controller** (`octoface`, planned) is the layer that is missing today. It
  turns button and encoder events into engine commands and an LED frame, one manual
  workflow at a time, each with a fixture cited to its page.
- **The app** (`apps/web`, planned) draws the panel from a control inventory and talks Web
  MIDI: two output ports, clock out, notes and clock in, program change.
- **Ableton Live** is a client of the app, not its host. Tier 1 is a virtual MIDI port that
  Live reads and writes (IAC Driver on macOS, loopMIDI on Windows). A Max for Live device
  that shows the page in `jweb` is a spike, decided on evidence.

Why v1 read as generated (`docs/00-north-star.md` §2) comes down to one thing: it drew a
picture of the panel and put little behind it. v2 puts the behaviour first, and draws the
panel as a faithful, restrained set of controls whose only colour is the colour of a lit LED.

### What this makes the product

A 2009 German sequencer, reachable from any current Chrome or Edge tab, that plays and
programs the way the manual says it does, and can be routed into Ableton Live or into
hardware on the desk.

---

## 2. Non-negotiables

These are the things that, if violated, mean we have failed regardless of what else
shipped. Each maps to an automated gate in [Verification](docs/07-verification.md), or says
which gate is still to be built.

**N1. The controls are an inventory, not a drawing.**
Every control's id, kind, zone, the manual's own name for it and its logical position come from
`contracts/controls.json` (planned, P2). No hardcoded pixel coordinates in the app.
Gate: `verify:arch` (not built yet).

**N2. The only saturated colour is an LED's.**
Red, green and orange, steady or flashing, and only as roles: the colourways in the manual
change the hue, not the role. Everything else is one neutral palette. No gradient stands in
for a material, no drop shadow fakes depth, no accent colour. Gate: `verify:slop` (not built
yet).

**N3. Motion is state, not decoration.**
An LED is off, on or flashing at the manual's rate; a button is up or down. Nothing else
animates, and nothing animates on its own. No easing keyword and no `cubic-bezier` anywhere.
Gate: `verify:slop` (not built yet).

**N4.** Retired with the photoreal render (ADR-0006). It required static regions of a 120 Hz
frame to be bitwise identical between frames. The number is not reused.

**N5. Timing beats pixels.**
If the page and the sequencer contend, the sequencer wins. The audio clock keeps time; the
page does not. The audio thread never allocates, never locks, never waits on the UI or on
MIDI. A dropped frame is a bug. A late note is a catastrophe. Gate: `verify:timing`.

**N6. Colour has one source.**
The LED roles and the neutral palette in [Design System](docs/05-design-system.md). A colour
literal anywhere else fails review. Gate: `verify:tokens` (not built yet).

**N7. Every taste judgement becomes a test.**
When a review finds something wrong, the fix is not "make it nicer". The fix is a new
deterministic assertion in the harness plus the change that satisfies it. The harness only
grows. Gate: `verify:regressions` (assertion count is monotonic).

**N8. The hardware's behaviour is the manual's behaviour.**
Where v2 and the CE v5.30 reference manual disagree about what a control does, the manual
wins, and the discrepancy is logged as a fixture in `tests/conformance/`. Where the manual
is silent or contradicts itself, the fixture is `pending` with the ambiguity written next
to it. Gate: `verify:conformance`.

---

## 3. What we are building, concretely

Deliverables, in dependency order. Phases and exit criteria are in
[Roadmap](docs/09-roadmap.md); the design is in `specs/SPEC-0002/`.

**D1. `octocore`** (Rust, built)
The sequencer. Ten tracks of sixteen steps, banks of sixteen pages, the full attribute
model (VEL PIT LEN STA POS DIR AMT GRV MCC MCH), chains, hypersteps, phrases, the effector,
directions, scales. Deterministic under a seed. Compiles to a native library and to
WebAssembly. No allocation on the render path after init.

**D2. `octoface`** (Rust, planned, P2)
The panel controller: buttons and encoders in, engine commands, an LED frame and the
displays out. A pure state machine driven by a millisecond count the caller passes in. Each
manual workflow becomes fixtures.

**D3. `apps/web`** (TypeScript, planned, P3 and P4)
A static page. Draws the matrix, the encoders and the LEDs from `contracts/controls.json`,
runs the engine in an AudioWorklet, owns Web MIDI (outputs, inputs, clock) and the saved
state. No backend, no account.

**D4. Ableton routing** (documentation and a spike, P4)
Tier 1: the virtual-port guide, tested against a real Live. Tier 2: `hosts/m4l`, a Max for
Live `jweb` device, only if spike S4 shows it can work. No plugin formats.

Supporting pieces that already exist: `crates/octorun` (headless runner, golden event
streams), `crates/octoffi` (the C ABI the WebAssembly smoke test loads; frozen), and the
harness and ratchet in `xtask/` and `harness/`.

---

## 4. What we are deliberately not doing

Scope discipline is a design decision. Explicitly out for v1 (`specs/SPEC-0002/product.md`
section 8):

- Anything that generates a space, a scene or a sound with a model. The instrument is MIDI
  only, and the tree is kept clean of the old scope by `verify:scope`.
- A native app, Metal, Swift, VST3 or AU wrappers.
- A built-in synth or any audio output.
- SysEx exchange with real Octopus or Nemo hardware. The manual gives sizes and procedures
  but not the byte layout.
- Photo-exact geometry, calibrated materials, or a blind comparison against photographs.
- Accounts, a backend, sharing, multiplayer.
- Skinning, theming, or user-supplied panel layouts. There is one Octopus.
- Safari and iOS MIDI. The browsers do not offer Web MIDI; the app loads and says so.

---

## 5. Repo layout

```
wenge/
├── SPEC.md                     ← you are here
├── CLAUDE.md                   → symlink to agents/CLAUDE.md
├── justfile                    ← every verb an agent needs
├── contracts/                  ← frozen. Conductor-only via ADR.
│   ├── verification.report.schema.json
│   └── controls.json           ← planned, P2: the control inventory (by ADR)
├── crates/
│   ├── octocore/               ← D1  (owner: Metronome)
│   ├── octoface/               ← D2  (planned, P2; owner: Panelwright)
│   ├── octorun/                ← headless runner (owner: Conductor)
│   └── octoffi/                ← C ABI boundary (frozen)
├── apps/
│   └── web/                    ← D3  (planned, P3; owner: Forge)
├── hosts/
│   └── m4l/                    ← D4, Tier 2 only (scaffold; owner: Metronome)
├── xtask/  harness/            ← the gates, the ratchet, the report (owner: Referee)
├── reference/
│   ├── manual/                 ← CE v5.30, page-indexed
│   ├── plates/                 ← reference photography (optional now)
│   └── NOTES.md
├── tests/
│   ├── conformance/            ← manual-derived behavioural fixtures
│   └── golden/  examples/      ← golden event streams
├── specs/                      ← SPEC-0001 (engine plans), SPEC-0002 (the web product)
├── docs/                       ← this spec
├── archive/                    ← retired documents, kept for reference
├── adr/                        ← architecture decision records
├── handoffs/                   ← task handoff records
└── journal/                    ← per-agent working logs (append-only)
```

Whether a directory exists yet is in `journal/STATE.md`. A path marked planned is built by
the phase named in the roadmap, not before.

---

## 6. The single most important paragraph in this document

Most ambitious specs fail at the same place: they describe a good destination and say
nothing about how you would know, mechanically, at 3am, with no human awake, whether the
last commit made things better or worse. That is the difference between a project that
converges and one that oscillates forever while burning tokens.

So the centre of gravity of this specification is [Verification](docs/07-verification.md),
and specifically the rule in **N7**: *every taste judgement becomes a test*. A model looking
at a screenshot and saying "the edit light looks wrong" is worth nothing on its own, because
it will say something different tomorrow. That same observation, converted into "after one
press of EDIT the LED frame shows EDIT orange and flashing (manual p068)", is worth
everything, because it is true tomorrow and it fails loudly when someone breaks it.

Build the ratchet before you build the thing. The ratchet is what makes a thousand
agent-hours add up instead of cancel out.

---

## 7. Definition of done for v1

The product ships when all of the following are simultaneously true on `main`. The
measurable form of each is in `specs/SPEC-0002/product.md` section 9.

1. `just verify` is green, with at least 400 assertions in the harness (304 on the day
   ADR-0006 landed), and the count has only grown since.
2. Conformance: every fixture for every workflow in scope passes, and each cites its manual
   page (A1).
3. Determinism: the WebAssembly engine plays every golden pattern byte for byte like the
   native one, in the browser as well as in Node (A2). Today the Node smoke test plays three
   of the five.
4. Timing: note-on jitter from browser to instrument, and clock follow against Live, are
   within the numbers the owner sets from spikes S1 and S3 (A3, A4). The old 120 microsecond
   figure was for a plugin inside Live and does not apply.
5. Ableton: Live plays a Live instrument from a pattern and follows Live's transport over a
   virtual port (A5).
6. Hardware: one pattern plays a real synth on port 1 and a second device on port 2 at the
   same time (A6).
7. Browsers, keyboard and screen reader, scope, ratchet: A7 to A10.

Item 6 is the real one. The rest exist to make it reachable, and it is checked by ear on
real gear.

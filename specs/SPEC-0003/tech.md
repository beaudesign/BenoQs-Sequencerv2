# SPEC-0003 tech spec: the circle

Draft for the owner. Nothing here is built, and this pull request changes no code, contract or fixture. Every number
in section 1 was counted from `main` at `6c789e7` (`contracts/controls.json`, `apps/web/layout/panel.layout.json`,
`crates/octoface`, `crates/octocore`, `tests/conformance/panel`).

## 1. What exists and what is missing

| Layer | State |
|---|---|
| **Inventory** (`contracts/controls.json`, Panelwright, by ADR) | 247 controls. The circle's: outer ring 7 (`circle.tempo`, `.select`, `.chr`, `.follow`, `.chain`, `.clock200`, `.numeric`), inner circle 17 (12 notes, `upper_c`, `low_c`, `scale_mod`, `scale_sel`, `scale_cad`), chord 7, transport 4, mode 6, the main knob 1: **42**. Open items on them: `circle.numbers` (Q32), `transport.triangles` (Q21), and the questions on Q09, Q10, Q23, Q33 |
| **Layout** (`apps/web/layout/panel.layout.json`) | 38 of the 42 are placed. Not placed, with the file's own reasons: `circle.low_c` (the plate has 13 note keys, the inventory 14), `circle.chr`, `circle.follow`, `circle.numeric` (a display, not a key). **25 more keys are drawn and do nothing** because no inventory names them: `hw.num.1` to `.10` (the number keys), `hw.ring.1` to `.13` (the unnamed ring keys), `hw.mode.map`, `hw.transport.left` |
| **Controller** (`crates/octoface`, Panelwright) | `PanelMode` is `Page` or `Step`. It binds 14 controls; of the circle's 42 it binds **five** (`mode.page`, `mode.step`, `mode.play`, `mode.edit`, `transport.stop`) and drives one LED, Program. The main knob, the twelve notes, the chord block, the scale keys, Tempo, Select, X, Follow and 200 are ignored ("a number the controller does not use is ignored", `ABI.md`). **It has no display output** (the P5 plan, D-P5-5) and no way to release held keys (`QUESTIONS.md`, last item) |
| **Engine module** (`apps/web/engine`, Forge) | `octocore` and `octoface` joined behind C exports for the AudioWorklet. `octoweb_input(now_ms, kind, control, detents)` takes presses, releases and turns; the LED frame comes out; ABI revision 2. No export for a display |
| **Page** (`apps/web`) | Draws the 38 placed controls and the 25 inert keys. Knobs turn and send, and show nothing (D-P7-5). The strip holds Tempo, the MIDI Clock select and four others as furniture, outside the panel |
| **Engine writes** (`octocore`) | The only writers of musical state are `Command::SetTrack` (15 attributes) and `Command::SetStep` (12), plus transport, `SetActivePage`, `SetMode` and the page's tempo. Everything else the circle needs **exists as engine logic with no command to set it**: page scale and grid scale (`Page.scale`, `Grid.global_scale`), the track multiplier (`multiplier_num`, `multiplier_den`), chains (`chain_head`, `chain_members`, `chain_base`), chords (`ChordPool`: offsets, count, polyphony), user directions (`user_directions[].slices[]` with `certainty_next` and nine triggers), step events (`Step.event`). The hyperstep has `StepAttr::Hyperstep`. **Absent altogether:** program and bank change (`Event` has five kinds), CC resolution, legato length (Q40), keyboard transpose, force-to-scale from MIDI in |
| **Panel fixtures** (`tests/conformance/panel`) | 47 asserted (`edit` 6, `leaving_step_zoom` 4, `page_view` 5, `play_mode` 6, `step_attributes` 10, `step_zoom` 16) and 20 `pending`. The circle appears in them only as the MODE keys and the PLAY and Program LEDs |

So the circle's functions are mostly **engine logic that nothing can reach**. The work is three things: a way to *read*
(displays), a way to *set* (engine commands), and a controller that maps a gesture to both.

## 2. Design principles

1. **A display has a meaning layer and a rendering layer.** The meaning ("the tempo is 120", "the pitch is G in octave
   3", "the chord is C E G, octave view 2") is what the manual states, and fixtures assert it. Which LED lights is the
   rendering, and the manual often does not say (Q43, Q45). A fixture never asserts a guessed LED; the rendering
   function is written once, named `provisional` and listed in `QUESTIONS.md`, as the flash rate (Q25) and `Shine` (Q03) already are. A hardware
   answer changes one function and its fixtures, not the controller.
2. **No display widget.** The numeric quadrant and the inner circle are, as far as the manual says, LEDs on keys the plate already has (T05:
   "the 100 position is lit"; `p54`: "an orange dot at the 1 position"). They are drawn by the LED frame that exists.
   The only new thing the page gets is text for the accessibility tree: the meaning layer, as a sentence ("Tempo,
   120"; "Pitch, G, octave 3"). This also settles the look: **no new colour, no new token**.
3. **The controller stays a pure state machine.** Time is passed in. A hold is state in the controller, not a timer on
   the page. A second, larger machine is not built: the controller grows by modules (section 6), one per block.
4. **The engine owns time and chance.** The "next 1/16", the strum ticks, the random picks, the certainty coin-flips
   are engine behaviour with engine fixtures. The panel sends a command and reads a result. This is the line that
   keeps A2 (byte-identical native and WebAssembly) true.
5. **One state, two views** for anything the strip and the panel both show (tempo, clock state). The panel is the
   source of truth where the hardware has the control; the strip's field is a second view of the same value.
6. **Additive, per slice.** Engine commands and ABI exports are appended (as `SetTrack` was after `LoadState`) in the
   slice that needs them, never in one batch, so each pull request is small and no tag moves.

## 3. The display model

Two read models cross the ABI, both pure data.

```rust
/// What the circle means right now. Asserted by fixtures.
pub struct Readout {
    pub numeric: Numeric,
    pub pitch: PitchRead,
}

pub enum Numeric {
    Off,
    Tempo { bpm: u16 },
    Bank { playing: u16 /* bit per bank */, shown: u8 },
    Value { what: ValueKind, value: i32, lo: i32, hi: i32 },     // velocity, a map value, an attribute sum
    Factor { value: u8, neutral: u8 },                           // 1..=17, or LEN and STA 0..=16
    EventRange { attr: AttrId, value: u8, max: u8 },
    Certainty { percent: u8 },
    Program { n: u8 },                                           // 0..=128
    BankNumber { n: u8 },
    Strum { level: i8 },                                         // -9..=9
}

pub enum PitchRead {
    Off,
    Pitch { octave: u8, class: u8 },                             // the p52 reading, before any dots are chosen
    Scale { members: u16, base: u8, forced: Force },             // a bit per pitch class
    Chord { view: ChordView, notes: [u8; 3] /* bits per octave layer */, root: u8 },
}
```

`ChordView` is `Chord` or `Octave(1..=3)`. The shapes above are a sketch for review, not a final API: the Panelwright
fixes them in S1 with the first fixtures. What is fixed here is the split: `Readout` is asserted, and `leds(&Readout)`
is the provisional rendering into the LED frame.

**Fixture grammar** gains, in `crates/octoface/tests/panel_fixtures.rs` (Panelwright's zone):

```text
given scale page|grid = <classes> base <class> force on|off|locked
given track <t> mult = <n>/<d> | pit = <int> | chain head <t> members <t,t,..> base own|head
given step <t> <s> chord = <semitone offsets> poly <n> | strum <n> | event <attr> amt <n> range <n>
at 0 ms dblclick <control>                       two clicks inside the (provisional) double-click window (Q24)
at 0 ms hold <control> | release_all
expect display numeric = tempo 120 | certainty 50 | strum -3 | ...
expect display pitch = pitch octave 3 class G | scale <classes> base C | chord octave 2 notes <..> root C
expect engine scale page = <classes> base <class> forced
expect engine track <t> mult = <n>/<d>
expect engine chain <t> head = <t> members = <t,t,..>
expect engine step <t> <s> chord = <semitone offsets> poly = <n>
```

A `dblclick` is the line a pending fixture has needed since Q24; its window is the controller's provisional constant.

## 4. Holds, and the web

The circle is the most hold-driven part of the plate: TEMPO held and then a matrix key; a CHORD button held while the
main knob turns; Select held while a note is pressed; a track selector held while a step is pressed (hypersteps). The
controller already treats Step Mode this way (`Input::Down` and `Input::Up` are the gesture). Two additions:

- **`release_all`** in the controller, called by the page on blur, visibility change and pointer cancel. It is the gap
  `QUESTIONS.md` records. Without it a lost key-up leaves a hold forever, and on this panel a hold changes what every
  other key means. CA8 tests it.
- **The latch** (SPEC-0002 `tech.md` section 5): Shift-click or a long press latches a hold, shown by the held
  control's LED flashing, and a second click releases it. This is an owner decision already open (SPEC-0002 section
  9.7). It is needed before S4 (the first slice where a *knob* turns under a held key) and is decision D6 here.

## 5. The engine surface

Each row is a request to the Metronome (`journal/metronome/requests/`), made when its slice starts, not before. All are
additive. "Event" rows change emitted events, which is Medium tier by `AGENTS.md`; "ABI" rows change `octoweb`'s
exports, which are Forge's and additive.

| Slice | Command or read | Purpose | Pages | Notes |
|---|---|---|---|---|
| S1 | read: tempo, active bank, which banks are playing | The quadrant | p77 | Reads from `Grid`; no write. Tempo is already a page-side `setTempo` |
| S2 | none (pitch already `SetTrack Pitch`, `SetStep PitchOffset`) | The ring | p46, p52 | |
| S3 | `SetScale { target: Page or Grid, members: u16, base: u8, force: bool }`; `ScaleCadence` (rewrite the page's track pitches into the scale) | SCALE SEL, Select, a name, MOD, CAD | p71, p72 | The note path already forces a pitch into `Page.scale`; verify that, do not assume it |
| S4 | `SetStepChord { track, step, offsets: [i8; 6], count }`; `StepAttr::Polyphony`; read: the chord | Chord and Octave views, polyphony | p20 to p23 | `Strum` is already a `StepAttr`. Octave View is a view of the same offsets (0 to 35 semitones): no new storage |
| S4 | read: the strum table | Strum display | p22 | Already `tables::strum_offset_ticks` |
| S5 | `TrackAttr::MultiplierNum`, `MultiplierDen`; `StepAttr::LengthMultiplier` (exists); "applies on the next 1/16" | Time | p16, p49, p50 | |
| S5 | `SetChain { head, members, base }`, `ClearChain { track }`, `SetChainBase` | X, base switch | p47, p48 | Follow is view state, in the controller |
| S5 | `SetHyperstep { track, step, hyped_track }` (check what `StepAttr::Hyperstep` carries) | Engage and destroy | p31 | "One hyperstep per hyped track" is an engine rule |
| S6 | `SetStepEvent { track, step, attr, range }`, `ClearStepEvent`; `TrackAttr::IsFeeder` and `IsListener` (exist) | Events, EFF | p34, p59 to p61 | |
| S7 | `Event::ProgramChange`, `Event::BankSelect`; `TrackAttr::CcResolution` | Program, bank, MCC resolution | p50, p51, p53 | Event kind: Medium. Also `octoweb`'s event record needs a kind (ABI) |
| S8 | `SetDirectionSlice { dir, slice, triggers: [u8; 9], count, certainty }`, `ClearDirection { dir }`, `RandomiseDirection { dir }` | The direction map | p56 to p58 | Directions 1 to 5 are read-only: the engine refuses the write |
| S9 | the page's measured load into the CHORD LEDs; clock state | Overload, 200 | p93, p102 | Page-side; not the engine's |

Not requested: keyboard transpose, force-to-scale from MIDI in, control maps (wave 4).

## 6. The controller's shape

`panel.rs` is one 458-line file whose single `input` function handles every control. Seven blocks added to it would be conflicts waiting to
happen, and the Panelwright is one hat in one crate. S0b splits it with **no change of behaviour**, proved by the 47
fixtures staying green and the assertion count unchanged:

| Module | Owns |
|---|---|
| `numeric.rs` | the quadrant's context, number entry, the main knob's target |
| `pitch.rs` | the inner ring: pitch reading and entry, C keys |
| `scale.rs` | SCALE SEL, MOD, CAD, Select-compose, name keys, ESC lock |
| `chord.rs` | the CHORD block: view, strum, polyphony |
| `time.rs` | multipliers, the triangle keys, TEMPO held |
| `order.rs` | chains, Follow, hypersteps, directions |
| `modulate.rs` | events and EFF |

## 7. Contracts, and who changes what

| Change | Where | By | Tier |
|---|---|---|---|
| The ring's keys become controls: `circle.num.1` to `.10` (the plate labels them) and `circle.ring.1` to `.13` (positional, until Q32 is answered, then renamed to the function they are); the numeric quadrant's LED positions are some of the `circle.num.*` and `circle.ring.*` keys, which ones being Q43; the open items this closes are removed from `open` | `contracts/controls.json` r2, ADR-0012 | Conductor (Panelwright proposes) | High |
| `layout` r3: the 23 promoted keys get ids; the count test (P7-1: 242 drawn, 5 not) is updated by the ADR | `apps/web/layout/panel.layout.json` | Forge, under the ADR | Medium |
| The display export and the new event kinds | `apps/web/engine/ABI.md`, revision 3 | Forge, additive | Medium |
| New questions Q43 to Q56 | `tests/conformance/panel/QUESTIONS.md` | Panelwright | Low |
| The grammar lines of section 3 | `crates/octoface/tests/panel_fixtures.rs` | Panelwright | Low |
| New fixtures | `tests/conformance/panel/<family>/` | Panelwright | Low |
| Commands of section 5 | `crates/octocore` | Metronome | Medium |
| Drawing text for the readouts | `apps/web/src/panel.ts` | Forge | Medium |

Nothing here edits a threshold. `harness/baseline.txt` only grows. If S0 changes the count in P7-1, the ADR says why and
does not loosen it: the new count is higher.

**Amended by A1 (README, ADR-0012, proposed).** The first row above becomes ten controls, `circle.num.1` to `.10` (n 295 to
304, no `led`), and the thirteen ring keys get no ids until they are named. The second row promotes ten keys, not 23, and the
counts are 257 controls, 252 drawn, 27 inert. The rest of this section stands.

## 8. Risks

| Risk | Consequence | Mitigation |
|---|---|---|
| The hardware's LED rendering of numbers is not in the manual | A faithful-looking ring that lights the wrong keys | Section 2 principle 1; `provisional` list; ask the owner (D5) before S1 ends |
| The ring's thirteen unnamed keys have no identity | S3, S5 and S8 wait on the owner | Logical ids by function (what the manual names), positions by order and flagged, as D-P7-1; a later answer is a data change |
| Hold gestures on a mouse | Unusable on a laptop | D6, before S4 |
| Track and Grid modes are not built | Half the circle's features (multipliers, program, directions, grid scale) cannot be reached | D3: the series is cut at a gate; S1 to S6 stand alone |
| Panelwright is one hat in one crate | Slices serialise; a long PR blocks the next | S0b split; one slice per PR; 15 fixtures at most per PR |
| Engine commands grow the ABI | Churn for the page | Additive per slice (principle 6) |
| The tutorials are 2007 | A fixture asserts old behaviour | The v5.30 text wins; a tutorial fixture cites both and, on a conflict, the page |
| 18 features is a lot | The work never finishes | Slices; each is a demo; the tutorial fixtures say when the USP is real |

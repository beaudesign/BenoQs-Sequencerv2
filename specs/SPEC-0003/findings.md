# SPEC-0003 findings

What was read, what was counted, what the circle's functions stand at, and the questions only a person with an Octopus
can answer. Every count is from `main` at `6c789e7` (the work's base) and can be rerun.

## 1. What was read and how

- **The manual, in full.** The owner attached `Octopus_ReferenceManual_v530-2.pdf`. It was extracted page by page and
  compared with `reference/manual/pages/p001` to `p112` of this repo after collapsing whitespace: **112 of 112 pages
  are equal**, at an offset of two (the repo's printed page `NNN` is the PDF's page `NNN+2`). So the attached file *is*
  the manual this repo holds, and every `[pNNN]` in this spec is a printed page of both. The ten tutorial pages
  (`tutorial-01` to `tutorial-10`, the PDF's pages 115 to 124) were read as well.
- **The repo:** `contracts/controls.json`, `apps/web/layout/panel.layout.json`, `crates/octoface/src` and its fixture
  runner, the `Command`, `TrackAttr` and `StepAttr` enums of `crates/octocore/src/types.rs`, the domain types
  (`Page.scale`, `Grid.global_scale`, `ChordPool`, the chain fields, `DirectionSlice`, `Step.event`), the web engine
  module's `ABI.md`, `tests/conformance/panel/QUESTIONS.md`, and SPEC-0002 `product.md`, `tech.md` and `findings.md`.
- **Not read:** any photograph (`CLAUDE.md` rule 7: the plate's geometry is `panel.layout.json`'s numbers), the release
  notes and the blog beyond what SPEC-0002 quotes, the engine's tests (a feature is called "present" when its type or
  function exists, not when its behaviour was audited), a real Octopus.

## 2. The circle's functions against the repo

`yes` means present and used; `type` means the engine has the state but no command sets it; `no` means absent.
"Panel" is the controller (`octoface`).

| Function | Pages | Inventory | Placed | Panel | Engine logic | Engine command |
|---|---|---|---|---|---|---|
| Tempo shown and set | p77 | yes (`circle.tempo`, `main.enc`) | yes | no | host | page-side `setTempo` |
| Bank view | p77 | yes (`circle.select`) | yes | no | yes (banks in `Grid`) | `SetActivePage` |
| Number entry | p17, p52, p57 | open (Q32) | 10 inert keys | no | n/a | n/a |
| Pitch reading | p46, p52 | yes (12 notes) | yes | no | n/a | n/a |
| Pitch entry, C keys | p46, p52 | yes; `low_c` unplaced (Q33) | 13 of 14 | no | yes | `SetTrack`, `SetStep` |
| Page scale, force, compose, base, lock | p71, p72 | yes (MOD, SEL, CAD) | yes | no | yes (`scale.rs`, `Page.scale`) | no |
| Cadence (CAD) | p71 | yes | yes | no | no | no |
| Grid scale | p72 | yes | yes | no | yes (`Grid.global_scale`) | no |
| Scale names, Chr. | p71 | `chr` yes; names open | `chr` unplaced | no | n/a | no |
| Chord View and Octave View | p20, p21 | yes (chord x7) | yes | no | yes (`ChordPool`) | no |
| Strum | p22 | n/a | n/a | no | yes (`tables`) | yes (`StepAttr::Strum`) |
| Polyphony, random picks, rests | p22, p23 | n/a | n/a | no | yes | no |
| CC resolution | p53 | n/a | n/a | no | no | no |
| Track multiplier | p49, p50 | open (Q21) | `hw.transport.left` inert | no | yes (`multiplier_num`, `_den`) | no |
| Step LEN multiplier | p16 | n/a | n/a | no | yes | yes (`StepAttr::LengthMultiplier`) |
| Pause and re-trigger | p50 | yes (`transport.pause`) | yes | no | yes | yes (`TrackAttr::Paused`) |
| Program and bank change | p50, p51 | yes (`keys.program`; Select) | yes | LED only | no | no |
| Chains: build, remove, base, show | p47, p48 | yes (`circle.chain`) | yes | no | yes | no |
| Follow, presets | p48, T05 | `follow` yes; presets no | `follow` unplaced | no | no | no |
| Hyperstep | p31 to p33 | n/a | n/a | no | yes | yes (`StepAttr::Hyperstep`) |
| Direction maps, `certainty_next` | p56, p57 | n/a | n/a | no | yes | `TrackAttr::DirectionRaw` only |
| Step events, event range | p34, p35 | n/a | n/a | no | yes (`Step.event`) | no |
| Effector (EFF) | p59 to p61 | yes (mutator) | yes | no | yes | yes (`IsFeeder`, `IsListener`) |
| The 200 clock key | p93 | yes (`circle.clock200`) | yes | no | master clock yes (ADR-0009); slave page-side | `octoweb_set_clock` |
| Overload on CHORD LEDs | p102 | yes (chord x7) | yes | no | no | n/a |

Read across: **nothing in the circle works from the panel today.** What is present is mostly engine logic and some of
the writers; what is missing is the controller, the readouts, the ring's identities, and about eight commands
(`tech.md` section 5).

## 3. Findings

- **F1. Of the circle's 42 controls the controller binds five** (PAGE, STEP, PLAY, EDIT, Stop) and one LED (Program).
  The page draws 38 of the 42, and 25 further keys that exist on the plate and in no inventory.
- **F2. The circle has no display, in the controller or on the page.** The controller has no output but commands,
  intents and an LED frame (the P5 plan, D-P5-5), and the page shows no value for any knob (D-P7-5). A value cannot be
  entered, checked or taught.
- **F3. The numeric quadrant and the inner circle are, as far as the manual says, LEDs on keys the plate has.** The manual
  says the "100 position" is lit (tutorial 5, T05) and "an orange dot at the 1 position" (p54), and the quadrant must show values
  to 17 and -9 to 9, which is more positions than the ten labelled keys: so it is probably those and some of the
  unnamed ring keys (Q43). Either way they need no widget, only LED writes to keys that need ids. That is why S0 comes first and why this spec adds no token.
- **F4. Number entry is stated, and is the matrix's too.** "A double-click will set a 10's value and a single-click
  will set a 1's value" [p17] for the matrix keys; the circle's keys do the same [p52, p57]. It is the one rule that
  covers tempo, velocity, certainty, program and bank numbers. What the manual does not say is which key is which.
- **F5. The outer ring has thirteen keys with no identity.** The manual numbers its keys in Roman numerals up to XXXI:
  XII (TEMPO), XIV (Select), XX (Chr.), XXVII (Follow), XXVIII (X), XXIX to XXXI (presets), and "Maj, Min etc" for the
  scales. The plate's outer ring has 27 keys placed: ten labelled 1 to 10, TEMPO, Select, 200 and X, and **thirteen whose
  printed names are unreadable** (about four pixels high in the photograph). The manual accounts for at least five
  functions that must be among those thirteen or beside them (Chr., Follow, the three presets) and for a number of
  scale names it never lists, and says nowhere which key is which. This is Q32 and the largest single unknown. The
  design absorbs it (`tech.md` section 2 and S0), but the owner, or someone with the hardware, can close it in
  minutes by reading the ring.
- **F6. About half of the circle needs modes that do not exist.** Track mode holds the multipliers, program and bank,
  map factors and the direction map; Grid mode holds the grid scale. Only Page and Step are built. S1 to S6 are the
  features reachable from Page and Step (`plan.md`); S7 and S8 wait at Gate T.
- **F7. The engine can already do most of the USP.** Directions with `certainty_next`, chains, hypersteps, events, the
  effector, chords with polyphony and strum, scales and multipliers all exist as engine state or logic. The unique
  patterns are an interface problem, not an engine one. (Present is not audited: `findings.md` of SPEC-0002 section 3.)
- **F8. The tutorials are fixtures waiting to be written.** T01 to T06 are button-by-button. They are the cheapest
  end-to-end check that the USP is reachable from the panel (`plan.md`).
- **F9. One state has two homes.** The strip's Tempo and MIDI Clock select (page furniture) and the panel's Tempo and 200
  keys are the same values. Without a rule they will drift. CA6 is the rule.
- **F10. The strum, multiplier and hyperstep tables are in the manual as numbers.** p22 (54), p49 (19 rows), p32 (14
  rows by 16 columns): each can be a generated fixture, which makes them the safest tests in the series.

## 4. New questions, Q43 to Q56

They continue `tests/conformance/panel/QUESTIONS.md` (Q01 to Q42), where the Panelwright will file them when S0 starts.
**What the spec assumes** is the provisional choice a slice may ship; each is written in the code naming its number, and
no fixture asserts it. "Who" is who can settle it.

| Q | Question | Pages | What the spec assumes | Who |
|---|---|---|---|---|
| Q43 | How does the numeric quadrant draw a number: which LEDs for 120, for 50, for a factor of 11, for strum -3? What is the tempo range, and how far does a detent move it? | p15, p54, p57, p77, B | The meaning is asserted. The rendering is the p15 rule (tens red, ones green) on the number keys, and is `provisional`. A detent is one unit (as Q39). The range is the page's 20 to 300 | Hardware |
| Q44 | A half-entered number: does a double click on `k` set the tens digit and keep the ones, or set the value to 10k? | p17, p57 | Sets the tens, keeps the ones; a single click sets the ones | Hardware |
| Q45 | Which MIDI octave is "octave 3"? What do the dots show at octave 0 and at the top? What does each press of Low C do? | p46, p52 | Octave `n` holds notes `12n` to `12n+11`; the fixtures assert the dot count against `n`, not `n` against a note. Low C: first press to C, second press down an octave | Hardware |
| Q46 | What does ESC do while the scale is being composed (Select flashing)? | p72 | The same as in SCALE mode: the page stays forced and SEL flashes orange | Hardware |
| Q47 | How is a chord cleared whole? | p20 | By toggling its notes off | Hardware |
| Q48 | Does strum -n play the same delays as +n with the notes in the opposite order? | p22 | Yes | Hardware |
| Q49 | Random picks, rests and certainty: do two runs of one pattern differ? The hardware's do. A replayable engine needs a seed | p22, p56 | The engine's generator is seeded by the host. The page seeds it from the clock at start and keeps the seed in the saved state, so any session replays exactly (CA5) | Owner |
| Q50 | How long is "the next 1/16 beat of the master clock" in ticks? It depends on D0, the tick length | p50 | 12 ticks | Owner (D0) |
| Q51 | What do chain presets XXX and XXXI chain? XXIX is "10 separate tracks" | T05 | Nothing: XXX and XXXI are `pending` | Hardware |
| Q52 | How is an empty slice shown in the direction map? | p56, p57 | No lights in rows 1 to 9 (the manual says "none, one or more") | Hardware |
| Q53 | Bank change: which controller number, which port and channel? | p51 | Bank select MSB (controller 0) on the track's port and channel | Hardware |
| Q54 | What is "port full" in a browser, where Web MIDI gives no back-pressure? | p102 | The number of messages queued more than a threshold ahead of now, per port; the threshold is a spike S1 number | Owner |
| Q55 | What is the gesture that sets the polyphony "on the chord buttons"? | p23 | A click on chord button `n` sets polyphony `n`; holding shows the chord | Hardware |
| Q56 | Which scales are on the name keys? | p71, p72, p86 | None: compose-by-notes and Chr. work without names, and the names wait for the list | Hardware |

### Open questions from SPEC-0002 that this spec depends on

| Q | In | Slice it blocks or constrains |
|---|---|---|
| Q01 | The 200 cycle | S9 (M-6) |
| Q05 | "Main" and "top right" rotary: one knob or two | S1 onward (this spec reads them as one) |
| Q06 | Row order of attributes | S7 (T-13) |
| Q09, Q10 | Which SEL; Select against SCALE MOD | S3, S5 |
| Q14 | Program change at the red 200 LED | S9 |
| Q17 | Where the chain selection and base indicator come from | S5 |
| Q21 | Which is the play key with one triangle | S7 |
| Q23 | What the MODE block holds | S0 (`hw.mode.map`) |
| Q24, Q25 | The double-click window; the flash rate | S1 onward: `dblclick` uses a provisional constant |
| Q32, Q33 | The ring's keys; the inner ring's notes | S0 |
| Q39, Q40, Q42 | A detent; legato; both rows and ring at once | S1, S2 |

## 5. What this spec does not claim

- That the plate's 13 unnamed ring keys are scale names, presets and Follow. They may be. The numbers 27 against 31 say
  only that the manual numbers more keys than it names.
- That the engine's behaviour matches the manual. SPEC-0002 section 3 says it was not audited; this spec's tests (the
  strum, multiplier and hyperstep tables, the Brownian bound) are how it will be.
- That a person will find the result faithful. The fixtures say the controller does what the manual says. Only someone
  with an Octopus says it feels like one (`plan.md`, "Done for the whole series", item 3).

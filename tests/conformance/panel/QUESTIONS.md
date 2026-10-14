# Panel questions

Things the manual does not settle and only a real Octopus, or the owner, can. **Q01 to Q20 are
the twenty items of `specs/SPEC-0002/findings.md` section 6, in that order**, and are not
repeated here. Questions found while building the controller continue from Q21. The numbers are
used by `contracts/controls.json` (`pending` and `open`) and by the `# question:` header of a
pending fixture (`pending/`).

Kept by the Panelwright. An answer is recorded in the Amendments list of ADR-0007 and the
fixture it frees moves out of `pending/` with the assertions the answer supports. Until then a
question is open, and the controller's behaviour for it is either unasserted or written in the
code as a provisional choice that names its number.

| Q | Question | Pages | Where it bites |
|---|---|---|---|
| Q21 | Which keys does the manual mean by "Play"? The MODE block has a green PLAY button (p066); the transport has a Play button (p016, p049, "Stop & Play") and "play buttons with one, two or four triangles" (p049). One key, two, or five? | p016, p049, p066 | `mode.play`, `transport.play`, open group `transport.triangles`; pending `the_mode_play_key_and_the_transport_play_key` |
| Q22 | The MIX TARGET row: stores 1 to 5 (p018), Mix Target MAP 1 to 4 (p038), control maps 0 to 5 (p074, p091), ATR and VOL PAN MOD EXP (p071 to p074). How many physical keys, and which are the same key? | p018, p038, p074 | open group `target.row` |
| Q23 | What does the MODE block hold? p066 and p089 name PLAY and STEP in it; PAGE, TRACK, GRID and EDIT are named as mode or LED buttons without saying they are in it. | p013, p015, p066, p067, p068, p079 | `mode.*` `pending` |
| Q24 | The double-click interval, and the hold threshold (`tech.md` section 9, item 1). | p027, p034, p038, p068, p070 | pending: double-click on EDIT, on a step; two quick clicks on EDIT |
| Q25 | The flash rate and duty cycle (`tech.md` section 9, item 2). | none | The UI only. The controller says `Flash`; it never times it |
| Q26 | Do the transport Play, Stop and Pause keys have an LED, and what does it show? p067 gives the Record key one; no page gives the other three one. | p049, p067 | `transport.stop`, `transport.pause` `pending`; pending `the_transport_play_led` |
| Q27 | What does the Program LED show when PLAY mode is not engaged? p066 says Red in PLAY mode. | p066 | pending `program_led_outside_play_mode` |
| Q28 | In Step zoom, what do the 15 unselected row 0 keys show? p013 makes row 0 the track's 16 steps and gives no colours. The controller shows the Page view colours, and no fixture asserts it. | p013 | pending `row_zero_leds_of_unselected_steps_in_step_zoom` |
| Q29 | ALN: p050 says "the ALN functionality"; p086 and p096 name an EXC / ALN button. One key with two names? Where is it, and is it a mutator? | p050, p086, p096 | `mutator.aln` `pending` |
| Q30 | Is there one Mute button per matrix row (p067, "its 'Mute' button"), and are they the MUT column of p082? | p014, p067, p082 | open group `mute.row` |
| Q31 | Does a click on Step Mode with no matrix key do anything? p013 describes hold and press only. | p013 | pending `a_bare_click_on_step_mode` |
| Q32 | The outer circle's number keys and scale keys: how many, and where? The chain selectors XXIX to XXXI appear only in the 2007 tutorial (T05). | p052, p071, T05 | open group `circle.numbers` |
| Q33 | Which note is each of the twelve inner circle keys, and what is their order on the ring? | p046, p052 | `circle.note.k1` to `k12` `pending` |
| Q34 | Hold Step Mode and press a key while EDIT is in preview or perform: does it zoom, or play the step? | p013, p068, p069 | pending `step_zoom_while_edit_is_preview` |
| Q35 | In the Step zoom table a step can be "Off" and "Chord/Event" at once. Which row does a chord on a toggled-off step use? Only the footnote on the Event and Skip row mentions the step being off. | p014 | pending `a_chord_step_that_is_toggled_off_in_step_zoom` |
| Q36 | Does Program (keep), or stopping the sequencer, end PLAY mode? p066 says Program makes the changes permanent and that stopping loses them; it says PLAY pressed again "exits the PLAY mode". | p066 | pending `program_ends_play_mode`, `stopping_ends_play_mode` |
| Q37 | What do a chord step and a hyperstep show in Page view? p034 and p038 give an event Orange; p014's Chord/Event and Hyperstep rows are Step zoom's. | p014, p034, p038 | pending `a_chord_step_or_hyperstep_in_page_view` |
| Q38 | Does a matrix key toggle its step when pressed or when released? p014 skips a step by holding it and clicking MUT, so a toggle on press would flip the step first. | p014 | pending `holding_a_step_and_clicking_mut_skips_it` |
| Q39 | How far does one detent of an attribute knob move the value? p068 says "every click of the encoder" changes it; nothing says a fast turn moves it further, and a real encoder may accelerate. | p015, p016, p068 | pending `a_detent_may_be_more_than_one_unit`; the single-detent fixtures in `step_attributes/` assert one unit |
| Q40 | What does LEN do below 1/192? p016 says the last four LEDs light Green and the step is legato (no note off). The engine clamps `LengthTicks` to 1 to 192 and has no legato state. | p016 | pending `a_length_below_one_tick_is_legato` |
| Q41 | Do the attribute knobs edit a grabbed step in Page view, in which EDIT states, and does a turn play the step again (p068, "with every click of the encoder the steps is re-triggered")? Does Step zoom with EDIT in preview do the same? | p068 | pending `a_grabbed_step_in_page_view_takes_the_knobs` |
| Q42 | p068 shows the step velocity in the numeric quadrant of the outer circle and the pitch in the inner circle (EDIT preview); p015 shows both in their matrix rows. Both at once, or one in each mode? | p015, p068 | pending `the_numeric_quadrant_and_the_inner_circle_show_velocity_and_pitch` |
| Q43 | How does the numeric quadrant draw a number: which LEDs for 120, for 50, for a factor of 11, for strum -3? What is the tempo range, and how far does a detent move it? | p015, p054, p057, p077 | SPEC-0003 assumes, provisional and asserted by no fixture: The meaning is asserted. The rendering is the p15 rule (tens red, ones green) on the number keys, and is `provisional`. A detent is one unit (as Q39). The range is the page's 20 to 300. Settled by: Hardware |
| Q44 | A half-entered number: does a double click on `k` set the tens digit and keep the ones, or set the value to 10k? | p017, p057 | SPEC-0003 assumes, provisional and asserted by no fixture: Sets the tens, keeps the ones; a single click sets the ones. Settled by: Hardware |
| Q45 | Which MIDI octave is "octave 3"? What do the dots show at octave 0 and at the top? What does each press of Low C do? | p046, p052 | SPEC-0003 assumes, provisional and asserted by no fixture: Octave `n` holds notes `12n` to `12n+11`; the fixtures assert the dot count against `n`, not `n` against a note. Low C: first press to C, second press down an octave. Settled by: Hardware |
| Q46 | What does ESC do while the scale is being composed (Select flashing)? | p072 | SPEC-0003 assumes, provisional and asserted by no fixture: The same as in SCALE mode: the page stays forced and SEL flashes orange. Settled by: Hardware |
| Q47 | How is a chord cleared whole? | p020 | SPEC-0003 assumes, provisional and asserted by no fixture: By toggling its notes off. Settled by: Hardware |
| Q48 | Does strum -n play the same delays as +n with the notes in the opposite order? | p022 | SPEC-0003 assumes, provisional and asserted by no fixture: Yes. Settled by: Hardware |
| Q49 | Random picks, rests and certainty: do two runs of one pattern differ? The hardware's do. A replayable engine needs a seed | p022, p056 | SPEC-0003 assumes, provisional and asserted by no fixture: The engine's generator is seeded by the host. The page seeds it from the clock at start and keeps the seed in the saved state, so any session replays exactly (CA5). Settled by: Owner |
| Q50 | How long is "the next 1/16 beat of the master clock" in ticks? It depends on D0, the tick length | p050 | SPEC-0003 assumes, provisional and asserted by no fixture: 12 ticks. Settled by: Owner (D0) |
| Q51 | What do chain presets XXX and XXXI chain? XXIX is "10 separate tracks" | T05 | SPEC-0003 assumes, provisional and asserted by no fixture: Nothing: XXX and XXXI are `pending`. Settled by: Hardware |
| Q52 | How is an empty slice shown in the direction map? | p056, p057 | SPEC-0003 assumes, provisional and asserted by no fixture: No lights in rows 1 to 9 (the manual says "none, one or more"). Settled by: Hardware |
| Q53 | Bank change: which controller number, which port and channel? | p051 | SPEC-0003 assumes, provisional and asserted by no fixture: Bank select MSB (controller 0) on the track's port and channel. Settled by: Hardware |
| Q54 | What is "port full" in a browser, where Web MIDI gives no back-pressure? | p102 | SPEC-0003 assumes, provisional and asserted by no fixture: The number of messages queued more than a threshold ahead of now, per port; the threshold is a spike S1 number. Settled by: Owner |
| Q55 | What is the gesture that sets the polyphony "on the chord buttons"? | p023 | SPEC-0003 assumes, provisional and asserted by no fixture: A click on chord button `n` sets polyphony `n`; holding shows the chord. Settled by: Hardware |
| Q56 | Which scales are on the name keys? | p071, p072, p086 | SPEC-0003 assumes, provisional and asserted by no fixture: None: compose-by-notes and Chr. work without names, and the names wait for the list. Settled by: Hardware |

Q43 to Q56 are SPEC-0003's (`specs/SPEC-0003/findings.md` section 4; the circle). Each says what the spec assumes so that a slice may ship one named provisional choice (D7); none is built yet, and when one is, the code names its question number.

## Provisional choices in the controller

Places where the controller had to show or do something and the manual does not say. Most are
asserted by no fixture; where one is, the entry says so. Each names its question.

- Step zoom shows the zoomed track's 16 steps in row 0 in the Page view colours (Q28).
- The EDIT LED keeps showing its state in Step zoom (p068 describes it "in PAGE mode" only).
- The Step zoom gesture works in every EDIT state (Q34).
- A chord or event on a toggled-off step takes the Chord/Event row (Q35).
- A step that is a hyperstep takes the Hyperstep row before the Chord/Event and On rows. The
  table does not order them.
- A matrix key toggles its step when it goes down, so holding a step and clicking MUT will flip the
  step first (Q38).
- In Page view a chord step is orange steady, as an event is (p034, p038), and a hyperstep looks like
  any other step (Q37). The fixture that once asserted the chord case is cut back to the pages.
- Program and Stop end PLAY mode, so the PLAY LED returns to green. p066 says so only for pressing
  PLAY again (Q36). No fixture asserts it.
- The footnote "(*Orange)" on the Event and Skip row of p014 is read as steady orange, because the table
  writes "Flash" where it means flash. Fixture `table_event_and_skipped_but_toggled_off` asserts that reading.
- `Shine` is the manual's own word for a third LED state (p014, p021, p033). The two hyperstep fixtures
  assert the word, not what it looks like (Q03, the Forge's). `findings.md` item 3 had planned these
  as pending and `tech.md` section 4 lists Steady and Flash only, so this is a deviation the owner may want to
  reverse. Reversing it means moving the two fixtures to `pending/`, which needs an ADR, because the ratchet
  holds them (`cargo xtask baseline --remove fixture … --adr`).
- One detent of the VEL, PIT, LEN or STA knob is one unit, and clockwise is up (Q39). A knob turn outside
  Step zoom does nothing (Q41). The attribute rows are not drawn, because the manual gives no row order
  (Q06) and shows velocity and pitch in the numeric quadrant and the inner circle as well (Q42). The single-detent
  fixtures in `step_attributes/` assert one unit; no fixture asserts the rest.
- EDIT cycles in Step zoom as it does in Page view. PLAY pressed in Step zoom takes the snapshot and stays
  in Step zoom.
- A key-up that never arrives leaves the key held, and a held Step Mode turns every press into a zoom. The
  web app must release every key when the page loses focus; the controller has no `release_all` yet.

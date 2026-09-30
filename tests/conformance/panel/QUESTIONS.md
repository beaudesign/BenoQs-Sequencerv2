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
- EDIT cycles in Step zoom as it does in Page view. PLAY pressed in Step zoom takes the snapshot and stays
  in Step zoom.
- A key-up that never arrives leaves the key held, and a held Step Mode turns every press into a zoom. The
  web app must release every key when the page loses focus; the controller has no `release_all` yet.

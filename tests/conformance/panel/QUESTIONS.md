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
| Q26 | What do the transport Play, Stop and Pause LEDs show while playing and while stopped? The manual gives no colours. | p049 | pending `the_transport_play_led` |
| Q27 | What does the Program LED show when PLAY mode is not engaged? p066 says Red in PLAY mode. | p066 | pending `program_led_outside_play_mode` |
| Q28 | In Step zoom, what do the 15 unselected row 0 keys show? p013 makes row 0 the track's 16 steps and gives no colours. The controller shows the Page view colours, and no fixture asserts it. | p013 | pending `row_zero_leds_of_unselected_steps_in_step_zoom` |
| Q29 | ALN: p050 says "the ALN functionality"; p086 and p096 name an EXC / ALN button. One key with two names? Where is it, and is it a mutator? | p050, p086, p096 | `mutator.aln` `pending` |
| Q30 | Is there one Mute button per matrix row (p067, "its 'Mute' button"), and are they the MUT column of p082? | p014, p067, p082 | open group `mute.row` |
| Q31 | Does a click on Step Mode with no matrix key do anything? p013 describes hold and press only. | p013 | pending `a_bare_click_on_step_mode` |
| Q32 | The outer circle's number keys and scale keys: how many, and where? The chain selectors XXIX to XXXI appear only in the 2007 tutorial (T05). | p052, p071, T05 | open group `circle.numbers` |
| Q33 | Which note is each of the twelve inner circle keys, and what is their order on the ring? | p046, p052 | `circle.note.k1` to `k12` `pending` |
| Q34 | Hold Step Mode and press a key while EDIT is in preview or perform: does it zoom, or play the step? | p013, p068, p069 | pending `step_zoom_while_edit_is_preview` |
| Q35 | In the Step zoom table a step can be "Off" and "Chord/Event" at once. Which row does a chord on a toggled-off step use? Only the footnote on the Event and Skip row mentions the step being off. | p014 | pending `a_chord_step_that_is_toggled_off_in_step_zoom` |

## Provisional choices in the controller

Places where the controller had to show or do something and the manual does not say. None is
asserted by a fixture. Each names its question.

- Step zoom shows the zoomed track's 16 steps in row 0 in the Page view colours (Q28).
- The EDIT LED keeps showing its state in Step zoom (p068 describes it "in PAGE mode" only).
- The Step zoom gesture works in every EDIT state (Q34).
- A chord or event on a toggled-off step takes the Chord/Event row (Q35).
- A step that is a hyperstep takes the Hyperstep row before the Chord/Event and On rows. The
  table does not order them.

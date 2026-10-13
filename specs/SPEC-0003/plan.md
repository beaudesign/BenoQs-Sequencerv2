# SPEC-0003 plan: the series

Each slice is its own pull request, one hat, one tier, red first (`CLAUDE.md`: every taste judgement becomes a test). A
slice starts when the one before it has merged: no pull request is stacked on another (the trap in
`.claude/skills/babysit/SKILL.md`). The agent merges nothing. Nothing here starts until the owner has answered the
README's decisions.

**How a slice runs.** The Panelwright writes the fixtures first and watches them fail. The Metronome adds the engine
commands the slice needs, with engine fixtures first. The Forge draws what the readout says and adds a Chromium test.
The Conductor writes any ADR before the code that needs it. Gate numbers go in every commit message. A fixture whose
answer is a numbered question is `pending` with the question written in it; a provisional behaviour names its question
in a comment and in `QUESTIONS.md`.

**Sizes** are fixture counts, floors not ceilings: the count only grows.

## The series at a glance

| Slice | Name | Features | Hat(s) | Tier | Needs | Fixtures |
|---|---|---|---|---|---|---|
| S0 | The ring is named | the inventory | Conductor | High | D5 | schema and count tests |
| S0b | Split the controller | none | Panelwright | Medium | S0 | 0 new, 47 unchanged |
| S1 | The number and the tempo | C1, C2 | Panelwright, Forge | Medium | S0b | 12 |
| S2 | The pitch ring | C3, C4 | Panelwright, Forge | Medium | S1 | 10 |
| S3 | Scales | C5 | Metronome, Panelwright, Forge | Medium | S2 | 12, 3 pending |
| S4 | Chords, strum, chance | C6, C7, C8 | Metronome, Panelwright, Forge | Medium | S3, D6 | 14 |
| S5 | Chains and hypersteps | C11, C12 | Metronome, Panelwright | Medium | S4 | 10, some pending |
| S6 | Events and the effector | C14, C15 | Metronome, Panelwright | Medium | S5 | 10 |
| | **Gate T**: Track mode exists (SPEC-0002 wave 2, the matrix half) | | | | D3 | |
| S7 | Time, program and bank | C9, C10, C17 | Metronome, Panelwright, Forge | Medium | Gate T | 16 |
| S8 | Order: directions | C13 | Metronome, Panelwright | Medium | S7 | 14 |
| S9 | The system keys | C16, C18 | Forge | Medium | S1 | 6 |

About 100 asserted fixtures, and the tutorial fixtures below. S9 depends only on S1 and may go any time after it.

## S0. The ring is named

Hat: **Conductor** (ADR-0012), then the Forge for `layout`. Tier **High** (`contracts/**`).

The 23 keys the plate has and no inventory names (the ten number keys and the thirteen ring keys) become controls, with
the ids of `tech.md` section 7 (`circle.num.1` to `.10`, `circle.ring.1` to `.13`). Where the manual names a function but not its key (Chr., Follow, the presets, the scale
names), the function keeps its id (`circle.chr`, `circle.follow`) and is bound to a ring key **only when the key's
identity is answered** (Q32, D5); until then the ring keys are `circle.ring.1` to `.13` by position, basis `order`.

**Amended by A1 (README; ADR-0012, proposed, draft PR #60).** S0 promotes **ten** keys, `circle.num.1` to `.10`, with no
`led`. The thirteen ring keys keep their `hw.ring.k` ids in the layout until each is named; then it takes the id the
contract already has, or one ADR-0007 amendment adds the control. The text above is the plan as approved; read S0-2 with
the numbers below.

| # | Observable | Test |
|---|---|---|
| S0-1 | Every function in `product.md` section 3.2 is an id in `controls.json` or an `open` item with a question number | `controls` inventory test, extended |
| S0-2 | The 23 promoted keys are no longer `hw.*`; the drawn count rises by 23 and the unplaced list only shrinks; the inert-key count falls from 37 to 14 *(as amended by A1: the ten promoted keys are no longer `hw.*`; drawn 242 to 252, controls 247 to 257, inert 37 to 27, the unplaced list unchanged)* | `layout.test.ts` and the two browser tests (P7-1, by the ADR) |
| S0-3 | The schema passes; ids are unique; no id is both a control and an `hw` key | `contract.rs` |
| S0-4 | The number of `open` items on the circle zones does not rise | The Referee's check |

## S0b. Split the controller

Hat: **Panelwright**. No behaviour changes. `panel.rs` becomes the modules of `tech.md` section 6, each with the code that
is today in `input`. Done when the 47 asserted fixtures pass unchanged, the 20 pending are unchanged, and the assertion
count is the same. It exists so S1 to S8 do not all edit one function.

## S1. The number and the tempo

Hat: **Panelwright** (controller, fixtures, grammar, `release_all`), **Forge** (ABI revision 3 for the readout, the
text for the accessibility tree, the Chromium tests). The engine read is `Grid`'s, which `PageView` already reads.

| # | Observable | Pages | Fixture |
|---|---|---|---|
| N-1 | In Page mode with nothing held, TEMPO is lit orange and Select green | p77 | `numeric/tempo_is_orange_and_select_green_in_page_mode` |
| N-2 | `expect display numeric = tempo <bpm>` follows the engine's tempo | p77 | `numeric/the_quadrant_shows_the_engine_tempo` |
| N-3 | The main knob moves the tempo one unit per detent; a turn the engine refuses sends nothing it would refuse | p77, Q39 | `numeric/one_detent_is_one_bpm` |
| N-4 | A single click on a number key sets the ones, a double click the tens | p17, p57 | `numeric/a_click_sets_the_ones_and_a_double_click_the_tens` |
| N-5 | `100` is 100 and its double click is 0 | p57 | `numeric/the_100_key` |
| N-6 | Select shows the bank view: banks with a playing page green, the one in the editor flashing orange | p77 | `numeric/select_shows_the_banks_that_are_playing` |
| N-7 | A green bank key goes to that bank's playing page | p77 | `numeric/a_bank_key_goes_to_its_playing_page` |
| N-8 | In bank view the main knob steps through the playing pages | p77 | `numeric/the_knob_steps_through_playing_pages` |
| N-9 | TEMPO leaves bank view | p77 | `numeric/tempo_returns_to_the_tempo_display` |
| N-10 | `release_all` releases every held key and sends no command | tech 4 | `numeric/release_all_lets_go_of_a_held_key` |
| N-11 | The strip's Tempo and the quadrant are one value, both ways | CA6 | Chromium |
| N-12 | The readout is a sentence in the accessibility tree: "Tempo, 120" | CA7 | Chromium |
| N-13 | *(pending)* How the quadrant draws a number: which LEDs for 120 | Q43 | `pending/how_the_quadrant_draws_a_number` |
| N-14 | *(pending)* A half-entered number: whether a double click sets the tens and keeps the ones | Q44 | `pending/a_double_click_keeps_the_ones` |

**Done when** a person can turn the knob and watch the tempo change on the strip, and key in 96 with two clicks of
`9` and `6`. **Heard:** the sequence speeds up when they do.

## S2. The pitch ring

| # | Observable | Pages | Fixture |
|---|---|---|---|
| P-1 | The combined pitch of the track and the step shows as red dots (the octave, from C up) and a green dot (the note) on the twelve note keys | p15, p52 | `pitch/g_in_octave_3_is_three_red_and_a_green_g` |
| P-2 | The exception: D in octave 3 is two red dots and an orange D | p52 | `pitch/d_in_octave_3_is_two_red_and_an_orange_d` |
| P-3 | In Step zoom a note key sets the pitch within the shown octave: `SetStep PitchOffset` | p52 | `pitch/a_note_key_sets_the_step_pitch` |
| P-4 | Upper C: one octave up; Low C: to the C first, then an octave down | p46 | `pitch/upper_c_and_low_c` |
| P-5 | The PIT knob and the ring agree: turning the knob moves the dots | p15, Q42 | `pitch/the_knob_moves_the_dots` |
| P-6 | The ring shows the same pitch for a step in EDIT preview as in Normal | p68 | `pitch/preview_shows_the_same_pitch` |
| P-7 | A held matrix step 14 to 16 does the p17 quick keys on pitch (default, octave down, octave up) | p17 | `pitch/quick_keys_on_pitch` |
| P-8 | The readout text is "Pitch, G, octave 3" | CA7 | Chromium |
| P-9 | *(pending)* Which octave number is which MIDI octave; the dots at the extremes; each press of Low C | Q45 | `pending/the_octave_numbering` |
| P-10 | *(pending)* Both rows and ring at once in preview | Q42 | already pending |

## S3. Scales

| # | Observable | Pages | Fixture |
|---|---|---|---|
| K-1 | On a fresh page SCALE SEL is green. Pressing it turns it red and the ring shows the twelve notes, C orange, the upper C unlit | p71 | `scale/force_to_scale_starts_chromatic_on_c` |
| K-2 | Pressing the red SEL turns it green | p71 | `scale/the_red_sel_turns_it_off` |
| K-3 | While forced, Select flashes orange and the Chr. LED is orange; a note key toggles that note in or out | p71 | `scale/select_composes_the_scale` |
| K-4 | SCALE MOD makes the next note key the base; the scale carries over to the new base | p71 | `scale/mod_moves_the_base` |
| K-5 | ESC leaves the page forced, hides the notes and flashes SEL orange; pressing the orange SEL returns to editing | p72 | `scale/esc_locks_the_scale` |
| K-6 | CAD flashes red; a scale change rewrites the page's track pitches | p71 | `scale/cad_rewrites_track_pitches` |
| K-7 | A forced page plays no note outside its scale (engine) | p71 | engine fixture `scale_forces_pitch_into_the_scale` |
| K-8 | A chromatic page is exempt from the grid scale; any other page overrules it (engine) | p72 | engine fixture |
| K-9 | The scale is one value per page; changing page changes the ring | p71 | `scale/the_scale_belongs_to_the_page` |
| K-10 | The readout text is "Scale, C D E ..., base C, forced" | CA7 | Chromium |
| K-11 | *(pending)* Grid scale needs Grid mode | p72 | `pending/the_grid_scale` |
| K-12 | *(pending)* A scale by name | Q56 | `pending/a_scale_by_name` |
| K-13 | *(pending)* Select against SCALE MOD; what ESC does while composing | Q10, Q46 | already pending, and new |

## S4. Chords, strum, chance

Needs D6 first: the strum gesture holds a CHORD button and turns the knob.

| # | Observable | Pages | Fixture |
|---|---|---|---|
| Ch-1 | In Step zoom the single-note chord button is lit orange | p20 | `chord/the_single_note_button_is_orange` |
| Ch-2 | Holding a CHORD button shows the chord in Chord View: the base flashes orange; notes within an octave orange; the second octave green; the third red | p20 | `chord/chord_view_colours` |
| Ch-3 | A note key cycles off, orange, green, red, off | p20 | `chord/the_toggle_sequence` |
| Ch-4 | A chord holds at most seven notes | p20 | `chord/the_eighth_note_is_refused` |
| Ch-5 | Select (flashing orange) switches to Octave View; MOD flashes orange, SEL and CAD are green | p20 | `chord/select_switches_to_octave_view` |
| Ch-6 | The six note states of Octave View (orange, green, red, orange flash, shine green, shine red) | p21 | `chord/octave_view_note_states` |
| Ch-7 | A double click on a note with no copies sets it in all three octaves; with copies, only here | p21 | `chord/a_double_click_stacks_or_unstacks` |
| Ch-8 | A double click on a CHORD button enters Octave View | p20 | `chord/double_click_enters_octave_view` |
| Ch-9 | Holding a CHORD button and turning the knob sets the strum from 0 to 9 (green) and 0 to -9 (red) | p22 | `chord/strum_levels` |
| Ch-10 | The strum delays each note by the table of p22: 54 numbers | p22 | engine fixture `strum_table_matches_the_manual` |
| Ch-11 | A strum on a single note plays it and six repeats | p22 | engine fixture |
| Ch-12 | The chord block shows the chord size in red, the polyphony in green, equal in orange | p22 | `chord/polyphony_leds` |
| Ch-13 | Polyphony below the chord size picks that many notes at random; above it adds rests. Same seed, same notes | p22, p23 | engine fixtures, seeded |
| Ch-14 | A single note with polyphony 2 is a 50% rest (counted over a fixed seed's 1,000 plays; the bound is the measured count) | p23 | engine fixture |
| Ch-15 | *(pending)* The gesture that sets the polyphony; whether strum -n mirrors +n; clearing a chord whole | Q55, Q48, Q47 | pending |

## S5. Chains and hypersteps

Provisional on Q09 (which SEL) and Q17 (where the chain selection and its base indicator come from); the fixtures that
need them are `pending` and the behaviour is written as a named provisional choice.

| # | Observable | Pages | Fixture |
|---|---|---|---|
| H-1 | With tracks selected (flashing orange) X builds a chain: the top selected track is the head and the rest the tail | p47 | `chain/x_builds_a_chain_from_the_selection` |
| H-2 | A track already in a chain is removed from it and the old chain shrinks | p47 | `chain/a_chained_track_leaves_its_old_chain` |
| H-3 | Selecting a chained track shows the head red, members green, the held one flashing orange (if the head is held, only the flash) | p47 | `chain/showing_a_chain` |
| H-4 | Holding a chained track and pressing X removes it | p47 | `chain/x_removes_the_held_track` |
| H-5 | Toggling the chain indicator between orange and red switches the base between the tracks' own and the head's; muted tracks are un-muted | p48 | `chain/the_base_switch` |
| H-6 | Muting a member mutes the whole chain when the head is the base | p48 | engine fixture |
| H-7 | Follow goes green to red and the view follows the chase-light through the chain | p48 | `chain/follow` |
| H-8 | Holding a track selector and pressing a step in another row makes a hyperstep: the two flash green and orange, the step's LEN becomes 192 | p31 | `hyper/engaging_a_hyperstep` |
| H-9 | One hyperstep per hyped track; none inside a hyped track | p31 | engine fixture |
| H-10 | Destroying: hold the hyped track's selector and press any step of its row; LEN returns to 12 | p31 | `hyper/destroying_a_hyperstep` |
| H-11 | Holding the hyped track's selector makes its hyperstep shine red | p33 | `hyper/identify_the_hyperstep` |
| H-12 | The hyped track plays at the speed the table of p32 gives | p32 | engine fixture, one case per row of the table |
| H-13 | *(pending)* The three presets | Q51 | pending |

## S6. Events and the effector

| # | Observable | Pages | Fixture |
|---|---|---|---|
| E-1 | In Step zoom a SEL attribute chosen becomes an event: it flashes, orange for a factor, green for a value; the step LED is orange | p34 | `event/choosing_an_attribute_makes_an_event` |
| E-2 | Pressing the flashing attribute clears it | p34 | `event/clearing_an_event` |
| E-3 | The AMT of the step is the change per play, modulo the range | p34 | engine fixture |
| E-4 | The event range appears in the quadrant and the main knob sets it; the maxima are 17, 16 (DIR) and 32 (MCH) | p34 | `event/the_range_in_the_quadrant` |
| E-5 | The change is bounded by the base value and the base plus the range | p35 | engine fixture |
| E-6 | A step AMT of 0 discards the event's offset | p35 | engine fixture |
| E-7 | An event runs on the beat, whatever the step's STA | p35 | engine fixture |
| E-8 | EFF cycles off, green (feeder), red (listener), orange (listening feeder), off | p61 | `effector/the_eff_cycle` |
| E-9 | A feeder's offsets add onto listeners with a lower track index; a muted feeder has no effect | p60, p61 | engine fixtures (the 9, 6, 3, 2 example sums to -2) |
| E-10 | The AMT map factor is itself modulated by an AMT event | p35 | engine fixture |

## Gate T: Track mode

Not this spec's, and S7 and S8 cannot start without it. D3 says when.

## S7. Time, program and bank

| # | Observable | Pages | Fixture |
|---|---|---|---|
| T-1 | In Track mode TEMPO held shows the track multiplier in the MCH row; a click sets a multiplier (red), a double click a divisor (green); valid values are orange | p49 | `time/tempo_held_edits_the_track_multiplier` |
| T-2 | The 19 multipliers of the table of p49, by hold, click and double-click of the triangle keys | p49 | `time/the_multiplier_table` (19 fixtures, generated from the table) |
| T-3 | The LED reading: red multipliers, green divisors; the exceptions 1.5, 1/1.5, 8, 1/8, 16, 1/16 | p49 | `time/multiplier_leds` |
| T-4 | While playing, a change applies on the next 1/16 of the master clock; stopped, at once | p50 | engine fixture |
| T-5 | A step LEN multiplier 1 to 8 by the same keys, in Step zoom | p16 | `time/step_len_multiplier` |
| T-6 | Pause on a held track pauses it; pressing it again advances it at once; Play resumes it on the next 1/16 | p50 | `time/pause_and_retrigger` |
| T-7 | With the sequencer stopped, a double click on Pause un-pauses every track | p50 | `time/double_pause_unpauses_all` |
| T-8 | In Track mode the quadrant shows the program number 0 to 128; Program sends a change on the track's channel; held while turning, sends as it goes | p50, p51 | `program/program_sends_a_change` |
| T-9 | Select toggles to the bank number; Select again sends the bank change | p51 | `program/select_sends_a_bank_change` |
| T-10 | Program and bank are sent once when a page is enabled for play | p51 | engine fixture |
| T-11 | A map factor shows in the quadrant: 1 to 17, orange 1 neutral, green below, red above; LEN and STA 0 to 16 | p53, p54 | `time/the_map_factor_display` |
| T-12 | In MAP mode with MCC the chord buttons set the CC resolution 1 to 7 | p53 | `time/cc_resolution` |
| T-13 | *(pending)* Which key is the one-triangle key; the row order | Q21, Q06 | pending |

## S8. Order: directions

| # | Observable | Pages | Fixture |
|---|---|---|---|
| D-1 | Directions 1 to 5 refuse an edit; 6 to 16 allow it | p56 | `direction/the_first_five_are_read_only` |
| D-2 | A double click on DIR enters the direction map: row 0 shows the index being edited in orange, the slice being edited flashing green | p57 | `direction/entering_the_map` |
| D-3 | Rows 1 to 9 show the slice's triggers in red, one per row; a row holds one trigger | p57 | `direction/one_trigger_per_row` |
| D-4 | `certainty_next` appears in the quadrant and the main knob sets it | p57 | `direction/certainty_in_the_quadrant` |
| D-5 | `5` double-clicked is 50; `100` is 100; `100` double-clicked is 0 | p57 | `direction/the_certainty_shortcuts` |
| D-6 | 100% always goes to the next slice; 0% to the previous | p56 | engine fixture (exists) |
| D-7 | A slice's triggers all play before the next slice is chosen | p56 | engine fixture |
| D-8 | An empty slice picks a trigger at random, from the engine's generator | p56 | engine fixture |
| D-9 | CLR restores forward in 6 to 16; RND fills the map | p57 | `direction/clr_and_rnd` |
| D-10 | ESC leaves the map; the change is kept | p57 | `direction/esc_keeps_the_change` |
| D-11 | A DIR step event takes effect on the following step | p34 | engine fixture |
| D-12 | *(pending)* How an empty slice shows; the map in a track chain | Q52 | pending |

## S9. The system keys

| # | Observable | Pages | Test |
|---|---|---|---|
| M-1 | The 200 key shows the page's clock state: off, orange master, green slave, red slave with echo | p93 | Chromium |
| M-2 | Pressing it changes the strip's select, and the strip's select changes the key | p93, CA6 | Chromium |
| M-3 | Switching is refused outside Grid and Page mode | p93 | `system/200_only_switches_in_grid_or_page` |
| M-4 | CHORD1 is red when port 1's send queue is over its limit; CHORD2 the same for port 2 | p102 | Chromium (a stalled output) |
| M-5 | CHORD3 to 5 orange at about 80% of the worklet's block budget; CHORD6 and 7 red over it | p102 | Chromium (a loaded block) |
| M-6 | *(pending)* The 200 cycle and program change at red | Q01, Q14 | already pending |

## The tutorial fixtures

The 2007 tutorials are written as button-by-button walk-throughs, which is a fixture's shape. They are the
end-to-end check that the USP works from the panel alone. They are older than v5.30, so each cites its page and, where
the two differ, the manual wins and the difference is a comment.

| Fixture | Source | Starts after | Asserts |
|---|---|---|---|
| `tutorial/t01_t02_lfo` | T01, T02 | S7 | A track's CC map goes out as a CC stream; its period follows the host track's length, its multiplier and a chain; its amplitude follows the map factor and a step event |
| `tutorial/t03_editing_directions` | T03 | S8 | Direction 6 programmed from the panel plays the trigger order the tutorial gives |
| `tutorial/t05_chain_and_certainty` | T05 | S8 | The default 10-track chain (XXIX) with direction 6 bounces as the tutorial says; certainty 0 on every slice plays 16 down to 1 |
| `tutorial/t06_brownian_motion` | T06 | S8 | With `certainty_next` 90, a fixed seed gives a walk whose forward moves are within the measured bound of 90% |

## Done for the whole series

1. The 100 or so fixtures pass in native and in the WebAssembly build, and a recorded session replays to one hash (CA5).
2. The four tutorial fixtures pass.
3. A person who knows the hardware follows tutorials 3, 5 and 6 on the page with the panel alone. (This is the owner's
   check, and the one measurement here that is a person.)
4. `QUESTIONS.md` lists every provisional choice with its number, and the hardware check list is the owner's.

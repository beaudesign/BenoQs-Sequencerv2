# SPEC-0003 product spec: the circle

Citations are `[pNNN]` for a page of the CE v5.30 reference manual in `reference/manual/pages/` (the printed page
number; the PDF page is two higher), `[TNN]` for the 2007 tutorials (`tutorial-NN.txt`, older than v5.30: the v5.30
text wins on a conflict), `[B]` for the blog post and `[W]` for `panel.layout.json` (the measured plate). Control ids
are those of `contracts/controls.json` (`controls.json`); `hw.*` ids are keys the plate has and the inventory does not.
Everything here is a draft until the owner approves it (README).

## 1. What the circle is

The matrix is where you write a pattern. **The circle is where you say what the pattern means**: its tempo, its
pitches, its scale, its chords, the order it runs in, how fast each track runs, which track bends which. On the plate
it is the whole right-hand side: the *outer circle* (a numeric quadrant that reads out values, a ring of number keys
and named keys), the *inner circle* (twelve note keys and two C keys, which show a pitch, a chord or a scale and take
pitches in), the SCALE keys inside it, and the MODE, TRANSPORT and CHORD blocks round it. The blog calls it "the
spiral" [B].

Today the page draws all of it and nearly all of it is inert (`tech.md` section 1). A user can toggle steps and
turn four knobs that show no value. They cannot read a tempo from the panel, enter a pitch, build a chord, force a
scale, chain tracks, change an order. **Without the circle the page is a step toggler. With it, it is an Octopus.**

## 2. The USP: five sources of variation

A step sequencer says *what plays when*. The Octopus also says *in what order, how fast, in which key, how often,
and what changes it*, and lets any track change any other. Everything below is set from the circle, or from the matrix
with a value read in the circle. Every variation has a visible parameter: none of it is a hidden "generate" button.

| # | Source | What the Octopus does that a grid does not | Where the circle comes in | Pages |
|---|---|---|---|---|
| V1 | **Order** | A track's direction is a table of 16 *slices*; each slice holds up to nine triggers (chase-light positions) and a `certainty_next` that chooses the next or the previous slice. 100% is forward, 0% is backward, 50% is a coin flip each slice: any walk from a straight run to Brownian motion. An empty slice picks a trigger at random. Directions 6 to 16 are yours; 1 to 5 are fixed | The numeric quadrant is `certainty_next`: `5` double-clicked is 50, `100` is 100, `100` double-clicked is 0 | p56 to p58, T03, T05, T06 |
| V2 | **Time** | Each track has its own clock multiplier from 16 down to 1/16, including 1.5 for triplets; a step can be up to 8 whole notes long; tracks chain end to end into structures longer than 16 steps, with the head's values as the base if you want; a *hyperstep* makes one step play a whole other track, stretched to the step's length, at the step's velocity and pitch | TEMPO (XII) held turns the matrix into a multiplier picker; the triangle keys of the transport; X (XXVIII), Follow (XXVII), the presets (XXIX to XXXI) | p16, p31 to p33, p47 to p50 |
| V3 | **Pitch** | A scale per page, forced on every note, and one for the whole grid; composed note by note or picked by name; a base tone; CAD rewrites the track pitches into the scale. Chords of up to seven notes over three octaves, strummed up or down | The inner circle is the scale, the chord and the pitch. SCALE MOD, SEL, CAD and the CHORD block | p20 to p22, p46, p52, p71 to p72 |
| V4 | **Chance** | Polyphony below the chord size picks notes at random from the chord; above it adds rests to the pool, so a single note with polyphony 2 plays half the time; empty direction slices; RND on any map | The CHORD block's LEDs *are* the polyphony: red is chord size, green is polyphony, orange is both | p22, p23, p56, p57 |
| V5 | **Modulation** | A *step event* changes a track attribute, or the factor of its map, every time the step plays (with a range, and a modulo). The *effector* projects the offsets of "feeder" tracks onto "listener" tracks below them. Together: evolving, partly unpredictable sequences from a small pattern | The numeric quadrant shows and sets the event range; EFF and the SEL column set the roles | p34 to p40, p59 to p63, T01, T02 |

Not the circle's, and not in this spec: PLAY snapshots, On-The-Measure, clusters and page sets (the performance
layer, SPEC-0002 wave 3) and recording (wave 4). They are the other half of what makes the instrument playable live.
The five above are what make its *patterns* unique.

**The promise to the owner.** After the slices below, someone who knows the hardware can follow tutorials 3, 5 and 6
on the page with no tool but the panel, and hear the order change (`plan.md`, the tutorial fixtures).

## 3. The circle, control by control

Columns: the manual's name · the inventory id (or `hw.*`, or *new*) · what it does · pages. "State" is in `tech.md`
section 1.

### 3.1 The numeric quadrant and the main knob

The quadrant is a display, not a key. The manual calls it the numeric field, the numeric quadrant and the top-left
quadrant of the outer circle. It has one meaning at a time, chosen by context:

| It shows | When | Set with | Pages |
|---|---|---|---|
| **Master tempo** | Page mode; TEMPO LED orange, Select LED green. The default | Main knob; the number keys | p77, T05 |
| **Bank view**: the banks that have a playing page (green), the one in the matrix editor (flashing orange) | Page mode, Select pressed. Tempo returns to tempo | A green button jumps to that bank's playing page; the main knob steps through the playing pages | p77 |
| **Step velocity** | Step zoom, and EDIT preview, with the step held | Main knob; the number keys | p68 |
| **An attribute value**, as the sum of the track's and the step's | MAP mode, a step held in row 0 (use preview, so no step toggles) | The number keys, "similar to entering a tempo value" | p52 |
| **A map factor** (1 to 17, 9 neutral; LEN and STA show 0 to 16, 8 neutral): an orange 1 is neutral, a green bar reduces, a red bar amplifies | Track mode with an attribute selector held; MAP mode | Main knob | p53, p54 |
| **The event range** (to 17; DIR 16; MCH 32) | Step zoom with an event active | Main knob | p34, p35 |
| **`certainty_next`**, 0 to 100% | Direction map, a slice chosen in row 0 | Main knob; the keys `5`, `100` (above) | p57, T05 |
| **Program number** 0 to 128 (0 sends nothing), or **bank number** after Select | Track mode; Program LED orange, Select green | Main knob; the number keys; Program sends, Select toggles to the bank number | p50, p51 |
| **Strum**, -9 to 9: green up, red down | Step zoom, a CHORD button held | Main knob | p22 |

Held-down TEMPO (XII) turns other things into number entry: in Step zoom the step LEN multiplier (red, 1 to 8) in the
LEN row, and in Track mode the track multiplier in the MCH row, both picked on the matrix [p16, p49].

The **main knob** (`main.enc`, top right) is the one knob of this block. Which thing it edits is whatever the quadrant
shows [p22, p34, p54, p57, p77]. The manual says "main" and "top right" rotary and does not say they are one (Q05); the plate
has one big knob there, so this spec reads them as one.

### 3.2 The outer ring's keys

The manual numbers the keys of the outer circle in Roman numerals (XII, XIV, XX, XXVII to XXXI, so at least 31 of
them) and names few. What it says about each:

| Key | Id | Does | Pages |
|---|---|---|---|
| **TEMPO** (XII) | `circle.tempo` | Orange in Page mode: the quadrant shows tempo. Held: the multiplier pickers above, and the transport's Stop and Play are available. In bank view, returns to tempo | p16, p49, p77 |
| **Select** (XIV) | `circle.select` | Page mode: bank view. Track mode: toggles program and bank number. With SCALE SEL red: composes the scale (flashes orange, Chr's LED orange). With a CHORD button held: switches Chord View and Octave View | p20, p51, p71, p77 |
| **Program** | `keys.program` | Track mode: sends a program change on the track's channel, and keeps sending as the knob turns if held. PLAY mode: keeps the changes | p50, p51, p66 |
| **200** | `circle.clock200` | The MIDI clock state. Off, single click: orange, master (clock and transport out of both ports). Double click: green, slave. In slave, single click: red, with echo. Switching only in Grid or Page mode. Saved with the machine state | p93; the prose, the diagram and the release notes disagree (Q01) |
| **Chr.** (XX) | `circle.chr` | The chromatic scale; its LED is orange when it is the active scale | p71 |
| **Scale names** ("Maj, Min etc") | *new* | Put the named scale's structure on the base tone. The manual names two and says "etc" (Q56) | p71, p72, p86 |
| **X** (XXVIII) | `circle.chain` | With tracks selected (flashing orange): builds a chain, top track the head. With one chained track held: removes it | p47 |
| **Follow** (XXVII) | `circle.follow` | Green to red: the view follows the chase-light through a chain | p48 |
| **Chain presets** (XXIX, XXX, XXXI) | *new* | Shortcut buttons for stock chains; XXIX is "10 separate tracks". Only the 2007 tutorial names them | T05 |
| **Number keys** | `hw.num.1` to `hw.num.10`, `hw.ring.*` | Number entry. For whole-number attributes "a double-click will set a 10's value and a single-click will set a 1's value" [p17, written for the matrix keys; p52 says the circle's keys work "similar to entering a tempo value"]. `5` double-clicked is 50, `100` is 100, `100` double-clicked is 0 [p57]. Which key is which is open (Q32) | p17, p52, p57 |

### 3.3 The inner circle

Twelve note keys and the two C keys [W: 13 keys on the plate, `circle.low_c` unplaced, Q33]. It always means *pitch
class and octave*, and it has a different content in each context:

| It shows | When | Reading | Pages |
|---|---|---|---|
| **A pitch** | Step zoom (PIT), Track mode (track PIT), a step held in MAP | The *combined* pitch of the track and the step [p15]. Red dots: the octave, one per octave from C. Green dot: the note. Three red from C and a green G is G in octave 3. Exception: two red from C and an orange D is D in octave 3 | p15, p46, p52 |
| **A scale** | Page mode, SCALE SEL red | Lit notes are in the scale. Orange is the base tone. Fresh page: all twelve lit, C orange, upper C unlit | p71 |
| **A chord, Chord View** | Step zoom, a CHORD button held | The base flashes orange. Notes within an octave of it: orange. The second octave: green. The third: red | p20 |
| **A chord, Octave View** | The same, with Select (flashing orange) | MOD, SEL and CAD are octaves 1, 2 and 3 (the current one flashes orange). Orange: set only in this octave. Green: set here and elsewhere. Red: set elsewhere only. Orange flashing: the root. Shining green: set in all three. Shining red: the root, set in all three | p20, p21 |
| **Save progress** | Saving | Out of scope here (SPEC-0002 wave 5) | p96 |

Keys: a **note key** enters a pitch (a pitch, a scale note, or a chord note, by context). **Upper C** transposes the
track one octave up. **Low C** transposes first to C and then one octave down [p46]. The manual says "the left and
right keys labelled C" [p52]: which is which is Q33.

**The SCALE block** (`circle.scale_mod`, `circle.scale_sel`, `circle.scale_cad`):

| Key | State | Meaning | Pages |
|---|---|---|---|
| SEL | green | Not forced to a scale | p71, p72 |
| SEL | red | Forced to the scale. Press: back to green | p71 |
| SEL | flashing orange | Forced, and scale editing locked (after ESC). Press: edit again | p72 |
| MOD | pressed | Choose a new base tone from the scale's notes; the signature moves with it. Pressed again: back to choosing notes | p71 |
| CAD | flashing red | While forcing: every scale change rewrites the track pitches in the page. "Not a good idea" on the default octave | p71, p72 |
| SEL, in GRID mode | | The same, for the whole grid. A page's own scale overrules it; a chromatic page is exempt (use it for drums) | p72 |
| SEL, with tracks selected in Page mode | | Keyboard transpose: red relative, green absolute | p78 (SPEC-0002 wave 4) |

### 3.4 The CHORD block

Seven buttons (`chord.b1` to `chord.b7`). It is the most overloaded block on the plate:

| Meaning | When | Pages |
|---|---|---|
| **Chord size and polyphony**: a red LED at the chord size, a green LED at the polyphony, orange where they are equal | Step zoom | p22 |
| **Chord selection**: hold a button and the inner circle shows that chord. A double click enters Octave View | Step zoom | p20 |
| **Setting the polyphony** "on the chord buttons". The gesture is not described (Q55) | Step zoom | p23 |
| **Strum**: hold a button and turn the main knob | Step zoom | p22 |
| **CC resolution**, 1 to 7 messages per step | MAP mode with MCC | p53 |
| **Overload**: CHORD1 red is port 1 over capacity; CHORD2 red is port 2. CPU: CHORD3 to 5 orange at about 80%; CHORD6 and 7 red when overloaded | Always | p102 |

### 3.5 The TRANSPORT and MODE blocks

| Control | Id | Does | Pages |
|---|---|---|---|
| Play, Stop, Pause, Record | `transport.play`, `.stop`, `.pause`, `.rec` | Run the sequencer. With a **track held**: Pause pauses it, and pressing Pause on a paused track advances it at once; Play resumes it on the next 1/16 of the master clock. With the sequencer stopped, a click on Pause in Track zoom sets the track's paused state, and a double-click un-pauses every track | p50, p67 |
| The play keys with one, two and four triangles | `hw.transport.left` and Q21 | Track multipliers in Track mode, step LEN multipliers in Step zoom, by hold, click and double-click | p16, p49 |
| GRID, PAGE, TRACK, STEP, PLAY | `mode.grid`, `.page`, `.track`, `.step`, `.play` | Choose the view. PAGE, STEP and PLAY are built (SPEC-0002 P5); TRACK and GRID are not | p13, p66, p79 |
| The sixth MODE key | `hw.mode.map` | Reads MAP on the plate [W]; not in any inventory (Q23) | |

## 4. How each feature should work

One row per feature. **Rule** is the manual's behaviour in one or two sentences; **Not stated** is what the manual
leaves to a person with an Octopus (the numbered questions are in `findings.md`). The slice that builds it is in
`plan.md`.

| # | Feature | Rule | Not stated |
|---|---|---|---|
| C1 | Tempo | The quadrant shows the master tempo in Page mode and the main knob changes it. The page's Tempo field and this are one value | The range (the page offers 20 to 300; the engine takes 1 to 999), the step of a detent, how the quadrant draws a three-digit number (Q43) |
| C2 | Number entry | Keys enter the number the quadrant shows. A single click sets the ones and a double click the tens [p17]; `100` and its double click are 100 and 0 [p57]. Step 13 to 16 of the matrix are quick keys (invert, clear, octave down, octave up) [p17] | What each key is (Q32), and whether a double click on `k` sets the tens digit and keeps the ones, or sets the value to 10k (Q44) |
| C3 | Pitch reading | Octave in red dots, note in green; the two exceptions of p52 | The exact dot positions for octaves 0 and 8, which octave is which (Q45) |
| C4 | Pitch entry | A note key sets the pitch within the current octave; Upper C and Low C move an octave. In Track mode the track pitch, in Step zoom the step pitch | Whether both the inner circle and the matrix PIT row show at once in EDIT preview (Q42, already open) |
| C5 | Scale | SEL toggles force-to-scale; Select toggles composing; a name key sets a structure on the base; MOD moves the base; CAD remaps track pitches; ESC locks. The grid has its own, below the page's | The list of scale names (Q56); whether Select and SCALE MOD are one key (Q10, open); what ESC does while composing (Q46) |
| C6 | Chords | Up to seven notes over three octaves above the step pitch. Chord View toggles each note off, orange, green, red, off. Octave View stacks per octave; a double click on a note with no copies turns it on in all three, and on a note with copies keeps only the current one | How a chord is cleared whole (Q47) |
| C7 | Strum | Hold a CHORD button, turn the main knob: 0 to 9 up, 0 to -9 down. Each strum level delays each note by the ticks in the table of p22 (note 7 at level 9 is 45 ticks). A single note strummed plays six repeats | Whether strum -n mirrors +n exactly (Q48) |
| C8 | Polyphony and chance | The notes played are the smaller of chord size and polyphony, picked at random from the chord; if polyphony is greater, the pool gets that many rests. Polyphony 2 on a single note is a 50% rest | The generator's seed handling across Stop (Q49) |
| C9 | Track multiplier | Table on p49 by hold, click and double-click of the three triangle keys; or TEMPO held, then the matrix. While playing, a change takes effect on the next 1/16 of the master clock | Which is the one-triangle key; the exact timing of "next 1/16" in ticks (Q21 and Q50) |
| C10 | Step LEN multiplier | 1 to 8, picked the same way; the longest step is 8 whole notes at master speed [p16]. Recording sets it automatically | |
| C11 | Chains | Select tracks, press X: the top selected track is the head, the rest the tail; tracks leave their old chains. Orange selection: tracks use their own base. Red: they take the head's. Muting a chain member mutes the chain when the head is the base. Follow moves the view | Where the chain selection and the base indicator come from (Q17); what the three presets chain (Q51) |
| C12 | Hypersteps | Hold a track selector and press a step in another row. The step becomes the trigger, its LEN becomes 192; destroying resets it to 12. A hyped track has one hyperstep; hypersteps do not nest. Click and hold the hyped track's selector: its hyperstep shines red | |
| C13 | Directions | Directions 6 to 16 are editable: 16 slices, nine triggers each, one per row, plus `certainty_next` in the quadrant. CLR restores forward; RND is the inspiration button | The unit of a "slice" shown as the matrix row 0 (clear); the reading of an empty slice in the display (Q52) |
| C14 | Events | In Step zoom a lit SEL attribute chosen becomes an event (flashing: orange changes the map factor, green changes the value). AMT is the change per play, modulo the range. AMT 0 discards the offset. Events run on the beat, ignoring STA | |
| C15 | Effector | EFF toggles green (feeder), red (listener), orange (both), off. Feeders pass VEL, PIT, LEN and MCC offsets to listeners with a lower track index, additively | |
| C16 | MIDI clock key | The 200 key's states of section 3.2 are the page's MIDI clock selector. They are one state with two views | The cycle itself (Q01) and program change at Red (Q14), both still open |
| C17 | Program and bank | Program number 0 to 128 (0 is none). Program sends a change; Select then Select sends a bank change. Both are sent once when a page is enabled for play [p51] | Port and the two-byte bank message (Q53) |
| C18 | Overload | CHORD1 and 2 for the ports, CHORD3 to 7 for the CPU. They reflect real load, so they are driven by the page's measured queue and worklet time | What "port full" means in a browser (Q54) |

## 5. Acceptance criteria

Every criterion is a measurement. The thresholds are the manual's own numbers; nothing here is a taste.

| # | Criterion | How it is measured |
|---|---|---|
| CA1 | **Every function the manual names for the circle is an id or a numbered question.** The count of `open` items in the circle zones of `controls.json` only falls | The Referee's inventory test, extended in S0 |
| CA2 | **Each slice ships with its fixtures, red first**, and the fixture count only grows | `cargo xtask` gate `conformance` (A1 of SPEC-0002) |
| CA3 | **Meaning is asserted, rendering is not guessed.** A display fixture asserts the value or the pitch or the chord notes. Which LED lights for a value is `provisional` until the hardware says (Q43) | `expect display ...` lines; the provisional list in `QUESTIONS.md` |
| CA4 | **The tutorials work.** T01 and T02 (an LFO made from a CC map, its period and amplitude), T03, T05 and T06 (directions, `certainty_next`, Brownian motion) run as fixtures of panel inputs and assert what the engine plays | `tests/conformance/panel/tutorial/` |
| CA5 | **Replay is exact.** A recorded panel session replays to the same event hash twice, in the native build and in the browser's WebAssembly build | The A2 check of SPEC-0002, extended with a session file |
| CA6 | **One state, two views.** Tempo and the clock state are the same value on the strip and on the ring: set one, the other changes, in both directions | A Chromium test |
| CA7 | **Reachable.** Every circle key is a tab stop in the plate's order, with the manual's name; the ring is drawn from the tokens with no new colour | The A8 check, extended; the slop lint |
| CA8 | **Nothing is lost on blur.** When the page loses focus every held key is released | A Chromium test (`release_all`, `tech.md` section 4) |

## 6. Not in this spec

SysEx; recording and MIDI-in transpose (wave 4); MIDI control maps and their learn mode [p74, p91]; the Device View and
the file formats (wave 5); phrases beyond what Step zoom already offers (the phrase editor is the matrix's); Grid and
performance modes; sound design. These belong to other halves of SPEC-0002 and are named only where a circle key
touches them.

# Octopus CE v5.30 manual: UI and interaction extraction, pp. 1-63

Scope: p001-p063 read in full, in order. Blank pages (footer only): p002, p008, p012. Citation form [pNNN] = pages/pNNN.txt (= printed page number). No engine maths reproduced; pointers only.
Vocabulary used by the manual: "single-click", "double-click", "hold" (press and keep pressed while doing something else), "grab" (= hold a track selector), "zoom", "mutator", "selector", "encoder/rotary/knob/main rotary", "shine" and "flash" LED states.

## 0. Source findings (not UI, but affect the spec)

- F1. The printed TOC page numbers [p005] do not match the real page files. Real chapter starts (heading peek of p064+, OUT OF RANGE, pointers only): Page Mode p065; Grid Mode p079; Recording p087; MIDI p093; Load/Save p095; Appendix p099. TOC says 71 / 89 / 99 / 109 / 113 / 119. The manual's own alphabetical index [p006-p007] agrees with the real files (Grid 79-86, Recording 87-92, MIDI 93-94, Load/Save 95-97).
- F2. INDEX.md rows `midi 109-112`, `load-save 113-118`, `appendix 113-124` are stale/wrong versus the files: pages/p113..p124 do not exist; MIDI is p093-094, Load/Save p095-097, Appendix p099+. Candidate finding for the Conductor. Nothing was edited.
- F3. The basic front-panel tour ("start-up section", pre-set chains, "Page" basics, ALN, how track selections are made) is NOT in p001-p063. This range covers Step, Track, Effector. Anything about Page/Grid mode UI here is only incidental.
- F4. "Shine" / "Shine_Red" / "Shine_Green" is used as an LED state distinct from solid and from Flash [p014, p021, p033, p038], but is never defined in this range. UNCLEAR: whether it is a dim glow, slow pulse, or fast flicker.
- F5. Buttons are cross-referenced by Roman numerals that are printed on the panel: circle TEMPO key = (XII) [p016, p049]; Follow button = (XXVII) [p048]; chain "X" button = (XXVIII) [p047]. The circle appears to be a clock face labelled I..XII (inference; the manual only names XII).
- F6. Alternate LED colourways exist: Red = Blue, Green = Yellow, Orange = Purple [p004]. The spec should treat red/green/orange as roles, not fixed hues.

## 1. Physical UI elements (names as the manual uses them)

### 1.1 Matrix
- Matrix: 16 columns (steps 1-16) x 10 rows numbered 0-9. Row 0 is the bottom row; row 9 is the top [p013, p027, p047, p052]. Evidence for 10 rows: default track pitches listed for "Tracks 9 - 0" [p043]; "row 9 to row 0" [p047]; toggle Range max 10 = "all tracks" [p039]. Track 0 is the lowest, track 9 the top [p039, p060].
- One matrix row = one track in Page view (row index = track index) [p047, p060]. Steps per track = 16; a track "of 16 steps" [p032]; 16 direction slices [p056].
- Step button = a matrix key. Bottom row (row 0) in Step zoom = the track's 16 steps; clicking one makes it the selected step [p013].
- Chase-light / chaser light: the moving light through the step positions of a track [p014, p056]. Colours in section 3.
- In map/attribute views the matrix rows 1-9 are a bar-graph area, 16 columns, row 0 = the track pattern [p052].

### 1.2 Column left of the matrix (selector column)
- "SEL column" / "selector column" / "selector LED column": one button+LED per matrix row [p015, p047, p051]. UNCLEAR: count is not stated in this range (implied 10 by the row count).
- Page view: each is a "track selector" (grab/hold, double-click) [p031, p042, p047].
- Step zoom: all attribute buttons light up, waiting to be selected as events [p015, p034]. Attributes named: VEL, PIT, LEN, STA, AMT, GRV, MCC (map-factor attributes; drawn Orange) and DIR, POS, MCH (direct-value attributes; drawn Green) [p034].
- Track mode: lit buttons = the track's attribute maps: VEL, PIT, LEN, STA, AMT, GRV, MCC, plus DIR (not a real map, opens the direction editor) [p051].
- Row order of the attribute rows in Step/Track zoom is not stated in p001-p063. UNCLEAR.

### 1.3 MUT / MUTATOR column ("mutators")
- Named mutator buttons in this range: TGL (toggle), SOL (solo), CLR (clear), RND (randomize), FLT (flat), ZOM (zoom), RMX (remix), CPY, PST (copy, paste), EFF (effector) [p013, p014, p028, p041-p043, p059]. ALN (align) is mentioned only as "the ALN functionality" [p050].
- "A lit up mutator indicates that it is available." All Track-mode mutators are also available from PAGE mode as soon as a track is selected [p041].
- "Main Mute button" / "Main MUT" / "MUT button": the mutate column's mute/skip key. In Step zoom it toggles the step's Skip; in Page view hold-step + MUT skips [p014]; in Track zoom its LED shows solo state [p011].
- In map mode, holding an attribute selector makes only CLR, RND, ZOM available [p051]; RMX is also usable in map view [p053].
- Step zoom uses "the Copy button" and "the Paste button" [p013].

### 1.4 MIX block ("MIXER block", "MIX encoders")
- One rotary encoder per row (rows 1-9 named, "the rotary of row #9 at the top") plus buttons/LEDs [p027]. Count = one per matrix row (inference). In Track view, "turn the desired Track MIX encoder" moves to a different track [p004].
- In Page mode with a step selection active, MIX encoders edit the NOT-selected steps in tracks that hold selected steps [p018].
- In map mode, ticking the MIX encoder of a row fetches that track into the map view [p053].
- In the phrase editor, MIX rotaries edit the selected attribute of phrase note = matching row; row 9's rotary changes phrase type; top four Orange LEDs = VEL, PIT, LEN, STA (top-down) [p027].

### 1.5 EDIT block ("EDIT encoders", "Editor block")
- Attribute encoders named: VEL knob, PIT knob, LEN knob, STA knob, GRV encoder, POS knob, DIR encoder, MCH rotary encoder [p015-p017, p028, p046, p053]. AMT and MCC are edited with "their respective edit knob" [p017] (not enumerated). UNCLEAR: total EDIT encoder count and layout.
- In Page mode with a step selection active, EDIT encoders modify all selected steps [p018].
- EDIT LED: toggling it to flashing Green or flashing Orange = "preview mode", so pressing steps does not change their toggle state [p052]. (Full Edit states are at p068-p069, out of range.)
- DIR encoder in Step zoom = Step Shift; with a single-track selection = Step Swap [p015, p019]. POS knob in map mode shifts the map [p053].

### 1.6 Main rotary
- "main rotary" / "main knob" / "main encoder": cycles phrase in the phrase editor [p027], sets strum with a chord button held [p022], sets event range [p034], sets map scale factor [p054]. Also "the rotary encoder on the top right" for the program number [p050] and "top right rotary encoder" for certainty_next [p057]. UNCLEAR: whether "top right" is the same physical knob as "main".

### 1.7 MIX TARGET row (below the matrix)
- Buttons "labelled 1,2,3,4,5" = step selection stores 1-5 [p018].
- "Map select buttons below the matrix (lit orange)": "Mix Target MAP 1 / 2 / 3 / 4" = Track toggle Mute / Solo / Record / Pause [p038]. UNCLEAR: whether MAP 1-4 are the same physical keys as store 1-4.

### 1.8 The circle (numeric and pitch displays)
- Outer circle: "numeric field/quadrant". Shows global tempo in PAGE mode [p050]; program number in Track mode [p050]; event interval [p034]; track map factor when an attribute is held [p054]; the "top left quadrant" = certainty_next in the direction editor [p057]. Has number keys: values are keyed in like a tempo entry [p052]. Keys named: 5, 100 [p057].
- Inner circle = "pitch circle": 12 note keys plus an "upper C" and a "low C" key [p046, p052]. Shows combined track+step pitch [p015]. Shows chord notes when a chord button is held [p020]. At its bottom, three buttons labelled MOD, SEL, CAD double as octave selectors 1-3 in Octave View [p020]. Shows save progress ("progress lights complete the circle") [p028].
- Circle TEMPO (XII) key: while held, matrix is a value editor (Step LEN multiplier row [p016], Track multiplier in MCH row [p049]); Transport Stop and Play stay available while it is held [p016, p049].

### 1.9 CHORD button block
- "CHORD button block at the bottom right of the front panel" [p020]. Chord notes: up to 7 [p020]; 7 chord note LEDs [p027]. Default state: the single-note chord button lit Orange [p020].
- Also used for phrase polyphony in the phrase editor (7 "note" buttons) [p027] and for MCC CC-map resolution in map mode [p053].

### 1.10 Transport
- Buttons: Play, Stop, Pause; and the ">", ">>", ">>>>" (one/two/four triangles) buttons [p049]. Named "transport bar", "transport area", "Transport Pause button", "transport Play button" [p016, p049, p050].
- "Play (Snapshot)" is a Page-mode item [p006]; Play "flashing Orange (Play Mode)" [p034]; "Snapshot" red [p038]. UNCLEAR: same button or a different one; the manual names both.

### 1.11 Mode and system buttons
- MODE selector section: PAGE mode button, STEP mode button ("Step Mode button"), TRACK mode button, GRID mode button [p013, p015, p028]. Each has an LED (e.g. "Track Mode led") [p011].
- ESC [p015, p018, p057]. CLEAR button (boot function) [p004]. Program button/key + Program LED [p050]. Select button/Select LED (also SEL) [p020, p050]. SEL button with a "Sel led" [p018]. UNCLEAR: whether "Select" and "SEL" are one button or several (SEL also appears as a chord-octave key at the bottom of the inner circle [p020]).
- "X (XXVIII)" = chain build/remove button [p047]. "Follow button (XXVII)" [p048].
- "200" button: MIDI clock master/slave state LED+button [p011].
- ZOM key doubles as zoom-in/out toggle across Page/Track/Step/Map [p014, p042, p053].
- Boot combos: reboot holding Esc = version on the matrix; reboot holding Clear = clear grid of all pages (not flash unless Saved) [p004].

### 1.12 MIDI ports
- Two MIDI ports selectable per track: port 1 and port 2 [p046].

## 2. Mode/view hierarchy and transitions

Named modes in range: GRID, PAGE, TRACK, STEP. Sub-views in range: Step Zoom (= STEP mode), Track Zoom (= TRACK mode), MAP mode (attribute map), Direction map, Phrase editor, Chord View / Octave View, Edit preview. Out of range (pointers only): Grid-Track mode, Mix mode, Edit states, Scale mode, Bank view, On-The-Measure [p005, p006].

```
GRID  >  PAGE  >  TRACK (track zoom)  >  MAP mode (attribute map)
                                      >  Direction map (DIR 6-16 only)
                                      >  Phrase editor (via GRV in SEL column)
             >  STEP (step zoom)       >  Phrase editor
                                       >  Chord View <-> Octave View (with chord button held)
```

### 2.1 PAGE -> STEP
- Hold Step Mode button + press a matrix key [p013].
- Double-click a step [p027, p034, p038].
- Within STEP, press a bottom-row (row 0) button to jump to another step of the same track [p013].

### 2.2 STEP -> PAGE
- Press ESC "anytime" [p015]; or the PAGE mode button [p015]; or the ZOM key ("In STEP mode the LED [ZOM] is lit Red ... pressing ZOM will exit") [p014].

### 2.3 PAGE -> TRACK
- Double-click a track selector [p027, p042].
- Hold a track selector and press ZOM ("same effect as double click") [p042].

### 2.4 TRACK -> PAGE
- ZOM. "Zooming out is indicated by a Red LED ... as opposed to an Orange one" [p042].
- ESC from Track is not stated explicitly. UNCLEAR. (ESC is stated for the direction map [p057] and STEP [p015].)

### 2.5 TRACK <-> MAP mode (attribute maps)
- Enter: hold an attribute selector (it flashes Orange; CLR/RND/ZOM lit) then ZOM; OR double-click the attribute button [p051].
- Leave: ZOM (now lit Red) returns to Track mode without un-selecting the attribute; ZOM again re-enters the map [p053].
- Map mode shows a Red chase-light in the row of the track being worked on [p052].
- Switch track inside map: tick the corresponding MIX encoder, either direction [p053].

### 2.6 TRACK -> Direction map
- In Track mode choose DIR 6-16 (1-5 read-only) then double-click the DIR attribute button [p057]. Only accessible from Track mode via double-click of the track DIR selector; no longer from Step mode in v5.30 [p011].
- Leave: ESC at any time returns to PAGE mode [p057].

### 2.7 Phrase editor
- From STEP: double-click the step, then double-click the GRV button in the SEL column [p027].
- From TRACK: double-click a track selector, then double-click the GRV button in the SEL column [p027].
- Recognised by all 7 chord LEDs lit [p027]. Exit method: not stated. UNCLEAR.

### 2.8 Chord View / Octave View (only while a chord button is held, in STEP)
- Holding a chord button shows the chord in the inner circle in Chord View; Select LED Orange [p020].
- Click Select (chord button still held) toggles Octave View (Select LED flashing Orange) [p020].
- Double-click a chord button (and keep holding it) enters Octave View directly [p020].

### 2.9 Selection state (PAGE)
- Hold SEL + click a step = step selection into the current store 1-5; ESC = deselect and empty that store [p018].
- Track selection lives on the SEL button (SEL LED Red), not in stores 1-5 [p018].

### 2.10 GRID
- Only used here for: "Go to GRID mode, stop, press GRID + Program" to save machine state [p028]. Details are p079+.

### 2.11 Click vocabulary in use
- Hold (grab track selector, hold SEL, hold Step Mode button, hold TEMPO (XII), hold chord button, hold Program, hold attribute selector, hold step in row 0).
- Single-click vs double-click carry distinct meanings on the same control (see section 4).

## 3. LED colour and flash semantics

### 3.1 Step LEDs (Page view matrix)
- Green: step is On [p018 "active step (Green)"].
- Red: step is Skipped [p014].
- Orange: step carries a Chord or an Event [p014, p034, p038].
- Flash: the step is selected/zoomed [p013]; flashing Green = selected active step in a selection [p018].
- Step-zoom LED table (selected step LED | Main Mute LED) [p014]:
  - Off: Flash Red | Off
  - Off & Skip: Flash Red | Red
  - On: Flash Green | Off
  - On & Skip: Flash Red | Flash Green
  - Chord/Event: Flash Orange | Off
  - Chord/Event & Skip: Flash Red | Flash Orange (Orange if Event & Skip but underlying step is toggled off)
  - Hyperstep: Shine_Red | Off
  - Hyperstep & Skip: Flash Red | Shine_Red
- Paste confirmation: the step LED ceases to flash [p013].

### 3.2 Chase-light
- Red for a track with MCC "none"; Orange if the track MCC is anything else [p004, p046]. (p017 words it as step MCC; treat p046 as the authority; UNCLEAR minor.)

### 3.3 Step value rows (Step zoom)
- VEL, PIT: Red LEDs = tens, Green LED = ones; negative offsets look the same plus 3 Green LEDs in columns 14-16 [p015]. MCC uses the same decimal style [p017]. AMT display style is not described in range.
- LEN: Green dot advances to 11, then Red value increments; Green step = 1/192, Red step = 12/192 = 1/16 [p015]. Legato = last 4 LEDs Green [p016].
- STA: Red bar left to right = push/retard; Green bar growing from position 16 leftwards = pull/advance [p016]. Empty row = default [p016].
- MCC none = 4 Green LEDs at the last positions (13-16) [p017, p046]. BENDER = Red dot at position 16; CHANNEL PRESSURE = Red dots at 15 and 16 [p046].
- GRV pointer LED colour = phrase bank: colour changes when 16 is passed [p017]; bank colours are Green, Red, Orange (order inferred from the p023 listing) [p023]. 0 = no phrase.
- POS (phrase time compression) appears only once a phrase is selected; 8 is neutral [p017].

### 3.4 Track value rows (Track zoom)
- MCH: Green 1-16 = MIDI port 1; Red 1-16 = MIDI port 2. Default channel 1 port 1 = Green light in position 1 [p046].
- LEN/STA/attribute scale factor: neutral = Orange 1 (dot at position 1); reduce = Green bar; amplify = Red bar [p054].
- Track multiplier display: Red LEDs = multipliers, Green LEDs = divisors [p049]. Special: 1.5 = Orange 2 + Red 1; 1/1.5 = Orange 2 + Green 1; 8 = Orange 4 + Red 2; 1/8 = Orange 4 + Green 2; 16 = Orange 4 and 2 + Red 1; 1/16 = Orange 4 and 2 + Green 1 [p049]. UNCLEAR: "LEDs off and lit Green at the same time denotes a divisor of 5" [p049] does not say which LEDs.
- Pitch display: Red dots = octave, Green dot = note; exception: two Red dots from C plus an Orange dot at D = D in octave 3 [p052].
- Transport-area Step LEN multiplier uses the same pattern [p016]; TEMPO-held matrix shows the current value Red and selectable values Orange [p016, p049].

### 3.5 Zoom / mode LEDs
- ZOM Orange = can zoom in; ZOM Red = zooming out [p042]. STEP mode: ZOM lit Red [p014]. MAP mode: ZOM lit Red [p053].
- Page view: Track Mode LED Green in v5.30 (legacy Off), for On-The-Measure [p011].
- Track zoom, when solo'd: Main MUT LED Green (legacy: flashed Green); flashes Green when armed for Solo (OTM) [p011].
- Program LED Orange and Select LED Green in Track mode [p050]. Select turned Orange = bank-change entry [p051].
- Select LED Orange = Chord View; flashing Orange = Octave View [p020].
- SEL LED Red = a track selection is stored on SEL; off = none [p018]. Selection Slot button flashing Orange = current store (double-click adds all On steps) [p018].
- Follow button: Green = off; Red = on [p048].
- Play "flashing Orange" = Play Mode (events lost at stop); Snapshot "red" = Snapshot Mode (events lost at stop) [p034, p038].
- 200 button: Orange = Master MIDI clock send; Red = Slave clock send (echoed); Off = Master no clock sent; Green = Slave no clock sent [p011].

### 3.6 Chords
- Chord View [p020]: base pitch flashes Orange; notes within 1 octave Orange; second octave Green; third octave Red. Manual toggle cycle: off > Orange > Green > Red > off.
- Octave View [p021]:
  - Note Orange = set only in the viewed octave
  - Green = set in the viewed octave and another
  - Red = set in another octave but not this one
  - Orange flash = chord root
  - Shine_Green = set in all 3 octaves; Shine_Red = root set in all 3 octaves
  - Octave selectors MOD / SEL / CAD = octaves 1 / 2 / 3; current one flashes Orange; the other two lit Green [p020]
- Chord block: Red LED = chord size; Green LED = polyphony; Orange LED = equal size and polyphony at that position [p022].
- Strum display: Green values 0..9 = strum up (turn main rotary CW); Red values -9..0 = strum down (CCW) [p022].

### 3.7 Phrase editor
- Row 0 flashing LED, in bank colour, = phrase index; nothing = phrase 0 [p027].
- 7 chord note LEDs Green (pressable); press to flash Orange = set phrase polyphony; press again to clear [p027].
- Row end LED Green = phrase note defined; Red = only for phrase #0 (read-only) [p027].
- Top row = phrase type as 1-4 Orange LEDs [p027]. Top four MIX LEDs Orange = VEL/PIT/LEN/STA; active one flashing Orange [p027].

### 3.8 Hypersteps, chains, effector
- Engaging a hyperstep: track selector and hyperstep both flash between Green and Orange [p031]. Hold the hyped-track selector: its hyperstep Shine Red [p033].
- Chain display (select a chain member): selected track flashes Orange; head Red; other members Green. Head = selected: single flashing Orange [p047].
- Chain base indicator: Orange = tracks play with own base values; Red = take head track values as base. Muted tracks un-mute when the base is changed [p048].
- EFF mutator: Green = feeder; Red = listener; Orange = listening feeder; Off = not participating [p061].

### 3.9 Events
- Attribute button in SEL column: Orange = event modulates the attribute MAP FACTOR (VEL, PIT, LEN, STA, AMT, GRV, MCC); Green = event changes the VALUE directly (DIR, POS, MCH) [p034]. Selected-as-event = flashing; step LED of an event = Orange [p034].
- Track toggle events: MAP 1-4 buttons lit Orange; activated = flashing Orange; flip-flop mode = Shine_Green [p038].
- Track Rotate (POS double-click) and Track Skip Rotate (DIR double-click) = Shine_Green; single-click POS/DIR = Step Event (Orange) [p038, p039].
- Clearing an event: flashing attribute button clicked so it is solid Orange/Green (or MAP button "green again"); or AMT 0 [p034, p040].

### 3.10 Direction map
- Row 0: Orange light = index of the position being edited; flashing Green = slice index being edited; overlap = flashing Orange [p057].
- Rows 1-9: Red lights = triggers in the selected slice; one trigger per row [p057].

### 3.11 Map view
- Value bars in rows 1-9; zero line on row 5 for signed attributes (VEL, PIT, STA); positive above, negative below [p052].
- Attribute button pressed and held flashes Orange [p051].

## 4. Workflows (button sequences in order)

### 4.1 Step zoom and basic step edit
- W1. Enter step zoom (a): in PAGE, hold Step Mode button, press matrix key [p013]. (b): double-click the step [p027]. (c): switch step within same track: press its bottom-row key [p013].
- W2. Select the step to work on: click step on row 0; it flashes [p013].
- W3. Toggle step: press TGL mutator on the selected step [p014].
- W4. Clear step: press CLR; defaults recalled; turns step off if it was on [p014].
- W5. Randomize step: press RND (amount fixed at 50; GRV and POS unaffected) [p014].
- W6. Copy step: in step zoom click Copy (all attributes). Paste: click Paste; step LED stops flashing when done [p013].
- W7. Skip step in zoom: select step, click Main Mute [p014]. Skip in Page view: hold step button, click MUT, repeat for more steps; un-skip: press the step once (red goes off) [p014].
- W8. Exit step zoom: ESC, or PAGE mode button, or ZOM [p014, p015].
- W9. Step Shift: in step zoom turn the DIR encoder; shifts all steps to the right of the flashing step in row 0 (advance right, or close up if next step is off, not skipped, not an event) [p015]. Overflow beyond step 16 is lost unless an over-run track is configured [p015]. Hypersteps do not move; shifted steps hop over them [p015]. Track AMT controls the over-run track [p046].

### 4.2 Step attributes
- W10. Edit VEL / PIT / LEN / STA / AMT / GRV / MCC: turn that attribute's knob [p015-p017].
- W11. Quick edit by matrix keys (step zoom) [p017]: integers (VEL, LEN, AMT, MCC): double-click = tens value, single-click = ones value. Quick keys on steps 13-16: header prints "Click" over Step 13 and "Double-click" over 14-16.
  - Velocity: 13 invert, 14 clear, 15 clear, 16 clear
  - Pitch: 13 invert, 14 default, 15 octave - (8vb), 16 octave + (8va)
  - Amount: 13 invert, 14 clear, 15 clear, 16 value -127 (mask)
  - MCC: 13 no action ("-"), 14 clear, 15 clear, 16 clear
  - Double-click on the GRV button advances to the next phrase group [p017].
  - UNCLEAR: which matrix row hosts these keys, and how keys 1-12 map to digits.
- W12. Track-level quick keys (Track zoom), header "Double-click" over all [p043]: Velocity: 13 default, 14 zero; Pitch: 13 default, 14 middle C (C5), 15 octave -, 16 octave +; Amount: 13 default, 14 zero, 15 clear. UNCLEAR: column-to-key mapping when fewer entries than keys.
- W13. Edit step LEN multiplier (1-8; also 1.5 in table): in STEP mode use the transport ">", ">>", ">>>>" buttons, or hold circle TEMPO (XII) and press a matrix key in the LEN row (current value Red, choices Orange) [p016]. Button combos on p016 match the track multiplier table [p049].
- W14. Ghost toggle: hold two or more step buttons in different rows, same column; then toggling steps in one of those rows toggles the same columns in the other [p017].

### 4.3 Step selection (PAGE)
- W15. Make a selection: hold SEL, click an active (Green) step; it flashes Green; MIX TARGET store (default 1) flashes [p018].
- W16. Use stores: click 1-5 in MIX TARGET; empty store = selection disappears; hold SEL and pick steps to fill it [p018].
- W17. Edit selection: EDIT encoders edit selected; MIX encoders edit unselected steps of tracks holding selected steps [p018].
- W18. Delete a store: with a selection active click ESC [p018]. Reboot deletes all stores; track selection is saved with the page [p018].
- W19. Add all On steps: double-click the current slot button (flashing Orange) [p018]. Double-click again when all On steps already selected = remove all steps and the store [p019]. Use: correct slave-clock recorded latency with STA [p018].
- W20. Step Swap: selection entirely on one track; turn the DIR encoder to swap with the track above or below [p019]. Steps with hyperstep are dropped from the selection [p019]. Passing through several tracks does not restore them [p019]. Tracks with AMT 0 only swap with each other; a track with AMT != 0 can swap independently but not with an AMT 0 track [p019].
- W21. Track selection shares the same virtual space as step selection; do not fill all five stores if track selection is wanted [p018].

### 4.4 Chords
- W22. Enter/view a chord: zoom into the step, hold a chord button; notes show in the inner circle [p020]. Toggle notes with the inner-circle keys: cycle off > Orange > Green > Red > off [p020].
- W23. Octave View: hold chord button, click Select (LED flashes Orange); or double-click the chord button and keep holding [p020]. Move between octaves with MOD (1), SEL (2), CAD (3) [p020].
- W24. Add octave-spaced notes: with no notes at that pitch in any octave, double-click the note key; turns on in all three octaves [p021].
- W25. Remove octave-spaced notes: from any octave view, double-click a note that exists in other octaves; it stays on in the current octave and turns off elsewhere. Remove upper-octave notes by turning them off in their octave [p021].
- W26. Stacked root can only be turned off in Octave View [p021].
- W27. Strum: zoom in, hold a chord button, turn the main rotary CW (0..9 up, Green) or CCW (-9..0 down, Red) [p022]. Also works on a single-note step (plays note + 6 duplicates) [p022].
- W28. Polyphony: press a chord button position to set polyphony (independent of chord size) [p022]. Random step rest: single-note step, polyphony 2 = 50% note, 4 = 25% [p023]. Step GRV does not work as expected on such steps [p023].
- W29. Chords may also be recorded from a MIDI keyboard (see Recording, p087+) [p020].

### 4.5 Phrases
- W30. Assign phrase to a step: in step zoom turn the GRV encoder right (0 = none, 1-16, colour bank changes after 16) [p017]. POS then controls time compression [p017].
- W31. Open the phrase editor: from a zoomed step, double-click the GRV button in the SEL column; or via Track mode, double-click a track selector then the GRV button [p027].
- W32. Change phrase: main rotary or GRV rotary [p027, p028].
- W33. Edit an attribute: press one of the 4 Orange MIX buttons (VEL/PIT/LEN/STA); MIX rotary of row n edits phrase note n; row-9 rotary edits phrase type [p027].
- W34. CLR: clears the phrase if non-empty; pressed again on an empty phrase restores the factory phrase; while grabbing an Orange attribute button, only that attribute [p027].
- W35. RND: randomize whole phrase, or only the grabbed attribute [p028].
- W36. CPY / PST: copy and paste between phrase pool slots; phrase 0 copy-only [p028].
- W37. POS knob in the editor: stretches or compresses timing to next straight/triplet/dotted version; GRV knob selects phrase [p028].
- W38. Save phrases (with full machine state): go to GRID mode, stop the machine, press GRID + Program, wait for progress lights to complete the circle [p028].
- W39. Set phrase polyphony: press one of the 7 note buttons (flashes Orange); press again to restore full polyphony [p027].

### 4.6 Hypersteps
- W40. Engage: in PAGE hold a track selector and press a matrix step in another row; both flash Green/Orange [p031].
- W41. Destroy: hold the hyped track's selector, press any step button in the hyped track's row [p031].
- W42. Identify: in Page view click-and-hold the hyped-track's select button; its hyperstep Shine Red [p033].
- W43. Variable step sequences recipe [p033]: hyperstep on step 1; on the hyped-track mute/skip steps 9-16; on the hyperstep track mute steps 5-16; hyperstep track LEN 8; zoom hyperstep and set LEN 192/192 -> 48/192; on step 4 make a step event for LEN with AMT -1.
- W44. Constraints: multiple hypersteps per track allowed; a hyped-track has only one hyperstep; no nesting [p031]. Making a hyperstep sets LEN 192/192; destroying resets 12/192 [p031].

### 4.7 Step events
- W45. Create an attribute event: double-click step to zoom, click one of the SEL-column attributes; it flashes; step LED goes Orange [p034].
- W46. Set event size: AMT of the step (+/-, modulo if beyond the range) [p034]. AMT 0 discards the offset made by an event [p035].
- W47. Set event range: in step mode, activate an event, main rotary changes the interval shown in the outer-circle numeric field [p034].
- W48. Clear an event: zoom the step, press the flashing SEL attribute button until solid [p034]; or press the flashing MAP button until green [p040].
- W49. Track toggle events: double-click step to zoom; click Mix Target MAP 1 (Mute) / 2 (Solo) / 3 (Record) / 4 (Pause); flashing Orange; AMT sign = On/Off; double-click the MAP button = flip-flop (Shine_Green) [p038].
- W50. Target track = AMT value (positive On, negative Off); AMT 0 = off, so track 0 uses AMT 10 [p039]. Range (max 10) = number of tracks toggled, wraps around; one Range per track per toggle type [p039].
- W51. Track Rotate event: zoom step, double-click the POS attribute button (Shine_Green). Single-click = Step Event POS. Switching resets AMT. Positive = forward, negative = backward [p038, p039]. Mask a step by AMT -127; mask an event by VEL -127 [p038].
- W52. Track Skip Rotate event: zoom step, double-click the DIR attribute button (Shine_Green). Single-click = Step Event DIR. Mask via AMT -127 [p039].
- W53. Track toggle events fire before the step plays; muting a track that only targets other tracks blocks its toggles [p039, p040]. Paused track can only be un-paused by another track's un-pause event or manually [p040].

### 4.8 Track operations (Track zoom or PAGE with a track grabbed)
- W54. TGL = mute/unmute; SOL = solo within its page; CLR = preset values (MCH kept; pitch back to 60); RND = random step pattern only; RMX = variation (AMT = amount) [p041-p043]. CLR called on a PAGE recalls factory pitch assignment; "grab a Track [in PAGE] and clear it, everything is reset" [p042]. UNCLEAR: conflicts with MCH-kept on p041.
- W55. FLT: select 2 or more tracks in a page; destination = lowest-index track in selection; apply FLT; lowest 7 pitches stacked as chord; VEL/LEN/STA from last active step; MIDI channel agnostic; GRV reset to 0 unless same [p042]. Best practice: mute the destination first [p042].
- W56. Track copy/paste: select, copy to an internal clipboard, paste into destination; the clipboard holds a REFERENCE (paste takes the latest data) [p043].
- W57. Track pitch direct entry: press an inner-circle note key; upper C = one octave up; low C = first to C, then one octave down [p046]. Default pitches, tracks 9-0: C3 D3 E3 G3 A3 C5 D5 E5 G5 A5 [p043].
- W58. Change track: in Track view turn the target track's MIX encoder [p004].

### 4.9 Track chains, multipliers, pause
- W59. Create a chain (PAGE): create a track selection (flashing Orange), press X (XXVIII); plays top to bottom (row 9 to 0); top track = head [p047]. UNCLEAR: how the selection is made (not in range).
- W60. Show a chain: select a chain track; head Red, other members Green, selected flashes Orange [p047].
- W61. Remove from chain: hold the track's button, click X (XXVIII) [p047]. A track removed from an old chain when re-chained [p047].
- W62. Base switch: toggle the chain selection indicator between Orange (own base) and Red (head base) [p048]. UNCLEAR: which control toggles it.
- W63. Follow: press Follow (XXVII) Green -> Red; view follows the chase-light through a chain while zoomed into a chain track [p048].
- W64. Track multiplier: Track mode, use ">", ">>", ">>>>" buttons with hold/click/double-click combos (table [p049]); double-click ">>" and ">>>>" set 1/2 and 1/4. Alternative: hold circle TEMPO (XII); multiplier row shown in the MCH row; single-click = multiplier (Red), double-click = divisor (Green); Pause sets track paused/play; Stop and Play available [p049].
- W65. Multiplier change: immediate with no quantization when stopped; effective on next 1/16 of master clock while playing [p050].
- W66. Pause a track (or selection): hold track selector, press Pause; resume: hold selector, click Play [p050]. Paused tracks advance/re-trigger via Pause [p050].
- W67. Reset paused tracks (stopped): double-click Pause [p050]. Set pause while stopped: zoom into track, click Pause [p050].

### 4.10 Program, bank, maps
- W68. Program change: Track mode; select the MIDI channel as track MCH; dial a program number (top-right rotary or matrix/circle keys); press Program (sent on each press) [p050-p051]. Hold Program while turning to send live [p051].
- W69. Bank change: press Select (turns Orange), dial bank number, press Select again to send [p051].
- W70. View an attribute map: hold attribute selector then ZOM, or double-click the attribute [p051]. Edit: click a bar position in a column; hold a row-0 step to read the real value [p052]. Set values with circle keys; enter pitches with note keys and left/right "C" [p052].
- W71. Shift a map: POS knob in Editor block [p053]. CLR / RND / RMX act on the map [p053].
- W72. CC resolution: in MAP mode with MCC selected, use the chord button block (1 = one message per step, default; 5 = 5 messages) [p053].
- W73. Map factor: hold an attribute selector in track mode and turn the main rotary; also in MAP mode [p054].

### 4.11 Direction map
- W74. Pick DIR 6-16; double-click the DIR attribute button; row 0 pick slice; rows 1-9 toggle triggers (one per row, order = row order); edit certainty_next in circle top-left quadrant with the top-right rotary or keys (double-click 5 = 50; click 100 = 100; double-click 100 = 0); ESC to leave; CLR restores forward in 6-16; RND for inspiration [p057]. No PLAY for directions; changes permanent [p057].

### 4.12 Effector
- W75. Feeder: EFF mutator to Green [p061]. Listener: click EFF to Red; click Red to Orange (listening feeder); click again = off [p061]. Also "double-click EFF" toggles to Red [p061]. UNCLEAR: which toggles are single/double; and text also says "TGL button in MUT column" for the toggle; UNCLEAR whether TGL = EFF here.

### 4.13 Clock and boot
- W76. 200 button: Single-click / Double-click cycles [p011]. v5.30: Orange (Master send) -single-click-> Red (Slave echo) -double-click-> Off (Master no clock) -single-click-> Green (Slave no clock). Legacy: Orange -single-> Off -double-> Green -single-> Red. UNCLEAR: how it returns to Orange; the table can also be read as separate rows.
- W77. Boot: hold Esc while rebooting = version on the matrix; hold Clear while rebooting = clear the grid [p004].

Total workflows recorded: W1-W77 (77 numbered, some are variants/constraints).

## 5. Numbers and limits

- Tracks per page: 10 (0-9) [p039, p043, p060]. Steps per track: 16 [p014, p056]. Step selection stores per page: 5 [p018].
- Chord: up to 7 notes, spanning up to 3 octaves; strum 0-9 up, -9..0 down [p020, p022]. Single-note strum: 6 duplicate notes [p022]. Chord octave layers 1-3 [p020].
- Step LEN: 1/192 min; 192/192 (one whole note) natural max; multiplier 1-8 (table lists 1, 1.5, 2, 3, 4, 5, 6, 7, 8); max 8 whole notes; legato below 1/192; Green step = 1/192, Red = 12/192 (1/16); default 1/16 [p014-p016].
- Step STA: max push 5/192, max pull 5/192 [p016]. Track STA/LEN display 0..16 (8 neutral); internal 1..17 (9 neutral) [p053]. Step VEL and PIT are signed offsets with tens+ones LED display [p015].
- Step defaults: VEL 0, PIT 0, LEN 1/16, STA 0, AMT 0, MCC none [p014]. Step AMT -127 = mask [p011].
- Phrases: 3 banks x 16 = 48 default phrases; phrase notes: 8; phrase types 1-4; phrase POS 1-16, 8 neutral; phrase pool saved with the Grid [p016, p017, p023, p028, p030].
- Track MCH: 1-16 on port 1 (Green), 1-16 on port 2 (Red) [p046]. Track MCC: 0-127, none, BENDER, CHANNEL PRESSURE [p046]. Track GRV shuffle: 0-16 [p046]. Program numbers 1-128 (0 = none) [p051].
- Track DIR: 1-16; 1-5 read-only (forward, reverse, ping-pong, brownian 2/3 fwd 1/3 rev, random); 6-16 user-editable, default forward [p045, p056]. Direction map: 16 slices, up to 9 triggers per slice, certainty_next 0-100% [p056, p057].
- Track multipliers (19): 16, 8, 7, 6, 5, 4, 3, 2, 1.5, 1, 1/1.5, 1/2, 1/3, 1/4, 1/5, 1/6, 1/7, 1/8, 1/16 [p049].
- Events: max range 17 for each map factor, 16 for DIR, 32 for MCH; default 17 (0-16) [p034, p036]. Track toggle Range max 10 [p039]. AMT 10 addresses track 0 (AMT 0 = off) [p039].
- Hyperstep: max 192 ticks; LEN 192/192 when created [p031]. Hyper track Track LEN scaling 8 neutral..1 fastest; 9-12 = 8, 13-16 = 1-4 [p032].
- Effector: modulated attributes VEL, PIT, LEN, MCC [p059]; top-down only (track 9 can modulate all; track 0 none) [p060]; feeder influence up to 12 ticks [p062].
- Ticks: 192 per whole note; 12 per 1/16 [p015, p062]. 
- Middle C: note number 60 = C5 [p004, p043].

## 6. MIDI, sync, ports, clock (in p001-p063)

- 200 button/LED = clock state (Master send / Slave echo / no clock) [p011]. Master/slave clock chapter is p093-094 (out of range).
- MIDI ports: port 1, port 2, per track through MCH [p046].
- Track sends MCC only if MCC != none; MCC 0 is a valid value; BENDER (pitch bend) and CHANNEL PRESSURE flags [p046].
- CC map resolution: 1..n messages per step, interpolated linear slope [p053].
- Program change sent on Program press, while held-and-turned, and once when a page is enabled for play; bank change messages sent on page activation but not from Track-mode program presses [p051]. Program range shown 1-128 [p051].
- Note-off: legato step sends no note off [p016]. Velocity 0 = note not transmitted [p040].
- Slave-clock recording latency can be corrected with STA on a step selection [p018].
- Effector passes MCC only if the feeder track has an MCC set; use different MCH from listeners [p061].
- Tempo multiplier changes take effect on the next 1/16 of master clock during play [p050].
- Keyboard chord recording: see Recording p087+ [p020]. MIDI Merge, Anti-Echo, ALL NOTES OFF, Sysex: out of range (p087-p097).
- On the manual's own alphabetical index, MIDI-related items sit at p093-p094: Master Clock, Slave Clock, Echo, Merge, ALL NOTES OFF, Toggle Page/Set Extn Pgm Ch [p006].

## 7. Engine maths pointers (do not re-derive)

- Step LEN chart / multiplier table: [p016]. Track LEN reference chart [p044]. Track STA reference chart [p045]. Track GRV shuffle delay table [p046]. Attribute scaling reference chart [p055]. Step event range/scaling charts [p035, p036]; AMT worked examples [p037].
- Chord strum ticks table [p022]. Phrase default charts Green [p024], Red [p025], Orange [p026]; custom phrase blank chart [p029]; POS timing chart [p030].
- Hyperstep length chart [p032]. Direction model [p056-p057]; blank custom direction charts [p058].
- Feeder/listener influence timing [p060, p062, p063].
- Track toggle event ordering [p040].

## 8. UNCLEAR register (consolidated)

1. Definition of "Shine" LED state [F4].
2. Count/layout of EDIT encoders; whether AMT and MCC have dedicated knobs [1.5].
3. "Top right rotary" vs "main rotary" [1.6].
4. Whether MIX TARGET MAP 1-4 are the same keys as stores 1-4 [1.7].
5. Play (flashing Orange, Play Mode) vs Snapshot (red) [1.10].
6. Select vs SEL button identity; SEL as octave key [1.11].
7. ESC from Track mode; exit from the phrase editor [2.4, 2.7].
8. Row-to-attribute mapping in zoom views; digit-key mapping for quick entry; quick-key column headers [W11, W12].
9. GRV double-click: phrase-group advance [p017] vs open phrase editor [p027]; probably matrix key vs SEL-column button.
10. Track multiplier LED encoding of divisor 5 [3.4].
11. How a track selection is created; how the chain base indicator is toggled [W59, W62].
12. 200 button cycle return and transition labels [W76].
13. EFF toggle path: single vs double click; TGL vs EFF wording [W75].
14. Chase-light Orange based on track MCC [p046] vs "step" MCC [p017].
15. Step Zoom edit of over-run track: "Page Edit Preview mode" reference [p015] is out of range.
16. Hypersteps: text says "muting steps" in the hyper/hyped tracks [p031, p033] where the rest of the manual uses "skip". Both terms appear for the same effect.

## 9. Ten items that define the instrument's UI (for summary)

1. Whole product is one matrix: 16 x 10 (row = track, column = step) [p013, p039, p060].
2. Semantics of LED colour: Green = on, Red = skipped/selector head/pull, Orange = event/chord/MCC, Flash = selected [p014, p018, p047].
3. Zoom navigation: PAGE > TRACK > STEP by double-click or hold+key; ZOM Red = out [p013, p042].
4. ESC returns to PAGE anytime; ZOM exits [p015, p057].
5. Single vs double click on the same control gives different results everywhere [p017, p038, p049].
6. Hold-a-track-selector + press a step (hypersteps) [p031].
7. Numbers on the LEDs: Red = tens, Green = ones [p015].
8. Steps are offsets of track base values; track LEN/STA are scale factors of the step offsets (neutral 8) [p015, p044, p053].
9. Events attach to steps: Orange = scale factor, Green = value; step LED Orange [p034].
10. Effector top-down modulation and EFF colours Green/Red/Orange [p060-p061].

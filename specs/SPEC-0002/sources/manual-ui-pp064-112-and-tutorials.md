# genoQs Octopus CE v5.30: UI and interaction extract, manual pp.064-124 plus 2007 tutorials

Scope read in full, in order: p064-p112, tutorial-01..10 (PDF pp.115-124). Blank or near-blank: p064, p098, p110 (footer only), tutorial-09 (empty), tutorial-10 (title line only).

**Citation key.** `[pNNN]` = manual page. `[T01]..[T08]` = bundled 2007 "Octopus - tutorial series" (T01 = PDF p115 ... T08 = PDF p122). Anything cited only `[Tnn]` is 2007 tutorial material, older than v5.30, and loses to the v5.30 text on conflict (section 8). The task's "pp.113-124" is PDF numbering: PDF 113-114 = manual p111-112 (Nemo sysex), PDF 115-124 = tutorials.

**Terms.** "shine"/"lit" = steady LED, "flash" = blinking [p067, p112]. "Click" = single press. "Double-click" and "press twice" are different gestures [p068]. Roman numerals (XIV, XX, XXIX-XXXI) are the printed labels of outer-circle buttons [p071, T05].

**Not in this range (do not spec from this file):** Track mode, Step mode, attributes, chords, phrases, hypersteps, Effector, first-run Device View entry (pp.1-63). The range only mentions them in passing.

---

## 1. Physical UI elements named in this range (exact manual names)

Counts are given only where the range states or implies one. Where it is silent, it says UNCLEAR.

### 1.1 Matrix and its edges
- **Matrix**: 16 columns x 10 rows. Page mode: columns = steps 1-16, rows = the 10 tracks [p083, p100, p111]. Rows are numbered top to bottom 9 to 0 [p111].
- **Row 0**: not a track row in Grid/Page-switch contexts. It is a control row: page-set slots (16 buttons) [p084], page repeats/length entry [p083], MIDI Control (drum) steps [p085]. PAGE+matrix jump ignores row 0 [p066].
- **Grid mode matrix**: rows 9..1 = banks 9 (I)..1 (A) top to bottom, columns 1-16 = pages [p103-104]. Bank 1 = row 1 [p080]. "Bank" = 16 pages on one Grid row [p077].
- **Selector column** ("SEL column", "track SEL", "track selectors", "selector of row 2"): one button per row. Doubles as attribute selectors in Page/Track contexts (row 2 = GRV [p073]; row 0 = MCH in Grid [p085]; AMT, MCC, MCH selectors in MAP view [p074]; POS, PIT selectors [p073, p078]; DIR Selector [T03]). Its Grid role: per-row cluster select ("SEL button of that row") [p082]; bank SYSEX export button in EXC mode [p097].
- **MUT column** (Grid): bank mute toggles [p082]. **Track "Mute" buttons** (Page) [p067]; **main MUT button** (mute patterns; Grid-Track global toggle) [p067, p081]; **Solo button** (Track Zoom, or when track selected via Sel) [p067]. Side of the matrix for each column: UNCLEAR (not stated).
- **MIDI Control Map selectors 0-5**, "at the bottom of the matrix field" / "row of buttons under the matrix" [p091, T08]. Tutorial calls the first "MAP 1" [T07] vs manual "MAP 0" [p074] (see section 8).
- **Virtual track selectors** (Grid-Track): matrix buttons in columns 4-13 (10 LEDs), 3 blank columns each side [p080].

### 1.2 Mode / transport / mutator buttons
- **MODE block**: contains at least the green **PLAY** button [p066] and a **STEP** key/LED [p089]. Full membership not enumerated here (UNCLEAR). Other named mode buttons: **PAGE** [p065], **GRID** ("GRID mode button") [p079], **TRACK** ("Track Mode button/led") [p067, p080], **EDIT** ("EDIT LED button", "EDIT master button") [p068, p070].
- **Transport**: **Play**, **Stop**, **Record / REC key**, and a **pause** button ("pressing pause again" resumes) [p067, p086, p090, p094].
- **Program** button/LED [p066, p095]. **ESC** button [p072, p095-096]. **CLR** ("Clear button", "CLR Mutator") [p079, p096]. **ZOM** [T07].
- **Mutator buttons**: TGL, SOL, CLR, RND, RMX, CPY, PST [p065-066]. Grid additionally: Page Clear/Solo/Copy/Paste/Remix by holding a page [p079]. Mix-map extras: ALN, CPY, PST, CLR, RND [p073].
- **ALN / "EXC / ALN" button** (dual: chase-light align; SYSEX EXC mode when GRID held) [p086, p096].
- **Scale controls**: **SCALE SEL**, **SCALE MOD**, **CAD** ("SCALE CAD") [p071]; outer-circle scale buttons ("Maj, Min etc", "Chr." XX), **Select (XIV)** [p071, p086].
- **Follow** key ("Green Follow key") [p084]. **MAP** button (anti-echo) [p087]. **MIX** button ("Mix Master key", "Green lit MIX button on the bottom left") [p073, p097, p112]. **ATR** button in the **MIX TARGET** field [p071-072].
- **200** button/LED: MIDI clock selector, top part of outer circle [p093].
- **Reset button**: on the bottom pane, just underneath the Tempo knob [p096].

### 1.3 Encoders and knobs
- **MIX encoder block / MIX knobs**: 10 knobs, one per track row (one MIX stream per track) [p072-074, T07]. "Bottom left Mix encoder" is the transpose-channel encoder [p078].
- **EDIT knob group / EDIT block / editor knobs** [p070, p080]. Named encoders in it: **DIR rotary** [p069], **STA knob** [p083], **VEL encoder**, **PIT encoder** [p084], **MCH encoder** [p085].
- **Main knob / tempo knob** (tempo; bank browse) [p077, p086].

### 1.4 Circle (outer/inner) and displays
- **Outer circle**: numeric field / quadrants (velocity, tempo, bank view) [p068, p077], **Tempo** LED/button (Orange in Page mode) and **Select** LED (Green) in the top-right quadrant [p077], **200** button [p093], scale buttons, chain selectors XXIX-XXXI [T05], certainty_next "100" position [T05].
- **Inner circle** = pitch/note circle ("scale circle", "note circle", "pitch circle") with **High C** and **Low C** keys (octave up/down) [p071, p084]. It also shows save progress [p096] and page pitch offset [p084].
- **CHORD block** LEDs CHORD1..CHORD7 (overload display) [p102]; chord buttons set step polyphony [p100].
- **MIDI ports**: two, addressed as "port 1/2", "MIDI IN 1/2", "MIDI Out 1", "both MIDI ports" [p078, p088, p093, p096]. See section 6.

---

## 2. Mode and view hierarchy

**Top level (this range):** GRID, PAGE, TRACK (zoom; only touched on here), Device View (boot-time), plus overlays (PLAY, EXC, lock).

```
GRID mode  (matrix = 9 banks x 16 pages; row 0 = controls)            [p079, p103]
 |- Grid EDIT   (EDIT flashing Orange; default)                        [p079]
 |- Grid LIVE   (EDIT Green; page buttons are toggles)                 [p079-080]
 |    '- switching timing: "on the beat" (Tempo LED Red) / "immediate" (Green)  [p080]
 |- Cluster states per row (SEL LED): Off Red / On Green / On+Track-Across-Cluster flashing Orange  [p082]
 |- GRID-TRACK mode (sub-mode of GRID: GRID Orange + TRACK flashing Orange)  [p080]
 |    |- Grid-Track EDIT (default)  |- Grid-Track LIVE (EDIT Green)    [p081]
 |    '- TRACK zoom via double-click on a virtual selector (Edit)      [p080-081]
 |- EXC mode (SYSEX dump; EXC LED flashing Orange)                     [p096]
 |- Grid scale (SCALE SEL in GRID)                                     [p072, p085]
 |- Drum Control View / "Page Set as MIDI Controller"                  [p085]
 |- Interface lock (dark panel, tempo LED only)                        [p086]
 '- Page Sets (row 0 slots)                                            [p084-085]
PAGE mode  (matrix = 16 steps x 10 tracks)
 |- EDIT states: normal (steady Green) -> EDIT PREVIEW (flashing Orange) -> EDIT PERFORM (flashing Green); MCC state (Red); Editor ATR state (knob-group Orange)  [p068-070]
 |- On-The-Measure (TRACK LED Orange)                                  [p067]
 |- PLAY mode (snapshot; PLAY flashing Orange, Program Red)            [p066]
 |- SCALE mode (SCALE SEL Red) / scale locked (SCALE SEL flashing Orange)  [p071-072]
 |- Tempo view <-> Bank view (Tempo vs Select LED)                     [p077]
 |- MIX map views: ATR / VOL PAN MOD EXP / MAP 0-5 targets             [p072-074]
 |- Keyboard-transpose setup (hold main SEL + track selectors)         [p078]
 |- Recording states (section 3.5)                                     [p087-092]
 '- Hold-PAGE overlay: mutators + page jump                            [p065-066]
TRACK mode / Track Zoom (out of range; entry from Grid-Track by double-click; exit by GRID+TRACK together)  [p080-081]
Device View (boot with ESC held)                                       [p099]
```

### Entry/exit table

| View | Enter | Leave | Cite |
|---|---|---|---|
| Grid | press GRID (lock exit: double-click GRID) | (mode switch buttons; not detailed here) | [p086, T08] |
| Grid EDIT/LIVE | EDIT LED: flashing Orange = EDIT; press once to Green = LIVE | press EDIT again | [p079] |
| Grid page-switch timing | press **Tempo** button | press again | [p080] |
| Grid-Track | in GRID, press the Green-lit TRACK button | press GRID **or** TRACK: both return to GRID | [p080-081] |
| Grid-Track LIVE | press EDIT until Green | press EDIT | [p081] |
| Track Zoom from Grid-Track | double-click a virtual selector (Edit mode) | press GRID and TRACK at the same time, returns to GRID-TRACK | [p080-081] |
| Cluster mode (row) | single-click the row SEL (Red -> Green) | click back to Red (single click turns off cluster operation) | [p082] |
| Track-Across-Cluster | double-click the Cluster select button (Flashing Orange) | single click = cluster off; double-click = Cluster On (Green), TAC off | [p082] |
| EXC (SYSEX) | hold GRID + press green flashing EXC/ALN | ESC, returns to GRID | [p096] |
| Interface lock | hold GRID + press ESC | double-click GRID | [p086] |
| PAGE-hold overlay | hold PAGE (mutators, page jump; PAGE press flashes Orange at current page position) | release | [p065-066, p086] |
| PLAY mode | press green PLAY (from PAGE or GRID); PLAY flashes Orange, Program lit Red | Program = keep; PLAY again = discard and recall; stopping the sequencer also discards | [p066] |
| EDIT PREVIEW | click EDIT once (steady Green -> flashing Orange) | press EDIT twice (not double-click) to return to steady Green | [p068] |
| EDIT PERFORM | click EDIT twice from steady Green (Orange, then flashing Green) | click again to steady Green (by p068's "twice" rule from Orange) | [p068-069] |
| Editor MCC state | double-click EDIT (LED Red) | press EDIT once | [p070] |
| Editor ATR state | hold attribute selector + press EDIT master (knob group Orange) | press EDIT master once | [p070] |
| Force to scale (SCALE mode) | press green SCALE SEL (goes Red) | press red SCALE SEL to disable; ESC leaves scale forced but exits SCALE mode (SCALE SEL flashes Orange) | [p071-072] |
| Re-enter scale edit | press the Orange flashing SCALE SEL | | [p072] |
| Scale note composing | press Select (XIV) (flashes Orange) | press SCALE MOD / Select again | [p071, p086] |
| Bank view | press Select (Green LED in top-right quadrant) | press Tempo | [p077] |
| MIX map (ATR target) | double-click ATR in MIX TARGET; or click any MIX target button | (timeout in Page mode: values vanish after last encoder click) | [p072, p074] |
| MIX quick assign | hold attribute selector + press MIX, or hold MIX + press selector | | [p073] |
| Keyboard transpose | hold main SEL + track selector(s) | release selection | [p078] |
| On-The-Measure | Page View, TRACK LED Green: click TRACK button (turns Orange) | click TRACK again (implied; not stated) | [p067] |
| MIDI Control Map mode | disable recording; double-click a Map selector 0-5 | (ESC per tutorial) | [p091, T07] |
| Map learn | in Map mode press REC (Red flashing light in selector column) | press REC again | [p091] |
| Recording (track armed) | see workflows W29-W33 | | [p087-088] |
| Grid program-change listen | in GRID hold GRID + click REC (REC flashes Orange) | press REC again | [p094] |
| Device View | reboot holding ESC | (reboot) | [p099] |
| DIR editor (tutorial) | zoom track, DIR edit knob, double-click DIR Selector | exit out to Page mode | [T03-T04] |

---

## 3. LED colour and flash semantics (by element)

Colours in play: Green, Red, Orange. Steady = shine/lit; blinking = flash. No other colours are named in this range.

### 3.1 Mode LEDs
- **EDIT (Page)**: steady Green = default, steps toggle; flashing Orange = EDIT PREVIEW; flashing Green = EDIT PERFORM; Red = MCC send state [p068-070]. **EDIT knob-group indicator** Orange = ATR assigned to edit knobs [p070].
- **EDIT (Grid)**: flashing Orange = Grid EDIT (default); Green = Grid LIVE [p079]. Grid-Track LIVE = Green [p081]. Also required flashing Orange when saving an individual page [p095]. "EDIT block indicator flashes Orange" when a virtual track is held [p080].
- **PLAY**: Green idle; flashing Orange while PLAY mode active; **Program** LED Red while PLAY mode active [p066].
- **TRACK LED** (Page view): Green = normal; Orange = On-The-Measure engaged [p067]; flashing Orange = TAC + OTM both on [p083]. In Grid: TRACK Green-lit = can enter Grid-Track; flashing Orange = in Grid-Track; **GRID** stays Orange in Grid-Track [p080].
- **Tempo LED**: Orange (default in Page mode) [p077]; in Grid: Red = "on the beat", Green = "immediate" [p080]; flashes at internal clock rate under interface lock, all other LEDs off [p086].
- **Select LED** (top-right quadrant): Green in Page default; in bank view, playing banks = Green buttons in the numeric field, current matrix bank = Orange flashing [p077].
- **STEP LED** (MODE block): Red when a track is grabbed (step tapping available) [p089].

### 3.2 Page/matrix state LEDs (Grid)
- Page button: **Red** = not playing; **Green** = playing [p080]. **Orange flash** = position of the current page (on PAGE press) [p066, p086]. **Flashing Green** = Cluster Hold Page playing [p082].
- Page jump with PAGE held: current page flashes Orange; newly pressed page flashes Orange, previous shows Green (still playing); pressing TGL: at end of the track new page turns Green, previous Red [p066].
- Solo-ed page: Page LED in the circle flashes **Green** (versus Orange when not solo) [p065].
- Cluster select (row SEL): **Red** = cluster off; **Green** = cluster on, TAC off; **flashing Orange** = cluster on + TAC on [p082]. The Red/Green states are labelled "v5.00 & earlier" [p082].
- Page-set slots (row 0): Green = that set's pages are playing; Red = stored, not playing; Orange = empty slot [p085]; "selected set flashing Orange" once all pages toggled off; after SEL-click delete "flashing Green (empty)" [p085] (Orange-vs-Green empty: UNCLEAR, section 8).
- Page length / repeat: Red LED in the first position of row 0 = default full 16-step cycle; row-0 SEL LED toggles Red/Green, Green needed to enter length [p083].
- Follow: **Green** = off ("stick to the Green steps"), **Red** = on ("stick to the Red chase-light") [p084]. Tutorial calls Follow-off "green LED" [T08].
- During SYSEX export a sent page LED changes Green -> Orange [p096].
- Orange chase light in Page mode on tracks whose MCC is not "none" [p090].

### 3.3 Track LEDs
- Track Mute LED: flashes Red = mute toggle armed for next measure; flashes Green = solo toggle armed [p067]. With TAC + OTM: shines Red (armed mute/unmute) or shines Green (armed solo) [p083]. Grid-Track live: Orange = soloed; flashing Orange = armed for solo in OTM page [p081].
- Grid-Track bar: 10 LEDs (columns 4-13) show each track's toggle state; Green with no mutes [p080]. Step 16 of the Grid-Track row Red/Green = stored mute pattern active/inactive [p081].
- Track SEL LED flashing Red = armed for recording; also rehearse mode [p088]; it travels along a chain with the chase-light [p090].
- MCC none = four Green LEDs at positions 13-16 [p074, p090].

### 3.4 Scale, mix, MAP LEDs
- **SCALE SEL**: Green = no force-to-scale; Red = force-to-scale active; flashing Orange = forced but scale mode exited/locked [p071-072, p086]. With a track selection: flashing = pick transpose mode: Red = Relative, Green = Absolute [p078].
- Circle when scale active: scale notes lit except upper C; lower C lit Orange = base tone; base shown Orange [p071, p086]. Select (XIV) flashes Orange in note-composing; Chr. (XX) lit Orange when chromatic C is active [p071]. **CAD** flashes Red when cadence is active [p071].
- **MIX target**: ATR select: SEL buttons Green (selectable), selected flashes Orange [p074]; all Mix Target LEDs lit with selected target flashing Orange [p072]; selected mix attribute flashes Orange in the selector column after quick assign [p073]; POS: flashing Orange -> click POS again -> shines Red = knobs rotate Skip condition only [p073]. Map selector LED Orange = active map [p074]; AMT flashes Orange when a map is selected [p074].
- **MAP** LED flashes Red when MIDI note and MCC playback during recording is disabled (anti-echo) [p087].
- **Map learn**: Red flashing light in the selector column marks the encoder being learned [p091].
- **MCH (row 0, drum control)**: LED flashes Red after double-click; flashes Orange if selected step has a note-out stored, Red if empty [p085]. Selected row-0 step flashes Orange [p085].
- **Mix Master** (import convert): shine_red = armed, shine_green = done [p112].

### 3.5 Recording LEDs
- **REC** flashing Red = recording [p067, p088]; shining Red = On-The-Measure armed (start/stop at next measure) [p067]; flashing Orange = page armed for recording (step-note, external force-to-scale, external scale edit) [p088, p091-092], and also grid program-change listening armed [p094].
- Track SEL flashing Red with REC not flashing = rehearse mode [p088].
- Device View: Record flashing Red = confirm prompt [p099].

### 3.6 MIDI clock ("200") LED, per prose [p093]
| LED | Meaning |
|---|---|
| Off | not sending, not receiving clock (default) |
| Orange | MIDI Clock master: sends clock + transport out both ports |
| Green | slave: listening, no echo (default colour when entering slave) |
| Red | slave with clock echo (passed through to out) |
Single click toggles inside a group (Off<->Orange; Green<->Red). Double-click switches master<->slave. Diagram on p093 disagrees on axes (section 8).

### 3.7 Other LEDs
- **CHORD1** Red = MIDI 1 overload; **CHORD2** Red = MIDI 2 overload; **CHORD3-CHORD5** Orange = CPU about 80% (stage one); **CHORD6-CHORD7** Red = CPU overloaded (stage two) [p102].
- **Program** flashes while GRID held (save armed); "Red flashing Program" when a page is held in Grid EDIT (page save); "Green flashing ESC" when PAGE held (page load); "Green flashing EXC/ALN" when GRID held [p095-096].
- **EXC** Orange, flashing steadily = EXC mode [p096]. Save progress: inner note circle fills; full circle = complete [p096].
- Transpose channel encoder LED: Green = port 1 (values 1-16), Red = port 2 (1-16); turning left = listen off, step 1 LED flashes Red momentarily [p078].
- Device View defaults: Grid Mode LED Red, Scale LED Green (= port 1 priority); Grid Mode click -> Green (start in Grid), Scale click -> Red (port 2 priority) [p099].
- Nemo scale convention: Off = green, On = red [p112].
- DIR editor (tutorial): flashing DIR SEL, flashing LED in row 0 (slice being edited), solid Orange LED in step 0 (map number), red LEDs in rows 1 and 2 = trigger positions, "100" position lit in outer circle = certainty_next 100% [T03, T05].

---

## 4. Workflows (button sequences in order)

W1 **Page mutators** (Page mode): hold PAGE, press mutator: TGL (toggle page play), SOL (solo/un-solo), CLR (reset to defaults incl. forward direction), RND (random step patterns, attributes untouched), RMX, CPY (includes local chain config), PST (paste into present page position) [p065-066].
W2 **Jump to page**: hold PAGE (current page flashes Orange), press any other matrix key in rows 1-9; still holding PAGE, press TGL to switch at the end of the track [p066].
W3 **"Where are you from"**: press PAGE, Orange flash in the matrix marks page position [p066, p086].
W4 **PLAY snapshot**: press PLAY (flashes Orange, Program Red) -> edit (EDIT PERFORM suggested) -> Program keeps, PLAY again discards [p066].
W5 **On-The-Measure**: Page View, click TRACK (Green -> Orange). Mute: press Mute (LED flashes Red, applies at next measure, cancellable). Solo: double-click Mute (flashes Green); un-solo by clicking Mute or main Mut. Also applies to Main MUT patterns [p067].
W6 **OTM record**: arm track, sequencer playing, click Record (LED shines Red, starts at next measure, then flashes Red). To stop: click Record while recording (shines Red until end of measure) [p067].
W7 **EDIT PREVIEW**: click EDIT once (flashing Orange); matrix presses play the step's MIDI without setting it; encoder click re-triggers; velocity in numeric quadrant, pitch in inner circle. Length: hold step + press a button at a distance (1/16 units) or a button in the same column for 192/192. Track selectors play track vel/pitch [p068].
W8 **EDIT PERFORM**: click EDIT twice (flashing Green): as preview with no MIDI out. Step shift: hold a step, turn DIR rotary. Over-run into a lower track only if that track's AMT = 100; the over-run features are only available with no Step or Track selection active [p069].
W9 **Editor ATR**: hold attribute selector + press EDIT master; press EDIT master to release [p070].
W10 **Editor MCC**: double-click EDIT (Red); knobs send MCC on each track's MIDI channel/controller; press EDIT once to leave [p070].
W11 **Page scale**: PAGE mode -> press green SCALE SEL (Red, notes lit, lower C Orange) -> pick outer-circle scale, or press Select (XIV) then toggle note keys in circle -> SCALE MOD to pick a new base -> ESC to lock (SCALE SEL flashes Orange); press flashing Orange SCALE SEL to reopen. Press red SCALE SEL to disable [p071-072, p086].
W12 **Scale cadence**: with force-to-scale, press CAD (flashes Red); optionally double-click ATR in MIX TARGET then PIT to see it [p071]. Warning: changes default note octave [p086].
W13 **Grid scale**: in GRID mode use SCALE SEL as for pages; page scales override it; force a page to chromatic to exempt [p072, p085].
W14 **MIX map view**: double-click ATR (target assign, SEL buttons Green, selected flashing Orange); or click any target button; matrix rows show per-track values; knobs edit. CLR/RND/ALN (highest value to top row)/CPY-PST act on all tracks [p072-073].
W15 **MIX quick assign**: hold attribute selector + press MIX, or hold MIX + press selector. **POS skip-rotate**: with MIX held and POS flashing Orange, click POS again (shine Red) [p073].
W16 **MIDI Control Map edit**: double-click MAP n (0-5) -> AMT flashes Orange (matrix = CC amounts); press MCC (matrix = controller numbers, 4 Green at 13-16 = none); press MCH (channel per knob; independent of track channel). Select map 0-5 (LED Orange = active) from GRID or PAGE. Encoder output = AMT start +/- turn amount. Only "hard reset" restores stored values after live changes [p074].
W17 **Bank view**: press Select (numeric field shows playing banks in Green, current in flashing Orange); press a Green bank to jump to its playing page or turn main knob to step through; press Tempo to exit [p077].
W18 **Keyboard transpose**: set keyboard output channel; hold main SEL and select track(s); turn bottom-left Mix encoder to that channel (Green port 1, Red port 2; left = off); click SCALE SEL to pick Relative (Red) / Absolute (Green); start sequencer, play a key; velocity >88 restores original offset. Edit original: Track Zoom, hold Scale Sel + click PIT [p078].
W19 **Grid Clear**: hold GRID + press Clear (RAM only, FLASH untouched) [p079, p096].
W20 **Grid EDIT to LIVE**: press EDIT once (flashing Orange -> Green). Live: page button Red -> on, Green -> off. Tempo button toggles on-the-beat (Red) vs immediate (Green) [p079-080].
W21 **Grid page operations**: in Grid EDIT hold a page, click Clear/Solo/Copy/Paste/Remix mutator [p079].
W22 **Grid-Track**: in GRID press Green TRACK (TRACK flashes Orange, GRID Orange) -> bars of 10 LEDs (columns 4-13). Hold a virtual selector for TGL/SOL (EDIT indicator flashes Orange, encoders edit track; single selected track shows pitch in inner circle). Quick toggle: hold any button in columns 1-3 or 14-16 while pressing a virtual selector. Zoom: double-click selector. Return: GRID+TRACK together. Leave: press GRID or TRACK [p080-081].
W23 **Grid-Track live**: EDIT Green; buttons toggle tracks; double-click = solo, single click = un-solo; step 16 toggles the page's stored mute pattern; Main MUT toggles mute patterns of all concurrent pages (instant or OTM per page) [p081].
W24 **Build a cluster**: use CPY/PST/CLR to make 2+ adjacent pages; sequencer playing; toggle one page Green; click the row SEL (Red -> Green) [p082].
W25 **Hold page**: set page STA = 0 (hold shows flashing Green); cluster will not advance until STA is changed [p082-083].
W26 **Bank mute**: in GRID press a MUT-column button; pages keep playing, output not sent [p082].
W27 **Track-Across-Cluster**: double-click cluster select (flashing Orange); single click = cluster off; double-click = back to Cluster On. Mute/solo buttons and main MUT then apply to all pages of the cluster [p082]. With OTM the Track Mute LEDs shine Red/Green [p083].
W28 **Page repeats (STA)**: hold page in Grid, press 1-16 in row 0 (double-click for 0), or turn STA knob; press page to see repeats and remaining count [p083].
W29 **Page length (LEN)**: GRID, hold page; row-0 SEL LED must be Green; use row 0 buttons (click/double-click) to set length, up to 8 cycles = 128 steps at x1 [p083].
W30 **Page PIT / VEL**: select page in Grid; pitch circle lights, edit or use PIT encoder; High C / Low C = octave; turn VEL encoder, row 1 shows the factor [p084].
W31 **Follow**: press Follow (Green -> Red) to track the playing page; again to freeze [p084].
W32 **Page Sets**: with pages playing, hold Green SEL and press a row-0 slot (slot LED Green); press an empty slot (all stop, slot Orange, old slot Red); press Red slot to recall. Delete: toggle all pages off (slot flashes Orange), hold SEL and click the slot (flashes Green, empty) [p084-085].
W33 **Page Set as MIDI Controller**: in Grid, double-click row-0 MCH (flashes Red); hold MCH + MCH encoder for note-out channel (global); select a row-0 step (flashes Orange), hold MCH + PIT encoder for note value in the note circle [p085].
W34 **Chase-light align**: press ALN. Also realigns on Stop then Play (not on pause/resume) [p086].
W35 **Interface lock**: hold GRID + ESC; unlock: double-click GRID [p086].
W36 **Start with factory defaults**: hold CLR while powering up (default track PITs 9..0: C3 D3 E3 G3 A3 C5 D5 E5 G5 A5) [p086].
W37 **Arm a track for note-stream recording**: Page mode, press REC (Recording mode), hold a Track SEL and press REC again; press Play. With one track armed, all MIDI in is re-channelled to that track [p087]. Selecting other tracks moves the armed state; works in Stop for pre-listen [p087].
W38 **Rehearse**: with all record activity off, hold a Track SEL and click REC (SEL flashes Red); press REC again to record [p088]. Disable: press flashing REC (back to rehearse), REC again (normal) [p088].
W39 **Multitrack record**: multi-select tracks and press REC; source must send on the tracks' channels and to the tracks' IN port (1 or 2). Cancel: press a track selector together with Stop (clears record selection on all pages) [p088].
W40 **Anti-echo**: press MAP in Page/Grid mode (MAP flashes Red); global, saved with machine state [p087].
W41 **Record chords**: stack notes; step takes length, velocity and start of the last note [p088].
W42 **Step-note recording**: no track selected, hold PAGE + press REC (REC flashes Orange, page armed); hold a step, play a key (default length 1/16, follows track flow). On an off step (press on, off, keep holding) = fresh; on an on step = stacked chord [p088-089].
W43 **Track live transposition**: page armed + track selected (SEL can lock it); hold the track selector, note pitch shows in the circle [p089].
W44 **Step tapping**: grab a track (STEP LED Red), sequencer playing, tap STEP; placement resolution 1/192; clear the track to retry [p089].
W45 **Quantized recording**: non-destructive via track STA map factor; lowest value = map not in effect, raise to pull steps [p089].
W46 **Chained recording**: enable one track of a chain for takes over 16 steps; cluster pages, chain them and enable recording tracks for takes over 128 steps; the red lock follows the chase-light [p090].
W47 **Overdub / re-take**: with sequencer running, click Play = delete track under chase-light; double-click Play = delete whole chain; or use Clear [p090].
W48 **CC / pitch bend / channel pressure recording**: same arming; auto-sensing of controller; a track plays only the last sensed controller [p090-091].
W49 **MIDI Control Map learn**: disable recording, double-click Map selector 0-5, press REC (Red flashing light in selector column), press selectors to choose target knob, move controller (amount, channel and controller number recorded), REC to exit [p091].
W50 **External force-to-scale**: page armed (hold PAGE + REC); MIDI in forced to the page scale. Chromatic scale = MIDI merge (notes only). Recorded pitch is the raw incoming pitch [p091, p094].
W51 **External scale edit**: page armed, press SCALE SEL, MIDI notes on MIDI IN act as circle keys; REC again or SCALE SEL again to disable [p092].
W52 **MIDI clock**: press 200: Off -> Orange (master). Press again: Off. Double-click: slave Green; click: Green <-> Red (echo). Change only in GRID or PAGE mode [p093].
W53 **External program change**: only in Slave Clock mode; in GRID hold GRID + click REC (REC flashes Orange); REC again to disarm [p094].
W54 **Save machine state**: sequencer stopped, GRID mode, hold GRID (Program flashes), press Program; 5-10 s; progress in inner circle; complete circle = done [p095-096].
W55 **Save one page**: stopped, GRID, EDIT flashing Orange; hold page's matrix button; click the Red flashing Program [p095].
W56 **Load one page from flash**: PAGE mode of that page; hold PAGE, press green flashing ESC (allowed while playing) [p095].
W57 **Revert / clear**: press hardware reset button (reverts to last save); hold CLR at power-up (skip FLASH load); GRID held + CLR (RAM clear) [p096].
W58 **SYSEX export**: stopped, connect MIDI Out 1; hold GRID + press green flashing EXC/ALN (EXC Orange flashing); press a lit page (page, about 10 KB), a SEL-column bank button (bank, about 160 KB), the green MIX button (Grid-only data, about 37 KB), SEL key (all pages), EDIT key (full state); ESC to leave [p096-097].
W59 **SYSEX import**: play the dump back to the Octopus at any time, even while running; page goes to its original slot, overwrites, machine switches to GRID [p096-097].
W60 **Device View**: reboot holding ESC; click Grid Mode (Green) then flashing Red Record to set start in Grid mode; click Scale to toggle MIDI Port Priority, then Record to confirm [p099].
W61 **Nemo import convert**: Grid Edit, hold matrix page key, click Mix Master once (shine_red), click again (shine_green when done) [p112].

**Tutorial workflows (2007)**
T-a **LFO-type CC modulation**: Page mode, track sends CC on chosen channel/CC#, zoom to the track's CC map to draw the shape, set speed relation, put note tracks on the same channel [T01-T02].
T-b **Direction editor**: empty page, zoom track 9, DIR edit knob to 6, double-click DIR Selector; row 0 selects slice, row 1/2 = trigger positions; empty slice = random; clear/exit to Page mode [T03-T05].
T-c **certainty_next**: Track mode, direction map 7, row-0 slice, double-click "100" in outer circle to set 0, single-click to reset to 100 [T05-T06].
T-d **Real-time CC**: Track mode set MCC; MCC selector + ZOM opens MCC map [T07]. Page mode: double-click MAP 1, AMT/MCC/MCH SEL buttons light, press MCH then MCC, ESC out, twist MIX knobs [T07].
T-e **CC maps across a cluster**: in Grid turn on clustering (row SEL Green), double-click next page button to activate, Follow off (green), MIX CC knobs keep working on the displayed page while another page plays [T08].

**Count:** 61 numbered manual workflows (W1-W61) plus 5 tutorial workflows (T-a..T-e) = 66.

---

## 5. Numbers and limits

- **Grid**: 9 banks (numbered 1-9, lettered A-I; 9 (I) on top) x 16 pages [p103-104]. 144 pages is derived, not stated. Row 0 = control row [p083, p085].
- **Concurrent playing**: one page per bank; "up to 9" concurrent pages [p077, T02].
- **Tracks/steps**: 10 tracks per page (rows 9..0), 16 steps [p083, p111]; Nemo has 4 (x2 = 8) [p111].
- **Page repeats (STA)**: 0-16, default 1; 0 = Cluster Hold [p083]. **Page LEN**: default 16; up to 8 cycles = 128 steps at x1; below 16 retriggers on every start, 16+ does not [p083]. **Measure** = 16 steps at x1 or Page LEN if less [p067].
- **Page sets**: 16 slots in row 0 [p084, p107]. PC channel 10 selects set = (PC mod 16)+1 [p094].
- **MIDI Control Maps**: 6 (0-5), global, 10 knobs each; per-knob channel, controller, amount [p074-076]. Tutorial: 5 maps per page, "50 CCs" [T07-T08] (conflict).
- **Preset MIX CCs**: VOL 7, PAN 10, MOD 1, EXP 11 [p074].
- **Resolution**: step tap placement 1/192 [p089]; step LEN 192/192 [p068]; pitch bend 14-bit recorded, editing MSB 7 bits only [p091].
- **MIDI**: 2 ports x 16 = 32 channels [p078, p094]. Velocity threshold for transpose restore: >88 [p078]. Data intervals: note/velocity range constrained versus 0..127; exact constraint not given in range (UNCLEAR) [p090].
- **Memory**: ONE machine state in FLASH, replaced on save; save takes 5-10 s; sequencer must be stopped [p095]. Page SYSEX about 10 KB; bank about 160 KB; Grid-only about 37 KB [p096-097]. Hardware reset reverts to last save [p096].
- **Overload**: CPU stage one about 80% [p102].
- **Defaults**: track PIT 9..0 = C3 D3 E3 G3 A3 C5 D5 E5 G5 A5 [p086]. Page length default 16, STA default 1 [p083].
- **Overrun**: only if track AMT = 100; flows to lower tracks [p069].
- **Chain ranges**: takes over 16 steps via track chain; over 128 via page cluster [p090].
- **Tutorial only**: default track chain of 10 separate tracks = XXIX; XXX, XXXI other chains [T05].
- **Unstated in this range**: BPM range, undo depth, USB, DIN, total RAM.

---

## 6. MIDI: complete I/O capability list (this range)

### 6.1 Ports and physical
- Two MIDI ports; each track is set to send on port 1 or 2 ("IN 1 or 2") [p088]. Channels are quoted 1-16 per port; 32 channels total [p078, p094].
- Named connectors: MIDI IN (1, 2), MIDI Out 1 (SYSEX) and, by implication, Out 2 ("both MIDI ports" for clock) [p088, p093, p096].
- **DIN vs USB: not mentioned anywhere in p064-p124.** The word "USB" and "DIN" never occur. Treat as a spec gap; do not assume USB MIDI from this range. UNCLEAR.

### 6.2 Data types played out
- Notes, MIDI CC ("MCC" per track), pitch bend, channel pressure (recorded and replayed as controller-like data) [p087, p090-091].
- MIX/EDIT knobs send CC via MIDI Control Maps (per-knob channel + controller + AMT start) [p074] or preset CCs VOL/PAN/MOD/EXP [p074]; Editor MCC state sends MCC on the track's channel/controller [p070].
- Page Sets can send MIDI notes on-the-measure to a drum machine, one global MCH channel, note value per row-0 step [p085]. Output port for this: UNCLEAR.
- ALL NOTES OFF (CC 123) on each of the 32 channels [p094].

### 6.3 Data types received
- Live record: notes (polyphonic, multitrack), controllers, pitch bend, channel pressure [p087]. Input is re-channelled to the armed track; in multitrack the source must match track channels and ports [p087-088]. Anti-echo (MAP) stops note/MCC playback while recording [p087].
- Keyboard transpose: per-track input channel (1-16 on port 1 Green / port 2 Red), Relative or Absolute; velocity >88 restores [p078]. Stored with the page [p078].
- External force-to-scale, MIDI merge (chromatic scale; notes only) [p091, p094]; external scale edit [p092]; MIDI Control Map learn (channel + controller + amount) [p091].
- External program change (Slave Clock mode only; port ignored; channel 10 = Page Set select, channels 1-9 = toggle page in bank n) [p094].
- SYSEX import at any time [p097].

### 6.4 Clock and transport
- Default: Octopus neither sends nor reacts to clock [p093]. Master: clock and "associated transport commands" out both ports [p093]. Slave: clock in on one port, auto-detected; state saved with the machine state [p093].
- Echo (slave only): clock passed to the out port for daisy-chaining [p093].
- Master/slave switching only in GRID and PAGE mode [p093].
- Pressing Stop while not running and defined master or slave (200 Orange or Green) sends ALL NOTES OFF on all 32 channels [p094].
- Port priority (Device View): Port 1 or Port 2 gets timing priority for event queue; useful with a DAW sending clock to Port 2 that then forwards clock and transport to "timing sensitive" gear; does not change SYSEX port (Out 1) [p099].
- Not documented in range: MIDI start/stop/continue message handling in slave mode, song position pointer, tempo range (UNCLEAR).

### 6.5 SYSEX and dumps
- All page, bank, Grid, all-pages and full-state dumps are MIDI SYSEX out of Out 1 only; export only when stopped, import any time [p096-097, p099].
- Sizes: page about 10 KB, bank about 160 KB, Grid-only about 37 KB [p096-097]. Transmission speed follows the tempo at send time [p097].
- Byte format, manufacturer ID, checksum: not in range (UNCLEAR). Standard MIDI File import/export: not in range; the manual names only SYSEX for "export to MIDI" [p096]. The Grid Usage Chart mentions "Sysex Filename" only [p103].
- Octopus <-> Nemo pages: same motherboard; row mapping Octopus 9-6 -> Nemo 1-4, 5-2 -> Nemo 5-8; Octopus tracks 1 and 0 play on Nemo but cannot be reached; "Grid Only" data replaces Nemo step phrases; Import Convert clears or remaps [p111-112].

### 6.6 Overload indication (MIDI pipes)
- CHORD1/CHORD2 Red = port 1/2 pipe full, data possibly dropped; balance load across both ports [p102].

---

## 7. Behavioural pointers (engine maths: page only)
- Mix-map maths/attribute scaling: p072-074 (map factors: pp.53-55 out of range). Page LEN/STA repeat maths: p083. Cluster advance rules: p082-083. Step-shift/overrun: p069. Force-to-scale mapping and CAD: p071, p086. Record quantization via STA factor: p089. Program-change modulo: p094. Random step rest / phrase / hyperstep tricks: p100-101.
- Direction editor, certainty_next, Brownian motion: T03-T06 (engine model already described in INDEX note).

---

## 8. UNCLEAR items and conflicts in the source

1. **INDEX.md page map is wrong for this range.** It lists midi 109-112, load-save 113-118, appendix 113-124. Actual: MIDI chapter p093-094, Load/Save p095-097, Appendix "12." p099-112 (Device View p099, techniques p100-101, load handling p102, worksheets p103-109, Nemo p111-112), tutorials in PDF 115-124. `just manual midi` would return Nemo pages.
2. **"200" diagram vs prose [p093].** Prose: single click toggles Off<->Orange and Green<->Red, double-click switches master<->slave. The diagram puts "Single-click" horizontally between master and slave columns and "Double-click" vertically. Prose is used here.
3. **MIDI maps: 6 global maps (0-5) [p074] vs tutorial "5 maps per page, saved per page" and "MAP 1" [T07-T08].** Tutorial is likely pre-CE. p074 also says maps are global and only a hard reset restores them.
4. **Empty page-set LED**: "Orange (it is empty)" [p085] vs "flash Green (empty)" after delete [p085].
5. **How page mode is entered from Grid** and how Grid EDIT selects (versus toggles) a page is not stated in range. Tutorial: double-click a page button activates or selects it [T08]. UNCLEAR whether double-click toggles play in Grid EDIT.
6. **Which SEL**: "Green SEL button" (page set store/delete, dump all pages) [p084-085, p097] vs "SEL of that row" (cluster) [p082] vs "main SEL button" (transpose) [p078] vs "SEL LED in row 0" (length) [p083]. Identity of each UNCLEAR.
7. **Row-0 use**: Page STA (repeats) and LEN share row 0; the row-0 SEL LED colour (Red default vs Green required for length) seems to pick which one, but the manual never states what Red selects [p083].
8. **REC arming sequence**: p087 "press REC to activate Recording mode, then hold Track SEL and press REC again" vs p088 "with all record activity off, hold track SEL and click REC = rehearse; REC again = record". The extra first REC press is UNCLEAR.
9. **Select (XIV) vs SCALE MOD**: p071 uses "Select button (XIV)" then "press SCALE MOD again"; p086 treats them as two buttons. UNCLEAR whether one control.
10. **MAP button** is anti-echo toggle [p087] but p090 also says "enter MAP mode" and p074 uses "MAP 0-5" selectors. UNCLEAR whether one MAP button plus six selectors.
11. **ALL NOTES OFF** condition names 200 LED "Orange or Green" only [p094]; Red (slave + echo) not mentioned.
12. **External program change** "only in Slave Clock mode" [p094]; whether it works with echo (Red) UNCLEAR.
13. **Page Set as MIDI Controller**: "Scenes" is used once ("MCH is global for all Scenes") [p085]; no other definition (probably Page Sets). Output port UNCLEAR.
14. **Nemo conversion**: "Tracks 9 & 10 are cleared" [p112] vs "tracks 1 & 0" [p111]. Numbering mismatch UNCLEAR.
15. **"Sound Control"** appears once as a name for CC maps [p111]; not defined.
16. **Appendix numbering** is inconsistent: TOC lists VII items but headings say III/IV/V/VI [p099, p103-109]. Chapter numbers jump 8 -> 12 [p095, p099].
17. **EDIT cycle**: p068 says "twice" returns to steady Green; p069 says one more click after flashing Orange gives flashing Green. Together this is a 3-state cycle; confirmed only by combining both pages.
18. **OTM leave**: exit gesture for On-The-Measure not given (assumed a second TRACK click) [p067].
19. **Grid-Track double-click**: Edit = enter Track Zoom, Live = solo [p081]. Consistent but easy to mis-implement.
20. **Mix Master vs MIX button** [p097, p112]: same control? UNCLEAR.

---

## 9. Tenets that define the interface (for spec authors)

1. One LED, many meanings, three colours only: Red, Green, Orange, with steady vs flashing as a second axis [p067, p080, p093].
2. Chords, not menus: hold a mode button (PAGE, GRID, SEL, MIX, MCH) and press a target; click count (single/double/"twice") selects state [p065-066, p082, p068].
3. Grid > Page > Track/Step hierarchy: 9 banks x 16 pages; 10 tracks x 16 steps; row 0 carries controls [p103, p111, p083].
4. Same 16 x 10 matrix, different meaning per mode: steps (Page), pages (Grid), virtual tracks (Grid-Track) [p080-081].
5. EDIT is a cycle: steady Green normal, flashing Orange preview, flashing Green perform, Red MCC [p068-070].
6. Quantised-to-measure actions are armed with flashing LEDs and can be cancelled before they fire [p067].
7. Scale is a visible object: circle shows notes, Orange base, Red SCALE SEL forcing, ESC locks [p071-072].
8. Mix maps put a whole attribute on the matrix while the encoder block edits it, with a timeout revert [p072-074].
9. Recording state lives on the selector LEDs (Red flash = armed) and REC colour (Red flash record, Orange flash page-armed) [p087-089, p094].
10. MIDI is minimal and global: one "200" button for clock, SYSEX only from Out 1, stop to export, receive any time, overload shown on the CHORD block [p093, p096-097, p102].

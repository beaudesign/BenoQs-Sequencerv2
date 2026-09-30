# SPEC-0002 findings

The audit behind the README's F1 to F6, the limits of the sources, and the questions the manual
leaves open. Audited commit `fcf1aec`. Every count comes from a command run on that tree; where
a claim is a search and not a reading, it says so.

## 1. Sources and their limits

| Source | How it was read | Limit |
|---|---|---|
| CE v5.30 reference manual | The repo's own page-split copy, `reference/manual/pages/pNNN.txt` (122 files). Two subagent sessions digested pages 1 to 63 and 64 to 112 plus the bundled 2007 tutorials into `sources/` | The digests are model-made. Three claims were checked against the pages by hand (`p093`, `p094`, `p102`) and matched. The rest is unchecked. Each fact carries a page number so a person can check it |
| v5.30 release notes (PDF) | Fetched with a summarising tool | A small model summarised the PDF, so the text is second-hand. Cites to `[RN]` say what the release notes are reported to say. Read the PDF before any decision rests on one |
| Blog post, "genoQ Octopus basic navigation" | Fetched with a summarising tool | Same limit. The blog is one player's walkthrough, not a specification |
| Web MIDI, browsers, Ableton, Max | Fetched with a summarising tool; see `tech.md` section 10 | The MDN page gave little. A third-party article said Firefox has no Web MIDI; Mozilla's release note for Firefox 108 says it has, behind an add-on. Mozilla's note is used |
| Repo | `git ls-files`, `rg`, and reading files | Searches for a concept are not audits (section 3) |

The 2007 tutorials are older than v5.30. Where they disagree with the v5.30 pages, the v5.30
page wins, and the disagreement is listed in section 5.

Not checked: whether the reference photography and Xcode are still absent. The linked computer
was not connected, so `STATE.md`'s statement that Xcode is not installed is taken as written.

## 2. F1 to F3: the engine, the panel and the browser

**F1. The engine has no front panel.**

- `Command::ButtonDown`, `ButtonUp` and `EncoderTurn` are defined in
  `crates/octocore/src/types.rs` (lines 108 to 110) and, in `Engine::handle_command`
  (`engine.rs` line 551), match an empty arm together with `LoadState`. The doc comment above
  it (line 516) says so and gives the reason: no `panel.truth.json`, so `ControlId` maps to
  nothing.
- `Snapshot.leds` exists (`types.rs` line 194, `snapshot.rs` line 31). In `crates/octocore/src`
  only the snapshot helpers in `snapshot.rs` touch it; `engine.rs` has no reference to it, so
  the engine never lights an LED.
- The LED type is a float RGB colour with a target (`Led { color: LedColor { r, g, b }, target }`).
  The manual's LED is one of three colours, steady or flashing. The panel controller in
  `tech.md` section 4 defines its own frame type, `{Off, Red, Green, Orange} x {Steady, Flash}`,
  and does not use `Snapshot.leds`.
- Steps and tracks carry state that the panel would edit (`Mode`, `record_armed`, `hyperstep`,
  `cluster_mode`), so the data model is ahead of the interface. The comment on `Step.hyperstep`
  (`domain.rs` line 296) says the gesture that makes a link "is still out of scope".
- The two digests hold **138 numbered workflows** (77 in pages 1 to 63, 61 in pages 64 to 112 and
  the tutorials). None is implemented as a button sequence. The commands that change a pattern
  today are `SetTrack`, `SetStep`, `SetActivePage` and `SetMode` (`engine.rs` lines 537 to 550),
  which set values directly.

**F2. The photography blocker does not apply to a functional web UI.** `panel.truth.json`
exists to give the native renderer millimetre positions and materials. The commands in F1 need
only a control identity, and the manual names every control and gives its count (for example
"16 by 10 matrix", 10 MIX encoders, 7 chord LEDs). A logical inventory,
`contracts/controls.json` (`tech.md` section 4), is enough. Where the manual does not give a
count or a position (section 6 items 3, 4 and 6), the layout has a `pending` marker and a
hardware question, not a guess.

**F3. The engine already compiles to WebAssembly.** `just wasm-smoke` builds `octoffi` for
`wasm32-unknown-unknown` and plays the five golden patterns in Node, byte-identical to native
(CI job `wasm-smoke`, passing on every branch tip of the SPEC-0001 series). Two of the five
skip today, because the C interface cannot yet program phrases or MCC (`journal/conductor/
2026-09-29.md`); the web build would use the Rust interface and not the C one.

## 3. F4: coverage against the manual and the release notes

**A code search, not an audit.** A search of `crates/octocore/src` for each term below found
no match. A missing match means "look here first", not "proven absent".

| Feature (source) | Search terms | Result |
|---|---|---|
| MIDI clock out, in, slave, echo [p093] | `clock` near `midi` | None. The engine keeps its own tempo |
| MIDI input and recording [p087 to p092] | `record`, `input` | One flag: `Track.record_armed`, settable |
| Program change, page sets [p085, p094] | `program_change`, `page_set` | None |
| MIDI control maps, learn [p074] | `map` | Attribute maps for VEL and PIT exist (conformance); controller maps do not |
| Grid-Track, virtual tracks [p080, RN] | `grid_track` | None |
| Clusters and Track-Across-Cluster [p082, RN] | `cluster` | One field, `cluster_mode: bool` (`domain.rs` line 537), no behaviour |
| Anti-echo, keyboard transpose [p087, p078, RN] | `anti_echo`, `transpose` | None |
| On-The-Measure [p067, RN] | `ToggleKind`, `on_measure` | Track toggles apply through `ToggleKind` (`engine.rs` line 1371); the arm-and-cancel flow is not modelled |
| SysEx dump and load [p096, p097] | `sysex` | None. The byte layout is not in the manual either |
| Save and load | `LoadState` | An accepted command that does nothing |

Present and conformance-tested, from `STATE.md` and `tests/conformance/`: steps and tracks with
their attributes, chords and strum, phrases (48 factory), hypersteps, custom directions, the
effector, track rotate and skip rotate, the LEN and STA lookup tables (p044, p045), mute and
solo gating, Stop flushing notes (ALL NOTES OFF), and the two-port, 32-channel event model.

## 4. F5: the manual index is wrong for the back third

`reference/manual/INDEX.md` lines 35 to 37 give MIDI as pages 109 to 112, load-save as 113 to
118 and the appendix as 113 to 124. From the pages themselves (digest, section 8, item 1):

| Chapter | INDEX.md | Pages |
|---|---|---|
| MIDI | 109 to 112 | **93 and 94** |
| Load and save | 113 to 118 | **95 to 97** |
| Appendix "12." | 113 to 124 | **99 to 112** (Device View 99, techniques 100 and 101, load handling 102, worksheets 103 to 109, Nemo 111 and 112) |
| Bundled tutorials | inside the appendix | PDF pages 115 to 124 |

`just manual midi` therefore returns the Nemo conversion pages, not the MIDI chapter. This is
the Scribe's zone. A request is in `journal/conductor/requests/2026-09-30-fix-manual-index.md`
and this spec does not edit the index.

## 5. F6: two meanings of "world model"

| Where | Meaning | Action |
|---|---|---|
| `journal/STATE.md`, `agents/CLAUDE.md`, the `justfile` (`just report`), `harness/README.md`, `contracts/verification.report.schema.json` | The factory's name for its shared project state | Stays |
| The owner's message; `SPEC.md` and `docs/06` in effect | A Genie-style generated environment: rooms, light probes, acoustics | Removed (`removal-plan.md`) |
| `crates/octocore/loom/`, `crates/octocore/tests/loom_sync.rs` | The `loom` concurrency model checker | Stays. Unrelated to the Loom role in `agents/ROLES.md`, which is removed |

The scope gate's banned list is written to not match the first or third row.

## 6. Conflicts and open questions in the manual

The two digests list 36 items between them: 16 in pages 1 to 63 and 20 in pages 64 to 112 and
the tutorials. The ones below decide a design or a fixture. The rest are in `sources/`.

Each fixture for an open item is written as `pending`: it records the ambiguity and what is
assumed, it does not assert, and it is counted separately by the ratchet. That is how
`tests/conformance/pending/` already works.

| # | Item | Source | What the spec assumes | Who can settle it |
|---|---|---|---|---|
| 1 | The "200" button cycle. Prose: single click toggles Off and Orange, and Green and Red; double-click switches master and slave. The diagram puts single-click between the columns and double-click between the rows. p011 gives another cycle (Orange, Red, Off, Green for v5.30, with a legacy cycle) | p093, p011 | None. Fixture `pending`; the mock starts from the p093 prose | Hardware |
| 2 | Control maps: six global maps (0 to 5), or five per page, saved per page | p074 against T07, T08 | Six, global (v5.30) | Owner, hardware |
| 3 | The "shine" LED state | p014, p021, p033 | None. Fixture `pending`; the mock shows a plain flash | Hardware |
| 4 | Count and layout of EDIT encoders; whether AMT and MCC have knobs | p015, p017 | Ten knobs: the eight named (VEL, PIT, LEN, STA, GRV, POS, DIR, MCH) plus AMT and MCC (p017 says "their respective edit knob") | Hardware |
| 5 | "Main" and "top right" rotary: one knob or two | p022, p027, p034, p050, p054, p057 | None. Fixture `pending` | Hardware |
| 6 | Row order of attributes in Step zoom and Track zoom | p015 | None. Fixture `pending` | Hardware |
| 7 | ESC from Track zoom and On-The-Measure; exit from the phrase editor and the chord views | p042, p067, p027 | ESC leaves any zoom; OTM leaves on a second TRACK click | Hardware |
| 8 | The REC arming sequence: one REC press or two | p087 against p088 | p088 (rehearse, then record) | Hardware |
| 9 | SEL: which of at least four buttons, and whether Select and SEL are one | p018, p020, p050, p078, p082, p083, p084 to 085 | None. Each context's SEL is named as the manual words it; fixture `pending` | Hardware |
| 10 | Select (XIV) against SCALE MOD | p071, p086 | None. Fixture `pending` (p086 treats them as two) | Hardware |
| 11 | MAP: an anti-echo toggle and six selectors, or one button | p074, p087, p090 | None. Fixture `pending` | Hardware |
| 12 | EDIT cycle | p068 against p069 | Three states, as the digest reads them together | Consistent; confirm |
| 13 | Empty page-set LED: Orange, or flashing Green after delete | p085 | Orange when empty; flashing Green as the delete confirmation | Hardware |
| 14 | ALL NOTES OFF and program change with the "200" LED at Red (slave and echo) | p094 | ALL NOTES OFF as written (Orange or Green). Program change: Slave Clock mode; Red is `pending` | Hardware |
| 15 | MIDI start, stop, continue and song position pointer in slave mode; tempo range | p093 to p094 | None. Fixture `pending`; the manual is silent | Hardware |
| 16 | Row 0 in Grid: page STA and LEN share it; the SEL LED colour appears to choose | p083 | None. Fixture `pending` | Hardware |
| 17 | Where the chain selection and the base indicator come from | p047, p048 | None. Fixture `pending` | Hardware |
| 18 | Nemo conversion: "tracks 9 and 10" or "1 and 0" | p111, p112 | Out of scope (`product.md` section 8) | Not needed |
| 19 | SysEx and Standard MIDI File export: bytes not given | p096, p097 | Out of scope for v1 | Not needed |
| 20 | USB and DIN never appear in the manual | p064 to p124 | Not applicable to a web app | Not needed |

## 7. Not audited

- The release notes and the blog were not read as documents. Their feature lists are in
  `product.md` with `[RN]` and `[B]` marks and should be re-read before a wave starts.
- No page image was looked at (`CLAUDE.md` rule 7). The digests come from the text layer.
- No browser, Ableton or Max run was made. Every timing claim is a spike (`tech.md` section 8).
- The 26-file, 131-line scope count in `removal-plan.md` is a preview with the banned list and
  the allow-list; the real gate may find more or fewer if the list changes (D9).

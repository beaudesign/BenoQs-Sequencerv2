# SPEC-0002 product spec: a web sequencer that behaves like the Octopus

Citations are `[pNNN]` for a page of the CE v5.30 reference manual in `reference/manual/pages/`,
`[RN]` for the v5.30 release notes, `[B]` for the blog post, and `[T0n]` for the bundled 2007
tutorials (older than v5.30; the v5.30 text wins on a conflict). The digests they come from are
in `sources/`. Everything here is a draft until the owner approves it (README).

## 1. The product in one paragraph

A web app, served as static files, that runs the Octopus sequencing engine and its front panel in
the browser. You program and play patterns with the same modes, gestures and LED language as the
hardware; the app sends real MIDI to external instruments through the browser's Web MIDI, receives
MIDI to record and to follow a clock, and works with Ableton Live. There is no room, no
generated world and no audio engine in v1.

## 2. Who it is for and the jobs

| Job | What "done" looks like |
|---|---|
| **Play the Octopus without the hardware** | Someone who knows the hardware finds every control where and how they expect, and the manual's workflows work as written. |
| **Drive external instruments** | A pattern plays a class-compliant USB synth or a drum machine over a MIDI interface, on two ports and up to 32 channels, with steady timing. |
| **Sequence inside Ableton Live** | The pattern plays a Live instrument, stays in time with Live's transport, and Live's clock can drive it. |

## 3. The tenets, taken from the sources

The interface is defined by these. Each is a design rule and a test target.

1. **One LED, many meanings, three colours.** Red, green and orange, steady or flashing. The
   only colour in the interface is emitted by an LED. Alternate colourways (red is blue, green
   is yellow, orange is purple) exist, so the colours are roles, not fixed hues [p004, p067,
   p080, p093].
2. **Zoom is navigation.** Grid, then Page, then Track, then Step. You burrow down and come back
   with ZOM and ESC. The blog calls it a tree, with the spiral of controls beside it [B, p013,
   p042, p053, p079].
3. **One matrix, different meaning per mode**: 16 by 10 buttons are steps in Page mode, pages
   in Grid mode (9 banks of 16), and virtual tracks in Grid-Track. Row 0 is a control row
   [p080, p083, p103].
4. **Chords, not menus.** Hold a mode button and press a target; a single click and a double
   click on the same control do different things [p017, p038, p065, p068, p082].
5. **Values are read from LEDs.** Tens in red, ones in green; multipliers red over divisors
   green; the pitch circle shows octave in red and note in green [p015, p049, p052].
6. **Offsets, not absolutes.** Steps are offsets from track bases; track LEN and STA are scale
   factors, neutral at 8 [p044, p053].
7. **EDIT is a cycle**: steady green (normal), flashing orange (preview), flashing green
   (perform), red (MCC) [p068 to p070].
8. **Actions can be armed and cancelled.** On-The-Measure mute, solo and record flash until the
   measure ends [p067, RN].
9. **Scale is a visible object**: the circle shows the notes, orange marks the base, red SCALE
   SEL means force-to-scale, ESC locks it [p071, p072].
10. **MIDI is minimal and global.** One button ("200") sets the clock state; two ports; overload
    is shown on the CHORD LEDs [p093, p102].

## 4. Features, from the sources, and where they stand

**Where** is `E` engine exists (and is conformance-tested), `P` needs the panel controller,
`M` needs MIDI input, output or clock work. "Code search" means the engine source was searched
for the concept, not audited (`findings.md` section 3).

| Area | What the sources describe | Where |
|---|---|---|
| Hierarchy | Grid of 9 banks by 16 pages; Page of 10 tracks by 16 steps; Track zoom; Step zoom; attribute maps; direction map; phrase editor; Chord and Octave views [p013 to p015, p027, p042, p051, p057, p079 to p081] | P (the modes exist as data, `Mode::{Grid,Page,Track,Step}`) |
| Front panel | 16 by 10 matrix, selector column, mutator column, 10 MIX encoders, EDIT encoders, circle (numeric outer, pitch inner), chord block, transport, tempo encoder, mode buttons, the spiral [B, p011, p013, p027, p070] | P |
| Step attributes | VEL PIT LEN STA AMT GRV MCC and DIR POS MCH; chords with strum; phrases (48 factory); hypersteps; step events [p012 to p040] | E (step events partial), P |
| Track attributes | The ten attributes as scale factors, multipliers, MCH port and channel, maps [p041 to p057] | E, P |
| Effector | Feeders and listeners, listener step mask (AMT -127) [p059 to p063, RN] | E, P |
| Page | Scale, force-to-scale, cadence; page LEN and STA repeats; follow; chase-light align; mutators (TGL SOL CLR RND RMX CPY PST) [p065 to p078, p083 to p086] | E partly, P |
| Grid | Edit and Live; on-the-beat or immediate switching; clusters and Track-Across-Cluster; page sets (16 slots); page set as MIDI controller; interface lock [p079 to p086, RN] | P (cluster is a data field only) |
| Grid-Track | Virtual track selectors, mute patterns on step 16, solo by double-click [p080, p081, RN] | P (code search finds none) |
| Performance | On-The-Measure mute, solo and record; PLAY snapshot; Step Shift, Step Swap; hyperstep identification; track pause and alignment [p066 to p069, RN] | E partly (mute and solo gating), P |
| Recording | Arm a track, rehearse, multitrack, chords, step-note, step tapping at 1/192, quantised recording, CC, bend and pressure, controller map learn, anti-echo, keyboard transpose [p087 to p092, p078, RN] | M, P (only a `record_armed` flag exists) |
| Tempo and transport | Play, Stop, Pause, and the fast-forward buttons; tempo encoder shown as a bar graph; jump to fixed tempos [B, p049] | E, P |
| MIDI | Two ports of 16 channels; notes, CC, pitch bend, channel pressure out; record in; clock master, slave and echo; program change; ALL NOTES OFF on Stop [p093, p094] | E output only, M |
| Save and load | Machine state (one), a single page, factory reset, Device View at start, MIDI port priority [p095 to p099, B, RN] | P, plus a new file format (`tech.md` 7) |
| Interop | SysEx dumps, and Octopus to Nemo page conversion [p096, p097, p111, p112, RN] | Out of scope for v1 (below) |

## 5. Waves

Each wave ends with its fixtures passing (`tech.md` section 4) and a demonstration on real
gear. The owner may reorder them.

| Wave | Name | Contents |
|---|---|---|
| 1 | Play | Page mode: toggle steps, the chase-light, tempo, Play, Stop and Pause. Step zoom for VEL, PIT, LEN and STA. The MIX and EDIT encoders. Two MIDI outputs. Save and load in the browser. |
| 2 | Shape | Track zoom and the attribute maps, chords, phrases, hypersteps, scales, DIR and the direction map, chains, the effector, step events. |
| 3 | Perform | Grid mode and Live, On-The-Measure, Grid-Track, page sets, clusters, follow, PLAY snapshot. |
| 4 | Record and sync | MIDI input and recording, keyboard transpose, force-to-scale, controller map learn, the "200" clock states, program change, ALL NOTES OFF. |
| 5 | System | Device View (start mode, port priority), the file formats, MIDI file export, factory reset. |

## 6. MIDI requirements

Taken from the manual's MIDI chapter [p093, p094] and the record chapter [p087 to p092].

| Capability | Requirement | Web mapping |
|---|---|---|
| Ports and channels | Two ports of 16 channels; each track picks port 1 or 2 [p046, p078] | Port 1 and port 2 are each assigned to any Web MIDI output in a settings view |
| Out | Notes, CC (per-track MCC), pitch bend, channel pressure [p087, p090] | `MIDIOutput.send` with a future timestamp |
| Stop | ALL NOTES OFF (CC 123) on all 32 channels when the sequencer is stopped and the "200" LED is lit orange or green [p094] | As the manual; the engine already sends CC 123 on Stop |
| In | Record notes, controllers, bend and pressure; re-channel to the armed track; anti-echo [p087] | `MIDIInput` messages |
| Clock | Master: clock and transport out of both ports. Slave: clock in on one port, found automatically, with optional echo. Switching only in Grid or Page mode [p093] | `0xF8`, start, stop and continue on Web MIDI. Slave picks the port that sends clock |
| Program change | Slave mode only. Channel 10 selects a page set; channels 1 to 9 toggle pages in the matching bank [p094] | As the manual |
| SysEx | Dumps out of port 1 only, while stopped; import at any time [p096, p097] | **Not in v1.** The byte layout is not in the manual (`findings.md`), and SysEx needs its own browser permission |
| USB and DIN | The manual never mentions either [p064 to p124] | Not applicable: any Web MIDI port works |

## 7. Ableton

Three tiers. Tier 1 is the default first delivery (README D2). The sources for every claim are
in `tech.md` section 6.

| Tier | What it is | What the user does | Status |
|---|---|---|---|
| 1. MIDI routing | The web app in Chrome or Edge sends MIDI to a virtual port that Live reads, and can follow Live's MIDI clock | macOS: enable the IAC Driver. Windows: install loopMIDI. In Live, turn on Track for the port, arm a track; turn on Sync for the port that carries clock | Works with no extra code on either side. Needs the spike S1 and S3 numbers before any timing promise |
| 2. Inside Live | A Max for Live device that shows the same web app in a `jweb` view and passes events to the track | Drop the device on a MIDI track | Unproven. Spike S4 |
| 3. Link and plugins | Ableton Link through a small local helper; a VST3 or AU with an embedded web view | | Later. Not scoped |

## 8. Not doing in v1

- Rooms, generated worlds, light probes, impulse responses, or anything that needs a language
  model or a generative model. Removed, not deferred.
- A native app, Metal, Swift, VST3 or AU wrappers.
- A built-in synth or any audio output. The instrument is MIDI only. (A click or an audition
  voice is a later, separate decision.)
- SysEx exchange with real Octopus or Nemo hardware. The manual gives sizes and procedures but
  not the byte format [p096, p097].
- Photo-exact geometry, calibrated materials, or a blind comparison against photographs.
- Accounts, a backend, sharing, or multiplayer.
- Safari and iOS MIDI (the browsers do not offer Web MIDI).

## 9. Acceptance criteria

Every criterion is a measurement. Thresholds marked *set by spike* are decided from the spike's
numbers, by the owner, before the wave that depends on them starts. Loosening one later needs
an ADR with a reason and an expiry (`CLAUDE.md` rule 3).

| # | Criterion | How it is measured |
|---|---|---|
| A1 | **Manual conformance.** Every workflow in a wave's scope has a fixture: a timed sequence of button and encoder events in, an expected LED frame and engine state out. | `cargo xtask` gate `conformance`, extended in P2. The count only grows |
| A2 | **Engine determinism.** The WebAssembly engine plays every golden pattern byte-identically to native. | `just wasm-smoke` (exists), plus the same check in the browser (S2) |
| A3 | **MIDI timing, browser to instrument.** Note-on jitter over 30 minutes at 120 BPM, sixteenth notes, measured at the receiver. | *Set by spike S1.* The old 120 microsecond figure was for a plugin inside Live and does not apply |
| A4 | **Clock follow.** With Live as master at 120 BPM for 30 minutes over a virtual port, the app's beat stays within a set phase of Live's grid, with no drift. | *Set by spike S3* |
| A5 | **Live plays a Live instrument** from a pattern, and stops and starts with Live's transport. | A written procedure and a recorded run (`tech.md` 6) |
| A6 | **Hardware.** A pattern plays a real external synth on port 1 and a second device on port 2 at the same time. | A recorded run with the gear named |
| A7 | **Browsers.** The app loads, plays and passes the fixtures in current Chrome and Edge; in Safari it loads and runs without MIDI and says so. | Automated in Chrome (Playwright); a manual check for the others |
| A8 | **Keyboard and screen reader.** Every control is reachable by keyboard in the panel's layout order and has a label from the manual's own names. | An automated a11y check plus a written traversal order |
| A9 | **Scope.** The live tree contains no world-model term outside the history allow-list. | `verify:scope` (`removal-plan.md` 5) |
| A10 | **The ratchet holds.** No test is removed except by ADR-0006, and the assertion count only grows afterward. | `verify:regressions` (exists) |

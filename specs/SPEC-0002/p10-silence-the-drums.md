# SPEC-0002 P10: a button that silences the drums

| Field | Value |
|---|---|
| Task | `WENGE-0026` (the owner, Tue 2026-10-13 12:00 London: "let's add a button where you can silence the drums - it should be purely the synth and all of the midi that you can hear") |
| Spec | SPEC-0002 P6 R-2 (the built-in sound, `src/monitor.ts`) |
| Plan revision | r0, 2026-10-13 |
| Tier | **Medium**, Forge: `apps/web/`. No contract, threshold, golden file or engine code is touched |
| Stacked on | PR B (#57, the strip's look), which has the strip this adds a button to. It merges after #56 and #57 |

## 1. What the owner asked, and how it is read

The page plays the engine's own notes in the browser (the built-in sound, P6 R-2) so a page opened with nothing plugged in is heard to run: a pitched voice for a note, and a kick, a clap and a hat for the notes on MIDI channel 10. The owner wants to take the drums out of what is heard, leaving "purely the synth and all of the midi".

Read as: **a toggle in the strip that stops the built-in sound making the drums.** The synth voices are as they were. "All of the midi" is read as nothing sent to a MIDI output changes: the drum notes on channel 10 still go to a device that is chosen, because a hardware drum machine on that output is the thing the owner would then be hearing. The button does not touch the engine, the events or the click.

## 2. What changes, with the observable for each (tests written first)

| # | Change | Observable |
|---|---|---|
| P10-1 | The monitor has `drums`, on to begin with and independent of `on`. With it off a drum note adds nothing, exactly, and the synth notes beside it sound sample for sample as they do with no drums in the block | `monitor.test.ts` |
| P10-2 | Switching it off does not cut a hit that is sounding: the hit finishes as it would have (so it ends in the fade to zero it already has and makes no step), and every hit after it is silent; switching it on again makes the next hit | `monitor.test.ts` |
| P10-3 | Switching the sound off and on does not change `drums` | `monitor.test.ts` |
| P10-4 | The `drums` message reaches the monitor in the worklet: the demo with drums off is, sample for sample, the demo without its three drum tracks | `monitor.test.ts` (worklet rig) |
| P10-5 | It is not MIDI: the events the worklet makes are byte for byte the same with drums on and off | `monitor.test.ts` (worklet rig) |
| P10-6 | The strip has a `Drums` toggle after `Sound`, on to begin with (`aria-pressed="true"`, in `?bare` too), a stop for the Tab key in that place, and a click turns it off and on | `app.browser.test.ts`, `shell-look.browser.test.ts` |
| P10-7 | In the page, a track on channel 10 is heard with the button on and is silent, exactly, a moment after it is turned off (and heard again when it is back on), while the output the owner chose still receives its Note Ons | `drums.browser.test.ts` |
| P10-8 | In the page, a track on a synth channel is still heard with the button off | `drums.browser.test.ts` |
| P10-9 | A choice made before Play is kept: with the button turned off first, a drum track is silent from its first hit and the output still gets its Note Ons | `drums.browser.test.ts` |

## 3. Decisions the owner can change, with the default taken

| # | Decision | Default and why |
|---|---|---|
| D-P10-1 | The button is a toggle that says what is sounding: pressed (the raised tone, an ink border) means the drums are heard, as `Sound` does | One rule for the strip's toggles. The alternative, `Mute drums`, would show the raised tone for the quiet state |
| D-P10-2 | A hit that is sounding when the button is pressed finishes (at most 0.4 s) | Cutting a voice mid-wave is a click. A kick ends in the fade to zero it already has |
| D-P10-3 | The drums are the notes on channel 10, which is how the built-in sound already tells a drum from a note | The engine has no "drum track"; the monitor's own rule is the only definition there is. A synth on channel 10 is a drum to it, as it is now |
| D-P10-4 | The button is independent of `Sound`: with `Sound` off nothing is heard either way, and the choice is kept for when it is on | Two switches that mean what they say; nothing is disabled behind another |
| D-P10-5 | The choice is not stored between visits | `Sound` and the click are not either; the page keeps no settings yet |
| D-P10-6 | The click (the metronome) is not a drum and is not changed | It has its own button |

## 4. What is not done

- A mute for each drum (kick, clap, hat), or for any other track: the owner asked for the drums.
- Anything on the MIDI side: no note is dropped, moved or re-channelled.
- Storing the choice, and a keyboard shortcut.

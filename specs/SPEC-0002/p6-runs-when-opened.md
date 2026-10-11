# SPEC-0002 P6: it runs when it is opened

| Field | Value |
|---|---|
| Task | `WENGE-0017` (the owner's review of 2026-10-07: "this is really worrying how bad this project has got ... Not a sequencer that doesn't even run") |
| Spec | SPEC-0002 r1, Wave 1 (`product.md`: the chase-light) and `tech.md` section 5 (the playhead is delayed by the engine's lead). Stacked on `p6a-tick-length.md`, because a step must last what the manual says before anything is shown to move with it |
| Plan revision | r0, 2026-10-07 |
| Tier | **Medium**, Forge. `apps/web/` and one additive export in `apps/web/engine/`. `contracts/`, the thresholds and the golden files are not touched; the engine's notes are byte for byte what they were (R-2, the test that says so) |

## 1. What the owner saw

The page opened to a dark, empty panel, a **Start** button and, on the panel, a **Play** key that was not the transport's. Pressing Start made nothing happen. Pressing the panel's Play made the engine run an empty
pattern in silence, with no light moving, because: the pattern was empty, there is no sound of its own (MIDI only, and Web MIDI is refused in a sandboxed frame), the chase-light was never drawn, there was no tempo field and
the steps were four times too fast (P6a). Each of those is true of `cdaed68` and was measured on 2026-10-07 (`journal/forge/2026-10-07.md`).

## 2. What changes, with the observable for each

| # | Change | Observable (test, written first) |
|---|---|---|
| R-1 | The page opens with a demo pattern: five tracks (kick, hat and clap on channel 10, a bass and a lead), 23 steps. It is data (`src/demo.ts`), written through `setTrack` and `setStep` like anything else; the engine does not know it is a demo | `demo.test.ts`: the LED frame lights exactly the steps it names, played it makes exactly the notes it names; browser: 12 or more matrix keys lit after Play |
| R-2 | A built-in **monitor** sound in the worklet: a pitched voice for every note and three drum one-shots on channel 10. **On by default**, with a button to turn it off. It is audio, not MIDI: the events are byte for byte the same with it on or off | `monitor.test.ts`: sample-accurate start, pitch, release, never past full scale, deterministic; the events are identical on and off; browser, in a Chromium with its default autoplay policy: the output peaks above 0.02 within 4 s of one click |
| R-3 | The **chase-light**: the manual's Red LED on the step each track is playing, held back by the engine's lead (`MAX_EARLY_TICKS` = 12 ticks) so that it is on the step you hear | `chase.test.ts`: for three bars at 120 BPM the light is within one panel message (21 ms) of the step the audio is on; the engine's raw playhead is a step off and fails the same check, which is why the hold exists; browser: the light on track 1 visits 15 or more steps in 2.6 s, forward only |
| R-4 | **One Play.** `Start` is gone. The strip's button is `Play` and then `Stop`; the first press starts the engine and plays. The panel's own transport Play key does the same | browser: no `#start`; one click on `#play` makes `data-engine=running` and `data-transport=playing`; so does the key |
| R-5 | A **tempo** field, 120. It goes through the engine's own `set_tempo`; a value the engine refuses is put back | browser: the light moves twice as fast at 240; 99999 is put back |
| R-6 | The sound button | browser: on to begin with; off is silent with the light still moving; on again is heard |
| R-7 | Stop stops the sound and puts the light out; the pattern stays lit | browser |
| R-8 | `?bare` is the page the P3 to P5 tests drive: Play only starts the engine, nothing is loaded, nothing plays, nothing is heard. The 60 earlier browser tests run in it, with the helper, four sentences that named the button, and the tab order changed | the 60 earlier tests |

The engine gains **`octoweb_set_step(track, step, attr, value)`**, the step's `SetStep` as `octoweb_set_track` is the track's. It is additive, so the ABI number stays 2 as it did for `set_track` in P4b. Tests: `engine/tests/abi.rs`, `test/abi.test.ts`.

## 3. Decisions taken by default, so that nothing waits (each is the owner's to overturn)

- **D-P6-1: the sound is on by default.** A sequencer opened with nothing connected must be heard to run, and the review page cannot reach MIDI. A visitor with a synth plugged in hears the monitor as well as the synth, so the button is on the strip, next to the click.
- **D-P6-2: the engine starts on the first Play press, not on page load.** A browser keeps audio silent until the page has been used, so the panel is dark until then. The alternative (start the engine suspended and show the pattern at once) is an improvement for a later slice; it needs a measurement on the owner's browsers that I cannot make here.
- **D-P6-3: the monitor is a simple synth, not a design.** Sine and two harmonics for pitches, three noise and sine one-shots for drums. It exists to make the sequencer audible and is not the instrument's voice; a better one is not a Metronome or Forge decision.
- **D-P6-4: the demo's notes are a C minor pentatonic groove at the tempo the page opens at (120).** Nothing in it is a claim about the Octopus.
- **D-P6-5: the chase-light is Red whatever the track.** The manual has Orange where a Track MCC is set (`product.md`, Wave 1); MCC is not built, so the orange is not drawn, and the test for it is not written. It comes with MCC.

## 4. Not in this slice

The 21 encoders and the display are still not drawn (R2: all 247 controls); the page still does not look like hardware (R3, needs the Conductor's ADR on SPEC-0002 D3); the first step still sounds 11 ticks after Play (R0b, `p6a-tick-length.md`). Web MIDI cannot start inside the
review artifact's sandbox: for MIDI the page must be opened from `just web-app`.

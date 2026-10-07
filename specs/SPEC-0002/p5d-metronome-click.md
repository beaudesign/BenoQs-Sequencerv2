# SPEC-0002 P5 side slice: the metronome click

| Field | Value |
|---|---|
| Task | `WENGE-0016` (follows the exact resume, #41 and #42) |
| Spec | SPEC-0002 r1. Plan: `p5-plan.md` D-P5-13 **reading (b)**, which that plan said "not in P5, its own Forge slice (Medium)" and which the owner has now asked for: "Metronome needs to be built" (Sun 2026-10-04 16:09 London) |
| Plan revision | r0, 2026-10-04 |
| Stage | Built in the same pull request as this plan. The plan is short because the slice is small and cannot change what the engine plays |
| Tier | **Medium**, Forge. `crates/`, the contracts, the golden files and the thresholds are not touched. The click is audio the page makes; the MIDI events are byte for byte what they were (test C7) |

## 1. What the owner asked for

The four lines of Sun 11:07 end "tick should just be metronome on or off - which follows the midi and tempo set". The plan read
that two ways (D-P5-13). The 16:09 message, "Metronome needs to be built", is read as **(b): an audible click, on or off, that
follows the transport and the tempo.** D0 (192 ticks to a whole note or to a quarter) was not answered when this was written and the click does not need
it: it counts the engine's own clock grid, 24 pulses to a quarter note, `TICKS_PER_CLOCK` ticks to a pulse, so one click is
`24 * TICKS_PER_CLOCK` engine ticks (192 at the time, 48 since D0 was decided by default, `p6a-tick-length.md`). That is the beat an external device counts from the engine's MIDI clock, whatever the manual calls
a tick.

The click is **a page control, not a panel control**: the Octopus has no metronome key on its face (`controls.json` has none),
so nothing is added to the panel and no contract is edited.

## 2. Design

**W (chosen): the worklet works out each beat from the engine's tick position.** Each 128-frame block the worklet reads
`tickPosition()` before and after `render()`, finds the beats `k` with `p0 <= k * QUARTER_TICKS < p1` (`QUARTER_TICKS = 24 * TICKS_PER_CLOCK`), and writes a click into its own
output at the sample where each falls, by linear interpolation inside the block. No engine change, no ABI change.

- It follows the tempo because the position does: a tempo change moves where the next beat falls, with no rule of its own.
- It follows the transport because a stopped engine's position does not move, so there is nothing to find; and after a Play it
  carries on from where the position was, which is exactly where the exact resume (#41) put it. A Stop then Play clicks the beat
  it had not yet reached, not a new one. The engine's own Reset sets the position to 0 and the first beat is then the next Play's first
  frame, with nothing here told of it. (The worklet's `reset` message is not that: it drops the engine, `octoweb_reset`, and the page
  never sends it.)
- There is **no state to get wrong**: the beats of one block and the next are `[p0, p1)` and `[p1, p2)`, so none is clicked twice
  and none is skipped, including when a beat falls exactly on a block edge (the `<` is strict).

**E (rejected): the engine emits beat events.** It would sound identically and would put the click in the byte stream the
scheduler and the SMF export read, which is a contract and a golden file. A click that is not MIDI should not be in the MIDI.

## 3. Defaults the owner can overturn

| # | Default | Why |
|---|---|---|
| M-1 | Off at start. One click for each quarter note of the tempo set. | A sequencer that clicks on load would surprise a room |
| M-2 | No accent on the bar, no count-in, no volume control, no choice of sound. | The panel has no time signature to take a bar from; the others are each a decision of their own |
| M-3 | The sound is a fixed damped sine, 1 kHz, written by the worklet. | No sample to ship, no file to fetch, nothing to decode on the audio thread |
| M-4 | The choice made before Start is kept and takes effect at Start. | A switch that silently does nothing is a bug |
| M-5 | Switching off lets the click that is ringing finish. | A cut in the middle of a sine is itself a click |
| M-6 | No click while stopped. | "which follows the midi": it follows the transport |
| M-7 | The click is not trimmed against the MIDI. Audio goes out the audio device and MIDI out the MIDI device, and each has its own latency. | Only a measurement on the owner's setup can say by how much; README says so |

## 4. What I will be able to see

| # | Observable | Where |
|---|---|---|
| C1 | `beatsIn(p0, p1)`: exactly the beats in `[p0, p1)`, in order, none at `p1`; a beat on `p0` is in | `test/click.test.ts` |
| C2 | The offset of a beat in a block is its interpolated frame, clamped to the block | same |
| C3 | The voice is silent before its onset, its first sample is not zero, it peaks, it decays to nothing and then stops being active; the same samples whatever the block size | same |
| C4 | Off by default: a run of blocks is silent. On and playing at 120 BPM and 48 kHz: onsets at frames 0, 24000, 48000, ... within one frame | `test/worklet.test.ts` |
| C5 | Stop silences it; Play carries on from the point it stopped (the beat it had not reached, its distance from Stop unchanged), including a Stop that lands exactly on a beat; a position that goes back to 0 clicks beat 0 again | same, and `test/click.test.ts` |
| C6 | Tempo 240 puts the onsets half as far apart; a tempo change mid-run moves the next onset | same |
| C7 | The MIDI events of a run are identical bytes with the click on and off | same |
| C8 | The page has one button, `Metronome`, `aria-pressed` false then true, reachable by keyboard, with no colour of its own; and in a real browser the page's output carries signal when on and none when off | `test/app-css.test.ts`, `test/browser/` |

## 5. Not done

- Accent, count-in, a volume or a tone, a click for the bars, or a click on the subdivision.
- Latency compensation against external gear (M-7).
- Anything on the panel, in a contract, in the engine, or in the byte stream.
- A choice kept between page loads. Browser storage is not used for anything that matters here, and the owner's save and load
  (D-P5-1) is its own plan.

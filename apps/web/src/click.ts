// The metronome's click, made where the audio is made: in the worklet's own output (specs/SPEC-0002/p5d-metronome-click.md).
//
// It is derived from the engine's tick position and from nothing else. Each block the worklet reads the position before and after the
// engine renders, and the beats that fall in between are clicked, at the frame the interpolated position puts them. So the click follows
// the tempo, because the position does; it follows the transport, because a stopped engine's position does not move; and after a Stop it
// carries on from where the sound stopped, because the engine's own position does. No count is kept here to go out of step with the engine.
//
// One click is a quarter note of the MIDI clock: 24 pulses of `TICKS_PER_CLOCK` ticks. That is the beat a device counts from the engine's
// clock whatever a "tick" is called in the manual (D0), and it is why the unit is written as pulses and not as a number of ticks.
import { TICKS_PER_CLOCK } from "./abi.ts";

export const QUARTER_TICKS = 24 * TICKS_PER_CLOCK;

// The sound: a damped sine. The constants are the author's, chosen to be a clear short tick that does not startle: a thousand hertz is
// high enough to cut through a mix of low notes and low enough not to be shrill, and the click is over in a twentieth of a second.
export const CLICK_HZ = 1000;
/** The amplitude the click starts at, as a fraction of full scale. Half scale at most: it is a cue, not a signal. */
export const CLICK_PEAK = 0.3;
/** The amplitude falls by a factor of e in this long. */
const CLICK_DECAY_SECONDS = 0.004;
/** The click ends when it has fallen to this fraction of where it began (minus 80 dB), and is exactly nothing after. */
const CLICK_FLOOR = 1e-4;
/** How long a click lasts, in seconds. */
export const CLICK_TAIL_SECONDS = CLICK_DECAY_SECONDS * Math.log(1 / CLICK_FLOOR);

/**
 * The beats whose tick, `beat * QUARTER_TICKS`, is in `[p0, p1)`: a beat on the start is in and a beat on the end is not, so a run cut into
 * blocks at any places finds every beat once. (At the tempos the engine allows, a block holds one beat at most.) A span that does not move,
 * or goes backwards, holds none.
 */
export function beatsIn(p0: number, p1: number): number[] {
  const out: number[] = [];
  if (!(p1 > p0)) return out;
  let k = Math.ceil(p0 / QUARTER_TICKS);
  // The division can land a step either side of the beat; the products are exact, so settle it with them.
  while (k * QUARTER_TICKS < p0) k++;
  while ((k - 1) * QUARTER_TICKS >= p0) k--;
  for (; k * QUARTER_TICKS < p1; k++) out.push(k);
  return out;
}

/** The frame of a block of `frames` at which `beat` sounds, when the block runs from tick `p0` to tick `p1`: the position is taken as steady across it. */
export function beatFrame(beat: number, p0: number, p1: number, frames: number): number {
  const at = Math.round(((beat * QUARTER_TICKS - p0) / (p1 - p0)) * frames);
  return Math.min(frames - 1, Math.max(0, at));
}

/** One click. It is a function of how many frames old it is, so the same frames are written however the blocks are cut. */
export class ClickVoice {
  private age = 0;
  private live = false;
  private readonly length: number;
  private readonly omega: number;
  private readonly decay: number;

  constructor(sampleRate: number) {
    this.length = Math.ceil(CLICK_TAIL_SECONDS * sampleRate);
    this.omega = (2 * Math.PI * CLICK_HZ) / sampleRate;
    this.decay = 1 / (CLICK_DECAY_SECONDS * sampleRate);
  }

  /** True while the click has frames left to write. */
  get active(): boolean {
    return this.live;
  }

  /** Begins a click at the next frame written, over any that is still ringing. */
  start(): void {
    this.age = 0;
    this.live = true;
  }

  /** Adds the click's frames to `out[from..to)`. The first frame of a click is not zero: it is the click, not the silence before it. */
  render(out: Float32Array, from: number, to: number): void {
    const end = Math.min(to, out.length);
    for (let i = from; i < end && this.live; i++) {
      out[i] = (out[i] ?? 0) + CLICK_PEAK * Math.exp(-this.age * this.decay) * Math.sin(this.omega * (this.age + 1));
      if (++this.age >= this.length) this.live = false;
    }
  }
}

/** The click, on or off, for each block. */
export class Metronome {
  /** Off until the page says otherwise. */
  on = false;
  private readonly voice: ClickVoice;

  constructor(sampleRate: number) {
    this.voice = new ClickVoice(sampleRate);
  }

  /**
   * Writes the block's whole output: silence, with a click at every beat from `p0` to `p1` when the metronome is on and the transport was
   * running. A click that is ringing when the metronome is switched off, or the transport stopped, rings out: a cut part-way through a
   * sine would be a click of its own.
   */
  render(out: Float32Array, p0: number, p1: number, running: boolean): void {
    out.fill(0);
    let from = 0;
    if (this.on && running) {
      for (const beat of beatsIn(p0, p1)) {
        const frame = beatFrame(beat, p0, p1, out.length);
        this.voice.render(out, from, frame);
        this.voice.start();
        from = frame;
      }
    }
    this.voice.render(out, from, out.length);
  }
}

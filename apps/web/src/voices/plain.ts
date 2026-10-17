// Plain: the tone the page has always played, a sine and two harmonics (specs/SPEC-0002/p6-runs-when-opened.md, R-2), moved here
// from `monitor.ts` without changing a sample (SPEC-0004 S0, H-1). The arithmetic and its order are exactly what they were, so the
// output is bit for bit what it was: `test/voice-plain.test.ts` holds it to a frozen copy of the old monitor. Do not tidy it.
import { SLOT_COUNT, type Voice, type VoiceNote } from "./voice.ts";

/** Seconds. A held tone starts in 3 ms, falls to its sustain with a 60 ms time constant, and is let go over 100 ms. */
const ATTACK = 0.003;
const DECAY = 0.06;
const SUSTAIN = 0.6;
const RELEASE = 0.1;
/** A note whose Note Off never comes (a dropped message) is let go after this long, so no voice hangs. */
const LONGEST_HOLD = 10;
const VOICE_GAIN = 0.2;

class PlainNote implements VoiceNote {
  live = false;
  age = 0;
  private readonly rate: number;
  private amp = 0;
  private freq = 0;
  private phase = 0;
  /** Frames since its Note Off, or -1 while it is held. */
  private released = -1;

  constructor(sampleRate: number) {
    this.rate = sampleRate;
  }

  get held(): boolean {
    return this.released < 0;
  }

  start(note: number, velocity: number): void {
    this.live = true;
    this.amp = (Math.max(1, Math.min(127, velocity)) / 127) * VOICE_GAIN;
    this.age = 0;
    this.released = -1;
    this.phase = 0;
    this.freq = 440 * Math.pow(2, (note - 69) / 12);
  }

  release(): void {
    this.released = 0;
  }

  next(): number {
    const t = this.age / this.rate;
    let y = 0;
    if (this.released >= 0) {
      const r = this.released / this.rate;
      if (r >= RELEASE) {
        this.live = false;
        return 0;
      }
      y = this.level(t) * (1 - r / RELEASE);
      this.released++;
    } else {
      y = this.level(t);
      if (t >= LONGEST_HOLD) this.released = 0;
    }
    this.phase += (2 * Math.PI * this.freq) / this.rate;
    y *= (Math.sin(this.phase) + 0.5 * Math.sin(2 * this.phase) + 0.25 * Math.sin(3 * this.phase)) / 1.75;
    this.age++;
    return y * this.amp;
  }

  private level(t: number): number {
    return Math.min(1, t / ATTACK) * (SUSTAIN + (1 - SUSTAIN) * Math.exp(-t / DECAY));
  }
}

export const PLAIN: Voice = {
  id: "plain",
  name: "Plain",
  maxNotes: SLOT_COUNT,
  traits: { velocityBrightens: false },
  create: (sampleRate) => new PlainNote(sampleRate),
};

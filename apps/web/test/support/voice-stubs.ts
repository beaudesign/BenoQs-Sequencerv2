// Voices that are wrong on purpose, and one that is right, so the checks in `voice-checks.ts` can be seen to fail. A test double
// of a voice; never in the registry and never imported by `src/`.
import type { Voice, VoiceNote } from "../../src/voices/voice.ts";

export interface StubOptions {
  id: string;
  /** Detune from the note, in cents. */
  centsOff?: number;
  /** Has 48000 written into it where the sample rate belongs: right at 48 kHz, wrong at the other rates the page runs at. */
  fixedRate?: boolean;
  /** A naive sawtooth: the reason band-limiting exists. */
  saw?: boolean;
  /** A third harmonic whose level follows velocity: the sound gets brighter as it gets louder. */
  brightens?: boolean;
  /** What the voice says about itself. Defaults to the truth for `brightens`. */
  declares?: boolean;
  /** Not a number from this sample on. */
  nanAfter?: number;
  /** Uses `Math.random`. */
  random?: boolean;
  /** A release that never ends. */
  endless?: boolean;
  /** `start()` spends this many milliseconds of CPU: free in a workload that starts the notes once, ruinous when they restart. */
  slowStart?: number;
  /** Brightens with velocity during the attack only (the first 3000 samples), and says it does not. */
  transientBrightens?: boolean;
  /** Brightens with velocity up to about velocity 80 and dulls above it, while the declaration says yes. */
  dips?: boolean;
  /** `Math.random` once per 40,000 samples: past the first 20,000. */
  randomLate?: boolean;
  /** A naive sawtooth only for notes at or above this MIDI note. */
  sawAbove?: number;
  /** No sound for notes at or above this MIDI note. */
  silentAbove?: number;
  /** Does this many `Math.sin` a sample for its first `heavyDies` samples, then ends: cheap if it is started once and left. */
  heavyDies?: number;
  maxNotes?: number;
  /** Makes no sound at all (but lives and ends like any note), so a test can look at the drums alone. */
  silent?: boolean;
}

class StubNote implements VoiceNote {
  live = false;
  held = false;
  age = 0;
  private phase = 0;
  private step = 0;
  private level = 0;
  private velocity = 0;
  private sawNow = false;
  private released = -1;
  private readonly o: StubOptions;
  private readonly rate: number;

  constructor(o: StubOptions, rate: number) {
    this.o = o;
    this.rate = rate;
  }

  start(note: number, velocity: number): void {
    this.live = true;
    this.held = true;
    this.age = 0;
    this.phase = 0;
    this.released = -1;
    this.velocity = velocity / 127;
    this.level = this.o.silent || (this.o.silentAbove !== undefined && note >= this.o.silentAbove) ? 0 : 0.2 * this.velocity;
    this.sawNow = !!this.o.saw || (this.o.sawAbove !== undefined && note >= this.o.sawAbove);
    if (this.o.slowStart) {
      const t0 = performance.now();
      while (performance.now() - t0 < this.o.slowStart);
    }
    this.step = (440 * 2 ** ((note - 69) / 12 + (this.o.centsOff ?? 0) / 1200)) / (this.o.fixedRate ? 48_000 : this.rate);
  }

  release(): void {
    this.held = false;
    this.released = 0;
  }

  next(): number {
    if (!this.live) return 0;
    let fade = 1;
    if (this.released >= 0 && !this.o.endless) {
      fade = 1 - this.released++ / (0.1 * this.rate);
      if (fade <= 0) {
        this.live = false;
        return 0;
      }
    }
    this.phase += this.step;
    this.phase -= Math.floor(this.phase);
    const p = 2 * Math.PI * this.phase;
    let y = this.sawNow ? 2 * this.phase - 1 : Math.sin(p);
    if (this.o.brightens) y += this.velocity * 0.8 * Math.sin(3 * p);
    if (this.o.transientBrightens && this.age < 3000) y += this.velocity * 0.8 * Math.sin(3 * p);
    if (this.o.dips) y += (0.2 + 0.8 * Math.sin(Math.PI * this.velocity * 0.75)) * 0.8 * Math.sin(3 * p);
    if (this.o.randomLate && this.age % 40_000 === 39_999) y += Math.random() * 1e-3;
    if (this.o.heavyDies !== undefined) {
      if (this.age >= this.o.heavyDies) {
        this.live = false;
        return 0;
      }
      let z = 0;
      for (let j = 0; j < 40; j++) z += Math.sin(this.age + j);
      y += z * 1e-9;
    }
    if (this.o.random) y += Math.random() * 1e-3;
    if (this.o.nanAfter !== undefined && this.age >= this.o.nanAfter) y = Number.NaN;
    this.age++;
    return y * this.level * fade;
  }
}

export function stub(o: StubOptions): Voice {
  return {
    id: o.id,
    name: o.id,
    maxNotes: o.maxNotes ?? 24,
    traits: { velocityBrightens: o.declares ?? o.brightens ?? false },
    create: (rate: number) => new StubNote(o, rate),
  };
}

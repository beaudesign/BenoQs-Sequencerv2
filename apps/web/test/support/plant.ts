// A stand-in for what the follower steers: the engine's tick position as a function of the tempo it was last given, with the delays
// and the noise that sit between the follower and the engine. It is made up and says so: the real engine is in the loop in
// test/follower-engine.test.ts, and what a browser does between a message and the worklet, and between a block and the page's clock,
// is for spike S3 to measure. Here the delays are numbers that are plausible and are the ones the fixtures were tuned at.
//
//   * A tempo or a transport command takes `commandDelayMs` to reach the engine (a message to the worklet, applied before the next block).
//   * The engine's position is reported every `positionEveryMs` (the worklet's panel message, about 21 ms) with the time it holds at on the
//     page's clock wrong by up to `mapJitterMs` (the audio-to-page clock map moves by a millisecond or two, ADR-0009 decision 5).
//   * The engine ticks `TICKS_PER_QUARTER` to the quarter note: `ticks = integral of bpm * TICKS_PER_QUARTER / 60000` over milliseconds, and a
//     pulse is `TICKS_PER_CLOCK` ticks. D0: tick is 48 to the quarter note (the manual counts 192 to the whole note), so a pulse is 2 ticks;
//     it was 192 and 8. The pulse is taken from src/abi.ts, which test/abi.test.ts holds equal to the engine's.
import { TICKS_PER_CLOCK } from "../../src/abi.ts";
import { Rng } from "./rng.ts";
import type { Sim } from "./sim-clock.ts";

export { TICKS_PER_CLOCK };
export const TICKS_PER_QUARTER = TICKS_PER_CLOCK * 24;

export interface PlantConfig {
  seed: number;
  commandDelayMs: number;
  mapJitterMs: number;
  positionEveryMs: number;
  /** The tempo the engine has until it is told otherwise. */
  startBpm: number;
}

export const PLANT: Readonly<PlantConfig> = { seed: 1, commandDelayMs: 4, mapJitterMs: 1.5, positionEveryMs: 21.33, startBpm: 120 };

export class Plant {
  bpm: number;
  running = false;
  /** Ticks since the engine began, as a fraction. It stands still while stopped. */
  ticks = 0;
  /** Every tempo the engine was given, with the time it took effect. */
  readonly tempos: { at: number; bpm: number }[] = [];
  private now: number;
  private readonly config: PlantConfig;
  private readonly rng: Rng;
  private queue: { at: number; apply: () => void }[] = [];
  private nextPosition: number;

  constructor(startMs: number, config: Partial<PlantConfig> = {}) {
    this.config = { ...PLANT, ...config };
    this.rng = new Rng(this.config.seed * 31337 + 7);
    this.bpm = this.config.startBpm;
    this.now = startMs;
    this.nextPosition = startMs;
  }

  /** What the follower calls to set the engine's tempo. */
  setTempo(bpm: number): void {
    this.queue.push({ at: this.now + this.config.commandDelayMs, apply: () => void ((this.bpm = bpm), this.tempos.push({ at: this.now, bpm })) });
  }

  /** What the follower calls to start or stop the engine. */
  transport(play: boolean): void {
    this.queue.push({ at: this.now + this.config.commandDelayMs, apply: () => void (this.running = play) });
  }

  /** Time passes to `to`, in steps of at most `stepMs`. Calls `report` for each position the engine would report on the way. */
  advanceTo(to: number, report: (ticks: number, pageTimeMs: number) => void, stepMs = 0.5): void {
    while (this.now < to - 1e-9) {
      const next = Math.min(to, this.now + stepMs);
      const due = this.queue.filter((q) => q.at <= next).sort((a, b) => a.at - b.at);
      this.queue = this.queue.filter((q) => q.at > next);
      for (const q of due) {
        this.step(Math.max(this.now, q.at));
        q.apply();
      }
      this.step(next);
      if (this.now >= this.nextPosition) {
        report(this.ticks, this.now + (this.rng.next() * 2 - 1) * this.config.mapJitterMs);
        this.nextPosition += this.config.positionEveryMs;
      }
    }
  }

  private step(to: number): void {
    if (to <= this.now) return;
    if (this.running) this.ticks += ((to - this.now) * this.bpm * TICKS_PER_QUARTER) / 60_000;
    this.now = to;
  }

  get time(): number {
    return this.now;
  }
}

/** The sender's pulse count at page time `t`, as a fraction, from the true pulse times (no jitter). Clamps outside the run. */
export function senderPulses(sim: Sim, t: number): number {
  const truth = sim.truth;
  if (t <= truth[0]!) return (t - truth[0]!) / (truth[1]! - truth[0]!);
  const last = truth.length - 1;
  if (t >= truth[last]!) return last + (t - truth[last]!) / (truth[last]! - truth[last - 1]!);
  let lo = 0;
  let hi = last;
  while (hi - lo > 1) {
    const mid = (lo + hi) >> 1;
    if (truth[mid]! <= t) lo = mid;
    else hi = mid;
  }
  return lo + (t - truth[lo]!) / (truth[hi]! - truth[lo]!);
}

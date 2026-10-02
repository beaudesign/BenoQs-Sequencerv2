// A simulated external MIDI clock: what a sender's 0xF8 pulses look like to the page when they arrive with jitter, with a tempo that
// moves, with pulses missing and with pulses doubled. It is made up, and says so: nothing here was measured on a real port (spike S3
// measures that, SPEC-0002 P4 plan section 5). What it does give is a truth to compare an estimator against, because the true time of
// every pulse is kept next to the time it was delivered.
//
// The sender's tempo is a function of true time, and the pulses are placed by integrating it (a pulse every `2500 / bpm` milliseconds:
// 24 to the quarter note, so a quarter is `60000 / bpm`).
import { Rng } from "./rng.ts";

export interface SimConfig {
  seed: number;
  /** How long the sender runs, in seconds of its own time. */
  seconds: number;
  /** The tempo, in BPM, at `seconds` since the sender started. */
  bpm: (seconds: number) => number;
  /** Timestamp noise on every delivered pulse: `uniform` is plus or minus `ms`, `gaussian` is a standard deviation of `ms` (cut at 4). */
  jitter?: { shape: "uniform" | "gaussian"; ms: number };
  /** Pulses that never arrive, by index. */
  drop?: (index: number) => boolean;
  /** An extra copy of a pulse, this many milliseconds after the pulse's own time, by index. `null` for none. */
  duplicate?: (index: number) => number | null;
  /** The page time of pulse 0. */
  startMs?: number;
}

export interface SimPulse {
  /** When the page was told, in its own milliseconds. */
  at: number;
  /** The sender's count: 0 for its first pulse. An extra copy has the index of the pulse it copies. */
  index: number;
  extra: boolean;
}

export interface Sim {
  /** What the page receives, in the order it receives it. */
  pulses: SimPulse[];
  /** The true time of each pulse, before jitter, by index. One more than the pulses sent, so the next pulse's time is known. */
  truth: number[];
  /** The sender's tempo at each pulse. */
  tempo: number[];
}

export const steady = (bpm: number) => (): number => bpm;

/** A straight line in BPM from `from` to `to` over `over` seconds, then `to`. */
export const ramp =
  (from: number, to: number, over: number) =>
  (seconds: number): number =>
    seconds >= over ? to : from + ((to - from) * seconds) / over;

/** A step from `from` to `to` at `at` seconds. */
export const step =
  (from: number, to: number, at: number) =>
  (seconds: number): number =>
    seconds < at ? from : to;

function noise(rng: Rng, jitter: SimConfig["jitter"]): number {
  if (!jitter || jitter.ms === 0) return 0;
  if (jitter.shape === "uniform") return (rng.next() * 2 - 1) * jitter.ms;
  const u = Math.max(rng.next(), 1e-12);
  const g = Math.sqrt(-2 * Math.log(u)) * Math.cos(2 * Math.PI * rng.next());
  return Math.max(-4, Math.min(4, g)) * jitter.ms;
}

export function simulate(config: SimConfig): Sim {
  const rng = new Rng(config.seed * 7919 + 1);
  const start = config.startMs ?? 1_000;
  const truth: number[] = [];
  const tempo: number[] = [];
  let t = start;
  while (t - start <= config.seconds * 1000) {
    const bpm = config.bpm((t - start) / 1000);
    truth.push(t);
    tempo.push(bpm);
    t += 2500 / bpm;
  }
  truth.push(t);
  tempo.push(config.bpm((t - start) / 1000));
  const pulses: SimPulse[] = [];
  for (let index = 0; index < truth.length - 1; index++) {
    if (config.drop?.(index)) continue;
    const at = truth[index]! + noise(rng, config.jitter);
    pulses.push({ at, index, extra: false });
    const copy = config.duplicate?.(index) ?? null;
    if (copy !== null) pulses.push({ at: truth[index]! + copy, index, extra: true });
  }
  // The page hears things in the order they arrive.
  pulses.sort((a, b) => a.at - b.at);
  return { pulses, truth, tempo };
}

/** The time of the pulse after `index`, and how long a tick is there (a pulse is eight ticks). */
export function nextAfter(sim: Sim, index: number): { at: number; tickMs: number } {
  const at = sim.truth[index + 1]!;
  return { at, tickMs: (at - sim.truth[index]!) / 8 };
}

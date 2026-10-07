// The follower in a loop with a simulated sender (test/support/sim-clock.ts) and a simulated engine (test/support/plant.ts), on one
// timeline of page milliseconds. What it records is the HEARD error: where the engine's pulses are, plus the lookahead the scheduler adds
// to every event, against where the sender's pulses are, in milliseconds. That is the number a player hears as a note off the beat.
//
//   heard error (ms) = (engine pulse position at t  -  sender pulse position at t + lookahead + offset) * the sender's period
//
// positive means the engine is ahead (its notes land early). Everything here is made up and says so, in the files it uses.
import { Follower, type FollowerActions, type FollowerSettings, type FollowerTuning } from "../../src/follower.ts";
import { Plant, TICKS_PER_CLOCK, senderPulses, type PlantConfig } from "./plant.ts";
import type { Sim } from "./sim-clock.ts";

export interface Timed {
  at: number;
  run: (follower: Follower, plant: Plant) => void;
}

export interface LoopOptions {
  sim: Sim;
  settings?: Partial<FollowerSettings>;
  tuning?: Partial<FollowerTuning>;
  plant?: Partial<PlantConfig>;
  /** Page time of the Start message; default one millisecond before the first pulse to arrive. `null` for no Start at all. */
  startAt?: number | null;
  /** More things that happen, such as a Stop, or the offset changing. */
  script?: Timed[];
  /** Run until this page time. Default the last pulse. */
  until?: number;
  /** Where in whole pulses the engine begins, and where the sender's pulse 0 is taken to be. */
  engineStart?: number;
}

export interface Sample {
  /** Page time. */
  at: number;
  /** Milliseconds from the Start message. */
  sinceStart: number;
  heardMs: number;
  /** One unit of heard error (an eighth of a pulse, `HEARD_TICKS_PER_PULSE`) at the sender's tempo here, in milliseconds. */
  tickMs: number;
  running: boolean;
}

export interface LoopResult {
  samples: Sample[];
  follower: Follower;
  plant: Plant;
  /** Every call the follower made, in order. */
  calls: { at: number; what: "tempo" | "transport"; value: number | boolean }[];
}

const MEASURE_EVERY_MS = 5;

/**
 * The unit the heard error is counted in, and that every tolerance in test/follower.test.ts is in: an eighth of a pulse, 2.6 ms at 120 BPM.
 * D0: tick is 48 to the quarter note, so an ENGINE tick is now half a pulse (10.4 ms at 120 BPM) and this unit is a quarter of one. It was
 * one engine tick (a pulse was 8 ticks), and it is kept as what it was in time and not in engine ticks: counting the same tolerances in
 * the new tick would loosen them fourfold.
 */
export const HEARD_TICKS_PER_PULSE = 8;

export function runLoop(options: LoopOptions): LoopResult {
  const { sim } = options;
  const first = sim.truth[0]!;
  const plant = new Plant(first - 50, options.plant);
  const calls: LoopResult["calls"] = [];
  let lookahead = options.settings?.lookaheadMs ?? 30;
  let offset = options.settings?.offsetMs ?? 0;
  const actions: FollowerActions = {
    setTempo: (bpm) => {
      calls.push({ at: plant.time, what: "tempo", value: bpm });
      plant.setTempo(bpm);
    },
    transport: (play) => {
      calls.push({ at: plant.time, what: "transport", value: play });
      plant.transport(play);
    },
  };
  const follower = new Follower(actions, { lookaheadMs: lookahead, offsetMs: offset, ...options.settings }, options.tuning);
  // The Start goes out before the first pulse, as it does on a port (one port keeps its messages in order): one millisecond before the
  // first pulse to ARRIVE, which with jitter is not always the first the sender made. (A draft put it a millisecond before the sender's
  // first pulse and so, in 18 of 40 runs on seeds 100 to 139, after the pulse that had come early: the engine's first pulse was then the
  // sender's second, and every run of those settled a whole pulse off. Found by measuring on seeds the tests did not use.)
  const startAt = options.startAt === undefined ? sim.pulses[0]!.at - 1 : options.startAt;
  const until = options.until ?? sim.pulses[sim.pulses.length - 1]!.at;

  // One timeline: the pulses, the script, the measurements.
  type Item = { at: number; order: number; run: () => void };
  const items: Item[] = [];
  if (startAt !== null) items.push({ at: startAt, order: 0, run: () => follower.onStart(startAt) });
  for (const p of sim.pulses) items.push({ at: p.at, order: 1, run: () => follower.onClock(p.at) });
  for (const s of options.script ?? []) items.push({ at: s.at, order: 2, run: () => s.run(follower, plant) });
  const samples: Sample[] = [];
  const engineStart = options.engineStart ?? 0;
  for (let t = first; t < until; t += MEASURE_EVERY_MS) {
    items.push({
      at: t,
      order: 3,
      run: () => {
        const pulses = plant.ticks / TICKS_PER_CLOCK - engineStart;
        const index = Math.max(0, Math.min(sim.tempo.length - 1, Math.floor(senderPulses(sim, t))));
        const period = 2500 / sim.tempo[index]!;
        const expected = senderPulses(sim, t + lookahead + offset);
        samples.push({ at: t, sinceStart: t - (startAt ?? first), heardMs: (pulses - expected) * period, tickMs: period / HEARD_TICKS_PER_PULSE, running: plant.running });
      },
    });
  }
  items.sort((a, b) => a.at - b.at || a.order - b.order);
  const report = (ticks: number, pageTimeMs: number): void => follower.onPosition({ ticks, pageTimeMs });
  // `setLookahead` and `setOffset` reach the follower through the script, and the loop's own copies follow them.
  const originalLookahead = follower.setLookahead.bind(follower);
  const originalOffset = follower.setOffset.bind(follower);
  follower.setLookahead = (ms: number): void => {
    lookahead = ms;
    originalLookahead(ms);
  };
  follower.setOffset = (ms: number): void => {
    offset = ms;
    originalOffset(ms);
  };
  for (const item of items) {
    if (item.at > until) break;
    plant.advanceTo(item.at, report);
    item.run();
  }
  return { samples, follower, plant, calls };
}

/** The largest heard error, in ticks, among the samples from `fromMs` after Start to `toMs`, while the engine is running. */
export function worstTicks(samples: Sample[], fromMs: number, toMs = Infinity): number {
  const picked = samples.filter((s) => s.running && s.sinceStart >= fromMs && s.sinceStart < toMs);
  if (picked.length === 0) throw new Error(`no running samples between ${fromMs} and ${toMs} ms after Start`);
  return Math.max(...picked.map((s) => Math.abs(s.heardMs) / s.tickMs));
}

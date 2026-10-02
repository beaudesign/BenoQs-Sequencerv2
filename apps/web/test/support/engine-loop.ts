// The follower in front of the REAL engine, in a loop with a simulated sender, on one fake clock: the real worklet.ts and the real wasm
// module (test/support/worklet-scope.ts), the real scheduler (src/midi-out.ts) and the real follower (src/follower.ts). Used by
// test/follower-engine.test.ts and scripts/measure-follower-engine.ts. Only the sender, the clocks and the receiver are made up: the
// sender is test/support/sim-clock.ts (its pulses arrive with jitter), the receiver is the stand-in output of test/support/receiver.ts,
// and the delay between a block and the page hearing of it is a few milliseconds of Session's own making. What a browser does between
// a message and the worklet, and the audio clock's map to the page's, is for spike S3 to measure.
//
// What is measured is what reaches the DEVICE: the engine is switched to master only so that its pulses go out (the app never has both,
// src/clock-state.ts), and the time each one is delivered, which is when it was made plus the lookahead, is set against the time of the
// sender's pulse with the same number. That difference, in ticks (an eighth of a pulse), is what a player hears as a note off the beat.
//
//   heard error (ticks) = (delivered time of the engine's pulse j  -  true time of the sender's pulse j  +  the offset) / one tick
//
// The engine's pulse 0 is the one that goes out with its Start and the sender's pulse 0 is the first it sends, so j is the same number
// for both: a downbeat a whole pulse out would read 8 ticks, which is the failure the follower's alignment is there to prevent.
import { RENDER_FRAMES } from "../../src/abi.ts";
import { Follower } from "../../src/follower.ts";
import { Session } from "./session.ts";
import { Rng } from "./rng.ts";
import { simulate, type Sim, type SimConfig } from "./sim-clock.ts";

const BASE_MS = 1_000; // test/support/session.ts: the page time of audio time 0
const RENDER_AHEAD_MS = 10; // and how far ahead of the speakers a block is made, by default
const SAMPLE_RATE = 48_000;
export const BLOCK_MS = (RENDER_FRAMES / SAMPLE_RATE) * 1000;
export const JITTER = { shape: "uniform", ms: 3 } as const;

export interface Script {
  at: number;
  run: (follower: Follower, session: Session) => void;
}

export interface Options {
  seed?: number;
  lookaheadMs?: number;
  /** The scheduler's lookahead, if it is not the same as the follower's (it is meant to be). */
  schedulerLookaheadMs?: number;
  offsetMs?: number;
  /** How far the page's idea of audio time is wrong, at each batch of events sent to the device, up to this many ms. The output's own noise, not the follower's: off unless asked for, so that what is measured is the follower. */
  outputJitterMs?: number;
  script?: Script[];
  /** Page time of the sender's Stop, if it stops. */
  stopAt?: number;
}

export interface Run {
  /** Engine pulses delivered to port 1's device, in order, with when. */
  pulses: number[];
  /** Note Ons delivered, with when. */
  notes: number[];
  stopDelivered: number | null;
  follower: Follower;
  session: Session;
  sim: Sim;
  startAt: number;
}

export function closedLoop(config: Omit<SimConfig, "seed" | "jitter">, options: Options = {}): Run {
  const seed = options.seed ?? 1;
  const lookahead = options.lookaheadMs ?? 30;
  const sim = simulate({ seed, jitter: JITTER, startMs: 2_000, ...config });
  const session = new Session({ seed, supportsClear: false, mapJitterMs: options.outputJitterMs ?? 0, delayMs: 4, lookaheadMs: options.schedulerLookaheadMs ?? lookahead, renderAheadMs: RENDER_AHEAD_MS });
  const rng = new Rng(seed * 104729 + 3);
  // What the follower asks of the engine waits for the end of the block, as a message to a worklet does.
  const pending: Parameters<Session["send"]>[0][] = [];
  const follower = new Follower(
    { setTempo: (bpm) => pending.push({ type: "tempo", bpm }), transport: (play) => pending.push({ type: "transport", play }) },
    { lookaheadMs: lookahead, offsetMs: options.offsetMs ?? 0 },
  );
  session.onPanel = (panel) => {
    follower.onPosition({ ticks: panel.position, pageTimeMs: BASE_MS + (panel.positionFrame / SAMPLE_RATE) * 1000 + (rng.next() * 2 - 1) * 1.5 });
  };
  // A pattern with a note on step 1 and one on step 9: pulses 0 and 12 of 24.
  session.clickStep(0, 0);
  session.clickStep(0, 8);
  session.send({ type: "clock", master: true });

  const startAt = sim.pulses[0]!.at - 1;
  const events: { at: number; run: () => void }[] = [{ at: startAt, run: () => follower.onStart(startAt) }];
  for (const p of sim.pulses) events.push({ at: p.at, run: () => follower.onClock(p.at) });
  if (options.stopAt !== undefined) events.push({ at: options.stopAt, run: () => follower.onStop(options.stopAt!) });
  for (const s of options.script ?? []) events.push({ at: s.at, run: () => s.run(follower, session) });
  events.sort((a, b) => a.at - b.at);

  const last = sim.pulses[sim.pulses.length - 1]!.at;
  let next = 0;
  for (let block = 0; ; block++) {
    const now = BASE_MS + block * BLOCK_MS - RENDER_AHEAD_MS;
    if (now > last + 200) break;
    while (next < events.length && events[next]!.at <= now) events[next++]!.run();
    for (const m of pending.splice(0)) session.send(m);
    session.step();
  }
  session.drain(500);
  const out = session.outputs.get(1)!.delivered;
  return {
    pulses: out.filter((d) => d.bytes[0] === 0xf8).map((d) => d.at),
    notes: out.filter((d) => (d.bytes[0]! & 0xf0) === 0x90 && (d.bytes[2] ?? 0) > 0).map((d) => d.at),
    stopDelivered: out.find((d) => d.bytes[0] === 0xfc)?.at ?? null,
    follower,
    session,
    sim,
    startAt,
  };
}

/** The heard error of each pulse delivered, in ticks, against the sender's pulse with the same number. */
export function pulseErrors(run: Run, offsetMs = 0): { at: number; sinceStart: number; ticks: number }[] {
  return run.pulses.slice(0, run.sim.truth.length - 1).map((at, j) => {
    const period = run.sim.truth[j + 1]! - run.sim.truth[j]!;
    return { at, sinceStart: at - run.startAt, ticks: (at - run.sim.truth[j]! + offsetMs) / (period / 8) };
  });
}

/** The largest heard error in ticks among the pulses from `from` to `to` milliseconds after the Start. */
export const worst = (errors: { sinceStart: number; ticks: number }[], from: number, to = Infinity): number => {
  const picked = errors.filter((e) => e.sinceStart >= from && e.sinceStart < to);
  if (picked.length <= 20) throw new Error(`${picked.length} pulses between ${from} and ${to} ms`);
  return Math.max(...picked.map((e) => Math.abs(e.ticks)));
};


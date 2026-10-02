// The page the browser tests drive. The real `Host` (module compiled here, worklet started, messages routed),
// the real scheduler and clock map, and a recording stand-in for the MIDI output. No UI.
import { layoutFromControls } from "../src/abi.ts";
import { Host } from "../src/host.ts";
import { ContextTimeMap, MidiScheduler, type SchedulerStats, type SentInfo } from "../src/midi-out.ts";
import { wireMidi } from "../src/wire.ts";

export interface Send extends SentInfo {
  /** `performance.now()` when `send` was called. */
  wall: number;
}

export interface StartOptions {
  lookaheadMs?: number;
  /** Smoothing of the clock map; 1 (the default) uses the latest pair alone. */
  alpha?: number;
  wasm?: string;
}

export interface Harness {
  start(options?: StartOptions): Promise<{ sampleRate: number; baseLatency: number; outputLatency: number; visibility: string }>;
  click(id: string): void;
  tempo(bpm: number): void;
  transport(play: boolean): void;
  /** Makes the engine the MIDI clock master, or not (ADR-0009). */
  clock(master: boolean): void;
  sends: Send[];
  errors: string[];
  panelCount(): number;
  lastLeds(): number[] | null;
  status(): { running: boolean; zoomed: boolean; playheads: number[]; droppedIntents: number };
  stats(): SchedulerStats;
  /** What a receiver ends up holding if it plays the sends in timestamp order. */
  held(): string[];
  setLookahead(ms: number): void;
  visibility(): string;
  close(): Promise<void>;
}

interface Control {
  n: number;
  id: string;
}

let host: Host | null = null;
let scheduler: MidiScheduler | null = null;
let numbers = new Map<string, number>();
let lastLeds: Uint8Array | null = null;
let lastStatus = { running: false, zoomed: false, playheads: [] as number[], droppedIntents: 0 };
let panels = 0;
const sends: Send[] = [];
const errors: string[] = [];

const harness: Harness = {
  async start(options = {}) {
    const controls = (await (await fetch("/contracts/controls.json")).json()) as { controls: Control[] };
    numbers = new Map(controls.controls.map((c) => [c.id, c.n]));
    const wasm = await (await fetch(options.wasm ?? "/dist/octoweb.wasm")).arrayBuffer();
    host = await Host.start({ wasm, layout: layoutFromControls(controls), workletUrl: "/dist/src/worklet.js", seed: 0n });
    const map = new ContextTimeMap(host.context, performance, options.alpha ?? 1);
    scheduler = new MidiScheduler(map, options.lookaheadMs ?? 30);
    scheduler.onSend = (info) => sends.push({ ...info, wall: performance.now() });
    // The recording output: it keeps what the scheduler gave it; `held()` replays it.
    scheduler.setOutput(1, { send: () => undefined });
    scheduler.setOutput(2, { send: () => undefined });
    wireMidi(host, scheduler, map);
    host.onPanel = (p) => {
      panels++;
      if (p.leds) lastLeds = p.leds;
      lastStatus = { running: p.running, zoomed: p.zoomed, playheads: [...p.playheads], droppedIntents: p.droppedIntents };
    };
    host.onError = (m) => errors.push(m);
    return {
      sampleRate: host.sampleRate,
      baseLatency: host.context.baseLatency,
      outputLatency: host.context.outputLatency,
      visibility: document.visibilityState,
    };
  },
  click(id) {
    const n = numbers.get(id);
    if (!host || n === undefined) throw new Error(`cannot click ${id}`);
    host.press(n);
    host.release(n);
  },
  tempo: (bpm) => host?.setTempo(bpm),
  transport: (play) => host?.transport(play),
  clock: (master) => host?.setClock(master),
  sends,
  errors,
  panelCount: () => panels,
  lastLeds: () => (lastLeds ? [...lastLeds] : null),
  status: () => lastStatus,
  stats: () => (scheduler ? { ...scheduler.stats } : ({} as SchedulerStats)),
  held() {
    const ordered = sends
      .map((s, i) => ({ s, i, at: s.timestamp ?? s.wall }))
      .sort((a, b) => a.at - b.at || a.i - b.i);
    const held = new Set<string>();
    for (const { s } of ordered) {
      const status = (s.bytes[0] ?? 0) & 0xf0;
      const key = `${s.port}/${(s.bytes[0] ?? 0) & 0x0f}/${s.bytes[1]}`;
      if (status === 0x90 && (s.bytes[2] ?? 0) > 0) held.add(key);
      else if (status === 0x80) held.delete(key);
    }
    return [...held];
  },
  setLookahead(ms) {
    if (scheduler) scheduler.lookaheadMs = ms;
  },
  visibility: () => document.visibilityState,
  async close() {
    await host?.close();
    host = null;
  },
};

(window as unknown as { harness: Harness }).harness = harness;

// The page and the worklet and a MIDI receiver, in one process on a fake clock: the real worklet.ts
// (through test/support/worklet-scope.ts), the real routing and wiring (src/host.ts routeFromWorklet,
// src/wire.ts) and the real scheduler (src/midi-out.ts). Only the clocks, the message delays and the
// receiver are made up, and each is described where it is made.
import { RENDER_FRAMES, decodeEvents, type MidiEvent } from "../../src/abi.ts";
import { routeFromWorklet, type PanelFrame, type WorkletSink } from "../../src/host.ts";
import { MidiScheduler, type TimeMap } from "../../src/midi-out.ts";
import type { FromWorklet } from "../../src/protocol.ts";
import { wireMidi } from "../../src/wire.ts";
import { controlNumbers, matrixId } from "./module.ts";
import { FakeClock, FakeOutput, type LogEntry } from "./receiver.ts";
import { WorkletRig, type Posted } from "./worklet-scope.ts";

/** A small deterministic generator (xorshift32), so a failing run can be repeated from its seed. */
export class Rng {
  private s: number;
  constructor(seed: number) {
    this.s = seed >>> 0 || 0x9e3779b9;
  }
  next(): number {
    let x = this.s;
    x ^= x << 13;
    x >>>= 0;
    x ^= x >>> 17;
    x ^= x << 5;
    x >>>= 0;
    this.s = x;
    return x / 0x1_0000_0000;
  }
  int(n: number): number {
    return Math.floor(this.next() * n);
  }
}

export interface SessionConfig {
  seed: number;
  supportsClear: boolean;
  /** The page's estimate of the audio-to-page clock offset is wrong by up to this many ms, redrawn at each batch. */
  mapJitterMs: number;
  /** Messages from the worklet reach the page up to this many ms after the block that made them was rendered. */
  delayMs: number;
  /** One message in this many (0 to 1) takes `spikeMs` more, as when the main thread is busy. */
  spikeChance?: number;
  spikeMs?: number;
  lookaheadMs?: number;
  /** How far ahead of the speakers the browser renders a block. */
  renderAheadMs?: number;
}

const BASE_MS = 1_000;

class JitterMap implements TimeMap {
  private err = 0;
  private readonly clock: FakeClock;
  private readonly rng: Rng;
  private readonly jitter: number;
  constructor(clock: FakeClock, rng: Rng, jitter: number) {
    this.clock = clock;
    this.rng = rng;
    this.jitter = jitter;
  }
  now(): number {
    return this.clock.now;
  }
  refresh(): void {
    this.err = (this.rng.next() * 2 - 1) * this.jitter;
  }
  toPage(seconds: number): number {
    return seconds * 1000 + BASE_MS + this.err;
  }
}

export class Session implements WorkletSink {
  onEvents: WorkletSink["onEvents"] = null;
  onPanel: ((panel: PanelFrame) => void) | null = null;
  onError: ((message: string) => void) | null = (m) => this.errors.push(m);
  readonly errors: string[] = [];
  readonly rig = new WorkletRig();
  readonly clock = new FakeClock();
  readonly log: LogEntry[] = [];
  readonly outputs = new Map<number, FakeOutput>();
  readonly scheduler: MidiScheduler;
  readonly sampleRate = this.rig.sampleRate;
  /** Where in `log` the page was when it was last told the transport had stopped, or null. */
  stopAt: number | null = null;
  block = 0;
  private wasRunning = false;
  private lastDelivery = 0;
  private readonly rng: Rng;
  private readonly config: SessionConfig;
  private readonly n = controlNumbers();

  constructor(config: SessionConfig) {
    this.config = config;
    this.rng = new Rng(config.seed * 7919 + 13);
    for (const port of [1, 2]) this.outputs.set(port, new FakeOutput(this.clock, port, this.log, config.supportsClear));
    const map = new JitterMap(this.clock, this.rng, config.mapJitterMs);
    this.scheduler = new MidiScheduler(map, config.lookaheadMs ?? 30);
    for (const [port, out] of this.outputs) this.scheduler.setOutput(port, out);
    wireMidi(this, this.scheduler, map);
    // The panel message that first says "stopped" marks the Stop for the checks.
    this.onPanel = (panel) => {
      if (!panel.running && this.wasRunning) this.stopAt = this.log.length;
      this.wasRunning = panel.running;
    };
    this.rig.take();
  }

  private pageTimeOfBlock(b: number): number {
    return BASE_MS + ((b * RENDER_FRAMES) / this.sampleRate) * 1000 - (this.config.renderAheadMs ?? 10);
  }

  private deliver(posted: Posted[], at: number): void {
    for (const p of posted) {
      const spike = this.rng.next() < (this.config.spikeChance ?? 0) ? (this.config.spikeMs ?? 0) : 0;
      const when = Math.max(this.lastDelivery, at + this.rng.next() * this.config.delayMs + spike);
      for (const o of this.outputs.values()) o.advanceTo(when);
      this.clock.now = Math.max(this.clock.now, when);
      this.lastDelivery = when;
      routeFromWorklet(p.message, this);
    }
  }

  /** Presses and releases a control, as the panel would: the page tells the worklet between blocks. */
  click(id: string): void {
    const control = this.n.get(id);
    if (control === undefined) throw new Error(`${id} is not a control`);
    for (const kind of [0, 1]) {
      this.rig.send({ type: "input", nowMs: this.pageTimeOfBlock(this.block), kind, control, detents: 0 });
    }
    this.deliver(this.rig.take(), this.pageTimeOfBlock(this.block));
  }

  clickStep(row: number, step: number): void {
    this.click(matrixId(row, step));
  }

  send(message: Parameters<WorkletRig["send"]>[0]): void {
    this.rig.send(message);
    this.deliver(this.rig.take(), this.pageTimeOfBlock(this.block));
  }

  /** Renders one block. Returns the events it made. */
  step(): MidiEvent[] {
    const posted = this.rig.block();
    this.deliver(posted, this.pageTimeOfBlock(this.block));
    this.block++;
    const out: MidiEvent[] = [];
    for (const p of posted) if (p.message.type === "events") out.push(...decodeEvents(new Uint8Array(p.message.bytes)));
    return out;
  }

  /** Time passes with nothing new from the worklet. */
  drain(ms: number): void {
    const to = this.clock.now + ms;
    for (const o of this.outputs.values()) o.advanceTo(to);
    this.clock.now = to;
  }

  held(): string[] {
    return [...this.outputs.entries()].flatMap(([port, o]) => [...o.held].map((k) => `port ${port} ${k}`));
  }

  messagesOfType(messages: FromWorklet[], type: FromWorklet["type"]): FromWorklet[] {
    return messages.filter((m) => m.type === type);
  }
}

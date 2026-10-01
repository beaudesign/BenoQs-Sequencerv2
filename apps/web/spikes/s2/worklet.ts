// Spike S2's worklet: the octoweb module in an AudioWorkletGlobalScope, doing the work the spike asks for in
// message handlers (so it can be timed with `Date`, the only clock the scope has). The shipped worklet is
// src/worklet.ts; this one is only for the spike and is not part of the app.
/// <reference path="../../src/worklet-env.d.ts" />
import { Octoweb, type Exports } from "../../src/abi.ts";
import { play, type Programs } from "../program.ts";

export type S2Request =
  | { id: number; type: "hash"; name: string; source: Uint8Array }
  | { id: number; type: "program"; layout: Uint8Array; numbers: [string, number][]; doc: Programs; name: string }
  | { id: number; type: "timing"; layout: Uint8Array; numbers: [string, number][]; doc: Programs; name: string; blocks: number; batch: number };

export type S2Reply =
  | { type: "ready"; sampleRate: number; measure: boolean; spike: boolean; hasPerformance: boolean; hasDate: boolean; hasTextDecoder: boolean }
  | { id: number; type: "hash"; name: string; isError: boolean; text: string; allocs: number | null }
  | { id: number; type: "program"; name: string; digest: string; events: number; blocks: number }
  | { id: number; type: "timing"; name: string; blocks: number; batch: number; batchMs: number[]; allocsBefore: number | null; allocsAfter: number | null; events: number }
  | { id: number; type: "error"; message: string };

class S2Processor extends AudioWorkletProcessor {
  private readonly x: Exports;

  constructor(options: { processorOptions: unknown }) {
    super();
    const o = options.processorOptions as { module: WebAssembly.Module };
    this.x = new WebAssembly.Instance(o.module, {}).exports as unknown as Exports;
    this.port.onmessage = (e: MessageEvent<S2Request>) => this.handle(e.data);
    const scope = globalThis as unknown as Record<string, unknown>;
    this.reply({
      type: "ready",
      sampleRate,
      measure: typeof this.x.octoweb_alloc_count === "function",
      spike: typeof this.x.octoweb_spike_run === "function",
      hasPerformance: typeof scope["performance"] !== "undefined",
      hasDate: typeof Date.now === "function",
      hasTextDecoder: typeof scope["TextDecoder"] !== "undefined",
    });
  }

  private reply(m: S2Reply): void {
    this.port.postMessage(m);
  }

  private allocs(): number | null {
    return this.x.octoweb_alloc_count ? this.x.octoweb_alloc_count() >>> 0 : null;
  }

  private handle(r: S2Request): void {
    try {
      switch (r.type) {
        case "hash": {
          const run = this.x.octoweb_spike_run;
          if (!run) throw new Error("the module has no octoweb_spike_run");
          const before = this.allocs();
          const p = this.x.octoweb_scratch(r.source.byteLength);
          new Uint8Array(this.x.memory.buffer, p, r.source.byteLength).set(r.source);
          const out = run(r.source.byteLength) >>> 0;
          const len = out & 0x7fff_ffff;
          const bytes = new Uint8Array(this.x.memory.buffer, this.x.octoweb_scratch(len), len);
          let text = "";
          for (const b of bytes) text += String.fromCharCode(b);
          const after = this.allocs();
          this.reply({ id: r.id, type: "hash", name: r.name, isError: out >>> 31 === 1, text, allocs: before === null || after === null ? null : after - before });
          break;
        }
        case "program": {
          const program = r.doc.programs[r.name];
          if (!program) throw new Error(`no program ${r.name}`);
          const engine = this.fresh(r.layout, r.doc.sample_rate, program.seed);
          const played = play(engine, new Map(r.numbers), r.doc, program);
          this.reply({ id: r.id, type: "program", name: r.name, digest: played.digest, events: played.events, blocks: played.blocks });
          break;
        }
        case "timing": {
          const program = r.doc.programs[r.name];
          if (!program) throw new Error(`no program ${r.name}`);
          const engine = this.fresh(r.layout, r.doc.sample_rate, program.seed);
          const batchMs: number[] = [];
          let started = 0;
          let allocsBefore: number | null = null;
          let allocsAfter: number | null = null;
          // The presses and the start allocate (the controller makes small vectors); the render loop must not.
          const played = play(engine, new Map(r.numbers), r.doc, program, {
            blocks: r.blocks,
            hash: false,
            onBlock: (b) => {
              if (b === 0) {
                allocsBefore = this.allocs();
                started = Date.now();
              } else if (b % r.batch === 0) {
                const now = Date.now();
                batchMs.push(now - started);
                started = now;
              }
              if (b === r.blocks - 1) allocsAfter = this.allocs();
            },
          });
          this.reply({ id: r.id, type: "timing", name: r.name, blocks: r.blocks, batch: r.batch, batchMs, allocsBefore, allocsAfter, events: played.events });
          break;
        }
      }
    } catch (e) {
      this.reply({ id: r.id, type: "error", message: String(e) });
    }
  }

  private fresh(layout: Uint8Array, rate: number, seed: number): Octoweb {
    const engine = new Octoweb(this.x);
    try {
      engine.reset();
    } catch {
      // not initialised yet
    }
    engine.init(layout, rate, BigInt(seed));
    return engine;
  }

  override process(): boolean {
    return true;
  }
}

registerProcessor("octoweb-s2", S2Processor);

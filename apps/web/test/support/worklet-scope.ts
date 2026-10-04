// A stand-in for the AudioWorkletGlobalScope, so the real src/worklet.ts runs in Node: the same
// class, the same module, the same messages. It supplies only what the worklet scope supplies:
// `AudioWorkletProcessor`, `registerProcessor`, `sampleRate`, `currentFrame`, `WebAssembly`.
import { RENDER_FRAMES } from "../../src/abi.ts";
import { WORKLET_NAME, type FromWorklet, type ToWorklet, type WorkletOptions } from "../../src/protocol.ts";
import { compile, layoutBytes } from "./module.ts";

export interface Posted {
  message: FromWorklet;
  transfer: unknown[];
}

class FakePort {
  onmessage: ((e: { data: ToWorklet }) => void) | null = null;
  readonly posted: Posted[] = [];
  postMessage(message: FromWorklet, transfer: unknown[] = []): void {
    this.posted.push({ message, transfer });
  }
}

interface Processor {
  port: FakePort;
  process(inputs?: Float32Array[][], outputs?: Float32Array[][]): boolean;
}
type ProcessorClass = new (options: { processorOptions: WorkletOptions }) => Processor;

const scope = globalThis as unknown as Record<string, unknown>;
const registered = new Map<string, ProcessorClass>();

scope["AudioWorkletProcessor"] = class {
  port = new FakePort();
};
scope["registerProcessor"] = (name: string, ctor: ProcessorClass) => {
  registered.set(name, ctor);
};
scope["sampleRate"] = 48_000;
scope["currentFrame"] = 0;

await import("../../src/worklet.ts");

export interface RigOptions {
  sampleRate?: number;
  seed?: bigint;
  layout?: Uint8Array;
  module?: WebAssembly.Module;
}

/** One processor, driven block by block the way the audio thread drives it. */
export class WorkletRig {
  readonly processor: Processor;
  readonly sampleRate: number;
  /** The `currentFrame` the next block will see. */
  frame = 0;
  private read = 0;
  private readonly chunks: Float32Array[] = [];

  constructor(options: RigOptions = {}) {
    this.sampleRate = options.sampleRate ?? 48_000;
    scope["sampleRate"] = this.sampleRate;
    scope["currentFrame"] = this.frame;
    const ctor = registered.get(WORKLET_NAME);
    if (!ctor) throw new Error(`worklet.ts did not register "${WORKLET_NAME}"`);
    this.processor = new ctor({
      processorOptions: { module: options.module ?? compile(), layout: options.layout ?? layoutBytes(), seed: options.seed ?? 0n },
    });
  }

  /** A message from the page, delivered between blocks. */
  send(message: ToWorklet): void {
    scope["sampleRate"] = this.sampleRate;
    scope["currentFrame"] = this.frame;
    this.processor.port.onmessage?.({ data: message });
  }

  /**
   * Runs one 128-frame quantum, handing the processor the one mono output the page's node has, as the browser does. Returns the messages
   * the processor posted since the last call to `take`. What it wrote to the output is kept: see `audio`.
   */
  block(): Posted[] {
    scope["sampleRate"] = this.sampleRate;
    scope["currentFrame"] = this.frame;
    const out = new Float32Array(RENDER_FRAMES);
    if (!this.processor.process([], [[out]])) throw new Error("process() returned false: the node would be dropped");
    this.chunks.push(out);
    this.frame += RENDER_FRAMES;
    return this.take();
  }

  /** Everything the processor has written to its output so far, one frame per frame since the rig started: frame `i` is `audio()[i]`. */
  audio(): Float32Array {
    const all = new Float32Array(this.chunks.length * RENDER_FRAMES);
    this.chunks.forEach((c, i) => all.set(c, i * RENDER_FRAMES));
    return all;
  }

  /** Messages posted since the last call. */
  take(): Posted[] {
    const all = this.processor.port.posted;
    const out = all.slice(this.read);
    this.read = all.length;
    return out;
  }
}

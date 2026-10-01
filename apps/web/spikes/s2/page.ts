// Spike S2's page. `run.ts` calls `window.s2.*`. The same module is run on the main thread (timed with
// `performance.now()`, which the worklet scope lacks) and in the worklet (timed in batches with `Date`).
import { Octoweb, layoutFromControls, type Exports } from "../../src/abi.ts";
import { play, type Programs } from "../program.ts";
import type { S2Reply, S2Request } from "./worklet.ts";

type Distributive<T> = T extends unknown ? Omit<T, "id"> : never;

export interface S2 {
  start(wasmUrl: string): Promise<Extract<S2Reply, { type: "ready" }> & { audioSampleRate: number; crossOriginIsolated: boolean; userAgent: string; cores: number; timerResolutionUs: number }>;
  hash(name: string, source: string): Promise<Extract<S2Reply, { type: "hash" }>>;
  program(doc: Programs, name: string): Promise<Extract<S2Reply, { type: "program" }>>;
  timingWorklet(doc: Programs, name: string, blocks: number, batch: number): Promise<Extract<S2Reply, { type: "timing" }>>;
  timingMain(doc: Programs, name: string, blocks: number): Promise<{ blocks: number; perBlockMs: number[]; allocsBefore: number | null; allocsAfter: number | null; events: number }>;
  programMain(doc: Programs, name: string): Promise<{ digest: string; events: number }>;
}

let context: AudioContext;
let node: AudioWorkletNode;
let module: WebAssembly.Module;
let layout: Uint8Array;
let numbers: [string, number][] = [];
let nextId = 1;
const pending = new Map<number, (r: S2Reply) => void>();

function ask<T extends S2Reply>(request: Distributive<S2Request>): Promise<T> {
  const id = nextId++;
  return new Promise((resolve, reject) => {
    pending.set(id, (r) => (r.type === "error" ? reject(new Error((r as { message: string }).message)) : resolve(r as T)));
    node.port.postMessage({ ...request, id });
  });
}

/** The smallest step `performance.now()` takes, in microseconds, over 200 steps. */
function timerResolutionUs(): number {
  let smallest = Infinity;
  for (let i = 0; i < 200; i++) {
    const a = performance.now();
    let b = a;
    while (b === a) b = performance.now();
    smallest = Math.min(smallest, b - a);
  }
  return smallest * 1000;
}

function mainEngine(): Octoweb {
  return new Octoweb(new WebAssembly.Instance(module, {}).exports as unknown as Exports);
}

const s2: S2 = {
  async start(wasmUrl) {
    const controls = (await (await fetch("/contracts/controls.json")).json()) as { controls: { n: number; id: string }[] };
    numbers = controls.controls.map((c) => [c.id, c.n]);
    layout = layoutFromControls(controls);
    module = await WebAssembly.compile(await (await fetch(wasmUrl)).arrayBuffer());
    context = new AudioContext({ sampleRate: 48_000, latencyHint: "interactive" });
    await context.audioWorklet.addModule("/dist/spikes/s2/worklet.js");
    node = new AudioWorkletNode(context, "octoweb-s2", { numberOfInputs: 0, numberOfOutputs: 1, processorOptions: { module } });
    node.connect(context.destination);
    const ready = await new Promise<Extract<S2Reply, { type: "ready" }>>((resolve) => {
      node.port.onmessage = (e: MessageEvent<S2Reply>) => {
        const m = e.data;
        if (m.type === "ready") resolve(m);
        else if ("id" in m) pending.get(m.id)?.(m);
      };
    });
    await context.resume();
    return {
      ...ready,
      audioSampleRate: context.sampleRate,
      crossOriginIsolated: self.crossOriginIsolated,
      userAgent: navigator.userAgent,
      cores: navigator.hardwareConcurrency,
      timerResolutionUs: timerResolutionUs(),
    };
  },
  hash: (name, source) => ask({ type: "hash", name, source: new TextEncoder().encode(source) }),
  program: (doc, name) => ask({ type: "program", layout, numbers, doc, name }),
  timingWorklet: (doc, name, blocks, batch) => ask({ type: "timing", layout, numbers, doc, name, blocks, batch }),
  async programMain(doc, name) {
    const program = doc.programs[name]!;
    const engine = mainEngine();
    engine.init(layout, doc.sample_rate, BigInt(program.seed));
    const played = play(engine, new Map(numbers), doc, program);
    return { digest: played.digest, events: played.events };
  },
  async timingMain(doc, name, blocks) {
    const program = doc.programs[name]!;
    const engine = mainEngine();
    engine.init(layout, doc.sample_rate, BigInt(program.seed));
    const perBlockMs: number[] = [];
    let last = 0;
    let allocsBefore: number | null = null;
    let allocsAfter: number | null = null;
    const counter = engine.x.octoweb_alloc_count;
    const played = play(engine, new Map(numbers), doc, program, {
      blocks,
      hash: false,
      onBlock: (b) => {
        const now = performance.now();
        if (b === 0) allocsBefore = counter ? counter() >>> 0 : null;
        else perBlockMs.push(now - last);
        if (b === blocks - 1) allocsAfter = counter ? counter() >>> 0 : null;
        last = performance.now();
      },
    });
    return { blocks, perBlockMs, allocsBefore, allocsAfter, events: played.events };
  },
};

(window as unknown as { s2: S2 }).s2 = s2;

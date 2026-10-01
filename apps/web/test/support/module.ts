// Loads the octoweb module in Node, the way the worklet does: compiled, instantiated with no imports.
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { Octoweb, layoutFromControls, type Exports } from "../../src/abi.ts";

const here = dirname(fileURLToPath(import.meta.url));
export const webRoot = join(here, "../..");
export const repoRoot = join(webRoot, "../..");

/** `just web-wasm` builds the shipped module here. `OCTOWEB_WASM` points a test at another build. */
export const shippedPath = process.env["OCTOWEB_WASM"] ?? join(webRoot, "dist/octoweb.wasm");
export const spikePath = process.env["OCTOWEB_SPIKE_WASM"] ?? join(webRoot, "dist/octoweb-spike.wasm");

const compiled = new Map<string, WebAssembly.Module>();

export function compile(path: string = shippedPath): WebAssembly.Module {
  let m = compiled.get(path);
  if (!m) {
    m = new WebAssembly.Module(readFileSync(path));
    compiled.set(path, m);
  }
  return m;
}

export function instantiate(path: string = shippedPath): Exports {
  return new WebAssembly.Instance(compile(path), {}).exports as unknown as Exports;
}

export interface Control {
  n: number;
  id: string;
}

export function controlsDoc(): { controls: Control[] } {
  return JSON.parse(readFileSync(join(repoRoot, "contracts/controls.json"), "utf8")) as { controls: Control[] };
}

export function layoutBytes(without: string[] = []): Uint8Array {
  const doc = controlsDoc();
  return layoutFromControls({ controls: doc.controls.filter((c) => !without.includes(c.id)) });
}

export function controlNumbers(): Map<string, number> {
  return new Map(controlsDoc().controls.map((c) => [c.id, c.n]));
}

/** An initialised module, each call with its own instance and therefore its own state. */
export function start(sampleRate = 48_000, seed = 0n, path: string = shippedPath): Octoweb {
  const engine = new Octoweb(instantiate(path));
  engine.init(layoutBytes(), sampleRate, seed);
  return engine;
}

export function matrixId(row: number, step: number): string {
  return `matrix.r${row}.c${step + 1}`;
}

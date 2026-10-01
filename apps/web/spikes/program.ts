// The programs in apps/web/engine/golden/programs.json, played through the module and hashed. Used by the
// Node test and by spike S2's AudioWorklet, and checked against the digests the native test pins.
// Runs in the AudioWorkletGlobalScope, so: no TextEncoder, no performance, no fetch.
import { EVENT_BYTES, KIND_DOWN, KIND_UP, type Octoweb } from "../src/abi.ts";

export interface Program {
  seed: number;
  bpm: number;
  blocks: number;
  steps: [number, number][];
  digest: string;
}

export interface Programs {
  sample_rate: number;
  frames_per_block: number;
  click_gap_ms: number;
  programs: Record<string, Program>;
}

export function matrixId(row: number, step: number): string {
  return `matrix.r${row}.c${step + 1}`;
}

const FNV_OFFSET = 0xcbf2_9ce4_8422_2325n;
const FNV_PRIME = 0x0000_0100_0000_01b3n;
const MASK = 0xffff_ffff_ffff_ffffn;

/** FNV-1a, 64 bits. */
export class Fnv64 {
  private h = FNV_OFFSET;
  update(bytes: Uint8Array): void {
    let h = this.h;
    for (const b of bytes) h = ((h ^ BigInt(b)) * FNV_PRIME) & MASK;
    this.h = h;
  }
  hex(): string {
    return `fnv1a64:${this.h.toString(16).padStart(16, "0")}`;
  }
}

export interface Played {
  digest: string;
  events: number;
  /** Blocks rendered. */
  blocks: number;
}

/**
 * Presses the program's steps, sets the tempo, starts the transport, and renders `blocks` blocks (the
 * program's own number by default), hashing every event with the number of the block it came from.
 * `onBlock` is called after each block with its index, for the spikes' timing.
 */
export function play(
  engine: Octoweb,
  numbers: ReadonlyMap<string, number>,
  doc: Programs,
  program: Program,
  options: { blocks?: number; hash?: boolean; onBlock?: (block: number, events: number) => void } = {},
): Played {
  const click = (row: number, step: number, t: number): void => {
    const id = matrixId(row, step);
    const n = numbers.get(id);
    if (n === undefined) throw new Error(`${id} is not in controls.json`);
    engine.input(t, KIND_DOWN, n);
    engine.input(t + 50, KIND_UP, n);
  };
  program.steps.forEach(([row, step], i) => click(row, step, i * doc.click_gap_ms));
  engine.setTempo(program.bpm);
  engine.transport(true);
  const hash = options.hash === false ? null : new Fnv64();
  const index = new Uint8Array(4);
  const view = new DataView(index.buffer);
  const blocks = options.blocks ?? program.blocks;
  let events = 0;
  for (let b = 0; b < blocks; b++) {
    const count = engine.render(doc.frames_per_block);
    if (hash && count > 0) {
      const bytes = engine.eventBytes(count);
      view.setUint32(0, b, true);
      for (let i = 0; i < count; i++) {
        hash.update(index);
        hash.update(bytes.subarray(i * EVENT_BYTES, (i + 1) * EVENT_BYTES));
      }
    }
    events += count;
    options.onBlock?.(b, count);
  }
  return { digest: hash ? hash.hex() : "", events, blocks };
}

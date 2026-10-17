// SPEC-0004 H-1: Plain is unchanged. With the voice `Plain` the monitor's output equals what it was before the voices existed,
// sample for sample, for any notes and any block sizes. The reference is `support/monitor-v0.ts`, a frozen copy of the monitor on
// `main` at c079f8f; the real monitor and the frozen one are fed the same events in lockstep and compared bit for bit.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { DRUM_CHANNEL, Monitor, type NoteEvent } from "../src/monitor.ts";
import { PLAIN } from "../src/voices/plain.ts";
import { DEFAULT_VOICE } from "../src/voices/registry.ts";
import { demoBlocks } from "./support/demo-blocks.ts";
import { webRoot } from "./support/module.ts";
import { MonitorV0 } from "./support/monitor-v0.ts";
import { Rng } from "./support/rng.ts";

const RATE = 48_000;
const BLOCK = 128;

/** The frozen reference must stay frozen: the hash of its text. If this fails someone edited it, and H-1 below proves nothing. */
const FROZEN_SHA256 = "3d47dbb96970bee73f11f1797684c8adcfa30e32dc747a7aa6234eb2ff24c073";

test("H-1: the reference monitor is the one on main at c079f8f, byte for byte as it was frozen", () => {
  const text = readFileSync(join(webRoot, "test/support/monitor-v0.ts"));
  assert.equal(createHash("sha256").update(text).digest("hex"), FROZEN_SHA256);
});

test("H-1: Plain is the default voice, and a new monitor has it", () => {
  assert.equal(DEFAULT_VOICE, PLAIN);
  assert.equal(new Monitor(RATE).voice, PLAIN);
});

/** Both monitors, driven the same way, and the first difference if there is one. */
function compare(blocks: readonly NoteEvent[][], act?: (b: number, a: Monitor, z: MonitorV0) => void): { samples: number; first: number } {
  const a = new Monitor(RATE);
  const z = new MonitorV0(RATE);
  a.on = true;
  z.on = true;
  let samples = 0;
  for (let b = 0; b < blocks.length; b++) {
    act?.(b, a, z);
    const x = new Float32Array(BLOCK);
    const y = new Float32Array(BLOCK);
    a.render(x, blocks[b]!);
    z.render(y, blocks[b]!);
    for (let i = 0; i < BLOCK; i++, samples++) if (!Object.is(x[i], y[i])) return { samples, first: samples };
  }
  return { samples, first: -1 };
}

test("H-1: the demo, 10 seconds of it as the engine plays it, is the same sample for sample", () => {
  const blocks = demoBlocks(10);
  const heard = blocks.flat().length;
  assert.ok(heard > 100, `${heard} events in the demo`);
  const r = compare(blocks);
  assert.equal(r.first, -1, `first differing sample ${r.first} of ${r.samples}`);
  assert.equal(r.samples, blocks.length * BLOCK);
});

/** A seeded storm of notes: every channel, velocities 0 to 127 (a note on at velocity 0 is a note off), many at once, drums among them. */
function storm(seed: number, blocks: number, density: number): NoteEvent[][] {
  const rng = new Rng(seed);
  return Array.from({ length: blocks }, () => {
    const events: NoteEvent[] = [];
    for (let k = rng.int(density + 1); k > 0; k--) {
      const channel = rng.int(4) === 0 ? DRUM_CHANNEL : 1 + rng.int(16);
      const note = 24 + rng.int(80);
      const at = rng.int(BLOCK);
      const kind = rng.int(3) === 0 ? 1 : 0;
      events.push({ kind, channel, d1: note, d2: kind === 1 ? 0 : rng.int(128), atSample: at });
    }
    return events.sort((p, q) => p.atSample - q.atSample);
  });
}

test("H-1: a storm of notes, more than the 24 voices at once, with drums, is the same sample for sample", () => {
  for (const [seed, density] of [[1, 2], [7, 6], [99, 14]] as const) {
    const r = compare(storm(seed, 3000, density));
    assert.equal(r.first, -1, `seed ${seed}: first differing sample ${r.first}`);
  }
});

test("H-1: switching the sound and the drums off and on in the middle of it changes nothing", () => {
  const r = compare(storm(5, 3000, 8), (b, a, z) => {
    if (b % 500 === 100) {
      a.on = false;
      z.on = false;
    }
    if (b % 500 === 130) {
      a.on = true;
      z.on = true;
    }
    if (b % 400 === 50) {
      a.drums = false;
      z.drums = false;
    }
    if (b % 400 === 90) {
      a.drums = true;
      z.drums = true;
    }
    if (b === 2000) {
      a.clear();
      z.clear();
    }
  });
  assert.equal(r.first, -1, `first differing sample ${r.first}`);
});

test("H-1: a note whose Note Off never comes is let go after ten seconds, at the same sample as before", () => {
  const blocks: NoteEvent[][] = Array.from({ length: Math.ceil((12 * RATE) / BLOCK) }, (_, b) => (b === 0 ? [{ kind: 0, channel: 1, d1: 57, d2: 100, atSample: 5 }] : []));
  const r = compare(blocks);
  assert.equal(r.first, -1);
  const m = new Monitor(RATE);
  m.on = true;
  const out = new Float32Array(blocks.length * BLOCK);
  blocks.forEach((e, b) => m.render(out.subarray(b * BLOCK, (b + 1) * BLOCK), e));
  let last = out.length - 1;
  while (last > 0 && out[last] === 0) last--;
  assert.ok(last > 10 * RATE && last < 10.2 * RATE, `the last sound is at ${last / RATE} s`);
});

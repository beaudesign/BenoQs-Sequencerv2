import assert from "node:assert/strict";
import { test } from "node:test";
import { analyse, match, spread, type Receipt, type Sent } from "../spikes/s1/analyse.ts";

const on = (n: number): number[] => [0x90, n, 100];
const off = (n: number): number[] => [0x80, n, 0];
const sent = (bytes: number[], audioTime: number, target: number, extra: Partial<Sent> = {}): Sent => ({ bytes, audioTime, target, marginMs: 40, hidden: false, ...extra });

test("spread: mean, sigma (population), percentiles by rank, and the empty case", () => {
  const s = spread([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
  assert.equal(s.n, 10);
  assert.equal(s.mean, 5.5);
  assert.ok(Math.abs(s.sigma - Math.sqrt(8.25)) < 1e-12);
  assert.equal(s.min, 1);
  assert.equal(s.max, 10);
  assert.equal(s.p50, 6);
  assert.equal(s.p99, 10);
  assert.deepEqual(spread([]), { n: 0, mean: 0, sigma: 0, min: 0, p50: 0, p99: 0, max: 0 });
});

test("match pairs each receipt with the earliest unmatched send of the same bytes, and counts what is left", () => {
  const s = [sent(on(60), 0, 100), sent(off(60), 0.1, 200), sent(on(60), 0.2, 300), sent(on(62), 0.3, 400)];
  const r: Receipt[] = [{ bytes: on(60), at: 101 }, { bytes: on(60), at: 301 }, { bytes: off(60), at: 201 }, { bytes: on(64), at: 5 }];
  const m = match(s, r);
  assert.deepEqual(m.pairs.map(([a, b]) => [a.target, b.at]), [[100, 101], [300, 301], [200, 201]]);
  assert.deepEqual(m.missing.map((x) => x.bytes), [on(62)]);
  assert.deepEqual(m.extra.map((x) => x.bytes), [on(64)]);
});

test("latency is arrival minus the timestamp given; interval error removes the constant latency", () => {
  // Events made 125 ms apart by the engine, heard with a constant 3 ms latency and one 2 ms late.
  const s = [sent(on(60), 1.0, 1000), sent(off(60), 1.125, 1125), sent(on(60), 1.25, 1250), sent(off(60), 1.375, 1375)];
  const r: Receipt[] = [{ bytes: on(60), at: 1003 }, { bytes: off(60), at: 1128 }, { bytes: on(60), at: 1255 }, { bytes: off(60), at: 1378 }];
  const a = analyse(s, r);
  assert.equal(a.matched, 4);
  assert.equal(a.missing, 0);
  assert.equal(a.extra, 0);
  assert.deepEqual([a.latencyMs.min, a.latencyMs.max], [3, 5]);
  assert.equal(a.latencyMs.mean, 3.5);
  assert.deepEqual(a.intervalErrorMs.n, 3);
  // intervals heard: 125, 127, 123; made: 125 each; errors 0, +2, -2
  assert.ok(Math.abs(a.intervalErrorMs.min + 2) < 1e-9 && Math.abs(a.intervalErrorMs.max - 2) < 1e-9);
  assert.ok(Math.abs(a.intervalErrorMs.mean) < 1e-9);
});

test("interval error follows the engine's order, not the order of arrival", () => {
  const s = [sent(on(60), 1.0, 1000), sent(on(61), 1.1, 1100), sent(on(62), 1.2, 1200)];
  const r: Receipt[] = [{ bytes: on(62), at: 1203 }, { bytes: on(60), at: 1003 }, { bytes: on(61), at: 1103 }];
  const a = analyse(s, r);
  assert.equal(a.intervalErrorMs.n, 2);
  assert.ok(Math.abs(a.intervalErrorMs.min) < 1e-9 && Math.abs(a.intervalErrorMs.max) < 1e-9, "heard out of order, still evenly spaced");
});

test("late events and margins are counted over what was sent, and a filter splits visible from hidden", () => {
  const s = [sent(on(60), 1, 1000, { marginMs: 35 }), sent(off(60), 1.1, 1100, { marginMs: -2, hidden: true }), sent(on(61), 1.2, 1200, { marginMs: 5, hidden: true })];
  const r: Receipt[] = [{ bytes: on(60), at: 1001 }, { bytes: off(60), at: 1101 }];
  const all = analyse(s, r);
  assert.equal(all.late, 1);
  assert.equal(all.marginMs.min, -2);
  assert.equal(all.missing, 1);
  const hidden = analyse(s, r, (x) => x.hidden);
  assert.equal(hidden.sent, 2);
  assert.equal(hidden.matched, 1);
  assert.equal(hidden.missing, 1);
  const visible = analyse(s, r, (x) => !x.hidden);
  assert.equal(visible.sent, 1);
  assert.equal(visible.matched, 1);
  assert.equal(visible.missing, 0);
});

import assert from "node:assert/strict";
import { test } from "node:test";
import { EVENT_BYTES } from "../src/abi.ts";
import { MidiScheduler } from "../src/midi-out.ts";
import { wireMidi, type EventSource } from "../src/wire.ts";

function source(sampleRate: number): EventSource {
  return { sampleRate, onEvents: null };
}

test("events from the worklet go to the scheduler with the worklet's sample rate, after the clock map is refreshed", () => {
  const calls: string[] = [];
  const time = {
    now: () => 0,
    toPage: (s: number) => {
      calls.push(`toPage ${s.toFixed(6)}`);
      return s * 1000;
    },
    refresh: () => calls.push("refresh"),
  };
  const sent: { data: number[]; ts: number | undefined }[] = [];
  const scheduler = new MidiScheduler(time, 30);
  scheduler.setOutput(1, { send: (data, ts) => sent.push({ data: Array.from(data), ts }) });
  const src = source(32_000);
  wireMidi(src, scheduler, time);
  const rec = new Uint8Array(EVENT_BYTES);
  rec.set([0, 1, 1, 60], 0);
  new DataView(rec.buffer).setUint16(4, 100, true);
  new DataView(rec.buffer).setUint32(8, 320, true); // 10 ms at 32 kHz
  src.onEvents?.(32_000, rec); // one second in
  assert.deepEqual(calls, ["refresh", "toPage 1.010000"]);
  assert.deepEqual(sent.map((x) => x.data), [[0x90, 60, 100]]);
  assert.ok(Math.abs((sent[0]?.ts ?? 0) - 1040) < 1e-6, `stamped ${sent[0]?.ts}`);
});

test("a time map without refresh() works too", () => {
  const time = { now: () => 0, toPage: (s: number) => s * 1000 };
  const scheduler = new MidiScheduler(time, 30);
  const sent: unknown[] = [];
  scheduler.setOutput(1, { send: (d) => sent.push(d) });
  const src = source(48_000);
  wireMidi(src, scheduler, time);
  const rec = new Uint8Array(EVENT_BYTES);
  rec.set([1, 1, 1, 60], 0);
  src.onEvents?.(0, rec);
  assert.equal(sent.length, 1);
});

import assert from "node:assert/strict";
import { test } from "node:test";
import { EVENT_BYTES } from "../src/abi.ts";
import { ContextTimeMap, MidiScheduler, ORDER_WINDOW_MS, midiBytes, type MidiOutputLike, type SentInfo, type TimeMap } from "../src/midi-out.ts";

/** A clock map with the page time of an audio time given by `toPage`, and `now` set by the test. */
class FakeTime implements TimeMap {
  nowMs = 0;
  offsetMs = 0;
  now(): number {
    return this.nowMs;
  }
  toPage(seconds: number): number {
    return seconds * 1000 + this.offsetMs;
  }
}

class RecordingOutput implements MidiOutputLike {
  sent: { data: number[]; timestamp: number | undefined }[] = [];
  clears = 0;
  send(data: ArrayLike<number>, timestamp?: number): void {
    this.sent.push({ data: Array.from(data), timestamp });
  }
  clear(): void {
    this.clears++;
  }
}

function record(kind: number, port: number, channel: number, d1: number, d2: number, at: number): Uint8Array {
  const out = new Uint8Array(EVENT_BYTES);
  const v = new DataView(out.buffer);
  out.set([kind, port, channel, d1], 0);
  v.setUint16(4, d2, true);
  v.setUint32(8, at, true);
  return out;
}

function concat(...parts: Uint8Array[]): Uint8Array {
  const out = new Uint8Array(parts.length * EVENT_BYTES);
  parts.forEach((p, i) => out.set(p, i * EVENT_BYTES));
  return out;
}

test("MIDI bytes: Note On, Note Off, control change, pitch bend, channel pressure", () => {
  assert.deepEqual(midiBytes(0, 1, 60, 100), [0x90, 60, 100]);
  assert.deepEqual(midiBytes(0, 16, 127, 127), [0x9f, 127, 127]);
  assert.deepEqual(midiBytes(1, 3, 61, 0), [0x82, 61, 0]);
  assert.deepEqual(midiBytes(2, 10, 7, 64), [0xb9, 7, 64]);
  assert.deepEqual(midiBytes(3, 1, 0, 8192), [0xe0, 0x00, 0x40]);
  assert.deepEqual(midiBytes(3, 2, 0, 0), [0xe1, 0, 0]);
  assert.deepEqual(midiBytes(3, 1, 0, 16383), [0xe0, 0x7f, 0x7f]);
  assert.deepEqual(midiBytes(3, 1, 0, 0x1234), [0xe0, 0x34, 0x24]);
  assert.deepEqual(midiBytes(4, 5, 90, 0), [0xd4, 90]);
});

test("MIDI bytes: what is not valid MIDI gives null", () => {
  assert.equal(midiBytes(0, 1, 60, 0), null, "a Note On with velocity 0 is a Note Off in MIDI; the engine never sends one");
  assert.equal(midiBytes(0, 1, 60, 128), null);
  assert.equal(midiBytes(0, 0, 60, 100), null, "channel 0");
  assert.equal(midiBytes(0, 17, 60, 100), null, "channel 17");
  assert.equal(midiBytes(0, 1, 128, 100), null, "key 128");
  assert.equal(midiBytes(2, 1, 7, 128), null);
  assert.equal(midiBytes(3, 1, 0, 16384), null);
  assert.equal(midiBytes(9, 1, 0, 0), null, "an unknown kind");
});

test("an event with time in hand is sent with a timestamp: its due time on the page's clock plus the lookahead", () => {
  const t = new FakeTime();
  t.nowMs = 1000;
  t.offsetMs = 500;
  const out = new RecordingOutput();
  const s = new MidiScheduler(t, 30);
  s.setOutput(1, out);
  // frame 96000 at 48 kHz is 2 s; 64 samples more is 1.333 ms. Due at page time 2000 + 1.333 + 500; sent at 30 ms after.
  s.schedule(96_000, 48_000, record(0, 1, 1, 60, 100, 64));
  assert.equal(out.sent.length, 1);
  assert.deepEqual(out.sent[0]?.data, [0x90, 60, 100]);
  assert.ok(Math.abs((out.sent[0]?.timestamp ?? 0) - (2000 + 64 / 48 + 500 + 30)) < 1e-9);
  assert.equal(s.stats.sent, 1);
  assert.equal(s.stats.late, 0);
});

test("an event that is already late is sent at once, never dropped, and counted with how late", () => {
  const t = new FakeTime();
  t.nowMs = 10_000; // far past anything due
  const out = new RecordingOutput();
  const s = new MidiScheduler(t, 30);
  s.setOutput(1, out);
  const seen: SentInfo[] = [];
  s.onSend = (i) => seen.push(i);
  s.schedule(0, 48_000, concat(record(0, 1, 1, 60, 100, 0), record(1, 1, 1, 60, 0, 48)));
  assert.equal(out.sent.length, 2);
  assert.equal(out.sent[0]?.timestamp, undefined, "no timestamp means now");
  assert.equal(s.stats.late, 2);
  assert.equal(s.stats.sent, 2);
  assert.ok(s.stats.lateMaxMs > 9_900 && s.stats.lateMaxMs < 10_000);
  assert.ok(seen.every((i) => i.timestamp === null && i.marginMs < 0));
  assert.deepEqual(seen.map((i) => i.audioTime), [0, 48 / 48_000], "the engine's own time of each event, in audio-context seconds");
});

test("each event goes to the output of its port; a port with no output is counted and the rest still go", () => {
  const t = new FakeTime();
  const a = new RecordingOutput();
  const b = new RecordingOutput();
  const s = new MidiScheduler(t, 30);
  s.setOutput(1, a);
  s.setOutput(2, b);
  s.schedule(0, 48_000, concat(record(0, 1, 1, 60, 100, 0), record(0, 2, 2, 61, 100, 0), record(0, 3, 1, 62, 100, 0)));
  assert.equal(a.sent.length, 1);
  assert.equal(b.sent.length, 1);
  assert.deepEqual(b.sent[0]?.data, [0x91, 61, 100]);
  assert.equal(s.stats.unrouted, 1);
  s.setOutput(2, null);
  s.schedule(0, 48_000, record(0, 2, 2, 61, 100, 0));
  assert.equal(s.stats.unrouted, 2);
});

test("a record that is not valid MIDI is counted and not sent", () => {
  const t = new FakeTime();
  const out = new RecordingOutput();
  const s = new MidiScheduler(t, 30);
  s.setOutput(1, out);
  s.schedule(0, 48_000, concat(record(0, 1, 1, 60, 0, 0), record(0, 1, 0, 60, 90, 0), record(0, 1, 1, 60, 90, 0)));
  assert.equal(out.sent.length, 1);
  assert.equal(s.stats.invalid, 2);
  assert.equal(s.stats.sent, 1);
});

test("the scheduler never calls clear(), even on an output that has it", () => {
  // Chromium has no clear(), and where it exists it drops Note Offs the engine already counted as sent
  // (p3-plan finding F-P3-2). The property test in stop-property.test.ts runs with and without it.
  const t = new FakeTime();
  const out = new RecordingOutput();
  const s = new MidiScheduler(t, 30);
  s.setOutput(1, out);
  s.schedule(0, 48_000, concat(record(0, 1, 1, 60, 100, 0), record(1, 1, 1, 60, 0, 64)));
  s.setOutput(1, out);
  assert.equal(out.clears, 0);
});

test("timestamps never go backwards on one output, even when the clock map shifts between batches", () => {
  // The map's offset jitters from one batch to the next. A flush Note Off one sample after a Note On
  // must not be stamped earlier than it: the output sorts by timestamp and the note would stay on.
  const t = new FakeTime();
  t.nowMs = 0;
  const out = new RecordingOutput();
  const s = new MidiScheduler(t, 30);
  s.setOutput(1, out);
  t.offsetMs = 0;
  s.schedule(0, 48_000, record(0, 1, 1, 60, 100, 127)); // the last sample of block 0
  t.offsetMs = -2; // the next batch's map is 2 ms earlier
  s.schedule(128, 48_000, record(1, 1, 1, 60, 0, 0)); // the first sample of block 1
  const [on, off] = out.sent;
  assert.ok(on?.timestamp !== undefined && off?.timestamp !== undefined);
  assert.ok(off.timestamp >= on.timestamp, `Note Off at ${off.timestamp} is before its Note On at ${on.timestamp}`);
  assert.equal(s.stats.raised, 1, "the raise is counted");
  assert.ok(s.stats.raisedMaxMs > 1.9 && s.stats.raisedMaxMs < 2.1, `raised by ${s.stats.raisedMaxMs}`);
});

test("a step back larger than ORDER_WINDOW_MS is a clock jump, and the new time is believed", () => {
  const t = new FakeTime();
  const out = new RecordingOutput();
  const s = new MidiScheduler(t, 30);
  s.setOutput(1, out);
  t.nowMs = 0;
  s.schedule(0, 48_000, record(0, 1, 1, 60, 100, 0));
  t.offsetMs = -(ORDER_WINDOW_MS + 5);
  t.nowMs = -1000; // nothing is late
  s.schedule(128, 48_000, record(1, 1, 1, 60, 0, 0));
  const [on, off] = out.sent;
  assert.ok(on?.timestamp !== undefined && off?.timestamp !== undefined);
  assert.ok(off.timestamp < on.timestamp, "not held back");
  assert.equal(s.stats.raised, 0);
});

test("a step back just inside ORDER_WINDOW_MS is held back", () => {
  const t = new FakeTime();
  const out = new RecordingOutput();
  const s = new MidiScheduler(t, 30);
  s.setOutput(1, out);
  t.nowMs = -1000;
  s.schedule(0, 48_000, record(0, 1, 1, 60, 100, 0));
  t.offsetMs = -(ORDER_WINDOW_MS - 5);
  s.schedule(128, 48_000, record(1, 1, 1, 60, 0, 0));
  const [on, off] = out.sent;
  assert.equal(off?.timestamp, on?.timestamp);
});

test("a late event does not overtake an earlier one that was queued for the future", () => {
  const t = new FakeTime();
  const out = new RecordingOutput();
  const s = new MidiScheduler(t, 30);
  s.setOutput(1, out);
  t.nowMs = 0;
  s.schedule(0, 48_000, record(0, 1, 1, 60, 100, 100)); // stamped about 32 ms
  t.nowMs = 20;
  t.offsetMs = -40; // the map now says the next event was due long ago
  s.schedule(128, 48_000, record(1, 1, 1, 60, 0, 0));
  const [on, off] = out.sent;
  assert.ok(on?.timestamp !== undefined);
  assert.ok(off?.timestamp !== undefined, "it must carry the earlier event's timestamp, not go at once");
  assert.ok(off.timestamp >= on.timestamp);
});

test("ordering is per output: another output's timestamps do not hold this one back", () => {
  const t = new FakeTime();
  const a = new RecordingOutput();
  const b = new RecordingOutput();
  const s = new MidiScheduler(t, 30);
  s.setOutput(1, a);
  s.setOutput(2, b);
  t.nowMs = 0;
  s.schedule(48_000, 48_000, record(0, 1, 1, 60, 100, 0)); // port 1, due at 1 s
  s.schedule(0, 48_000, record(0, 2, 1, 60, 100, 0)); // port 2, due at 0
  assert.ok((b.sent[0]?.timestamp ?? 1e9) < 100, "port 2 keeps its own, earlier, timestamp");
});

test("ContextTimeMap: audio time to page time from the latest pair of clocks", () => {
  let pair = { contextTime: 10, performanceTime: 5_000 };
  const clock = { now: () => 5_500 };
  const m = new ContextTimeMap({ getOutputTimestamp: () => pair }, clock);
  assert.equal(m.now(), 5_500);
  assert.equal(m.toPage(10), 5_000, "converts with a pair it took by itself the first time");
  assert.equal(m.toPage(12), 7_000);
  pair = { contextTime: 20, performanceTime: 15_003 }; // the clocks drifted 3 ms
  assert.equal(m.toPage(20), 15_000, "no refresh yet: the old offset (-5000 ms) still applies");
  m.refresh();
  assert.equal(m.toPage(20), 15_003);
});

test("ContextTimeMap with alpha below 1 smooths the offset over the pairs it has taken", () => {
  const offsets = [0, 4, 0, 4, 0, 4, 0, 4, 0, 4, 0, 4];
  let i = 0;
  const raw = new ContextTimeMap({ getOutputTimestamp: () => ({ contextTime: 0, performanceTime: offsets[i] ?? 0 }) }, { now: () => 0 }, 1);
  const smooth = new ContextTimeMap({ getOutputTimestamp: () => ({ contextTime: 0, performanceTime: offsets[i] ?? 0 }) }, { now: () => 0 }, 0.25);
  let rawSpread = 0;
  let smoothSpread = 0;
  let prevRaw = 0;
  let prevSmooth = 0;
  for (i = 0; i < offsets.length; i++) {
    raw.refresh();
    smooth.refresh();
    if (i > 4) {
      rawSpread = Math.max(rawSpread, Math.abs(raw.toPage(0) - prevRaw));
      smoothSpread = Math.max(smoothSpread, Math.abs(smooth.toPage(0) - prevSmooth));
    }
    prevRaw = raw.toPage(0);
    prevSmooth = smooth.toPage(0);
  }
  assert.equal(rawSpread, 4);
  assert.ok(smoothSpread < 2, `smoothed spread ${smoothSpread}`);
});

test("ContextTimeMap ignores a timestamp the browser could not give", () => {
  const m = new ContextTimeMap({ getOutputTimestamp: () => ({}) }, { now: () => 0 });
  assert.equal(m.toPage(1), 1000, "with no pair yet the offset is 0");
  m.refresh();
  assert.equal(m.toPage(2), 2000);
});

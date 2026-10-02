// The input path (SPEC-0002 P4, plan observable M6): what a MIDI input hands the page is decoded and counted, and nothing consumes it yet.
import assert from "node:assert/strict";
import { test } from "node:test";
import { InputPath, decodeMessage, describeCount, describeInput, type InMessage, type MidiInputLike } from "../src/midi-in.ts";

class FakeInput implements MidiInputLike {
  readonly id: string;
  readonly name: string;
  onmidimessage: ((event: { data: Uint8Array | null; timeStamp: number }) => unknown) | null = null;
  constructor(id: string, name = id) {
    this.id = id;
    this.name = name;
  }
  emit(data: number[] | null, timeStamp: number): void {
    this.onmidimessage?.({ data: data ? Uint8Array.from(data) : null, timeStamp });
  }
}

test("a Note On, a Note Off and a Note On with velocity 0 (which MIDI defines as a Note Off)", () => {
  assert.deepEqual(decodeMessage([0x90, 60, 100]), { kind: "noteOn", channel: 1, note: 60, velocity: 100 });
  assert.deepEqual(decodeMessage([0x9f, 127, 1]), { kind: "noteOn", channel: 16, note: 127, velocity: 1 });
  assert.deepEqual(decodeMessage([0x82, 61, 40]), { kind: "noteOff", channel: 3, note: 61, velocity: 40 });
  assert.deepEqual(decodeMessage([0x90, 60, 0]), { kind: "noteOff", channel: 1, note: 60, velocity: 0 });
});

test("a controller, a pitch bend (14 bits, low byte first), channel pressure and a program change", () => {
  assert.deepEqual(decodeMessage([0xb9, 7, 64]), { kind: "controller", channel: 10, controller: 7, value: 64 });
  assert.deepEqual(decodeMessage([0xe0, 0x00, 0x40]), { kind: "bend", channel: 1, value: 8192 });
  assert.deepEqual(decodeMessage([0xe1, 0x7f, 0x7f]), { kind: "bend", channel: 2, value: 16383 });
  assert.deepEqual(decodeMessage([0xe0, 0x34, 0x24]), { kind: "bend", channel: 1, value: 0x1234 });
  assert.deepEqual(decodeMessage([0xd4, 90]), { kind: "pressure", channel: 5, value: 90 });
  assert.deepEqual(decodeMessage([0xc0, 0]), { kind: "program", channel: 1, program: 0 });
  assert.deepEqual(decodeMessage([0xcf, 127]), { kind: "program", channel: 16, program: 127 });
});

test("the four real-time messages the clock needs, by name", () => {
  assert.deepEqual(decodeMessage([0xf8]), { kind: "realtime", message: "clock" });
  assert.deepEqual(decodeMessage([0xfa]), { kind: "realtime", message: "start" });
  assert.deepEqual(decodeMessage([0xfb]), { kind: "realtime", message: "continue" });
  assert.deepEqual(decodeMessage([0xfc]), { kind: "realtime", message: "stop" });
});

test("a message that is valid MIDI but not one the sequencer uses is ignored, with its status byte, and is not malformed", () => {
  assert.deepEqual(decodeMessage([0xfe]), { kind: "ignored", status: 0xfe }, "active sensing, which hardware sends every 300 ms");
  assert.deepEqual(decodeMessage([0xff]), { kind: "ignored", status: 0xff });
  assert.deepEqual(decodeMessage([0xf2, 0, 0]), { kind: "ignored", status: 0xf2 }, "song position pointer: the manual is silent on it (findings section 6, item 15)");
  assert.deepEqual(decodeMessage([0xa0, 60, 50]), { kind: "ignored", status: 0xa0 }, "polyphonic key pressure");
  assert.deepEqual(decodeMessage([0xf0, 1, 2, 0xf7]), { kind: "ignored", status: 0xf0 }, "system exclusive is not asked for");
});

test("what cannot be a MIDI message is malformed: empty, no status byte, short, long, or a data byte above 127", () => {
  for (const bad of [[], [60, 100], [0x90, 60], [0x90], [0x90, 60, 100, 5], [0x90, 200, 100], [0x90, 60, 128], [0xe0, 0, 200], [0xd0], [0xd0, 1, 2], [0xc0], [0xf8, 0]]) {
    assert.deepEqual(decodeMessage(bad), { kind: "malformed" }, JSON.stringify(bad));
  }
  assert.deepEqual(decodeMessage(null), { kind: "malformed" }, "the event's data can be null");
});

test("a chosen input's messages reach the page's input path with the browser's timestamp, in order, and are counted", () => {
  const input = new FakeInput("a");
  const path = new InputPath();
  const got: [InMessage, number][] = [];
  path.onMessage = (m, t) => got.push([m, t]);
  path.select(input);
  input.emit([0x90, 60, 100], 1000);
  input.emit([0xf8], 1020.5);
  input.emit([0xb0, 1, 5], 1021);
  assert.deepEqual(got, [
    [{ kind: "noteOn", channel: 1, note: 60, velocity: 100 }, 1000],
    [{ kind: "realtime", message: "clock" }, 1020.5],
    [{ kind: "controller", channel: 1, controller: 1, value: 5 }, 1021],
  ]);
  assert.equal(path.stats.received, 3);
  assert.equal(path.stats.decoded, 3);
  assert.equal(path.stats.byKind.noteOn, 1);
  assert.equal(path.stats.byKind.realtime, 1);
  assert.equal(path.stats.byKind.controller, 1);
  assert.equal(path.stats.realtime.clock, 1);
  assert.equal(path.stats.lastTimeStamp, 1021);
});

test("with no consumer set the messages are still counted: nothing consumes them yet", () => {
  const input = new FakeInput("a");
  const path = new InputPath();
  assert.equal(path.onMessage, null);
  path.select(input);
  input.emit([0x90, 60, 100], 1);
  input.emit([0x80, 60, 0], 2);
  assert.equal(path.stats.decoded, 2);
  assert.equal(path.stats.byKind.noteOn, 1);
  assert.equal(path.stats.byKind.noteOff, 1);
});

test("ignored and malformed messages are counted apart and are not handed on", () => {
  const input = new FakeInput("a");
  const path = new InputPath();
  const got: InMessage[] = [];
  path.onMessage = (m) => got.push(m);
  path.select(input);
  input.emit([0xfe], 1);
  input.emit([0x90, 60], 2);
  input.emit(null, 3);
  input.emit([0xfa], 4);
  assert.deepEqual(got, [{ kind: "realtime", message: "start" }]);
  assert.equal(path.stats.received, 4);
  assert.equal(path.stats.ignored, 1);
  assert.equal(path.stats.malformed, 2);
  assert.equal(path.stats.decoded, 1);
  assert.equal(path.stats.realtime.start, 1);
});

test("choosing another input lets go of the first, and choosing none lets go of both", () => {
  const a = new FakeInput("a");
  const b = new FakeInput("b");
  const path = new InputPath();
  path.select(a);
  assert.notEqual(a.onmidimessage, null);
  path.select(b);
  assert.equal(a.onmidimessage, null, "the first input no longer reports to the page");
  assert.notEqual(b.onmidimessage, null);
  b.emit([0x90, 60, 100], 1);
  assert.equal(path.stats.received, 1);
  path.select(null);
  assert.equal(b.onmidimessage, null);
  assert.equal(path.selected, null);
});

test("choosing an input starts the counts again, so each count belongs to one port", () => {
  const a = new FakeInput("a");
  const b = new FakeInput("b");
  const path = new InputPath();
  path.select(a);
  a.emit([0xf8], 1);
  path.select(b);
  assert.equal(path.stats.received, 0);
  assert.equal(path.stats.realtime.clock, 0);
  assert.equal(path.stats.lastTimeStamp, null);
  assert.equal(path.selected?.id, "b");
});

test("a timestamp that goes backwards on one input is counted, because the follower will rely on them going forward", () => {
  const input = new FakeInput("a");
  const path = new InputPath();
  path.select(input);
  input.emit([0xf8], 100);
  input.emit([0xf8], 120);
  input.emit([0xf8], 120); // equal is not backwards
  input.emit([0xf8], 119.5);
  input.emit([0xf8], 140);
  assert.equal(path.stats.backwards, 1);
  assert.equal(path.stats.received, 5);
});

test("a consumer that throws does not stop the path counting the next message", () => {
  const input = new FakeInput("a");
  const path = new InputPath();
  path.onMessage = () => {
    throw new Error("consumer bug");
  };
  path.select(input);
  assert.throws(() => input.emit([0x90, 60, 100], 1), /consumer bug/);
  assert.equal(path.stats.decoded, 1, "counted before it was handed on");
  path.onMessage = null;
  input.emit([0x80, 60, 0], 2);
  assert.equal(path.stats.decoded, 2);
});

test("the input sentences: where it listens, what it has sent, and the browser's sentence when none is chosen", () => {
  assert.equal(describeInput(null, "Choose a MIDI input to listen to."), "Choose a MIDI input to listen to.");
  assert.equal(describeInput("Launchkey", "x"), "Listening to Launchkey. Nothing uses its messages yet.");
  const input = new FakeInput("a");
  const path = new InputPath();
  path.select(input);
  assert.equal(describeCount(path.stats), "No messages yet.");
  input.emit([0x90, 60, 100], 1);
  assert.equal(describeCount(path.stats), "1 message: 1 used, 0 ignored, 0 malformed.");
  input.emit([0xfe], 2);
  input.emit(null, 3);
  assert.equal(describeCount(path.stats), "3 messages: 1 used, 1 ignored, 1 malformed.");
});

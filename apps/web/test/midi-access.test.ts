// What the page says and does when Web MIDI is missing, refused, empty or present (A7).
import assert from "node:assert/strict";
import { test } from "node:test";
import { openMidi, type MidiAccessLike } from "../src/midi-access.ts";

class FakeOutput {
  sent: unknown[] = [];
  readonly id: string;
  readonly name: string;
  constructor(id: string, name: string) {
    this.id = id;
    this.name = name;
  }
  send(data: number[], timestamp?: number): void {
    this.sent.push([data, timestamp]);
  }
}

function access(outputs: FakeOutput[]): MidiAccessLike & { outputs: Map<string, FakeOutput>; fire(): void } {
  const a = {
    outputs: new Map(outputs.map((o) => [o.id, o])),
    onstatechange: null as (() => void) | null,
    fire() {
      a.onstatechange?.();
    },
  };
  return a;
}

test("a browser without requestMIDIAccess says so in a sentence, and offers no output", async () => {
  const midi = await openMidi({});
  assert.equal(midi.status, "unavailable");
  assert.match(midi.message, /no Web MIDI/);
  assert.deepEqual(midi.outputs(), []);
});

test("a refused request says it was refused and how to fix it", async () => {
  const midi = await openMidi({ requestMIDIAccess: () => Promise.reject(new DOMException("denied", "SecurityError")) });
  assert.equal(midi.status, "denied");
  assert.match(midi.message, /refused/);
  assert.match(midi.message, /Start/);
});

test("a request that throws instead of rejecting is the same as a refusal", async () => {
  const midi = await openMidi({
    requestMIDIAccess: () => {
      throw new Error("not allowed in this context");
    },
  });
  assert.equal(midi.status, "denied");
});

test("with no output attached it says none was found", async () => {
  const midi = await openMidi({ requestMIDIAccess: () => Promise.resolve(access([])) });
  assert.equal(midi.status, "ready");
  assert.match(midi.message, /No MIDI output/);
  assert.deepEqual(midi.outputs(), []);
});

test("outputs are listed by name, in the order the browser gives them, with the real port to send to", async () => {
  const a = new FakeOutput("1", "IAC Driver Bus 1");
  const b = new FakeOutput("2", "loopMIDI Port");
  const midi = await openMidi({ requestMIDIAccess: () => Promise.resolve(access([a, b])) });
  assert.match(midi.message, /Choose/);
  assert.deepEqual(midi.outputs().map((o) => [o.id, o.name]), [["1", "IAC Driver Bus 1"], ["2", "loopMIDI Port"]]);
  midi.outputs()[1]!.output.send([0x90, 60, 100], 5);
  assert.deepEqual(b.sent, [[[0x90, 60, 100], 5]]);
});

test("a port that is plugged in or removed later changes the list and tells the page", async () => {
  const a = new FakeOutput("1", "One");
  const acc = access([a]);
  const midi = await openMidi({ requestMIDIAccess: () => Promise.resolve(acc) });
  let told = 0;
  midi.onChange = () => told++;
  acc.outputs.set("2", new FakeOutput("2", "Two"));
  acc.fire();
  assert.equal(told, 1);
  assert.deepEqual(midi.outputs().map((o) => o.name), ["One", "Two"]);
  acc.outputs.delete("1");
  acc.fire();
  assert.deepEqual(midi.outputs().map((o) => o.name), ["Two"]);
});

test("the request asks for no system exclusive access", async () => {
  let asked: unknown;
  await openMidi({
    requestMIDIAccess: (options) => {
      asked = options;
      return Promise.resolve(access([]));
    },
  });
  assert.deepEqual(asked, { sysex: false });
});

test("every message is sentence case and fits the 68-character measure when wrapped (no all caps, no arrows)", async () => {
  const messages = [
    (await openMidi({})).message,
    (await openMidi({ requestMIDIAccess: () => Promise.reject(new Error("x")) })).message,
    (await openMidi({ requestMIDIAccess: () => Promise.resolve(access([])) })).message,
    (await openMidi({ requestMIDIAccess: () => Promise.resolve(access([new FakeOutput("1", "A")])) })).message,
  ];
  for (const m of messages) {
    assert.match(m, /^[A-Z]/, m);
    assert.ok(!/^[A-Z]{3,}\s/.test(m.replace(/^MIDI /, "")), m);
    assert.ok(!/[→·]/.test(m), m);
    assert.ok(!/\b(?!MIDI\b)[A-Z]{4,}\b/.test(m), `all caps in: ${m}`);
    for (const word of m.split(" ")) assert.ok(word.length <= 68);
  }
});

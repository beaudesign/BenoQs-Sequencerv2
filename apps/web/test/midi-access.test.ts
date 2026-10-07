// What the page says and does when Web MIDI is missing, refused, empty or present (A7).
import assert from "node:assert/strict";
import { test } from "node:test";
import { describeOutputs, openMidi, type MidiAccessLike } from "../src/midi-access.ts";

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

class FakeInput {
  readonly id: string;
  readonly name: string;
  onmidimessage: ((event: { data: Uint8Array | null; timeStamp: number }) => unknown) | null = null;
  constructor(id: string, name: string) {
    this.id = id;
    this.name = name;
  }
}

function access(outputs: FakeOutput[], inputs: FakeInput[] = []): MidiAccessLike & { outputs: Map<string, FakeOutput>; inputs: Map<string, FakeInput>; fire(): void } {
  const a = {
    outputs: new Map(outputs.map((o) => [o.id, o])),
    inputs: new Map(inputs.map((i) => [i.id, i])),
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
  assert.match(midi.message, /Play/);
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
    (await openMidi({ requestMIDIAccess: () => Promise.reject(new DOMException("Platform dependent initialization failed.", "InvalidStateError")) })).message,
    (await openMidi({ requestMIDIAccess: () => Promise.resolve(access([])) })).inputMessage,
    (await openMidi({ requestMIDIAccess: () => Promise.resolve(access([], [new FakeInput("i", "A")])) })).inputMessage,
  ];
  for (const m of messages) {
    assert.match(m, /^[A-Z]/, m);
    assert.ok(!/^[A-Z]{3,}\s/.test(m.replace(/^MIDI /, "")), m);
    assert.ok(!/[→·]/.test(m), m);
    assert.ok(!/\b(?!MIDI\b)[A-Z]{4,}\b/.test(m), `all caps in: ${m}`);
    for (const word of m.split(" ")) assert.ok(word.length <= 68);
  }
});

test("inputs are listed by name, in the browser's order, with the real port to listen to", async () => {
  const a = new FakeInput("i1", "IAC Driver Bus 1");
  const b = new FakeInput("i2", "Launchkey");
  const midi = await openMidi({ requestMIDIAccess: () => Promise.resolve(access([], [a, b])) });
  assert.deepEqual(midi.inputs().map((i) => [i.id, i.name]), [["i1", "IAC Driver Bus 1"], ["i2", "Launchkey"]]);
  assert.equal(midi.inputs()[1]!.input, b);
  assert.match(midi.inputMessage, /Choose a MIDI input/);
});

test("with no input attached it says none was found, and that this does not stop the outputs", async () => {
  const midi = await openMidi({ requestMIDIAccess: () => Promise.resolve(access([new FakeOutput("1", "A")])) });
  assert.deepEqual(midi.inputs(), []);
  assert.match(midi.inputMessage, /No MIDI input/);
  assert.match(midi.message, /Choose a MIDI output/, "the output sentence is its own");
});

test("an input plugged in or removed later changes the list and tells the page", async () => {
  const acc = access([], [new FakeInput("i1", "One")]);
  const midi = await openMidi({ requestMIDIAccess: () => Promise.resolve(acc) });
  let told = 0;
  midi.onChange = () => told++;
  acc.inputs.set("i2", new FakeInput("i2", "Two"));
  acc.fire();
  assert.equal(told, 1);
  assert.deepEqual(midi.inputs().map((i) => i.name), ["One", "Two"]);
  acc.inputs.delete("i1");
  acc.inputs.delete("i2");
  acc.fire();
  assert.deepEqual(midi.inputs(), []);
  assert.match(midi.inputMessage, /No MIDI input/);
});

test("without Web MIDI, or when it is refused, there is no input and the one sentence covers both directions", async () => {
  for (const midi of [await openMidi({}), await openMidi({ requestMIDIAccess: () => Promise.reject(new DOMException("denied", "NotAllowedError")) })]) {
    assert.deepEqual(midi.inputs(), []);
    assert.equal(midi.inputMessage, "");
  }
});

test("a browser that has Web MIDI but cannot start it on this system says so, and does not call it a refusal", async () => {
  // What Chromium 141 gives in a container with no MIDI backend, measured for P4b: InvalidStateError, "Platform dependent initialization failed."
  const midi = await openMidi({ requestMIDIAccess: () => Promise.reject(new DOMException("Platform dependent initialization failed.", "InvalidStateError")) });
  assert.equal(midi.status, "failed");
  assert.match(midi.message, /could not start/);
  assert.match(midi.message, /Platform dependent initialization failed/, "the browser's own reason is in the sentence");
  assert.doesNotMatch(midi.message, /refused|Allow it/);
  assert.deepEqual(midi.outputs(), []);
  assert.deepEqual(midi.inputs(), []);
});

test("a refusal by name (NotAllowedError, SecurityError) says refused and how to fix it", async () => {
  for (const name of ["NotAllowedError", "SecurityError"]) {
    const midi = await openMidi({ requestMIDIAccess: () => Promise.reject(new DOMException("x", name)) });
    assert.equal(midi.status, "denied", name);
    assert.match(midi.message, /refused/, name);
  }
});

test("the output sentence says where each port sends, says so when one device serves both, and falls back to the browser's sentence for none", () => {
  const a = { id: "1", name: "Synth" };
  const b = { id: "2", name: "Drums" };
  assert.equal(describeOutputs({ 1: null, 2: null }, "Choose a MIDI output."), "Choose a MIDI output.");
  assert.equal(describeOutputs({ 1: a, 2: null }, "x"), "Port 1 sends to Synth.");
  assert.equal(describeOutputs({ 1: null, 2: b }, "x"), "Port 2 sends to Drums.");
  assert.equal(describeOutputs({ 1: a, 2: b }, "x"), "Port 1 sends to Synth. Port 2 sends to Drums.");
  assert.equal(describeOutputs({ 1: a, 2: a }, "x"), "Port 1 and port 2 send to Synth.");
  assert.equal(describeOutputs({ 1: a, 2: { id: "3", name: "Synth" } }, "x"), "Port 1 sends to Synth. Port 2 sends to Synth.", "two devices with one name are two devices: the id decides");
});

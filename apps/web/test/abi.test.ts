// The TypeScript wrapper against the real module, in Node. The native twin of these tests is
// apps/web/engine/tests/abi.rs; the two keep the ABI honest from both sides.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import {
  ABI_VERSION,
  EVENT_BYTES,
  KIND_DOWN,
  KIND_REALTIME,
  KIND_UP,
  LED_COUNT,
  Octoweb,
  RENDER_FRAMES,
  OctowebError,
  TRACK_ATTR,
  asciiToString,
  decodeEvents,
  decodeLed,
  layoutFromControls,
  type Exports,
} from "../src/abi.ts";
import { compile, controlNumbers, controlsDoc, instantiate, layoutBytes, matrixId, repoRoot, start } from "./support/module.ts";

const n = controlNumbers();
const ctl = (id: string): number => {
  const v = n.get(id);
  assert.ok(v !== undefined, `${id} is in controls.json`);
  return v;
};

function led(e: Octoweb, id: string): number {
  const v = e.ledBytes()[ctl(id)];
  assert.ok(v !== undefined);
  return v;
}

function click(e: Octoweb, id: string, t: number): void {
  assert.equal(e.input(t, KIND_DOWN, ctl(id)), 0);
  assert.equal(e.input(t + 50, KIND_UP, ctl(id)), 0);
}

test("the module reports the ABI number the page speaks", () => {
  assert.equal(ABI_VERSION, 2, "P4c: event kind 5, octoweb_set_clock and octoweb_tick_position (ADR-0009)");
  assert.equal(instantiate().octoweb_abi() >>> 0, ABI_VERSION);
});

test("the wrapper refuses a module that speaks another ABI", () => {
  const real = instantiate();
  const other: Exports = { ...real, octoweb_abi: () => 1 };
  assert.throws(
    () => new Octoweb(other),
    (e: unknown) => e instanceof OctowebError && /abi\/1/.test(e.message) && /speaks 2/.test(e.message),
  );
});

test("the module has no imports, so the worklet can instantiate it synchronously", () => {
  assert.deepEqual(WebAssembly.Module.imports(compile()), []);
});

test("the shipped module exports exactly the documented functions, and none of the feature-gated ones", () => {
  const names = WebAssembly.Module.exports(compile())
    .map((e) => e.name)
    .filter((x) => x.startsWith("octoweb_"))
    .sort();
  assert.deepEqual(names, [
    "octoweb_abi",
    "octoweb_dropped_intents",
    "octoweb_events",
    "octoweb_init",
    "octoweb_input",
    "octoweb_leds",
    "octoweb_message_len",
    "octoweb_playheads",
    "octoweb_refresh_leds",
    "octoweb_render",
    "octoweb_reset",
    "octoweb_scratch",
    "octoweb_set_clock",
    "octoweb_set_tempo",
    "octoweb_set_track",
    "octoweb_status",
    "octoweb_tick_position",
    "octoweb_transport",
  ]);
});

test("init builds the controller from the layout text of controls.json", () => {
  const e = new Octoweb(instantiate());
  e.init(layoutBytes(), 48_000, 0n);
  assert.equal(e.running(), false);
  assert.equal(e.ledBytes().length, 512, "one byte per control number, 0 to 511 (ABI.md)");
  assert.equal(LED_COUNT, 512);
});

test("the AudioWorklet quantum is 128 frames", () => {
  assert.equal(RENDER_FRAMES, 128);
});

test("the 64-bit seed crosses as two 32-bit halves, low word first", () => {
  const real = instantiate();
  let seen: number[] = [];
  const spy: Exports = {
    ...real,
    octoweb_init: (sampleRate, lo, hi, len) => {
      seen = [lo >>> 0, hi >>> 0];
      return real.octoweb_init(sampleRate, lo, hi, len);
    },
  };
  new Octoweb(spy).init(layoutBytes(), 48_000, 0x0123_4567_89ab_cdefn);
  assert.deepEqual(seen, [0x89ab_cdef, 0x0123_4567]);
});

test("init refuses a layout without a control the controller needs, and names it", () => {
  const e = new Octoweb(instantiate());
  assert.throws(
    () => e.init(layoutBytes(["mode.edit"]), 48_000, 0n),
    (err: unknown) => err instanceof OctowebError && err.code === 4 && /mode\.edit/.test(err.message),
  );
});

test("init refuses text that is not a layout, with code 3", () => {
  const e = new Octoweb(instantiate());
  const text = new TextEncoder().encode("this is not a layout\n");
  assert.throws(
    () => e.init(text, 48_000, 0n),
    (err: unknown) => err instanceof OctowebError && err.code === 3,
  );
});

test("init refuses a sample rate the engine cannot use, with code 5", () => {
  for (const rate of [0, -1, Number.NaN]) {
    const e = new Octoweb(instantiate());
    assert.throws(
      () => e.init(layoutBytes(), rate, 0n),
      (err: unknown) => err instanceof OctowebError && err.code === 5,
      `rate ${rate}`,
    );
  }
});

test("a call before init does nothing and says it was not initialised", () => {
  const e = new Octoweb(instantiate());
  assert.equal(e.input(0, KIND_DOWN, ctl("mode.edit")), 1);
  assert.throws(() => e.transport(true), (err: unknown) => err instanceof OctowebError && err.code === 1);
  assert.equal(e.render(128), 0);
});

test("a matrix press lights the step green and a second press puts it out", () => {
  const e = start();
  e.refreshLeds();
  assert.equal(led(e, matrixId(0, 0)) & 3, 0);
  click(e, matrixId(0, 0), 0);
  assert.equal(e.refreshLeds(), true);
  assert.equal(led(e, matrixId(0, 0)), 2, "green, steady");
  click(e, matrixId(0, 0), 500);
  e.refreshLeds();
  assert.equal(led(e, matrixId(0, 0)), 0);
});

test("refresh says whether anything changed", () => {
  const e = start();
  e.refreshLeds();
  assert.equal(e.refreshLeds(), false);
  click(e, matrixId(0, 0), 0);
  assert.equal(e.refreshLeds(), true);
  assert.equal(e.refreshLeds(), false);
});

test("a control number the controller does not use is ignored and is not an error", () => {
  const e = start();
  assert.equal(e.input(0, KIND_DOWN, 500), 0);
  assert.equal(e.input(1, 9, ctl("mode.edit")), 2, "an unknown kind is code 2");
});

test("views are made fresh, so they survive the memory growing", () => {
  const e = start();
  click(e, matrixId(1, 1), 0);
  e.x.memory.grow(1);
  e.refreshLeds();
  assert.equal(led(e, matrixId(1, 1)), 2);
  assert.equal(e.playheadBytes().length, 10);
});

test("the transport message starts and stops, and the status word says so", () => {
  const e = start();
  e.transport(true);
  assert.equal(e.running(), true);
  e.transport(false);
  assert.equal(e.running(), false);
});

test("a tempo the engine cannot use is refused", () => {
  const e = start();
  for (const bpm of [0, -5, 1000, Number.NaN, Number.POSITIVE_INFINITY]) {
    assert.throws(() => e.setTempo(bpm), (err: unknown) => err instanceof OctowebError && err.code === 5, `bpm ${bpm}`);
  }
  e.setTempo(133);
});

test("the first Note On is a real MIDI Note On, with a velocity of at least 1", () => {
  const e = start();
  click(e, matrixId(0, 0), 0);
  e.transport(true);
  let first = null;
  for (let b = 0; b < 400 && !first; b++) {
    const count = e.render(128);
    first = decodeEvents(e.eventBytes(count)).find((ev) => ev.kind === 0) ?? null;
  }
  assert.ok(first, "a Note On within about a second");
  assert.ok(first.port >= 1 && first.port <= 2, `port ${first.port}`);
  assert.ok(first.channel >= 1 && first.channel <= 16, `channel ${first.channel}`);
  assert.ok(first.d1 < 128);
  assert.ok(first.d2 >= 1 && first.d2 < 128, `velocity ${first.d2}`);
  assert.ok(first.atSample < 128);
});

test("decodeEvents reads every field of the 12-byte record, little endian, unsigned", () => {
  const bytes = new Uint8Array(EVENT_BYTES * 2);
  const v = new DataView(bytes.buffer);
  // kind 3 (pitch bend), port 2, channel 16, d1 0, d2 0x2001 (a value above 255), at 0xfffffff0
  bytes.set([3, 2, 16, 0], 0);
  v.setUint16(4, 0x2001, true);
  v.setUint32(8, 0xffff_fff0, true);
  bytes.set([0, 1, 1, 60], EVENT_BYTES);
  v.setUint16(EVENT_BYTES + 4, 100, true);
  v.setUint32(EVENT_BYTES + 8, 7, true);
  assert.deepEqual(decodeEvents(bytes), [
    { kind: 3, port: 2, channel: 16, d1: 0, d2: 0x2001, atSample: 0xffff_fff0 },
    { kind: 0, port: 1, channel: 1, d1: 60, d2: 100, atSample: 7 },
  ]);
  assert.deepEqual(decodeEvents(bytes.subarray(0, EVENT_BYTES - 1)), [], "a partial record is not read");
});

test("decodeEvents reads a view that does not start at the beginning of its buffer", () => {
  const backing = new Uint8Array(40);
  backing.set([0, 1, 5, 64, 90, 0, 0, 0, 3, 0, 0, 0], 20);
  const events = decodeEvents(backing.subarray(20, 32));
  assert.deepEqual(events, [{ kind: 0, port: 1, channel: 5, d1: 64, d2: 90, atSample: 3 }]);
});

test("asciiToString reads bytes as ASCII", () => {
  assert.equal(asciiToString(new Uint8Array([109, 111, 100, 101, 46, 101])), "mode.e");
  assert.equal(asciiToString(new Uint8Array()), "");
});

test("layoutFromControls writes one `n id` line per control, in file order", () => {
  const doc = controlsDoc();
  const text = asciiToString(layoutFromControls(doc));
  const lines = text.split("\n");
  assert.equal(lines.pop(), "", "the text ends with a newline");
  assert.equal(lines.length, doc.controls.length);
  assert.equal(lines[0], `${doc.controls[0]?.n} ${doc.controls[0]?.id}`);
  assert.equal(lines.at(-1), `${doc.controls.at(-1)?.n} ${doc.controls.at(-1)?.id}`);
});

test("reset drops the module, and it can start again", () => {
  const e = start();
  click(e, matrixId(0, 0), 0);
  e.reset();
  assert.equal(e.input(0, KIND_DOWN, ctl("mode.edit")), 1);
  e.init(layoutBytes(), 48_000, 0n);
  e.refreshLeds();
  assert.equal(led(e, matrixId(0, 0)) & 3, 0, "the old pattern is gone");
});

test("two instances with the same seed play the same bytes", () => {
  const run = (): string => {
    const e = start(48_000, 7n);
    for (const [row, step] of [[0, 0], [0, 4], [0, 8], [3, 2]] as const) click(e, matrixId(row, step), row * 100 + step * 10);
    e.setTempo(133);
    e.transport(true);
    let out = "";
    for (let b = 0; b < 600; b++) {
      const count = e.render(128);
      out += `${b}:${Buffer.from(e.eventBytes(count)).toString("hex")};`;
    }
    return out;
  };
  const a = run();
  assert.equal(a, run());
  assert.ok(/:[0-9a-f]{24}/.test(a), "the run should contain events");
});

test("an LED byte is decoded into its colour and its phase, and an undefined phase reads as steady", () => {
  const colours = ["off", "red", "green", "orange"] as const;
  const phases = ["steady", "flash", "shine"] as const;
  for (let c = 0; c < 4; c++) {
    for (let p = 0; p < 3; p++) assert.deepEqual(decodeLed(c | (p << 2)), { colour: colours[c], phase: phases[p] });
  }
  assert.deepEqual(decodeLed(0b1110), { colour: "green", phase: "steady" }, "phase 3 is not defined");
  assert.deepEqual(decodeLed(0), { colour: "off", phase: "steady" });
});

test("set_track: a track set to a channel above 16 plays on port 2, through the wrapper, with the attribute numbered as ABI.md says", () => {
  const e = start();
  click(e, matrixId(0, 0), 0);
  click(e, matrixId(1, 0), 100);
  e.setTrack(0, TRACK_ATTR.midiChannel, 17);
  e.setTrack(1, TRACK_ATTR.midiChannel, 3);
  e.setTempo(240);
  e.transport(true);
  const routes = new Set<string>();
  for (let b = 0; b < 600; b++) {
    const n = e.render(RENDER_FRAMES);
    for (const ev of decodeEvents(e.eventBytes(n))) if (ev.kind === 0) routes.add(`${ev.port}:${ev.channel}`);
  }
  assert.deepEqual([...routes].sort(), ["1:3", "2:1"]);
});

test("set_track: a track or an attribute that does not exist is code 5, and before init it is code 1", () => {
  const e = start();
  for (const [track, attr] of [[10, 8], [-1, 8], [0, 15], [0, -1], [0, 99]] as const) {
    assert.throws(() => e.setTrack(track, attr, 3), (err: unknown) => err instanceof OctowebError && err.code === 5, `track ${track} attribute ${attr}`);
  }
  const bare = new Octoweb(instantiate());
  assert.throws(() => bare.setTrack(0, TRACK_ATTR.midiChannel, 17), (err: unknown) => err instanceof OctowebError && err.code === 1);
});

test("TRACK_ATTR names the attribute numbers ABI.md gives, and ABI.md gives the engine's list in its order", () => {
  const doc = readFileSync(`${repoRoot}/apps/web/engine/ABI.md`, "utf8");
  const row = doc.split("\n").find((l) => l.startsWith("| `octoweb_set_track("));
  assert.ok(row, "ABI.md has a row for octoweb_set_track");
  const list = /engine's list: (.*?)\. A track/.exec(row)?.[1];
  assert.ok(list, "the row gives the list of attributes");
  const listed = [...list.matchAll(/(\d+) (\w+)/g)].map((m) => [Number(m[1]), m[2]] as const);
  assert.deepEqual(
    listed.map(([, name]) => name),
    ["Pitch", "Velocity", "LengthFactor", "StartFactor", "DirectionRaw", "Rotation", "Amount", "Groove", "MidiChannel", "Muted", "Soloed", "Paused", "RecordArmed", "IsFeeder", "IsListener"],
  );
  listed.forEach(([n], i) => assert.equal(n, i));
  assert.equal(TRACK_ATTR.midiChannel, 8);
  assert.equal(listed[TRACK_ATTR.midiChannel]?.[1], "MidiChannel");
});

test("KIND_REALTIME is the record kind ABI.md gives the clock", () => {
  assert.equal(KIND_REALTIME, 5);
});

test("set_clock: with the clock on, Play makes a kind 5 record for Start and then Clock, with the status byte in d1 and no port or channel", () => {
  const e = start();
  e.setClock(true);
  e.transport(true);
  const events = decodeEvents(e.eventBytes(e.render(RENDER_FRAMES)));
  assert.deepEqual(
    events.map((x) => [x.kind, x.port, x.channel, x.d1, x.d2, x.atSample]),
    [
      [5, 0, 0, 0xfa, 0, 0],
      [5, 0, 0, 0xf8, 0, 0],
    ],
    "Start, then the first pulse, both on the first sample",
  );
});

test("set_clock: the clock is off until it is asked for, and off again when it is turned off", () => {
  const e = start();
  e.transport(true);
  let kinds = new Set<number>();
  for (let b = 0; b < 100; b++) for (const x of decodeEvents(e.eventBytes(e.render(RENDER_FRAMES)))) kinds.add(x.kind);
  assert.ok(!kinds.has(5), "no kind 5 record by default");
  e.setClock(true);
  kinds = new Set();
  for (let b = 0; b < 100; b++) for (const x of decodeEvents(e.eventBytes(e.render(RENDER_FRAMES)))) kinds.add(x.kind);
  assert.ok(kinds.has(5), "pulses once it is on");
  e.setClock(false);
  e.render(RENDER_FRAMES); // what was waiting is dropped
  kinds = new Set();
  for (let b = 0; b < 100; b++) for (const x of decodeEvents(e.eventBytes(e.render(RENDER_FRAMES)))) kinds.add(x.kind);
  assert.ok(!kinds.has(5), "and none after it is turned off");
});

test("set_clock: pulses are 1,000 samples apart at 120 BPM and 48 kHz", () => {
  const e = start();
  e.setClock(true);
  e.transport(true);
  const at: number[] = [];
  for (let b = 0; b < 400; b++) {
    for (const x of decodeEvents(e.eventBytes(e.render(RENDER_FRAMES)))) if (x.kind === KIND_REALTIME && x.d1 === 0xf8) at.push(b * RENDER_FRAMES + x.atSample);
  }
  assert.ok(at.length >= 50, `${at.length} pulses`);
  for (let i = 1; i < at.length; i++) assert.equal(at[i]! - at[i - 1]!, 1000);
});

test("set_clock before init is code 1, and tick_position before init is 0", () => {
  const e = new Octoweb(instantiate());
  assert.throws(() => e.setClock(true), (x: unknown) => x instanceof OctowebError && x.code === 1);
  assert.equal(e.tickPosition(), 0);
});

test("tick_position: 0 before play, then the audio's place on the tick grid: 125 samples a tick at 120 BPM and 48 kHz", () => {
  const e = start();
  assert.equal(e.tickPosition(), 0);
  e.transport(true);
  let last = 0;
  for (let b = 1; b <= 50; b++) {
    e.render(RENDER_FRAMES);
    const p = e.tickPosition();
    assert.ok(Math.abs(p - (b * RENDER_FRAMES) / 125) < 1e-6, `after ${b} blocks the position is ${p}`);
    assert.ok(p >= last);
    last = p;
  }
  e.transport(false);
  e.render(RENDER_FRAMES);
  const held = e.tickPosition();
  for (let b = 0; b < 10; b++) e.render(RENDER_FRAMES);
  assert.equal(e.tickPosition(), held, "stopped: it stands still");
});

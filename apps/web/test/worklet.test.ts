// src/worklet.ts, the real file, in a stand-in for the worklet scope (test/support/worklet-scope.ts).
import assert from "node:assert/strict";
import { test } from "node:test";
import { EVENT_BYTES, KIND_DOWN, KIND_UP, LED_COUNT, RENDER_FRAMES, decodeEvents } from "../src/abi.ts";
import { PANEL_EVERY_BLOCKS, type FromWorklet } from "../src/protocol.ts";
import { controlNumbers, layoutBytes, matrixId } from "./support/module.ts";
import { WorkletRig, type Posted } from "./support/worklet-scope.ts";

const n = controlNumbers();
const ctl = (id: string): number => {
  const v = n.get(id);
  assert.ok(v !== undefined, id);
  return v;
};

const types = (posted: Posted[]): string[] => posted.map((p) => p.message.type);
function only<T extends FromWorklet["type"]>(posted: Posted[], type: T): Extract<FromWorklet, { type: T }>[] {
  return posted.map((p) => p.message).filter((m): m is Extract<FromWorklet, { type: T }> => m.type === type);
}

function press(rig: WorkletRig, id: string, t = 0): void {
  rig.send({ type: "input", nowMs: t, kind: KIND_DOWN, control: ctl(id), detents: 0 });
  rig.send({ type: "input", nowMs: t + 50, kind: KIND_UP, control: ctl(id), detents: 0 });
}

test("the constructor starts the engine and says ready, with the sample rate the context runs at", () => {
  const rig = new WorkletRig({ sampleRate: 44_100 });
  const first = rig.take();
  assert.deepEqual(first.map((p) => p.message), [{ type: "ready", abi: 2, sampleRate: 44_100 }]);
});

test("a layout the controller refuses stops the constructor, so the page hears of it as a processor error", () => {
  assert.throws(() => new WorkletRig({ layout: layoutBytes(["transport.stop"]) }), /code 4.*transport\.stop/);
});

test("a block with nothing to say posts nothing but the panel, and only every PANEL_EVERY_BLOCKS blocks", () => {
  const rig = new WorkletRig();
  rig.take();
  const seen: string[][] = [];
  for (let b = 0; b < PANEL_EVERY_BLOCKS * 2; b++) seen.push(types(rig.block()));
  assert.deepEqual(seen.flat(), ["panel", "panel"]);
  assert.equal(seen.findIndex((s) => s.length > 0), PANEL_EVERY_BLOCKS - 1);
});

test("the panel is sent about every 21 ms: often enough for an LED, not so often it floods the page", () => {
  const ms = (PANEL_EVERY_BLOCKS * RENDER_FRAMES * 1000) / 48_000;
  assert.ok(ms >= 10 && ms <= 33, `${ms} ms between panel messages`);
});

test("a press posts the LED frame at once, and the same frame is not posted twice", () => {
  const rig = new WorkletRig();
  rig.take();
  press(rig, matrixId(0, 0));
  const panels = only(rig.take(), "panel");
  const withLeds = panels.filter((p) => p.leds !== null);
  assert.equal(withLeds.length, 1, "one of the two messages (down, up) changed the LEDs");
  const leds = new Uint8Array(withLeds[0]!.leds!);
  assert.equal(leds.length, LED_COUNT);
  assert.equal(leds[ctl(matrixId(0, 0))], 2, "green, steady");
  rig.send({ type: "input", nowMs: 500, kind: 9, control: 1, detents: 0 }); // an unknown kind changes nothing
  assert.ok(only(rig.take(), "panel").every((p) => p.leds === null));
});

test("playheads are ten bytes and the status word follows the transport", () => {
  const rig = new WorkletRig();
  rig.take();
  rig.send({ type: "transport", play: true });
  const p = only(rig.take(), "panel").at(-1)!;
  assert.equal(new Uint8Array(p.playheads).length, 10);
  assert.equal(p.status & 1, 1);
});

test("events come with the frame of the block they were rendered in, and whole 12-byte records", () => {
  const rig = new WorkletRig();
  rig.take();
  press(rig, matrixId(0, 0));
  rig.send({ type: "transport", play: true });
  rig.take();
  const seen: { frame: number; count: number; block: number }[] = [];
  for (let b = 0; b < 400; b++) {
    for (const e of only(rig.block(), "events")) {
      assert.equal(e.bytes.byteLength % EVENT_BYTES, 0);
      seen.push({ frame: e.frame, count: e.bytes.byteLength / EVENT_BYTES, block: b });
      for (const ev of decodeEvents(new Uint8Array(e.bytes))) assert.ok(ev.atSample < RENDER_FRAMES, "an offset inside the block");
    }
  }
  assert.ok(seen.length >= 1, "a note within about a second");
  for (const s of seen) assert.equal(s.frame, s.block * RENDER_FRAMES, "the frame is the block's first frame");
});

test("events and the LED frame are transferred, not copied", () => {
  const rig = new WorkletRig();
  rig.take();
  press(rig, matrixId(0, 0));
  rig.send({ type: "transport", play: true });
  const posted = rig.take();
  for (const p of posted.filter((x) => x.message.type === "panel")) {
    const m = p.message as Extract<FromWorklet, { type: "panel" }>;
    assert.ok(p.transfer.includes(m.playheads));
    if (m.leds) assert.ok(p.transfer.includes(m.leds));
  }
  for (let b = 0; b < 400; b++) {
    for (const p of rig.block().filter((x) => x.message.type === "events")) {
      assert.deepEqual(p.transfer, [(p.message as Extract<FromWorklet, { type: "events" }>).bytes]);
    }
  }
});

test("Stop from the page: the flush ends every note, and no Note On follows", () => {
  const rig = new WorkletRig();
  rig.take();
  for (const s of [0, 2, 4, 6, 8, 10, 12, 14]) press(rig, matrixId(0, s), s * 100);
  rig.send({ type: "transport", play: true });
  const held = new Set<string>();
  const apply = (m: FromWorklet): void => {
    if (m.type !== "events") return;
    for (const e of decodeEvents(new Uint8Array(m.bytes))) {
      const key = `${e.port}/${e.channel}/${e.d1}`;
      if (e.kind === 0) held.add(key);
      if (e.kind === 1) held.delete(key);
    }
  };
  let sounding = false;
  for (let b = 0; b < 1000 && !sounding; b++) {
    for (const p of rig.block()) apply(p.message);
    sounding = held.size > 0;
  }
  assert.ok(sounding, "stop with a note sounding");
  rig.send({ type: "transport", play: false });
  const stopped = only(rig.take(), "panel").at(-1)!;
  assert.equal(stopped.status & 1, 0, "the panel message that follows says the transport stopped");
  let noteOnsAfter = 0;
  for (let b = 0; b < 400; b++) {
    for (const p of rig.block()) {
      apply(p.message);
      if (p.message.type === "events") noteOnsAfter += decodeEvents(new Uint8Array(p.message.bytes)).filter((e) => e.kind === 0).length;
    }
  }
  assert.equal(held.size, 0, `still held: ${[...held]}`);
  assert.equal(noteOnsAfter, 0);
});

test("the panel's Stop key stops the transport, and the next panel message says so", () => {
  const rig = new WorkletRig();
  press(rig, matrixId(0, 0));
  rig.send({ type: "transport", play: true });
  assert.equal(only(rig.take(), "panel").at(-1)!.status & 1, 1);
  press(rig, "transport.stop", 100);
  assert.equal(only(rig.take(), "panel").at(-1)!.status & 1, 0);
});

test("a tempo the engine refuses is posted as an error and the processor keeps running", () => {
  const rig = new WorkletRig();
  rig.take();
  rig.send({ type: "tempo", bpm: Number.NaN });
  const errors = only(rig.take(), "error");
  assert.equal(errors.length, 1);
  assert.match(errors[0]!.message, /code 5/);
  rig.send({ type: "tempo", bpm: 150 });
  assert.equal(only(rig.take(), "error").length, 0);
  rig.block();
});

test("reset drops the pattern; the processor then reports errors rather than crashing", () => {
  const rig = new WorkletRig();
  press(rig, matrixId(0, 0));
  rig.send({ type: "reset" });
  rig.take();
  const out = rig.block();
  assert.deepEqual(out.filter((p) => p.message.type === "events"), [], "a module that is not initialised renders nothing");
  rig.send({ type: "transport", play: true });
  assert.equal(only(rig.take(), "error").length, 1);
});

test("two processors with the same seed post the same events", () => {
  const run = (): string[] => {
    const rig = new WorkletRig({ seed: 7n });
    for (const s of [0, 4, 8, 12]) press(rig, matrixId(0, s), s);
    rig.send({ type: "tempo", bpm: 133 });
    rig.send({ type: "transport", play: true });
    rig.take();
    const out: string[] = [];
    for (let b = 0; b < 600; b++) for (const e of only(rig.block(), "events")) out.push(`${e.frame}:${Buffer.from(e.bytes).toString("hex")}`);
    return out;
  };
  const a = run();
  assert.ok(a.length >= 4);
  assert.deepEqual(a, run());
});

test("a track message sets one attribute of one track: a track set to channel 17 then plays on port 2", () => {
  const rig = new WorkletRig();
  rig.take();
  press(rig, matrixId(0, 0), 0);
  rig.send({ type: "track", track: 0, attr: 8, value: 17 });
  rig.send({ type: "tempo", bpm: 240 });
  rig.send({ type: "transport", play: true });
  const routes = new Set<string>();
  for (let b = 0; b < 600; b++) {
    for (const m of only(rig.block(), "events")) for (const e of decodeEvents(new Uint8Array(m.bytes))) if (e.kind === 0) routes.add(`${e.port}:${e.channel}`);
  }
  assert.deepEqual([...routes], ["2:1"]);
});

test("a track message the engine refuses is reported to the page as an error and does not stop the worklet", () => {
  const rig = new WorkletRig();
  rig.take();
  rig.send({ type: "track", track: 10, attr: 8, value: 17 });
  const posted = rig.take();
  assert.match(only(posted, "error")[0]?.message ?? "", /set track.*code 5/);
  rig.send({ type: "tempo", bpm: 120 });
  assert.deepEqual(only(rig.take(), "error"), []);
});

test("a clock message turns the engine's clock on and off: Start and pulses reach the page in the events message, kind 5", () => {
  const rig = new WorkletRig();
  rig.take();
  const clockRecords = (blocks: number) => {
    const out: { kind: number; d1: number; port: number; channel: number }[] = [];
    for (let b = 0; b < blocks; b++) for (const m of only(rig.block(), "events")) for (const e of decodeEvents(new Uint8Array(m.bytes))) if (e.kind === 5) out.push(e);
    return out;
  };
  rig.send({ type: "transport", play: true });
  assert.deepEqual(clockRecords(100), [], "off by default");
  rig.send({ type: "clock", master: true });
  const on = clockRecords(100);
  assert.equal(on[0]?.d1, 0xfb, "turned on while running: the engine says where it is, and it is not at tick 0, so Continue");
  assert.ok(on.slice(1).every((e) => e.d1 === 0xf8 && e.port === 0 && e.channel === 0), "then pulses");
  assert.ok(on.length > 10);
  rig.send({ type: "clock", master: false });
  rig.block();
  assert.deepEqual(clockRecords(100), [], "off again");
});

test("a clock message after reset is reported as an error, like any call the module refuses, and does not stop the worklet", () => {
  const rig = new WorkletRig();
  rig.take();
  rig.send({ type: "reset" });
  rig.take();
  rig.send({ type: "clock", master: true });
  assert.match(only(rig.take(), "error")[0]?.message ?? "", /set clock.*code 1/);
});

// src/worklet.ts, the real file, in a stand-in for the worklet scope (test/support/worklet-scope.ts).
import assert from "node:assert/strict";
import { test } from "node:test";
import { EVENT_BYTES, KIND_DOWN, KIND_UP, LED_COUNT, RENDER_FRAMES, decodeEvents } from "../src/abi.ts";
import { QUARTER_TICKS } from "../src/click.ts";
import { routeFromWorklet, type PanelFrame, type WorkletSink } from "../src/host.ts";
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
  const on = clockRecords(400); // a pulse every 1000 frames at 120 BPM (a quarter note of 24000 frames, 24 pulses): about 51 in 400 blocks of 128
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

// ----- the engine's position, for the clock follower (SPEC-0002 P4d, ADR-0009 decision 4) -----

const TICKS_PER_SECOND_120 = (2 * QUARTER_TICKS); // 120 BPM is two quarter notes a second; the engine counts QUARTER_TICKS (48) to the quarter note, so 96 a second (D0: 192 to the whole note, as the manual does)

test("the panel message carries the engine's tick position and the frame it holds at: the end of the block, or the frame of a message between blocks", () => {
  const rig = new WorkletRig();
  rig.take();
  for (let b = 0; b < PANEL_EVERY_BLOCKS * 3; b++) {
    for (const p of only(rig.block(), "panel")) {
      assert.equal(typeof p.position, "number");
      assert.equal(p.positionFrame, rig.frame, "block b was rendered from frame b * 128, and the position is where it ended: the next block's first frame");
    }
  }
  rig.send({ type: "tempo", bpm: 120 });
  const handled = only(rig.take(), "panel").at(-1)!;
  assert.equal(handled.positionFrame, rig.frame, "a message between blocks reads the position as of the next block's first frame");
});

test("the position is 0 and stands still while the transport is stopped, and the frame goes on", () => {
  const rig = new WorkletRig();
  rig.take();
  const seen: { position: number; positionFrame: number }[] = [];
  for (let b = 0; b < PANEL_EVERY_BLOCKS * 4; b++) for (const p of only(rig.block(), "panel")) seen.push(p);
  assert.ok(seen.length >= 4);
  for (const s of seen) assert.equal(s.position, 0);
  assert.ok(seen.every((s, i) => i === 0 || s.positionFrame > seen[i - 1]!.positionFrame));
});

test("the position runs at the tick rate while playing: 96 ticks a second at 120 BPM (two quarter notes a second), to within half a percent", () => {
  const rig = new WorkletRig();
  rig.take();
  rig.send({ type: "tempo", bpm: 120 });
  rig.send({ type: "transport", play: true });
  rig.take();
  const seen: { position: number; positionFrame: number }[] = [];
  for (let b = 0; b < PANEL_EVERY_BLOCKS * 40; b++) for (const p of only(rig.block(), "panel")) seen.push(p);
  const first = seen[2]!;
  const last = seen[seen.length - 1]!;
  const seconds = (last.positionFrame - first.positionFrame) / 48_000;
  const ticks = last.position - first.position;
  assert.ok(Math.abs(ticks / seconds / TICKS_PER_SECOND_120 - 1) < 0.005, `${(ticks / seconds).toFixed(2)} ticks a second over ${seconds.toFixed(2)} s`);
  for (let i = 1; i < seen.length; i++) assert.ok(seen[i]!.position >= seen[i - 1]!.position, "it never goes backwards");
});

test("a Stop leaves the position where it was, and Play goes on from there", () => {
  const rig = new WorkletRig();
  rig.take();
  rig.send({ type: "transport", play: true });
  for (let b = 0; b < 200; b++) rig.block();
  rig.send({ type: "transport", play: false });
  const atStop = only(rig.take(), "panel").at(-1)!;
  const later: number[] = [];
  for (let b = 0; b < PANEL_EVERY_BLOCKS * 3; b++) for (const p of only(rig.block(), "panel")) later.push(p.position);
  assert.ok(atStop.position > 50, `it had run: ${atStop.position}`);
  assert.ok(later.every((p) => p === later[0]), "and then stood still");
  assert.ok(Math.abs(later[0]! - atStop.position) < 1, `at ${later[0]} after a Stop at ${atStop.position}`);
  rig.send({ type: "transport", play: true });
  let resumed = 0;
  for (let b = 0; b < 200; b++) for (const p of only(rig.block(), "panel")) resumed = p.position;
  assert.ok(resumed > later[0]! + 50, `it went on: ${resumed}`);
});

test("routeFromWorklet hands the position and its frame to the page", () => {
  const got: PanelFrame[] = [];
  const sink: WorkletSink = { onEvents: null, onError: null, onPanel: (p) => got.push(p) };
  routeFromWorklet(
    { type: "panel", frame: 1_000, leds: null, playheads: new Uint8Array(10).buffer, status: 1, droppedIntents: 0, position: 123.5, positionFrame: 1_128 },
    sink,
  );
  assert.equal(got[0]!.position, 123.5);
  assert.equal(got[0]!.positionFrame, 1_128);
  assert.equal(got[0]!.running, true);
});

// The clock end to end on the page side, in Node: the real worklet.ts and the real module (through test/support/worklet-scope.ts), the
// real wiring (src/wire.ts) and the real scheduler (src/midi-out.ts), into two stand-in MIDI outputs. Only the clocks and the outputs are
// made up (test/support/session.ts, receiver.ts). What this pins is what reaches a device and when; where the pulses fall in samples is
// pinned in crates/octocore/tests/clock.rs and apps/web/engine/tests/abi_clock.rs.
import assert from "node:assert/strict";
import { test } from "node:test";
import { RENDER_FRAMES } from "../src/abi.ts";
import { Session } from "./support/session.ts";

const CLOCK = 0xf8;
const START = 0xfa;
const CONTINUE = 0xfb;
const STOP = 0xfc;

function session(): Session {
  return new Session({ seed: 1, supportsClear: false, mapJitterMs: 0, delayMs: 0 });
}

const BLOCKS_PER_SECOND = 48_000 / RENDER_FRAMES;

function run(s: Session, seconds: number): void {
  for (let b = 0; b < seconds * BLOCKS_PER_SECOND; b++) s.step();
  s.drain(200);
}

const realtimeOf = (s: Session, port: number): { at: number; byte: number }[] =>
  s.outputs.get(port)!.delivered.filter((d) => (d.bytes[0] ?? 0) >= 0xf8).map((d) => ({ at: d.at, byte: d.bytes[0]! }));

test("nothing on the clock reaches a device until the clock is turned on", () => {
  const s = session();
  s.clickStep(0, 0);
  s.clickStep(0, 8);
  s.send({ type: "transport", play: true });
  run(s, 2);
  for (const port of [1, 2]) assert.deepEqual(realtimeOf(s, port), [], `port ${port}'s device`);
  assert.ok(s.outputs.get(1)!.delivered.length > 0, "the notes did");
});

test("Start and then a steady clock reach every chosen device, 24 pulses to 500 ms at 120 BPM", () => {
  const s = session();
  s.send({ type: "clock", master: true });
  s.send({ type: "transport", play: true });
  run(s, 2);
  for (const port of [1, 2]) {
    const rt = realtimeOf(s, port);
    assert.equal(rt[0]!.byte, START, `port ${port}'s device hears Start first`);
    const pulses = rt.filter((r) => r.byte === CLOCK);
    assert.ok(pulses.length >= 90, `${pulses.length} pulses in two seconds`);
    for (let i = 1; i < pulses.length; i++) assert.ok(Math.abs(pulses[i]!.at - pulses[i - 1]!.at - 1000 / 48) < 1e-6, `pulse ${i}: ${pulses[i]!.at - pulses[i - 1]!.at} ms`);
    assert.ok(Math.abs(pulses[24]!.at - pulses[0]!.at - 500) < 1e-6, "24 pulses are a quarter note: 500 ms at 120 BPM");
  }
  const [a, b] = [realtimeOf(s, 1), realtimeOf(s, 2)];
  assert.deepEqual(a, b, "both devices hear the same messages at the same times");
});

test("one device on both ports hears the clock once", () => {
  const s = session();
  s.scheduler.setOutput(2, s.outputs.get(1)!);
  s.send({ type: "clock", master: true });
  s.send({ type: "transport", play: true });
  run(s, 1);
  const one = realtimeOf(s, 1).filter((r) => r.byte === CLOCK).length;
  assert.ok(one >= 40 && one <= 50, `${one} pulses in one second at 120 BPM is 48, and not 96`);
  assert.deepEqual(realtimeOf(s, 2), [], "port 2's own device was let go");
});

test("Stop is heard once, after the last pulse and before anything else, and Play again says Continue", () => {
  const s = session();
  s.send({ type: "clock", master: true });
  s.send({ type: "transport", play: true });
  run(s, 1);
  s.send({ type: "transport", play: false });
  run(s, 1);
  s.send({ type: "transport", play: true });
  run(s, 1);
  const bytes = realtimeOf(s, 1).map((r) => r.byte);
  const stops = bytes.flatMap((b, i) => (b === STOP ? [i] : []));
  assert.equal(stops.length, 1, "one Stop");
  assert.ok(bytes.slice(stops[0]! + 1).indexOf(CLOCK) > 0, "pulses resume after the Stop");
  assert.equal(bytes[stops[0]! + 1], CONTINUE, "the next thing after Stop is Continue, because the sequencer was not rewound");
  assert.ok(bytes.slice(0, stops[0]).every((b) => b === START || b === CLOCK), "before the Stop only Start and pulses");
});

test("the notes a device hears are the same bytes at the same times with the clock on and off", () => {
  const notes = (clock: boolean) => {
    const s = session();
    for (const step of [0, 4, 8, 12]) s.clickStep(0, step);
    s.clickStep(3, 2);
    if (clock) s.send({ type: "clock", master: true });
    s.send({ type: "tempo", bpm: 133 });
    s.send({ type: "transport", play: true });
    run(s, 3);
    return [1, 2].map((port) => s.outputs.get(port)!.delivered.filter((d) => (d.bytes[0] ?? 0) < 0xf8));
  };
  const off = notes(false);
  const on = notes(true);
  assert.ok(off[0]!.length >= 8, `${off[0]!.length} note messages`);
  assert.deepEqual(on, off);
});

test("Stop with the clock on leaves no note held on any device", () => {
  const s = session();
  for (const step of [0, 2, 4, 6, 8, 10, 12, 14]) s.clickStep(0, step);
  s.send({ type: "clock", master: true });
  s.send({ type: "transport", play: true });
  let sounding = false;
  for (let b = 0; b < 1000 && !sounding; b++) {
    s.step();
    s.drain(1);
    sounding = s.held().length > 0;
  }
  assert.ok(sounding, "stopping with a note sounding");
  s.send({ type: "transport", play: false });
  run(s, 1);
  assert.deepEqual(s.held(), []);
});

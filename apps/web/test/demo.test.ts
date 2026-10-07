// The pattern the page opens with (specs/SPEC-0002/p6-runs-when-opened.md, observable R-1): a sequencer that is opened empty and silent is
// not seen to run. The pattern is data, written through the same `setTrack` and `setStep` any page code uses, and nothing in the engine
// knows it is a demo.
import assert from "node:assert/strict";
import { test } from "node:test";
import { KIND_REALTIME, RENDER_FRAMES, STEP_ATTR, TRACK_ATTR, TRACK_COUNT, decodeEvents } from "../src/abi.ts";
import { DEMO, loadDemo, type DemoSink } from "../src/demo.ts";
import type { FromWorklet } from "../src/protocol.ts";
import { controlNumbers, matrixId } from "./support/module.ts";
import { WorkletRig } from "./support/worklet-scope.ts";

const numbers = controlNumbers();
const GREEN_STEADY = 2;

function sinkFor(rig: WorkletRig): DemoSink {
  return {
    setTrack: (track, attr, value) => rig.send({ type: "track", track, attr, value }),
    setStep: (track, step, attr, value) => rig.send({ type: "step", track, step, attr, value }),
  };
}

/** What the worklet has posted since the last call: errors, and the last LED frame it sent, which is the page's picture of the panel. */
function posted(rig: WorkletRig, frame: Uint8Array | null = null): { errors: string[]; leds: Uint8Array | null } {
  const errors: string[] = [];
  for (const p of rig.take()) {
    const m: FromWorklet = p.message;
    if (m.type === "error") errors.push(m.message);
    if (m.type === "panel" && m.leds) frame = new Uint8Array(m.leds);
  }
  return { errors, leds: frame };
}

/** The LED frame the page would hold after the last message: the one the messages made, and any a few blocks add. */
function leds(rig: WorkletRig): Uint8Array {
  let frame = posted(rig).leds;
  for (let b = 0; b < 20; b++) {
    rig.block();
    frame = posted(rig, frame).leds;
  }
  assert.ok(frame, "the worklet sent an LED frame");
  return frame;
}

test("R-1: the demo is a real pattern: several tracks, a dozen or more steps, every number one the engine takes", () => {
  assert.ok(DEMO.length >= 4 && DEMO.length <= TRACK_COUNT, `${DEMO.length} tracks`);
  assert.equal(new Set(DEMO.map((t) => t.track)).size, DEMO.length, "each track once");
  let steps = 0;
  for (const t of DEMO) {
    assert.ok(Number.isInteger(t.track) && t.track >= 0 && t.track < TRACK_COUNT, `track ${t.track}`);
    assert.ok(Number.isInteger(t.channel) && t.channel >= 1 && t.channel <= 16, `${t.name}: channel ${t.channel} (port 1)`);
    assert.ok(t.steps.length >= 2, `${t.name} has steps`);
    assert.equal(new Set(t.steps.map((s) => s.step)).size, t.steps.length, `${t.name}: each step once`);
    for (const s of t.steps) {
      assert.ok(Number.isInteger(s.step) && s.step >= 0 && s.step < 16, `${t.name} step ${s.step}`);
      const note = t.pitch + (s.offset ?? 0);
      assert.ok(Number.isInteger(note) && note >= 0 && note <= 127, `${t.name} step ${s.step} is note ${note}`);
      steps++;
    }
  }
  assert.ok(steps >= 12, `${steps} steps`);
});

test("R-1: loading it lights exactly the steps it names, in the engine's own LED frame, with nothing pressed", () => {
  const rig = new WorkletRig();
  loadDemo(sinkFor(rig));
  const sent = posted(rig);
  assert.deepEqual(sent.errors, [], "the engine took every write");
  const frame = sent.leds ?? leds(rig);
  const lit = new Set<number>();
  for (const t of DEMO) for (const s of t.steps) lit.add(numbers.get(matrixId(t.track, s.step))!);
  for (let row = 0; row < TRACK_COUNT; row++) {
    for (let step = 0; step < 16; step++) {
      const n = numbers.get(matrixId(row, step))!;
      assert.equal(frame[n], lit.has(n) ? GREEN_STEADY : 0, `row ${row} step ${step}`);
    }
  }
});

test("R-1: played, it makes the notes it names on the channels it names, and nothing else", () => {
  const rig = new WorkletRig();
  loadDemo(sinkFor(rig));
  rig.send({ type: "transport", play: true });
  const heard = new Set<string>();
  const blocks = Math.ceil((2 * 96_000) / RENDER_FRAMES) + 40; // two bars at 120 BPM and a little
  for (let b = 0; b < blocks; b++) {
    for (const p of rig.block()) {
      if (p.message.type !== "events") continue;
      for (const e of decodeEvents(new Uint8Array(p.message.bytes))) if (e.kind === 0) heard.add(`${e.channel}:${e.d1}`);
    }
  }
  const named = new Set<string>();
  for (const t of DEMO) for (const s of t.steps) named.add(`${t.channel}:${t.pitch + (s.offset ?? 0)}`);
  assert.deepEqual([...heard].sort(), [...named].sort());
  assert.ok(![...heard].some((h) => h.startsWith("0:")), `no real-time record is counted as a note (kind ${KIND_REALTIME})`);
});

test("R-1: loading it twice is the same as once, and a refused write is the caller's to see", () => {
  const once = new WorkletRig();
  loadDemo(sinkFor(once));
  const twice = new WorkletRig();
  loadDemo(sinkFor(twice));
  loadDemo(sinkFor(twice));
  assert.deepEqual(leds(twice), leds(once));
  const calls: string[] = [];
  loadDemo({ setTrack: (t, a, v) => void calls.push(`t${t}.${a}=${v}`), setStep: (t, s, a, v) => void calls.push(`s${t}.${s}.${a}=${v}`) });
  assert.ok(calls.some((c) => c.startsWith(`t${DEMO[0]!.track}.${TRACK_ATTR.pitch}=`)), "a track's pitch is written");
  assert.ok(calls.some((c) => c.includes(`.${STEP_ATTR.active}=1`)), "a step is made active");
});

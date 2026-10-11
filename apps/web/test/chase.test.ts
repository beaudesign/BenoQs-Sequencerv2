// The chase-light (specs/SPEC-0002/p6-runs-when-opened.md, observable R-3): the manual's Red LED on the step each track is playing, and
// the step it shows is the one you hear. The engine steps its tracks up to `MAX_EARLY_TICKS` ahead of the audio, so its playhead is
// ahead of the sound by about a step; the light is held back until the audio reaches the tick the playhead was read at.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { RENDER_FRAMES, STEP_ATTR, TRACK_COUNT, decodeEvents } from "../src/abi.ts";
import { ChaseLight, LED_RED_STEADY, LEAD_TICKS, composeChase, matrixMap } from "../src/chase.ts";
import { QUARTER_TICKS } from "../src/click.ts";
import { controlsDoc, matrixId, repoRoot } from "./support/module.ts";
import { WorkletRig } from "./support/worklet-scope.ts";

const doc = controlsDoc();
const map = matrixMap(doc.controls);

test("R-3: the lead is the engine's own, 12 ticks: the light waits for the audio to reach what the engine has already stepped", () => {
  const engine = readFileSync(`${repoRoot}/crates/octocore/src/engine.rs`, "utf8");
  const early = /pub const MAX_EARLY_TICKS: u32 = (\d+);/.exec(engine);
  assert.ok(early, "engine.rs names MAX_EARLY_TICKS");
  assert.equal(LEAD_TICKS, Number(early[1]));
});

test("R-3: the matrix map has a control number for each of 10 tracks and 16 steps, found by id", () => {
  assert.equal(map.length, TRACK_COUNT);
  const seen = new Set<number>();
  for (let row = 0; row < TRACK_COUNT; row++) {
    assert.equal(map[row]!.length, 16);
    for (let step = 0; step < 16; step++) {
      const id = matrixId(row, step);
      assert.equal(map[row]![step], doc.controls.find((c) => c.id === id)!.n, id);
      seen.add(map[row]![step]!);
    }
  }
  assert.equal(seen.size, 160, "no two steps share a control");
});

test("R-3: composing puts the manual's Red on the step each track is on, changes nothing else, and does not touch the frame it was given", () => {
  const base = new Uint8Array(512);
  base[map[0]![3]!] = 2; // a green step the playhead lands on
  base[map[1]![7]!] = 2; // a green step it does not
  const heads = Uint8Array.from({ length: TRACK_COUNT }, (_, t) => (t === 0 ? 3 : (t * 5) % 16));
  const kept = base.slice();
  const out = composeChase(base, heads, map);
  assert.deepEqual(base, kept, "the engine's frame is not written to");
  const playing = new Set(Array.from(heads, (step, t) => map[t]![step]!));
  assert.equal(playing.size, TRACK_COUNT, "one key a track");
  for (const n of playing) assert.equal(out[n], LED_RED_STEADY, `control ${n} is red`);
  assert.equal(out[map[0]![3]!], LED_RED_STEADY, "the playing step is red, over its green");
  assert.equal(out[map[1]![7]!], 2, "a step that is not playing keeps its colour");
  out.forEach((v, i) => void (!playing.has(i) && assert.equal(v, base[i], `control ${i} is as the engine had it`)));
});

test("R-3: with no playheads the frame comes back as it was", () => {
  const base = new Uint8Array(512).fill(2);
  assert.deepEqual(composeChase(base, null, map), base);
});

/** The panel messages a play of `track 0 all steps on` makes, and when each of its notes sounded, in audio frames. */
function playAllSteps(blocks: number): { chase: { frame: number; step: number | null }[]; heard: number[] } {
  const rig = new WorkletRig();
  for (let s = 0; s < 16; s++) rig.send({ type: "step", track: 0, step: s, attr: STEP_ATTR.active, value: 1 });
  rig.send({ type: "transport", play: true });
  const light = new ChaseLight();
  const chase: { frame: number; step: number | null }[] = [];
  const heard: number[] = [];
  for (let b = 0; b < blocks; b++) {
    for (const p of rig.block()) {
      const m = p.message;
      if (m.type === "events") for (const e of decodeEvents(new Uint8Array(m.bytes))) if (e.kind === 0) heard.push(m.frame + e.atSample);
      if (m.type === "panel") {
        const heads = light.update(m.position, new Uint8Array(m.playheads).subarray(0, TRACK_COUNT), (m.status & 1) === 1);
        chase.push({ frame: m.positionFrame, step: heads ? heads[0]! : null });
      }
    }
  }
  return { chase, heard };
}

test("R-3: the step the light is on is the step you hear, to within one panel message (21 ms), for three bars at 120 BPM", () => {
  const { chase, heard } = playAllSteps(Math.ceil((3 * 96_000) / RENDER_FRAMES));
  assert.ok(heard.length >= 40, `${heard.length} notes`);
  const slack = 8 * RENDER_FRAMES; // one panel message
  const stepHeardAt = (frame: number): number | null => {
    let n = 0;
    for (const t of heard) if (t <= frame) n++;
    return n === 0 ? null : (n - 1) % 16;
  };
  let compared = 0;
  for (const c of chase) {
    if (c.step === null) continue;
    const allowed = new Set([stepHeardAt(c.frame - slack), stepHeardAt(c.frame), stepHeardAt(c.frame + slack)]);
    assert.ok(allowed.has(c.step), `at frame ${c.frame} the light is on step ${c.step} and the audio is on ${[...allowed].join(" or ")}`);
    compared++;
  }
  assert.ok(compared > 100, `${compared} messages compared`);
  assert.ok(new Set(chase.map((c) => c.step)).size >= 16, "it visited every step");
});

test("R-3: the engine's own playhead is not the light: unheld, it runs a step ahead of the sound, which is why the hold exists", () => {
  const rig = new WorkletRig();
  for (let s = 0; s < 16; s++) rig.send({ type: "step", track: 0, step: s, attr: STEP_ATTR.active, value: 1 });
  rig.send({ type: "transport", play: true });
  const heard: number[] = [];
  const raw: { frame: number; step: number }[] = [];
  for (let b = 0; b < Math.ceil(96_000 / RENDER_FRAMES); b++) {
    for (const p of rig.block()) {
      const m = p.message;
      if (m.type === "events") for (const e of decodeEvents(new Uint8Array(m.bytes))) if (e.kind === 0) heard.push(m.frame + e.atSample);
      if (m.type === "panel") raw.push({ frame: m.positionFrame, step: new Uint8Array(m.playheads)[0]! });
    }
  }
  let ahead = 0;
  for (const r of raw) {
    const n = heard.filter((t) => t <= r.frame).length;
    if (n > 0 && r.step !== (n - 1) % 16) ahead++;
  }
  assert.ok(ahead > raw.length / 4, `the raw playhead disagreed with the sound ${ahead} times of ${raw.length}`);
});

test("R-3: nothing is lit while stopped, and a Stop and a Play start the light again from the audio's new position", () => {
  const light = new ChaseLight();
  const heads = new Uint8Array(TRACK_COUNT).fill(5);
  assert.equal(light.update(0, heads, false), null, "stopped");
  assert.equal(light.update(0, heads, true), null, "a Play: the first playhead is not due until the audio reaches it");
  assert.equal(light.update(LEAD_TICKS - 1, heads, true), null);
  assert.deepEqual(light.update(LEAD_TICKS, new Uint8Array(TRACK_COUNT).fill(6), true), heads, "the first playhead is shown when the audio gets to its tick, whatever has come since");
  assert.equal(light.update(LEAD_TICKS + 2, heads, false), null, "a Stop clears it");
  assert.equal(light.update(LEAD_TICKS + 2, heads, true), null, "and it is not shown again until the audio has reached what is read after the Play");
  const jumped = new ChaseLight();
  jumped.update(1000, heads, true);
  jumped.update(1000 + LEAD_TICKS, heads, true);
  assert.equal(jumped.update(0, heads, true), null, "a position that goes back (a reset) forgets everything");
  assert.ok(QUARTER_TICKS > LEAD_TICKS, "a lead is shorter than a beat");
});

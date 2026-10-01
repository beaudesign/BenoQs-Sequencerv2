// The programs the native test pins (apps/web/engine/tests/program.rs), played here through the module in
// Node and through the real worklet file in a stand-in scope. Same digest, or the worklet is not playing
// what the native engine plays.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { Fnv64, matrixId, play, type Programs } from "../spikes/program.ts";
import { decodeEvents } from "../src/abi.ts";
import { controlNumbers, repoRoot, start } from "./support/module.ts";
import { WorkletRig } from "./support/worklet-scope.ts";

const doc = JSON.parse(readFileSync(join(repoRoot, "apps/web/engine/golden/programs.json"), "utf8")) as Programs;
const numbers = controlNumbers();

test("FNV-1a 64 matches its published test vectors", () => {
  const h = (s: string): string => {
    const f = new Fnv64();
    f.update(new TextEncoder().encode(s));
    return f.hex();
  };
  assert.equal(h(""), "fnv1a64:cbf29ce484222325");
  assert.equal(h("a"), "fnv1a64:af63dc4c8601ec8c");
  assert.equal(h("foobar"), "fnv1a64:85944171f73967e8");
});

for (const name of Object.keys(doc.programs)) {
  const program = doc.programs[name]!;
  test(`the ${name} program through the module in Node gives the digest the native test pins`, () => {
    const engine = start(doc.sample_rate, BigInt(program.seed));
    const played = play(engine, numbers, doc, program);
    assert.ok(played.events > 50, `${played.events} events`);
    assert.equal(played.digest, program.digest);
  });

  test(`the ${name} program through the real worklet file gives the same digest`, () => {
    const rig = new WorkletRig({ sampleRate: doc.sample_rate, seed: BigInt(program.seed) });
    rig.take();
    program.steps.forEach(([row, step], i) => {
      const control = numbers.get(matrixId(row, step))!;
      rig.send({ type: "input", nowMs: i * doc.click_gap_ms, kind: 0, control, detents: 0 });
      rig.send({ type: "input", nowMs: i * doc.click_gap_ms + 50, kind: 1, control, detents: 0 });
    });
    rig.send({ type: "tempo", bpm: program.bpm });
    rig.send({ type: "transport", play: true });
    rig.take();
    const hash = new Fnv64();
    const index = new Uint8Array(4);
    const view = new DataView(index.buffer);
    for (let b = 0; b < program.blocks; b++) {
      for (const p of rig.block()) {
        if (p.message.type !== "events") continue;
        assert.equal(p.message.frame, b * doc.frames_per_block);
        const bytes = new Uint8Array(p.message.bytes);
        view.setUint32(0, b, true);
        for (let i = 0; i < decodeEvents(bytes).length; i++) {
          hash.update(index);
          hash.update(bytes.subarray(i * 12, (i + 1) * 12));
        }
      }
    }
    assert.equal(hash.hex(), program.digest);
  });
}

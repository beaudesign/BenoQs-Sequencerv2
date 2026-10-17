// SPEC-0004 H-16: no voice skips the tests. One test walks the registry of voices and holds every entry to the voice-wide
// observables: pitch (H-2), no aliasing (H-3), stability (H-6), velocity (H-9), determinism (H-11) and cost (H-13). A voice added
// to `src/voices/registry.ts` is held to them with no edit here. (The test files are flat in test/, as the others are; the spec
// names them `voices/*.test.ts`.)
import assert from "node:assert/strict";
import { readdirSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { MAX_VOICES } from "../src/monitor.ts";
import { PLAIN } from "../src/voices/plain.ts";
import { DEFAULT_VOICE, VOICES, voiceById } from "../src/voices/registry.ts";
import { webRoot } from "./support/module.ts";
import { checkAlias, checkCost, checkDeterministic, checkNoAllocationInFile, checkPitch, checkStable, checkVelocity } from "./support/voice-checks.ts";

test("H-16: the registry is a list of voices: unique ids, names a button can hold, caps that fit the pool; Plain is first and the default", () => {
  assert.ok(VOICES.length >= 1);
  assert.equal(VOICES[0], PLAIN, "Plain is the first entry");
  assert.equal(DEFAULT_VOICE, PLAIN, "and the default, until a later slice says otherwise (SPEC-0004 D2)");
  assert.equal(new Set(VOICES.map((v) => v.id)).size, VOICES.length, "ids are unique");
  for (const v of VOICES) {
    assert.match(v.id, /^[a-z][a-z0-9-]*$/, v.id);
    assert.ok(v.name.length >= 1 && v.name.length <= 16, `${v.id}: a name a strip button can hold`);
    assert.ok(Number.isInteger(v.maxNotes) && v.maxNotes >= 1 && v.maxNotes <= MAX_VOICES, `${v.id}: maxNotes ${v.maxNotes} fits the pool of ${MAX_VOICES}`);
    assert.equal(typeof v.traits.velocityBrightens, "boolean");
    assert.equal(voiceById(v.id), v);
  }
  assert.equal(voiceById("nope"), undefined, "a voice that is not in the registry cannot be chosen");
});

for (const voice of VOICES) {
  test(`H-16, H-2: ${voice.id} plays each note from MIDI 24 to 96 within 2 cents`, () => {
    assert.deepEqual(checkPitch(voice), []);
  });
  test(`H-16, H-3: ${voice.id} has no energy off the harmonics above -50 dB, MIDI 60 to 84 and a spread from 36 to 96`, () => {
    assert.deepEqual(checkAlias(voice), []);
  });
  test(`H-16, H-6: ${voice.id} is stable over 10,000 random notes at 44.1, 48 and 96 kHz and is silent within 5 s of the Note Off`, () => {
    assert.deepEqual(checkStable(voice), []);
  });
  test(`H-16, H-9: ${voice.id} says truly whether velocity brightens it`, () => {
    assert.deepEqual(checkVelocity(voice), []);
  });
  test(`H-16, H-11: ${voice.id} makes the same samples twice`, () => {
    assert.deepEqual(checkDeterministic(voice), []);
  });
  test(`H-16, H-13: 24 notes of ${voice.id} render 10 s in under a quarter of real time`, () => {
    const { problems, ratio } = checkCost(voice);
    assert.deepEqual(problems, []);
    console.log(`# cost ${voice.id}: ${(ratio * 100).toFixed(1)}% of real time for 24 notes`);
  });
}

test("H-13: nothing in a sample loop in src/voices/ allocates, and there is at least one sample loop to look at", () => {
  const dir = join(webRoot, "src/voices");
  let loops = 0;
  for (const f of readdirSync(dir).filter((n) => n.endsWith(".ts"))) {
    const r = checkNoAllocationInFile(join(dir, f));
    assert.deepEqual(r.problems, []);
    loops += r.loops;
  }
  assert.ok(loops >= VOICES.length, `${loops} sample loops for ${VOICES.length} voices`);
});

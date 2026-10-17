// The voice rig, tested (SPEC-0004 S0). A rig that measures a voice is only worth anything if it measures right and if each of its
// checks can be seen to fail on a voice that is wrong. So: the measurements against signals whose answer is known, then each check
// against a voice made wrong on purpose (test/support/voice-stubs.ts).
import assert from "node:assert/strict";
import { test } from "node:test";
import { PLAIN } from "../src/voices/plain.ts";
import { cents, centroid, fundamental, inharmonicDb, powerSpectrum } from "./support/spectrum.ts";
import {
  ALIAS_BOUND_DB,
  PITCH_TOLERANCE_CENTS,
  checkAlias,
  checkCost,
  checkDeterministic,
  checkNoAllocation,
  checkPitch,
  checkStable,
  checkVelocity,
  sampleLoops,
} from "./support/voice-checks.ts";
import { stub } from "./support/voice-stubs.ts";

const RATE = 48_000;
const sine = (hz: number, n: number, amp = 1): Float64Array => Float64Array.from({ length: n }, (_, i) => amp * Math.sin((2 * Math.PI * hz * i) / RATE));

// ----- the measurements -----

test("rig: the frequency is measured to a hundredth of a cent, from a bass note to a high one", () => {
  for (const f of [32.703, 110, 440.123, 1000.3, 2093.005]) {
    const err = Math.abs(cents(fundamental(sine(f, 90_000), RATE, f), f));
    assert.ok(err < 0.01, `${f} Hz measured ${err} cents out`);
  }
});

test("rig: it measures the note it was asked about, even when a louder component sits outside the semitone it searches", () => {
  const a = sine(440, 90_000);
  const b = sine(880, 90_000, 1.5);
  const mix = a.map((v, i) => v + b[i]!);
  assert.ok(Math.abs(cents(fundamental(mix, RATE, 440), 440)) < 0.01);
});

test("rig: the spectrum puts a sine in its bin, and the centroid of a sine is its frequency", () => {
  const p = powerSpectrum(sine(1500, 65_536), 65_536);
  const peak = p.indexOf(Math.max(...p));
  assert.ok(Math.abs((peak * RATE) / 65_536 - 1500) < RATE / 65_536);
  assert.ok(Math.abs(centroid(sine(1500, 65_536), RATE, 65_536) / 1500 - 1) < 0.01);
});

test("rig: energy off the harmonics: a sine has none, and a naive saw has a great deal", () => {
  assert.ok(inharmonicDb(sine(1046.5, 65_536), RATE, 1046.5, { n: 65_536 }) < -90);
  const saw = Float64Array.from({ length: 65_536 }, (_, i) => 2 * (((1046.5 * i) / RATE) % 1) - 1);
  assert.ok(inharmonicDb(saw, RATE, 1046.5, { n: 65_536 }) > -30, "a naive saw at C6 folds its upper harmonics onto frequencies that are not harmonics");
});

// ----- each check fails on a voice that is wrong -----

test("rig, H-2: a voice ten cents flat fails the pitch check, and one that is right passes", () => {
  assert.deepEqual(checkPitch(stub({ id: "right" }), [24, 60, 96]), []);
  const flat = checkPitch(stub({ id: "flat", centsOff: -10 }), [24, 60, 96]);
  assert.equal(flat.length, 9, "three notes at each of three rates");
  assert.match(flat[0]!, /-10\.0\d cents/);
  assert.ok(PITCH_TOLERANCE_CENTS === 2);
});

test("rig, H-2: a voice with 48000 written into it passes at 48 kHz and fails at 44.1 and 96 kHz", () => {
  const fixed = stub({ id: "fixed-rate", fixedRate: true });
  assert.deepEqual(checkPitch(fixed, [24, 60, 96], [48_000]), [], "right at the one rate the rig used to check");
  const wrong = checkPitch(fixed, [24, 60, 96]);
  assert.equal(wrong.length, 6, "three notes at each of the two other rates");
  assert.ok(wrong.every((p) => / at (44100|96000) Hz /.test(p)), wrong.join("; "));
});

test("rig, H-3: a naive saw fails the alias check, a sine passes, and so does Plain", () => {
  assert.deepEqual(checkAlias(stub({ id: "sine" }), [60, 72, 84]), []);
  const naive = checkAlias(stub({ id: "naive-saw", saw: true }), [60, 72, 84]);
  assert.ok(naive.length >= 1, "at least the top note aliases");
  assert.match(naive[0]!, /dB of energy off the harmonics/);
  assert.deepEqual(checkAlias(PLAIN, [60, 72, 84]), []);
  assert.equal(ALIAS_BOUND_DB, -50);
});

test("rig, H-6: a voice that goes to NaN, and one that never ends, fail the stability check; a good one passes", () => {
  assert.deepEqual(checkStable(stub({ id: "good" }), 200), []);
  assert.match(checkStable(stub({ id: "nan", nanAfter: 50 }), 50)[0]!, /is NaN/);
  assert.match(checkStable(stub({ id: "endless", endless: true }), 5)[0]!, /still sounding/);
});

test("rig, H-9: the declaration is checked both ways", () => {
  assert.deepEqual(checkVelocity(stub({ id: "bright", brightens: true })), []);
  assert.deepEqual(checkVelocity(stub({ id: "flat" })), []);
  assert.match(checkVelocity(stub({ id: "liar-yes", declares: true }))[0]!, /declares that velocity brightens it/);
  assert.match(checkVelocity(stub({ id: "liar-no", brightens: true, declares: false }))[0]!, /declares that velocity does not brighten it/);
});

test("rig, H-11: a voice that uses Math.random fails the determinism check, and a good one passes", () => {
  assert.deepEqual(checkDeterministic(stub({ id: "good" })), []);
  assert.match(checkDeterministic(stub({ id: "dice", random: true }))[0]!, /two runs differ/);
});

test("rig, H-13: the cost check measures a real ratio, and fails when the bound is made too small", () => {
  // This test is about the measurement, not about Plain: a generous bound, so a busy machine cannot fail it. The registry test is what holds Plain to the 25 percent.
  const ok = checkCost(PLAIN, { seconds: 1, runs: 1, bound: 1 });
  assert.ok(ok.ratio > 0 && Number.isFinite(ok.ratio));
  assert.deepEqual(ok.problems, []);
  assert.match(checkCost(PLAIN, { seconds: 0.5, runs: 1, bound: 1e-9 }).problems[0]!, /of real time/);
});

const CLEAN = `class A { next(): number { const t = this.x * 2; this.x += 1; return Math.sin(t) * this.g; } other() { return [1]; } }`;

test("rig, H-13: the text check finds a sample loop, and nothing else, and passes a clean one", () => {
  assert.equal(sampleLoops(CLEAN).length, 1);
  assert.ok(!sampleLoops(CLEAN)[0]!.includes("[1]"), "other methods are not looked at");
  assert.deepEqual(checkNoAllocation(CLEAN, "clean"), []);
  assert.equal(sampleLoops("class B { foo() { return 1; } }").length, 0);
});

test("rig, H-13: the text check flags each way a sample loop can allocate", () => {
  const bad: [string, RegExp][] = [
    ["const a = new Float64Array(2); return a[0];", /`new`/],
    ["const a = [1, 2]; return a[0];", /array literal/],
    ["return [this.x][0];", /array literal/],
    ["const o = { a: 1 }; return o.a;", /object literal/],
    ["return this.xs.map(x => x)[0];", /array method|closure/],
    ["const f = () => 1; return f();", /closure/],
    ["const f = function () { return 1; }; return f();", /function expression/],
    ["const s = `${this.x}`; return s.length;", /template string/],
    ["return Math.max(...this.xs);", /spread/],
    ["return Array.from(this.xs)[0]!;", /Array\.from/],
  ];
  for (const [body, what] of bad) {
    const found = checkNoAllocation(`class C { next(): number { ${body} } }`, "bad");
    assert.ok(found.some((p) => what.test(p)), `${body} -> ${found.join("; ")}`);
  }
  assert.deepEqual(checkNoAllocation(`class D { next(): number { // new [] => \n return 0; /* new Foo() */ } }`, "comments"), [], "words in comments are not code");
});

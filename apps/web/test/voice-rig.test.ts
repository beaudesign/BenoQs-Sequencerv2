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

const CLEAN = `class A { private xs = new Float64Array(4); constructor() { this.xs = new Float64Array(8); } next(): number { const t = this.x * 2; this.x += 1; return Math.sin(t) * this.g; } other(): number { return this.xs[1]!; } }`;

test("rig, H-13: the text check looks at every method a note runs except the constructor, and passes a clean class", () => {
  assert.equal(sampleLoops(CLEAN).length, 2, "next and other; the constructor may allocate");
  assert.deepEqual(checkNoAllocation(CLEAN, "clean"), []);
  assert.equal(sampleLoops("class B { foo() { return 1; } }").length, 1);
  assert.equal(sampleLoops("const f = () => { return [1]; };").length, 0, "no class, nothing a note runs");
  assert.equal(sampleLoops("class E { next(): number { if (this.x) { return 1; } return 0; } }").length, 1, "an `if (...) {` is not a method");
});

test("rig, H-13: the text check finds an allocation wherever a note runs it, in the forms a review found it missed", () => {
  const where: [string, string][] = [
    ["a helper method", "class C { next(): number { return this.h(); } private h(): number { const a = [1, 2]; return a[0]!; } }"],
    ["start()", "class C { start(): void { this.xs = new Float64Array(8); } next(): number { return 0; } }"],
    ["release()", "class C { release(): void { this.log.push(1); } next(): number { return 0; } }"],
    ["a getter", "class C { get live(): boolean { return Object.keys(this.o).length > 0; } next(): number { return 0; } }"],
    ["an arrow-function next", "class C { next = (): number => { const a = new Float64Array(2); return a[0]!; }; }"],
    ["an unannotated next", "class C { next() { const o = { a: 1 }; return o.a; } }"],
    [".push", "class C { next(): number { this.xs.push(1); return 0; } }"],
    ["Float64Array.from", "class C { next(): number { return Float64Array.from(this.xs)[0]!; } }"],
    ["Array(8)", "class C { next(): number { return Array(8).length; } }"],
    ["Object.keys", "class C { next(): number { return Object.keys(this.o).length; } }"],
    ["string building", "class C { next(): number { const s = 'a' + this.x; return s.length; } }"],
    [".toFixed", "class C { next(): number { return this.x.toFixed(2).length; } }"],
  ];
  for (const [what, source] of where) assert.notDeepEqual(checkNoAllocation(source, what), [], `${what} passed the text check`);
});

test("rig, H-13: a voice that is cheap held but dear when its notes restart fails the churn workload and passes the held one", () => {
  // `heavyDies`: heavy work for the first 9000 samples of a note, then the note ends. Started once and left, that is a small share of
  // ten seconds; restarted every quarter second (12,000 samples) it is three quarters of every frame.
  const heavy = stub({ id: "heavy-then-dies", heavyDies: 9000 });
  const r = checkCost(heavy, { seconds: 2, runs: 2 });
  assert.equal(r.problems.filter((p) => /held notes/.test(p)).length, 0, r.problems.join("; "));
  assert.ok(r.problems.some((p) => /restarted every quarter second/.test(p)), `churn did not fail it: ${r.problems.join("; ")} ratio ${r.ratio}`);
  // `slowStart`: 4 ms of CPU in start(). Paid 24 times when held, about 216 times when restarted.
  const slow = checkCost(stub({ id: "slow-start", slowStart: 4 }), { seconds: 2, runs: 2 });
  assert.equal(slow.problems.filter((p) => /held notes/.test(p)).length, 0, slow.problems.join("; "));
  assert.ok(slow.problems.some((p) => /restarted every quarter second/.test(p)), `churn did not fail it: ${slow.problems.join("; ")} ratio ${slow.ratio}`);
});

test("rig, H-9: a brightening that happens only in the attack, and a rise that turns into a fall, are both found", () => {
  const attack = checkVelocity(stub({ id: "attack-only", transientBrightens: true }));
  assert.ok(attack.some((p) => /first 4096 samples brighten/.test(p)), attack.join("; "));
  const dip = checkVelocity(stub({ id: "dips", dips: true, declares: true }));
  assert.ok(dip.some((p) => /centroid falls from velocity/.test(p)), dip.join("; "));
  assert.deepEqual(checkVelocity(stub({ id: "bright", brightens: true })), [], "a voice that brightens all the way up still passes");
});

test("rig, H-11: a voice that rolls the dice once in 40,000 samples is found, past the first 20,000", () => {
  assert.match(checkDeterministic(stub({ id: "late-dice", randomLate: true }))[0]!, /two runs differ/);
  assert.deepEqual(checkDeterministic(stub({ id: "good" })), []);
});

test("rig, H-2 and H-3: a voice silent above MIDI 90 is called silent, and a naive saw above MIDI 90 aliases", () => {
  const quiet = checkPitch(stub({ id: "quiet-top", silentAbove: 90 }), [60, 90, 96], [48_000]);
  assert.equal(quiet.length, 2);
  assert.ok(quiet.every((p) => /is silent/.test(p)), quiet.join("; "));
  const aliased = checkAlias(stub({ id: "saw-top", sawAbove: 90 }));
  assert.ok(aliased.length >= 1 && aliased.every((p) => /MIDI (90|96) /.test(p)), aliased.join("; "));
  assert.deepEqual(checkAlias(stub({ id: "sine" })), [], "a sine passes the wider range");
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

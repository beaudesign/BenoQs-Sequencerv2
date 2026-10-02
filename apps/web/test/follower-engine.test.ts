// The follower in front of the REAL engine (test/support/engine-loop.ts says how the loop is made and what is measured): what reaches a
// device, against the sender's pulses. Both ends of the wire the follower is on are made up (the sender's pulses, the receiver); the
// engine, the worklet, the scheduler and the follower are the real ones. Thresholds are the author's proposals and the owner's to
// change; where they were set from a measurement it is said, and the measurement is in handoffs/evidence/p4d-follower-measured.txt.
import assert from "node:assert/strict";
import { test } from "node:test";
import { BLOCK_MS, closedLoop, pulseErrors, worst } from "./support/engine-loop.ts";
import { ramp, steady, step } from "./support/sim-clock.ts";

test("a steady sender: the engine's pulses reach the device on the sender's, to within a tick, after the catch-up; and the first one is the first", () => {
  for (const seed of [1, 2, 3, 4]) {
    const run = closedLoop({ seconds: 10, bpm: steady(120) }, { seed });
    const errors = pulseErrors(run);
    assert.ok(errors.length > 400, `${errors.length} pulses`);
    assert.ok(worst(errors, 2_500) <= 1, `seed ${seed}: worst ${worst(errors, 2_500).toFixed(2)} ticks after the first 2.5 s`);
    // Pulse 0 is the sender's pulse 0 to within a pulse's own catch-up: nowhere near a whole pulse (8 ticks) away.
    assert.ok(Math.abs(errors[0]!.ticks) < 8 * 3, `the first pulse is ${errors[0]!.ticks.toFixed(1)} ticks out`);
    assert.equal(run.follower.running, true);
  }
});

/** The engine's first note, and every one after it, sounds 11 ticks after the pulse of its step: a fact of the engine, not of the clock (crates/octocore/tests/clock.rs, AMBIGUITIES.md "the first note after Play"), and a question for the owner. */
const NOTE_LAG_TICKS = 11;

test("the notes land where the pulses do, which is the engine's own 11 ticks after the sender's pulse of their step, within a tick and a half", () => {
  const run = closedLoop({ seconds: 10, bpm: steady(120) }, { seed: 2 });
  assert.ok(run.notes.length >= 8, `${run.notes.length} notes`);
  // A note on step 1 and one on step 9: the sender's pulses 0 and 12 of every 24.
  const settled = run.notes.map((at, m) => ({ at, pulse: 12 * m })).filter((n) => n.at - run.startAt > 3_000 && n.pulse < run.sim.truth.length - 1);
  assert.ok(settled.length >= 6, `${settled.length} notes after the catch-up`);
  for (const n of settled) {
    const period = run.sim.truth[n.pulse + 1]! - run.sim.truth[n.pulse]!;
    const ticks = (n.at - run.sim.truth[n.pulse]!) / (period / 8);
    assert.ok(Math.abs(ticks - NOTE_LAG_TICKS) <= 1.5, `note ${n.pulse / 12} is ${ticks.toFixed(2)} ticks after the sender's pulse ${n.pulse}; the engine puts it ${NOTE_LAG_TICKS} after`);
  }
});

test("another tempo, and a step in the tempo with the count kept: still the same pulse, within a tick, a few seconds later", () => {
  const slow = closedLoop({ seconds: 10, bpm: steady(70) }, { seed: 3 });
  assert.ok(worst(pulseErrors(slow), 3_000) <= 1, `70 BPM: ${worst(pulseErrors(slow), 3_000).toFixed(2)} ticks`);
  const fast = closedLoop({ seconds: 10, bpm: steady(160) }, { seed: 3 });
  assert.ok(worst(pulseErrors(fast), 3_000) <= 1, `160 BPM: ${worst(pulseErrors(fast), 3_000).toFixed(2)} ticks`);
  for (const seed of [1, 2, 3]) {
    const run = closedLoop({ seconds: 14, bpm: step(120, 90, 5) }, { seed });
    const errors = pulseErrors(run);
    assert.ok(worst(errors, 3_000, 5_000) <= 1, `seed ${seed}: before the step ${worst(errors, 3_000, 5_000).toFixed(2)}`);
    assert.ok(worst(errors, 9_500) <= 1, `seed ${seed}: after it, settled: ${worst(errors, 9_500).toFixed(2)} ticks (a pulse off would be 8)`);
  }
});

test("a ramp in the sender's tempo is followed: the error stays under four ticks while it moves, and under one when it has stopped", () => {
  // 3.75 BPM a second. Four is the bound the simulated follower's fast ramp has (test/follower.test.ts); measured here on seeds 100 to 119
  // the worst is 3.51 (p50 2.96), which leaves little room: a ramp is followed with a lag the loop and the estimator share.
  for (const seed of [4, 5]) {
    const errors = pulseErrors(closedLoop({ seconds: 14, bpm: ramp(100, 130, 8) }, { seed }));
    assert.ok(worst(errors, 3_000, 8_000) <= 4, `seed ${seed}: during the ramp ${worst(errors, 3_000, 8_000).toFixed(2)} ticks`);
    assert.ok(worst(errors, 11_000) <= 1, `seed ${seed}: after it ${worst(errors, 11_000).toFixed(2)} ticks`);
  }
});

test("with the output's own timing noise on (the page's idea of the audio clock wrong by up to 1.5 ms at each batch) a steady 120 BPM is still within a tick", () => {
  // Measured on seeds 100 to 119: worst 0.85 (p50 0.74). It is the noise that is most of that; at 160 BPM, where a tick is 1.9 ms, it
  // reaches 1.11 and the bound of a tick would not hold: the figure is in handoffs/evidence/p4d-follower-measured.txt.
  for (const seed of [1, 2, 3]) {
    const errors = pulseErrors(closedLoop({ seconds: 10, bpm: steady(120) }, { seed, outputJitterMs: 1.5 }));
    assert.ok(worst(errors, 2_500) <= 1, `seed ${seed}: ${worst(errors, 2_500).toFixed(2)} ticks`);
  }
});

test("the offset moves the engine against the sender's beat by as much as it says, positive earlier, and it settles there", () => {
  for (const offsetMs of [20, -15]) {
    const run = closedLoop({ seconds: 10, bpm: steady(120) }, { seed: 5, offsetMs });
    // A positive offset puts the pulses `offsetMs` earlier at the device: their delivery less the sender's pulse is minus the offset.
    const errors = pulseErrors(run, offsetMs);
    assert.ok(worst(errors, 3_000) <= 1, `offset ${offsetMs}: ${worst(errors, 3_000).toFixed(2)} ticks from where it was asked to be`);
    const raw = pulseErrors(run, 0).filter((e) => e.sinceStart > 3_000).map((e) => e.ticks);
    const meanMs = (raw.reduce((a, b) => a + b, 0) / raw.length) * (2500 / 120 / 8);
    assert.ok(Math.abs(meanMs - -offsetMs) < 2, `delivered ${meanMs.toFixed(1)} ms from the sender's pulse; asked for ${-offsetMs}`);
  }
});

test("a change of offset while it runs moves the engine to the new place within about two seconds", () => {
  const run = closedLoop(
    { seconds: 12, bpm: steady(120) },
    { seed: 6, script: [{ at: 7_000, run: (f) => f.setOffset(25) }] },
  );
  const raw = pulseErrors(run, 0).map((e) => ({ ...e, ms: e.ticks * (2500 / 120 / 8) }));
  const before = raw.filter((e) => e.sinceStart > 3_000 && e.at < 7_000);
  const after = raw.filter((e) => e.at > 9_500);
  assert.ok(before.every((e) => Math.abs(e.ms) < 2.6), "before: on the beat");
  assert.ok(after.every((e) => Math.abs(e.ms - -25) < 2.6), `after: 25 ms early: ${after.map((e) => e.ms.toFixed(1)).slice(0, 3).join(", ")}`);
});

test("the follower's lookahead has to be the scheduler's: when it is not, the engine lands by the difference", () => {
  const same = closedLoop({ seconds: 8, bpm: steady(120) }, { seed: 7, lookaheadMs: 60 });
  assert.ok(worst(pulseErrors(same), 3_000) <= 1, `the same 60 ms in both: ${worst(pulseErrors(same), 3_000).toFixed(2)} ticks`);
  const apart = closedLoop({ seconds: 8, bpm: steady(120) }, { seed: 7, lookaheadMs: 30, schedulerLookaheadMs: 60 });
  const raw = pulseErrors(apart, 0).filter((e) => e.sinceStart > 3_000).map((e) => e.ticks * (2500 / 120 / 8));
  const mean = raw.reduce((a, b) => a + b, 0) / raw.length;
  assert.ok(Math.abs(mean - 30) < 2, `the scheduler's extra 30 ms is heard as ${mean.toFixed(1)} ms late`);
});

test("Stop reaches the device with the sender's, and nothing is played after it", () => {
  const run = closedLoop({ seconds: 10, bpm: steady(120) }, { seed: 1, stopAt: 6_000 });
  assert.notEqual(run.stopDelivered, null, "the Stop was delivered");
  // Delivered a lookahead after the engine made it, and the engine made it when the page said so, at the next block.
  const late = run.stopDelivered! - 6_000;
  assert.ok(late >= 0 && late < 30 + 3 * BLOCK_MS + 4 + 1, `the Stop was delivered ${late.toFixed(1)} ms after the sender's`);
  assert.equal(run.pulses.filter((at) => at > run.stopDelivered! + 1).length, 0, "no pulse after it");
  assert.equal(run.notes.filter((at) => at > run.stopDelivered! + 1).length, 0, "no note after it");
  assert.deepEqual(run.session.held(), [], "no note is left on");
  assert.equal(run.follower.running, false);
});

// Spike S3's arithmetic, apart from the pages so it can be tested: what a followed clock looks like to the page, in numbers, and what a
// clock the app sends looks like when it comes back round a loopback port. The data here is made up and says so: it is built to have a
// known answer. What a real Live and a real port do is what the owner's run measures.
import assert from "node:assert/strict";
import { test } from "node:test";
import { analyseFollow, analyseLoop, analyseStampMap, followVerdict, unpackPulses, type FollowRaw, type FollowSample, type Pulse } from "../spikes/s3/analyse.ts";
import { Rng } from "./support/rng.ts";

/** Pulses at `bpm` for `seconds`, each stamped `age` ms before it was seen, with uniform noise of plus or minus `noise` ms on the stamp. */
function pulses(bpm: number, seconds: number, opts: { age?: number; noise?: number; seed?: number; start?: number; skip?: (i: number) => boolean } = {}): Pulse[] {
  const rng = new Rng((opts.seed ?? 1) * 99991 + 5);
  const out: Pulse[] = [];
  const period = 2500 / bpm;
  const start = opts.start ?? 10_000;
  for (let i = 0; i * period < seconds * 1000; i++) {
    if (opts.skip?.(i)) continue;
    const stamp = start + i * period + (rng.next() * 2 - 1) * (opts.noise ?? 0);
    out.push({ stamp, seen: stamp + (opts.age ?? 2) });
  }
  return out;
}

const sample = (at: number, over: Partial<FollowSample> = {}): FollowSample => ({ at, phase: "following", bpm: 120, phaseMs: 0, jitterMs: 0.5, pulses: 100, running: true, hidden: false, ...over });
const raw = (over: Partial<FollowRaw> = {}): FollowRaw => ({ pulses: [], samples: [], messages: [], ...over });

test("the clock domain: a stamp a little behind the time it was seen is on the page's clock; a stamp on another clock is told apart, with the numbers", () => {
  const same = analyseFollow(raw({ pulses: pulses(120, 10, { age: 2, noise: 0.5 }) }));
  assert.equal(same.clockDomain.verdict, "same-clock");
  assert.ok(Math.abs(same.clockDomain.ageMs.p50 - 2) < 0.6, `${same.clockDomain.ageMs.p50}`);
  // A clock that began at another moment: the stamps are a million milliseconds earlier than the page's own.
  const other = analyseFollow(raw({ pulses: pulses(120, 10, { age: 1_000_000 }) }));
  assert.equal(other.clockDomain.verdict, "other-clock");
  assert.match(other.clockDomain.why, /1000000\.0/);
  // Stamps in the future of the moment they were seen cannot be on the same clock whatever the size.
  const future = analyseFollow(raw({ pulses: pulses(120, 10, { age: -40 }) }));
  assert.equal(future.clockDomain.verdict, "other-clock");
  // Late, but not wildly: 80 ms is still the same clock (a busy page); 300 ms is not a delay to call the same.
  assert.equal(analyseFollow(raw({ pulses: pulses(120, 10, { age: 80 }) })).clockDomain.verdict, "same-clock");
  assert.equal(analyseFollow(raw({ pulses: pulses(120, 10, { age: 300 }) })).clockDomain.verdict, "other-clock");
  assert.equal(analyseFollow(raw()).clockDomain.verdict, "no-data");
});

test("the intervals: the tempo from the stamps, the scatter of the periods, and no drift in a steady clock", () => {
  const a = analyseFollow(raw({ pulses: pulses(120, 300, { noise: 1 }) }));
  assert.equal(a.pulses, 14_400);
  assert.ok(Math.abs(a.intervals.tempoBpm.median - 120) < 0.2, `${a.intervals.tempoBpm.median}`);
  assert.ok(Math.abs(a.intervals.tempoBpm.fromRun - 120) < 0.001, `${a.intervals.tempoBpm.fromRun}`);
  assert.ok(Math.abs(a.intervals.periodMs.mean - 2500 / 120) < 0.01);
  assert.ok(a.intervals.periodMs.sigma > 0.4 && a.intervals.periodMs.sigma < 1.2, `a period's sigma is that of two noises of +-1 ms: ${a.intervals.periodMs.sigma}`);
  assert.ok(Math.abs(a.intervals.driftBpmPerHour) < 1, `${a.intervals.driftBpmPerHour}`);
  assert.deepEqual([a.intervals.backwards, a.intervals.equalStamps, a.intervals.bursts, a.intervals.stalls], [0, 0, 0, 0]);
});

test("the median of the tempos is exact for a clean clock, and a sender quantised to a millisecond has its fifth of a step from the run's tempo and not from that median", () => {
  const clean = analyseFollow(raw({ pulses: pulses(120, 10) }));
  assert.ok(Math.abs(clean.intervals.tempoBpm.median - 120) < 1e-6, `${clean.intervals.tempoBpm.median}`);
  // Stamps rounded to whole milliseconds, as a timer gives them: periods of 20 and 21 ms (median 21, which reads 119.05 BPM) over a run that is 120.
  const out: Pulse[] = [];
  for (let i = 0; i < 1_200; i++) out.push({ stamp: 10_000 + Math.round((i * 2500) / 120), seen: 10_000 + Math.round((i * 2500) / 120) + 1 });
  const q = analyseFollow(raw({ pulses: out }));
  assert.ok(Math.abs(q.intervals.tempoBpm.median - 119.05) < 0.01, `the median of the periods reads ${q.intervals.tempoBpm.median}`);
  assert.ok(Math.abs(q.intervals.tempoBpm.fromRun - 120) < 0.05, `${q.intervals.tempoBpm.fromRun}`);
  assert.ok(Math.abs(q.phase.fifthOfStepMs - 6.25) < 0.01, `a fifth of a step is ${q.phase.fifthOfStepMs} ms, from the run's 120 BPM`);
});

test("the drift: a sender whose tempo creeps up 0.6 BPM in ten minutes is read as about 3.6 BPM an hour", () => {
  const out: Pulse[] = [];
  let t = 0;
  while (t < 600_000) {
    const bpm = 120 + (0.6 * t) / 600_000;
    out.push({ stamp: 5_000 + t, seen: 5_000 + t + 1 });
    t += 2500 / bpm;
  }
  const a = analyseFollow(raw({ pulses: out }));
  assert.ok(Math.abs(a.intervals.driftBpmPerHour - 3.6) < 0.3, `${a.intervals.driftBpmPerHour}`);
});

test("the drift leaves out the short block at the end of a run: five seconds at another tempo are not a trend", () => {
  const out: Pulse[] = [];
  let t = 0;
  while (t < 240_000) {
    out.push({ stamp: 5_000 + t, seen: 5_001 + t });
    t += 2500 / 120;
  }
  while (t < 245_000) {
    out.push({ stamp: 5_000 + t, seen: 5_001 + t });
    t += 2500 / 130;
  }
  assert.ok(Math.abs(analyseFollow(raw({ pulses: out })).intervals.driftBpmPerHour) < 0.5);
});

test("stalls, a stamp that goes backwards, an equal stamp, and a burst (several pulses seen at once) are counted and kept out of the periods", () => {
  const p = pulses(120, 20);
  const base = p.length;
  // A stall: nothing for a second after pulse 100.
  const stalled = p.map((x, i) => (i > 100 ? { stamp: x.stamp + 1_000, seen: x.seen + 1_000 } : x));
  const s = analyseFollow(raw({ pulses: stalled }));
  assert.equal(s.intervals.stalls, 1);
  assert.ok(Math.abs(s.intervals.periodMs.max - 2500 / 120) < 1e-6, "the stall is not a period");
  assert.equal(s.pulses, base);
  // A stamp that goes back, and an equal one.
  const odd = p.map((x) => ({ ...x }));
  odd[50]!.stamp = odd[49]!.stamp - 3;
  odd[70]!.stamp = odd[69]!.stamp;
  odd[90]!.stamp = odd[89]!.stamp - 0.5; // half a millisecond back is back
  const o = analyseFollow(raw({ pulses: odd }));
  assert.equal(o.intervals.backwards, 2);
  assert.equal(o.intervals.equalStamps, 1);
  // Four pulses seen in the same instant: the browser handed them over together.
  const burst = p.map((x) => ({ ...x }));
  for (let i = 200; i < 204; i++) burst[i]!.seen = burst[203]!.stamp + 0.2;
  const b = analyseFollow(raw({ pulses: burst }));
  assert.equal(b.intervals.bursts, 3, "three gaps of under a millisecond between pulses whose stamps are 20 ms apart");
  // Pulses whose stamps are also together were sent together: that is the sender's burst, not the browser handing them over late.
  const together = p.map((x) => ({ ...x }));
  together[300]!.stamp = together[299]!.stamp + 2;
  together[300]!.seen = together[299]!.seen + 0.5;
  assert.equal(analyseFollow(raw({ pulses: together })).intervals.bursts, 0);
});

test("the phase: samples while the transport runs and the clock is not lost, after the settling, against a fifth of a step", () => {
  // 120 BPM: a step is 31.25 ms, a fifth of it is 6.25 ms.
  const messages = [{ name: "start" as const, stamp: 1_000, seen: 1_000 }];
  const samples: FollowSample[] = [];
  for (let i = 0; i < 100; i++) samples.push(sample(1_000 + i * 250, { phaseMs: i % 10 === 0 ? 7 : 1.5 }));
  samples.push(sample(30_000, { phase: "lost", phaseMs: 400 }), sample(30_250, { running: false, phaseMs: null }), sample(30_500, { phaseMs: null }), sample(30_750, { running: false, phaseMs: 3 }));
  const a = analyseFollow(raw({ pulses: pulses(120, 30), samples, messages }), { settleMs: 3_000 });
  assert.ok(Math.abs(a.phase.fifthOfStepMs - 6.25) < 0.05, `${a.phase.fifthOfStepMs}`);
  // The first 3 s after the Start are 12 samples (at 1000 to 3750): 100 - 12 = 88 count, ten of every hundred are 7 ms.
  assert.equal(a.phase.counted, 88);
  assert.equal(a.phase.beyondFifth, samples.slice(12, 100).filter((s) => s.phaseMs === 7).length);
  assert.ok(Math.abs(a.phase.share - a.phase.beyondFifth / 88) < 1e-12);
  assert.equal(a.phase.maxAbsMs, 7, "the lost sample's 400 ms is not counted");
  assert.equal(a.phase.skipped.settling, 12);
  assert.equal(a.phase.skipped.lost, 1);
  assert.equal(a.phase.skipped.stopped, 3, "a stopped transport, a sample with no phase, and a stopped transport that still has an old phase");
});

test("a sample taken while the follower is itself still waiting for a clock or settling on one is not counted, whatever the Start says", () => {
  const messages = [{ name: "start" as const, stamp: 0, seen: 0 }];
  const samples = [sample(5_000, { phase: "waiting", phaseMs: 2 }), sample(6_000, { phase: "settling", phaseMs: 2 }), sample(7_000, { phase: "following", phaseMs: 2 })];
  const a = analyseFollow(raw({ pulses: pulses(120, 8), samples, messages }), { settleMs: 3_000 });
  assert.equal(a.phase.counted, 1);
  assert.equal(a.phase.skipped.settling, 2);
});

test("the phase is counted from the latest Start or Continue, so a second Start has its own settling", () => {
  const messages = [{ name: "start" as const, stamp: 0, seen: 0 }, { name: "stop" as const, stamp: 10_000, seen: 10_000 }, { name: "continue" as const, stamp: 20_000, seen: 20_000 }];
  const samples = [sample(1_000), sample(3_500), sample(21_000), sample(22_500), sample(24_000)];
  const a = analyseFollow(raw({ pulses: pulses(120, 30), samples, messages }), { settleMs: 3_000 });
  assert.equal(a.phase.counted, 2, "the ones at 3500 and 24000");
  assert.equal(a.phase.skipped.settling, 3);
});

test("the clock map: a step of the audio-to-page offset between two samples of more than 3 ms is a jump, counted with the largest", () => {
  const messages = [{ name: "start" as const, stamp: 0, seen: 0 }];
  // Offsets of 100 ms, with a jump of 10 ms at the 5th sample that stays, and one of 25 ms at the 12th that goes back at the 13th.
  const maps = [100, 100.1, 99.9, 100, 110, 110.1, 110, 110.2, 109.9, 110, 110.1, 135, 110, 110, 110.1, 110];
  const samples = maps.map((m, i) => sample(4_000 + i * 250, { mapMs: m }));
  const a = analyseFollow(raw({ pulses: pulses(120, 10), samples, messages }), { settleMs: 3_000 });
  assert.equal(a.clockMap.samples, 16);
  assert.equal(a.clockMap.jumps, 3, "up 10, up 25, down 25");
  assert.ok(Math.abs(a.clockMap.maxJumpMs - 25) < 1e-9, `${a.clockMap.maxJumpMs}`);
  assert.ok(Math.abs(a.clockMap.rangeMs - 35.1) < 0.2, `${a.clockMap.rangeMs}`);
  // Samples with no map in them (a file from before the page recorded it) say nothing, and are not zero.
  const none = analyseFollow(raw({ pulses: pulses(120, 10), samples: [sample(4_000), sample(5_000)], messages }), { settleMs: 3_000 });
  assert.deepEqual([none.clockMap.samples, none.clockMap.jumps, none.clockMap.maxJumpMs, none.clockMap.rangeMs], [0, 0, 0, 0]);
});

test("a phase beyond a fifth of a step within two seconds after a jump of the clock map is attributed to it, and one that is not is not", () => {
  const messages = [{ name: "start" as const, stamp: 0, seen: 0 }];
  const samples: FollowSample[] = [];
  // 60 samples a quarter second apart from 4 s: the map jumps 10 ms at the 8th (6 s) and the phase goes to -10, decays by 2 ms a sample, and is
  // within a fifth again at the 3rd sample after. A second excursion at the 50th sample (16.25 s) has no jump before it.
  for (let i = 0; i < 60; i++) {
    const afterJump = i >= 8;
    const phase = i >= 8 && i < 13 ? -10 + (i - 8) * 2 : i === 50 ? 9 : 0.5;
    samples.push(sample(4_000 + i * 250, { mapMs: afterJump ? 110 : 100, phaseMs: phase }));
  }
  const a = analyseFollow(raw({ pulses: pulses(120, 20), samples, messages }), { settleMs: 3_000 });
  assert.equal(a.phase.beyondFifth, 2 + 1, "-10 and -8 after the jump, and the 9 at the 50th (-6, -4 and -2 are within a fifth of 6.25)");
  assert.equal(a.phase.beyondFifthNearMapJump, 2);
  assert.equal(a.clockMap.jumps, 1);
});

test("the hidden tab is a filter: the same raw data, read for the samples taken while it was hidden, or while it was not", () => {
  const messages = [{ name: "start" as const, stamp: 0, seen: 0 }];
  const samples = [sample(4_000, { phaseMs: 1 }), sample(5_000, { phaseMs: 2 }), sample(6_000, { phaseMs: 20, hidden: true }), sample(7_000, { phaseMs: 30, hidden: true })];
  const all = analyseFollow(raw({ pulses: pulses(120, 8), samples, messages }), { settleMs: 3_000 });
  const visible = analyseFollow(raw({ pulses: pulses(120, 8), samples, messages }), { settleMs: 3_000, hidden: false });
  const hidden = analyseFollow(raw({ pulses: pulses(120, 8), samples, messages }), { settleMs: 3_000, hidden: true });
  assert.deepEqual([all.phase.counted, visible.phase.counted, hidden.phase.counted], [4, 2, 2]);
  assert.deepEqual([all.phase.maxAbsMs, visible.phase.maxAbsMs, hidden.phase.maxAbsMs], [30, 2, 30]);
});

test("the tempo the follower read, against the one the sender was set to when it is given", () => {
  const messages = [{ name: "start" as const, stamp: 0, seen: 0 }];
  const samples = [sample(4_000, { bpm: 120.1 }), sample(5_000, { bpm: 119.9 }), sample(6_000, { bpm: 120.3 })];
  const given = analyseFollow(raw({ pulses: pulses(120, 8), samples, messages }), { settleMs: 3_000, nominalBpm: 120 });
  assert.ok(Math.abs(given.tempo.meanErrorBpm! - 0.1) < 1e-9);
  assert.ok(Math.abs(given.tempo.maxAbsErrorBpm! - 0.3) < 1e-9);
  assert.equal(analyseFollow(raw({ pulses: pulses(120, 8), samples, messages })).tempo.meanErrorBpm, null);
});

test("a run that is too short, or empty, says so and does not divide by zero", () => {
  const empty = analyseFollow(raw());
  assert.equal(empty.pulses, 0);
  assert.equal(empty.phase.counted, 0);
  assert.equal(empty.phase.share, 0);
  assert.equal(empty.intervals.tempoBpm.median, 0);
  assert.equal(empty.intervals.driftBpmPerHour, 0);
  assert.ok(Number.isFinite(analyseFollow(raw({ pulses: pulses(120, 1) })).intervals.driftBpmPerHour));
  // Two blocks of a minute are not a line to fit: it is 0, not a slope through two noisy points.
  assert.equal(analyseFollow(raw({ pulses: pulses(120, 130, { noise: 1 }) })).intervals.driftBpmPerHour, 0);
});

test("the loop: each pulse sent is paired with the one heard at the same place in the order, and the latency and the interval error are read from the pairs", () => {
  // Sent 20 ms apart (given as timestamps), heard 3 ms after, with one of them 2 ms late.
  const sent = [0, 1, 2, 3, 4].map((i) => ({ target: 1_000 + i * 20 }));
  const heard = [1_003, 1_023, 1_045, 1_063, 1_083].map((at) => ({ at }));
  const a = analyseLoop(sent, heard);
  assert.deepEqual([a.sent, a.received, a.matched, a.missing, a.extra], [5, 5, 5, 0, 0]);
  assert.deepEqual([a.latencyMs.min, a.latencyMs.max], [3, 5]);
  assert.equal(a.latencyMs.mean, 3.4);
  assert.equal(a.intervalErrorMs.n, 4);
  assert.deepEqual([a.intervalErrorMs.min, a.intervalErrorMs.max], [-2, 2]);
});

test("the loop: pulses that never came back are missing, and pulses heard that were never sent are extra, counted from the end", () => {
  const sent = [0, 1, 2, 3].map((i) => ({ target: 1_000 + i * 20 }));
  const fewer = analyseLoop(sent, [{ at: 1_003 }, { at: 1_023 }]);
  assert.deepEqual([fewer.matched, fewer.missing, fewer.extra], [2, 2, 0]);
  const more = analyseLoop(sent, [1_003, 1_023, 1_043, 1_063, 1_083, 1_103].map((at) => ({ at })));
  assert.deepEqual([more.matched, more.missing, more.extra], [4, 0, 2]);
  assert.deepEqual([analyseLoop([], []).matched, analyseLoop([], []).latencyMs.n], [0, 0]);
});

test("saved pulses, as [stamp, seen] pairs, come back as pulses", () => {
  assert.deepEqual(unpackPulses([[1, 3], [21.5, 23.25]]), [{ stamp: 1, seen: 3 }, { stamp: 21.5, seen: 23.25 }]);
  assert.deepEqual(unpackPulses([]), []);
});

/** A run of `seconds` at 120 BPM whose samples are all `phaseMs` out, after a Start at 0. */
function run(seconds: number, phaseMs: (i: number) => number, over: { age?: number } = {}): FollowRaw {
  const samples: FollowSample[] = [];
  for (let i = 0; i < seconds * 4; i++) samples.push(sample(i * 250, { phaseMs: phaseMs(i) }));
  return raw({ pulses: pulses(120, seconds, { age: over.age ?? 2 }), samples, messages: [{ name: "start", stamp: 0, seen: 0 }] });
}

test("the verdict: every sample inside a fifth of a step is the plan's criterion met, and a run shorter than 30 minutes says it is not the S3 run", () => {
  const good = followVerdict(analyseFollow(run(60, () => 2)), 60, 0);
  assert.equal(good.met, true);
  assert.match(good.lines.join("\n"), /shorter than the 30 minutes/);
  const long = followVerdict(analyseFollow(run(1800, () => 2)), 1800, 0);
  assert.equal(long.met, true);
  assert.doesNotMatch(long.lines.join("\n"), /shorter than/);
  // One sample at 7 ms in a fifth of 6.25 is not met, and the sentence says how many and how far.
  const bad = followVerdict(analyseFollow(run(60, (i) => (i === 100 ? 7 : 2))), 60, 0);
  assert.equal(bad.met, false);
  assert.match(bad.lines.join("\n"), /1 of \d+ samples/);
  assert.match(bad.lines.join("\n"), /7\.0 ms/);
});

test("the verdict says when every excursion beyond a fifth of a step follows a jump of the browser's clock map, and when some do not", () => {
  const phases = (i: number): number => (i >= 8 && i < 11 ? -10 : 1);
  const make = (extra: (i: number) => number): FollowRaw => {
    const samples: FollowSample[] = [];
    for (let i = 0; i < 60; i++) samples.push(sample(4_000 + i * 250, { mapMs: i >= 8 ? 110 : 100, phaseMs: phases(i) + extra(i) }));
    return raw({ pulses: pulses(120, 20), samples, messages: [{ name: "start", stamp: 0, seen: 0 }] });
  };
  const all = followVerdict(analyseFollow(make(() => 0)), 20, 0);
  assert.equal(all.met, false);
  assert.match(all.lines.join("\n"), /All 3 of them follow a jump/);
  const some = followVerdict(analyseFollow(make((i) => (i === 50 ? 9 : 0))), 20, 0);
  assert.match(some.lines.join("\n"), /3 of the 4 follow a jump/);
  assert.doesNotMatch(followVerdict(analyseFollow(run(60, () => 2)), 60, 0).lines.join("\n"), /jump/);
});

test("the verdict cannot be met or missed when there is nothing to read: stamps on another clock, no pulses, no Start heard", () => {
  const other = followVerdict(analyseFollow(run(60, () => 2, { age: 1_000_000 })), 60, 0);
  assert.equal(other.met, null);
  assert.match(other.lines.join("\n"), /not on the page's clock/i);
  assert.equal(followVerdict(analyseFollow(raw()), 0, 0).met, null);
  const noStart = analyseFollow(raw({ pulses: pulses(120, 30), samples: [sample(5_000, { running: false, phaseMs: null })] }));
  const v = followVerdict(noStart, 30, 0);
  assert.equal(v.met, null);
  assert.match(v.lines.join("\n"), /no phase was counted/i);
});

test("the verdict says how much of the run the tab was hidden, because the plan's criterion is for a visible tab", () => {
  const v = followVerdict(analyseFollow(run(60, () => 2)), 60, 40);
  assert.match(v.lines.join("\n"), /hidden for 40 s of 60/);
});

/** Pulses of an engine at `bpm`: `audioTime` in seconds, and a `target` that is the audio time on the page's clock (offset `mapMs`) plus a lookahead of 30 ms. */
function sentPulses(n: number, bpm: number, mapMs: (i: number) => number, late: (i: number) => boolean = () => false): { target: number; audioTime: number; marginMs: number; hidden: boolean }[] {
  return Array.from({ length: n }, (_, i) => {
    const audioTime = (i * 2.5) / bpm;
    return { target: audioTime * 1000 + mapMs(i) + 30, audioTime, marginMs: late(i) ? -3 : 60, hidden: false };
  });
}

test("the stamp map: the engine's own tempo from its audio times, and the jumps of the clock map the stamps were made with", () => {
  const steady = analyseStampMap(sentPulses(200, 120, () => 500), 30);
  assert.equal(steady.pulses, 200);
  assert.ok(Math.abs(steady.engineBpm - 120) < 1e-9, `${steady.engineBpm}`);
  assert.deepEqual([steady.jumps, steady.maxJumpMs, steady.totalJumpMs], [0, 0, 0]);
  assert.ok(steady.rangeMs < 1e-6, `a steady map has no range beyond arithmetic: ${steady.rangeMs}`);
  // The map moves 25 ms at the 11th pulse and 4 ms back at the 51st.
  const moved = analyseStampMap(sentPulses(200, 120, (i) => (i >= 50 ? 521 : i >= 10 ? 525 : 500)), 30);
  assert.equal(moved.jumps, 2);
  assert.ok(Math.abs(moved.maxJumpMs - 25) < 1e-6 && Math.abs(moved.totalJumpMs - 29) < 1e-6 && Math.abs(moved.rangeMs - 25) < 1e-6, JSON.stringify(moved));
  assert.ok(Math.abs(moved.engineBpm - 120) < 1e-9, "the engine's tempo does not depend on the map");
  // A pulse that went at once (it was late) has the time of the call and not a stamp, so it says nothing about the map.
  const late = analyseStampMap(sentPulses(200, 120, (i) => (i === 7 ? 9_000 : 500), (i) => i === 7), 30);
  assert.equal(late.jumps, 0);
  assert.equal(late.pulses, 199);
  assert.deepEqual([analyseStampMap([], 30).pulses, analyseStampMap([], 30).engineBpm], [0, 0]);
});

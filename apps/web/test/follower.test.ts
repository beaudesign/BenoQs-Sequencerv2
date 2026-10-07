// The follower (SPEC-0002 P4 plan observables M4 and M5, ADR-0009 decision 5): what a clock arriving from outside does to the engine's tempo
// and transport. The estimator is tested on its own (clock-estimator.test.ts); here it is the whole loop: pulses and 0xFA, 0xFB and 0xFC in,
// `setTempo` and `transport` out, and the engine's position coming back.
//
// Two kinds of test. The first kind drives the follower with a recorder in place of the engine and says what it calls and when. The second
// kind closes the loop on a simulated engine (test/support/plant.ts) and a simulated sender (test/support/sim-clock.ts) and measures the
// HEARD error: where the engine's notes land against the sender's beat, in ticks (an eighth of a pulse: 2.6 ms at 120 BPM). Both
// simulations are made up; the real engine is in the loop in follower-engine.test.ts and a real sender is spike S3's.
//
// THE NUMBERS ARE PROPOSALS AND THE OWNER'S TO CHANGE (plan D-P4-10). Where one was changed after it was measured, the evidence file says so
// (handoffs/evidence/p4d-follower-measured.txt).
import assert from "node:assert/strict";
import { test } from "node:test";
import { TICKS_PER_CLOCK } from "../src/abi.ts";
import { Follower, type FollowerActions } from "../src/follower.ts";
import { ramp, simulate, steady, step } from "./support/sim-clock.ts";
import { HEARD_TICKS_PER_PULSE, runLoop, worstTicks } from "./support/loop.ts";

// D0: tick is 48 to the quarter note, so an engine tick is now half a pulse (10.4 ms at 120 BPM). The "ticks" the tolerances below are in are
// NOT engine ticks: they are the heard error's unit, an eighth of a pulse (HEARD_TICKS_PER_PULSE in support/loop.ts), which is what they
// were in time before D0 (when it was also one engine tick). The numbers are unchanged; in engine ticks they would be four times looser.

// ----- the proposals -----
const STEADY_TICKS = 1; // from SETTLE_S after Start (the plan's "within one tick"; the first draft said 1.5, and it was tightened to this once it was measured on 40 seeds the tests do not use)
const SETTLE_S = 2;
const CATCH_UP_TICKS = 1; // from CATCH_UP_S after Start: the engine starts a lookahead late and has to catch up (the first draft said 2)
const CATCH_UP_S = 1.5;
const START_TRANSIENT_MS = 70; // never worse than this, at any time after Start, with a 30 ms lookahead
const SLOW_RAMP_TICKS = 3;
const FAST_RAMP_TICKS = 4;
const AFTER_STEP_S = 3;

const BAD_TEMPO_BOUND = 0.02;

class Recorder implements FollowerActions {
  readonly calls: { what: "tempo" | "transport"; value: number | boolean; at: number }[] = [];
  now = 0;
  setTempo(bpm: number): void {
    this.calls.push({ what: "tempo", value: bpm, at: this.now });
  }
  transport(play: boolean): void {
    this.calls.push({ what: "transport", value: play, at: this.now });
  }
  get tempos(): number[] {
    return this.calls.filter((c) => c.what === "tempo").map((c) => c.value as number);
  }
  get transports(): boolean[] {
    return this.calls.filter((c) => c.what === "transport").map((c) => c.value as boolean);
  }
}

const PERIOD_120 = 2500 / 120;

function feedPulses(f: Follower, r: Recorder, from: number, count: number, period = PERIOD_120): number {
  let t = from;
  for (let i = 0; i < count; i++) {
    r.now = t;
    f.onClock(t);
    t += period;
  }
  return t;
}

// ----- transport (M5) -----

test("M5: a clock alone never starts the transport", () => {
  const r = new Recorder();
  const f = new Follower(r);
  feedPulses(f, r, 1_000, 120);
  assert.deepEqual(r.transports, []);
  assert.equal(f.running, false);
});

test("M5: 0xFA starts the transport at the next pulse, not at the message", () => {
  const r = new Recorder();
  const f = new Follower(r);
  feedPulses(f, r, 1_000, 30);
  r.now = 1_700;
  f.onStart(1_700);
  assert.deepEqual(r.transports, [], "the message alone starts nothing");
  assert.equal(f.running, false);
  r.now = 1_705;
  f.onClock(1_705);
  assert.deepEqual(r.transports, [true], "the pulse after it starts the engine, once");
  assert.equal(r.calls.filter((c) => c.what === "transport")[0]!.at, 1_705);
  assert.equal(f.running, true);
  feedPulses(f, r, 1_726, 50);
  assert.deepEqual(r.transports, [true], "and later pulses do not start it again");
});

test("M5: 0xFB continues the transport the same way, and 0xFC stops it at once", () => {
  const r = new Recorder();
  const f = new Follower(r);
  let t = feedPulses(f, r, 1_000, 30);
  f.onContinue(t);
  assert.deepEqual(r.transports, []);
  t = feedPulses(f, r, t + 1, 10);
  assert.deepEqual(r.transports, [true]);
  r.now = t;
  f.onStop(t);
  assert.deepEqual(r.transports, [true, false], "the Stop is not held for a pulse");
  assert.equal(f.running, false);
  t = feedPulses(f, r, t + 1, 24);
  assert.deepEqual(r.transports, [true, false], "pulses after a Stop do not start it");
});

test("M5: a Stop before the pulse that would have started the transport cancels the start", () => {
  const r = new Recorder();
  const f = new Follower(r);
  const t = feedPulses(f, r, 1_000, 30);
  f.onStart(t);
  f.onStop(t + 1);
  feedPulses(f, r, t + 2, 10);
  assert.deepEqual(r.transports, [false], "only the Stop was passed on, and no pulse started anything");
  assert.equal(f.running, false);
});

test("a Start while the transport is already running does not start it again", () => {
  const r = new Recorder();
  const f = new Follower(r);
  let t = feedPulses(f, r, 1_000, 30);
  f.onStart(t);
  t = feedPulses(f, r, t + 1, 30);
  f.onStart(t);
  t = feedPulses(f, r, t + 1, 30);
  assert.deepEqual(r.transports, [true]);
  assert.equal(f.running, true);
});

test("a Stop while stopped is still passed on: the engine's Stop clears hanging notes whatever state it was in", () => {
  const r = new Recorder();
  const f = new Follower(r);
  f.onStop(1_000);
  f.onStop(1_100);
  assert.deepEqual(r.transports, [false, false]);
});

test("feed routes the four real-time names to the four handlers", () => {
  const r = new Recorder();
  const f = new Follower(r);
  let t = 1_000;
  for (let i = 0; i < 30; i++) {
    f.feed("clock", t);
    t += PERIOD_120;
  }
  f.feed("start", t);
  f.feed("clock", t + 1);
  f.feed("stop", t + 2);
  f.feed("continue", t + 3);
  f.feed("clock", t + 4);
  assert.deepEqual(r.transports, [true, false, true]);
});

// ----- tempo -----

test("the tempo follows the estimate, and nothing is set before there is an estimate", () => {
  const r = new Recorder();
  const f = new Follower(r);
  feedPulses(f, r, 1_000, 3, 2500 / 90);
  assert.deepEqual(r.tempos, [], "three pulses are not a tempo");
  feedPulses(f, r, 1_000 + 3 * (2500 / 90), 60, 2500 / 90);
  const last = r.tempos[r.tempos.length - 1]!;
  assert.ok(Math.abs(last - 90) < 0.05, `the tempo set was ${last}`);
  assert.equal(f.appliedTempo, last);
});

test("the tempo follows while the transport is stopped, so the first bar starts at the right tempo", () => {
  const r = new Recorder();
  const f = new Follower(r);
  feedPulses(f, r, 1_000, 120, 2500 / 100);
  assert.ok(Math.abs(r.tempos[r.tempos.length - 1]! - 100) < 0.05);
  assert.deepEqual(r.transports, []);
});

test("a change of less than the minimum is not sent: a steady clock does not flood the engine with the same tempo", () => {
  const r = new Recorder();
  const f = new Follower(r);
  feedPulses(f, r, 1_000, 240, PERIOD_120);
  const settled = r.tempos.slice(-100);
  assert.ok(settled.length < 10, `${settled.length} tempo messages in the last 100 pulses of a steady clock`);
});

test("a tempo the engine would refuse (above 999 BPM) is never sent; below 5 BPM a pulse is more than 500 ms apart and never locks at all", () => {
  for (const period of [2500 / 1500, 2500 / 1000.5]) {
    const r = new Recorder();
    const f = new Follower(r);
    feedPulses(f, r, 1_000, 200, period);
    for (const t of r.tempos) assert.ok(t >= 1 && t <= 999, `${t} BPM was sent`);
  }
});

test("a tempo is not changed while the clock is lost: the engine plays on at the last one", () => {
  const r = new Recorder();
  const f = new Follower(r);
  const end = feedPulses(f, r, 1_000, 120);
  const before = r.calls.length;
  for (let now = end; now < end + 3_000; now += 25) {
    r.now = now;
    f.onPosition({ ticks: 1_000 + now / 10, pageTimeMs: now });
    f.status(now);
  }
  assert.equal(r.calls.length, before, "nothing was sent while no pulse came");
});

test("M5: with no clock for 500 ms the status says so, and it keeps the last tempo", () => {
  const r = new Recorder();
  const f = new Follower(r);
  assert.equal(f.status(0).phase, "waiting", "nothing yet");
  let t = 1_000;
  const phases: string[] = [];
  for (let i = 0; i < 40; i++) {
    r.now = t;
    f.onClock(t);
    phases.push(f.status(t).phase);
    t += PERIOD_120;
  }
  assert.equal(phases[0], "settling");
  assert.equal(phases[phases.length - 1], "following");
  const last = t - PERIOD_120;
  assert.equal(f.status(last + 500).phase, "following", "500 ms is not yet a stall");
  const lost = f.status(last + 501);
  assert.equal(lost.phase, "lost");
  assert.ok(lost.bpm !== null && Math.abs(lost.bpm - 120) < 0.2, `the last tempo, still reported: ${lost.bpm}`);
  assert.equal(f.status(last + 10_000).phase, "lost");
  assert.ok(Math.abs(f.appliedTempo! - 120) < 0.2, "and the engine still has it");
});

test("the status carries the tempo, the pulses heard and the jitter, for the settings view's sentence", () => {
  const r = new Recorder();
  const f = new Follower(r);
  const sim = simulate({ seed: 4, seconds: 5, bpm: steady(120), jitter: { shape: "uniform", ms: 3 } });
  for (const p of sim.pulses) {
    r.now = p.at;
    f.onClock(p.at);
  }
  const s = f.status(sim.pulses[sim.pulses.length - 1]!.at);
  assert.equal(s.phase, "following");
  assert.ok(Math.abs(s.bpm! - 120) < 0.3);
  assert.ok(s.pulses > 200);
  assert.ok(s.jitterMs > 1 && s.jitterMs < 2.5, `jitter ${s.jitterMs}`);
  assert.equal(s.running, false);
  assert.equal(s.phaseMs, null, "no phase to report while the transport is stopped");
});

test("bad input is ignored: a position or a message that is not a number changes nothing", () => {
  const r = new Recorder();
  const f = new Follower(r);
  const t = feedPulses(f, r, 1_000, 40);
  const before = JSON.stringify(r.calls);
  f.onPosition({ ticks: NaN, pageTimeMs: t });
  f.onPosition({ ticks: 10, pageTimeMs: Infinity });
  f.onClock(NaN);
  f.setLookahead(NaN);
  f.setOffset(Infinity);
  assert.equal(JSON.stringify(r.calls), before);
  assert.doesNotThrow(() => f.status(t));
});

test("the trim is at most 8% either way, and it is used in full when the engine is a long way off", () => {
  // An engine five pulses (104 ms) behind the grid or ahead of it asks for far more than 8%: it gets 8%, and not what the loop asked for.
  // Found as a survivor in the mutation run for P4d ("trim unbounded" and "no trim" were both caught only by the loop's tolerances).
  for (const [direction, expected] of [[-1, 1.08], [1, 0.92]] as const) {
    const r = new Recorder();
    const f = new Follower(r);
    f.onPosition({ ticks: 100 * TICKS_PER_CLOCK, pageTimeMs: 900 }); // the engine is at pulse 100 when the Start comes (D0: tick is 48 to the quarter note, a pulse is 2 ticks, not 8)
    let t = feedPulses(f, r, 1_000, 30);
    f.onStart(t);
    r.now = t + 1;
    f.onClock(t + 1);
    t = feedPulses(f, r, t + 22, 40);
    const at = t + 20;
    const grid = f.estimator.pulseIndex(at + 30)!; // the sender's pulse the engine should be at, with the default 30 ms lookahead
    const shouldBe = 100 + (grid - 30); // the engine was anchored at pulse 100 on the sender's pulse 30, the one that came after the Start
    f.onPosition({ ticks: (shouldBe + direction * 5) * TICKS_PER_CLOCK, pageTimeMs: at }); // D0: tick is 48 to the quarter note, a pulse is 2 ticks, not 8
    const ratio = r.tempos[r.tempos.length - 1]! / f.estimator.bpm!;
    assert.ok(Math.abs(ratio - expected) < 1e-9, `${direction > 0 ? "ahead" : "behind"}: the tempo was ${ratio.toFixed(4)} times the sender's, not ${expected}`);
  }
});

// ----- the loop closed (M4, M5) -----

const jitter = { shape: "uniform", ms: 3 } as const;

test("M4: steady 120 BPM from a Start: the engine's notes land within a tick of the sender's beat from two seconds on", () => {
  for (let seed = 1; seed <= 10; seed++) {
    const { samples } = runLoop({ sim: simulate({ seed, seconds: 12, bpm: steady(120), jitter }), plant: { seed } });
    assert.ok(worstTicks(samples, SETTLE_S * 1000) <= STEADY_TICKS, `seed ${seed}: ${worstTicks(samples, SETTLE_S * 1000).toFixed(2)} ticks`);
  }
});

test("M4: the engine starts a lookahead late and catches up: within a tick by 1.5 s, and never worse than the lookahead and a message", () => {
  for (let seed = 1; seed <= 10; seed++) {
    const { samples } = runLoop({ sim: simulate({ seed, seconds: 8, bpm: steady(120), jitter }), plant: { seed } });
    assert.ok(worstTicks(samples, CATCH_UP_S * 1000) <= CATCH_UP_TICKS, `seed ${seed}: ${worstTicks(samples, CATCH_UP_S * 1000).toFixed(2)} ticks`);
    const running = samples.filter((s) => s.running);
    assert.ok(Math.max(...running.map((s) => Math.abs(s.heardMs))) <= START_TRANSIENT_MS, `seed ${seed}: the start transient`);
  }
});

test("M5: the engine starts at the first pulse after the Start, and plays at the sender's tempo from the first bar", () => {
  const { calls, plant } = runLoop({ sim: simulate({ seed: 3, seconds: 6, bpm: steady(90), jitter }) });
  const transports = calls.filter((c) => c.what === "transport");
  assert.deepEqual(transports.map((c) => c.value), [true]);
  assert.ok(Math.abs(plant.bpm - 90) < 0.6, `the engine ended at ${plant.bpm} BPM`);
});

test("M4: an engine that starts at the wrong tempo (120) under a sender at 160 or 70 is brought to it and held", () => {
  for (const bpm of [160, 70]) {
    const { samples, plant } = runLoop({ sim: simulate({ seed: 2, seconds: 12, bpm: steady(bpm), jitter }) });
    assert.ok(worstTicks(samples, 2_500) <= STEADY_TICKS, `${bpm} BPM: ${worstTicks(samples, 2_500).toFixed(2)} ticks`);
    assert.ok(Math.abs(plant.bpm - bpm) < 0.8, `the engine ended at ${plant.bpm} BPM, the sender at ${bpm}`);
  }
});

test("the lead: a larger lookahead and an offset move the engine so the sender's beat is still met (60 ms lookahead, +10 ms offset)", () => {
  for (const settings of [{ lookaheadMs: 60, offsetMs: 10 }, { lookaheadMs: 0, offsetMs: 0 }, { lookaheadMs: 30, offsetMs: -15 }]) {
    const { samples } = runLoop({ sim: simulate({ seed: 5, seconds: 14, bpm: steady(120), jitter }), settings });
    assert.ok(worstTicks(samples, 3_500) <= STEADY_TICKS, `${JSON.stringify(settings)}: ${worstTicks(samples, 3_500).toFixed(2)} ticks`);
  }
});

test("the lead can be changed while it runs: a new offset is taken within two and a half seconds", () => {
  // The script's time is page time, so the windows are in page time too: the sample at the moment of the change is read against the new
  // target at once, and the engine has not moved, so it is a whole offset out by construction and is not "before".
  const CHANGE_AT = 7_000;
  const { samples } = runLoop({
    sim: simulate({ seed: 6, seconds: 16, bpm: steady(120), jitter }),
    script: [{ at: CHANGE_AT, run: (f) => f.setOffset(20) }],
  });
  const ticks = (from: number, to: number): number =>
    Math.max(...samples.filter((s) => s.running && s.sinceStart >= SETTLE_S * 1000 && s.at >= from && s.at < to).map((s) => Math.abs(s.heardMs) / s.tickMs));
  assert.ok(ticks(0, CHANGE_AT) <= STEADY_TICKS, "steady before");
  assert.ok(ticks(CHANGE_AT, CHANGE_AT + 100) > 3, "and the change is a real one: the sample at the moment of it is far from the new target");
  assert.ok(ticks(CHANGE_AT + 2_500, Infinity) <= STEADY_TICKS, "steady after");
});

test("M4: a slow ramp (100 to 120 BPM over 20 s) is followed", () => {
  for (let seed = 1; seed <= 5; seed++) {
    const { samples } = runLoop({ sim: simulate({ seed, seconds: 24, bpm: ramp(100, 120, 20), jitter }), plant: { seed } });
    assert.ok(worstTicks(samples, 3_000) <= SLOW_RAMP_TICKS, `seed ${seed}: ${worstTicks(samples, 3_000).toFixed(2)} ticks`);
  }
});

test("M4: a fast ramp (100 to 140 BPM over 8 s) is followed, and the engine settles after it", () => {
  for (let seed = 1; seed <= 5; seed++) {
    const { samples } = runLoop({ sim: simulate({ seed, seconds: 18, bpm: ramp(100, 140, 8), jitter }), plant: { seed } });
    assert.ok(worstTicks(samples, 3_000, 9_000) <= FAST_RAMP_TICKS, `seed ${seed}: on the ramp ${worstTicks(samples, 3_000, 9_000).toFixed(2)} ticks`);
    assert.ok(worstTicks(samples, 8_000 + AFTER_STEP_S * 1000) <= STEADY_TICKS, `seed ${seed}: after it ${worstTicks(samples, 11_000).toFixed(2)} ticks`);
  }
});

test("M4: seeds the tuning never saw: steady 120 BPM and a step to 90, forty seeds each, are held to the same tick", () => {
  // The first draft passed on seeds 1 to 12 and put 18 of 40 steady runs on seeds 100 to 139 a whole pulse out: the harness's Start came
  // after the first pulse. These seeds are in the test so that it cannot happen again unseen (handoffs/evidence/p4d-follower-measured.txt).
  for (let seed = 100; seed < 140; seed++) {
    const steadyRun = runLoop({ sim: simulate({ seed, seconds: 8, bpm: steady(120), jitter }), plant: { seed } });
    assert.ok(worstTicks(steadyRun.samples, SETTLE_S * 1000) <= STEADY_TICKS, `steady, seed ${seed}: ${worstTicks(steadyRun.samples, SETTLE_S * 1000).toFixed(2)} ticks`);
    const stepRun = runLoop({ sim: simulate({ seed, seconds: 14, bpm: step(120, 90, 5), jitter }), plant: { seed } });
    assert.ok(worstTicks(stepRun.samples, 5_000 + AFTER_STEP_S * 1000) <= STEADY_TICKS, `step, seed ${seed}: ${worstTicks(stepRun.samples, 8_000).toFixed(2)} ticks`);
  }
});

test("M4: a step in tempo (120 to 90 BPM) is taken: back within a tick three seconds after", () => {
  for (let seed = 1; seed <= 5; seed++) {
    const { samples } = runLoop({ sim: simulate({ seed, seconds: 16, bpm: step(120, 90, 7), jitter }), plant: { seed } });
    assert.ok(worstTicks(samples, 7_000 + AFTER_STEP_S * 1000) <= STEADY_TICKS, `seed ${seed}: ${worstTicks(samples, 10_000).toFixed(2)} ticks`);
  }
});

test("M4: a step up, a step down, a doubling and a halving are taken too, on the beat and not a whole pulse off it", () => {
  // The first step test alone passed with the estimator counting one pulse too many after the step (it read a late pulse as a missing one):
  // the follower's own phase read zero while the heard error was a whole pulse. These steps are the ones that went wrong then, and a whole
  // pulse is 8 ticks, so the bound that holds for 120 to 90 holds for them.
  for (const [from, to] of [[90, 120], [100, 140], [150, 100], [120, 60], [60, 120]] as const) {
    for (let seed = 1; seed <= 4; seed++) {
      const { samples } = runLoop({ sim: simulate({ seed, seconds: 16, bpm: step(from, to, 7), jitter }), plant: { seed } });
      const worst = worstTicks(samples, 7_000 + AFTER_STEP_S * 1000);
      assert.ok(worst <= STEADY_TICKS, `${from} to ${to} BPM, seed ${seed}: ${worst.toFixed(2)} ticks`);
    }
  }
});

test("M4: missing pulses and doubled pulses do not move the engine off the beat", () => {
  for (let seed = 1; seed <= 5; seed++) {
    const sim = simulate({
      seed,
      seconds: 14,
      bpm: steady(120),
      jitter,
      drop: (i) => i > 40 && i % 53 === 0,
      duplicate: (i) => (i > 40 && i % 37 === 0 ? 0.4 : null),
    });
    const { samples } = runLoop({ sim, plant: { seed } });
    assert.ok(worstTicks(samples, SETTLE_S * 1000) <= STEADY_TICKS, `seed ${seed}: ${worstTicks(samples, 2_000).toFixed(2)} ticks`);
  }
});

test("M5: the clock stalls for 1.2 s: the engine plays on at the last tempo, nothing is sent, and the beat is found again when it returns", () => {
  const sim = simulate({ seed: 8, seconds: 20, bpm: steady(120), jitter, drop: (i) => i >= 300 && i < 360 });
  const out = runLoop({ sim, plant: { seed: 8 } });
  const stallStart = sim.truth[300]!;
  const stallEnd = sim.truth[360]!;
  assert.equal(out.calls.filter((c) => c.at > stallStart + 600 && c.at < stallEnd).length, 0, "nothing was sent during the stall");
  assert.ok(out.samples.filter((s) => s.at > stallStart && s.at < stallEnd).every((s) => s.running), "the engine kept playing");
  assert.ok(Math.abs(out.plant.bpm - 120) < 0.6);
  const back = out.samples.filter((s) => s.at >= stallEnd + 2_500);
  assert.ok(Math.max(...back.map((s) => Math.abs(s.heardMs) / s.tickMs)) <= STEADY_TICKS, "on the beat again");
});

test("M5: a Stop and a Continue (the sender goes on sending its clock): the engine stops, resumes, and is on the beat within two seconds", () => {
  const sim = simulate({ seed: 9, seconds: 20, bpm: steady(120), jitter });
  const stopAt = sim.truth[320]!;
  const continueAt = sim.truth[480]!;
  let engineOffset = 0;
  const out = runLoop({
    sim,
    plant: { seed: 9 },
    script: [
      { at: stopAt, run: (f) => f.onStop(stopAt) },
      { at: continueAt - 1, run: (f, plant) => { engineOffset = Math.round(plant.ticks / TICKS_PER_CLOCK) - 480; f.onContinue(continueAt - 1); } },
    ],
  });
  assert.deepEqual(out.calls.filter((c) => c.what === "transport").map((c) => c.value), [true, false, true]);
  assert.ok(out.samples.some((s) => s.at > stopAt + 200 && s.at < continueAt - 200 && !s.running), "the engine was stopped between");
  // After the Continue the engine is read against the sender with the whole pulses the engine had counted put in line with the pulse
  // that restarted it: the fine placement is what is measured.
  const resumed = out.samples.filter((s) => s.running && s.at >= continueAt + 2_000);
  const fine = resumed.map((s) => {
    // D0: the heard unit is an eighth of a pulse and not an engine tick (support/loop.ts), so a pulse is still 8 of them.
    const wholeOffset = s.heardMs / (s.tickMs * HEARD_TICKS_PER_PULSE);
    return (wholeOffset - Math.round(wholeOffset)) * HEARD_TICKS_PER_PULSE;
  });
  assert.ok(Math.max(...fine.map(Math.abs)) <= STEADY_TICKS, `fine placement after the Continue: ${Math.max(...fine.map(Math.abs)).toFixed(2)} ticks (engine pulses ${engineOffset} from the sender's)`);
});

test("the tempo the follower sends stays within the trim of the sender's tempo, and never outside what the engine accepts", () => {
  const sim = simulate({ seed: 11, seconds: 14, bpm: ramp(100, 130, 12), jitter });
  const { calls } = runLoop({ sim });
  const tempos = calls.filter((c) => c.what === "tempo" && c.at > sim.truth[0]! + 2_000);
  assert.ok(tempos.length > 20);
  for (const c of tempos) {
    const index = Math.min(sim.tempo.length - 1, Math.max(0, Math.round((c.at - sim.truth[0]!) / (2500 / 115))));
    const truth = sim.tempo[index]!;
    assert.ok(Math.abs((c.value as number) / truth - 1) <= 0.08 + BAD_TEMPO_BOUND, `${c.value} BPM at ${c.at} against ${truth}`);
    assert.ok((c.value as number) >= 1 && (c.value as number) <= 999);
  }
});

test("the same run twice gives the same calls, to the last bit", () => {
  const make = () => runLoop({ sim: simulate({ seed: 12, seconds: 8, bpm: ramp(110, 125, 8), jitter }), plant: { seed: 12 } });
  const a = make();
  const b = make();
  assert.deepEqual(a.calls, b.calls);
  assert.deepEqual(a.samples, b.samples);
});

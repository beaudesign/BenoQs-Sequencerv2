// The clock estimator (SPEC-0002 P4 plan observable M4, ADR-0009 decision 5): the page-clock time of each incoming 0xF8 in, a tempo and a
// predicted place for the next pulse out, against simulated clocks (test/support/sim-clock.ts) whose true pulse times are known.
//
// THE NUMBERS BELOW ARE PROPOSALS, AND THE OWNER'S TO CHANGE (plan D-P4-10). The plan gave the steady case as "a tempo within 0.1 BPM and
// a phase error within one tick, after 2 seconds, with up to 3 ms of jitter". A tempo estimate from noisy timestamps is a random number,
// so "within 0.1 BPM" can only be a share of runs, and the share is stated here (nine in ten, and never beyond twice the bound). The plan
// gave no number for a ramp; the ones here come from what the loop needs (the follower trims toward the grid this predicts, so a phase
// error is a note that lands that far from the sender's beat) and are recorded with the measured values in handoffs/evidence.
//
// "Phase" is where the estimator says the NEXT pulse falls, against where the sender's grid puts it (no jitter in the truth): that is the
// prediction the follower steers to. A tick is an eighth of a pulse (TICKS_PER_CLOCK in the engine), so it is 2.6 ms at 120 BPM.
import assert from "node:assert/strict";
import { test } from "node:test";
import { ClockEstimator, LOCK_PULSES, LOST_AFTER_MS, SEED_PULSES, type LockState, type PushOutcome } from "../src/clock-estimator.ts";
import { Rng } from "./support/rng.ts";
import { nextAfter, ramp, simulate, steady, step, type Sim, type SimConfig } from "./support/sim-clock.ts";

// ----- the proposals -----
const SETTLE_S = 2;
const STEADY_BPM = 0.1; // ... for at least STEADY_SHARE of the runs
const STEADY_SHARE = 0.9;
const STEADY_BPM_WORST = 0.2; // ... and for every run, at 120 BPM. At other tempos it scales with the tempo: the jitter is a fixed number of milliseconds, so
// it is a larger share of a shorter period (the first draft said 0.2 BPM at every tempo; at 200 BPM the estimator reads 0.27 on these seeds).
const STEADY_TICKS = 1;
const SLOW_RAMP = { bpmLag: 1.5, ticks: 2 }; // 1 BPM a second
const FAST_RAMP = { bpmLag: 3.5, ticks: 3 }; // 5 BPM a second (the first draft said 4 ticks; measured 2.1 on 60 unseen seeds)
const AFTER_RAMP = { seconds: 3, bpm: 0.3, ticks: 1 };
const RELOCK_WITHIN_S = 2;

interface Record {
  index: number;
  /** Seconds of the sender's time at this pulse. */
  elapsed: number;
  outcome: PushOutcome;
  state: LockState;
  bpmError: number | null;
  phaseTicks: number | null;
}

const ACCEPTING: readonly PushOutcome[] = ["first", "seeding", "accepted", "relocked", "restarted"];

function run(sim: Sim, est = new ClockEstimator()): Record[] {
  const out: Record[] = [];
  // The grid is read from the latest pulse the estimator took in, which is not the latest it was given: a pulse it turned away did not
  // move the grid, and `gridTime(1)` is one pulse after the one it did take.
  let taken = 0;
  for (const p of sim.pulses) {
    const outcome = est.push(p.at);
    if (ACCEPTING.includes(outcome)) taken = p.index;
    const next = nextAfter(sim, taken);
    const grid = est.gridTime(1);
    out.push({
      index: p.index,
      elapsed: (sim.truth[p.index]! - sim.truth[0]!) / 1000,
      outcome,
      state: est.state(p.at),
      bpmError: est.bpm === null ? null : est.bpm - sim.tempo[p.index]!,
      phaseTicks: grid === null ? null : (grid - next.at) / next.tickMs,
    });
  }
  return out;
}

const worst = (records: Record[], from: number, to: number, pick: (r: Record) => number | null): number => {
  const values = records.filter((r) => r.elapsed >= from && r.elapsed < to).map(pick);
  assert.ok(values.length > 0, `there are pulses between ${from} s and ${to} s`);
  assert.ok(values.every((v) => v !== null), `the estimator has a value for every pulse from ${from} s`);
  return Math.max(...values.map((v) => Math.abs(v!)));
};
const bpmOf = (r: Record): number | null => r.bpmError;
const ticksOf = (r: Record): number | null => r.phaseTicks;

const jittered = (bpm: number, seed: number, seconds = 10, extra: Partial<SimConfig> = {}): Sim =>
  simulate({ seed, seconds, bpm: steady(bpm), jitter: { shape: "uniform", ms: 3 }, ...extra });

// ----- the shape -----

test("before any pulse there is nothing to say: idle, no tempo, no grid", () => {
  const e = new ClockEstimator();
  assert.equal(e.state(0), "idle");
  assert.equal(e.bpm, null);
  assert.equal(e.periodMs, null);
  assert.equal(e.gridTime(1), null);
  assert.equal(e.pulseIndex(0), null);
});

test("a perfect clock: no tempo until the seed pulses are in, locking until a beat of pulses, then locked", () => {
  const e = new ClockEstimator();
  const period = 2500 / 120;
  const states: LockState[] = [];
  const bpms: (number | null)[] = [];
  for (let i = 0; i < LOCK_PULSES + 4; i++) {
    e.push(1_000 + i * period);
    states.push(e.state(1_000 + i * period));
    bpms.push(e.bpm);
  }
  for (let i = 0; i < SEED_PULSES - 1; i++) assert.equal(bpms[i], null, `no tempo after ${i + 1} pulses`);
  assert.ok(Math.abs(bpms[SEED_PULSES - 1]! - 120) < 0.01, `the tempo is there at pulse ${SEED_PULSES}: ${bpms[SEED_PULSES - 1]}`);
  for (let i = 0; i < LOCK_PULSES - 1; i++) assert.equal(states[i], "locking", `pulse ${i + 1} is still locking`);
  for (let i = LOCK_PULSES - 1; i < states.length; i++) assert.equal(states[i], "locked", `pulse ${i + 1} is locked`);
  assert.ok(Math.abs(e.bpm! - 120) < 0.001);
  assert.ok(Math.abs(e.periodMs! - period) < 0.001);
});

test("a perfect clock at 40, 60, 90, 120, 200 and 300 BPM is read to a hundredth of a BPM in a beat", () => {
  for (const bpm of [40, 60, 90, 120, 200, 300]) {
    const e = new ClockEstimator();
    for (let i = 0; i < LOCK_PULSES; i++) e.push(500 + (i * 2500) / bpm);
    assert.ok(Math.abs(e.bpm! - bpm) < 0.01, `${bpm} BPM read as ${e.bpm}`);
    assert.equal(e.state(500 + (LOCK_PULSES * 2500) / bpm), "locked", `${bpm} BPM`);
  }
});

test("the grid: the predicted pulse after the last is one period on, and the pulse index counts the sender's pulses", () => {
  const e = new ClockEstimator();
  const period = 2500 / 100;
  for (let i = 0; i < 30; i++) e.push(2_000 + i * period);
  const last = 2_000 + 29 * period;
  assert.ok(Math.abs(e.gridTime(0)! - last) < 0.01);
  assert.ok(Math.abs(e.gridTime(1)! - (last + period)) < 0.01);
  assert.ok(Math.abs(e.gridTime(-1)! - (last - period)) < 0.01);
  assert.ok(Math.abs(e.gridTime(2.5)! - (last + 2.5 * period)) < 0.01);
  const here = e.pulseIndex(last)!;
  assert.ok(Math.abs(e.pulseIndex(last + period)! - (here + 1)) < 0.001, "one period later is one pulse on");
  assert.ok(Math.abs(e.pulseIndex(last + period / 2)! - (here + 0.5)) < 0.001, "half a period later is half a pulse on");
});

// ----- steady clocks (M4) -----

test("M4: steady 120 BPM with up to 3 ms of jitter, after two seconds the next pulse is placed within a tick and the tempo within 0.1 BPM", () => {
  const seeds = 40;
  let within = 0;
  for (let seed = 1; seed <= seeds; seed++) {
    const r = run(jittered(120, seed));
    const ticks = worst(r, SETTLE_S, 10, ticksOf);
    const bpm = worst(r, SETTLE_S, 10, bpmOf);
    assert.ok(ticks <= STEADY_TICKS, `seed ${seed}: the phase was ${ticks.toFixed(2)} ticks out`);
    assert.ok(bpm <= STEADY_BPM_WORST, `seed ${seed}: the tempo was ${bpm.toFixed(3)} BPM out`);
    if (bpm <= STEADY_BPM) within++;
  }
  assert.ok(within >= seeds * STEADY_SHARE, `${within} of ${seeds} runs held 0.1 BPM from two seconds on; the proposal is ${STEADY_SHARE * 100}%`);
});

test("steady clocks at 60, 90, 150 and 200 BPM with 3 ms of jitter hold a tick and 0.17% of the tempo from two seconds on", () => {
  for (const bpm of [60, 90, 150, 200]) {
    for (let seed = 1; seed <= 15; seed++) {
      const r = run(jittered(bpm, seed));
      assert.ok(worst(r, SETTLE_S, 10, ticksOf) <= STEADY_TICKS, `${bpm} BPM, seed ${seed}: phase`);
      assert.ok(worst(r, SETTLE_S, 10, bpmOf) <= (STEADY_BPM_WORST * bpm) / 120, `${bpm} BPM, seed ${seed}: tempo`);
    }
  }
});

test("a noisy first beat does not throw the estimate off at a fast tempo (found on seed 137 at 200 BPM, with seeds the fixtures above did not use)", () => {
  // The first fit of four pulses is out by several percent at 200 BPM with 3 ms of jitter. If pulses are counted by that period a late one
  // reads as two, or an early one as a repeat, and the filter settles on a tempo 6% wrong and stays there.
  for (let seed = 100; seed < 160; seed++) {
    const r = run(jittered(200, seed));
    assert.ok(worst(r, SETTLE_S, 10, ticksOf) <= STEADY_TICKS, `seed ${seed}: phase`);
    assert.ok(worst(r, SETTLE_S, 10, bpmOf) <= (STEADY_BPM_WORST * 200) / 120 * 2, `seed ${seed}: tempo`);
  }
});

test("jitter with a normal shape (a standard deviation of 1.5 ms) is held as well", () => {
  for (let seed = 1; seed <= 20; seed++) {
    const r = run(simulate({ seed, seconds: 10, bpm: steady(120), jitter: { shape: "gaussian", ms: 1.5 } }));
    assert.ok(worst(r, SETTLE_S, 10, ticksOf) <= STEADY_TICKS, `seed ${seed}: phase`);
    assert.ok(worst(r, SETTLE_S, 10, bpmOf) <= STEADY_BPM_WORST, `seed ${seed}: tempo`);
  }
});

test("the estimator says how much jitter it sees: the root mean square of how far the pulses fall from its grid", () => {
  const e = new ClockEstimator();
  const sim = jittered(120, 3, 10);
  for (const p of sim.pulses) e.push(p.at);
  // A uniform spread of plus or minus 3 ms has a standard deviation of 3 / sqrt(3) = 1.73 ms.
  assert.ok(e.stats.jitterMs > 1.2 && e.stats.jitterMs < 2.4, `jitter read as ${e.stats.jitterMs.toFixed(2)} ms`);
  const calm = new ClockEstimator();
  for (const p of simulate({ seed: 3, seconds: 10, bpm: steady(120) }).pulses) calm.push(p.at);
  assert.ok(calm.stats.jitterMs < 0.01, `a perfect clock has no jitter: ${calm.stats.jitterMs}`);
});

// ----- ramps -----

test("M4: a slow ramp (100 to 120 BPM over 20 s) is followed: the tempo lags by little and the next pulse is placed within two ticks", () => {
  for (let seed = 1; seed <= 20; seed++) {
    const r = run(simulate({ seed, seconds: 20, bpm: ramp(100, 120, 20), jitter: { shape: "uniform", ms: 3 } }));
    assert.ok(worst(r, SETTLE_S, 20, bpmOf) <= SLOW_RAMP.bpmLag, `seed ${seed}: tempo lag`);
    assert.ok(worst(r, SETTLE_S, 20, ticksOf) <= SLOW_RAMP.ticks, `seed ${seed}: phase`);
    assert.ok(r.filter((x) => x.elapsed >= SETTLE_S).every((x) => x.state === "locked"), `seed ${seed}: the lock held through the ramp`);
  }
});

test("M4: a fast ramp (100 to 140 BPM over 8 s) is followed without losing the lock, and the estimator settles after it ends", () => {
  for (let seed = 1; seed <= 20; seed++) {
    const r = run(simulate({ seed, seconds: 16, bpm: ramp(100, 140, 8), jitter: { shape: "uniform", ms: 3 } }));
    assert.ok(worst(r, SETTLE_S, 8, bpmOf) <= FAST_RAMP.bpmLag, `seed ${seed}: tempo lag on the ramp`);
    assert.ok(worst(r, SETTLE_S, 8, ticksOf) <= FAST_RAMP.ticks, `seed ${seed}: phase on the ramp`);
    assert.ok(r.filter((x) => x.elapsed >= SETTLE_S).every((x) => x.state === "locked"), `seed ${seed}: the lock held through the ramp`);
    const after = 8 + AFTER_RAMP.seconds;
    assert.ok(worst(r, after, 16, bpmOf) <= AFTER_RAMP.bpm, `seed ${seed}: tempo ${AFTER_RAMP.seconds} s after the ramp`);
    assert.ok(worst(r, after, 16, ticksOf) <= AFTER_RAMP.ticks, `seed ${seed}: phase ${AFTER_RAMP.seconds} s after the ramp`);
  }
});

test("a ramp downward is followed as well as one upward", () => {
  for (let seed = 1; seed <= 10; seed++) {
    const r = run(simulate({ seed, seconds: 20, bpm: ramp(130, 110, 20), jitter: { shape: "uniform", ms: 3 } }));
    assert.ok(worst(r, SETTLE_S, 20, bpmOf) <= SLOW_RAMP.bpmLag, `seed ${seed}: tempo lag`);
    assert.ok(worst(r, SETTLE_S, 20, ticksOf) <= SLOW_RAMP.ticks, `seed ${seed}: phase`);
  }
});

test("a step in tempo (120 to 90 BPM) is taken: within two seconds the tempo and the phase are back to the steady bounds", () => {
  for (let seed = 1; seed <= 20; seed++) {
    const r = run(simulate({ seed, seconds: 14, bpm: step(120, 90, 6), jitter: { shape: "uniform", ms: 3 } }));
    const from = 6 + RELOCK_WITHIN_S;
    assert.ok(worst(r, from, 14, bpmOf) <= STEADY_BPM_WORST + 0.1, `seed ${seed}: tempo ${RELOCK_WITHIN_S} s after the step`);
    assert.ok(worst(r, from, 14, ticksOf) <= STEADY_TICKS, `seed ${seed}: phase ${RELOCK_WITHIN_S} s after the step`);
  }
});

test("steps whose new pulses land on the old grid by coincidence (120 to 80, 80 to 120, 120 to 60) are taken as steps, not as missing pulses", () => {
  // After a step the first pulses fall off the old grid. A pulse that happens to fall on it again (the second at 80 BPM is on the old
  // third) must not be read as "two pulses were never heard", or the filter follows its own mistake.
  for (const [from, to] of [[120, 80], [80, 120], [120, 60], [60, 120]] as const) {
    for (let seed = 1; seed <= 10; seed++) {
      const r = run(simulate({ seed, seconds: 14, bpm: step(from, to, 6), jitter: { shape: "uniform", ms: 3 } }));
      const label = `${from} to ${to} BPM, seed ${seed}`;
      assert.ok(worst(r, 6 + RELOCK_WITHIN_S, 14, bpmOf) <= STEADY_BPM_WORST + 0.1, `${label}: tempo`);
      assert.ok(worst(r, 6 + RELOCK_WITHIN_S, 14, ticksOf) <= STEADY_TICKS, `${label}: phase`);
    }
  }
});

test("the pulse count goes through a step in tempo: afterwards it is still the sender's pulse number (a late pulse is not read as a missing one)", () => {
  // Found by the follower, which keeps the engine in line by whole pulses: at 120 to 90 BPM on seed 1 the first pulse after the step fell
  // 27 ms after the last, nearer to two old periods than to one of the filter's, and was counted as two. The count stayed one too high
  // for good and the engine settled a whole pulse (20 ms) out. The count is checked against the sender's own pulse index.
  const steps = [[120, 90], [90, 120], [120, 100], [100, 140], [120, 80], [80, 120], [120, 60], [60, 120], [150, 100], [90, 60]] as const;
  for (const [from, to] of steps) {
    for (let seed = 1; seed <= 10; seed++) {
      const sim = simulate({ seed, seconds: 14, bpm: step(from, to, 6), jitter: { shape: "uniform", ms: 3 } });
      const est = new ClockEstimator();
      let before: number | null = null;
      let after: number | null = null;
      for (const p of sim.pulses) {
        est.push(p.at);
        const elapsed = (sim.truth[p.index]! - sim.truth[0]!) / 1000;
        if (elapsed > 5.5 && elapsed < 6 && before === null) before = est.pulseCount - p.index;
        if (elapsed > 6 + RELOCK_WITHIN_S) after = est.pulseCount - p.index;
      }
      assert.equal(after, before, `${from} to ${to} BPM, seed ${seed}: the count is ${after} off the sender's, it was ${before} before the step`);
    }
  }
});

// ----- missing, doubled and stray pulses (M4) -----

test("M4: a missing pulse does not make the estimator run away: it counts the pulse it did not hear and keeps the grid", () => {
  for (let seed = 1; seed <= 20; seed++) {
    const dropped = (i: number): boolean => i > 30 && i % 50 === 0;
    const sim = jittered(120, seed, 10, { drop: dropped });
    const r = run(sim);
    assert.ok(worst(r, SETTLE_S, 10, ticksOf) <= STEADY_TICKS, `seed ${seed}: phase`);
    assert.ok(worst(r, SETTLE_S, 10, bpmOf) <= STEADY_BPM_WORST, `seed ${seed}: tempo`);
    assert.ok(r.filter((x) => x.elapsed >= SETTLE_S).every((x) => x.state === "locked"), `seed ${seed}: still locked`);
    const e = new ClockEstimator();
    for (const p of sim.pulses) e.push(p.at);
    const missing = sim.truth.slice(0, -1).filter((_, i) => dropped(i)).length;
    assert.equal(e.stats.dropped, missing, `seed ${seed}: the pulses it did not hear are counted`);
  }
});

test("M4: three missing pulses in a row, and a pulse count that goes on counting across them", () => {
  const e = new ClockEstimator();
  const period = 2500 / 120;
  for (let i = 0; i < 40; i++) e.push(1_000 + i * period);
  const before = e.pulseIndex(1_000 + 39 * period)!;
  // 40, 41 and 42 never arrive.
  e.push(1_000 + 43 * period);
  assert.ok(Math.abs(e.pulseIndex(1_000 + 43 * period)! - (before + 4)) < 0.01, "the sender's count went on by four");
  assert.equal(e.stats.dropped, 3);
  assert.equal(e.state(1_000 + 43 * period), "locked");
  assert.ok(Math.abs(e.bpm! - 120) < 0.01);
});

test("M4: a doubled pulse (the same message twice, a moment apart) is ignored and counted", () => {
  for (let seed = 1; seed <= 20; seed++) {
    const sim = jittered(120, seed, 10, { duplicate: (i) => (i > 30 && i % 40 === 0 ? 0.4 : null) });
    const r = run(sim);
    assert.ok(worst(r, SETTLE_S, 10, ticksOf) <= STEADY_TICKS, `seed ${seed}: phase`);
    assert.ok(worst(r, SETTLE_S, 10, bpmOf) <= STEADY_BPM_WORST, `seed ${seed}: tempo`);
    assert.ok(r.filter((x) => x.elapsed >= SETTLE_S).every((x) => x.state === "locked"), `seed ${seed}: still locked`);
    const extras = sim.pulses.filter((p) => p.extra).length;
    const e = new ClockEstimator();
    for (const p of sim.pulses) e.push(p.at);
    assert.equal(e.stats.duplicates + e.stats.outliers, extras, `seed ${seed}: every extra is counted as a duplicate or an outlier`);
    assert.equal(e.stats.dropped, 0, `seed ${seed}: an extra pulse is not a gap`);
  }
});

test("a doubled pulse while the lock is still being made (the first beat) is ignored and counted, and the count is not moved by it", () => {
  // While locking, the grid is not yet known well enough to say a pulse is a repeat (the estimator reads it from the gap to the last pulse),
  // and this is the case that rule is for. Found as a survivor in the mutation run for P4d: nothing before this tested it.
  for (let seed = 1; seed <= 20; seed++) {
    const extras = [6, 9, 14, 20];
    const sim = simulate({ seed, seconds: 6, bpm: steady(120), jitter: { shape: "uniform", ms: 3 }, duplicate: (i) => (extras.includes(i) ? 0.4 : null) });
    const e = new ClockEstimator();
    for (const p of sim.pulses) e.push(p.at);
    const last = sim.pulses.filter((p) => !p.extra).length - 1;
    assert.equal(e.stats.duplicates, extras.length, `seed ${seed}: each extra was counted as a repeat`);
    assert.equal(e.pulseCount, last, `seed ${seed}: the count is the sender's, not moved by the extras`);
    assert.ok(Math.abs(e.bpm! - 120) < 0.3, `seed ${seed}: tempo ${e.bpm}`);
  }
});

test("M4: a stray pulse half way between two real ones is turned away and does not move the grid", () => {
  for (let seed = 1; seed <= 20; seed++) {
    const period = 2500 / 120;
    const sim = jittered(120, seed, 10, { duplicate: (i) => (i > 30 && i % 30 === 0 ? period / 2 : null) });
    const r = run(sim);
    assert.ok(worst(r, SETTLE_S, 10, ticksOf) <= STEADY_TICKS, `seed ${seed}: phase`);
    assert.ok(worst(r, SETTLE_S, 10, bpmOf) <= STEADY_BPM_WORST, `seed ${seed}: tempo`);
    assert.ok(r.filter((x) => x.elapsed >= SETTLE_S).every((x) => x.state === "locked"), `seed ${seed}: still locked`);
  }
});

test("a short burst of pulses far from the grid (fewer than the relock count) is turned away and the lock holds", () => {
  for (let seed = 1; seed <= 20; seed++) {
    const sim = jittered(120, seed, 10);
    // Pulses 100, 101 and 102 arrive 9 ms late: more than the estimator's gate, less than half a pulse.
    for (const p of sim.pulses) if (p.index >= 100 && p.index <= 102) p.at += 9;
    const r = run(sim);
    assert.ok(worst(r, SETTLE_S, 10, ticksOf) <= STEADY_TICKS + 0.5, `seed ${seed}: phase`);
    assert.ok(r.filter((x) => x.elapsed >= SETTLE_S).every((x) => x.state === "locked"), `seed ${seed}: still locked`);
    const e = new ClockEstimator();
    for (const p of sim.pulses) e.push(p.at);
    assert.equal(e.stats.relocks, 0, `seed ${seed}: no relock for three`);
  }
});

// ----- a stalled clock -----

test("M5 (the estimator's part): no pulse for 500 ms is `lost`, the last tempo is kept, and the pulses coming back lock again", () => {
  const e = new ClockEstimator();
  const period = 2500 / 120;
  const sim = jittered(120, 5, 4);
  for (const p of sim.pulses) e.push(p.at);
  const last = sim.pulses[sim.pulses.length - 1]!.at;
  assert.equal(e.state(last), "locked");
  assert.equal(e.state(last + LOST_AFTER_MS), "locked", "exactly 500 ms is not yet lost");
  assert.equal(e.state(last + LOST_AFTER_MS + 1), "lost");
  const kept = e.bpm!;
  assert.ok(Math.abs(kept - 120) < 0.3, `the last tempo, kept: ${kept}`);
  assert.equal(e.state(last + 10_000), "lost", "and it stays lost");
  assert.equal(e.bpm, kept, "the tempo does not move while nothing arrives");
  // The sender comes back 2 seconds on, at the same tempo.
  const back = last + 2_000;
  const outcomes: PushOutcome[] = [];
  const states: LockState[] = [];
  for (let i = 0; i < LOCK_PULSES + 2; i++) {
    outcomes.push(e.push(back + i * period));
    states.push(e.state(back + i * period));
  }
  assert.equal(outcomes[0], "restarted", "the first pulse after a stall starts a new lock");
  assert.equal(states[0], "locking");
  assert.equal(states[LOCK_PULSES - 1], "locked");
  assert.ok(Math.abs(e.bpm! - 120) < 0.01);
  assert.equal(e.stats.restarts, 1);
});

test("a stall changes the epoch, so a follower knows the pulse count began again; a dropped pulse does not", () => {
  const e = new ClockEstimator();
  const period = 2500 / 120;
  for (let i = 0; i < 30; i++) e.push(1_000 + i * period);
  const epoch = e.epoch;
  e.push(1_000 + 31 * period); // one missing
  assert.equal(e.epoch, epoch, "a missing pulse keeps the count");
  e.push(1_000 + 31 * period + 3_000);
  assert.equal(e.epoch, epoch + 1, "a stall starts a new count");
});

test("after a stall the new tempo is read from the pulses that follow, not from the old one", () => {
  const e = new ClockEstimator();
  for (let i = 0; i < 40; i++) e.push(1_000 + (i * 2500) / 120);
  for (let i = 0; i < 40; i++) e.push(10_000 + (i * 2500) / 75);
  assert.ok(Math.abs(e.bpm! - 75) < 0.01, `read as ${e.bpm}`);
});

// ----- what the estimator will not believe -----

test("a timestamp that is not later than the last one is counted and ignored", () => {
  const e = new ClockEstimator();
  const period = 2500 / 120;
  for (let i = 0; i < 30; i++) e.push(1_000 + i * period);
  const bpm = e.bpm;
  const grid = e.gridTime(1);
  assert.equal(e.push(1_000 + 20 * period), "backwards");
  assert.equal(e.push(1_000 + 29 * period), "backwards", "the same time again is not later either");
  assert.equal(e.stats.backwards, 2);
  assert.equal(e.bpm, bpm);
  assert.equal(e.gridTime(1), grid);
});

test("NaN, infinity and non-numbers are ignored and counted, never thrown at and never let into the state", () => {
  const e = new ClockEstimator();
  for (let i = 0; i < 30; i++) e.push(1_000 + (i * 2500) / 120);
  const bpm = e.bpm;
  for (const bad of [NaN, Infinity, -Infinity, undefined as unknown as number, null as unknown as number, "5" as unknown as number]) {
    assert.equal(e.push(bad), "ignored");
  }
  assert.equal(e.stats.ignored, 6);
  assert.equal(e.bpm, bpm);
  assert.ok(Number.isFinite(e.gridTime(1)!));
});

test("the same pulses give the same answers every time, to the last bit", () => {
  const sim = jittered(120, 9, 12, { drop: (i) => i % 61 === 60, duplicate: (i) => (i % 77 === 76 ? 0.3 : null) });
  const a = run(sim);
  const b = run(sim);
  assert.deepEqual(a, b);
});

test("reset forgets everything: idle again, no tempo, the counts at zero", () => {
  const e = new ClockEstimator();
  const sim = jittered(120, 2, 3);
  for (const p of sim.pulses) e.push(p.at);
  const last = sim.pulses[sim.pulses.length - 1]!.at;
  assert.equal(e.state(last), "locked");
  e.reset();
  assert.equal(e.state(last), "idle");
  assert.equal(e.bpm, null);
  assert.equal(e.gridTime(1), null);
  assert.deepEqual({ ...e.stats }, new ClockEstimator().stats);
});

test("the estimator has no clock of its own: what it says depends only on the timestamps it was given and the time it is asked about", () => {
  const e = new ClockEstimator();
  for (let i = 0; i < 30; i++) e.push(1_000_000 + (i * 2500) / 120);
  const grid = e.gridTime(1)!;
  assert.ok(Math.abs(grid - (1_000_000 + (30 * 2500) / 120)) < 0.01, "a large page time is fine");
  const late = new ClockEstimator();
  for (let i = 0; i < 30; i++) late.push(0.5 + (i * 2500) / 120);
  assert.ok(Math.abs(late.gridTime(1)! - (0.5 + (30 * 2500) / 120)) < 0.01, "and so is a small one");
});

test("whatever arrives, the estimator never throws and never says a number that is not a number: 300 random streams of 400 messages", () => {
  for (let seed = 1; seed <= 300; seed++) {
    const rng = new Rng(seed * 104729);
    const e = new ClockEstimator();
    let t = rng.next() * 1e6;
    for (let i = 0; i < 400; i++) {
      // Gaps from nothing to a stall, a clock that is nearly regular for a while, and now and then something that is not a time at all.
      const kind = rng.int(10);
      if (kind === 0) t += rng.next() * 900;
      else if (kind === 1) t -= rng.next() * 30;
      else if (kind === 2) t += 0;
      else t += 20.8 + (rng.next() * 2 - 1) * 4;
      const outcome = e.push(kind === 3 && rng.int(5) === 0 ? NaN : t);
      assert.ok(typeof outcome === "string", `seed ${seed}, message ${i}`);
      const bpm = e.bpm;
      assert.ok(bpm === null || (Number.isFinite(bpm) && bpm > 0), `seed ${seed}, message ${i}: bpm ${bpm}`);
      const grid = e.gridTime(1);
      assert.ok(grid === null || Number.isFinite(grid), `seed ${seed}, message ${i}: grid ${grid}`);
      const index = e.pulseIndex(t);
      assert.ok(index === null || Number.isFinite(index), `seed ${seed}, message ${i}: index ${index}`);
      assert.ok(Number.isFinite(e.stats.jitterMs), `seed ${seed}, message ${i}: jitter`);
    }
  }
});

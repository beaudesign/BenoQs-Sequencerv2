// What the clock estimator does against simulated clocks, in numbers: `node scripts/measure-estimator.ts`. The fixtures in
// test/clock-estimator.test.ts say what it must do; this says how well it does it, on seeds the fixtures do not use (100 and up), so a
// change to the estimator can be judged on more than the cases it was tuned on. The clocks are made up (test/support/sim-clock.ts):
// nothing here was measured on a real port.
import { ClockEstimator, type PushOutcome } from "../src/clock-estimator.ts";
import { nextAfter, ramp, simulate, steady, step, type Sim, type SimConfig } from "../test/support/sim-clock.ts";

const JITTER = { shape: "uniform", ms: 3 } as const;
const TAKES: readonly PushOutcome[] = ["first", "seeding", "accepted", "relocked", "restarted"];

interface Worst {
  bpm: number;
  ticks: number;
  notLocked: boolean;
}

function worstOf(sim: Sim, from: number, to: number): Worst {
  const e = new ClockEstimator();
  const w: Worst = { bpm: 0, ticks: 0, notLocked: false };
  let taken = 0;
  for (const p of sim.pulses) {
    const outcome = e.push(p.at);
    if (TAKES.includes(outcome)) taken = p.index;
    const seconds = (sim.truth[p.index]! - sim.truth[0]!) / 1000;
    if (seconds < from || seconds >= to) continue;
    if (e.state(p.at) !== "locked") w.notLocked = true;
    const next = nextAfter(sim, taken);
    w.ticks = Math.max(w.ticks, Math.abs(e.gridTime(1)! - next.at) / next.tickMs);
    w.bpm = Math.max(w.bpm, Math.abs(e.bpm! - sim.tempo[taken]!));
  }
  return w;
}

function row(label: string, config: Omit<SimConfig, "seed">, from: number, to: number, seeds: number, bound = 0.1): string {
  const worst: Worst[] = [];
  for (let seed = 100; seed < 100 + seeds; seed++) worst.push(worstOf(simulate({ seed, jitter: JITTER, ...config }), from, to));
  const sorted = worst.map((w) => w.bpm).sort((a, b) => a - b);
  const share = (worst.filter((w) => w.bpm <= bound).length / seeds) * 100;
  const ticks = Math.max(...worst.map((w) => w.ticks));
  const unlocked = worst.filter((w) => w.notLocked).length;
  const at = (f: number): string => sorted[Math.min(seeds - 1, Math.floor(seeds * f))]!.toFixed(3);
  return `${label.padEnd(22)} bpm: p50 ${at(0.5)} p90 ${at(0.9)} max ${at(1)} (${share.toFixed(0)}% within ${bound}) | ticks max ${ticks.toFixed(2)} | not locked in ${unlocked} of ${seeds}`;
}

const lines = [
  `# estimator measured on made-up clocks, uniform jitter of plus or minus ${JITTER.ms} ms, seeds 100 up (the fixtures use 1 to 40)`,
  "# bpm is the worst tempo error and ticks the worst error in placing the next pulse (a tick is an eighth of a pulse), over the window",
  row("steady 60 (2-10 s)", { seconds: 10, bpm: steady(60) }, 2, 10, 300),
  row("steady 90 (2-10 s)", { seconds: 10, bpm: steady(90) }, 2, 10, 300),
  row("steady 120 (2-10 s)", { seconds: 10, bpm: steady(120) }, 2, 10, 300),
  row("steady 150 (2-10 s)", { seconds: 10, bpm: steady(150) }, 2, 10, 300),
  row("steady 200 (2-10 s)", { seconds: 10, bpm: steady(200) }, 2, 10, 300),
  row("ramp 100-120 in 20 s", { seconds: 20, bpm: ramp(100, 120, 20) }, 2, 20, 100),
  row("ramp 130-110 in 20 s", { seconds: 20, bpm: ramp(130, 110, 20) }, 2, 20, 100),
  row("ramp 100-140 in 8 s", { seconds: 16, bpm: ramp(100, 140, 8) }, 2, 8, 100),
  row("3 s after that ramp", { seconds: 16, bpm: ramp(100, 140, 8) }, 11, 16, 100),
  row("step 120-90 (2 s on)", { seconds: 14, bpm: step(120, 90, 6) }, 8, 14, 100),
  row("step 120-60 (2 s on)", { seconds: 14, bpm: step(120, 60, 6) }, 8, 14, 100),
  row("step 60-120 (2 s on)", { seconds: 14, bpm: step(60, 120, 6) }, 8, 14, 100),
  row("one pulse in 50 lost", { seconds: 10, bpm: steady(120), drop: (i) => i > 30 && i % 50 === 0 }, 2, 10, 100),
  row("one pulse in 40 doubled", { seconds: 10, bpm: steady(120), duplicate: (i) => (i > 30 && i % 40 === 0 ? 0.4 : null) }, 2, 10, 100),
];
console.log(lines.join("\n"));

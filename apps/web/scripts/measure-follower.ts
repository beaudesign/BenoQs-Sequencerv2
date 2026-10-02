// What the follower does in a loop with a simulated sender and a simulated engine, in numbers: `node scripts/measure-follower.ts`. The
// tests in test/follower.test.ts say what it must do; this says how well it does it, on seeds they do not use (100 and up), so a change
// can be judged on more than the cases it was tuned on. Both ends are made up (test/support/sim-clock.ts, test/support/plant.ts):
// nothing here was measured on a real port or a real engine. The figure is the HEARD error: where the engine's notes land against the
// sender's beat, in ticks (an eighth of a pulse: 2.6 ms at 120 BPM), the worst over the window, over the seeds.
import { ramp, simulate, steady, step, type SimConfig } from "../test/support/sim-clock.ts";
import { runLoop, worstTicks, type LoopOptions } from "../test/support/loop.ts";

const JITTER = { shape: "uniform", ms: 3 } as const;
const SEEDS = 40;

function row(label: string, config: Omit<SimConfig, "seed">, from: number, to: number, bound: number, options: Omit<LoopOptions, "sim" | "plant"> = {}): string {
  const worst: number[] = [];
  for (let seed = 100; seed < 100 + SEEDS; seed++) {
    const { samples } = runLoop({ sim: simulate({ seed, jitter: JITTER, ...config }), plant: { seed }, ...options });
    worst.push(worstTicks(samples, from, to));
  }
  const sorted = worst.slice().sort((a, b) => a - b);
  const at = (f: number): string => sorted[Math.min(SEEDS - 1, Math.floor(SEEDS * f))]!.toFixed(2);
  const over = worst.filter((w) => w > bound).length;
  return `${label.padEnd(34)} ticks: p50 ${at(0.5)} p90 ${at(0.9)} max ${at(1)} | over the ${bound} proposed: ${over} of ${SEEDS}`;
}

const lines = [
  `# follower measured on made-up clocks and a made-up engine, uniform jitter of plus or minus ${JITTER.ms} ms, seeds 100 to ${99 + SEEDS} (the tests use 1 to 12)`,
  "# worst heard error in ticks over the window (seconds after the Start message), lookahead 30 ms unless said",
  row("steady 120 (2-12 s)", { seconds: 12, bpm: steady(120) }, 2_000, Infinity, 1),
  row("steady 70 (2.5-12 s)", { seconds: 12, bpm: steady(70) }, 2_500, Infinity, 1),
  row("steady 160 (2.5-12 s)", { seconds: 12, bpm: steady(160) }, 2_500, Infinity, 1),
  row("catch-up 120 (1.5-8 s)", { seconds: 8, bpm: steady(120) }, 1_500, Infinity, 1),
  row("lookahead 60 offset +10 (3.5-14 s)", { seconds: 14, bpm: steady(120) }, 3_500, Infinity, 1, { settings: { lookaheadMs: 60, offsetMs: 10 } }),
  row("lookahead 0 (3.5-14 s)", { seconds: 14, bpm: steady(120) }, 3_500, Infinity, 1, { settings: { lookaheadMs: 0, offsetMs: 0 } }),
  row("ramp 100-120 in 20 s (3-24 s)", { seconds: 24, bpm: ramp(100, 120, 20) }, 3_000, Infinity, 3),
  row("ramp 100-140 in 8 s (3-9 s)", { seconds: 18, bpm: ramp(100, 140, 8) }, 3_000, 9_000, 4),
  row("3 s after that ramp (11-18 s)", { seconds: 18, bpm: ramp(100, 140, 8) }, 11_000, Infinity, 1),
  ...(
    [[120, 90], [90, 120], [100, 140], [150, 100], [120, 60], [60, 120]] as const
  ).map(([a, b]) => row(`step ${a}-${b} (3 s on)`, { seconds: 16, bpm: step(a, b, 7) }, 10_000, Infinity, 1)),
  row("one pulse in 53 lost, 1 in 37 doubled", { seconds: 14, bpm: steady(120), drop: (i) => i > 40 && i % 53 === 0, duplicate: (i) => (i > 40 && i % 37 === 0 ? 0.4 : null) }, 2_000, Infinity, 1),
];
console.log(lines.join("\n"));

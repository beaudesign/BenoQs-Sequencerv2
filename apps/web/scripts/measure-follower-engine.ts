// What the follower does in front of the real engine, in numbers: `node scripts/measure-follower-engine.ts`. The tests in
// test/follower-engine.test.ts say what it must do; this says how well, on seeds they do not use (100 and up). The sender and the receiver
// are made up (test/support/sim-clock.ts, receiver.ts), the engine, worklet, scheduler and follower are the real ones, and nothing was
// measured on a browser or a port. The figure is the HEARD error in ticks (an eighth of a pulse: 2.6 ms at 120 BPM, 1.9 ms at 160): the
// time a pulse of the engine is delivered to a device, against the time of the sender's pulse with the same number; the worst over a
// window, over the seeds. The output's own timing noise (the page's idea of the audio clock, wrong by up to 1.5 ms at each batch) is run
// both off and on, since it is the scheduler's and not the follower's and a second run shows how much of the figure it is.
import { closedLoop, pulseErrors, worst, type Options } from "../test/support/engine-loop.ts";
import { ramp, steady, step, type SimConfig } from "../test/support/sim-clock.ts";

const SEEDS = 20;
const first = 100;

function row(label: string, config: Omit<SimConfig, "seed" | "jitter">, from: number, to: number, bound: number, options: Options = {}): string {
  const figures: number[] = [];
  for (let seed = first; seed < first + SEEDS; seed++) figures.push(worst(pulseErrors(closedLoop(config, { seed, ...options })), from, to));
  const sorted = figures.slice().sort((a, b) => a - b);
  const at = (f: number): string => sorted[Math.min(SEEDS - 1, Math.floor(SEEDS * f))]!.toFixed(2);
  return `${label.padEnd(30)} output noise ${String(options.outputJitterMs ?? 0).padEnd(3)} ms   ticks: p50 ${at(0.5)} p90 ${at(0.9)} max ${at(1)} | over the ${bound} proposed: ${figures.filter((f) => f > bound).length} of ${SEEDS}`;
}

const lines = [
  `# follower in front of the real engine: simulated sender (uniform jitter of plus or minus 3 ms), real engine, worklet, scheduler and follower; seeds ${first} to ${first + SEEDS - 1} (the tests use 1 to 7)`,
  "# worst heard error in ticks over the window (milliseconds after the Start message), lookahead 30 ms",
];
for (const outputJitterMs of [0, 1.5]) {
  const o = { outputJitterMs };
  lines.push(
    row("steady 120 (2.5-10 s)", { seconds: 10, bpm: steady(120) }, 2_500, Infinity, 1, o),
    row("steady 70 (3-10 s)", { seconds: 10, bpm: steady(70) }, 3_000, Infinity, 1, o),
    row("steady 160 (3-10 s)", { seconds: 10, bpm: steady(160) }, 3_000, Infinity, 1, o),
    row("step 120 to 90 at 5 s (9.5-14 s)", { seconds: 14, bpm: step(120, 90, 5) }, 9_500, Infinity, 1, o),
    row("ramp 100 to 130 over 8 s, on it", { seconds: 14, bpm: ramp(100, 130, 8) }, 3_000, 8_000, 3, o),
    row("ramp 100 to 130, after (11-14 s)", { seconds: 14, bpm: ramp(100, 130, 8) }, 11_000, Infinity, 1, o),
  );
}
console.log(lines.join("\n"));

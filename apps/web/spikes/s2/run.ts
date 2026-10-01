// Spike S2: the engine in an AudioWorklet. Run `just spike-s2` (or, after `just web-wasm` and `npm run build` in
// apps/web: `node spikes/s2/run.ts [--blocks N] [--write] [--headed]`). Prints a record; with --write, saves it to
// handoffs/evidence/p3-s2-run.{txt,json}. Exit code 1 if a hash differs or a render allocates.
//
// Pass, from SPEC-0002 tech.md section 8: the five golden hashes equal native, the programs' digests equal native,
// no allocation in `render`. The timing budget (p99 under 25 % of the block) is *proposed*, and a miss is a finding
// for the owner, not a failure of the run.
import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { launch } from "../../scripts/browser.ts";
import { repoRoot, serve } from "../../scripts/serve.ts";
import type { Programs } from "../program.ts";
import type { S2 } from "./page.ts";

declare const s2: S2;

const argv = process.argv.slice(2);
const flag = (name: string): boolean => argv.includes(name);
const opt = (name: string, fallback: number): number => {
  const i = argv.indexOf(name);
  return i >= 0 ? Number(argv[i + 1]) : fallback;
};
const BLOCKS = opt("--blocks", 20_000);
const BATCH = opt("--batch", 1_000);
const PATTERNS = ["hello", "chords_and_strums", "effector", "phrases", "mcc_and_transport"];

function stats(values: number[]): { n: number; mean: number; p50: number; p99: number; p999: number; max: number } {
  const v = [...values].sort((a, b) => a - b);
  const at = (q: number): number => v[Math.min(v.length - 1, Math.floor(q * v.length))] ?? 0;
  return { n: v.length, mean: v.reduce((a, b) => a + b, 0) / Math.max(1, v.length), p50: at(0.5), p99: at(0.99), p999: at(0.999), max: v[v.length - 1] ?? 0 };
}
const us = (ms: number): string => `${(ms * 1000).toFixed(1)} us`;

const doc = JSON.parse(readFileSync(join(repoRoot, "apps/web/engine/golden/programs.json"), "utf8")) as Programs;
const lines: string[] = [];
const out = (s = ""): void => {
  lines.push(s);
  console.log(s);
};
const failures: string[] = [];

const server = await serve();
const browser = await launch({ headed: flag("--headed") });
const page = await browser.newPage();
page.on("pageerror", (e) => failures.push(`page error: ${e.message}`));
await page.goto(`${server.url}/spikes/s2/page.html`);
const info = await page.evaluate(() => s2.start("/dist/octoweb-spike.wasm"));

const dirty = execFileSync("git", ["status", "--porcelain", "--", "apps/web", "contracts"], { cwd: repoRoot }).toString().trim() !== "";
const commit = execFileSync("git", ["rev-parse", "--short", "HEAD"], { cwd: repoRoot }).toString().trim() + (dirty ? " (uncommitted changes in apps/web)" : "");
out(`# Spike S2: the engine in an AudioWorklet (SPEC-0002 P3, WENGE-0014)`);
out(`# commit ${commit}, ${new Date().toISOString()}`);
out(`# ${browser.version() ? "Chromium " + browser.version() : ""}${flag("--headed") ? " (headed)" : " (headless)"}, ${info.cores} cores, crossOriginIsolated ${info.crossOriginIsolated}, performance.now() steps by ${info.timerResolutionUs.toFixed(1)} us`);
out(`# audio context ${info.audioSampleRate} Hz; the worklet scope has: performance ${info.hasPerformance}, Date ${info.hasDate}, TextDecoder ${info.hasTextDecoder}`);
out(`# module: octoweb-spike.wasm (the build with the allocation counter and the pattern runner): measure ${info.measure}, spike ${info.spike}`);
out();

// 1. The five patterns, hashed inside the worklet, against the native goldens.
out("## 1. Golden patterns hashed in the worklet (octorun inside the module) against examples/golden");
for (const name of PATTERNS) {
  const source = readFileSync(join(repoRoot, `examples/${name}.pattern`), "utf8");
  const golden = readFileSync(join(repoRoot, `examples/golden/${name}.sha256`), "utf8").split("\n")[0]?.replace(/^events /, "") ?? "";
  const r = await page.evaluate(([n, s]) => s2.hash(n as string, s as string), [name, source]);
  const ok = !r.isError && r.text === golden;
  if (!ok) failures.push(`golden ${name}: ${r.isError ? "error: " : "got "}${r.text}, want ${golden}`);
  out(`  ${ok ? "equal " : "DIFFER"}  ${name.padEnd(18)} ${r.text}${r.allocs === null ? "" : `  (${r.allocs} allocations while it ran, the runner builds its own engine)`}`);
}
out();

// 2. The programs pressed on the panel, 128-frame blocks, against the digests the native test pins.
out("## 2. Programs pressed on the panel, rendered in 128-frame blocks (golden/programs.json, pinned by the native test)");
for (const name of Object.keys(doc.programs)) {
  const want = doc.programs[name]!.digest;
  const inWorklet = await page.evaluate(([d, n]) => s2.program(d as Programs, n as string), [doc, name]);
  const onMain = await page.evaluate(([d, n]) => s2.programMain(d as Programs, n as string), [doc, name]);
  for (const [where, got] of [["worklet", inWorklet.digest], ["main thread", onMain.digest]] as const) {
    if (got !== want) failures.push(`program ${name} in the ${where}: ${got}, want ${want}`);
    out(`  ${got === want ? "equal " : "DIFFER"}  ${name.padEnd(8)} ${where.padEnd(11)} ${got}  (${inWorklet.events} events in ${inWorklet.blocks} blocks)`);
  }
}
out();

// 3. Timing and allocation, the dense program (ten tracks, every step on, 240 BPM).
const blockMs = (128 / info.audioSampleRate) * 1000;
out(`## 3. Cost of render(128): the dense program, ${BLOCKS} blocks; the block is ${blockMs.toFixed(3)} ms`);
const main = await page.evaluate(([d, b]) => s2.timingMain(d as Programs, "dense", b as number), [doc, BLOCKS]);
const m = stats(main.perBlockMs);
const mainAlloc = main.allocsBefore === null || main.allocsAfter === null ? null : main.allocsAfter - main.allocsBefore;
out(`  main thread, performance.now() per block (${main.events} events; a figure under ${info.timerResolutionUs.toFixed(0)} us is below the clock's step):`);
out(`    mean ${us(m.mean)}, p50 ${us(m.p50)}, p99 ${us(m.p99)}, p99.9 ${us(m.p999)}, max ${us(m.max)}`);
out(`    p99 is ${((m.p99 / blockMs) * 100).toFixed(1)} % of the block; the proposed budget is 25 %`);
out(`    allocations during the render loop: ${mainAlloc}`);
const wk = await page.evaluate(([d, b, batch]) => s2.timingWorklet(d as Programs, "dense", b as number, batch as number), [doc, BLOCKS, BATCH]);
const perBlock = wk.batchMs.map((ms) => ms / wk.batch);
const w = stats(perBlock);
const wkAlloc = wk.allocsBefore === null || wk.allocsAfter === null ? null : wk.allocsAfter - wk.allocsBefore;
out(`  worklet, Date over batches of ${wk.batch} blocks (${wk.batchMs.length} batches, ${wk.events} events), per block:`);
out(`    mean ${us(w.mean)}, fastest batch ${us(Math.min(...perBlock))}, slowest batch ${us(w.max)}   (a batch's mean hides a slow block; the main-thread numbers show the tail)`);
out(`    slowest batch is ${((w.max / blockMs) * 100).toFixed(1)} % of the block`);
out(`    allocations during the render loop: ${wkAlloc}`);
for (const [where, a] of [["main thread", mainAlloc], ["worklet", wkAlloc]] as const) {
  if (a === null) failures.push(`${where}: no allocation counter in the module`);
  else if (a !== 0) failures.push(`${where}: render allocated ${a} times`);
}
out();

out(failures.length ? `## RESULT: FAIL\n${failures.map((f) => "  - " + f).join("\n")}` : "## RESULT: PASS (hashes equal native, no allocation in render). Timing is recorded above for the owner; the budget is a proposal.");
out("# Timing here is this container's CPU in headless Chromium, not a laptop or the owner's machine.");

if (flag("--write")) {
  const dir = join(repoRoot, "handoffs/evidence");
  mkdirSync(dir, { recursive: true });
  writeFileSync(join(dir, "p3-s2-run.txt"), lines.join("\n") + "\n");
  writeFileSync(
    join(dir, "p3-s2-run.json"),
    JSON.stringify({ commit, chromium: browser.version(), blockMs, blocks: BLOCKS, main: { ...m, allocations: mainAlloc }, worklet: { ...w, batch: wk.batch, allocations: wkAlloc }, failures }, null, 1) + "\n",
  );
}
await browser.close();
await server.close();
process.exit(failures.length ? 1 : 0);

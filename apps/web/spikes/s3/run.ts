// Spike S3, the half that runs here: the real engine, worklet, scheduler and follower, with a SIMULATED clock. Run after `just web-wasm` and
// `npm run build` in apps/web:
//
//   node spikes/s3/run.ts [--seconds 30] [--bpm 120] [--write]
//
// WHAT THIS IS AND IS NOT. The sender in the follow run is a timer chain in the page that sends a Start and then a 0xF8 every 2500/bpm ms,
// each stamped with the page's own `performance.now()` when the timer fired. The port in the send run is a `setTimeout`. Neither is
// Ableton Live, a MIDI port, or Web MIDI: headless Chromium refuses Web MIDI and this container has no MIDI port or Live. So nothing here
// says how Live's clock looks to a browser, whether `event.timeStamp` is on `performance.now()`'s clock for a real port (the simulation
// makes it so), or what a loopback adds. It shows that the harness runs from end to end on the real engine and follower, that the analysis
// reads what they do with a clock that has a timer's own scatter, and what a person's saved file will look like. The run that matters is
// the owner's, with Live: apps/web/ABLETON.md, `spikes/s3/page.html`, then `node spikes/s3/report.ts <file>`.
import { execFileSync } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { launch } from "../../scripts/browser.ts";
import { repoRoot, serve } from "../../scripts/serve.ts";
import { formatFollow, formatSend } from "./format.ts";
import { SCATTER_MS, SIMULATED_FOLLOW, SIMULATED_SEND, STAMP_AGE_MS, standInScript } from "./stand-in.ts";
import type { FollowConfig, FollowResult, S3, SendConfig, SendResult } from "./page.ts";

declare const s3: S3;
declare const sender: { start(bpm: number): void; stop(): void };

const argv = process.argv.slice(2);
const flag = (name: string): boolean => argv.includes(name);
const opt = (name: string, fallback: number): number => {
  const i = argv.indexOf(name);
  return i >= 0 ? Number(argv[i + 1]) : fallback;
};
const SECONDS = opt("--seconds", 30);
const BPM = opt("--bpm", 120);

const lines: string[] = [];
const out = (s = ""): void => {
  lines.push(s);
  console.log(s);
};
const sleep = (ms: number): Promise<void> => new Promise((ok) => setTimeout(ok, ms));

const dirty = execFileSync("git", ["status", "--porcelain", "--", "apps/web", "contracts"], { cwd: repoRoot }).toString().trim() !== "";
const commit = execFileSync("git", ["rev-parse", "--short", "HEAD"], { cwd: repoRoot }).toString().trim() + (dirty ? " (uncommitted changes in apps/web)" : "");

const server = await serve();
const browser = await launch({ realisticBackground: true });
const browserName = `Chromium ${browser.version()} (headless, Playwright)`;
const failures: string[] = [];
const follow: FollowConfig = { minutes: SECONDS / 60, inputId: "in1", lookaheadMs: 30, offsetMs: 0, nominalBpm: BPM };
const send: SendConfig = { minutes: SECONDS / 60, bpm: BPM, outputId: "out1", inputId: "in1", lookaheadMs: 30 };

async function page(): Promise<import("playwright-core").Page> {
  const context = await browser.newContext();
  await context.addInitScript(standInScript, { stampAgeMs: STAMP_AGE_MS, scatterMs: SCATTER_MS });
  const p = await context.newPage();
  p.on("pageerror", (e) => failures.push(`page error: ${e.message}`));
  await p.goto(`${server.url}/spikes/s3/page.html`);
  await p.evaluate(() => s3.access());
  return p;
}

const followPage = await page();
const followRunning = followPage.evaluate((c) => s3.follow(c), follow);
await sleep(1_500); // the page is listening and the engine is past the start of its audio context
await followPage.evaluate((bpm) => sender.start(bpm), BPM);
const followed: FollowResult = await followRunning;
await followPage.close();

const sendPage = await page();
const sent: SendResult = await sendPage.evaluate((c) => s3.send(c), send);
await sendPage.close();
await browser.close();
await server.close();

const followText = formatFollow({ simulated: SIMULATED_FOLLOW, commit, browser: browserName, result: followed });
const sendText = formatSend({ simulated: SIMULATED_SEND, commit, browser: browserName, result: sent });

if (followed.worklet.errors.length) failures.push(`follow: worklet errors: ${followed.worklet.errors.join("; ")}`);
if (sent.worklet.errors.length) failures.push(`send: worklet errors: ${sent.worklet.errors.join("; ")}`);
// The sender starts 1.5 s into the run.
if (followed.analysis.all.pulses < (SECONDS - 2) * BPM * 0.4 * 0.9) failures.push(`follow: only ${followed.analysis.all.pulses} pulses in ${SECONDS} s at ${BPM} BPM`);
if (followed.analysis.all.phase.counted === 0) failures.push("follow: no phase was counted");
if (sent.loop.matched < sent.loop.sent * 0.99) failures.push(`send: ${sent.loop.matched} of ${sent.loop.sent} pulses matched`);

out("# SIMULATED CLOCK AND PORT: this is not Ableton Live, not a MIDI port, and not Web MIDI's timing (SPEC-0002 P4e, WENGE-0015).");
out("# It shows the harness running end to end on the real engine and follower. The run that measures S3 is the owner's: apps/web/ABLETON.md.");
out(`# ${commit}, ${new Date().toISOString()}, ${browserName}`);
out();
for (const line of followText) out(line);
out();
for (const line of sendText) out(line);
out();
out(failures.length ? `## RESULT: FAIL\n${failures.map((f) => "  - " + f).join("\n")}` : "## RESULT: both runs completed. Nothing here is a figure for Live or for a MIDI port.");

if (flag("--write")) {
  const dir = join(repoRoot, "handoffs/evidence");
  mkdirSync(dir, { recursive: true });
  writeFileSync(join(dir, "p4e-s3-simulated.txt"), lines.join("\n") + "\n");
  writeFileSync(join(dir, "p4e-s3-simulated-follow.json"), JSON.stringify({ simulated: SIMULATED_FOLLOW, commit, browser: browserName, result: followed }) + "\n");
  writeFileSync(join(dir, "p4e-s3-simulated-send.json"), JSON.stringify({ simulated: SIMULATED_SEND, commit, browser: browserName, result: sent }) + "\n");
}
process.exit(failures.length ? 1 : 0);

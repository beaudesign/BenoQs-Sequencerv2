// Spike S1, the half that runs here: the real engine, worklet and scheduler send every step of one track to a
// STAND-IN output, and the page measures the scheduling margin of every send. Run after `just web-wasm` and
// `npm run build` in apps/web:
//
//   node spikes/s1/run.ts [--seconds 30] [--bpm 30] [--lookahead 30] [--alpha 1] [--write]
//   xvfb-run -a node spikes/s1/run.ts --hidden               (also runs with the tab in the background)
//
// WHAT THIS MEASURES AND WHAT IT DOES NOT. The margin (the timestamp given to `send`, less the time of the call) is
// a property of the page: how early the worklet's events reach the scheduler. It is real. The arrival numbers come
// from a stand-in whose "port" is a `setTimeout` in the page, so they measure this machine's timers and not Web MIDI,
// and must not be read as Web MIDI's jitter. Headless Chromium refuses Web MIDI and this container has no MIDI port,
// so the real-port half is the page a person runs: `apps/web/spikes/s1/page.html`, which the plan asks the owner for.
import { execFileSync } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { launch } from "../../scripts/browser.ts";
import { Tab, launchRaw } from "../../scripts/cdp.ts";
import { repoRoot, serve } from "../../scripts/serve.ts";
import type { Analysis } from "./analyse.ts";
import type { S1, S1Config, S1Result } from "./page.ts";

declare const s1: S1;

const argv = process.argv.slice(2);
const flag = (name: string): boolean => argv.includes(name);
const opt = (name: string, fallback: number): number => {
  const i = argv.indexOf(name);
  return i >= 0 ? Number(argv[i + 1]) : fallback;
};
const SECONDS = opt("--seconds", 30);
const LOOKAHEAD = opt("--lookahead", 30);
// The engine counts 192 ticks to the quarter note and a step is 12 ticks (D0, the tick length, is open), so 30 BPM
// is a step every 125 ms, a sixteenth note at 120 BPM, and 120 BPM is a step every 31 ms: four times denser.
const BPM = opt("--bpm", 30);
const ALPHA = opt("--alpha", 1);
// A tab in the background needs a browser that is not Playwright's (Playwright keeps every page visible), so --hidden
// starts Chromium itself, headed, and drives it over the DevTools protocol (scripts/cdp.ts).
const hidden = flag("--hidden");

const lines: string[] = [];
const out = (s = ""): void => {
  lines.push(s);
  console.log(s);
};
const ms = (x: number): string => `${x.toFixed(2)} ms`;

/** Runs in the page before its own scripts: a MIDI access with one output and one input, joined by a timer. */
function standInScript(): void {
  const input = { id: "in1", name: "Stand-in loopback (a timer in the page, not a MIDI port)", type: "input", onmidimessage: null as null | ((e: unknown) => void) };
  const output = {
    id: "out1",
    name: "Stand-in output (a timer in the page, not a MIDI port)",
    type: "output",
    send(data: ArrayLike<number>, timestamp?: number): void {
      const now = performance.now();
      const delay = timestamp !== undefined && timestamp > now ? timestamp - now : 0;
      const bytes = Uint8Array.from(data);
      setTimeout(() => input.onmidimessage?.({ data: bytes, timeStamp: performance.now() }), delay);
    },
  };
  (navigator as unknown as { requestMIDIAccess: () => Promise<unknown> }).requestMIDIAccess = async () => ({
    inputs: new Map([["in1", input]]),
    outputs: new Map([["out1", output]]),
  });
}

function report(label: string, r: S1Result): void {
  const line = (name: string, a: Analysis): void => {
    out(`  ${name.padEnd(8)} sent ${String(a.sent).padStart(5)}, late ${String(a.late).padStart(3)}, margin min ${ms(a.marginMs.min)}, median ${ms(a.marginMs.p50)}, mean ${ms(a.marginMs.mean)}, sigma ${ms(a.marginMs.sigma)}`);
  };
  out(`### ${label}: engine at ${r.config.bpm} BPM, a step every ${r.stepMs.toFixed(1)} ms; ${r.config.minutes * 60} s requested, ${r.durationS.toFixed(1)} s run, tab hidden ${r.hiddenS.toFixed(1)} s, lookahead ${r.config.lookaheadMs} ms, clock-map smoothing ${r.config.alpha}`);
  out(`  audio context ${r.environment.audioSampleRate} Hz, base latency ${r.environment.baseLatencyMs.toFixed(1)} ms, output latency ${r.environment.outputLatencyMs.toFixed(1)} ms`);
  line("all", r.all);
  if (r.hidden.sent > 0) {
    line("visible", r.visible);
    line("hidden", r.hidden);
  }
  out(`  scheduler: sent ${r.scheduler.sent}, late ${r.scheduler.late} (worst ${ms(r.scheduler.lateMaxMs)}), timestamps raised for order ${r.scheduler.raised} (worst ${ms(r.scheduler.raisedMaxMs)}), invalid ${r.scheduler.invalid}, unrouted ${r.scheduler.unrouted}`);
  out(`  headroom: the worst margin less the lookahead is ${ms(r.headroomAtZeroLookaheadMs)}, so with these conditions the lookahead could be that much shorter before the first event went late`);
  out(`  worklet errors: ${r.worklet.errors.length}`);
  out(`  STAND-IN ARRIVAL, not Web MIDI: matched ${r.all.matched} of ${r.all.sent}, missing ${r.all.missing}, extra ${r.all.extra}; interval error sigma ${ms(r.all.intervalErrorMs.sigma)}, max ${ms(Math.max(Math.abs(r.all.intervalErrorMs.min), Math.abs(r.all.intervalErrorMs.max)))} (this machine's timers)`);
}

const server = await serve();
const results: { label: string; result: S1Result }[] = [];
const failures: string[] = [];
const config: S1Config = { minutes: SECONDS / 60, lookaheadMs: LOOKAHEAD, alpha: ALPHA, outputId: "out1", inputId: "in1", bpm: BPM };
let browserName = "";

/** Headless Chromium through Playwright: the tab is always visible. */
async function playwrightRun(label: string): Promise<void> {
  const browser = await launch({ realisticBackground: true });
  browserName = `Chromium ${browser.version()} (headless, Playwright)`;
  const context = await browser.newContext();
  await context.addInitScript(standInScript);
  const page = await context.newPage();
  page.on("pageerror", (e) => failures.push(`${label}: page error: ${e.message}`));
  await page.goto(`${server.url}/spikes/s1/page.html`);
  await page.evaluate(() => s1.access());
  results.push({ label, result: await page.evaluate((c) => s1.run(c), config) });
  await browser.close();
}

/** Chromium started directly, headed: the second tab really puts the first in the background. */
async function rawRuns(): Promise<void> {
  const raw = await launchRaw();
  browserName = `${raw.version} (headed, driven over the DevTools protocol)`;
  const start = async (): Promise<Tab> => {
    const tab = await Tab.open(raw.cdp);
    await tab.send("Page.enable");
    await tab.send("Page.addScriptToEvaluateOnNewDocument", { source: `(${standInScript.toString()})()` });
    await tab.send("Page.navigate", { url: `${server.url}/spikes/s1/page.html` });
    const until = Date.now() + 15_000;
    while (!(await tab.evaluate<boolean>('typeof s1 !== "undefined"').catch(() => false))) {
      if (Date.now() > until) throw new Error("the S1 page did not load");
      await new Promise((ok) => setTimeout(ok, 100));
    }
    await tab.evaluate("s1.access().then(() => true)");
    return tab;
  };
  const visible = await start();
  results.push({ label: "visible tab", result: await visible.evaluate<S1Result>(`s1.run(${JSON.stringify(config)})`) });
  await visible.close();

  const background = await start();
  const running = background.evaluate<S1Result>(`s1.run(${JSON.stringify(config)})`);
  await new Promise((ok) => setTimeout(ok, 3_000));
  const front = await Tab.open(raw.cdp, "about:blank");
  await front.bringToFront();
  results.push({ label: "tab in the background", result: await running });
  await front.close();
  await background.close();
  await raw.close();
}

const dirty = execFileSync("git", ["status", "--porcelain", "--", "apps/web", "contracts"], { cwd: repoRoot }).toString().trim() !== "";
const commit = execFileSync("git", ["rev-parse", "--short", "HEAD"], { cwd: repoRoot }).toString().trim() + (dirty ? " (uncommitted changes in apps/web)" : "");
if (hidden) await rawRuns();
else await playwrightRun("visible tab");
out("# Spike S1, the half that runs here: scheduling margin with a stand-in MIDI output (SPEC-0002 P3, WENGE-0014)");
out(`# commit ${commit}, ${new Date().toISOString()}, ${browserName}`);
out("# THE ARRIVAL NUMBERS BELOW ARE FROM A TIMER IN THE PAGE, NOT FROM A MIDI PORT. Only the scheduling margins describe the page's real behaviour.");
out("# The real-port half is spikes/s1/page.html, to be run by the owner on a machine with a loopback port.");
if (hidden) out("# In the background run, the stand-in's own timer is throttled by Chromium, so its arrival numbers fall apart there. That is the stand-in, not Web MIDI; it only shows why events must not be timed by a page timer.");
for (const { label, result } of results) {
  out();
  report(label, result);
}
if (hidden) {
  const bg = results[1]?.result;
  if (!bg || bg.hiddenS < SECONDS / 2) failures.push(`the tab was hidden for only ${bg?.hiddenS.toFixed(1) ?? "0"} s of ${SECONDS}; the hidden-tab numbers are not what they say`);
}
for (const { label, result } of results) {
  if (result.worklet.errors.length) failures.push(`${label}: worklet errors: ${result.worklet.errors.join("; ")}`);
  const expected = (SECONDS * 1000 / result.stepMs) * 2 * 0.8;
  if (result.all.sent < expected) failures.push(`${label}: only ${result.all.sent} events sent in ${SECONDS} s (a step every ${result.stepMs.toFixed(0)} ms)`);
}
out();
out(failures.length ? `## RESULT: FAIL\n${failures.map((f) => "  - " + f).join("\n")}` : "## RESULT: the run completed. Nothing here is a Web MIDI jitter figure.");

if (flag("--write")) {
  const dir = join(repoRoot, "handoffs/evidence");
  mkdirSync(dir, { recursive: true });
  const name = `p3-s1-here-bpm${BPM}${hidden ? "-visible-and-hidden" : ""}`;
  writeFileSync(join(dir, `${name}.txt`), lines.join("\n") + "\n");
  writeFileSync(join(dir, `${name}.json`), JSON.stringify({ commit, browser: browserName, results, failures }, null, 1) + "\n");
}
await server.close();
process.exit(failures.length ? 1 : 0);

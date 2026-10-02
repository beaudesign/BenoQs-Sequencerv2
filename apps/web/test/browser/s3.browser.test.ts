// Spike S3's page in Chromium, with the stand-ins of spikes/s3/stand-in.ts: a clock sender that is a timer in the page, and a port that is
// a setTimeout. NOTHING HERE IS ABLETON LIVE OR A MIDI PORT. What these check is the harness: the page a person uses (the buttons, the saved
// file) and the numbers it records are what the real engine and follower did, so that the owner's run with Live is a run of something that
// works. The bounds are the plan's proposed criterion (a fifth of a step) applied to a clock that has only a timer's scatter.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { after, before, test } from "node:test";
import type { Browser, Page } from "playwright-core";
import { launch } from "../../scripts/browser.ts";
import { serve, type Server } from "../../scripts/serve.ts";
import { formatFollow, formatSend, reanalyse, readSaved } from "../../spikes/s3/format.ts";
import { standInScript } from "../../spikes/s3/stand-in.ts";
import type { FollowResult, S3, SendResult } from "../../spikes/s3/page.ts";

declare const s3: S3;
declare const sender: { start(bpm: number): void; stop(): void };

let server: Server;
let browser: Browser;

before(async () => {
  server = await serve();
  browser = await launch();
});

after(async () => {
  await browser?.close();
  await server?.close();
});

async function open(): Promise<Page> {
  const context = await browser.newContext({ acceptDownloads: true });
  await context.addInitScript(standInScript);
  const page = await context.newPage();
  page.on("pageerror", (e) => assert.fail(`page error: ${e.message}`));
  await page.goto(`${server.url}/spikes/s3/page.html`);
  return page;
}

/** Clicks through the page as a person does, up to the run's start. */
async function grant(page: Page): Promise<void> {
  await page.click("#grant");
  await page.waitForSelector("#form:not([hidden])");
  assert.match((await page.textContent("#status")) ?? "", /1 outputs and 1 inputs found/);
}

async function saved(page: Page): Promise<string> {
  const [download] = await Promise.all([page.waitForEvent("download"), page.click("#save")]);
  return readFileSync(await download.path(), "utf8");
}

test("follow, from the page's own buttons: the clock is heard, followed, recorded, saved, and the saved file reads back", async () => {
  const page = await open();
  await grant(page);
  await page.fill("#follow-minutes", "0.17"); // 10 s
  await page.fill("#follow-nominal", "120");
  await page.click("#follow-start");
  await page.waitForFunction(() => /^\d+ s: /.test(document.getElementById("status")?.textContent ?? ""));
  // Before there is a clock the status line says where to look.
  assert.match((await page.textContent("#status")) ?? "", /no clock yet/);
  await page.waitForTimeout(1_000);
  await page.evaluate(() => sender.start(120));
  await page.waitForFunction(() => document.getElementById("status")?.textContent?.startsWith("Done"), null, { timeout: 30_000 });
  assert.equal(await page.isEnabled("#save"), true);
  const text = await saved(page);
  const file = readSaved(text);
  assert.equal(file.result.schema, "wenge.s3.follow/1");
  const r = file.result as FollowResult;

  // The pulses: a Start, then about 48 a second for the rest of the run, all on the page's own clock (the simulation makes them so).
  assert.ok(r.raw.pulses.length > 8 * 48 * 0.9, `${r.raw.pulses.length} pulses`);
  assert.equal(r.input.realtime.clock, r.raw.pulses.length, "the input path's count is the page's record (a stat read after the port was let go would be 0)");
  assert.equal(r.input.realtime.start, 1);
  assert.deepEqual(r.raw.messages.map((m) => m.name), ["start"]);
  assert.equal(r.analysis.all.clockDomain.verdict, "same-clock", r.analysis.all.clockDomain.why);
  assert.ok(Math.abs(r.analysis.all.intervals.tempoBpm.fromRun - 120) < 0.5, `${r.analysis.all.intervals.tempoBpm.fromRun} BPM`);
  assert.deepEqual(r.worklet.errors, []);

  // The follower followed it: samples while running and following, and the phase within a fifth of a step (6.25 ms at 120 BPM).
  const phase = r.analysis.all.phase;
  assert.ok(phase.counted >= 20, `${phase.counted} phase samples counted`);
  assert.ok(phase.maxAbsMs <= phase.fifthOfStepMs, `the largest phase is ${phase.maxAbsMs.toFixed(2)} ms, a fifth of a step is ${phase.fifthOfStepMs.toFixed(2)}`);
  assert.equal(phase.beyondFifth, 0);
  assert.ok(r.raw.samples.some((s) => s.phase === "following" && s.running), "the follower was following and running");
  assert.ok(Math.abs(r.analysis.all.tempo.meanErrorBpm!) < 0.2, `the tempo the follower read was ${r.analysis.all.tempo.meanErrorBpm} BPM from 120`);

  // The file says the same when it is read again, and the report words it, with the plan's 30 minutes not met by a ten-second run.
  assert.deepEqual(reanalyse(r, { nominalBpm: 120 }), r.analysis.all);
  const report = formatFollow({ result: r }).join("\n");
  assert.match(report, /same-clock/);
  assert.match(report, /All \d+ samples were within a fifth of a step/);
  assert.match(report, /shorter than the 30 minutes/);
  await page.close();
});

test("follow with no clock coming in: the run ends, and the record says so and does not invent a phase", async () => {
  const page = await open();
  await page.evaluate(() => s3.access());
  const r = (await page.evaluate((c) => s3.follow(c), { minutes: 0.05, inputId: "in1", lookaheadMs: 30, offsetMs: 0, nominalBpm: null })) as FollowResult;
  assert.equal(r.raw.pulses.length, 0);
  assert.equal(r.analysis.all.clockDomain.verdict, "no-data");
  assert.equal(r.analysis.all.phase.counted, 0);
  assert.ok(r.raw.samples.length >= 8, `${r.raw.samples.length} samples in 3 s`);
  assert.ok(r.raw.samples.every((s) => s.phase === "waiting" && !s.running && s.phaseMs === null));
  assert.match(formatFollow({ result: r }).join("\n"), /No pulses were heard/);
  await page.close();
});

test("follow, and the sender stops: the Stop is recorded with the time it was seen, and the transport is not running after it", async () => {
  const page = await open();
  await page.evaluate(() => s3.access());
  const running = page.evaluate((c) => s3.follow(c), { minutes: 0.12, inputId: "in1", lookaheadMs: 30, offsetMs: 0, nominalBpm: 120 });
  await page.waitForTimeout(1_000);
  await page.evaluate(() => sender.start(120));
  await page.waitForTimeout(4_000);
  await page.evaluate(() => sender.stop());
  const r = (await running) as FollowResult;
  assert.deepEqual(r.raw.messages.map((m) => m.name), ["start", "stop"]);
  const stop = r.raw.messages[1]!;
  assert.ok(Math.abs(stop.seen - stop.stamp) < 5, "a stamp and the time it was seen are the same moment on this clock");
  const after = r.raw.samples.filter((s) => s.at > stop.seen + 600);
  assert.ok(after.length >= 4, `${after.length} samples after the Stop`);
  assert.ok(after.every((s) => !s.running && s.phaseMs === null), "after the Stop, not running and no phase");
  assert.ok(r.analysis.all.phase.skipped.stopped >= after.length);
  await page.close();
});

test("send, from the page's own buttons: every pulse the engine sent comes back, in order, with the transport messages, and the file reads back", async () => {
  const page = await open();
  await grant(page);
  await page.fill("#send-minutes", "0.12"); // 7 s
  await page.fill("#send-bpm", "120");
  await page.click("#send-start");
  await page.waitForFunction(() => document.getElementById("status")?.textContent?.startsWith("Done"), null, { timeout: 30_000 });
  const file = readSaved(await saved(page));
  assert.equal(file.result.schema, "wenge.s3.send/1");
  const r = file.result as SendResult;
  // 120 BPM is 48 pulses a second; allow for the first moments and the Stop.
  assert.ok(r.loop.sent > 7 * 48 * 0.9, `${r.loop.sent} pulses sent`);
  assert.deepEqual([r.loop.matched, r.loop.missing, r.loop.extra], [r.loop.sent, 0, 0]);
  assert.deepEqual(r.transportSent, [1, 0, 1]);
  assert.deepEqual(r.transportHeard, [1, 0, 1]);
  assert.equal(r.marginMs.late, 0, "no pulse reached the scheduler late");
  assert.deepEqual(r.worklet.errors, []);
  assert.equal(r.heardAsClock.clockDomain.verdict, "same-clock", r.heardAsClock.clockDomain.why);
  // The tempo as it came back, from the run: this machine's timers deliver a pulse a little late now and then, and the run's pulses over its time hold.
  assert.ok(Math.abs(r.heardAsClock.intervals.tempoBpm.fromRun - 120) < 1, `${r.heardAsClock.intervals.tempoBpm.fromRun} BPM`);
  const report = formatSend({ result: r }).join("\n");
  assert.match(report, new RegExp(`sent ${r.loop.sent}, heard ${r.loop.received}, matched ${r.loop.matched}, missing 0, extra 0`));
  await page.close();
});

test("follow, and the tab goes behind another and back: the page records when, for how long, and which samples were taken hidden", async () => {
  const page = await open();
  await page.evaluate(() => s3.access());
  const running = page.evaluate((c) => s3.follow(c), { minutes: 0.15, inputId: "in1", lookaheadMs: 30, offsetMs: 0, nominalBpm: 120 });
  await page.waitForTimeout(1_000);
  await page.evaluate(() => sender.start(120));
  // Headless Chromium keeps its one page visible, so the page's own view of it is changed and the event the browser would send is sent.
  const setState = (state: string): Promise<void> =>
    page.evaluate((s) => {
      Object.defineProperty(document, "visibilityState", { configurable: true, get: () => s });
      document.dispatchEvent(new Event("visibilitychange"));
    }, state);
  await page.waitForTimeout(3_000);
  await setState("hidden");
  await page.waitForTimeout(3_000);
  await setState("visible");
  const r = (await running) as FollowResult;
  assert.deepEqual(r.visibility.map((v) => v.state), ["visible", "hidden", "visible"]);
  assert.ok(Math.abs(r.hiddenS - 3) < 0.6, `hidden for ${r.hiddenS.toFixed(2)} s`);
  const hiddenSamples = r.raw.samples.filter((s) => s.hidden);
  assert.ok(hiddenSamples.length >= 8 && hiddenSamples.length <= 14, `${hiddenSamples.length} samples taken hidden in 3 s`);
  assert.ok(r.raw.samples.some((s) => !s.hidden), "and some visible");
  const { all, visible, hidden } = r.analysis;
  assert.ok(hidden.phase.counted > 0 && visible.phase.counted > 0);
  assert.equal(all.phase.counted, visible.phase.counted + hidden.phase.counted);
  assert.match(formatFollow({ result: r }).join("\n"), /tab hidden: counted \d+/);
  await page.close();
});

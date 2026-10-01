// The real host, worklet and scheduler in Chromium, with a recording MIDI output (Web MIDI itself cannot be
// granted in headless Chromium, so the output is a stand-in; spike S1 covers the port). Needs `npm run build`
// first, which `npm run test:browser` does.
import assert from "node:assert/strict";
import { after, before, test } from "node:test";
import type { Browser, Page } from "playwright-core";
import { launch } from "../../scripts/browser.ts";
import { serve, type Server } from "../../scripts/serve.ts";
import type { Harness } from "../../pages/host.ts";

declare const harness: Harness;

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
  const page = await browser.newPage();
  page.on("pageerror", (e) => assert.fail(`page error: ${e.message}`));
  await page.goto(`${server.url}/pages/host.html`);
  return page;
}

test("the module is compiled on the page, the worklet starts, and the audio context says its rate", async () => {
  const page = await open();
  const info = await page.evaluate(() => harness.start());
  assert.ok(info.sampleRate >= 8_000 && info.sampleRate <= 192_000, `${info.sampleRate}`);
  assert.deepEqual(await page.evaluate(() => harness.errors), []);
  await page.close();
});

test("a click on a matrix key lights its LED green, as the panel controller says", async () => {
  const page = await open();
  await page.evaluate(() => harness.start());
  await page.evaluate(() => harness.click("matrix.r2.c3"));
  await page.waitForFunction(() => harness.lastLeds() !== null, null, { timeout: 5_000 });
  const n = await page.evaluate(async () => {
    const doc = (await (await fetch("/contracts/controls.json")).json()) as { controls: { n: number; id: string }[] };
    return doc.controls.find((c) => c.id === "matrix.r2.c3")!.n;
  });
  assert.equal(await page.evaluate((i) => harness.lastLeds()![i], n), 2, "green, steady");
  assert.equal(await page.evaluate(() => harness.stats().sent), 0, "nothing plays until Play");
  await page.close();
});

test("Play: Note Ons reach the scheduler in the future, with the margin the lookahead gives", async () => {
  const page = await open();
  await page.evaluate(() => harness.start({ lookaheadMs: 30 }));
  await page.evaluate(() => {
    for (const s of [1, 5, 9, 13]) harness.click(`matrix.r0.c${s}`);
    harness.tempo(240);
    harness.transport(true);
  });
  await page.waitForFunction(() => harness.sends.length >= 8, null, { timeout: 8_000 });
  const { sends, stats, status, errors } = await page.evaluate(() => ({ sends: harness.sends.slice(0, 40), stats: harness.stats(), status: harness.status(), errors: harness.errors }));
  assert.deepEqual(errors, []);
  assert.equal(status.running, true);
  const first = sends[0]!;
  assert.equal(first.bytes[0]! & 0xf0, 0x90, "the first event is a Note On");
  assert.ok(first.bytes[2]! >= 1, "with a velocity of at least 1");
  assert.ok(typeof first.timestamp === "number" && first.timestamp > first.wall, "stamped in the future");
  const margins = sends.map((s) => s.marginMs).sort((a, b) => a - b);
  const median = margins[Math.floor(margins.length / 2)]!;
  assert.ok(median >= 15, `median margin ${median.toFixed(1)} ms with a 30 ms lookahead`);
  assert.ok(stats.late <= Math.ceil(stats.sent * 0.1), `${stats.late} of ${stats.sent} events were already late`);
  assert.equal(stats.invalid, 0);
  assert.equal(stats.unrouted, 0);
  await page.close();
});

test("Stop: the transport stops, and a receiver that plays the sends in timestamp order holds no note", async () => {
  const page = await open();
  await page.evaluate(() => harness.start());
  await page.evaluate(() => {
    for (const s of [1, 3, 5, 7, 9, 11, 13, 15]) harness.click(`matrix.r0.c${s}`);
    harness.tempo(240);
    harness.transport(true);
  });
  await page.waitForFunction(() => harness.sends.length >= 6, null, { timeout: 8_000 });
  await page.evaluate(() => harness.transport(false));
  await page.waitForFunction(() => !harness.status().running, null, { timeout: 5_000 });
  const sentAtStop = await page.evaluate(() => harness.sends.length);
  await page.waitForTimeout(600);
  const { held, sends, errors } = await page.evaluate(() => ({ held: harness.held(), sends: harness.sends.slice(), errors: harness.errors }));
  assert.deepEqual(errors, []);
  assert.deepEqual(held, [], "no note is left on");
  const flush = sends.slice(sentAtStop);
  assert.ok(flush.every((s) => (s.bytes[0]! & 0xf0) !== 0x90), "nothing starts after Stop");
  await page.close();
});

test("the panel is sent while the page is idle: the playhead moves with the transport", async () => {
  const page = await open();
  await page.evaluate(() => harness.start());
  await page.evaluate(() => {
    harness.click("matrix.r0.c1");
    harness.tempo(240);
    harness.transport(true);
  });
  const seen = new Set<number>();
  const until = Date.now() + 2_000;
  while (Date.now() < until) {
    seen.add((await page.evaluate(() => harness.status().playheads[0]))!);
    await page.waitForTimeout(40);
  }
  assert.ok(seen.size >= 4, `the playhead visited ${[...seen]}`);
  await page.close();
});

test("a worklet that is given a layout the controller refuses reports it instead of hanging", async () => {
  const page = await open();
  const message = await page.evaluate(async () => {
    const path = "/dist/src/host.js";
    const { Host } = (await import(path)) as typeof import("../../src/host.ts");
    const wasm = await (await fetch("/dist/octoweb.wasm")).arrayBuffer();
    try {
      await Host.start({ wasm, layout: new TextEncoder().encode("1 not.a.control\n"), workletUrl: "/dist/src/worklet.js", readyTimeoutMs: 8_000 });
      return "started";
    } catch (e) {
      return String(e);
    }
  });
  assert.notEqual(message, "started");
  assert.match(message, /could not start/, `reported by the processor error, not by the timeout: ${message}`);
  await page.close();
});

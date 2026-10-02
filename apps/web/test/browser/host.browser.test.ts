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

/**
 * Waits until the worklet has been running for a few dozen milliseconds. Chromium (141, headless) starts an AudioContext with a jump: the
 * second `process()` call has a `currentFrame` 896 or 1024 frames (7 or 8 blocks) after the first, and the engine, which renders only the
 * blocks it is called for, has nothing for the blocks between. A Play that lands in that window has a gap of about 20 ms in its first
 * pulse (measured: handoffs/evidence/p4d-startup-frame-jump.txt), which a test of exact pulse spacing is not about. The app never plays
 * that early (a person presses Start, then Play); the tests that measure spacing wait.
 */
async function pastStartup(page: Page): Promise<void> {
  await page.waitForFunction(() => harness.panelCount() >= 8, null, { timeout: 5_000 });
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

test("the clock: with it on, Start and then a pulse every 24th of a quarter note reach every device, in the future, through the real worklet", async () => {
  const page = await open();
  await page.evaluate(() => harness.start({ lookaheadMs: 30 }));
  await pastStartup(page);
  await page.evaluate(() => {
    harness.tempo(120);
    harness.clock(true);
    harness.transport(true);
  });
  // Two devices hear every message, so 120 sends are 60 pulses: a little over a second of audio.
  await page.waitForFunction(() => harness.sends.filter((s) => s.port === 0).length >= 120, null, { timeout: 10_000 });
  const { sends, stats, errors } = await page.evaluate(() => ({ sends: harness.sends.filter((s) => s.port === 0).slice(0, 160), stats: harness.stats(), errors: harness.errors }));
  assert.deepEqual(errors, []);
  assert.equal(sends[0]!.bytes[0], 0xfa, "Start first");
  assert.equal(sends[1]!.bytes[0], 0xfa, "to the second device too");
  assert.equal(stats.invalid, 0);
  assert.equal(stats.unrouted, 0);
  assert.equal(stats.sent, 0, "no notes: the pattern is empty, and `sent` counts notes");
  assert.ok(stats.realtime >= 120);
  const pulses = sends.filter((s) => s.bytes[0] === 0xf8);
  const times = [...new Set(pulses.map((s) => s.audioTime))].sort((a, b) => a - b);
  assert.ok(times.length >= 50, `${times.length} distinct pulses`);
  for (let i = 1; i < times.length; i++) {
    // 120 BPM: a pulse every 20.833 ms. The engine places each on a whole sample, so a rate that does not divide it evenly (44.1 kHz) moves one by under a sample.
    assert.ok(Math.abs((times[i]! - times[i - 1]!) * 1000 - 1000 / 48) < 0.05, `pulse ${i} came ${((times[i]! - times[i - 1]!) * 1000).toFixed(3)} ms after the one before`);
  }
  assert.ok(Math.abs(((times.at(-1)! - times[0]!) * 1000) / (times.length - 1) - 1000 / 48) < 0.01, "and on average to a hundredth of a millisecond");
  const margins = pulses.map((s) => s.marginMs).sort((a, b) => a - b);
  assert.ok(margins[Math.floor(margins.length / 2)]! >= 15, `median margin ${margins[Math.floor(margins.length / 2)]!.toFixed(1)} ms with a 30 ms lookahead`);
  const stamped = sends.filter((s) => s.timestamp !== null).map((s) => s.timestamp!);
  assert.deepEqual(stamped, [...stamped].sort((a, b) => a - b), "timestamps on a device never go backwards");
  await page.close();
});

test("the clock: Stop is heard after the last pulse and Play again says Continue", async () => {
  const page = await open();
  await page.evaluate(() => harness.start({ lookaheadMs: 30 }));
  await pastStartup(page);
  await page.evaluate(() => {
    harness.clock(true);
    harness.transport(true);
  });
  await page.waitForFunction(() => harness.sends.filter((s) => s.bytes[0] === 0xf8).length >= 20, null, { timeout: 10_000 });
  await page.evaluate(() => harness.transport(false));
  await page.waitForFunction(() => harness.sends.some((s) => s.bytes[0] === 0xfc), null, { timeout: 5_000 });
  await page.evaluate(() => harness.transport(true));
  await page.waitForFunction(() => harness.sends.some((s) => s.bytes[0] === 0xfb), null, { timeout: 5_000 });
  const { all, errors } = await page.evaluate(() => ({ all: harness.sends.filter((s) => s.port === 0).map((s) => [s.bytes[0]!, s.audioTime] as const), errors: harness.errors }));
  assert.deepEqual(errors, []);
  const stop = all.find(([b]) => b === 0xfc)!;
  assert.ok(all.every(([b, t]) => b !== 0xf8 || t < stop[1] || t > all.find(([x]) => x === 0xfb)![1]), "no pulse between Stop and Continue");
  assert.equal(all.filter(([b]) => b === 0xfc).length, 2, "one Stop, once to each device");
  assert.ok(all.find(([b]) => b === 0xfb)![1] > stop[1], "Continue after Stop");
  await page.close();
});

test("the clock: turned off while it runs, no more pulses reach a device once what was already on its way has gone", async () => {
  const page = await open();
  await page.evaluate(() => harness.start({ lookaheadMs: 30 }));
  await pastStartup(page);
  await page.evaluate(() => {
    harness.clock(true);
    harness.transport(true);
  });
  await page.waitForFunction(() => harness.sends.filter((s) => s.bytes[0] === 0xf8).length >= 20, null, { timeout: 10_000 });
  await page.evaluate(() => harness.clock(false));
  await page.waitForTimeout(400); // what the worklet had already posted, and the messages in flight, arrive in this time
  const settled = await page.evaluate(() => harness.sends.filter((s) => s.bytes[0] === 0xf8).length);
  await page.waitForTimeout(600);
  const later = await page.evaluate(() => ({ pulses: harness.sends.filter((s) => s.bytes[0] === 0xf8).length, stops: harness.sends.filter((s) => s.bytes[0] === 0xfc).length, errors: harness.errors }));
  assert.equal(later.pulses, settled, "a second of 120 BPM would have been 48 more pulses to each device");
  assert.equal(later.stops, 0, "turning the clock off sends no Stop: the transport is still running");
  assert.deepEqual(later.errors, []);
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

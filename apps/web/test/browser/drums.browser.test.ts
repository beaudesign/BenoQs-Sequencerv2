// The Drums button in the page (specs/SPEC-0002/p10-silence-the-drums.md, observables P10-6 to P10-8). The owner, Tue 2026-10-13 12:00 London:
// "a button where you can silence the drums - it should be purely the synth and all of the midi that you can hear".
//
// Heard is measured as in runs.browser.test.ts: an analyser beside the audio node's connection to the speakers. The pattern is the page's own `?bare`
// with one track routed to a channel (`?route=`) and four keys of its row pressed, so what is heard is one track and nothing else.
// This browser keeps Chromium's own autoplay policy: audio starts only because a click started it, as in a visitor's browser.
import assert from "node:assert/strict";
import { after, before, test } from "node:test";
import { chromium, type Browser, type Page } from "playwright-core";
import { serve, type Server } from "../../scripts/serve.ts";

let server: Server;
let browser: Browser;

before(async () => {
  server = await serve();
  const path = process.env["CHROMIUM_PATH"];
  browser = await chromium.launch({ headless: true, ...(path ? { executablePath: path } : {}), args: ["--disable-background-timer-throttling", "--disable-renderer-backgrounding"] });
});

after(async () => {
  await browser?.close();
  await server?.close();
});

declare global {
  interface Window {
    __tap: { peakOver(ms: number): number };
    __drumsOut: { sent: { data: number[] }[] };
  }
}

/** Taps what the page's audio node plays, and gives the page one named MIDI output that records what is sent to it. */
function install(): void {
  const proto = AudioNode.prototype as unknown as { connect: (...a: unknown[]) => unknown };
  const real = proto.connect;
  let made = false;
  proto.connect = function (this: AudioNode, ...args: unknown[]) {
    const result = real.apply(this, args);
    if (!made && this instanceof AudioWorkletNode) {
      made = true;
      const analyser = this.context.createAnalyser();
      analyser.fftSize = 2048;
      real.call(this, analyser);
      const buffer = new Float32Array(analyser.fftSize);
      const seen: { at: number; peak: number }[] = [];
      setInterval(() => {
        analyser.getFloatTimeDomainData(buffer);
        let peak = 0;
        for (const x of buffer) peak = Math.max(peak, Math.abs(x));
        seen.push({ at: performance.now(), peak });
        if (seen.length > 400) seen.shift();
      }, 20);
      window.__tap = { peakOver: (ms) => seen.filter((s) => s.at > performance.now() - ms).reduce((m, s) => Math.max(m, s.peak), 0) };
    }
    return result;
  };
  const sent: { data: number[] }[] = [];
  const out = { id: "fake-1", name: "Fake output", send: (data: ArrayLike<number>) => void sent.push({ data: Array.from(data) }) };
  window.__drumsOut = { sent };
  Object.defineProperty(Navigator.prototype, "requestMIDIAccess", {
    configurable: true,
    value: () => Promise.resolve({ outputs: new Map([[out.id, out]]), inputs: new Map(), onstatechange: null }),
  });
}

const face = (id: string): string => `g[data-id="${id}"] .face`;

/** A page with track 0 routed to `channel`, the sound on, four keys of its row pressed (every fourth step), and the transport running. */
async function playing(channel: number): Promise<Page> {
  const page = await browser.newPage({ viewport: { width: 1280, height: 900 } });
  page.on("pageerror", (e) => assert.fail(`page error: ${e.message}`));
  await page.addInitScript(install);
  await page.goto(`${server.url}/pages/app.html?bare&route=0:${channel}`);
  await page.waitForSelector('main[data-engine="idle"]');
  await page.click("#play");
  await page.waitForSelector('main[data-engine="running"]', { timeout: 15_000 });
  await page.selectOption("#midi-out-1", "fake-1");
  await page.click("#monitor");
  for (const c of [1, 5, 9, 13]) await page.click(face(`matrix.r0.c${c}`));
  await page.click(face("transport.play"));
  return page;
}

const noteOns = (page: Page, channel: number): Promise<number> =>
  page.evaluate((ch) => window.__drumsOut.sent.filter((s) => (s.data[0]! & 0xf0) === 0x90 && (s.data[0]! & 0x0f) === ch - 1 && (s.data[2] ?? 0) > 0).length, channel);

test("P10-6: the strip has a Drums toggle after Sound, on to begin with, that a click turns off and on", async () => {
  const page = await browser.newPage({ viewport: { width: 1280, height: 900 } });
  await page.goto(`${server.url}/pages/app.html?bare`);
  await page.waitForSelector('main[data-engine="idle"]');
  assert.equal((await page.textContent("#drums"))?.trim(), "Drums");
  assert.equal(await page.getAttribute("#drums", "aria-pressed"), "true", "the drums are heard to begin with, in a bare page too");
  assert.equal(await page.$eval("#drums", (el) => el.previousElementSibling?.id), "monitor", "it is next to Sound, after it");
  await page.click("#drums");
  assert.equal(await page.getAttribute("#drums", "aria-pressed"), "false");
  await page.click("#drums");
  assert.equal(await page.getAttribute("#drums", "aria-pressed"), "true");
  await page.focus("#monitor");
  await page.keyboard.press("Tab");
  assert.equal(await page.evaluate(() => document.activeElement?.id), "drums", "the next stop for the Tab key after Sound");
  await page.keyboard.press("Space");
  assert.equal(await page.getAttribute("#drums", "aria-pressed"), "false", "and the keyboard works it");
  await page.close();
});

test("P10-7: a track on channel 10 is heard, is silent a moment after Drums is turned off, is heard again when it is back on, and the output still gets its Note Ons", async () => {
  const page = await playing(10);
  await page.waitForFunction(() => window.__tap !== undefined && window.__tap.peakOver(600) > 0.02, null, { timeout: 6_000 });
  await page.waitForFunction(() => window.__drumsOut.sent.length >= 2, null, { timeout: 6_000 });
  assert.ok((await noteOns(page, 10)) >= 1, "the drum notes go to the output on channel 10");
  await page.click("#drums");
  assert.equal(await page.getAttribute("#drums", "aria-pressed"), "false");
  await page.waitForTimeout(900); // a hit that was sounding finishes: at most 0.4 s
  assert.equal(await page.evaluate(() => window.__tap.peakOver(400)), 0, "silent, exactly: not a sample of the drums");
  const before = await noteOns(page, 10);
  await page.waitForTimeout(1500);
  const after = await noteOns(page, 10);
  assert.ok(after >= before + 2, `and the output still gets every drum note: ${before} Note Ons, then ${after} in 1.5 s`);
  assert.equal(await page.evaluate(() => window.__tap.peakOver(400)), 0, "while the output is getting them the page is still silent");
  await page.click("#drums");
  await page.waitForFunction(() => window.__tap.peakOver(600) > 0.02, null, { timeout: 6_000 });
  await page.close();
});

test("P10-8: a track on a synth channel is still heard with Drums off", async () => {
  const page = await playing(2);
  await page.click("#drums");
  assert.equal(await page.getAttribute("#drums", "aria-pressed"), "false");
  await page.waitForFunction(() => window.__tap !== undefined && window.__tap.peakOver(600) > 0.02, null, { timeout: 6_000 });
  await page.waitForTimeout(1200);
  assert.ok((await page.evaluate(() => window.__tap.peakOver(700))) > 0.02, "the synth is heard, a second on");
  assert.ok((await noteOns(page, 2)) >= 2, "and sent");
  await page.close();
});

test("P10-9: a choice made before Play is kept: Drums turned off first, the drum track is silent from its first hit, and the output still gets it", async () => {
  const page = await browser.newPage({ viewport: { width: 1280, height: 900 } });
  page.on("pageerror", (e) => assert.fail(`page error: ${e.message}`));
  await page.addInitScript(install);
  await page.goto(`${server.url}/pages/app.html?bare&route=0:10`);
  await page.waitForSelector('main[data-engine="idle"]');
  await page.click("#drums");
  await page.click("#monitor");
  await page.click("#play");
  await page.waitForSelector('main[data-engine="running"]', { timeout: 15_000 });
  await page.selectOption("#midi-out-1", "fake-1");
  for (const c of [1, 5, 9, 13]) await page.click(face(`matrix.r0.c${c}`));
  await page.click(face("transport.play"));
  await page.waitForFunction(() => window.__drumsOut.sent.filter((s) => (s.data[0]! & 0xf0) === 0x90).length >= 3, null, { timeout: 8_000 });
  assert.equal(await page.evaluate(() => window.__tap.peakOver(2000)), 0, "three drum notes were sent and not one was heard");
  assert.equal(await page.getAttribute("#drums", "aria-pressed"), "false");
  await page.close();
});

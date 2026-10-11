// The page as a person meets it (specs/SPEC-0002/p6-runs-when-opened.md): opened, with nothing connected, one press of Play and it runs.
// R-1 the demo pattern is lit, R-2 it is heard, R-3 a red light moves along the steps with the sound, R-4 there is one Play, R-5 the
// tempo is a field, R-6 the sound can be turned off, R-7 Stop stops all of it.
//
// This browser keeps Chromium's own autoplay policy (the other browser tests turn it off): audio starts only because a click started it,
// which is how a visitor's browser behaves.
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
  }
}

/** Taps what the page's audio node plays, so the test can hear it: an analyser beside the node's connection to the speakers. */
function installTap(): void {
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
}

async function open(query = ""): Promise<Page> {
  const page = await browser.newPage({ viewport: { width: 1280, height: 900 } });
  page.on("pageerror", (e) => assert.fail(`page error: ${e.message}`));
  await page.addInitScript(installTap);
  await page.goto(`${server.url}/pages/app.html${query}`);
  await page.waitForSelector('main[data-engine="idle"]');
  return page;
}

const face = (id: string): string => `g[data-id="${id}"] .face`;

/** The matrix keys that are red now, as [track, step]. */
function redKeys(page: Page): Promise<[number, number][]> {
  return page.evaluate(() =>
    [...document.querySelectorAll<SVGElement>('g[data-id^="matrix."] .face.led[data-colour="red"]')].map((el) => {
      const m = /^matrix\.r(\d+)\.c(\d+)$/.exec(el.closest("g")!.getAttribute("data-id")!)!;
      return [Number(m[1]), Number(m[2]) - 1] as [number, number];
    }),
  );
}

function litCount(page: Page): Promise<number> {
  return page.evaluate(() => document.querySelectorAll('g[data-id^="matrix."] .face.led:not([data-colour="off"])').length);
}

/** Samples where track `track`'s red light is, every 15 ms for `ms`. `null` is no red light on that row. */
function followLight(page: Page, track: number, ms: number): Promise<(number | null)[]> {
  return page.evaluate(
    ([t, duration]) =>
      new Promise<(number | null)[]>((resolve) => {
        const out: (number | null)[] = [];
        const started = performance.now();
        const tick = (): void => {
          const reds = [...document.querySelectorAll<SVGElement>(`g[data-id^="matrix.r${t}."] .face.led[data-colour="red"]`)];
          const m = reds.length ? /\.c(\d+)$/.exec(reds[0]!.closest("g")!.getAttribute("data-id")!) : null;
          out.push(m ? Number(m[1]) - 1 : null);
          if (performance.now() - started < duration) setTimeout(tick, 15);
          else resolve(out);
        };
        tick();
      }),
    [track, ms] as const,
  );
}

/** How many times the light changed step in the samples. */
function moves(seen: (number | null)[]): number {
  let n = 0;
  for (let i = 1; i < seen.length; i++) {
    const a = seen[i - 1];
    const b = seen[i];
    if (a !== null && a !== undefined && b !== null && b !== undefined && a !== b) n++;
  }
  return n;
}

test("R-4: there is one Play, and one press of it starts the engine and plays: no Start, no second press", async () => {
  const page = await open();
  assert.equal(await page.locator("#start").count(), 0, "no Start button");
  assert.equal((await page.textContent("#play"))?.trim(), "Play");
  await page.click("#play");
  await page.waitForSelector('main[data-engine="running"][data-transport="playing"]', { timeout: 15_000 });
  assert.equal((await page.textContent("#play"))?.trim(), "Stop", "and the same button stops it");
  await page.close();
});

test("R-1: the demo pattern is lit on the panel with nothing pressed but Play", async () => {
  const page = await open();
  assert.equal(await litCount(page), 0, "before Play the panel is dark: there is no engine to light it yet");
  await page.click("#play");
  await page.waitForSelector('main[data-transport="playing"]', { timeout: 15_000 });
  await page.waitForFunction(() => document.querySelectorAll('g[data-id^="matrix."] .face.led:not([data-colour="off"])').length >= 12, null, { timeout: 5_000 });
  assert.ok((await litCount(page)) >= 12);
  await page.close();
});

test("R-2: it is heard within a second of Play, with no MIDI device and no other setting", async () => {
  const page = await open();
  await page.click("#play");
  await page.waitForFunction(() => window.__tap !== undefined && window.__tap.peakOver(250) > 0.02, null, { timeout: 4_000 });
  await page.close();
});

test("R-3: a red light moves along the steps, one step at a time, and visits the whole bar", async () => {
  const page = await open();
  await page.click("#play");
  await page.waitForSelector('main[data-transport="playing"]', { timeout: 15_000 });
  await page.waitForTimeout(400);
  const seen = await followLight(page, 0, 2600); // 20 steps at 120 BPM
  const visited = new Set(seen.filter((s): s is number => s !== null));
  assert.ok(visited.size >= 15, `the light on track 1 visited ${visited.size} steps: ${[...visited].sort((a, b) => a - b)}`);
  const present = seen.filter((s) => s !== null).length;
  assert.ok(present > seen.length * 0.9, "there is a light on the row nearly all the time");
  let backwards = 0;
  for (let i = 1; i < seen.length; i++) {
    const a = seen[i - 1];
    const b = seen[i];
    if (a !== null && a !== undefined && b !== null && b !== undefined && b !== a && b !== (a + 1) % 16 && b !== (a + 2) % 16) backwards++;
  }
  assert.equal(backwards, 0, "it goes forward, a step or at most two between samples, and wraps from 16 to 1");
  assert.ok(moves(seen) >= 15 && moves(seen) <= 26, `${moves(seen)} moves in 2.6 s at 120 BPM (a step is 125 ms: about 21)`);
  const reds = await redKeys(page);
  assert.equal(new Set(reds.map(([t]) => t)).size, reds.length, "one light on each row, and no row has two");
  await page.close();
});

test("R-5: the tempo is a field, 120 to begin with, and the light goes twice as fast at 240", async () => {
  const page = await open();
  assert.equal(await page.inputValue("#tempo"), "120");
  await page.click("#play");
  await page.waitForSelector('main[data-transport="playing"]', { timeout: 15_000 });
  await page.waitForTimeout(400);
  const slow = moves(await followLight(page, 0, 2000));
  await page.fill("#tempo", "240");
  await page.dispatchEvent("#tempo", "change");
  await page.waitForTimeout(500);
  const fast = moves(await followLight(page, 0, 2000));
  assert.ok(fast / slow > 1.6 && fast / slow < 2.4, `${slow} moves in 2 s at 120 and ${fast} at 240`);
  await page.fill("#tempo", "99999");
  await page.dispatchEvent("#tempo", "change");
  assert.notEqual(await page.inputValue("#tempo"), "99999", "a number the engine would refuse is put back");
  await page.close();
});

test("R-6: the sound is on to begin with, can be turned off without stopping the light, and back on", async () => {
  const page = await open();
  assert.equal(await page.getAttribute("#monitor", "aria-pressed"), "true");
  await page.click("#play");
  await page.waitForFunction(() => window.__tap !== undefined && window.__tap.peakOver(250) > 0.02, null, { timeout: 4_000 });
  await page.click("#monitor");
  assert.equal(await page.getAttribute("#monitor", "aria-pressed"), "false");
  await page.waitForTimeout(600);
  assert.equal(await page.evaluate(() => window.__tap.peakOver(300)), 0, "silent");
  assert.ok(moves(await followLight(page, 0, 1200)) >= 6, "and the light goes on");
  await page.click("#monitor");
  await page.waitForFunction(() => window.__tap.peakOver(250) > 0.02, null, { timeout: 4_000 });
  await page.close();
});

test("R-7: Stop stops the sound and puts the light out, and Play again starts it again", async () => {
  const page = await open();
  await page.click("#play");
  await page.waitForFunction(() => window.__tap !== undefined && window.__tap.peakOver(250) > 0.02, null, { timeout: 4_000 });
  await page.click("#play");
  await page.waitForSelector('main[data-transport="stopped"]', { timeout: 5_000 });
  await page.waitForTimeout(700);
  assert.equal(await page.evaluate(() => window.__tap.peakOver(300)), 0, "silent");
  assert.deepEqual(await redKeys(page), [], "no light");
  assert.ok((await litCount(page)) >= 12, "the pattern stays lit");
  assert.equal((await page.textContent("#play"))?.trim(), "Play");
  await page.click("#play");
  await page.waitForFunction(() => window.__tap.peakOver(250) > 0.02, null, { timeout: 4_000 });
  await page.close();
});

test("R-4: the Play key on the panel starts the engine and plays, the same as the button", async () => {
  const page = await open();
  await page.click(face("transport.play"));
  await page.waitForSelector('main[data-engine="running"][data-transport="playing"]', { timeout: 15_000 });
  await page.waitForFunction(() => window.__tap !== undefined && window.__tap.peakOver(250) > 0.02, null, { timeout: 4_000 });
  await page.close();
});

test("R-8: ?bare is the page the earlier tests drive: Play only starts the engine, nothing is loaded, nothing plays, nothing is heard", async () => {
  const page = await open("?bare");
  assert.equal((await page.textContent("#play"))?.trim(), "Play");
  await page.click("#play");
  await page.waitForSelector('main[data-engine="running"]', { timeout: 15_000 });
  assert.equal(await page.getAttribute("main", "data-transport"), "stopped");
  assert.equal(await litCount(page), 0);
  assert.equal(await page.getAttribute("#monitor", "aria-pressed"), "false");
  assert.equal(await page.inputValue("#tempo"), "120");
  await page.close();
});

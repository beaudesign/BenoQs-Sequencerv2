// The page as a visitor meets it on a project site (specs/SPEC-0002/p11-published-page.md): the folder `scripts/site.ts` assembles, served
// under an address prefix with no cross-origin-isolation headers, the way GitHub Pages serves `https://<owner>.github.io/<repo>/`.
// P11-1 the page runs from under a prefix, P11-2 every file it loads comes from inside the site, P11-3 the site's own address opens it,
// P11-4 one press of Play plays and is heard, P11-5 nothing in the run fails or logs an error.
//
// Like runs.browser.test.ts this browser keeps Chromium's own autoplay policy: audio starts only because a click started it.
import assert from "node:assert/strict";
import { mkdtempSync, readdirSync, rmSync, statSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { after, before, test } from "node:test";
import { chromium, type Browser, type Page } from "playwright-core";
import { assembleSite } from "../../scripts/site.ts";
import { serveFolder, type Server } from "../../scripts/serve.ts";

const PREFIX = "/BenoQs-Sequencerv2";

let dir: string;
let server: Server;
let browser: Browser;

before(async () => {
  dir = mkdtempSync(join(tmpdir(), "wenge-site-"));
  assembleSite(dir);
  server = await serveFolder(dir, PREFIX);
  const path = process.env["CHROMIUM_PATH"];
  browser = await chromium.launch({ headless: true, ...(path ? { executablePath: path } : {}), args: ["--disable-background-timer-throttling", "--disable-renderer-backgrounding"] });
});

after(async () => {
  await browser?.close();
  await server?.close();
  if (dir) rmSync(dir, { recursive: true, force: true });
});

declare global {
  interface Window {
    __tap: { peakOver(ms: number): number };
  }
}

/** Taps what the page's audio node plays, so the test can hear it (the same tap as runs.browser.test.ts). */
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

interface Run {
  page: Page;
  requests: { url: string; status: number }[];
  problems: string[];
}

/** Opens `path` on the site, and records every response and every error until it is closed. */
async function open(path: string): Promise<Run> {
  const page = await browser.newPage({ viewport: { width: 1280, height: 900 } });
  const run: Run = { page, requests: [], problems: [] };
  page.on("response", (r) => run.requests.push({ url: r.url(), status: r.status() }));
  page.on("pageerror", (e) => run.problems.push(`page error: ${e.message}`));
  page.on("console", (m) => m.type() === "error" && run.problems.push(`console error: ${m.text()}`));
  await page.addInitScript(installTap);
  await page.goto(`${server.url}${PREFIX}${path}`);
  await page.waitForSelector('main[data-engine="idle"]', { timeout: 10_000 });
  // `main` is in the page's own HTML, so it is there before the script has fetched anything: the panel is what says the files came.
  await page.waitForSelector("svg g[data-id]", { timeout: 10_000 });
  return run;
}

function files(root: string, here = ""): string[] {
  return readdirSync(join(root, here)).flatMap((name) => {
    const rel = here ? `${here}/${name}` : name;
    return statSync(join(root, rel)).isDirectory() ? files(root, rel) : [rel];
  });
}

test("P11-1: the page runs from under an address prefix: it draws the panel and the engine is idle and ready", async () => {
  const { page, problems } = await open("/pages/app.html");
  assert.equal(await page.$$eval('svg g[data-id][role="button"]', (els) => els.length), 252, "all 252 controls are drawn");
  assert.equal(await page.locator("#play").count(), 1, "and there is a Play");
  assert.deepEqual(problems, []);
  await page.close();
});

test("P11-2: every file the page loads comes from inside the site, and none fails", async () => {
  const run = await open("/pages/app.html");
  await run.page.click("#play");
  await run.page.waitForSelector('main[data-engine="running"]', { timeout: 15_000 });
  const outside = run.requests.filter((r) => !r.url.startsWith(`${server.url}${PREFIX}/`));
  assert.deepEqual(outside, [], "nothing is fetched from outside the prefix");
  const failed = run.requests.filter((r) => r.status >= 400);
  assert.deepEqual(failed, [], "and nothing comes back with an error");
  // The worklet's own module is fetched from the audio thread, which the page's response events do not see; P11-4 hears it play.
  for (const wanted of ["/dist/pages/app.js", "/dist/octoweb.wasm", "/contracts/controls.json", "/contracts/design.tokens.json", "/layout/panel.layout.json", "/dist/tokens.css", "/pages/app.css", "/pages/fonts/inter-latin-wght-normal.woff2"]) {
    assert.ok(run.requests.some((r) => r.url === `${server.url}${PREFIX}${wanted}`), `${wanted} was loaded from the site`);
  }
  await run.page.close();
});

test("P11-3: the site's own address opens the app", async () => {
  const { page, problems } = await open("/");
  assert.ok(page.url().endsWith(`${PREFIX}/pages/app.html`), `the index sent the visitor to the page, not ${page.url()}`);
  assert.equal(await page.$$eval('svg g[data-id][role="button"]', (els) => els.length), 252);
  assert.deepEqual(problems, []);
  await page.close();
});

test("P11-4: one press of Play on the site plays, and it is heard", async () => {
  const { page } = await open("/pages/app.html");
  await page.click("#play");
  await page.waitForSelector('main[data-engine="running"][data-transport="playing"]', { timeout: 15_000 });
  await page.waitForFunction(() => window.__tap !== undefined && window.__tap.peakOver(250) > 0.02, null, { timeout: 4_000 });
  await page.close();
});

test("P11-5: the run says nothing is wrong: no page error and no console error from the first load to a second of playing", async () => {
  const { page, problems } = await open("/pages/app.html");
  await page.click("#play");
  await page.waitForSelector('main[data-transport="playing"]', { timeout: 15_000 });
  await page.waitForTimeout(1_000);
  assert.deepEqual(problems, []);
  await page.close();
});

test("P11-2: the site holds the files the page needs and the licence of the face it ships, and no spike, test or source", () => {
  const all = files(dir).sort();
  assert.ok(all.includes("index.html") && all.includes(".nojekyll"));
  assert.ok(all.includes("pages/fonts/inter-LICENSE.txt"), "the font's licence goes with the font");
  assert.ok(all.includes("dist/octoweb.wasm") && !all.includes("dist/octoweb-spike.wasm"), "the shipped module, not the one with the spike exports");
  assert.deepEqual(all.filter((f) => /(^|\/)spikes?\/|\.test\.|\.ts$/.test(f)), [], "no spike, no test, no TypeScript source");
});

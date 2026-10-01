// Takes the screenshots of the panel for a pull request (P3c: the owner ratifies the tokens by looking at them).
// `npm run screenshot -- <folder>` writes p3c-panel-idle.png, p3c-panel-running.png and p3c-panel-detail.png there.
import { mkdirSync } from "node:fs";
import { resolve } from "node:path";
import { launch } from "./browser.ts";
import { serve } from "./serve.ts";

const out = resolve(process.argv[2] ?? "screenshots");
mkdirSync(out, { recursive: true });

const server = await serve();
const browser = await launch();
const page = await browser.newPage({ viewport: { width: 1280, height: 900 } });
// One output with a plausible name, so the strip reads as it would with a loopback port.
await page.addInitScript(() => {
  const output = { id: "iac-1", name: "IAC Driver Bus 1", send() {} };
  Object.defineProperty(Navigator.prototype, "requestMIDIAccess", {
    configurable: true,
    value: () => Promise.resolve({ outputs: new Map([[output.id, output]]), inputs: new Map(), onstatechange: null }),
  });
});
await page.goto(`${server.url}/pages/app.html`);
await page.waitForSelector('main[data-engine="idle"]');
await page.screenshot({ path: `${out}/p3c-panel-idle.png` });

await page.click("#start");
await page.waitForSelector('main[data-engine="running"]');
await page.selectOption("#midi-output", "iac-1");
const face = (id: string) => `g[data-id="${id}"] .face`;
// A small pattern: four on the floor, the off-beats above it, and a run of sixteenths on track 2.
for (const c of [1, 5, 9, 13]) await page.click(face(`matrix.r0.c${c}`));
for (const c of [3, 7, 11, 15]) await page.click(face(`matrix.r1.c${c}`));
for (const c of [1, 2, 4, 6, 9, 10, 12, 14]) await page.click(face(`matrix.r2.c${c}`));
await page.click(face("mode.play"));
await page.waitForSelector('g[data-id="mode.play"] .led[data-phase="flash"]');
// Hold the flashing LEDs at the lit half of their period, so the picture shows them lit.
await page.evaluate(() => document.getAnimations().forEach((a) => { a.pause(); a.currentTime = 0; }));
// Keyboard focus on a lit key, to show the ring on an LED.
await page.focus('g[data-id="matrix.r0.c5"]');
await page.keyboard.press("Tab");
await page.keyboard.press("Shift+Tab");
await page.waitForTimeout(200);
await page.screenshot({ path: `${out}/p3c-panel-running.png` });

// The lower controls and a lit key with its ring, at twice the size.
const detail = await browser.newPage({ viewport: { width: 1280, height: 900 }, deviceScaleFactor: 2 });
await detail.addInitScript(() => {
  const output = { id: "iac-1", name: "IAC Driver Bus 1", send() {} };
  Object.defineProperty(Navigator.prototype, "requestMIDIAccess", { configurable: true, value: () => Promise.resolve({ outputs: new Map([[output.id, output]]), inputs: new Map(), onstatechange: null }) });
});
await detail.goto(`${server.url}/pages/app.html`);
await detail.click("#start");
await detail.waitForSelector('main[data-engine="running"]');
for (const c of [1, 5]) await detail.click(face(`matrix.r0.c${c}`));
await detail.click(face("mode.play"));
await detail.waitForSelector('g[data-id="mode.play"] .led[data-phase="flash"]');
await detail.evaluate(() => document.getAnimations().forEach((a) => { a.pause(); a.currentTime = 0; }));
await detail.focus('g[data-id="matrix.r0.c1"]');
await detail.keyboard.press("Tab");
await detail.keyboard.press("Shift+Tab");
const a = await detail.$eval('g[data-id="matrix.r0.c1"] .face', (el) => el.getBoundingClientRect().toJSON());
const b = await detail.$eval('g[data-id="transport.stop"] .face', (el) => el.getBoundingClientRect().toJSON());
await detail.screenshot({ path: `${out}/p3c-panel-detail.png`, clip: { x: a.x - 30, y: a.y - 30, width: 560, height: b.y + b.height - a.y + 230 } });

await browser.close();
await server.close();
console.log(`wrote the screenshots to ${out}`);

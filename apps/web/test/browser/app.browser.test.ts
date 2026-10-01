// The panel in Chromium: the real page, the real worklet, the real scheduler, and a fake `MIDIOutput` that records
// what it is given (Web MIDI itself cannot be granted in headless Chromium). Observables E1, E2, E7 and E8 of
// specs/SPEC-0002/p3-plan.md. Needs `npm run build` first, which `npm run test:browser` does.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { after, before, test } from "node:test";
import type { Browser, Page } from "playwright-core";
import { launch } from "../../scripts/browser.ts";
import { serve, type Server } from "../../scripts/serve.ts";
import { midiBytes } from "../../src/midi-out.ts";
import { parseLayout, place, tabOrder, type ControlsDoc } from "../../src/layout.ts";
import { parseTokens } from "../../src/tokens.ts";
import { repoRoot } from "../support/module.ts";
import { Session } from "../support/session.ts";

const read = (rel: string): unknown => JSON.parse(readFileSync(`${repoRoot}/${rel}`, "utf8"));
const controls = read("contracts/controls.json") as ControlsDoc;
const tokens = parseTokens(read("contracts/design.tokens.json"));
const geometry = place(parseLayout(read("apps/web/layout/panel.layout.json"), controls), controls, tokens);

const rgb = (hex: string): string => `rgb(${[1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16)).join(", ")})`;

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

interface FakeSend {
  data: number[];
  timestamp: number | null;
  at: number;
}

declare global {
  interface Window {
    __midi: { sent: FakeSend[] };
  }
}

/** One output, named, that records every send with the time it was made. */
function installFakeMidi(): void {
  const sent: FakeSend[] = [];
  const out = {
    id: "fake-1",
    name: "Fake output",
    send(data: ArrayLike<number>, timestamp?: number) {
      sent.push({ data: Array.from(data), timestamp: timestamp ?? null, at: performance.now() });
    },
  };
  window.__midi = { sent };
  Object.defineProperty(Navigator.prototype, "requestMIDIAccess", {
    configurable: true,
    value: () => Promise.resolve({ outputs: new Map([[out.id, out]]), inputs: new Map(), onstatechange: null }),
  });
}

function removeMidi(): void {
  delete (Navigator.prototype as { requestMIDIAccess?: unknown }).requestMIDIAccess;
}

function refuseMidi(): void {
  Object.defineProperty(Navigator.prototype, "requestMIDIAccess", {
    configurable: true,
    value: () => Promise.reject(new DOMException("denied", "SecurityError")),
  });
}

async function open(init?: () => void): Promise<Page> {
  const page = await browser.newPage({ viewport: { width: 1280, height: 900 } });
  page.on("pageerror", (e) => assert.fail(`page error: ${e.message}`));
  if (init) await page.addInitScript(init);
  await page.goto(`${server.url}/pages/app.html`);
  await page.waitForSelector('main[data-engine="idle"]');
  return page;
}

async function start(page: Page): Promise<void> {
  await page.click("#start");
  await page.waitForSelector('main[data-engine="running"]', { timeout: 15_000 });
}

const key = (id: string): string => `g[data-id="${id}"]`;
const led = (id: string): string => `${key(id)} .led`;
/** The circle itself: a pointer press lands on the key, and the label beside it is not part of the button. */
const face = (id: string): string => `${key(id)} .face`;

async function ledState(page: Page, id: string): Promise<{ colour: string | null; phase: string | null }> {
  return page.$eval(led(id), (el) => ({ colour: el.getAttribute("data-colour"), phase: el.getAttribute("data-phase") }));
}

// ---------------------------------------------------------------------------------------------------------------
// E1
// ---------------------------------------------------------------------------------------------------------------

test("E1: a press on a matrix key lights it green, and a second press turns it off", async () => {
  const page = await open(installFakeMidi);
  await start(page);
  assert.deepEqual(await ledState(page, "matrix.r2.c3"), { colour: "off", phase: "steady" });
  await page.click(face("matrix.r2.c3"));
  await page.waitForSelector(`${led("matrix.r2.c3")}[data-colour="green"]`, { timeout: 5_000 });
  assert.deepEqual(await ledState(page, "matrix.r2.c3"), { colour: "green", phase: "steady" });
  const lit = await page.$$eval("g[data-id^='matrix.'] .led:not([data-colour='off'])", (els) => els.length);
  assert.equal(lit, 1, "of the 160 matrix keys, only the one that was pressed is lit");
  await page.click(face("matrix.r2.c3"));
  await page.waitForSelector(`${led("matrix.r2.c3")}[data-colour="off"]`, { timeout: 5_000 });
  await page.close();
});

test("E1: a lit key is drawn in the token's green, and an unlit one in the token's off colour", async () => {
  const page = await open(installFakeMidi);
  await start(page);
  const fill = (id: string) => page.$eval(led(id), (el) => getComputedStyle(el).fill);
  assert.equal(await fill("matrix.r2.c3"), rgb(tokens.colour["led.off"].value));
  await page.click(face("matrix.r2.c3"));
  await page.waitForSelector(`${led("matrix.r2.c3")}[data-colour="green"]`);
  assert.equal(await fill("matrix.r2.c3"), rgb(tokens.colour["led.green"].value));
  const ground = await page.$eval("body", (el) => getComputedStyle(el).backgroundColor);
  assert.equal(ground, rgb(tokens.colour["surface"].value));
  await page.close();
});

test("a press before Start lights nothing and says what to do", async () => {
  const page = await open(installFakeMidi);
  await page.click(face("matrix.r2.c3"));
  await page.waitForTimeout(150);
  assert.equal((await ledState(page, "matrix.r2.c3")).colour, "off");
  assert.match(await page.textContent("#engine-status") ?? "", /Press Start/);
  await page.close();
});

test("a key can be pressed from the keyboard: Space presses and releases, and holding it down presses once", async () => {
  const page = await open(installFakeMidi);
  await start(page);
  await page.focus(key("matrix.r1.c2"));
  await page.keyboard.down("Space");
  await page.keyboard.down("Space"); // key repeat
  await page.keyboard.up("Space");
  await page.waitForSelector(`${led("matrix.r1.c2")}[data-colour="green"]`, { timeout: 5_000 });
  await page.keyboard.press("Enter");
  await page.waitForSelector(`${led("matrix.r1.c2")}[data-colour="off"]`, { timeout: 5_000 });
  await page.close();
});

test("a click that no pointer made, as a screen reader's Activate sends it, presses the key once", async () => {
  const page = await open(installFakeMidi);
  await start(page);
  await page.$eval(key("matrix.r1.c2"), (el) => el.dispatchEvent(new MouseEvent("click", { bubbles: true, detail: 0 })));
  await page.waitForSelector(`${led("matrix.r1.c2")}[data-colour="green"]`, { timeout: 5_000 });
  await page.close();
});

test("Space and Enter on a control are handled, so Space does not scroll the page; other keys are left alone", async () => {
  const page = await open(installFakeMidi);
  const prevented = (k: string) =>
    page.$eval(key("matrix.r1.c2"), (el, k) => {
      const e = new KeyboardEvent("keydown", { key: k, bubbles: true, cancelable: true });
      el.dispatchEvent(e);
      return e.defaultPrevented;
    }, k);
  assert.equal(await prevented(" "), true);
  assert.equal(await prevented("Enter"), true);
  assert.equal(await prevented("Tab"), false, "Tab must still move focus");
  assert.equal(await prevented("a"), false);
  await page.close();
});

test("a mouse click presses the key once and not twice (its click event is not a second press)", async () => {
  const page = await open(installFakeMidi);
  await start(page);
  await page.click(face("matrix.r1.c2"));
  await page.waitForSelector(`${led("matrix.r1.c2")}[data-colour="green"]`, { timeout: 5_000 });
  await page.waitForTimeout(250);
  assert.equal((await ledState(page, "matrix.r1.c2")).colour, "green", "it stayed on: one press, not two");
  await page.close();
});

// ---------------------------------------------------------------------------------------------------------------
// E2
// ---------------------------------------------------------------------------------------------------------------

/** What the engine plays for step 1 of track 0, from the same module run in Node: the bytes the page must give the output. */
function expectedFirstNote(): { on: number[]; off: number[] } {
  const s = new Session({ seed: 1, supportsClear: false, mapJitterMs: 0, delayMs: 0 });
  s.click("matrix.r0.c1");
  s.send({ type: "transport", play: true });
  for (let i = 0; i < 400; i++) s.step();
  const sends = s.log.flatMap((l) => (l.what === "send" ? [l.bytes] : []));
  const on = sends.find((b) => (b[0]! & 0xf0) === 0x90);
  assert.ok(on, "the engine plays a note in Node");
  const off = sends.find((b) => (b[0]! & 0xf0) === 0x80 && b[0]! === ((on[0]! & 0x0f) | 0x80) && b[1] === on[1]);
  assert.ok(off, "and ends it");
  return { on, off };
}

test("E2: Play sends a Note On with the right bytes to the chosen output, then its Note Off, stamped ahead of now", async () => {
  const want = expectedFirstNote();
  const page = await open(installFakeMidi);
  await start(page);
  await page.selectOption("#midi-output", "fake-1");
  await page.click(face("matrix.r0.c1"));
  await page.click(face("transport.play"));
  await page.waitForFunction(() => window.__midi.sent.length >= 2, null, { timeout: 8_000 });
  const sent = await page.evaluate(() => window.__midi.sent.slice());
  const on = sent.find((s) => (s.data[0]! & 0xf0) === 0x90)!;
  assert.deepEqual(on.data, want.on, "the Note On the engine makes, byte for byte");
  const off = sent.find((s) => s.data[0] === want.off[0] && s.data[1] === want.off[1] && (s.data[0]! & 0xf0) === 0x80)!;
  assert.deepEqual(off.data.slice(0, 2), want.off.slice(0, 2));
  assert.ok(on.timestamp !== null && off.timestamp !== null, "both are stamped");
  assert.ok(off.timestamp! > on.timestamp!, "the Note Off is stamped after the Note On");
  const margins = sent.map((s) => (s.timestamp === null ? -1 : s.timestamp - s.at));
  assert.ok(margins.every((m) => m > 0), `every send is stamped ahead of the moment it was made: ${margins.map((m) => m.toFixed(1))}`);
  const median = [...margins].sort((a, b) => a - b)[Math.floor(margins.length / 2)]!;
  assert.ok(median >= 30, `the median stamp is ${median.toFixed(1)} ms ahead, the lookahead is 30 ms`);
  await page.close();
});

test("E2: nothing is sent until an output is chosen, and the strip says to choose one", async () => {
  const page = await open(installFakeMidi);
  await start(page);
  assert.match((await page.textContent("#midi-status")) ?? "", /Choose a MIDI output/);
  await page.click(face("matrix.r0.c1"));
  await page.click(face("transport.play"));
  await page.waitForTimeout(700);
  assert.equal(await page.evaluate(() => window.__midi.sent.length), 0);
  await page.selectOption("#midi-output", "fake-1");
  await page.waitForFunction(() => window.__midi.sent.length >= 2, null, { timeout: 8_000 });
  assert.match((await page.textContent("#midi-status")) ?? "", /Fake output/);
  await page.close();
});

test("E2: the lookahead field sets how far ahead of now the sends are stamped", async () => {
  const page = await open(installFakeMidi);
  await start(page);
  await page.selectOption("#midi-output", "fake-1");
  await page.fill("#lookahead", "90");
  await page.press("#lookahead", "Tab");
  await page.click(face("matrix.r0.c1"));
  await page.click(face("matrix.r0.c9"));
  await page.click(face("transport.play"));
  await page.waitForFunction(() => window.__midi.sent.length >= 4, null, { timeout: 8_000 });
  const sent = await page.evaluate(() => window.__midi.sent.slice());
  const margins = sent.map((s) => s.timestamp! - s.at).sort((a, b) => a - b);
  assert.ok(margins[Math.floor(margins.length / 2)]! >= 90, `median ${margins[Math.floor(margins.length / 2)]!.toFixed(1)} ms with a 90 ms lookahead`);
  await page.close();
});

test("the Stop key stops the transport and leaves no note on", async () => {
  const page = await open(installFakeMidi);
  await start(page);
  await page.selectOption("#midi-output", "fake-1");
  for (const c of [1, 5, 9, 13]) await page.click(face(`matrix.r0.c${c}`));
  await page.click(face("transport.play"));
  await page.waitForFunction(() => window.__midi.sent.length >= 4, null, { timeout: 8_000 });
  await page.click(face("transport.stop"));
  await page.waitForTimeout(500);
  const before = await page.evaluate(() => window.__midi.sent.length);
  await page.waitForTimeout(700);
  const { sent, count } = await page.evaluate(() => ({ sent: window.__midi.sent.slice(), count: window.__midi.sent.length }));
  assert.equal(count, before, "nothing is sent once the transport has stopped and the queue is out");
  const held = new Set<string>();
  for (const s of [...sent].sort((a, b) => (a.timestamp ?? a.at) - (b.timestamp ?? b.at))) {
    const status = s.data[0]! & 0xf0;
    const k = `${s.data[0]! & 0x0f}/${s.data[1]}`;
    if (status === 0x90 && s.data[2]! > 0) held.add(k);
    else if (status === 0x80) held.delete(k);
  }
  assert.deepEqual([...held], [], "every Note On has its Note Off");
  await page.close();
});

// ---------------------------------------------------------------------------------------------------------------
// E7
// ---------------------------------------------------------------------------------------------------------------

test("E7: without Web MIDI the app loads, runs, and says it has no MIDI", async () => {
  const page = await open(removeMidi);
  assert.equal(await page.evaluate(() => typeof navigator.requestMIDIAccess), "undefined");
  await start(page);
  assert.match((await page.textContent("#midi-status")) ?? "", /no Web MIDI/);
  assert.deepEqual(await page.$eval("#midi-output", (el) => [...(el as HTMLSelectElement).options].map((o) => o.textContent)), ["No output"], "only the no-output option");
  // It runs: a press lights a key, and Play moves the engine without an error.
  await page.click(face("matrix.r3.c4"));
  await page.waitForSelector(`${led("matrix.r3.c4")}[data-colour="green"]`, { timeout: 5_000 });
  await page.click(face("transport.play"));
  await page.waitForTimeout(300);
  assert.equal(await page.getAttribute("main", "data-engine"), "running");
  await page.close();
});

test("E7: a refusal of MIDI access is told apart from a browser without it, and the sequencer still runs", async () => {
  const page = await open(refuseMidi);
  await start(page);
  assert.match((await page.textContent("#midi-status")) ?? "", /refused/);
  await page.click(face("matrix.r3.c4"));
  await page.waitForSelector(`${led("matrix.r3.c4")}[data-colour="green"]`, { timeout: 5_000 });
  await page.close();
});

// ---------------------------------------------------------------------------------------------------------------
// E8
// ---------------------------------------------------------------------------------------------------------------

test("E8: every control the page draws is a button with the manual's own name, and nothing is drawn that the layout does not hold", async () => {
  const page = await open();
  const drawn = await page.$$eval("svg g[data-id]", (els) =>
    els.map((el) => ({ id: el.getAttribute("data-id"), role: el.getAttribute("role"), name: el.getAttribute("aria-label"), tab: el.getAttribute("tabindex") })),
  );
  assert.equal(drawn.length, geometry.placements.length);
  assert.equal(drawn.length, 171);
  for (const d of drawn) {
    const control = controls.controls.find((c) => c.id === d.id);
    assert.ok(control, `${d.id} is in controls.json`);
    assert.equal(d.name, control.name, `${d.id} is named as the manual names it`);
    assert.equal(d.role, "button");
    assert.equal(d.tab, "0", "reachable, in document order (no positive tabindex)");
  }
  assert.deepEqual(drawn.map((d) => d.id), tabOrder(geometry), "drawn in layout order");
  await page.close();
});

test("E8: the browser's own accessibility tree finds each control by its manual name", async () => {
  const page = await open();
  for (const p of geometry.placements) {
    const n = await page.getByRole("button", { name: p.name, exact: true }).count();
    assert.equal(n, 1, `${p.name}`);
  }
  await page.close();
});

test("E8: Tab walks the strip, then the matrix in reading order, then the other controls in the layout's order", async () => {
  const page = await open();
  const seen: string[] = [];
  const total = 3 + geometry.placements.length;
  for (let i = 0; i < total; i++) {
    await page.keyboard.press("Tab");
    seen.push(await page.evaluate(() => {
      const el = document.activeElement as HTMLElement | null;
      return el?.getAttribute("data-id") ?? el?.id ?? "";
    }));
  }
  assert.deepEqual(seen, ["start", "midi-output", "lookahead", ...tabOrder(geometry)]);
  await page.keyboard.press("Tab");
  const after = await page.evaluate(() => (document.activeElement as HTMLElement).tagName);
  assert.notEqual(after, "G", "the next Tab leaves the panel");
  await page.close();
});

test("E8: the focus ring is drawn from the focus tokens, and appears on keyboard focus and not on a pointer press", async () => {
  const page = await open();
  const ring = (id: string, which: "outer" | "inner") =>
    page.$eval(`${key(id)} .ring-${which}`, (el) => {
      const s = getComputedStyle(el);
      return { display: s.display, stroke: s.stroke, width: s.strokeWidth };
    });
  assert.equal((await ring("matrix.r9.c1", "outer")).display, "none");
  await page.keyboard.press("Tab"); // start
  await page.keyboard.press("Tab"); // output
  await page.keyboard.press("Tab"); // lookahead
  await page.keyboard.press("Tab"); // r9.c1
  const outer = await ring("matrix.r9.c1", "outer");
  const inner = await ring("matrix.r9.c1", "inner");
  assert.equal(outer.display, "inline");
  assert.equal(outer.stroke, rgb(tokens.colour["focus"].value));
  assert.equal(outer.width, `${tokens.stroke.focus.value}px`);
  assert.equal(inner.stroke, rgb(tokens.colour["focus.inner"].value));
  assert.equal(inner.width, `${tokens.stroke.focus_inner.value}px`);
  await page.click(face("matrix.r5.c5"));
  assert.equal((await ring("matrix.r5.c5", "outer")).display, "none", "a mouse press does not leave a ring");
  // The ring is two strokes either side of the key's edge: the light one just outside, the dark one just inside.
  const radii = await page.$eval(key("matrix.r9.c1"), (g) => {
    const r = (sel: string) => Number(g.querySelector(sel)!.getAttribute("r"));
    return { face: r(".face"), outer: r(".ring-outer"), inner: r(".ring-inner") };
  });
  assert.equal(radii.outer, radii.face + tokens.stroke.focus.value / 2, "the outer stroke sits just outside the key");
  assert.equal(radii.inner, radii.face - tokens.stroke.focus_inner.value / 2, "the inner stroke sits just inside its edge");
  await page.close();
});

test("E8: the strip's own controls are labelled and the strip does not sit on the panel", async () => {
  const page = await open();
  for (const [selector, name] of [["#start", "Start"], ["#midi-output", "MIDI output"], ["#lookahead", "Lookahead in milliseconds"]] as const) {
    assert.equal(await page.getByRole(selector === "#start" ? "button" : selector === "#midi-output" ? "combobox" : "spinbutton", { name, exact: true }).count(), 1, name);
  }
  const strip = await page.$eval("#strip", (el) => el.getBoundingClientRect().toJSON() as DOMRect);
  const panel = await page.$eval("svg#panel", (el) => el.getBoundingClientRect().toJSON() as DOMRect);
  assert.ok(strip.bottom <= panel.top + 0.5 || panel.bottom <= strip.top + 0.5, "the strip and the panel do not overlap");
  await page.close();
});

// ---------------------------------------------------------------------------------------------------------------
// The panel as an object
// ---------------------------------------------------------------------------------------------------------------

test("the panel scales as one object and never rearranges", async () => {
  const page = await open();
  const shape = async () =>
    page.evaluate(() => {
      const c = (id: string) => {
        const r = document.querySelector(`g[data-id="${id}"] .face`)!.getBoundingClientRect();
        return { x: r.x + r.width / 2, y: r.y + r.height / 2, w: r.width };
      };
      const a = c("matrix.r9.c1");
      const b = c("matrix.r0.c16");
      const p = c("transport.stop");
      return { across: (b.x - a.x) / a.w, down: (b.y - a.y) / a.w, stop: (p.y - a.y) / a.w, width: a.w };
    });
  const big = await shape();
  await page.setViewportSize({ width: 700, height: 900 });
  const small = await shape();
  assert.ok(small.width < big.width, "it got smaller");
  for (const k of ["across", "down", "stop"] as const) assert.ok(Math.abs(big[k] - small[k]) < 0.01, `${k} changed with the window: ${big[k]} then ${small[k]}`);
  await page.close();
});

test("a flashing LED is dark and lit at the token's period, and two flashing LEDs flash together", async () => {
  const page = await open(installFakeMidi);
  await start(page);
  await page.click(face("mode.play"));
  await page.waitForSelector(`${led("mode.play")}[data-phase="flash"]`, { timeout: 5_000 });
  const first = await ledState(page, "mode.play");
  assert.equal(first.colour, "orange");
  await page.waitForTimeout(190);
  await page.click(face("mode.edit"));
  await page.waitForSelector(`${led("mode.edit")}[data-phase="flash"]`, { timeout: 5_000 });
  const flashing = await page.$$eval('.led[data-phase="flash"]', (els) => els.length);
  assert.ok(flashing >= 2, `${flashing} LEDs flash`);
  const fills = await page.evaluate(
    async (period) => {
      const lit: string[] = [];
      const els = [...document.querySelectorAll<SVGElement>('.led[data-phase="flash"]')];
      const t0 = performance.now();
      const rows: { t: number; fills: string[] }[] = [];
      while (performance.now() - t0 < period * 2.2) {
        rows.push({ t: performance.now() - t0, fills: els.map((e) => getComputedStyle(e).fill) });
        await new Promise((r) => setTimeout(r, 15));
      }
      void lit;
      return rows;
    },
    tokens.flash.period_ms.value,
  );
  const together = fills.filter((r) => new Set(r.fills).size === 1).length;
  assert.ok(together >= fills.length * 0.95, `${together} of ${fills.length} samples had every flashing LED in the same state`);
  const seen = new Set(fills.flatMap((r) => r.fills));
  assert.ok(seen.has(rgb(tokens.colour["led.orange"].value)), "lit in the orange token");
  assert.ok(seen.has(rgb(tokens.colour["led.off"].value)), "dark in the off token");
  // The period: time between the starts of lit runs.
  const lit = rgb(tokens.colour["led.orange"].value);
  const rises: number[] = [];
  for (let i = 1; i < fills.length; i++) if (fills[i]!.fills[0] === lit && fills[i - 1]!.fills[0] !== lit) rises.push(fills[i]!.t);
  assert.ok(rises.length >= 2, "two rising edges were seen");
  const period = rises[1]! - rises[0]!;
  assert.ok(Math.abs(period - tokens.flash.period_ms.value) < 60, `the period is ${period.toFixed(0)} ms, the token says ${tokens.flash.period_ms.value}`);
  await page.close();
});

// The panel drawn like the Octopus, in Chromium (WENGE-0019, specs/SPEC-0002/p7-faithful-panel.md): every control in the inventory is a button by its
// manual name, the plate's own keys are drawn and are not buttons, an LED is a dot beside its key, a knob turns and the turn reaches the engine, and
// the whole plate is on screen at a size a hand can use.
// Needs `npm run build` first, which `npm run test:browser` does.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { after, before, test } from "node:test";
import type { Browser, Page } from "playwright-core";
import { launch } from "../../scripts/browser.ts";
import { serve, type Server } from "../../scripts/serve.ts";
import { parseLayout, place, tabOrder, type ControlsDoc } from "../../src/layout.ts";
import { parseTokens } from "../../src/tokens.ts";
import { repoRoot } from "../support/module.ts";

const read = (rel: string): unknown => JSON.parse(readFileSync(`${repoRoot}/${rel}`, "utf8"));
const controls = read("contracts/controls.json") as ControlsDoc;
const tokens = parseTokens(read("contracts/design.tokens.json"));
const geometry = place(parseLayout(read("apps/web/layout/panel.layout.json"), controls), controls, tokens);
const numberOf = (id: string): number => controls.controls.find((c) => c.id === id)!.n;

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

interface Sent {
  type: string;
  kind: number;
  control: number;
  detents: number;
}

declare global {
  interface Window {
    __sent: Sent[];
  }
}

/** Records what the page posts to the engine's worklet, so a test can say what a turn or a press sent. */
function recordInputs(): void {
  const sent: Sent[] = [];
  window.__sent = sent;
  const original = MessagePort.prototype.postMessage;
  MessagePort.prototype.postMessage = function (this: MessagePort, message: unknown, ...rest: unknown[]) {
    const m = message as Sent | null;
    if (m && m.type === "input") sent.push({ type: m.type, kind: m.kind, control: m.control, detents: m.detents });
    return (original as (...a: unknown[]) => void).call(this, message, ...rest);
  } as typeof MessagePort.prototype.postMessage;
}

async function open(width = 1280, height = 900): Promise<Page> {
  const page = await browser.newPage({ viewport: { width, height } });
  page.on("pageerror", (e) => assert.fail(`page error: ${e.message}`));
  await page.addInitScript(recordInputs);
  await page.goto(`${server.url}/pages/app.html?bare`);
  await page.waitForSelector('main[data-engine="idle"]');
  return page;
}

async function start(page: Page): Promise<void> {
  await page.click("#play");
  await page.waitForSelector('main[data-engine="running"]', { timeout: 15_000 });
}

const key = (id: string): string => `g[data-id="${id}"]`;
const KIND_TURN = 2;
const inputs = (page: Page) => page.evaluate(() => window.__sent);

test("P7-1: 242 controls are buttons found by the manual's own names, and none is missing", async () => {
  const page = await open();
  const drawn = await page.$$eval('svg g[data-id][role="button"]', (els) => els.map((el) => el.getAttribute("data-id")));
  assert.equal(drawn.length, 242);
  assert.deepEqual(drawn, tabOrder(geometry), "each once, in the plate's reading order");
  for (const id of ["mode.page", "edit.enc.vel", "chord.b3", "circle.scale_mod", "transport.rec", "mutator.mute", "main.enc", "mix.enc.r0", "matrix.r9.c16"]) {
    const name = controls.controls.find((c) => c.id === id)!.name;
    assert.equal(await page.getByRole("button", { name, exact: true }).count(), 1, name);
  }
  await page.close();
});

test("P7-2: the plate's own keys are drawn, and are not buttons, not tab stops and not in the accessibility tree", async () => {
  const page = await open();
  const inert = await page.$$eval("svg g.inert", (els) => els.map((el) => ({ id: el.getAttribute("data-id"), role: el.getAttribute("role"), tab: el.getAttribute("tabindex"), hidden: el.getAttribute("aria-hidden") })));
  assert.equal(inert.length, 37);
  for (const k of inert) {
    assert.equal(k.role, null, `${k.id}`);
    assert.equal(k.tab, null, `${k.id}`);
    assert.equal(k.hidden, "true", `${k.id}`);
  }
  assert.equal(await page.$$eval("svg g[data-id]", (els) => els.length), geometry.placements.length, "nothing is drawn that the layout does not hold");
  await page.close();
});

test("P7-3: a click on one of the plate's own keys sends nothing to the engine", async () => {
  const page = await open();
  await start(page);
  await page.click(`${key("hw.target.atr")} .face`);
  await page.click(`${key("hw.num.3")} .face`);
  await page.waitForTimeout(150);
  assert.deepEqual(await inputs(page), []);
  await page.close();
});

test("P7-4: the plate is a rectangle under the controls, and its printed legends are on it", async () => {
  const page = await open();
  const plate = await page.$eval("svg#panel rect.plate", (el) => ({ w: Number(el.getAttribute("width")), h: Number(el.getAttribute("height")), x: Number(el.getAttribute("x")), y: Number(el.getAttribute("y")) }));
  assert.deepEqual(plate, { x: 0, y: 0, w: geometry.width, h: geometry.height });
  const legends = await page.$$eval("svg#panel text.legend", (els) => els.map((el) => el.textContent));
  for (const t of ["MIX", "SEL", "MUT", "EDIT", "CHORD", "MODE", "SCALE", "TRANSPORT", "MIX TARGET", "\u{1d11e}", "\u{1d122}"]) assert.ok(legends.includes(t), t);
  assert.equal(legends.filter((t) => /^\d$/.test(t ?? "")).length, 10, "row numbers 0 to 9");
  await page.close();
});

test("P7-5: an LED is a dot beside its key, and lighting it changes the dot and not the key", async () => {
  const page = await open();
  await start(page);
  const shape = (id: string) =>
    page.$eval(key(id), (g) => {
      const f = g.querySelector(".face")!;
      const l = g.querySelector(".led")!;
      return {
        face: { cx: Number(f.getAttribute("cx")), cy: Number(f.getAttribute("cy")), r: Number(f.getAttribute("r")), fill: getComputedStyle(f).fill },
        led: { cx: Number(l.getAttribute("cx")), cy: Number(l.getAttribute("cy")), r: Number(l.getAttribute("r")), fill: getComputedStyle(l).fill },
      };
    });
  const before = await shape("matrix.r3.c5");
  assert.equal(before.face.r, tokens.space["4"].value / 2);
  assert.equal(before.led.r, tokens.space["2"].value / 2);
  assert.ok(before.led.cy < before.face.cy && before.led.cx === before.face.cx, "the dot is above the key");
  await page.click(`${key("matrix.r3.c5")} .face`);
  await page.waitForSelector(`${key("matrix.r3.c5")} .led[data-colour="green"]`, { timeout: 5_000 });
  const after = await shape("matrix.r3.c5");
  assert.notEqual(after.led.fill, before.led.fill, "the dot lit");
  assert.equal(after.face.fill, before.face.fill, "the key did not");
  await page.close();
});

test("P7-6: a knob turns: the arrow keys and the wheel send detents to the engine, and a press sends nothing", async () => {
  const page = await open();
  await start(page);
  const n = numberOf("mix.enc.r4");
  await page.focus(key("mix.enc.r4"));
  await page.keyboard.press("ArrowUp");
  await page.keyboard.press("ArrowRight");
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("ArrowLeft");
  const box = await page.$eval(`${key("mix.enc.r4")} .nut`, (el) => el.getBoundingClientRect().toJSON() as DOMRect);
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.wheel(0, -100);
  await page.mouse.wheel(0, 100);
  await page.keyboard.press("Space");
  await page.keyboard.press("Enter");
  await page.waitForTimeout(150);
  const sent = await inputs(page);
  assert.deepEqual(sent, [1, 1, -1, -1, 1, -1].map((detents) => ({ type: "input", kind: KIND_TURN, control: n, detents })), "turns only: up and right are clockwise, down and left the other way, the wheel up is clockwise");
  await page.close();
});

test("P7-7: turning a knob before Play says what to do, as a key does, and sends nothing", async () => {
  const page = await open();
  await page.focus(key("edit.enc.vel"));
  await page.keyboard.press("ArrowUp");
  await page.waitForTimeout(100);
  assert.match((await page.textContent("#engine-status")) ?? "", /Press Play/);
  assert.deepEqual(await inputs(page), []);
  await page.close();
});

test("P7-8: the knobs are hex nuts round a ball, and the wheel over one does not scroll the page", async () => {
  const page = await open();
  const k = await page.$eval(key("edit.enc.vel"), (g) => ({ nut: g.querySelectorAll("polygon.nut").length, ball: g.querySelectorAll("circle.ball").length, roledescription: g.getAttribute("aria-roledescription") }));
  assert.deepEqual(k, { nut: 1, ball: 1, roledescription: "knob" });
  const prevented = await page.$eval(key("edit.enc.vel"), (el) => {
    const e = new WheelEvent("wheel", { deltaY: -100, bubbles: true, cancelable: true });
    el.dispatchEvent(e);
    return e.defaultPrevented;
  });
  assert.equal(prevented, true);
  await page.close();
});

test("P7-9: the whole plate is on screen at 1280 by 900, a key is at least 12 px across, and a printed label is at least 9 px high", async () => {
  const page = await open();
  const size = await page.evaluate(() => {
    const svg = document.querySelector("svg#panel")!;
    const r = svg.getBoundingClientRect();
    const face = document.querySelector('g[data-id="matrix.r4.c7"] .face')!.getBoundingClientRect();
    const plate = document.querySelector("svg#panel rect.plate")!.getBoundingClientRect();
    const label = document.querySelector('g[data-id="mode.page"] text.label')!;
    const scale = plate.width / Number(document.querySelector("svg#panel rect.plate")!.getAttribute("width"));
    return { w: r.width, h: r.height, plateRight: plate.right, plateBottom: plate.bottom, plateLeft: plate.left, plateTop: plate.top, key: face.width, font: parseFloat(getComputedStyle(label).fontSize) * scale, vw: innerWidth, vh: innerHeight };
  });
  assert.ok(size.plateLeft >= 0 && size.plateTop >= 0 && size.plateRight <= size.vw && size.plateBottom <= size.vh, "the plate is inside the window");
  assert.ok(size.key >= 12, `a key is ${size.key.toFixed(1)} px across`);
  assert.ok(size.font >= 9, `a label is ${size.font.toFixed(1)} px high`);
  await page.close();
});

test("P7-10: the panel scales as one object: the circle's keys keep their places against the matrix when the window narrows", async () => {
  const page = await open();
  const shape = () =>
    page.evaluate(() => {
      const c = (id: string) => {
        const r = document.querySelector(`g[data-id="${id}"] .face, g[data-id="${id}"] .nut`)!.getBoundingClientRect();
        return { x: r.x + r.width / 2, y: r.y + r.height / 2, w: r.width };
      };
      const a = c("matrix.r9.c1");
      return { chord: (c("chord.b7").x - a.x) / a.w, main: (c("main.enc").y - a.y) / a.w, rec: (c("transport.rec").y - a.y) / a.w, width: a.w };
    });
  const big = await shape();
  await page.setViewportSize({ width: 900, height: 700 });
  const small = await shape();
  assert.ok(small.width < big.width);
  for (const k of ["chord", "main", "rec"] as const) assert.ok(Math.abs(big[k] - small[k]) < 0.01, `${k}: ${big[k]} then ${small[k]}`);
  await page.close();
});

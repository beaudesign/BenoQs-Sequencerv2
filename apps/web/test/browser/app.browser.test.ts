// The panel in Chromium: the real page, the real worklet, the real scheduler, and fake `MIDIOutput`s and a fake `MIDIInput` that record
// what they are given and hand over what the test says (Web MIDI itself cannot start in this headless Chromium: it is refused without a
// permission and fails with InvalidStateError with one, because the container has no MIDI backend; see specs/SPEC-0002/p4b-findings.md).
// Observables E1, E2, E7 and E8 of specs/SPEC-0002/p3-plan.md and M1, M6, M7 and M8 of specs/SPEC-0002/p4-plan.md.
// Needs `npm run build` first, which `npm run test:browser` does.
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

interface FakeIn {
  data: Uint8Array | null;
  timeStamp: number;
}

declare global {
  interface Window {
    __ports: {
      sent: { 1: FakeSend[]; 2: FakeSend[] };
      inputOpen(): boolean;
      emit(data: number[] | null, timeStamp: number): void;
      /** A device is plugged in or pulled out, and the browser says its ports changed. */
      plugIn(id: string, name: string): void;
      unplug(id: string): void;
    };
  }
}

/** Two outputs and one input, as a rig with a synth and a drum machine would show. */
function installTwoPorts(): void {
  const sent: { 1: FakeSend[]; 2: FakeSend[] } = { 1: [], 2: [] };
  const output = (id: string, name: string, log: FakeSend[]) => ({
    id,
    name,
    send(data: ArrayLike<number>, timestamp?: number) {
      log.push({ data: Array.from(data), timestamp: timestamp ?? null, at: performance.now() });
    },
  });
  const input = {
    id: "fake-in",
    name: "Fake input",
    onmidimessage: null as ((e: FakeIn) => unknown) | null,
  };
  const o1 = output("out-1", "Fake synth", sent[1]);
  const o2 = output("out-2", "Fake drums", sent[2]);
  const access = {
    outputs: new Map([[o1.id, o1], [o2.id, o2]]),
    inputs: new Map<string, { id: string; name: string; onmidimessage: ((e: FakeIn) => unknown) | null }>([[input.id, input]]),
    onstatechange: null as ((e: unknown) => unknown) | null,
  };
  window.__ports = {
    sent,
    inputOpen: () => input.onmidimessage !== null,
    emit: (data, timeStamp) => void input.onmidimessage?.({ data: data ? Uint8Array.from(data) : null, timeStamp }),
    plugIn(id, name) {
      access.inputs.set(id, { id, name, onmidimessage: null });
      access.onstatechange?.({});
    },
    unplug(id) {
      access.inputs.delete(id);
      access.onstatechange?.({});
    },
  };
  Object.defineProperty(Navigator.prototype, "requestMIDIAccess", { configurable: true, value: () => Promise.resolve(access) });
}

function failMidi(): void {
  Object.defineProperty(Navigator.prototype, "requestMIDIAccess", {
    configurable: true,
    value: () => Promise.reject(new DOMException("Platform dependent initialization failed.", "InvalidStateError")),
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

async function open(init?: () => void, query = ""): Promise<Page> {
  const page = await browser.newPage({ viewport: { width: 1280, height: 900 } });
  page.on("pageerror", (e) => assert.fail(`page error: ${e.message}`));
  if (init) await page.addInitScript(init);
  await page.goto(`${server.url}/pages/app.html${query}`);
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
  await page.selectOption("#midi-out-1", "fake-1");
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
  await page.selectOption("#midi-out-1", "fake-1");
  await page.waitForFunction(() => window.__midi.sent.length >= 2, null, { timeout: 8_000 });
  assert.match((await page.textContent("#midi-status")) ?? "", /Fake output/);
  await page.close();
});

test("E2: the lookahead field sets how far ahead of now the sends are stamped", async () => {
  const page = await open(installFakeMidi);
  await start(page);
  await page.selectOption("#midi-out-1", "fake-1");
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
  await page.selectOption("#midi-out-1", "fake-1");
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
  for (const [selector, none] of [["#midi-out-1", "No output"], ["#midi-out-2", "No output"], ["#midi-in", "No input"]] as const) {
    assert.deepEqual(await page.$eval(selector, (el) => [...(el as HTMLSelectElement).options].map((o) => o.textContent)), [none], `${selector}: only the empty option`);
  }
  assert.equal(await page.textContent("#midi-in-status"), "", "one sentence, in the MIDI line, covers both directions");
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
  const total = 5 + geometry.placements.length;
  for (let i = 0; i < total; i++) {
    await page.keyboard.press("Tab");
    seen.push(await page.evaluate(() => {
      const el = document.activeElement as HTMLElement | null;
      return el?.getAttribute("data-id") ?? el?.id ?? "";
    }));
  }
  assert.deepEqual(seen, ["start", "midi-out-1", "midi-out-2", "midi-in", "lookahead", ...tabOrder(geometry)]);
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
  for (const _stop of ["start", "out 1", "out 2", "in", "lookahead"]) await page.keyboard.press("Tab");
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
  for (const [role, name] of [["button", "Start"], ["combobox", "MIDI Out 1"], ["combobox", "MIDI Out 2"], ["combobox", "MIDI In"], ["spinbutton", "Lookahead in milliseconds"]] as const) {
    assert.equal(await page.getByRole(role, { name, exact: true }).count(), 1, name);
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

// ---------------------------------------------------------------------------------------------------------------
// P4b: two ports, the input path, the settings (plan observables M1, M6, M7, M8)
// ---------------------------------------------------------------------------------------------------------------

const noteOns = (sends: FakeSend[]): FakeSend[] => sends.filter((s) => (s.data[0]! & 0xf0) === 0x90);
const channelsOf = (sends: FakeSend[]): number[] => [...new Set(sends.map((s) => s.data[0]! & 0x0f))].sort((a, b) => a - b);

test("M1: with a device chosen for each port, each port's notes go to its own device and to no other", async () => {
  // Track 0 is routed to channel 17 (port 2, channel 1) and track 1 to channel 3 (port 1, channel 3).
  const page = await open(installTwoPorts, "?route=0:17,1:3");
  await start(page);
  await page.selectOption("#midi-out-1", "out-1");
  await page.selectOption("#midi-out-2", "out-2");
  await page.click(face("matrix.r0.c1"));
  await page.click(face("matrix.r1.c1"));
  await page.click(face("transport.play"));
  await page.waitForFunction(() => window.__ports.sent[1].length >= 2 && window.__ports.sent[2].length >= 2, null, { timeout: 8_000 });
  const sent = await page.evaluate(() => ({ 1: window.__ports.sent[1].slice(), 2: window.__ports.sent[2].slice() }));
  assert.ok(noteOns(sent[1]).length >= 1 && noteOns(sent[2]).length >= 1);
  assert.deepEqual(channelsOf(sent[1]), [2], "the synth hears channel 3 (nibble 2) and nothing else");
  assert.deepEqual(channelsOf(sent[2]), [0], "the drums hear channel 1 (nibble 0) and nothing else");
  assert.match((await page.textContent("#midi-status")) ?? "", /Port 1 sends to Fake synth\. Port 2 sends to Fake drums\./);
  await page.close();
});

test("M1: a port with no device chosen sends nothing, and the other port still plays", async () => {
  const page = await open(installTwoPorts, "?route=0:17,1:3");
  await start(page);
  await page.selectOption("#midi-out-1", "out-1");
  await page.click(face("matrix.r0.c1"));
  await page.click(face("matrix.r1.c1"));
  await page.click(face("transport.play"));
  await page.waitForFunction(() => window.__ports.sent[1].length >= 2, null, { timeout: 8_000 });
  await page.waitForTimeout(400);
  assert.equal(await page.evaluate(() => window.__ports.sent[2].length), 0, "port 2 has no device, so its notes have nowhere to go");
  assert.match((await page.textContent("#midi-status")) ?? "", /^Port 1 sends to Fake synth\.$/);
  await page.close();
});

test("M1: one device chosen for both ports hears both, and the sentence says it is one device", async () => {
  const page = await open(installTwoPorts, "?route=0:17,1:3");
  await start(page);
  await page.selectOption("#midi-out-1", "out-1");
  await page.selectOption("#midi-out-2", "out-1");
  await page.click(face("matrix.r0.c1"));
  await page.click(face("matrix.r1.c1"));
  await page.click(face("transport.play"));
  await page.waitForFunction(() => window.__ports.sent[1].length >= 4, null, { timeout: 8_000 });
  const sent = await page.evaluate(() => window.__ports.sent[1].slice());
  assert.deepEqual(channelsOf(sent), [0, 2], "channels 1 (port 2) and 3 (port 1) both reach the one device");
  assert.equal(await page.evaluate(() => window.__ports.sent[2].length), 0);
  assert.match((await page.textContent("#midi-status")) ?? "", /^Port 1 and port 2 send to Fake synth\.$/);
  await page.close();
});

test("M1: with no route, every track plays on port 1, and a device chosen for port 2 hears nothing", async () => {
  const page = await open(installTwoPorts);
  await start(page);
  await page.selectOption("#midi-out-1", "out-1");
  await page.selectOption("#midi-out-2", "out-2");
  await page.click(face("matrix.r0.c1"));
  await page.click(face("transport.play"));
  await page.waitForFunction(() => window.__ports.sent[1].length >= 2, null, { timeout: 8_000 });
  assert.equal(await page.evaluate(() => window.__ports.sent[2].length), 0);
  assert.equal(await page.isHidden("#route-status"), true, "no route in the address, nothing to say about one");
  await page.close();
});

test("the route stand-in says what it applied, and what it left out and why", async () => {
  const page = await open(installTwoPorts, "?route=3:17,bad,4:99");
  await start(page);
  const text = (await page.textContent("#route-status")) ?? "";
  assert.equal(await page.isVisible("#route-status"), true);
  assert.match(text, /Track 3 sends on port 2, channel 1\./);
  assert.match(text, /"bad"/);
  assert.match(text, /"4:99"/);
  await page.close();
});

test("M7: each port's output and the input are set from the keyboard, each with the manual's own name", async () => {
  const page = await open(installTwoPorts);
  await start(page);
  await page.focus("#midi-out-2");
  await page.keyboard.press("ArrowDown");
  assert.equal(await page.inputValue("#midi-out-2"), "out-1", "ArrowDown chose the first device");
  assert.match((await page.textContent("#midi-status")) ?? "", /^Port 2 sends to Fake synth\.$/);
  await page.focus("#midi-in");
  await page.keyboard.press("ArrowDown");
  assert.equal(await page.inputValue("#midi-in"), "fake-in");
  assert.equal(await page.evaluate(() => window.__ports.inputOpen()), true);
  assert.match((await page.textContent("#midi-in-status")) ?? "", /Listening to Fake input/);
  await page.close();
});

test("M6: a chosen input's messages are counted on the strip, and nothing else happens: no sound, no transport", async () => {
  const page = await open(installTwoPorts);
  await start(page);
  assert.deepEqual(await page.$eval("#midi-in", (el) => [...(el as HTMLSelectElement).options].map((o) => o.textContent)), ["No input", "Fake input"]);
  assert.match((await page.textContent("#midi-in-status")) ?? "", /Choose a MIDI input/);
  assert.equal(await page.evaluate(() => window.__ports.inputOpen()), false, "a port is not listened to until it is chosen");
  await page.selectOption("#midi-out-1", "out-1");
  await page.selectOption("#midi-in", "fake-in");
  await page.evaluate(() => {
    for (const [data, t] of [[[0x90, 60, 100], 10], [[0xf8], 20], [[0xfa], 21], [[0xb0, 1, 5], 30]] as const) window.__ports.emit([...data], t);
  });
  await page.waitForFunction(() => /4 messages/.test(document.querySelector("#midi-in-count")!.textContent ?? ""), null, { timeout: 5_000 });
  assert.match((await page.textContent("#midi-in-status")) ?? "", /Nothing uses its messages yet/);
  await page.waitForTimeout(300);
  assert.equal(await page.getAttribute("main", "data-transport"), "stopped", "a Start message from the input does not start the transport yet");
  assert.deepEqual(await page.evaluate(() => [window.__ports.sent[1].length, window.__ports.sent[2].length]), [0, 0], "and nothing is sent");
  await page.selectOption("#midi-in", "");
  assert.equal(await page.evaluate(() => window.__ports.inputOpen()), false, "choosing no input lets go of the port");
  assert.match((await page.textContent("#midi-in-status")) ?? "", /Choose a MIDI input/);
  await page.close();
});

test("M6: a message with no data and one that is not MIDI are counted apart and do not break the page", async () => {
  const page = await open(installTwoPorts);
  await start(page);
  await page.selectOption("#midi-in", "fake-in");
  await page.evaluate(() => {
    window.__ports.emit(null, 1);
    window.__ports.emit([0x90, 60], 2);
    window.__ports.emit([0xfe], 3);
    window.__ports.emit([0x90, 60, 100], 4);
  });
  await page.waitForFunction(() => /4 messages/.test(document.querySelector("#midi-in-count")!.textContent ?? ""), null, { timeout: 5_000 });
  const count = (await page.textContent("#midi-in-count")) ?? "";
  assert.match(count, /1 used/);
  assert.match(count, /1 ignored/);
  assert.match(count, /2 malformed/);
  await page.close();
});

test("M8: with the browser's own refusal (Chromium 141, no permission) the app says so, offers no port, and still plays", async () => {
  const page = await open(); // no stand-in: the real requestMIDIAccess, which this headless Chromium refuses with NotAllowedError
  await start(page);
  assert.match((await page.textContent("#midi-status")) ?? "", /refused/);
  for (const selector of ["#midi-out-1", "#midi-out-2", "#midi-in"]) assert.equal(await page.$$eval(`${selector} option`, (o) => o.length), 1, selector);
  await page.click(face("matrix.r3.c4"));
  await page.waitForSelector(`${led("matrix.r3.c4")}[data-colour="green"]`, { timeout: 5_000 });
  await page.close();
});

test("M8: a browser that has Web MIDI and cannot start it says that, and not that it was refused", async () => {
  const page = await open(failMidi);
  await start(page);
  const text = (await page.textContent("#midi-status")) ?? "";
  assert.match(text, /could not start/);
  assert.match(text, /Platform dependent initialization failed/);
  assert.doesNotMatch(text, /refused/);
  await page.click(face("matrix.r3.c4"));
  await page.waitForSelector(`${led("matrix.r3.c4")}[data-colour="green"]`, { timeout: 5_000 });
  await page.close();
});

test("M7: a device plugged in later appears in the input list, and the chosen one pulled out lets go and says to choose again", async () => {
  const page = await open(installTwoPorts);
  await start(page);
  await page.evaluate(() => window.__ports.plugIn("late-in", "Late input"));
  assert.deepEqual(await page.$eval("#midi-in", (el) => [...(el as HTMLSelectElement).options].map((o) => o.textContent)), ["No input", "Fake input", "Late input"]);
  await page.selectOption("#midi-in", "late-in");
  assert.match((await page.textContent("#midi-in-status")) ?? "", /Listening to Late input/);
  await page.evaluate(() => window.__ports.unplug("late-in"));
  assert.equal(await page.inputValue("#midi-in"), "", "the choice went with the device");
  assert.match((await page.textContent("#midi-in-status")) ?? "", /Choose a MIDI input/);
  await page.waitForFunction(() => document.querySelector("#midi-in-count")!.textContent === "", null, { timeout: 5_000 });
  await page.close();
});

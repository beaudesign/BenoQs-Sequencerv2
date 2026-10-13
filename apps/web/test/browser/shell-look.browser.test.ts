// The strip and the page in the midnight look, in Chromium (WENGE-0024, ADR-0011): a pure-grey ground, a hairline under the strip, controls drawn as a border
// with one corner, Play the one filled control, Inter Variable actually loaded, the case hugging the plate with the ground showing round it, and three
// hairline line drawings in the ground above the case. What is measured is what the browser computed and rendered, not what the stylesheet says: a rule
// that never applied would pass a scan. Needs `npm run build` first, which `npm run test:browser` does.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { after, before, test } from "node:test";
import type { Browser, Page } from "playwright-core";
import { launch } from "../../scripts/browser.ts";
import { serve, type Server } from "../../scripts/serve.ts";
import { parseTokens, type ColourKey } from "../../src/tokens.ts";
import { repoRoot } from "../support/module.ts";

const tokens = parseTokens(JSON.parse(readFileSync(`${repoRoot}/contracts/design.tokens.json`, "utf8")));

type Rgb = readonly [number, number, number];
const hex = (role: ColourKey): Rgb => {
  const v = tokens.colour[role].value;
  return [parseInt(v.slice(1, 3), 16), parseInt(v.slice(3, 5), 16), parseInt(v.slice(5, 7), 16)];
};
const parse = (css: string): Rgb => {
  const m = /^rgba?\((\d+), (\d+), (\d+)(?:, 1)?\)$/.exec(css);
  assert.ok(m, `not an opaque colour: ${css}`);
  return [Number(m[1]), Number(m[2]), Number(m[3])];
};
const luminance = ([r, g, b]: Rgb): number => {
  const lin = (c: number): number => (c / 255 <= 0.04045 ? c / 255 / 12.92 : ((c / 255 + 0.055) / 1.055) ** 2.4);
  return 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
};
const contrast = (a: Rgb, b: Rgb): number => {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi! + 0.05) / (lo! + 0.05);
};
const show = (c: Rgb): string => `rgb(${c.join(", ")})`;
const eq = (got: string, want: Rgb, what: string): void => assert.deepEqual(parse(got), [...want], `${what}: ${got}, wanted ${show(want)}`);

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

async function open(width = 1280, height = 900): Promise<Page> {
  const page = await browser.newPage({ viewport: { width, height } });
  page.on("pageerror", (e) => assert.fail(`page error: ${e.message}`));
  await page.goto(`${server.url}/pages/app.html?bare`);
  await page.waitForSelector('main[data-engine="idle"]');
  return page;
}

const style = (page: Page, selector: string, prop: string): Promise<string> => page.$eval(selector, (el, p) => getComputedStyle(el).getPropertyValue(p), prop);
const rectOf = (page: Page, selector: string): Promise<{ left: number; top: number; right: number; bottom: number; width: number; height: number }> =>
  page.$eval(selector, (el) => {
    const r = el.getBoundingClientRect();
    return { left: r.left, top: r.top, right: r.right, bottom: r.bottom, width: r.width, height: r.height };
  });

/** Reads pixels out of a PNG with the browser's own decoder, on a blank page, so no pixel is guessed from a stylesheet. */
async function pixels(png: Buffer, points: readonly (readonly [number, number])[]): Promise<Rgb[]> {
  const blank = await browser.newPage();
  try {
    const read = (await blank.evaluate(
      async ({ b64, pts }) => {
        const img = new Image();
        img.src = `data:image/png;base64,${b64}`;
        await img.decode();
        const canvas = document.createElement("canvas");
        canvas.width = img.width;
        canvas.height = img.height;
        const ctx = canvas.getContext("2d", { willReadFrequently: true })!;
        ctx.drawImage(img, 0, 0);
        return pts.map(([x, y]) => [...ctx.getImageData(x, y, 1, 1).data.slice(0, 3)]);
      },
      { b64: png.toString("base64"), pts: points.map(([x, y]) => [Math.round(x), Math.round(y)] as const) },
    )) as number[][];
    return read.map(([r, g, b]) => [r!, g!, b!] as const);
  } finally {
    await blank.close();
  }
}

const CONTROLS = "#strip button, #strip select, #strip input";

test("P9-1: the page is the pure-grey ground, and a hairline of the edge tone is all that sets the strip off", async () => {
  const page = await open();
  eq(await style(page, "body", "background-color"), hex("surface"), "the page");
  eq(await style(page, "html", "background-color"), hex("surface"), "the root, so no other colour shows past the page");
  eq(await style(page, "#strip", "border-bottom-color"), hex("edge"), "the rule under the strip");
  assert.equal(await style(page, "#strip", "border-bottom-width"), `${tokens.stroke.hairline.value}px`, "a hairline");
  assert.equal(await style(page, "#strip", "border-bottom-style"), "solid");
  assert.equal(await style(page, "#strip", "background-color"), "rgba(0, 0, 0, 0)", "the strip has no ground of its own: it is the page");
  await page.close();
});

test("P9-2: a control of the strip is a border on the ground with one corner; Play is the one filled control; a toggle that is on takes the raised tone", async () => {
  const page = await open();
  const controls = await page.$$eval(CONTROLS, (els) =>
    els.map((el) => {
      const s = getComputedStyle(el);
      return { id: el.id, tag: el.tagName, bg: s.backgroundColor, border: s.borderTopColor, width: s.borderTopWidth, radius: s.borderTopLeftRadius, colour: s.color, pressed: el.getAttribute("aria-pressed") };
    }),
  );
  assert.ok(controls.length >= 9, `${controls.length} controls`);
  for (const c of controls) {
    assert.equal(c.radius, `${tokens.radius.control.value}px`, `${c.id}: the one corner`);
    assert.equal(c.width, `${tokens.stroke.hairline.value}px`, `${c.id}: a hairline border`);
  }
  const by = (id: string) => controls.find((c) => c.id === id)!;
  // Play: ink on the ground, the one filled control.
  eq(by("play").bg, hex("ink"), "Play's fill");
  eq(by("play").colour, hex("surface"), "Play's text");
  const filled = controls.filter((c) => c.bg !== "rgba(0, 0, 0, 0)" && c.pressed !== "true");
  assert.deepEqual(filled.map((c) => c.id), ["play"], "Play is the only control filled at rest");
  // A toggle that is off (both start off in a bare page), then one that is on (turned on with a click, as a person would).
  assert.equal(by("monitor").pressed, "false");
  assert.equal(by("monitor").bg, "rgba(0, 0, 0, 0)", "a toggle that is off has no fill");
  eq(by("monitor").border, hex("edge.control"), "a toggle that is off: the control border");
  await page.click("#monitor");
  await page.mouse.move(0, 0);
  assert.equal(await page.getAttribute("#monitor", "aria-pressed"), "true");
  eq(await style(page, "#monitor", "background-color"), hex("surface.raised"), "a toggle that is on: the raised tone");
  eq(await style(page, "#monitor", "border-top-color"), hex("ink"), "a toggle that is on: its border takes the ink");
  assert.ok(contrast(hex("ink"), parse(await style(page, "#monitor", "background-color"))) >= 7, "and the ink on the raised tone is read at 7 to 1");
  // The rest are a border on the ground.
  for (const id of ["tempo", "midi-out-1", "midi-out-2", "midi-in", "clock-state", "clock-offset", "lookahead"]) {
    eq(by(id).border, hex("edge.control"), `${id}: the control border`);
    assert.equal(by(id).bg, "rgba(0, 0, 0, 0)", `${id}: no fill`);
  }
  await page.close();
});

test("P9-3: a control's boundary is 3 to 1 on the ground and the strip's text is 7 to 1 on what it sits on, measured on the colours the browser computed", async () => {
  const page = await open();
  const ground = parse(await style(page, "body", "background-color"));
  const borders = await page.$$eval(`${CONTROLS}`, (els) => els.map((el) => ({ id: el.id, border: getComputedStyle(el).borderTopColor, pressed: el.getAttribute("aria-pressed") })));
  for (const b of borders) {
    if (b.id === "play") continue; // its boundary is its fill, checked below
    assert.ok(contrast(parse(b.border), ground) >= 3, `${b.id}: border ${b.border} is ${contrast(parse(b.border), ground).toFixed(2)}:1 on the ground`);
  }
  const text = await page.$$eval("#strip label, #strip p, #strip button, #strip select, #strip input", (els) =>
    els
      .filter((el) => (el.textContent ?? "").trim().length > 0 || el instanceof HTMLInputElement)
      .map((el) => {
        const s = getComputedStyle(el);
        // The tone under the text: the element's own fill if it has one, otherwise the page's.
        const own = s.backgroundColor;
        return { id: el.id || el.tagName, colour: s.color, under: own === "rgba(0, 0, 0, 0)" ? getComputedStyle(document.body).backgroundColor : own };
      }),
  );
  assert.ok(text.length > 12, `${text.length} text elements`);
  for (const t of text) assert.ok(contrast(parse(t.colour), parse(t.under)) >= 7, `${t.id}: ${t.colour} on ${t.under} is ${contrast(parse(t.colour), parse(t.under)).toFixed(2)}:1`);
  await page.close();
});

test("P9-4: the face is Inter Variable, loaded from the file the tokens name, and the page sets it in the two weights and nowhere bold", async () => {
  const page = await open();
  await page.evaluate(() => document.fonts.ready);
  const face = tokens.type.face;
  assert.ok(await page.evaluate((name) => document.fonts.check(`14px "${name}"`), face.name), "the face is loaded and usable");
  const loaded = await page.evaluate((name) => [...document.fonts].filter((f) => f.family.replaceAll('"', "") === name).map((f) => f.status), face.name);
  assert.deepEqual(loaded, ["loaded"], "one @font-face, and its file arrived");
  assert.ok((await style(page, "body", "font-family")).startsWith(`"${face.name}"`), "the page's family leads with the face");
  assert.equal(await style(page, "body", "font-feature-settings"), face.features.map((f) => `"${f}"`).join(", "), "its OpenType features are on");
  assert.equal(await style(page, "body", "font-size"), `${tokens.type.size.body.value}px`);
  const weights = await page.$$eval("#strip *", (els) => [...new Set(els.map((el) => getComputedStyle(el).fontWeight))]);
  for (const w of weights) assert.ok([tokens.type.weight.regular.value, tokens.type.weight.strong.value].includes(Number(w)), `font-weight ${w} is not one of the two`);
  assert.equal(await style(page, "#play", "font-weight"), String(tokens.type.weight.strong.value), "Play is set in the strong weight");
  await page.close();
});

test("P9-5: nothing in the strip or the page has a shadow, a gradient, a transition, tracking or capitals", async () => {
  const page = await open();
  const bad = await page.$$eval("html, body, main, #strip, #strip *, #stage, #stage *:not(#panel *)", (els) => {
    const out: string[] = [];
    for (const el of els) {
      const s = getComputedStyle(el);
      const name = el.id || el.tagName.toLowerCase() + (el.getAttribute("class") ? `.${el.getAttribute("class")}` : "");
      if (s.boxShadow !== "none") out.push(`${name}: box-shadow ${s.boxShadow}`);
      if (s.textShadow !== "none") out.push(`${name}: text-shadow`);
      if (s.backgroundImage !== "none") out.push(`${name}: background-image ${s.backgroundImage}`);
      if (s.transitionDuration.split(", ").some((d) => d !== "0s")) out.push(`${name}: transition ${s.transitionDuration}`);
      if (s.letterSpacing !== "normal") out.push(`${name}: letter-spacing ${s.letterSpacing}`);
      if (s.textTransform !== "none") out.push(`${name}: text-transform ${s.textTransform}`);
      if (s.filter !== "none") out.push(`${name}: filter ${s.filter}`);
    }
    return out;
  });
  assert.deepEqual(bad, []);
  await page.close();
});

test("P9-6: keyboard focus on a control of the strip is the two-stroke ring, 3 to 1 on the ground", async () => {
  const page = await open();
  await page.focus("#tempo");
  await page.keyboard.press("Shift+Tab");
  await page.keyboard.press("Tab");
  const ring = await page.$eval("#tempo", (el) => {
    const s = getComputedStyle(el);
    return { colour: s.outlineColor, width: s.outlineWidth, style: s.outlineStyle };
  });
  eq(ring.colour, hex("focus"), "the ring");
  assert.equal(ring.width, `${tokens.stroke.focus.value}px`);
  assert.notEqual(ring.style, "none");
  assert.ok(contrast(parse(ring.colour), parse(await style(page, "body", "background-color"))) >= 3);
  await page.close();
});

for (const [width, height] of [
  [1280, 900],
  [1920, 1080],
  [1024, 768],
  [600, 800],
  [1600, 480],
] as const) {
  test(`P9-7: at ${width} by ${height} the case is the plate and its frame and no more, the page ground shows round it, and the page does not scroll`, async () => {
    const page = await open(width, height);
    const strip = await rectOf(page, "#strip");
    const stage = await rectOf(page, "#stage");
    const panel = await rectOf(page, "#panel");
    const frame = await rectOf(page, "#panel .frame");
    const plate = await rectOf(page, "#panel rect.plate");
    // The case is the SVG: the frame rect fills it, so the case hugs the plate with the margin the layout gives.
    assert.ok(Math.abs(frame.width - panel.width) < 1 && Math.abs(frame.height - panel.height) < 1, `the frame is the whole case: ${frame.width}x${frame.height} in ${panel.width}x${panel.height}`);
    assert.ok(plate.left > frame.left && plate.top > frame.top && plate.right < frame.right && plate.bottom < frame.bottom, "the plate does not touch the edge of the case");
    eq(await style(page, "#panel .frame", "fill"), hex("frame"), "the frame");
    assert.equal(await style(page, "#panel-frame", "background-color"), "rgba(0, 0, 0, 0)", "the element that holds the case has no colour: the page shows round the case");
    // The case is inside the stage, under the strip, clear of the edges of the window.
    assert.ok(panel.left >= stage.left - 1 && panel.right <= stage.right + 1 && panel.top >= strip.bottom - 1 && panel.bottom <= stage.bottom + 1, `the case is inside the stage: ${JSON.stringify({ panel, stage })}`);
    // It scales as one object: the aspect of the frame is the aspect of the view box, to a pixel.
    const [, , vw, vh] = await page.$eval("#panel", (el) => (el.getAttribute("viewBox") ?? "").split(" ").map(Number));
    assert.ok(Math.abs(panel.width / panel.height - vw! / vh!) < 0.01, "the aspect of the plate is kept");
    // The pixels: the case's corner is the frame, the plate is the plate, the ground beside the strip is the page.
    const [corner, inPlate, ground] = await pixels(await page.screenshot(), [
      [panel.left + 2, panel.top + 2],
      [plate.left + 4, plate.top + 4],
      [stage.left + 1, stage.top + 1],
    ]);
    assert.deepEqual(corner, [...hex("frame")], "the corner of the case");
    assert.deepEqual(inPlate, [...hex("plate")], "the corner of the plate");
    assert.deepEqual(ground, [...hex("surface")], "the ground");
    const scroll = await page.evaluate(() => ({ h: document.documentElement.scrollHeight - innerHeight, w: document.documentElement.scrollWidth - innerWidth }));
    assert.ok(scroll.h <= 0 && scroll.w <= 0, `the page scrolls: ${JSON.stringify(scroll)}`);
    await page.close();
  });
}

test("P9-8: the ground carries three hairline line drawings, drawn in the control-edge tone, with no fill, that nothing can touch", async () => {
  const page = await open();
  assert.equal(await page.$eval("#ground", (el) => el.getAttribute("aria-hidden")), "true", "decoration, hidden from the accessibility tree");
  assert.equal(await page.$eval("#ground", (el) => el.getAttribute("focusable")), "false");
  assert.equal(await page.$$eval("#ground a, #ground button, #ground [tabindex], #ground text", (els) => els.length), 0, "nothing in it is a control or says anything");
  assert.equal(await style(page, "#ground", "pointer-events"), "none");
  assert.equal(await page.$$eval("#ground g.drawing", (els) => els.length), 3, "three drawings");
  const shapes = await page.$$eval("#ground .line, #ground .dots", (els) =>
    els.map((el) => {
      const s = getComputedStyle(el);
      return { cls: el.getAttribute("class") ?? "", fill: s.fill, stroke: s.stroke, width: s.strokeWidth, effect: s.getPropertyValue("vector-effect"), dash: s.strokeDasharray, cap: s.strokeLinecap };
    }),
  );
  assert.ok(shapes.length > 20, `${shapes.length} strokes`);
  const solid = shapes.filter((s) => s.cls.includes("line"));
  const dotted = shapes.filter((s) => s.cls.includes("dots"));
  assert.ok(solid.length > 0 && dotted.length > 0, "solid lines and dotted construction lines");
  for (const s of shapes) {
    assert.equal(s.fill, "none", `${s.cls}: no fill`);
    assert.equal(s.width, `${tokens.stroke.hairline.value}px`, `${s.cls}: a hairline`);
    assert.equal(s.effect, "non-scaling-stroke", `${s.cls}: the hairline stays a hairline when the drawing scales`);
  }
  for (const s of solid) eq(s.stroke, hex("edge.control"), "a solid line");
  // The hairline tone is 1.28:1 on the ground: a dotted line in it is not there, so the dots are in the control edge too, and measured.
  assert.ok(contrast(hex("edge.control"), hex("surface")) >= 3, "the drawings are seen");
  for (const s of dotted) {
    eq(s.stroke, hex("edge.control"), "a dotted construction line");
    assert.equal(s.dash, `0px, ${tokens.space["1"].value}px`, "dots: a zero-length dash every space.1");
    assert.equal(s.cap, "round", "dots: a round cap makes the dot");
  }
  // Nothing in the ground can be hit: a click through the middle of it lands on the stage.
  const box = await rectOf(page, "#ground");
  assert.ok(box.width > 0 && box.height > 0, "there is ground at this size");
  const hit = await page.evaluate(([x, y]) => document.elementFromPoint(x!, y!)?.closest("#ground") !== null, [box.left + box.width / 2, box.top + box.height / 2] as const);
  assert.equal(hit, false, "the ground takes no pointer");
  await page.close();
});

test("P9-9: the drawings are in the room the case leaves: above it, under the strip, never behind either, and gone when there is none", async () => {
  for (const [width, height, room] of [
    [1280, 900, true],
    [1920, 1080, true],
    [1600, 480, false],
  ] as const) {
    const page = await open(width, height);
    const strip = await rectOf(page, "#strip");
    const panel = await rectOf(page, "#panel");
    const ground = await rectOf(page, "#ground");
    assert.ok(ground.top >= strip.bottom - 1, `${width}x${height}: the ground is under the strip`);
    assert.ok(ground.bottom <= panel.top + 1, `${width}x${height}: the ground is above the case, not behind it (${ground.bottom} against ${panel.top})`);
    assert.ok(ground.left >= 0 && ground.right <= width + 1, `${width}x${height}: inside the window`);
    if (room) assert.ok(ground.height >= 60, `${width}x${height}: the case leaves ${ground.height}px of ground`);
    else assert.ok(ground.height < 12, `${width}x${height}: no room is left, so there is none (${ground.height}px)`);
    // Every stroke of every drawing is inside the ground, so nothing is cut off or runs under the case.
    const strokes = await page.$$eval("#ground .line, #ground .dots", (els) =>
      els.map((el) => {
        const r = el.getBoundingClientRect();
        return { left: r.left, top: r.top, right: r.right, bottom: r.bottom };
      }),
    );
    for (const r of strokes) assert.ok(r.top >= ground.top - 1 && r.bottom <= ground.bottom + 1 && r.left >= ground.left - 1 && r.right <= ground.right + 1, `${width}x${height}: a stroke is outside the ground`);
    await page.close();
  }
});

test("P9-10: the drawings are still: nothing in the ground animates or moves between two frames", async () => {
  const page = await open();
  const before = await page.$eval("#ground", (el) => el.outerHTML);
  assert.equal(await page.evaluate(() => document.getAnimations().filter((a) => (a.effect as KeyframeEffect | null)?.target?.closest?.("#ground")).length), 0, "no animation targets the ground");
  const shot1 = await page.screenshot({ clip: { ...(await rectOf(page, "#ground")), x: (await rectOf(page, "#ground")).left, y: (await rectOf(page, "#ground")).top } });
  await page.waitForTimeout(400);
  const shot2 = await page.screenshot({ clip: { ...(await rectOf(page, "#ground")), x: (await rectOf(page, "#ground")).left, y: (await rectOf(page, "#ground")).top } });
  assert.ok(shot1.equals(shot2), "the ground is the same picture 400 ms later");
  assert.equal(await page.$eval("#ground", (el) => el.outerHTML), before, "and the same markup");
  await page.close();
});

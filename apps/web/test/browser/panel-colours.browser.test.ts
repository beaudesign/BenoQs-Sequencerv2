// The panel drawn in the hardware's own materials, in Chromium (WENGE-0022, ADR-0010, specs/SPEC-0002/p8b-panel-in-the-hardwares-materials.md):
// a cream plate in a dark frame, keys in one flat metal tone, printed names in a dark ink, an LED rim and a focus ring that show on the plate.
// What is measured is what the browser computed and what it rendered, not what the stylesheet says: a rule that never applied would pass a scan.
// Needs `npm run build` first, which `npm run test:browser` does.
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
/** `rgb(1, 2, 3)` as the browser computes a colour. */
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

const key = (id: string): string => `g[data-id="${id}"]`;
const style = (page: Page, selector: string, prop: string): Promise<string> => page.$eval(selector, (el, p) => getComputedStyle(el).getPropertyValue(p), prop);
const styles = (page: Page, selector: string, prop: string): Promise<string[]> => page.$$eval(selector, (els, p) => els.map((el) => getComputedStyle(el).getPropertyValue(p)), prop);

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

test("P8-1: the plate, the frame, the keys and the knobs are the hardware's flat tones, and the strip is not", async () => {
  const page = await open();
  const eq = (got: string, want: Rgb, what: string): void => assert.deepEqual(parse(got), [...want], `${what}: ${got}, wanted ${show(want)}`);
  eq(await style(page, "#panel rect.plate", "fill"), hex("plate"), "the plate");
  assert.equal(await style(page, "#panel rect.plate", "stroke"), "none", "the plate has no outline: the frame is its edge");
  eq(await style(page, "#panel-frame", "background-color"), hex("frame"), "the frame");
  assert.ok(parseFloat(await style(page, "#panel-frame", "padding-top")) >= tokens.space["3"].value, "the plate does not touch the edge of its frame");
  for (const [what, selector] of [
    ["a matrix key", `${key("matrix.r0.c1")} .face`],
    ["a key of the plate that does nothing", "#panel g.inert .face"],
    ["a knob's ball", `${key("main.enc")} .ball`],
  ] as const) {
    eq(await style(page, selector, "fill"), hex("key"), `${what}: its metal`);
    eq(await style(page, selector, "stroke"), hex("key.edge"), `${what}: its edge`);
  }
  eq(await style(page, `${key("main.enc")} .nut`, "fill"), hex("key"), "a knob's nut");
  eq(await style(page, "body", "background-color"), hex("surface"), "the page");
  eq(await style(page, "#strip", "border-bottom-color"), hex("ink.quiet"), "the strip's rule");
  eq(await style(page, "#play", "background-color"), hex("surface"), "a button of the strip");
  await page.close();
});

test("P8-2: the printed names are dark ink on the plate, 7 to 1 or better, measured on the colours the browser computed", async () => {
  const page = await open();
  const plate = parse(await style(page, "#panel rect.plate", "fill"));
  const printed = await page.$$eval("#panel text.legend, #panel text.label", (els) => els.map((el) => ({ cls: el.getAttribute("class") ?? "", text: el.textContent ?? "", fill: getComputedStyle(el).fill })));
  assert.ok(printed.length > 100, `${printed.length} printed names`);
  for (const t of printed) {
    const quiet = /\b(caption|row|clef)\b/.test(t.cls);
    const fill = parse(t.fill);
    assert.deepEqual(fill, [...hex(quiet ? "plate.ink.quiet" : "plate.ink")], `"${t.text}" (${t.cls})`);
    assert.ok(contrast(fill, plate) >= 7, `"${t.text}" is ${contrast(fill, plate).toFixed(2)}:1 on the plate`);
  }
  await page.close();
});

test("P8-3: rendered, the plate, the frame, a key and the page are those tones, pixel for pixel", async () => {
  const page = await open();
  const at = await page.evaluate(() => {
    const rect = (sel: string): DOMRect => document.querySelector(sel)!.getBoundingClientRect();
    const plate = rect("#panel rect.plate");
    const frame = rect("#panel-frame");
    const face = rect('g[data-id="matrix.r4.c7"] .face');
    const strip = rect("#strip");
    return {
      plate: [plate.left + 3, plate.top + 3],
      frame: [frame.left + 3, frame.top + 3],
      key: [face.left + face.width / 2, face.top + face.height / 2],
      strip: [strip.right - 3, strip.top + 3],
    } as const;
  });
  const [plate, frame, face, strip] = await pixels(await page.screenshot(), [at.plate, at.frame, at.key, at.strip]);
  assert.deepEqual(plate, [...hex("plate")], "the plate");
  assert.deepEqual(frame, [...hex("frame")], "the frame");
  assert.deepEqual(face, [...hex("key")], "the middle of a key");
  assert.deepEqual(strip, [...hex("surface")], "the strip");
  await page.close();
});

test("P8-4: a lit LED is marked by more than its colour: a rim, dark enough on the plate, whatever the colour and the phase", async () => {
  const page = await open();
  const plate = parse(await style(page, "#panel rect.plate", "fill"));
  const led = `${key("matrix.r3.c5")} .led`;
  const lit = async (colour: string, phase: string) => {
    await page.$eval(led, (el, [c, p]) => (el.setAttribute("data-colour", c!), el.setAttribute("data-phase", p!)), [colour, phase]);
    return { fill: await style(page, led, "fill"), rim: await style(page, led, "stroke"), width: parseFloat(await style(page, led, "stroke-width")) };
  };
  for (const colour of ["red", "green", "orange"] as const) {
    const steady = await lit(colour, "steady");
    assert.deepEqual(parse(steady.fill), [...hex(`led.${colour}`)], `${colour}: the lit colour`);
    assert.deepEqual(parse(steady.rim), [...hex("plate.ink")], `${colour}: the rim is the ink`);
    assert.equal(steady.width, tokens.stroke.hairline.value, `${colour}: a hairline`);
    assert.ok(contrast(parse(steady.rim), plate) >= 3, `${colour}: the rim is ${contrast(parse(steady.rim), plate).toFixed(2)}:1 on the plate`);
    const shine = await lit(colour, "shine");
    assert.deepEqual(parse(shine.rim), [...hex("plate.ink")], `${colour} shining: the rim is the ink`);
    assert.ok(shine.width > steady.width, `${colour} shining: a heavier rim than steady (${shine.width} against ${steady.width})`);
    const flash = await lit(colour, "flash");
    assert.deepEqual(parse(flash.rim), [...hex("plate.ink")], `${colour} flashing: the rim holds while the fill flashes`);
    assert.equal(flash.width, steady.width);
  }
  const off = await lit("off", "steady");
  assert.deepEqual(parse(off.fill), [...hex("led.off")], "a dark LED");
  assert.deepEqual(parse(off.rim), [...hex("plate.ink")], "a dark LED has the rim too, so it is a lens and not a hole");
  const dead = await page.$eval("#panel .dot", (el) => ({ fill: getComputedStyle(el).fill, rim: getComputedStyle(el).stroke }));
  assert.deepEqual(parse(dead.fill), [...hex("led.off")], "a dot the engine has no LED for stays dark");
  assert.deepEqual(parse(dead.rim), [...hex("plate.ink")]);
  await page.close();
});

test("P8-5: a pressed key shows it on the plate: its edge goes to the ink", async () => {
  const page = await open();
  const face = `${key("matrix.r4.c7")} .face`;
  const rest = parse(await style(page, face, "stroke"));
  assert.deepEqual(rest, [...hex("key.edge")]);
  await page.$eval(key("matrix.r4.c7"), (g) => g.setAttribute("data-down", ""));
  const down = parse(await style(page, face, "stroke"));
  assert.deepEqual(down, [...hex("plate.ink")], "pressed");
  assert.ok(contrast(down, parse(await style(page, "#panel rect.plate", "fill"))) > contrast(rest, parse(await style(page, "#panel rect.plate", "fill"))), "pressed is darker against the plate than at rest");
  await page.close();
});

test("P8-6: the focus ring shows on the plate and on the key: one stroke is 3 to 1 or better against each, measured on computed colours", async () => {
  const page = await open();
  await page.focus(key("matrix.r1.c2"));
  await page.keyboard.press("Tab");
  await page.keyboard.press("Shift+Tab");
  assert.equal(await page.evaluate(() => document.activeElement?.getAttribute("data-id")), "matrix.r1.c2");
  const ring = await page.$eval(key("matrix.r1.c2"), (g) => {
    const read = (cls: string) => {
      const s = getComputedStyle(g.querySelector(cls)!);
      return { display: s.display, stroke: s.stroke, width: parseFloat(s.strokeWidth) };
    };
    return { outer: read(".ring-outer"), inner: read(".ring-inner"), face: getComputedStyle(g.querySelector(".face")!).fill };
  });
  const plate = parse(await style(page, "#panel rect.plate", "fill"));
  const face = parse(ring.face);
  assert.notEqual(ring.outer.display, "none", "the outer stroke shows on focus");
  assert.notEqual(ring.inner.display, "none", "the inner stroke shows on focus");
  const outer = parse(ring.outer.stroke);
  const inner = parse(ring.inner.stroke);
  assert.notDeepEqual(outer, inner, "two strokes of two tones");
  assert.equal(ring.outer.width, tokens.stroke.focus.value);
  assert.equal(ring.inner.width, tokens.stroke.focus_inner.value);
  assert.ok(contrast(outer, plate) >= 3, `the outer stroke is ${contrast(outer, plate).toFixed(2)}:1 on the plate`);
  assert.ok(Math.max(contrast(outer, face), contrast(inner, face)) >= 3, `the ring is ${Math.max(contrast(outer, face), contrast(inner, face)).toFixed(2)}:1 on the key`);
  await page.close();
});

test("P8-7: the whole plate is still on screen, in its frame, at 1280 by 900 and at 900 by 700", async () => {
  for (const [w, h] of [
    [1280, 900],
    [900, 700],
  ] as const) {
    const page = await open(w, h);
    const box = await page.evaluate(() => {
      const plate = document.querySelector("#panel rect.plate")!.getBoundingClientRect();
      const frame = document.querySelector("#panel-frame")!.getBoundingClientRect();
      return { plate, frame, vw: innerWidth, vh: innerHeight };
    });
    assert.ok(box.plate.left >= box.frame.left && box.plate.top >= box.frame.top && box.plate.right <= box.frame.right && box.plate.bottom <= box.frame.bottom, `the plate is inside its frame at ${w} by ${h}`);
    assert.ok(box.plate.left > box.frame.left && box.plate.top > box.frame.top, `there is frame round the plate at ${w} by ${h}`);
    assert.ok(box.frame.right <= box.vw && box.frame.bottom <= box.vh, `the frame is on screen at ${w} by ${h}`);
    await page.close();
  }
});

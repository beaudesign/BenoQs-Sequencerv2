// The Curve button's presets and its place in the strip (SPEC-0005 r1 S2, D6, CV-12). The engine's side is apps/web/engine/tests/abi.rs;
// the button in a running page is test/browser/curve.browser.test.ts.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { CHORD_STYLE } from "../src/abi.ts";
import { CURVE_PRESETS, curveText, nextCurve } from "../src/curve.ts";
import { webRoot } from "./support/module.ts";

const html = readFileSync(`${webRoot}/pages/app.html`, "utf8");
const css = readFileSync(`${webRoot}/pages/app.css`, "utf8");

test("the presets are Off, Accel (-0.5), Bounce (-1) and Rit (0.5), in that order, in the engine's own numbers (D6)", () => {
  assert.deepEqual(
    CURVE_PRESETS.map((p) => [p.label, p.style, p.warpTenths]),
    [
      ["Off", 0, 0],
      ["Accel", 1, -5],
      ["Bounce", 1, -10],
      ["Rit", 1, 5],
    ],
  );
  assert.deepEqual(CHORD_STYLE, { manual: 0, curve: 1 });
});

test("every preset is one of the engine's stored warps (-1, -0.5, 0, 0.5, 1), so the engine never refuses the button", () => {
  for (const p of CURVE_PRESETS) {
    assert.ok([CHORD_STYLE.manual, CHORD_STYLE.curve].includes(p.style as 0 | 1), p.label);
    if (p.style === CHORD_STYLE.curve) assert.ok([-10, -5, 0, 5, 10].includes(p.warpTenths), `${p.label}: ${p.warpTenths}`);
  }
});

test("a press steps Off, Accel, Bounce, Rit and round to Off", () => {
  const seen = [0];
  for (let i = 0; i < 8; i++) seen.push(nextCurve(seen[i]!));
  assert.deepEqual(seen, [0, 1, 2, 3, 0, 1, 2, 3, 0]);
});

test("the button names its state: `Curve: ` and the preset", () => {
  assert.deepEqual(CURVE_PRESETS.map((_, i) => curveText(i)), ["Curve: Off", "Curve: Accel", "Curve: Bounce", "Curve: Rit"]);
});

test("the strip has one Curve button, after Metronome, a plain button, off to begin with, named for its state", () => {
  const tags = [...html.matchAll(/<button\b[^>]*\bid="curve"[^>]*>([^<]*)<\/button>/g)];
  assert.equal(tags.length, 1);
  assert.match(tags[0]![0], /type="button"/);
  assert.match(tags[0]![0], /aria-pressed="false"/);
  assert.equal(tags[0]![1], curveText(0));
  const order = [...html.matchAll(/<button\b[^>]*\bid="([a-z0-9-]+)"/g)].map((m) => m[1]);
  assert.equal(order[order.indexOf("metronome") + 1], "curve");
  const strip = html.slice(html.indexOf('<section id="strip"'), html.indexOf("</section>"));
  assert.ok(strip.includes('id="curve"'), "in the strip, not on the panel");
});

test("CV-12: the button has no rule of its own: it takes the strip's raised and pressed tones and nothing else", () => {
  assert.ok(!/#curve\b/.test(css), "app.css does not name #curve");
  assert.ok(/#strip button\[aria-pressed="true"\]/.test(css), "the shared rule that gives a button that is on its tone");
});

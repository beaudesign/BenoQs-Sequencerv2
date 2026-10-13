// The ground's line drawings (ADR-0011 decision 10) are drawn from the space tokens and nothing else: no length is written in src/ground.ts (rule S12),
// so the picture is a function of the scale. What they look like is the stylesheet's, and is measured in Chromium (test/browser/shell-look.browser.test.ts).
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { drawGround } from "../src/ground.ts";
import { parseTokens, SPACE_KEYS, type Tokens } from "../src/tokens.ts";
import { repoRoot } from "./support/module.ts";

const tokens = parseTokens(JSON.parse(readFileSync(`${repoRoot}/contracts/design.tokens.json`, "utf8")));

/** The little of a DOM the drawing needs: elements with attributes and children. */
class El {
  readonly attrs = new Map<string, string>();
  readonly children: El[] = [];
  readonly name: string;
  constructor(name: string) {
    this.name = name;
  }
  setAttribute(k: string, v: string): void {
    this.attrs.set(k, v);
  }
  append(...c: El[]): void {
    this.children.push(...c);
  }
  walk(): El[] {
    return [this, ...this.children.flatMap((c) => c.walk())];
  }
}
const doc = { createElementNS: (_ns: string, name: string) => new El(name) } as unknown as Document;

const draw = (t: Tokens): El => drawGround(doc, t) as unknown as El;
const scaled = (factor: number): Tokens => {
  const t = structuredClone(tokens);
  for (const k of SPACE_KEYS) t.space[k].value *= factor;
  return t;
};

/** Every number in the geometry of an element, in order. Arc flags in a path (`0 1 0`) are not lengths and are left out. */
const numbers = (el: El): number[] =>
  [...el.attrs]
    .filter(([k]) => ["x1", "y1", "x2", "y2", "cx", "cy", "r", "points", "d", "transform", "viewBox"].includes(k))
    .flatMap(([k, v]) => (k === "d" ? v.replaceAll(" 0 1 0 ", " ") : v).match(/-?\d+(?:\.\d+)?/g) ?? [])
    .map(Number);

test("the ground is one decorative SVG with three drawings: a detent, a stack and an orbit", () => {
  const root = draw(tokens);
  assert.equal(root.name, "svg");
  assert.equal(root.attrs.get("id"), "ground");
  assert.equal(root.attrs.get("aria-hidden"), "true", "decoration is hidden from the accessibility tree");
  assert.equal(root.attrs.get("focusable"), "false");
  assert.equal(root.attrs.get("preserveAspectRatio"), "xMidYMid meet", "the row stays whole in the room the case leaves");
  assert.equal((root.attrs.get("viewBox") ?? "").split(" ").length, 4);
  assert.deepEqual(root.children.map((g) => g.attrs.get("class")), ["drawing detent", "drawing stack", "drawing orbit"]);
});

test("a drawing is strokes only: solid lines and dotted construction lines, and nothing that carries a colour, a fill, a style or a word", () => {
  const shapes = draw(tokens).walk().filter((e) => !["svg", "g"].includes(e.name));
  assert.ok(shapes.length >= 20, `${shapes.length} shapes`);
  for (const s of shapes) {
    assert.ok(["line", "circle", "polygon", "path"].includes(s.name), s.name);
    assert.ok(["line", "dots"].includes(s.attrs.get("class") ?? ""), `${s.name} is a solid line or a dotted one`);
    for (const k of s.attrs.keys()) assert.ok(!["fill", "stroke", "style", "stroke-width", "opacity", "filter"].includes(k), `${s.name} sets ${k}: the stylesheet draws`);
  }
  const kinds = new Set(shapes.map((s) => s.attrs.get("class")));
  assert.deepEqual([...kinds].sort(), ["dots", "line"]);
  assert.equal(draw(tokens).walk().filter((e) => e.name === "text").length, 0, "no words");
  assert.ok(shapes.filter((s) => s.attrs.get("class") === "dots").every((s) => s.name === "line" || s.name === "circle"), "construction lines are lines and a ring");
});

test("the drawing is a function of the space scale: doubling the scale doubles every length, and the same tokens give the same bytes", () => {
  const one = draw(tokens).walk();
  const two = draw(scaled(2)).walk();
  assert.equal(one.length, two.length);
  one.forEach((e, i) => {
    assert.deepEqual(numbers(two[i]!), numbers(e).map((n) => n * 2), `${e.name}.${e.attrs.get("class") ?? ""} at ${i}`);
  });
  const again = draw(tokens).walk();
  assert.deepEqual(again.map(numbers), one.map(numbers));
  assert.ok(numbers(one[0]!).length > 0, "the view box has numbers");
});

test("every coordinate is a step of the scale, a sum of them, or half of one: nothing is placed by eye", () => {
  const unit = tokens.space["1"].value / 2; // half the smallest step
  for (const e of draw(tokens).walk()) for (const n of numbers(e)) assert.ok(n % unit === 0, `${e.name}.${e.attrs.get("class") ?? ""}: ${n} is not a multiple of ${unit}`);
});

test("the row fits its view box: every circle, line and plane is inside it", () => {
  const root = draw(tokens);
  const [vx, vy, vw, vh] = (root.attrs.get("viewBox") ?? "").split(" ").map(Number) as [number, number, number, number];
  for (const g of root.children) {
    const [tx, ty] = (/translate\((-?[\d.]+) (-?[\d.]+)\)/.exec(g.attrs.get("transform") ?? "") ?? []).slice(1).map(Number) as [number, number];
    for (const s of g.children) {
      const pts: [number, number][] = [];
      const a = (k: string): number => Number(s.attrs.get(k));
      if (s.name === "line") pts.push([a("x1"), a("y1")], [a("x2"), a("y2")]);
      if (s.name === "circle") pts.push([a("cx") - a("r"), a("cy") - a("r")], [a("cx") + a("r"), a("cy") + a("r")]);
      if (s.name === "polygon") for (const p of (s.attrs.get("points") ?? "").split(" ")) pts.push(p.split(",").map(Number) as [number, number]);
      for (const [x, y] of pts) assert.ok(tx + x >= vx && tx + x <= vx + vw && ty + y >= vy && ty + y <= vy + vh, `${g.attrs.get("class")}: (${tx + x}, ${ty + y}) is outside ${root.attrs.get("viewBox")}`);
    }
  }
});

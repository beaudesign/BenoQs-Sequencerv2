// The first panel's geometry: what is drawn, where, in what order, and that every number in it comes from a token
// or from the control inventory (no pixel literal in the panel code, rule S12).
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { parseLayout, place, tabOrder, wrapLabel, type ControlsDoc, type LayoutFile } from "../src/layout.ts";
import { parseTokens, type Tokens } from "../src/tokens.ts";
import { repoRoot } from "./support/module.ts";

const read = (rel: string): unknown => JSON.parse(readFileSync(`${repoRoot}/${rel}`, "utf8"));
const controls = read("contracts/controls.json") as ControlsDoc;
const tokens = parseTokens(read("contracts/design.tokens.json"));
const layoutFile = read("apps/web/layout/panel.layout.json");

/** The eleven the panel draws besides the matrix (ADR-0008 decision 4, D-P3-4). */
const ELEVEN = [
  "mode.page", "mode.step", "mode.play", "mode.edit", "keys.esc", "keys.program",
  "mutator.tgl", "mutator.zom", "mutator.mute", "transport.stop", "transport.play",
];

function geometry(t: Tokens = tokens, file: unknown = layoutFile) {
  return place(parseLayout(file, controls), controls, t);
}

test("the panel draws the 160 matrix keys, the ten roles octoface acts on, and transport.play, and nothing else", () => {
  const ids = geometry().placements.map((p) => p.id);
  const matrix = controls.controls.filter((c) => c.zone === "matrix").map((c) => c.id);
  assert.equal(matrix.length, 160);
  assert.deepEqual([...ids].sort(), [...matrix, ...ELEVEN].sort());
  assert.equal(new Set(ids).size, ids.length, "each control once");
});

test("tab order is the matrix in reading order (top row first, left to right), then the other controls in the file's order", () => {
  const order = tabOrder(geometry());
  assert.deepEqual(order.slice(0, 3), ["matrix.r9.c1", "matrix.r9.c2", "matrix.r9.c3"]);
  assert.equal(order[15], "matrix.r9.c16");
  assert.equal(order[16], "matrix.r8.c1");
  assert.equal(order[159], "matrix.r0.c16");
  assert.deepEqual(order.slice(160).sort(), [...ELEVEN].sort());
  const file = layoutFile as LayoutFile;
  assert.deepEqual(order.slice(160), file.controls.map((c) => c.id), "the layout file's order is the tab order");
});

test("the tab order is the visual reading order: each control is no earlier than the one before it, row by row", () => {
  const g = geometry();
  const rows = g.placements.map((p) => p.cy);
  for (let i = 1; i < rows.length; i++) assert.ok(rows[i]! >= rows[i - 1]! - 1e-9, `${g.placements[i]!.id} is above ${g.placements[i - 1]!.id}`);
  // Within a row, left to right.
  for (let i = 1; i < g.placements.length; i++) {
    const a = g.placements[i - 1]!;
    const b = g.placements[i]!;
    if (Math.abs(a.cy - b.cy) < 1e-9) assert.ok(b.cx > a.cx, `${b.id} is left of ${a.id} in the same row`);
  }
});

test("a key sits at its column and row times the pitch, and the pitch is key plus gap from the tokens", () => {
  const key = tokens.space["6"].value;
  const gap = tokens.space["2"].value;
  const pitch = key + gap;
  const g = geometry();
  const at = (id: string) => g.placements.find((p) => p.id === id)!;
  assert.equal(at("matrix.r9.c1").cx, key / 2);
  assert.equal(at("matrix.r9.c1").cy, key / 2);
  assert.equal(at("matrix.r0.c16").cx, 15 * pitch + key / 2);
  assert.equal(at("matrix.r0.c16").cy, 9 * pitch + key / 2, "row 0 is at the bottom (controls.json, zone matrix)");
  assert.equal(at("matrix.r4.c7").r, key / 2);
});

test("no pixel is written in the layout: changing a token moves the panel, and nothing else does", () => {
  const bigger = structuredClone(tokens);
  bigger.space["6"].value = 84; // a key of the largest step
  const a = geometry(tokens);
  const b = geometry(bigger);
  const first = (g: typeof a, id: string) => g.placements.find((p) => p.id === id)!;
  assert.ok(first(b, "matrix.r0.c16").cx > first(a, "matrix.r0.c16").cx);
  assert.equal(first(b, "matrix.r9.c1").r, 42);
  assert.ok(b.width > a.width && b.height > a.height);
  // The proportions of the layout (cells) do not change: the pitch ratio does what the tokens say.
  const pa = first(a, "matrix.r0.c2").cx - first(a, "matrix.r0.c1").cx;
  const pb = first(b, "matrix.r0.c2").cx - first(b, "matrix.r0.c1").cx;
  assert.equal(pa, tokens.space["6"].value + tokens.space["2"].value);
  assert.equal(pb, 84 + tokens.space["2"].value);
});

test("the layout file holds cells and token names, and no number that is a length", () => {
  const text = JSON.stringify(layoutFile);
  assert.ok(!/\d+(\.\d+)?\s*(px|rem|em|pt)/.test(text), "no unit");
  const file = layoutFile as LayoutFile;
  for (const ref of [file.size.key, file.size.gap, file.size.margin]) assert.match(ref, /^space\.[0-7]$/, "a token name");
});

test("nothing overlaps, and everything is inside the view box", () => {
  const g = geometry();
  const [x0, y0, w, h] = g.viewBox;
  for (const p of g.placements) {
    assert.ok(p.cx - p.r >= x0 && p.cy - p.r >= y0 && p.cx + p.r <= x0 + w && p.cy + p.r <= y0 + h, `${p.id} leaves the view box`);
    if (p.label) {
      assert.ok(p.label.x >= x0 && p.label.x <= x0 + w, `${p.id}'s label starts outside`);
    }
  }
  for (let i = 0; i < g.placements.length; i++) {
    for (let j = i + 1; j < g.placements.length; j++) {
      const a = g.placements[i]!;
      const b = g.placements[j]!;
      assert.ok(Math.hypot(a.cx - b.cx, a.cy - b.cy) >= a.r + b.r, `${a.id} touches ${b.id}`);
    }
  }
});

test("a control's label does not run into the next control in its row", () => {
  const file = layoutFile as LayoutFile;
  const g = geometry();
  const pitch = tokens.space["6"].value + tokens.space["2"].value;
  const labelled = g.placements.filter((p) => p.label);
  for (const p of labelled) {
    const widest = Math.max(...p.label!.lines.map((l) => l.length));
    assert.ok(widest <= file.label.chars, `${p.id}: ${widest} characters`);
    const right = p.label!.x + file.label.span * pitch - tokens.space["2"].value;
    for (const q of g.placements) {
      if (q === p || Math.abs(q.cy - p.cy) > 1e-9 || q.cx < p.cx) continue;
      assert.ok(q.cx - q.r >= right - 1e-9 || q.cx === p.cx, `${p.id}'s label (${file.label.span} cells) reaches ${q.id}`);
    }
  }
});

test("matrix keys carry no visible label; the other controls carry the manual's name, wrapped on whole words", () => {
  const g = geometry();
  for (const p of g.placements) {
    const control = controls.controls.find((c) => c.id === p.id)!;
    if (p.id.startsWith("matrix.")) assert.equal(p.label, null);
    else {
      assert.ok(p.label, p.id);
      assert.equal(p.label.lines.join(" "), control.name, "the manual's own name, whole");
    }
    assert.equal(p.name, control.name);
    assert.equal(p.n, control.n);
  }
});

test("an LED is drawn for exactly the controls the manual gives one (transport.stop and transport.play have none)", () => {
  for (const p of geometry().placements) {
    const control = controls.controls.find((c) => c.id === p.id)!;
    assert.equal(p.led, control.led === true, p.id);
  }
  const at = (id: string) => geometry().placements.find((p) => p.id === id)!;
  assert.equal(at("transport.stop").led, false);
  assert.equal(at("transport.play").led, false);
  assert.equal(at("matrix.r0.c1").led, true);
});

test("wrapLabel breaks on spaces, keeps a long word whole, and loses nothing", () => {
  assert.deepEqual(wrapLabel("Transport Stop button", 16), ["Transport Stop", "button"]);
  assert.deepEqual(wrapLabel("ESC", 16), ["ESC"]);
  assert.deepEqual(wrapLabel("ab cd", 5), ["ab cd"], "a line of exactly the limit fits");
  assert.deepEqual(wrapLabel("ab cd", 4), ["ab", "cd"], "one more than the limit does not");
  assert.deepEqual(wrapLabel("Matrix key, row 0, column 1", 12), ["Matrix key,", "row 0,", "column 1"]);
  assert.deepEqual(wrapLabel("Extraordinarily long", 5), ["Extraordinarily", "long"]);
  for (const c of controls.controls) assert.equal(wrapLabel(c.name, 14).join(" "), c.name);
});

test("parseLayout refuses what cannot be drawn", () => {
  const good = structuredClone(layoutFile) as LayoutFile;
  const bad = (edit: (f: LayoutFile) => void): Error => {
    const f = structuredClone(good);
    edit(f);
    assert.throws(() => parseLayout(f, controls), (e: unknown) => e instanceof Error);
    try {
      parseLayout(f, controls);
    } catch (e) {
      return e as Error;
    }
    throw new Error("unreachable");
  };
  assert.match(bad((f) => (f.controls[0]!.id = "no.such.control")).message, /no\.such\.control/);
  assert.match(bad((f) => f.controls.push({ ...f.controls[0]! })).message, /twice/);
  assert.match(bad((f) => f.controls.push({ id: "matrix.r0.c1", col: 0, row: 20 })).message, /matrix/);
  assert.match(bad((f) => (f.size.key = "space.9")).message, /space\.9/);
  assert.match(bad((f) => (f.size.gap = "12px" as never)).message, /12px/);
  assert.match(bad((f) => ((f as { schema: string }).schema = "panel.layout/2")).message, /schema/);
  assert.match(bad((f) => (f.controls[1]!.col = -1)).message, /col/);
  assert.match(bad((f) => (f.controls[1]!.row = 1.5)).message, /row/);
  assert.match(bad((f) => (f.status = "final" as never)).message, /provisional/);
});

test("the layout is marked provisional and says why", () => {
  const file = layoutFile as LayoutFile;
  assert.equal(file.status, "provisional");
  assert.match(file.note, /Q23/);
  assert.match(file.note, /manual/);
});

test("the panel is wide enough for a label that runs past the matrix", () => {
  const key = tokens.space["6"].value;
  const gap = tokens.space["2"].value;
  const pitch = key + gap;
  const file = structuredClone(layoutFile) as LayoutFile;
  const stop = file.controls.find((c) => c.id === "transport.stop")!;
  stop.col = 18; // past the sixteenth column of the matrix
  const g = geometry(tokens, file);
  assert.equal(g.width, (18 + 1 + file.label.span) * pitch - gap, "the label's two cells are inside the width");
  const [x0, , w] = g.viewBox;
  const p = g.placements.find((q) => q.id === "transport.stop")!;
  assert.ok(p.label!.x + file.label.span * pitch - gap <= x0 + w, "the label ends inside the view box");
});

test("the height is the last row drawn", () => {
  const key = tokens.space["6"].value;
  const gap = tokens.space["2"].value;
  const file = structuredClone(layoutFile) as LayoutFile;
  const last = Math.max(...file.controls.map((c) => c.row));
  assert.equal(geometry(tokens, file).height, (last + 1) * (key + gap) - gap);
});

test("a label starts one gap to the right of its key's edge, and is centred on the key's height", () => {
  const gap = tokens.space["2"].value;
  for (const p of geometry().placements.filter((q) => q.label)) {
    assert.equal(p.label!.x, p.cx + p.r + gap, p.id);
    assert.equal(p.label!.y, p.cy, p.id);
  }
});

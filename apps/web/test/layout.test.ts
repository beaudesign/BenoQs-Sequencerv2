// The panel's geometry (WENGE-0019, specs/SPEC-0002/p7-faithful-panel.md): where every control the inventory holds is drawn, that the
// positions are the ones measured from the photograph the owner named, that the blocks sit where the hardware has them, that nothing
// overlaps, and that every length comes from a token or from the inventory (no pixel literal in the panel code, rule S12).
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { LABEL_CHARS_EM, parseLayout, place, tabOrder, type ControlsDoc, type Geometry, type LayoutFile, type Placement } from "../src/layout.ts";
import { parseTokens, type Tokens } from "../src/tokens.ts";
import { repoRoot } from "./support/module.ts";

const read = (rel: string): unknown => JSON.parse(readFileSync(`${repoRoot}/${rel}`, "utf8"));
const controls = read("contracts/controls.json") as ControlsDoc;
const tokens = parseTokens(read("contracts/design.tokens.json"));
const layoutFile = read("apps/web/layout/panel.layout.json") as LayoutFile;
const measured = read("handoffs/evidence/panel-layout/measured.json") as {
  plate_u: [number, number][];
  matrix: { row_over_col: number };
  found: { kind: "key" | "knob"; u: [number, number] }[];
};

function geometry(t: Tokens = tokens, file: unknown = layoutFile): Geometry {
  return place(parseLayout(file, controls), controls, t);
}

const g = geometry();
const byId = (id: string): Placement => {
  const p = g.placements.find((q) => q.id === id);
  assert.ok(p, `${id} is drawn`);
  return p;
};
const ids = (prefix: string): Placement[] => g.placements.filter((p) => p.id.startsWith(prefix));
const pitch = tokens.space["4"].value + tokens.space["3"].value;

/** The five controls the photograph shows no place for. The list can shrink and must not grow (the ratchet on what is not drawn). */
const UNPLACED = ["circle.low_c", "circle.chr", "circle.follow", "mutator.aln", "circle.numeric"];

// ---------------------------------------------------------------------------------------------------------------
// What is drawn
// ---------------------------------------------------------------------------------------------------------------

test("every control in the inventory is drawn once or named as not drawn, with the reason", () => {
  const inventory = controls.controls.map((c) => c.id);
  const drawn = g.placements.filter((p) => !p.inert).map((p) => p.id);
  assert.equal(new Set(drawn).size, drawn.length, "each control once");
  assert.equal(inventory.length, 247);
  const file = layoutFile;
  const unplaced = file.unplaced.map((u) => u.id);
  for (const id of unplaced) assert.ok(UNPLACED.includes(id), `${id} is not drawn, and is not on the list that may be`);
  for (const u of file.unplaced) assert.ok(u.why.length > 20, `${u.id} says why`);
  assert.deepEqual([...drawn, ...unplaced].sort(), [...inventory].sort(), "every control is drawn or named");
  assert.equal(drawn.length, 242);
});

test("the plate has keys that are in no inventory: they are drawn, they are not buttons, and none is a control number", () => {
  const inert = g.placements.filter((p) => p.inert);
  assert.equal(inert.length, 37);
  for (const p of inert) {
    assert.match(p.id, /^hw\./);
    assert.equal(p.n, -1, `${p.id} is no control`);
    assert.ok(!controls.controls.some((c) => c.id === p.id));
  }
  assert.ok(!tabOrder(g).some((id) => id.startsWith("hw.")), "an inert key is not a tab stop");
});

test("the matrix is sixteen columns by ten rows, a row is 1.495 column pitches, and row 0 is at the bottom", () => {
  const m = ids("matrix.");
  assert.equal(m.length, 160);
  const key = (r: number, c: number) => byId(`matrix.r${r}.c${c}`);
  assert.ok(Math.abs(key(0, 2).cx - key(0, 1).cx - pitch) < 1e-9, "a column is one pitch");
  const row = key(8, 1).cy - key(9, 1).cy;
  assert.ok(Math.abs(row / pitch - measured.matrix.row_over_col) < 0.01, `a row is ${(row / pitch).toFixed(4)} pitches, the photograph measures ${measured.matrix.row_over_col}`);
  assert.ok(key(0, 1).cy > key(9, 1).cy, "row 0 is below row 9");
  for (let r = 0; r < 10; r++) for (let c = 1; c <= 16; c++) assert.ok(Math.abs(key(r, c).cx - (key(0, 1).cx + (c - 1) * pitch)) < 1e-9, `r${r}.c${c} is on its column`);
});

test("each position is where the photograph measures it: every drawn key and knob is within a tenth of a pitch of one measured point, and no point serves two", () => {
  const minX = Math.min(...measured.plate_u.map((p) => p[0]));
  const minY = Math.min(...measured.plate_u.map((p) => p[1]));
  const taken = new Set<number>();
  const tol = 0.1;
  for (const p of g.placements) {
    const kind = p.kind;
    const x = p.cx / pitch;
    const y = p.cy / pitch;
    let best = -1;
    let bestD = Infinity;
    measured.found.forEach((f, i) => {
      if (f.kind !== kind) return;
      const d = Math.hypot(f.u[0] - minX - x, f.u[1] - minY - y);
      if (d < bestD) {
        bestD = d;
        best = i;
      }
    });
    assert.ok(bestD <= tol, `${p.id} is ${bestD.toFixed(2)} pitches from the nearest measured ${kind}`);
    assert.ok(!taken.has(best), `${p.id} shares a measured point with another`);
    taken.add(best);
  }
  assert.equal(taken.size, g.placements.length);
});

// ---------------------------------------------------------------------------------------------------------------
// Where the blocks are (the things a player's hands know)
// ---------------------------------------------------------------------------------------------------------------

const xs = (ps: Placement[]) => ps.map((p) => p.cx);
const ys = (ps: Placement[]) => ps.map((p) => p.cy);

test("left to right: the MIX knobs, the SEL column, the matrix, the mutator column, the EDIT knobs, then the circle", () => {
  const mix = ids("mix.enc.");
  const sel = ids("selector.");
  const matrix = ids("matrix.");
  const mut = ids("mutator.").filter((p) => p.id !== "mutator.mute");
  const edit = ids("edit.enc.");
  const circle = g.placements.filter((p) => /^(circle|chord|transport|mode)\./.test(p.id) || p.id === "keys.program" || p.id.startsWith("hw.num.") || p.id.startsWith("hw.ring."));
  assert.equal(mix.length, 10);
  assert.equal(sel.length, 10);
  assert.equal(mut.length, 9, "nine of the mutators are in the column (the tenth, ALN, is not drawn)");
  assert.equal(edit.length, 10);
  assert.ok(Math.max(...xs(mix)) < Math.min(...xs(sel)));
  assert.ok(Math.max(...xs(sel)) < Math.min(...xs(matrix)));
  assert.ok(Math.max(...xs(matrix)) < Math.min(...xs(mut)));
  assert.ok(Math.max(...xs(mut)) < Math.min(...xs(edit)));
  assert.ok(Math.max(...xs(edit)) < Math.min(...xs(circle)));
});

test("the MIX knobs, the SEL keys, the mutator column and the EDIT knobs are on the matrix's rows", () => {
  for (let r = 0; r < 10; r++) {
    const row = byId(`matrix.r${r}.c1`).cy;
    for (const id of [`mix.enc.r${r}`, `selector.r${r}`]) assert.ok(Math.abs(byId(id).cy - row) < pitch * 0.1, `${id} is on matrix row ${r}`);
  }
  const rows = Array.from({ length: 10 }, (_, r) => byId(`matrix.r${9 - r}.c1`).cy);
  const mut = ["tgl", "sol", "clr", "rnd", "flt", "zom", "rmx", "cpy", "pst", "eff"].map((n) => g.placements.find((p) => p.id === `mutator.${n}`));
  mut.forEach((p, i) => i < 10 && p && assert.ok(Math.abs(p.cy - rows[i]!) < pitch * 0.1, `${p.id} is on row ${9 - i}`));
  const edit = ["vel", "pit", "len", "sta", "pos", "dir", "amt", "grv", "mcc", "mch"].map((n) => byId(`edit.enc.${n}`));
  edit.forEach((p, i) => assert.ok(Math.abs(p.cy - rows[i]!) < pitch * 0.1, `${p.id} is on row ${9 - i}`));
});

test("the EDIT knobs read VEL PIT LEN STA POS DIR AMT GRV MCC MCH from the top, and bow towards the matrix in the middle", () => {
  const order = ["vel", "pit", "len", "sta", "pos", "dir", "amt", "grv", "mcc", "mch"].map((n) => byId(`edit.enc.${n}`));
  for (let i = 1; i < order.length; i++) assert.ok(order[i]!.cy > order[i - 1]!.cy, `${order[i]!.id} is below ${order[i - 1]!.id}`);
  const top = order[0]!.cx;
  const middle = Math.min(...order.slice(3, 7).map((p) => p.cx));
  assert.ok(top - middle > pitch, `the column bows by ${((top - middle) / pitch).toFixed(2)} pitches (the photograph: about 1.5)`);
  assert.ok(order[9]!.cx - middle > pitch, "and comes back at the bottom");
});

test("under the matrix: the MIX key, the SEL key, the eleven MIX TARGET keys, the MUT key, the EDIT key and ESC, in that order from the left", () => {
  const bottom = ["mix.master", "hw.sel", "hw.target.atr", "hw.target.vol", "hw.target.pan", "hw.target.mod", "hw.target.exp", "hw.target.map", "hw.target.1", "hw.target.2", "hw.target.3", "hw.target.4", "hw.target.5", "mutator.mute", "mode.edit", "keys.esc"];
  const row = bottom.map(byId);
  const lowest = Math.max(...ids("matrix.").map((p) => p.cy));
  for (const p of row) assert.ok(p.cy > lowest + pitch, `${p.id} is below the matrix`);
  for (let i = 1; i < row.length; i++) assert.ok(row[i]!.cx > row[i - 1]!.cx, `${row[i]!.id} is right of ${row[i - 1]!.id}`);
  assert.ok(Math.abs(byId("mix.master").cx - byId("mix.enc.r0").cx) < pitch * 0.3, "the MIX key is under the MIX knobs");
  assert.ok(Math.abs(byId("mutator.mute").cx - byId("mutator.tgl").cx) < pitch * 0.3, "the MUT key is under the mutator column");
});

test("the note ring is chromatic, clockwise from the C at the bottom left, round the top, to the C at the bottom right", () => {
  const ring = ["circle.note.k1", "circle.note.k2", "circle.note.k3", "circle.note.k4", "circle.note.k5", "circle.note.k6", "circle.note.k7", "circle.note.k8", "circle.note.k9", "circle.note.k10", "circle.note.k11", "circle.note.k12", "circle.upper_c"].map(byId);
  const cx = ring.reduce((a, p) => a + p.cx, 0) / ring.length;
  const cy = ring.reduce((a, p) => a + p.cy, 0) / ring.length;
  const radii = ring.map((p) => Math.hypot(p.cx - cx, p.cy - cy));
  const mean = radii.reduce((a, b) => a + b, 0) / radii.length;
  for (const [i, r] of radii.entries()) assert.ok(Math.abs(r - mean) < pitch * 0.5, `${ring[i]!.id} is on the ring`);
  // Screen angle, clockwise from straight up (y grows downwards).
  const angle = (p: Placement) => (Math.atan2(p.cx - cx, -(p.cy - cy)) + 2 * Math.PI) % (2 * Math.PI);
  const a = ring.map(angle);
  // From the bottom left (about 200 degrees) round through the top to the bottom right (about 160 degrees, 520 when unwrapped).
  const unwrapped = a.map((v, i) => (i === 0 ? v : v < a[0]! ? v + 2 * Math.PI : v));
  for (let i = 1; i < unwrapped.length; i++) assert.ok(unwrapped[i]! > unwrapped[i - 1]!, `${ring[i]!.id} follows ${ring[i - 1]!.id} clockwise`);
  assert.ok(byId("circle.note.k1").cy > cy && byId("circle.note.k1").cx < cx, "C is at the bottom left");
  assert.ok(byId("circle.upper_c").cy > cy && byId("circle.upper_c").cx > cx, "the upper C is at the bottom right");
  assert.ok(byId("circle.note.k7").cy < cy, "F sharp is at the top");
});

test("the scale keys MOD, SEL and CAD sit under the note ring, left to right", () => {
  const scale = ["circle.scale_mod", "circle.scale_sel", "circle.scale_cad"].map(byId);
  const lowestNote = Math.max(...ids("circle.note.").map((p) => p.cy));
  for (const p of scale) assert.ok(p.cy > lowestNote, `${p.id} is under the note ring`);
  assert.ok(scale[0]!.cx < scale[1]!.cx && scale[1]!.cx < scale[2]!.cx);
});

test("the MODE block is inside the note ring, and transport is round the bottom of the outer ring, left to right REC STOP PAUSE PLAY", () => {
  const notes = ids("circle.note.");
  const left = Math.min(...xs(notes));
  const right = Math.max(...xs(notes));
  for (const id of ["mode.grid", "mode.page", "mode.track", "mode.step", "mode.play"]) {
    const p = byId(id);
    assert.ok(p.cx > left && p.cx < right, `${id} is inside the note ring`);
  }
  const t = ["transport.rec", "transport.stop", "transport.pause", "transport.play"].map(byId);
  for (let i = 1; i < t.length; i++) assert.ok(t[i]!.cx > t[i - 1]!.cx, `${t[i]!.id} is right of ${t[i - 1]!.id}`);
  const lowest = Math.max(...ys([...notes, ...ids("circle.scale_")]));
  for (const p of t) assert.ok(p.cy > lowest + pitch, `${p.id} is below the circle`);
});

test("the seven chord keys run down the right-hand arm, CHORD1 at the top, to the right of the whole circle", () => {
  const chord = Array.from({ length: 7 }, (_, i) => byId(`chord.b${i + 1}`));
  for (let i = 1; i < chord.length; i++) assert.ok(chord[i]!.cy > chord[i - 1]!.cy, `${chord[i]!.id} is below ${chord[i - 1]!.id}`);
  const ring = [...ids("circle.note."), ...ids("mode.")];
  for (const p of chord) assert.ok(p.cx > Math.max(...xs(ring)), `${p.id} is right of the note ring`);
});

test("the main knob is at the top right of the plate, above everything else on that side", () => {
  const main = byId("main.enc");
  assert.equal(main.kind, "knob");
  assert.ok(main.cx > g.plate.w * pitch * 0.8, "right of the plate");
  assert.ok(main.cy < Math.min(...ys(ids("chord."))), "above the chord arm");
});

// ---------------------------------------------------------------------------------------------------------------
// Sizes: every length is a token's
// ---------------------------------------------------------------------------------------------------------------

test("a key is a round key of the key token, a knob of the knob token, and an LED dot of the dot token; the pitch is key plus gap", () => {
  const key = tokens.space["4"].value;
  const knob = tokens.space["5"].value;
  const dot = tokens.space["2"].value;
  assert.equal(pitch, key + tokens.space["3"].value);
  for (const p of g.placements) {
    assert.equal(p.r, p.kind === "knob" ? knob / 2 : key / 2, p.id);
    if (p.led) assert.equal(p.led.r, dot / 2, `${p.id}'s dot`);
  }
  assert.equal(byId("matrix.r4.c7").kind, "key");
  assert.equal(byId("mix.enc.r4").kind, "knob");
});

test("an LED dot sits beside its key on the side the plate puts it: above a matrix key, left of a mutator key, right of a SEL key", () => {
  const dot = (id: string) => {
    const p = byId(id);
    assert.ok(p.led, `${id} has a dot`);
    return { dx: p.led.cx - p.cx, dy: p.led.cy - p.cy };
  };
  const above = dot("matrix.r3.c5");
  assert.ok(above.dy < 0 && Math.abs(above.dx) < 1e-9, "above");
  assert.ok(Math.abs(Math.hypot(above.dx, above.dy) - layoutFile.led.distance * pitch) < 1e-9, "half a pitch from the key's centre");
  assert.ok(dot("selector.r3").dx > 0, "a SEL key's LED is to its right");
  assert.ok(dot("mutator.sol").dx < 0, "a mutator key's LED is to its left");
});

test("an LED is drawn for the controls the manual gives one, and a dead dot for the plate's keys; encoders have none", () => {
  for (const p of g.placements.filter((q) => !q.inert)) {
    const c = controls.controls.find((x) => x.id === p.id)!;
    assert.equal(p.led !== null, c.led === true, p.id);
    assert.equal(p.n, c.n);
    assert.equal(p.name, c.name, `${p.id} is named as the manual names it`);
  }
  for (const p of g.placements.filter((q) => q.kind === "knob")) assert.equal(p.led, null, p.id);
  assert.equal(byId("transport.stop").led, null);
});

test("no pixel is written in the layout: changing a token moves the panel, and nothing else does", () => {
  const bigger = structuredClone(tokens);
  bigger.space["4"].value = 40;
  const a = geometry(tokens);
  const b = geometry(bigger);
  const at = (x: Geometry, id: string) => x.placements.find((p) => p.id === id)!;
  assert.ok(at(b, "matrix.r0.c16").cx > at(a, "matrix.r0.c16").cx);
  assert.equal(at(b, "matrix.r9.c1").r, 20);
  assert.ok(b.width > a.width && b.height > a.height);
  // The proportions do not change: every position is the same number of pitches from the plate's corner.
  const pa = tokens.space["4"].value + tokens.space["3"].value;
  const pb = 40 + tokens.space["3"].value;
  for (const id of ["matrix.r0.c16", "chord.b7", "main.enc", "edit.enc.mch"]) {
    assert.ok(Math.abs(at(a, id).cx / pa - at(b, id).cx / pb) < 1e-9, `${id} x`);
    assert.ok(Math.abs(at(a, id).cy / pa - at(b, id).cy / pb) < 1e-9, `${id} y`);
  }
});

test("the layout file holds pitches and token names, and no number with a unit", () => {
  const text = JSON.stringify(layoutFile);
  assert.ok(!/\d+(\.\d+)?\s*(px|rem|em|pt)/.test(text), "no unit");
  for (const ref of Object.values(layoutFile.size)) assert.match(ref, /^space\.[0-7]$/, "a token name");
  for (const k of [...layoutFile.controls, ...layoutFile.keys, ...layoutFile.legends]) {
    assert.ok(Number.isFinite(k.x) && Number.isFinite(k.y) && k.x >= 0 && k.y >= 0, `${"id" in k ? k.id : k.text} is a position on the plate`);
  }
});

// ---------------------------------------------------------------------------------------------------------------
// Nothing overlaps, and everything is on the plate
// ---------------------------------------------------------------------------------------------------------------

interface Box {
  what: string;
  x0: number;
  y0: number;
  x1: number;
  y1: number;
}
const overlap = (a: Box, b: Box): boolean => a.x0 < b.x1 && b.x0 < a.x1 && a.y0 < b.y1 && b.y0 < a.y1;
const fontPx = tokens.type.size.label.value;
function boxOf(text: string, x: number, y: number, anchor: "start" | "middle" | "end", what: string): Box {
  const w = text.length * LABEL_CHARS_EM * fontPx;
  const x0 = anchor === "start" ? x : anchor === "end" ? x - w : x - w / 2;
  return { what, x0, x1: x0 + w, y0: y - fontPx * 0.6, y1: y + fontPx * 0.6 };
}

test("no two keys or knobs touch, and no LED dot touches a key or another dot", () => {
  const ps = g.placements;
  for (let i = 0; i < ps.length; i++) {
    for (let j = i + 1; j < ps.length; j++) {
      const a = ps[i]!;
      const b = ps[j]!;
      assert.ok(Math.hypot(a.cx - b.cx, a.cy - b.cy) >= a.r + b.r, `${a.id} touches ${b.id}`);
    }
  }
  for (const a of ps.filter((p) => p.led)) {
    for (const b of ps) {
      if (b !== a) assert.ok(Math.hypot(a.led!.cx - b.cx, a.led!.cy - b.cy) >= a.led!.r + b.r, `${a.id}'s dot touches ${b.id}`);
      if (b !== a && b.led) assert.ok(Math.hypot(a.led!.cx - b.led.cx, a.led!.cy - b.led.cy) >= a.led!.r + b.led.r, `${a.id}'s dot touches ${b.id}'s`);
    }
    assert.ok(Math.hypot(a.led!.cx - a.cx, a.led!.cy - a.cy) >= a.led!.r + a.r, `${a.id}'s dot touches its own key`);
  }
});

test("everything is on the plate, and the view box holds the plate and a margin", () => {
  const { w, h } = g.plate;
  for (const p of g.placements) {
    assert.ok(p.cx - p.r >= 0 && p.cy - p.r >= 0 && p.cx + p.r <= w * pitch && p.cy + p.r <= h * pitch, `${p.id} is off the plate`);
    if (p.led) assert.ok(p.led.cx - p.led.r >= 0 && p.led.cy - p.led.r >= 0 && p.led.cx + p.led.r <= w * pitch && p.led.cy + p.led.r <= h * pitch, `${p.id}'s dot is off the plate`);
  }
  const margin = tokens.space["4"].value;
  assert.deepEqual(g.viewBox, [-margin, -margin, w * pitch + 2 * margin, h * pitch + 2 * margin]);
  assert.equal(g.width, w * pitch);
  assert.equal(g.height, h * pitch);
});

test("a printed label does not run into a key, an LED dot, a knob, another label or a legend, and stays on the plate", () => {
  const boxes: Box[] = [];
  for (const p of g.placements) {
    boxes.push({ what: p.id, x0: p.cx - p.r, x1: p.cx + p.r, y0: p.cy - p.r, y1: p.cy + p.r });
    if (p.led) boxes.push({ what: `${p.id} dot`, x0: p.led.cx - p.led.r, x1: p.led.cx + p.led.r, y0: p.led.cy - p.led.r, y1: p.led.cy + p.led.r });
  }
  const texts: Box[] = [];
  for (const p of g.placements) if (p.label) texts.push(boxOf(p.label.text, p.label.x, p.label.y, p.label.anchor, `${p.id} label`));
  for (const l of g.legends) texts.push(boxOf(l.text, l.x, l.y, l.anchor, `legend ${l.text}`));
  for (const t of texts) {
    if (t.what.startsWith("legend") && /[\u{1d11e}\u{1d122}]/u.test(t.what)) continue; // a clef is a drawing, sized by its own token
    assert.ok(t.x0 >= 0 && t.x1 <= g.width && t.y0 >= 0 && t.y1 <= g.height, `${t.what} leaves the plate`);
    for (const b of boxes) assert.ok(!overlap(t, b) || b.what === t.what.replace(" label", ""), `${t.what} runs into ${b.what}`);
  }
  for (let i = 0; i < texts.length; i++) {
    for (let j = i + 1; j < texts.length; j++) assert.ok(!overlap(texts[i]!, texts[j]!), `${texts[i]!.what} runs into ${texts[j]!.what}`);
  }
});

test("the printed legends are the plate's own: the block names, the caption, the row numbers and the two clefs, none inside a key", () => {
  const text = g.legends.map((l) => l.text);
  for (const t of ["MIX", "SEL", "MUT", "EDIT", "CHORD", "MODE", "SCALE", "TRANSPORT", "MIX TARGET"]) assert.ok(text.includes(t), `${t}`);
  for (let r = 0; r < 10; r++) assert.ok(g.legends.some((l) => l.kind === "row" && l.text === String(r)), `row ${r}`);
  assert.equal(g.legends.filter((l) => l.kind === "clef").length, 2);
  const treble = g.legends.find((l) => l.text === "\u{1d11e}")!;
  const bass = g.legends.find((l) => l.text === "\u{1d122}")!;
  assert.ok(treble.y < bass.y, "the treble clef is above the bass clef");
  assert.ok(treble.x < byId("mix.enc.r9").cx && bass.x < byId("mix.enc.r0").cx, "the clefs are at the far left, beside the MIX knobs");
  const rowNumber = (r: number) => g.legends.find((l) => l.kind === "row" && l.text === String(r))!;
  for (let r = 0; r < 10; r++) assert.ok(Math.abs(rowNumber(r).y - byId(`matrix.r${r}.c1`).cy) < pitch * 0.3, `row ${r}'s number is on row ${r}`);
});

// ---------------------------------------------------------------------------------------------------------------
// Names and order
// ---------------------------------------------------------------------------------------------------------------

test("a printed label is the manual's own short name: it is in the control's name, or it is read from the plate and says so", () => {
  for (const e of layoutFile.controls) {
    if (!e.text) continue;
    const name = controls.controls.find((c) => c.id === e.id)!.name.toLowerCase();
    assert.ok(e.text_basis === "photo" || name.includes(e.text.toLowerCase()), `${e.id}: "${e.text}" is neither in "${name}" nor read from the plate`);
  }
  assert.equal(byId("mode.page").label!.text, "PAGE");
  assert.equal(byId("edit.enc.vel").label!.text, "VEL");
  assert.equal(byId("matrix.r4.c4").label, null, "a matrix key is named by its place");
});

test("reading order is the plate's: bands of one matrix row from the top, left to right in each band; inert keys are skipped", () => {
  const order = tabOrder(g);
  assert.equal(order.length, 242);
  assert.equal(order[0], "mix.enc.r9", "the top left knob first");
  const band = (p: Placement) => Math.round((p.cy - byId("matrix.r9.c1").cy) / (layoutFile.matrix.row_pitch * pitch));
  const at = order.map(byId);
  for (let i = 1; i < at.length; i++) {
    const a = at[i - 1]!;
    const b = at[i]!;
    assert.ok(band(b) > band(a) || (band(b) === band(a) && b.cx > a.cx), `${b.id} comes before ${a.id} on the plate`);
  }
  // Within the first band: the MIX knob, the SEL key, the sixteen matrix keys, the mutator key, the EDIT knob.
  assert.deepEqual(order.slice(0, 5), ["mix.enc.r9", "selector.r9", "matrix.r9.c1", "matrix.r9.c2", "matrix.r9.c3"]);
  assert.equal(order[18], "matrix.r9.c16");
});

test("parseLayout refuses what cannot be drawn", () => {
  const bad = (edit: (f: LayoutFile) => void): Error => {
    const f = structuredClone(layoutFile);
    edit(f);
    try {
      parseLayout(f, controls);
    } catch (e) {
      return e as Error;
    }
    throw new Error("the layout was accepted");
  };
  assert.match(bad((f) => ((f as { schema: string }).schema = "panel.layout/1")).message, /schema/);
  assert.match(bad((f) => (f.status = "final" as never)).message, /provisional/);
  assert.match(bad((f) => (f.controls[0]!.id = "no.such.control")).message, /no\.such\.control/);
  assert.match(bad((f) => f.controls.push({ ...f.controls[0]! })).message, /twice/);
  assert.match(bad((f) => f.controls.push({ id: "matrix.r0.c1", x: 1, y: 1, basis: "photo" })).message, /matrix/);
  assert.match(bad((f) => (f.size.key = "space.9")).message, /space\.9/);
  assert.match(bad((f) => (f.size.dot = "8px" as never)).message, /8px/);
  assert.match(bad((f) => (f.controls[1]!.x = -1)).message, /x/);
  assert.match(bad((f) => (f.controls[1]!.y = Number.NaN)).message, /y/);
  assert.match(bad((f) => (f.controls[1]!.basis = "guess" as never)).message, /basis/);
  assert.match(bad((f) => (f.controls.find((c) => c.led)!.led = "up" as never)).message, /led/);
  assert.match(bad((f) => (f.controls.find((c) => c.text)!.label = "left" as never)).message, /label/);
  assert.match(bad((f) => (f.controls.find((c) => c.id === "mode.page")!.text = "RENAMED")).message, /RENAMED/);
  assert.match(bad((f) => f.controls.splice(f.controls.findIndex((c) => c.id === "chord.b3"), 1)).message, /chord\.b3/);
  assert.match(bad((f) => f.unplaced.push({ id: "chord.b1", why: "it is drawn" })).message, /chord\.b1/);
  assert.match(bad((f) => (f.unplaced[0]!.why = "")).message, /why/);
  assert.match(bad((f) => f.keys.push({ ...f.keys[0]! })).message, /twice/);
  assert.match(bad((f) => (f.keys[0]!.id = "mode.page")).message, /hw\./);
});

test("the layout is marked provisional, says what 'order' means, and the identity of the keys it is least sure of is flagged", () => {
  assert.equal(layoutFile.status, "provisional");
  assert.match(layoutFile.note, /order/);
  assert.match(layoutFile.note, /photograph/);
  const order = layoutFile.controls.filter((c) => c.basis === "order").map((c) => c.id);
  assert.ok(order.includes("mutator.tgl") && order.includes("mutator.zom"), "the mutator column's order is a guess and says so");
  assert.ok(order.includes("chord.b1"), "so is the chord arm's");
  assert.ok(layoutFile.controls.filter((c) => c.basis === "photo").length >= 60);
});

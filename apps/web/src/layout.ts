// Where each control is drawn: the Octopus's plate, as layout/panel.layout.json holds it (WENGE-0019, specs/SPEC-0002/p7-faithful-panel.md).
// Positions in the file are pitches (one matrix column: the key size plus the gap); every length here is a pitch times a token (the key, the
// gap, the dot, the knob, the margin) or an inventory count, so no pixel is written in this file or in the panel code (rule S12).
import { spaceOf, type Tokens } from "./tokens.ts";

export type Side = "n" | "e" | "s" | "w";
const SIDES: readonly string[] = ["n", "e", "s", "w"];
export type Basis = "photo" | "position" | "order";
const BASES: readonly string[] = ["photo", "position", "order"];

/** The width of a printed character, in ems of the label size: what the label-clearance rules assume of the face. */
export const LABEL_CHARS_EM = 0.7;
/** The height of a line of printed text, in ems of the label size. */
const LINE_EM = 1.2;

export interface ControlEntry {
  n: number;
  id: string;
  kind: string;
  zone: string;
  name: string;
  at?: { col: number; row: number };
  led?: boolean;
}

export interface ControlsDoc {
  zones: { id: string; grid?: { cols: number; rows: number; row_zero: string } }[];
  controls: ControlEntry[];
}

export interface ControlPlace {
  id: string;
  x: number;
  y: number;
  /** The side the LED dot is on. Present when the plate has a dot beside the key (the manual may not give it an LED: it is then a dead dot). */
  led?: Side;
  /** The short printed name. It is in the control's name in controls.json, or it says `text_basis: "photo"`. */
  text?: string;
  text_basis?: "photo";
  /** The side the printed name is on. */
  label?: Side;
  basis: Basis;
}

export interface HwKey {
  id: string;
  x: number;
  y: number;
  led?: Side;
  text?: string;
  label?: Side;
  basis: "photo";
}

export interface Legend {
  text: string;
  x: number;
  y: number;
  kind: "section" | "caption" | "row" | "clef";
  anchor: "start" | "middle" | "end";
}

export interface LayoutFile {
  schema: "panel.layout/2";
  status: "provisional";
  note: string;
  source: { photograph: string; measurement: string; roll_degrees: number };
  size: { key: string; gap: string; margin: string; dot: string; knob: string };
  /** The ball inside a knob's nut, as a fraction of the knob. */
  knob: { ball: number };
  /** How far an LED dot is from its key's centre, in pitches. */
  led: { distance: number };
  /** The plate's size in pitches. `top_cropped` is how many pitches of the photographed plate's top (the maker's name and logo, not drawn) are cut off. */
  plate: { w: number; h: number; top_cropped: number };
  /** The top left key (row 9, column 1), and how many pitches a row is. */
  matrix: { x: number; y: number; row_pitch: number; led: Side };
  controls: ControlPlace[];
  keys: HwKey[];
  legends: Legend[];
  /** Controls in the inventory that the plate shows no place for. They are not drawn. */
  unplaced: { id: string; why: string }[];
}

export interface Dot {
  cx: number;
  cy: number;
  r: number;
  /** The engine's LED frame drives it. A dot on a key the manual gives no LED is dead until the engine has a use for it. */
  live: boolean;
}

export interface Label {
  text: string;
  x: number;
  y: number;
  anchor: "start" | "middle" | "end";
}

export interface Placement {
  id: string;
  /** The control number the engine knows it by. -1 for a key on the plate that is in no inventory. */
  n: number;
  /** The manual's own name, which is the accessible name. For a key in no inventory, what is printed on it. */
  name: string;
  kind: "key" | "knob";
  /** On the plate and in no inventory: drawn, not a button. */
  inert: boolean;
  cx: number;
  cy: number;
  /** A key's radius; a knob's outer radius (the nut's corners). */
  r: number;
  led: Dot | null;
  label: Label | null;
  /** A knob's nut: six corners. */
  nut: [number, number][] | null;
  /** A knob's ball. */
  ball: number | null;
}

export interface LegendPlacement {
  text: string;
  x: number;
  y: number;
  kind: Legend["kind"];
  anchor: Legend["anchor"];
}

export interface Geometry {
  width: number;
  height: number;
  pitch: number;
  /** The plate, from its top left corner. */
  plate: { w: number; h: number };
  /** x, y, width, height of the SVG view box, margin included. */
  viewBox: [number, number, number, number];
  /** In the plate's reading order: bands of one matrix row from the top, left to right in each. */
  placements: Placement[];
  legends: LegendPlacement[];
}

function bad(what: string): never {
  throw new Error(`panel.layout.json: ${what}`);
}

const isPos = (v: unknown): v is number => typeof v === "number" && Number.isFinite(v) && v >= 0;

function checkSide(v: unknown, what: string, field: string): void {
  if (v !== undefined && !SIDES.includes(v as string)) bad(`${what}: ${field} must be one of n, e, s, w, not ${JSON.stringify(v)}`);
}

/** Checks the file against the inventory and returns it typed. */
export function parseLayout(json: unknown, doc: ControlsDoc): LayoutFile {
  const f = json as LayoutFile;
  if (typeof f !== "object" || f === null || f.schema !== "panel.layout/2") bad('schema must be "panel.layout/2"');
  if (f.status !== "provisional") bad('status must be "provisional" until the Panelwright settles what the photograph does not');
  for (const k of ["key", "gap", "margin", "dot", "knob"] as const) {
    if (typeof f.size?.[k] !== "string" || !/^space\.[0-7]$/.test(f.size[k])) bad(`size.${k} must be a space token name (space.0 to space.7), not ${JSON.stringify(f.size?.[k])}`);
  }
  if (typeof f.knob?.ball !== "number" || !(f.knob.ball > 0 && f.knob.ball < 1)) bad("knob.ball must be a fraction between 0 and 1");
  if (typeof f.led?.distance !== "number" || !(f.led.distance > 0)) bad("led.distance must be positive");
  if (!isPos(f.plate?.w) || !isPos(f.plate?.h) || f.plate.w === 0 || f.plate.h === 0) bad("plate.w and plate.h must be positive pitches");
  if (!isPos(f.plate.top_cropped)) bad("plate.top_cropped must be a number of pitches, 0 or more");
  if (!isPos(f.matrix?.x) || !isPos(f.matrix?.y)) bad("matrix.x and matrix.y must be positions on the plate");
  if (typeof f.matrix?.row_pitch !== "number" || !(f.matrix.row_pitch > 0)) bad("matrix.row_pitch must be positive");
  if (!SIDES.includes(f.matrix?.led)) bad("matrix.led (the side of a matrix key's LED) must be one of n, e, s, w");
  if (!Array.isArray(f.controls) || !Array.isArray(f.keys) || !Array.isArray(f.legends) || !Array.isArray(f.unplaced)) bad("controls, keys, legends and unplaced must be lists");

  const known = new Map(doc.controls.map((c) => [c.id, c]));
  const seen = new Set<string>();
  for (const c of f.controls) {
    const entry = known.get(c.id);
    if (!entry) bad(`${c.id} is not in controls.json`);
    if (entry.zone === "matrix") bad(`${c.id} is a matrix key; the matrix is placed as a block, not key by key`);
    if (seen.has(c.id)) bad(`${c.id} is listed twice`);
    seen.add(c.id);
    if (!isPos(c.x)) bad(`${c.id}: x must be a position on the plate (a number of pitches, 0 or more), not ${c.x}`);
    if (!isPos(c.y)) bad(`${c.id}: y must be a position on the plate (a number of pitches, 0 or more), not ${c.y}`);
    if (!BASES.includes(c.basis)) bad(`${c.id}: basis must be one of ${BASES.join(", ")}, not ${JSON.stringify(c.basis)}`);
    checkSide(c.led, c.id, "led");
    checkSide(c.label, c.id, "label");
    if (entry.led === true && c.led === undefined) bad(`${c.id}: the manual gives this control an LED, so led (its side) is needed`);
    if (c.label !== undefined && c.text === undefined) bad(`${c.id}: a label side without a text`);
    if (c.text !== undefined && c.text_basis !== "photo" && !entry.name.toLowerCase().includes(c.text.toLowerCase())) {
      bad(`${c.id}: the printed name ${JSON.stringify(c.text)} is not in the manual's name ${JSON.stringify(entry.name)}, and is not marked as read from the plate (text_basis)`);
    }
  }
  const hw = new Set<string>();
  for (const k of f.keys) {
    if (typeof k.id !== "string" || !k.id.startsWith("hw.")) bad(`${k.id}: a key on the plate that is in no inventory has an id that starts with hw.`);
    if (hw.has(k.id)) bad(`${k.id} is listed twice`);
    hw.add(k.id);
    if (!isPos(k.x) || !isPos(k.y)) bad(`${k.id}: x and y must be positions on the plate`);
    checkSide(k.led, k.id, "led");
    checkSide(k.label, k.id, "label");
  }
  for (const l of f.legends) {
    if (typeof l.text !== "string" || l.text === "") bad("a legend needs its text");
    if (!isPos(l.x) || !isPos(l.y)) bad(`legend ${l.text}: x and y must be positions on the plate`);
    if (!["section", "caption", "row", "clef"].includes(l.kind)) bad(`legend ${l.text}: kind`);
    if (!["start", "middle", "end"].includes(l.anchor)) bad(`legend ${l.text}: anchor`);
  }
  const unplaced = new Set<string>();
  for (const u of f.unplaced) {
    if (!known.has(u.id)) bad(`${u.id} (unplaced) is not in controls.json`);
    if (seen.has(u.id)) bad(`${u.id} is both placed and unplaced`);
    if (typeof u.why !== "string" || u.why.trim() === "") bad(`${u.id}: unplaced, and no why`);
    unplaced.add(u.id);
  }
  for (const c of doc.controls) {
    if (c.zone !== "matrix" && !seen.has(c.id) && !unplaced.has(c.id)) bad(`${c.id} is in controls.json and is neither placed nor listed under unplaced`);
  }
  return f;
}

function labelAt(text: string, side: Side, cx: number, cy: number, r: number, gap: number, font: number): Label {
  const half = (font * LINE_EM) / 2;
  switch (side) {
    case "e":
      return { text, x: cx + r + gap, y: cy, anchor: "start" };
    case "w":
      return { text, x: cx - r - gap, y: cy, anchor: "end" };
    case "n":
      return { text, x: cx, y: cy - r - gap - half, anchor: "middle" };
    case "s":
      return { text, x: cx, y: cy + r + gap + half, anchor: "middle" };
  }
}

function dotAt(side: Side, cx: number, cy: number, distance: number, r: number, live: boolean): Dot {
  const dx = side === "e" ? distance : side === "w" ? -distance : 0;
  const dy = side === "s" ? distance : side === "n" ? -distance : 0;
  return { cx: cx + dx, cy: cy + dy, r, live };
}

/** The six corners of a nut of circumradius `r`, one corner pointing right. */
function hexagon(cx: number, cy: number, r: number): [number, number][] {
  const corners = 6;
  return Array.from({ length: corners }, (_, k) => {
    const a = (k * 2 * Math.PI) / corners;
    return [cx + r * Math.cos(a), cy + r * Math.sin(a)];
  });
}

export function place(file: LayoutFile, doc: ControlsDoc, tokens: Tokens): Geometry {
  const key = spaceOf(tokens, file.size.key);
  const gap = spaceOf(tokens, file.size.gap);
  const margin = spaceOf(tokens, file.size.margin);
  const dot = spaceOf(tokens, file.size.dot);
  const knob = spaceOf(tokens, file.size.knob);
  const near = spaceOf(tokens, "space.1");
  const font = tokens.type.size.label.value;
  const pitch = key + gap;
  const rows = doc.zones.find((z) => z.id === "matrix")?.grid?.rows;
  if (!rows) throw new Error("controls.json has no matrix grid");

  const placements: Placement[] = [];
  const keyOf = (id: string, n: number, name: string, inert: boolean, x: number, y: number, led: Side | undefined, live: boolean): Placement => {
    const cx = x * pitch;
    const cy = y * pitch;
    return { id, n, name, kind: "key", inert, cx, cy, r: key / 2, led: led ? dotAt(led, cx, cy, file.led.distance * pitch, dot / 2, live) : null, label: null, nut: null, ball: null };
  };
  const matrix = doc.controls.filter((c) => c.zone === "matrix" && c.at);
  for (const c of matrix) {
    const x = file.matrix.x + c.at!.col - 1;
    const y = file.matrix.y + (rows - 1 - c.at!.row) * file.matrix.row_pitch;
    placements.push(keyOf(c.id, c.n, c.name, false, x, y, file.matrix.led, c.led === true));
  }
  const byId = new Map(doc.controls.map((c) => [c.id, c]));
  for (const c of file.controls) {
    const entry = byId.get(c.id)!;
    const isKnob = entry.kind === "encoder";
    let p: Placement;
    if (isKnob) {
      const cx = c.x * pitch;
      const cy = c.y * pitch;
      p = { id: c.id, n: entry.n, name: entry.name, kind: "knob", inert: false, cx, cy, r: knob / 2, led: null, label: null, nut: hexagon(cx, cy, knob / 2), ball: (knob / 2) * file.knob.ball };
    } else {
      p = keyOf(c.id, entry.n, entry.name, false, c.x, c.y, c.led, entry.led === true);
    }
    if (c.text && c.label) p.label = labelAt(c.text, c.label, p.cx, p.cy, p.r, near, font);
    placements.push(p);
  }
  for (const k of file.keys) {
    const p = keyOf(k.id, -1, k.text ?? k.id, true, k.x, k.y, k.led, false);
    if (k.text && k.label) p.label = labelAt(k.text, k.label, p.cx, p.cy, p.r, near, font);
    placements.push(p);
  }

  // Reading order: bands of one matrix row from the top, left to right in each band.
  const top = file.matrix.y * pitch;
  const band = (p: Placement): number => Math.round((p.cy - top) / (file.matrix.row_pitch * pitch));
  placements.sort((a, b) => band(a) - band(b) || a.cx - b.cx || a.cy - b.cy || (a.id < b.id ? -1 : 1));

  const width = file.plate.w * pitch;
  const height = file.plate.h * pitch;
  const legends = file.legends.map((l) => ({ text: l.text, x: l.x * pitch, y: l.y * pitch, kind: l.kind, anchor: l.anchor }));
  return { width, height, pitch, plate: { w: file.plate.w, h: file.plate.h }, viewBox: [-margin, -margin, width + 2 * margin, height + 2 * margin], placements, legends };
}

/** The tab stops: every control that is a button or a knob, in the plate's reading order. The plate's own keys are not stops. */
export function tabOrder(g: Geometry): string[] {
  return g.placements.filter((p) => !p.inert).map((p) => p.id);
}

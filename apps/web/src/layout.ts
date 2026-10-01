// Where each control is drawn. The matrix goes where the manual puts it; the other controls the controller acts on go
// where layout/panel.layout.json puts them, in cells. Every length here comes from a token (the key size, the gap, the
// margin) or from the inventory (columns and rows), so no pixel is written in this file or in the panel code (rule S12).
import { spaceOf, type Tokens } from "./tokens.ts";

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

export interface LayoutFile {
  schema: "panel.layout/1";
  status: "provisional";
  note: string;
  size: { key: string; gap: string; margin: string };
  label: { span: number; chars: number; leading_em: number };
  matrix: { col: number; row: number };
  controls: { id: string; col: number; row: number }[];
}

export interface Label {
  x: number;
  y: number;
  lines: string[];
  leadingEm: number;
}

export interface Placement {
  id: string;
  n: number;
  name: string;
  cx: number;
  cy: number;
  r: number;
  led: boolean;
  /** The manual's name in words, beside the key; null for the matrix, whose keys are named by their place. */
  label: Label | null;
}

export interface Geometry {
  width: number;
  height: number;
  /** x, y, width, height of the SVG view box, margin included. */
  viewBox: [number, number, number, number];
  /** In tab order: the matrix in reading order, then the other controls in the layout file's order. */
  placements: Placement[];
}

function bad(what: string): never {
  throw new Error(`panel.layout.json: ${what}`);
}

const isCell = (v: unknown): v is number => typeof v === "number" && Number.isInteger(v) && v >= 0;

/** Checks the file against the inventory and returns it typed. */
export function parseLayout(json: unknown, doc: ControlsDoc): LayoutFile {
  const f = json as LayoutFile;
  if (typeof f !== "object" || f === null || f.schema !== "panel.layout/1") bad('schema must be "panel.layout/1"');
  if (f.status !== "provisional") bad('status must be "provisional" until the Panelwright settles the zones the manual leaves open');
  for (const k of ["key", "gap", "margin"] as const) {
    if (typeof f.size?.[k] !== "string" || !/^space\.[0-7]$/.test(f.size[k])) bad(`size.${k} must be a space token name (space.0 to space.7), not ${JSON.stringify(f.size?.[k])}`);
  }
  if (!isCell(f.label?.span) || f.label.span < 1) bad("label.span must be a whole number of cells, at least 1");
  if (!isCell(f.label?.chars) || f.label.chars < 4) bad("label.chars must be at least 4");
  if (typeof f.label?.leading_em !== "number" || f.label.leading_em <= 0) bad("label.leading_em must be positive");
  if (!isCell(f.matrix?.col) || !isCell(f.matrix?.row)) bad("matrix.col and matrix.row must be whole cells");
  if (!Array.isArray(f.controls)) bad("controls must be a list");
  const known = new Map(doc.controls.map((c) => [c.id, c]));
  const seen = new Set<string>();
  for (const c of f.controls) {
    const entry = known.get(c.id);
    if (!entry) bad(`${c.id} is not in controls.json`);
    if (entry.zone === "matrix") bad(`${c.id} is a matrix key; the matrix is placed as a block, not key by key`);
    if (seen.has(c.id)) bad(`${c.id} is listed twice`);
    seen.add(c.id);
    if (!isCell(c.col)) bad(`${c.id}: col must be a whole number of cells, not ${c.col}`);
    if (!isCell(c.row)) bad(`${c.id}: row must be a whole number of cells, not ${c.row}`);
  }
  return f;
}

/** Breaks a name on spaces into lines of at most `chars` characters. A word longer than that stays whole. */
export function wrapLabel(text: string, chars: number): string[] {
  const lines: string[] = [];
  let line = "";
  for (const word of text.split(" ")) {
    if (line === "") line = word;
    else if (line.length + 1 + word.length <= chars) line += ` ${word}`;
    else {
      lines.push(line);
      line = word;
    }
  }
  if (line !== "") lines.push(line);
  return lines;
}

export function place(file: LayoutFile, doc: ControlsDoc, tokens: Tokens): Geometry {
  const key = spaceOf(tokens, file.size.key);
  const gap = spaceOf(tokens, file.size.gap);
  const margin = spaceOf(tokens, file.size.margin);
  const pitch = key + gap;
  const r = key / 2;
  const zone = doc.zones.find((z) => z.id === "matrix");
  const rows = zone?.grid?.rows;
  if (!rows) throw new Error("controls.json has no matrix grid");

  const placements: Placement[] = [];
  const matrix = doc.controls.filter((c) => c.zone === "matrix" && c.at);
  // Row 0 is at the bottom (the zone says so), so reading order starts at the highest row.
  matrix.sort((a, b) => b.at!.row - a.at!.row || a.at!.col - b.at!.col);
  let cols = 0;
  let last = 0;
  for (const c of matrix) {
    const col = file.matrix.col + c.at!.col - 1;
    const row = file.matrix.row + (rows - 1 - c.at!.row);
    placements.push({ id: c.id, n: c.n, name: c.name, cx: col * pitch + r, cy: row * pitch + r, r, led: c.led === true, label: null });
    cols = Math.max(cols, col + 1);
    last = Math.max(last, row + 1);
  }
  const byId = new Map(doc.controls.map((c) => [c.id, c]));
  for (const c of file.controls) {
    const entry = byId.get(c.id)!;
    const cx = c.col * pitch + r;
    const cy = c.row * pitch + r;
    placements.push({
      id: entry.id,
      n: entry.n,
      name: entry.name,
      cx,
      cy,
      r,
      led: entry.led === true,
      label: { x: cx + r + gap, y: cy, lines: wrapLabel(entry.name, file.label.chars), leadingEm: file.label.leading_em },
    });
    cols = Math.max(cols, c.col + 1 + file.label.span);
    last = Math.max(last, c.row + 1);
  }
  const width = cols * pitch - gap;
  const height = last * pitch - gap;
  return { width, height, viewBox: [-margin, -margin, width + 2 * margin, height + 2 * margin], placements };
}

export function tabOrder(g: Geometry): string[] {
  return g.placements.map((p) => p.id);
}

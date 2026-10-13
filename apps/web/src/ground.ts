// The ground's line drawings (ADR-0011 decision 10): three small hairline drawings in the page ground above the case, in the language of technical
// drawing (strokes with no fill, dotted construction lines, geometric solids), and the only decoration in the product. They are drawn from the space
// scale and nothing else, so no length is written in this file (rule S12): every coordinate is a space token, a sum of them, or half of one. Like the
// panel they are one SVG that scales as one object; what they look like (the control-edge tone, a hairline that stays a hairline, dots) is the
// stylesheet's, under #ground.
import { SPACE_KEYS, type Tokens } from "./tokens.ts";

const SVG = "http://www.w3.org/2000/svg";

type Attrs = Record<string, string | number>;
type Point = readonly [number, number];

function svg(doc: Document, name: string, attrs: Attrs): SVGElement {
  const el = doc.createElementNS(SVG, name);
  for (const [k, v] of Object.entries(attrs)) el.setAttribute(k, String(v));
  return el;
}

/** One drawing: a group, and the two kinds of stroke it is made of, solid lines (`line`) and dotted construction lines (`dots`). */
class Pen {
  readonly g: SVGElement;
  private readonly doc: Document;
  constructor(doc: Document, name: string, x: number, y: number) {
    this.doc = doc;
    this.g = svg(doc, "g", { class: `drawing ${name}`, transform: `translate(${x} ${y})` });
  }
  private add(kind: "line" | "dots", shape: string, attrs: Attrs): void {
    this.g.append(svg(this.doc, shape, { class: kind, ...attrs }));
  }
  line(kind: "line" | "dots", [x1, y1]: Point, [x2, y2]: Point): void {
    this.add(kind, "line", { x1, y1, x2, y2 });
  }
  circle(kind: "line" | "dots", cx: number, cy: number, r: number): void {
    this.add(kind, "circle", { cx, cy, r });
  }
  /** An ellipse as two arcs, `across` the half width and `down` the half height. (Not an <ellipse>: rule S15 reads the attribute names it uses for a rounded rectangle.) */
  ellipse(cx: number, cy: number, across: number, down: number): void {
    const arc = `A ${across} ${down} 0 1 0`;
    this.add("line", "path", { d: `M ${cx - across} ${cy} ${arc} ${cx + across} ${cy} ${arc} ${cx - across} ${cy} Z` });
  }
  outline(points: readonly Point[]): void {
    this.add("line", "polygon", { points: points.map(([x, y]) => `${x},${y}`).join(" ") });
  }
}

/**
 * The ground: one SVG with a view box of its own and three drawings in a row. `preserveAspectRatio` keeps the row whole and centred in whatever room
 * the case leaves; the stylesheet gives it that room and no more.
 */
export function drawGround(doc: Document, tokens: Tokens): SVGSVGElement {
  // The scale, from nothing to the largest: 0, then 4, 8, 12, 20, 32, 52 and 84 as the tokens stand. The names are the steps, not the values.
  const [zero, s1, s2, s3, s4, s5, s6, s7] = SPACE_KEYS.map((k) => tokens.space[k].value) as [number, number, number, number, number, number, number, number];

  // The detent: rings whose radii are the scale (so the gaps widen as the scale does), a dotted ring just outside them, and a dotted crosshair.
  const reach = s7 + s2 + s3; // how far the crosshair runs from the centre
  const detentSize = reach + reach;

  // The stack: four oblique planes, one to a track, joined at their corners by dotted verticals. A plane is a parallelogram `wide` across, its far
  // edge `back` further along and `rise` higher.
  const wide = s7;
  const back = s5;
  const rise = s3;
  const layers = [zero, s4, s4 + s4, s4 + s4 + s4] as const;
  const [firstLayer, , , lastLayer] = layers;
  const stackWidth = wide + back;
  const stackHeight = rise + lastLayer;

  // The orbit: two ellipses on dotted axes, and a small circle on the outer one where the minor axis meets it.
  const orbitHalfWidth = s7 + s3;
  const orbitHalfHeight = s5 + s3;
  const orbitWidth = orbitHalfWidth + orbitHalfWidth;
  const orbitHeight = orbitHalfHeight + orbitHalfHeight;

  // The row: as tall as the tallest drawing, each centred in it, a gap of the sixth step between them, a margin of the fourth step round it all.
  const gap = s6;
  const width = detentSize + gap + stackWidth + gap + orbitWidth;
  const height = detentSize;
  const stackAt = detentSize + gap;
  const orbitAt = stackAt + stackWidth + gap;

  const detent = new Pen(doc, "detent", zero, zero);
  for (const r of [s2, s3, s4, s5, s6, s7]) detent.circle("line", reach, reach, r);
  detent.circle("dots", reach, reach, s7 + s2);
  detent.line("dots", [zero, reach], [detentSize, reach]);
  detent.line("dots", [reach, zero], [reach, detentSize]);

  const stack = new Pen(doc, "stack", stackAt, (height - stackHeight) / 2);
  const plane = (y: number): readonly Point[] => [
    [zero, y],
    [wide, y],
    [wide + back, y - rise],
    [back, y - rise],
  ];
  const top = plane(rise + firstLayer);
  const bottom = plane(rise + lastLayer);
  top.forEach((p, i) => stack.line("dots", p, bottom[i] as Point));
  for (const y of layers) stack.outline(plane(rise + y));

  const orbit = new Pen(doc, "orbit", orbitAt, (height - orbitHeight) / 2);
  orbit.ellipse(orbitHalfWidth, orbitHalfHeight, s7, s5);
  orbit.ellipse(orbitHalfWidth, orbitHalfHeight, s5 + s4, s3);
  orbit.line("dots", [zero, orbitHalfHeight], [orbitWidth, orbitHalfHeight]);
  orbit.line("dots", [orbitHalfWidth, zero], [orbitHalfWidth, orbitHeight]);
  orbit.circle("line", orbitHalfWidth, orbitHalfHeight - s5, s2);

  const root = svg(doc, "svg", {
    id: "ground",
    viewBox: [-s4, -s4, width + s4 + s4, height + s4 + s4].join(" "),
    preserveAspectRatio: "xMidYMid meet",
    "aria-hidden": "true",
    focusable: "false",
  }) as SVGSVGElement;
  root.append(detent.g, stack.g, orbit.g);
  return root;
}

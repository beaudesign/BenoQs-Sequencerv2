// The panel: one SVG, drawn from a geometry (layout.ts) and the tokens: the plate, its printed legends, and each control a labelled button or knob.
// It holds no sequencing logic: a press goes out through `Presses`, a turn through the `turn` callback, and the LED frame comes in through `paintLeds`.
//
// No colour, length or type size is written here. Colours, strokes and fonts are the stylesheet's, from the tokens; the only lengths are the ones
// layout.ts computes from the tokens and the two ring radii below, which come from the same key size and the two stroke tokens.
import { decodeLed } from "./abi.ts";
import type { Geometry, LegendPlacement, Placement } from "./layout.ts";
import type { Presses } from "./press.ts";
import type { Tokens } from "./tokens.ts";

const SVG = "http://www.w3.org/2000/svg";

function svg<K extends keyof SVGElementTagNameMap>(doc: Document, name: K, attrs: Record<string, string | number> = {}): SVGElementTagNameMap[K] {
  const el = doc.createElementNS(SVG, name);
  for (const [k, v] of Object.entries(attrs)) el.setAttribute(k, String(v));
  return el;
}

export interface Panel {
  readonly svg: SVGSVGElement;
  /** Width over height of the whole case: the frame, and the plate inside it. The page sizes the case by this and by nothing else. */
  readonly aspect: number;
  /** Applies an LED frame (one byte per control number). Only what changed is touched. */
  paintLeds(frame: Uint8Array): void;
}

/** A knob has been turned by `detents` clicks: positive is clockwise. */
export type Turn = (control: number, detents: number) => void;

const SPOKEN: Record<string, string> = { steady: "lit", flash: "flashing", shine: "shining" };

interface LedRef {
  el: SVGElement;
  group: Element;
  last: number;
  id: string;
}

export function drawPanel(doc: Document, geometry: Geometry, tokens: Tokens, presses: Presses, turn: Turn): Panel {
  const root = svg(doc, "svg", { id: "panel", viewBox: geometry.viewBox.join(" "), role: "group", "aria-label": "Octopus panel" });
  const leds = new Map<number, LedRef>();

  // The case: the frame is the first thing drawn and fills the view box, which is the plate and the margin round it, so the case is exactly as big as the
  // panel and the page shows round it (ADR-0011 decision 4).
  const [frameX, frameY, frameW, frameH] = geometry.viewBox;
  root.append(svg(doc, "rect", { class: "frame", x: frameX, y: frameY, width: frameW, height: frameH }));
  root.append(svg(doc, "rect", { class: "plate", x: 0, y: 0, width: geometry.width, height: geometry.height }));
  for (const l of geometry.legends) root.append(drawLegend(doc, l));
  for (const p of geometry.placements) root.append(p.inert ? drawInert(doc, p) : drawControl(doc, p, tokens, presses, turn, leds));

  return {
    svg: root,
    aspect: frameW / frameH,
    paintLeds(frame) {
      let flashChanged = false;
      for (const [n, led] of leds) {
        const byte = frame[n] ?? 0;
        if (byte === led.last) continue;
        led.last = byte;
        const { colour, phase } = decodeLed(byte);
        led.el.setAttribute("data-colour", colour);
        led.el.setAttribute("data-phase", phase);
        // Colour is not the only carrier: the state is also said in words (docs/05 section 5).
        if (colour === "off") led.group.removeAttribute("aria-description");
        else led.group.setAttribute("aria-description", `${SPOKEN[phase]} ${colour}`);
        if (phase === "flash") flashChanged = true;
      }
      // Every flashing LED starts its animation when its own state changes. Pin them all to the page's clock
      // so that LEDs that flash, flash together, as the hardware's do.
      if (flashChanged) {
        for (const a of root.getAnimations({ subtree: true })) if (a.startTime !== 0) a.startTime = 0;
      }
    },
  };
}

function drawLegend(doc: Document, l: LegendPlacement): SVGTextElement {
  const text = svg(doc, "text", { class: `legend ${l.kind}`, x: l.x, y: l.y, "text-anchor": l.anchor });
  text.textContent = l.text;
  return text;
}

function drawLabel(doc: Document, p: Placement): SVGTextElement | null {
  if (!p.label) return null;
  const text = svg(doc, "text", { class: "label", x: p.label.x, y: p.label.y, "text-anchor": p.label.anchor });
  text.textContent = p.label.text;
  return text;
}

/** The key's face and its LED dot. A control the engine can light has a dot the LED frame drives; any other dot is dead. */
function drawFace(doc: Document, p: Placement): SVGElement[] {
  const face = svg(doc, "circle", { class: "face", cx: p.cx, cy: p.cy, r: p.r });
  if (!p.led) return [face];
  const dot = svg(doc, "circle", { class: p.led.live ? "led" : "dot", cx: p.led.cx, cy: p.led.cy, r: p.led.r });
  return [face, dot];
}

/** A key on the plate that is in no inventory: drawn, and nothing more. It is not a button, not a tab stop, and says nothing to the engine. */
function drawInert(doc: Document, p: Placement): SVGGElement {
  const g = svg(doc, "g", { class: "key inert", "data-id": p.id, "aria-hidden": "true" });
  g.append(...drawFace(doc, p));
  const label = drawLabel(doc, p);
  if (label) g.append(label);
  return g;
}

function drawControl(doc: Document, p: Placement, tokens: Tokens, presses: Presses, turn: Turn, leds: Map<number, LedRef>): SVGGElement {
  const g = svg(doc, "g", { class: p.kind === "knob" ? "control knob" : "control", "data-id": p.id, "data-n": p.n, role: "button", tabindex: 0, "aria-label": p.name });
  if (p.kind === "knob") {
    g.setAttribute("aria-roledescription", "knob");
    g.setAttribute("aria-description", "Turn with the arrow keys or the mouse wheel");
    g.append(svg(doc, "polygon", { class: "nut", points: p.nut!.map(([x, y]) => `${x},${y}`).join(" ") }), svg(doc, "circle", { class: "ball", cx: p.cx, cy: p.cy, r: p.ball! }));
  } else {
    const [face, dot] = drawFace(doc, p);
    g.append(face!);
    if (dot) {
      g.append(dot);
      if (p.led!.live) {
        dot.setAttribute("data-colour", "off");
        dot.setAttribute("data-phase", "steady");
        leds.set(p.n, { el: dot, group: g, last: 0, id: p.id });
      }
    }
  }
  // The focus ring is two strokes, so that it shows on the ground and on a lit LED (docs/05 section 5): a light one
  // just outside the key and a dark one just inside its edge.
  g.append(
    svg(doc, "circle", { class: "ring-inner", cx: p.cx, cy: p.cy, r: p.r - tokens.stroke.focus_inner.value / 2 }),
    svg(doc, "circle", { class: "ring-outer", cx: p.cx, cy: p.cy, r: p.r + tokens.stroke.focus.value / 2 }),
  );
  const label = drawLabel(doc, p);
  if (label) g.append(label);

  if (p.kind === "knob") {
    wireKnob(g, p, turn);
    return g;
  }

  const source = (e: PointerEvent): string => `pointer ${e.pointerId}`;
  g.addEventListener("pointerdown", (e) => {
    if (e.pointerType === "mouse" && e.button !== 0) return;
    g.setPointerCapture(e.pointerId);
    presses.down(p.n, source(e));
    g.toggleAttribute("data-down", presses.isHeld(p.n));
  });
  const lift = (e: PointerEvent): void => {
    presses.up(p.n, source(e));
    g.toggleAttribute("data-down", presses.isHeld(p.n));
  };
  g.addEventListener("pointerup", lift);
  g.addEventListener("pointercancel", lift);
  g.addEventListener("lostpointercapture", lift);

  // A click no pointer made (detail 0) is a screen reader's Activate or a voice command: one press and one release.
  // A pointer's own click has a detail of 1 or more and has already been through pointerdown and pointerup.
  g.addEventListener("click", (e) => {
    if (e.detail !== 0) return;
    presses.down(p.n, "click");
    presses.up(p.n, "click");
  });

  const isActivate = (e: KeyboardEvent): boolean => e.key === " " || e.key === "Enter";
  g.addEventListener("keydown", (e) => {
    if (!isActivate(e)) return;
    e.preventDefault(); // Space would scroll the page
    presses.down(p.n, "key");
    g.toggleAttribute("data-down", presses.isHeld(p.n));
  });
  g.addEventListener("keyup", (e) => {
    if (!isActivate(e)) return;
    presses.up(p.n, "key");
    g.toggleAttribute("data-down", presses.isHeld(p.n));
  });
  g.addEventListener("blur", () => {
    presses.up(p.n, "key");
    g.toggleAttribute("data-down", presses.isHeld(p.n));
  });
  return g;
}

/** A knob turns by the arrow keys (up and right clockwise) and the wheel (up clockwise), one detent a press or a notch. It takes no press. */
function wireKnob(g: SVGGElement, p: Placement, turn: Turn): void {
  const detent: Record<string, number> = { ArrowUp: 1, ArrowRight: 1, ArrowDown: -1, ArrowLeft: -1 };
  g.addEventListener("keydown", (e) => {
    const d = detent[e.key];
    if (d === undefined) return;
    e.preventDefault(); // the arrows would scroll the page
    turn(p.n, d);
  });
  g.addEventListener(
    "wheel",
    (e) => {
      if (e.deltaY === 0) return;
      e.preventDefault(); // over a knob the wheel turns it and does not scroll the page
      turn(p.n, e.deltaY < 0 ? 1 : -1);
    },
    { passive: false },
  );
}

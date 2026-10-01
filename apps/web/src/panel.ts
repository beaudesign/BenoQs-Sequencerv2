// The panel: one SVG, drawn from a geometry (layout.ts) and the tokens, with each control a labelled button. It holds
// no sequencing logic: a press goes out through `Presses`, and the LED frame comes in through `paintLeds`.
//
// No colour, length or type size is written here. Colours, strokes and fonts are the stylesheet's, from the tokens;
// the only lengths are the ones layout.ts computes from the tokens and the two ring radii below, which come from the
// same key size and the two stroke tokens.
import { decodeLed } from "./abi.ts";
import type { Geometry, Placement } from "./layout.ts";
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
  /** Applies an LED frame (one byte per control number). Only what changed is touched. */
  paintLeds(frame: Uint8Array): void;
}

const SPOKEN: Record<string, string> = { steady: "lit", flash: "flashing", shine: "shining" };

export function drawPanel(doc: Document, geometry: Geometry, tokens: Tokens, presses: Presses): Panel {
  const root = svg(doc, "svg", { id: "panel", viewBox: geometry.viewBox.join(" "), role: "group", "aria-label": "Octopus panel" });
  const leds = new Map<number, { el: SVGElement; group: Element; last: number; id: string }>();

  for (const p of geometry.placements) root.append(drawControl(doc, p, tokens, presses, leds));

  return {
    svg: root,
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

function drawControl(doc: Document, p: Placement, tokens: Tokens, presses: Presses, leds: Map<number, { el: SVGElement; group: Element; last: number; id: string }>): SVGGElement {
  const g = svg(doc, "g", { class: "control", "data-id": p.id, "data-n": p.n, role: "button", tabindex: 0, "aria-label": p.name });
  const face = svg(doc, "circle", { class: p.led ? "face led" : "face", cx: p.cx, cy: p.cy, r: p.r });
  if (p.led) {
    face.setAttribute("data-colour", "off");
    face.setAttribute("data-phase", "steady");
    leds.set(p.n, { el: face, group: g, last: 0, id: p.id });
  }
  // The focus ring is two strokes, so that it shows on the ground and on a lit LED (docs/05 section 5): a light one
  // just outside the key and a dark one just inside its edge.
  const inner = svg(doc, "circle", { class: "ring-inner", cx: p.cx, cy: p.cy, r: p.r - tokens.stroke.focus_inner.value / 2 });
  const outer = svg(doc, "circle", { class: "ring-outer", cx: p.cx, cy: p.cy, r: p.r + tokens.stroke.focus.value / 2 });
  g.append(face, inner, outer);

  if (p.label) {
    const text = svg(doc, "text", { class: "label", x: p.label.x, y: p.label.y });
    const n = p.label.lines.length;
    p.label.lines.forEach((line, i) => {
      // The block of lines is centred on the key: the first line starts half the block above it.
      const dy = (i === 0 ? -(n - 1) / 2 : 1) * p.label!.leadingEm;
      const span = svg(doc, "tspan", { x: p.label!.x, dy: `${dy}em` });
      span.textContent = line;
      text.append(span);
    });
    g.append(text);
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

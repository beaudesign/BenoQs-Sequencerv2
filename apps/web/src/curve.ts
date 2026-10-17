// The Curve button's presets (SPEC-0005 r1, D6): how a roll, the strum of a chord or the seven strikes of a single note, is spaced. The
// numbers are the engine's own (`octoweb_set_chord_style`, ABI.md): a style, and for the curve a warp in tenths. Even (0) and Drag (10) are
// stored in the engine and left off the button to keep it to four states.
import { CHORD_STYLE } from "./abi.ts";

export interface CurvePreset {
  /** What the button says after `Curve: `. */
  label: string;
  style: number;
  warpTenths: number;
}

/** Off is the Octopus's own spacing; the others bend the same span. Accel and Bounce bunch the notes at the end of the roll, Bounce more; Rit bunches them at the start. */
export const CURVE_PRESETS: readonly CurvePreset[] = [
  { label: "Off", style: CHORD_STYLE.manual, warpTenths: 0 },
  { label: "Accel", style: CHORD_STYLE.curve, warpTenths: -5 },
  { label: "Bounce", style: CHORD_STYLE.curve, warpTenths: -10 },
  { label: "Rit", style: CHORD_STYLE.curve, warpTenths: 5 },
];

/** The preset a press moves to: round to Off after the last. */
export function nextCurve(index: number): number {
  return (index + 1) % CURVE_PRESETS.length;
}

/** The button's text, which names its state. */
export function curveText(index: number): string {
  return `Curve: ${CURVE_PRESETS[index]?.label ?? CURVE_PRESETS[0]!.label}`;
}

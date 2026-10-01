// The stylesheet of tokens, generated from contracts/design.tokens.json. Every other stylesheet in the app names
// these custom properties and writes no colour, length or type size of its own (rule S2, ADR-0008 decision 6).
import { COLOUR_KEYS, SPACE_KEYS, type Tokens } from "./tokens.ts";

const name = (key: string): string => key.replace(/[._]/g, "-");

export function tokensCss(t: Tokens): string {
  const lines: string[] = [];
  const add = (prop: string, value: string | number, unit = ""): void => void lines.push(`  --${prop}: ${value}${unit};`);
  for (const k of COLOUR_KEYS) add(`colour-${name(k)}`, t.colour[k].value);
  for (const k of SPACE_KEYS) add(`space-${k}`, t.space[k].value, "px");
  add("type-family", t.type.family.value);
  add("type-size-label", t.type.size.label.value, "px");
  add("type-size-body", t.type.size.body.value, "px");
  add("type-measure", t.type.measure.value, "ch");
  add("stroke-hairline", t.stroke.hairline.value, "px");
  add("stroke-focus", t.stroke.focus.value, "px");
  add("stroke-focus-inner", t.stroke.focus_inner.value, "px");
  add("flash-period", t.flash.period_ms.value, "ms");
  add("flash-shine", `var(--colour-${name(t.flash.shine.value)})`);
  return [
    "/* Generated from contracts/design.tokens.json by scripts/build-css.ts. Do not edit. */",
    ":root {",
    ...lines,
    "}",
    "",
    "/* The one animation: an LED that flashes is lit, then dark at the duty point, once a period. */",
    "@keyframes led-flash {",
    "  0% { fill: var(--lit); }",
    `  ${t.flash.duty_percent.value}% { fill: var(--colour-led-off); }`,
    "  100% { fill: var(--colour-led-off); }",
    "}",
    "",
  ].join("\n");
}

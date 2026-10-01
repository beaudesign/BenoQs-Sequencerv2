// The design tokens (contracts/design.tokens.json) as the app reads them. Nothing here holds a value: it checks the
// file has the shape of the contract and hands it on. The values are the file's, and the rules they keep are asserted
// by apps/web/engine/tests/tokens.rs.

export interface Pending {
  question: string;
  note: string;
}

export interface Leaf<T> {
  value: T;
  pending?: Pending;
}

export const COLOUR_KEYS = ["led.red", "led.green", "led.orange", "led.off", "ink", "ink.quiet", "surface", "focus", "focus.inner"] as const;
export type ColourKey = (typeof COLOUR_KEYS)[number];

export const SPACE_KEYS = ["0", "1", "2", "3", "4", "5", "6", "7"] as const;
export type SpaceKey = (typeof SPACE_KEYS)[number];

export interface Tokens {
  colour: Record<ColourKey, Leaf<string>>;
  space: Record<SpaceKey, Leaf<number>>;
  type: { family: Leaf<string>; size: { label: Leaf<number>; body: Leaf<number> }; measure: Leaf<number> };
  stroke: { hairline: Leaf<number>; focus: Leaf<number>; focus_inner: Leaf<number> };
  flash: { period_ms: Leaf<number>; duty_percent: Leaf<number>; shine: Leaf<"ink" | "ink.quiet"> };
}

function fail(path: string, want: string): never {
  throw new Error(`design.tokens.json: ${path} must be ${want}`);
}

function object(v: unknown, path: string): Record<string, unknown> {
  if (typeof v !== "object" || v === null || Array.isArray(v)) fail(path, "an object");
  return v as Record<string, unknown>;
}

function leaf<T>(v: unknown, path: string, check: (x: unknown) => x is T, want: string): Leaf<T> {
  const o = object(v, path);
  if (!check(o["value"])) fail(`${path}.value`, want);
  return o as unknown as Leaf<T>;
}

const isString = (x: unknown): x is string => typeof x === "string" && x.length > 0;
const isNumber = (x: unknown): x is number => typeof x === "number" && Number.isFinite(x);
const isHex = (x: unknown): x is string => typeof x === "string" && /^#[0-9a-f]{6}$/.test(x);
const isShine = (x: unknown): x is "ink" | "ink.quiet" => x === "ink" || x === "ink.quiet";

/** Checks the shape of the contract (the schema and the rule test do the rest) and returns it typed. */
export function parseTokens(json: unknown): Tokens {
  const root = object(json, "the file");
  if (root["schema"] !== "design.tokens/1") fail("schema", '"design.tokens/1"');
  const colour = object(root["colour"], "colour");
  const space = object(root["space"], "space");
  const type = object(root["type"], "type");
  const size = object(type["size"], "type.size");
  const stroke = object(root["stroke"], "stroke");
  const flash = object(root["flash"], "flash");
  for (const k of COLOUR_KEYS) leaf(colour[k], `colour.${k}`, isHex, "#rrggbb");
  for (const k of SPACE_KEYS) leaf(space[k], `space.${k}`, isNumber, "a number");
  leaf(type["family"], "type.family", isString, "a font family list");
  leaf(size["label"], "type.size.label", isNumber, "a number");
  leaf(size["body"], "type.size.body", isNumber, "a number");
  leaf(type["measure"], "type.measure", isNumber, "a number");
  leaf(stroke["hairline"], "stroke.hairline", isNumber, "a number");
  leaf(stroke["focus"], "stroke.focus", isNumber, "a number");
  leaf(stroke["focus_inner"], "stroke.focus_inner", isNumber, "a number");
  leaf(flash["period_ms"], "flash.period_ms", isNumber, "a number");
  leaf(flash["duty_percent"], "flash.duty_percent", isNumber, "a number");
  leaf(flash["shine"], "flash.shine", isShine, '"ink" or "ink.quiet"');
  return root as unknown as Tokens;
}

/** `"space.6"` to its value. The layout file names its sizes this way, so it never holds a length. */
export function spaceOf(tokens: Tokens, ref: string): number {
  const m = /^space\.([0-7])$/.exec(ref);
  if (!m) throw new Error(`${ref} is not a space token (space.0 to space.7)`);
  return tokens.space[m[1] as SpaceKey].value;
}

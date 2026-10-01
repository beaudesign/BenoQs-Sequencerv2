// The stylesheet is generated from contracts/design.tokens.json and holds every token once, as a custom property.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { parseTokens } from "../src/tokens.ts";
import { tokensCss } from "../src/tokens-css.ts";
import { repoRoot } from "./support/module.ts";

const raw = JSON.parse(readFileSync(`${repoRoot}/contracts/design.tokens.json`, "utf8")) as unknown;
const tokens = parseTokens(raw);
const css = tokensCss(tokens);

test("every colour, space, type, stroke and flash token is a custom property with its value", () => {
  for (const [key, leaf] of Object.entries(tokens.colour)) assert.ok(css.includes(`--colour-${key.replace(".", "-")}: ${leaf.value};`), key);
  for (const [key, leaf] of Object.entries(tokens.space)) assert.ok(css.includes(`--space-${key}: ${leaf.value}px;`), `space ${key}`);
  assert.ok(css.includes(`--type-family: ${tokens.type.family.value};`));
  assert.ok(css.includes(`--type-size-label: ${tokens.type.size.label.value}px;`));
  assert.ok(css.includes(`--type-size-body: ${tokens.type.size.body.value}px;`));
  assert.ok(css.includes(`--type-measure: ${tokens.type.measure.value}ch;`));
  assert.ok(css.includes(`--stroke-hairline: ${tokens.stroke.hairline.value}px;`));
  assert.ok(css.includes(`--stroke-focus: ${tokens.stroke.focus.value}px;`));
  assert.ok(css.includes(`--stroke-focus-inner: ${tokens.stroke.focus_inner.value}px;`));
  assert.ok(css.includes(`--flash-period: ${tokens.flash.period_ms.value}ms;`));
});

test("the shine state is drawn from the neutral the tokens name, not from a colour of its own", () => {
  assert.ok(css.includes("--flash-shine: var(--colour-ink);"));
  const other = structuredClone(tokens);
  other.flash.shine.value = "ink.quiet";
  assert.ok(tokensCss(other).includes("--flash-shine: var(--colour-ink-quiet);"));
});

test("there is one keyframes rule, for the LED flash, and it turns the light off at the duty point", () => {
  assert.equal((css.match(/@keyframes/g) ?? []).length, 1);
  assert.match(css, /@keyframes led-flash \{[^}]*0% \{ fill: var\(--lit\); \}/);
  assert.ok(css.includes(`${tokens.flash.duty_percent.value}% { fill: var(--colour-led-off); }`));
  const other = structuredClone(tokens);
  other.flash.duty_percent.value = 30;
  assert.ok(tokensCss(other).includes("30% { fill: var(--colour-led-off); }"));
});

test("a changed token changes the stylesheet, and the same tokens give the same bytes", () => {
  const other = structuredClone(tokens);
  other.colour["led.green"].value = "#00ff00";
  assert.notEqual(tokensCss(other), css);
  assert.ok(tokensCss(other).includes("--colour-led-green: #00ff00;"));
  assert.equal(tokensCss(tokens), css);
});

test("the stylesheet has no gradient, shadow, radius, transition or easing, and says it is generated", () => {
  assert.match(css, /^\/\* Generated from contracts\/design\.tokens\.json/);
  for (const bad of ["gradient", "shadow", "radius", "transition", "cubic-bezier", "ease", "linear("]) assert.ok(!css.includes(bad), bad);
});

test("parseTokens refuses a file that is not the contract", () => {
  assert.throws(() => parseTokens({}), /schema/);
  const missing = structuredClone(raw) as { colour: Record<string, unknown> };
  delete missing.colour["ink"];
  assert.throws(() => parseTokens(missing), /colour\.ink/);
  const wrong = structuredClone(raw) as { schema: string };
  wrong.schema = "design.tokens/2";
  assert.throws(() => parseTokens(wrong), /design\.tokens\/1/);
  const badHex = structuredClone(raw) as { colour: { ink: { value: string } } };
  badHex.colour.ink.value = "white";
  assert.throws(() => parseTokens(badHex), /#rrggbb/);
});

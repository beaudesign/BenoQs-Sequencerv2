// The app's stylesheet names only properties the generated tokens stylesheet defines, and the generated one defines exactly
// the properties apps/web/engine/tests/app_source_rules.rs expects (the two lists are the same list, pinned in both places).
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { parseTokens } from "../src/tokens.ts";
import { tokensCss } from "../src/tokens-css.ts";
import { repoRoot, webRoot } from "./support/module.ts";

const tokens = parseTokens(JSON.parse(readFileSync(`${repoRoot}/contracts/design.tokens.json`, "utf8")));
const generated = tokensCss(tokens);
const defined = [...generated.matchAll(/^\s+--([a-z0-9-]+):/gm)].map((m) => m[1]!).sort();

test("the tokens stylesheet defines exactly the properties the Rust rule test lists", () => {
  const want = [
    ...["led-red", "led-green", "led-orange", "led-off", "ink", "ink-quiet", "surface", "focus", "focus-inner"].map((k) => `colour-${k}`),
    ...[0, 1, 2, 3, 4, 5, 6, 7].map((k) => `space-${k}`),
    "type-family", "type-size-label", "type-size-body", "type-measure",
    "stroke-hairline", "stroke-focus", "stroke-focus-inner",
    "flash-period", "flash-shine",
  ].sort();
  assert.deepEqual(defined, want);
});

test("every var() in pages/app.css is a token property or the stylesheet's own --lit", () => {
  const css = readFileSync(`${webRoot}/pages/app.css`, "utf8");
  const used = [...css.matchAll(/var\(--([a-z0-9-]+)/g)].map((m) => m[1]!);
  assert.ok(used.length > 20);
  for (const name of used) assert.ok(name === "lit" || defined.includes(name), `--${name}`);
  assert.ok(generated.includes("var(--lit)"), "the keyframes read --lit, which app.css sets per colour");
  assert.ok(css.includes("--lit: var(--colour-led-red)") && css.includes("--lit: var(--colour-led-green)") && css.includes("--lit: var(--colour-led-orange)"));
});

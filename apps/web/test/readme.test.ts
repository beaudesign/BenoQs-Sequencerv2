// The tab order in the README is the one the layout produces (acceptance criterion A8: "a written traversal order").
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { parseLayout, place, tabOrder, type ControlsDoc } from "../src/layout.ts";
import { parseTokens } from "../src/tokens.ts";
import { repoRoot, webRoot } from "./support/module.ts";

const read = (rel: string): unknown => JSON.parse(readFileSync(`${repoRoot}/${rel}`, "utf8"));
const controls = read("contracts/controls.json") as ControlsDoc;
const geometry = place(parseLayout(read("apps/web/layout/panel.layout.json"), controls), controls, parseTokens(read("contracts/design.tokens.json")));
const readme = readFileSync(`${webRoot}/README.md`, "utf8");

function section(title: string): string {
  const start = readme.indexOf(`## ${title}`);
  assert.ok(start >= 0, `README has no "## ${title}"`);
  const end = readme.indexOf("\n## ", start + 1);
  return readme.slice(start, end < 0 ? undefined : end);
}

test("the README's tab order lists the strip, then the matrix in reading order, then the other controls in the layout's order", () => {
  const text = section("Tab order");
  const strip = /`start`, `metronome`, `midi-out-1`, `midi-out-2`, `midi-in`, `clock-state`, `clock-offset`, `lookahead`/.exec(text);
  assert.ok(strip, "the strip's eight stops");
  assert.match(text, /row 9 \(the top row\) from column 1 to column 16, then row 8/);
  assert.match(text, /row 0 \(the bottom row\)/);
  const listed = [...text.matchAll(/^\s+\d+\.\s+`([a-z0-9_.]+)`\s*$/gm)].map((m) => m[1]);
  assert.deepEqual(listed, tabOrder(geometry).slice(160), "the numbered controls are the layout's, in its order");
  assert.ok(text.indexOf("`start`") < text.indexOf("row 9") && text.indexOf("row 9") < text.indexOf("`mode.page`"), "strip, matrix, controls");
});

test("the README names every control the panel draws besides the matrix", () => {
  const text = section("The panel");
  for (const id of tabOrder(geometry).slice(160)) assert.ok(text.includes(`\`${id}\``), id);
});

test("the README says what the route hook is, and that it is a hook and not a control", () => {
  const text = section("The route hook");
  assert.match(text, /`\?route=0:17,1:3`/);
  assert.match(text, /not a control/);
  assert.match(text, /Track zoom/);
  assert.match(text, /matrix\.r3\.c1/, "the track is the row the control ids give");
});

test("the README says what the metronome is, what it follows, and that its latency is not measured or trimmed", () => {
  const text = section("The metronome");
  assert.match(text, /off until it is pressed/);
  assert.match(text, /quarter note/);
  assert.match(text, /not MIDI/);
  assert.match(text, /not on the panel/);
  assert.match(text, /not measured/i);
  assert.match(text, /not trimmed/);
});

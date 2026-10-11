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

test("the README's tab order lists the strip, then the plate's controls in reading order, and says what is not a stop", () => {
  const text = section("Tab order");
  const html = readFileSync(`${webRoot}/pages/app.html`, "utf8");
  const strip = [...html.matchAll(/<(?:button|input|select)\b[^>]*\bid="([a-z0-9-]+)"/g)].map((m) => m[1]);
  assert.match(text, new RegExp(`${strip.map((id) => `\`${id}\``).join(", ")}`), "the strip's stops, in the page's order");
  const order = tabOrder(geometry);
  assert.match(text, new RegExp(`${order.length} controls in its reading order`));
  assert.match(text, /bands one matrix row high from the top, left to right in each band/);
  const first = /The first stops are\s+([^;]*?); then row 8/s.exec(text);
  assert.ok(first, "the README names the first stops");
  const named = [...first[1]!.matchAll(/`([a-z0-9_.]+)`/g)].map((m) => m[1]!);
  assert.deepEqual(named, ["mix.enc.r9", "selector.r9", "matrix.r9.c1", "matrix.r9.c2", "matrix.r9.c16", "mutator.tgl", "edit.enc.vel", "main.enc"]);
  assert.deepEqual(order.slice(0, 4), ["mix.enc.r9", "selector.r9", "matrix.r9.c1", "matrix.r9.c2"]);
  assert.equal(order[17], "matrix.r9.c16");
  assert.deepEqual(order.slice(18, 21), ["mutator.tgl", "edit.enc.vel", "main.enc"]);
  assert.equal(order[21], "mix.enc.r8");
  const last = /The last stops are\s+(.*?)\n\s+The plate's own keys/s.exec(text);
  assert.ok(last, "the README names the last stops");
  assert.deepEqual([...last[1]!.matchAll(/`([a-z0-9_.]+)`/g)].map((m) => m[1]), order.slice(-5));
  assert.match(text, /`hw\.\*`\) are not stops/);
});

test("the README says what the panel draws: the counts, and the controls it does not draw", () => {
  const text = section("The panel");
  const file = JSON.parse(readFileSync(`${repoRoot}/apps/web/layout/panel.layout.json`, "utf8")) as { unplaced: { id: string }[]; keys: unknown[] };
  assert.match(text, new RegExp(`${tabOrder(geometry).length} of the ${controls.controls.length} controls`));
  assert.match(text, new RegExp(`${file.keys.length} keys\\*\\* the plate has`));
  for (const u of file.unplaced) assert.ok(text.includes(`\`${u.id}\``), `${u.id} is named as not drawn`);
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

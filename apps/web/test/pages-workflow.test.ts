// The Pages workflow keeps the shape specs/SPEC-0002/p11-published-page.md gives it (P11-6): it can only be started from main or by hand,
// it asks for the least it needs, the deploy is the only job that may publish, and it builds the same thing the tests serve.
// Text checks, because the repo has no YAML reader and takes no dependency for one.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { repoRoot } from "./support/module.ts";

const file = ".github/workflows/pages.yml";
const text = readFileSync(`${repoRoot}/${file}`, "utf8");
const code = text.split("\n").filter((l) => !/^\s*#/.test(l)).join("\n");

/** The lines of a top-level key's block (a `jobs:` child, say), up to the next key at the same depth. */
function block(source: string, header: RegExp, indent: number): string {
  const lines = source.split("\n");
  const start = lines.findIndex((l) => header.test(l));
  assert.notEqual(start, -1, `${header} is in the workflow`);
  const out: string[] = [];
  for (const l of lines.slice(start + 1)) {
    if (l.trim() !== "" && l.length - l.trimStart().length <= indent) break;
    out.push(l);
  }
  return out.join("\n");
}

test("P11-6: it starts on a push to main and by hand, and on nothing a stranger can cause", () => {
  const on = block(code, /^on:/, 0);
  assert.match(on, /push:\s*\n\s*branches: \[main\]/);
  assert.match(on, /workflow_dispatch:/);
  assert.doesNotMatch(code, /pull_request|pull_request_target|issue_comment|workflow_run|schedule:/);
});

test("P11-6: the workflow as a whole may only read; the deploy job alone may publish, and says where", () => {
  assert.match(block(code, /^permissions:/, 0), /^\s+contents: read\s*$/m);
  assert.doesNotMatch(block(code, /^permissions:/, 0), /pages|id-token|write/);
  const deploy = block(code, /^  deploy:/, 2);
  assert.match(deploy, /needs: build/);
  assert.match(deploy, /pages: write/);
  assert.match(deploy, /id-token: write/);
  assert.match(deploy, /name: github-pages/);
  assert.match(deploy, /uses: actions\/deploy-pages@v\d+/);
  const build = block(code, /^  build:/, 2);
  assert.doesNotMatch(build, /write/, "the build job, which runs the repository's code, cannot publish");
});

test("P11-6: it builds what the tests serve: the shipped module, the page, scripts/site.ts, and publishes that folder and nothing else", () => {
  const build = block(code, /^  build:/, 2);
  assert.match(build, /cargo build --release -p octoweb --target wasm32-unknown-unknown --no-default-features/);
  assert.match(build, /cp target\/wasm32-unknown-unknown\/release\/octoweb\.wasm apps\/web\/dist\/octoweb\.wasm/);
  assert.match(build, /npm run build/);
  const assemble = /node scripts\/site\.ts "?([^"\s]+)"?/.exec(build);
  assert.ok(assemble, "the site is assembled by scripts/site.ts");
  const upload = block(build, /uses: actions\/upload-pages-artifact@v\d+/, 6);
  assert.match(upload, /path: \$\{\{ runner\.temp \}\}\/site/);
  assert.equal(assemble[1], "$RUNNER_TEMP/site", "and the folder it writes is the folder it uploads");
});

test("P11-6: it uses no secret, and every action is pinned to a version tag", () => {
  assert.doesNotMatch(code, /secrets\./);
  const uses = [...code.matchAll(/uses:\s*(\S+)/g)].map((m) => m[1]!);
  assert.ok(uses.length >= 5, `found ${uses.length} actions`);
  for (const u of uses) assert.match(u, /^actions\/[a-z-]+@v\d+$/, `${u} is an official action at a version tag`);
});

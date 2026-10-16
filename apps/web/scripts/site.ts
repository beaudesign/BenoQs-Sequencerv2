// Assembles the folder a static host serves (SPEC-0002 P11): the page and exactly the files it loads, in the layout the page expects, and an
// index that opens it. `npm run site` writes it to dist/site; the Pages workflow writes it where the host picks it up.
//   node scripts/site.ts [outDir]
// Needs `npm run build` and `just web-wasm` first, and says which file is missing if they were not run.
import { cpSync, existsSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const webRoot = resolve(here, "..");
const repoRoot = resolve(webRoot, "../..");

/** [where it is, where it goes in the site]. A folder goes with everything in it. */
const FILES: [string, string][] = [
  [join(webRoot, "dist/tokens.css"), "dist/tokens.css"],
  [join(webRoot, "dist/octoweb.wasm"), "dist/octoweb.wasm"],
  [join(webRoot, "dist/pages/app.js"), "dist/pages/app.js"],
  [join(webRoot, "dist/src"), "dist/src"],
  [join(webRoot, "pages/app.html"), "pages/app.html"],
  [join(webRoot, "pages/app.css"), "pages/app.css"],
  [join(webRoot, "pages/fonts"), "pages/fonts"],
  [join(webRoot, "layout/panel.layout.json"), "layout/panel.layout.json"],
  [join(repoRoot, "contracts/controls.json"), "contracts/controls.json"],
  [join(repoRoot, "contracts/design.tokens.json"), "contracts/design.tokens.json"],
];

/** A page that sends a visitor to the app, so the site's address is the address to give. Carries no style of its own. */
const INDEX = `<!doctype html>
<html lang="en">
<meta charset="utf-8">
<meta http-equiv="refresh" content="0; url=pages/app.html">
<title>WENGE</title>
<p><a href="pages/app.html">Open the sequencer</a></p>
</html>
`;

export function assembleSite(outDir: string): void {
  const out = resolve(outDir);
  for (const [from] of FILES) {
    if (!existsSync(from)) throw new Error(`${from} is missing: run \`just web-wasm\` and \`npm run build\` first`);
  }
  rmSync(out, { recursive: true, force: true });
  mkdirSync(out, { recursive: true });
  for (const [from, to] of FILES) {
    mkdirSync(dirname(join(out, to)), { recursive: true });
    cpSync(from, join(out, to), { recursive: true });
  }
  writeFileSync(join(out, "index.html"), INDEX);
  // The host runs no Jekyll over it: the files are served as they are.
  writeFileSync(join(out, ".nojekyll"), "");
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const out = resolve(process.argv[2] ?? join(webRoot, "dist/site"));
  assembleSite(out);
  console.log(out);
}

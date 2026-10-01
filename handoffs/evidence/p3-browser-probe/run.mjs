// node run.mjs   (needs the `playwright` package, and a Chromium it can find)
// Serves this folder on localhost, opens page.html in headless Chromium with the Web MIDI
// permission granted, and prints what the page found. WENGE P3 (WENGE-0014), ADR-0008.
import { createRequire } from "node:module";
import http from "node:http";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const require = createRequire(process.env.PLAYWRIGHT_NODE_PATH ?? "/home/claude/.npm-global/lib/node_modules/");
const { chromium } = require("playwright");
const here = dirname(fileURLToPath(import.meta.url));
const types = { ".js": "text/javascript", ".html": "text/html" };
const server = http.createServer((req, res) => {
  const name = req.url.split("?")[0].slice(1) || "page.html";
  try {
    res.writeHead(200, { "content-type": types[name.slice(name.lastIndexOf("."))] ?? "text/plain" });
    res.end(readFileSync(join(here, name)));
  } catch { res.writeHead(404); res.end(); }
}).listen(0);

const browser = await chromium.launch({ headless: true });
const context = await browser.newContext({ permissions: ["midi"] });
const page = await context.newPage();
await page.goto(`http://localhost:${server.address().port}/page.html`);
await page.waitForFunction(() => window.__probe, null, { timeout: 20000 });
console.log(JSON.stringify({ chromium: browser.version(), ...(await page.evaluate(() => window.__probe)) }, null, 2));
await browser.close();
server.close();

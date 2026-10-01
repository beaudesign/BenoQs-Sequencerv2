// node midioutput-probe.mjs   (needs the `playwright` package and a Chromium it can find)
// Does this Chromium implement MIDIOutput.clear()? A localhost page is a secure context, so Web MIDI's interfaces exist
// there (an about:blank page has none). The permission prompt is not exercised: only the interface is read.
// WENGE P3 (WENGE-0014), ADR-0008 amendment 1.
import { createRequire } from "node:module";
import http from "node:http";
const require = createRequire("/home/claude/.npm-global/lib/node_modules/");
const { chromium } = require("playwright");
const server = http.createServer((req, res) => { res.writeHead(200, { "content-type": "text/html" }); res.end("<!doctype html><title>p</title>"); }).listen(0);
const browser = await chromium.launch({ headless: true });
const page = await browser.newPage();
await page.goto(`http://localhost:${server.address().port}/`);
console.log(browser.version(), JSON.stringify(await page.evaluate(() => ({
  secure: isSecureContext,
  requestMIDIAccess: typeof navigator.requestMIDIAccess,
  MIDIOutput: typeof MIDIOutput,
  own: typeof MIDIOutput === "function" ? Object.getOwnPropertyNames(MIDIOutput.prototype) : [],
  clear: typeof MIDIOutput === "function" ? typeof MIDIOutput.prototype.clear : "n/a",
}))));
await browser.close(); server.close();

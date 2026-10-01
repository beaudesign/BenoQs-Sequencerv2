// A static file server for the browser tests and the spikes: localhost is a secure context, which Web MIDI and
// AudioWorklet need. It serves four folders and nothing else, GET only.
import { readFile } from "node:fs/promises";
import http from "node:http";
import type { AddressInfo } from "node:net";
import { dirname, extname, join, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
export const webRoot = resolve(here, "..");
export const repoRoot = resolve(webRoot, "../..");

const ROUTES: [string, string][] = [
  ["/dist/", join(webRoot, "dist")],
  ["/pages/", join(webRoot, "pages")],
  ["/spikes/", join(webRoot, "spikes")],
  ["/contracts/", join(repoRoot, "contracts")],
];

const TYPES: Record<string, string> = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".json": "application/json",
  ".wasm": "application/wasm",
  ".css": "text/css; charset=utf-8",
};

/** Cross-origin isolation gives `performance.now()` its fine resolution (5 microseconds, not 100), which spike S2 needs. */
const ISOLATION = { "cross-origin-opener-policy": "same-origin", "cross-origin-embedder-policy": "require-corp" };

export interface Server {
  url: string;
  close(): Promise<void>;
}

export async function serve(port = 0): Promise<Server> {
  const server = http.createServer((req, res) => {
    void (async () => {
      try {
        const path = decodeURIComponent((req.url ?? "/").split("?")[0] ?? "/");
        const route = ROUTES.find(([prefix]) => path.startsWith(prefix));
        if (req.method !== "GET" || !route) {
          res.writeHead(404).end();
          return;
        }
        const file = resolve(route[1], path.slice(route[0].length));
        if (file !== route[1] && !file.startsWith(route[1] + sep)) {
          res.writeHead(403).end();
          return;
        }
        const body = await readFile(file);
        res.writeHead(200, { "content-type": TYPES[extname(file)] ?? "application/octet-stream", "cache-control": "no-store", ...ISOLATION }).end(body);
      } catch {
        res.writeHead(404).end();
      }
    })();
  });
  await new Promise<void>((ok) => server.listen(port, "127.0.0.1", ok));
  const address = server.address() as AddressInfo;
  return {
    url: `http://localhost:${address.port}`,
    close: () => new Promise<void>((ok) => server.close(() => ok())),
  };
}

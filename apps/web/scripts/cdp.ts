// A small Chrome DevTools Protocol client over Node's built-in WebSocket, and a launcher that starts Chromium
// itself. It exists for one thing Playwright cannot do: Playwright keeps every page "visible and focused", so a
// tab put in the background never reports `document.visibilityState === "hidden"`. Spike S1's hidden-tab run needs
// a browser whose tabs really go to the background, so it starts Chromium directly, drives the first tab through
// this client, and opens a second tab in the same window.
import { spawn, type ChildProcess } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { chromium } from "playwright-core";

interface Pending {
  resolve(value: unknown): void;
  reject(error: Error): void;
}

export class Cdp {
  private readonly ws: WebSocket;
  private id = 0;
  private readonly pending = new Map<number, Pending>();
  private readonly listeners = new Map<string, ((params: unknown, sessionId?: string) => void)[]>();

  private constructor(ws: WebSocket) {
    this.ws = ws;
    ws.addEventListener("message", (e: MessageEvent<string>) => {
      const m = JSON.parse(e.data) as { id?: number; result?: unknown; error?: { message: string }; method?: string; params?: unknown; sessionId?: string };
      if (m.id !== undefined) {
        const p = this.pending.get(m.id);
        this.pending.delete(m.id);
        if (m.error) p?.reject(new Error(m.error.message));
        else p?.resolve(m.result);
      } else if (m.method) {
        for (const l of this.listeners.get(m.method) ?? []) l(m.params, m.sessionId);
      }
    });
  }

  static async connect(url: string): Promise<Cdp> {
    const ws = new WebSocket(url);
    await new Promise<void>((ok, fail) => {
      ws.addEventListener("open", () => ok());
      ws.addEventListener("error", () => fail(new Error(`cannot connect to ${url}`)));
    });
    return new Cdp(ws);
  }

  send<T = unknown>(method: string, params: object = {}, sessionId?: string): Promise<T> {
    const id = ++this.id;
    return new Promise<T>((resolve, reject) => {
      this.pending.set(id, { resolve: resolve as (v: unknown) => void, reject });
      this.ws.send(JSON.stringify({ id, method, params, ...(sessionId ? { sessionId } : {}) }));
    });
  }

  on(method: string, listener: (params: unknown, sessionId?: string) => void): void {
    this.listeners.set(method, [...(this.listeners.get(method) ?? []), listener]);
  }

  close(): void {
    this.ws.close();
  }
}

export interface RawBrowser {
  cdp: Cdp;
  version: string;
  close(): Promise<void>;
}

/** Starts Chromium with a debugging port. Headed, so it needs a display (`xvfb-run -a`). */
export async function launchRaw(): Promise<RawBrowser> {
  const exe = process.env["CHROMIUM_PATH"] ?? chromium.executablePath();
  const dir = mkdtempSync(join(tmpdir(), "wenge-chromium-"));
  const child: ChildProcess = spawn(
    exe,
    ["--remote-debugging-port=0", `--user-data-dir=${dir}`, "--no-first-run", "--no-default-browser-check", "--autoplay-policy=no-user-gesture-required", "--no-sandbox", "about:blank"],
    { stdio: "ignore" },
  );
  const portFile = join(dir, "DevToolsActivePort");
  const until = Date.now() + 15_000;
  while (!existsSync(portFile)) {
    if (Date.now() > until) {
      child.kill();
      throw new Error("Chromium did not open a debugging port");
    }
    await new Promise((ok) => setTimeout(ok, 100));
  }
  const [port, path] = readFileSync(portFile, "utf8").split("\n");
  const cdp = await Cdp.connect(`ws://127.0.0.1:${port}${path}`);
  const { product } = await cdp.send<{ product: string }>("Browser.getVersion");
  return {
    cdp,
    version: product,
    async close() {
      cdp.close();
      child.kill();
      await new Promise((ok) => setTimeout(ok, 300));
      rmSync(dir, { recursive: true, force: true });
    },
  };
}

/** A tab, driven over a session of the browser connection. */
export class Tab {
  readonly cdp: Cdp;
  readonly targetId: string;
  readonly sessionId: string;

  private constructor(cdp: Cdp, targetId: string, sessionId: string) {
    this.cdp = cdp;
    this.targetId = targetId;
    this.sessionId = sessionId;
  }

  /** Opens a tab. Without `newWindow` it joins the window the browser has, so opening a second one hides the first. */
  static async open(cdp: Cdp, url = "about:blank"): Promise<Tab> {
    const { targetId } = await cdp.send<{ targetId: string }>("Target.createTarget", { url, newWindow: false });
    const { sessionId } = await cdp.send<{ sessionId: string }>("Target.attachToTarget", { targetId, flatten: true });
    return new Tab(cdp, targetId, sessionId);
  }

  send<T = unknown>(method: string, params: object = {}): Promise<T> {
    return this.cdp.send<T>(method, params, this.sessionId);
  }

  async evaluate<T>(expression: string): Promise<T> {
    const r = await this.send<{ result: { value?: T }; exceptionDetails?: { text: string; exception?: { description?: string } } }>("Runtime.evaluate", {
      expression,
      awaitPromise: true,
      returnByValue: true,
    });
    if (r.exceptionDetails) throw new Error(r.exceptionDetails.exception?.description ?? r.exceptionDetails.text);
    return r.result.value as T;
  }

  async bringToFront(): Promise<void> {
    await this.cdp.send("Target.activateTarget", { targetId: this.targetId });
  }

  async close(): Promise<void> {
    await this.cdp.send("Target.closeTarget", { targetId: this.targetId });
  }
}

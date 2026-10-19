// The Curve button in the page (SPEC-0005 r1 S2, observables CV-11 and CV-12). The owner, Sat 2026-10-17 14:50 London: "Let's build r1 -
// Benny's curved time as well". The button steps Off, Accel, Bounce, Rit for all ten tracks. What it changes is where the notes of a roll
// start, so that is what is measured: the timestamps of the Note Ons the page hands to a (recording) MIDI output, and, as in
// drums.browser.test.ts, what the audio node plays, from an analyser beside its connection to the speakers.
// The page cannot enter a chord today (the ABI writes tracks and the attributes of steps; chords come from a loaded pattern), so the roll
// measured is the other one the style governs: a single note with a strum of 9 is struck seven times (SPEC-0005 section 4, CV-6).
import assert from "node:assert/strict";
import { after, before, test } from "node:test";
import { chromium, type Browser, type Page } from "playwright-core";
import { serve, type Server } from "../../scripts/serve.ts";
import { ROLL_9, TICK_MS } from "../support/roll.ts";

let server: Server;
let browser: Browser;

before(async () => {
  server = await serve();
  const path = process.env["CHROMIUM_PATH"];
  browser = await chromium.launch({ headless: true, ...(path ? { executablePath: path } : {}), args: ["--disable-background-timer-throttling", "--disable-renderer-backgrounding"] });
});

after(async () => {
  await browser?.close();
  await server?.close();
});

interface Sent {
  data: number[];
  /** The timestamp the page gave the send, or the page's time when it gave none (a send that was already due). */
  t: number;
}

declare global {
  interface Window {
    __tap: { peakOver(ms: number): number };
    __curveOut: { sent: Sent[] };
    __curveNode: AudioWorkletNode;
    __curvePosted: { type: string; track: number; style: number; warpTenths: number }[];
  }
}

/** Taps what the page's audio node plays, keeps the node so the test can write a step the way any page code would, records the chord-style messages the page posts, and gives the page one named MIDI output that records what it is sent. */
function install(): void {
  const proto = AudioNode.prototype as unknown as { connect: (...a: unknown[]) => unknown };
  const real = proto.connect;
  let made = false;
  proto.connect = function (this: AudioNode, ...args: unknown[]) {
    const result = real.apply(this, args);
    if (!made && this instanceof AudioWorkletNode) {
      made = true;
      window.__curveNode = this;
      const analyser = this.context.createAnalyser();
      analyser.fftSize = 2048;
      real.call(this, analyser);
      const buffer = new Float32Array(analyser.fftSize);
      const seen: { at: number; peak: number }[] = [];
      setInterval(() => {
        analyser.getFloatTimeDomainData(buffer);
        let peak = 0;
        for (const x of buffer) peak = Math.max(peak, Math.abs(x));
        seen.push({ at: performance.now(), peak });
        if (seen.length > 400) seen.shift();
      }, 20);
      window.__tap = { peakOver: (ms) => seen.filter((s) => s.at > performance.now() - ms).reduce((m, s) => Math.max(m, s.peak), 0) };
    }
    return result;
  };
  const posted: Window["__curvePosted"] = [];
  window.__curvePosted = posted;
  const original = MessagePort.prototype.postMessage;
  MessagePort.prototype.postMessage = function (this: MessagePort, message: unknown, ...rest: unknown[]) {
    const m = message as { type?: string; track?: number; style?: number; warpTenths?: number } | null;
    if (m && m.type === "chordStyle") posted.push({ type: m.type, track: m.track ?? -1, style: m.style ?? -1, warpTenths: m.warpTenths ?? 0 });
    return (original as (...a: unknown[]) => void).call(this, message, ...rest);
  } as typeof MessagePort.prototype.postMessage;
  const sent: Sent[] = [];
  const out = { id: "fake-1", name: "Fake output", send: (data: ArrayLike<number>, t?: number) => void sent.push({ data: Array.from(data), t: t ?? performance.now() }) };
  window.__curveOut = { sent };
  Object.defineProperty(Navigator.prototype, "requestMIDIAccess", {
    configurable: true,
    value: () => Promise.resolve({ outputs: new Map([[out.id, out]]), inputs: new Map(), onstatechange: null }),
  });
}

const face = (id: string): string => `g[data-id="${id}"] .face`;

async function open(): Promise<Page> {
  const page = await browser.newPage({ viewport: { width: 1280, height: 900 } });
  page.on("pageerror", (e) => assert.fail(`page error: ${e.message}`));
  await page.addInitScript(install);
  await page.goto(`${server.url}/pages/app.html?bare&route=0:2`);
  await page.waitForSelector('main[data-engine="idle"]');
  return page;
}

const text = async (page: Page): Promise<string> => ((await page.textContent("#curve")) ?? "").trim();

/** Starts the engine, routes track 0 to the recording output, puts the sound on, writes step 1 of track 0 as a single note with a strum of 9 and starts the transport. */
async function playing(page: Page): Promise<void> {
  await page.click("#play");
  await page.waitForSelector('main[data-engine="running"]', { timeout: 15_000 });
  await page.selectOption("#midi-out-1", "fake-1");
  await page.click("#monitor");
  await page.evaluate(() => {
    // Step attribute 0 is Active and 8 is Strum, in the engine's order (ABI.md); the same messages the demo loader posts.
    window.__curveNode.port.postMessage({ type: "step", track: 0, step: 0, attr: 0, value: 1 });
    window.__curveNode.port.postMessage({ type: "step", track: 0, step: 0, attr: 8, value: 9 });
  });
  await page.click(face("transport.play"));
}

/** The Note On times in milliseconds, grouped into rolls: a gap of a second starts a new one (a roll is under half a second long and the pattern comes round every two). */
function rolls(sent: Sent[]): number[][] {
  const ons = sent.filter((s) => (s.data[0]! & 0xf0) === 0x90 && (s.data[2] ?? 0) > 0).map((s) => s.t).sort((a, b) => a - b);
  const groups: number[][] = [];
  for (const t of ons) {
    const last = groups[groups.length - 1];
    if (last && t - last[last.length - 1]! < 1000) last.push(t);
    else groups.push([t]);
  }
  return groups;
}

/** The first roll of seven that begins at least 400 ms after `mark`, once there is one: the engine has had the new style for a whole step by then. */
async function rollAfter(page: Page, mark: number): Promise<number[]> {
  for (let i = 0; i < 70; i++) {
    const sent = await page.evaluate(() => window.__curveOut.sent);
    const found = rolls(sent).find((g) => g.length >= 7 && g[0]! > mark + 400);
    if (found) return found.slice(0, 7).map((t) => (t - found[0]!) / TICK_MS);
    await page.waitForTimeout(150);
  }
  throw new Error("no roll of seven notes arrived");
}

/** Presses Curve until it says `want`. */
async function choose(page: Page, want: string): Promise<void> {
  for (let i = 0; i < 4 && (await text(page)) !== want; i++) await page.click("#curve");
  assert.equal(await text(page), want);
}

const near = (got: number[], want: readonly number[], what: string): void => {
  assert.equal(got.length, want.length, what);
  got.forEach((g, k) => assert.ok(Math.abs(g - want[k]!) <= 0.6, `${what}: strike ${k + 1} starts ${g.toFixed(2)} ticks after the first, the table says ${want[k]} (${got.map((x) => x.toFixed(1)).join(", ")})`));
};

test("CV-11: the strip has a Curve button after Metronome that names its state: Off to begin with, then Accel, Bounce, Rit, and round to Off", async () => {
  const page = await open();
  assert.equal(await page.$eval("#curve", (el) => el.previousElementSibling?.id), "metronome", "next to Metronome, after it");
  assert.equal(await page.$eval("#curve", (el) => el.closest("#strip") !== null && el.closest("svg") === null), true, "in the strip, not on the panel");
  assert.equal(await page.$eval("#curve", (el) => (el as HTMLButtonElement).disabled), false, "usable before Start");
  const seen: [string, string | null][] = [[await text(page), await page.getAttribute("#curve", "aria-pressed")]];
  for (let i = 0; i < 4; i++) {
    await page.click("#curve");
    seen.push([await text(page), await page.getAttribute("#curve", "aria-pressed")]);
  }
  assert.deepEqual(seen, [
    ["Curve: Off", "false"],
    ["Curve: Accel", "true"],
    ["Curve: Bounce", "true"],
    ["Curve: Rit", "true"],
    ["Curve: Off", "false"],
  ]);
  assert.equal(await page.getByRole("button", { name: "Curve: Accel" }).count(), 0, "it is one button whose name follows its state");
  await page.close();
});

test("CV-11: the keyboard works it, with Space and with Enter", async () => {
  const page = await open();
  await page.focus("#curve");
  await page.keyboard.press("Space");
  assert.equal(await text(page), "Curve: Accel");
  await page.keyboard.press("Enter");
  assert.equal(await text(page), "Curve: Bounce");
  await page.close();
});

test("CV-12: a style that is on shows it by the raised tone and an ink border, as the other toggles do, and no colour is added", async () => {
  const page = await open();
  const look = async () => page.$eval("#curve", (el) => ({ background: getComputedStyle(el).backgroundColor, color: getComputedStyle(el).color, border: getComputedStyle(el).borderTopColor, shadow: getComputedStyle(el).boxShadow, image: getComputedStyle(el).backgroundImage, transition: getComputedStyle(el).transitionDuration }));
  const grey = (css: string): boolean => {
    const m = /^rgb\((\d+), (\d+), (\d+)\)$/.exec(css);
    return m !== null && m[1] === m[2] && m[2] === m[3];
  };
  const off = await look();
  await page.click("#curve");
  await page.mouse.move(0, 0);
  const on = await look();
  assert.notEqual(on.background, off.background, "a style that is on looks different");
  assert.notEqual(on.border, off.border, "and the border changed with the tone, so the state is not a shade alone");
  assert.equal(on.color, off.color);
  assert.ok(grey(on.background) && grey(on.border) && grey(on.color), `no colour is added: ${JSON.stringify(on)}`);
  assert.equal(on.shadow, "none", "no shadow");
  assert.equal(on.image, "none", "no gradient");
  assert.equal(on.transition, "0s", "no easing");
  await page.click("#curve");
  await page.click("#curve");
  await page.click("#curve");
  await page.mouse.move(0, 0); // off the button, or the hover border is what is measured
  assert.deepEqual(await look(), off, "round to Off, and it looks as it did");
  await page.close();
});

test("CV-11: the choice reaches the worklet, one message for each of the ten tracks each time it changes once the sequencer has started, and a choice made before Start is sent at Start", async () => {
  const page = await open();
  await page.click("#curve"); // Accel, before Start
  assert.deepEqual(await page.evaluate(() => window.__curvePosted), [], "nothing to send it to before Start");
  await page.click("#play");
  await page.waitForSelector('main[data-engine="running"]', { timeout: 15_000 });
  await page.waitForFunction(() => window.__curvePosted.length >= 10, null, { timeout: 5_000 });
  const tracks = (m: Window["__curvePosted"]): number[] => m.map((x) => x.track);
  let posted = await page.evaluate(() => window.__curvePosted);
  assert.deepEqual(tracks(posted), [0, 1, 2, 3, 4, 5, 6, 7, 8, 9], "all ten tracks, once");
  assert.ok(posted.every((m) => m.style === 1 && m.warpTenths === -5), "Accel is the curve at a warp of -0.5: style 1, -5 tenths");
  await page.click("#curve"); // Bounce
  await page.waitForFunction(() => window.__curvePosted.length >= 20, null, { timeout: 5_000 });
  posted = (await page.evaluate(() => window.__curvePosted)).slice(10);
  assert.ok(posted.length === 10 && posted.every((m) => m.style === 1 && m.warpTenths === -10), "Bounce: -10 tenths");
  await page.click("#curve"); // Rit
  await page.click("#curve"); // Off
  await page.waitForFunction(() => window.__curvePosted.length >= 40, null, { timeout: 5_000 });
  posted = (await page.evaluate(() => window.__curvePosted)).slice(20);
  assert.ok(posted.slice(0, 10).every((m) => m.style === 1 && m.warpTenths === 5), "Rit: 5 tenths");
  assert.ok(posted.slice(10).every((m) => m.style === 0), "Off is the manual's table, style 0");
  await page.close();
});

test("CV-11: each preset moves where the notes of a roll start, in what the MIDI output gets, to the table of SPEC-0005 for strum 9; Off is the Octopus's own; and the monitor plays them", async () => {
  const page = await open();
  await playing(page);
  const chosen: [string, readonly number[]][] = [
    ["Curve: Off", ROLL_9.manual],
    ["Curve: Accel", ROLL_9.accel],
    ["Curve: Bounce", ROLL_9.bounce],
    ["Curve: Rit", ROLL_9.rit],
    ["Curve: Off", ROLL_9.manual],
  ];
  for (const [label, table] of chosen) {
    await choose(page, label);
    const mark = await page.evaluate(() => performance.now());
    near(await rollAfter(page, mark), table, label);
    assert.ok((await page.evaluate(() => window.__tap.peakOver(700))) > 0.02, `${label}: the monitor plays the roll`);
  }
  await page.close();
});

test("CV-11: a change made while it plays takes hold from the next roll, and the notes that were already struck are not moved", async () => {
  const page = await open();
  await playing(page);
  await choose(page, "Curve: Off");
  const first = await page.evaluate(() => performance.now());
  near(await rollAfter(page, first), ROLL_9.manual, "before");
  await choose(page, "Curve: Bounce");
  const mark = await page.evaluate(() => performance.now());
  near(await rollAfter(page, mark), ROLL_9.bounce, "after");
  const sent = await page.evaluate(() => window.__curveOut.sent);
  const all = rolls(sent);
  assert.ok(all.every((g) => g.length === 7 || g === all[all.length - 1]), "every roll was struck whole: seven notes");
  await page.close();
});

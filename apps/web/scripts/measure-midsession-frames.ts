// Whether `process()` is ever skipped in the middle of a session, as an AudioWorklet sees it: `node scripts/measure-midsession-frames.ts [--runs 10] [--seconds 8]`.
// A made-up processor (not the engine) logs the step of `currentFrame` from one `process()` call to the next, and this prints every step that is not
// 128 (the render quantum) after the start-up, with the main thread busy as the S3 page's is (a timer every 20 ms, a message to the worklet every 8 blocks).
// A skipped block leaves the engine, which renders only the calls it gets, behind the audio timeline for good; the follower then reads a phase error
// and corrects it. Measured on the Chromium that playwright finds, headless, with no sound device; what a real device does is S1's and S3's to say.
import { launch } from "./browser.ts";
import { serve } from "./serve.ts";

const argv = process.argv.slice(2);
const opt = (name: string, fallback: number): number => {
  const i = argv.indexOf(name);
  return i >= 0 ? Number(argv[i + 1]) : fallback;
};
const RUNS = opt("--runs", 10);
const SECONDS = opt("--seconds", 8);

const server = await serve();
const browser = await launch();
const lines = [`# steps of currentFrame between process() calls that are not 128, after the first 20 calls, ${RUNS} fresh contexts of ${SECONDS} s with a 50 Hz timer busy on the main thread`];
let skips = 0;
for (let run = 0; run < RUNS; run++) {
  const page = await browser.newPage();
  await page.goto(`${server.url}/pages/host.html`);
  const result = await page.evaluate(async (seconds) => {
    const code = `class P extends AudioWorkletProcessor {
      constructor() { super(); this.last = null; this.calls = 0; this.odd = []; this.port.onmessage = () => this.port.postMessage(this.odd); }
      process() {
        if (this.last !== null && this.calls > 20 && currentFrame - this.last !== 128) this.odd.push([currentFrame, currentFrame - this.last]);
        this.last = currentFrame; this.calls++; return true;
      }
    }
    registerProcessor("p", P);`;
    const ctx = new AudioContext();
    await ctx.audioWorklet.addModule(URL.createObjectURL(new Blob([code], { type: "text/javascript" })));
    const node = new AudioWorkletNode(ctx, "p");
    node.connect(ctx.destination);
    await ctx.resume();
    const stop = performance.now() + seconds * 1000;
    let ticks = 0;
    await new Promise<void>((done) => {
      const tick = (): void => {
        ticks++;
        if (performance.now() >= stop) done();
        else setTimeout(tick, 20);
      };
      tick();
    });
    const odd = await new Promise<[number, number][]>((resolve) => {
      node.port.onmessage = (e) => resolve(e.data as [number, number][]);
      node.port.postMessage(0);
    });
    await ctx.close();
    return { odd, rate: ctx.sampleRate, ticks };
  }, SECONDS);
  skips += result.odd.length;
  lines.push(`run ${String(run).padStart(2)}: ${result.odd.length === 0 ? "no skipped block" : result.odd.map(([at, step]) => `step ${step} frames (${((step - 128) / result.rate * 1000).toFixed(1)} ms missing) at frame ${at}`).join("; ")}  (${result.rate} Hz, ${result.ticks} timer ticks)`);
  await page.close();
}
lines.push(`# ${skips} skips in ${RUNS * SECONDS} s`);
await browser.close();
await server.close();
console.log(lines.join("\n"));

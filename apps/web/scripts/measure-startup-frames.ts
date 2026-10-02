// How the audio context's frame counter moves at the start, as an AudioWorklet sees it: `node scripts/measure-startup-frames.ts`.
// A made-up processor (not the engine) logs `currentFrame` at each `process()` call; this prints the step from one call to the next for
// the first calls of each of several fresh contexts. 128 is the render quantum, and anything else is a jump. Measured on the Chromium
// that playwright finds, headless, with no sound device; what a real device does is spike S1's and S3's to say.
import { launch } from "./browser.ts";
import { serve } from "./serve.ts";

const RUNS = 12;
const CALLS = 12;

const server = await serve();
const browser = await launch();
const lines = [`# currentFrame step between process() calls, first ${CALLS} calls of ${RUNS} fresh contexts (128 is the render quantum)`];
const jumps: number[] = [];
for (let run = 0; run < RUNS; run++) {
  const page = await browser.newPage();
  await page.goto(`${server.url}/pages/host.html`);
  const frames = await page.evaluate(async (calls) => {
    const code = `class P extends AudioWorkletProcessor {
      constructor() { super(); this.log = []; this.port.onmessage = () => this.port.postMessage(this.log); }
      process() { if (this.log.length < ${calls}) this.log.push(currentFrame); return true; }
    }
    registerProcessor("p", P);`;
    const ctx = new AudioContext();
    await ctx.audioWorklet.addModule(URL.createObjectURL(new Blob([code], { type: "text/javascript" })));
    const node = new AudioWorkletNode(ctx, "p");
    node.connect(ctx.destination);
    await ctx.resume();
    await new Promise((r) => setTimeout(r, 300));
    const got = await new Promise<number[]>((resolve) => {
      node.port.onmessage = (e) => resolve(e.data as number[]);
      node.port.postMessage(0);
    });
    await ctx.close();
    return { got, rate: ctx.sampleRate };
  }, CALLS);
  const steps = frames.got.map((f, i) => (i === 0 ? f : f - frames.got[i - 1]!));
  jumps.push(...steps.slice(1).filter((s) => s !== 128));
  lines.push(`run ${String(run).padStart(2)}  first call at frame ${String(steps[0]).padStart(4)}, then steps: ${steps.slice(1).join(" ")}   (${frames.rate} Hz)`);
  await page.close();
}
lines.push(`# steps that were not 128: ${jumps.length === 0 ? "none" : [...new Set(jumps)].sort((a, b) => a - b).join(", ")} (${jumps.length} in ${RUNS} runs)`);
await browser.close();
await server.close();
console.log(lines.join("\n"));

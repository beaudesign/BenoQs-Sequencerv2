// Probe, not a gated test: Stop then Play through the shipped wasm, the way the page drives it (128-frame blocks,
// transport(false) then transport(true)). 16 steps with distinct pitches are programmed with the P5b PIT knob.
import { decodeEvents, RENDER_FRAMES } from "../../../apps/web/src/abi.ts";
import { controlNumbers, matrixId, start } from "../../../apps/web/test/support/module.ts";
const N = controlNumbers();
const num = (id: string) => N.get(id)!;
function build() {
  const e = start(48_000, 1n);
  let t = 0;
  const click = (id: string) => { e.input((t += 20), 0, num(id)); e.input((t += 20), 1, num(id)); };
  for (let s = 0; s < 16; s++) click(matrixId(0, s)); // every step of track 0 on
  for (let s = 1; s < 16; s++) { // the PIT knob gives step s the pitch offset s
    e.input((t += 20), 0, num("mode.step")); click(matrixId(0, s)); e.input((t += 20), 1, num("mode.step"));
    e.input((t += 20), 2, num("edit.enc.pit"), s);
    e.input((t += 20), 0, num("mode.step")); click(matrixId(0, s)); e.input((t += 20), 1, num("mode.step")); // leave zoom (click the same step again is not assumed; see below)
  }
  return e;
}
function runBlocks(e: ReturnType<typeof start>, n: number, log: number[]) {
  for (let i = 0; i < n; i++) { const c = e.render(RENDER_FRAMES); for (const ev of decodeEvents(e.eventBytes(c))) if (ev.kind === 0) log.push(ev.d1 - 69); }
}
// First, does the programming work? (an uninterrupted run should be the ramp 0..15)
{ const e = build(); const log: number[] = []; e.transport(true); runBlocks(e, 200, log); console.log("uninterrupted, first 20 pitch offsets:", log.slice(0, 20).join(" ")); }
let lost = 0, total = 0;
const offsets = [];
for (let k = 330; k < 330 + 12; k++) offsets.push(k); // stop after k blocks: spans one 12-tick step (1500 samples = 11.7 blocks)
for (const k of offsets) {
  const e = build(); const log: number[] = [];
  e.transport(true); runBlocks(e, k, log);
  e.transport(false); runBlocks(e, 40, log);
  e.transport(true); runBlocks(e, 200, log);
  const d = log.slice(1).map((v, i) => ((v - log[i]) % 16 + 16) % 16);
  const skips = d.filter((x) => x === 2).length, other = d.filter((x) => x !== 1 && x !== 2).length;
  total++; if (skips || other) lost++;
  if (k < 334) console.log(`stop after ${k} blocks: pitch offsets around the stop: ${log.slice(0, 40).join(" ")}  skips=${skips} other=${other}`);
}
console.log(`through the wasm: ${lost} of ${total} stop positions lost a note across Stop then Play`);

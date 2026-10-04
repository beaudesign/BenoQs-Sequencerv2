// Probe, not a gated test: Stop then Play through the shipped wasm, the way the page drives it (128-frame blocks, transport(false) then
// transport(true)). Every step of track 0 is switched on from the panel with matrix presses, so it needs nothing the P5b knobs add.
//
// Run from the repo root after `just web-wasm`:  node handoffs/evidence/exact-resume/stop-resume-wasm.ts
//
// The property (crates/octocore/tests/resume.rs, in Node and through the module): the Note Ons after a Play are the uninterrupted run's Note Ons from
// the Stop on, moved by the time stopped. Checked at 24 stop positions across two steps, with a gap of 40 blocks, for three tempos.
import { decodeEvents, RENDER_FRAMES } from "../../../apps/web/src/abi.ts";
import { controlNumbers, matrixId, start } from "../../../apps/web/test/support/module.ts";

const N = controlNumbers();
const num = (id: string) => N.get(id)!;

function build(bpm: number) {
  const e = start(48_000, 1n);
  let t = 0;
  for (let s = 0; s < 16; s++) {
    e.input((t += 20), 0, num(matrixId(0, s)));
    e.input((t += 20), 1, num(matrixId(0, s)));
  }
  e.setTempo(bpm);
  return e;
}

/** Absolute sample of every Note On, rendering `plan` blocks: true is a block with the transport running, false one with it stopped. */
function noteOns(bpm: number, plan: boolean[]): number[] {
  const e = build(bpm);
  const out: number[] = [];
  let running = false;
  plan.forEach((play, i) => {
    if (play !== running) {
      e.transport(play);
      running = play;
    }
    const c = e.render(RENDER_FRAMES);
    for (const ev of decodeEvents(e.eventBytes(c))) if (ev.kind === 0) out.push(i * RENDER_FRAMES + ev.atSample);
  });
  return out;
}

const IDLE = 40;
const TOTAL = 300;
let wrong = 0;
let positions = 0;
let lostNotes = 0;
for (const bpm of [60, 120, 240]) {
  const uninterrupted = noteOns(bpm, new Array(TOTAL).fill(true));
  const stepBlocks = Math.round((12 * 48_000 * 60) / (bpm * 192) / RENDER_FRAMES); // a step, in blocks
  for (let k = 100; k < 100 + 2 * Math.max(stepBlocks, 12); k += Math.max(1, Math.floor((2 * Math.max(stepBlocks, 12)) / 24))) {
    const plan = [...new Array(k).fill(true), ...new Array(IDLE).fill(false), ...new Array(TOTAL - k).fill(true)];
    const got = noteOns(bpm, plan);
    const gap = IDLE * RENDER_FRAMES;
    const want = uninterrupted.filter((s) => s >= k * RENDER_FRAMES && s < TOTAL * RENDER_FRAMES).map((s) => s + gap);
    const after = got.filter((s) => s >= (k + IDLE) * RENDER_FRAMES);
    const before = got.filter((s) => s < k * RENDER_FRAMES);
    const same = JSON.stringify(after) === JSON.stringify(want) && JSON.stringify(before) === JSON.stringify(uninterrupted.filter((s) => s < k * RENDER_FRAMES));
    positions++;
    if (!same) {
      wrong++;
      lostNotes += Math.max(0, want.length - after.length);
      if (wrong <= 3) console.log(`bpm ${bpm}, stop after ${k} blocks: ${after.length} Note Ons after the Play, ${want.length} expected; first got ${after.slice(0, 3)} want ${want.slice(0, 3)}`);
    }
    if (got.some((s) => s >= k * RENDER_FRAMES && s < (k + IDLE) * RENDER_FRAMES)) console.log(`bpm ${bpm}, stop after ${k}: a Note On while stopped`);
  }
}
console.log(`through the wasm: ${wrong} of ${positions} stop positions differ from the uninterrupted run with the gap taken out (${lostNotes} notes missing)`);

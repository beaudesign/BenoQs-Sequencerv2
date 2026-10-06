// Probe, not a gated test: the P5b knobs end to end through the shipped wasm. Panel input (ABI kind 2) -> controller ->
// Command::SetStep -> engine -> the MIDI the page would send. Run from the repo root after `just web-wasm`: node handoffs/evidence/scorecard-2026-10-04/p5b-knobs-wasm.ts
import { decodeEvents, RENDER_FRAMES } from "../../../apps/web/src/abi.ts";
import { controlNumbers, matrixId, start } from "../../../apps/web/test/support/module.ts";

const N = controlNumbers();
const num = (id: string) => { const n = N.get(id); if (n === undefined) throw new Error("no control " + id); return n; };

function session(turns: [string, number][]) {
  const e = start(48_000, 1n);
  let t = 0;
  const click = (id: string) => { e.input((t += 50), 0, num(id)); e.input((t += 50), 1, num(id)); };
  click(matrixId(0, 0)); // Page view: toggle track 0 step 1 on
  if (turns.length) {
    e.input((t += 50), 0, num("mode.step")); // hold Step Mode ...
    click(matrixId(0, 0)); // ... and press the step: Step zoom on it
    e.input((t += 50), 1, num("mode.step"));
    for (const [id, d] of turns) e.input((t += 100), 2, num(id), d);
  }
  e.transport(true);
  const on: { note: number; vel: number; at: number }[] = [];
  const off: number[] = [];
  let frame = 0;
  for (let i = 0; i < 400; i++) {
    const count = e.render(RENDER_FRAMES);
    for (const ev of decodeEvents(e.eventBytes(count))) {
      if (ev.kind === 0) on.push({ note: ev.d1, vel: ev.d2, at: frame + ev.atSample });
      if (ev.kind === 1) off.push(frame + ev.atSample);
    }
    frame += RENDER_FRAMES;
  }
  return { on: on[0], firstOff: off[0], count: on.length };
}
const base = session([]);
console.log("untouched      :", JSON.stringify(base));
const cases: [string, [string, number][]][] = [
  ["PIT +3", [["edit.enc.pit", 3]]],
  ["PIT -5", [["edit.enc.pit", -5]]],
  ["VEL +10", [["edit.enc.vel", 10]]],
  ["VEL -20", [["edit.enc.vel", -20]]],
  ["LEN +12 (ticks)", [["edit.enc.len", 12]]],
  ["STA +3", [["edit.enc.sta", 3]]],
  ["STA +9 (clamps at 5)", [["edit.enc.sta", 9]]],
  ["PIT +2 then +2 in two turns", [["edit.enc.pit", 2], ["edit.enc.pit", 2]]],
];
for (const [name, turns] of cases) {
  const r = session(turns);
  const len = r.firstOff - r.on.at, len0 = base.firstOff - base.on.at;
  console.log(`${name.padEnd(30)}: note ${base.on.note}->${r.on.note} (${r.on.note - base.on.note >= 0 ? "+" : ""}${r.on.note - base.on.note}), velocity ${base.on.vel}->${r.on.vel}, first note-on at sample ${base.on.at}->${r.on.at} (${r.on.at - base.on.at}), length ${len0}->${len} samples (${len - len0})`);
}

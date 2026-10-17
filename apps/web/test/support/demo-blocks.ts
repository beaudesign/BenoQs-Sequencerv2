// The demo as the engine plays it, block by block: the notes the monitor is given, without the worklet around them. Node only.
import { RENDER_FRAMES, decodeEvents } from "../../src/abi.ts";
import { loadDemo } from "../../src/demo.ts";
import type { NoteEvent } from "../../src/monitor.ts";
import { start } from "./module.ts";

/**
 * `seconds` of the page's demo at 120 BPM, as one list of note events per 128-frame block, then `tailSeconds` more after a Stop (whose
 * flush Note Offs come out of the next render, as on the page, so no note is left hanging). Same engine, same seed: the same events every time.
 */
export function demoBlocks(seconds: number, rate = 48_000, tailSeconds = 0): NoteEvent[][] {
  const engine = start(rate, 0n);
  engine.setTempo(120);
  loadDemo({ setTrack: (track, attr, value) => engine.setTrack(track, attr, value), setStep: (track, step, attr, value) => engine.setStep(track, step, attr, value) });
  engine.transport(true);
  const blocks: NoteEvent[][] = [];
  for (let b = 0; b < Math.ceil((seconds * rate) / RENDER_FRAMES); b++) {
    const n = engine.render(RENDER_FRAMES);
    blocks.push(n > 0 ? decodeEvents(engine.eventBytes(n)) : []);
  }
  if (tailSeconds > 0) {
    engine.transport(false);
    for (let b = 0; b < Math.ceil((tailSeconds * rate) / RENDER_FRAMES); b++) {
      const n = engine.render(RENDER_FRAMES);
      blocks.push(n > 0 ? decodeEvents(engine.eventBytes(n)) : []);
    }
  }
  return blocks;
}

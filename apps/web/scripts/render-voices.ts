// The offline renderer (SPEC-0004 S0): a WAV of any voice playing the page's demo, so the sound can be listened to on a phone
// without opening the page. It is the page's own monitor fed the engine's own notes (the demo, at 120 BPM, as the page plays it),
// block by block, with nothing added: what is in the file is what the page makes. Same voice, same seconds: the same bytes.
//   node scripts/render-voices.ts [--voice <id>|all] [--seconds 16] [--out dist/voices]
// Needs `just web-wasm` first (the engine plays the demo).
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { RENDER_FRAMES } from "../src/abi.ts";
import { Monitor, type NoteEvent } from "../src/monitor.ts";
import { VOICES, voiceById } from "../src/voices/registry.ts";
import type { Voice } from "../src/voices/voice.ts";
import { demoBlocks } from "../test/support/demo-blocks.ts";
import { encodeWav } from "./wav.ts";

export const RATE = 48_000;
/** Silence after the last note so that a release is heard to end. */
export const TAIL_SECONDS = 1;

/** The demo's blocks of notes, then a Stop and `TAIL_SECONDS` more, so every note ends and the file ends in silence. */
export function demoWithTail(seconds: number): NoteEvent[][] {
  return demoBlocks(seconds, RATE, TAIL_SECONDS);
}

/** What the page's monitor makes of `blocks` with `voice`. */
export function renderBlocks(voice: Voice, blocks: readonly NoteEvent[][]): Float32Array {
  const monitor = new Monitor(RATE);
  monitor.voice = voice;
  monitor.on = true;
  const out = new Float32Array(blocks.length * RENDER_FRAMES);
  blocks.forEach((events, b) => monitor.render(out.subarray(b * RENDER_FRAMES, (b + 1) * RENDER_FRAMES), events));
  return out;
}

/** Writes `<id>.wav` into `outDir` for each voice; returns the paths. */
export function renderToFiles(voices: readonly Voice[], seconds: number, outDir: string): string[] {
  const blocks = demoWithTail(seconds);
  mkdirSync(outDir, { recursive: true });
  return voices.map((voice) => {
    const path = join(outDir, `${voice.id}.wav`);
    writeFileSync(path, encodeWav(renderBlocks(voice, blocks), RATE));
    return path;
  });
}

/** Reads the arguments; returns the voices, the seconds and the folder, or says what is wrong. */
export function parseArgs(argv: readonly string[]): { voices: Voice[]; seconds: number; outDir: string } {
  let which = "all";
  let seconds = 16;
  let outDir = resolve(dirname(fileURLToPath(import.meta.url)), "../dist/voices");
  for (let i = 0; i < argv.length; i++) {
    const flag = argv[i];
    const value = argv[++i];
    if (value === undefined) throw new Error(`${flag} needs a value`);
    if (flag === "--voice") which = value;
    else if (flag === "--seconds") seconds = Number(value);
    else if (flag === "--out") outDir = resolve(value);
    else throw new Error(`unknown argument ${flag}`);
  }
  if (!Number.isFinite(seconds) || seconds <= 0 || seconds > 120) throw new Error(`--seconds must be between 0 and 120, not ${seconds}`);
  const voices = which === "all" ? [...VOICES] : [voiceById(which)];
  if (voices.some((v) => v === undefined)) throw new Error(`no voice called ${which}; the voices are: ${VOICES.map((v) => v.id).join(", ")}`);
  return { voices: voices as Voice[], seconds, outDir };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const { voices, seconds, outDir } = parseArgs(process.argv.slice(2));
    for (const path of renderToFiles(voices, seconds, outDir)) console.log(path);
  } catch (e) {
    console.error(String(e instanceof Error ? e.message : e));
    process.exit(1);
  }
}

// WebAssembly smoke test (SPEC-0001 O7, WENGE-0007).
//
// Loads octoffi.wasm (the C ABI of the sequencer core built for wasm32-unknown-unknown, no
// imports), programs a pattern through the same FFI setters a host would use, renders it in
// the pattern's own buffer size, writes the event log in octorun's format and checks its
// SHA-256 against the native golden in examples/golden/. So the same pattern gives the same
// bytes in a browser engine as in the native build.
//
// Only the FFI surface is used, so it runs the patterns that need nothing beyond it:
// tracks, steps, roles, channels. Patterns with phrases or MCC (no FFI setter yet) are
// reported as skipped, never as passed.
//
// usage: node harness/wasm/smoke.mjs [wasm-file]   (just wasm-smoke builds it first)

import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const wasmPath = process.argv[2] ?? join(root, "target/wasm32-unknown-unknown/release/octoffi.wasm");
const PATTERNS = ["hello", "chords_and_strums", "effector", "phrases", "mcc_and_transport"];

// OctoTrackAttr and OctoStepAttr, in the order of octoffi.h.
const TRACK = { pitch: 0, vel: 1, mch: 8, dir: 4, grv: 7 };
const ROLE = { feeder: 13, listener: 14 };
const STEP = { active: 0, skip: 1, pit: 2, vel: 3, len: 4, lenmul: 5, sta: 6, amt: 7, strum: 8, phrase: 10 };

const { instance } = await WebAssembly.instantiate(readFileSync(wasmPath), {});
const x = instance.exports;

class Unsupported extends Error {}

function runPattern(name) {
  const source = readFileSync(join(root, "examples", `${name}.pattern`), "utf8");
  let seed = 0n, bpm = 120, sampleRate = 48000, playing = false, clock = 0;
  const events = [];
  const tempo = [[0, 120]];

  // The engine's own allocation happens in octocore_engine_new. Scratch memory for the calls
  // comes from pages grown after that, which nothing in wasm-land allocates from again.
  let engine = null, scratch = 0, paramsPtr, outPtr, countPtr, cmdPtr;
  const ensureEngine = () => {
    if (engine !== null) return;
    engine = x.octocore_engine_new(seed);
    const before = x.memory.buffer.byteLength;
    x.memory.grow(1);
    scratch = before;
    paramsPtr = scratch; // OctoRenderParams: f32, u32, f32, bool (16 bytes)
    countPtr = scratch + 16;
    cmdPtr = scratch + 32; // Command: tag u32 then payload, 32 bytes is plenty
    outPtr = scratch + 128; // 256 events x 12 bytes
  };

  const num = (t) => Number(t.replace(/^\+/, ""));
  for (const raw of source.split("\n")) {
    const line = raw.trim();
    if (!line || line.startsWith("#")) continue;
    const t = line.split(/\s+/);
    const fail = () => { throw new Unsupported(`${name}: \`${line}\` needs more than the FFI offers`); };
    if (t[0] === "expect") continue; // the native run checks these
    if (t[0] === "seed") { seed = BigInt(t[1]); continue; }
    if (t[0] === "bpm") {
      bpm = num(t[1]);
      if (!Number.isInteger(bpm)) fail();
      if (tempo[tempo.length - 1][0] === clock) tempo[tempo.length - 1][1] = bpm; else tempo.push([clock, bpm]);
      continue;
    }
    if (t[0] === "samplerate") { sampleRate = num(t[1]); continue; }
    if (t[0] === "play" || t[0] === "stop") {
      ensureEngine();
      const dv = new DataView(x.memory.buffer);
      for (let i = 0; i < 32; i += 4) dv.setUint32(cmdPtr + i, 0, true);
      dv.setUint32(cmdPtr, t[0] === "play" ? 0 : 1, true); // OCTO_CMD_PLAY = 0, OCTO_CMD_STOP = 1
      x.octocore_engine_handle_command(engine, cmdPtr);
      playing = t[0] === "play";
      continue;
    }
    if (t[0] === "track") {
      ensureEngine();
      const tr = Number(t[1]);
      if (t[2] === "step") {
        const attr = STEP[t[4]];
        if (attr === undefined || t[4] === "phrase") fail();
        const steps = t[3] === "all" ? [...Array(16).keys()] : [Number(t[3])];
        for (const s of steps) if (!x.octocore_step_set_i32(engine, tr, s, attr, num(t[5]))) fail();
      } else if (t[2] === "role") {
        const isFeeder = t[3] === "feeder" || t[3] === "both", isListener = t[3] === "listener" || t[3] === "both";
        x.octocore_track_set_i32(engine, tr, ROLE.feeder, isFeeder ? 1 : 0);
        x.octocore_track_set_i32(engine, tr, ROLE.listener, isListener ? 1 : 0);
      } else if (t[2] in TRACK && t[2] !== "pitch") {
        if (!x.octocore_track_set_i32(engine, tr, TRACK[t[2]], num(t[3]))) fail();
      } else fail();
      continue;
    }
    if (t[0] === "render") {
      ensureEngine();
      const bufferLen = Number(t[3].replace("buffer=", ""));
      const count = t[2] === "s" ? Math.ceil((Number(t[1]) * sampleRate) / bufferLen) : Number(t[1]);
      for (let i = 0; i < count; i++) {
        const dv = new DataView(x.memory.buffer);
        dv.setFloat32(paramsPtr, sampleRate, true);
        dv.setUint32(paramsPtr + 4, bufferLen, true);
        dv.setFloat32(paramsPtr + 8, bpm, true);
        dv.setUint8(paramsPtr + 12, playing ? 1 : 0);
        dv.setUint32(countPtr, 0, true);
        if (x.octocore_engine_render(engine, paramsPtr, outPtr, 256, countPtr) !== 0) throw new Error("render failed");
        const n = dv.getUint32(countPtr, true);
        for (let e = 0; e < n; e++) {
          const p = outPtr + e * 12;
          const at = clock + dv.getUint32(p + 8, true);
          const [port, ch, b6, b7] = [dv.getUint8(p + 4), dv.getUint8(p + 5), dv.getUint8(p + 6), dv.getUint8(p + 7)];
          const tag = dv.getUint32(p, true);
          events.push(
            tag === 0 ? `{"t":${at},"kind":"on","port":${port},"ch":${ch},"note":${b6},"vel":${b7}}`
            : tag === 1 ? `{"t":${at},"kind":"off","port":${port},"ch":${ch},"note":${b6}}`
            : tag === 2 ? `{"t":${at},"kind":"cc","port":${port},"ch":${ch},"cc":${b6},"val":${b7}}`
            : tag === 3 ? `{"t":${at},"kind":"bend","port":${port},"ch":${ch},"value":${dv.getUint16(p + 6, true)}}`
            : `{"t":${at},"kind":"pressure","port":${port},"ch":${ch},"value":${b6}}`
          );
        }
        clock += bufferLen;
      }
      continue;
    }
    fail(); // phrase, page.tracks, flatten, clear and the rest
  }
  const header = `{"octorun":"events/1","sample_rate":${sampleRate},"samples":${clock},"tempo":[${tempo.map(([a, b]) => `[${a},${b}]`).join(",")}]}`;
  x.octocore_engine_free(engine);
  return { log: header + "\n" + events.map((e) => e + "\n").join(""), count: events.length };
}

let failed = 0, ran = 0;
for (const name of PATTERNS) {
  const golden = readFileSync(join(root, "examples", "golden", `${name}.sha256`), "utf8").split("\n")[0].replace("events ", "");
  try {
    const t0 = performance.now();
    const { log, count } = runPattern(name);
    const ms = (performance.now() - t0).toFixed(0);
    const got = createHash("sha256").update(log).digest("hex");
    ran++;
    if (got === golden) console.log(`ok       ${name}: ${count} events, identical to the native golden (${ms} ms in wasm)`);
    else { failed++; console.log(`MISMATCH ${name}: wasm ${got}\n                 native ${golden}`); }
  } catch (e) {
    if (e instanceof Unsupported) console.log(`skipped  ${e.message}`);
    else { failed++; console.log(`ERROR    ${name}: ${e.stack}`); }
  }
}
console.log(`${ran} pattern(s) compared, ${failed} failed`);
if (failed > 0 || ran < 3) process.exit(1);

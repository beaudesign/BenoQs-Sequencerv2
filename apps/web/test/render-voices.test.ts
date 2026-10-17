// The offline renderer (SPEC-0004 S0): a WAV of any voice playing the demo. The file format, that Plain's file is what the page
// has always played, that it is the same bytes twice, and the command line.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, readdirSync, rmSync, statSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { RENDER_FRAMES } from "../src/abi.ts";
import { PLAIN } from "../src/voices/plain.ts";
import { RATE, TAIL_SECONDS, demoWithTail, parseArgs, renderBlocks, renderToFiles } from "../scripts/render-voices.ts";
import { encodeWav } from "../scripts/wav.ts";
import { webRoot } from "./support/module.ts";
import { MonitorV0 } from "./support/monitor-v0.ts";

const tmp = (): string => mkdtempSync(join(tmpdir(), "wenge-voices-"));

test("WAV: the header is a mono 16-bit PCM file at the rate given, and its sizes add up", () => {
  const wav = encodeWav([0, 1, -1, 0.5, 2, -2], 44_100);
  const v = new DataView(wav.buffer);
  const text = (at: number): string => String.fromCharCode(...wav.subarray(at, at + 4));
  assert.deepEqual([text(0), text(8), text(12), text(36)], ["RIFF", "WAVE", "fmt ", "data"]);
  assert.equal(v.getUint32(4, true), wav.length - 8);
  assert.deepEqual([v.getUint32(16, true), v.getUint16(20, true), v.getUint16(22, true), v.getUint32(24, true), v.getUint32(28, true), v.getUint16(32, true), v.getUint16(34, true)], [16, 1, 1, 44_100, 88_200, 2, 16]);
  assert.equal(v.getUint32(40, true), 12);
  assert.equal(wav.length, 44 + 12);
  assert.deepEqual([0, 1, 2, 3, 4, 5].map((i) => v.getInt16(44 + 2 * i, true)), [0, 32767, -32767, 16384, 32767, -32768], "full scale, half scale, and clipping");
});

test("renderer: Plain's file is the frozen monitor's output on the demo, rounded to 16 bits and no more", () => {
  const blocks = demoWithTail(4);
  const mine = renderBlocks(PLAIN, blocks);
  const frozen = new MonitorV0(RATE);
  frozen.on = true;
  const reference = new Float32Array(blocks.length * RENDER_FRAMES);
  blocks.forEach((e, b) => frozen.render(reference.subarray(b * RENDER_FRAMES, (b + 1) * RENDER_FRAMES), e));
  assert.deepEqual(mine, reference);
  const peak = mine.reduce((m, x) => Math.max(m, Math.abs(x)), 0);
  assert.ok(peak > 0.05 && peak <= 1, `peak ${peak}`);
  assert.ok(mine.subarray(mine.length - (TAIL_SECONDS * RATE) / 2).every((x) => x === 0), "and it ends in silence");
});

test("renderer: the file is 16-bit mono at 48 kHz, as long as the demo and its tail, and the same bytes twice", () => {
  const dir = tmp();
  try {
    const [a] = renderToFiles([PLAIN], 2, join(dir, "a"));
    const [b] = renderToFiles([PLAIN], 2, join(dir, "b"));
    const bytes = readFileSync(a!);
    assert.equal(bytes.length, 44 + 2 * (demoWithTail(2).length * RENDER_FRAMES));
    assert.equal(bytes.readUInt32LE(24), 48_000);
    assert.equal(createHash("sha256").update(bytes).digest("hex"), createHash("sha256").update(readFileSync(b!)).digest("hex"));
    assert.equal(a!.endsWith("plain.wav"), true);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("renderer: the arguments, and what is said when they are wrong", () => {
  assert.deepEqual(parseArgs(["--voice", "plain", "--seconds", "3", "--out", "/tmp/x"]), { voices: [PLAIN], seconds: 3, outDir: "/tmp/x" });
  assert.equal(parseArgs([]).seconds, 16);
  assert.ok(parseArgs([]).voices.includes(PLAIN), "all is the default");
  assert.throws(() => parseArgs(["--voice", "nope"]), /no voice called nope; the voices are: plain/);
  assert.throws(() => parseArgs(["--seconds", "0"]), /--seconds/);
  assert.throws(() => parseArgs(["--seconds"]), /needs a value/);
  assert.throws(() => parseArgs(["--loud", "1"]), /unknown argument --loud/);
});

test("renderer: run as a command it writes the file and prints its path; a wrong voice exits 1 and says why", () => {
  const dir = tmp();
  try {
    const run = (args: string[]) => spawnSync(process.execPath, ["scripts/render-voices.ts", ...args], { cwd: webRoot, encoding: "utf8" });
    const ok = run(["--voice", "plain", "--seconds", "1", "--out", dir]);
    assert.equal(ok.status, 0, ok.stderr);
    assert.equal(ok.stdout.trim(), join(dir, "plain.wav"));
    assert.deepEqual(readdirSync(dir), ["plain.wav"]);
    assert.ok(statSync(join(dir, "plain.wav")).size > 44);
    const bad = run(["--voice", "nope", "--out", dir]);
    assert.equal(bad.status, 1);
    assert.match(bad.stderr, /no voice called nope/);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

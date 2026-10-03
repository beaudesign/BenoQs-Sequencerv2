// What the owner reads: a saved S3 file, read again from its raw records and put in words. The data is made up and says so; the point is
// that the report does not lose what the analysis found (the stamps on another clock, the hidden tab, no Start) on the way to the page.
import assert from "node:assert/strict";
import { test } from "node:test";
import { analyseFollow, analyseLoop, analyseStampMap, type FollowSample } from "../spikes/s3/analyse.ts";
import { formatFollow, formatSend, readSaved, reanalyse } from "../spikes/s3/format.ts";
import type { FollowResult, SendResult } from "../spikes/s3/page.ts";

const BUILD = { abi: 2, wasmBytes: 184_056, sha256: "0123456789abcdef".repeat(4) };

function followResult(over: { age?: number; hiddenFrom?: number; phaseMs?: number; noStart?: boolean; noMap?: boolean; mapJump?: number } = {}): FollowResult {
  const pulses: [number, number][] = [];
  for (let i = 0; i < 1_440; i++) pulses.push([10_000 + i * (2500 / 120), 10_000 + i * (2500 / 120) + (over.age ?? 2)]);
  const samples: FollowSample[] = [];
  for (let i = 0; i < 240; i++) {
    samples.push({ at: 10_000 + i * 250, phase: "following", bpm: 120, phaseMs: over.phaseMs ?? 1.5, jitterMs: 0.4, pulses: i * 6, running: !over.noStart, hidden: over.hiddenFrom !== undefined && i >= over.hiddenFrom, ...(over.noMap ? {} : { mapMs: 100 + (over.mapJump !== undefined && i >= over.mapJump ? 10 : 0) }) });
  }
  const messages = over.noStart ? [] : [{ name: "start" as const, stamp: 10_000, seen: 10_001 }];
  const asRaw = { pulses: pulses.map(([stamp, seen]) => ({ stamp, seen })), samples, messages };
  return {
    schema: "wenge.s3.follow/1",
    startedAt: "2026-10-02T12:00:00.000Z",
    config: { minutes: 1, inputId: "in", lookaheadMs: 30, offsetMs: 0, nominalBpm: 120 },
    environment: { userAgent: "Test/1", platform: "Linux", audioSampleRate: 48_000, baseLatencyMs: 10.7, outputLatencyMs: 0 },
    port: "Made-up port",
    durationS: 60,
    hiddenS: 0,
    visibility: [{ atS: 0, state: "visible" }],
    input: { received: 1_441, decoded: 1_441, ignored: 0, malformed: 0, backwards: 0, byKind: { noteOn: 0, noteOff: 0, controller: 0, bend: 0, pressure: 0, program: 0, realtime: 1_441 }, realtime: { clock: 1_440, start: 1, continue: 0, stop: 0 }, lastTimeStamp: 70_000 },
    worklet: { errors: [] },
    tuning: { trimTimeMs: 250, maxTrim: 0.08, minTempoChange: 0.01 },
    raw: { pulses, samples, messages },
    analysis: { all: analyseFollow(asRaw, { nominalBpm: 120 }), visible: analyseFollow(asRaw, { nominalBpm: 120, hidden: false }), hidden: analyseFollow(asRaw, { nominalBpm: 120, hidden: true }) },
  };
}

function sendResult(): SendResult {
  const sent = [0, 1, 2, 3].map((i) => ({ target: 1_000 + i * 20.833, audioTime: i * 0.0208, marginMs: 28, hidden: false }));
  const heard: [number, number][] = sent.map((s) => [s.target + 3, s.target + 3.5]);
  return {
    schema: "wenge.s3.send/1",
    startedAt: "2026-10-02T12:00:00.000Z",
    config: { minutes: 1, bpm: 120, outputId: "out", inputId: "in", lookaheadMs: 30 },
    environment: { userAgent: "Test/1", platform: "Linux", audioSampleRate: 48_000, baseLatencyMs: 10.7, outputLatencyMs: 0 },
    ports: { output: "Made-up out", input: "Made-up in" },
    durationS: 60,
    hiddenS: 0,
    visibility: [{ atS: 0, state: "visible" }],
    scheduler: { sent: 0, late: 0, lateMaxMs: 0, unrouted: 0, realtime: 4, invalid: 0, raised: 0, raisedMaxMs: 0 },
    worklet: { errors: [] },
    transportSent: [1, 0, 1],
    transportHeard: [1, 0, 0],
    marginMs: { min: 24, p50: 28, late: 0 },
    loop: analyseLoop(sent, heard.map(([at]) => ({ at }))),
    stampMap: analyseStampMap(sent, 30),
    heardAsClock: analyseFollow({ pulses: heard.map(([stamp, seen]) => ({ stamp, seen })), samples: [], messages: [] }),
    raw: { sent, heard },
  };
}

test("a saved file is read as the page made it or as the runner wrapped it, and anything else is refused by saying what it saw", () => {
  const plain = readSaved(JSON.stringify(followResult()));
  assert.equal(plain.simulated, undefined);
  assert.equal(plain.result.schema, "wenge.s3.follow/1");
  const wrapped = readSaved(JSON.stringify({ simulated: "a timer in the page", commit: "abc", browser: "Chromium", result: followResult() }));
  assert.equal(wrapped.simulated, "a timer in the page");
  assert.equal(wrapped.commit, "abc");
  assert.throws(() => readSaved(JSON.stringify({ schema: "wenge.s1/1" })), /wenge\.s1\/1/);
  assert.throws(() => readSaved("not json"), /JSON/);
});

test("the report of a follow run says first that it is simulated when the file does, and carries the clock domain, the phase and the verdict", () => {
  const lines = formatFollow({ simulated: "a timer in the page, not Live and not a MIDI port", result: followResult() });
  assert.match(lines[0]!, /SIMULATED/);
  assert.match(lines[0]!, /not Live/);
  const text = lines.join("\n");
  assert.match(text, /same-clock/);
  assert.match(text, /counted \d+/);
  assert.match(text, /beyond a fifth/);
  assert.match(text, /All \d+ samples were within a fifth of a step/);
  assert.match(text, /shorter than the 30 minutes/);
  assert.match(text, /less the nominal 120 BPM/, "the tempo the file was run against is compared");
  assert.doesNotMatch(formatFollow({ result: followResult() })[0]!, /SIMULATED/);
});

test("the report keeps what went wrong: stamps on another clock, a phase out of bounds, no Start", () => {
  assert.match(formatFollow({ result: followResult({ age: 1_000_000 }) }).join("\n"), /NOT ON THE PAGE'S CLOCK/);
  assert.match(formatFollow({ result: followResult({ phaseMs: 9 }) }).join("\n"), /\d+ of \d+ samples were beyond/);
  assert.match(formatFollow({ result: followResult({ noStart: true }) }).join("\n"), /No phase was counted/);
});

test("the report says how the browser's clock map moved, and that a file without one did not record it", () => {
  const steady = formatFollow({ result: followResult() }).join("\n");
  assert.match(steady, /Clock map \(the browser's audio-to-page offset, read at each sample\): 0 jumps/);
  const jumped = formatFollow({ result: followResult({ mapJump: 100 }) }).join("\n");
  assert.match(jumped, /1 jumps of more than 3 ms between samples, the largest 10\.00 ms/);
  assert.match(formatFollow({ result: followResult({ noMap: true }) }).join("\n"), /not recorded in this file/);
});

test("the saved raw records are read again: other settling, and the hidden samples apart from the visible", () => {
  const r = followResult({ hiddenFrom: 120, phaseMs: 2 });
  const all = reanalyse(r, {});
  const visible = reanalyse(r, { hidden: false });
  const hidden = reanalyse(r, { hidden: true });
  assert.equal(all.phase.counted, visible.phase.counted + hidden.phase.counted);
  assert.ok(hidden.phase.counted > 0 && visible.phase.counted > 0);
  assert.ok(reanalyse(r, { settleMs: 30_000 }).phase.counted < all.phase.counted, "a longer settling counts fewer");
  assert.equal(reanalyse(r, { nominalBpm: 119 }).tempo.meanErrorBpm! > 0.9, true, "a nominal tempo of 119 against 120 read");
});

test("the report of a send run: what was sent and heard, the transport messages, and the clock as it came back", () => {
  const text = formatSend({ simulated: "a timer in the page", result: sendResult() }).join("\n");
  assert.match(text, /^SIMULATED/);
  assert.match(text, /sent 4, heard 4, matched 4, missing 0, extra 0/);
  assert.match(text, /Start 1 sent, 1 heard/);
  assert.match(text, /Stop 1 sent, 0 heard/);
  assert.match(text, /as a clock/);
  assert.match(text, /Stamp map .*engine's own tempo from its audio times is \d+\.\d{3} BPM; 0 jumps/);
});

test("a run whose tab was hidden throughout has nothing visible to read, and is read as a whole, with the hidden time said", () => {
  const r = followResult({ hiddenFrom: 0, phaseMs: 2 });
  r.hiddenS = 60;
  const text = formatFollow({ result: r }).join("\n");
  assert.match(text, /All \d+ samples were within a fifth of a step/);
  assert.match(text, /hidden for 60 s of 60/);
  assert.match(text, /tab hidden: counted \d+/);
});

test("a send run that lost a pulse says that the pairing by order cannot be trusted after it", () => {
  const r = sendResult();
  r.loop = analyseLoop(r.raw.sent, r.raw.heard.slice(0, 3).map(([at]) => ({ at })));
  assert.equal(r.loop.missing, 1);
  assert.match(formatSend({ result: r }).join("\n"), /missing 1.*cannot be trusted/);
  assert.doesNotMatch(formatSend({ result: sendResult() }).join("\n"), /cannot be trusted/);
});

test("the report names the engine build the file was made with, next to the environment, for a follow run and a send run", () => {
  const follow = formatFollow({ result: { ...followResult(), engine: BUILD } });
  const send = formatSend({ result: { ...sendResult(), engine: BUILD } });
  for (const lines of [follow, send]) {
    const at = lines.findIndex((l) => l.startsWith("Engine build:"));
    assert.ok(at >= 0 && at <= 3, `the build line is at ${at}, with the other lines about where and what the run was`);
    assert.equal(lines[at], `Engine build: octoweb-abi/2, octoweb.wasm ${BUILD.wasmBytes} bytes, sha256 ${BUILD.sha256}`);
  }
});

test("a file with no build in it (saved before this was recorded) says so and is still read; a build with no hash says why there is none", () => {
  for (const lines of [formatFollow({ result: followResult() }), formatSend({ result: sendResult() })]) {
    assert.ok(lines.includes("Engine build: not recorded in this file (it was saved before the page recorded it)"), lines.join("\n"));
  }
  const noHash = formatFollow({ result: { ...followResult(), engine: { ...BUILD, sha256: null } } }).join("\n");
  assert.match(noHash, /Engine build: octoweb-abi\/2, octoweb\.wasm 184056 bytes, sha256 not computed \(the page was not served from a secure context: use localhost\)/);
});

test("the build survives the save and the read, in a file the page wrote and in one the runner wrapped", () => {
  assert.deepEqual((readSaved(JSON.stringify({ ...followResult(), engine: BUILD })).result as FollowResult).engine, BUILD);
  assert.deepEqual((readSaved(JSON.stringify({ simulated: "x", commit: "abc", result: { ...sendResult(), engine: BUILD } })).result as SendResult).engine, BUILD);
});

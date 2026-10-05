// The metronome in the real worklet (src/worklet.ts in the stand-in scope of test/support/worklet-scope.ts): the click it writes to its
// output follows the transport and the tempo. specs/SPEC-0002/p5d-metronome-click.md, observables C4 to C7.
//
// "Follows the MIDI" is tested against the engine's own MIDI clock where it matters: a click is one quarter note, 24 pulses, so the click
// of beat `k` is at the audio frame of clock pulse `24 * k`, whatever the tempo did on the way.
import assert from "node:assert/strict";
import { test } from "node:test";
import { KIND_DOWN, KIND_REALTIME, KIND_UP, RENDER_FRAMES, decodeEvents } from "../src/abi.ts";
import type { FromWorklet } from "../src/protocol.ts";
import { controlNumbers, matrixId } from "./support/module.ts";
import { WorkletRig, type Posted } from "./support/worklet-scope.ts";

const numbers = controlNumbers();
const key = (id: string): number => {
  const v = numbers.get(id);
  assert.ok(v !== undefined, id);
  return v;
};

function only<T extends FromWorklet["type"]>(posted: Posted[], type: T): Extract<FromWorklet, { type: T }>[] {
  return posted.map((p) => p.message).filter((m): m is Extract<FromWorklet, { type: T }> => m.type === type);
}

/** The first sample of every click: a sample that is not silence, after at least `quiet` frames of exact silence. */
function onsets(audio: Float32Array, quiet = 3000): number[] {
  const out: number[] = [];
  let silent = quiet; // the start of the run counts as quiet
  for (let i = 0; i < audio.length; i++) {
    if (audio[i] === 0) silent++;
    else {
      if (silent >= quiet) out.push(i);
      silent = 0;
    }
  }
  return out;
}

/** Runs `blocks` blocks, keeping what was posted. */
function run(rig: WorkletRig, blocks: number, posted: Posted[] = []): Posted[] {
  for (let b = 0; b < blocks; b++) posted.push(...rig.block());
  return posted;
}

const secondsToBlocks = (rig: WorkletRig, s: number): number => Math.ceil((s * rig.sampleRate) / RENDER_FRAMES);

/** The audio frame of every MIDI clock pulse, in order. */
function pulseFrames(posted: Posted[]): number[] {
  const out: number[] = [];
  for (const m of only(posted, "events")) for (const e of decodeEvents(new Uint8Array(m.bytes))) if (e.kind === KIND_REALTIME && e.d1 === 0xf8) out.push(m.frame + e.atSample);
  return out;
}

function near(got: number, want: number, within = 1, what = ""): void {
  assert.ok(Math.abs(got - want) <= within, `${what} ${got} against ${want} (within ${within})`);
}

test("C4: off, the worklet's output is silence for as long as it plays", () => {
  const rig = new WorkletRig();
  rig.send({ type: "tempo", bpm: 120 });
  rig.send({ type: "transport", play: true });
  run(rig, secondsToBlocks(rig, 2.2));
  const audio = rig.audio();
  assert.ok(audio.length > 100_000);
  assert.ok(audio.every((x) => x === 0));
});

test("C4: on and playing at 120 BPM, a click on every quarter: at the Play, then every 24000 frames at 48 kHz", () => {
  const rig = new WorkletRig();
  rig.send({ type: "metronome", on: true });
  rig.send({ type: "tempo", bpm: 120 });
  run(rig, 3);
  const playFrame = rig.frame;
  rig.send({ type: "transport", play: true });
  run(rig, secondsToBlocks(rig, 2.3));
  const found = onsets(rig.audio());
  assert.ok(found.length >= 5, `${found}`);
  found.slice(0, 5).forEach((f, k) => near(f, playFrame + 24_000 * k, 1, `click ${k}`));
  assert.equal(found[0], playFrame, "the first click is the first frame of the Play: nothing earlier, no wait for the second beat");
  for (const x of rig.audio()) assert.ok(Math.abs(x) < 0.5, "never loud");
});

test("C4: the spacing is a quarter note at the tempo and at the sample rate: 44.1 kHz at 90 BPM", () => {
  const rig = new WorkletRig({ sampleRate: 44_100 });
  rig.send({ type: "metronome", on: true });
  rig.send({ type: "tempo", bpm: 90 });
  rig.send({ type: "transport", play: true });
  run(rig, secondsToBlocks(rig, 3));
  const found = onsets(rig.audio());
  assert.ok(found.length >= 4, `${found}`);
  const gap = (60 / 90) * 44_100;
  found.slice(0, 4).forEach((f, k) => near(f, gap * k, 1, `click ${k}`));
});

test("C4: switched on while stopped, it waits: no click while stopped, and the Play starts with one", () => {
  const rig = new WorkletRig();
  rig.send({ type: "metronome", on: true });
  run(rig, 300);
  assert.ok(rig.audio().every((x) => x === 0), "stopped: silence");
  const playFrame = rig.frame;
  rig.send({ type: "transport", play: true });
  run(rig, 40);
  assert.equal(onsets(rig.audio())[0], playFrame);
});

test("C4: switched on in the middle of a run, the first click is the next beat and not at once", () => {
  const rig = new WorkletRig();
  rig.send({ type: "tempo", bpm: 120 });
  const playFrame = rig.frame;
  rig.send({ type: "transport", play: true });
  run(rig, 50); // 6400 frames in, a quarter of the way to beat 1
  rig.send({ type: "metronome", on: true });
  run(rig, secondsToBlocks(rig, 1.2));
  const found = onsets(rig.audio());
  assert.ok(found.length >= 2, `${found}`);
  near(found[0]!, playFrame + 24_000, 1, "the first click is beat 1");
  near(found[1]!, playFrame + 48_000, 1, "and then the one after");
});

test("C4: switched off, the click that is ringing finishes, and there is no other", () => {
  const rig = new WorkletRig();
  rig.send({ type: "metronome", on: true });
  rig.send({ type: "transport", play: true });
  run(rig, 4);
  rig.send({ type: "metronome", on: false });
  run(rig, secondsToBlocks(rig, 1.2));
  const kept = rig.audio();
  const other = new WorkletRig();
  other.send({ type: "metronome", on: true });
  other.send({ type: "transport", play: true });
  run(other, 4 + secondsToBlocks(other, 1.2));
  const whole = other.audio();
  const tail = Math.ceil(0.04 * 48_000);
  assert.deepEqual(kept.subarray(0, tail), whole.subarray(0, tail), "the first click is whole, off or on");
  assert.ok(kept.subarray(tail).every((x) => x === 0), "and then it is silent");
  assert.equal(onsets(whole).length >= 2, true, "while the one left on goes on clicking");
});

test("C5: Stop silences it. Play carries on from the point it stopped, the same distance from the next beat", () => {
  const rig = new WorkletRig();
  rig.send({ type: "metronome", on: true });
  rig.send({ type: "tempo", bpm: 120 });
  const firstPlay = rig.frame;
  rig.send({ type: "transport", play: true });
  run(rig, 235); // 30080 frames: beat 0 and beat 1 have sounded, and the audio is at tick 240.64
  const stopFrame = rig.frame;
  rig.send({ type: "transport", play: false });
  const stopped = only(rig.take(), "panel").at(-1)!;
  near(stopped.position, (stopFrame - firstPlay) / 125, 0.5, "the audio's tick position at the Stop");
  run(rig, secondsToBlocks(rig, 1.5));
  const beforePlay = onsets(rig.audio());
  assert.deepEqual(beforePlay.length, 2, `two clicks before the Stop and none after: ${beforePlay}`);
  const playFrame = rig.frame;
  rig.send({ type: "transport", play: true });
  const resumed = only(rig.take(), "panel").at(-1)!;
  near(resumed.position, stopped.position, 1e-9, "Play does not move the position");
  run(rig, secondsToBlocks(rig, 0.9));
  const all = onsets(rig.audio());
  assert.ok(all.length >= 3, `${all}`);
  // Beat 2 is at tick 384; the Play is at tick 240.64, so it is 143.36 ticks of 125 frames on.
  near(all[2]!, playFrame + (384 - stopped.position) * 125, 1, "the third click");
});

test("C5: a Stop and Play on a beat's exact frame does not lose the beat and does not click it twice", () => {
  // The Stop lands where the audio is exactly on tick 384, beat 2: the beat is not in the block that ended there, so it is the Play's first frame.
  const rig = new WorkletRig();
  rig.send({ type: "metronome", on: true });
  rig.send({ type: "tempo", bpm: 120 });
  rig.send({ type: "transport", play: true });
  run(rig, 375); // 48000 frames: the audio is on tick 384 (beat 2), at a block edge
  rig.send({ type: "transport", play: false });
  const stopped = only(rig.take(), "panel").at(-1)!;
  near(stopped.position, 384, 0.01, "the Stop is on beat 2");
  assert.equal(onsets(rig.audio()).length, 2, `beats 0 and 1 have sounded, and beat 2 has not: ${onsets(rig.audio())}`);
  run(rig, 100);
  const playFrame = rig.frame;
  rig.send({ type: "transport", play: true });
  run(rig, 40);
  const all = onsets(rig.audio());
  assert.equal(all.length, 3);
  assert.equal(all[2], playFrame, "beat 2 sounds at the Play's first frame");
});

test("C6: the click follows the MIDI clock: it is at the frame of pulse 24 k, at 240 BPM as at 120, and through a change of tempo", () => {
  const rig = new WorkletRig();
  rig.send({ type: "metronome", on: true });
  rig.send({ type: "clock", master: true });
  rig.send({ type: "tempo", bpm: 120 });
  const posted: Posted[] = [];
  rig.send({ type: "transport", play: true });
  run(rig, 150, posted);
  rig.send({ type: "tempo", bpm: 240 });
  run(rig, 150, posted);
  rig.send({ type: "tempo", bpm: 77 });
  run(rig, 400, posted);
  rig.send({ type: "tempo", bpm: 180 });
  run(rig, 400, posted);
  const pulses = pulseFrames(posted);
  const found = onsets(rig.audio(), 1500);
  assert.ok(found.length >= 6, `${found}`);
  found.forEach((f, k) => near(f, pulses[24 * k]!, 1, `click ${k} against pulse ${24 * k}`));
});

test("C6: tempo 240 puts the clicks half as far apart as 120", () => {
  const gap = (bpm: number): number => {
    const rig = new WorkletRig();
    rig.send({ type: "metronome", on: true });
    rig.send({ type: "tempo", bpm });
    rig.send({ type: "transport", play: true });
    run(rig, secondsToBlocks(rig, 3));
    const f = onsets(rig.audio(), 1500);
    return f[2]! - f[1]!;
  };
  near(gap(240) * 2, gap(120), 2, "twice the gap");
});

test("C7: the click is not MIDI: the events of a run are the same bytes, at the same frames, with it on and off", () => {
  const events = (on: boolean): string[] => {
    const rig = new WorkletRig({ seed: 7n });
    rig.send({ type: "clock", master: true });
    for (const s of [0, 3, 5, 8, 12]) {
      rig.send({ type: "input", nowMs: 0, kind: KIND_DOWN, control: key(matrixId(0, s)), detents: 0 });
      rig.send({ type: "input", nowMs: 50, kind: KIND_UP, control: key(matrixId(0, s)), detents: 0 });
    }
    rig.send({ type: "tempo", bpm: 150 });
    if (on) rig.send({ type: "metronome", on: true });
    rig.send({ type: "transport", play: true });
    const posted = run(rig, 700);
    rig.send({ type: "transport", play: false });
    run(rig, 50, posted);
    return only(posted, "events").map((m) => `${m.frame}:${[...new Uint8Array(m.bytes)].join(",")}`);
  };
  const off = events(false);
  assert.ok(off.length > 20, "there is something to compare");
  assert.deepEqual(events(true), off);
});

test("a processor called with no output at all, as a browser may call a node that has none, still renders and does not throw", () => {
  const rig = new WorkletRig();
  rig.send({ type: "metronome", on: true });
  rig.send({ type: "transport", play: true });
  assert.equal(rig.processor.process(), true);
  assert.equal(rig.processor.process([], []), true);
  assert.equal(rig.processor.process([], [[]]), true, "an output with no channel");
  assert.deepEqual(only(rig.take(), "error"), []);
});

test("a metronome message is audio the worklet makes and nothing the panel shows: it posts nothing", () => {
  const rig = new WorkletRig();
  rig.take();
  rig.send({ type: "metronome", on: true });
  rig.send({ type: "metronome", on: false });
  assert.deepEqual(rig.take(), []);
});

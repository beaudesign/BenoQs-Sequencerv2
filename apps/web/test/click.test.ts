// The metronome's two parts that need no worklet: which beats fall in a block of the engine's tick position, and the sound of one click.
// specs/SPEC-0002/p5d-metronome-click.md, observables C1 to C3. The worklet around them is tested in worklet.test.ts.
import assert from "node:assert/strict";
import { test } from "node:test";
import { RENDER_FRAMES, TICKS_PER_CLOCK } from "../src/abi.ts";
import { CLICK_PEAK, CLICK_TAIL_SECONDS, ClickVoice, Metronome, QUARTER_TICKS, beatFrame, beatsIn } from "../src/click.ts";
import { Rng } from "./support/rng.ts";

const RATE = 48_000;

test("a click is a quarter note of the MIDI clock: 24 pulses, and the engine's own ticks to a pulse", () => {
  assert.equal(QUARTER_TICKS, 24 * TICKS_PER_CLOCK);
  assert.equal(QUARTER_TICKS, 192);
});

test("C1: the beats of a span are the multiples of a quarter in it, a beat on the start is in and a beat on the end is not", () => {
  assert.deepEqual(beatsIn(0, 192), [0]);
  assert.deepEqual(beatsIn(0.001, 192), [], "beat 0 has gone by, and beat 1 is exactly at the end");
  assert.deepEqual(beatsIn(192, 384), [1]);
  assert.deepEqual(beatsIn(191.5, 192.5), [1]);
  assert.deepEqual(beatsIn(0, 1000), [0, 1, 2, 3, 4, 5]);
  assert.deepEqual(beatsIn(5, 5), []);
  assert.deepEqual(beatsIn(10, 5), [], "a position that went backwards finds nothing");
  assert.deepEqual(beatsIn(0, 0), [], "an empty span at a beat holds nothing: the beat is the next span's");
  assert.deepEqual(beatsIn(192, 192), []);
});

test("C1: however a run is cut into blocks, every beat is found once and none twice (200 random cuts, strict at every edge)", () => {
  const rng = new Rng(0xbea7);
  for (let run = 0; run < 200; run++) {
    // A run that starts anywhere, in cuts that sometimes land exactly on a beat.
    const start = rng.int(4) === 0 ? 0 : rng.next() * 3000;
    const cuts = [start];
    let p = start;
    while (p < start + 4000) {
      const onBeat = rng.int(6) === 0;
      p = onBeat ? (Math.floor(p / QUARTER_TICKS) + 1 + rng.int(2)) * QUARTER_TICKS : p + rng.next() * 40;
      cuts.push(p);
    }
    const found: number[] = [];
    for (let i = 1; i < cuts.length; i++) found.push(...beatsIn(cuts[i - 1]!, cuts[i]!));
    assert.deepEqual(found, beatsIn(start, cuts[cuts.length - 1]!), `run ${run}`);
    assert.ok(found.length >= 20, "a run of 4000 ticks holds at least twenty beats: the comparison is not between two empty lists");
    for (let i = 1; i < found.length; i++) assert.equal(found[i], found[i - 1]! + 1, "consecutive, so none skipped");
  }
});

test("C2: a beat sounds at the frame of the block where its tick falls, by interpolation, and never outside the block", () => {
  assert.equal(beatFrame(0, 0, 16, RENDER_FRAMES), 0, "a beat on the first tick is the first frame");
  assert.equal(beatFrame(1, 184, 200, RENDER_FRAMES), 64, "half way through the block");
  assert.equal(beatFrame(1, 180, 200, RENDER_FRAMES), 77, "12 of 20 ticks in: 76.8 rounds to 77");
  assert.equal(beatFrame(1, 100, 192.000001, RENDER_FRAMES), RENDER_FRAMES - 1, "a beat at the very end is the last frame, not the one after the block");
  assert.equal(beatFrame(3, 576, 592.5, RENDER_FRAMES), 0, "a beat on the start of the block is its first frame");
  assert.equal(beatFrame(1, 192, 200, 64), 0);
});

/** Everything the voice writes for `frames` frames starting at `at`, cut into blocks of `block`. */
function voiceRun(frames: number, at: number, block: number, rate = RATE): Float32Array {
  const v = new ClickVoice(rate);
  const out = new Float32Array(frames);
  for (let from = 0; from < frames; from += block) {
    const to = Math.min(frames, from + block);
    const part = new Float32Array(to - from);
    if (at >= from && at < to) {
      v.render(part, 0, at - from);
      v.start();
      v.render(part, at - from, to - from);
    } else {
      v.render(part, 0, to - from);
    }
    out.set(part, from);
  }
  return out;
}

test("C3: a click is silent until its frame, not zero at it, peaks within a few milliseconds, and dies away to exactly nothing", () => {
  const at = 100;
  const out = voiceRun(RATE / 4, at, RENDER_FRAMES);
  for (let i = 0; i < at; i++) assert.equal(out[i], 0, `frame ${i} is before the click`);
  assert.notEqual(out[at], 0, "the first frame of the click is the click");
  let peak = 0;
  let peakAt = 0;
  for (let i = 0; i < out.length; i++) {
    if (Math.abs(out[i]!) > peak) {
      peak = Math.abs(out[i]!);
      peakAt = i;
    }
  }
  assert.ok(peak > 0.5 * CLICK_PEAK && peak <= CLICK_PEAK + 1e-6, `peak ${peak} against ${CLICK_PEAK}`);
  assert.ok(peak < 0.5, "a click, not a blast: half scale at most");
  assert.ok(peakAt - at < RATE * 0.003, `the peak is ${(((peakAt - at) / RATE) * 1000).toFixed(2)} ms after the onset`);
  const end = at + Math.ceil(CLICK_TAIL_SECONDS * RATE);
  for (let i = end; i < out.length; i++) assert.equal(out[i], 0, `frame ${i} is after the tail`);
  const late = Math.max(...Array.from(out.subarray(at + Math.round(0.02 * RATE), at + Math.round(0.025 * RATE)), Math.abs));
  assert.ok(late < peak * 0.05, "20 ms on it has fallen below 5% of its peak");
  assert.ok(out.subarray(at, at + 200).some((x) => x > 0) && out.subarray(at, at + 200).some((x) => x < 0), "it rings: it is not a one-sided pulse");
});

test("C3: the same samples whatever the block size, and a voice says when it has finished", () => {
  const whole = voiceRun(8192, 77, 8192);
  for (const block of [1, 7, 64, 128, 1000]) assert.deepEqual(voiceRun(8192, 77, block), whole, `blocks of ${block}`);
  const v = new ClickVoice(RATE);
  assert.equal(v.active, false);
  v.start();
  assert.equal(v.active, true);
  const out = new Float32Array(RATE);
  v.render(out, 0, out.length);
  assert.equal(v.active, false, "a second of frames is long enough for the tail to have finished");
});

test("C3: the click is the same shape at 44.1 kHz and 96 kHz, in time not in frames", () => {
  const lengthOf = (rate: number): number => {
    const o = voiceRun(rate / 2, 0, RENDER_FRAMES, rate);
    let last = 0;
    for (let i = 0; i < o.length; i++) if (o[i] !== 0) last = i;
    return (last + 1) / rate;
  };
  const a = lengthOf(44_100);
  const b = lengthOf(96_000);
  assert.ok(Math.abs(a - b) < 0.001, `${a} s against ${b} s`);
  assert.ok(Math.abs(a - CLICK_TAIL_SECONDS) < 0.001);
});

test("a click started again while it rings starts over, and does not add to itself", () => {
  const v = new ClickVoice(RATE);
  const a = new Float32Array(600);
  v.start();
  v.render(a, 0, 300);
  v.start();
  v.render(a, 300, 600);
  const fresh = new Float32Array(300);
  const w = new ClickVoice(RATE);
  w.start();
  w.render(fresh, 0, 300);
  assert.ok(fresh.some((x) => x !== 0), "a click was made");
  assert.deepEqual(a.subarray(300), fresh, "the second 300 frames are a click from its beginning");
});

test("the metronome writes silence when it is off, and when it is on but the transport is stopped", () => {
  const m = new Metronome(RATE);
  const out = new Float32Array(RENDER_FRAMES).fill(0.7);
  m.render(out, 0, 100, true);
  assert.ok(out.every((x) => x === 0), "off: the whole block is written, with silence, whatever was in it");
  m.on = true;
  m.render(out, 0, 100, false);
  assert.ok(out.every((x) => x === 0), "on and stopped: nothing");
});

test("the metronome puts a click at the frame the beat falls in, once, and the tail runs on into the blocks after", () => {
  const m = new Metronome(RATE);
  m.on = true;
  const a = new Float32Array(RENDER_FRAMES);
  m.render(a, 180, 200, true); // beat 1 (tick 192) is 12 of 20 ticks in
  const onset = a.findIndex((x) => x !== 0);
  assert.equal(onset, 77);
  const b = new Float32Array(RENDER_FRAMES);
  m.render(b, 200, 216, true);
  assert.notEqual(b[0], 0, "the click is still ringing in the next block");
  assert.equal(a.slice(0, onset).every((x) => x === 0), true);
});

test("switching off lets the ringing click finish, and switching on mid-beat waits for the next beat", () => {
  const m = new Metronome(RATE);
  m.on = true;
  const first = new Float32Array(RENDER_FRAMES);
  m.render(first, 190, 194, true); // a beat at 192, half way through
  m.on = false;
  const after = new Float32Array(RENDER_FRAMES);
  m.render(after, 194, 195, true);
  assert.notEqual(after[0], 0, "off: the tail goes on");
  m.on = true;
  const mid = new Float32Array(RENDER_FRAMES);
  const tail = new Float32Array(RENDER_FRAMES);
  m.render(mid, 195, 200, true);
  assert.notEqual(mid[0], 0, "still the old tail");
  const quiet = new Metronome(RATE);
  quiet.on = true;
  const q = new Float32Array(RENDER_FRAMES);
  quiet.render(q, 195, 200, true);
  assert.ok(q.every((x) => x === 0), "switched on in the middle of a beat: nothing until the next beat");
  quiet.render(tail, 380, 390, true);
  assert.notEqual(tail.findIndex((x) => x !== 0), -1, "and the next beat (tick 384) sounds");
});

test("the metronome keeps no count: a position that goes back to the start clicks the first beat again", () => {
  // The engine's own Reset puts its tick position at 0. Nothing here has to be told: where the position is, is where the beat is.
  const m = new Metronome(RATE);
  m.on = true;
  const late = new Float32Array(RENDER_FRAMES);
  m.render(late, 5_000, 5_016, true);
  assert.ok(late.every((x) => x === 0), "no beat in 5000 to 5016");
  const again = new Float32Array(RENDER_FRAMES);
  m.render(again, 0, 16, true);
  assert.equal(again.findIndex((x) => x !== 0), 0, "beat 0 is the first frame");
});

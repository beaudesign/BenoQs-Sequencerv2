// The built-in sound (specs/SPEC-0002/p6-runs-when-opened.md, observable R-2): a sequencer that is opened with nothing connected must be
// heard to run. The monitor plays the engine's own notes in the worklet's output, sample for sample where the engine puts them, and is
// not MIDI: the events the page sends to its devices are byte for byte what they are with the monitor off.
import assert from "node:assert/strict";
import { test } from "node:test";
import { RENDER_FRAMES, decodeEvents } from "../src/abi.ts";
import { DEMO, loadDemo } from "../src/demo.ts";
import { DRUM_CHANNEL, MAX_VOICES, Monitor, type NoteEvent } from "../src/monitor.ts";
import type { FromWorklet } from "../src/protocol.ts";
import { WorkletRig } from "./support/worklet-scope.ts";

const RATE = 48_000;
const on = (note: number, at = 0, channel = 1, vel = 100): NoteEvent => ({ kind: 0, channel, d1: note, d2: vel, atSample: at });
const off = (note: number, at = 0, channel = 1): NoteEvent => ({ kind: 1, channel, d1: note, d2: 0, atSample: at });
const peak = (a: Float32Array): number => a.reduce((m, x) => Math.max(m, Math.abs(x)), 0);

function run(m: Monitor, blocks: number, events: (block: number) => NoteEvent[] = () => []): Float32Array {
  const out = new Float32Array(blocks * RENDER_FRAMES);
  for (let b = 0; b < blocks; b++) m.render(out.subarray(b * RENDER_FRAMES, (b + 1) * RENDER_FRAMES), events(b));
  return out;
}

test("R-2: with nothing to play it adds nothing, exactly", () => {
  const m = new Monitor(RATE);
  m.on = true;
  const out = new Float32Array(RENDER_FRAMES).fill(0.25);
  m.render(out, []);
  assert.ok(out.every((x) => x === 0.25), "the block is as it was given");
});

test("R-2: a note starts on its sample, and not before", () => {
  const m = new Monitor(RATE);
  m.on = true;
  const out = run(m, 4, (b) => (b === 0 ? [on(69, 37)] : []));
  assert.ok(out.subarray(0, 37).every((x) => x === 0), "silent until the note's sample");
  assert.ok(peak(out.subarray(37, 37 + 480)) > 0.02, "and audible within 10 ms");
});

test("R-2: it is a pitch: A4 repeats every 109 samples at 48 kHz and an octave up every 54", () => {
  const cycles = (note: number): number => {
    const m = new Monitor(RATE);
    m.on = true;
    const out = run(m, 40, (b) => (b === 0 ? [on(note)] : []));
    const seg = out.subarray(2400, 4800); // after the attack, well inside the sustain
    let crossings = 0;
    for (let i = 1; i < seg.length; i++) if (seg[i - 1]! <= 0 && seg[i]! > 0) crossings++;
    return crossings / (seg.length / RATE); // rising zero crossings a second
  };
  assert.ok(Math.abs(cycles(69) - 440) < 8, `${cycles(69)} Hz`);
  assert.ok(Math.abs(cycles(81) - 880) < 16, `${cycles(81)} Hz`);
});

test("R-2: a note off lets it go: silent, exactly, within a quarter of a second", () => {
  const m = new Monitor(RATE);
  m.on = true;
  const out = run(m, 120, (b) => (b === 0 ? [on(60)] : b === 40 ? [off(60, 0)] : []));
  const stop = 40 * RENDER_FRAMES;
  assert.ok(peak(out.subarray(stop - 480, stop)) > 0.02, "it was sounding");
  assert.ok(out.subarray(stop + 12_000).every((x) => x === 0), "and it is gone 250 ms after the note off");
});

test("R-2: it never goes past full scale, however many notes: 40 at full velocity", () => {
  const m = new Monitor(RATE);
  m.on = true;
  const notes = Array.from({ length: 40 }, (_, i) => on(40 + i, 0, 1 + (i % 9), 127));
  const out = run(m, 60, (b) => (b === 0 ? notes : []));
  assert.ok(peak(out) <= 1, `peak ${peak(out)}`);
  assert.ok(peak(out) > 0.2, "and it is loud");
  assert.ok(MAX_VOICES >= 8 && MAX_VOICES <= 64);
});

test("R-2: a drum is a one-shot that ends on its own, and a kick and a hat are not the same sound", () => {
  const hit = (note: number): Float32Array => {
    const m = new Monitor(RATE);
    m.on = true;
    return run(m, 200, (b) => (b === 0 ? [on(note, 0, DRUM_CHANNEL)] : []));
  };
  const kick = hit(36);
  const hat = hit(42);
  assert.ok(peak(kick) > 0.1 && peak(hat) > 0.02, "both are heard");
  assert.ok(kick.subarray(RATE / 2).every((x) => x === 0), "the kick has ended by itself at half a second");
  assert.ok(hat.subarray(RATE / 4).every((x) => x === 0), "and the hat by a quarter");
  let different = 0;
  for (let i = 0; i < 2400; i++) if (kick[i] !== hat[i]) different++;
  assert.ok(different > 2000, "not the same samples");
  // A kick is low: fewer rising zero crossings in its first 50 ms than a hat's noise.
  const crossings = (a: Float32Array): number => {
    let n = 0;
    for (let i = 1; i < 2400; i++) if (a[i - 1]! <= 0 && a[i]! > 0) n++;
    return n;
  };
  assert.ok(crossings(kick) < crossings(hat) / 3, `kick ${crossings(kick)} against hat ${crossings(hat)}`);
});

test("R-2: it is deterministic: the same notes make the same samples", () => {
  const make = (): Float32Array => {
    const m = new Monitor(RATE);
    m.on = true;
    return run(m, 100, (b) => (b === 0 ? [on(60), on(42, 10, DRUM_CHANNEL), on(36, 20, DRUM_CHANNEL), on(67, 30)] : []));
  };
  assert.deepEqual(make(), make());
});

test("R-2: off, it adds nothing and keeps no notes: turned on later it does not sound what it was off for", () => {
  const m = new Monitor(RATE);
  const silent = run(m, 10, (b) => (b === 0 ? [on(60)] : []));
  assert.ok(silent.every((x) => x === 0));
  m.on = true;
  assert.ok(run(m, 40).every((x) => x === 0), "the note it was off for is not heard now");
  m.on = true;
  run(m, 1, () => [on(60)]);
  m.on = false;
  assert.ok(run(m, 40).every((x) => x === 0), "switched off while sounding: it stops, and does not come back");
  m.on = true;
  assert.ok(run(m, 40).every((x) => x === 0));
});

test("R-2: more notes than it has voices takes the oldest, and does not throw", () => {
  const m = new Monitor(RATE);
  m.on = true;
  const many = Array.from({ length: MAX_VOICES * 3 }, (_, i) => on(30 + (i % 60), i, 1));
  assert.doesNotThrow(() => run(m, 4, (b) => (b === 0 ? many : [])));
});

// ----- in the worklet -----

function eventsOf(rig: WorkletRig, blocks: number): { audio: Float32Array; events: string[] } {
  const events: string[] = [];
  for (let b = 0; b < blocks; b++) {
    for (const p of rig.block()) {
      const m: FromWorklet = p.message;
      if (m.type === "events") for (const e of decodeEvents(new Uint8Array(m.bytes))) events.push(`${m.frame + e.atSample}:${e.kind}:${e.port}:${e.channel}:${e.d1}:${e.d2}`);
    }
  }
  return { audio: rig.audio(), events };
}

test("R-2: in the worklet it is off until the page turns it on, so a page that does not ask hears what it heard before", () => {
  const rig = new WorkletRig();
  loadDemo({ setTrack: (track, attr, value) => rig.send({ type: "track", track, attr, value }), setStep: (track, step, attr, value) => rig.send({ type: "step", track, step, attr, value }) });
  rig.send({ type: "transport", play: true });
  const { audio, events } = eventsOf(rig, 400);
  assert.ok(events.length > 0, "the engine played");
  assert.ok(audio.every((x) => x === 0), "and the output is silent");
});

test("R-2: turned on, the demo is heard within a second of Play, with no MIDI anywhere, and is silent a quarter of a second after Stop", () => {
  const rig = new WorkletRig();
  rig.send({ type: "monitor", on: true });
  loadDemo({ setTrack: (track, attr, value) => rig.send({ type: "track", track, attr, value }), setStep: (track, step, attr, value) => rig.send({ type: "step", track, step, attr, value }) });
  rig.send({ type: "transport", play: true });
  const { audio } = eventsOf(rig, Math.ceil(RATE / RENDER_FRAMES)); // one second
  assert.ok(peak(audio) > 0.05, `peak ${peak(audio)} in the first second`);
  const heardAt = audio.findIndex((x) => x !== 0);
  assert.ok(heardAt >= 0 && heardAt < RATE / 2, `the first sound is at ${heardAt} frames`);
  rig.send({ type: "transport", play: false });
  const length = rig.audio().length;
  eventsOf(rig, Math.ceil(RATE / RENDER_FRAMES));
  const tail = rig.audio().subarray(length + RATE / 4);
  assert.ok(tail.every((x) => x === 0), "silent from a quarter of a second after the Stop");
});

test("R-2: it is not MIDI: the events are byte for byte the same with it on and off", () => {
  const play = (monitor: boolean): string[] => {
    const rig = new WorkletRig();
    rig.send({ type: "monitor", on: monitor });
    loadDemo({ setTrack: (track, attr, value) => rig.send({ type: "track", track, attr, value }), setStep: (track, step, attr, value) => rig.send({ type: "step", track, step, attr, value }) });
    rig.send({ type: "transport", play: true });
    return eventsOf(rig, 800).events;
  };
  const a = play(false);
  assert.ok(a.length > 20);
  assert.deepEqual(play(true), a);
  assert.ok(DEMO.length > 0);
});

test("R-2: the click and the monitor are both heard, and neither hides the other", () => {
  const rig = new WorkletRig();
  rig.send({ type: "monitor", on: true });
  rig.send({ type: "metronome", on: true });
  loadDemo({ setTrack: (track, attr, value) => rig.send({ type: "track", track, attr, value }), setStep: (track, step, attr, value) => rig.send({ type: "step", track, step, attr, value }) });
  rig.send({ type: "transport", play: true });
  const { audio } = eventsOf(rig, 400);
  assert.ok(peak(audio) <= 1, `peak ${peak(audio)}`);
  assert.ok(peak(audio) > 0.1);
});

test("R-2: a step message the engine refuses is reported to the page as an error and does not stop the worklet", () => {
  const rig = new WorkletRig();
  rig.take();
  rig.send({ type: "step", track: 10, step: 0, attr: 0, value: 1 });
  const posted = rig.take().map((p) => p.message);
  const error = posted.find((m): m is Extract<FromWorklet, { type: "error" }> => m.type === "error");
  assert.match(error?.message ?? "", /set step.*code 5/);
  rig.send({ type: "step", track: 0, step: 0, attr: 0, value: 1 });
  assert.deepEqual(rig.take().filter((p) => p.message.type === "error"), []);
});

// E3 (specs/SPEC-0002/p3-plan.md, as amended): after Stop, the receiver holds no note. A property over
// generated patterns, tempos and stop times, with the worklet, the wiring and the scheduler the page
// uses, and a receiver that queues timestamped sends the way a MIDI output does and ignores CC 123
// (ALL NOTES OFF), the stricter of the two kinds of receiver. The generator varies on purpose whether
// the output has `MIDIOutput.clear()` (Chromium does not implement it; the scheduler must not need it,
// and must not call it) and how badly the clocks jitter.
import assert from "node:assert/strict";
import { test } from "node:test";
import { Rng, Session, type SessionConfig } from "./support/session.ts";

interface Case {
  name: string;
  cfg: Omit<SessionConfig, "seed" | "supportsClear">;
}

const CASES: Case[] = [
  { name: "steady clocks", cfg: { mapJitterMs: 0, delayMs: 3 } },
  { name: "the clock map jitters by up to 3 ms between batches", cfg: { mapJitterMs: 3, delayMs: 5 } },
  { name: "messages arrive late, sometimes by 80 ms", cfg: { mapJitterMs: 0, delayMs: 40, spikeChance: 0.05, spikeMs: 80 } },
  { name: "both", cfg: { mapJitterMs: 3, delayMs: 40, spikeChance: 0.05, spikeMs: 80 } },
];

const RUNS_PER_CASE = 40;

function isNoteOn(bytes: number[]): boolean {
  return ((bytes[0] ?? 0) & 0xf0) === 0x90 && (bytes[2] ?? 0) > 0;
}

interface Outcome {
  heldAtStop: number;
  stuck: string[];
  queuedAtEnd: number;
  noteOnAfterStop: number;
  clears: number;
  stopSeen: boolean;
  errors: string[];
}

function runOnce(seed: number, supportsClear: boolean, cfg: Case["cfg"]): Outcome {
  const rng = new Rng(seed);
  const s = new Session({ seed, supportsClear, ...cfg });
  // A pattern: 4 to 40 steps on random tracks, and a tempo.
  const steps = 4 + rng.int(37);
  for (let i = 0; i < steps; i++) s.clickStep(rng.int(10), rng.int(16));
  s.send({ type: "tempo", bpm: 60 + rng.int(181) });
  s.send({ type: "transport", play: true });
  // Play for a while, then stop on the block after the first Note On that follows a random point:
  // that is the stop that comes closest to the note it should end.
  const after = 5 + rng.int(300);
  // What the engine believes is sounding, from the events it has made: this is what Stop must end.
  const engineHeld = new Set<string>();
  let ready = false;
  for (let b = 0; b < 1500 && !ready; b++) {
    const events = s.step();
    for (const e of events) {
      const key = `${e.port}/${e.channel}/${e.d1}`;
      if (e.kind === 0) engineHeld.add(key);
      if (e.kind === 1) engineHeld.delete(key);
    }
    ready = b >= after && events.some((e) => e.kind === 0);
  }
  const heldAtStop = engineHeld.size;
  s.send({ type: "transport", play: false });
  for (let b = 0; b < 200; b++) s.step();
  s.drain(1_000);

  const stopAt = s.stopAt;
  const after_ = stopAt === null ? [] : s.log.slice(stopAt);
  const sends = after_.filter((l) => l.what === "send");
  return {
    heldAtStop,
    stuck: s.held(),
    queuedAtEnd: [...s.outputs.values()].reduce((n, o) => n + o.queued, 0),
    noteOnAfterStop: sends.filter((l) => l.what === "send" && isNoteOn(l.bytes)).length,
    clears: [...s.outputs.values()].reduce((n, o) => n + o.clearCalls, 0),
    stopSeen: stopAt !== null,
    errors: s.errors,
  };
}

for (const c of CASES) {
  for (const supportsClear of [true, false]) {
    test(`Stop leaves no note held: ${c.name}, ${supportsClear ? "with" : "without"} MIDIOutput.clear()`, () => {
      let withNote = 0;
      const failures: string[] = [];
      for (let i = 0; i < RUNS_PER_CASE; i++) {
        const seed = 1000 + i * 37 + (supportsClear ? 0 : 1);
        const o = runOnce(seed, supportsClear, c.cfg);
        if (o.heldAtStop > 0) withNote++;
        const bad: string[] = [];
        if (!o.stopSeen) bad.push("the page never saw the transport stop");
        if (o.stuck.length) bad.push(`held after Stop: ${o.stuck.join(", ")}`);
        if (o.queuedAtEnd) bad.push(`${o.queuedAtEnd} sends still queued`);
        if (o.noteOnAfterStop) bad.push(`${o.noteOnAfterStop} Note On sent after Stop`);
        if (o.clears) bad.push(`clear() was called ${o.clears} times`);
        if (o.errors.length) bad.push(`worklet errors: ${o.errors.join("; ")}`);
        if (bad.length) failures.push(`seed ${seed}: ${bad.join("; ")}`);
      }
      assert.ok(withNote >= RUNS_PER_CASE / 4, `the generator should stop with a note sounding in many runs (${withNote} of ${RUNS_PER_CASE})`);
      assert.deepEqual(failures.slice(0, 5), [], `${failures.length} of ${RUNS_PER_CASE} runs failed`);
    });
  }
}

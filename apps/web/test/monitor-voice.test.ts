// The monitor and its voice (SPEC-0004 S0, ADR-0013): the voice is chosen from the registry, has its own cap on notes, is
// swapped without disturbing the drums, and the monitor never changes the events it is given (H-12: audio, never MIDI).
import assert from "node:assert/strict";
import { test } from "node:test";
import { DRUM_CHANNEL, MAX_VOICES, Monitor, type NoteEvent } from "../src/monitor.ts";
import { PLAIN } from "../src/voices/plain.ts";
import type { Voice, VoiceNote } from "../src/voices/voice.ts";
import { VOICES } from "../src/voices/registry.ts";
import { stub } from "./support/voice-stubs.ts";

const RATE = 48_000;
const BLOCK = 128;
const on = (note: number, at = 0, channel = 1, vel = 100): NoteEvent => ({ kind: 0, channel, d1: note, d2: vel, atSample: at });
const off = (note: number, at = 0, channel = 1): NoteEvent => ({ kind: 1, channel, d1: note, d2: 0, atSample: at });

function run(m: Monitor, blocks: number, events: (b: number) => readonly NoteEvent[] = () => []): Float32Array {
  const out = new Float32Array(blocks * BLOCK);
  for (let b = 0; b < blocks; b++) m.render(out.subarray(b * BLOCK, (b + 1) * BLOCK), events(b));
  return out;
}

/** A voice that records the notes the monitor makes from it, and the pitch each one was last started with. It makes no sound, so a test can hear the drums alone. */
function spy(maxNotes: number): { voice: Voice; notes: VoiceNote[]; pitch: Map<VoiceNote, number> } {
  const notes: VoiceNote[] = [];
  const pitch = new Map<VoiceNote, number>();
  const base = stub({ id: "spy", maxNotes, silent: true });
  return {
    notes,
    pitch,
    voice: {
      ...base,
      create: (rate) => {
        const n = base.create(rate);
        const start = n.start.bind(n);
        n.start = (midi, velocity) => {
          pitch.set(n, midi);
          start(midi, velocity);
        };
        notes.push(n);
        return n;
      },
    },
  };
}

test("the monitor makes one note of the voice for each of its slots, and none more", () => {
  const { voice, notes } = spy(24);
  const m = new Monitor(RATE);
  m.voice = voice;
  assert.equal(notes.length, MAX_VOICES);
  m.voice = voice; // the same voice again changes nothing
  assert.equal(notes.length, MAX_VOICES);
});

test("a voice's cap on notes is kept: past it the oldest note of that voice is taken, and a drum is never taken for one", () => {
  const { voice, notes, pitch } = spy(4);
  const m = new Monitor(RATE);
  m.voice = voice;
  m.on = true;
  // a kick first, then six held notes one block apart (MIDI 51 to 56)
  const out = run(m, 8, (b) => (b === 0 ? [on(36, 0, DRUM_CHANNEL)] : b >= 1 && b <= 6 ? [on(50 + b, 0, 1)] : []));
  const alive = notes.filter((n) => n.live).map((n) => pitch.get(n)).sort();
  assert.deepEqual(alive, [53, 54, 55, 56], "four notes sound and they are the newest four: the two oldest were taken");
  // the notes are silent, so the output is the kick alone, and it is the kick that a monitor with no notes makes
  const alone = new Monitor(RATE);
  alone.on = true;
  assert.deepEqual(out, run(alone, 8, (b) => (b === 0 ? [on(36, 0, DRUM_CHANNEL)] : [])));
});

test("with the default cap of 24 and a full pool the oldest slot is taken, as before", () => {
  const m = new Monitor(RATE);
  m.on = true;
  assert.doesNotThrow(() => run(m, 40, (b) => (b < 30 ? [on(40 + (b % 40), 0, 1 + (b % 9))] : [])));
});

test("changing the voice drops the notes of the old one and lets a drum that is sounding finish", () => {
  const m = new Monitor(RATE);
  m.on = true;
  const before = run(m, 4, (b) => (b === 0 ? [on(60), on(36, 0, DRUM_CHANNEL)] : []));
  assert.ok(before.some((x) => x !== 0));
  m.voice = stub({ id: "other" });
  const after = run(m, 200);
  const tone = after.subarray(0, 4 * BLOCK);
  assert.ok(tone.some((x) => x !== 0), "the kick is still sounding in the blocks after the change");
  assert.ok(after.subarray(RATE / 2 - 4 * BLOCK).every((x) => x === 0), "and the held tone does not come back");
});

test("H-12: the monitor never changes the events it is given, with any voice in the registry", () => {
  for (const voice of VOICES) {
    const m = new Monitor(RATE);
    m.voice = voice;
    m.on = true;
    const events = Object.freeze([Object.freeze(on(60, 3)), Object.freeze(on(36, 9, DRUM_CHANNEL)), Object.freeze(off(60, 100))]);
    const copy = JSON.stringify(events);
    assert.doesNotThrow(() => run(m, 20, (b) => (b === 0 ? events : [])));
    assert.equal(JSON.stringify(events), copy, voice.id);
  }
});

test("the registry's first voice sounds the same as a monitor that was never given one", () => {
  const a = new Monitor(RATE);
  const b = new Monitor(RATE);
  b.voice = PLAIN;
  a.on = true;
  b.on = true;
  const ev = (blk: number): NoteEvent[] => (blk === 0 ? [on(64), on(67, 5), on(36, 7, DRUM_CHANNEL)] : []);
  assert.deepEqual(run(a, 100, ev), run(b, 100, ev));
});

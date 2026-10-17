// The starts of a roll, as the page gets them (SPEC-0005 r1, CV-11). A single note with a strum of 9 is struck seven times, and where
// each strike starts is what the chord style changes: the numbers here are SPEC-0005 section 4's table for level 9 (the Octopus's own
// column for Manual) and are written out, not taken from the crate, so a test that uses them holds the page to the spec.
import type { MidiEvent } from "../../src/abi.ts";

/** Samples to the tick at 120 BPM and 48 kHz: 60 / 120 / 48 seconds. */
export const TICK_SAMPLES = 500;
/** Milliseconds to the tick at 120 BPM. */
export const TICK_MS = 60_000 / 120 / 48;

/** The starts of the seven strikes at strum 9, in ticks after the first. */
export const ROLL_9 = {
  manual: [0, 5, 10, 17, 23, 30, 45],
  accel: [0, 15, 25, 33, 38, 42, 45],
  bounce: [0, 22, 34, 40, 43, 44, 45],
  rit: [0, 3, 7, 12, 20, 30, 45],
} as const;

/** The Note Ons among `events` that fall on `channel` (1 to 16, as the events carry it), each with its sample counted from the first block. */
export function noteOnSamples(blocks: readonly MidiEvent[][], channel: number, frames = 128): number[] {
  const out: number[] = [];
  blocks.forEach((events, b) => {
    for (const e of events) if (e.kind === 0 && e.channel === channel) out.push(b * frames + e.atSample);
  });
  return out;
}

/** Strike times in samples to ticks after the first: the first `count` of them. */
export function ticksAfterFirst(samples: readonly number[], count = 7): number[] {
  const taken = samples.slice(0, count);
  return taken.map((s) => Math.round((s - taken[0]!) / TICK_SAMPLES));
}

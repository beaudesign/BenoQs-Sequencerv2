// The pattern the page opens with (specs/SPEC-0002/p6-runs-when-opened.md, R-1). It is data, written through `setTrack` and `setStep` like
// anything a page writes: the engine does not know it is a demo. Five tracks over 16 steps of a sixteenth each, at the page's 120 BPM: a bar
// of four on the floor, offbeat hats, a clap on 2 and 4, a bass and a lead in C minor pentatonic. Nothing in it is a claim about the Octopus.
import { STEP_ATTR, TRACK_ATTR } from "./abi.ts";

export interface DemoStep {
  /** 0 to 15 (the manual's steps 1 to 16). */
  step: number;
  /** Semitones above the track's pitch. */
  offset?: number;
}

export interface DemoTrack {
  /** 0 to 9: the row of the matrix. */
  track: number;
  name: string;
  /** The track's own note. A drum's note says which drum it is (`src/monitor.ts`). */
  pitch: number;
  /** 1 to 16, on port 1. */
  channel: number;
  steps: DemoStep[];
}

export const DEMO: readonly DemoTrack[] = [
  { track: 0, name: "Kick", pitch: 36, channel: 10, steps: [0, 4, 8, 10, 12].map((step) => ({ step })) },
  { track: 1, name: "Hat", pitch: 42, channel: 10, steps: [2, 6, 10, 14].map((step) => ({ step })) },
  { track: 2, name: "Clap", pitch: 39, channel: 10, steps: [4, 12].map((step) => ({ step })) },
  {
    track: 3,
    name: "Bass",
    pitch: 36,
    channel: 1,
    steps: [{ step: 0 }, { step: 3 }, { step: 6, offset: 3 }, { step: 8 }, { step: 11, offset: 5 }, { step: 14, offset: 3 }],
  },
  {
    track: 4,
    name: "Lead",
    pitch: 60,
    channel: 2,
    steps: [
      { step: 2, offset: 7 },
      { step: 5, offset: 10 },
      { step: 7, offset: 12 },
      { step: 10, offset: 7 },
      { step: 13, offset: 5 },
      { step: 15, offset: 3 },
    ],
  },
];

/** What the page can write: the two calls of `Host`, and the same two in a test. */
export interface DemoSink {
  setTrack(track: number, attr: number, value: number): void;
  setStep(track: number, step: number, attr: number, value: number): void;
}

/** Writes a pattern. Writing it twice is the same as once: every step it names is made active and given its offset, and nothing else is touched. */
export function loadDemo(sink: DemoSink, demo: readonly DemoTrack[] = DEMO): void {
  for (const t of demo) {
    sink.setTrack(t.track, TRACK_ATTR.pitch, t.pitch);
    sink.setTrack(t.track, TRACK_ATTR.midiChannel, t.channel);
    for (const s of t.steps) {
      sink.setStep(t.track, s.step, STEP_ATTR.pitchOffset, s.offset ?? 0);
      sink.setStep(t.track, s.step, STEP_ATTR.active, 1);
    }
  }
}

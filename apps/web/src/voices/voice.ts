// A voice: a sound the page can play a pitched note with (SPEC-0004, ADR-0013). The monitor owns the slots and the engine's notes;
// a voice owns only how one note sounds. Runs in the AudioWorkletGlobalScope, in the same place and under the same rules as the
// monitor: no allocation per sample, no `Math.random`, nothing that differs between two runs.

/** The monitor's pool of slots, shared by the pitched voice and the three drums. A voice's cap on notes cannot exceed it. */
export const SLOT_COUNT = 24;

/** One sounding note. Made once per slot when the voice is chosen and reused: `start` begins a note in it, and nothing allocates after that. */
export interface VoiceNote {
  /** True from `start` until the note has finished sounding, its release included. The monitor reads it to free the slot. */
  readonly live: boolean;
  /** True from `start` until `release`: the key is down. */
  readonly held: boolean;
  /** The samples this note has rendered since `start`. The oldest is the one taken when a cap is reached. */
  readonly age: number;
  /** Begins a note, in place of whatever this object was doing. `note` is MIDI 0 to 127; `velocity` 1 to 127, clamped by the voice. */
  start(note: number, velocity: number): void;
  /** The key is let go: the note begins its release. */
  release(): void;
  /** The next sample, already scaled by velocity, before the monitor's mix and master. Returns 0 once the note has ended, and then `live` is false. */
  next(): number;
}

/** What a voice says about itself. Each is checked against the measurement (SPEC-0004 H-16), so a declaration cannot lie. */
export interface VoiceTraits {
  /** Whether a louder note is also brighter (SPEC-0004 H-9). `Plain` is not: its spectrum does not move with velocity. */
  readonly velocityBrightens: boolean;
}

/** A sound in the registry (`registry.ts`). Only a voice in the registry can be chosen. */
export interface Voice {
  /** Kebab-case, unique, never reused: it is how a voice is named in a file, a message and a test. */
  readonly id: string;
  /** What the strip's button says. At most 16 characters. */
  readonly name: string;
  /** The most notes of this voice sounding at once; past it the oldest note of the voice is taken (SPEC-0004 D14). At most `SLOT_COUNT`. */
  readonly maxNotes: number;
  readonly traits: VoiceTraits;
  /** One note's worth of state at `sampleRate`. Called `SLOT_COUNT` times when the voice is chosen, never while sounding. */
  create(sampleRate: number): VoiceNote;
}

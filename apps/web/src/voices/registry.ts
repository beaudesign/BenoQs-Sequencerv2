// The voices the page can play with (SPEC-0004, ADR-0014). A voice that is not in this list cannot be chosen, and every voice in it is
// held to the voice-wide observables by one test (`test/voice-registry.test.ts`, H-16) with no edit to the test.
import { PLAIN } from "./plain.ts";
import type { Voice } from "./voice.ts";

export const VOICES: readonly Voice[] = [PLAIN];

/** What the page sounds like when it opens. Plain until a later slice changes it, and only with the owner's say (SPEC-0004 D2). */
export const DEFAULT_VOICE: Voice = PLAIN;

export function voiceById(id: string): Voice | undefined {
  return VOICES.find((v) => v.id === id);
}

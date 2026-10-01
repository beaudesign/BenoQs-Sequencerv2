// Joins the worklet's messages to the MIDI scheduler. The page calls this once; the tests call the
// same function with a stand-in for the worklet, so what they check is what the page runs.
import type { MidiScheduler, TimeMap } from "./midi-out.ts";

export interface EventSource {
  readonly sampleRate: number;
  onEvents: ((frame: number, bytes: Uint8Array) => void) | null;
}

/** A clock map that can take a fresh pair of times (`ContextTimeMap` does). */
export type RefreshableTimeMap = TimeMap & { refresh?: () => void };

export function wireMidi(source: EventSource, scheduler: MidiScheduler, time: RefreshableTimeMap): void {
  source.onEvents = (frame, bytes) => {
    time.refresh?.();
    scheduler.schedule(frame, source.sampleRate, bytes);
  };
}

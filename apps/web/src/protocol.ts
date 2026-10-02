// The messages between the page and the worklet. Plain objects, structured-cloned by postMessage.

export interface WorkletOptions {
  /** The compiled octoweb module. A compiled module can be sent to a worklet; the worklet cannot fetch it. */
  module: WebAssembly.Module;
  /** The layout text (`n id` lines) as bytes: the worklet has no TextEncoder. */
  layout: Uint8Array;
  seed: bigint;
}

export type ToWorklet =
  | { type: "input"; nowMs: number; kind: number; control: number; detents: number }
  | { type: "transport"; play: boolean }
  | { type: "tempo"; bpm: number }
  | { type: "track"; track: number; attr: number; value: number }
  | { type: "reset" };

export type FromWorklet =
  | { type: "ready"; abi: number; sampleRate: number }
  /** Events of one render block. `frame` is the audio context's `currentFrame` at the start of the block. */
  | { type: "events"; frame: number; bytes: ArrayBuffer }
  /** About every 21 ms. `leds` is present only when an LED changed. */
  | { type: "panel"; frame: number; leds: ArrayBuffer | null; playheads: ArrayBuffer; status: number; droppedIntents: number }
  | { type: "error"; message: string };

export const WORKLET_NAME = "octoweb";
/** Blocks of 128 frames between panel messages: 8 is about 21 ms at 48 kHz. */
export const PANEL_EVERY_BLOCKS = 8;

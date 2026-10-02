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
  /** Makes the engine the MIDI clock master (ADR-0009), or not. */
  | { type: "clock"; master: boolean }
  | { type: "reset" };

export type FromWorklet =
  | { type: "ready"; abi: number; sampleRate: number }
  /** Events of one render block. `frame` is the audio context's `currentFrame` at the start of the block. */
  | { type: "events"; frame: number; bytes: ArrayBuffer }
  /**
   * About every 21 ms, and after every message from the page. `leds` is present only when an LED changed. `position` is where the
   * audio is on the engine's tick grid, in ticks and as a fraction (`octoweb_tick_position`), and `positionFrame` is the audio
   * context's frame at which that holds: the end of the block just rendered, or `frame` itself for a message handled between blocks.
   */
  | { type: "panel"; frame: number; leds: ArrayBuffer | null; playheads: ArrayBuffer; status: number; droppedIntents: number; position: number; positionFrame: number }
  | { type: "error"; message: string };

export const WORKLET_NAME = "octoweb";
/** Blocks of 128 frames between panel messages: 8 is about 21 ms at 48 kHz. */
export const PANEL_EVERY_BLOCKS = 8;

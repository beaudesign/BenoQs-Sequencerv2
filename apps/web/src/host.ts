// The page's side of the worklet: compile the module, start the worklet, pass panel input in and
// events and panel frames out. It holds no sequencing logic.

import { KIND_DOWN, KIND_TURN, KIND_UP, TRACK_COUNT } from "./abi.ts";
import { WORKLET_NAME, type FromWorklet, type ToWorklet, type WorkletOptions } from "./protocol.ts";

export interface HostConfig {
  /** The octoweb module: its bytes, or one already compiled. */
  wasm: BufferSource | WebAssembly.Module;
  /** The layout text as bytes (`layoutFromControls`). */
  layout: Uint8Array;
  /** Where `worklet.js` is served from. */
  workletUrl: string | URL;
  seed?: bigint;
  context?: AudioContext;
  /** How long to wait for the worklet to say it is ready. */
  readyTimeoutMs?: number;
}

export interface PanelFrame {
  frame: number;
  /** One byte per control, present when an LED changed. */
  leds: Uint8Array | null;
  /** The step each track is on. */
  playheads: Uint8Array;
  running: boolean;
  zoomed: boolean;
  droppedIntents: number;
  /** Where the audio is on the engine's tick grid, in ticks, as a fraction. */
  position: number;
  /** The audio context's frame at which `position` holds. */
  positionFrame: number;
}

/** What the page registers to hear from the worklet. */
export interface WorkletSink {
  onEvents: ((frame: number, bytes: Uint8Array) => void) | null;
  onPanel: ((panel: PanelFrame) => void) | null;
  onError: ((message: string) => void) | null;
}

/** One message from the worklet to the callbacks. Messages from one port arrive in the order they were posted. */
export function routeFromWorklet(m: FromWorklet, sink: WorkletSink): void {
  switch (m.type) {
    case "events":
      sink.onEvents?.(m.frame, new Uint8Array(m.bytes));
      break;
    case "panel":
      sink.onPanel?.({
        frame: m.frame,
        leds: m.leds ? new Uint8Array(m.leds) : null,
        playheads: new Uint8Array(m.playheads).subarray(0, TRACK_COUNT),
        running: (m.status & 1) === 1,
        zoomed: (m.status & 2) === 2,
        droppedIntents: m.droppedIntents,
        position: m.position,
        positionFrame: m.positionFrame,
      });
      break;
    case "error":
      sink.onError?.(m.message);
      break;
    case "ready":
      break;
  }
}

export class Host implements WorkletSink {
  readonly context: AudioContext;
  readonly node: AudioWorkletNode;
  readonly sampleRate: number;
  onEvents: ((frame: number, bytes: Uint8Array) => void) | null = null;
  onPanel: ((panel: PanelFrame) => void) | null = null;
  onError: ((message: string) => void) | null = null;

  private constructor(context: AudioContext, node: AudioWorkletNode, sampleRate: number) {
    this.context = context;
    this.node = node;
    this.sampleRate = sampleRate;
    node.port.onmessage = (e: MessageEvent<FromWorklet>) => this.receive(e.data);
  }

  /** Call from a user gesture, or the browser keeps the audio context suspended. */
  static async start(config: HostConfig): Promise<Host> {
    const context = config.context ?? new AudioContext({ latencyHint: "interactive" });
    const module = config.wasm instanceof WebAssembly.Module ? config.wasm : await WebAssembly.compile(config.wasm);
    await context.audioWorklet.addModule(config.workletUrl);
    const options: WorkletOptions = { module, layout: config.layout, seed: config.seed ?? 0n };
    const node = new AudioWorkletNode(context, WORKLET_NAME, { numberOfInputs: 0, numberOfOutputs: 1, outputChannelCount: [1], processorOptions: options });
    // The output is silent. The node is connected anyway, or the browser may not run it (tech.md section 2).
    node.connect(context.destination);
    const ready = new Promise<number>((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error("the worklet did not start")), config.readyTimeoutMs ?? 10_000);
      // A constructor that throws (a layout the controller refuses, say) is reported here, not by a message.
      node.onprocessorerror = () => {
        clearTimeout(timer);
        reject(new Error("the worklet could not start (the module or its layout was refused)"));
      };
      node.port.onmessage = (e: MessageEvent<FromWorklet>) => {
        const m = e.data;
        if (m.type === "ready") {
          clearTimeout(timer);
          resolve(m.sampleRate);
        } else if (m.type === "error") {
          clearTimeout(timer);
          reject(new Error(m.message));
        }
      };
    });
    const sampleRate = await ready;
    await context.resume();
    return new Host(context, node, sampleRate);
  }

  private receive(m: FromWorklet): void {
    routeFromWorklet(m, this);
  }

  private post(m: ToWorklet): void {
    this.node.port.postMessage(m);
  }

  press(control: number): void {
    this.post({ type: "input", nowMs: performance.now(), kind: KIND_DOWN, control, detents: 0 });
  }

  release(control: number): void {
    this.post({ type: "input", nowMs: performance.now(), kind: KIND_UP, control, detents: 0 });
  }

  turn(control: number, detents: number): void {
    this.post({ type: "input", nowMs: performance.now(), kind: KIND_TURN, control, detents });
  }

  /** The stopgap of ADR-0008 decision 4: the page starts and stops the transport. */
  transport(play: boolean): void {
    this.post({ type: "transport", play });
  }

  setTempo(bpm: number): void {
    this.post({ type: "tempo", bpm });
  }

  /** One attribute of one track, in the engine's own numbering (`TRACK_ATTR`). The engine clamps the value; a track or attribute that does not exist comes back as an `onError`. */
  setTrack(track: number, attr: number, value: number): void {
    this.post({ type: "track", track, attr, value });
  }

  /** Makes the engine the MIDI clock master, or not (ADR-0009). The clock reaches every chosen output as real-time records in the events. */
  setClock(master: boolean): void {
    this.post({ type: "clock", master });
  }

  async close(): Promise<void> {
    this.node.disconnect();
    await this.context.close();
  }
}

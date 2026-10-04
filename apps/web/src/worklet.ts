// Runs in the AudioWorkletGlobalScope: the engine and the panel controller, once per 128 frames.
// No sequencing and no panel logic lives here: it calls the module and posts what comes back.
/// <reference path="./worklet-env.d.ts" />
import { ABI_VERSION, KIND_DOWN, KIND_TURN, KIND_UP, Octoweb, RENDER_FRAMES, type Exports } from "./abi.ts";
import { Metronome } from "./click.ts";
import { PANEL_EVERY_BLOCKS, WORKLET_NAME, type FromWorklet, type ToWorklet, type WorkletOptions } from "./protocol.ts";

class OctowebProcessor extends AudioWorkletProcessor {
  private readonly engine: Octoweb;
  private readonly metronome: Metronome;
  private blocks = 0;

  constructor(options: { processorOptions: unknown }) {
    super();
    const o = options.processorOptions as WorkletOptions;
    const instance = new WebAssembly.Instance(o.module, {});
    this.engine = new Octoweb(instance.exports as unknown as Exports);
    this.engine.init(o.layout, sampleRate, o.seed);
    this.metronome = new Metronome(sampleRate);
    this.port.onmessage = (e: MessageEvent<ToWorklet>) => this.handle(e.data);
    this.send({ type: "ready", abi: ABI_VERSION, sampleRate });
  }

  private send(message: FromWorklet, transfer: Transferable[] = []): void {
    this.port.postMessage(message, transfer);
  }

  private handle(m: ToWorklet): void {
    try {
      switch (m.type) {
        case "input":
          if (m.kind === KIND_DOWN || m.kind === KIND_UP || m.kind === KIND_TURN) this.engine.input(m.nowMs, m.kind, m.control, m.detents);
          break;
        case "transport":
          this.engine.transport(m.play);
          break;
        case "tempo":
          this.engine.setTempo(m.bpm);
          break;
        case "track":
          this.engine.setTrack(m.track, m.attr, m.value);
          break;
        case "clock":
          this.engine.setClock(m.master);
          break;
        case "reset":
          this.engine.reset();
          break;
        case "metronome":
          // Audio the worklet makes, and nothing the engine or the panel knows of: no panel message follows.
          this.metronome.on = m.on;
          return;
      }
      // A Stop, from the transport message or from a panel key, shows in the panel's status word. The
      // engine's flush Note Offs come out of the next render and are scheduled like any other event.
      this.postPanel(currentFrame);
    } catch (e) {
      this.send({ type: "error", message: String(e) });
    }
  }

  /**
   * `positionFrame` is the audio context's frame at which the engine's tick position holds: it is where the last render ended. For a
   * message handled between blocks that is the next block's first frame, `currentFrame`; for the panel message made at the end of
   * `process()` it is one block on.
   */
  private postPanel(positionFrame: number): void {
    const changed = this.engine.refreshLeds();
    const leds = changed ? this.engine.ledBytes().slice().buffer : null;
    const playheads = this.engine.playheadBytes().slice().buffer;
    this.send(
      { type: "panel", frame: currentFrame, leds, playheads, status: this.engine.status(), droppedIntents: this.engine.droppedIntents(), position: this.engine.tickPosition(), positionFrame },
      leds ? [leds, playheads] : [playheads],
    );
  }

  /**
   * Renders one block. The node's one output carries the metronome's click and nothing else: the engine's notes are MIDI and leave in the
   * `events` message. The click is placed from where the engine's tick position was before the render and is after it (`click.ts`).
   */
  override process(_inputs: Float32Array[][] = [], outputs: Float32Array[][] = []): boolean {
    try {
      const running = this.engine.running();
      const from = this.engine.tickPosition();
      const n = this.engine.render(RENDER_FRAMES);
      if (n > 0) {
        const bytes = this.engine.eventBytes(n).slice().buffer;
        this.send({ type: "events", frame: currentFrame, bytes }, [bytes]);
      }
      const out = outputs[0]?.[0];
      if (out) this.metronome.render(out, from, this.engine.tickPosition(), running);
      if (++this.blocks % PANEL_EVERY_BLOCKS === 0) this.postPanel(currentFrame + RENDER_FRAMES);
    } catch (e) {
      this.send({ type: "error", message: String(e) });
    }
    return true;
  }
}

registerProcessor(WORKLET_NAME, OctowebProcessor);

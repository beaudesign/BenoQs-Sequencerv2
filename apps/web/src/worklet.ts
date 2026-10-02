// Runs in the AudioWorkletGlobalScope: the engine and the panel controller, once per 128 frames.
// No sequencing and no panel logic lives here: it calls the module and posts what comes back.
/// <reference path="./worklet-env.d.ts" />
import { ABI_VERSION, KIND_DOWN, KIND_TURN, KIND_UP, Octoweb, RENDER_FRAMES, type Exports } from "./abi.ts";
import { PANEL_EVERY_BLOCKS, WORKLET_NAME, type FromWorklet, type ToWorklet, type WorkletOptions } from "./protocol.ts";

class OctowebProcessor extends AudioWorkletProcessor {
  private readonly engine: Octoweb;
  private blocks = 0;

  constructor(options: { processorOptions: unknown }) {
    super();
    const o = options.processorOptions as WorkletOptions;
    const instance = new WebAssembly.Instance(o.module, {});
    this.engine = new Octoweb(instance.exports as unknown as Exports);
    this.engine.init(o.layout, sampleRate, o.seed);
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
      }
      // A Stop, from the transport message or from a panel key, shows in the panel's status word. The
      // engine's flush Note Offs come out of the next render and are scheduled like any other event.
      this.postPanel();
    } catch (e) {
      this.send({ type: "error", message: String(e) });
    }
  }

  private postPanel(): void {
    const changed = this.engine.refreshLeds();
    const leds = changed ? this.engine.ledBytes().slice().buffer : null;
    const playheads = this.engine.playheadBytes().slice().buffer;
    this.send(
      { type: "panel", frame: currentFrame, leds, playheads, status: this.engine.status(), droppedIntents: this.engine.droppedIntents() },
      leds ? [leds, playheads] : [playheads],
    );
  }

  override process(): boolean {
    try {
      const n = this.engine.render(RENDER_FRAMES);
      if (n > 0) {
        const bytes = this.engine.eventBytes(n).slice().buffer;
        this.send({ type: "events", frame: currentFrame, bytes }, [bytes]);
      }
      if (++this.blocks % PANEL_EVERY_BLOCKS === 0) this.postPanel();
    } catch (e) {
      this.send({ type: "error", message: String(e) });
    }
    return true;
  }
}

registerProcessor(WORKLET_NAME, OctowebProcessor);

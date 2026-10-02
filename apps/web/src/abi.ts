// A typed wrapper over the exports of the octoweb module (apps/web/engine/ABI.md, octoweb-abi/2).
// It runs inside the AudioWorklet and in Node, so it uses nothing the worklet lacks: no
// TextDecoder, no performance, no fetch.

export const ABI_VERSION = 2;
export const EVENT_BYTES = 12;
export const LED_COUNT = 512;
export const TRACK_COUNT = 10;
/** The frames in one AudioWorklet render quantum. */
export const RENDER_FRAMES = 128;

export type LedColour = "off" | "red" | "green" | "orange";
export type LedPhase = "steady" | "flash" | "shine";
const LED_COLOURS: readonly LedColour[] = ["off", "red", "green", "orange"];
const LED_PHASES: readonly LedPhase[] = ["steady", "flash", "shine"];

/** One LED byte (ABI.md): the colour in the low two bits, the phase in the next two. A phase the ABI does not define reads as steady. */
export function decodeLed(byte: number): { colour: LedColour; phase: LedPhase } {
  return { colour: LED_COLOURS[byte & 0b11] ?? "off", phase: LED_PHASES[(byte >> 2) & 0b11] ?? "steady" };
}

/** The attribute numbers `octoweb_set_track` takes: the engine's own order (ABI.md). Only the one the page writes is named. */
export const TRACK_ATTR = { midiChannel: 8 } as const;

/** The event record kind for a MIDI real-time message: `d1` is the status byte (0xF8 Clock, 0xFA Start, 0xFB Continue, 0xFC Stop), port and channel are 0. */
export const KIND_REALTIME = 5;

export const KIND_DOWN = 0;
export const KIND_UP = 1;
export const KIND_TURN = 2;

export interface Exports {
  memory: WebAssembly.Memory;
  octoweb_abi(): number;
  octoweb_scratch(len: number): number;
  octoweb_init(sampleRate: number, seedLo: number, seedHi: number, layoutLen: number): number;
  octoweb_message_len(): number;
  octoweb_reset(): number;
  octoweb_input(nowMs: number, kind: number, control: number, detents: number): number;
  octoweb_transport(play: number): number;
  octoweb_set_tempo(bpm: number): number;
  octoweb_set_track(track: number, attr: number, value: number): number;
  octoweb_set_clock(master: number): number;
  octoweb_tick_position(): number;
  octoweb_render(frames: number): number;
  octoweb_events(): number;
  octoweb_refresh_leds(): number;
  octoweb_leds(): number;
  octoweb_playheads(): number;
  octoweb_status(): number;
  octoweb_dropped_intents(): number;
  /** Only in a build with the `measure` feature. */
  octoweb_alloc_count?: () => number;
  /** Only in a build with the `spike` feature. */
  octoweb_spike_run?: (len: number) => number;
}

export class OctowebError extends Error {
  code: number;
  constructor(message: string, code: number) {
    super(message);
    this.name = "OctowebError";
    this.code = code;
  }
}

/** One record of the events buffer, decoded. Kind 5 is a real-time message (`KIND_REALTIME`); the others are notes and controllers. */
export interface MidiEvent {
  kind: number;
  port: number;
  channel: number;
  d1: number;
  d2: number;
  atSample: number;
}

/** Bytes to text for the module's messages, which are ASCII (control ids and short sentences). */
export function asciiToString(bytes: Uint8Array): string {
  let s = "";
  for (const b of bytes) s += String.fromCharCode(b);
  return s;
}

export function decodeEvents(bytes: Uint8Array): MidiEvent[] {
  const out: MidiEvent[] = [];
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  for (let i = 0; i + EVENT_BYTES <= bytes.byteLength; i += EVENT_BYTES) {
    out.push({
      kind: view.getUint8(i),
      port: view.getUint8(i + 1),
      channel: view.getUint8(i + 2),
      d1: view.getUint8(i + 3),
      d2: view.getUint16(i + 4, true),
      atSample: view.getUint32(i + 8, true),
    });
  }
  return out;
}

export class Octoweb {
  readonly x: Exports;

  constructor(exports: Exports) {
    this.x = exports;
    const v = exports.octoweb_abi() >>> 0;
    if (v !== ABI_VERSION) throw new OctowebError(`the module speaks octoweb-abi/${v}, this page speaks ${ABI_VERSION}`, 0);
  }

  private bytes(ptr: number, len: number): Uint8Array {
    // Memory can grow, so the view is made fresh each time.
    return new Uint8Array(this.x.memory.buffer, ptr, len);
  }

  private writeScratch(data: Uint8Array): void {
    const p = this.x.octoweb_scratch(data.byteLength);
    this.bytes(p, data.byteLength).set(data);
  }

  private message(): string {
    const len = this.x.octoweb_message_len() >>> 0;
    return asciiToString(this.bytes(this.x.octoweb_scratch(len), len));
  }

  private check(code: number, what: string): void {
    const c = code >>> 0;
    if (c !== 0) throw new OctowebError(`${what} failed with code ${c}`, c);
  }

  /** `layout` is the text of `n id` lines, as bytes. */
  init(layout: Uint8Array, sampleRate: number, seed: bigint): void {
    this.writeScratch(layout);
    const code = this.x.octoweb_init(sampleRate, Number(seed & 0xffff_ffffn), Number((seed >> 32n) & 0xffff_ffffn), layout.byteLength) >>> 0;
    if (code !== 0) throw new OctowebError(`init failed with code ${code}: ${this.message()}`, code);
  }

  reset(): void {
    this.check(this.x.octoweb_reset(), "reset");
  }

  /** Returns the module's code. 0 is success. The caller decides what to do with a refusal. */
  input(nowMs: number, kind: number, control: number, detents = 0): number {
    return this.x.octoweb_input(nowMs, kind, control, detents) >>> 0;
  }

  transport(play: boolean): void {
    this.check(this.x.octoweb_transport(play ? 1 : 0), "transport");
  }

  setTempo(bpm: number): void {
    this.check(this.x.octoweb_set_tempo(bpm), "set tempo");
  }

  /** One attribute of one track (`TRACK_ATTR`). A track or attribute that does not exist is code 5. */
  setTrack(track: number, attr: number, value: number): void {
    this.check(this.x.octoweb_set_track(track, attr, value), "set track");
  }

  /** Makes the engine the MIDI clock master, or not. Off at `init`. Turned on while the transport runs, the engine says where it is (Start or Continue) and the pulses follow. */
  setClock(master: boolean): void {
    this.check(this.x.octoweb_set_clock(master ? 1 : 0), "set clock");
  }

  /** Where the audio is on the engine's tick grid, in ticks, at the end of the last render. 0 before `init` and before the first Play. */
  tickPosition(): number {
    return this.x.octoweb_tick_position();
  }

  /** Renders a block and returns the number of records now in the events buffer: notes and the clock, in sample order. */
  render(frames: number): number {
    return this.x.octoweb_render(frames) >>> 0;
  }

  /** A view of the first `count` event records, valid until the next call into the module. */
  eventBytes(count: number): Uint8Array {
    return this.bytes(this.x.octoweb_events(), count * EVENT_BYTES);
  }

  /** True if any LED changed since the last refresh. */
  refreshLeds(): boolean {
    return (this.x.octoweb_refresh_leds() >>> 0) === 1;
  }

  ledBytes(): Uint8Array {
    return this.bytes(this.x.octoweb_leds(), LED_COUNT);
  }

  playheadBytes(): Uint8Array {
    return this.bytes(this.x.octoweb_playheads(), TRACK_COUNT);
  }

  /** Bit 0: the transport is running. Bit 1: the panel is in Step zoom. */
  status(): number {
    return this.x.octoweb_status() >>> 0;
  }

  running(): boolean {
    return (this.status() & 1) === 1;
  }

  droppedIntents(): number {
    return this.x.octoweb_dropped_intents() >>> 0;
  }
}

/** The layout text the module wants, from the parsed `contracts/controls.json`. */
export function layoutFromControls(doc: { controls: { n: number; id: string }[] }): Uint8Array {
  const text = doc.controls.map((c) => `${c.n} ${c.id}\n`).join("");
  const out = new Uint8Array(text.length);
  for (let i = 0; i < text.length; i++) out[i] = text.charCodeAt(i);
  return out;
}

// The input path (SPEC-0002 P4, plan observable M6): what a chosen MIDI input hands the page is decoded and counted.
//
// Nothing consumes it yet. Recording, transpose, force-to-scale, map learn and program change belong to later waves (D-P4-1), and
// the clock follower arrives with P4d. `InputPath.onMessage` is the one place they will attach, and it is empty.
//
// Web MIDI gives a page one complete message per event, never running status, so a message is decoded from its own bytes.
// A system exclusive message is not delivered unless the page asked for the permission, and this page does not (D-P4-9).

export type RealtimeName = "clock" | "start" | "continue" | "stop";

export type InMessage =
  | { kind: "noteOn"; channel: number; note: number; velocity: number }
  | { kind: "noteOff"; channel: number; note: number; velocity: number }
  | { kind: "controller"; channel: number; controller: number; value: number }
  | { kind: "bend"; channel: number; value: number }
  | { kind: "pressure"; channel: number; value: number }
  | { kind: "program"; channel: number; program: number }
  | { kind: "realtime"; message: RealtimeName }
  /** Valid MIDI the sequencer has no use for (active sensing, song position, key pressure, system exclusive). */
  | { kind: "ignored"; status: number }
  /** Not a MIDI message: no data, no status byte, the wrong length, or a data byte above 127. */
  | { kind: "malformed" };

export type DecodedKind = Exclude<InMessage["kind"], "ignored" | "malformed">;

const REALTIME: Readonly<Record<number, RealtimeName>> = { 0xf8: "clock", 0xfa: "start", 0xfb: "continue", 0xfc: "stop" };
const MALFORMED: InMessage = { kind: "malformed" };

/** The message in `data`, or `malformed` or `ignored` and never a throw. */
export function decodeMessage(data: ArrayLike<number> | null): InMessage {
  if (!data || data.length === 0) return MALFORMED;
  const status = data[0]!;
  if (status < 0x80 || status > 0xff) return MALFORMED;
  if (status >= 0xf0) {
    const name = REALTIME[status];
    if (name) return data.length === 1 ? { kind: "realtime", message: name } : MALFORMED;
    return { kind: "ignored", status };
  }
  const channel = (status & 0x0f) + 1;
  const length = (status & 0xf0) === 0xc0 || (status & 0xf0) === 0xd0 ? 2 : 3;
  if (data.length !== length) return MALFORMED;
  for (let i = 1; i < length; i++) if (data[i]! > 127 || data[i]! < 0) return MALFORMED;
  const d1 = data[1]!;
  const d2 = length === 3 ? data[2]! : 0;
  switch (status & 0xf0) {
    case 0x80:
      return { kind: "noteOff", channel, note: d1, velocity: d2 };
    case 0x90:
      // A Note On with velocity 0 is a Note Off in MIDI.
      return d2 === 0 ? { kind: "noteOff", channel, note: d1, velocity: 0 } : { kind: "noteOn", channel, note: d1, velocity: d2 };
    case 0xb0:
      return { kind: "controller", channel, controller: d1, value: d2 };
    case 0xc0:
      return { kind: "program", channel, program: d1 };
    case 0xd0:
      return { kind: "pressure", channel, value: d1 };
    case 0xe0:
      return { kind: "bend", channel, value: d1 | (d2 << 7) };
    default:
      return { kind: "ignored", status }; // 0xa0, polyphonic key pressure
  }
}

/** The part of a `MIDIMessageEvent` the page reads. */
export interface MidiMessageLike {
  data: Uint8Array | null;
  timeStamp: number;
}

// A method signature is compared bivariantly, which lets the browser's own `MIDIInput` (whose handler takes the whole event) stand
// where this narrower one is asked for.
type MessageHandler = { bivarianceHack(event: MidiMessageLike): unknown }["bivarianceHack"];

/** The part of `MIDIInput` the page uses. Assigning `onmidimessage` opens the port. */
export interface MidiInputLike {
  readonly id: string;
  readonly name?: string | null;
  onmidimessage: MessageHandler | null;
}

export interface InputStats {
  /** Every message the chosen input handed over. */
  received: number;
  /** Those the sequencer understands: `received` less `ignored` and `malformed`. */
  decoded: number;
  ignored: number;
  malformed: number;
  /** Messages whose timestamp is earlier than the one before it on this input. The follower relies on them going forward. */
  backwards: number;
  byKind: Record<DecodedKind, number>;
  realtime: Record<RealtimeName, number>;
  /** The browser's timestamp of the latest message, or null if none yet. */
  lastTimeStamp: number | null;
}

function freshStats(): InputStats {
  return {
    received: 0,
    decoded: 0,
    ignored: 0,
    malformed: 0,
    backwards: 0,
    byKind: { noteOn: 0, noteOff: 0, controller: 0, bend: 0, pressure: 0, program: 0, realtime: 0 },
    realtime: { clock: 0, start: 0, continue: 0, stop: 0 },
    lastTimeStamp: null,
  };
}

export class InputPath {
  /** Where the consumers attach. Empty: nothing uses input yet. Called after the message is counted, for decoded messages only. */
  onMessage: ((message: Exclude<InMessage, { kind: "ignored" | "malformed" }>, timeStamp: number) => void) | null = null;
  stats: InputStats = freshStats();
  private port: MidiInputLike | null = null;

  get selected(): MidiInputLike | null {
    return this.port;
  }

  /** Listen to `port`, and to no other. `null` lets go. The counts start again, so each belongs to one port. */
  select(port: MidiInputLike | null): void {
    if (this.port) this.port.onmidimessage = null;
    this.port = port;
    this.stats = freshStats();
    if (port) port.onmidimessage = (event) => this.handle(event.data, event.timeStamp);
  }

  private handle(data: Uint8Array | null, timeStamp: number): void {
    const s = this.stats;
    s.received++;
    if (s.lastTimeStamp !== null && timeStamp < s.lastTimeStamp) s.backwards++;
    s.lastTimeStamp = timeStamp;
    const message = decodeMessage(data);
    if (message.kind === "malformed") {
      s.malformed++;
      return;
    }
    if (message.kind === "ignored") {
      s.ignored++;
      return;
    }
    s.decoded++;
    s.byKind[message.kind]++;
    if (message.kind === "realtime") s.realtime[message.message]++;
    this.onMessage?.(message, timeStamp);
  }
}

/** The sentence for the input line: where it is listening, or `fallback` (the browser's own sentence) when no input is chosen. */
export function describeInput(name: string | null, fallback: string): string {
  return name === null ? fallback : `Listening to ${name}. Nothing uses its messages yet.`;
}

/** What the chosen input has sent, in a sentence. Words are sentence case and the counts are the ones `InputStats` keeps. */
export function describeCount(stats: InputStats): string {
  if (stats.received === 0) return "No messages yet.";
  const n = stats.received;
  return `${n} ${n === 1 ? "message" : "messages"}: ${stats.decoded} used, ${stats.ignored} ignored, ${stats.malformed} malformed.`;
}

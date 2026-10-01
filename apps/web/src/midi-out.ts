// Turns the engine's events into Web MIDI sends (SPEC-0002 tech.md section 3, ADR-0008 5).
//
// An event has an audio-clock time. The page converts it to its own clock, adds the lookahead `L`
// so the timestamp is in the future when the send is made, and calls `send(data, timestamp)`. An
// event that is already late is sent at once and counted, never dropped.
//
// Two rules keep a Note Off from overtaking the Note On it ends, and so keep a note from sticking:
//
// 1. Timestamps on one output never go backwards. The audio-to-page clock map is re-read for each
//    batch and moves by a millisecond or two between reads, and a MIDI output delivers by timestamp.
//    A flush Note Off one sample after a Note On could otherwise be stamped before it.
// 2. `MIDIOutput.clear()` is never called. Chromium does not implement it (checked on 141), and where
//    it exists it also drops Note Offs the engine has already counted as sent, which leaves those
//    notes on. Stop works without it: the engine's flush Note Offs are stamped after everything
//    already queued, so what was queued plays out (at most `L` plus the clock jitter) and is then
//    ended. specs/SPEC-0002/p3-plan.md (finding F-P3-2) and ADR-0008 amendment 1.

import { EVENT_BYTES } from "./abi.ts";

export interface TimeMap {
  /** The page's high-resolution clock in milliseconds (`performance.now()`). */
  now(): number;
  /** The page time, in milliseconds, at which audio-context time `seconds` falls. */
  toPage(seconds: number): number;
}

/** The part of `MIDIOutput` the scheduler uses: `send`, and nothing else. */
export interface MidiOutputLike {
  send(data: ArrayLike<number>, timestamp?: number): void;
}

/**
 * A send is held back to follow the one before it only if its own time is less than this far behind.
 * A larger step back means the clocks jumped (the audio context was suspended and resumed, say), and
 * the new time is believed.
 */
export const ORDER_WINDOW_MS = 50;

export interface SentInfo {
  port: number;
  bytes: number[];
  /** The timestamp passed to `send`, or null if the event was late and went at once. */
  timestamp: number | null;
  /** `timestamp - now` at the call, in milliseconds. Negative means the event was late. */
  marginMs: number;
}

export interface SchedulerStats {
  sent: number;
  /** Events that were already due when they reached the scheduler, sent at once. */
  late: number;
  lateMaxMs: number;
  /** Events for a port with no output chosen. */
  unrouted: number;
  /** Records that are not valid MIDI (channel out of range, Note On with velocity 0) and were not sent. */
  invalid: number;
  /** Events whose timestamp was raised to follow the previous send to the same output, and the largest raise. */
  raised: number;
  raisedMaxMs: number;
}

/** The MIDI bytes for one 12-byte event record, or null if it is not valid MIDI. */
export function midiBytes(kind: number, channel: number, d1: number, d2: number): number[] | null {
  if (channel < 1 || channel > 16 || d1 > 127) return null;
  const c = channel - 1;
  switch (kind) {
    case 0:
      return d2 >= 1 && d2 <= 127 ? [0x90 | c, d1, d2] : null; // velocity 0 is a Note Off in MIDI; the engine never sends it
    case 1:
      return [0x80 | c, d1, 0];
    case 2:
      return d2 <= 127 ? [0xb0 | c, d1, d2] : null;
    case 3:
      return d2 <= 0x3fff ? [0xe0 | c, d2 & 0x7f, (d2 >> 7) & 0x7f] : null;
    case 4:
      return [0xd0 | c, d1];
    default:
      return null;
  }
}

export class MidiScheduler {
  lookaheadMs: number;
  readonly stats: SchedulerStats = { sent: 0, late: 0, lateMaxMs: 0, unrouted: 0, invalid: 0, raised: 0, raisedMaxMs: 0 };
  /** Called for every send, for the spikes. */
  onSend: ((info: SentInfo) => void) | null = null;
  private readonly time: TimeMap;
  private readonly outputs = new Map<number, MidiOutputLike>();
  /** The latest timestamp given to each output, for rule 1. */
  private readonly latest = new Map<MidiOutputLike, number>();

  constructor(time: TimeMap, lookaheadMs = 30) {
    this.time = time;
    this.lookaheadMs = lookaheadMs;
  }

  setOutput(port: number, output: MidiOutputLike | null): void {
    if (output) this.outputs.set(port, output);
    else this.outputs.delete(port);
    this.latest.clear();
  }

  /** `frame` is the audio context's frame at the start of the block the events came from. */
  schedule(frame: number, sampleRate: number, records: Uint8Array): void {
    const view = new DataView(records.buffer, records.byteOffset, records.byteLength);
    const base = frame / sampleRate;
    for (let i = 0; i + EVENT_BYTES <= records.byteLength; i += EVENT_BYTES) {
      const port = view.getUint8(i + 1);
      const output = this.outputs.get(port);
      if (!output) {
        this.stats.unrouted++;
        continue;
      }
      const bytes = midiBytes(view.getUint8(i), view.getUint8(i + 2), view.getUint8(i + 3), view.getUint16(i + 4, true));
      if (!bytes) {
        this.stats.invalid++;
        continue;
      }
      const at = base + view.getUint32(i + 8, true) / sampleRate;
      let target = this.time.toPage(at) + this.lookaheadMs;
      const before = this.latest.get(output);
      if (before !== undefined && target < before && before - target < ORDER_WINDOW_MS) {
        this.stats.raised++;
        this.stats.raisedMaxMs = Math.max(this.stats.raisedMaxMs, before - target);
        target = before;
      }
      this.latest.set(output, Math.max(target, before ?? target));
      const now = this.time.now();
      const marginMs = target - now;
      if (marginMs > 0) {
        output.send(bytes, target);
      } else {
        // Already due. A past or missing timestamp means "now" (W3C Web MIDI), and sends keep their order.
        output.send(bytes);
        this.stats.late++;
        this.stats.lateMaxMs = Math.max(this.stats.lateMaxMs, -marginMs);
      }
      this.stats.sent++;
      this.onSend?.({ port, bytes, timestamp: marginMs > 0 ? target : null, marginMs });
    }
  }
}

/**
 * Audio-context time to page time, from `AudioContext.getOutputTimestamp()`. Each call gives a pair
 * `(contextTime, performanceTime)`. The offset between the clocks, `performanceTime - contextTime`,
 * jitters from call to call, so `alpha` smooths it: 1 uses the latest pair alone, a smaller value
 * averages over about `1 / alpha` pairs. Spike S1 measures which is steadier.
 */
export class ContextTimeMap implements TimeMap {
  private offsetMs: number | null = null;
  private readonly source: { getOutputTimestamp(): { contextTime?: number; performanceTime?: number } };
  private readonly clock: { now(): number };
  private readonly alpha: number;

  constructor(source: { getOutputTimestamp(): { contextTime?: number; performanceTime?: number } }, clock: { now(): number }, alpha = 1) {
    this.source = source;
    this.clock = clock;
    this.alpha = alpha;
  }

  /** Take a new pair. Call it before converting a batch of events. */
  refresh(): void {
    const ts = this.source.getOutputTimestamp();
    if (ts.contextTime === undefined || ts.performanceTime === undefined) return;
    const offset = ts.performanceTime - ts.contextTime * 1000;
    this.offsetMs = this.offsetMs === null ? offset : this.offsetMs + this.alpha * (offset - this.offsetMs);
  }

  now(): number {
    return this.clock.now();
  }

  toPage(seconds: number): number {
    if (this.offsetMs === null) this.refresh();
    return seconds * 1000 + (this.offsetMs ?? 0);
  }
}

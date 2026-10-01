// A stand-in for a MIDI output and the synth behind it, with the behaviour that matters here:
// a send with a timestamp in the future is queued and delivered in timestamp order, a send without
// one (or in the past) is delivered at once and can overtake what is queued, `clear()` drops the
// queue (and is absent in browsers that do not implement it), and the synth holds a note from a
// Note On until a Note Off for the same channel and key.
import type { MidiOutputLike } from "../../src/midi-out.ts";

export class FakeClock {
  now = 0;
}

export type LogEntry =
  | { at: number; what: "send"; port: number; bytes: number[]; timestamp: number | null }
  | { at: number; what: "clear"; port: number }
  | { at: number; what: "deliver"; port: number; bytes: number[] };

interface Queued {
  timestamp: number;
  seq: number;
  bytes: number[];
}

export class FakeOutput implements MidiOutputLike {
  readonly held = new Set<string>();
  readonly delivered: { at: number; bytes: number[] }[] = [];
  clear?: () => void;
  clearCalls = 0;
  private queue: Queued[] = [];
  private seq = 0;
  private readonly clock: FakeClock;
  private readonly port: number;
  private readonly log: LogEntry[];

  constructor(clock: FakeClock, port: number, log: LogEntry[], supportsClear: boolean) {
    this.clock = clock;
    this.port = port;
    this.log = log;
    if (supportsClear) {
      this.clear = () => {
        this.clearCalls++;
        this.log.push({ at: this.clock.now, what: "clear", port: this.port });
        this.queue = [];
      };
    }
  }

  send(data: ArrayLike<number>, timestamp?: number): void {
    const bytes = Array.from(data);
    this.log.push({ at: this.clock.now, what: "send", port: this.port, bytes, timestamp: timestamp ?? null });
    if (timestamp === undefined || timestamp <= this.clock.now) this.deliver(bytes);
    else this.queue.push({ timestamp, seq: this.seq++, bytes });
  }

  /** Time passes: everything queued up to `t` is delivered, in timestamp order and then send order. */
  advanceTo(t: number): void {
    const due = this.queue.filter((q) => q.timestamp <= t).sort((a, b) => a.timestamp - b.timestamp || a.seq - b.seq);
    this.queue = this.queue.filter((q) => q.timestamp > t);
    const before = this.clock.now;
    for (const q of due) {
      this.clock.now = Math.max(before, q.timestamp);
      this.deliver(q.bytes);
    }
    this.clock.now = Math.max(before, t);
  }

  get queued(): number {
    return this.queue.length;
  }

  private deliver(bytes: number[]): void {
    this.log.push({ at: this.clock.now, what: "deliver", port: this.port, bytes });
    this.delivered.push({ at: this.clock.now, bytes });
    const status = (bytes[0] ?? 0) & 0xf0;
    const key = `${(bytes[0] ?? 0) & 0x0f}/${bytes[1]}`;
    if (status === 0x90 && (bytes[2] ?? 0) > 0) this.held.add(key);
    else if (status === 0x80 || (status === 0x90 && bytes[2] === 0)) this.held.delete(key);
  }
}

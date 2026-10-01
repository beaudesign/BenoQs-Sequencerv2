// Spike S1's arithmetic, apart from the page so a test can check it. A send is what the scheduler gave a
// MIDI output; a receipt is what a MIDI input in the same browser heard come back (a loopback port).

export interface Sent {
  bytes: number[];
  /** The timestamp the scheduler gave the output, or the call time if it went at once (page clock, ms). */
  target: number;
  /** When the engine made the event, in audio-context seconds. */
  audioTime: number;
  /** `timestamp - now` at the call, ms. */
  marginMs: number;
  /** The tab was hidden when the event was sent. */
  hidden: boolean;
}

export interface Receipt {
  bytes: number[];
  /** `MIDIMessageEvent.timeStamp`: when the browser got the message, on the page clock (ms). */
  at: number;
}

export interface Spread {
  n: number;
  mean: number;
  sigma: number;
  min: number;
  p50: number;
  p99: number;
  max: number;
}

export interface Analysis {
  sent: number;
  received: number;
  matched: number;
  /** Sent and never heard back. */
  missing: number;
  /** Heard and never sent (or sent fewer times). */
  extra: number;
  /** Arrival minus the timestamp it was given, ms. Its mean is the port's latency; its spread includes the clock map's error. */
  latencyMs: Spread;
  /**
   * How far each interval between two heard events is from the interval the engine made them at, ms (signed). This is the
   * jitter a listener hears: it has no constant latency and no clock map in it.
   */
  intervalErrorMs: Spread;
  /** The scheduling margin of every event sent, ms. */
  marginMs: Spread;
  late: number;
}

export function spread(values: number[]): Spread {
  const n = values.length;
  if (n === 0) return { n: 0, mean: 0, sigma: 0, min: 0, p50: 0, p99: 0, max: 0 };
  const v = [...values].sort((a, b) => a - b);
  const mean = v.reduce((a, b) => a + b, 0) / n;
  const sigma = Math.sqrt(v.reduce((a, b) => a + (b - mean) ** 2, 0) / n);
  const at = (q: number): number => v[Math.min(n - 1, Math.floor(q * n))] ?? 0;
  return { n, mean, sigma, min: v[0] ?? 0, p50: at(0.5), p99: at(0.99), max: v[n - 1] ?? 0 };
}

const same = (a: number[], b: number[]): boolean => a.length === b.length && a.every((x, i) => x === b[i]);

/** Pairs each receipt with the earliest unmatched send that has the same bytes. */
export function match(sent: Sent[], received: Receipt[]): { pairs: [Sent, Receipt][]; missing: Sent[]; extra: Receipt[] } {
  const free = sent.map((s, i) => ({ s, i, used: false }));
  const pairs: [Sent, Receipt][] = [];
  const extra: Receipt[] = [];
  for (const r of received) {
    const hit = free.find((f) => !f.used && same(f.s.bytes, r.bytes));
    if (hit) {
      hit.used = true;
      pairs.push([hit.s, r]);
    } else extra.push(r);
  }
  return { pairs, missing: free.filter((f) => !f.used).map((f) => f.s), extra };
}

export function analyse(sent: Sent[], received: Receipt[], filter?: (s: Sent) => boolean): Analysis {
  const kept = filter ? sent.filter(filter) : sent;
  const { pairs, missing, extra } = match(kept, received);
  const latency = pairs.map(([s, r]) => r.at - s.target);
  // Intervals between consecutive heard events, in the order the engine made them.
  const ordered = [...pairs].sort((a, b) => a[0].audioTime - b[0].audioTime);
  const interval: number[] = [];
  for (let i = 1; i < ordered.length; i++) {
    const [s0, r0] = ordered[i - 1]!;
    const [s1, r1] = ordered[i]!;
    interval.push(r1.at - r0.at - (s1.audioTime - s0.audioTime) * 1000);
  }
  return {
    sent: kept.length,
    received: received.length,
    matched: pairs.length,
    missing: missing.length,
    extra: filter ? 0 : extra.length,
    latencyMs: spread(latency),
    intervalErrorMs: spread(interval),
    marginMs: spread(kept.map((s) => s.marginMs)),
    late: kept.filter((s) => s.marginMs <= 0).length,
  };
}

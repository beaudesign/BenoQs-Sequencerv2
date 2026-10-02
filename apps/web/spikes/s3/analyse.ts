// Spike S3's arithmetic, apart from the pages so a test can check it (test/s3-analyse.test.ts). Two things are read here. A followed clock:
// what the page heard from a MIDI input, 0xF8 by 0xF8, and what the follower said about it every 250 ms. And a clock the app sends: the
// engine's pulses, sent to an output that comes back round a loopback port.
//
// The numbers it makes are the owner's to judge. The thresholds in this file (what is the same clock, what is a stall, how long the
// settling is, a fifth of a step) are the author's proposals, each one named where it is used.
import { spread, type Spread } from "../s1/analyse.ts";

export { spread };
export type { Spread };

/** A 0xF8 as the page saw it: `stamp` is the event's `timeStamp`, `seen` is `performance.now()` in the handler. Both in ms. */
export interface Pulse {
  stamp: number;
  seen: number;
}

/** What the follower said (`Follower.status`) at `at`, page time in ms, and whether the tab was hidden at the time. */
export interface FollowSample {
  at: number;
  phase: "waiting" | "settling" | "following" | "lost";
  bpm: number | null;
  phaseMs: number | null;
  jitterMs: number;
  pulses: number;
  running: boolean;
  hidden: boolean;
}

/** A 0xFA, 0xFB or 0xFC as the page saw it. */
export interface FollowMessage {
  name: "start" | "continue" | "stop";
  stamp: number;
  seen: number;
}

export interface FollowRaw {
  pulses: Pulse[];
  samples: FollowSample[];
  messages: FollowMessage[];
}

export interface FollowOptions {
  /** How long after a Start or a Continue the phase is not counted, ms. The follower catches up in about a second; three is a margin. Proposed. */
  settleMs?: number;
  /** The tempo the sender was set to, if it is known: the tempo the follower read is compared with it. */
  nominalBpm?: number;
  /** Only the samples taken while the tab was hidden (true) or visible (false). Left out: all of them. */
  hidden?: boolean;
}

export interface FollowAnalysis {
  pulses: number;
  /**
   * Whether the pulses' stamps are on `performance.now()`'s clock, the one the page reads `seen` from. The follower treats them as being,
   * and nothing in the Web MIDI text says they are for every browser. `ageMs` is `seen - stamp`: a few milliseconds if they are.
   */
  clockDomain: { ageMs: Spread; verdict: "same-clock" | "other-clock" | "no-data"; why: string };
  intervals: {
    /** The gaps between consecutive stamps, less the stalls, backwards and equal ones. */
    periodMs: Spread;
    /** `median` is the median of the tempos the periods give one by one; `fromRun` is the run's pulses over the run's time. */
    tempoBpm: { median: number; fromRun: number };
    /** How the tempo of a minute moves, a straight line through the minutes, in BPM an hour. 0 for a run under three minutes' blocks. */
    driftBpmPerHour: number;
    backwards: number;
    equalStamps: number;
    /** Gaps of under a millisecond in the time pulses were seen, between pulses whose stamps are further apart than that. */
    bursts: number;
    stalls: number;
  };
  phase: {
    counted: number;
    absMs: Spread;
    maxAbsMs: number;
    /** A fifth of a step at the tempo of the run: the plan's bound for S3. */
    fifthOfStepMs: number;
    beyondFifth: number;
    share: number;
    skipped: { settling: number; lost: number; stopped: number };
  };
  tempo: { meanErrorBpm: number | null; maxAbsErrorBpm: number | null };
}

/** A pulse is 1/24 of a quarter note and a step 1.5 pulses, so a step is 1/16 of a quarter: 3750 / bpm ms. A pulse is 2500 / bpm ms. */
const PULSE_MS_AT_1_BPM = 2500;
const STEP_MS_AT_1_BPM = 3750;

/** Stamps further apart than this are a stall (a pause in the sender, a tab the browser slept), not a period. Proposed; the slowest tempo an Octopus runs at, 30 BPM, has a pulse every 83 ms. */
const STALL_MS = 500;
/** The same clock if the stamps are never more than this in the future of the moment they were seen (a rounding) ... */
const FUTURE_TOLERANCE_MS = 5;
/** ... and 99 of 100 were seen within this long of their stamp (a busy page; a different clock is seconds to years). Proposed. */
const SAME_CLOCK_P99_MS = 100;
const DRIFT_BLOCK_MS = 60_000;
const DEFAULT_SETTLE_MS = 3_000;

const mean = (v: number[]): number => (v.length === 0 ? 0 : v.reduce((a, b) => a + b, 0) / v.length);
const median = (v: number[]): number => spread(v).p50;

function clockDomain(pulses: Pulse[]): FollowAnalysis["clockDomain"] {
  const ageMs = spread(pulses.map((p) => p.seen - p.stamp));
  if (pulses.length === 0) return { ageMs, verdict: "no-data", why: "no pulses were heard" };
  const detail = `seen less stamped: median ${ageMs.p50.toFixed(1)} ms, smallest ${ageMs.min.toFixed(1)}, 99th ${ageMs.p99.toFixed(1)}`;
  if (ageMs.min < -FUTURE_TOLERANCE_MS) return { ageMs, verdict: "other-clock", why: `stamps in the future of the moment they were seen (${detail})` };
  if (ageMs.p99 > SAME_CLOCK_P99_MS) return { ageMs, verdict: "other-clock", why: `stamps a long way behind the moment they were seen (${detail})` };
  return { ageMs, verdict: "same-clock", why: detail };
}

function intervals(pulses: Pulse[]): FollowAnalysis["intervals"] {
  const periods: number[] = [];
  let backwards = 0;
  let equalStamps = 0;
  let stalls = 0;
  let bursts = 0;
  /** The pulses each period belongs to, for the blocks. */
  const periodAt: { at: number; period: number }[] = [];
  for (let i = 1; i < pulses.length; i++) {
    const a = pulses[i - 1]!;
    const b = pulses[i]!;
    const dt = b.stamp - a.stamp;
    if (b.seen - a.seen < 1 && dt >= 5) bursts++;
    if (dt < 0) backwards++;
    else if (dt === 0) equalStamps++;
    else if (dt > STALL_MS) stalls++;
    else {
      periods.push(dt);
      periodAt.push({ at: b.stamp, period: dt });
    }
  }
  const sum = periods.reduce((a, b) => a + b, 0);
  const tempos = periods.map((p) => PULSE_MS_AT_1_BPM / p);

  // Blocks of a minute of stamps, each one's tempo from its own periods. A block short of 50 s (the end of a run) is left out.
  const blocks = new Map<number, { count: number; sum: number; at: number[] }>();
  const first = periodAt[0]?.at ?? 0;
  for (const p of periodAt) {
    const k = Math.floor((p.at - first) / DRIFT_BLOCK_MS);
    const block = blocks.get(k) ?? { count: 0, sum: 0, at: [] };
    block.count++;
    block.sum += p.period;
    block.at.push(p.at);
    blocks.set(k, block);
  }
  const points = [...blocks.values()].filter((b) => b.sum >= DRIFT_BLOCK_MS - 10_000).map((b) => ({ hours: mean(b.at) / 3_600_000, bpm: (PULSE_MS_AT_1_BPM * b.count) / b.sum }));
  let drift = 0;
  if (points.length >= 3) {
    const mx = mean(points.map((p) => p.hours));
    const my = mean(points.map((p) => p.bpm));
    const sxy = points.reduce((a, p) => a + (p.hours - mx) * (p.bpm - my), 0);
    const sxx = points.reduce((a, p) => a + (p.hours - mx) ** 2, 0);
    drift = sxx > 0 ? sxy / sxx : 0;
  }
  return {
    periodMs: spread(periods),
    tempoBpm: { median: median(tempos), fromRun: sum > 0 ? (PULSE_MS_AT_1_BPM * periods.length) / sum : 0 },
    driftBpmPerHour: drift,
    backwards,
    equalStamps,
    bursts,
    stalls,
  };
}

export function analyseFollow(raw: FollowRaw, options: FollowOptions = {}): FollowAnalysis {
  const settleMs = options.settleMs ?? DEFAULT_SETTLE_MS;
  const iv = intervals(raw.pulses);
  // From the run's own tempo (pulses over time), not the median of the periods, which a sender whose pulses are quantised to a millisecond skews.
  const fifthOfStepMs = iv.tempoBpm.fromRun > 0 ? STEP_MS_AT_1_BPM / iv.tempoBpm.fromRun / 5 : 0;

  // The latest Start or Continue seen at or before each sample: the settling is counted from it.
  const starts = raw.messages.filter((m) => m.name !== "stop").map((m) => m.seen).sort((a, b) => a - b);
  const startBefore = (at: number): number | null => {
    let found: number | null = null;
    for (const s of starts) {
      if (s > at) break;
      found = s;
    }
    return found;
  };

  const phases: number[] = [];
  const tempoErrors: number[] = [];
  const skipped = { settling: 0, lost: 0, stopped: 0 };
  for (const s of raw.samples) {
    if (options.hidden !== undefined && s.hidden !== options.hidden) continue;
    // In this order: a transport that is not running, or a sample with no phase to read, is stopped; a sample in the first moments after a
    // Start or a Continue, or while the follower itself is still waiting or settling, is settling; then a clock that is lost.
    if (!s.running || s.phaseMs === null) {
      skipped.stopped++;
      continue;
    }
    const since = startBefore(s.at);
    if (since === null || s.at - since < settleMs || s.phase === "waiting" || s.phase === "settling") {
      skipped.settling++;
      continue;
    }
    if (s.phase === "lost") {
      skipped.lost++;
      continue;
    }
    phases.push(s.phaseMs);
    if (options.nominalBpm !== undefined && s.bpm !== null) tempoErrors.push(s.bpm - options.nominalBpm);
  }
  const absMs = spread(phases.map(Math.abs));
  const beyondFifth = phases.filter((p) => Math.abs(p) > fifthOfStepMs).length;
  return {
    pulses: raw.pulses.length,
    clockDomain: clockDomain(raw.pulses),
    intervals: iv,
    phase: { counted: phases.length, absMs, maxAbsMs: absMs.max, fifthOfStepMs, beyondFifth, share: phases.length > 0 ? beyondFifth / phases.length : 0, skipped },
    tempo: tempoErrors.length > 0 ? { meanErrorBpm: mean(tempoErrors), maxAbsErrorBpm: Math.max(...tempoErrors.map(Math.abs)) } : { meanErrorBpm: null, maxAbsErrorBpm: null },
  };
}

export interface LoopAnalysis {
  sent: number;
  received: number;
  matched: number;
  missing: number;
  extra: number;
  /** Heard less the time it was sent for, ms: the port's latency, with the page's clock map in it. */
  latencyMs: Spread;
  /** How far each interval between two heard pulses is from the one they were sent at, ms (signed): the jitter a listener hears, with no latency in it. */
  intervalErrorMs: Spread;
}

/**
 * The k-th pulse sent is the k-th heard: a 0xF8 carries nothing to tell one from another. A pulse lost in the middle of a run shifts every
 * pair after it, so a run with `missing` or `extra` above zero has latency and interval figures that cannot be trusted, and the report says so.
 */
export function analyseLoop(sent: { target: number }[], heard: { at: number }[]): LoopAnalysis {
  const matched = Math.min(sent.length, heard.length);
  const latency: number[] = [];
  const interval: number[] = [];
  for (let k = 0; k < matched; k++) {
    latency.push(heard[k]!.at - sent[k]!.target);
    if (k > 0) interval.push(heard[k]!.at - heard[k - 1]!.at - (sent[k]!.target - sent[k - 1]!.target));
  }
  return {
    sent: sent.length,
    received: heard.length,
    matched,
    missing: sent.length - matched,
    extra: heard.length - matched,
    latencyMs: spread(latency),
    intervalErrorMs: spread(interval),
  };
}

/** A saved pulse: [stamp, seen]. A run of 30 minutes at 120 BPM has 86,400 of them, so the saved file keeps each as a pair and not an object. */
export type SavedPulse = [number, number];

export function unpackPulses(saved: SavedPulse[]): Pulse[] {
  return saved.map(([stamp, seen]) => ({ stamp, seen }));
}

export interface Verdict {
  /** The plan's S3 criterion (every phase sample within a fifth of a step): true if met, false if not, null if the run gives nothing to read it from. */
  met: boolean | null;
  lines: string[];
}

/** The plan asks for 30 minutes (p4-plan §5). */
const S3_SECONDS = 1_800;

/**
 * The plan's S3 criterion, read from an analysis: the phase within a fifth of a step for the whole run, with the tab visible. That bound is
 * the author's proposal (the owner's to set) and `met` is nothing more than that proposal applied. `null` means there is nothing to read:
 * the stamps are not on the page's clock (then the phase is not what it says), there were no pulses, or no phase was counted.
 */
export function followVerdict(a: FollowAnalysis, runSeconds: number, hiddenSeconds: number): Verdict {
  const lines: string[] = [];
  let met: boolean | null = null;
  const tempo = a.intervals.tempoBpm.fromRun;
  if (a.clockDomain.verdict === "no-data") {
    lines.push("No pulses were heard: nothing to read.");
  } else if (a.clockDomain.verdict === "other-clock") {
    lines.push(`THE STAMPS ARE NOT ON THE PAGE'S CLOCK (${a.clockDomain.why}). The follower takes them to be, so the phase figures do not mean what they say; this is the finding.`);
  } else if (a.phase.counted === 0) {
    const s = a.phase.skipped;
    lines.push(`No phase was counted (settling ${s.settling}, lost ${s.lost}, stopped ${s.stopped}). If most were stopped, no Start was heard: press Play on the sender after the run has begun, because a clock alone does not start the engine.`);
  } else {
    met = a.phase.beyondFifth === 0;
    const fifth = `a fifth of a step (${a.phase.fifthOfStepMs.toFixed(2)} ms at ${tempo.toFixed(1)} BPM)`;
    lines.push(
      met
        ? `All ${a.phase.counted} samples were within ${fifth}; the largest was ${a.phase.maxAbsMs.toFixed(1)} ms.`
        : `${a.phase.beyondFifth} of ${a.phase.counted} samples were beyond ${fifth}; the largest was ${a.phase.maxAbsMs.toFixed(1)} ms.`,
    );
  }
  if (runSeconds < S3_SECONDS) lines.push(`The run is ${Math.round(runSeconds)} s, shorter than the 30 minutes the plan asks for, so it is not the S3 run.`);
  if (hiddenSeconds > 0) lines.push(`The tab was hidden for ${Math.round(hiddenSeconds)} s of ${Math.round(runSeconds)}. The plan's criterion is for a visible tab: read the visible samples and the hidden ones apart.`);
  return { met, lines };
}

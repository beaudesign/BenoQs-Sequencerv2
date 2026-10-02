// The clock estimator (SPEC-0002 P4 plan observable M4, ADR-0009 decision 5): the page-clock time of each incoming 0xF8 in; a tempo, the
// place of the next pulse and a lock state out. It is a pure function of the times it is given (no clock, no random numbers, no I/O), so
// the same pulses always give the same answers and the fixtures in test/clock-estimator.test.ts can say exactly what it does.
//
// The model. A sender's pulses fall on a grid `t0 + k * period`. The estimator keeps two numbers, the filtered time of the latest pulse
// and the period, and a 2 x 2 covariance for them (a Kalman filter). Each pulse is a noisy look at the grid: the innovation is how far it
// fell from where the filter said it would. Three things make it fit a clock and not a textbook:
//
//   * Which pulse it was. The sender's count is not known, so a pulse's index is `round((t - t0) / period)`. One is the next pulse, two
//     means one pulse was never heard, and zero means the same pulse again. The index is taken against the filtered grid and not against
//     the previous raw time, so one pulse's jitter does not move the rounding of the next. This needs the jitter to stay well under half
//     a period (10 ms at 120 BPM); beyond that the index is a guess, and the gate and the relock below are what keep a wrong guess short.
//   * How much to trust the grid. The process noise `q`, how fast the period may drift, is small when the tempo is steady (so the estimate
//     averages over a couple of seconds and holds 0.1 BPM under 3 ms of jitter) and rises when the innovations lean one way for a while,
//     which is what a tempo ramp does to a filter that believes the tempo is constant. It falls back when they stop leaning.
//   * What not to believe. A pulse far from the grid (more than `gateSigmas` standard deviations of the innovation) is an outlier and
//     does not move anything. `relockAfter` of them in a row mean the grid is wrong and not the pulses (a step in tempo, say), so the
//     filter starts again from those pulses. A gap of more than 500 ms is a stall: the next pulse starts again and the count restarts.
//
// What this does not know: the real jitter of a real port. S3 measures it (plan section 5); the 3 ms in the fixtures is a figure from the
// plan and not a measurement. `jitterMs` is the assumed standard deviation of a timestamp, and the default is the figure those fixtures
// were tuned at, so a quieter sender is followed a little more slowly than it could be.

export type LockState = "idle" | "locking" | "locked" | "lost";
export type PushOutcome = "first" | "seeding" | "accepted" | "duplicate" | "outlier" | "relocked" | "restarted" | "backwards" | "ignored";

export interface EstimatorConfig {
  /** The standard deviation of a timestamp's error that the filter assumes, in milliseconds. */
  jitterMs: number;
  /** How many standard deviations from the grid a pulse may fall before it is turned away. */
  gateSigmas: number;
  /** Pulses turned away in a row before the filter starts again from them. */
  relockAfter: number;
  /** The process noise (milliseconds squared of period drift per pulse) when the tempo is steady, and the most it rises to. */
  qMin: number;
  qMax: number;
  /** How many innovations the lean is judged over, how many standard deviations of lean it takes to start raising `q`, and how fast it rises. */
  biasWindow: number;
  biasThreshold: number;
  biasGain: number;
}

export const DEFAULT_ESTIMATOR: Readonly<EstimatorConfig> = {
  jitterMs: 2,
  gateSigmas: 4,
  relockAfter: 4,
  qMin: 1e-7,
  qMax: 1e-3,
  biasWindow: 16,
  biasThreshold: 2.5,
  biasGain: 12,
};

export interface EstimatorStats {
  /** Pulses that moved the grid. */
  accepted: number;
  /** The same pulse again, a moment later: ignored. */
  duplicates: number;
  /** Pulses that fell too far from the grid: ignored (unless they were `relockAfter` in a row). */
  outliers: number;
  /** Pulses the sender made that were never heard, counted from the gaps. */
  dropped: number;
  /** Times the filter started again because the grid was wrong. */
  relocks: number;
  /** Times it started again because the clock had stopped for more than `LOST_AFTER_MS`. */
  restarts: number;
  /** Timestamps not later than the one before, ignored. */
  backwards: number;
  /** Things that were not finite numbers, ignored. */
  ignored: number;
  /** The root mean square of the innovations, in milliseconds: how far the pulses fall from the grid the filter predicted. */
  jitterMs: number;
}

/** With no pulse for longer than this the clock is `lost`, and the next pulse starts a new lock. Plan M5. */
export const LOST_AFTER_MS = 500;
/** Pulses (24 to the quarter note, so one beat) from the start of a lock until it is called locked. */
export const LOCK_PULSES = 24;
/** Consecutive pulses the first tempo is fitted to. A pulse that is missing among them is read as a slower tempo, and corrected once it is locked. */
export const SEED_PULSES = 4;
/** How many of the latest messages are looked at for half of them having been turned away, and how many is half. */
const MESSAGE_WINDOW = 8;
const HALF_OF_WINDOW = 4;
/** Accepted pulses in a row that each follow a gap before the gaps are taken to be a slower tempo. */
const GAPS_IN_A_ROW = 3;
/** While locking, a message this close to the last pulse (a fraction of a period) is the same pulse again. */
const REPEAT_WITHIN = 0.25;
/** How much each innovation counts toward the jitter figure (about the last 32). */
const JITTER_ALPHA = 1 / 32;

function freshStats(): EstimatorStats {
  return { accepted: 0, duplicates: 0, outliers: 0, dropped: 0, relocks: 0, restarts: 0, backwards: 0, ignored: 0, jitterMs: 0 };
}

export class ClockEstimator {
  readonly stats: EstimatorStats = freshStats();
  private readonly cfg: EstimatorConfig;
  private readonly r: number;
  // The filter: the filtered time of the latest accepted pulse, the period, and their covariance (c00, c01, c11).
  private t0 = 0;
  private period = 0;
  private c00 = 0;
  private c01 = 0;
  private c11 = 0;
  private q: number;
  private seeded = false;
  private seeds: number[] = [];
  private pending: number[] = [];
  /** The last few messages and whether each was turned away (a repeat or an outlier), for noticing that most of them are. */
  private messages: { at: number; turnedAway: boolean }[] = [];
  private innovations: number[] = [];
  private jitterSquared = 0;
  /** The time of the latest message of any kind that was in order, for stalls and for ordering. */
  private lastRaw: number | null = null;
  /** The time, as it arrived, of the latest pulse that moved the grid. */
  private lastAcceptedRaw = 0;
  /** The arrival times of the last few pulses that moved the grid, to start again from if the tempo turns out to have changed. */
  private recent: number[] = [];
  /** How many accepted pulses in a row were read as following a gap, and how many pulses those gaps were said to hold. */
  private gapRun = 0;
  private gapExcess = 0;
  /** The sender's count for the latest accepted pulse, from 0 at the first pulse of the epoch. */
  private count = 0;
  /** Pulses accepted since the lock began (the seed included). */
  private sinceStart = 0;
  private epochNumber = 0;

  constructor(config: Partial<EstimatorConfig> = {}) {
    this.cfg = { ...DEFAULT_ESTIMATOR, ...config };
    this.r = this.cfg.jitterMs * this.cfg.jitterMs;
    this.q = this.cfg.qMin;
  }

  /** The tempo, in BPM (24 pulses to the quarter note), or null before the first tempo can be fitted. Kept while the clock is lost. */
  get bpm(): number | null {
    return this.seeded ? 2500 / this.period : null;
  }

  get periodMs(): number | null {
    return this.seeded ? this.period : null;
  }

  /** Counts the times the pulse count began again (a stall). A missing pulse does not change it; the count goes on over it. */
  get epoch(): number {
    return this.epochNumber;
  }

  /** The place, in the page's milliseconds, of the pulse `pulses` after the latest accepted one. Negative and fractional are fine. */
  gridTime(pulses: number): number | null {
    return this.seeded ? this.t0 + pulses * this.period : null;
  }

  /** The sender's pulse count at page time `timeMs`, as a fraction. Counts from 0 at the first pulse of the epoch. */
  pulseIndex(timeMs: number): number | null {
    return this.seeded ? this.count + (timeMs - this.t0) / this.period : null;
  }

  /** `now` is the page's clock. Lost is a matter of how long ago the last pulse was, so it can only be said from outside. */
  state(nowMs: number): LockState {
    if (this.lastRaw === null) return "idle";
    if (nowMs - this.lastRaw > LOST_AFTER_MS) return "lost";
    return this.sinceStart < LOCK_PULSES ? "locking" : "locked";
  }

  reset(): void {
    Object.assign(this.stats, freshStats());
    this.t0 = this.period = this.c00 = this.c01 = this.c11 = 0;
    this.q = this.cfg.qMin;
    this.seeded = false;
    this.seeds = [];
    this.pending = [];
    this.messages = [];
    this.innovations = [];
    this.jitterSquared = 0;
    this.lastRaw = null;
    this.lastAcceptedRaw = 0;
    this.recent = [];
    this.gapRun = 0;
    this.gapExcess = 0;
    this.count = 0;
    this.sinceStart = 0;
    this.epochNumber = 0;
  }

  /** One incoming 0xF8, at page time `timeMs`. Never throws. */
  push(timeMs: number): PushOutcome {
    if (typeof timeMs !== "number" || !Number.isFinite(timeMs)) {
      this.stats.ignored++;
      return "ignored";
    }
    if (this.lastRaw !== null && timeMs <= this.lastRaw) {
      this.stats.backwards++;
      return "backwards";
    }
    if (this.lastRaw === null) return this.start(timeMs, "first");
    if (timeMs - this.lastRaw > LOST_AFTER_MS) {
      this.stats.restarts++;
      this.epochNumber++;
      return this.start(timeMs, "restarted");
    }
    this.lastRaw = timeMs;
    return this.seeded ? this.follow(timeMs) : this.seed(timeMs);
  }

  /** A new lock begins at this pulse: the count is 0, nothing is known of the period. */
  private start(timeMs: number, outcome: "first" | "restarted"): PushOutcome {
    this.lastRaw = timeMs;
    this.lastAcceptedRaw = timeMs;
    this.recent = [timeMs];
    this.gapRun = 0;
    this.gapExcess = 0;
    this.seeded = false;
    this.seeds = [timeMs];
    this.pending = [];
    this.messages = [];
    this.innovations = [];
    this.q = this.cfg.qMin;
    this.count = 0;
    this.sinceStart = 1;
    return outcome;
  }

  private seed(timeMs: number): PushOutcome {
    this.seeds.push(timeMs);
    this.lastAcceptedRaw = timeMs;
    this.recent.push(timeMs);
    this.sinceStart++;
    this.count++;
    if (this.seeds.length >= SEED_PULSES) this.fit(this.seeds);
    return "seeding";
  }

  /** A straight line through consecutive pulses by least squares: the first period, and the covariance that line has. */
  private fit(times: number[]): void {
    const n = times.length;
    const mean = (n - 1) / 2;
    let sxx = 0;
    let sxy = 0;
    const average = times.reduce((a, b) => a + b, 0) / n;
    for (let i = 0; i < n; i++) {
      sxx += (i - mean) * (i - mean);
      sxy += (i - mean) * (times[i]! - average);
    }
    const period = sxy / sxx;
    const lastOffset = n - 1 - mean;
    this.period = period;
    this.t0 = average + period * lastOffset;
    this.c00 = this.r * (1 / n + (lastOffset * lastOffset) / sxx);
    this.c01 = (-this.r * lastOffset) / sxx;
    this.c11 = this.r / sxx;
    let sse = 0;
    for (let i = 0; i < n; i++) {
      const miss = times[i]! - (this.t0 - (n - 1 - i) * period);
      sse += miss * miss;
    }
    this.jitterSquared = n > 2 ? sse / (n - 2) : 0;
    this.stats.jitterMs = Math.sqrt(this.jitterSquared);
    this.seeds = [];
    this.pending = [];
    this.messages = [];
    this.innovations = [];
    this.q = this.cfg.qMin;
    this.seeded = Number.isFinite(period) && period > 0;
    if (!this.seeded) this.restartAt(times[times.length - 1]!);
  }

  /** The fit made no sense (a period of zero or less): begin again from the latest pulse, as a stall would, without counting one. */
  private restartAt(timeMs: number): void {
    this.seeds = [timeMs];
    this.seeded = false;
    this.sinceStart = 1;
    this.count = 0;
  }

  private follow(timeMs: number): PushOutcome {
    const dkRaw = Math.round((timeMs - this.t0) / this.period);
    // Until a beat of pulses has been accepted the period is not known well enough to count pulses by it (at 200 BPM, with 3 ms of
    // jitter, the first fit is out by 6% and a late pulse reads as two, or an early one as a repeat). So while locking, every message
    // is the next pulse; only one that follows the last within a quarter of a period is called a repeat; and one that does not fit is
    // turned away and, if there are enough of them, starts the lock again. A pulse that went missing in the first beat costs a lock,
    // not a tempo.
    const locking = this.sinceStart < LOCK_PULSES;
    if (locking ? timeMs - this.lastAcceptedRaw < REPEAT_WITHIN * this.period : dkRaw < 1) {
      this.stats.duplicates++;
      return this.turnedAway(timeMs) ?? "duplicate";
    }
    const held = this.pending.length;
    // After a pulse was turned away the next real one is either the next pulse (the turned-away one was a stray) or `held + 1` on
    // (it was a real pulse that fell far out). Anything else is not read as a gap: this is what stops a step in tempo being taken for
    // pulses that were never heard.
    const readable = held === 0 || dkRaw === 1 || dkRaw === held + 1;
    const dk = locking ? 1 : dkRaw;
    const p00 = this.c00 + 2 * dk * this.c01 + dk * dk * this.c11;
    const p01 = this.c01 + dk * this.c11;
    const p11 = this.c11 + this.q * dk;
    const s = p00 + this.r;
    const miss = timeMs - (this.t0 + dk * this.period);
    if (!readable || Math.abs(miss) > this.cfg.gateSigmas * Math.sqrt(s)) {
      this.stats.outliers++;
      this.pending.push(timeMs);
      return this.turnedAway(timeMs) ?? "outlier";
    }

    const k0 = p00 / s;
    const k1 = p01 / s;
    this.t0 = this.t0 + dk * this.period + k0 * miss;
    this.period = this.period + k1 * miss;
    this.c00 = (1 - k0) * p00;
    this.c01 = (1 - k0) * p01;
    this.c11 = p11 - k1 * p01;
    this.stats.accepted++;
    this.lastAcceptedRaw = timeMs;
    this.remember(timeMs, false);
    this.recent.push(timeMs);
    if (this.recent.length > SEED_PULSES) this.recent.shift();
    if (held === 0) this.stats.dropped += dk - 1;
    if (held === 0 && dk >= 2) {
      this.gapRun++;
      this.gapExcess += dk - 1;
    } else {
      this.gapRun = 0;
      this.gapExcess = 0;
    }
    this.count += dk;
    this.sinceStart++;
    this.pending = [];
    this.jitterSquared += JITTER_ALPHA * (miss * miss - this.jitterSquared);
    this.stats.jitterMs = Math.sqrt(this.jitterSquared);
    this.adapt(miss / Math.sqrt(s));
    if (!(this.period > 0) || !Number.isFinite(this.t0) || !Number.isFinite(this.period)) {
      this.seeded = false;
      this.restartAt(timeMs);
    } else if (this.gapRun >= GAPS_IN_A_ROW) {
      return this.readAsNewTempo(timeMs);
    }
    return "accepted";
  }

  /**
   * Every pulse for a while has come after a gap: not pulses going missing but a tempo that is a half, a third, a quarter of the old
   * one, which the gaps explain just as well (a step from 120 to 60 BPM is every second pulse "missing"). Real losses are not in a row
   * like this. The gaps are given back and the filter starts again from the last pulses, which are consecutive at the new tempo.
   */
  private readAsNewTempo(timeMs: number): PushOutcome {
    this.stats.dropped -= this.gapExcess;
    this.count -= this.gapExcess;
    this.stats.relocks++;
    const times = this.recent.slice();
    this.gapRun = 0;
    this.gapExcess = 0;
    this.fit(times);
    this.lastAcceptedRaw = timeMs;
    this.sinceStart = this.seeded ? times.length : 1;
    return "relocked";
  }

  /**
   * The process noise follows how much the recent innovations lean one way. For a steady clock their sum is a random walk, about
   * `sqrt(n)` standard deviations after `n`; a tempo ramp makes it grow in a straight line. Above `biasThreshold` the noise rises
   * exponentially, up to `qMax`, and below it falls back to `qMin`.
   */
  private adapt(normalised: number): void {
    this.innovations.push(normalised);
    if (this.innovations.length > this.cfg.biasWindow) this.innovations.shift();
    const lean = Math.abs(this.innovations.reduce((a, b) => a + b, 0)) / Math.sqrt(this.innovations.length);
    const over = Math.max(0, lean - this.cfg.biasThreshold);
    this.q = Math.min(this.cfg.qMax, this.cfg.qMin * Math.exp(this.cfg.biasGain * over));
  }

  private remember(at: number, turnedAway: boolean): void {
    this.messages.push({ at, turnedAway });
    if (this.messages.length > MESSAGE_WINDOW) this.messages.shift();
  }

  /**
   * A message was turned away. If the last `relockAfter` were all turned away, or half of the last eight were, it is the grid that is
   * wrong and not the pulses: a step in tempo, or a new tempo that lands on the old grid now and then (a doubled tempo falls on it
   * every other pulse, which a run of turned-away pulses alone would miss). The filter starts again from the latest messages, which
   * were consecutive pulses of the new grid. Returns `relocked` when it did, and null when the message is just turned away.
   */
  private turnedAway(timeMs: number): PushOutcome | null {
    this.remember(timeMs, true);
    const last = this.messages.slice(-this.cfg.relockAfter);
    const allTurnedAway = last.length >= this.cfg.relockAfter && last.every((m) => m.turnedAway);
    const halfTurnedAway = this.messages.length >= HALF_OF_WINDOW && this.messages.filter((m) => m.turnedAway).length >= HALF_OF_WINDOW;
    if (!allTurnedAway && !halfTurnedAway) return null;
    this.count += this.pending.length;
    this.stats.relocks++;
    this.sinceStart = 0;
    this.fit(last.map((m) => m.at));
    this.lastAcceptedRaw = timeMs;
    this.recent = last.map((m) => m.at);
    this.gapRun = 0;
    this.gapExcess = 0;
    this.sinceStart = this.seeded ? last.length : 1;
    return "relocked";
  }
}

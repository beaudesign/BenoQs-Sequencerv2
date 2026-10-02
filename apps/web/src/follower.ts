// The follower (SPEC-0002 P4 plan observables M4 and M5, ADR-0009 decision 5): what an incoming MIDI clock does to the engine.
//
// The engine keeps its own sample clock and ticks on it; it does not tick on pulses (that is "host lock", a larger change that waits for
// D0 and for spike S3 to say it is needed). So the page steers it from outside, with the two things it already owns, the tempo
// (`octoweb_set_tempo`) and the transport, and with the engine's position coming back in the panel message:
//
//   estimator   the pulses' times in, a tempo and the place of the pulse grid out                    (src/clock-estimator.ts)
//   comparator  where the engine is, against where the sender's grid says it should be, in time
//   actuator    the engine's tempo is the sender's tempo trimmed in proportion to that error (a first-order loop)
//   lead        the engine runs ahead of the grid by the lookahead plus a user offset (below)
//
// Start, Continue and Stop follow 0xFA, 0xFB and 0xFC; a pulse alone never starts the transport; with no pulse for 500 ms the status
// says `lost` and nothing more is sent, so the engine plays on at the last tempo it was given (plan M5).
//
// THE LEAD. Every event the engine makes reaches the MIDI output a lookahead `L` after the engine made it (src/midi-out.ts), so for a
// note to land on the sender's beat the engine has to be that far ahead of it: the engine's pulse `k` should fall at the sender's
// pulse `k` minus `L` minus the offset. In terms of position: at page time `t` the engine should be where the sender's grid will be at
// `t + L + offset`. A positive offset makes the notes land earlier (it is the same constant that Live's "MIDI Clock Sync Delay"
// corrects, with the opposite sign: that one delays what Live sends, this one advances what the app plays).
//
// ALIGNMENT. At the first pulse after a Start or Continue the engine is made to coincide with the sender's pulse: the engine's whole
// pulse position at that moment (it does not rewind on Stop, so it resumes where it stopped, rounded to a whole pulse) is taken to be the
// sender's pulse number at that moment. From then on the error is counted in whole pulses and not modulo one: the engine starts `L`
// late (the transport can only be started when the first pulse arrives, and by then the engine should already be `L` ahead) and the
// loop catches up, instead of settling a whole pulse out, which would be a downbeat 20 ms off. When the count cannot be trusted (a
// stall, the estimator starting again, a Start while running) it is taken again as the nearest whole pulse, so only the fine phase is
// kept. What it does NOT do: follow song position (0xF2 is decoded and ignored), so a Continue from the middle of a song resumes where
// the sequencer stopped and not where the sender is.
import { TICKS_PER_CLOCK } from "./abi.ts";
import { ClockEstimator } from "./clock-estimator.ts";
import type { RealtimeName } from "./midi-in.ts";
import { LOOKAHEAD_MS } from "./settings.ts";

export interface FollowerActions {
  setTempo(bpm: number): void;
  transport(play: boolean): void;
}

export interface FollowerSettings {
  /** The lookahead the scheduler adds to every event, in milliseconds. Keep it equal to the scheduler's. */
  lookaheadMs: number;
  /** Extra lead in milliseconds: positive makes the notes land earlier. */
  offsetMs: number;
}

export interface FollowerTuning {
  /** The time constant of the loop: an error of this many milliseconds asks for a trim of 100% (before the limit). */
  trimTimeMs: number;
  /** The most the tempo is trimmed away from the sender's, as a fraction. */
  maxTrim: number;
  /** A tempo that differs from the last one sent by less than this, in BPM, is not sent. */
  minTempoChange: number;
}

export const FOLLOWER: Readonly<FollowerTuning> = { trimTimeMs: 250, maxTrim: 0.08, minTempoChange: 0.01 };
/** The tempos `octoweb_set_tempo` accepts (ABI.md). */
export const ENGINE_BPM = { min: 1, max: 999 } as const;
/** An engine more than this far from where the grid says it should be (in time) gives up on counting whole pulses and takes the nearest. */
const GIVE_UP_MS = 500;

export interface PositionSample {
  /** The engine's tick position (`octoweb_tick_position`): ticks, as a fraction. */
  ticks: number;
  /** The page time at which it held. */
  pageTimeMs: number;
}

export type FollowPhase = "waiting" | "settling" | "following" | "lost";

export interface FollowStatus {
  phase: FollowPhase;
  /** The tempo read from the clock, or null before there is one. Still reported while lost. */
  bpm: number | null;
  /** Whether the transport is running because the sender said so. */
  running: boolean;
  /** How far the engine is from where it should be, in milliseconds (positive: ahead). Null when there is nothing to say yet. */
  phaseMs: number | null;
  /** How much the pulses' times scatter, in milliseconds. */
  jitterMs: number;
  /** Pulses used so far. */
  pulses: number;
}

const finite = (n: unknown): n is number => typeof n === "number" && Number.isFinite(n);

export class Follower {
  readonly estimator: ClockEstimator;
  private readonly actions: FollowerActions;
  private readonly settings: FollowerSettings;
  private readonly tuning: FollowerTuning;
  private believedRunning = false;
  private startPending = false;
  /** Sender's pulse `ext` is the engine's pulse `eng`, in whole pulses, once the lead is allowed for. Null until set. */
  private anchor: { ext: number; eng: number } | null = null;
  private retakeAnchor = false;
  private sample: PositionSample | null = null;
  private applied: number | null = null;
  private trim = 0;
  private phase: number | null = null;
  private epoch = 0;

  constructor(
    actions: FollowerActions,
    settings: Partial<FollowerSettings> = {},
    tuning: Partial<FollowerTuning> = {},
    estimator = new ClockEstimator(),
  ) {
    this.actions = actions;
    this.settings = { lookaheadMs: LOOKAHEAD_MS.default, offsetMs: 0, ...settings };
    this.tuning = { ...FOLLOWER, ...tuning };
    this.estimator = estimator;
  }

  /** One real-time message from the input, with the page time it arrived. */
  feed(name: RealtimeName, timeMs: number): void {
    switch (name) {
      case "clock":
        this.onClock(timeMs);
        break;
      case "start":
        this.onStart(timeMs);
        break;
      case "continue":
        this.onContinue(timeMs);
        break;
      case "stop":
        this.onStop(timeMs);
        break;
    }
  }

  get running(): boolean {
    return this.believedRunning;
  }

  /** The tempo last sent to the engine, or null if none has been. */
  get appliedTempo(): number | null {
    return this.applied;
  }

  setLookahead(ms: number): void {
    if (finite(ms) && ms >= 0) this.settings.lookaheadMs = ms;
  }

  setOffset(ms: number): void {
    if (finite(ms)) this.settings.offsetMs = ms;
  }

  /** 0xFA. The engine starts at the next pulse, not now: the first pulse is the first step. */
  onStart(timeMs: number): void {
    if (!finite(timeMs)) return;
    this.startPending = true;
  }

  /** 0xFB. The same: the pulse after it resumes. */
  onContinue(timeMs: number): void {
    if (!finite(timeMs)) return;
    this.startPending = true;
  }

  /** 0xFC. At once. It is passed on even if the transport was already stopped: the engine's Stop clears hanging notes whatever state it was in. */
  onStop(timeMs: number): void {
    if (!finite(timeMs)) return;
    this.startPending = false;
    this.believedRunning = false;
    this.anchor = null;
    this.retakeAnchor = false;
    this.phase = null;
    this.trim = 0;
    this.actions.transport(false);
  }

  /** 0xF8. */
  onClock(timeMs: number): void {
    if (!finite(timeMs)) return;
    const outcome = this.estimator.push(timeMs);
    if (outcome === "ignored" || outcome === "backwards") return;
    if (this.estimator.epoch !== this.epoch) {
      this.epoch = this.estimator.epoch;
      this.lostCount();
    }
    if (this.startPending) {
      this.startPending = false;
      if (this.believedRunning) {
        this.lostCount();
      } else {
        this.believedRunning = true;
        this.anchor = { ext: this.estimator.pulseCount, eng: Math.round((this.sample?.ticks ?? 0) / TICKS_PER_CLOCK) };
        this.retakeAnchor = false;
        this.actions.transport(true);
      }
    }
    this.apply(timeMs);
  }

  /** The engine's position, from the panel message, mapped to page time. */
  onPosition(sample: PositionSample): void {
    if (!finite(sample.ticks) || !finite(sample.pageTimeMs)) return;
    this.sample = sample;
    if (!this.believedRunning) return;
    if (this.estimator.state(sample.pageTimeMs) === "lost") return;
    const period = this.estimator.periodMs;
    const lead = this.settings.lookaheadMs + this.settings.offsetMs;
    const ext = this.estimator.pulseIndex(sample.pageTimeMs + lead);
    if (period === null || ext === null) return;
    const eng = sample.ticks / TICKS_PER_CLOCK;
    if (this.retakeAnchor || this.anchor === null) {
      // The nearest whole pulse: only the fine phase is kept.
      this.anchor = { ext: 0, eng: Math.round(eng - ext) };
      this.retakeAnchor = false;
    }
    let error = eng - (this.anchor.eng + (ext - this.anchor.ext));
    if (Math.abs(error * period) > GIVE_UP_MS) {
      this.anchor = { ext: 0, eng: Math.round(eng - ext) };
      error = eng - (this.anchor.eng + ext);
    }
    this.phase = error * period;
    this.trim = Math.max(-this.tuning.maxTrim, Math.min(this.tuning.maxTrim, -this.phase / this.tuning.trimTimeMs));
    this.apply(sample.pageTimeMs);
  }

  status(nowMs: number): FollowStatus {
    const state = this.estimator.state(nowMs);
    const phase: FollowPhase = state === "idle" ? "waiting" : state === "locking" ? "settling" : state === "locked" ? "following" : "lost";
    return {
      phase,
      bpm: this.estimator.bpm,
      running: this.believedRunning,
      phaseMs: this.believedRunning && this.anchor !== null && phase !== "lost" ? this.phase : null,
      jitterMs: this.estimator.stats.jitterMs,
      pulses: this.estimator.stats.accepted,
    };
  }

  /** The count of pulses cannot be relied on any more: take the nearest whole pulse at the next position. */
  private lostCount(): void {
    if (this.believedRunning) this.retakeAnchor = true;
  }

  /** Sends the tempo the loop asks for if it is worth sending: the sender's tempo, trimmed while the engine is running. */
  private apply(nowMs: number): void {
    const bpm = this.estimator.bpm;
    if (bpm === null || this.estimator.state(nowMs) === "lost") return;
    const trimmed = this.believedRunning && this.anchor !== null ? bpm * (1 + this.trim) : bpm;
    if (!finite(trimmed) || trimmed < ENGINE_BPM.min || trimmed > ENGINE_BPM.max) return;
    if (this.applied !== null && Math.abs(trimmed - this.applied) < this.tuning.minTempoChange) return;
    this.applied = trimmed;
    this.actions.setTempo(trimmed);
  }
}

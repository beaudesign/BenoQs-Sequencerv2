// STUB: the follower's shape, with nothing behind it, so the fixtures in test/follower.test.ts can fail for the right reason.
// Replaced by the implementation in the commit that follows the red run (handoffs/evidence/p4d-follower-red.txt).
import { ClockEstimator } from "./clock-estimator.ts";
import type { RealtimeName } from "./midi-in.ts";

export interface FollowerActions {
  setTempo(bpm: number): void;
  transport(play: boolean): void;
}

export interface FollowerSettings {
  lookaheadMs: number;
  offsetMs: number;
}

export interface FollowerTuning {
  trimTimeMs: number;
  maxTrim: number;
  minTempoChange: number;
}

export interface PositionSample {
  ticks: number;
  pageTimeMs: number;
}

export type FollowPhase = "waiting" | "settling" | "following" | "lost";

export interface FollowStatus {
  phase: FollowPhase;
  bpm: number | null;
  running: boolean;
  phaseMs: number | null;
  jitterMs: number;
  pulses: number;
}

export class Follower {
  readonly estimator: ClockEstimator;
  constructor(_actions: FollowerActions, _settings: Partial<FollowerSettings> = {}, _tuning: Partial<FollowerTuning> = {}, estimator = new ClockEstimator()) {
    this.estimator = estimator;
  }
  feed(_name: RealtimeName, _timeMs: number): void {}
  onClock(_timeMs: number): void {}
  onStart(_timeMs: number): void {}
  onContinue(_timeMs: number): void {}
  onStop(_timeMs: number): void {}
  onPosition(_sample: PositionSample): void {}
  setLookahead(_ms: number): void {}
  setOffset(_ms: number): void {}
  get running(): boolean {
    return false;
  }
  get appliedTempo(): number | null {
    return null;
  }
  status(_nowMs: number): FollowStatus {
    return { phase: "waiting", bpm: null, running: false, phaseMs: null, jitterMs: 0, pulses: 0 };
  }
}

// STUB: the estimator's shape, with nothing behind it, so the fixtures in test/clock-estimator.test.ts can fail for the right reason.
// Replaced by the implementation in the commit that follows the red run (handoffs/evidence/p4d-estimator-red.txt).

export type LockState = "idle" | "locking" | "locked" | "lost";
export type PushOutcome = "first" | "seeding" | "accepted" | "duplicate" | "outlier" | "relocked" | "restarted" | "backwards" | "ignored";

export interface EstimatorConfig {
  jitterMs: number;
}

export interface EstimatorStats {
  accepted: number;
  duplicates: number;
  outliers: number;
  dropped: number;
  relocks: number;
  restarts: number;
  backwards: number;
  ignored: number;
  jitterMs: number;
}

export const LOST_AFTER_MS = 500;
export const LOCK_PULSES = 24;
export const SEED_PULSES = 4;

export class ClockEstimator {
  readonly stats: EstimatorStats = { accepted: 0, duplicates: 0, outliers: 0, dropped: 0, relocks: 0, restarts: 0, backwards: 0, ignored: 0, jitterMs: 0 };
  constructor(_config: Partial<EstimatorConfig> = {}) {}
  push(_timeMs: number): PushOutcome {
    return "ignored";
  }
  state(_nowMs: number): LockState {
    return "idle";
  }
  get bpm(): number | null {
    return null;
  }
  get periodMs(): number | null {
    return null;
  }
  get epoch(): number {
    return 0;
  }
  gridTime(_pulses: number): number | null {
    return null;
  }
  pulseIndex(_timeMs: number): number | null {
    return null;
  }
  reset(): void {}
}

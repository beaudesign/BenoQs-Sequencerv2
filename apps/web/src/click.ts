// The metronome's click. STUB: the shapes the tests are written against, doing nothing, so the tests fail on what they check and not on
// a missing file. Replaced by the real thing in the next commit.
import { TICKS_PER_CLOCK } from "./abi.ts";

export const QUARTER_TICKS = 24 * TICKS_PER_CLOCK;
export const CLICK_PEAK = 0.3;
export const CLICK_TAIL_SECONDS = 0.037;

export function beatsIn(_p0: number, _p1: number): number[] {
  return [];
}

export function beatFrame(_beat: number, _p0: number, _p1: number, _frames: number): number {
  return 0;
}

export class ClickVoice {
  active = false;
  constructor(_sampleRate: number) {}
  start(): void {}
  render(_out: Float32Array, _from: number, _to: number): void {}
}

export class Metronome {
  on = false;
  constructor(_sampleRate: number) {}
  render(out: Float32Array, _p0: number, _p1: number, _running: boolean): void {
    out.fill(0);
  }
}

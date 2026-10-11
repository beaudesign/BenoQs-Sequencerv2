// The chase-light (specs/SPEC-0002/p6-runs-when-opened.md, R-3): the manual's Red LED on the step each track is playing.
//
// The engine steps its tracks up to `MAX_EARLY_TICKS` ahead of the audio (crates/octocore/src/engine.rs), so the playhead it reports is
// ahead of what is heard by about a step. A light drawn as it comes would be on the step you are about to hear. So each playhead is kept
// with the tick at which it will be heard, the tick position of the audio when it was read plus the lead, and is shown when the audio
// gets there (specs/SPEC-0002/tech.md section 5). This works in ticks, not milliseconds, so it needs no tempo and follows a tempo change.
import { TRACK_COUNT } from "./abi.ts";

/** `MAX_EARLY_TICKS` in `crates/octocore/src/engine.rs`, which test/chase.test.ts reads to check this. */
export const LEAD_TICKS = 12;
/** The LED byte for Red, steady: the colour in the low two bits (1 is red) and the phase above (0 is steady) (ABI.md). */
export const LED_RED_STEADY = 1;

const KEPT = 256;

export class ChaseLight {
  private readonly pending: { due: number; heads: Uint8Array }[] = [];
  private last = 0;

  /**
   * One panel message: where the audio is on the engine's tick grid (`position`), the playheads the engine reported, and whether the
   * transport runs. Returns the playheads to draw for the audio at `position`, or `null` for no light: the transport is stopped, or the audio
   * has not yet reached the first playhead read since the Play.
   */
  update(position: number, heads: Uint8Array, running: boolean): Uint8Array | null {
    if (!running || position < this.last) this.pending.length = 0;
    this.last = position;
    if (!running) return null;
    this.pending.push({ due: position + LEAD_TICKS, heads: heads.slice(0, TRACK_COUNT) });
    if (this.pending.length > KEPT) this.pending.shift();
    // The newest playhead whose tick the audio has reached; the ones before it are never shown.
    while (this.pending.length >= 2 && this.pending[1]!.due <= position) this.pending.shift();
    const first = this.pending[0]!;
    return first.due <= position ? first.heads : null;
  }
}

/** The control number of each matrix key: `map[track][step]`, found from the control ids (`matrix.r<track>.c<step + 1>`). */
export function matrixMap(controls: readonly { n: number; id: string }[]): number[][] {
  const map: number[][] = Array.from({ length: TRACK_COUNT }, () => Array.from({ length: 16 }, () => -1));
  for (const c of controls) {
    const m = /^matrix\.r(\d+)\.c(\d+)$/.exec(c.id);
    if (!m) continue;
    const row = Number(m[1]);
    const step = Number(m[2]) - 1;
    if (row < TRACK_COUNT && step >= 0 && step < 16) map[row]![step] = c.n;
  }
  return map;
}

/** The engine's LED frame with the chase-light over it: a copy, with Red on the step each track is on. The frame it is given is not written to. */
export function composeChase(base: Uint8Array, heads: Uint8Array | null, map: readonly (readonly number[])[]): Uint8Array {
  if (!heads) return base;
  const out = base.slice();
  for (let track = 0; track < TRACK_COUNT; track++) {
    const n = map[track]?.[heads[track] ?? 0];
    if (n !== undefined && n >= 0 && n < out.length) out[n] = LED_RED_STEADY;
  }
  return out;
}

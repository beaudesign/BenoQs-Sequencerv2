// The numbers the strip offers. They are the page's own settings and not design tokens.

/** The tempo field, in beats per minute. The engine takes 1 to 999; the strip offers the range a person plays in. */
export const TEMPO_BPM = { default: 120, min: 20, max: 300 } as const;

export const LOOKAHEAD_MS = { default: 30, min: 0, max: 250 } as const;

/**
 * Extra lead for the notes when the app follows a clock, in milliseconds: positive makes them land earlier. It is the constant that Live's
 * "MIDI Clock Sync Delay" corrects, with the opposite sign (src/follower.ts, THE LEAD). Zero is where the lookahead alone puts them.
 */
export const CLOCK_OFFSET_MS = { default: 0, min: -100, max: 100 } as const;

/** How often the numbers under the clock sentence (tempo, pulses, jitter, phase) are redrawn. The sentence itself changes only with the state. */
export const CLOCK_DETAIL_REFRESH_MS = 250;

/** The two output ports the engine numbers (ABI.md: an event's `port` is 1 or 2). */
export const OUTPUT_PORTS = [1, 2] as const;
export type OutputPort = (typeof OUTPUT_PORTS)[number];

/** How often the count of what the chosen input has sent is redrawn on the strip. The messages themselves are counted as they arrive. */
export const INPUT_COUNT_REFRESH_MS = 500;

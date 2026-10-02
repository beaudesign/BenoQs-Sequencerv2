// The numbers the strip offers. They are the page's own settings and not design tokens.

export const LOOKAHEAD_MS = { default: 30, min: 0, max: 250 } as const;

/** The two output ports the engine numbers (ABI.md: an event's `port` is 1 or 2). */
export const OUTPUT_PORTS = [1, 2] as const;
export type OutputPort = (typeof OUTPUT_PORTS)[number];

/** How often the count of what the chosen input has sent is redrawn on the strip. The messages themselves are counted as they arrive. */
export const INPUT_COUNT_REFRESH_MS = 500;

// The `?route=` stand-in for port 2 (SPEC-0002 P4 plan D-P4-2, finding F-P4-2).
//
// A track reaches port 2 through its MIDI channel: 17 to 32 is port 2, channel 1 to 16. The hardware sets that channel in Track
// zoom, which does not exist yet, so until it does the address can say it: `?route=0:17,1:3` is track 0 on port 2 channel 1 and
// track 1 on port 1 channel 3. The track is the row of the matrix, 0 to 9, as the control ids give it (`matrix.r3.c1` is track 3).
//
// This is a test and demonstration hook and not a control: nothing draws it, the page reads it once at start, and it goes when Track
// zoom lands. This file only parses text; the one place that reads the address is `pages/app.ts` (a rule in
// `engine/tests/app_source_rules.rs` fails if another file under `src/` reads `location`).

export interface Route {
  /** The matrix row, 0 to 9. */
  track: number;
  /** The engine's MIDI channel for the track: 1 to 16 on port 1, 17 to 32 on port 2. */
  channel: number;
}

export interface RouteResult {
  routes: Route[];
  /** One sentence for each entry that was left out. */
  problems: string[];
}

const TRACKS = 10;
const CHANNELS = 32;
const PER_PORT = 16;

const whole = /^\d+$/;

function parseEntry(entry: string): Route | string {
  const parts = entry.split(":");
  if (parts.length !== 2 || !whole.test(parts[0]!.trim()) || !whole.test(parts[1]!.trim())) {
    return `The route entry "${entry}" is not track:channel, so it was left out.`;
  }
  const track = Number(parts[0]);
  const channel = Number(parts[1]);
  if (track >= TRACKS) return `The route entry "${entry}" names a track above ${TRACKS - 1}, so it was left out.`;
  if (channel < 1 || channel > CHANNELS) return `The route entry "${entry}" names a channel outside 1 to ${CHANNELS}, so it was left out.`;
  return { track, channel };
}

/** `search` is the address's query string, with or without the leading question mark. */
export function parseRoute(search: string): RouteResult {
  const params = new URLSearchParams(search);
  const all = params.getAll("route");
  const result: RouteResult = { routes: [], problems: [] };
  if (all.length === 0 || all[0]!.trim() === "") return result;
  if (all.length > 1) result.problems.push("The route is read once; the second route in the address was left out.");
  const entries = all[0]!.split(",").map((e) => e.trim());
  const parsed = entries.map(parseEntry);
  const seen = new Map<number, number>();
  for (const p of parsed) if (typeof p !== "string") seen.set(p.track, (seen.get(p.track) ?? 0) + 1);
  const twice = new Set([...seen].filter(([, n]) => n > 1).map(([t]) => t));
  for (const t of twice) result.problems.push(`Track ${t} is given more than once in the route, so none of its entries were used.`);
  for (const p of parsed) {
    if (typeof p === "string") result.problems.push(p);
    else if (!twice.has(p.track)) result.routes.push(p);
  }
  return result;
}

/** The sentence the strip shows for the routes that were applied. Empty for none. */
export function describeRoutes(routes: readonly Route[]): string {
  return routes
    .map((r) => `Track ${r.track} sends on port ${r.channel > PER_PORT ? 2 : 1}, channel ${((r.channel - 1) % PER_PORT) + 1}.`)
    .join(" ");
}

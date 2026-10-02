// The `?route=` stand-in for port 2 (SPEC-0002 P4 plan D-P4-2, finding F-P4-2): a test and demonstration hook, read once at start.
// The track is the row of the matrix, 0 to 9, as the control ids give it (`matrix.r3.c1` is track 3). The channel is the engine's
// MCH value: 1 to 16 is port 1, 17 to 32 is port 2.
import assert from "node:assert/strict";
import { test } from "node:test";
import { describeRoutes, parseRoute } from "../src/route.ts";

test("no route in the query string is no routes and no complaint", () => {
  for (const q of ["", "?", "?other=1", "?route=", "?route"]) assert.deepEqual(parseRoute(q), { routes: [], problems: [] }, JSON.stringify(q));
});

test("track:channel pairs, with or without the leading question mark", () => {
  assert.deepEqual(parseRoute("?route=3:17,4:18"), { routes: [{ track: 3, channel: 17 }, { track: 4, channel: 18 }], problems: [] });
  assert.deepEqual(parseRoute("route=0:1"), { routes: [{ track: 0, channel: 1 }], problems: [] });
  assert.deepEqual(parseRoute("?x=1&route=9:32&y=2"), { routes: [{ track: 9, channel: 32 }], problems: [] });
});

test("spaces around a pair are allowed; the browser sends a comma as %2C", () => {
  assert.deepEqual(parseRoute("?route=3%3A17%2C%204%3A18").routes, [{ track: 3, channel: 17 }, { track: 4, channel: 18 }]);
});

test("a pair that cannot be used is left out and named, and the others still apply", () => {
  const r = parseRoute("?route=3:17,x,4:99,10:1,5:0,6:2.5,,7:3");
  assert.deepEqual(r.routes, [{ track: 3, channel: 17 }, { track: 7, channel: 3 }]);
  assert.equal(r.problems.length, 6);
  for (const bad of ["x", "4:99", "10:1", "5:0", "6:2.5", ""]) assert.ok(r.problems.some((p) => p.includes(`"${bad}"`)), `a problem names ${JSON.stringify(bad)}: ${r.problems.join(" | ")}`);
});

test("a track given twice is refused for both pairs, because there is no right answer", () => {
  const r = parseRoute("?route=3:17,3:18,4:2");
  assert.deepEqual(r.routes, [{ track: 4, channel: 2 }]);
  assert.equal(r.problems.length, 1);
  assert.match(r.problems[0]!, /Track 3/);
});

test("a number with a sign, an exponent or letters is not a number here", () => {
  for (const bad of ["+3:17", "3:+17", "3:1e1", "0x3:17", "3:17x", "-1:17"]) {
    const r = parseRoute(`?route=${encodeURIComponent(bad)}`);
    assert.deepEqual(r.routes, [], bad);
    assert.equal(r.problems.length, 1, bad);
  }
});

test("only the first route parameter is read; a second one is a problem and is not applied", () => {
  const r = parseRoute("?route=3:17&route=4:18");
  assert.deepEqual(r.routes, [{ track: 3, channel: 17 }]);
  assert.equal(r.problems.length, 1);
  assert.match(r.problems[0]!, /once/);
});

test("the port changes at channel 17: 16 is the last of port 1 and 17 the first of port 2", () => {
  assert.equal(describeRoutes([{ track: 0, channel: 16 }]), "Track 0 sends on port 1, channel 16.");
  assert.equal(describeRoutes([{ track: 0, channel: 1 }]), "Track 0 sends on port 1, channel 1.");
  assert.equal(describeRoutes([{ track: 0, channel: 17 }]), "Track 0 sends on port 2, channel 1.");
});

test("the sentence the strip shows says each track's port and channel in the manual's terms", () => {
  assert.equal(describeRoutes([]), "");
  assert.equal(describeRoutes([{ track: 3, channel: 17 }]), "Track 3 sends on port 2, channel 1.");
  assert.equal(
    describeRoutes([{ track: 3, channel: 17 }, { track: 4, channel: 5 }, { track: 9, channel: 32 }]),
    "Track 3 sends on port 2, channel 1. Track 4 sends on port 1, channel 5. Track 9 sends on port 2, channel 16.",
  );
});

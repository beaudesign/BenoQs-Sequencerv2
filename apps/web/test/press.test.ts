// Pointer and keyboard presses become one press and one release per control, however many fingers and keys are down.
import assert from "node:assert/strict";
import { test } from "node:test";
import { Presses } from "../src/press.ts";

function rig() {
  const calls: string[] = [];
  const presses = new Presses({ press: (n) => calls.push(`down ${n}`), release: (n) => calls.push(`up ${n}`) });
  return { calls, presses };
}

test("a pointer down and up is one press and one release", () => {
  const { calls, presses } = rig();
  presses.down(5, "pointer 1");
  presses.up(5, "pointer 1");
  assert.deepEqual(calls, ["down 5", "up 5"]);
});

test("key repeat does not press again", () => {
  const { calls, presses } = rig();
  for (let i = 0; i < 5; i++) presses.down(5, "key");
  presses.up(5, "key");
  assert.deepEqual(calls, ["down 5", "up 5"]);
});

test("two sources on one control hold it until the last lets go", () => {
  const { calls, presses } = rig();
  presses.down(5, "pointer 1");
  presses.down(5, "key");
  presses.up(5, "pointer 1");
  assert.deepEqual(calls, ["down 5"]);
  presses.up(5, "key");
  assert.deepEqual(calls, ["down 5", "up 5"]);
});

test("a release nobody pressed is ignored", () => {
  const { calls, presses } = rig();
  presses.up(5, "key");
  assert.deepEqual(calls, []);
});

test("two controls at once (a chord) are two presses, released independently", () => {
  const { calls, presses } = rig();
  presses.down(1, "pointer 1");
  presses.down(2, "pointer 2");
  presses.up(1, "pointer 1");
  presses.up(2, "pointer 2");
  assert.deepEqual(calls, ["down 1", "down 2", "up 1", "up 2"]);
});

test("releaseAll lets go of everything held, once, as when the window loses focus", () => {
  const { calls, presses } = rig();
  presses.down(1, "pointer 1");
  presses.down(2, "key");
  presses.releaseAll();
  presses.releaseAll();
  assert.deepEqual(calls.sort(), ["down 1", "down 2", "up 1", "up 2"]);
  assert.equal(presses.held(), 0);
});

test("held says whether a control is down", () => {
  const { presses } = rig();
  presses.down(7, "key");
  assert.equal(presses.isHeld(7), true);
  assert.equal(presses.isHeld(8), false);
  presses.up(7, "key");
  assert.equal(presses.isHeld(7), false);
});

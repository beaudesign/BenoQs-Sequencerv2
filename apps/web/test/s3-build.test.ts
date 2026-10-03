// Which engine made a saved S3 file (the Ableton MCP review's Q-M6, owner's default "yes"): the module's ABI number, its size and the SHA-256 of
// its bytes. A file the owner saves from the page after a run with Live is read later, perhaps after the engine has changed; without this the file
// does not say which build it was a run of, and the runner's `commit` is not in a file the page saved. The hash is what ties the file to the build:
// the same bytes give the same hash, and a different engine gives a different one.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { test } from "node:test";
import { engineBuild } from "../spikes/s3/build.ts";

const bytes = (n: number, seed = 31): ArrayBuffer => Uint8Array.from({ length: n }, (_, i) => (i * seed) % 251).buffer;
const sha = (b: ArrayBuffer): string => createHash("sha256").update(new Uint8Array(b)).digest("hex");

test("the build record is the module's ABI number, its size in bytes and the SHA-256 of its bytes, in lower-case hex", async () => {
  const wasm = bytes(1_000);
  const b = await engineBuild(wasm, 2);
  assert.deepEqual(b, { abi: 2, wasmBytes: 1_000, sha256: sha(wasm) });
  assert.match(b.sha256 ?? "", /^[0-9a-f]{64}$/);
});

test("one changed byte changes the hash, and the size and the ABI number are what they are given", async () => {
  const a = bytes(1_000);
  const changed = a.slice(0);
  const view = new Uint8Array(changed);
  view[500] = view[500]! ^ 1;
  assert.notEqual((await engineBuild(a, 2)).sha256, (await engineBuild(changed, 2)).sha256);
  assert.equal((await engineBuild(a, 2)).sha256, (await engineBuild(a.slice(0), 2)).sha256, "the same bytes, the same hash");
  const longer = await engineBuild(bytes(1_001), 3);
  assert.equal(longer.wasmBytes, 1_001);
  assert.equal(longer.abi, 3);
});

test("a page with no SubtleCrypto (one not served from a secure context) still records the size and the ABI, and says it has no hash", async () => {
  const b = await engineBuild(bytes(10), 2, null);
  assert.deepEqual(b, { abi: 2, wasmBytes: 10, sha256: null });
});

// Which engine a saved S3 file is a run of: the module's ABI number, its size and the SHA-256 of its bytes (the Ableton MCP review's Q-M6).
//
// The runner (`run.ts`) writes the git commit beside a simulated run, but a file the owner saves from the page after a run with Live has no
// commit, and a file read a week later may be read against an engine that has since changed. The hash is what ties the file to the module the
// page was served: the same bytes give the same hash, and any rebuild that changes the engine changes it. It is computed from the bytes the page
// fetched, before they are handed to the host, so it is the module that ran and not the one on disk now.

export interface EngineBuild {
  /** The ABI number the module reported and the page checked (`octoweb_abi()`, `ABI_VERSION`). */
  abi: number;
  wasmBytes: number;
  /** SHA-256 of the module's bytes, lower-case hex. `null` where the page has no SubtleCrypto, which a page not served from a secure context has not. */
  sha256: string | null;
}

const hex = (bytes: ArrayBuffer): string => [...new Uint8Array(bytes)].map((b) => b.toString(16).padStart(2, "0")).join("");

export async function engineBuild(wasm: ArrayBuffer, abi: number, subtle: Pick<SubtleCrypto, "digest"> | null = globalThis.crypto?.subtle ?? null): Promise<EngineBuild> {
  return { abi, wasmBytes: wasm.byteLength, sha256: subtle === null ? null : hex(await subtle.digest("SHA-256", wasm)) };
}

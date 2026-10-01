// Runs inside an AudioWorkletGlobalScope. Reports which globals exist there, and whether a
// WebAssembly.Module passed in `processorOptions` can be instantiated synchronously.
class Probe extends AudioWorkletProcessor {
  constructor(options) {
    super();
    const names = ["performance", "TextDecoder", "TextEncoder", "Date", "WebAssembly", "SharedArrayBuffer", "Atomics", "fetch", "setTimeout", "queueMicrotask", "crypto", "console"];
    let wasm;
    try {
      const instance = new WebAssembly.Instance(options.processorOptions.module, {});
      wasm = { ok: true, add_2_40: instance.exports.add(2, 40), isModule: options.processorOptions.module instanceof WebAssembly.Module };
    } catch (e) {
      wasm = { ok: false, error: String(e) };
    }
    this.port.postMessage({ scope: Object.fromEntries(names.map((n) => [n, typeof globalThis[n]])), sampleRate, wasm });
  }
  process() { return true; }
}
registerProcessor("probe", Probe);

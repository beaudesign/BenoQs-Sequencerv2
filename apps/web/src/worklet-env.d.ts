// What an AudioWorkletGlobalScope offers that the DOM typings do not declare. Only the parts
// `worklet.ts` uses. The scope has no `performance`, `TextDecoder`, `fetch` or `crypto`
// (handoffs/evidence/p3-browser-probe/), and the code that runs there must not use them.
declare const sampleRate: number;
declare const currentFrame: number;
declare const currentTime: number;
declare class AudioWorkletProcessor {
  readonly port: MessagePort;
  constructor(options?: unknown);
  process(inputs: Float32Array[][], outputs: Float32Array[][], parameters: Record<string, Float32Array>): boolean;
}
declare function registerProcessor(name: string, processor: new (options: { processorOptions: unknown }) => AudioWorkletProcessor): void;

// The stand-ins of the simulated S3 runs, in a module of their own so that a test can use the same ones `run.ts` does (`run.ts` runs when it is
// imported). Nothing here is Live or a MIDI port; what each stands for is in the two sentences below, which go on the first line of a record.
// Neither is on a page a person runs: they are installed by the runner before the page's own scripts.

export const SIMULATED_FOLLOW = "the sender is a timer in the page that stamps each pulse with performance.now() as it fires. It is not Ableton Live, not a MIDI port, and not Web MIDI's timestamps; the stamps are on the page's clock because the simulation makes them so.";
export const SIMULATED_SEND = "the port is a setTimeout in the page. It is not a MIDI port and not a loopback; its latency and its jitter are this machine's timers.";

/** Runs in the page before its own scripts: a MIDI access with one input and one output, and a clock sender that talks to the input. */
export function standInScript(): void {
  type Handler = ((e: { data: Uint8Array; timeStamp: number }) => void) | null;
  const input = { id: "in1", name: "Stand-in input (a timer in the page, not a MIDI port)", type: "input", onmidimessage: null as Handler };
  const emit = (bytes: number[]): void => input.onmidimessage?.({ data: Uint8Array.from(bytes), timeStamp: performance.now() });
  const output = {
    id: "out1",
    name: "Stand-in output (a timer in the page, not a MIDI port)",
    type: "output",
    send(data: ArrayLike<number>, timestamp?: number): void {
      const now = performance.now();
      const delay = timestamp !== undefined && timestamp > now ? timestamp - now : 0;
      const bytes = Array.from(data);
      setTimeout(() => emit(bytes), delay);
    },
  };
  (navigator as unknown as { requestMIDIAccess: () => Promise<unknown> }).requestMIDIAccess = async () => ({
    inputs: new Map([["in1", input]]),
    outputs: new Map([["out1", output]]),
  });
  // A sender that does not drift: each pulse is timed from the start, not from the one before, as a hardware clock is.
  let timer: ReturnType<typeof setTimeout> | undefined;
  (window as unknown as { sender: unknown }).sender = {
    start(bpm: number): void {
      clearTimeout(timer);
      const period = 2500 / bpm;
      const origin = performance.now();
      let count = 0;
      emit([0xfa]);
      const pulse = (): void => {
        emit([0xf8]);
        count++;
        timer = setTimeout(pulse, Math.max(0, origin + count * period - performance.now()));
      };
      timer = setTimeout(pulse, 2);
    },
    stop(): void {
      clearTimeout(timer);
      emit([0xfc]);
    },
  };
}


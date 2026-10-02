// The stand-ins of the simulated S3 runs, in a module of their own so that a test can use the same ones `run.ts` does (`run.ts` runs when it is
// imported). Nothing here is Live or a MIDI port; what each stands for is in the two sentences below, which go on the first line of a record.
// They are installed by the runner before the page's own scripts, and are not on the page a person runs.
//
// WHEN A MESSAGE IS STAMPED. A real port stamps a message when the system delivers it, and the page's handler runs after that, in a burst if
// the page's main thread was busy. The first stand-in stamped with `performance.now()` when its own timer fired, so a stall of the page (once
// in 14 runs, about 200 ms) looked like a stall of the sender and made the follower lose its lock, which a real port would not do
// (handoffs/evidence/p4e-follow-stall.txt). The stamp is now the time the message was due, and the page's timer only delivers it.

/** How far behind the time the message was due the stand-ins stamp it, in ms: made up, so that a stamp and the time it was seen cannot be mistaken for each other. */
export const STAMP_AGE_MS = 2;
/** The scatter of a stamp, plus and minus this many milliseconds: made up, deterministic. */
export const SCATTER_MS = 0.3;

export const SIMULATED_FOLLOW = "the sender is a timer chain in the page that delivers each pulse and gives it a stamp of the time it was due, 2 ms behind, with a made-up scatter of 0.3 ms. It is not Ableton Live, not a MIDI port, and not Web MIDI's timestamps; the stamps are on the page's clock because the simulation makes them so.";
export const SIMULATED_SEND = "the port delivers each message from a setTimeout in the page, stamped with the time it was sent for, 2 ms behind, with a made-up scatter of 0.3 ms. It is not a MIDI port and not a loopback: it has no latency of its own and its jitter is made up.";

/** Runs in the page before its own scripts: a MIDI access with one input and one output, and a clock sender that talks to the input. */
export function standInScript(options: { stampAgeMs: number; scatterMs: number }): void {
  type Handler = ((e: { data: Uint8Array; timeStamp: number }) => void) | null;
  const input = { id: "in1", name: "Stand-in input (a timer in the page, not a MIDI port)", type: "input", onmidimessage: null as Handler };
  let counter = 0;
  // A fixed pseudo-random scatter in [-scatterMs, scatterMs]: the same in every run.
  const scatter = (): number => (((++counter * 2654435761) % 1000) / 1000 - 0.5) * 2 * options.scatterMs;
  /** Hands the page a message that was due at `due` (page time, ms). */
  const emit = (bytes: number[], due: number): void => input.onmidimessage?.({ data: Uint8Array.from(bytes), timeStamp: due - options.stampAgeMs + scatter() });
  const output = {
    id: "out1",
    name: "Stand-in output (a timer in the page, not a MIDI port)",
    type: "output",
    send(data: ArrayLike<number>, timestamp?: number): void {
      const now = performance.now();
      const due = timestamp !== undefined && timestamp > now ? timestamp : now;
      const bytes = Array.from(data);
      setTimeout(() => emit(bytes, due), due - now);
    },
  };
  (navigator as unknown as { requestMIDIAccess: () => Promise<unknown> }).requestMIDIAccess = async () => ({
    inputs: new Map([["in1", input]]),
    outputs: new Map([["out1", output]]),
  });
  // A sender that does not drift: each pulse is due at a multiple of the period from the start, as a hardware clock's is.
  let timer: ReturnType<typeof setTimeout> | undefined;
  (window as unknown as { sender: unknown }).sender = {
    start(bpm: number): void {
      clearTimeout(timer);
      const period = 2500 / bpm;
      const origin = performance.now();
      let count = 0;
      emit([0xfa], origin);
      const pulse = (): void => {
        emit([0xf8], origin + 2 + count * period);
        count++;
        timer = setTimeout(pulse, Math.max(0, origin + 2 + count * period - performance.now()));
      };
      timer = setTimeout(pulse, 2);
    },
    stop(): void {
      clearTimeout(timer);
      emit([0xfc], performance.now());
    },
  };
}

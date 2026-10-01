// Web MIDI as the page needs it: whether there is any, whether it was allowed, and the outputs it lists. A page
// without it still loads and runs, and says so (acceptance criterion A7).
import type { MidiOutputLike } from "./midi-out.ts";

export interface MidiPortLike extends MidiOutputLike {
  readonly id: string;
  readonly name?: string | null;
}

export interface MidiAccessLike {
  readonly outputs: { values(): Iterable<MidiPortLike> };
  onstatechange: ((event: never) => unknown) | null;
}

export interface MidiNavigatorLike {
  requestMIDIAccess?: (options?: { sysex?: boolean }) => Promise<MidiAccessLike>;
}

export interface OutputChoice {
  id: string;
  name: string;
  output: MidiOutputLike;
}

export interface Midi {
  status: "unavailable" | "denied" | "ready";
  /** One sentence for the strip. */
  message: string;
  outputs(): OutputChoice[];
  /** Called when a port is plugged in or removed. */
  onChange: (() => void) | null;
}

const NO_MIDI = "This browser has no Web MIDI, so the sequencer runs without MIDI out. Chrome and Edge have it.";
const REFUSED = "MIDI access was refused. Allow it for this site, then press Start again.";
const NO_OUTPUT = "No MIDI output found. Connect one, or turn on a loopback port, and it will appear here.";
const CHOOSE = "Choose a MIDI output to hear the sequencer.";

function closed(status: "unavailable" | "denied", message: string): Midi {
  return { status, message, outputs: () => [], onChange: null };
}

/** Asks the browser for MIDI. Never throws: every failure is a `Midi` that says what happened. */
export async function openMidi(nav: MidiNavigatorLike): Promise<Midi> {
  if (typeof nav.requestMIDIAccess !== "function") return closed("unavailable", NO_MIDI);
  let access: MidiAccessLike;
  try {
    access = await nav.requestMIDIAccess({ sysex: false });
  } catch {
    return closed("denied", REFUSED);
  }
  const midi: Midi = {
    status: "ready",
    message: "",
    outputs: () => [...access.outputs.values()].map((p) => ({ id: p.id, name: p.name?.trim() || p.id, output: p })),
    onChange: null,
  };
  const describe = (): string => (midi.outputs().length === 0 ? NO_OUTPUT : CHOOSE);
  midi.message = describe();
  access.onstatechange = () => {
    midi.message = describe();
    midi.onChange?.();
  };
  return midi;
}

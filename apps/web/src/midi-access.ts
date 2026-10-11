// Web MIDI as the page needs it: whether there is any, whether it was allowed, and the outputs and inputs it lists. A page
// without it still loads and runs, and says so (acceptance criterion A7).
import type { MidiInputLike } from "./midi-in.ts";
import type { MidiOutputLike } from "./midi-out.ts";

export interface MidiPortLike extends MidiOutputLike {
  readonly id: string;
  readonly name?: string | null;
}

export interface MidiAccessLike {
  readonly outputs: { values(): Iterable<MidiPortLike> };
  readonly inputs: { values(): Iterable<MidiInputLike> };
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

export interface InputChoice {
  id: string;
  name: string;
  input: MidiInputLike;
}

export interface Midi {
  /** `unavailable`: the browser has no Web MIDI. `denied`: it was refused. `failed`: the browser has it and could not start it here. */
  status: "unavailable" | "denied" | "failed" | "ready";
  /** One sentence for the strip: what the browser said about MIDI, and, when it is ready, about the outputs. */
  message: string;
  /** One sentence about the inputs. Empty unless the status is `ready`, when `message` covers both directions. */
  inputMessage: string;
  outputs(): OutputChoice[];
  inputs(): InputChoice[];
  /** Called when a port is plugged in or removed. */
  onChange: (() => void) | null;
}

const NO_MIDI = "This browser has no Web MIDI, so the sequencer runs without MIDI out. Chrome and Edge have it.";
const REFUSED = "MIDI access was refused. Allow it for this site, then press Play again.";
const NO_OUTPUT = "No MIDI output found. Connect one, or turn on a loopback port, and it will appear here.";
const CHOOSE = "Choose a MIDI output to hear the sequencer.";
const NO_INPUT = "No MIDI input found. Connect one, or turn on a loopback port, and it will appear here.";
const CHOOSE_INPUT = "Choose a MIDI input to listen to.";

function closed(status: "unavailable" | "denied" | "failed", message: string): Midi {
  return { status, message, inputMessage: "", outputs: () => [], inputs: () => [], onChange: null };
}

/** What Chromium 141 gives when it has Web MIDI and the system has no MIDI backend (measured for P4b): `InvalidStateError`. */
function failure(e: unknown): Midi | null {
  const name = typeof e === "object" && e !== null && "name" in e ? String((e as { name: unknown }).name) : "";
  if (name !== "InvalidStateError") return null;
  const why = String((e as { message?: unknown }).message ?? "").trim().replace(/\.$/, "");
  return closed("failed", `Web MIDI could not start on this system${why ? `. The browser said: ${why}` : ""}.`);
}

/** Asks the browser for MIDI. Never throws: every failure is a `Midi` that says what happened. */
export async function openMidi(nav: MidiNavigatorLike): Promise<Midi> {
  if (typeof nav.requestMIDIAccess !== "function") return closed("unavailable", NO_MIDI);
  let access: MidiAccessLike;
  try {
    access = await nav.requestMIDIAccess({ sysex: false });
  } catch (e) {
    return failure(e) ?? closed("denied", REFUSED);
  }
  const midi: Midi = {
    status: "ready",
    message: "",
    inputMessage: "",
    outputs: () => [...access.outputs.values()].map((p) => ({ id: p.id, name: p.name?.trim() || p.id, output: p })),
    inputs: () => [...access.inputs.values()].map((p) => ({ id: p.id, name: p.name?.trim() || p.id, input: p })),
    onChange: null,
  };
  const describe = (): void => {
    midi.message = midi.outputs().length === 0 ? NO_OUTPUT : CHOOSE;
    midi.inputMessage = midi.inputs().length === 0 ? NO_INPUT : CHOOSE_INPUT;
  };
  describe();
  access.onstatechange = () => {
    describe();
    midi.onChange?.();
  };
  return midi;
}

/** The sentence for the output line: where each port sends, or `fallback` (the browser's own sentence) when neither has a device. */
export function describeOutputs(chosen: { 1: Pick<OutputChoice, "id" | "name"> | null; 2: Pick<OutputChoice, "id" | "name"> | null }, fallback: string): string {
  const { 1: one, 2: two } = chosen;
  if (!one && !two) return fallback;
  if (one && two && one.id === two.id) return `Port 1 and port 2 send to ${one.name}.`;
  return [one && `Port 1 sends to ${one.name}.`, two && `Port 2 sends to ${two.name}.`].filter(Boolean).join(" ");
}

// The strip: the plain furniture a browser needs and the hardware does not (docs/05 section 1). It is written in
// pages/app.html; this binds it. It never sits on the panel and holds no colour of its own.
import type { InputChoice, OutputChoice } from "./midi-access.ts";
import type { OutputPort } from "./settings.ts";

export interface Strip {
  start: HTMLButtonElement;
  /** One list of devices for each of the engine's two output ports. */
  outputs: Record<OutputPort, HTMLSelectElement>;
  input: HTMLSelectElement;
  /** The clock state: off, master, slave, slave with echo (src/clock-state.ts). */
  clock: HTMLSelectElement;
  /** The extra lead, in milliseconds, for the notes when following a clock. */
  clockOffset: HTMLInputElement;
  lookahead: HTMLInputElement;
  /** The app's own state, in a sentence. */
  say(engine: string): void;
  /** What the browser's MIDI said, and where each port is sending, in a sentence. */
  sayMidi(message: string): void;
  /** What the MIDI input is doing, in a sentence. Empty when the line above covers it. */
  sayInput(message: string): void;
  /** How many messages the chosen input has sent. Not a live region: it changes twice a second. */
  sayInputCount(text: string): void;
  /** What the clock state is doing, in a sentence. A live region: it changes only when the state does. */
  sayClock(text: string): void;
  /** The numbers under it (tempo, pulses, jitter, phase). Not a live region: it changes four times a second. Empty hides the line. */
  sayClockDetail(text: string): void;
  /** The routes the address asked for, in a sentence. Empty hides the line. */
  sayRoute(text: string): void;
  /** Lists the outputs the browser offers in both lists and keeps each selection if it is still there. */
  setOutputs(choices: Pick<OutputChoice, "id" | "name">[]): void;
  /** Lists the inputs the browser offers and keeps the selection if it is still there. */
  setInputs(choices: Pick<InputChoice, "id" | "name">[]): void;
}

function find<T extends Element>(doc: Document, selector: string): T {
  const el = doc.querySelector<T>(selector);
  if (!el) throw new Error(`the page has no ${selector}`);
  return el;
}

function fill(doc: Document, select: HTMLSelectElement, none: string, choices: Pick<OutputChoice, "id" | "name">[]): void {
  const kept = select.value;
  select.replaceChildren();
  const empty = doc.createElement("option");
  empty.value = "";
  empty.textContent = none;
  select.append(empty);
  for (const c of choices) {
    const option = doc.createElement("option");
    option.value = c.id;
    option.textContent = c.name;
    select.append(option);
  }
  select.value = choices.some((c) => c.id === kept) ? kept : "";
}

export function bindStrip(doc: Document): Strip {
  const start = find<HTMLButtonElement>(doc, "#start");
  const outputs = { 1: find<HTMLSelectElement>(doc, "#midi-out-1"), 2: find<HTMLSelectElement>(doc, "#midi-out-2") };
  const input = find<HTMLSelectElement>(doc, "#midi-in");
  const clock = find<HTMLSelectElement>(doc, "#clock-state");
  const clockOffset = find<HTMLInputElement>(doc, "#clock-offset");
  const lookahead = find<HTMLInputElement>(doc, "#lookahead");
  const engine = find<HTMLElement>(doc, "#engine-status");
  const midi = find<HTMLElement>(doc, "#midi-status");
  const inStatus = find<HTMLElement>(doc, "#midi-in-status");
  const inCount = find<HTMLElement>(doc, "#midi-in-count");
  const route = find<HTMLElement>(doc, "#route-status");
  const clockStatus = find<HTMLElement>(doc, "#clock-status");
  const clockDetail = find<HTMLElement>(doc, "#clock-detail");
  return {
    start,
    outputs,
    input,
    clock,
    clockOffset,
    lookahead,
    say: (text) => void (engine.textContent = text),
    sayMidi: (text) => void (midi.textContent = text),
    sayInput: (text) => void (inStatus.textContent = text),
    sayInputCount: (text) => void (inCount.textContent = text),
    sayClock: (text) => void (clockStatus.textContent = text),
    sayClockDetail: (text) => void (clockDetail.textContent = text),
    sayRoute(text) {
      route.textContent = text;
      route.hidden = text === "";
    },
    setOutputs(choices) {
      fill(doc, outputs[1], "No output", choices);
      fill(doc, outputs[2], "No output", choices);
    },
    setInputs(choices) {
      fill(doc, input, "No input", choices);
    },
  };
}

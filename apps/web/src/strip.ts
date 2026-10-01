// The strip: the plain furniture a browser needs and the hardware does not (docs/05 section 1). It is written in
// pages/app.html; this binds it. It never sits on the panel and holds no colour of its own.
import type { OutputChoice } from "./midi-access.ts";

export interface Strip {
  start: HTMLButtonElement;
  output: HTMLSelectElement;
  lookahead: HTMLInputElement;
  /** The app's own state, in a sentence. */
  say(engine: string): void;
  /** What the browser's MIDI said, in a sentence. */
  sayMidi(message: string): void;
  /** Lists the outputs the browser offers and keeps the selection if it is still there. Returns the selected id, or "". */
  setOutputs(choices: Pick<OutputChoice, "id" | "name">[]): string;
}

function find<T extends Element>(doc: Document, selector: string): T {
  const el = doc.querySelector<T>(selector);
  if (!el) throw new Error(`the page has no ${selector}`);
  return el;
}

export function bindStrip(doc: Document): Strip {
  const start = find<HTMLButtonElement>(doc, "#start");
  const output = find<HTMLSelectElement>(doc, "#midi-output");
  const lookahead = find<HTMLInputElement>(doc, "#lookahead");
  const engine = find<HTMLElement>(doc, "#engine-status");
  const midi = find<HTMLElement>(doc, "#midi-status");
  return {
    start,
    output,
    lookahead,
    say: (text) => void (engine.textContent = text),
    sayMidi: (text) => void (midi.textContent = text),
    setOutputs(choices) {
      const kept = output.value;
      output.replaceChildren();
      const none = doc.createElement("option");
      none.value = "";
      none.textContent = "No output";
      output.append(none);
      for (const c of choices) {
        const option = doc.createElement("option");
        option.value = c.id;
        option.textContent = c.name;
        output.append(option);
      }
      output.value = choices.some((c) => c.id === kept) ? kept : "";
      return output.value;
    },
  };
}

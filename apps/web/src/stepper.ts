// The stepper of a number field (ADR-0011 decisions 6 and 7): the small block at the right of a number box with an up and a down arrow in it.
// The browser draws its own and neither gives it a corner nor lets the page choose its tone, so the page draws this one and hides the browser's.
// It is the same thing to the page: a press steps the value by the field's step, within its limits, and tells the page's listeners with an
// `input` and a `change`, as the browser's did. It repeats while it is held, as the browser's did. It is not a control of its own: it repeats the
// arrow keys, which still work, so it is hidden from the accessibility tree and is not a stop for the Tab key.

/** Milliseconds. Held down, the first repeat comes after `FIRST_REPEAT_MS` and the rest at `REPEAT_MS`. */
const FIRST_REPEAT_MS = 400;
const REPEAT_MS = 70;

function span(doc: Document, className: string): HTMLSpanElement {
  const el = doc.createElement("span");
  el.className = className;
  return el;
}

/** One step, up or down. It says nothing when the value did not move (a limit), so a listener is never told of a change that did not happen. */
function step(input: HTMLInputElement, direction: 1 | -1): void {
  const before = input.value;
  try {
    if (direction === 1) input.stepUp();
    else input.stepDown();
  } catch {
    return; // a field with no valid step, or a value that cannot be stepped: the browser's own would not have moved either
  }
  if (input.value === before) return;
  input.dispatchEvent(new Event("input", { bubbles: true }));
  input.dispatchEvent(new Event("change", { bubbles: true }));
}

/** Wraps one number field and gives it its block. Calling it twice on the same field does nothing the second time. */
export function addStepper(doc: Document, input: HTMLInputElement): void {
  if (input.parentElement?.classList.contains("stepper")) return;
  const wrap = span(doc, "stepper");
  const steps = span(doc, "steps");
  steps.setAttribute("aria-hidden", "true");
  const up = span(doc, "up");
  const down = span(doc, "down");
  steps.append(up, down);
  input.replaceWith(wrap);
  wrap.append(input, steps);

  let waiting: ReturnType<typeof setTimeout> | undefined;
  let repeating: ReturnType<typeof setInterval> | undefined;
  const release = (): void => {
    clearTimeout(waiting);
    clearInterval(repeating);
    waiting = undefined;
    repeating = undefined;
  };
  const hold = (zone: HTMLElement, direction: 1 | -1): void => {
    zone.addEventListener("pointerdown", (e) => {
      if (e.button !== 0) return;
      e.preventDefault(); // no text selection, and the field takes the focus below, as it did when the browser's own was pressed
      release();
      input.focus();
      step(input, direction);
      waiting = setTimeout(() => void (repeating = setInterval(() => step(input, direction), REPEAT_MS)), FIRST_REPEAT_MS);
    });
    for (const end of ["pointerup", "pointercancel", "pointerleave"]) zone.addEventListener(end, release);
  };
  hold(up, 1);
  hold(down, -1);
}

/** Every number field of the strip. */
export function addSteppers(doc: Document): void {
  for (const input of doc.querySelectorAll<HTMLInputElement>('#strip input[type="number"]')) addStepper(doc, input);
}

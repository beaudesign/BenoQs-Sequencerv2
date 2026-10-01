// Turns pointer and keyboard presses into one press and one release per control. A control is held while any
// source (a pointer, the keyboard) holds it, so key repeat and a second finger do not press it twice.

export interface PressHooks {
  press(control: number): void;
  release(control: number): void;
}

export class Presses {
  private readonly sources = new Map<number, Set<string>>();
  private readonly hooks: PressHooks;

  constructor(hooks: PressHooks) {
    this.hooks = hooks;
  }

  down(control: number, source: string): void {
    let held = this.sources.get(control);
    if (!held) {
      held = new Set();
      this.sources.set(control, held);
    }
    const first = held.size === 0;
    held.add(source);
    if (first) this.hooks.press(control);
  }

  up(control: number, source: string): void {
    const held = this.sources.get(control);
    if (!held || !held.delete(source)) return;
    if (held.size === 0) {
      this.sources.delete(control);
      this.hooks.release(control);
    }
  }

  isHeld(control: number): boolean {
    return this.sources.has(control);
  }

  held(): number {
    return this.sources.size;
  }

  /** The window lost focus or the page is closing: let go of everything, as a hand lifting off the panel would. */
  releaseAll(): void {
    const all = [...this.sources.keys()];
    this.sources.clear();
    for (const control of all) this.hooks.release(control);
  }
}

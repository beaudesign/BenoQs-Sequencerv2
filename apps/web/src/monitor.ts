// The built-in sound (specs/SPEC-0002/p6-runs-when-opened.md, R-2). The Octopus makes no sound of its own, but a page opened with nothing
// plugged in must be heard to run, and Web MIDI cannot be reached from every frame the page is shown in. So the worklet plays the engine's
// own notes in its output: a pitched voice for a note and a one-shot for each of three drums on MIDI channel 10 (kick, clap and hat).
//
// It is audio and not MIDI: it reads the events the engine made for this block, changes none of them, and keeps nothing the engine can see.
// It is deliberately plain (sine and two harmonics, noise and a falling sine): it makes the sequencer audible and is not the instrument's voice.
// Runs in the AudioWorkletGlobalScope: no allocation per sample, no `Math.random`, nothing that differs between two runs.
import type { MidiEvent } from "./abi.ts";

/** The channel the drums are on (General MIDI's percussion channel, as the engine numbers channels: 1 to 16). */
export const DRUM_CHANNEL = 10;
export const MAX_VOICES = 24;

/** A note event as the monitor reads it: the fields of a decoded record that it uses. */
export type NoteEvent = Pick<MidiEvent, "kind" | "channel" | "d1" | "d2" | "atSample">;

const NOTE_ON = 0;
const NOTE_OFF = 1;

type Shape = "tone" | "kick" | "clap" | "hat";

/** Seconds. A held tone starts in 3 ms, falls to its sustain with a 60 ms time constant, and is let go over 100 ms. */
const ATTACK = 0.003;
const DECAY = 0.06;
const SUSTAIN = 0.6;
const RELEASE = 0.1;
/** A note whose Note Off never comes (a dropped message) is let go after this long, so no voice hangs. */
const LONGEST_HOLD = 10;
/** How long each drum lasts. The last 20 ms of each is a fade to exactly zero, so a voice ends in silence and not in a step. */
const KICK_LENGTH = 0.4;
const CLAP_LENGTH = 0.25;
const HAT_LENGTH = 0.12;
const FADE = 0.02;
const VOICE_GAIN = 0.2;
const MASTER = 0.8;

class Voice {
  live = false;
  shape: Shape = "tone";
  channel = 1;
  note = 0;
  amp = 0;
  freq = 0;
  /** Frames since it began. */
  age = 0;
  /** Frames since its Note Off, or -1 while it is held. */
  released = -1;
  phase = 0;
  /** The noise a drum last drew, for the hat's high-pass. */
  last = 0;
}

export class Monitor {
  private readonly voices: Voice[] = Array.from({ length: MAX_VOICES }, () => new Voice());
  private readonly rate: number;
  private enabled = false;
  private seed = 0x9e3779b9;

  constructor(sampleRate: number) {
    this.rate = sampleRate;
  }

  /** Off until the page says otherwise. Switching it off drops every note: they do not come back when it is switched on. */
  get on(): boolean {
    return this.enabled;
  }

  set on(value: boolean) {
    this.enabled = value;
    if (!value) this.clear();
  }

  /** Drops every voice at once. Used by a reset; a Stop does not need it, because the engine's Note Offs come out of the next render. */
  clear(): void {
    for (const v of this.voices) v.live = false;
  }

  private noise(): number {
    // xorshift32, scaled to -1..1. One generator for the whole monitor, so two runs of the same notes make the same samples.
    let x = this.seed;
    x ^= x << 13;
    x ^= x >>> 17;
    x ^= x << 5;
    this.seed = x >>> 0;
    return (this.seed / 0x80000000) - 1;
  }

  private start(channel: number, note: number, velocity: number): void {
    let slot = this.voices.find((v) => !v.live);
    if (!slot) slot = this.voices.reduce((oldest, v) => (v.age > oldest.age ? v : oldest)); // the oldest is taken
    slot.live = true;
    slot.channel = channel;
    slot.note = note;
    slot.amp = (Math.max(1, Math.min(127, velocity)) / 127) * VOICE_GAIN;
    slot.age = 0;
    slot.released = -1;
    slot.phase = 0;
    slot.last = 0;
    slot.freq = 440 * Math.pow(2, (note - 69) / 12);
    slot.shape = channel !== DRUM_CHANNEL ? "tone" : note < 38 ? "kick" : note < 42 ? "clap" : "hat";
  }

  private stop(channel: number, note: number): void {
    let target: Voice | null = null;
    for (const v of this.voices) if (v.live && v.shape === "tone" && v.released < 0 && v.channel === channel && v.note === note && (!target || v.age > target.age)) target = v;
    if (target) target.released = 0;
  }

  /** The voice's next sample, and whether it has more. */
  private sample(v: Voice): number {
    const t = v.age / this.rate;
    let y = 0;
    let end = Infinity;
    if (v.shape === "tone") {
      if (v.released >= 0) {
        const r = v.released / this.rate;
        if (r >= RELEASE) {
          v.live = false;
          return 0;
        }
        y = this.held(t) * (1 - r / RELEASE);
        v.released++;
      } else {
        y = this.held(t);
        if (t >= LONGEST_HOLD) v.released = 0;
      }
      v.phase += (2 * Math.PI * v.freq) / this.rate;
      y *= (Math.sin(v.phase) + 0.5 * Math.sin(2 * v.phase) + 0.25 * Math.sin(3 * v.phase)) / 1.75;
    } else if (v.shape === "kick") {
      end = KICK_LENGTH;
      v.phase += (2 * Math.PI * (45 + 105 * Math.exp(-t / 0.04))) / this.rate; // a falling sine, 150 Hz to 45
      y = Math.exp(-t / 0.12) * Math.sin(v.phase) * 1.6;
    } else if (v.shape === "clap") {
      end = CLAP_LENGTH;
      v.phase += (2 * Math.PI * 190) / this.rate;
      y = Math.exp(-t / 0.06) * 0.8 * this.noise() + Math.exp(-t / 0.04) * 0.5 * Math.sin(v.phase);
    } else {
      end = HAT_LENGTH;
      const n = this.noise();
      y = Math.exp(-t / 0.02) * 0.9 * (n - v.last); // the difference of white noise is its high-pass
      v.last = n;
    }
    if (t >= end) {
      v.live = false;
      return 0;
    }
    if (t > end - FADE) y *= (end - t) / FADE;
    v.age++;
    return y * v.amp * (v.shape === "tone" ? 1 : 2.5);
  }

  private held(t: number): number {
    return Math.min(1, t / ATTACK) * (SUSTAIN + (1 - SUSTAIN) * Math.exp(-t / DECAY));
  }

  /**
   * Adds the sound of this block to `out`. `events` are the block's records in sample order, each with its offset inside the block; a note
   * starts on its own sample. With nothing playing and nothing to start the block is left exactly as it was.
   */
  render(out: Float32Array, events: readonly NoteEvent[]): void {
    if (!this.enabled) return;
    let e = 0;
    let any = this.voices.some((v) => v.live);
    for (let i = 0; i < out.length; i++) {
      while (e < events.length && events[e]!.atSample <= i) {
        const ev = events[e++]!;
        if (ev.kind === NOTE_ON && ev.d2 > 0) {
          this.start(ev.channel, ev.d1, ev.d2);
          any = true;
        } else if (ev.kind === NOTE_ON || ev.kind === NOTE_OFF) this.stop(ev.channel, ev.d1);
      }
      if (!any) continue;
      let mix = 0;
      let live = false;
      for (const v of this.voices) {
        if (!v.live) continue;
        mix += this.sample(v);
        live = live || v.live;
      }
      any = live;
      out[i] = Math.max(-1, Math.min(1, (out[i] ?? 0) + MASTER * Math.tanh(mix)));
    }
  }
}

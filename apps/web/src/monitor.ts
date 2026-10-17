// The built-in sound (specs/SPEC-0002/p6-runs-when-opened.md, R-2). The Octopus makes no sound of its own, but a page opened with nothing
// plugged in must be heard to run, and Web MIDI cannot be reached from every frame the page is shown in. So the worklet plays the engine's
// own notes in its output: a pitched voice for a note and a one-shot for each of three drums on MIDI channel 10 (kick, clap and hat). The drums can be switched off (`drums`).
//
// It is audio and not MIDI: it reads the events the engine made for this block, changes none of them, and keeps nothing the engine can see.
// The page's sound is a feature (SPEC-0004, ADR-0014): the pitched voice is one of the voices in `voices/registry.ts` (`voice`), and `Plain`,
// the sine and two harmonics the page began with, is the one it has until the owner says otherwise. The drums (noise and a falling sine) are
// not voices and are unchanged. This file owns the slots, the drums and the mix; a voice owns how one note sounds.
// Runs in the AudioWorkletGlobalScope: no allocation per sample, no `Math.random`, nothing that differs between two runs.
import type { MidiEvent } from "./abi.ts";
import { DEFAULT_VOICE } from "./voices/registry.ts";
import { SLOT_COUNT, type Voice, type VoiceNote } from "./voices/voice.ts";

/** The channel the drums are on (General MIDI's percussion channel, as the engine numbers channels: 1 to 16). */
export const DRUM_CHANNEL = 10;
export const MAX_VOICES = SLOT_COUNT;

/** A note event as the monitor reads it: the fields of a decoded record that it uses. */
export type NoteEvent = Pick<MidiEvent, "kind" | "channel" | "d1" | "d2" | "atSample">;

const NOTE_ON = 0;
const NOTE_OFF = 1;

type Shape = "tone" | "kick" | "clap" | "hat";

/** How long each drum lasts. The last 20 ms of each is a fade to exactly zero, so a voice ends in silence and not in a step. */
const KICK_LENGTH = 0.4;
const CLAP_LENGTH = 0.25;
const HAT_LENGTH = 0.12;
const FADE = 0.02;
/** A drum's level at velocity 127, before the mix. */
const DRUM_GAIN = 0.2;
const MASTER = 0.8;

/** One of the monitor's slots: a pitched note (its sound is `tone`, made by the chosen voice) or a drum hit (its state is the fields below). */
class Slot {
  live = false;
  shape: Shape = "tone";
  channel = 1;
  note = 0;
  amp = 0;
  /** A drum's frames since it began. A pitched note's age is its `tone`'s. */
  age = 0;
  phase = 0;
  /** The noise a drum last drew, for the hat's high-pass. */
  last = 0;
  tone: VoiceNote;

  constructor(tone: VoiceNote) {
    this.tone = tone;
  }
}

export class Monitor {
  private readonly slots: Slot[];
  private readonly rate: number;
  private chosen: Voice = DEFAULT_VOICE;
  private enabled = false;
  /**
   * Whether the drums (the notes on `DRUM_CHANNEL`) are played. On until the page says otherwise, and independent of `on`: the sound being switched off and on
   * does not change it. Off, a drum note is not started and adds nothing; a hit that is already sounding finishes as it would have (it ends in its own fade
   * to zero, so there is no step), and the synth's voices are not touched, so the output is the synth's sample for sample (specs/SPEC-0002/p10-silence-the-drums.md).
   * It is audio and nothing else: the drum notes are still in the engine's events and go to every output that is chosen.
   */
  drums = true;
  private seed = 0x9e3779b9;

  constructor(sampleRate: number) {
    this.rate = sampleRate;
    this.slots = Array.from({ length: MAX_VOICES }, () => new Slot(this.chosen.create(sampleRate)));
  }

  /** The voice the pitched notes are played with. */
  get voice(): Voice {
    return this.chosen;
  }

  /**
   * Chooses the voice for the notes that start from now on. A different voice drops every pitched note that is sounding (they do not
   * carry over: the old voice's state means nothing to the new one); a drum that is sounding finishes as it would have. The same voice again changes nothing.
   */
  set voice(next: Voice) {
    if (next === this.chosen) return;
    this.chosen = next;
    for (const slot of this.slots) {
      slot.tone = next.create(this.rate);
      if (slot.shape === "tone") slot.live = false;
    }
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
    for (const slot of this.slots) slot.live = false;
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

  /** A slot's age in frames: the oldest is the one taken. */
  private ageOf(slot: Slot): number {
    return slot.shape === "tone" ? slot.tone.age : slot.age;
  }

  private start(channel: number, note: number, velocity: number): void {
    const pitched = channel !== DRUM_CHANNEL;
    let slot: Slot | undefined;
    if (pitched) {
      let sounding = 0;
      for (const s of this.slots) if (s.live && s.shape === "tone") sounding++;
      // The voice's own cap: past it, the oldest note of this voice is taken (SPEC-0004 D14). A voice that allows the whole pool never gets here first.
      if (sounding >= this.chosen.maxNotes) slot = this.slots.filter((s) => s.live && s.shape === "tone").reduce((oldest, s) => (s.tone.age > oldest.tone.age ? s : oldest));
    }
    slot ??= this.slots.find((s) => !s.live);
    slot ??= this.slots.reduce((oldest, s) => (this.ageOf(s) > this.ageOf(oldest) ? s : oldest)); // the oldest is taken
    slot.live = true;
    slot.channel = channel;
    slot.note = note;
    slot.shape = pitched ? "tone" : note < 38 ? "kick" : note < 42 ? "clap" : "hat";
    if (pitched) {
      slot.tone.start(note, velocity);
    } else {
      slot.amp = (Math.max(1, Math.min(127, velocity)) / 127) * DRUM_GAIN;
      slot.age = 0;
      slot.phase = 0;
      slot.last = 0;
    }
  }

  private stop(channel: number, note: number): void {
    let target: Slot | null = null;
    for (const s of this.slots) if (s.live && s.shape === "tone" && s.tone.held && s.channel === channel && s.note === note && (!target || s.tone.age > target.tone.age)) target = s;
    if (target) target.tone.release();
  }

  /** The slot's next sample, and whether it has more. */
  private sample(v: Slot): number {
    if (v.shape === "tone") {
      const y = v.tone.next();
      v.live = v.tone.live;
      return y;
    }
    const t = v.age / this.rate;
    let y = 0;
    let end = Infinity;
    if (v.shape === "kick") {
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
    return y * v.amp * 2.5;
  }

  /**
   * Adds the sound of this block to `out`. `events` are the block's records in sample order, each with its offset inside the block; a note
   * starts on its own sample. With nothing playing and nothing to start the block is left exactly as it was.
   */
  render(out: Float32Array, events: readonly NoteEvent[]): void {
    if (!this.enabled) return;
    let e = 0;
    let any = this.slots.some((v) => v.live);
    for (let i = 0; i < out.length; i++) {
      while (e < events.length && events[e]!.atSample <= i) {
        const ev = events[e++]!;
        if (ev.kind === NOTE_ON && ev.d2 > 0) {
          if (ev.channel !== DRUM_CHANNEL || this.drums) {
            this.start(ev.channel, ev.d1, ev.d2);
            any = true;
          }
        } else if (ev.kind === NOTE_ON || ev.kind === NOTE_OFF) this.stop(ev.channel, ev.d1);
      }
      if (!any) continue;
      let mix = 0;
      let live = false;
      for (const v of this.slots) {
        if (!v.live) continue;
        mix += this.sample(v);
        live = live || v.live;
      }
      any = live;
      out[i] = Math.max(-1, Math.min(1, (out[i] ?? 0) + MASTER * Math.tanh(mix)));
    }
  }
}

// The checks every voice is held to (SPEC-0004 section 5, H-2, H-3, H-6, H-9, H-11, H-13). Each takes a voice and returns the
// problems it found as sentences; an empty list is a pass. They are functions, and not assertions, so a test can also hand them
// a voice that is wrong on purpose and see them say so: a check that cannot fail proves nothing.
import { readFileSync } from "node:fs";
import type { Voice, VoiceNote } from "../../src/voices/voice.ts";
import { Rng } from "./rng.ts";
import { centroid, cents, fundamental, inharmonicDb } from "./spectrum.ts";

/** H-2: each voice's fundamental is within this many cents of the note, for MIDI 24 to 96. */
export const PITCH_TOLERANCE_CENTS = 2;
export const PITCH_NOTES: readonly number[] = Array.from({ length: 73 }, (_, i) => 24 + i);
/** H-3: the energy that is not on a harmonic is at least this many decibels under the fundamental, for MIDI 60 to 84. Aimed at 50; only ever tightened. */
export const ALIAS_BOUND_DB = -50;
export const ALIAS_NOTES: readonly number[] = Array.from({ length: 25 }, (_, i) => 60 + i);
/** H-13: the heaviest voice, 24 notes at once, renders 10 s of audio in under this share of real time in Node. */
export const COST_BOUND = 0.25;
/** H-6: random settings tried at 44.1, 48 and 96 kHz. */
export const STABILITY_CASES = 10_000;
/** H-9: a voice that declares velocity brightens it must raise its spectral centroid by at least this ratio from velocity 20 to 127. Set by the first voice that declares it (S3); until then only the other direction runs. */
export const VELOCITY_RISE_MIN = 1.02;

const RATE = 48_000;
/** Samples to let an attack and a decay settle before a note is measured: 0.6 s. */
const SETTLE = 28_800;
/** The window the spectrum measurements use. */
const WINDOW = 65_536;

/** One note of `voice` as plain numbers: started at `velocity`, released at sample `hold`, `total` samples long. */
export function renderNote(voice: Voice, rate: number, midi: number, velocity: number, hold: number, total: number): Float64Array {
  const note = voice.create(rate);
  note.start(midi, velocity);
  const out = new Float64Array(total);
  for (let i = 0; i < total; i++) {
    if (i === hold) note.release();
    out[i] = note.next();
  }
  return out;
}

const hz = (midi: number): number => 440 * 2 ** ((midi - 69) / 12);

/** H-2. */
export function checkPitch(voice: Voice, notes: readonly number[] = PITCH_NOTES): string[] {
  const problems: string[] = [];
  for (const midi of notes) {
    const x = renderNote(voice, RATE, midi, 100, Infinity, SETTLE + WINDOW + WINDOW / 4);
    const c = cents(fundamental(x.subarray(SETTLE), RATE, hz(midi)), hz(midi));
    if (!(Math.abs(c) <= PITCH_TOLERANCE_CENTS)) problems.push(`${voice.id}: MIDI ${midi} is ${c.toFixed(2)} cents off (at most ${PITCH_TOLERANCE_CENTS})`);
  }
  return problems;
}

/** H-3. */
export function checkAlias(voice: Voice, notes: readonly number[] = ALIAS_NOTES, bound = ALIAS_BOUND_DB): string[] {
  const problems: string[] = [];
  for (const midi of notes) {
    const x = renderNote(voice, RATE, midi, 100, Infinity, SETTLE + WINDOW);
    const db = inharmonicDb(x.subarray(SETTLE), RATE, hz(midi), { n: WINDOW });
    if (!(db <= bound)) problems.push(`${voice.id}: MIDI ${midi} has ${db.toFixed(1)} dB of energy off the harmonics (at most ${bound})`);
  }
  return problems;
}

/** H-6, at the voice: no NaN, no infinity, nothing above 1, and silence within 5 s of the Note Off, with exactly zero after. */
export function checkStable(voice: Voice, cases = STABILITY_CASES, seed = 1): string[] {
  const problems: string[] = [];
  const rng = new Rng(seed);
  const rates = [44_100, 48_000, 96_000];
  for (let c = 0; c < cases && problems.length < 5; c++) {
    const rate = rates[rng.int(rates.length)]!;
    const midi = rng.int(128);
    const velocity = 1 + rng.int(127);
    const hold = rng.int(Math.floor(0.04 * rate));
    const note = voice.create(rate);
    note.start(midi, velocity);
    const label = `${voice.id}: MIDI ${midi}, velocity ${velocity}, ${rate} Hz, held ${hold} samples`;
    let released = -1;
    let bad = "";
    for (let i = 0; i < 6 * rate && !bad; i++) {
      if (i === hold) {
        note.release();
        released = i;
      }
      const y = note.next();
      if (!Number.isFinite(y)) bad = `${label}: sample ${i} is ${y}`;
      else if (Math.abs(y) > 1) bad = `${label}: sample ${i} is ${y}, past full scale`;
      else if (released >= 0 && !note.live) {
        if (note.next() !== 0) bad = `${label}: not exactly zero after it ended`;
        if (i - released > 5 * rate) bad = `${label}: still sounding ${i - released} samples after the Note Off`;
        break;
      }
      if (released >= 0 && i - released > 5 * rate) bad = `${label}: still sounding 5 s after the Note Off`;
    }
    if (bad) problems.push(bad);
  }
  return problems;
}

/** H-9, both ways: a voice that says velocity brightens it must show the centroid rise, and one that says it does not must show none. Either way a louder note is louder. */
export function checkVelocity(voice: Voice): string[] {
  const at = (velocity: number): Float64Array => renderNote(voice, RATE, 60, velocity, Infinity, SETTLE + WINDOW).subarray(SETTLE);
  const quiet = at(20);
  const loud = at(127);
  const problems: string[] = [];
  const peak = (a: Float64Array): number => a.reduce((m, v) => Math.max(m, Math.abs(v)), 0);
  if (!(peak(loud) > peak(quiet))) problems.push(`${voice.id}: velocity 127 is not louder than velocity 20`);
  const ratio = centroid(loud, RATE, WINDOW) / centroid(quiet, RATE, WINDOW);
  if (voice.traits.velocityBrightens) {
    if (!(ratio >= VELOCITY_RISE_MIN)) problems.push(`${voice.id}: declares that velocity brightens it, but the centroid moved by ${ratio.toFixed(4)} (at least ${VELOCITY_RISE_MIN})`);
  } else if (!(Math.abs(ratio - 1) <= 1e-6)) {
    problems.push(`${voice.id}: declares that velocity does not brighten it, but the centroid moved by ${ratio.toFixed(6)}`);
  }
  return problems;
}

/** H-11: two runs of the same notes are equal sample for sample. */
export function checkDeterministic(voice: Voice): string[] {
  const run = (): Float64Array[] => [40, 64, 91].map((m) => renderNote(voice, RATE, m, 90, 5000, 20_000));
  const a = run();
  const b = run();
  const problems: string[] = [];
  a.forEach((x, n) => {
    const at = x.findIndex((v, i) => !Object.is(v, b[n]![i]));
    if (at >= 0) problems.push(`${voice.id}: two runs differ at sample ${at} of note ${n + 1}`);
  });
  return problems;
}

/** H-13, the time half: 24 notes of the voice render `seconds` of audio in under `bound` of real time, measured in this process's CPU time. Best of `runs` (three), so one slow moment on a busy machine does not fail it; the bound is not what gives. */
export function checkCost(voice: Voice, options: { seconds?: number; notes?: number; bound?: number; runs?: number } = {}): { problems: string[]; ratio: number } {
  const seconds = options.seconds ?? 10;
  const count = options.notes ?? 24;
  const bound = options.bound ?? COST_BOUND;
  const frames = Math.floor(seconds * RATE);
  let best = Infinity;
  for (let r = 0; r < (options.runs ?? 3); r++) {
    const notes: VoiceNote[] = Array.from({ length: count }, (_, i) => {
      const n = voice.create(RATE);
      n.start(36 + 2 * i, 100);
      return n;
    });
    let sum = 0;
    const c0 = process.cpuUsage();
    for (let i = 0; i < frames; i++) for (let k = 0; k < count; k++) sum += notes[k]!.next();
    const c1 = process.cpuUsage(c0);
    best = Math.min(best, (c1.user + c1.system) / 1e6 / seconds); // CPU time, so another process on the machine does not count against the voice
    if (!Number.isFinite(sum)) return { problems: [`${voice.id}: the cost run made ${sum}`], ratio: best };
  }
  return { problems: best < bound ? [] : [`${voice.id}: ${count} notes took ${(best * 100).toFixed(1)}% of real time (under ${bound * 100}%)`], ratio: best };
}

/** The text of every `next(): number { ... }` in `source`, the sample loop of a voice, with comments removed. */
export function sampleLoops(source: string): string[] {
  const clean = source.replace(/\/\*[\s\S]*?\*\//g, "").replace(/(^|[^:])\/\/.*$/gm, "$1");
  const bodies: string[] = [];
  const head = /\bnext\(\)\s*:\s*number\s*\{/g;
  for (let m = head.exec(clean); m; m = head.exec(clean)) {
    let depth = 1;
    let i = head.lastIndex;
    for (; i < clean.length && depth > 0; i++) {
      if (clean[i] === "{") depth++;
      else if (clean[i] === "}") depth--;
    }
    bodies.push(clean.slice(head.lastIndex, i - 1));
  }
  return bodies;
}

const ALLOCATIONS: [RegExp, string][] = [
  [/\bnew\b/, "`new`"],
  [/(^|[=(,:?&|+\-*/]|\breturn)\s*\[/, "an array literal"],
  [/(^|[=(,:?&|+\-*/]|\breturn)\s*\{/, "an object literal"],
  [/\.(map|filter|reduce|forEach|slice|concat|flatMap|splice)\s*\(/, "an array method"],
  [/\bArray\.(from|of)\b/, "`Array.from`"],
  [/=>/, "a closure"],
  [/\bfunction\b/, "a function expression"],
  [/`/, "a template string"],
  [/\.\.\./, "a spread"],
];

/** H-13, the text half: nothing in a sample loop allocates. */
export function checkNoAllocation(source: string, label: string): string[] {
  const problems: string[] = [];
  for (const body of sampleLoops(source)) for (const [re, what] of ALLOCATIONS) if (re.test(body)) problems.push(`${label}: a sample loop contains ${what}`);
  return problems;
}

export function checkNoAllocationInFile(path: string): { problems: string[]; loops: number } {
  const source = readFileSync(path, "utf8");
  return { problems: checkNoAllocation(source, path), loops: sampleLoops(source).length };
}

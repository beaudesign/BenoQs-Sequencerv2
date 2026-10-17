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
/** The sample rates the page runs at. A voice must hold its pitch at each: one that has 48000 written into it is wrong at the others. */
export const PITCH_RATES: readonly number[] = [44_100, 48_000, 96_000];
/** At the rates other than 48 kHz, a spread of notes (73 notes at three rates is three times the time for no more information). */
export const PITCH_NOTES_OFF_RATE: readonly number[] = [24, 36, 48, 60, 69, 84, 96];
/** H-3: the energy that is not on a harmonic is at least this many decibels under the fundamental, for MIDI 60 to 84. Aimed at 50; only ever tightened. */
export const ALIAS_BOUND_DB = -50;
export const ALIAS_NOTES: readonly number[] = [...[36, 42, 48, 54], ...Array.from({ length: 25 }, (_, i) => 60 + i), ...[90, 96]];
/** A note below this level (RMS, full scale 1) is silence: -80 dB. A voice that is silent on a note it should play is reported as silent, not as an odd pitch. */
export const AUDIBLE_RMS = 1e-4;
/** The velocities the velocity check looks at, quietest first. */
export const VELOCITIES: readonly number[] = [20, 50, 80, 110, 127];
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
/** The attack window the velocity check also looks at: 4096 samples, 85 ms. */
const EARLY = 4096;

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
const rms = (x: ArrayLike<number>): number => {
  let s = 0;
  for (let i = 0; i < x.length; i++) s += x[i]! * x[i]!;
  return Math.sqrt(s / Math.max(1, x.length));
};

/** H-2, at 48 kHz for every note asked for and at 44.1 and 96 kHz for a spread of them (an independent review found the rig ran at 48 kHz only, so a voice with 48000 written into it passed). */
export function checkPitch(voice: Voice, notes: readonly number[] = PITCH_NOTES, rates: readonly number[] = PITCH_RATES): string[] {
  const problems: string[] = [];
  for (const rate of rates) {
    // 0.6 s of settling at any rate; the window is a fixed number of samples, as the spectrum helpers want.
    const settle = rate === RATE ? SETTLE : Math.round(0.6 * rate);
    for (const midi of rate === RATE ? notes : notes.filter((n) => PITCH_NOTES_OFF_RATE.includes(n))) {
      const x = renderNote(voice, rate, midi, 100, Infinity, settle + WINDOW + WINDOW / 4);
      if (!(rms(x.subarray(settle)) >= AUDIBLE_RMS)) {
        problems.push(`${voice.id}: MIDI ${midi} at ${rate} Hz is silent`);
        continue;
      }
      const c = cents(fundamental(x.subarray(settle), rate, hz(midi)), hz(midi));
      if (!(Math.abs(c) <= PITCH_TOLERANCE_CENTS)) problems.push(`${voice.id}: MIDI ${midi} at ${rate} Hz is ${c.toFixed(2)} cents off (at most ${PITCH_TOLERANCE_CENTS})`);
    }
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

/**
 * H-9, both ways, at five velocities and in two windows (an independent review found two velocities, 0.6 s in, let a brightening that
 * only happens in the attack, and a rise-then-fall with velocity, through). A louder note is always louder. A voice that says velocity
 * brightens it must show the steady-state centroid rise with each step up in velocity (never fall) and by the bound overall; one that
 * says it does not must show no movement at all, in the steady state or in the first 4096 samples.
 */
export function checkVelocity(voice: Voice): string[] {
  const steady = (velocity: number): Float64Array => renderNote(voice, RATE, 60, velocity, Infinity, SETTLE + WINDOW).subarray(SETTLE);
  const early = (velocity: number): Float64Array => renderNote(voice, RATE, 60, velocity, Infinity, EARLY);
  const problems: string[] = [];
  const peak = (a: Float64Array): number => a.reduce((m, v) => Math.max(m, Math.abs(v)), 0);
  const s = VELOCITIES.map(steady);
  const e = VELOCITIES.map(early);
  for (let i = 1; i < VELOCITIES.length; i++) {
    if (!(peak(s[i]!) > peak(s[i - 1]!))) problems.push(`${voice.id}: velocity ${VELOCITIES[i]} is not louder than velocity ${VELOCITIES[i - 1]}`);
  }
  const cs = s.map((x) => centroid(x, RATE, WINDOW));
  const ce = e.map((x) => centroid(x, RATE, EARLY));
  if (voice.traits.velocityBrightens) {
    const ratio = cs[cs.length - 1]! / cs[0]!;
    if (!(ratio >= VELOCITY_RISE_MIN)) problems.push(`${voice.id}: declares that velocity brightens it, but the centroid moved by ${ratio.toFixed(4)} from velocity ${VELOCITIES[0]} to ${VELOCITIES[VELOCITIES.length - 1]} (at least ${VELOCITY_RISE_MIN})`);
    for (let i = 1; i < cs.length; i++) {
      if (!(cs[i]! >= cs[i - 1]! * (1 - 1e-3))) problems.push(`${voice.id}: declares that velocity brightens it, but the centroid falls from velocity ${VELOCITIES[i - 1]} to ${VELOCITIES[i]}`);
    }
  } else {
    const spread = (c: number[]): number => Math.max(...c.map((v) => Math.abs(v / c[0]! - 1)));
    if (!(spread(cs) <= 1e-6)) problems.push(`${voice.id}: declares that velocity does not brighten it, but the centroid moved by ${spread(cs).toFixed(6)}`);
    if (!(spread(ce) <= 1e-6)) problems.push(`${voice.id}: declares that velocity does not brighten it, but the first ${EARLY} samples brighten with velocity (${spread(ce).toFixed(6)})`);
  }
  return problems;
}

/**
 * H-11: two runs of the same notes are equal sample for sample. Four notes of different pitch, velocity and length, up to six seconds
 * (a review found `Math.random` once per 40,000 samples passed 20,000 samples at velocity 90).
 */
export function checkDeterministic(voice: Voice): string[] {
  const notes: [number, number, number, number][] = [
    [40, 30, 5000, 20_000],
    [64, 90, 144_000, 288_000],
    [91, 127, Infinity, 288_000],
    [100, 110, 30_000, 288_000],
  ];
  const run = (): Float64Array[] => notes.map(([m, v, hold, total]) => renderNote(voice, RATE, m, v, hold, total));
  const a = run();
  const b = run();
  const problems: string[] = [];
  a.forEach((x, n) => {
    const at = x.findIndex((v, i) => !Object.is(v, b[n]![i]));
    if (at >= 0) problems.push(`${voice.id}: two runs differ at sample ${at} of note ${n + 1}`);
  });
  return problems;
}

/**
 * H-13, the time half: `count` (24) notes of the voice render `seconds` of audio in under `bound` of real time, measured in this
 * process's CPU time, in two workloads, and the worse of the two is the ratio. **Held**: 24 notes started once and left to sound.
 * **Churn**: every note restarted every quarter of a second (staggered), across MIDI 24 to 108 and velocity 20 to 127, so a note's
 * `start()` is paid for, a note that dies young does not hide the cost of the sounding part, and high notes and hard velocities are
 * played (an independent review found `start()` untimed, a voice that ends at 0.5 s reading 4 percent against 36 percent when held,
 * and nothing above MIDI 90 or velocity 110 exercised). Best of `runs` (three) for each, so one slow moment on a busy machine does
 * not fail it; the bound is not what gives.
 */
export function checkCost(voice: Voice, options: { seconds?: number; notes?: number; bound?: number; runs?: number } = {}): { problems: string[]; ratio: number } {
  const seconds = options.seconds ?? 10;
  const count = options.notes ?? 24;
  const bound = options.bound ?? COST_BOUND;
  const frames = Math.floor(seconds * RATE);
  const runs = options.runs ?? 3;
  const CHURN_PERIOD = 12_000; // a quarter of a second at 48 kHz
  let held = Infinity;
  let churn = Infinity;
  for (let r = 0; r < runs; r++) {
    const notes: VoiceNote[] = Array.from({ length: count }, (_, i) => {
      const n = voice.create(RATE);
      n.start(36 + 2 * i, 100);
      return n;
    });
    let sum = 0;
    const c0 = process.cpuUsage();
    for (let i = 0; i < frames; i++) for (let k = 0; k < count; k++) sum += notes[k]!.next();
    const c1 = process.cpuUsage(c0);
    held = Math.min(held, (c1.user + c1.system) / 1e6 / seconds); // CPU time, so another process on the machine does not count against the voice
    if (!Number.isFinite(sum)) return { problems: [`${voice.id}: the cost run made ${sum}`], ratio: held };
  }
  for (let r = 0; r < runs; r++) {
    const notes: VoiceNote[] = Array.from({ length: count }, () => voice.create(RATE));
    const stagger = Math.floor(CHURN_PERIOD / count);
    let sum = 0;
    let n = 0;
    const c0 = process.cpuUsage();
    for (let i = 0; i < frames; i++) {
      for (let k = 0; k < count; k++) {
        const at = i - k * stagger;
        if (at >= 0 && at % CHURN_PERIOD === 0) {
          n++;
          notes[k]!.start(24 + ((n * 37 + k * 11) % 85), 20 + ((n * 53 + k * 7) % 108));
        }
        sum += notes[k]!.next();
      }
    }
    const c1 = process.cpuUsage(c0);
    churn = Math.min(churn, (c1.user + c1.system) / 1e6 / seconds);
    if (!Number.isFinite(sum)) return { problems: [`${voice.id}: the churn cost run made ${sum}`], ratio: churn };
  }
  const problems: string[] = [];
  if (!(held < bound)) problems.push(`${voice.id}: ${count} held notes took ${(held * 100).toFixed(1)}% of real time (under ${bound * 100}%)`);
  if (!(churn < bound)) problems.push(`${voice.id}: ${count} notes restarted every quarter second took ${(churn * 100).toFixed(1)}% of real time (under ${bound * 100}%)`);
  return { problems, ratio: Math.max(held, churn) };
}

/** `name(...) {` heads: a class member or a method, not a control statement. */
const MEMBER = /(?:^|[\s;{}])(?:(?:public|private|protected|static|readonly|async|get|set)\s+)*([A-Za-z_$][\w$]*)\s*\([^()]*\)\s*(?::\s*[^{;=]+?)?\s*\{/g;
/** `name = (...) => {` heads: an arrow-function property. */
const ARROW_MEMBER = /(?:^|[\s;{}])(?:(?:public|private|protected|static|readonly)\s+)*([A-Za-z_$][\w$]*)\s*=\s*(?:async\s*)?\([^()]*\)\s*(?::\s*[^=]+?)?\s*=>\s*\{/g;
const NOT_MEMBERS = new Set(["if", "for", "while", "switch", "catch", "function", "return", "constructor", "with"]);

function matchingBrace(s: string, open: number): number {
  let depth = 1;
  let i = open + 1;
  for (; i < s.length && depth > 0; i++) {
    if (s[i] === "{") depth++;
    else if (s[i] === "}") depth--;
  }
  return i - 1;
}

/**
 * The text of the code a note runs while it sounds: the body of every method of every class in `source` except the constructor, with
 * comments removed. That is `next()`, and `start()`, `release()`, getters and any helper they call (an independent review found the
 * first version read only `next(): number { ... }`, so an allocating helper, an arrow-function `next` and an unannotated `next()`
 * passed). A module-level function a method calls is not looked at.
 */
export function sampleLoops(source: string): string[] {
  const clean = source.replace(/\/\*[\s\S]*?\*\//g, "").replace(/(^|[^:])\/\/.*$/gm, "$1");
  const bodies: string[] = [];
  const cls = /\bclass\s+[A-Za-z_$][\w$]*[^{]*\{/g;
  for (let c = cls.exec(clean); c; c = cls.exec(clean)) {
    const open = cls.lastIndex - 1;
    const close = matchingBrace(clean, open);
    const body = clean.slice(open + 1, close);
    const depthAt = new Int32Array(body.length + 1);
    let d = 0;
    for (let i = 0; i < body.length; i++) {
      depthAt[i] = d;
      if (body[i] === "{") d++;
      else if (body[i] === "}") d--;
    }
    for (const re of [MEMBER, ARROW_MEMBER]) {
      re.lastIndex = 0;
      for (let m = re.exec(body); m; m = re.exec(body)) {
        const braceAt = re.lastIndex - 1;
        // Only members of the class itself (depth 0 at the head), not an `if (...) {` inside a method.
        if (depthAt[m.index + (m[0].length - m[0].trimStart().length)] !== 0 || NOT_MEMBERS.has(m[1]!)) continue;
        bodies.push(body.slice(braceAt + 1, matchingBrace(body, braceAt)));
      }
    }
  }
  return bodies;
}

const ALLOCATIONS: [RegExp, string][] = [
  [/\bnew\b/, "`new`"],
  [/(^|[=(,:?&|+\-*/]|\breturn)\s*\[/, "an array literal"],
  [/(^|[=(,:?&|+\-*/]|\breturn)\s*\{/, "an object literal"],
  [/\.(map|filter|reduce|forEach|slice|concat|flatMap|splice|push|pop|shift|unshift|join|split|toFixed|toString|padStart|padEnd|toLocaleString)\s*\(/, "an array or string method"],
  [/\w*Array\.(from|of)\b/, "`Array.from`"],
  [/\bArray\s*\(/, "`Array(`"],
  [/\b(String|JSON)\b/, "`String` or `JSON`"],
  [/\bObject\.(keys|values|entries|assign|fromEntries|create)\b/, "an `Object` function that makes an object"],
  [/["'][^"'\n]*["']\s*\+|\+\s*["']/, "string building"],
  [/=>/, "a closure"],
  [/\bfunction\b/, "a function expression"],
  [/`/, "a template string"],
  [/\.\.\./, "a spread"],
];

/** H-13, the text half: nothing a sounding note runs allocates. */
export function checkNoAllocation(source: string, label: string): string[] {
  const problems: string[] = [];
  for (const body of sampleLoops(source)) for (const [re, what] of ALLOCATIONS) if (re.test(body)) problems.push(`${label}: a method a note runs contains ${what}`);
  return problems;
}

export function checkNoAllocationInFile(path: string): { problems: string[]; loops: number } {
  const source = readFileSync(path, "utf8");
  return { problems: checkNoAllocation(source, path), loops: sampleLoops(source).length };
}

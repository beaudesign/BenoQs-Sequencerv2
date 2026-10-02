// What the owner reads: a saved S3 file, read again from its raw records and put in words. Used by `report.ts` (the owner's file) and by
// `run.ts` (the simulated run); both go through the same code, so what the owner's file says is worded as the one made here was.
//
// The numbers are recomputed from the raw records and not taken from the analysis the page stored in the file, so a different settling
// or a tempo to compare with can be asked for without another 30 minutes.
import type { FollowResult, SendResult } from "./page.ts";
import { analyseFollow, followVerdict, unpackPulses, type FollowAnalysis, type FollowOptions, type Spread } from "./analyse.ts";

export interface Saved {
  /** Set by the runner when the run was made with a stand-in: what the clock and the port were. A file from the page itself has none. */
  simulated?: string;
  commit?: string;
  browser?: string;
  result: FollowResult | SendResult;
}

export function readSaved(text: string): Saved {
  let parsed: unknown;
  try {
    parsed = JSON.parse(text);
  } catch (e) {
    throw new Error(`the file is not JSON: ${e instanceof Error ? e.message : String(e)}`);
  }
  const outer = (parsed ?? {}) as { schema?: unknown; result?: { schema?: unknown } };
  const inner = outer.result !== undefined ? outer.result : (outer as { schema?: unknown });
  const schema = inner.schema;
  if (schema !== "wenge.s3.follow/1" && schema !== "wenge.s3.send/1") throw new Error(`not an S3 result: its schema is ${JSON.stringify(schema)}`);
  return outer.result !== undefined ? { ...(parsed as Omit<Saved, "result">), result: outer.result as FollowResult | SendResult } : { result: parsed as FollowResult | SendResult };
}

/** The records of a follow run read again, for all of the samples, or those taken with the tab hidden or visible. */
export function reanalyse(result: FollowResult, options: FollowOptions = {}): FollowAnalysis {
  return analyseFollow({ pulses: unpackPulses(result.raw.pulses), samples: result.raw.samples, messages: result.raw.messages }, options);
}

const ms = (x: number): string => `${x.toFixed(2)} ms`;
const spreadLine = (label: string, s: Spread): string => `  ${label}: n ${s.n}, mean ${ms(s.mean)}, sigma ${ms(s.sigma)}, min ${ms(s.min)}, median ${ms(s.p50)}, 99th ${ms(s.p99)}, max ${ms(s.max)}`;
const head = (saved: Saved): string[] => (saved.simulated !== undefined ? [`SIMULATED: ${saved.simulated}`] : []);
const where = (saved: Saved): string => [saved.commit ? `commit ${saved.commit}` : null, saved.browser ?? null].filter(Boolean).join(", ");

function phaseLine(label: string, a: FollowAnalysis): string[] {
  const p = a.phase;
  const s = p.skipped;
  return [
    `  ${label}: counted ${p.counted} (left out: settling ${s.settling}, clock lost ${s.lost}, transport stopped or no phase ${s.stopped}); beyond a fifth of a step (${ms(p.fifthOfStepMs)}): ${p.beyondFifth} (${(p.share * 100).toFixed(2)}%); |phase| median ${ms(p.absMs.p50)}, 99th ${ms(p.absMs.p99)}, max ${ms(p.maxAbsMs)}`,
  ];
}

function intervalLines(a: FollowAnalysis): string[] {
  const i = a.intervals;
  return [
    `Clock domain: ${a.clockDomain.verdict}: ${a.clockDomain.why}`,
    `Pulses: ${a.pulses}; tempo from the periods: median ${i.tempoBpm.median.toFixed(3)} BPM, from the run ${i.tempoBpm.fromRun.toFixed(3)} BPM; drift ${i.driftBpmPerHour.toFixed(2)} BPM an hour`,
    spreadLine("period", i.periodMs),
    `  stalls ${i.stalls} (a gap over 500 ms), stamps going backwards ${i.backwards}, equal stamps ${i.equalStamps}, bursts ${i.bursts} (pulses handed over together)`,
  ];
}

export function formatFollow(saved: { simulated?: string; commit?: string; browser?: string; result: FollowResult }, options: FollowOptions = {}): string[] {
  const r = saved.result;
  const nominal = options.nominalBpm ?? r.config.nominalBpm ?? undefined;
  const opts: FollowOptions = { ...options, ...(nominal === undefined ? {} : { nominalBpm: nominal }) };
  const all = reanalyse(r, opts);
  const visible = reanalyse(r, { ...opts, hidden: false });
  const hidden = reanalyse(r, { ...opts, hidden: true });
  // The plan's criterion is for a visible tab. A run that was hidden throughout has nothing visible to read, and is read as a whole.
  const source = visible.phase.counted > 0 || all.phase.counted === 0 ? visible : all;
  const verdict = followVerdict(source, r.durationS, r.hiddenS);
  const lines = [
    ...head(saved),
    `Spike S3, follow a clock: input "${r.port}", ${r.durationS.toFixed(0)} s run started ${r.startedAt}, tab hidden ${r.hiddenS.toFixed(1)} s (${Math.max(0, r.visibility.length - 1)} changes of visibility)`,
    `${where(saved)}${where(saved) ? "; " : ""}${r.environment.userAgent}, ${r.environment.platform}; audio ${r.environment.audioSampleRate} Hz, base latency ${r.environment.baseLatencyMs.toFixed(1)} ms, output latency ${r.environment.outputLatencyMs.toFixed(1)} ms; follower lookahead ${r.config.lookaheadMs} ms, offset ${r.config.offsetMs} ms`,
    `Input path: received ${r.input.received}, decoded ${r.input.decoded}, ignored ${r.input.ignored}, malformed ${r.input.malformed}, timestamps going backwards ${r.input.backwards}; clock ${r.input.realtime.clock}, Start ${r.input.realtime.start}, Continue ${r.input.realtime.continue}, Stop ${r.input.realtime.stop}; worklet errors ${r.worklet.errors.length}`,
    ...intervalLines(all),
    "Phase (the engine against where the sender's clock says it should be; positive is ahead):",
    ...phaseLine("all samples", all),
    ...(r.hiddenS > 0 ? [...phaseLine("tab visible", visible), ...phaseLine("tab hidden", hidden)] : []),
  ];
  if (all.tempo.meanErrorBpm !== null) lines.push(`Tempo the follower read, less the nominal ${nominal} BPM: mean ${all.tempo.meanErrorBpm.toFixed(3)}, largest ${all.tempo.maxAbsErrorBpm!.toFixed(3)} BPM`);
  lines.push(`The plan's criterion (every phase sample within a fifth of a step, tab visible; a proposal, the owner's to set): ${verdict.met === null ? "not read" : verdict.met ? "MET" : "NOT MET"}`);
  for (const line of verdict.lines) lines.push(`  ${line}`);
  return lines;
}

export function formatSend(saved: { simulated?: string; commit?: string; browser?: string; result: SendResult }): string[] {
  const r = saved.result;
  const l = r.loop;
  const hasOrderProblem = l.missing > 0 || l.extra > 0;
  const [Start = 0, Continue = 0, Stop = 0] = r.transportSent;
  const [startHeard = 0, continueHeard = 0, stopHeard = 0] = r.transportHeard;
  return [
    ...head(saved),
    `Spike S3, send a clock: output "${r.ports.output}", input "${r.ports.input}", engine at ${r.config.bpm} BPM, ${r.durationS.toFixed(0)} s run started ${r.startedAt}, tab hidden ${r.hiddenS.toFixed(1)} s`,
    `${where(saved)}${where(saved) ? "; " : ""}${r.environment.userAgent}, ${r.environment.platform}; audio ${r.environment.audioSampleRate} Hz, base latency ${r.environment.baseLatencyMs.toFixed(1)} ms, output latency ${r.environment.outputLatencyMs.toFixed(1)} ms; lookahead ${r.config.lookaheadMs} ms`,
    `Clock pulses: sent ${l.sent}, heard ${l.received}, matched ${l.matched}, missing ${l.missing}, extra ${l.extra}${hasOrderProblem ? " (the pairing is by order, so a pulse lost in the middle shifts every pair after it: the latency and interval figures below cannot be trusted)" : ""}`,
    `Transport messages: Start ${Start} sent, ${startHeard} heard; Continue ${Continue} sent, ${continueHeard} heard; Stop ${Stop} sent, ${stopHeard} heard`,
    spreadLine("latency (heard stamp less the time it was sent for; the port's, with the page's clock map in it)", l.latencyMs),
    spreadLine("interval error (heard interval less the interval sent; the jitter a listener hears)", l.intervalErrorMs),
    `  scheduling margin of the pulses: smallest ${ms(r.marginMs.min)}, median ${ms(r.marginMs.p50)}, late ${r.marginMs.late}; scheduler: sent ${r.scheduler.sent}, real-time ${r.scheduler.realtime}, raised for order ${r.scheduler.raised}, invalid ${r.scheduler.invalid}; worklet errors ${r.worklet.errors.length}`,
    "The clock as it came back, read as a clock the way a followed one is:",
    ...intervalLines(r.heardAsClock).map((line) => `  ${line}`),
  ];
}

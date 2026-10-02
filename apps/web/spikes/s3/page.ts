// Spike S3's page: the clock between this app and another program (Ableton Live), in both directions, on a real Web MIDI port.
//
//   follow: the page listens on one MIDI input for a clock somebody else sends. The real follower (src/follower.ts) steers the real engine
//           with it, as the app does, and the page records every 0xF8 (the browser's timestamp for it and the page's own time when the
//           handler ran), the Start, Continue and Stop, and what the follower says about the phase every 250 ms. `analyse.ts` reads that.
//   send:   the engine is the clock master and its clock goes to one MIDI output. Joined to an input by a loopback port, the page hears it
//           come back. This is the app's clock as a port delivers it: the half of S3 that needs no Live.
//
// The page is for a person to run (a real port) and for `run.ts` to drive with a stand-in. What each run says about itself is in the saved
// file; the numbers are computed from the raw records, so they can be read again with other thresholds (`report.ts`).
import { layoutFromControls } from "../../src/abi.ts";
import { Follower, FOLLOWER } from "../../src/follower.ts";
import { Host } from "../../src/host.ts";
import { InputPath, type InputStats } from "../../src/midi-in.ts";
import { ContextTimeMap, MidiScheduler, type SchedulerStats } from "../../src/midi-out.ts";
import { LOOKAHEAD_MS } from "../../src/settings.ts";
import { wireMidi } from "../../src/wire.ts";
import { analyseFollow, analyseLoop, type FollowAnalysis, type FollowMessage, type FollowSample, type LoopAnalysis, type SavedPulse } from "./analyse.ts";

export interface PortInfo {
  id: string;
  name: string;
}

export interface Environment {
  userAgent: string;
  platform: string;
  audioSampleRate: number;
  baseLatencyMs: number;
  outputLatencyMs: number;
}

export interface Visibility {
  atS: number;
  state: string;
}

export interface FollowConfig {
  minutes: number;
  inputId: string;
  /** The follower's lookahead: the scheduler's, which this page has none of. The phase is measured against it, so it is a shift and not a loss. */
  lookaheadMs: number;
  offsetMs: number;
  /** The tempo the sender was set to, if the person says: the tempo the follower read is compared with it. */
  nominalBpm: number | null;
}

export interface FollowResult {
  schema: "wenge.s3.follow/1";
  startedAt: string;
  config: FollowConfig;
  environment: Environment;
  port: string;
  durationS: number;
  hiddenS: number;
  visibility: Visibility[];
  input: InputStats;
  worklet: { errors: string[] };
  tuning: typeof FOLLOWER;
  raw: { pulses: SavedPulse[]; samples: FollowSample[]; messages: FollowMessage[] };
  analysis: { all: FollowAnalysis; visible: FollowAnalysis; hidden: FollowAnalysis };
}

export interface SendConfig {
  minutes: number;
  bpm: number;
  outputId: string;
  inputId: string;
  lookaheadMs: number;
}

export interface SendResult {
  schema: "wenge.s3.send/1";
  startedAt: string;
  config: SendConfig;
  environment: Environment;
  ports: { output: string; input: string };
  durationS: number;
  hiddenS: number;
  visibility: Visibility[];
  scheduler: SchedulerStats;
  worklet: { errors: string[] };
  /** Real-time messages other than the clock, sent and heard: [Start, Continue, Stop]. */
  transportSent: number[];
  transportHeard: number[];
  /** Each clock pulse's scheduling margin (target less the time of the call), ms: how early the engine's pulses reach the scheduler. */
  marginMs: { min: number; p50: number; late: number };
  loop: LoopAnalysis;
  /** The clock as it came back, read as a followed clock is (the clock domain, the periods, the tempo, the drift). */
  heardAsClock: FollowAnalysis;
  raw: { sent: { target: number; audioTime: number; hidden: boolean }[]; heard: SavedPulse[] };
}

export interface S3 {
  access(): Promise<{ outputs: PortInfo[]; inputs: PortInfo[] }>;
  follow(config: FollowConfig, progress?: (line: string) => void): Promise<FollowResult>;
  send(config: SendConfig, progress?: (line: string) => void): Promise<SendResult>;
  abort(): void;
}

/** The sampling period of the follower's status, ms. A hidden tab is allowed to run a timer slower than this, and the samples show it. */
const SAMPLE_MS = 250;

interface Control {
  n: number;
  id: string;
}

let midi: MIDIAccess | null = null;
let aborted = false;

const sleep = (ms: number): Promise<void> => new Promise((ok) => setTimeout(ok, ms));
const round3 = (x: number): number => Math.round(x * 1000) / 1000;
const ports = (m: { values(): Iterable<{ id: string; name?: string | null }> }): PortInfo[] => [...m.values()].map((p) => ({ id: p.id, name: p.name ?? p.id }));
const hiddenNow = (): boolean => document.visibilityState === "hidden";

function environment(host: Host): Environment {
  return {
    userAgent: navigator.userAgent,
    platform: navigator.platform,
    audioSampleRate: host.context.sampleRate,
    baseLatencyMs: host.context.baseLatency * 1000,
    outputLatencyMs: host.context.outputLatency * 1000,
  };
}

async function startHost(): Promise<Host> {
  const controls = (await (await fetch("/contracts/controls.json")).json()) as { controls: Control[] };
  const wasm = await (await fetch("/dist/octoweb.wasm")).arrayBuffer();
  return Host.start({ wasm, layout: layoutFromControls(controls), workletUrl: "/dist/src/worklet.js", seed: 0n });
}

/** Keeps a record of how long the tab was hidden and each change, from `t0` (page time, ms). */
function watchVisibility(t0: number): { visibility: Visibility[]; hiddenMs: () => number; stop: () => void } {
  const visibility: Visibility[] = [{ atS: 0, state: document.visibilityState }];
  let since: number | null = hiddenNow() ? t0 : null;
  let total = 0;
  const onChange = (): void => {
    const now = performance.now();
    visibility.push({ atS: (now - t0) / 1000, state: document.visibilityState });
    if (hiddenNow()) since = now;
    else if (since !== null) {
      total += now - since;
      since = null;
    }
  };
  document.addEventListener("visibilitychange", onChange);
  return { visibility, hiddenMs: () => total + (since !== null ? performance.now() - since : 0), stop: () => document.removeEventListener("visibilitychange", onChange) };
}

const s3: S3 = {
  async access() {
    if (!("requestMIDIAccess" in navigator)) throw new Error("this browser has no Web MIDI (Chrome, Edge and Opera have it; Safari does not)");
    midi = await navigator.requestMIDIAccess();
    return { outputs: ports(midi.outputs), inputs: ports(midi.inputs) };
  },

  abort() {
    aborted = true;
  },

  async follow(config, progress = () => undefined) {
    if (!midi) throw new Error("ask for MIDI access first");
    const input = midi.inputs.get(config.inputId);
    if (!input) throw new Error("the chosen input is no longer there");
    aborted = false;

    const host = await startHost();
    const time = new ContextTimeMap(host.context, performance);
    const errors: string[] = [];
    host.onError = (m) => errors.push(m);
    const follower = new Follower({ setTempo: (bpm) => host.setTempo(bpm), transport: (play) => host.transport(play) }, { lookaheadMs: config.lookaheadMs, offsetMs: config.offsetMs });
    // Where the engine is, at the page time that holds for: the follower's comparator, as the app feeds it (pages/app.ts).
    host.onPanel = (p) => {
      time.refresh();
      follower.onPosition({ ticks: p.position, pageTimeMs: time.toPage(p.positionFrame / host.sampleRate) });
    };

    const pulses: SavedPulse[] = [];
    const messages: FollowMessage[] = [];
    const samples: FollowSample[] = [];
    // The app's own input path decodes what comes in, so the page hears what the app would.
    const path = new InputPath();
    path.onMessage = (message, stamp) => {
      if (message.kind !== "realtime") return;
      const seen = performance.now();
      if (message.message === "clock") pulses.push([round3(stamp), round3(seen)]);
      else messages.push({ name: message.message, stamp: round3(stamp), seen: round3(seen) });
      follower.feed(message.message, stamp);
    };
    const t0 = performance.now();
    const watch = watchVisibility(t0);
    path.select(input);
    const sampling = setInterval(() => {
      const at = performance.now();
      const s = follower.status(at);
      samples.push({ at: round3(at), phase: s.phase, bpm: s.bpm, phaseMs: s.phaseMs === null ? null : round3(s.phaseMs), jitterMs: round3(s.jitterMs), pulses: s.pulses, running: s.running, hidden: hiddenNow() });
    }, SAMPLE_MS);

    const end = t0 + config.minutes * 60_000;
    while (performance.now() < end && !aborted) {
      await sleep(1000);
      const s = follower.status(performance.now());
      const phase = s.phaseMs === null ? "no phase" : `phase ${s.phaseMs.toFixed(1)} ms`;
      const hint = pulses.length === 0 ? " (no clock yet: is the sender's clock going to this input?)" : !s.running ? " (a clock, no Start: press Play on the sender)" : "";
      progress(`${Math.round((performance.now() - t0) / 1000)} s: ${pulses.length} pulses, ${s.bpm === null ? "no tempo" : `${s.bpm.toFixed(2)} BPM`}, ${s.phase}, ${phase}, tab ${document.visibilityState}${hint}`);
    }
    clearInterval(sampling);
    // `select` starts the counts again, so they are copied first.
    const counts = structuredClone(path.stats);
    path.select(null);
    watch.stop();
    host.transport(false);
    const endNow = performance.now();
    const raw = { pulses, samples, messages };
    const asRaw = { pulses: pulses.map(([stamp, seen]) => ({ stamp, seen })), samples, messages };
    const nominal = config.nominalBpm === null ? {} : { nominalBpm: config.nominalBpm };
    const result: FollowResult = {
      schema: "wenge.s3.follow/1",
      startedAt: new Date(Date.now() - (endNow - t0)).toISOString(),
      config,
      environment: environment(host),
      port: input.name ?? input.id,
      durationS: (endNow - t0) / 1000,
      hiddenS: watch.hiddenMs() / 1000,
      visibility: watch.visibility,
      input: counts,
      worklet: { errors },
      tuning: FOLLOWER,
      raw,
      analysis: { all: analyseFollow(asRaw, nominal), visible: analyseFollow(asRaw, { ...nominal, hidden: false }), hidden: analyseFollow(asRaw, { ...nominal, hidden: true }) },
    };
    await host.close();
    return result;
  },

  async send(config, progress = () => undefined) {
    if (!midi) throw new Error("ask for MIDI access first");
    const output = midi.outputs.get(config.outputId);
    const input = midi.inputs.get(config.inputId);
    if (!output || !input) throw new Error("the chosen port is no longer there");
    aborted = false;

    const host = await startHost();
    const time = new ContextTimeMap(host.context, performance);
    const scheduler = new MidiScheduler(time, config.lookaheadMs);
    scheduler.setOutput(1, output);
    wireMidi(host, scheduler, time);
    const errors: string[] = [];
    host.onError = (m) => errors.push(m);
    const sent: SendResult["raw"]["sent"] = [];
    const margins: number[] = [];
    const transportSent = [0, 0, 0];
    const slot: Record<number, number> = { 0xfa: 0, 0xfb: 1, 0xfc: 2 };
    scheduler.onSend = (info) => {
      const status = info.bytes[0] ?? 0;
      if (info.port !== 0 || info.bytes.length !== 1) return;
      if (status === 0xf8) {
        sent.push({ target: round3(info.timestamp ?? performance.now()), audioTime: info.audioTime, hidden: hiddenNow() });
        margins.push(info.marginMs);
      } else if (status in slot) transportSent[slot[status]!]!++;
    };
    const heard: SavedPulse[] = [];
    const transportHeard = [0, 0, 0];
    input.onmidimessage = (e) => {
      const data = e.data;
      if (!data || data.length !== 1) return;
      if (data[0] === 0xf8) heard.push([round3(e.timeStamp), round3(performance.now())]);
      else if (data[0]! in slot) transportHeard[slot[data[0]!]!]!++;
    };

    const t0 = performance.now();
    const watch = watchVisibility(t0);
    // The engine is the master: the clock and Start go to the output. Tempo first, so the first pulse is at it.
    host.setClock(true);
    host.setTempo(config.bpm);
    host.transport(true);
    const end = t0 + config.minutes * 60_000;
    while (performance.now() < end && !aborted) {
      await sleep(1000);
      progress(`${Math.round((performance.now() - t0) / 1000)} s: sent ${sent.length} pulses, heard ${heard.length}, tab ${document.visibilityState}`);
    }
    host.transport(false);
    await sleep(1500); // the Stop and whatever is queued arrive
    input.onmidimessage = null;
    watch.stop();
    const endNow = performance.now();
    const sorted = [...margins].sort((a, b) => a - b);
    const result: SendResult = {
      schema: "wenge.s3.send/1",
      startedAt: new Date(Date.now() - (endNow - t0)).toISOString(),
      config,
      environment: environment(host),
      ports: { output: output.name ?? output.id, input: input.name ?? input.id },
      durationS: (endNow - t0) / 1000,
      hiddenS: watch.hiddenMs() / 1000,
      visibility: watch.visibility,
      scheduler: { ...scheduler.stats },
      worklet: { errors },
      transportSent,
      transportHeard,
      marginMs: { min: sorted[0] ?? 0, p50: sorted[Math.floor(sorted.length / 2)] ?? 0, late: margins.filter((m) => m <= 0).length },
      loop: analyseLoop(sent, heard.map(([stamp]) => ({ at: stamp }))),
      heardAsClock: analyseFollow({ pulses: heard.map(([stamp, seen]) => ({ stamp, seen })), samples: [], messages: [] }),
      raw: { sent, heard },
    };
    await host.close();
    return result;
  },
};

(window as unknown as { s3: S3 }).s3 = s3;

// ---- the page a person uses ----
const el = <T extends HTMLElement>(id: string): T => document.getElementById(id) as T;
if (document.getElementById("grant")) {
  let last: { name: string; json: string } | null = null;
  const status = el<HTMLParagraphElement>("status");
  const fill = (select: HTMLSelectElement, list: PortInfo[]): void => {
    select.replaceChildren(...list.map((p) => Object.assign(document.createElement("option"), { value: p.id, textContent: p.name })));
  };
  const number = (id: string): number => Number(el<HTMLInputElement>(id).value);
  const buttons = (busy: boolean): void => {
    for (const id of ["follow-start", "send-start"]) el<HTMLButtonElement>(id).disabled = busy;
    el<HTMLButtonElement>("stop").disabled = !busy;
    if (busy) el<HTMLButtonElement>("save").disabled = true;
  };
  const finish = (name: string, result: FollowResult | SendResult): void => {
    last = { name: `${name}-${result.startedAt.replace(/[:.]/g, "-")}.json`, json: JSON.stringify(result) };
    status.textContent = "Done. Save the result as a file and send it back (spikes/s3/report.ts reads it).";
    // The page shows a summary, not the raw records: 30 minutes of pulses are tens of thousands of lines.
    const summary = result.schema === "wenge.s3.follow/1" ? { ...result, raw: `${result.raw.pulses.length} pulses, ${result.raw.samples.length} samples, ${result.raw.messages.length} messages (in the saved file)` } : { ...result, raw: `${result.raw.sent.length} sent, ${result.raw.heard.length} heard (in the saved file)` };
    el("result").hidden = false;
    el("result").textContent = JSON.stringify(summary, null, 1);
    el<HTMLButtonElement>("save").disabled = false;
  };
  const guarded = (name: string, work: () => Promise<FollowResult | SendResult>): void => {
    void (async () => {
      buttons(true);
      try {
        finish(name, await work());
      } catch (e) {
        status.textContent = `The run failed: ${String(e)}`;
      }
      buttons(false);
    })();
  };
  el<HTMLButtonElement>("grant").addEventListener("click", () => {
    void (async () => {
      try {
        const found = await s3.access();
        for (const id of ["follow-input", "send-input"]) fill(el(id), found.inputs);
        fill(el("send-output"), found.outputs);
        el("form").hidden = false;
        status.textContent = `${found.outputs.length} outputs and ${found.inputs.length} inputs found.`;
      } catch (e) {
        status.textContent = String(e);
      }
    })();
  });
  el<HTMLButtonElement>("stop").addEventListener("click", () => s3.abort());
  el<HTMLButtonElement>("follow-start").addEventListener("click", () =>
    guarded("s3-follow", () =>
      s3.follow(
        {
          minutes: number("follow-minutes"),
          inputId: el<HTMLSelectElement>("follow-input").value,
          lookaheadMs: LOOKAHEAD_MS.default,
          offsetMs: number("follow-offset"),
          nominalBpm: el<HTMLInputElement>("follow-nominal").value === "" ? null : number("follow-nominal"),
        },
        (line) => (status.textContent = line),
      ),
    ),
  );
  el<HTMLButtonElement>("send-start").addEventListener("click", () =>
    guarded("s3-send", () =>
      s3.send(
        {
          minutes: number("send-minutes"),
          bpm: number("send-bpm"),
          outputId: el<HTMLSelectElement>("send-output").value,
          inputId: el<HTMLSelectElement>("send-input").value,
          lookaheadMs: LOOKAHEAD_MS.default,
        },
        (line) => (status.textContent = line),
      ),
    ),
  );
  el<HTMLButtonElement>("save").addEventListener("click", () => {
    if (!last) return;
    const a = Object.assign(document.createElement("a"), { href: URL.createObjectURL(new Blob([last.json], { type: "application/json" })), download: last.name });
    a.click();
    URL.revokeObjectURL(a.href);
  });
}

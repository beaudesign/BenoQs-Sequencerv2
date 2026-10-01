// Spike S1's page: the real engine, worklet and scheduler play every step of one track to one MIDI output while
// one MIDI input listens for them to come back. `analyse.ts` turns the sends and receipts into numbers. The page is
// for a person to run (a loopback port) and for `run.ts` to drive with a stand-in port.
import { layoutFromControls } from "../../src/abi.ts";
import { Host } from "../../src/host.ts";
import { ContextTimeMap, MidiScheduler, type SchedulerStats } from "../../src/midi-out.ts";
import { wireMidi } from "../../src/wire.ts";
import { analyse, type Analysis, type Receipt, type Sent } from "./analyse.ts";

export interface PortInfo {
  id: string;
  name: string;
}

export interface S1Config {
  minutes: number;
  lookaheadMs: number;
  alpha: number;
  outputId: string;
  inputId: string;
  bpm: number;
}

export interface S1Result {
  schema: "wenge.s1/1";
  startedAt: string;
  config: S1Config;
  environment: { userAgent: string; platform: string; audioSampleRate: number; baseLatencyMs: number; outputLatencyMs: number };
  ports: { output: string; input: string };
  durationS: number;
  /** The median time between one Note On and the next as the engine made them, ms: the step length actually played. */
  stepMs: number;
  /** Seconds the tab was hidden, and each change of state, in seconds from the start. */
  hiddenS: number;
  visibility: { atS: number; state: string }[];
  scheduler: SchedulerStats;
  worklet: { errors: string[] };
  /** The stricter reading of "no event was late": the worst margin, less the lookahead, is what the lookahead could be cut by. */
  headroomAtZeroLookaheadMs: number;
  all: Analysis;
  visible: Analysis;
  hidden: Analysis;
}

export interface S1 {
  access(): Promise<{ outputs: PortInfo[]; inputs: PortInfo[] }>;
  run(config: S1Config, progress?: (line: string) => void): Promise<S1Result>;
  abort(): void;
}

interface Control {
  n: number;
  id: string;
}

let midi: MIDIAccess | null = null;
let aborted = false;

const sleep = (ms: number): Promise<void> => new Promise((ok) => setTimeout(ok, ms));
const ports = (m: Map<string, { id: string; name?: string | null }> | { values(): Iterable<{ id: string; name?: string | null }> }): PortInfo[] =>
  [...(m as { values(): Iterable<{ id: string; name?: string | null }> }).values()].map((p) => ({ id: p.id, name: p.name ?? p.id }));

function medianStepMs(sends: Sent[]): number {
  const onsets = sends.filter((s) => ((s.bytes[0] ?? 0) & 0xf0) === 0x90).map((s) => s.audioTime).sort((a, b) => a - b);
  const gaps = onsets.slice(1).map((t, i) => (t - onsets[i]!) * 1000).sort((a, b) => a - b);
  return gaps[Math.floor(gaps.length / 2)] ?? 0;
}

const s1: S1 = {
  async access() {
    if (!("requestMIDIAccess" in navigator)) throw new Error("this browser has no Web MIDI (Chrome, Edge and Opera have it; Safari does not)");
    midi = await navigator.requestMIDIAccess();
    return { outputs: ports(midi.outputs), inputs: ports(midi.inputs) };
  },

  abort() {
    aborted = true;
  },

  async run(config, progress = () => undefined) {
    if (!midi) throw new Error("ask for MIDI access first");
    const output = midi.outputs.get(config.outputId);
    const input = midi.inputs.get(config.inputId);
    if (!output || !input) throw new Error("the chosen port is no longer there");
    aborted = false;

    const controls = (await (await fetch("/contracts/controls.json")).json()) as { controls: Control[] };
    const numberOf = new Map(controls.controls.map((c) => [c.id, c.n]));
    const wasm = await (await fetch("/dist/octoweb.wasm")).arrayBuffer();
    const host = await Host.start({ wasm, layout: layoutFromControls(controls), workletUrl: "/dist/src/worklet.js", seed: 0n });
    const map = new ContextTimeMap(host.context, performance, config.alpha);
    const scheduler = new MidiScheduler(map, config.lookaheadMs);
    scheduler.setOutput(1, output);
    scheduler.setOutput(2, output);
    const sends: Sent[] = [];
    const receipts: Receipt[] = [];
    const errors: string[] = [];
    scheduler.onSend = (i) => sends.push({ bytes: i.bytes, target: i.timestamp ?? performance.now(), audioTime: i.audioTime, marginMs: i.marginMs, hidden: document.visibilityState === "hidden" });
    wireMidi(host, scheduler, map);
    host.onError = (m) => errors.push(m);
    input.onmidimessage = (e) => {
      if (e.data) receipts.push({ bytes: [...e.data], at: e.timeStamp });
    };

    const t0 = performance.now();
    const visibility: S1Result["visibility"] = [{ atS: 0, state: document.visibilityState }];
    let hiddenSince: number | null = document.visibilityState === "hidden" ? t0 : null;
    let hiddenMs = 0;
    const onVisibility = (): void => {
      const now = performance.now();
      visibility.push({ atS: (now - t0) / 1000, state: document.visibilityState });
      if (document.visibilityState === "hidden") hiddenSince = now;
      else if (hiddenSince !== null) {
        hiddenMs += now - hiddenSince;
        hiddenSince = null;
      }
    };
    document.addEventListener("visibilitychange", onVisibility);

    // Sixteenth notes: every step of the first track, at the chosen tempo.
    for (let step = 1; step <= 16; step++) {
      const n = numberOf.get(`matrix.r0.c${step}`);
      if (n === undefined) throw new Error(`no control matrix.r0.c${step}`);
      host.press(n);
      host.release(n);
    }
    host.setTempo(config.bpm);
    host.transport(true);

    const end = t0 + config.minutes * 60_000;
    while (performance.now() < end && !aborted) {
      await sleep(1000);
      progress(`${Math.round((performance.now() - t0) / 1000)} s: sent ${sends.length}, heard ${receipts.length}, tab ${document.visibilityState}`);
    }
    host.transport(false);
    await sleep(1500); // the flush and anything queued arrive
    input.onmidimessage = null;
    document.removeEventListener("visibilitychange", onVisibility);
    const endNow = performance.now();
    if (hiddenSince !== null) hiddenMs += endNow - hiddenSince;
    const result: S1Result = {
      schema: "wenge.s1/1",
      startedAt: new Date(Date.now() - (endNow - t0)).toISOString(),
      config,
      environment: {
        userAgent: navigator.userAgent,
        platform: navigator.platform,
        audioSampleRate: host.context.sampleRate,
        baseLatencyMs: host.context.baseLatency * 1000,
        outputLatencyMs: host.context.outputLatency * 1000,
      },
      ports: { output: output.name ?? output.id, input: input.name ?? input.id },
      durationS: (endNow - t0) / 1000,
      stepMs: medianStepMs(sends),
      hiddenS: hiddenMs / 1000,
      visibility,
      scheduler: { ...scheduler.stats },
      worklet: { errors },
      headroomAtZeroLookaheadMs: analyse(sends, receipts).marginMs.min - config.lookaheadMs,
      all: analyse(sends, receipts),
      visible: analyse(sends, receipts, (s) => !s.hidden),
      hidden: analyse(sends, receipts, (s) => s.hidden),
    };
    await host.close();
    return result;
  },
};

(window as unknown as { s1: S1 }).s1 = s1;

// ---- the page a person uses ----
const el = <T extends HTMLElement>(id: string): T => document.getElementById(id) as T;
if (document.getElementById("grant")) {
  let last: S1Result | null = null;
  const status = el<HTMLParagraphElement>("status");
  const fill = (select: HTMLSelectElement, list: PortInfo[]): void => {
    select.replaceChildren(...list.map((p) => Object.assign(document.createElement("option"), { value: p.id, textContent: p.name })));
  };
  el<HTMLButtonElement>("grant").addEventListener("click", () => {
    void (async () => {
      try {
        const found = await s1.access();
        fill(el("output"), found.outputs);
        fill(el("input"), found.inputs);
        el("form").hidden = false;
        status.textContent = `${found.outputs.length} outputs and ${found.inputs.length} inputs found.`;
      } catch (e) {
        status.textContent = String(e);
      }
    })();
  });
  el<HTMLButtonElement>("stop").addEventListener("click", () => s1.abort());
  el<HTMLButtonElement>("start").addEventListener("click", () => {
    void (async () => {
      el<HTMLButtonElement>("start").disabled = true;
      el<HTMLButtonElement>("stop").disabled = false;
      el<HTMLButtonElement>("save").disabled = true;
      try {
        last = await s1.run(
          {
            minutes: Number(el<HTMLInputElement>("minutes").value),
            lookaheadMs: Number(el<HTMLInputElement>("lookahead").value),
            alpha: Number(el<HTMLInputElement>("alpha").value),
            outputId: el<HTMLSelectElement>("output").value,
            inputId: el<HTMLSelectElement>("input").value,
            bpm: Number(el<HTMLInputElement>("bpm").value),
          },
          (line) => (status.textContent = line),
        );
        status.textContent = "Done. Save the result as a file and send it back, or paste the text below.";
        el("result").hidden = false;
        el("result").textContent = JSON.stringify(last, null, 1);
        el<HTMLButtonElement>("save").disabled = false;
      } catch (e) {
        status.textContent = `The run failed: ${String(e)}`;
      }
      el<HTMLButtonElement>("start").disabled = false;
      el<HTMLButtonElement>("stop").disabled = true;
    })();
  });
  el<HTMLButtonElement>("save").addEventListener("click", () => {
    if (!last) return;
    const stamp = last.startedAt.replace(/[:.]/g, "-");
    const a = Object.assign(document.createElement("a"), {
      href: URL.createObjectURL(new Blob([JSON.stringify(last, null, 1)], { type: "application/json" })),
      download: `s1-${stamp}.json`,
    });
    a.click();
    URL.revokeObjectURL(a.href);
  });
}

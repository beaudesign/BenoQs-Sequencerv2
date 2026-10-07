// The app: the panel, the strip, and between them the engine in its worklet, two Web MIDI outputs (one for each port) and one input,
// and the clock: the engine as the master (its clock goes out with its notes), or the page as a slave (the input's clock steers the engine
// from outside: src/follower.ts), with or without the echo.
// Everything it needs is fetched from the folders the server serves: the control inventory and the tokens (contracts),
// the layout, the module. `Play` is a button because a browser keeps audio silent until the page has been used: the first press starts the
// engine and, unless the address says `?bare`, loads the demo pattern, turns the sound on and plays (specs/SPEC-0002/p6-runs-when-opened.md).
import { LED_COUNT, TRACK_ATTR, layoutFromControls } from "../src/abi.ts";
import { ChaseLight, composeChase, matrixMap } from "../src/chase.ts";
import { CLOCK_DEFAULT, decideClock, describeClock, isClockMode, type ClockDecision, type ClockWorld } from "../src/clock-state.ts";
import { loadDemo } from "../src/demo.ts";
import { Follower } from "../src/follower.ts";
import { Host } from "../src/host.ts";
import { parseLayout, place, type ControlsDoc } from "../src/layout.ts";
import { InputPath, REALTIME_STATUS, describeCount, describeInput } from "../src/midi-in.ts";
import { describeOutputs, openMidi, type Midi, type OutputChoice } from "../src/midi-access.ts";
import { ContextTimeMap, MidiScheduler } from "../src/midi-out.ts";
import { drawPanel } from "../src/panel.ts";
import { Presses } from "../src/press.ts";
import { describeRoutes, parseRoute } from "../src/route.ts";
import { CLOCK_DETAIL_REFRESH_MS, CLOCK_OFFSET_MS, INPUT_COUNT_REFRESH_MS, LOOKAHEAD_MS, OUTPUT_PORTS, TEMPO_BPM } from "../src/settings.ts";
import { bindStrip } from "../src/strip.ts";
import { parseTokens } from "../src/tokens.ts";
import { wireMidi } from "../src/wire.ts";

async function json(path: string): Promise<unknown> {
  const r = await fetch(path);
  if (!r.ok) throw new Error(`${path}: ${r.status}`);
  return r.json();
}

type EngineState = "idle" | "starting" | "running" | "failed";

async function main(): Promise<void> {
  const main = document.querySelector("main")!;
  const strip = bindStrip(document);
  // The one place that reads the address: the `?route=` stand-in for port 2 (src/route.ts; README, "The route hook").
  const route = parseRoute(location.search);
  // `?bare` is the page the P3 to P5 tests drive: Play only starts the engine, and nothing is loaded, played or heard until the test says so.
  // It is a test hook like `?route=`, and read here, once.
  const bare = new URLSearchParams(location.search).has("bare");
  const setState = (s: EngineState): void => void main.setAttribute("data-engine", s);

  const [controls, tokensJson, layoutJson] = await Promise.all([
    json("/contracts/controls.json") as Promise<ControlsDoc>,
    json("/contracts/design.tokens.json"),
    json("/layout/panel.layout.json"),
  ]);
  const tokens = parseTokens(tokensJson);
  const geometry = place(parseLayout(layoutJson, controls), controls, tokens);
  const numberOf = (id: string): number => {
    const c = controls.controls.find((x) => x.id === id);
    if (!c) throw new Error(`controls.json has no ${id}`);
    return c.n;
  };
  // The Play key: the engine's `Play` command, until the transport workflow is built from the manual (ADR-0008 decision 4).
  const playKey = numberOf("transport.play");
  const stepMap = matrixMap(controls.controls);

  let host: Host | null = null;
  let starting = false;
  /** The one Play, for the strip's button and the panel's own key: it starts the engine if there is none, and then it is Play and Stop. */
  const playOrStop = (): void => {
    if (host) host.transport(main.getAttribute("data-transport") !== "playing");
    else void startEngine();
  };
  const down = new Set<number>();
  const presses = new Presses({
    press(n) {
      if (!host) {
        if (n === playKey && !bare) playOrStop();
        else strip.say("Press Play first, then the keys work.");
        return;
      }
      down.add(n);
      if (n === playKey) host.transport(true);
      else host.press(n);
    },
    release(n) {
      if (!host || !down.delete(n)) return;
      if (n !== playKey) host.release(n);
    },
  });

  const panel = drawPanel(document, geometry, tokens, presses);
  document.querySelector("#panel-frame")!.append(panel.svg);
  addEventListener("blur", () => presses.releaseAll());

  strip.lookahead.min = String(LOOKAHEAD_MS.min);
  strip.lookahead.max = String(LOOKAHEAD_MS.max);
  strip.lookahead.value = String(LOOKAHEAD_MS.default);
  strip.clockOffset.min = String(CLOCK_OFFSET_MS.min);
  strip.clockOffset.max = String(CLOCK_OFFSET_MS.max);
  strip.clockOffset.value = String(CLOCK_OFFSET_MS.default);

  let scheduler: MidiScheduler | null = null;
  let midi: Midi | null = null;

  // The clock. What was asked for is the select's value; what is in force is `decision`, which refuses what cannot work.
  let follower: Follower | null = null;
  let masterOn = false;
  let outputDevices = 0;
  let inputName: string | null = null;
  let decision: ClockDecision = decideClock("off", { started: false, midi: "unknown", outputs: 0, input: null });
  let shownClock = "";
  let shownDetail = "";
  const clockWorld = (): ClockWorld => ({ started: host !== null, midi: midi?.status ?? "unknown", outputs: outputDevices, input: inputName });
  const offsetNow = (): number => {
    const ms = Number(strip.clockOffset.value);
    return Number.isFinite(ms) && ms >= CLOCK_OFFSET_MS.min && ms <= CLOCK_OFFSET_MS.max ? ms : CLOCK_OFFSET_MS.default;
  };
  const drawClock = (): void => {
    const status = follower && decision.follow ? follower.status(performance.now()) : null;
    const text = describeClock(decision, clockWorld(), status);
    if (text.status !== shownClock) strip.sayClock((shownClock = text.status));
    if (text.detail !== shownDetail) strip.sayClockDetail((shownDetail = text.detail));
  };
  setInterval(drawClock, CLOCK_DETAIL_REFRESH_MS);

  const inputPath = new InputPath();
  const drawInput = (): void => {
    if (midi) strip.sayInput(describeInput(inputName, midi.inputMessage, decision.follow));
  };
  /** Puts the clock state in force: the engine as master or not, a follower or none, the echo or none, and the words. */
  const applyClock = (): void => {
    const asked = isClockMode(strip.clock.value) ? strip.clock.value : CLOCK_DEFAULT;
    decision = decideClock(asked, clockWorld());
    if (decision.refused) strip.clock.value = CLOCK_DEFAULT; // a refusal puts the control back to off, and the sentence says why
    if (host && decision.master !== masterOn) host.setClock((masterOn = decision.master));
    if (!decision.follow) follower = null;
    else if (host && !follower) {
      const engine = host;
      follower = new Follower(
        { setTempo: (bpm) => engine.setTempo(bpm), transport: (play) => engine.transport(play) },
        { lookaheadMs: scheduler?.lookaheadMs ?? LOOKAHEAD_MS.default, offsetMs: offsetNow() },
      );
    }
    drawInput();
    drawClock();
  };
  // What the chosen input sends: the clock steers the engine (slave), and is passed on (echo). Everything else is counted and not used yet.
  inputPath.onMessage = (message, timeStamp) => {
    if (message.kind !== "realtime") return;
    if (decision.follow) follower?.feed(message.message, timeStamp);
    if (decision.echo) scheduler?.passThrough(REALTIME_STATUS[message.message]);
  };
  const applyOutputs = (): void => {
    if (!scheduler || !midi) return;
    const chosen: { 1: OutputChoice | null; 2: OutputChoice | null } = { 1: null, 2: null };
    for (const port of OUTPUT_PORTS) {
      const device = midi.outputs().find((o) => o.id === strip.outputs[port].value) ?? null;
      scheduler.setOutput(port, device ? device.output : null);
      chosen[port] = device;
    }
    outputDevices = new Set([chosen[1]?.id, chosen[2]?.id].filter((id) => id !== undefined)).size;
    strip.sayMidi(describeOutputs(chosen, midi.message));
    applyClock();
  };
  let counted = "";
  const drawCount = (): void => {
    const text = inputPath.selected ? describeCount(inputPath.stats) : "";
    if (text !== counted) strip.sayInputCount((counted = text));
  };
  const applyInput = (): void => {
    if (!midi) return;
    const device = midi.inputs().find((i) => i.id === strip.input.value) ?? null;
    inputPath.select(device ? device.input : null);
    inputName = device ? device.name : null;
    follower = null; // a different sender is a new clock: nothing of the last one's tempo or count is kept
    applyClock();
    drawCount();
  };
  setInterval(drawCount, INPUT_COUNT_REFRESH_MS);
  const applyLookahead = (): void => {
    const ms = Number(strip.lookahead.value);
    if (!scheduler || !Number.isFinite(ms) || ms < LOOKAHEAD_MS.min || ms > LOOKAHEAD_MS.max) return;
    scheduler.lookaheadMs = ms;
    follower?.setLookahead(ms); // the engine has to be ahead of the sender's grid by the lookahead the scheduler adds to everything it plays
  };
  const applyOffset = (): void => follower?.setOffset(offsetNow());
  for (const port of OUTPUT_PORTS) strip.outputs[port].addEventListener("change", applyOutputs);
  strip.input.addEventListener("change", applyInput);
  strip.lookahead.addEventListener("change", applyLookahead);
  strip.clock.addEventListener("change", applyClock);
  strip.clockOffset.addEventListener("change", applyOffset);
  applyClock();

  // The tempo. A value the engine would refuse (or no number) is put back; a good one goes to the engine at once, or when there is one.
  let tempoNow: number = TEMPO_BPM.default;
  strip.tempo.min = String(TEMPO_BPM.min);
  strip.tempo.max = String(TEMPO_BPM.max);
  strip.tempo.value = String(tempoNow);
  strip.tempo.addEventListener("change", () => {
    const bpm = Number(strip.tempo.value);
    if (strip.tempo.value.trim() !== "" && Number.isFinite(bpm) && bpm >= TEMPO_BPM.min && bpm <= TEMPO_BPM.max) {
      tempoNow = Math.round(bpm);
      host?.setTempo(tempoNow);
    }
    strip.tempo.value = String(tempoNow);
  });

  // The sound. On to begin with, except in `?bare`; a choice made before Play is kept and goes to the worklet when there is one.
  let monitorOn = !bare;
  strip.showMonitor(monitorOn);
  strip.monitor.addEventListener("click", () => {
    monitorOn = !monitorOn;
    strip.showMonitor(monitorOn);
    host?.setMonitor(monitorOn);
  });

  // The click. A choice made before Play is kept and goes to the worklet when there is one; after that each change goes at once.
  let metronomeOn = false;
  strip.metronome.addEventListener("click", () => {
    metronomeOn = !metronomeOn;
    strip.showMetronome(metronomeOn);
    host?.setMetronome(metronomeOn);
  });

  const startEngine = async (): Promise<void> => {
    if (host || starting) return;
    starting = true;
    setState("starting");
    strip.say("Starting.");
    try {
      const wasm = await (await fetch("/dist/octoweb.wasm")).arrayBuffer();
      // A fixed seed: the same session plays the same way until Save and Load exist.
      const started = await Host.start({ wasm, layout: layoutFromControls(controls), workletUrl: "/dist/src/worklet.js", seed: 0n });
      host = started;
      if (metronomeOn) started.setMetronome(true);
      const time = new ContextTimeMap(started.context, performance);
      scheduler = new MidiScheduler(time, LOOKAHEAD_MS.default);
      wireMidi(started, scheduler, time);
      applyLookahead();
      applyClock(); // a clock state chosen before Play waits for MIDI to be asked for
      // What the panel shows: the engine's LED frame, and over it the chase-light (src/chase.ts). The frame comes only when an LED changed.
      let base: Uint8Array = new Uint8Array(LED_COUNT);
      const chase = new ChaseLight();
      started.onPanel = (p) => {
        if (p.leds) base = p.leds;
        panel.paintLeds(composeChase(base, bare ? null : chase.update(p.position, p.playheads, p.running), stepMap));
        if (follower) {
          // Where the engine is, at the page time that holds for: the follower's comparator (src/follower.ts).
          time.refresh();
          follower.onPosition({ ticks: p.position, pageTimeMs: time.toPage(p.positionFrame / started.sampleRate) });
        }
        const state = p.running ? "playing" : "stopped";
        if (main.getAttribute("data-transport") !== state) {
          main.setAttribute("data-transport", state);
          strip.showPlay(p.running);
          strip.say(p.running ? "Started. The transport is playing." : "Started. The transport is stopped.");
        }
      };
      started.onError = (message) => strip.say(`The engine reported: ${message}`);
      started.setMonitor(monitorOn);
      started.setTempo(tempoNow);
      if (!bare) loadDemo(started);
      // The route is applied before the transport can run, and the worklet handles messages in the order they were posted.
      for (const r of route.routes) started.setTrack(r.track, TRACK_ATTR.midiChannel, r.channel);
      strip.sayRoute([describeRoutes(route.routes), ...route.problems].filter(Boolean).join(" "));
      setState("running");
      strip.say("Started. The transport is stopped.");
      if (!bare) started.transport(true);
    } catch (e) {
      setState("failed");
      strip.say(`The sequencer could not start: ${e instanceof Error ? e.message : String(e)}`);
      return;
    } finally {
      starting = false;
    }
    midi = await openMidi(navigator);
    midi.onChange = () => {
      strip.setOutputs(midi!.outputs());
      strip.setInputs(midi!.inputs());
      applyOutputs();
      applyInput();
    };
    strip.setOutputs(midi.outputs());
    strip.setInputs(midi.inputs());
    strip.sayMidi(midi.message);
    applyInput();
    applyClock();
  };
  strip.play.addEventListener("click", playOrStop);
}

main().catch((e: unknown) => {
  const status = document.querySelector("#engine-status");
  if (status) status.textContent = `The page could not load: ${e instanceof Error ? e.message : String(e)}`;
  throw e;
});

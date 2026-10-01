// The app: the panel, the strip, and between them the engine in its worklet and one Web MIDI output.
// Everything it needs is fetched from the folders the server serves: the control inventory and the tokens (contracts),
// the layout, the module. `Start` is a button because a browser keeps audio silent until the page has been used.
import { layoutFromControls } from "../src/abi.ts";
import { Host } from "../src/host.ts";
import { parseLayout, place, type ControlsDoc } from "../src/layout.ts";
import { openMidi, type Midi } from "../src/midi-access.ts";
import { ContextTimeMap, MidiScheduler } from "../src/midi-out.ts";
import { drawPanel } from "../src/panel.ts";
import { Presses } from "../src/press.ts";
import { LOOKAHEAD_MS, OUTPUT_PORT } from "../src/settings.ts";
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

  let host: Host | null = null;
  const down = new Set<number>();
  const presses = new Presses({
    press(n) {
      if (!host) {
        strip.say("Press Start first, then the keys work.");
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

  let scheduler: MidiScheduler | null = null;
  let midi: Midi | null = null;

  const applyOutput = (): void => {
    if (!scheduler || !midi) return;
    const chosen = midi.outputs().find((o) => o.id === strip.output.value);
    scheduler.setOutput(OUTPUT_PORT, chosen ? chosen.output : null);
    strip.sayMidi(chosen ? `Sending to ${chosen.name}.` : midi.message);
  };
  const applyLookahead = (): void => {
    const ms = Number(strip.lookahead.value);
    if (scheduler && Number.isFinite(ms) && ms >= LOOKAHEAD_MS.min && ms <= LOOKAHEAD_MS.max) scheduler.lookaheadMs = ms;
  };
  strip.output.addEventListener("change", applyOutput);
  strip.lookahead.addEventListener("change", applyLookahead);

  strip.start.addEventListener("click", () => {
    if (host) return;
    void (async () => {
      setState("starting");
      strip.say("Starting.");
      try {
        const wasm = await (await fetch("/dist/octoweb.wasm")).arrayBuffer();
        // A fixed seed: the same session plays the same way until Save and Load exist.
        const started = await Host.start({ wasm, layout: layoutFromControls(controls), workletUrl: "/dist/src/worklet.js", seed: 0n });
        host = started;
        const time = new ContextTimeMap(started.context, performance);
        scheduler = new MidiScheduler(time, LOOKAHEAD_MS.default);
        wireMidi(started, scheduler, time);
        applyLookahead();
        started.onPanel = (p) => {
          if (p.leds) panel.paintLeds(p.leds);
          const state = p.running ? "playing" : "stopped";
          if (main.getAttribute("data-transport") !== state) {
            main.setAttribute("data-transport", state);
            strip.say(p.running ? "Started. The transport is playing." : "Started. The transport is stopped.");
          }
        };
        started.onError = (message) => strip.say(`The engine reported: ${message}`);
        setState("running");
        strip.say("Started. The transport is stopped.");
      } catch (e) {
        setState("failed");
        strip.say(`The sequencer could not start: ${e instanceof Error ? e.message : String(e)}`);
        return;
      }
      midi = await openMidi(navigator);
      midi.onChange = () => {
        strip.setOutputs(midi!.outputs());
        applyOutput();
      };
      strip.setOutputs(midi.outputs());
      strip.sayMidi(midi.message);
    })();
  });
}

main().catch((e: unknown) => {
  const status = document.querySelector("#engine-status");
  if (status) status.textContent = `The page could not load: ${e instanceof Error ? e.message : String(e)}`;
  throw e;
});

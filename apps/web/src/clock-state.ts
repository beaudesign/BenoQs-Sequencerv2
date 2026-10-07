// The clock state the strip offers (SPEC-0002 P4 plan observables M7 and M8, decisions D-P4-5 and D-P4-8): which of four states was asked
// for, which is in force given what the browser can do and what is chosen, and the sentences that say so. Pure: the page's wiring of it
// (the select, the engine's `setClock`, the follower, the echo) is in pages/app.ts.
//
// The labels are the manual's own words (p. 93): "Master Clock", "Slave Clock" and "MIDI Clock echo (in slave mode)". The default is off,
// as the manual's is ("Octopus does not send or react to MIDI Clock information"). The manual's button 200 and its colours belong to the
// panel's workflow, which the Panelwright owns (D-P4-5); this is the strip's choice and says nothing of colour.
//
// A choice that cannot work is not shown as working (M8): with no Web MIDI, with access refused, or with it failing to start, master and
// slave are REFUSED, with a sentence, and the state in force is off. Before Start there is nothing to judge by, so the choice is PENDING
// and says that; it takes effect (or is refused) when MIDI has been asked for.
import type { FollowPhase, FollowStatus } from "./follower.ts";

export type ClockMode = "off" | "master" | "slave" | "slave-echo";
export const CLOCK_MODES: readonly ClockMode[] = ["off", "master", "slave", "slave-echo"];
/** What the clock is until it is asked for something else: the manual's default. */
export const CLOCK_DEFAULT: ClockMode = "off";
export const CLOCK_LABELS: Readonly<Record<ClockMode, string>> = {
  off: "Off",
  master: "Master Clock",
  slave: "Slave Clock",
  "slave-echo": "Slave Clock with MIDI Clock echo",
};

export const isClockMode = (v: unknown): v is ClockMode => typeof v === "string" && (CLOCK_MODES as readonly string[]).includes(v);

export interface ClockWorld {
  /** The engine has started. Before that there is nothing to send a clock with or to follow with. */
  started: boolean;
  /** What the browser said about Web MIDI. `unknown` until it has been asked, which is just after Start (and while a permission prompt is open). */
  midi: "unknown" | "unavailable" | "denied" | "failed" | "ready";
  /** How many devices the two ports send to, counted once each: where the clock goes (master, echo). */
  outputs: number;
  /** The name of the chosen input, or null: where the clock comes from (slave). */
  input: string | null;
}

export interface ClockDecision {
  /** The state in force: what was asked for, or `off` when it is refused or still pending. */
  mode: ClockMode;
  /** Asked for and refused: the control goes back to `off` and the sentence says why. */
  refused: boolean;
  /** Asked for before there is anything to judge by: kept, and takes effect when MIDI has been asked for. */
  pending: boolean;
  /** The engine emits the clock (`octoweb_set_clock`). */
  master: boolean;
  /** The page follows the clock on the chosen input. */
  follow: boolean;
  /** The page passes the incoming clock on to the chosen outputs. */
  echo: boolean;
  /** One sentence for the status line. */
  sentence: string;
}

const NOTHING = { master: false, follow: false, echo: false } as const;

const REFUSALS: Readonly<Record<"unavailable" | "denied" | "failed", string>> = {
  unavailable: "needs Web MIDI and this browser has none, so MIDI Clock stays off.",
  denied: "needs MIDI access, which was refused, so MIDI Clock stays off. Allow it for this site, then press Play again.",
  failed: "needs Web MIDI, which could not start on this system, so MIDI Clock stays off.",
};

const WHERE = (outputs: number): string => (outputs === 1 ? "the chosen output" : "both chosen outputs");

/** What a slave says for a follower in `phase`, `running` or not, after its label. */
function followingWords(input: string, phase: FollowPhase, running: boolean): string {
  switch (phase) {
    case "waiting":
      return `waiting for MIDI Clock from ${input}.`;
    case "settling":
      return `settling on the clock from ${input}.`;
    case "following":
      return running ? `following ${input}, and the transport is playing.` : `following ${input}. The transport is stopped until it sends Start.`;
    case "lost":
      return running ? `the clock from ${input} stopped. The sequencer plays on at the last tempo.` : `the clock from ${input} stopped.`;
  }
}

/** The echo's tail: where the clock goes on to, and the one way it can go wrong that no code can see (a loop through one virtual port). */
function echoWords(outputs: number): string {
  const where = outputs === 0 ? "Echo has no MIDI output to send to." : `Passing the clock on to ${WHERE(outputs)}.`;
  return `${where} Do not send the echo to the port the clock comes from.`;
}

function slaveSentence(mode: "slave" | "slave-echo", world: ClockWorld, phase: FollowPhase, running: boolean): string {
  const head = `${CLOCK_LABELS[mode]}: ${followingWords(world.input!, phase, running)}`;
  return mode === "slave-echo" ? `${head} ${echoWords(world.outputs)}` : head;
}

/** Which state is in force for the one asked for, given what the browser can do and what is chosen. */
export function decideClock(wanted: ClockMode, world: ClockWorld): ClockDecision {
  const label = CLOCK_LABELS[wanted];
  const off = (sentence: string, flags: { refused?: boolean; pending?: boolean } = {}): ClockDecision => ({
    mode: "off",
    refused: flags.refused ?? false,
    pending: flags.pending ?? false,
    ...NOTHING,
    sentence,
  });
  if (wanted === "off") return off("MIDI Clock is off. The sequencer neither sends nor follows a clock.");
  if (!world.started) return off(`Press Play first, then ${label} takes effect.`, { pending: true });
  if (world.midi === "unknown") return off(`Waiting for MIDI access, then ${label} takes effect.`, { pending: true });
  if (world.midi !== "ready") return off(`${label} ${REFUSALS[world.midi]}`, { refused: true });
  if (wanted === "master") {
    const sentence =
      world.outputs === 0
        ? "Master Clock is on, but no MIDI output is chosen, so nothing is sent yet."
        : `Master Clock: sending MIDI Clock and the transport to ${WHERE(world.outputs)}.`;
    return { mode: "master", refused: false, pending: false, master: true, follow: false, echo: false, sentence };
  }
  if (world.input === null) return { mode: wanted, refused: false, pending: false, ...NOTHING, sentence: `${label} needs a MIDI input. Choose one.` };
  return { mode: wanted, refused: false, pending: false, master: false, follow: true, echo: wanted === "slave-echo", sentence: slaveSentence(wanted, world, "waiting", false) };
}

/**
 * The two lines for the strip. `status` changes only when the state does (it is a live region): the mode, the input and output names,
 * the follower's phase and whether the transport is running, and never a number. `detail` carries the numbers and is not a live region.
 */
export function describeClock(decision: ClockDecision, world: ClockWorld, status: FollowStatus | null): { status: string; detail: string } {
  if (!decision.follow || status === null) return { status: decision.sentence, detail: "" };
  const mode = decision.mode === "slave-echo" ? "slave-echo" : "slave";
  const text = slaveSentence(mode, world, status.phase, status.running);
  if (status.bpm === null) return { status: text, detail: "" };
  const parts = [`${status.bpm.toFixed(1)} BPM`, `${status.pulses} ${status.pulses === 1 ? "pulse" : "pulses"}`, `jitter ${status.jitterMs.toFixed(1)} ms`];
  if (status.phaseMs !== null) parts.push(`engine ${Math.abs(status.phaseMs).toFixed(1)} ms ${status.phaseMs >= 0 ? "ahead" : "behind"}`);
  return { status: text, detail: parts.join(", ") };
}

// The clock state the strip offers (SPEC-0002 P4 plan observables M7 and M8, decisions D-P4-5 and D-P4-8): which of four states the owner
// asked for, what is in force given what the browser can do and what is chosen, and the sentences that say so. The page's wiring of it is
// in test/browser/app.browser.test.ts; here it is the decisions and the words, with no browser.
//
// The labels are the manual's own (p. 93): "Master Clock", "Slave Clock", and "MIDI Clock echo (in slave mode)"; the default is off, as the
// manual's default is ("Octopus does not send or react to MIDI Clock information").
import assert from "node:assert/strict";
import { test } from "node:test";
import type { FollowStatus } from "../src/follower.ts";
import { CLOCK_DEFAULT, CLOCK_LABELS, CLOCK_MODES, decideClock, describeClock, isClockMode, type ClockMode, type ClockWorld } from "../src/clock-state.ts";

const ready: ClockWorld = { started: true, midi: "ready", outputs: 1, input: "Live" };
const world = (over: Partial<ClockWorld> = {}): ClockWorld => ({ ...ready, ...over });
const status = (over: Partial<FollowStatus> = {}): FollowStatus => ({ phase: "following", bpm: 120, running: true, phaseMs: 0.4, jitterMs: 1.2, pulses: 480, ...over });

test("four states, off first, with the manual's own words as labels", () => {
  assert.deepEqual([...CLOCK_MODES], ["off", "master", "slave", "slave-echo"]);
  assert.deepEqual(CLOCK_LABELS, { off: "Off", master: "Master Clock", slave: "Slave Clock", "slave-echo": "Slave Clock with MIDI Clock echo" });
  assert.equal(CLOCK_DEFAULT, "off", "the manual's default: Octopus neither sends nor reacts to MIDI Clock");
  assert.equal(CLOCK_MODES[0], CLOCK_DEFAULT, "and it is the first in the list");
  assert.equal(isClockMode("master"), true);
  for (const bad of ["", "Master", "slave echo", null, undefined, 3, {}]) assert.equal(isClockMode(bad), false, String(bad));
});

test("off: nothing is sent and nothing is followed, whatever the world is", () => {
  for (const w of [ready, world({ started: false, midi: "unknown", input: null }), world({ midi: "denied" })]) {
    const d = decideClock("off", w);
    assert.deepEqual([d.mode, d.master, d.follow, d.echo, d.refused, d.pending], ["off", false, false, false, false, false]);
    assert.match(d.sentence, /^MIDI Clock is off\./);
  }
});

test("master: the engine is the clock, and the sentence says where it goes", () => {
  const d = decideClock("master", ready);
  assert.deepEqual([d.mode, d.master, d.follow, d.echo, d.refused, d.pending], ["master", true, false, false, false, false]);
  assert.match(d.sentence, /^Master Clock: sending MIDI Clock and the transport to the chosen output\.$/);
  assert.match(decideClock("master", world({ outputs: 2 })).sentence, /to both chosen outputs\.$/);
});

test("master with no output chosen is accepted, and says that nothing is sent yet", () => {
  const d = decideClock("master", world({ outputs: 0 }));
  assert.equal(d.master, true, "it is on: choosing an output later is heard at once");
  assert.equal(d.refused, false);
  assert.match(d.sentence, /no MIDI output is chosen, so nothing is sent yet\./);
});

test("slave follows the clock on the chosen input, and does not send one", () => {
  const d = decideClock("slave", ready);
  assert.deepEqual([d.mode, d.master, d.follow, d.echo], ["slave", false, true, false]);
});

test("slave with no input chosen is accepted and says to choose one: choosing it later starts the following", () => {
  const d = decideClock("slave", world({ input: null }));
  assert.equal(d.mode, "slave");
  assert.equal(d.refused, false);
  assert.match(d.sentence, /^Slave Clock needs a MIDI input\. Choose one\.$/);
  assert.equal(d.follow, false, "nothing to follow yet");
});

test("slave with echo follows and passes the clock on; with no output it follows and says the echo has nowhere to go", () => {
  const d = decideClock("slave-echo", ready);
  assert.deepEqual([d.mode, d.master, d.follow, d.echo], ["slave-echo", false, true, true]);
  const nowhere = decideClock("slave-echo", world({ outputs: 0 }));
  assert.equal(nowhere.follow, true);
  assert.equal(nowhere.echo, true);
  assert.match(describeClock(nowhere, world({ outputs: 0 }), status()).status, /Echo has no MIDI output to send to\./);
});

test("M8: master and slave are refused, with a sentence, when MIDI is not there to do it, and the state falls back to off", () => {
  const cases: [ClockWorld["midi"], RegExp][] = [
    ["unavailable", /needs Web MIDI and this browser has none, so MIDI Clock stays off\.$/],
    ["denied", /needs MIDI access, which was refused, so MIDI Clock stays off\. Allow it for this site, then press Start again\.$/],
    ["failed", /needs Web MIDI, which could not start on this system, so MIDI Clock stays off\.$/],
  ];
  for (const mode of ["master", "slave", "slave-echo"] as const) {
    for (const [midi, words] of cases) {
      const d = decideClock(mode, world({ midi }));
      assert.deepEqual([d.mode, d.master, d.follow, d.echo, d.refused], ["off", false, false, false, true], `${mode} with MIDI ${midi}`);
      assert.ok(d.sentence.startsWith(`${CLOCK_LABELS[mode]} `), `${mode}: names what was asked for: ${d.sentence}`);
      assert.match(d.sentence, words, `${mode} with MIDI ${midi}`);
    }
  }
});

test("before Start, and while MIDI is being asked for, there is nothing to send or follow with: the choice is kept as pending, with a sentence, and not refused", () => {
  for (const [w, words] of [
    [world({ started: false, midi: "unknown" }), /^Press Start first, then .+ takes effect\.$/],
    [world({ started: true, midi: "unknown" }), /^Waiting for MIDI access, then .+ takes effect\.$/],
  ] as const) {
    for (const mode of ["master", "slave", "slave-echo"] as const) {
      const d = decideClock(mode, w);
      assert.deepEqual([d.mode, d.master, d.follow, d.echo, d.refused, d.pending], ["off", false, false, false, false, true], `${mode}: ${JSON.stringify(w)}`);
      assert.match(d.sentence, words, mode);
      assert.ok(d.sentence.includes(CLOCK_LABELS[mode]), `${mode}: names what is waiting`);
    }
  }
});

test("the sentences are sentences: sentence case, one full stop at the end, no colour words, and the manual's own names", () => {
  assert.equal(CLOCK_MODES.length, 4, "there are states to look at");
  for (const mode of CLOCK_MODES) {
    for (const w of [ready, world({ outputs: 0 }), world({ input: null }), world({ midi: "denied" }), world({ started: false, midi: "unknown" })]) {
      const d = decideClock(mode, w);
      assert.match(d.sentence, /^[A-Z]/, d.sentence);
      assert.match(d.sentence, /\.$/, d.sentence);
      assert.doesNotMatch(d.sentence, /\b(orange|green|red)\b/i, "the manual's colours belong to the 200 key, which this strip is not");
    }
  }
});

// ----- the words while following -----

test("slave: the status line says what the follower is doing, in words that change only when the follower's state does", () => {
  const d = decideClock("slave", ready);
  const say = (s: Partial<FollowStatus>, w = ready): string => describeClock(d, w, status(s)).status;
  assert.equal(say({ phase: "waiting", bpm: null, running: false, phaseMs: null }), "Slave Clock: waiting for MIDI Clock from Live.");
  assert.equal(say({ phase: "settling", running: false, phaseMs: null }), "Slave Clock: settling on the clock from Live.");
  assert.equal(say({ phase: "following", running: false, phaseMs: null }), "Slave Clock: following Live. The transport is stopped until it sends Start.");
  assert.equal(say({ phase: "following", running: true }), "Slave Clock: following Live, and the transport is playing.");
  assert.equal(say({ phase: "lost", running: true, phaseMs: null }), "Slave Clock: the clock from Live stopped. The sequencer plays on at the last tempo.");
  assert.equal(say({ phase: "lost", running: false, phaseMs: null }), "Slave Clock: the clock from Live stopped.");
  // The same words for any tempo, jitter or phase: a live region that spoke each number would never stop.
  assert.equal(say({ bpm: 87.3, jitterMs: 2.9, phaseMs: -4.4, pulses: 12345 }), say({ bpm: 143, jitterMs: 0.1, phaseMs: 0.2, pulses: 3 }));
});

test("slave with echo says it is passing the clock on, and not to send it back to where it comes from", () => {
  const d = decideClock("slave-echo", ready);
  const text = describeClock(d, ready, status()).status;
  assert.match(text, /^Slave Clock with MIDI Clock echo: following Live, and the transport is playing\. Passing the clock on to the chosen output\./);
  assert.match(text, /Do not send the echo to the port the clock comes from\.$/);
  assert.match(describeClock(d, world({ outputs: 2 }), status()).status, /Passing the clock on to both chosen outputs\./);
});

test("the detail line carries the numbers: tempo, pulses, jitter, and how far ahead or behind the engine is", () => {
  const d = decideClock("slave", ready);
  assert.equal(describeClock(d, ready, status()).detail, "120.0 BPM, 480 pulses, jitter 1.2 ms, engine 0.4 ms ahead");
  assert.equal(describeClock(d, ready, status({ bpm: 87.34, phaseMs: -4.44, jitterMs: 2.91, pulses: 12 })).detail, "87.3 BPM, 12 pulses, jitter 2.9 ms, engine 4.4 ms behind");
  assert.equal(describeClock(d, ready, status({ pulses: 1 })).detail, "120.0 BPM, 1 pulse, jitter 1.2 ms, engine 0.4 ms ahead", "one pulse, not one pulses");
  assert.equal(describeClock(d, ready, status({ phaseMs: 0 })).detail, "120.0 BPM, 480 pulses, jitter 1.2 ms, engine 0.0 ms ahead", "exactly on the beat has to read as one or the other; a measured phase is never exactly zero");
  assert.equal(describeClock(d, ready, status({ phaseMs: null })).detail, "120.0 BPM, 480 pulses, jitter 1.2 ms", "no phase to report while the transport is stopped");
  assert.equal(describeClock(d, ready, status({ phase: "waiting", bpm: null, phaseMs: null, pulses: 0, jitterMs: 0 })).detail, "", "nothing to say before there is a tempo");
  assert.equal(describeClock(d, ready, null).detail, "");
});

test("the detail line is empty for off and master, and for a refusal", () => {
  for (const mode of ["off", "master"] as const) assert.equal(describeClock(decideClock(mode, ready), ready, null).detail, "", mode);
  assert.equal(describeClock(decideClock("slave", world({ midi: "denied" })), world({ midi: "denied" }), status()).detail, "");
});

test("without a follower status (off, master, a refusal, pending) the status line is the decision's own sentence", () => {
  for (const [mode, w] of [["off", ready], ["master", ready], ["slave", world({ midi: "denied" })], ["master", world({ started: false, midi: "unknown" })]] as [ClockMode, ClockWorld][]) {
    const d = decideClock(mode, w);
    assert.notEqual(d.sentence, "", "there is a sentence");
    assert.equal(describeClock(d, w, null).status, d.sentence);
  }
});

test("a name with odd characters is put in the sentence as text and never as markup", () => {
  const name = '<img src=x onerror=alert(1)> "Live" & co';
  const w = world({ input: name });
  const text = describeClock(decideClock("slave", w), w, status()).status;
  assert.ok(text.includes(name), "the name is there, character for character");
});

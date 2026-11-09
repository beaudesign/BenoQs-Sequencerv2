# SPEC-0008 tech: how the page does it

Companion to `README.md` (what and why) and `plan.md` (the slices). This file is the how: the files, types, signatures and algorithms, in terms of the code on `main`. It is a proposal, like
the rest of the spec, and nothing in it is approved. Where it refines a line of the README, section 14 lists the line, so the owner sees every changed claim in one place.

Every statement about existing code names the file it was read in, on `main` at `d705004`. *(Not checked)* marks what I could not read or run: the Web MIDI text, Chromium's source, and every
behaviour of real hardware. No code in this file has been written or run. The types are sketches in TypeScript for the Forge to take, not a patch.

## 1. What exists, as the code has it

| Fact | Where |
|---|---|
| The scheduler holds `outputs: Map<port, MidiOutputLike>`, `devices` (each device once, for real-time messages) and `latest: Map<output, number>` (rule 1's record). `setOutput` swaps a device with no Release to the one dropped | `src/midi-out.ts` |
| `send` stamps `toPage(at) + lookaheadMs`, raises the stamp to follow the previous one when it is less than `ORDER_WINDOW_MS` (50) behind, and sends unstamped if the result is not in the future. There is no `try`; a throwing `output.send` aborts the rest of the batch | `src/midi-out.ts` |
| `schedule(frame, sampleRate, records)` walks 12-byte records; kind 5 goes to every device, kinds 0 to 4 go to `outputs.get(port)` and are `unrouted` when there is none | `src/midi-out.ts`, `engine/ABI.md` |
| `passThrough(status)` sends a real-time byte unstamped to every device (the echo) | `src/midi-out.ts` |
| The worklet posts `events` (`frame`, `bytes`) for every block that has any, and `panel` every `PANEL_EVERY_BLOCKS` (8) blocks, about 21 ms at 48 kHz. It reads `engine.tickPosition()` before and after each render (`from`, and after) | `src/worklet.ts`, `src/protocol.ts` |
| `routeFromWorklet` hands `events` to `sink.onEvents(frame, bytes)` and `wireMidi` hands that to the scheduler after refreshing the clock map | `src/host.ts`, `src/wire.ts` |
| A port is chosen on the strip, one select for port 1 and one for port 2; `applyOutputs` calls `scheduler.setOutput(port, device.output)` and re-decides the clock. `applyLookahead` sets `scheduler.lookaheadMs`. Nothing is stored; `?route=track:channel` is read once, and channel 17 to 32 is port 2 | `pages/app.ts`, `src/route.ts` |
| The clock state defaults to **Off** (`CLOCK_DEFAULT`). The engine emits real-time records only when it is master (`octoweb_set_clock`). So today a device is sent no Clock, Start or Stop unless the person chose Master Clock | `src/clock-state.ts`, `engine/ABI.md` |
| `InputPath` has one consumer slot, `onMessage`, called for decoded messages only; the follower and the echo are what `app.ts` puts in it | `src/midi-in.ts`, `pages/app.ts` |
| `Monitor.render(out, events)` takes `NoteEvent`, a `Pick` of `MidiEvent` with kind, channel, d1, d2 and atSample: no `port`. The drums are channel 10 | `src/monitor.ts` |
| The stand-in `FakeOutput` queues stamped sends and delivers by timestamp, lets an unstamped send overtake, and holds a note from Note On to Note Off by `channel/key`. It does not model CC 123, a throw, a sound delay or a position. `Session` joins the real worklet, wiring and scheduler to it on a fake clock | `test/support/receiver.ts`, `test/support/session.ts` |
| `stop-property.test.ts` already asserts that after Stop the receiver holds nothing, for generated patterns, with `clear()` present and absent | `test/stop-property.test.ts` |
| The S3 follow-run analysis computes "a fifth of a step" from `STEP_MS_AT_1_BPM = 3750` (a step of **1.5 pulses**), so it reports 6.25 ms at 120 BPM. Two tests pin 6.25 | `spikes/s3/analyse.ts`, `test/s3-analyse.test.ts` |

The last row is a finding of this revision and is section 12.

## 2. Principles

1. **Page-side only.** No change to the engine, `engine/ABI.md`, the layout or `contracts/` (D-H1). Song Position is a field of a worklet message (section 8), which is `protocol.ts`, page code.
2. **Rule 1 stays.** Timestamps to one device never go backwards; `MIDIOutput.clear()` is never called. Every new way of sending (panic, a test note, a trim, thru) is built on the per-device record and asserted against it.
3. **The table is a record of what was sent, not a guess about what sounds.** It is checked against the stand-in at every step of a generated run (HW-11).
4. **Nothing goes out on its own.** No byte is sent on load, on rig read, or on a bind; only Play, the Panic control, a test button or a release (HW-5).
5. **A failure is contained and said once.** A device that throws is marked failed with a sentence and the rest carry on.
6. **Every belief about a device is in the stand-in and named there** (R-10): CC 123 and CC 120 clear a channel's notes, a Note Off names a key and not a strike, Start sets the position to 0, Continue keeps it, a Song Position sets it. Each is checked at the bench.

## 3. Module map

New files are in `apps/web/src/`, the Forge's. A slice adds only its own.

| File | Slice | What it holds |
|---|---|---|
| `output-stage.ts` | H1a | `SoundingTable`, `OutputStage` (a device's send guard, rule 1 record and table) |
| `release.ts` | H1a, H1b | `releaseMessages(state)` (H1a), `cutMessages(state)` (H1b): pure |
| `stall.ts` | H1b | `StallWatchdog`: pure, with an injected clock |
| `rig.ts` | H2 | the types of section 6, `defaultRig`, `parseRig`, `serialiseRig`, `validateRig`, `sharedAddresses` (HW-26) |
| `rig-bind.ts` | H2 | `bindDevices(rig, access)`: by id, then by name, never a guess |
| `rig-store.ts` | H2 | `RigStore` interface and the storage adapter, every access in `try` |
| `router.ts` | H2 | `Router.compile(rig, bound)` and `resolve(port, channel)` |
| `trim.ts` | H3 | `effectiveLookaheadMs`, the trim-change slack, `WireLoad` |
| `align.ts` | H3 | the align tool's schedule (pure) |
| `song-position.ts` | H4 | `songPositionBytes`, `TICKS_PER_SONG_POSITION`, `hasContinue` |
| `echo-guard.ts` | H4 | `echoTargets(inputName, devices)` |
| `grid-clock.ts` | H6 | `GridClock.tickAt` (HW-25) |
| `thru.ts` | H6 | the thru consumer and its same-name guard |

Existing files that change: `midi-out.ts` (H1a to H4), `wire.ts` and `host.ts` and `protocol.ts` and `worklet.ts` (H4's field, H3's align click, H5's port mask), `midi-in.ts` (H6), `monitor.ts` (H5),
`midi-access.ts` (an optional `manufacturer`, H2), `settings.ts` (the new constants), `pages/app.ts` (the wiring of each), the strip and its CSS (the controls: the Forge's, under `docs/05-design-system.md`:
text, no colour, no easing).

## 4. The output stage (H1a)

```ts
// src/output-stage.ts
export interface SoundingState {
  /** Notes held, as (channel << 7) | note with channel 0 to 15. A set: a Note Off names a key and not a strike (README 4.2). */
  readonly held: ReadonlySet<number>;
  /** Channels, 0 to 15, that have carried a channel message since the table was cleared. */
  readonly used: ReadonlySet<number>;
  /** The last value sent per channel, kept only when it is not the resting value: sustain (CC 64) not 0, bend not 8192, pressure not 0. */
  readonly sustain: ReadonlyMap<number, number>;
  readonly bend: ReadonlyMap<number, number>;
  readonly pressure: ReadonlyMap<number, number>;
}

export class SoundingTable {
  /** Applies one message that was sent. Note On adds, Note Off removes; CC 123 and CC 120 on a channel clear that channel's held notes; CC 64, bend and pressure are recorded. Real-time and unknown bytes change nothing. */
  observe(bytes: ArrayLike<number>): void;
  get state(): SoundingState;
  get isEmpty(): boolean; // nothing held, no sustain, bend or pressure away from rest
  clear(): void;
}

export class OutputStage {
  readonly output: MidiOutputLike;
  readonly id: string | null; // the port's id, when the page knows it
  readonly table = new SoundingTable();
  latest: number | undefined; // rule 1's record, moved here from MidiScheduler.latest
  failed: string | null = null; // one sentence, set once
  dropped = 0; // records not sent because it is failed
  /** One guarded send. True if the device took it. A throw marks the stage failed and is not rethrown. */
  deliver(bytes: number[], timestamp: number | null): boolean;
}
```

`deliver` is the only place the scheduler calls `output.send`. If `failed` is set it counts a drop and returns false. Otherwise it calls `send(bytes)` or `send(bytes, timestamp)` inside `try`; on a throw it sets `failed` to
a sentence ("MIDI output `<name>` stopped accepting messages. The browser said: `<message>`."), reports it once through `scheduler.onFail`, and returns false; on success it calls `table.observe(bytes)`. The table
is therefore updated **when a message is sent**, in send order, and not when the device delivers it: for a stamped send the device acts later.

**What HW-11 compares.** The stand-in's `held` changes on delivery, so "equal at every step" would be false while a Note On waits in its queue. The comparison is with `FakeOutput.heldEventually()`: the stand-in's `held`
with its queue applied in timestamp, then send, order. The scheduler never lets an unstamped send overtake a queued one (an unstamped send happens only when the stamp is not in the future, and rule 1 raised it to follow
the record, so if anything is queued the stamp is in the future), so the eventual set is the send-order set and the two are equal. The only unstamped sends that overtake on purpose are the first half of a panic (section 5), which HW-12 covers.

**The scheduler, changed.** `outputs` becomes `Map<number, OutputStage>` and `devices` becomes `OutputStage[]` (each once). `latest` is `stage.latest`. `send` calls `stage.deliver`. A `try` also guards `onSend`, which a spike supplies and
which today can abort a batch; a throw there is counted in `stats.hookErrors` and not rethrown. New stats: `failed` (stages marked failed) and `dropped`. All existing stats keep their meaning, and every existing `midi-out.test.ts`,
`wire.test.ts` and `stop-property.test.ts` case passes unchanged (a condition of H1a).

**Swap, loss and return (HW-13).** `setOutput(port, output, id?)` gains an optional `id`.

1. If the port's stage is replaced and no other port still uses it, the old stage gets a **Release** (section 5) before it is dropped.
2. If the stage's table is not empty and it has an `id`, the stage is parked in `parked: Map<id, OutputStage>` (at most 8; the oldest is dropped), so the notes it owes are not forgotten.
3. When `setOutput` is called with an `id` that is parked, the owed state is released to the new object first, before anything else is sent to it, then the parked entry is removed. This is how a synthesiser whose
   cable was pulled gets the Note Offs it never heard, the first time anything can reach it.
4. The Release to a lost device is sent inside `try`: a disconnected port may throw *(not checked: what each browser does)*, and a throw there is not an error.

`pages/app.ts` passes `device.id` in `applyOutputs` and remembers each port's last id, so that a device that has left `midi.outputs()` can still be named: `applyOutputs` then calls `setOutput(port, null, id)`, which is the loss path.

## 5. Panic, the watchdogs, the test note (H1b)

```ts
// src/release.ts: pure
export function releaseMessages(state: SoundingState): number[][];
// For each held note, ordered by channel then note: [0x80 | ch, note, 0]. Then, for each channel in `used`: [0xB0 | ch, 123, 0].
// Then, for each channel in `sustain`: [0xB0 | ch, 64, 0]. (Addition: a Note Off does not end a note while a sustain is down.)

export function cutMessages(state: SoundingState): number[][];
// releaseMessages(state), then for every channel 0 to 15: CC 120 value 0, CC 64 value 0, pitch bend centre [0xE0 | ch, 0x00, 0x40].
```

A Cut for one device is at most 144 bytes beyond the Release (16 channels, 9 bytes each), which is **46 ms on a DIN cable** at 0.32 ms a byte. It is the Panic button's cost, and a person can hear it end.

**The scheduler's `panic(level, now)`**, for each device (a failed one is skipped and counted):

1. `messages` = `cutMessages` if `level` is `"cut"` **and** the device's `panic` is `"cut"`, else `releaseMessages`, computed from the table **before** anything is sent.
2. Send every message at once: `stage.deliver(m, null)`.
3. If `stage.latest > now` (something is still queued for it), send every message again with the stamp `stage.latest + 1`, and set `stage.latest` to it. Whatever was queued plays, then this ends it. (F-H6: a single unstamped cut is overtaken by the queue.)

If nothing is queued, step 3 is skipped: a second copy would be harmless but is not needed.

`releaseAll(now)` is `panic("release", now)` over every device and every parked stage. It is what Stop's page-side safety, a swap, a stall and `pagehide` call. **Stop itself is unchanged**: the engine's own flush Note
Offs and CC 123 still come as events and are scheduled like any other.

**The Panic control** calls `host.transport(false)` first, then `panic("cut", now)` at once, without waiting for the worklet to answer. A worklet that is gone is the case the table exists for.

**`pagehide`.** `pages/app.ts` installs `pagehide` and the Page Lifecycle `freeze` handler, both calling `releaseAll`. `freeze` is Chromium's and is harmless where it never fires *(not checked)*. Whether a browser sends anything out
of a closing page is B5; the observable (HW-15) is only that the handlers exist and call `releaseAll`, with a stand-in `EventTarget`.

**The stall watchdog.**

```ts
// src/stall.ts: pure, clock injected
export class StallWatchdog {
  constructor(readonly thresholdMs = 250, readonly intervalMs = 50);
  heard(nowMs: number): void; // a `panel` message arrived
  check(nowMs: number, running: boolean): "ok" | "stalled" | "skipped";
}
```

`check` returns `stalled` once, when `running`, no `heard` has come for `thresholdMs`, **and** `heard` has not been called between this check and the one before it. It returns `skipped` when more than four intervals have passed since the
previous `check` (the page's own timer was asleep, so the silence may be the page's and not the engine's), and re-reads on the next. After `stalled` it returns `ok` until `heard` is called again, so it speaks once. On `stalled` the page
calls `releaseAll`, posts `{ type: "transport", play: false }` so a worklet that wakes does not carry on, and says "The engine stopped answering. The devices were released and the transport was stopped." It does not resume.
A hidden tab's timers can run as seldom as once a second *(not checked)*, so the 250 ms is honoured only for a visible tab; B2 measures what a hidden one does. The thresholds are proposals.

**The test note.** `scheduler.testNote(deviceIndex, channel, note, velocity, lengthMs, now)` sends the Note On at once and the Note Off **stamped** `now + lengthMs`, both through `deliver`, so the Note Off is the device's to deliver
and not a page timer's, and the table holds the note until then, so any Release ends it. It is called only from the control's click handler; no constructor, rig read or bind calls it (HW-5, HW-17). Velocity defaults to 64.

## 6. The rig (H2)

```ts
// src/rig.ts
export interface Rig {
  version: 1;
  name: string;
  devices: Device[];
  routes: Route[];
  input: InputSettings | null;
  /** Fields this version does not know, kept on export (HW-3). */
  extra?: Record<string, unknown>;
}
export interface Device {
  key: string; // /^[a-z0-9-]{1,24}$/, unique
  label: string; // the person's words
  outId: string;
  outName: string;
  outManufacturer?: string; // addition: the README says "name and manufacturer" and `MidiPortLike` has no such field yet
  inId?: string;
  inName?: string;
  trimMs: number; // -50 to +100, positive earlier, default 0 (H3; ignored before it)
  clock: boolean; // default true (D-H7; honoured from H4)
  transport: boolean; // default true (honoured from H4)
  panic: "cut" | "release"; // default "cut": what the Panic control sends this device (Q-8-5). Defined here: the README names the field and not its values
  channelLabels?: Record<string, string>; // "1" to "16"
}
export interface Route {
  port: 1 | 2;
  from: number; // 1 to 16
  to: number; // from to 16
  device: string; // a Device.key
  channel: "same" | number; // 1 to 16. A number with from < to merges the range onto one channel
}
export interface InputSettings {
  inId?: string;
  inName?: string;
  offsetMs: number; // for gridTickAt, default 0 (H6)
  thru: { from: "all" | number; device: string; channel: "same" | number; confirmedSameName?: boolean }[];
}
```

**Validation (`validateRig`)** returns sentences, one for each problem, and a rig with any is not applied: unknown `version` (refused with "This rig is version N; this page reads version 1"); a duplicate key; a route that names no device; two routes
that overlap on one port and channel (HW-2); `from > to`; a trim outside -50 to +100; a channel outside 1 to 16. **A device with no routes is valid**: it receives what its `clock` and `transport` flags allow and no notes (section 13 uses this).

**The default rig (`defaultRig(chosen)`)** is what the strip makes today: one device for each of port 1 and port 2 if chosen (the same device twice if the same one is chosen for both), `clock` and `transport` true, trim 0,
and two routes, port 1 channels 1 to 16 to port 1's device and port 2 to port 2's, `"same"`. With it the router's output is the scheduler's current output, byte for byte and stamp for stamp (HW-1), and every existing scheduler test passes
unchanged. The two selects stay: they are how a person with two devices and no stored rig works, and they make the default rig.

**Binding (`bindDevices`)**, for each device, in this order: (1) a port whose `id` equals `outId` is `bound` by id; (2) otherwise ports whose normalised name (trimmed, lower case) and manufacturer equal the stored ones: exactly one is `bound` by
name, more than one is `ambiguous` and the rig sheet asks, none is `missing`. A `missing` device keeps its routes and receives nothing, counted in `stats.toMissing` and never thrown. When the port list changes (`midi.onChange`) the binding is
run again; a `missing` device that returns is bound and its parked table (section 4) is released first. The same applies to inputs.

**Storage (`RigStore`)**: `load(): string | null` and `save(text: string): void`, with every access in `try` so a private window or blocked storage degrades to "not saved, and says so". The Forge chooses the backing (the page's own
`localStorage` is enough for a rig of a few kilobytes; IndexedDB if it prefers to match `octopus-state`'s plan, `specs/SPEC-0002/tech.md` section 7). Export is a download of `serialiseRig` as `wenge-rig-<name>.json`; import is a file input
read with `FileReader` and passed to `parseRig`. No network. A rig read from storage or a file sends no byte (HW-5); `serialiseRig(parseRig(text))` equals `text` for a text the page wrote (HW-3).

**The router (`Router.compile(rig, bound)`)** builds a 2 by 16 table from `(port, channel)` to `(stage index, channel nibble)`. `resolve(port, channel)` is one array read in `schedule`. Real-time messages go to the stages whose flag allows them (section 8).
`sharedAddresses(rig, trackChannels)` (HW-26) is pure: it returns the groups of tracks whose engine channel resolves to the same `(device, channel)`.

## 7. Timing (H3, High)

**The stamp.** `ts = toPage(at) + lookahead_eff - trim_d`, where

```ts
// src/trim.ts
export const GUARD_MS = 10; // proposal
export function effectiveLookaheadMs(userMs: number, trims: readonly number[]): number {
  const largest = Math.max(0, ...trims);
  return Math.max(userMs, largest + GUARD_MS);
}
```

For every device `lookahead_eff - trim_d >= GUARD_MS`, so the stamp is at least the guard ahead of the instant the audio leaves, whatever lead the batch reached the page with (HW-8). The page tells the person when it raised the figure:
"Lookahead raised from 30 to 60 ms so a device 50 ms early can be reached." A trim of +100 forces 110 ms for every device (the cost named in README 4.3). `stats.late` still counts any event that is late for another reason.

**A trim change cannot hang a note (HW-7).** A change of `delta` ms earlier lowers later stamps by `delta` relative to those already queued. Rule 1's raise has a 50 ms window beyond which it believes the new time ("the clocks jumped"), and a
trim step can be larger. So a stage keeps `orderSlackMs`: a change of `delta` earlier adds `delta` to it, rule 1 uses `ORDER_WINDOW_MS + orderSlackMs` for that stage, and the slack returns to zero as soon as `stage.latest` is in the past (nothing queued).
Stated limit: after a trim change the events due within `delta` ms are stamped together at `latest`, in send order, so a person changing a trim hears at most a short flam, and a Note Off still follows its Note On. A change later (negative) needs no slack.

**Wire load (HW-10).** `WireLoad` groups a device's sends whose stamps are within 1 ms of the first of a group and reports, per device, the worst group in a rolling 10 s window as `bytes x 0.32` ms. A seven-note chord on one tick is 21 bytes, 6.72 ms.
It is shown on the rig sheet and never gates anything. The 0.32 is MIDI 1.0's serial rate (general knowledge, not checked in this session).

**The align tool.** It needs one thing the worklet does not do today: put a click at a **given** audio frame (the metronome's click is tied to the engine's tick position, `src/click.ts`). So H3 adds a message
`{ type: "alignClick"; frame: number }` to `ToWorklet`, which the worklet renders as the same click voice at that frame (audio the worklet makes, nothing in the events or the panel). The tool, once per beat of a 120 BPM pattern of its own:
picks a frame a little ahead, posts the click for that frame, and sends the device a note stamped `toPage(frame / sampleRate) + lookahead_eff - trim_d`. The person moves the trim until click and note are one hit; the number it ends on is the
device's trim against the page's own sound (HW-9's arithmetic: two stand-ins with sound delays of 7 and 19 ms, trimmed by the difference, sound within 0.5 ms of each other). A second device is aligned against the first the same way, by moving only its trim.
The tool's notes go through `deliver`, so the table holds them and Panic ends them. *(Not checked: that a person can hear a click and a synthesiser as one hit to within a millisecond or two. That is B3, and its result sets how fine the control is.)*

## 8. Clock and transport (H4, High)

**Flags (HW-18).** In `schedule`'s kind 5 branch, a Clock (`0xF8`) goes to stages whose device has `clock` true; Start, Continue and Stop (`0xFA` to `0xFC`) go to those with `transport` true. `passThrough` (the echo) filters the same way.
The router gives each stage its device's flags, so before H2 (no rig) every device has both on, which is today's behaviour. `stats.realtime` counts what was sent, one for each device that heard it.

**Song Position (HW-19).**

```ts
// src/song-position.ts
export const TICKS_PER_SONG_POSITION = 6 * TICKS_PER_CLOCK; // 12: a MIDI beat is six clocks
export function songPositionBytes(units: number): number[]; // [0xF2, units & 0x7f, (units >> 7) & 0x7f], units clamped to 0 to 16383
export function hasContinue(records: Uint8Array): boolean; // any kind 5 record with status 0xFB
```

- **Derived in the worklet.** In `process()`, `from` (the engine's tick position before the render) is already read for the click. If the block's events contain a Continue, the worklet posts
  `{ type: "events", frame, bytes, songPosition: Math.floor(from / TICKS_PER_SONG_POSITION) }`. The field is absent otherwise. The position before the render is the held position from the Stop, which is where Continue resumes.
  `protocol.ts` gains the optional field; `host.ts`'s `WorkletSink.onEvents`, `wire.ts`'s `EventSource.onEvents` and `MidiScheduler.schedule` gain an optional fourth argument.
- **Sent before the Continue, to the devices that hear transport.** On reaching a Continue record with a `songPosition`, the scheduler sends `songPositionBytes(units)` to each stage whose `transport` is on, with the **same** `at` as the Continue and immediately before it, so
  rule 1 and the stand-in's order (timestamp, then send) keep them in that order. A Start is not preceded by one. Counted in a new `stats.songPosition`.
- **Stated limits.** A Song Position has the resolution of one step (12 ticks), so a Continue after a Stop in the middle of a step leaves the device up to one step ahead of or behind the engine, which resumes mid-step. A position past 16383 units
  (1,023 bars of four quarters) is clamped. An incoming Song Position stays ignored (D-H8, Q-8-8). *(Not checked: which gear follows a Song Position.)*
- **If SPEC-0007's P0a is approved first**, the derivation lives on its bus and this slice only consumes it. If not, the field above is the whole of it.

**The echo guard (HW-21).** `echoTargets(inputName, devices)` returns the devices that may be sent the echo and the sentences for those refused: a device whose `outName` equals the clock input's name (lower case, spaces removed) is refused with
"Not passing the clock to `<name>`: it is the port the clock comes from." It does not catch a loop through a virtual port with another name, and the strip's existing sentence ("Do not send the echo to the port the clock comes from") stays.

**R0b (the first note, 11 ticks after Start) is untouched** (D-H9). HW-20 is written when Q-8-4 is answered.

## 9. The input stage (H6)

- **Consumers (HW-22).** `InputPath` gains `addConsumer(fn): () => void` (returns the remover) and keeps `onMessage` working as a property that is itself one consumer, so `midi-in.test.ts` passes unchanged. Each decoded message reaches each consumer once, in arrival
  order, with the browser's timestamp, each call in its own `try`: a throw is counted in `stats.consumerErrors` and does not stop the others. `app.ts` moves the follower and the echo to `addConsumer`.
- **`gridTickAt` (HW-25).**

  ```ts
  // src/grid-clock.ts
  export interface GridSample { ticks: number; pageTimeMs: number; bpm: number; running: boolean }
  export class GridClock {
    constructor(history = 64);
    onPosition(s: GridSample): void; // from each `panel` message, as the follower already receives it
    /** The engine tick sounding at page time `timeStampMs - offsetMs`, or null if the transport was not running there or no sample reaches it. */
    tickAt(timeStampMs: number, offsetMs?: number): number | null;
  }
  ```

  Between two samples it interpolates linearly; after the last it extrapolates at that sample's tempo (`bpm x QUARTER_TICKS / 60000` ticks a millisecond; `QUARTER_TICKS` is 48, `src/click.ts`). A sample carries `running`, so a stopped transport gives `null` and not a
  moving tick. The page time of a sample is `time.toPage(positionFrame / sampleRate)`, as `app.ts` already computes for the follower. The browser's `timeStamp` is assumed to be on `performance.now()`'s clock, which the S3 report's "Clock domain" line checks on a real port (`ABLETON.md` section 7).
  HW-25's 0.5 tick is 5.2 ms at 120 BPM; the first green run sets the number for a tempo change inside one panel interval, with its margin, and it is only tightened.
- **Thru (HW-23, HW-24).** `thru.ts` is a consumer. A note, controller, bend or pressure message on a matching input channel becomes the same message on the chosen device and channel, **sent at once** through `deliver` (no lookahead; the table records it, so Panic and a lost input release it).
  `thruRefusal(inputName, deviceName, confirmed)` returns a sentence unless the person confirmed a same-name route. Off until a rig configures it.

## 10. The built-in sound steps aside (H5)

`NoteEvent` gains `port` (`MidiEvent` already has it; `worklet.ts` passes the decoded events). `Monitor` gains a mask, `ports`, both on by default, and `render` skips an event whose port is off. The notes a masked port had sounding are released when the
mask changes (each voice records its port), so a Note Off the mask now hides cannot leave a voice running. `ToWorklet` gains `{ type: "monitorPorts"; mask: number }` (bit 0 port 1, bit 1 port 2), audio only and no panel message, like `monitor` and `drums`.
`app.ts` sets the mask from the rig: a port with a device is off unless that device's "also play here" is on. The events the monitor reads are the events the scheduler gets, byte for byte (HW-27). The drums follow the mask because they are channel 10 notes on a port.

## 11. The stand-in (test support: the Forge, with each slice that needs it)

`test/support/receiver.ts` grows, and every belief it adds is a named comment (R-10):

| Addition | For | The belief it encodes |
|---|---|---|
| `heldEventually()` | HW-11, HW-12 | the queue applied in timestamp, then send, order to a copy of `held` |
| CC 123 and CC 120 clear a channel's held notes | HW-11, HW-12 | a device obeys them. (The Stop property test uses a receiver that **ignores** CC 123, the stricter one; both stay available through an option) |
| `latencyMs` | HW-9 | the device sounds this long after it receives a message |
| `throwOnSend` (boolean or a function of the call count) | HW-14 | a send can throw (a closed or disconnected port) |
| a position register: 0xFA sets 0 and runs, 0xFB runs, 0xFC stops, 0xF8 advances 2 ticks when running, 0xF2 sets `units x 12` | HW-19 | a drum machine follows Song Position and Start, and a Continue keeps the place |
| `FakeMidiAccess` implementing `MidiAccessLike`: `plug(port)`, `unplug(id)` firing `onstatechange` | HW-4 | ports come and go; an id can change; two ports can share a name |
| a worklet stand-in that can go silent | HW-16 | a stalled worklet posts no `panel` |
| `heldBy(device)` per output in `Session` | HW-2, HW-13 | each device has its own held set |

Generated runs use `Rng` and `Session` as `stop-property.test.ts` does, with the seed in the failure message.

## 12. The follow-run report (H0b)

`spikes/s3/analyse.ts` computes `STEP_MS_AT_1_BPM = 3750`, "a step 1.5 pulses", which is the step before D0 (2026-10-07). A step is 12 ticks, 6 pulses, `15000 / bpm` ms: 125 ms at 120 BPM. So the S3 report prints "a fifth of a step (6.25 ms at 120.0 BPM)" where
a fifth of a step is **25 ms**, and its verdict reads each run against 6.25 ms, which is one twentieth of a step. `test/s3-analyse.test.ts` pins 6.25 in two tests (one comment there says "a step is 31.25 ms"), and a comment in `test/browser/s3.browser.test.ts` repeats it. The fix is not a loosening: the label said "a fifth of a step" and was
wrong about the step. Which figure is the criterion is the owner's (README Q-8-13); the slice (plan.md, H0b) makes the report **state both** and take its step from the code's constants, so it cannot go stale again:

- `abi.ts` exports `STEP_TICKS = 12` with a test that reads `crates/octocore/src/domain.rs` for `DEFAULT_STEP_TICKS`, as `abi.test.ts` already does for `TICKS_PER_CLOCK`.
- The analysis takes `stepMs(bpm) = STEP_TICKS * 60000 / (QUARTER_TICKS * bpm)` and reports the bound as a fraction of a step **and** in milliseconds at the run's tempo, and the share of samples within 6.25 ms as information.
- The verdict line reads against the fraction (default; README Q-8-13), says what the figure is at the run's tempo, and names the other.

## 13. The owner's two units as a rig

**Default for Q-8-12 (a): notes only.** The two units get notes and nothing else from the page's clock. A rig for the owner's pair, with `<...>` for what only the owner's computer can say:

```json
{
  "version": 1,
  "name": "Desk: Super 6 and Mother-32",
  "devices": [
    { "key": "super6", "label": "Super 6", "outId": "<port id>", "outName": "<as the browser lists it>", "trimMs": 0, "clock": false, "transport": false, "panic": "cut" },
    { "key": "mother32", "label": "Mother-32", "outId": "<port id>", "outName": "<the interface's port that reaches its MIDI In>", "trimMs": 0, "clock": false, "transport": false, "panic": "cut" }
  ],
  "routes": [
    { "port": 1, "from": 1, "to": 16, "device": "super6", "channel": "same" },
    { "port": 2, "from": 1, "to": 16, "device": "mother32", "channel": "same" }
  ],
  "input": null
}
```

- **Before H4 exists** the flags are not read. The default (a) is then already today's behaviour with the clock state left at **Off**: no Clock, Start or Stop is sent. The flags matter the moment Master Clock is on for something else (Live or a drum machine) while these two are chosen outputs, because
  today the clock goes to every chosen device. That is what H4 fixes, and until it lands the answer is to leave the clock Off when the two units are chosen.
- **The channels.** A track reaches port 1 on channels 1 to 16 and port 2 on 17 to 32 (`route.ts`). With the rig above, a track on channel 1 plays the Super 6 and a track on channel 17 plays the Mother-32 on its channel 1 (its default, per Moog's addendum; the owner confirms the unit's own setting, `bench.md` section 0). Until Track zoom exists, `?route=0:1,1:17` sets that.
  The Mother-32 is one voice: a track for it plays single notes (`bench.md`, B8).
- **Not sent under (a):** Clock, Start, Continue, Stop (flags off), Song Position. Still sent: notes, a track's controller, and Stop's CC 123 (the engine's, as the manual says).

**If the owner answers (b)** for a unit, the unit's flags are on and the unit's own sequencer or arpeggiator runs from our clock. The consequences, each from the makers' own text where it reached me (README 2.3, second-hand): the Mother-32's Follow MIDI Clock and Follow MIDI Start Stop are on by default, so it follows once it is sent them; the Super 6 switches to
clock-receive by itself on seeing a clock and then follows Start and Stop, and its arpeggiator and sequencer do not run without one; by user reports (not Moog) a Mother-32 that is running its own sequencer ignores incoming notes. So for a unit in (b) the rig should have **no route to it** for notes (a device with no routes is valid, section 6), or the person
mutes the track that reaches it. **The Super 6's arpeggiator from this page** is a (b)-case: it needs `clock` on for that unit, and the arpeggiator then plays the chords the page sends it. Default answer: not used. B6 and B7 are the runs that measure (b).

## 14. What this changes in the README, and what it does not

Changes to the r0.2 wording, each proposed here for the owner to see (the README carries the same list in its revision history):

1. **4.4, the sounding table is a set, not counts**, because MIDI's Note Off names a key and the stand-in (and a synthesiser) forget a retriggered key on one Note Off; a count would disagree with it.
2. **HW-11 compares the table with the stand-in's eventual held set**, after its queue is applied, not its instantaneous one (section 4).
3. **4.4, Release also lifts a sustain the page itself put down**, because a Note Off does not end a note while sustain is down. Cut still does all 16 channels.
4. **The `panic` field of a device is defined** (`"cut"` or `"release"`); the README names it and gives no values.
5. **`outManufacturer` is added to a device**, because the binding rule names manufacturer and the port type had no such field.
6. **A device with no routes is valid.**
7. **A new observable, HW-28**, and a new slice **H0b**, and a new question **Q-8-13** (section 12).

It changes no engine, ABI, contract, fixture, threshold or baseline entry, and approves nothing.

## 15. Risks the build adds

| Risk | Mitigation |
|---|---|
| The `deliver` guard hides a real bug by containing it | `stats.failed` and the sentence; the test for HW-14 asserts the sentence appears exactly once |
| A trim step collapses stamps onto one instant | Stated limit (section 7); HW-7 asserts order; trims are set in set-up, not in performance |
| The watchdog false-alarms in a busy page | The `skipped` rule and two-check condition; thresholds are proposals tuned at B2 |
| Parked tables grow | At most 8; the oldest dropped |
| `host.ts`, `wire.ts`, `worklet.ts` and `protocol.ts` change together for the Song Position field | One slice (H4), the existing `wire.test.ts`, `worklet.test.ts` and `Session` run unchanged first |
| H3 and H4 are High tier | The Referee reviews (`docs/08`), and each is its own pull request |

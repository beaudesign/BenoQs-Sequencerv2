# octoweb-abi/1

The functions the page calls on the WebAssembly module `octoweb` (ADR-0008). The module has no
imports. Every function takes and returns numbers; strings and tables cross as bytes in the
module's own memory. `tests/source_rules.rs` fails if this table and `src/exports.rs` list
different functions, and `tests/abi.rs` pins the byte layouts below. A breaking change bumps the
number `octoweb_abi` returns.

Build the module the app ships with `--no-default-features`. The default features add the
counting allocator (`measure`) and the pattern runner (`spike`) that spike S2 and the native
tests use; both are absent from the shipped module.

## Exports

A result of `0` is success. Other codes: 1 not initialised, 2 unknown input kind, 3 the layout text
is malformed, 4 the layout is not one the controller accepts, 5 a number out of range.

| Export | Does |
|---|---|
| `octoweb_abi() -> u32` | The ABI number, 1 |
| `octoweb_scratch(len) -> ptr` | At least `len` bytes of scratch memory, and the address of the first. Valid until the next call to this function. The page writes the layout here, and reads messages from here |
| `octoweb_init(sample_rate, seed_lo, seed_hi, layout_len) -> code` | Makes the engine and the panel controller. Reads the layout from the first `layout_len` bytes of scratch: one `<n> <id>` per line, the `n` and `id` of each control in `contracts/controls.json`. On failure the module is left uninitialised, and the reason is in scratch, `octoweb_message_len` bytes of UTF-8 |
| `octoweb_message_len() -> u32` | The length of the message the last `octoweb_init` left in scratch. 0 after a success |
| `octoweb_reset() -> code` | Drops the engine and the panel. Everything is then "not initialised" until the next `octoweb_init` |
| `octoweb_input(now_ms, kind, control, detents) -> code` | One panel input. `kind` 0 is a press, 1 a release, 2 an encoder turn of `detents`. `control` is a control number. A number the controller does not use is ignored and is not an error. `now_ms` is the page's time in milliseconds |
| `octoweb_transport(play) -> code` | Starts (`play` nonzero) or stops the transport. The page is the host for this until the controller builds the transport workflow (ADR-0008 decision 4) |
| `octoweb_set_tempo(bpm) -> code` | The tempo the next renders use. Refused outside 1 to 999 and for non-finite values |
| `octoweb_set_track(track, attr, value) -> code` | Writes one attribute of one track through the engine's own `SetTrack`. `track` is 0 to 9. `attr` is the attribute's place in the engine's list: 0 Pitch, 1 Velocity, 2 LengthFactor, 3 StartFactor, 4 DirectionRaw, 5 Rotation, 6 Amount, 7 Groove, **8 MidiChannel**, 9 Muted, 10 Soloed, 11 Paused, 12 RecordArmed, 13 IsFeeder, 14 IsListener. A track or attribute that does not exist is code 5 and changes nothing. The value is the engine's to clamp: a `MidiChannel` of 1 to 16 is port 1 and 17 to 32 is port 2. Added in P4b; the ABI number stays 1 (a page built for 1 never calls it) |
| `octoweb_render(frames) -> count` | Renders `frames` frames (at most 4096; the page asks for 128) and returns how many events are now in the events buffer. Events carry sample offsets inside this block |
| `octoweb_events() -> ptr` | The events buffer: 256 records of 12 bytes, valid for the first `count` of the last render. Null before `octoweb_init` |
| `octoweb_refresh_leds() -> u32` | Recomputes the LED frame from the controller and the engine's page. Returns 1 if any LED changed since the last call, else 0 |
| `octoweb_leds() -> ptr` | The LED frame: 512 bytes, indexed by control number. Null before `octoweb_init` |
| `octoweb_playheads() -> ptr` | 10 bytes: the step (0 to 15) each track is on, updated by `octoweb_render`. Null before `octoweb_init` |
| `octoweb_status() -> u32` | Bit 0: the transport is running. Bit 1: the panel is in Step zoom |
| `octoweb_dropped_intents() -> u32` | How many panel intents (Audition, PLAY snapshots) the engine has no capability for. They are counted and not applied; the engine side is a request to the Metronome |
| `octoweb_alloc_count() -> u32` | `measure` feature only. Allocations on this thread since it started |
| `octoweb_spike_run(len) -> u32` | `spike` feature only. Runs the pattern in the first `len` bytes of scratch through the headless runner and leaves the result in scratch: 64 hex digits of the event log's SHA-256, or an error message. The return value is the result's length, with bit 31 set for an error message |

## Buffers

All buffers belong to the module, are never reallocated, and stay valid until `octoweb_init` or
`octoweb_reset`. Read them through a fresh `DataView` or `Uint8Array` on `memory.buffer` each time,
because memory can grow.

**An event** is 12 bytes: `kind`, `port`, `channel`, `d1`, `d2` (16 bits, little endian), two zero
bytes, `at_sample` (32 bits, little endian, the offset inside the block).

| Kind | Event | `d1` | `d2` |
|---|---|---|---|
| 0 | Note On | note | velocity |
| 1 | Note Off | note | 0 |
| 2 | Control change | controller | value |
| 3 | Pitch bend | 0 | the 14-bit value, 8192 centre |
| 4 | Channel pressure | the value | 0 |

`port` is 1 or 2 and `channel` 1 to 16, as the engine numbers them.

**An LED** is one byte: the colour in the low two bits (0 off, 1 red, 2 green, 3 orange) and the
phase in the next two (0 steady, 1 flash, 2 shine). Red, green and orange are roles, not hues.

## The calls a worklet makes

At start, in the worklet constructor: `octoweb_scratch`, write the layout, `octoweb_init`. Then, on
each 128-frame quantum: `octoweb_render(128)` and read the events. On each message from the page:
`octoweb_input`, `octoweb_transport`, `octoweb_set_tempo` or `octoweb_set_track`. About every 16 ms:
`octoweb_refresh_leds` and, if it returned 1, send the LED frame to the page.

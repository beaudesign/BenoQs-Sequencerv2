# octoffi

C ABI boundary over `octocore`. Owner: Conductor.

`octocore_engine_new/free/handle_command/render/is_running`, the read-only
`octocore_engine_diagnostics` (counters for events the engine refused, deferred or sent
late, `OctoDiagnostics` in the header) plus a
Grid-mutation surface (`octocore_track_set_i32`/`get_i32`,
`octocore_step_set_i32`/`get_i32`) — see `octoffi.h` for the C-side
declarations (hand-written; no `cbindgen` in this dev environment yet, so
keep it in sync by hand with `src/lib.rs`) and `journal/conductor/`'s dated
entries for how each was verified (a real C program, compiled and linked
against the built `liboctoffi.dylib`, not just Rust's own tests).

The Grid-mutation surface addresses by logical (track, step) index, not a
physical panel `ControlId` — programming a pattern doesn't need to know
where a control lives on the panel, so this didn't actually need
`panel.truth.json` to exist first (an earlier note here said otherwise; see
`lib.rs`'s module comment on the surface for the reasoning). What still
needs `ControlId` is *physical actuation* — `Command::ButtonDown/Up/
EncoderTurn` remain no-ops, since there's no coordinate system yet to
resolve a control id against.

`octocore_engine_render` writes at most `out_capacity` events per call. Events that do not
fit wait for the next call and are counted in `deferred_events`; none is dropped for that
reason. The event union has five variants: `OCTO_EVT_NOTE_ON`, `_NOTE_OFF`, `_CC`,
`_PITCH_BEND` (14-bit `value`) and `_CHANNEL_PRESSURE` (7-bit `value`). The layout is
pinned by tests in `src/lib.rs` that check sizes and offsets from the Rust side. They do not
parse `octoffi.h`, so the header is still kept in sync by hand.

## The link: commands in, snapshots out (SPEC-0001 O6)

The engine is single-threaded, and the grid setters above write it in place, so they are for
offline use and tests, never while another thread renders. A host that edits a running
engine uses `octocore_engine_open_link`, once, which returns an `OctoSender` for the main
thread and an `OctoReader` for the render thread. `octocore_sender_push` takes any
`OctoCommand`, including the new `OCTO_CMD_SET_TRACK` and `OCTO_CMD_SET_STEP`, and never
waits. The engine applies up to 256 of them at the start of each render and publishes an
`OctoSnapshot` at the end. `octocore_reader_claim` returns the newest one; compare
`generation` to see whether it is new. LED and encoder fields are zero until
`panel.truth.json` exists. `octocore_abi_version` and `OCTOFFI_ABI_VERSION` (now 2) say which
header the library was built with.

Header drift is checked two ways: `f4` keeps every line the header had before the link, and
`f1_a_c_compiler_agrees...` compiles `octoffi.h` with `cc` and `_Static_assert`s every size,
offset and tag against the Rust values. `e4` counts allocations across 1,000 rounds of push,
render and claim and expects zero.

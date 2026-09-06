# octoffi

C ABI boundary over `octocore`. Owner: Conductor.

`octocore_engine_new/free/handle_command/render/is_running` plus a
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

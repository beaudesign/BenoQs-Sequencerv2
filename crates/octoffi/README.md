# octoffi

C ABI boundary over `octocore`. Owner: Conductor.

`octocore_engine_new/free/handle_command/render/is_running` — see
`octoffi.h` for the C-side declarations (hand-written; no `cbindgen` in this
dev environment yet, so keep it in sync by hand with `src/lib.rs`) and
`journal/conductor/2026-09-06.md` for how it was verified (a real C program,
compiled and linked against the built `liboctoffi.dylib`).

No Grid-mutation surface exists yet — `Command` has no step-editing variant,
which needs `panel.truth.json`'s `ControlId` scheme first (see
`reference/NOTES.md`). Swift can start/stop transport and pull rendered
events through this boundary today; it cannot program a pattern through it
yet.

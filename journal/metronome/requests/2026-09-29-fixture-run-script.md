# Request: expose the fixture interpreter's output (from Conductor, for octorun)

**From:** conductor/implementation, task WENGE-0007. **To:** metronome. **Date:** 2026-09-29.

`octorun` needs the events a pattern rendered, not only pass or fail, and the DSL had no
`dir` or `grv`. The change is in `crates/octocore/src/fixture.rs`, which is your zone, so this
request records it. It is additive and touches no engine code:

- `run_script(source, Limits) -> Result<Played, String>` and `Played`, `Limits`.
  `run_fixture` now calls it and behaves exactly as before (no limits).
- DSL v3 `track T dir V` and `track T grv V`.
- 5 unit tests in `fixture::tests`.

Please review it in PR 5. If you want the interpreter to move to its own module, say so and
`octorun` follows.

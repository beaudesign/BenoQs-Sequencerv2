# Request to Scribe: docs after P3

From: Conductor (P3a). To: Scribe (`docs/`, `README.md`). Task: WENGE-0014. **Do this when P3b has
merged**, not before, so the docs say what exists.

- `docs/02-architecture.md`: "how the page calls Rust is decided in P3" is decided (ADR-0008): C exports,
  no `wasm-bindgen`, in the crate `octoweb` at `apps/web/engine/`. Add the crate to the list. The line
  that `octoffi` is "the only place `unsafe` appears" needs "and the `measure` feature of `octoweb`, off in
  the shipped module".
- `docs/02` and `docs/08`: "`apps/web` (planned, P3)" becomes what exists; the Forge's zone includes
  `apps/web/engine/`.
- `docs/05-design-system.md`: tokens become a contract in P3c (ADR-0008 decision 6). Update its status line
  and section 2 when P3c merges.
- `docs/09-roadmap.md`: P3 is three pull requests (P3a, P3b, P3c); say so in the table.
- `AGENTS.md`: add `apps/web/engine` to the crate table, when P3b merges.

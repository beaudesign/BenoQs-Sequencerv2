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
- `docs/02-architecture.md` section 6 (about line 190): "On Stop it calls `MIDIOutput.clear()` ... then sends ALL
  NOTES OFF" is replaced by ADR-0008 amendment 1: the page never calls `clear()` (Chromium has none; where it
  exists it drops Note Offs the engine already counted as sent), timestamps on one output never go backwards, and
  the engine's flush ends every note at most one lookahead after the key. `specs/SPEC-0002/tech.md` section 3 step 4
  already says so.
- `docs/07-verification.md`: the web gates (Node tests, Chromium tests, spike S2) run in the `web` job in
  `.github/workflows/verify.yml` and are not part of `cargo xtask verify`; say so until the Referee floors them
  (`journal/referee/requests/2026-10-01-browser-tests-and-web-gates.md`).

## Added by P3c

- `docs/05-design-system.md`: the tokens file exists (`contracts/design.tokens.json`, `design.tokens.schema.json`); update the status line
  and section 2's "will live in ... (planned with the app, P3, by ADR)". Section 2.1 gains `stroke` and `flash` in the tokens table
  (ADR-0008 amendment 2): focus ring 2 px with a 1 px inner stroke are tokens now. The "Status: proposed" line is changed when the owner
  merges P3c, which ratifies the values.
- **After P3c merges (Conductor):** change `"status": "candidate"` to `"ratified"` in `contracts/design.tokens.json`. It is a one-line
  contract change by ADR-0008 decision 6 ("by merging P3c") and needs no further ADR.
- `docs/09-roadmap.md`: P3c's exit lists E1, E2, E7, E8 and the tokens test; the roadmap's kill criterion (S1) is unchanged and still untested.
- `docs/07-verification.md`: `verify:tokens` and the source half of `verify:slop` exist as Rust tests (`apps/web/engine/tests/tokens.rs`,
  `app_source_rules.rs`); the gates themselves are the Referee's (`journal/referee/requests/2026-10-01-browser-tests-and-web-gates.md`).

# archive/native-panel

Four files kept for reference: `01-panel-truth.md` (how a photoreal panel would be measured
from photographs), `04-render-engine.md` (a native ray-traced renderer for it), and two contract
schemas, `contracts/panel.truth.schema.json` and `contracts/motion.registry.schema.json`, which
ADR-0007 retired from `contracts/`. Their replacement is `contracts/controls.json`.

They describe the native, photoreal panel path that ADR-0006 retired
(`adr/0006-web-sequencer-supersedes-the-rooms-scope.md`). The product is a web panel now
(`specs/SPEC-0002/`, decision D3), and nothing in the build reads these files. They stay only
in case photo-exact fidelity is ever wanted again; if it is, it comes back by a new spec and
an ADR, not by editing these.

The scope gate (`verify:scope`) allows this directory, so these files still contain the old
vocabulary. Do not copy text out of them into live documents.

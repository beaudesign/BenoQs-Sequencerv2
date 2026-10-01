# Request to Panelwright: the chase-light, and the transport Play key

From: Conductor (P3a). To: Panelwright (`crates/octoface/`, `contracts/controls.json`, the panel
fixtures). Task: WENGE-0014 (`specs/SPEC-0002/p3-plan.md`, decisions D-P3-5 and D-P3-6).

The first browser panel (P3c) draws what the controller's LED frame says and nothing else, so two
things the instrument obviously does are missing from it until the controller does them.

## 1. The chase-light

What the manual says: a Red chase-light runs along a track's row while the sequencer plays (p052);
it is Orange rather than Red when the track sends an MCC (p017); a skipped step is ignored by it and it
moves to the next unskipped step (p014); in a page cluster you can follow it (p084).

What it does not say, and so what the fixtures should be `pending` on, with a question number in
`tests/conformance/panel/QUESTIONS.md`:

- Whether the chase-light is drawn in Page view on every playing track or only in the zoomed one (p052
  describes attribute maps).
- How it is drawn over a lit Green step, a Red skipped step and an Orange step with an event: replacing the
  step's colour, or sharing the key.
- Whether it moves with the track's own direction and multiplier, which is the engine's business, or only
  steps forward (p056 says the chaser "is moved according to the triggers").

What the controller needs from the engine: each track's play position (`Snapshot::playheads` has it) in
the page view. The engine's read model is already a request to the Metronome
(`journal/metronome/requests/2026-09-30-engine-read-model-and-panel-intents.md`). The page view can take
the positions from `Engine::snapshot` meanwhile, as `PageView::from_page` takes steps from `Engine::grid`.

## 2. The transport Play key (`transport.play`, n 256)

`octoface` handles `transport.stop` (p049) and not the Play key. The page sends the engine's `Play`
command itself for `transport.play` as a stopgap (ADR-0008 decision 4). When the controller builds the
transport workflow the page drops its mapping and nothing else changes. Pages to start from: p016
(the transport bar), p049, p067 (Record). Also in the workflow: what Play does while already
playing, and whether Pause (`transport.pause`) is Continue (`Command::Continue` exists).

Neither item blocks P3.

## Added by P3c: what the first panel left provisional

The first panel is in P3c. Three things in it wait on the manual or on a decision, and none blocks anything.

1. **Where the other controls go.** `apps/web/layout/panel.layout.json` places the matrix as the manual says and puts the eleven other
   controls it draws in rows under it, grouped by `controls.json` zone, with `status: provisional` and a note naming Q23 (what the MODE block
   holds), Q04 (the EDIT block's order) and Q26 (whether the transport keys have LEDs). When a hardware check or a page settles a zone,
   the arrangement in that file changes (cells only; it holds no length) and the tab order follows it. The layout file is the Forge's;
   what a zone holds is yours.
2. **`shine`** is drawn as the lit colour with a ring in `ink` (`flash.shine` in the tokens, pending Q03), so it is told from steady by
   shape and not by a new colour. The controller emits it today only in the Step zoom table (p014).
3. **The flash rate and duty cycle** are 500 ms and half (tokens, pending `tech.md` 9.2). They are one line each in the tokens file when
   the manual or a hardware check says otherwise.

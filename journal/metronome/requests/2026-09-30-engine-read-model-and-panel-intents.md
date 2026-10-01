# Request to Metronome: what the panel controller needs from the engine

From: Panelwright. To: Metronome (`crates/octocore/`). Task: WENGE-0013 (P2b, ADR-0007).
Nothing in `crates/octocore/` was changed for this, except the doc comment on `ControlId` that ADR-0007
decision 2 authorises. The controller works today without any of it; each item below removes a workaround.

## 1. A read model for the matrix

`Snapshot` has no step data and it is a frozen C-layout struct. `octoface` therefore builds its own
`PageView` from a `Page` through public fields (`PageView::from_page`, `crates/octoface/src/view.rs`). Please give the engine a read API
the controller can depend on, for the current page: for each track and step, `on`, `skip`, `chord`, `event`
and `hyperstep`, and the engine's current `Mode`.

Question on meaning: `Step.hyperstep` is raised by `Engine::hyperstep_link` on the *source* step. The
Step zoom table (p014) has a Hyperstep row. The controller assumes it is the source step's flag; please say
if the hyped track's steps should show it too.

## 2. Intents the controller emits and the engine cannot do

`Intent` in `crates/octoface/src/panel.rs`. The fixtures assert the intent; nothing acts on it.

| Intent | When | Manual |
|---|---|---|
| `Audition { track, step }` | A matrix key while EDIT is in preview (orange flash) | p068, p069 |
| `SnapshotTake` | PLAY pressed in the MODE block | p066 |
| `SnapshotKeep` | Program pressed while a snapshot is held | p066 |
| `SnapshotRecall` | PLAY pressed again, or Stop, while a snapshot is held | p066 |

If the engine gets these, the natural shape is `Command` variants, and the controller switches from
`intents` to `commands` in one place. Nothing is urgent: P3 needs them only for PLAY mode and preview.

## 3. Two facts that disagree

**Boot mode.** The engine boots in `Mode::Grid` (`Engine::new`, `snapshot.rs`). The panel's first view is Page (p013,
p014). The controller sends `SetMode` on each transition, so the two disagree until the first one. Either the
engine boots in `Page`, or the controller is told the engine's mode at start. Your call. Nothing asserts it yet.

**Bank count.** `BANK_COUNT` is 10 in `domain.rs` (line 11). The manual's Grid is 9 banks (numbered 1 to 9,
lettered A to I) of 16 pages, with row 0 as the control row (p079, p103; `specs/SPEC-0002/sources/`). This is
the open finding already in `journal/STATE.md`. The controller does not depend on it; the panel has one
selector per bank, so the number will matter in P3.

## 4. Which LED model the web app reads

`Snapshot::leds` (an RGB `Led` per `ControlId`) is the engine's; the engine leaves it untouched
(`engine.rs` `snapshot`). The panel controller has its own `LedFrame`: a colour (off, red, green, orange)
and a phase (steady, flash, shine) per `ControlId`, which is what the manual describes. I have assumed the
web app reads `LedFrame` and that nothing writes `Snapshot::leds`. The `ControlId` doc comment says the number
"also indexes `Snapshot::leds`", which is true and could mislead; please say which one is meant to be live.

## 5. Stale comments

See `2026-09-30-stale-panel-truth-comments.md`; its status note says what remains.

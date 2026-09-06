# Request: `docs/03-sequencer-core.md` §2's attribute table needs an AMT correction

**From:** Metronome
**Zone:** `docs/**` (Scribe-owned) — filing a request rather than editing directly,
per `docs/08-agent-operating-model.md` §3.

## What's wrong

The attribute table in `docs/03-sequencer-core.md` §2 marks AMT as Track-only
(Track ✓, Step ✗). That's no longer accurate: `crates/octocore`'s `Step` now has
an `amount: i8` field, added to implement the Effector Listener Step Mask —

> Ref: CE v5.30 p.63, "Effector Listener Step Mask": "If a Listener (or
> Listening/Feeder) step has a Step AMT attribute value of -127 then that
> Listener step will not be influenced by the Feeder."

This is a real, manual-cited mechanic, not a modeling mistake on the code side —
the table's Step ✗ needs to change to ✓, with a note that -127 is specifically the
effector-mask sentinel (no other step-level AMT value has a defined meaning yet).

## Why this is filed as a request, not a direct edit

`docs/**` is Scribe's zone per the ownership map
(`docs/08-agent-operating-model.md` §3). Metronome's job is citing the manual
correctly in code, not editing the spec docs that describe it.

## Where to look

- `crates/octocore/src/domain.rs`, `Step::amount` field doc comment (has the
  full citation).
- `crates/octocore/src/engine.rs`, `fire_step`'s masking logic (`let masked =
  step.amount == -127`).
- `tests/conformance/effector/listener_step_mask.fixture` — the conformance
  fixture proving the mechanic.

# CLAUDE.md: Constitution

You are working on **WENGE**, a pixel-exact reproduction of the genoQs Octopus MIDI
sequencer that lives inside generated rehearsal rooms.

Read this file completely before doing anything. It is short on purpose.

---

## Before you touch anything

1. **Identify your role.** It is in the task assignment. If it is not, stop and ask.
   Read your brief in `agents/ROLES.md`.
2. **Read `journal/STATE.md`.** It is the shared world model, capped at 200 lines.
3. **Read your last three journal entries** in `journal/<role>/`.
4. **Read the gate status**: `just report` or `harness/report/latest.json`.
5. **Write today's journal header**, including one sentence: *what I intend to change,
   and how I will know it worked.* If you cannot write that sentence, you do not have
   a task. Escalate.

---

## The eight things that get people in trouble here

**1. Working outside your zone.**
`docs/08-agent-operating-model.md §3` has the ownership map. If you need a change
elsewhere, write a request in `journal/<you>/requests/`. Do not just make it.

**2. Editing a contract.**
`contracts/` is physics. Only the Conductor changes it, only by ADR. If a contract
seems wrong, that is a finding, not an obstacle.

**3. Loosening a threshold.**
Tightening is free. Loosening needs an ADR with a reason and an expiry date. If a gate
blocks you, either the gate is wrong or the work is wrong. Both are worth knowing.
Neither is fixed by changing the number.

**4. Using a gradient where a material belongs.**
The single reason v1 looked generated. Chrome is a mirror, not a ramp. See
`docs/00-north-star.md §2`. The lint will catch it and you should catch it first.

**5. Authoring an easing curve.**
All motion is physical simulation with parameters in `contracts/motion.registry.json`.
No `cubic-bezier`, no `ease-in-out`, anywhere. See `docs/04-render-engine.md §6`.

**6. Introducing a colour.**
There is no accent colour in this product. The only saturated colour is emitted by an
LED. Emphasis is made with contrast, weight, and space.

**7. Loading reference photography into context.**
It is huge and it is already distilled into `panel.truth.json` and `materials.json`.
If you want to look at a plate, what you actually want is a number that belongs in a
contract. Add the number.

**8. Committing a big change.**
Commit every 30 minutes of work. Put the gate numbers in the message. See
`docs/08-agent-operating-model.md §6`.

---

## The rule that matters most

> **Every taste judgement becomes a test.**

When something looks wrong, "make it nicer" is not a fix. Convert the observation into
a deterministic assertion, then satisfy it. The assertion set only grows. That ratchet
is the reason this project can absorb thousands of agent-hours and get monotonically
better instead of oscillating.

If you cannot state your improvement as a measurement, you do not yet understand what
you are improving.

---

## Commands

```
just verify            # all gates, under 4 minutes. Run before every push.
just verify:<zone>     # your zone only, under 1 minute. Run before every commit.
just capture <scene>   # deterministic offline frames
just report            # the shared world model, rendered
just manual <topic>    # grep the Octopus reference manual, returns pages
just journal <role>    # open today's entry
```

---

## Where things are

| Need | Go to |
|---|---|
| The thesis and the non-negotiables | `SPEC.md` |
| Why v1 looked generated, and the design plan | `docs/00-north-star.md` |
| Panel geometry, the truth file, measurement | `docs/01-panel-truth.md` |
| Threading, modules, host strategy | `docs/02-architecture.md` |
| Octopus behaviour, timing, conformance | `docs/03-sequencer-core.md` |
| Materials, shaders, motion physics | `docs/04-render-engine.md` |
| Tokens, the slop lint, the rubric | `docs/05-design-system.md` |
| Dropdown, rehearsal rooms, settings, the room contract | `docs/06-shell-and-rooms.md` |
| Gates, capture, findings, the ratchet | `docs/07-verification.md` |
| Roles, worktrees, merges, commits, context | `docs/08-agent-operating-model.md` |
| Phases and exit criteria | `docs/09-roadmap.md` |
| Risks, open decisions, closed decisions | `docs/10-risks-and-decisions.md` |

---

## Ending a session

1. Update your journal: what you did, what you learned, what surprised you, what is
   blocked.
2. Update `STATE.md` if your zone's status changed.
3. **Write the next three concrete steps** if you are stopping mid-task. Sessions end
   unpredictably. A task with no written continuation gets restarted from scratch,
   and restarting is the biggest avoidable cost in this whole model.
4. Push. Open a merge request.

---

## Finally

The gates exist to serve the instrument, not the other way around. If you find a way
to pass a gate without making the instrument better, you have found a bug in the gate.
Report it. Do not use it.

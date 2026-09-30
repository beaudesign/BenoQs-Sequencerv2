# CLAUDE.md: Constitution

You are working on **WENGE**, a faithful reproduction of the genoQs Octopus MIDI
sequencer as a web app that plays live MIDI to external instruments and can be routed into
Ableton Live. The product is specified in `specs/SPEC-0002/`, and `adr/0006` records the
decision that made it so.

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

## The nine things that get people in trouble here

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

**4. Using a gradient where a flat fill belongs.**
A gradient standing in for a material is what made v1 look generated. The panel is a
control surface, not a picture of metal: flat fills, hairlines, one neutral palette. See
`docs/00-north-star.md §2`. The `slop` gate will catch it once it is built, and you should
catch it first.

**5. Authoring an easing curve.**
Motion in the panel is state: an LED is off, on or flashing at the manual's rate, and a
button is up or down. No `cubic-bezier`, no `ease-in-out`, anywhere. If something seems to
need to move, it is a state change, with its number in a contract. See
`docs/05-design-system.md`.

**6. Introducing a colour.**
There is no accent colour in this product. The only saturated colour is emitted by an
LED. Emphasis is made with contrast, weight, and space.

**7. Loading the manual PDF or reference photography into context.**
They are huge and already distilled: the manual into `specs/SPEC-0002/sources/` and the
fixtures that cite it by page. If you want to read a page, what you actually want is a
fact that belongs in a fixture. Add the fixture. `just manual <topic>` returns the page
you need.

**8. Committing a big change.**
Commit every 30 minutes of work. Put the gate numbers in the message. See
`docs/08-agent-operating-model.md §6`.

**9. Committing without pushing.**
A commit that stays local is invisible to everyone but you — the human
tracks progress through GitHub, not your working tree. This has already gone
wrong once: a long session made 22 commits and pushed none of them until
asked, and from the outside it looked like nothing had happened at all. Push
after every commit (or small batch of them), and before you report a
summary or go quiet for a while. See the `commit-and-push` skill.

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
just verify-<zone>     # your zone only, under 1 minute. Run before every commit.
just report            # the shared world model, rendered
just manual <topic>    # grep the Octopus reference manual, returns pages
just journal <role>    # open today's entry
```

---

## Where things are

| Need | Go to |
|---|---|
| The thesis and the non-negotiables | `SPEC.md` |
| Task lifecycle, risk tiers, handoffs, review checklist | `AGENTS.md` |
| The product, its decisions and the removal plan | `specs/SPEC-0002/` |
| Why v1 looked generated, and the principles of the panel | `docs/00-north-star.md` |
| Threading, modules, the web architecture | `docs/02-architecture.md` |
| Octopus behaviour, timing, conformance | `docs/03-sequencer-core.md` |
| LED roles, the palette, the slop rules | `docs/05-design-system.md` |
| Gates, findings, the ratchet | `docs/07-verification.md` |
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

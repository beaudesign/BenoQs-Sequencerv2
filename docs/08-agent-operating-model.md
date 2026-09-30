# 08: Agent Operating Model

**Owner:** Conductor. **Non-optional reading for every role.**

---

## 1. The efficiency thesis

Multi-agent work fails for one reason above all others: **shared mutable surfaces**.
Two agents edit adjacent things, both are individually correct, the merge is a mess,
and the cost of reconciling exceeds the cost of having done it serially. Add more
agents and it gets worse, not better.

The countermeasures are all versions of the same idea: make the interfaces narrow,
frozen, and machine-checked, so that agents can work in genuine isolation and their
work composes.

1. **Contracts before code.** The files in `contracts/` define every boundary.
   They are edited by the Conductor only, via ADR. Everyone else treats them as
   physics.
2. **One zone, one owner.** Every path in the repo maps to exactly one role. Nobody
   edits outside their zone without a cross-zone request.
3. **One worktree per agent.** Physical isolation, not just social convention.
4. **Verification is local.** Every zone has `just verify-<zone>` that runs in under
   a minute, so an agent learns whether it is right without waiting for anyone.
5. **Communication is artefacts.** Journals and the report, not conversation. An
   agent that needs to ask another agent a question has found a missing contract.

---

## 2. Roles

Seven. Each has a brief in `agents/ROLES.md`. A role is a *hat*, not a person: one
model instance may wear several hats sequentially, but never two at once, because the
value of the role system is that it constrains what you are allowed to touch.

| Role | Owns | Primary artefact |
|---|---|---|
| **Conductor** | `contracts/`, `adr/`, `justfile`, `crates/octoffi/` (frozen), roadmap, merge queue | decisions |
| **Panelwright** | `reference/`, `crates/octoface/` (planned, P2); drafts `contracts/controls.json` (planned, P2), which the Conductor changes by ADR | the panel controller and its control inventory |
| **Forge** | `apps/web/` (planned, P3) | the web app |
| **Metronome** | `crates/octocore/`, `tests/conformance/`, `hosts/m4l/` (the Tier 2 Ableton spike), the timing path | the sequencer |
| **Curator** | the visual rules (LED colour roles, the one neutral palette), `docs/05` | taste, made explicit |
| **Referee** | `harness/`, `xtask/`, CI, all thresholds, the `scope` and `a11y` gates | the ratchet |
| **Scribe** | `docs/`, `README`, `CHANGELOG` | the record |

Roles have a second axis, the stage a task is in (coordinator, spec, implementation,
review, verification), plus risk tiers and handoff records. Those live in the root
`AGENTS.md` (ADR-0004).

Two rules about roles:

- **The Curator does not build and the Forge does not judge.** Separating the
  producing role from the evaluating role is the reason a judgement means anything.
  An agent that both makes and grades its own work converges on whatever it finds
  easy.
- **The Referee owns thresholds and nobody else may change them.** Tightening is
  encouraged and requires no approval. Loosening requires an ADR co-signed by the
  Conductor with an expiry date.

---

## 3. Ownership map

`CODEOWNERS`-style, enforced in the merge queue:

```
/contracts/**            @conductor
/adr/**                  @conductor
/justfile                @conductor
/reference/**            @panelwright
/crates/octoface/**      @panelwright    # planned, P2
/harness/**              @referee
/xtask/**                @referee
/crates/octocore/**      @metronome
/crates/octoffi/**       @conductor
/hosts/m4l/**            @metronome
/apps/web/**             @forge          # planned, P3
/tests/conformance/**    @metronome
/tests/golden/**         @referee
/docs/**                 @scribe
/docs/05-design-system.md @curator
/journal/<role>/**       @<role>
```

**Cross-zone requests.** When agent A needs a change in agent B's zone, A does not
make it. A writes `journal/<A>/requests/<date>-<slug>.md` describing what is needed
and why, and B picks it up. This costs a cycle of latency and saves a day of merge
archaeology. If the same cross-zone request recurs, the boundary is in the wrong
place and the Conductor moves it.

---

## 4. Parallelism

### Worktrees

Every agent works in its own git worktree off `main`:

```bash
git worktree add ../wt-forge     -b forge/<slug>     main
git worktree add ../wt-metronome -b metronome/<slug> main
```

No agent ever has `main` checked out. This eliminates the entire class of accidents
where an agent commits to the wrong branch or stashes over another agent's work.

### What can run at once

Wave-dependent. The Conductor publishes the current fan-out in `journal/STATE.md`.
An illustrative fan-out, once `contracts/controls.json` is frozen:

```
  ┌── Panelwright: the next workflow fixtures for `octoface`          (independent)
  ├── Forge:       the SVG panel, drawn from the inventory            (needs controls.json: frozen)
  ├── Metronome:   the clock-follow estimator and its fixtures        (independent)
  ├── Curator:     the LED colour roles                               (independent)
  ├── Referee:     the `a11y` gate                                    (needs the control names: frozen)
  └── Scribe:      manual index fixes (MIDI, load and save chapters)  (independent)
```

Six in parallel with zero contention, because every dependency runs through a frozen
contract. When a contract needs to change mid-phase, the Conductor freezes the fan-out
first, lands the ADR, then re-fans. Do not change a contract with agents live against
it.

### Where parallelism does not help

Be honest about this. Some work is irreducibly serial and adding agents makes it
worse:

- **The first vertical slice.** Wave 1 is deliberately narrow and mostly serial. Fan
  out after it lands, not before.
- **Anything touching the FFI boundary.** One at a time.
- **Threshold changes.** Serial by construction.

---

## 5. Session protocol

Every agent session, without exception.

### Start

1. `git pull` and read `journal/STATE.md`. This is the world.
2. Read your own last three journal entries.
3. Read `harness/report/index.html` (or the JSON) for current gate status.
4. Read the open findings tagged with your zone.
5. Write today's entry header, including **one sentence stating what you intend to
   change and how you will know it worked.**

Step 5 is not ceremony. An agent that cannot write that sentence does not have a task,
and the correct action is to escalate rather than to start typing.

### During

- Work in your zone only.
- Commit every meaningful step (see §6).
- Run `just verify:<zone>` before every commit and `just verify` before every push.

### End

1. Update your journal entry: what you did, what you learned, what surprised you,
   what is now blocked.
2. Update `journal/STATE.md` if your zone's status changed.
3. Push the branch and open a merge request.
4. **If you are stopping mid-task, write down the next three concrete steps.**
   Sessions end unpredictably; a task with no written continuation is a task that will
   be restarted from scratch, and restarting is the largest avoidable cost in
   multi-agent work.

---

## 6. Commit protocol

Commits are frequent, small, and honest. This project generates a large amount of
change and the history is the only way anyone will reconstruct why.

**Cadence:** commit at every point where the tree is coherent, even if incomplete.
Target under 30 minutes of work per commit. A commit that changes 800 lines across
four concerns cannot be reviewed, cannot be reverted, and cannot be bisected.

**Format:**

```
<zone>: <imperative summary under 60 chars>

<what changed and why, wrapped at 72>

Verify: <which gates ran, and their numbers>
Journal: journal/<role>/<date>.md#<anchor>
Refs: <ADR / finding / manual page>
```

Example:

```
metronome: flush sounding notes on Stop and Reset

The engine keeps a 4 KB table of notes the receivers are holding.
Stop and Reset now emit a NoteOff per NoteOn still owed, then CC 123
on each channel that had one, at sample 0 of the next render.

Verify: cargo xtask verify exit 0, octocore 84 unit + 8 invariants
        + 1 conformance runner over 7 fixtures.
Journal: journal/metronome/2026-09-29.md
Refs: SPEC-0001 O1 (WENGE-0001), CE v5.30 p.94
```

Putting the gate numbers in the commit message is unusual and worth the friction. It
makes `git log` a performance and quality history, and it makes a regression's cause
findable by reading rather than by bisecting.

**Never commit:** a red gate on `main`, a loosened threshold without an ADR, a
generated artefact that is not content-addressed, or a `TODO` without a journal
reference.

---

## 7. Merge queue

`main` is protected. Merges are serialised through a queue the Conductor operates.

For each merge request:
1. Rebase onto `main`.
2. `just verify` green.
3. `verify:regressions` confirms assertion count did not fall.
4. Zone ownership check.
5. If any contract changed, the ADR must be merged first, as a separate commit.
6. Squash only if the branch is a single concern. Otherwise merge with history.

The queue is strictly serial and that is fine, because `just verify` runs in under
four minutes. This is why the four-minute budget in
[Architecture §7](02-architecture.md) is a hard requirement: it is what makes serial
merging viable with several agents working in parallel.

---

## 8. Context economy

Agent sessions have finite context and this project has a large surface. Managing that
is an explicit responsibility, not an afterthought.

**Never load into context:**
- Reference photography. It is large, and the web panel does not need it: the control
  inventory comes from the manual (SPEC-0002 F2). If you find yourself wanting to look
  at a plate, what you actually want is a fact that belongs in a contract, so add it.
- The full 124-page manual. It is page-indexed in `reference/manual/`; load the
  section you need. `just manual <topic>` greps and returns the relevant pages.
- Other agents' zones.

**Always load:**
- `journal/STATE.md` (short by design; the Conductor prunes it).
- Your own last three journal entries.
- Your role brief.
- The contracts relevant to your zone.

**The `STATE.md` discipline.** It is capped at 200 lines. When it exceeds that, the
Conductor prunes rather than appends. A shared state file that grows without bound
becomes a file nobody reads, and once nobody reads it the whole communication model
fails silently.

---

## 9. Escalation

Escalate to the Conductor, immediately, when:

- A contract appears wrong. Do not work around it. A worked-around contract is a
  contract that will be worked around differently by the next agent.
- A gate blocks work that seems correct. Either the gate is wrong (valuable finding)
  or the work is wrong (valuable finding). Both need the Conductor.
- The same finding recurs three cycles. The fix is structural.
- Two roles need the same change. The boundary is wrong.
- A phase's exit criteria look unreachable. Better to learn this in week two than in
  week eight. See the kill criteria in [Roadmap](09-roadmap.md).

Escalation is not failure. Silent workarounds are.

---

## 10. A note on the humans

This spec is written for agents but the project has a human at its centre who is the
creative director and the person the instrument is for.

Reserve human attention for the three things only a human can do:

1. **Answer what only the hardware can.** The double-click interval, the flash rate
   and the other open questions in SPEC-0002 `tech.md` §9 need a person with an
   Octopus.
2. **Decide what the owner has kept.** D0, the tick length (`WENGE-0012`), is one.
3. **Judge the demonstration on real gear** that ends each wave (SPEC-0002
   `product.md` §5).

Everything else should reach them as a decision with the evidence already assembled,
or not reach them at all. An agent that asks a human to choose between two options
without having measured either is wasting the most expensive resource on the project.

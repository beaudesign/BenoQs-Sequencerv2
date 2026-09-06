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

1. **Contracts before code.** The four files in `contracts/` define every boundary.
   They are edited by the Conductor only, via ADR. Everyone else treats them as
   physics.
2. **One zone, one owner.** Every path in the repo maps to exactly one role. Nobody
   edits outside their zone without a cross-zone request.
3. **One worktree per agent.** Physical isolation, not just social convention.
4. **Verification is local.** Every zone has `just verify:<zone>` that runs in under
   a minute, so an agent learns whether it is right without waiting for anyone.
5. **Communication is artefacts.** Journals and the report, not conversation. An
   agent that needs to ask another agent a question has found a missing contract.

---

## 2. Roles

Eight. Each has a brief in `agents/roles/`. A role is a *hat*, not a person: one
model instance may wear several hats sequentially, but never two at once, because the
value of the role system is that it constrains what you are allowed to touch.

| Role | Owns | Primary artefact |
|---|---|---|
| **Conductor** | `contracts/`, `adr/`, `justfile`, roadmap, merge queue | decisions |
| **Panelwright** | `reference/`, `harness/measure/geometry/`, the truth file | `panel.truth.json` |
| **Forge** | `apps/OctoPanel/` | the renderer |
| **Metronome** | `crates/octocore/`, `hosts/` | the sequencer |
| **Sceneshaper** | `crates/octoroom/` | rooms, probes, IRs |
| **Loom** *(Fable)* | `apps/OctoShell/` | dropdown, rooms space, settings |
| **Curator** | `contracts/design.tokens.json`, rubric, anchors, `harness/lint/` | taste, made explicit |
| **Referee** | `harness/` (except measure/geometry), CI, thresholds | the ratchet |
| **Scribe** | `docs/`, `README`, `CHANGELOG` | the record |

Two rules about roles:

- **The Curator does not build and the Forge does not judge.** Separating the
  producing role from the evaluating role is the reason the rubric means anything.
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
/harness/measure/geometry/** @panelwright
/harness/**              @referee
/harness/lint/**         @curator
/crates/octocore/**      @metronome
/crates/octoroom/**      @sceneshaper
/crates/octoffi/**       @conductor
/hosts/**                @metronome
/apps/OctoPanel/**       @forge
/apps/OctoShell/**       @loom
/tests/conformance/**    @metronome
/tests/golden/**         @referee
/docs/**                 @scribe
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
git worktree add ../wt-forge   -b forge/<slug>   main
git worktree add ../wt-loom    -b loom/<slug>    main
```

No agent ever has `main` checked out. This eliminates the entire class of accidents
where an agent commits to the wrong branch or stashes over another agent's work.

### What can run at once

Phase-dependent. The Conductor publishes the current fan-out in `journal/STATE.md`.
A representative Phase 2 fan-out:

```
  ┌── Panelwright: spiral geometry adjudication          (independent)
  ├── Forge:       chrome BRDF + reflection rays         (independent)
  ├── Metronome:   directions 6+ port                    (independent)
  ├── Sceneshaper: acoustic tracer, band splitting       (independent)
  ├── Loom:        dropdown probe preview                (needs probe API: frozen)
  ├── Curator:     type identification ADR               (independent)
  ├── Referee:     verify:motion implementation          (needs registry: frozen)
  └── Scribe:      manual page indexing                  (independent)
```

Eight in parallel with zero contention, because every dependency runs through a frozen
contract. When a contract needs to change mid-phase, the Conductor freezes the fan-out
first, lands the ADR, then re-fans. Do not change a contract with eight agents live
against it.

### Where parallelism does not help

Be honest about this. Some work is irreducibly serial and adding agents makes it
worse:

- **Truth file ratification.** One agent, one human, once.
- **The first vertical slice.** Phase 1 is deliberately narrow and mostly serial. Fan
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
forge: trace real reflection rays for encoder crowns

Prefiltered mips softened the crown specular into plastic at
roughness 0.045. Crowns now shoot a real reflection ray with the
probe cubemap as the miss shader. Domes keep the prefiltered path
above 6 px/mm where the difference is sub-pixel.

Verify: geometry p95 0.21mm (was 0.21mm), color mean dE 1.62
        (was 2.41), frames 0 over budget, 4.9ms p95 (was 4.1ms)
Journal: journal/forge/2026-09-14.md#crowns
Refs: F-0214, docs/04-render-engine.md#2
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
merging viable at eight-way parallelism.

---

## 8. Context economy

Agent sessions have finite context and this project has a large surface. Managing that
is an explicit responsibility, not an afterthought.

**Never load into context:**
- Reference photography. It is enormous and it is already distilled into
  `panel.truth.json` and `materials.json`. Those files are the interface to the
  images. If you find yourself wanting to look at a plate, what you actually want is
  a number that should be in a contract, so add it.
- The full 124-page manual. It is page-indexed in `reference/manual/`; load the
  section you need. `just manual <topic>` greps and returns the relevant pages.
- Generated captures. Read the report's numbers, not the EXRs.
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

1. **Ratify the truth file.** Once, carefully. [Panel Truth §3.3](01-panel-truth.md).
2. **Choose the shell typeface.** From three rendered specimens at real size.
3. **Judge the blind comparison.** [Verification §5](07-verification.md), at every
   phase boundary.

Everything else should reach them as a decision with the evidence already assembled,
or not reach them at all. An agent that asks a human to choose between two options
without having measured either is wasting the most expensive resource on the project.

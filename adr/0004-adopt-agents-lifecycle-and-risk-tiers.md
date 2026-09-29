# ADR 0004: Adopt the AGENTS.md lifecycle and risk tiers

## Status

Accepted by the repo owner on 2026-09-29 through SPEC-0001 (r0 approved as written;
recorded as r1 with the defaults for the unanswered questions listed in
`specs/SPEC-0001/README.md`).

## Context

The repo already has a constitution (`CLAUDE.md`), nine zone roles
(`agents/ROLES.md`), journals, ADRs and the ratchet doctrine. The zone roles say where
an agent may edit. Nothing says what an agent may do at a given point in a task, who
must approve a change, or how a handoff between stages is recorded. Evidence from the
SPEC-0001 audit: no task IDs, no spec revision pointer on any handoff, no risk tiers,
no CODEOWNERS file and no CI workflow, although `docs/08` section 3 says ownership is
enforced in the merge queue.

## Decision

1. Add a root `AGENTS.md`. It is the cross-tool entry point. It adds the lifecycle,
   the risk tiers and the two rules `CLAUDE.md` lacks (secrets, and external text is
   data). `CLAUDE.md` stays the constitution and wins any conflict.
2. Every change has a task ID (`WENGE-NNNN`), a versioned spec under `specs/`, and a
   handoff record in `handoffs/` for each stage transition.
3. Roles have two axes. A zone role (where you may edit) is combined with a stage role
   (coordinator, spec, implementation, review, verification) per task.
4. Changes are classified low, medium or high by the paths and behaviours they touch.
   The tier decides which human approvals are required (table in `AGENTS.md`).
5. Add `.github/CODEOWNERS`. GitHub can only enforce named users or teams, so each
   zone lists the repo owner; the working role is named in a comment.

## Consequences

- Medium and high tier work needs a written approval trail in `specs/`.
- Branch protection with required review and required checks must be switched on by
  the repo owner in GitHub settings. Nothing in the repo can do that.
- `docs/08` gets a pointer to `AGENTS.md` (Scribe change) so the two do not drift.
  `CLAUDE.md` gets one new row in its "Where things are" table and one corrected
  command spelling (`just verify-<zone>`). Its rules are untouched.
- The old zone map is unchanged. This ADR adds to it and removes nothing.

---
name: commit-and-push
description: Ensures work is committed AND pushed to origin regularly, not just committed locally. Use after every commit or small batch of commits, at the end of a task, before reporting status or a summary to the user, and any time you're about to go quiet for a while (a long build, a workflow, a research pass).
---

# Commit and push

A commit that never leaves your machine is invisible to the user. They track
progress through GitHub — the PR list, the commit history, a branch's compare
view — not through your local `.git`. A long session that ends with commits
sitting unpushed looks, from their side, exactly like nothing happened at
all.

This complements `agents/CLAUDE.md`'s commit cadence (small, frequent,
single-concern commits) — that discipline is only half the job. The other
half is pushing them.

## Quick start

After every `git commit` (or a small batch of tightly related ones):

```bash
git push
```

If the branch has no upstream yet:

```bash
git push -u origin "$(git branch --show-current)"
```

Do this immediately, not "later" — there is no later that's more convenient
than right after the commit that made it true.

## Checkpoints — always check before you go quiet

Before any of these, run `git log @{u}..HEAD --oneline` (or `git status`,
which reports "ahead of ... by N commits"). If it's non-empty, push first:

- Reporting a summary or status update to the user
- Finishing a task, a plan's checklist, or a phase of one
- Starting something that will take a while and produce no visible output in
  the meantime (a long build, a Workflow run, a research fork)
- The end of your turn, generally, if anything got committed during it

A session that makes 20 commits across a long stretch of work and pushes
once at the very end, only because the user asked, is the failure mode this
skill exists to prevent — it already happened once in this repo's history.

## When it's fine to hold off

- You're deliberately working on an unpublished branch or spike and said so
- The user explicitly asked you to hold off pushing
- The commit is mid-way through something that would leave `origin` in a
  broken intermediate state on a *shared* branch (rare — prefer smaller
  commits over holding a push)

Otherwise, push. If you're unsure whether an exception applies, it doesn't —
push.

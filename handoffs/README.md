# Handoffs

One file per task: `handoffs/WENGE-NNNN.ndjson`. Append one JSON object per line at every
stage transition. Never edit or delete an earlier line. A correction is a new line.

A handoff that names no spec revision is not a handoff. The next stage starts from the
record and the artifact it points at, not from a conversation.

## Fields

| Field | Meaning |
|---|---|
| `task_id` | `WENGE-NNNN`. Number equals the opportunity number in `specs/SPEC-0001/`. `WENGE-0000` is the factory layer. |
| `stage` | The stage that just ended: `triage`, `product_spec`, `tech_spec`, `implementation`, `review`, `verification`, `release`, `feedback`. |
| `by` | Zone role and stage role of the author, for example `conductor/coordinator`. |
| `date` | `YYYY-MM-DD`. |
| `title` | Short task title. |
| `zone` | The path or paths the task may edit. |
| `risk` | `low`, `medium` or `high`. |
| `why_risk` | One clause saying why. |
| `spec_rev` | The approved revision this work follows, for example `SPEC-0001 product r1, tech r1`. |
| `artifact` | Path of the document, branch or commit the next stage starts from. |
| `evidence` | Array of command results or measurements. Each names the command and the result. |
| `unresolved` | Array of open questions, each with an owner. Empty array if none. |
| `next` | The next stage. |

## Rules

- A handoff into `review` must come from a different session than the one that reviews.
- A handoff out of `verification` attaches or names the report (`harness/report/latest.json`
  once it exists, otherwise the pasted command output).
- Three failed attempts at one stage: stop and hand to `feedback` with the reproduction.
- Do not put secrets or personal data in a record.

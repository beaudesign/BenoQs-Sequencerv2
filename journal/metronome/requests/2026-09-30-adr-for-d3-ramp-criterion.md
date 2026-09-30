# Request to the Conductor: the ADR for D3 (a weaker ramp criterion than A4)

From: Metronome. Task: WENGE-0004 (O4). Owner approval: r3, 2026-09-30, "Approve O4 plan, all
defaults" (`specs/SPEC-0001/README.md`). Nothing has been loosened yet: this is the request, not
the loosening.

**What is being loosened.** `specs/SPEC-0001/product.md` A4 says "ramp drift is at most 1 sample"
with no buffer size. The release plan (`o4-release-plan.md`, D3 and section 3) shows what can be
met from a tempo reported only at the start of a buffer: exact on a steady ramp at every buffer
size, and in the two buffers where a ramp starts, within 1 sample up to 512-sample buffers and 21
samples (0.44 ms) at 4,096. Under `CLAUDE.md` rule 3 a loosening needs an ADR with a reason and
an expiry, written by the Conductor.

**Proposed reason** (from the plan): a host that reports the tempo only at the start of a buffer
gives the engine no way to know a ramp has begun.

**Proposed expiry** (from the plan): the first host adapter that reports the tempo at both ends of
a buffer, or 2027-06-30, whichever comes first. The alternative the owner did not choose, fixed
latency of one buffer (`tech.md` F2), remains the way to meet A4 as written.

**When it is needed.** Before PR 4b or 4c merges, not before. 4b does not start until the owner
answers D0 (the tick length). `product.md` A4 is **not** amended until the ADR exists, and is
amended in the same change as the ADR.

Not touched by this note: `contracts/`, `docs/`, `adr/`, `product.md`, any threshold.

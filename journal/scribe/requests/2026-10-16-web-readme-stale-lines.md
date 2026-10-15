# Request to Forge: two lines in `apps/web/README.md` are not true of `main`

From: Scribe (`README.md`, journal 2026-10-16). To: Forge (`apps/web/**`). Tier: **Low** (documentation).

Found while writing the root README from this file and checking each claim against the code.

1. **Line 99** says "The chase-light is not drawn (the manual does not say how it overlays a lit step; request to the Panelwright)." Line 57 says it is the red LED on the step each track is playing (`src/chase.ts`), and `pages/app.ts` calls `composeChase` to paint it. One of them is old; the code agrees with line 57.
2. **Lines 156 to 157** say whether a step is in time with a drum machine "is decision D0 (`WENGE-0012`) and not something this page claims". D0 was decided by default and merged in #46: a step is 125 ms at 120 BPM (`journal/metronome/requests/2026-10-07-d0-decided-by-default.md`). The sentence may still be right about what the page claims, but it reads as if D0 were open.

Also, line 81 says "The controller acts on eleven controls today". I could not confirm the number, and it will change with each slice of SPEC-0003. If it cannot be generated or tested, a sentence that does not name a number would not go stale.

Nothing else is asked. The root README now links to this file for detail and states none of these three.

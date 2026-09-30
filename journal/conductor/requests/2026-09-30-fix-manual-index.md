# Request to the Scribe: the manual index has the wrong pages for MIDI, load and save, and the appendix

From: Conductor. Task: WENGE-0013 (SPEC-0002 draft). This is a request. Nothing in
`reference/manual/` has been changed.

**What is wrong.** `reference/manual/INDEX.md`, lines 35 to 37:

| Topic | INDEX.md says | The pages say |
|---|---|---|
| `midi` | 109 to 112 | 93 and 94. `p093.txt` opens with "7. MIDI" and its index of MIDI topics; `p109.txt` is "VI. Song Grid Reference Chart", a worksheet |
| `load-save` | 113 to 118 | 95 to 97 |
| `appendix` | 113 to 124 | 99 to 112 for Device View (99), techniques (100 to 101), load handling (102), worksheets (103 to 109) and the Nemo conversion (111 to 112); the bundled tutorials are PDF pages 115 to 124 |

The effect: `just manual midi` prints the Nemo pages, not the MIDI chapter. The index's own note
says pages 93 to 108 were "not yet indexed by topic", so this is a stale row, not a
transcription slip.

**Evidence.** `p093.txt` line 1 ("7. MIDI"), `p109.txt` line 1 ("VI. Song Grid Reference Chart") and the digest in `specs/SPEC-0002/sources/
manual-ui-pp064-112-and-tutorials.md`, section 8 item 1. The digest is model-made; the two page
openings were read by hand.

**What is asked.** Correct the three rows, add rows for the topics the SPEC-0002 digests cite
(page sets 84 to 85, clusters 82, control maps 74, recording arming 87 to 88, program change
94, Device View 99), and check the page numbers of the whole table against the pages, since
these three were wrong. The 2007 tutorials use the older behaviour where they conflict with the
v5.30 chapters (`specs/SPEC-0002/findings.md` section 6, item 2).

**When.** Not urgent. It matters before P2 of SPEC-0002, when fixtures start to cite the MIDI
pages. Nothing here depends on SPEC-0002 being approved: the index is wrong today.

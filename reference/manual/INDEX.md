# Manual page index

`CE-v5.30-reference-manual.pdf` (124pp, genoQs Machines, Stuttgart 2009), split
into per-page plain text at `pages/pNNN.txt` (three-digit, zero-padded, matching
the manual's own printed page numbers — there is no cover-page offset to correct
for). `just manual <topic>` greps this file for `<topic>`, then cats the matched
page range.

| Topic | Pages | Section |
|---|---|---|
| attributes-overview | 12 | Step: Attributes |
| vel | 15 | Step: Velocity attribute |
| pit | 15 | Step: Pitch attribute |
| len | 15-16, 44 | Step: Length attribute; Track LEN Reference Chart |
| sta | 16-17, 45 | Step: Start attribute; Track STA Reference Chart |
| amt | 16-17 | Step: Amount attribute |
| grv | 16-17, 46 | Step: Phrase attribute (GRV); Track: Groove attribute |
| mcc | 17, 46, 53 | Step: MCC attribute; Track: MCC attribute; MCC map resolution |
| chords | 19-23 | Step: Chords, ground rules, modes, strumming, polyphony |
| strum | 22 | Chord Strum Timings in Ticks table |
| phrases | 16-17, 23-30 | Step phrases: types, reference charts, custom editor, POS |
| hypersteps | 31-33 | Step: Hypersteps |
| step-events | 34-40 | Step Events: offsets, track toggles, considerations |
| track-attributes | 41-46 | Track: Attributes overview |
| flt | 42 | Track: Flat attribute |
| direction | 45, 56-58 | Track: DIR attribute; custom direction editing + chart |
| effector | 59-63 | The Effector: feeders/listener, at work, considerations, step mask |
| scaling | 53-55 | Attribute Scaling Factors + Reference Chart |
| page-scales | 71-72 | Page: Musical Scales, Force to Scale |
| grid-scales | 85-86 | Grid: Musical Scale, Force to Scale (Quick Guide) |
| on-the-measure | 67 | Page: On-The-Measure Mode |
| mch | 46 | Track: MIDI channel attribute |
| grid-track | 79-86 | Grid-Track mode |
| recording | 87-92 | Recording: note stream, step note, controller map learn |
| midi | 109-112 | MIDI |
| load-save | 113-118 | Load/Save, MIDI Export/Import |
| appendix | 113-124 (approx, see note) | Appendix (chapter 9), including bundled 2007 tutorials |

Not yet indexed by topic (present in the manual, not yet needed by any cited
constant in `crates/octocore`): pp.1-11, 48-52, 64-70, 73-78, 93-108. Add rows
here as new citations are added — this table only needs to cover what's actually
been cited, not the whole manual.

## Page numbering note

The manual's own running footer ("N ... Octopus Reference Manual CE v5.30 ... N")
covers PDF pages 3-114, which map to manual pages 1-112 (constant offset of -2 —
confirmed by extracting every page's own footer number, not assumed). `pages/pNNN.txt`
is indexed by that real manual page number, zero-padded to 3 digits.

PDF pages 115-124 (ten pages) carry a *different*, non-numbered footer ("© genoQs
Machines, 2007", "N of 4"/"N of 2") — these are older standalone "Octopus tutorial
series" documents bundled into the PDF as its Appendix chapter, not part of the
main running page-number sequence. They're extracted separately as
`pages/tutorial-01.txt` .. `tutorial-10.txt` (in PDF page order). `tutorial-03.txt`/
`tutorial-04.txt` ("Editing track directions") contains a full worked walkthrough
of multi-trigger direction slices and is what confirmed the custom-direction
trigger-firing model in `crates/octocore` (see `tests/conformance/AMBIGUITIES.md`).
`tutorial-05.txt`/`tutorial-06.txt` ("Certainty_next", "Brownian motion") confirms
the certainty_next percentage's forward/backward semantics.

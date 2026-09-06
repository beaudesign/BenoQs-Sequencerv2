# Conformance ambiguities

Where the CE v5.30 reference manual is unclear, record both readings and which
we chose here, so it can be revisited when someone with real hardware can
arbitrate. Per [`docs/03-sequencer-core.md`](../../docs/03-sequencer-core.md) §1.

Format per entry:

```
## <topic>

**Manual reference:** <section, page>
**Ambiguity:** <what's unclear>
**Readings considered:** <A> / <B>
**Chosen:** <which, and why>
**Fixture:** <path to the conformance fixture this governs>
```

No manual is available in this repo (see `reference/NOTES.md`), so every entry
below is provisional against the two archived prior implementations
(`archive/v1-max4live`, `archive/v1-cpp-juce`) rather than a real page citation.
All govern code in `crates/octocore`.

## direction 4/5 assignment

**Manual reference:** none available.
**Ambiguity:** `docs/03-sequencer-core.md` §3 prose lists the five fixed
directions as "forward, reverse, ping-pong, random, brownian" (implying 4 =
random, 5 = brownian). Both archived prior ports independently implement the
opposite pairing: 4 = a biased random walk (2/3 forward, 1/3 back), 5 =
uniform random jump.
**Readings considered:** doc prose order (4=random, 5=brownian) / both prior
implementations' order (4=brownian-walk, 5=uniform-random).
**Chosen:** the prior implementations' order, since two independent ports
agreeing outweighs one un-cited sentence in a doc drafted without the manual.
**Fixture:** none yet — needs a manual-sourced direction fixture.

## user-programmed directions (dir 6+)

**Manual reference:** none available.
**Ambiguity:** §3 describes lighting cells in rows above row 0 to define a
custom trigger order, with "if all rows are empty, the direction is random."
Neither archived implementation has this feature at all, so there is no prior
art for the data shape or the column-traversal rule for a *non-empty* custom
direction.
**Readings considered:** implement full column-order traversal from a guess /
represent the data shape (`domain::UserDirection`) but fall back to uniform
random for every `UserProgrammed` direction, empty or not, until traversal is
specified for real.
**Chosen:** the fallback. A wrong guess at the traversal rule would be worse
than an honest "not implemented", since it would look plausible in a demo and
be wrong in a way nothing catches.
**Fixture:** none yet.

## FLT: destination self-inclusion and "last encountered" order

**Manual reference:** CE v5.30 §3 Track Mode, "Track FLAT (FLT)", p.42.
**Resolution:** the original guess (a boolean `Page.flatten` attribute) was
outright wrong, not just unconfirmed — the manual describes FLT as a one-shot
multi-track selection-merge operation, like the sibling Track Mode operations
(TGL/SOL/CLR/RND/ZOM/RMX), not a persistent attribute at any level. Replaced
with `Page::apply_flatten(&mut self, selected: &[TrackIndex])`. The manual
gives the destination (lowest selected index), the base-pitch-plus-up-to-7-stack
chord rule, the VEL/LEN/STA "last encountered active step" carry-over, and the
GRV reset-unless-track-groove-matches rule, all confirmed and implemented.
**Still chosen, not cited:** whether the destination counts as its own source
(chosen: yes) and the iteration order for "last encountered" (chosen:
descending track index). Neither is stated explicitly in the manual text.
**Fixture:** `domain::tests::flatten_merges_two_tracks_into_lowest_index`,
`domain::tests::flatten_resets_phrase_unless_track_groove_matches`.

## scale grid/page combination

**Manual reference:** CE v5.30 §4 Page Mode, "Exempting pages from the grid
scale", p.72 (repeated verbatim in §5 Grid Mode, p.85).
**Resolution:** the original guess ("page wins if both enabled, else grid,
else none") matches the manual's actual (terse) precedence statement almost
exactly: "the GRID scale is overruled by any other scales active in
particular pages. Therefore, an easy way to exempt a page from the grid scale
is to force that page to a chromatic scale." So: page-scale-enabled always
overrules the grid scale (including the degenerate case of a page forced to
chromatic, which is still "a scale active on the page" per the manual's own
phrasing, so it still technically overrules the grid even though it doesn't
audibly quantize anything) — `engine::Engine::effective_scale`'s existing
"page wins if enabled, else grid, else none" logic already implements this
correctly, no code change needed. The manual does not contain a separately
enumerated "four documented behaviours" table beyond this one precedence rule
— that phrasing was this crate's own gloss on the rule, not a manual quote.
**Also confirmed while reading this section (CE v5.30 p.71):** the *default*
scale state (before any user edit) is chromatic-C (all 12 pitch classes), not
a 7-note major scale — "the chromatic C scale is currently active... all
notes are selected in the scale, and... C is the base tone." Fixed via
`ScaleForce::chromatic`, now used as the default in `Page::default_page` and
`Grid::default_grid` (`ScaleForce::major` remains available for when a real
"select Major from the outer circle" feature is built).
**Fixture:** `engine::tests::scale_quantization_pulls_pitch_into_scale`
(page-scale-only case only).

## phrase types 2 and 3

**Manual reference:** none available.
**Ambiguity:** §3 only describes phrase type 4 ("randomises the programmed
note attributes"). Types 2 and 3 are referenced by the existence of a "type"
field but never described anywhere available to this port.
**Chosen:** `domain::PhraseType::Reserved2`/`Reserved3` — named as placeholders
that behave identically to `Fixed` in `Engine::resolve_phrase`, rather than
guessed at.
**Fixture:** none.

## step events: timing and "on-the-measure" toggles

**Manual reference:** none available.
**Ambiguity:** §3 says a step event targeting a *higher-indexed* track
executes on the following step "because of top-down processing order", and
separately that track toggles (mute/solo/record) "are subordinate to
on-the-measure mode" — implying toggles from a step event may not take effect
immediately even then.
**Chosen:** every step event (regardless of target index) is applied at the
start of the tick following the one that fired it — no same-tick case for
lower-indexed targets, and no "on-the-measure" gating for toggles. Simpler
than trying to reproduce an asymmetric rule with no data on the toggle timing.
Also: `Engine::queue_step_event` exists but nothing calls it yet — `Step::event`
is never read during `fire_step`, so step events are a real data model with no
live behaviour wired up.
**Fixture:** none.

## hyperstep carry: live vs. persisted

**Manual reference:** none available.
**Ambiguity:** §3 says a hyperstep carries PIT/VEL into another row "while the
hyped step is held" — implying a temporary effect that should revert on
release. There's no held-state model for this yet (see below).
**Chosen:** `Engine::hyperstep_carry` writes the source step's PIT/VEL onto the
target step's own fields directly and permanently, rather than layering a
temporary override that reverts on release. There is also no `panel.truth.json`
yet, so there is no real input path that could call this from a button
hold/release pair — see `reference/NOTES.md`.
**Fixture:** none.

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

## FLT track-vs-page placement

**Manual reference:** none available.
**Ambiguity:** `docs/03-sequencer-core.md` §2's attribute table marks FLT
Track-column ✓, but the same row's prose says "page-level flattening", and the
table has no Page column at all, so it cannot represent a page-level
attribute even if that's what's meant.
**Chosen:** placed `flatten: bool` on `Page`, trusting the unambiguous prose
over a table that structurally can't express the alternative.
**Fixture:** none yet — `flatten` is not wired into the tick loop at all yet.

## scale grid/page combination

**Manual reference:** none available.
**Ambiguity:** §3 says grid scale and page scale, independently on/off, produce
"four documented behaviours (page scale, chromatic, locked to grid, and so
on)" — naming three, not four, and not stating the combination rule for any of
them.
**Chosen:** `engine::Engine::effective_scale` — page scale wins if enabled,
else grid scale if enabled, else no quantization. This reproduces exactly two
of the three named behaviours ("page scale", and an unquantized/"chromatic"
state) and treats "locked to grid" as equivalent to grid-only. The actual
fourth combination (both enabled, in some way other than "page wins") is
unimplemented.
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

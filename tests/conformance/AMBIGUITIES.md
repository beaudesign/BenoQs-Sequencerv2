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

## phrase types: RandomPitch/RandomAll attribute coupling

**Manual reference:** CE v5.30 §2 Step Mode, "Step phrases — Overview", p.23.
**Resolution:** all four types are now real, not just type 4. Quoted in full:
"Type 1: Forward: notes are played in the order 1,2,3.. Type 2: Reverse: notes
are played in the order 8,7,6.. Type 3: Random pitch[:] programmed notes
pitches are played in random order, determined at playtime. Type 4: Random
all: programmed note attributes played in random combinations, determined at
playtime." Forward/Reverse are unambiguous and implemented exactly. Random
all is implemented as a permutation of the 8 *programmed* (VEL,PIT,LEN,STA)
tuples — "random combinations... of programmed note attributes", not freshly
generated values, which is what the code guessed before this fixed it.
**Still chosen, not cited:** for Random pitch, whether only the pitch value
moves between slots (chosen) or the whole note follows its pitch. The manual
says only "notes pitches are played in random order", which reads as
pitch-only, but doesn't rule out the alternative explicitly.
**Fixture:** `engine::tests::phrase_forward_plays_programmed_order`,
`::phrase_reverse_plays_8_to_1`, `::phrase_random_pitch_permutes_pitch_only`,
`::phrase_random_all_permutes_whole_notes`.

## step events: real addressing/timing implemented, several gaps still open

**Manual reference:** CE v5.30 §2 Step Mode, "Step events", p.34-40.
**Resolution:** `Step::event` is now actually read in `step_one_track`/
`fire_step` — it was dead data before. Two real bugs fixed: (1) DIR/POS/MCH
step events were modeled as an *overwrite*; the manual is explicit they're
additive deltas ("if DIR is 6 a Step Event of '+2' will change it to 8", p.34),
wrapped mod 16/32 for DIR/MCH (p.34: "Those MAXIMUM values are... 16 for DIR
[and] 32 for MCH"). (2) Track Toggle events were modeled with an explicit
`target_track` field; the manual's real addressing is AMT/Range-based (p.39):
`|amt|` selects the target track (0 invalid, 10 means track 0), sign selects
On/Off, and `range` (max 10) selects how many consecutive tracks are affected,
wrapping — confirmed against the manual's own worked example ("AMT +1 & Range
4... Tracks 1 & 0 and Tracks 9 & 8 will be Muted") in
`engine::tests::track_toggle_range_wraps_manual_worked_example`. Also added
the two missing toggle kinds structurally: `Pause` is a real, working toggle;
`TrackRotate`/`TrackSkipRotate` exist as data (p.38-39 reveal these are a
*different* operation — whole-track step-data rotation, not a per-track
toggle at all) but are not wired into playback — seeded, not guessed at.
DIR/MCH now revert to the persisted `Track` attribute when the sequencer
stops via a live-override shadow on `TrackRuntime` (p.40: "DIR will default
to the Track attribute amount when the sequencer stops" / same for MCH); POS
deliberately has no shadow and mutates `Track::rotation` directly, since
"POS does not restore to the start POS(ition) when stopping the sequencer."
**Still a chosen simplification, not cited:** every step event applies at the
start of the tick *following* the one that fired it, for every target,
uniformly — the manual's real rule is asymmetric (p.40: "tracks are processed
top-down... when [an event] is applied to tracks higher in the matrix it will
be executed on the following step", implying same-tick application for
lower-indexed targets). Implementing genuine same-tick application would
require reworking the tick loop's per-tick `Page` snapshot (every track
currently reads one snapshot taken once at tick start specifically so
mid-tick mutations from other tracks aren't visible — see `engine.rs` module
docs) — a bigger architectural change than fits alongside the addressing fix,
so deferred rather than done halfway.
**New, still open (not attempted this pass):**
- The attribute-map-factor step-event sub-system (VEL/PIT/LEN/STA/AMT/GRV/MCC
  step events, p.34-37): "what changes is really the attribute map factor of
  any steps having offsets from the track value" — a real, non-trivial
  scaling-factor progression system ("Available Step Event Range = 17 - Track
  Attribute Scaling Factor", with a 9-row × 17-column reference chart on
  p.35). Confirmed real, not implemented — see the `Step` attribute-offset
  model, which still only supports VEL/PIT/LEN/STA/GRV/MCC as simple
  offsets/absolutes, no map-factor scaling at all.
- Track Rotate / Track Skip Rotate's actual step-data-shifting behaviour
  (p.38-39) — data model present, playback not implemented.
- On-the-measure deferral for Mute/Solo Track Toggles (p.40: "subordinate to
  the On-The-Measure mode condition of the Page") — tracked separately, see
  the next AMBIGUITIES entry once that lands.
- POS's own wrap maximum isn't given anywhere in the manual (unlike DIR's 16
  and MCH's 32) — left unwrapped; harmless since it only ever feeds a
  `% page_len` downstream.
**Fixture:** `engine::tests::step_event_set_dir_is_additive_and_wraps`,
`::step_event_set_dir_reverts_on_stop`,
`::track_toggle_range_wraps_manual_worked_example`,
`::track_toggle_amt_10_targets_track_0`, `::track_toggle_negative_amt_is_off`.

## hyperstep carry: was a wrong model, now corrected

**Manual reference:** CE v5.30 §2 Step Mode, "Hypersteps", p.31.
**Resolution:** the earlier guess (a hold-to-apply, revert-on-release
performance gesture) was outright wrong — the manual describes hold as only
the UI gesture to create/destroy a *persistent* link, not a temporary carry:
"if a track is linked to a hyperstep, once the hyperstep is played, the track
is being triggered to play at a speed corresponding to the step absolute
length (in 1/192), and taking over the hyperstep's velocity and pitch offset
... Changes to the hyperstep PIT and VEL will influence the hypedtrack in
real-time." Replaced `Engine::hyperstep_carry` (which permanently copied
values once) with `Page::hyperstep_links` (a persistent link) plus
`Engine::hyperstep_link`/`hyperstep_unlink`, and wired live PIT/VEL
substitution plus the 192-vs-12-tick length change into `step_one_track`.
**Still open:** the fine-grained intermediate LEN-scaling curve (p.32,
"Hyperstep LEN Reference" — only the confirmed binary 12↔192 boundary case is
implemented, not the source track's own LEN-scaling interaction with it).
Real link creation is still gated on a `ControlId` -> (track, step) mapping
that doesn't exist yet (no `panel.truth.json`).
**Fixture:** `engine::tests::hyperstep_linked_track_fires_every_192_ticks_not_12`,
`::hyperstep_pit_vel_are_read_live_from_source`,
`::hyperstep_unlink_restores_default_step_length`.

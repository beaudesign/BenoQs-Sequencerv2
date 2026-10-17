# Engineering spec v2: shape player (`feat/pattern-styles`) and SPEC-0004 S7 Complex (`feat/voice-analog`)

**For:** Claude Code, working in `beaudesign/BenoQs-Sequencerv2` (the WENGE repo).
**From:** Ben (repo owner, `@beaudesign`). Revision v2, 17 Oct 2026. It supersedes v1 (written against `6a14539`) and the addendum of the same day.
**Base:** `main` at `c079f8f` (#69, SPEC-0004 r1).
**SPEC-0004 Q-S4-4:** this is the engineering spec that question asks for. Section 5.7 holds the rows to copy into SPEC-0004 as H-17 onward.

This file is self-contained. If it disagrees with the repo about a fact, the repo wins and you report the difference. If it disagrees with the repo about a decision, stop and ask me.

---

## 1. Context

**The repo.**
- WENGE is a Rust rebuild of the genoQs Octopus sequencer core, cited against the CE v5.30 manual (`reference/manual/`, `just manual <topic>`).
- It compiles to wasm for the web app in `apps/web`.
- Before anything else, read `CLAUDE.md`, `AGENTS.md`, `journal/STATE.md` and `agents/ROLES.md`. On process, they win over this file.

**Crates.**
- `crates/octocore`: the core. 192 PPQN, `#![forbid(unsafe_code)]`, no dependencies, no allocation on the audio path.
- `crates/octoface`: the panel controller.
- `crates/octoffi`: the C ABI. Frozen (SPEC-0002 D5). Its `octoffi.h` is hand-maintained and pinned line by line by a test, and it mirrors `TrackAttr` and `Command`.
- `crates/octorun`: the headless runner. A pattern goes in; `target/octorun/NAME.ndjson` and `NAME.mid` come out.
- `apps/web/engine`: crate `octoweb`, the worklet's wasm. Its exports are in `apps/web/engine/ABI.md` (ADR-0008).
- `xtask`: the gates.

**How work is proven here.**
- **Red first.** A new test fails on the parent commit, and both runs are kept.
- **Goldens.**
  - Each `examples/*.pattern` has an `examples/golden/NAME.sha256` (lines `events …` and `smf …`).
  - `crates/octorun/tests/golden.rs` checks them, including a both-ways "every example has a golden" check.
  - The `determinism` gate reads that test.
- **The ratchet.** `harness/baseline.txt` only grows (`just baseline`).
- **Mutation runs.** These are hand-written mutant scripts run in a scratch copy, with their output kept (`handoffs/evidence/p5b-mutate.py`, `handoffs/evidence/metronome-click/mutate.py`). `cargo-mutants` is not used.
- **Evidence and handoffs.** Evidence goes in `handoffs/evidence/`. Every stage change appends a line to `handoffs/WENGE-NNNN.ndjson` (fields as in `handoffs/README.md`).
- **Gates.**
  - `just verify` runs them all. `conformance`, `regressions`, `determinism` and `scope` are required.
  - Before a PR, run `just verify --base origin/<base>`.
  - Web: `just web-test` and `just web-browser-test`. The engine in a worklet: `just spike-s2`.

**SPEC-0004 r1 on `main`** (WENGE-0030; planning only; **not yet approved**: "a merge is not an approval").
- The page's voices are TypeScript in `apps/web/src/voices/`, running in the AudioWorklet behind a `Voice` interface.
- Slices:
  - **S0**: the interface, Plain (today's tone, sample-exact), a test rig, an offline WAV renderer, and the one ADR (D1).
  - **S1**: oscillator parts.
  - **S2**: filters.
  - **S3**: Saw ladder and Pulse morph, plus the `Voice` button. Saw ladder becomes the default.
  - **S4**: Gate pluck, with the vactrol LPG and shared RC envelopes.
  - **S5**: a voice per channel.
  - **S6**: optional extras.
  - **S7 Complex**: added in r1. It runs after S1, S2 and S4, behind the same interface and button.
- r1 decisions:
  - **D12**: this voice is one more voice, not a second system. No `crates/octovoice`, no `octorun --wav`, no separate ADR; S0's one ADR records it.
  - **D13**: TypeScript, no Rust voice core. Bit-exact native/wasm sound is given up on purpose.
  - **D14**: per-voice caps, oldest note taken. Simple voices keep 24, **Complex 8 provisional** until H-13 says otherwise. Oversampling per **D7**: none first, 2× around the nonlinear stage only if H-3 or H-6 need it. **Complex is never the default** unless I ask. Plain stays selectable.
- Other rules: **H-16** is a registry test that runs H-2, H-3, H-6, H-9, H-11 and H-13 on every registered voice. D5 forbids hardware brands in the interface. D6 is mono. Equality tests run in Node; browsers assert properties.
- Its gates: **Q-S4-4** says nothing about Complex is built until this spec is in the repo and I have approved SPEC-0004 in my own words. Q-S4-5 is mine, on bit-exactness.

**SPEC-0005** (WENGE-0031).
- `main` has r0, **Drift**: a per-track `ChordStyle`, a web-only `octoweb_set_chord_style` (its D3), DR-1's 200-seed golden.
- Open PR #68 (r1, branch `conductor/spec-0005-r1-curved-time`) replaces it with **Curve**: the strum roll's note starts follow a warp curve, with a pinned integer table, no randomness, and CV-1's golden over strum levels, sizes and 200 seeds. Drift becomes a runner-up.
- r1's Q-S5-2 asks whether I also meant the shape player, and keeps "Shape ornaments" as a possible S4 `ChordStyle`.
- Either way: a per-track `ChordStyle`, a web-only setter, a strip button, no panel control, `contracts/` untouched.

## 2. Owner decisions (fixed; do not reopen)

1. **Feature A, `feat/pattern-styles`.** The benny-inspired shape player is the first non-manual pattern style.
   - It adds a note-length and velocity layer (rhythm) on top of benny's order-only shapes.
   - It lives in a new crate, `crates/octostyle`, outside the Octopus-manual core.
   - **Every existing golden hash stays byte-identical.**
   - **ADR-0013.**
2. **Feature B is SPEC-0004 S7 Complex, on `feat/voice-analog`** (SPEC-0004 D12–D14).
   - A Verbos-modelled voice in TypeScript behind S0's `Voice` interface, reusing S0's renderer and rig, S1's oscillator parts and S4's LPG and envelopes.
   - What it models: a modulation oscillator and a master oscillator, an AM/off/exponential-FM link, soft sync, and harmonics (timbre, symmetry, order folding) into the LPG.
   - **It is faithful by default.** 259-style linear through-zero FM and an audio-rate modulator range are optional extras, **off by default**, and marked `not-Verbos`.
   - Cap 8, provisional. Oversampling per D7. Never the default.
   - No separate ADR: S0's one ADR records it (D12).
3. **Licence and provenance.**
   - benny (playbenny/benny) is under the Hippocratic License 3.0, copyleft in effect, and bundles third-party code (Yofiel EPTR, Surreal Machines smFilterPack, Sacred Walls waves). **Copy, translate and paraphrase no benny code, patch, preset or text.** Take ideas only, from this spec, papers and behaviour.
   - Open no benny source file while you work.
   - Complex is modelled from Verbos's public behaviour only. No schematic, firmware or third-party DSP code.
   - Attribution goes in ADR-0013 (A) and S0's ADR (Complex, D12).
4. **Both features proceed.** The addendum's "build Feature A only" hold is lifted. Each waits only on the gates in §3.1.

## 3. Global rules for Claude

### 3.1 Process and gates before code

**Feature A.**
- The repo's lifecycle needs a product spec and my approval before implementation. A's first PR is therefore a Spec-stage PR: `specs/SPEC-0006/README.md` (the next free number; confirm it is free), written from §4 in SPEC-0004/0005's format, with the decisions in §2 recorded as mine and dated 17 Oct 2026.
- A1 starts only after I approve SPEC-0006 in my own words.

**Feature B (S7).**
- This file is attached to the repo (Q-S4-4).
- The rows in §5.7 are copied into SPEC-0004 as H-17 onward. That is a Spec-stage edit to `specs/SPEC-0004/`, in its own PR.
- I approve SPEC-0004 in my own words.
- S1, S2 and S4 have merged (S3 and S5 before S7e).
- Until all of that is true, write no Complex code.

**PRs and branches.**
- One PR per slice. `feat/pattern-styles` and `feat/voice-analog` are integration branches cut from `main`; cut the latter once S7's prerequisites are on `main`.
- Name slice branches `<role>/style-a<N>-<slug>` and `<role>/complex-s7<x>-<slug>`. Git cannot hold `feat/x` and `feat/x/y` at once.
- I merge everything. Never merge your own PR.

**Red first.**
- A slice's first commit adds only the failing tests, fixtures and expectations.
- Keep the red run in `handoffs/evidence/<slice>/red.txt` and the passing run in `green.txt`.

**Evidence per slice** goes in `handoffs/evidence/<slice>/`, where `<slice>` is `style-a1`…`style-a6` or `complex-s7a`…`complex-s7e`:
- `red.txt`, `green.txt`;
- `mutate.py` and `mutation-run.txt`: at least 8 mutants, each caught or argued equivalent;
- `verify.txt`: `just verify --base origin/<feature branch>`, plus `cargo test --workspace` for Rust slices or `just web-test` for TypeScript slices;
- the slice's extras;
- a row for each new file in `handoffs/evidence/README.md`.

**Task IDs.**
- WENGE-0030 and 0031 are taken. The Conductor plans SPEC-0004 S0 as WENGE-0032.
- Take the next free ID when you open each PR, one per slice, and list it.

**Ratchet and commits.**
- Run `just baseline` with every slice.
- Delete, weaken or rename no test. Loosen no existing threshold (CLAUDE.md rule 3).
- Commit about every 30 minutes and push each commit (rules 8 and 9). Messages carry the gate numbers and the task ID.
- Run `just journal <role>`, and write the next three steps when you stop.

### 3.2 ADR numbers

| ADR | Decision | Lands in |
|---|---|---|
| **0013** | Non-manual pattern styles live in `octostyle` (owner-fixed) | A1 |
| next free (expected **0014**) | SPEC-0004 D1, "the sound becomes a feature". Per D12 it also records Complex: its provenance, faithful by default, the `not-Verbos` extras, cap and oversampling | SPEC-0004 S0 |

- There is no ADR-0015 for Complex.
- If S0's ADR is written before A1 merges, it must not take 0013 (QG3).
- If anything else claims 0013, stop and ask.

### 3.3 Tiers and zones

- **Tiers.**
  - Medium (AGENTS.md): a new crate, additive `octoweb` exports, `justfile`, anything in `apps/web/`.
  - High, with its own PR after the ADR merges: anything in `contracts/**`, any `repr(C)` change, `octoffi.h`.
- **Zones.**
  - Conductor: `contracts/`, `adr/`, `justfile`, `octoffi`, and spec edits.
  - Metronome: `crates/octocore`, `tests/conformance`.
  - Forge: `apps/web`, which makes S7 Forge's.
  - Referee: `harness/`, `xtask/`.
  - Scribe: `docs/`.
  - `crates/octostyle` has no owner yet (QG2).
  - For a cross-zone change, file a request in `journal/<role>/requests/`.

### 3.4 Hard technical rules

- **Byte-identical output.**
  - Steps-style engine output does not change by one byte: the five goldens (`hello`, `chords_and_strums`, `phrases`, `effector`, `mcc_and_transport`), and whichever SPEC-0005 golden lands (DR-1 or CV-1).
  - Worklet events are byte-identical whatever voice plays (H-12).
- **`crates/octostyle`** is `#![no_std]`, `#![forbid(unsafe_code)]`, allocation-free, and has **no third-party dependencies**, including dev-dependencies. Use fixed arrays; there is no `heapless`.
- **Complex** follows SPEC-0004's voice rules:
  - no allocation in sample loops (H-13's text check: no `new`, array literal, `.map(` or closure);
  - no `Math.random`;
  - deterministic;
  - mono;
  - under 1 after the master stage.
- **The scope gate.** Read `harness/scope-banned.txt` before you write any DSP comment. Two of its phrases are ordinary filter-design terms (for a filter's time response, and for filtering with a kernel). Write "FIR taps", "kernel" or "filter" instead. Add no allow-list entries.
- **`Ref:` lines.** `octostyle` constants carry `Ref: not in the CE v5.30 manual; ADR-0013, …`. Complex constants carry `Ref: SPEC-0004 S7, [S1]` (or the paper).
- **MIDI.** Every NoteOn has velocity ≥ 1 and a matching NoteOff. `expect balance` holds after Stop.

### 3.5 Stop and ask me

Stop and ask in these cases:
- at any question in §3.6, §4.11 or §5.10, at the slice it gates;
- when a threshold can't be met as written;
- before touching `contracts/**`, `octoffi.h`, a `repr(C)` type, an existing golden, `harness/required-gates.txt`, S0's `Voice` interface, or SPEC-0005;
- when the repo contradicts a path or fact here in a way that changes the design.

Otherwise, report the contradiction and carry on.

### 3.6 Global open questions (owner's call)

- **QG1. Which way does the crate dependency run? (Gates A2.)** The engine hook (`octocore/src/engine.rs`) and the verbs (`octocore/src/fixture.rs`) are in `octocore`, so if `octostyle` depended on `octocore` the two would form a cycle. The options:
  - **(a)** `octostyle` has no dependencies. It gets its own splitmix64, cross-checked against `octocore::rng`, and `octocore` depends on it.
  - **(b)** `octocore` defines a hook trait and `octorun`/`octoweb` compose the two.
  - **(c)** Your alternative.

  A1 is built dependency-free, which suits all three.
- **QG2. Who owns `crates/octostyle`?** Proposed: Metronome.
- **QG3. Do you confirm that 0013 stays reserved for A?** If so, S0's ADR takes the next free number even if it is written first.

---

## 4. Feature A: shape player (`feat/pattern-styles`, ADR-0013)

### 4.1 Goal

A track can switch from the manual's step lane to a **shape** style. The track then plays an ordered list of scale degrees through a scale chosen from 8 slots:
- at a fixed arp rate, or with the shape's own rhythm lane;
- with in-scale transpose, rotation and seeded ornaments.

It is clocked by the engine and sent through the same event stream, so it is deterministic and goldens can be taken.

### 4.2 Scope and non-goals

**In scope:**
- scales and shapes in 8 slots, and degree-to-pitch;
- plain playback, the `dur` and `vel` lanes, capture from steps;
- 10 ornaments with fixed RNG consumption;
- scale swap, and re-rooting (subject to QA1);
- verbs, the web export, the demo.

**Non-goals:**
- René, wonky, curved time, Drift. SPEC-0005's styles stay separate.
- MIDI-in capture (QA4) and panel bindings (QA3).
- Persistence, triplet or dotted rates.
- Any change to steps-style output.
- Any edit to SPEC-0005.

### 4.3 Architecture

**The crate.** `crates/octostyle` is a new workspace member and has no dependencies (QG1). Its public API:
- `Scale`, `Shape`, `Slots`, `PlayerParams`;
- `ShapePlayer::on_tick(&mut self, tick: u64, view: &TrackView) -> NoteRequests`, which returns a fixed array of up to 16 entries (pitch, velocity, start offset, length in ticks) and a count;
- `capture_from_steps(...)`.

**Engine hook.**
- Add a per-track **`PatternStyle`**: `Steps` (the default) or `Shape`. It is named so that it doesn't clash with SPEC-0005's `ChordStyle`.
- A `Shape` track's clock ticks go to its `ShapePlayer` instead of its step lane.
- Its requests go through the engine's existing scheduler (`NoteFire`, the queue, "NoteOffs before controllers before NoteOns" at a sample). So Stop, balance, lookahead and Diagnostics are unchanged.
- The hook is one `match`. `Steps` takes the old path untouched.

**Living with SPEC-0005.**
- A `Shape` track never reaches the chord or strum branch, so no `ChordStyle` (Drift's tape, Curve's table) affects it.
- The shape RNGs are seeded `seed ^ track ^ SALT`, with salts distinct from any per-track generator SPEC-0005 adds.
- Tests: a shape track's events are identical with every `ChordStyle` value on another track; a `ChordStyle` track's events are identical whether its neighbour is a shape track or empty. These are written as pending until SPEC-0005's S1 lands.

**State placement.** Keep style state off `Snapshot`, which is `repr(C)` and FFI-facing. Hold it in engine-owned, non-FFI state.

**How the style reaches the engine (QA5).** SPEC-0005 D3 sets a precedent: a web-only additive export that leaves `TrackAttr`, `Command` and `octoffi.h` alone. For us that would be, for example, `octoweb_set_pattern_style` plus slot and player setters.

**Track attributes on a `Shape` track** (provisional, recorded in ADR-0013):
- Applied: `MidiChannel`, `Velocity` (the base, used when there is no `vel` lane), `Muted`, `Soloed`, `Paused`.
- Ignored: `Pitch`, `LengthFactor`, `StartFactor`, `DirectionRaw`, `Rotation`, `Amount`, `Groove`.

**Transport.** Play continues exactly where it stopped (docs/10 §3). The index and RNG persist across Stop/Play. `Reset` sets the index to 0 and reseeds.

### 4.4 Data model and parameters

```
Scale   { root: u8 /*pc*/, mask: u32 /*12 bits span 1, 24 bits span 2*/, span: 1|2 }
         // modes: major, dorian, phrygian, lydian, mixolydian, aeolian, locrian,
         // harmonic_minor, double_harmonic, super_locrian; plus custom
Shape   { len: 1..=32, deg: [i8; 32], dur: Option<[u8; 32]>, vel: Option<[u8; 32]> }
Slots   { scales: [Scale; 8], shapes: [Shape; 8] }
PlayerParams {
  scale_slot: 0..=7, shape_slot: 0..=7 | SAME,
  rate: 64n|32n|16n|8n|4n|2n|1n|2b|4b|8b   // 12,24,48,96,192,384,768,1536,3072,6144 ticks
  octave: 0..=10 (5), transpose_deg: i8, post_semis: -63..=63, rotate: 0..=31,
  range: Mute | Fold(1..=3), rooted: bool (true), seed: u64,
  prob: [u8 0..=127; 10]  // skip,double,flam,swap,trill,arp,sub_loop,back,fwd,reset
  arp_step: 1..=8, arp_limit: 1..=8, incline: -64..=63
}
```

- **Scale maths.** `scale_nth(d)`, with `n = popcount(mask)` and `period = 12*span`, is `period*floor(d/n) + offset_of(d mod n)` (Euclidean division).
- **Pitch.**
  - `d' = deg[(i+rotate)%len] + transpose_deg`, minus `deg[0]` when `rooted`.
  - Then `p = 12*octave + root + scale_nth(d') + post_semis`.
- **Range.** When `p` falls outside 0..=127, `Mute` drops the note (the draws still happen) and `Fold(k)` moves it by `12k` until it fits.
- **Rhythm.**
  - Without `dur`, each step lasts one `rate` period.
  - With `dur`, a step lasts `dur[i]*48` ticks (1/16 units, 1..=128).
  - Note length equals step length (legato; QA6). A same-pitch repeat is legal because the NoteOff goes first.
- **Velocity.** `vel[i]`, or the track's base velocity, clamped to 1..=127.

### 4.5 Pattern verbs (parsed where QG1 puts them; documented at the top of `fixture.rs` either way)

```
scale S mode <name> root <pc>        # or: scale S mask <12|24 binary digits> [span 2]
shape S deg d0 d1 ...                # 1..32 degrees
shape S dur t0 t1 ...                # optional, 1/16 units
shape S vel v0 v1 ...                # optional
shape S capture track T
track T style steps|shape
track T shape [slot S|same] [scale S] [rate R] [octave O] [transpose N] [post N] [rotate N]
              [range mute|fold1|fold2|fold3] [rooted 0|1]
track T orn <name> P [...]           # P 0..127
track T orn arp_step N arp_limit N incline N
```

- A verb given after `play` takes effect at the track's next shape step.
- Mismatched counts or out-of-range values are parse errors with `ctx()` context.

### 4.6 Ornaments (deterministic)

**Random draws.**
- Each track has its own splitmix64, seeded `seed ^ track ^ SALT_ORN`.
- Every shape step draws exactly **10 times**, in `prob[]` order. Ornament `k` fires if `next_below(127) < prob[k]`.
- Any extra randomness (sub_loop's length) comes from a second stream seeded with `SALT_AUX`.

**What each ornament does.**
- **skip**: a rest.
- **double**: two notes at half the step length.
- **flam**: a grace note one tick early, one degree away in the incline direction.
- **swap**: exchanges this step with the next.
- **trill**: alternates `d'±1` every 24 ticks.
- **arp**: up to `arp_limit` notes, `arp_step` degrees apart, splitting the step.
- **sub_loop**: replays the last 2–4 steps.
- **back** and **fwd**: move the index by ±1.
- **reset**: sets the index to 0.

**Direction and pitch.** The sign of `incline` sets the direction; 0 alternates, starting upward. Every ornament pitch is a scale degree.

**Precedence** (provisional; QA2):
1. Index moves: reset, then back/fwd.
2. skip, which suppresses everything else.
3. swap, then sub_loop.
4. arp, trill, double.
5. flam last.

### 4.7 ADR-0013, draft (`adr/0013-non-manual-pattern-styles-live-in-octostyle.md`, in A1, house format)

- **Status.** Proposed. Accepted in principle by the owner on 17 Oct 2026; it takes effect when the owner merges it.
- **Context.**
  - The core follows the manual (docs/03 §1), and the owner wants styles from outside it.
  - SPEC-0005 r0 §3 passed over the shape player. r1 (#68, open) keeps "Shape ornaments" as a possible `ChordStyle` and asks the owner about it (Q-S5-2).
  - On 17 Oct the owner chose the shape player as a **pattern style** (a track-level alternative to the step lane, with its own rhythm), which is a different axis from `ChordStyle`.
- **Decision.**
  1. `crates/octostyle`.
  2. Degrees plus `dur` and `vel` lanes.
  3. `PatternStyle` per track, `Steps` by default, byte-identical.
  4. Seeded RNGs with fixed consumption and their own salts.
  5. A 24-bit mask.
  6. The attribute table in §4.3.
  7. The range reading.
  8. The ornament precedence.
  9. The tests in §4.8.
- **Provenance.** Concepts only, from playbenny/benny `seq.shape.player` and `core.scales.shapes` (Hippocratic License 3.0), via the owner's notes. No benny code or patch was read or used. The modes are standard theory.
- **Alternatives rejected.**
  - Writing notes into steps.
  - Putting the style in `octocore`, which the owner rejected.
  - Order-only shapes, which the owner rejected.
  - Making the style a `ChordStyle` value: it replaces the step lane rather than shaping a chord.
- **Not decided here.** QG1, QA1–QA8.

### 4.8 Slices

Every slice:
- is red first;
- keeps the five goldens and any SPEC-0005 golden untouched;
- passes `just verify --base origin/feat/pattern-styles`;
- has its mutation run kept.

If SPEC-0005 slices land mid-feature, rebase and re-run every golden.

**A1. ADR-0013, the crate, scale maths** (after SPEC-0006 is approved).
- `crates/octostyle`, dependency-free: `Scale`, the 10 modes, `scale_nth`, `quantize`.
- Red tests:
  - interval tables for all 10 modes;
  - a hand-checked dorian table at degrees −8, −1, 0, 6, 7 and 14;
  - span 2 with a 24-bit mask.
- Properties, exhaustive over every mode and `d` in −64..=64:
  - `quantize(root+scale_nth(d)) == root+scale_nth(d)`;
  - `scale_nth(d+n) == scale_nth(d)+12*span`;
  - strictly increasing.
- The workspace still builds for `wasm32-unknown-unknown`.

**A2. Plain playback** (needs QG1 and QA5).
- `PatternStyle`, the hook, verbs, `rate`/`octave`/`transpose`/`post`/`rotate`/`range`/`rooted`.
- Red: `examples/shape_basic.pattern` with `examples/golden/shape_basic.sha256`. The name goes in the `golden!` list and in the `no_golden_pattern_overflows…` list in `crates/octorun/tests/golden.rs`, and in `apps/web/engine/tests/golden.rs`.
- Properties:
  - in-scale pitch classes (`post_semis = 0`);
  - `rotate = len` is the same as 0;
  - transpose ±n is the identity;
  - balance after Stop;
  - velocity ≥ 1;
  - a muted shape track is silent;
  - steps-only patterns hash identically before and after the hook (the five goldens plus a 64-seed randomised check);
  - the `ChordStyle`-isolation tests of §4.3.
- Determinism: 3 runs give identical bytes, and buffers of 64, 256 and 4096 give identical bytes.

**A3. Rhythm and capture.**
- `dur`, `vel`, `capture track T`.
- Capture walks T's active steps in order. Each step contributes its pitch plus the stored chord and phrase notes, ascending (descending if `strum < 0`). No polyphony sampling; it never consumes the engine RNG or any `ChordStyle` state. Each note goes to the nearest degree, and `dur` comes from the step spacing.
- Red: `examples/shape_capture.pattern`, which captures `chords_and_strums`' steps, with its golden.
- Properties:
  - a cycle lasts `Σdur*48` ticks;
  - with no `dur`, the output equals A2's;
  - `vel` appears verbatim;
  - capture then playback reproduces the in-scale pitches.

**A4. Ornaments** (needs QA2).
- Red: `examples/shape_ornaments.pattern` with its golden.
- Properties:
  - the same seed gives the same bytes three times;
  - 16 seeds give at least 15 distinct hashes;
  - with every prob at 0, the output equals A2/A3;
  - all notes in scale;
  - **fixed consumption:** moving `trill` from 0 to 127 leaves every `skip` decision unchanged, checked against a debug-only trace;
  - prob 127 always fires;
  - balance holds at 64n with flam and trill at 127;
  - no queue refusals.

**A5. Scale swap and re-rooting** (re-rooting needs QA1).
- Red: one shape played in two slots and modes, with a mid-run swap.
- Properties:
  - the degree trace is identical across scales;
  - the swap takes effect at the next shape step;
  - if re-rooting is approved, the root follows track N's lowest note per step, and the notes are in the scale transposed to that root.

**A6. Web export, parity, demo.**
- Additive `octoweb` exports per QA5, as one-line shims, with `ABI.md`, `tests/abi.rs` and `tests/source_rules.rs` kept green.
- Parity:
  - `octoweb_spike_run` on `shape_basic`, `shape_ornaments` and `shape_demo` matches native, in `apps/web/engine/tests/golden.rs` and under `just spike-s2`.
  - `harness/wasm/smoke.mjs` (the frozen octoffi) reports the shape patterns as **skipped**.
- No strip button unless QA3 says so.
- The demo, §4.9.

### 4.9 Demo render

`examples/shape_demo.pattern` has its own golden:

```
# Shape player demo: a dorian phrase, rotated and ornamented, over a strummed chord track.
seed 11
bpm 104
scale 0 mode dorian root 2
scale 1 mode phrygian root 2
shape 0 deg 0 2 4 7 6 4 2 1
shape 0 dur 2 2 2 4 2 2 1 1
track 0 mch 1
track 0 style shape
track 0 shape slot 0 scale 0 rate 16n transpose 0 range fold1 rooted 1
track 0 orn trill 25 double 15 flam 15 skip 8 arp 10
track 1 mch 2
track 1 step 0 active 1
track 1 step 0 strum 5
track 1 step 8 active 1
track 1 step 8 strum -5
play
render 20 s buffer=64
track 0 shape scale 1
render 12 s buffer=64
stop
render 4 s buffer=64
expect event on
expect balance
```

Render it to `handoffs/evidence/style-a6/shape_demo.wav` (and `.mp3`) with **SPEC-0004 S0's renderer**, using the channel 1 and 2 voices current on `main` (Complex too, if S7e has landed). If the renderer can't take an `octorun` event log, that is QA7. Until it is answered, render `.mid` → fluidsynth → ffmpeg and say so in the README row. Include a 3-line listening note (0:20 is the scale swap).

### 4.10 Risks

- Golden drift in old patterns or SPEC-0005's: covered by the five goldens, DR-1 or CV-1, and the randomised steps-only check.
- RNG coupling: fixed consumption and separate salted streams.
- Queue pressure at fast rates with ornaments: test at 64n with every prob at 127.
- Scope drift from the manual: the crate boundary, a one-`match` hook, the ADR.
- Overlap with SPEC-0005 r1's "Shape ornaments" candidate (QA8).
- Strip crowding: SPEC-0004 and SPEC-0005 each add a button.

### 4.11 Open questions (owner's call)

- **QA1 (gates A5's re-rooting).** Should shapes re-root under a named track's chord bass, or keep a fixed root?
- **QA2 (gates A4).** Ship all 10 ornaments, or trill, double, flam and skip first? Do you accept the precedence in §4.6?
- **QA3 (gates any UI or contract).** Where do the style controls live?
  - A strip control (the SPEC-0004/0005 precedent).
  - A new `controls.json` zone (the manual's physical inventory, ADR-0007; High).
  - Nothing until the panel binds them (ADR-0012).
- **QA4.** Live MIDI-in shape capture in the web app, or patterns and steps only?
- **QA5 (gates A2).** How does the style reach the engine?
  - A web-only export, per SPEC-0005 D3 (Medium).
  - `TrackAttr::Style` plus an `octoffi.h` update (High).
  - A new `Command` variant (High; it may change the `repr(C)` size).
- **QA6.** Notes are legato. Do you want a gate fraction or lane?
- **QA7 (gates A6's render).** May A6 (or a SPEC-0004 slice) extend S0's renderer to play an `octorun` ndjson event log, so pattern demos render through the voices? Or is fluidsynth fine?
- **QA8.** Does Feature A answer SPEC-0005 r1's Q-S5-2 ("did you mean the shape player too?")? Does it retire r1's S4 "Shape ornaments" `ChordStyle` candidate, or should that stay as a separate chord-only idea? I won't edit SPEC-0005; the Conductor records your answer.

---

## 5. Feature B: SPEC-0004 S7 Complex (`feat/voice-analog`)

### 5.1 Goal

**Complex** is one more SPEC-0004 voice behind S0's `Voice` interface, modelled on the Verbos Complex Oscillator. It has:
- a modulation oscillator and a master oscillator, linked by AM/off/exponential FM, with soft sync;
- a harmonics stage (timbre, symmetry, order);
- S4's vactrol LPG.

It is selectable from the `Voice` button and per channel (S5), and it renders through S0's renderer. MIDI and events are unchanged. It is never the default (D14).

### 5.2 Scope and non-goals

**In scope:**
- Complex's code in `apps/web/src/voices/`, laid out however S0 sets that folder up;
- its registry entry;
- the two `not-Verbos` extras, off by default;
- its patches;
- rows H-17 onward;
- WAV demos through S0's renderer.

**Non-goals** (D12/D13):
- a Rust voice core, `crates/octovoice`, `octorun --wav`, a second renderer, LPG, envelope, rig or allocator, a separate ADR, bit-exact native/wasm sound;
- CV inputs or a modulation matrix, stereo, effects (a wavefolder *as an effect* stays out per §9; Complex's harmonics stage is part of its oscillator), drums;
- per-parameter UI;
- any change to S0's interface without asking.

### 5.3 Dependencies (SPEC-0004 r1: "runs after S1, S2 and S4")

| Sub-slice | Content | Needs on `main` |
|---|---|---|
| — (spec) | Copy §5.7 into SPEC-0004 as H-17 onward (Conductor, Spec stage) | this file in the repo; then my approval of SPEC-0004 |
| **S7a** | Cores: mod and master oscillators, expo and linear FM (non-TZ) | S0, S1, S2, S4, and the spec PR |
| **S7b** | Harmonics stage; the D7 oversampling check | S7a |
| **S7c** | Link: AM, expo-FM depth, soft sync | S7a |
| **S7d** | `not-Verbos` extras: TZFM, audio-rate mod | S7c |
| **S7e** | The voice: S4 LPG and envelopes, cap 8, patches, registry, button, per channel, demos | S3, S5, S7b–d |

If SPEC-0004's earlier slices change shape when they are built, follow them and report what moved.

### 5.4 Design (in SPEC-0004's terms)

**Interface and runtime.**
- Complex implements S0's `Voice` and registers in the H-16 registry.
- It uses S1's polyBLEP edge correction and seeded drift (D8: on, under 6 cents, seeded).
- It uses S4's LPG (vca, lpg and both modes, Parker & D'Angelo) and the shared RC envelopes.
- Cap **8** (D14), oldest note taken. Mono. Passes the master stage. Any page sample rate (44.1, 48, 96 kHz).

**Signal path** (per voice, per sample):
1. **Mod core.**
   - Phase accumulator; tri, square and saw outputs, **unipolar 0..1**, with polyBLEP edges.
   - Frequency is `note_hz*2^mod_ratio` (`mod_track = note`) or `mod_hz` (`free`), clamped to **0.0167–500 Hz** [S1].
   - With `extra_audio_mod` (*not-Verbos*): up to 20 kHz, `mod_ratio` −6..+4, bipolar outputs.
2. **Link** (`mod_mode`):
   - **off**: no link.
   - **fm, exponential** [S2]: `f = f0*2^(mod_amount*FM_EXP_OCT_MAX*m)`, with `FM_EXP_OCT_MAX` starting at 2.0 and set by my listening check in S7c.
   - **am** [S1, S2], after the harmonics stage: `y = x*((1−d)+d*(2m−1))`, `d = mod_amount`.
   - **`lin_fm`**: `f = f0 + k*m`, `k = lin_fm*INDEX_MAX*f_mod`, `INDEX_MAX = 10`. The phase increment is clamped at ≥ 0, so it never goes through zero.
   - **`extra_tzfm`** (*not-Verbos*): bipolar `m`, signed increment; the phase runs backwards and polyBLEP handles reversed wraps.
3. **Master core.** Phase → triangle → sine, range 3 Hz–20 kHz [S1]. Square and saw with polyBLEP feed the high-order path.
4. **Soft sync** (`sync`) [S1, S2]: on a master wrap, reset the mod phase to 0 only if it is above `SYNC_THRESHOLD = 0.9`.
5. **Harmonics stage**, memoryless, before the LPG:
   - **Symmetry:** `s = x + 0.5*symmetry`, with a 5 Hz one-pole DC blocker after the stage.
   - **Low order (fold)**, starting law (tunable; the rows rule): `y_fold = (1−c)*s + c*sin(π/2*g*s)`, with `c = min(1, 4*timbre)` and `g = 1+12*timbre`. Apply **first-order ADAA** to the sine term (antiderivative `−cos(a s)/a`, midpoint fallback when `|Δs| < 1e-5`) (Parker, Esqueda & Bilbao 2016).
   - **High order:** sine → saturated square → saw → saw at double phase as timbre rises [S1, S3].
   - **Order** blends the two: `y = (1−order)*y_fold + order*y_edge`.
6. **LPG and envelope.** S4's LPG and envelopes, strike or gate. `env_timbre` (−1..1) adds the envelope to timbre. Velocity drives the LPG (H-9).

**Oversampling (D7, D14).**
- S7b first measures the harmonics stage at 1× with ADAA, red-first, against H-27 (and H-3/H-6 via the registry).
- If a row fails at 1×, D7 allows 2× around the nonlinear stage, which here is the harmonics stage. Bring it in with a half-band FIR (31 taps, a starting value) and record both numbers in `handoffs/evidence/complex-s7b/fold-alias.txt`.
- H-13 then sets Complex's cap; it may lower 8, never raise it without asking.
- 2× anywhere else, such as the TZFM oscillator path, is not covered by D7 (QB1).

### 5.5 Parameters (per Complex patch; hard clamps)

| Param | Range | Default | Note |
|---|---|---|---|
| `mod_track` | note / free | note | |
| `mod_ratio` | −6..+2 oct | −4 | −6..+4 only with `extra_audio_mod` |
| `mod_hz` | 0.0167–500 Hz | 2 | `free` only |
| `mod_shape` | tri / square / saw | tri | |
| `mod_mode` | off / am / fm | off | panel `a.m./off/f.m.` [S1] |
| `mod_amount`, `lin_fm` | 0..1 | 0 | |
| `sync` | bool | false | soft |
| `timbre` | 0..1 | 0 | 0 = sine |
| `symmetry` | −1..1 | 0 | |
| `order` | 0..1 | 0 | 0 = fold, 1 = edges |
| `env_timbre` | −1..1 | 0 | |
| LPG and envelope params | as S4 defines | S4's | reused, not redefined |
| `extra_tzfm` | bool | **false** | **not-Verbos** |
| `extra_audio_mod` | bool | **false** | **not-Verbos** |

- **Patches.** Complex pluck, Folded bass, Ring bell and Vibrato lead. All have the extras off. I tune the values from S7e's WAVs.
- **Naming.** In the interface the voice is "Complex" (D5). "Verbos" appears only in specs, S0's ADR and code comments. `not-Verbos` marks the extras in code doc comments and the patch table. Their UI is QB3.

### 5.6 Provenance (for S0's ADR, per D12)

- **Sources.**
  - [S1] Verbos Complex Oscillator tech card, https://verboselectronics.com/wp-content/uploads/2025/10/ComplexOscillatorTechcard.pdf
  - [S2] Learning Modular, "Verbos Complex Oscillator: Frequency Modulation"
  - [S3] Verbos launch text, Matrixsynth, 2014
  - [S4–S6] ModWiggler threads 156800, 171937 and 285506
  - Välimäki & Huovilainen 2007 (polyBLEP)
  - Parker, Esqueda & Bilbao, IEEE SPL 2016 (ADAA)
  - Esqueda, Pöntynen, Parker & Bilbao, Applied Sciences 2017 (wavefolders)
  - Parker & D'Angelo, DAFx-13 (LPG)
- **What was not used.** No schematic, firmware or third-party DSP code. No benny code, patch or text.
- **Naming.** "Verbos" names the reference behaviour only and implies no affiliation (docs/10 R8).
- **The model.** Faithful by default (exponential internal FM, no through-zero, a unipolar mod capped at 500 Hz). The two `not-Verbos` extras are off by default. Cap 8 and oversampling per D14.

### 5.7 Observables to copy into SPEC-0004 as H-17 onward

**Conventions.**
- Tests run in Node on S0's rig, with drift off unless the row says otherwise. Use Complex's default patch unless the row names one.
- **Inharmonic energy** is the power in bins more than 3 bins from any `k*f0` below 20 kHz, in dB relative to the fundamental.
- **THD** covers harmonics 2–20.
- "From the first measurement" follows SPEC-0004 §5: the first green run sets the number with a stated margin, and it is only ever tightened.
- Test files go next to S0's voice tests, e.g. `voices/complex.test.ts`.

| # | Observable | Test |
|---|---|---|
| H-17 | **A sine at rest.** Complex with `timbre` 0, `mod_mode` off, `lin_fm` 0: THD under 0.5 percent from MIDI 24 to 96, and the output equals the master sine path sample for sample | `voices/complex.test.ts` |
| H-18 | **The modulator stays slow and unipolar by default.** With both extras off (the default and every shipped patch), the modulator's frequency is held inside 0.0167 to 500 Hz for any `mod_ratio`, `mod_hz` and note, its output stays inside 0 to 1, and no patch sets an extra | `voices/complex.test.ts` |
| H-19 | **Exponential FM moves the primary by a musical interval.** With a 5 Hz triangle modulator, the vibrato depth in cents at C2, C4 and C6 is equal within 1 cent, and rises with `mod_amount` without ever falling | `voices/complex.test.ts` |
| H-20 | **Linear FM never runs through zero by default.** With the extras off, for 10,000 random settings of `lin_fm`, note and modulator, the master's phase step is never negative | `voices/complex.test.ts` |
| H-21 | **AM from tremolo to ring.** `mod_amount` 0 in AM mode equals no modulation sample for sample; at 1 the carrier's bin is at least 40 dB under the sidebands; tremolo depth rises with `mod_amount` | `voices/complex.test.ts` |
| H-22 | **Soft sync.** The modulator's phase is reset only on a master wrap that finds it above 0.9 (from a test-only trace); `sync` off equals a voice without the sync path; two runs are equal | `voices/complex.test.ts` |
| H-23 | **Fold depth.** `timbre` 1, `order` 0, `symmetry` 0 at 100 Hz: 13 ± 1 local maxima per master half-cycle; the spectral centroid never falls as `timbre` rises from 0 to 1 in steps of 0.1, at `order` 0, 0.5 and 1 | `voices/complex.test.ts` |
| H-24 | **Symmetry makes even harmonics.** `symmetry` 0: even harmonics at least 60 dB under the fundamental at any `timbre`. At `timbre` 0.5 their total never falls as the magnitude of `symmetry` rises | `voices/complex.test.ts` |
| H-25 | **Order sweeps to the doubled saw.** At `order` 1 and `timbre` 1 the component at the note's frequency is at least 40 dB under the one at twice it; at `order` 1 and `timbre` 0.33 the odd harmonics carry at least 20 dB more than the even (a square) | `voices/complex.test.ts` |
| H-26 | **No jumps when turned.** A 2 s sweep of `timbre` and `order` over their full range at 220 Hz changes the output by at most 0.2 between consecutive samples | `voices/complex.test.ts` |
| H-27 | **The fold does not alias.** At full fold for MIDI 36 to 84, the inharmonic energy is under a bound from the first measurement, aimed at 50 dB under the fundamental. A fold without ADAA, as the control, fails the same bound. The rate the stage runs at (1x, or 2x under D7) is recorded beside the number | `voices/complex.test.ts` |
| H-28 | **The gate, driven by Complex.** Through S4's gate at `timbre` 0.8, level and brightness fall together with the ratio H-10 pins, within 10 percent; with `env_timbre` 1 the spectral centroid falls faster than the level; note-on and note-off at `timbre` 0 for notes up to 1 kHz never change the output by more than 0.05 between samples | `voices/complex.test.ts` |
| H-29 | **The cap.** Complex sounds at most 8 notes (D14, or the lower number H-13 sets); the ninth takes the oldest, and the taken note fades without a step over 0.05 | `voices/complex.test.ts` |
| H-30 | **Through-zero FM, when turned on (not-Verbos).** With `extra_tzfm` on and an analytic sine on the master's FM input: at a carrier of 0 Hz the output has no DC above minus 60 dB and its phase returns each modulator cycle (drift under 1e-4 over 10 s); a negative instantaneous frequency matches `sin(2π∫f dt)` within minus 60 dB; at a 1:1 ratio every sideband is a harmonic (nothing off-harmonic above minus 80 dB) | `voices/complex.test.ts` |
| H-31 | **Through-zero FM does not alias.** Carrier 2 kHz, modulator 3 kHz, index 5: inharmonic energy, measured against the analytic reference, under a bound from the first measurement aimed at 50 dB | `voices/complex.test.ts` |
| H-32 | **Audio-rate modulator, when turned on (not-Verbos).** With `extra_audio_mod` on, the modulator reaches 20 kHz and no further, `mod_ratio` reaches minus 6 to plus 4 octaves, and its output is bipolar with a mean within 0.001 of zero over 1 s | `voices/complex.test.ts` |
| H-33 | **In the registry, never the default.** Complex is in the voice registry, so H-16 runs H-2, H-3, H-6, H-9, H-11 and H-13 on it (H-13 at its cap); a fresh page's voice is Saw ladder (Plain before S3), never Complex; no string the interface shows names a hardware maker (D5) | `voices/registry.test.ts`, `voice.browser.test.ts` |

These rows are written red first in the sub-slices of §5.8. H-12 (events byte-identical) and H-1 (Plain unchanged) stay green throughout.

### 5.8 Sub-slices (each is one PR, red first, with a mutation run aimed at the DSP)

Mutants to aim at include: the polyBLEP sign, the ADAA term, the sync threshold, AM `d`, the TZ clamp, the 500 Hz clamp, the fold gain.

- **S7a. Cores.**
  - Rows: H-17, H-18, H-20, and H-2 via the registry.
  - Mod and master cores, expo FM, non-TZ linear FM.
  - No NaN or inf over a 3 Hz–20 kHz sweep at 44.1, 48 and 96 kHz.
- **S7b. Harmonics stage.**
  - Rows: H-23 to H-27.
  - Red at 1×. If H-27, H-3 or H-6 fails, bring in 2× under D7 and record both numbers.
- **S7c. Link.**
  - Rows: H-19, H-21, H-22.
  - Set `FM_EXP_OCT_MAX` after my listening check; the handoff names the value.
- **S7d. Extras.**
  - Rows: H-30, H-31, H-32.
  - H-18 stays green: the extras are off by default.
- **S7e. The voice.**
  - Rows: H-28, H-29, H-33, and H-13 at the cap.
  - Wire S4's LPG, add the patches, the registry entry, the `Voice` button entry (QB2) and the per-channel choice (S5).
  - WAVs of each patch go in `handoffs/evidence/complex-s7e/`.
  - The demo, §5.9.

### 5.9 Demo render

Use S0's renderer and SPEC-0004's demo: kick, hat and clap on ch 10 (unchanged), bass on ch 1, lead on ch 2.
- `handoffs/evidence/complex-s7e/complex_demo.wav` and `.mp3`: ch 1 Folded bass (timbre 0.8, order 0); ch 2 Complex pluck, then Vibrato lead (`mod_mode fm`, `mod_amount 0.3`), then `sync` on.
- A second file, `complex_demo_not_verbos.wav`, with `extra_tzfm` on for ch 2, labelled not-Verbos in its README row.
- If QA7 is approved, also render `examples/shape_demo.pattern` through Complex.

### 5.10 Risks

- **Aliasing.** The fold, and TZFM, at 1×: H-27 and H-31 measure it first; D7 or QB1 answer it, never a looser bound.
- **Cost.** Complex is the heaviest voice. With 2×, H-13 may push its cap below 8, and a phone is slower than Node (SPEC-0004 §8).
- **Interface churn.** S0's `Voice` may not fit two coupled oscillators with sync. Ask before changing it.
- **Browser maths.** `Math.sin` differs between engines, so equality runs in Node and browsers assert properties.
- **Branding.** D5 versus the `not-Verbos` UI label (QB3). docs/10 R8.
- **The scope gate's DSP terms** (§3.4).

### 5.11 Open questions (owner's call)

- **QB1 (gates S7d if H-31 fails at 1×).** D7 covers 2× only around the nonlinear stage. If through-zero FM aliases past its bound, which do you choose?
  - (a) Allow 2× on the master oscillator while `extra_tzfm` is on.
  - (b) Cap the TZFM index by pitch.
  - (c) Accept the measured number for this not-Verbos extra.
- **QB2 (gates S7e).** How does Complex appear in the `Voice` button: one entry using one patch, or one entry per patch?
- **QB3 (gates S7e).** How are the `not-Verbos` extras reached, given D5 and no per-parameter UI?
  - Only in patches, the renderer and tests.
  - As a separate labelled button entry: what label, and is "not-Verbos" acceptable in the UI?

Answered by SPEC-0004 r1 and removed from v2: the oversampling policy for the harmonics stage (D7/D14), Complex's voice cap (D14), whether Complex is ever the default (D14), bit-exact audio (D13; Q-S4-5 is SPEC-0004's).

---

## 6. What changed

**From v2 as first written** (the copy in the deleted `/workspace/out`):
1. **Base.** Moved to `c079f8f` (SPEC-0004 r1).
2. **Feature B renamed and resequenced.** It is now **SPEC-0004 S7 Complex**, with sub-slices **S7a–S7e** replacing C1–C5. It runs after S1, S2 and S4 per r1 (S2 is newly a prerequisite), and is gated by Q-S4-4 and my approval of SPEC-0004.
3. **No ADR-0015.** Per D12, S0's one ADR records Complex, and §5.6 is its provenance text. The ADR table now reserves only 0013 (A); S0's ADR takes the next free number (QG3, reworded).
4. **D13 and D14 adopted.**
   - TypeScript only.
   - Cap 8 provisional, lowered by H-13.
   - Oversampling per D7: 2× around the harmonics stage is now allowed when a row needs it.
   - Never the default.
5. **New §5.7.** Rows H-17 to H-33, ready to copy into SPEC-0004. H-33 ties Complex to the H-16 registry.
6. **Questions removed** because r1 answered them: the oversampling choice for the folder (old QB1 a), Complex's voice cap (old QB4), Complex as default (old QB5). QB1 is narrowed to TZFM oscillator oversampling.
7. **Feature A.**
   - It now starts with a SPEC-0006 Spec-stage PR and my written approval, per the Conductor's r1 journal and the lifecycle.
   - Drift-specific text is generalised to "any `ChordStyle`", because SPEC-0005 r1 (#68, open) switches Drift for Curve.
   - New QA8 asks whether Feature A answers Q-S5-2 and retires r1's "Shape ornaments" candidate.
   - Task IDs: S0 is planned as WENGE-0032, so take the next free ID at PR time.

**From v1** (`6a14539`), still true:
- the Rust `octovoice`, `octorun --wav`, `examples/golden-wav/`, `voice` pattern lines, octoweb voice exports and bit-exact native/wasm audio are dropped;
- Feature A rebased (`PatternStyle`, distinct salts, isolation from `ChordStyle`, SPEC-0005 goldens join the byte-identical gate);
- the demo renders through SPEC-0004's renderer (QA7);
- the "build Feature A only" hold is removed.

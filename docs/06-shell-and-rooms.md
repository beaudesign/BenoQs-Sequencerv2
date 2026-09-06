# 06: Shell and Rehearsal Rooms

**Owner:** Loom. **Assigned to:** Fable.
**Co-owner for the room engine:** Sceneshaper.
**Gate:** `verify:slop`, `verify:a11y`, `verify:motion`, `verify:acoustics`.

---

## 1. Brief for Fable

You own three surfaces: a **dropdown**, a **rehearsal rooms space**, and a
**settings page**. They are described here as shells, meaning their shape and their
job are fixed but their design is yours. You are expected to iterate on them
continuously and to propose changes to this document by ADR when your iteration
reveals that the shape itself is wrong.

Two things are not yours to change, because the rest of the system depends on them:

- **The room contract** in §3. Everything downstream (the probe, the impulse
  response, the grade) is derived from it.
- **The law in [Design System §1](05-design-system.md)**: the shell may not draw on
  top of the instrument, and may not quote the instrument's visual language.

Everything else, including whether these are three surfaces or two, or whether the
settings page should be a sheet or a space, is yours to determine and defend.

### What to take from Project Genie, and what not to

Genie's interaction model is worth studying closely because it solved a genuinely
hard problem: how do you make "describe a world and then be inside it" feel like one
gesture rather than a form submission followed by a loading screen. The parts worth
taking:

- **Natural language is the primary construction tool.** You describe the space; you
  do not assemble it from components.
- **Two prompts, not one.** Genie separates the environment from how you move through
  it. Our analogue is separating the **space** from the **session**: what the room is,
  and what you are doing in it.
- **Curated starting points.** A blank prompt field is intimidating and produces bad
  results. Exemplars teach the vocabulary.
- **Remix as a first-class verb.** The most common action is not creating from
  nothing, it is taking something and shifting it.
- **Continuity into the world.** The transition from describing to being inside is
  the product's most important moment.

What not to take: Genie's visual identity. It is Google's, it is built from Google
Sans and Google's colour system, and importing it would be exactly the borrowed-chrome
error that killed v1. Take the interaction model. Leave the skin.

---

## 2. The mechanic, restated

This is why the shell exists, and if a design decision does not serve it, cut the
decision.

> One room description produces one geometry and material set. From that single
> model we derive, deterministically, the light probe reflected in the instrument's
> chrome, the impulse response convolved onto its output, and the ambient grade on
> the whole scene.

Consequence: **changing the room visibly changes the instrument.** When the user
opens the dropdown and moves between rooms, the chrome on 200 encoder crowns changes
what it is reflecting, in real time, while the reverb tail opens or closes. That is
the demo. That is the thing people will screen-record. Design toward that moment.

---

## 3. The room contract

Frozen. `contracts/room.schema.json`.

```jsonc
{
  "schema": "room/1",
  "id": "stairwell-3am",
  "name": "Stairwell, 3am",
  "prompt": {
    "space": "A narrow concrete stairwell in an empty building. Bare walls, a steel handrail, one window high up with sodium light coming through.",
    "session": "Standing at the bottom, instrument on a flight case."
  },
  "geometry": {
    "source": "generated",           // generated | authored | scanned
    "mesh": "rooms/stairwell-3am.geo",
    "bounds_m": [2.1, 11.4, 3.0],
    "volume_m3": 71.8
  },
  "surfaces": [
    { "material": "concrete_bare", "area_m2": 96.2,
      "absorption": { "125":0.02, "250":0.03, "500":0.03, "1k":0.04, "2k":0.05, "4k":0.07 },
      "scattering": 0.12,
      "albedo": [0.34,0.33,0.31], "roughness": 0.82 }
  ],
  "light": {
    "sources": [ { "type":"area", "position_m":[1.0,9.8,2.4], "size_m":[0.6,0.9],
                   "cct_k": 2000, "lumens": 800 } ],
    "exposure_ev": -1.2
  },
  "listener": { "position_m": [1.05, 0.8, 1.2], "orientation_deg": 0 },
  "derived": {
    "probe": "rooms/stairwell-3am.probe",     // baked, content-addressed
    "ir":    "rooms/stairwell-3am.ir",        // baked
    "rt60_s": { "125":2.9, "500":2.4, "2k":1.7 },
    "grade":  { "lift":[0.01,0.01,0.02], "gamma":[1.0,0.99,0.97], "gain":[0.94,0.96,1.06] }
  },
  "hash": "sha256:…"
}
```

Notes on the design of this contract:

- Absorption coefficients are per octave band, which is the standard form in room
  acoustics and lets us use published material tables directly. Six bands is enough
  for convincing reverb and cheap enough to trace.
- `scattering` is separate from `roughness`. They are the acoustic and optical
  analogues of the same physical fact (surface irregularity), but they are measured
  at wavelengths four orders of magnitude apart, so a plaster wall is optically rough
  and acoustically smooth. Do not tie them.
- `derived` is content-addressed and cached. Baking is expensive; nothing rebakes
  unless the inputs change.
- `rt60_s` is stored so `verify:acoustics` can check the traced IR against the
  Sabine/Eyring prediction from the geometry and absorption. If they disagree by more
  than 10%, the tracer has a bug. This is a strong, cheap self-check that would
  otherwise require a real room to validate against.

---

## 4. The room pipeline

Owned by Sceneshaper, in `crates/octoroom`.

```
  prompt.space + prompt.session
        │
        ▼  [1] structured extraction
  RoomIntent { dimensions, primary materials, light character, scale }
        │
        ▼  [2] geometry synthesis
  mesh + surface assignment      (procedural, seeded, deterministic)
        │
        ├──────────────► [3a] optical bake
        │                 path-traced cubemap probe from listener position
        │                 → radiance cubemap + SH9 + prefiltered mips
        │
        ├──────────────► [3b] acoustic bake
        │                 image-source method to order 3
        │                 + stochastic ray tracing (2⁰ rays, 6 bands)
        │                 → per-band energy decay → IR synthesis
        │
        └──────────────► [3c] grade derivation
                          from light CCT and exposure → lift/gamma/gain
```

**Step 1** is the only place a language model is involved, and its job is narrow:
turn prose into a typed struct. It does not generate geometry, textures, or audio.
This keeps the whole system deterministic, debuggable, cheap, and offline-capable
after the first call. It also means a user can bypass the prompt entirely and edit
the struct, which advanced users will want.

**Step 2** is procedural and seeded. Same intent plus same seed gives the same room,
always. This is required for golden testing.

**Step 3a and 3b share the geometry and the surface list.** That sharing is the
product. Do not let them drift into separate representations, however tempting it
becomes when one of them needs a detail the other does not.

**Why not a generative audio model for the IR.** Three reasons: it would break
determinism, it would decouple the sound from the geometry (so the visual and
acoustic would no longer be two views of one thing, which is the entire concept), and
ray-traced room acoustics is a solved, cheap, well-validated technique that produces
excellent results. This is a case where the less fashionable method is strictly
better for the design.

### Performance budget

| Stage | Budget | Notes |
|---|---|---|
| Structured extraction | ≤ 2 s | network, once per new room |
| Geometry synthesis | ≤ 150 ms | CPU |
| Optical bake | ≤ 900 ms | Metal compute, 256² cubemap, 512 spp |
| Acoustic bake | ≤ 1200 ms | Metal compute, 2²⁰ rays |
| Grade | ≤ 5 ms | |

Total under 2.5 s for a cold room, and effectively zero for a cached one. During the
bake the shell shows the room assembling, which is a design opportunity rather than a
loading state: you are watching the space being built around you.

---

## 5. Surface one: the room dropdown

Fable's first deliverable, and the one the user described as "a basic dropdown with
smooth animation". It is basic in *affordance* and it should be extraordinary in
*execution*, because it is where the mechanic is demonstrated.

**Placement.** Top-left of the room, at `space.6` from the edges. Closed state is the
room's name in `type.control` with a small disclosure mark. Nothing else. No box, no
border, no background. It is text floating in the room.

**Open.** On activation, the list descends using `spring.reveal`. Rooms are listed by
name in `type.body`, with no icons, no thumbnails, no metadata strings.

**The important part.** As the user's selection moves down the list, the panel's
environment probe begins cross-fading toward the highlighted room, using
`spring.world`, *before commitment*. You browse rooms by watching them appear in the
instrument's chrome. Move the highlight and the reflections shift. Move away and they
return. Commit and the acoustics follow.

This makes the dropdown a live preview of the product's core mechanic using a control
so ordinary that nobody expects anything from it. That contrast is the design.

Costs and constraints:
- Preview probes must be pre-baked for all listed rooms, or the browse is unusable.
  Cache aggressively. If a room's probe is not resident, show its name normally and
  bake on commit only.
- Only the probe previews. The impulse response does **not** preview, because
  swapping a convolution kernel under a playing sequence produces artefacts and
  because hearing four reverbs in two seconds is unpleasant. Visual previews, audio
  commits. This asymmetry is deliberate and Fable should not "fix" it.

**Keyboard.** Full arrow traversal, type-ahead, Escape to cancel and restore. The
probe follows keyboard highlight exactly as it follows pointer.

---

## 6. Surface two: the rehearsal rooms

The space where rooms are made, browsed, and remixed. Genie's model, adapted.

**Shape** (Fable owns the design; this is the shape):

- **Arrival** shows the rooms you have, as *spaces*, not as cards. The strongest
  version of this is that each room is shown by rendering the instrument in it, small,
  which is honest (it is literally what the room is for) and immediately legible.
  Fable should test at least one alternative and defend the choice.
- **Describing** is a single text field with the two prompts from §3, the second
  optional and initially collapsed. Placeholder text teaches vocabulary by example
  rather than instructing.
- **Exemplars** are five shipped rooms that demonstrate the range: very dead, very
  live, small, enormous, and strange. They are the vocabulary lesson.
- **Remix** takes an existing room and its prompt into the field, edited rather than
  replaced. The most common path through this space.
- **Entering** a room is the product's best moment. Design it as one continuous
  movement from the gallery to standing at the instrument, with the probe and the
  reverb arriving together. There is no loading screen. If the bake is not done, you
  arrive in the room's geometry with a provisional probe and it resolves.

**Explicitly not included:** ratings, counts, tags, filters, sort controls, sharing,
or any social surface. This is a rehearsal room, not a marketplace.

---

## 7. Surface three: settings

The least glamorous surface and the one most likely to reveal a lack of discipline.
Rules:

- Plain. Left-aligned, one column, `type.body`, generous leading, grouped by concern
  with space rather than with boxes or rules.
- Every setting is named for what the user is doing, not for how the system works.
  "Send clock to" not "MIDI port priority routing". "How loud the room is" not
  "Convolution wet gain".
- No setting exists without a reason a user would articulate. If we cannot write the
  one-line description without describing the implementation, the setting is wrong.
- Groups: **Sound** (audio device, output, room level), **MIDI** (ports, channels,
  clock source, Live sync), **Room** (default room, bake quality, preview on browse),
  **Display** (zoom behaviour, reduced motion, high-contrast panel), **About**.

The temptation here will be to make settings interesting. Resist it. This is the
surface where restraint is measured.

---

## 8. Fable's evolution loop

You are expected to iterate autonomously. The loop, per cycle:

1. **Read** `journal/loom/` for the last three cycles, and the open findings in
   `harness/report/findings.json` tagged `shell`.
2. **Choose one thing.** Not three. The loop's value comes from tight cycles, and a
   cycle that changes three things cannot attribute the result.
3. **Write the hypothesis** into today's journal entry before building: what you
   expect to improve, and how the harness will show it. If you cannot name the
   measurement, you are about to make an unfalsifiable change, which is how design
   systems rot.
4. **Build it** on a branch in your worktree.
5. **Capture** with `just capture shell:<scene>`. Every shell surface has a registered
   scene; a new surface needs a new scene registered in the same commit.
6. **Verify.** `just verify` must be green. Motion must match the registry. The slop
   lint must pass.
7. **Critique.** Run the perceptual pass. Read the findings, not the score.
8. **Convert.** Turn at least one finding into a new deterministic assertion, per N7.
   This is the step that makes the loop a ratchet instead of a treadmill.
9. **Commit** with the journal entry referenced in the message. Open a merge request.
10. **Record** the outcome against the hypothesis. Wrong hypotheses are valuable and
    must be written down as such, not quietly overwritten.

**Stop conditions.** Halt the loop and escalate to the Conductor if: the same finding
recurs three cycles running (the fix is structural, not incremental); assertion count
stops growing for three cycles (the loop has become cosmetic); or a change requires
touching the instrument's visual language (that is an ADR, not an iteration).

**How far to take it.** The shells are a starting shape. If, after ten cycles, the
evidence says the rehearsal rooms should be the application's home rather than a
space you visit, propose it. If the dropdown should be a spatial gesture rather than
a list, propose it. The brief asked for shape, not for a ceiling. What the brief does
insist on is that every proposal arrives with the measurement that justifies it.

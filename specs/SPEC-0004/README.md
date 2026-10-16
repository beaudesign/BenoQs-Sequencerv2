# SPEC-0004: A built-in sound that behaves like hardware

| Field | Value |
|---|---|
| Spec | SPEC-0004 |
| Revision | r0, draft, 2026-10-16 |
| Stage | **Planning only.** This pull request adds the spec and a journal entry and changes no code, contract, fixture or threshold |
| Risk tier | **Low** for this pull request (`specs/`, `journal/`). Every slice it plans is **Medium** (`apps/web/`, Forge); none touches `contracts/`, the engine or the ABI |
| Owner and approver | `@beaudesign` |
| Task | `WENGE-0030` |
| Extends | SPEC-0002 P6 requirement R-2 (the built-in sound, `apps/web/src/monitor.ts`) |
| Sister spec | SPEC-0005 (a chord style that drifts), on its own branch. The two do not depend on each other |
| Base | `main` at `6a14539` |

## 1. The ask

Recorded here because chat is not a source of truth. On Fri 2026-10-16 (17:15 London, from a phone) the owner wrote:

> new oscillator types that are played through the sequencer - benny has got a great example of real sounding oscillators
> and filters modelled on physical hardware. Let's explore how we can bring that in instead of piano.

(`https://github.com/playbenny/benny/`, "let's explore and plan".)

**The reading.** "Piano" is the page's built-in sound: today a note is a sine with two harmonics (`monitor.ts`), which
the owner hears as a piano or an organ. "Played through the sequencer" is read as: the engine's notes drive the new
voices exactly as they drive that tone, and nothing the engine or MIDI does changes. "Instead of" is read as: the new
voices become what the page sounds like when it opens. If the owner meant benny's voices to be reachable as devices on
the MIDI output, that is a different product (a synth, not a monitor) and is not planned here.

## 2. What the page's sound is today (read from the code, not recalled)

- Pitched note: `sin(p) + 0.5 sin(2p) + 0.25 sin(3p)`, over 1.75. Envelope: 3 ms attack, a 60 ms fall to 0.6, a 100 ms
  release. 24 voices, the oldest is taken. Master is `tanh`. One output channel.
- Drums: three one-shots on MIDI channel 10 (a falling sine, noise, high-passed noise). The `Drums` button silences them.
- It is audio only: it reads the engine's events, changes none, and keeps nothing the engine can see (P6, P10). Same
  notes in, same samples out: no `Math.random`, one xorshift generator.
- Its own header says it is "deliberately plain ... not the instrument's voice". That was right for a way to hear the
  page run. This spec changes that stance on purpose (decision D1).

## 3. What benny is, and what we may take from it

Read on 2026-10-16: the repository's README, `LICENSE.md`, `CHANGELOG.md`, the block descriptors (`help_text` of each
block) and the names of its code files. benny is a modular live-music environment built from **Max/MSP patches**
(`.maxpat`, `.gendsp`, `.genexpr`). It cannot be run or ported to a browser; only its ideas can be used.

**Licence.** benny is under the **Hippocratic License 3.0**. It lets anyone use and change the software, on conditions: a
list of ethical standards must be kept, the licence and notices must travel with any part of it, and any derivative work
must be licensed under the same terms (its sections 3, 5.1, 5.3 and 8.3). This repository has no licence file at its
root, so taking any part of benny's code would raise a question the owner has not decided. The plan therefore takes
**nothing from benny but the idea of a block** and implements each piece **from the published paper it points at**:

- No patch, code, preset, sample or text of benny is copied, translated or adapted. The slices cite a paper for each
  algorithm, and a reviewer can check the implementation against the paper without opening benny.
- While surveying I opened one benny file that holds its voice allocator, which this plan does not need. Nothing from it
  is used; the monitor keeps its own oldest-voice rule.
- Some of benny's pieces are third-party code (an oscillator "from Yofiel.com", filter models by Surreal Machines) or
  recordings of the author's own hardware. Those are not benny's to give and are not taken.

| What benny has (its own description) | The hardware idea | Implement from | Taken? |
|---|---|---|---|
| `source.basic.osc`: a shape control that fades sine, triangle, saw, rectangle; saw and triangle made by integrating a band-limited rectangle | A morphing analogue oscillator without aliasing | Välimäki and Huovilainen, "Antialiasing oscillators in subtractive synthesis" (IEEE Signal Processing Magazine, 2007): polynomial band-limited steps | **Yes** (S1) |
| `source.dual.osc`: two oscillators with cross modulation; `quanti.slide.osc` "unstable pitch" | A pair of oscillators with sync and a little drift | The same paper (sync); a seeded slow random walk for drift | **Yes** (S1) |
| `fx.filter.2pole`, `utility.sacred.channel`: a zero-delay-feedback 2-pole filter, "a 12 dB shape morphing filter similar to a SEM" | A state-variable filter whose low-pass, notch and high-pass are one control | Simper (Cytomic), "Solving the continuous SVF equations using trapezoidal integration and equivalent currents"; Zavalishin, "The Art of VA Filter Design" | **Yes** (S2) |
| a Sallen-Key filter with a saturating loop (`sm_sallenkey_functions`) | The bite of a 2-pole Sallen-Key low-pass at high resonance | Zavalishin, "The Art of VA Filter Design" (chapter numbers pinned in S2) | **Yes** (S2) |
| `fx.filter.vactrol.lpg`: "based on the paper and example patches by Julian Parker and Stefano D'Angelo" | A low-pass gate: level and brightness fall together, with a slow, uneven vactrol | Parker and D'Angelo, "A digital model of the Buchla lowpass-gate" (DAFx-13), the paper benny itself cites | **Yes** (S4) |
| `fx.filter.fixed.bank`, a 914-style bank | A fixed filter bank | | No (an effect, not a voice) |
| `source.sacred.waves.osc`: wavetables "sampled from a Prophet 600 and a MonoPoly" | Real hardware waveforms | Needs recordings | **No.** They are the author's samples. If the owner supplies their own, a later slice can play them |
| `voice.organ`, `voice.harmonic`: drawbar organ with key-contact behaviour | An organ | | Later (S6, optional) |
| `fx.wavefold` ("roughly after the Doepfer one"), tape and thermal mixers | Wavefolder, saturation | | No (effects) |

A 4-pole **ladder** low-pass is not in benny's list but is the filter most people mean by "analogue" (Huovilainen,
"Non-linear digital implementation of the Moog ladder filter", DAFx-04; Stilson and Smith, ICMC 1996). It is in S2.

## 4. What the slices build

Everything is in `apps/web/src/voices/` and runs in the AudioWorklet, in the same place and under the same rules as
`monitor.ts` (no allocation per sample, no `Math.random`, nothing that differs between two runs). The engine, the
events, the ABI and Web MIDI are untouched.

| Slice | Content | Tier |
|---|---|---|
| **S0** | A `Voice` interface; the current tone moved behind it as **Plain**, sample for sample; a test rig that measures spectrum, pitch, response and cost; an offline renderer that writes a WAV of any voice playing the demo (so the owner can listen on a phone). an ADR records D1 | Medium |
| **S1** | The oscillator: saw, pulse and triangle with a shape morph, a sub oscillator, hard sync, seeded drift | Medium |
| **S2** | The filters: state-variable (low-pass to notch to high-pass), 4-pole ladder, Sallen-Key, each with a saturating loop and key follow | Medium |
| **S3** | The first voices and the `Voice` button in the strip: **Saw ladder** (mono, bass and lead) and **Pulse morph** (pad, chords). The default sound becomes Saw ladder | Medium |
| **S4** | **Gate pluck**: a triangle through the vactrol low-pass gate, and the analogue (RC) envelopes the others share | Medium |
| **S5** | A voice per MIDI channel: the demo's bass (channel 1) and lead (channel 2) get different voices; a small choice in the strip | Medium |
| **S6** | Optional: Sallen-Key voice, drawbar organ | Medium |

The demo is the test pattern: kick, hat and clap on channel 10 (unchanged), bass on channel 1, lead on channel 2.

## 5. Observables (written red first)

"Sounds like hardware" is a taste judgement, and CLAUDE.md turns each of those into a measurement. What can be measured
is the behaviour that makes a circuit sound like itself: it does not alias, it tracks pitch, its filter has a known
slope and rings when pushed, it saturates instead of clipping, its envelope is a charging capacitor. Where a number is
marked "from the first measurement", the first green run sets it with a stated margin and it is only ever tightened.

| # | Observable | Test |
|---|---|---|
| H-1 | **Plain is unchanged.** With the Plain voice the monitor's output equals today's, sample for sample, and every existing `monitor.test.ts` and worklet fixture passes untouched | `monitor.test.ts` |
| H-2 | **Pitch.** Each voice's fundamental is within 2 cents of the note for MIDI 24 to 96; drift stays inside 6 cents and has mean zero over 10 s | `voices/osc.test.ts` |
| H-3 | **No aliasing.** The energy of components that are not harmonics, for a saw on MIDI 60 to 84, is below a bound set from the first measurement (aimed at 50 dB under the fundamental). A naive saw, as the control, fails the same test, which proves the test can fail | `voices/osc.test.ts` |
| H-4 | **Filter response.** State-variable low-pass: minus 3 dB within 0.5 dB at the cutoff for Q 0.707, 12 dB per octave within 1.5 dB two to four octaves up. Ladder: 24 dB per octave within 2 dB. Sallen-Key: 12 dB per octave. Each from a swept sine, as a circuit would be measured | `voices/filter.test.ts` |
| H-5 | **Resonance.** At maximum resonance the ladder rings at its cutoff within 1 percent, and its level stays under 1 because the loop saturates (a hardware ladder does not blow up) | `voices/filter.test.ts` |
| H-6 | **Stability.** 10,000 random settings of note, cutoff, resonance and drive at 44.1, 48 and 96 kHz: no NaN, no infinity, nothing above 1 after the master stage, and silence within 5 s of the last Note Off | `voices/stability.test.ts` |
| H-7 | **Key follow.** With key follow at 100 percent the cutoff-to-pitch ratio of the response is the same within 3 percent from MIDI 36 to 96 | `voices/filter.test.ts` |
| H-8 | **Envelope.** Decay and release are one-pole charges: the time to 1/e is within 3 percent of the stated constant, and a voice ends at exactly zero, never in a step (the rule the monitor already keeps) | `voices/env.test.ts` |
| H-9 | **Velocity.** A louder note is brighter as well as louder: the spectral centroid rises with velocity by a bound from the first measurement | `voices/voice.test.ts` |
| H-10 | **Low-pass gate.** Level and brightness fall together, and the fall is slower than the rise by the ratio the paper's model gives, within 10 percent (pinned in S4 from the paper) | `voices/lpg.test.ts` |
| H-11 | **Deterministic.** Two runs of the same events are sample for sample equal, including drift | `voices/voice.test.ts` |
| H-12 | **Not MIDI.** The events the worklet sends are byte for byte the same with any voice (the P10-5 rig) | `monitor.test.ts` |
| H-13 | **Cost.** 24 notes of the heaviest voice render 10 s of audio in under 25 percent of real time in Node, the bound set at S0 on the CI machine; plus a text check that no sample loop in `voices/` allocates (no `new`, array literal, `.map(` or closure inside it) | `voices/cost.test.ts` |
| H-14 | **The page.** The `Voice` button reaches the worklet; the chosen voice is heard (the AudioNode tap) and its output differs from Plain; the choice made before Play is kept | `voice.browser.test.ts` |
| H-15 | **The strip keeps its rules.** The button uses the existing raised and pressed tones and tokens, adds no colour, gradient, shadow or easing (`app_source_rules.rs`, `app-css.test.ts`) | existing gates |

**What cannot be measured:** whether it pleases the owner's ear. S0's renderer exists so the owner can listen to every
voice on a phone before S3 is built, and the first question (Q-S4-1) is theirs.

## 6. Decisions for the owner (defaults are taken; change any)

| # | Decision | Default and why |
|---|---|---|
| D1 | The page's sound stops being a "deliberately plain" check and becomes a feature | **Yes.** It is what was asked. The Octopus stays silent; the sound remains audio the page makes, never MIDI. Recorded as an ADR in S0 (numbered when it is written) |
| D2 | What happens to the piano-like tone | **Kept as the voice `Plain`**, selectable. It is the sample-exact baseline of today's tests and the fallback on a slow phone. The default sound becomes Saw ladder at S3 |
| D3 | Which voices first | Saw ladder, Pulse morph, Gate pluck, in that order. A Sallen-Key voice and an organ come after |
| D4 | The control | One `Voice` button in the strip that names the voice and steps through them, as the other buttons name their state. A voice per channel (S5) is a small second control. No panel control is added: the panel is the Octopus's |
| D5 | Names | Plain descriptions (Saw ladder, Pulse morph, Gate pluck). No hardware brand in the interface; the lineage is in this spec |
| D6 | Stereo | **Mono**, as the output is today. Detune spread would need a second channel and the metronome and drums are mono |
| D7 | Oversampling | None to begin with (polynomial band-limiting and a cheap saturator). 2x around the saturating filter only if H-3 or H-6 cannot be met without |
| D8 | Drift | On, small (within 6 cents), seeded, so two runs are equal |
| D9 | The drums | Unchanged. Better drums are a separate question |
| D10 | Licence | Clean-room from the papers (section 3). Whether the repository should carry a licence file is the owner's, and not decided here |
| D11 | Is this wanted in the first place beside SPEC-0003's circle work | Planned as independent; it can wait behind S1 of SPEC-0003 without harm |

## 7. Questions only the owner can answer

- **Q-S4-1.** After S0's WAVs: of Saw ladder, Pulse morph and Gate pluck, which character do you want first, and what is
  the song they should be good at? Default: Saw ladder, on the demo's bass and lead.
- **Q-S4-2.** Should the old tone be kept at all? Default: yes, as `Plain`, until a voice has been the default for a week.
- **Q-S4-3.** Is the sound for listening to the sequencer (a monitor) or a voice you play to a recording? Default: a
  monitor. A recording path (a bounce) is not planned.

## 8. Risks

- **Cost on a phone.** An AudioWorklet block is 128 frames, about 2.7 ms at 48 kHz, and 24 voices with a nonlinear
  filter each is the heaviest the page does. H-13 gates it in Node; a phone is slower. Mitigation: a voice cap per
  voice, Plain as the fallback, and cheap approximations for `tanh` and `exp`.
- **Denormals and instability** near Nyquist or at full resonance: H-6 sweeps for them; filters flush tiny values.
- **Browser differences.** `Math.sin` and `Math.exp` can differ in the last digit between engines, so equality
  tests run in Node and the browser tests assert measured properties, never a hash.
- **Aliasing at high notes** with no oversampling: H-3 measures it and D7 says what to do.
- **Taste.** The numbers prove a circuit is modelled, not that it is liked. That is Q-S4-1.

## 9. Not done

Effects (reverb, chorus, wavefolder, tape), MIDI-addressable devices, recording or bouncing, patch storage, a new look
for the strip beyond one button, and any change to the drums, the engine, the contracts or the ABI.

## 10. Sources

- benny, `https://github.com/playbenny/benny/` and its manual `https://playbenny.github.io/benny_manual/`, read 2026-10-16. Hippocratic License 3.0, `LICENSE.md`.
- Välimäki and Huovilainen, "Antialiasing oscillators in subtractive synthesis", IEEE Signal Processing Magazine 24(2), 2007.
- Simper (Cytomic), "Solving the continuous SVF equations using trapezoidal integration and equivalent currents", 2013.
- Zavalishin, "The Art of VA Filter Design" (Native Instruments, free to read).
- Huovilainen, "Non-linear digital implementation of the Moog ladder filter", DAFx-04. Stilson and Smith, "Analyzing the Moog VCF with considerations for digital implementation", ICMC 1996.
- Parker and D'Angelo, "A digital model of the Buchla lowpass-gate", DAFx-13 (`http://www.acoustics.hut.fi/publications/papers/dafx13-lpg/`).
- This repository: `apps/web/src/monitor.ts`, `worklet.ts`, `demo.ts`; `specs/SPEC-0002/p6-runs-when-opened.md` (R-2), `p10-silence-the-drums.md`.

## 11. Approval record

Not yet approved. Awaiting `@beaudesign`. Merging is the owner's act; the agent merges nothing.

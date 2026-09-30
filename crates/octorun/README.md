# octorun

Runs a pattern through the sequencer core with no host, no audio device and no clock, and
writes what it played. Owner: Conductor. SPEC-0001 O7 (WENGE-0007).

```
just run examples/hello          # writes target/octorun/hello.ndjson and hello.mid
octorun PATTERN --stdout         # the event log on standard output
octorun PATTERN --golden         # the two-line text kept in examples/golden/
```

A pattern is the conformance fixture language (`crates/octocore/src/fixture.rs`, DSL v1 to v3),
so a pattern file can check itself with `expect` lines. It may render at most an hour at
48 kHz and produce at most 5 million events.

## Output, byte for byte the same every run

- **Event log** (`NAME.ndjson`, format `events/1`): a header line, then one JSON object per
  event with its absolute sample position, in the order a synth would receive them. See
  `src/ndjson.rs` for the exact shape.
- **Standard MIDI File** (`NAME.mid`): format 1, 960 ticks per quarter, a tempo track, and one
  track per port with a MIDI port prefix. Sample positions become ticks through the pattern's
  tempo map. Opens in any DAW.
- **SHA-256** of each, printed on the last line and kept for five patterns in
  `examples/golden/`. `cargo test -p octorun` fails if the engine plays one of them
  differently. If that is intended, `just golden`, review, and say why in the commit.

## Determinism

The engine is seeded and uses no clock, thread or floating-point function that differs between
platforms, so the hashes are the same on any machine. `just wasm-smoke` checks the strongest
form of that claim: the engine compiled to WebAssembly, driven through the C ABI, writes the
identical log for the patterns the C ABI can program.

## Not done

- A live mode that sends real MIDI through `midir`. It needs an owner decision on the
  dependency (SPEC-0001 Q6) and a MIDI device to test against, and this environment has neither.
- Phrase and MCC setters in the C ABI, so that `phrases` and `mcc_and_transport` can also run
  in the WASM smoke test.

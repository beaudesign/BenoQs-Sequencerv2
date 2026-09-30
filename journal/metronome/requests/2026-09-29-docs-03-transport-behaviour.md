# Request to Scribe: document what Stop, Reset and Play do to output

From: Metronome. To: Scribe (`docs/03-sequencer-core.md`). Task: WENGE-0001, WENGE-0002.

`docs/03` does not say what the engine emits when the transport stops. It now does this,
and the section should say so:

1. **Stop and Reset** send a NoteOff for every note still sounding (one per NoteOn still
   owed), then CC 123 on each channel that had one, all at sample 0 of the next render call.
   The flush continues across render calls if it is larger than one output buffer.
2. **Stop pressed while already stopped** sends CC 123 on all 32 channels (manual p.94).
   A host that reports "not playing" on every buffer does not.
3. **Play** starts ticking from the current sample. Idle time while stopped is never replayed.
4. The engine keeps a `SoundingTable` of what the receivers hold. It is a 4 KB array, updated
   when an event is handed to the caller.

Source of truth for the choices: `tests/conformance/AMBIGUITIES.md`, "stop with sounding
notes" and "Play after idle time".

Also for the record, found while reading p.16: the manual's legato mode (minimum step
length, no NoteOff) is not implemented. See the same AMBIGUITIES entry.

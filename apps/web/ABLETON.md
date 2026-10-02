# Using the web sequencer with Ableton Live

**Not written yet.** This is the skeleton P4a leaves for P4e (SPEC-0002 P4, `specs/SPEC-0002/p4-plan.md`). Nothing below has been
run against Live, and it will say which version of Live it was checked against, or that none was available.

## What it will cover

1. **What you need.** Chrome, Edge or Opera; a secure context; one permission prompt. macOS: the IAC Driver in Audio MIDI Setup.
   Windows: loopMIDI.
2. **The app sends notes to Live.** The port, the channel, and Live's *Track* switch on the input port.
3. **Live sends its clock to the app.** Live's *Sync* switch on the output port, the app's clock state set to slave, and what
   "Ext" on Live's transport does.
4. **Starting and stopping together.**
5. **The constant delay.** The app's offset and Live's *MIDI Clock Sync Delay*.
6. **A test procedure** for spike S3 (Live master at 120 BPM over a virtual port, 30 minutes) and what to record.
7. **Known gaps.** The tick (D0) is not settled, so a pattern is not yet in time with a clock a human hears.

Sources for the routing steps are in `specs/SPEC-0002/tech.md` section 6 and the references at its end.

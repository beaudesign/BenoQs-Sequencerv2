# SPEC-0008 bench: the runs only a person with gear can do

Companion to `README.md`. Everything here is a **procedure for a person**, not a test the CI can run. Nothing in it has been run: there was no hardware where it was written. It is
the way the page's claims about timing, hanging notes and sync meet a real port and a real instrument, and the way criteria A3, A4 and A6 of SPEC-0002 get their numbers.
It proposes **no threshold**. The owner sets one from what a run shows, as SPEC-0002 already says, and a threshold is only ever tightened afterwards.

## 0. Rules for every run

1. **Name the gear.** A run with no gear table is not a run. Put this at the top of the record:

   | Item | Record |
   |---|---|
   | Date, and the commit the page was built from (the build line of the S3 report gives the module hash) | |
   | Computer, operating system and version | |
   | Browser and version; whether it is a tab or an installed-app window | |
   | Audio output device, its sample rate and buffer; whether the built-in sound is on | |
   | MIDI interface(s): maker, model, how each is connected (USB, hub), and how many ports it lists to the browser | |
   | Each synthesiser or drum machine: maker, model, firmware if known, the cable (USB or DIN), the channel it listens on, and any setting that changes its MIDI behaviour (local control, clock source, arpeggiator, thru) | |
   | What else was running, and whether another program had the same ports open | |
   | Window state for the run: in front, behind another tab, covered by another window | |

2. **Keep the raw files.** The S1 and S3 pages save a JSON file; keep it next to the record. A recording made for B3 or B7 is kept as the audio file plus the offsets read from it.
3. **Save to `handoffs/evidence/`** as `hw-<yyyy-mm-dd>-<run>.md` (and its raw files). Say what was not measured.
4. **Mind the volume.** A hung note is the thing several of these runs provoke on purpose. Start with the synthesiser's volume low and a hand on it. Use a Panic or a power cycle if a note sticks, and write down which was needed.
5. **Report what happened, not what was expected.** A run that disagrees with this spec is the more useful run.

**Gear on hand (from Q-8-1, 2026-10-26).** A UDO Super 6 desktop module and a Moog Mother-32 (README section 2.3). Not yet known: how each connects, the computer and browser, and whether there is a loopback or a two-track recorder. **Write these settings and the firmware version of each unit in every record.** Mother-32: its MIDI channel (Setup page 2, default 1), Follow MIDI Clock and Follow MIDI Start Stop (Setup page 8; both On by default, per Moog's addendum). Super 6: its MIDI channel, whether its clock option is send or receive (it switches to receive by itself when it sees a clock), whether TX/RX P is on, and local control. These change what our Start and our clock do to each unit. Q-8-12 is the owner's choice about them.

## B1. Output timing of a real port (sets criterion A3)

**Question.** How steady is Web MIDI output on a real port?
**Needs.** A loopback: a MIDI interface with a cable from its out to its in, or any device that returns what it is sent. A synthesiser is not needed.
**Do.** Use the real-port half of spike S1 (`spikes/s1/page.html`; `apps/web/README.md`, "Running spike S1 on your machine"). Sixteenth notes at 120 BPM for 30 minutes with the window in front and the built-in sound off. **Check the page's step length first:** the only S1 evidence on file (a page timer) shows a step every 31.2 ms, which predates the tick-length decision of 2026-10-07; a sixteenth at 120 BPM is 125 ms now. If the page still says 31.2 ms, say so in the record.
**Record.** The scatter and the largest error of the arrival intervals, dropouts, the scheduler's late count and its largest lateness, the margin the page reports, and the lookahead used.
**Becomes.** A proposal for A3, written by the owner.

**B1 without a loopback (r0.1).** With one synthesiser and no cable from an interface's out to its in, measure the sound instead. Pick the Super 6's shortest, sharpest sound (a click or a pluck, no release tail), the same sixteenths at 120 BPM for 30 minutes, built-in sound off, and record the synthesiser's audio on the computer or a recorder. Read the onset times with a script. The spread of the intervals is the page's jitter **plus the unit's and the connection's own**, so it is an upper bound for the page and not a measure of it. State that in the record. It is enough to start H0 and to compare a USB path with a DIN one, and it does not replace a loopback run.

## B2. A hidden or covered window, with nothing audible

**Question.** Does a window that is not in front keep time, when the page is silent?
**Do.** B1 again for 30 minutes, three times: (a) another tab in front; (b) another application's window covering the whole browser window; (c) the same as (a) with the display allowed to dim. The built-in sound is **off** in all three, because a silent page may be treated differently from one that makes a sound.
**Record.** Stalls (any gap of more than 500 ms between the pulses the page sees), late events, whether the page's engine-stall sentence appeared, and the time of the first anomaly. The S3 page flags samples taken while hidden, and S1's own report separates visible from hidden (*not re-checked for the real-port page*).
**Becomes.** The documented limit for a performance window (keep it in front; or the wording that is true), and whether Q-8-3 needs a different answer.

## B3. Two devices lined up

**Question.** How far apart do two real instruments sound when sent the same note at the same instant, and does the align tool's arithmetic close the gap?
**Needs.** Two sound sources on two ports (or two channels of one port, split by a route), and an audio recorder that takes both outputs on separate tracks.
**Do.** Put the same short, sharp note on the same step of two tracks, one for each device, built-in sound off. Record 50 hits. Read the onset of each hit on each track and take the difference. Then set the later device's trim to the mean difference (the page's align tool once H3 exists; until then a script that sends the trim) and record again.
**Record.** The mean and the spread of the difference before, and after.
**With the gear on hand.** The Super 6 (USB, if that is how it connects) and the Mother-32 (DIN, behind some interface) are a good pair for this run: two different paths, so the two latencies are unlikely to match. Use a sound with a fast attack on each, and note that the Mother-32's envelope and gate settings change when the onset is heard.
**Becomes.** The number H3's align tool must be able to reach by ear, and HW-9's reference.

## B4. A device lost in the middle of a chord

**Question.** What hangs when a device goes away, and what does the page do?
**Do.** Hold a long chord on a device (a long step length). Then: (a) pull its USB cable; (b) for a DIN device, pull the MIDI cable from the interface. Put it back after 10 seconds.
**Record.** Whether the note kept sounding while it was gone (it will, if the device never heard the Note Off), what the page said, whether a Release went out on return and whether the note stopped, and whether the page threw.
**Becomes.** The behaviour of HW-4, HW-13 and HW-14 on a real device; the sentence the strip shows.

## B5. A page that closes in the middle of a chord

**Do.** Hold a long chord, then (a) close the tab, (b) reload it, (c) end the browser's process. After each, say what the device is doing, then open the page again and press Panic.
**Record.** For each, whether the note stopped by itself, and whether Panic cleared it. Say whether the browser let a send out of a closing page.
**Becomes.** Whether HW-15's `pagehide` Release does anything on real gear, and what the document must say about a crash.

## B6. Clock out to a drum machine, and Live's clock in

**Do.** (a) Master Clock to a drum machine for 30 minutes at 120 BPM with its own pattern running: does it stay on our grid, and does Stop leave it silent. (b) Stop, then Continue from the middle: does the machine resume where we are, with and without Song Position (once H4 exists)? Switch the machine on late, or rewind it by hand, and Continue again. (c) The follow run of `ABLETON.md` section 7, with Live as the sender, 30 minutes at 120 BPM; the S3 report is the record.
**Record.** Which gear followed Song Position and which ignored it, drift over the 30 minutes, and the S3 report's phase figures.
**With the gear on hand.** Either unit's sequencer can stand in for the drum machine in (a) and (b) with its follow switches **on** (Q-8-12 option b): the Mother-32's Follow MIDI Start Stop, or the Super 6 in clock-receive with HOLD arming its sequencer to wait for Start. With them **off** the unit ignores our Start, which is itself a result worth recording. What it does with a Clock and no Start can be settled in (a): Moog's addendum says a connected MIDI clock overrides the internal one (the Tempo knob then divides it), and one user says the sequencer then ran on its own tempo. (a) settles which, and with which Follow settings. The Super 6 in receive mode does not run its arpeggiator or sequencer without a clock (UDO's addendum), which (a) can also confirm.
**Becomes.** Criterion A4, and the evidence for or against HW-19.

## B7. Start alignment

**Question.** How far is the first note from the device's own first step, on real gear? (F-H9.)
**Do.** Slave a drum machine to our Start. Put a kick on step 1 of both. Record 10 starts each at 60, 120 and 180 BPM, built-in sound off. Read the offset between the machine's first hit and ours.
**Record.** The offsets in ticks and in milliseconds, per tempo, and whether the machine plays its first step on Start or on the next clock (devices differ).
**With the gear on hand.** The Mother-32's sequencer (a note on step 1, Follow MIDI Start Stop on) or the Super 6's is the candidate; neither is a drum machine and its first step on Start has not been measured by anyone here. Say so in the record, and treat the result as that unit's, not as drum machines in general.
**Becomes.** The owner's answer to Q-8-4 (R0b), and HW-20 if the answer is to move it.

## B8. The A6 run, then two sessions

**A6.** A pattern made on the panel plays a real synthesiser on port 1 and a second device on port 2 at the same time. (With the gear on hand: the Super 6 as port 1 and the Mother-32 as port 2. Keep the Mother-32's track to single notes, since it is one voice.) The record is the gear table, the pattern (a screenshot of the panel is enough), and a note on how it sounded.

**The "desk" session.** One synthesiser on USB. Twenty minutes of continuous play. Change tempo and a step's pitch while it plays. Press Stop and Play ten times. Press Panic once while a chord is held. Pull the cable once (B4). Record anything that stuck, slipped or surprised.

**The "rig" session.** A drum machine on the clock, two synthesisers (one on port 1, one on port 2, or two routes), and a keyboard played through the page (once H6 exists). Forty-five minutes. Stop and Play every few minutes; one device unplugged and returned; one Panic with the keyboard held; the window covered once for five minutes. Record the same things, and anything that was hard to do with the strip.

**Becomes.** The statement, in the owner's words, of whether the page can be played on stage, and the list of what it still cannot do.

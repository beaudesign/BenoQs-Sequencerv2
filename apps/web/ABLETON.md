# Using the web sequencer with Ableton Live

**Not checked against Live.** No version of Live was available when this was written, and Ableton's help articles were not read in
this session (the fetch of them was refused). The routing steps are from `specs/SPEC-0002/tech.md` section 6, which read them through a
summarising tool earlier, and from what the app itself does, which is tested. Every name of a Live setting below is therefore a
claim to check against your Live: the first step of the procedure is to do that. Where this page says what Live sends or how a setting
moves the clock, it says it is expected and not seen. **The app's clock has been run only against a simulated sender and a
simulated port** (`handoffs/evidence/p4e-s3-simulated.txt`, every line of which says so). Spike S3, below, is the first run with Live.

## 1. What you need

- **Chrome, Edge or Opera.** Safari has no Web MIDI; Firefox has it only behind an add-on. The page must be a secure context
  (`https://`, or `http://localhost`), and the browser asks once for permission to use MIDI. The page does not ask for system
  exclusive.
- **A virtual MIDI port, since Live and the browser are two programs.** macOS: Audio MIDI Setup, Window, Show MIDI Studio, double-click
  the IAC Driver and tick *Device is online*; add a second bus if you want notes and the clock on separate ports. Windows: install
  loopMIDI and create a port (two for the same reason). Live lists these as it lists any MIDI port.
- The app: `just web-app`, then open `http://localhost:8080/pages/app.html`. Press **Start** (the browser keeps audio silent until the
  page has been used); it then asks for MIDI.

## 2. The app sends notes to Live

1. In the app, choose the virtual port for **MIDI Out 1** (and **MIDI Out 2** for tracks on channels 17 to 32).
2. In Live's preferences (expected: the *Link, Tempo & MIDI* tab, the *MIDI Ports* table), turn on **Track** for the port, in its
   *Input* row, so that Live accepts notes from it. Leave **Sync** and **Remote** off on that row: Sync on an input would make Live
   try to follow its clock, and Remote would map its notes to Live's controls.
3. Arm a MIDI track in Live with *MIDI From* set to that port and the channel the app plays on (a track's channel 1 to 16 is port 1;
   17 to 32 is port 2). Press Play in the app: the notes arrive stamped about 30 ms ahead (the lookahead in the strip), and a Stop
   in the app ends every note.

## 3. Live sends its clock to the app

1. In Live's preferences, in the *Output* row of the port you will use for the clock, turn on **Sync**. That makes Live send MIDI
   Clock (and Start, Continue, Stop) to that port. **Do not press Ext** on Live's transport: Ext makes Live a slave, which is
   the other direction (section 6 below).
2. In the app, choose that port as **MIDI In**, and set **MIDI Clock** to **Slave Clock**. The sentence under it says what is
   happening: *waiting for a clock*, then the tempo it reads, and whether it is following.
3. Press Play in Live. Live sends a Start (or a Continue, if it plays from somewhere other than the beginning; the app treats both the
   same, below) and then a pulse 24 times to every quarter note. The app's transport starts on the next pulse, takes Live's tempo (trimmed by up
   to 8% while it corrects the phase), and steers by the pulses' times. Stop in Live stops the app at once.
4. **Do not use *Slave Clock with MIDI Clock echo* with the port Live is sending the clock on.** The echo passes the clock, Start and
   Stop on to the app's outputs as they arrive; if an output goes to the port the clock comes in on, the clock feeds itself and the
   tempo runs away, and the page cannot see it. The echo is for a second device that should follow the same clock.

## 4. Starting and stopping together

| Live sends | The app does |
|---|---|
| Start (0xFA) or Continue (0xFB) | Starts the transport at the **next pulse**. It does not rewind on Stop, so the two are the same to it. A clock alone never starts it. |
| Clock (0xF8) | Estimates tempo and phase; the engine's tempo is the sender's, trimmed up to 8%. |
| Stop (0xFC) | Stops the transport at once and ends sounding notes. |
| Song Position (0xF2) | **Read and ignored.** A Continue from the middle of a song resumes where this sequencer stopped. |
| No pulse for 500 ms | The sentence says the clock stopped; the engine plays on at the last tempo. When pulses return they are followed again. |

The first moments after a Start are not in time: the engine starts about one lookahead (30 ms) late, because the follower can only start
it when the first pulse arrives, and then catches up, in about a second. The first note after a Start also sits 11 ticks after its
pulse at any tempo, which is a fact of the engine (`tests/conformance/AMBIGUITIES.md`, "the first note after Play") and a question for the owner.

## 5. The constant delay

Two settings move the same thing and have opposite signs; do not copy a number from one to the other.

- **The app's *Clock offset in milliseconds*** is extra lead for its notes while it follows a clock: positive makes them land
  *earlier*. Zero is where the lookahead alone puts them. It can be changed while playing and the engine moves to the new place in about
  two seconds.
- **Live's *MIDI Clock Sync Delay*** (expected: in the *MIDI Ports* table, for an output port) moves the clock Live sends against
  Live's own audio, to correct a port's fixed latency. The direction a positive value moves it was not checked against Live here.

If the notes the app plays sit consistently early or late against Live's grid, find the constant first (the S3 run below measures the
phase against the clock; recording the app's notes into a Live clip shows where they land against Live's grid), then set one of the two.

## 6. Live follows the app

The other direction. In the app set **Master Clock** and choose the port as **MIDI Out 1** (the clock goes to every chosen output, once
for each device, so one device chosen for both ports gets one clock). In Live, turn on **Sync** in the *Input* row of that port and press
**Ext** on Live's transport; Live then follows the app's tempo and Start. Press Play in the app. This direction has not been run against
Live. What a port does to the app's clock can be measured without Live (S3's send run, below).

## 7. A test procedure: spike S3

The plan's question is *how well can the app follow Live's clock*. The criterion proposed for it (the owner's to set, and not a gate): the
engine's phase against where the sender's clock puts it stays **within a fifth of a step** (a step is 12 ticks, 1.5 pulses, so 31.25 ms
at 120 BPM and a fifth is 6.25 ms) for **30 minutes at 120 BPM** with the tab in front.

**First: check the names.** Open Live's preferences and find the *MIDI Ports* table, the **Track**, **Sync** and **Remote** switches
on each port, the **Ext** button, and the **MIDI Clock Sync Delay** setting. If any is called something else in your version, or is
somewhere else, trust Live's screen and tell the Forge, so that this page is corrected.

**Setup.** `just web-wasm`; then in `apps/web`: `npm ci && npm run build`; serve the folder (README, "Running spike S1", step 1); open
`/spikes/s3/page.html` in Chrome or Edge. Click **Allow MIDI and list the ports**. In Live: the port as an output with **Sync** on, **Ext**
off, the tempo set to 120, something in the set so that Play plays (it is the transport that sends the clock).

**The follow run.**

1. In *Follow a clock* choose the port as the **Input**, leave 30 minutes and the nominal tempo at 120, and press **Start following**. The status
   line counts pulses; before a clock it says no clock yet, and with a clock and no Start it says to press Play on the sender.
2. Press **Play in Live**. The engine starts on Live's Start. A clock without a Start is recorded and followed but not counted, because the
   engine is not running (the report says so).
3. Leave the tab in front and Live playing for the 30 minutes. The page is silent: it plays the engine to nowhere, and measures only where the
   engine is against Live's pulses.
4. Press **Save the result as a file**; it is a JSON file of the raw records (each pulse's stamp and the time the page saw it, every
   Start, Continue and Stop, the follower's status every 250 ms). Then `node spikes/s3/report.ts <the file>`. Put the file and the report in
   `handoffs/evidence/` with the computer, the operating system, the Live version and edition, Live's audio buffer size, the browser
   version, and what else was running.

**A second run with the tab behind another** for the whole run, to see what the browser does to a tab it considers hidden: the page
flags each sample hidden or visible and the report shows the two apart (the plan's criterion is for the visible tab, and the
verdict line reads the visible samples).

**What the report says.**

- **Clock domain.** Whether `event.timeStamp` is on `performance.now()`'s clock, the one the follower measures against: the stamp
  is a few milliseconds behind the time the page saw the pulse if it is, and a long way off, or in the future, if it is not. The follower
  *assumes* it is; the Web MIDI text does not promise it in every browser. If the report says **other clock**, the phase figures do not
  mean what they say, and that is the finding.
- **Intervals.** The tempo from the pulses, the scatter of the periods, drift in BPM an hour (a line through the tempos of the minutes),
  stalls (a gap over 500 ms), stamps going backwards, equal stamps, and **bursts** (pulses the browser handed over together, which a busy
  or hidden tab does).
- **Phase.** The engine against the sender's grid, positive ahead, counted only while the transport is running and the clock is not lost,
  and not for the first 3 s after a Start or Continue. How many samples are beyond a fifth of a step, and the largest.
- **The criterion**, as a sentence: met, not met, or not read (and why), and that a run under 30 minutes is not the S3 run.

**The send run (no Live needed).** In *Send a clock* choose an output and an input that are the two ends of a loopback port (**not** a
port that goes to a synth or to Live). The engine becomes the clock master and the page hears its clock come back: how many pulses were
sent and heard, the latency and the jitter of the port as the page sees them, whether Start and Stop arrived, and the clock as it came
back read as a followed one is. Pulses are paired by order, so a pulse lost in the middle makes the figures after it untrustworthy, and
the report says so.

## 8. Known gaps

- **D0, the length of a tick, is open** (`WENGE-0012`). A step is 12 ticks, which is 1.5 pulses; whether that is a sixteenth note in time
  with a drum machine or a Live clock a person hears is the owner's decision and nothing here claims it.
- **Song position is ignored**, and Play on the panel stays the page's own command whatever the clock state: Play while slaved starts the
  engine on its own, and the sender's next Start finds it running.
- **The tempo encoder is not built**; turning the engine's tempo by hand while slaved is overwritten within a few milliseconds.
- **The start is off the beat for about a second**, and notes sit 11 ticks after their pulse (section 4).
- **The echo can feed itself** (section 3, item 4).
- **Chromium starts an audio context with a jump** in its frame count that leaves about 20 ms of the engine's timeline empty
  (`handoffs/evidence/p4d-startup-frame-jump.txt`). The app never plays that early, because Play comes after Start; a page that did
  would hear a gap.
- **Nothing here has been run against Live, a real MIDI port or a real hidden tab.** The simulation's stamps are on the page's clock because it
  makes them so; its timers are Chromium's. The thresholds in `spikes/s3/analyse.ts` (what counts as the same clock, a stall, the 3 s of
  settling, a fifth of a step) are the author's proposals, each named where it is used.

# Ableton MCP servers: what they do, and where they fit the web sequencer

**Status: proposal. Hat: Conductor. Documents and one evidence script only.** No code, contract, threshold, golden hash or baseline file is touched, and
nothing here changes SPEC-0002 r1. Every "recommend" is a proposal for the owner. There is no task number: this is research, and the handoff
README ties numbers to opportunities in `specs/SPEC-0001/`. The owner can assign one.

Companion: `specs/SPEC-0002/sequencing-theory.md` (the music theory half of the same request). Evidence: `handoffs/evidence/theory-check.py` and `.txt`.

## 0. The request, and how I read it

The owner, Fri 2026-10-02 16:19 Paris, with three links (abletonmcp.com, muse.art, the Ableton Knowledge connector on claude.com):

> review all of the available ableton mcps so they're better designed to fit into the application and also learn deep music theory for sequencing as well

**Reading taken:** (1) survey the Ableton MCP servers, the three given and the others I could find; (2) say how what they do relates to this application's design and
where, if anywhere, they should connect to it; (3) write down the theory a step sequencer needs, tied to the Octopus's own features and stated so that it can be
checked. **Not done:** changing or forking any MCP server, writing application code, or adding scope. If "so they're better designed to fit" meant making the MCP
servers themselves fit, say so and this is revised.

## 1. How I know what I know, and what I did not do

- **Read:** the three given pages, ten GitHub projects, Ableton's Groove Pool and MIDI effect pages, Muse's Max for Live guide and pricing, and four articles. Each page was
  read through a tool that summarises with a small model, so **a figure here is "as summarised", not counted.** Tool counts are the projects' own claims.
- **Not done:** I installed and ran none of them (no Live, no MCP client on a Live set here), and I did not test any claim of theirs. "Tested" below means "the page says
  it has tests".
- **Contradictions in the sources, left visible:** AbletonMCP's site and its repo description say 156 tools; the README as summarised says 154. The Ableton connector's page says
  it is "made by Ableton" and also "not an official Ableton product" (I could not resolve that from the page). A comparison page by VIXSOUND, a vendor of one of the
  products compared, says AbletonMCP has "no music features", while that project's repo lists generators. I do not rely on the vendor page. One article calls
  `Simon-Kansara/ableton-live-mcp-server` obsolete; its repo is still up.
- **No answer found:** Producer Pal's `DEVELOPERS.md` gave no design statements about its tools, so nothing below is claimed about how it designs them.

## 2. The landscape

| Project | Reaches Live by | Size claimed | Licence | What stands out | Evidence |
|---|---|---|---|---|---|
| **AbletonMCP** (wstierhout; abletonmcp.com) | Remote Script, TCP `127.0.0.1:9877`, no authentication; offline reading of saved `.als/.adg/.adv` | 154 or 156 tools | MIT | Typed, validated arguments and "no arbitrary-code path"; read-only and destructive hints; note probability; quantise with strength; Groove Pool; generators (drum patterns in 7 styles, Euclidean, chord progressions, voice-led jazz voicings, walking basslines, motif transforms, humanize); Live 11 and 12; CI on four Python versions. Limit stated: no render or freeze in Live's API | README and site; tests claimed, not run |
| **ahujasid/ableton-mcp** | Remote Script, TCP and JSON | not stated | MIT | 3.1k stars, Live 10+. Creates and edits MIDI clips; the page shows no quantise, velocity, probability or scale. "Complex arrangements may need breaking into smaller steps". The base many others start from | README |
| **xiaolaa2/ableton-copilot-mcp** | `ableton-js` plus a Remote Script | not stated | MIT | Add, delete and replace notes **with rollback, for note operations only**; warns that direct clip edits "cannot be undone with Ctrl + Z"; tested on Live 12.1.10 only | README |
| **uisato/ableton-mcp-extended** | Remote Script socket plus a UDP server for real-time control | not stated | MIT | Adds ElevenLabs speech; automation point placement "isn't working perfectly yet" | README |
| **ntworm/ableton-mcp-server** | Remote Script TCP `:9888`, WebSocket host `:9889` | 97 tools | MIT | Structured error codes (`CAPABILITY_UNAVAILABLE`, `AMBIGUOUS_MATCH`, `VERIFICATION_FAILED`); write-then-verify; SHA-256 check of the installed script; `docs/KNOWN_BUGS.md` (path-id drift, undo semantics); five "groove intelligence" tools; offline mix analysis | README |
| **Simon-Kansara/ableton-live-mcp-server** | AbletonOSC, UDP 11000 and 11001 | not stated | MIT | The first OSC route; no tool names documented | README |
| **TropinAlexey/ableton-and-max-mcp** | AbletonOSC plus Max for Live | 40 tools | MIT | 60+ scales, 30+ chord types, arpeggio, chord and random patterns. Limits stated: no time-signature control, polling only (no state streaming) | README |
| **jpoindexter/ableton-mcp** | MCP, REST and a Max for Live device | "200+" | MIT | Broad claim; the page shows no tests or benchmarks | README |
| **wolfiesch/Ableton-MCP** | Remote Script, TCP `:9877` | about 30 | MIT | Reads existing notes (its stated differentiator); groove templates, humanise, swing; automation for Session View clips only | README |
| **Producer Pal** (adamjmurray) | Max for Live device (`Producer_Pal.amxd`), MCP and REST | not stated | **GPL-3.0** | Author's stated aims: accessibility including voice, the person stays "the author", any AI provider, open source | README |
| **Ableton Knowledge** (claude.com connector) | A Claude connector, not a Live control | 9 tools | n/a | `search_live_manual`, `search_push_manual`, `search_move_manual`, `search_note_manual`, `search_release_notes`, `search_knowledge_base`, `search_videos`, `search_transcripts`, `get_ableton_knowledge_info`; covers Cloud, Link and Max for Live docs. Labelled "Experimental", results "may be incomplete, outdated, or incorrect" | the page. **Not connected in this session**, and a registry search for it returned nothing |
| **Muse** (muse.art) | Not an MCP: a web editor and a Max for Live device | n/a | commercial: Pro $15 a month, 200 credits, packs of 50, 110 and 250 credits at $5, $10 and $20 | Generates melody, chord and bass MIDI (2 to 16 bars). Key and Scale are set by hand and are **not** read from Live; Humanize covers onset, velocity and chord styles; places the clip at the earliest free position; exports `.mid` and `.wav` | its pages |

**Named in sources and not read:** FabianTinkl/AbletonMCP, whybothercoding/ableton-mcp-server, Milesy1/MCP-Ableton-API, nozomi-koborinai/ableton-osc-mcp,
bschoepke/ableton-live-mcp (voice), itsuzef/ableton-mcp, `mcp-server-midi` (a virtual MIDI output), VIXSOUND (commercial), MIDI Agent. The field moves monthly, and
one of the sources says so.

## 3. What they do, and what none of them does: two planes

Every project above reaches Live by one of three **control** channels: a Remote Script socket, AbletonOSC, or a Max for Live node. Through it they read and change
Live's **model**: tracks, clips, notes, devices, tempo, scenes. That is the *control plane*.

This application and Live meet on the **MIDI wire**: clock, Start, Continue, Stop, and notes (`apps/web/ABLETON.md`). That is the *data plane*, and
**no tool list I read contains a MIDI-clock tool or a MIDI-port tool.** The settings `ABLETON.md` depends on (Track, Sync and Remote on a port, the Ext button, MIDI
Clock Sync Delay) are Live preferences, and none of the lists names them. **Unverified:** I believe the Live API does not expose preferences; I have not checked, and
the Ableton Knowledge connector is the way to check (F1 below).

So an MCP cannot replace spike S3 or stand in for the wire. It can only drive Live around it.

## 4. Where they could fit the application

| # | Option | Needs | Proposal | How we would know |
|---|---|---|---|---|
| F1 | **Check `ABLETON.md` against Ableton's own manual and release notes** with the Ableton Knowledge connector | The connector added to this chat (the owner's to do; the registry search found no match here) | **Yes, first.** It is the one thing that retires an unchecked claim | Every "expected" in `ABLETON.md` sections 2, 3, 5 and 6 gets a manual page or release note, or a correction. The claims: the names *Link, Tempo & MIDI*, *MIDI Ports*, **Track**, **Sync**, **Remote**, **Ext**, *MIDI Clock Sync Delay* and its sign; whether Live sends Start or Continue and when; whether it sends a clock while stopped; whether it sends Song Position |
| F2 | **Script the owner's S3 session**: a Live-side MCP sets tempo, Play and Stop, so a run can include tempo steps and ramps, the follower's weakest case (3 to 4 ticks of lag on a ramp, `p4d-follower-measured.txt`) | One more install on the owner's Mac; it cannot set port preferences | **Later.** After the owner's first manual S3 | S3's tempo error against nominal is already in the report; a stepped tempo gives a second, harder figure |
| F3 | **Export a pattern as a Standard MIDI File** the owner drags into Live (as Muse does with `.mid`) | `octorun` already writes SMF; the tick-to-PPQ mapping needs **D0** | **Candidate for a roadmap wave, owner decides.** Zero coupling to any MCP | Round trip: export, parse, the same note starts in ticks; the owner confirms Live imports it on the grid |
| F4 | **Give the application its own typed agent surface** | A new scope item and an ADR; SPEC-0002 r1 does not have it | **Not now.** Keep the harness API typed (section 5) so the option stays cheap | None yet |
| F5 | **Let the page talk to a Live MCP or Remote Script itself** | A local server beside a web page; AbletonMCP's Remote Script (the one that says) has no authentication and is bound to loopback, so any local process can drive Live | **No.** It duplicates what Web MIDI does for the wire, and breaks "open a page and play" | n/a |
| F6 | **Ableton Link** (one server lists Link and tempo tools) | A page cannot join Link as far as I know (**unverified**) | **No.** | n/a |

## 5. Design lessons, against what this repo already has

| Lesson | Seen in | Already here | Action |
|---|---|---|---|
| **Typed operations, no arbitrary-code path** | AbletonMCP | The engine takes typed command words; a panel press the engine cannot carry out is counted (`dropped_intents`), not silent; a random-input probe of the host page (4 runs of 20 s, about 11,500 inputs) found 0 errors and no note held after Stop | Keep. The probe is not committed; **bounded version as a Chromium test is a candidate** |
| **Mark destructive operations, and offer undo** | AbletonMCP; copilot's "cannot be undone with Ctrl + Z" | No persistence yet (the `persistence` gate is `not_implemented`); the command ring and snapshot exist | When Save and Load arrive, clear and overwrite are marked and undo is a command-ring feature. The copilot warning is the failure to avoid |
| **Write, then read back and verify** | ntworm | Chromium tests read the panel back; the S3 report recomputes every figure from raw records | Any future "send to Live" reads back (F3: parse the file written) |
| **Know which build you are talking to** | ntworm's checksum of the installed script | The simulated S3 runner records the commit; **a file the owner saves from the page does not record the build** | **Small and in my zone:** record the engine build (the wasm's hash, the ABI number) in the saved S3 file. Test: the file names it; a changed wasm changes it |
| **Write known traps down** | ntworm's `KNOWN_BUGS.md` | `AMBIGUITIES.md`, `QUESTIONS.md`, `ABLETON.md` section 8 | Keep. The Chromium start-up frame jump and the clock-map jumps are the browser traps so far |
| **A small tool surface** | 154 and "200+" claimed against 40 and 97 | n/a | Opinion, not a finding: if F4 is ever built, keep it small and typed. The big counts are unverified and one project shows no tests |
| **Generators are common, and the Octopus has native counterparts** | Euclidean, drum styles, chord progressions, voice-led voicings, walking bass, motif transforms, humanize appear in two or more projects | RND and RMX mutators, step chords, strum and polyphony, five directions plus 11 programmable, phrases, the effector, page scales | For a reproduction, **keep the Octopus's own semantics** and treat Euclidean, swing and grooves as idioms *programmed on it* (`sequencing-theory.md` sections 1 and 4). New generators are a scope decision (Q-M5) |
| **Deterministic groove against random humanise** | Muse's Humanize and Live's Random against Roger Linn's stated view | STA is a deterministic push or pull per step, scaled by the track; the engine has a seed | Deterministic by default; any randomness seeded so a test can assert it. This is the constitution's rule restated |

## 6. Decisions for the owner

- **Q-M1** Is the reading in section 0 right, or did you mean changing the MCP servers? Default: as read.
- **Q-M2** Add the Ableton Knowledge connector to this chat so F1 can run. Default: not done; I cannot add connectors.
- **Q-M3** F3, SMF export, as a roadmap candidate after D0. Default: not started.
- **Q-M4** F4, an agent surface for the application. Default: no.
- **Q-M5** New generators (Euclidean, groove extraction). Default: no; the theory note shows how to program them with the Octopus's own controls.
- **Q-M6** Record the engine build in the S3 file I own. Default: yes, in the next change to `apps/web/spikes/s3/`.
- **Q-M7** Force-to-scale's tie rule (ties go down in the engine, "matching v1", and the manual does not say) needs an Octopus: see `sequencing-theory.md` section 2.3 for what turns on it.

## 7. Limits of this review

Nothing was run or installed. Licences matter: **Producer Pal is GPL-3.0**, and this repository has no licence file at its root, so no code should be copied from it without a decision;
the lessons above are ideas, not code. Prices and tool counts are as of 2026-10-02 and move. The Ableton Knowledge connector's own caution applies to anything learned from it:
verify against Ableton's documentation.

## 8. Sources (read 2026-10-02)

- [abletonmcp.com](https://abletonmcp.com), [wstierhout/ableton-live-mcp](https://github.com/wstierhout/ableton-live-mcp)
- [Ableton Knowledge connector](https://claude.com/marketplace/connectors/ableton-knowledge)
- [Muse](https://www.muse.art/home), [Muse Max for Live guide](https://www.muse.art/blog/muse-m4l-user-guide), [Muse pricing](https://www.muse.art/pricing)
- [ahujasid/ableton-mcp](https://github.com/ahujasid/ableton-mcp), [xiaolaa2/ableton-copilot-mcp](https://github.com/xiaolaa2/ableton-copilot-mcp), [uisato/ableton-mcp-extended](https://github.com/uisato/ableton-mcp-extended), [ntworm/ableton-mcp-server](https://github.com/ntworm/ableton-mcp-server), [Simon-Kansara/ableton-live-mcp-server](https://github.com/Simon-Kansara/ableton-live-mcp-server), [TropinAlexey/ableton-and-max-mcp](https://github.com/TropinAlexey/ableton-and-max-mcp), [jpoindexter/ableton-mcp](https://github.com/jpoindexter/ableton-mcp), [wolfiesch/Ableton-MCP](https://github.com/wolfiesch/Ableton-MCP), [Producer Pal](https://github.com/adamjmurray/producer-pal)
- [Options for Controlling Ableton Live with MCP](https://www.mslinn.com/av_studio/ableton-mcp-options.html), [AI Agents Enter the Studio](https://sonicfield.org/ai-agents-enter-the-studio-the-mcp-turn-in-audio), [VIXSOUND comparison](https://vixsound.com/compare/ai-ableton-control) (a vendor page; not relied on)
- [Ableton manual: Using Grooves](https://www.ableton.com/en/manual/using-grooves/), [Live MIDI Effect Reference](https://www.ableton.com/en/manual/live-midi-effect-reference/)

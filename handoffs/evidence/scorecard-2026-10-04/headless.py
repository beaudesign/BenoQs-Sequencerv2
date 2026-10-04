#!/usr/bin/env python3
"""Probes on the headless runner (octorun). Not a gated test: a record of what was measured on 2026-10-04.

Run from the repo root:  cargo build -p octorun && OCTORUN=target/debug/octorun python3 handoffs/evidence/scorecard-2026-10-04/headless.py
Needs only python3. Writes its patterns to a temporary directory.

A. tempo changes the speed of the MIDI: one pattern at 60, 120 and 240 BPM.
B. Stop then Play: does the sequence carry on from the next note, or lose one?
C. The same, swept over where in a step the Stop lands, for dense and sparse patterns at three tempos.
"""
import json, os, statistics, subprocess, tempfile

BIN = os.environ.get("OCTORUN", "target/debug/octorun")
TMP = tempfile.mkdtemp(prefix="wenge-probe-")


def pattern(active, bpm, body):
    h = f"seed 1\nbpm {bpm}\ntrack 0 mch 1\n"
    for i in active:
        h += f"track 0 step {i} active 1\n" + (f"track 0 step {i} pit +{i}\n" if i else "")
    return h + body


def run(text):
    path = os.path.join(TMP, "p.pattern")
    open(path, "w").write(text)
    out = subprocess.run([BIN, path, "--stdout"], capture_output=True, text=True, check=True).stdout
    return [json.loads(l) for l in out.splitlines()[1:] if l.startswith('{"t"')]


def ons(ev):
    return [(e["t"], e["note"] - 69) for e in ev if e["kind"] == "on"]


ALL = list(range(16))

print("== A. tempo changes the speed of the MIDI (16 steps, 3 s rendered)")
med = {}
for bpm in (60, 120, 240):
    o = ons(run(pattern(ALL, bpm, "play\nrender 3 s buffer=64\n")))
    gaps = [b[0] - a[0] for a, b in zip(o, o[1:])]
    med[bpm] = statistics.median(gaps)
    print(f"bpm {bpm:3d}: {len(o):3d} note-ons; step gap {med[bpm]:7.1f} samples = {med[bpm] / 48:6.2f} ms; range {min(gaps)}..{max(gaps)}")
print(f"gap ratio 60/120 = {med[60] / med[120]}, 120/240 = {med[120] / med[240]} (a clean 2.0 each)")
print(f"a 16th note at 120 BPM lasts 125 ms; the engine's 12-tick step at 120 BPM lasts {med[120] / 48} ms (D0, WENGE-0012)")

print()
print("== B. Stop at 0.9 s, silent 0.7 s, Play, 2.1 s more (16 steps, 120 BPM)")
ev = run(pattern(ALL, 120, "play\nrender 0.9 s buffer=64\nstop\nrender 0.7 s buffer=64\nplay\nrender 2.1 s buffer=64\n"))
seq = [n for t, n in ons(ev)]
print("note sequence:", seq[:40])
mid = [t for t, n in ons(ev) if 43200 <= t < 76800]
print("note-ons while stopped:", len(mid))
d = [(b - a) % 16 for a, b in zip(seq, seq[1:])]
print("steps skipped across the Stop:", sum(1 for x in d if x == 2), "; repeated:", sum(1 for x in d if x == 0))

print()
print("== C. how often a Stop loses a note, over where in the gap between two notes the Stop lands")
for name, active in (("every step (16 notes a bar)", ALL), ("every 4th step (4 notes a bar)", [0, 4, 8, 12])):
    for bpm in (60, 120, 240):
        step_samples = 1500 * 120 / bpm
        span = int(step_samples * 16 / len(active))
        offsets = list(range(0, span, max(1, span // 40)))
        bad = 0
        for off in offsets:
            text = pattern(active, bpm, f"play\nrender {1.0 + off / 48000:.6f} s buffer=128\nstop\nrender 0.7 s buffer=128\nplay\nrender 6 s buffer=128\n")
            cyc = sorted(active)
            idx = [cyc.index(n) for t, n in ons(run(text))]
            bad += any((b - a) % len(cyc) != 1 for a, b in zip(idx, idx[1:]))
        print(f"{name:32s} bpm {bpm:3d}: {bad:2d} of {len(offsets)} stop positions lost a note ({100 * bad / len(offsets):.0f}%)")
print("(the engine steps 12 ticks ahead of the audio, MAX_EARLY_TICKS; Stop drops what was scheduled in that window and Play resumes after it)")

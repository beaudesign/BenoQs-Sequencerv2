#!/usr/bin/env python3
"""How many tick times of each golden pattern land on a different whole sample if the clock
computes `floor(k * spt + 1e-9)` instead of adding `spt` k times (what the engine does today)?
Nothing here is O4 code: it replays the two formulas in f64, which is what the engine uses.
    python3 golden_floor_flips.py > ../o4-golden-floor.txt
"""
import math

SR = 48_000.0  # the octorun default
# (pattern, bpm) for the constant-tempo stretches of each golden pattern, 7 s of ticks each
CASES = [("hello", 120), ("chords_and_strums", 96), ("effector", 128),
         ("phrases", 110), ("mcc_and_transport (first 2 s)", 100), ("mcc_and_transport (after the change)", 140)]

for name, bpm in CASES:
    spt = SR * 60.0 / (float(bpm) * 192.0)
    n = int(7 * SR / spt)
    t, flips, first = 0.0, 0, None
    for k in range(n):
        old, new = math.floor(t), math.floor(k * spt + 1e-9)
        if old != new:
            flips += 1
            first = first or (k, old, new)
        t += spt
    print(f"{name:40s} {bpm:>3} BPM  spt {spt!r:>22}  ticks {n:>5}  floor differs on {flips:>3}  first (tick, sum, product) {first}")

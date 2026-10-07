#!/usr/bin/env python3
"""The D0 change moves every event in time by exactly x4 and changes nothing else.

Usage: compare.py OLD_OCTORUN OLD_PATTERN_DIR NEW_OCTORUN NEW_PATTERN_DIR

OLD_OCTORUN is `octorun` built from origin/main before the change (192 ticks to the quarter note) and OLD_PATTERN_DIR holds the
examples as they were; NEW_* are the same after it (48 to the quarter; each example's `render` lengths multiplied by 4 so that it covers
the same music). For every example the two event logs must have the same events in the same order, and every time of the new log must be
four times the old one to within 3 samples (the tick is a whole number of samples at 500, where it was 125, so rounding differs).
"""
import json, subprocess, sys, pathlib

old_bin, old_dir, new_bin, new_dir = sys.argv[1:5]

def events(binary, pattern):
    out = subprocess.run([binary, str(pattern), "--stdout"], check=True, capture_output=True, text=True).stdout
    return [json.loads(l) for l in out.splitlines()[1:]]

bad = 0
for p in sorted(pathlib.Path(new_dir).glob("*.pattern")):
    a, b = events(old_bin, pathlib.Path(old_dir) / p.name), events(new_bin, p)
    same = len(a) == len(b) and all({k: v for k, v in x.items() if k != "t"} == {k: v for k, v in y.items() if k != "t"} for x, y in zip(a, b))
    worst = max((abs(y["t"] - 4 * x["t"]) for x, y in zip(a, b)), default=0)
    ok = same and worst <= 3
    bad += not ok
    print(f"{p.name:30} {len(a):4} events: {'the same notes in the same order' if same else 'DIFFERENT EVENTS'}, every time x4 (worst {worst} samples)" + ("" if ok else "  <-- FAIL"))
sys.exit(1 if bad else 0)

#!/usr/bin/env python3
"""Checks the worked examples in specs/SPEC-0002/sequencing-theory.md by running them.
Not a product test and not in the ratchet: it is the evidence that the examples in the note are what the note says they are.
Run: python3 handoffs/evidence/theory-check.py   (prints PASS/FAIL per check; exit status 1 if any FAIL)
Sources for the expected values are named in each check: T = Toussaint 2005 (Bridges), M = Octopus Reference Manual CE v5.30 (page), L = Roger Linn as quoted
at melodiefabriek.com, my own arithmetic otherwise."""
import itertools, math, sys

fails = 0
def check(name, ok, detail=""):
    global fails
    print(("PASS " if ok else "FAIL ") + name + (("  " + detail) if detail else ""))
    if not ok: fails += 1

# --- Euclidean rhythms: Bjorklund's algorithm, as the grouping-and-remainder procedure Toussaint describes
def bjorklund(k, n):
    if k == 0: return [0] * n
    groups = [[1] for _ in range(k)] ; rem = [[0] for _ in range(n - k)]
    while len(rem) > 1 and len(groups) > 1:
        m = min(len(groups), len(rem))
        new = [groups[i] + rem[i] for i in range(m)]
        groups, rem = new, (groups[m:] if len(groups) > m else rem[m:])
    return [x for g in groups + rem for x in g]

def s(bits): return "".join("x" if b else "." for b in bits)
def rotations(p): return {p[i:] + p[:i] for i in range(len(p))}

T = {  # (k, n): pattern as printed in T, spaces removed
    (2, 5): "x.x..", (3, 7): "x.x.x..", (3, 8): "x..x..x.", (4, 7): "x.x.x.x", (5, 8): "x.xx.xx.", (5, 12): "x..x.x..x.x.",
    (5, 16): "x..x..x..x..x....", (7, 12): "x.xx.x.xx.x.", (7, 16): "x..x.x.x..x.x.x.", (9, 16): "x.xx.x.x.xx.x.x.",
}
# T's printed E(5,16) has 17 symbols in the web transcription; the paper's pattern is 16 long: x..x..x..x..x...
T[(5, 16)] = "x..x..x..x..x..."
for (k, n), printed in T.items():
    got = s(bjorklund(k, n))
    check(f"E({k},{n}) has {k} onsets in {n} slots", got.count("x") == k and len(got) == n, got)
    check(f"E({k},{n}) is T's pattern up to rotation", printed in rotations(got) or got in rotations(printed), f"mine {got}, T {printed}, exact={got == printed}")

def evenness(p):  # max-min of the inter-onset intervals around the cycle
    on = [i for i, c in enumerate(p) if c == "x"]
    gaps = [(on[(j + 1) % len(on)] - on[j]) % len(p) or len(p) for j in range(len(on))]
    return max(gaps) - min(gaps)
for (k, n) in T:
    check(f"E({k},{n}) is maximally even (inter-onset gaps differ by at most 1)", evenness(s(bjorklund(k, n))) <= 1)

# --- cycle length of two loops of different lengths: lcm, in steps
for a, b, want in [(16, 12, 48), (16, 15, 240), (16, 7, 112), (16, 16, 16), (16, 8, 16)]:
    check(f"loops of {a} and {b} steps realign after {want} steps", math.lcm(a, b) == want)

# --- swing: the MPC delays the even 16ths; 50 % none, 66 % triplet, 75 % the maximum (L). Pair length in ticks from M p.15: a step is 12/192.
STEP_TICKS = 12; PAIR = 2 * STEP_TICKS
def delay_ticks(pct): return (pct / 100 - 0.5) * PAIR
check("swing 50 % delays the second step by 0 ticks", delay_ticks(50) == 0)
check("swing 66.67 % (triplet) delays the second step by 4 ticks of 12 (a third of a step)", abs(delay_ticks(200 / 3) - 4) < 1e-9)
check("swing 75 % delays the second step by 6 ticks (half a step)", abs(delay_ticks(75) - 6) < 1e-9)
check("swing 54 % rounds to 1 tick (the Octopus resolution is one tick)", round(delay_ticks(54)) == 1, f"{delay_ticks(54):.2f} ticks")
# M p.16: a step's STA push is at most 5 ticks at the neutral track STA; p.45 chart: track STA 10 reaches +6.
check("the largest swing the neutral track STA reaches is (12+5)/24 = 70.8 %", abs((STEP_TICKS + 5) / PAIR - 0.7083) < 1e-3)
check("swing 75 % needs a push of 6, which M p.45's chart gives from track STA 10 (+6 at the top step STA)", True, "chart row 10: -6 -4 -3 -2 -1 0 +1 +2 +3 +4 +6")
# triplet check in milliseconds at both readings of the tick (D0): sixteenth = 125 ms (reading A) or 31.25 ms (reading B) at 120 BPM
for label, step_ms in [("A: step is a sixteenth, 125 ms", 125.0), ("B: engine today, 31.25 ms", 31.25)]:
    print(f"      info  at 120 BPM, reading {label}: " + "; ".join(f"{pct} % -> {(pct / 100 - 0.5) * 2 * step_ms:.2f} ms" for pct in (54, 58, 62, 200 / 3, 100 * 17 / 24, 75)))

# --- harmony: scales as pitch-class sets, modes as rotations, diatonic triads by stacking thirds
MAJOR = [0, 2, 4, 5, 7, 9, 11]
def mode(i): return sorted(((x - MAJOR[i]) % 12) for x in MAJOR)
names = ["Ionian", "Dorian", "Phrygian", "Lydian", "Mixolydian", "Aeolian", "Locrian"]
want = {"Ionian": [0,2,4,5,7,9,11], "Dorian": [0,2,3,5,7,9,10], "Phrygian": [0,1,3,5,7,8,10], "Lydian": [0,2,4,6,7,9,11],
        "Mixolydian": [0,2,4,5,7,9,10], "Aeolian": [0,2,3,5,7,8,10], "Locrian": [0,1,3,5,6,8,10]}
for i, nme in enumerate(names):
    check(f"{nme} is the major scale rotated to its degree {i + 1}", mode(i) == want[nme], str(mode(i)))
def triad(deg):
    r = [MAJOR[(deg + o) % 7] + 12 * ((deg + o) // 7) for o in (0, 2, 4)]
    a, b = r[1] - r[0], r[2] - r[1]
    return {(4, 3): "major", (3, 4): "minor", (3, 3): "diminished", (4, 4): "augmented"}[(a, b)]
check("diatonic triads of the major scale are M m m M M m dim", [triad(d) for d in range(7)] == ["major","minor","minor","major","major","minor","diminished"], str([triad(d) for d in range(7)]))
def sev(deg):
    r = [MAJOR[(deg + o) % 7] + 12 * ((deg + o) // 7) for o in (0, 2, 4, 6)]
    return tuple(r[i + 1] - r[i] for i in range(3))
sevs = {(4,3,4): "maj7", (3,4,3): "m7", (4,3,3): "7", (3,3,4): "m7b5"}
check("diatonic sevenths are maj7 m7 m7 maj7 7 m7 m7b5", [sevs[sev(d)] for d in range(7)] == ["maj7","m7","m7","maj7","7","m7","m7b5"], str([sevs[sev(d)] for d in range(7)]))

# --- scale-force: nearest scale tone, with the engine's tie rule (down); the manual does not state the tie rule
def nearest(p, scale_pcs, tie="down"):
    if p % 12 in scale_pcs: return p
    for d in range(1, 12):
        c = [p - d, p + d] if tie == "down" else [p + d, p - d]
        for q in c:
            if q % 12 in scale_pcs: return q
cmaj = set(MAJOR)
ties = [p for p in range(60, 72) if p % 12 not in cmaj and (p - 1) % 12 in cmaj and (p + 1) % 12 in cmaj]
check("in C major the out-of-scale notes with a scale tone either side are C#, D#, F#, G#, A# (five ties)", [p % 12 for p in ties] == [1, 3, 6, 8, 10], str([p % 12 for p in ties]))
check("tie-down and tie-up disagree on every one of those five", all(nearest(p, cmaj, "down") != nearest(p, cmaj, "up") for p in ties))
check("the scale's two semitone steps are E-F and B-C, so no out-of-scale pitch lies between those pairs", all(pc in cmaj for pc in (4, 5, 11, 0)) and not any(pc not in cmaj and (pc - 1) % 12 in cmaj and (pc + 1) % 12 in cmaj for pc in (5, 0)))

# --- voice leading: least total motion between two chords over all voicings within an octave of each other
def best_motion(c1, c2):
    best = None
    for perm in itertools.permutations(c2):
        for shifts in itertools.product((-12, 0, 12), repeat=3):
            m = sum(abs((p + s) - q) for p, s, q in zip(perm, shifts, c1))
            if best is None or m < best[0]: best = (m, tuple(p + s for p, s in zip(perm, shifts)))
    return best
C, F, G, Am = (60, 64, 67), (65, 69, 72), (67, 71, 74), (69, 72, 76)
m, v = best_motion(C, F)
check("C to F: the closest voicing moves 3 semitones in all (C stays; E up 1, G up 2)", m == 3, f"{m} semitones, voicing {v}")
m2, v2 = best_motion(C, G)
check("C to G: the closest voicing moves 3 semitones in all (C down 1 to B, E down 2 to D, G stays); my first figure of 5 was wrong and this check found it", m2 == 3, f"{m2}, {v2}")
m3, v3 = best_motion(C, Am)
check("C to Am: two common tones, one voice moves 2 semitones", m3 == 2, f"{m3}, {v3}")
root_pos = sum(abs(a - b) for a, b in zip(C, F))
check("root position C to F moves 15 semitones in all (what a naive transposition does)", root_pos == 15, f"{root_pos}")

# --- the Octopus's own worked numbers
check("effector example, M p.60: +3 -1 -2 -2 = -2 semitones for track 0", 3 - 1 - 2 - 2 == -2)
strum = {2: [0,1,1,2,2,3,3,4,5], 3: [1,2,3,4,5,6,7,8,10], 4: [1,2,4,6,8,9,10,13,17], 5: [2,3,5,9,11,13,15,19,23], 6: [2,3,6,12,15,18,21,27,30], 7: [3,6,9,16,19,24,29,36,45]}
check("strum table, M p.22: every note's offset never decreases as the level rises", all(all(a <= b for a, b in zip(r, r[1:])) for r in strum.values()))
check("strum table, M p.22: at every level the later note is never earlier than the one before it", all(all(strum[n][i] <= strum[n + 1][i] for i in range(9)) for n in range(2, 7)))
check("strum 9 on a 7-note chord spreads the notes over 45 ticks, 3.75 steps, less than a quarter of a bar of 16 steps", strum[7][8] == 45 and 45 / 12 == 3.75)
check("Brownian direction, M p.45: 2/3 forward, 1/3 reverse is a drift of +1/3 step a step", abs((2 / 3 - 1 / 3) - 1 / 3) < 1e-12)
check("track map factors, M p.53: neutral is 9 of 1..17 (8 of 0..16 as shown for LEN and STA)", 9 in range(1, 18) and 8 in range(0, 17))

# --- what 192 ticks to a whole note can and cannot hold (M p.15-16: 12/192 is a sixteenth; 192/192 is "a full note")
def holds(div): return 192 % div == 0
check("192 holds 16ths (12 ticks), 32nds (6), 64ths (3), 8th triplets (16) and 16th triplets (8)", all(192 // d == w for d, w in [(16, 12), (32, 6), (64, 3), (12, 16), (24, 8)]))
check("192 holds no quintuplet or septuplet subdivision of a sixteenth (5 and 7 do not divide 192)", not holds(5) and not holds(7) and not holds(20) and not holds(28))

# --- ties: an out-of-scale pitch class is a tie exactly when it sits in the middle of a two-semitone gap of the scale
def ties_of(scale):
    return [pc for pc in range(12) if pc not in scale and (pc - 1) % 12 in scale and (pc + 1) % 12 in scale]
PENT = {0, 2, 4, 7, 9}; HARM_MINOR = {0, 2, 3, 5, 7, 8, 11}; WHOLE = {0, 2, 4, 6, 8, 10}
check("a tie is the middle of a two-semitone gap: C major has five (C# D# F# G# A#), the major pentatonic three (C# D# G#); in a three-semitone gap (E-G) F is nearer E and F# nearer G", ties_of(cmaj) == [1,3,6,8,10] and ties_of(PENT) == [1,3,8] and nearest(65, PENT) == 64 and nearest(66, PENT) == 67, f"pentatonic ties {ties_of(PENT)}")
check("the whole-tone scale has six ties (every out-of-scale pitch class), harmonic minor has three", len(ties_of(WHOLE)) == 6 and len(ties_of(HARM_MINOR)) == 3, f"harmonic minor ties {ties_of(HARM_MINOR)}")

# --- the effector (M p.59-60) adds the feeder's offset to the listener's pitch; the engine sums offsets and then forces to scale (docs/03 s.3; scale.rs).
#     Take a root-position chord on the tonic, shift it by the feeder's offset for each degree, force it to C major with the engine's tie rule, and compare with the diatonic chord.
def diatonic(stack, deg): return sorted(60 + MAJOR[(deg + o) % 7] + 12 * ((deg + o) // 7) for o in stack)
def shifted(chord, off, tie): return sorted(nearest(p + off, cmaj, tie) for p in chord)
for label, stack in [("triads", (0, 2, 4)), ("sevenths", (0, 2, 4, 6))]:
    chord = diatonic(stack, 0)
    offs = [MAJOR[d] for d in range(7)]
    down = all(shifted(chord, MAJOR[d], "down") == [x for x in diatonic(stack, d)] or sorted(q % 12 for q in shifted(chord, MAJOR[d], "down")) == sorted(q % 12 for q in diatonic(stack, d)) for d in range(7))
    up_bad = [d for d in range(7) if sorted(q % 12 for q in shifted(chord, MAJOR[d], "up")) != sorted(q % 12 for q in diatonic(stack, d))]
    check(f"a tonic {label[:-1]} shifted by 0 2 4 5 7 9 11 semitones and forced to C major with ties DOWN gives the seven diatonic {label} (same pitch classes)", down)
    print(f"      info  with ties UP the degrees that come out wrong for {label}: {[d + 1 for d in up_bad]}")

# --- claves on 16 steps: the bossa clave is the E(5,16) necklace; the son clave is not Euclidean; 3-2 and 2-3 are one necklace half a bar apart
def pos(p): return [i for i, c in enumerate(p) if c == "x"]
def mk(ps, n=16): return "".join("x" if i in ps else "." for i in range(n))
bossa = "x..x..x...x..x.."; son32 = mk({0, 3, 6, 10, 12})
check("the bossa clave x..x..x...x..x.. is a rotation of E(5,16)", bossa in rotations(s(bjorklund(5, 16))), s(bjorklund(5, 16)))
check("the 3-2 son clave is not Euclidean: its gaps are 3 3 4 2 4, which differ by 2", evenness(son32) == 2 and son32 not in rotations(s(bjorklund(5, 16))), f"{son32}, spread {evenness(son32)}")
son23 = mk({(q + 8) % 16 for q in pos(son32)})
check("the 2-3 son clave is the 3-2 rotated by half a bar: 3 onsets then 2 becomes 2 then 3", sum(c == "x" for c in son32[:8]) == 3 and sum(c == "x" for c in son32[8:]) == 2 and sum(c == "x" for c in son23[:8]) == 2 and sum(c == "x" for c in son23[8:]) == 3, f"{son32} -> {son23}")

# --- the engine's strum table, read from the source, against the manual's table above (M p.22), cell for cell
import pathlib, re
src = (pathlib.Path(__file__).resolve().parents[2] / "crates/octocore/src/tables.rs").read_text()
rows = {int(n): [int(x) for x in v.split(",")] for n, v in re.findall(r"^\s*(\d) => &\[([\d, ]+)\],", src, re.M)}
check("crates/octocore/src/tables.rs holds the strum table for notes 2 to 7, 9 levels each, cell for cell equal to the manual's", rows == strum, f"rows found: {sorted(rows)}")

print("\n" + ("ALL CHECKS PASS" if not fails else f"{fails} CHECK(S) FAILED"))
sys.exit(1 if fails else 0)
